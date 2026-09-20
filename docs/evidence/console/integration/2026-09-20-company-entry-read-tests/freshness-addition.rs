#[sqlx::test(migrations = false)]
async fn company_entry_optional_eligibility_expiry_never_retains_active_identity(pool: PgPool) {
    let (app, signer) = signed_fixture(&pool).await;
    owner_ready(&pool).await;
    let (account, cookies) = enrolled(&app).await;
    let startup = startup(&pool).await;
    designate(&startup, &designation(&pool, account.account).await)
        .await
        .unwrap();
    for route in ["/account", "/account/register", "/"] {
        // Complete census before starting the intentionally short token lifetime.
        let before = all_rows(&pool).await;
        let now: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&pool)
            .await
            .unwrap();
        let expires = now.unix_timestamp() + 6;
        let mut claims = signed_claims(&cookies.0[ACCESS], &signer).unwrap();
        claims["exp"] = json!(expires);
        let mut short = cookies.clone();
        short
            .0
            .insert(ACCESS.into(), sign_proof_claims(&claims, &signer));
        assert!(
            native_entry_html(&document(&app, route, &short).await, StatusCode::OK)
                .contains("data-account-state=\"active\"")
        );
        let mut blocker = pool.begin().await.unwrap();
        let holder: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *blocker)
            .await
            .unwrap();
        sqlx::query(
            "SELECT receipt_id FROM public.deployment_operator_head WHERE singleton=1 FOR UPDATE",
        )
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
        let observation = async {
            let reached = tokio::time::timeout(std::time::Duration::from_secs(3), async {
                loop {
                    let waiting: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity a WHERE a.datname=current_database() AND a.usename='console_auth_rt' AND a.wait_event_type='Lock' AND $1=ANY(pg_catalog.pg_blocking_pids(a.pid))) AND extract(epoch FROM clock_timestamp()) < $2::bigint")
                        .bind(holder).bind(expires).fetch_one(&pool).await.unwrap();
                    if waiting { break; }
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            }).await;
            let elapsed = if reached.is_ok() {
                tokio::time::timeout(std::time::Duration::from_secs(7), sqlx::query("SELECT pg_sleep(GREATEST(0.0,$1::double precision-extract(epoch FROM clock_timestamp()))+0.025)")
                    .bind(expires as f64).execute(&pool)).await.is_ok_and(|r| r.is_ok())
            } else {
                false
            };
            blocker.rollback().await.unwrap();
            (reached.is_ok(), elapsed)
        };
        let (response, witness) = tokio::time::timeout(std::time::Duration::from_secs(15), async {
            tokio::join!(document(&app, route, &short), observation)
        })
        .await
        .expect("bounded document and lock release");
        assert!(
            witness == (true, true),
            "actual designation-head wait before expiry required"
        );
        let html = native_entry_html(&response, StatusCode::OK);
        assert!(!html.contains("data-account-state=\"active\""));
        assert!(!html.contains("data-native-action=\"logout\""));
        assert!(!html.contains("href=\"/account/companies/new\""));
        let anonymous = document(&app, route, &Cookies::default()).await;
        native_entry_html(&anonymous, StatusCode::OK);
        assert!(
            response.bytes == anonymous.bytes,
            "expired optional eligibility retained Account identity"
        );
        assert!(
            before == all_rows(&pool).await,
            "expired projection changed persistent state"
        );
        assert!(
            native_entry_html(&document(&app, route, &cookies).await, StatusCode::OK)
                .contains("data-account-state=\"active\"")
        );
    }
    startup.close().await;
}
