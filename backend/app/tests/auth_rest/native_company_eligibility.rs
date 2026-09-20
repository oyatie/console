// Private candidate included inside company_setup; current real owners/fixtures.
mod eligibility {
    use super::*;
    use sqlx::{Connection, Executor, Postgres};

    const HELPER: &str = "public.account_company_setup_eligibility_v1(uuid)";

    async fn eligible<'e, E>(executor: E, account: Option<Uuid>) -> Result<bool, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_scalar("SELECT public.account_company_setup_eligibility_v1($1)")
            .bind(account)
            .fetch_one(executor)
            .await
    }

    async fn exists(pool: &PgPool) {
        let found: bool = sqlx::query_scalar("SELECT to_regprocedure($1) IS NOT NULL")
            .bind(HELPER)
            .fetch_one(pool)
            .await
            .unwrap();
        assert!(
            found,
            "PREREQUISITE: exact Company eligibility owner missing; only catalog probe admits its installation"
        );
    }

    #[sqlx::test(migrations = false)]
    async fn native_company_eligibility_catalog_has_only_exact_read_capability(pool: PgPool) {
        let _app = fixture(&pool).await;
        owner_exists(&pool).await;
        let observed: (i64, Option<bool>) = sqlx::query_as(r#"
            SELECT count(*)::bigint, bool_and(
                p.proargtypes='2950'::oidvector AND p.pronargs=1 AND p.pronargdefaults=0
                AND p.proargnames=ARRAY['p_account']::text[]
                AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict
                AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u'
                AND NOT p.proretset AND p.prorettype='bool'::regtype
                AND p.proowner='console_account_owner'::regrole
                AND p.prolang=(SELECT oid FROM pg_language WHERE lanname='plpgsql')
                AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp','row_security=on']::text[]
                AND (SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),
                    CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
                    a.privilege_type,a.is_grantable)
                    ORDER BY CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END)
                    FROM aclexplode(p.proacl) a)
                    = '[ ["console_account_owner","console_account_owner","EXECUTE",false],
                         ["console_account_owner","console_auth_rt","EXECUTE",false] ]'::jsonb
            ) FROM pg_proc p
            WHERE p.pronamespace='public'::regnamespace
                AND p.proname='account_company_setup_eligibility_v1'
        "#).fetch_one(&pool).await.unwrap();
        assert!(
            observed == (1, Some(true)),
            "NATIVE_COMPANY_ELIGIBILITY_OWNER: exact single read capability/signature/owner/ACL required"
        );
    }

    #[sqlx::test(migrations = false)]
    async fn native_company_eligibility_uses_current_exact_account_and_makes_no_rows(pool: PgPool) {
        let app = fixture(&pool).await;
        exists(&pool).await;
        let (a, _) = enrolled(&app).await;
        let (b, _) = enrolled(&app).await;
        let startup = startup(&pool).await;
        let input = designation(&pool, a.account).await;
        let before = all_rows(&pool).await;
        for login in [TestDatabaseLogin::Auth] {
            let runtime = login_test_pool(&pool, login).await;
            assert!(!eligible(&runtime, Some(a.account)).await.unwrap());
            assert!(!eligible(&runtime, Some(b.account)).await.unwrap());
            runtime.close().await;
        }
        assert!(
            before == all_rows(&pool).await,
            "empty designation reads changed state"
        );
        let original = designate(&startup, &input).await.unwrap();
        assert!(original.1 == 1 && !original.2);
        let before = all_rows(&pool).await;
        for login in [TestDatabaseLogin::Auth] {
            let runtime = login_test_pool(&pool, login).await;
            assert!(eligible(&runtime, Some(a.account)).await.unwrap());
            assert!(!eligible(&runtime, Some(b.account)).await.unwrap());
            direct_authority_denied(&pool, &runtime).await;
            runtime.close().await;
        }
        assert!(
            before == all_rows(&pool).await,
            "eligibility read minted identity/grant/audit/receipt or changed history"
        );
        assert_no_company_identity(&pool, a.account).await;
        assert_no_company_identity(&pool, b.account).await;
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn native_company_eligibility_refuses_invalid_accounts_and_isolation(pool: PgPool) {
        let app = fixture(&pool).await;
        exists(&pool).await;
        let (active, _) = enrolled(&app).await;
        let pending = start(&app).await;
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
        let before = all_rows(&pool).await;
        for account in [
            None,
            Some(Uuid::nil()),
            Some(Uuid::new_v4()),
            Some(pending.account),
        ] {
            refused(
                eligible(&runtime, account).await,
                "P0001",
                Some("account.authentication_invalid"),
            );
        }
        for isolation in [
            "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ",
            "SET TRANSACTION ISOLATION LEVEL SERIALIZABLE",
        ] {
            let mut tx = runtime.begin().await.unwrap();
            sqlx::query(isolation).execute(&mut *tx).await.unwrap();
            let result = eligible(&mut *tx, Some(active.account)).await;
            tx.rollback().await.unwrap();
            refused(result, "P0001", Some("account.authority_unavailable"));
        }
        assert!(
            before == all_rows(&pool).await,
            "invalid projection changed state"
        );
        // Controlled invalid-state injection only, not recovery workflow acceptance.
        assert_eq!(sqlx::query("UPDATE public.account_security SET security_state='RECOVERY_REQUIRED',security_generation=security_generation+1,revision=revision+1,updated_at=clock_timestamp() WHERE account_id=$1 AND security_state='ACTIVE'")
            .bind(active.account).execute(&pool).await.unwrap().rows_affected(), 1);
        let recovery_state = all_rows(&pool).await;
        refused(
            eligible(&runtime, Some(active.account)).await,
            "P0001",
            Some("account.authentication_invalid"),
        );
        assert!(
            recovery_state == all_rows(&pool).await,
            "recovery-required refusal changed state"
        );
        runtime.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn native_company_eligibility_does_not_expand_command_login_authority(pool: PgPool) {
        let (app, a, _, startup, _) = designated(&pool).await;
        exists(&pool).await;
        let before = all_rows(&pool).await;
        for login in [
            TestDatabaseLogin::Business,
            TestDatabaseLogin::LeaveCommand,
            TestDatabaseLogin::OntologyCommand,
            TestDatabaseLogin::PlatformForceCommand,
        ] {
            let runtime = login_test_pool(&pool, login).await;
            refused(eligible(&runtime, Some(a.account)).await, "42501", None);
            direct_authority_denied(&pool, &runtime).await;
            runtime.close().await;
        }
        refused(eligible(&startup, Some(a.account)).await, "42501", None);
        assert!(
            before == all_rows(&pool).await,
            "rejected ordinary owner calls changed state"
        );
        assert_no_company_identity(&pool, a.account).await;
        drop(app);
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn native_company_eligibility_revoke_and_historical_replay_do_not_restore_it(
        pool: PgPool,
    ) {
        let (_app, a, _, startup, input) = designated(&pool).await;
        exists(&pool).await;
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
        assert!(eligible(&runtime, Some(a.account)).await.unwrap());
        let original = designate(&startup, &input).await.unwrap();
        assert!(original.2);
        let revoked = revoke(
            &startup,
            &input,
            Uuid::new_v4(),
            1,
            "TEST_ONLY Company setup eligibility revoked",
        )
        .await
        .unwrap();
        assert!(revoked.1 == 2 && !revoked.2);
        let before = all_rows(&pool).await;
        assert!(!eligible(&runtime, Some(a.account)).await.unwrap());
        assert_eq!(designate(&startup, &input).await.unwrap(), original);
        assert!(!eligible(&runtime, Some(a.account)).await.unwrap());
        assert!(
            before == all_rows(&pool).await,
            "old command replay or eligibility revived designation"
        );
        runtime.close().await;
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn native_company_eligibility_waits_for_account_invalidation_and_rechecks(pool: PgPool) {
        let (_app, a, _, startup, _) = designated(&pool).await;
        exists(&pool).await;
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
        let mut holder = pool.acquire().await.unwrap();
        let mut caller = runtime.acquire().await.unwrap();
        let holder_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *holder)
            .await
            .unwrap();
        let caller_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *caller)
            .await
            .unwrap();
        let mut change = holder.begin().await.unwrap();
        // Explicit negative invalidation injection after real WebAuthn and designation.
        // This does not certify a suspension/recovery product workflow.
        assert_eq!(sqlx::query("UPDATE public.account_security SET security_state='SECURITY_SUSPENDED',security_generation=security_generation+1,revision=revision+1,updated_at=clock_timestamp() WHERE account_id=$1 AND security_state='ACTIVE'")
            .bind(a.account).execute(&mut *change).await.unwrap().rows_affected(), 1);
        let mut call = tokio::spawn(async move { eligible(&mut *caller, Some(a.account)).await });
        let witnessed = blocking_witness(&pool, caller_pid, holder_pid).await;
        change.commit().await.unwrap();
        let completed = tokio::time::timeout(std::time::Duration::from_secs(5), &mut call).await;
        if completed.is_err() {
            call.abort();
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), &mut call).await;
        }
        let result = completed
            .expect("owned eligibility task completed")
            .unwrap();
        assert!(witnessed, "actual Account guard contention required");
        refused(result, "P0001", Some("account.authentication_invalid"));
        let before = all_rows(&pool).await;
        refused(
            eligible(&runtime, Some(a.account)).await,
            "P0001",
            Some("account.authentication_invalid"),
        );
        assert!(before == all_rows(&pool).await);
        runtime.close().await;
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn native_company_eligibility_retains_account_guard_until_commit(pool: PgPool) {
        let (_app, a, _, startup, input) = designated(&pool).await;
        exists(&pool).await;
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
        let mut caller = runtime.acquire().await.unwrap();
        let mut revoker = startup.acquire().await.unwrap();
        let caller_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *caller)
            .await
            .unwrap();
        let revoker_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *revoker)
            .await
            .unwrap();
        let mut read = caller.begin().await.unwrap();
        assert!(eligible(&mut *read, Some(a.account)).await.unwrap());
        let command = Uuid::new_v4();
        let mut revoke_call = tokio::spawn(async move {
            sqlx::query_as::<_, (Uuid,i64,bool)>("SELECT receipt_id,revision,replayed FROM public.deployment_operator_revoke_v1($1,$2,$3,$4,$5,$6,$7)")
                .bind(&input.target.system).bind(&input.target.database).bind(input.target.oid)
                .bind(command).bind(input.account).bind(1_i64).bind("TEST_ONLY serialized eligibility revoke")
                .fetch_one(&mut *revoker).await
        });
        let witnessed = blocking_witness(&pool, revoker_pid, caller_pid).await;
        read.commit().await.unwrap();
        drop(caller);
        let completed =
            tokio::time::timeout(std::time::Duration::from_secs(5), &mut revoke_call).await;
        if completed.is_err() {
            revoke_call.abort();
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), &mut revoke_call).await;
        }
        let result = completed
            .expect("owned revoke task completed")
            .unwrap()
            .unwrap();
        assert!(witnessed && result.1 == 2 && !result.2);
        let before = all_rows(&pool).await;
        assert!(!eligible(&runtime, Some(a.account)).await.unwrap());
        assert!(before == all_rows(&pool).await);
        runtime.close().await;
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn native_company_eligibility_retains_matching_head_lock_until_commit(pool: PgPool) {
        let (_app, a, _, startup, _) = designated(&pool).await;
        exists(&pool).await;
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
        let mut caller = runtime.acquire().await.unwrap();
        let mut observer = pool.acquire().await.unwrap();
        let caller_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *caller)
            .await
            .unwrap();
        let observer_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *observer)
            .await
            .unwrap();
        let before = all_rows(&pool).await;
        let mut read = caller.begin().await.unwrap();
        assert!(eligible(&mut *read, Some(a.account)).await.unwrap());
        // Private lock observer only; it changes no head or authority facts.
        let mut observed = tokio::spawn(async move {
            sqlx::query_scalar::<_, bool>("SELECT account_id=$1 FROM public.deployment_operator_head WHERE singleton=1 FOR UPDATE")
                .bind(a.account).fetch_one(&mut *observer).await
        });
        let witnessed = blocking_witness(&pool, observer_pid, caller_pid).await;
        read.commit().await.unwrap();
        drop(caller);
        let completed =
            tokio::time::timeout(std::time::Duration::from_secs(5), &mut observed).await;
        if completed.is_err() {
            observed.abort();
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), &mut observed).await;
        }
        let matched = completed
            .expect("owned lock observer completed")
            .unwrap()
            .unwrap();
        assert!(
            witnessed && matched,
            "matching designation head exclusion must last through transaction end"
        );
        assert!(before == all_rows(&pool).await);
        runtime.close().await;
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn native_company_eligibility_generation_is_current_session_not_old_admission(
        pool: PgPool,
    ) {
        let (app, signer) = signed_fixture(&pool).await;
        exists(&pool).await;
        let (mut account, old_cookies) = enrolled(&app).await;
        let startup = startup(&pool).await;
        let input = designation(&pool, account.account).await;
        designate(&startup, &input).await.unwrap();
        let history = operator_rows(&pool).await;
        // Controlled current-security fact transition; not recovery workflow acceptance.
        assert_eq!(sqlx::query("UPDATE public.account_security SET security_generation=security_generation+1,revision=revision+1,updated_at=clock_timestamp() WHERE account_id=$1 AND security_state='ACTIVE'")
            .bind(account.account).execute(&pool).await.unwrap().rows_affected(), 1);
        let before = all_rows(&pool).await;
        let old = document(&app, ENTRY, &old_cookies).await;
        native_entry_html(&old, StatusCode::UNAUTHORIZED);
        assert!(
            !std::str::from_utf8(&old.bytes)
                .unwrap()
                .contains("data-company-setup")
        );
        assert!(
            before == all_rows(&pool).await,
            "stale session refusal changed data"
        );
        let current_cookies = fresh_login(&app, &mut account).await;
        assert_eq!(
            claim_generation(&signed_claims(&current_cookies.0[ACCESS], &signer).unwrap()),
            input.generation + 1
        );
        let before = all_rows(&pool).await;
        let current = document(&app, ENTRY, &current_cookies).await;
        assert!(native_entry_html(&current, StatusCode::OK).contains("data-company-setup"));
        assert!(
            before == all_rows(&pool).await,
            "current form projection changed data"
        );
        assert!(
            history == operator_rows(&pool).await,
            "authentication generation retargeted original designation receipt"
        );
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn native_company_eligibility_document_expires_during_witnessed_head_wait(pool: PgPool) {
        let (app, signer) = signed_fixture(&pool).await;
        exists(&pool).await;
        let (account, cookies) = enrolled(&app).await;
        let startup = startup(&pool).await;
        designate(&startup, &designation(&pool, account.account).await)
            .await
            .unwrap();
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
            native_entry_html(&document(&app, ENTRY, &short).await, StatusCode::OK)
                .contains("data-company-setup")
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
            tokio::join!(document(&app, ENTRY, &short), observation)
        })
        .await
        .expect("bounded document and lock release");
        assert!(
            witness == (true, true),
            "actual designation-head wait before expiry required"
        );
        let html = native_entry_html(&response, StatusCode::UNAUTHORIZED);
        assert!(
            !html.contains("data-company-setup"),
            "expired request leaked setup form after eligibility wait"
        );
        assert!(
            before == all_rows(&pool).await,
            "expired projection changed persistent state"
        );
        assert!(
            native_entry_html(&document(&app, ENTRY, &cookies).await, StatusCode::OK)
                .contains("data-company-setup")
        );
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn native_company_eligibility_retains_specific_account_lock_until_commit(pool: PgPool) {
        let (_app, a, _, startup, _) = designated(&pool).await;
        exists(&pool).await;
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
        let mut caller = runtime.acquire().await.unwrap();
        let mut observer = pool.acquire().await.unwrap();
        let caller_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *caller)
            .await
            .unwrap();
        let observer_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *observer)
            .await
            .unwrap();
        let before = all_rows(&pool).await;
        let mut read = caller.begin().await.unwrap();
        assert!(eligible(&mut *read, Some(a.account)).await.unwrap());
        // Account-only observer cannot be blocked by the designation-head guard.
        let mut observed = tokio::spawn(async move {
            sqlx::query_scalar::<_, bool>(
                "SELECT account_id=$1 FROM public.account_security WHERE account_id=$1 FOR UPDATE",
            )
            .bind(a.account)
            .fetch_one(&mut *observer)
            .await
        });
        let witnessed = blocking_witness(&pool, observer_pid, caller_pid).await;
        read.commit().await.unwrap();
        drop(caller);
        let completed =
            tokio::time::timeout(std::time::Duration::from_secs(5), &mut observed).await;
        if completed.is_err() {
            observed.abort();
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), &mut observed).await;
        }
        let matched = completed
            .expect("owned Account-only observer completed")
            .unwrap()
            .unwrap();
        assert!(
            witnessed && matched,
            "specific Account-row exclusion must last through transaction end"
        );
        assert!(before == all_rows(&pool).await);
        runtime.close().await;
        startup.close().await;
    }
}
