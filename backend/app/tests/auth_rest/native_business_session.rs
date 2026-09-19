// Included inside account_browser. Real native enrollment and admitted LOGINs.
// This tests the existing read guard API, not Company authority or enrollment.
mod native_business_session {
    use super::*;
    use console_platform_auth::account::{
        AccountLiveSession, AccountOperationError, live_account_session_in_tx,
    };
    use console_platform_auth::{AccountAssurance, JwtSettings, JwtVerifier};
    use console_platform_test_support::login_test_pool;
    use sqlx::{Connection, PgConnection, Postgres, Transaction};
    use std::time::Duration as Wait;

    const EXTRA_TABLES: &[&str] = &[
        "users",
        "organizations",
        "groups",
        "group_memberships",
        "group_role_grants",
        "employees",
        "persons",
        "policy_roles",
        "policy_role_permissions",
        "policy_role_conditions",
        "user_role_assignments",
        "policy_assignment_preview_receipts",
        "policy_versions",
        "deployment_operator_receipts",
        "deployment_operator_head",
        "audit_events",
    ];

    fn verification(app: &Fixture, key: &SigningKey) -> (JwtVerifier, Duration) {
        let config = account_browser_config(&app.pool, app._artifacts.root.clone(), key);
        let jwt = config.jwt.unwrap();
        let ttl = config.auth_rest.unwrap().refresh_family_absolute_ttl;
        let verifier = JwtVerifier::from_es256_public_pem(
            JwtSettings {
                issuer: jwt.issuer,
                audience: jwt.audience,
                access_token_ttl: Duration::minutes(15),
            },
            jwt.public_key_pem.as_bytes(),
        )
        .unwrap();
        (verifier, ttl)
    }

    fn inventory_complete(rows: &BTreeMap<String, String>) -> bool {
        let expected: BTreeSet<_> = NATIVE_EXTENSION_TABLES
            .into_iter()
            .chain(["_sqlx_migrations"])
            .chain(EXTRA_TABLES.iter().copied())
            .collect();
        rows.keys().map(String::as_str).collect::<BTreeSet<_>>() == expected
            && rows
                .values()
                .all(|raw| serde_json::from_str::<Value>(raw).is_ok_and(|v| v.is_array()))
            && rows.get("_sqlx_migrations").is_some_and(|raw| {
                serde_json::from_str::<Value>(raw)
                    .is_ok_and(|v| v.as_array().is_some_and(|v| !v.is_empty()))
            })
    }

    // Raw database JSON text preserves numeric fidelity. For logout, exclude
    // only the subject's three independently checked mutable streams.
    async fn inventory(pool: &PgPool, logout_subject: Option<Uuid>) -> BTreeMap<String, String> {
        let mut captured = native_extension_rows(pool).await;
        for table in EXTRA_TABLES {
            let query = format!(
                "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text COLLATE \"C\"),'[]'::jsonb)::text FROM public.{table} t"
            );
            let raw: String = sqlx::query_scalar(sqlx::AssertSqlSafe(query))
                .fetch_one(pool)
                .await
                .unwrap();
            assert!(captured.insert((*table).to_owned(), raw).is_none());
        }
        if let Some(account) = logout_subject {
            for (table, owner) in [
                ("account_security_events", "account_id"),
                ("auth_refresh_token_families", "user_id"),
                ("auth_refresh_tokens", "user_id"),
            ] {
                let query = format!(
                    "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text COLLATE \"C\"),'[]'::jsonb)::text FROM public.{table} t WHERE {owner} IS DISTINCT FROM $1"
                );
                let raw: String = sqlx::query_scalar(sqlx::AssertSqlSafe(query))
                    .bind(account)
                    .fetch_one(pool)
                    .await
                    .unwrap();
                assert!(captured.insert(table.to_owned(), raw).is_some());
            }
        }
        assert!(
            inventory_complete(&captured),
            "complete native/business inventory required"
        );
        captured
    }

    #[test]
    fn business_inventory_oracle_rejects_paired_omission_and_non_arrays() {
        let original: BTreeMap<_, _> = NATIVE_EXTENSION_TABLES
            .into_iter()
            .chain(["_sqlx_migrations"])
            .chain(EXTRA_TABLES.iter().copied())
            .map(|name| {
                (
                    name.to_owned(),
                    if name == "_sqlx_migrations" {
                        "[{\"version\":1}]"
                    } else {
                        "[]"
                    }
                    .to_owned(),
                )
            })
            .collect();
        assert_eq!(original.len(), 32);
        assert!(inventory_complete(&original));
        for name in original.keys() {
            let mut missing = original.clone();
            missing.remove(name);
            assert!(
                !inventory_complete(&missing),
                "equal incomplete observations cannot pass"
            );
            let mut malformed = original.clone();
            malformed.insert(name.clone(), "null".into());
            assert!(!inventory_complete(&malformed));
        }
        let mut empty_ledger = original;
        empty_ledger.insert("_sqlx_migrations".into(), "[]".into());
        assert!(!inventory_complete(&empty_ledger));
    }

    async fn business(pool: &PgPool) -> PgPool {
        let runtime = login_test_pool(pool, TestDatabaseLogin::Business).await;
        let observed: (String, String, bool, bool) = sqlx::query_as("SELECT session_user::text,current_user::text,(SELECT rolcanlogin AND NOT rolsuper AND NOT rolbypassrls AND NOT rolcreaterole AND NOT rolcreatedb AND NOT rolreplication FROM pg_catalog.pg_roles WHERE rolname=current_user),current_setting('transaction_isolation')='read committed'")
            .fetch_one(&runtime).await.unwrap();
        assert!(observed == ("console_rt".into(), "console_rt".into(), true, true));
        for role in [
            "console_account_owner",
            "console_credential_owner",
            "console_terms_owner",
            "console_auth_rt",
            "console_app",
        ] {
            let inherited: bool = sqlx::query_scalar("SELECT pg_has_role(current_user,$1,'MEMBER') OR pg_has_role(current_user,$1,'SET') OR pg_has_role(current_user,$1,'USAGE')")
                .bind(role).fetch_one(&runtime).await.unwrap();
            assert!(
                !inherited,
                "Business must not inherit protected owner authority"
            );
        }
        runtime
    }

    async fn current(
        tx: &mut Transaction<'_, Postgres>,
        verifier: &JwtVerifier,
        access: &str,
        ttl: Duration,
    ) -> AccountLiveSession {
        match live_account_session_in_tx(tx, verifier, access, ttl).await {
            Ok(session) => session,
            Err(error) => panic!(
                "NATIVE_BUSINESS_SESSION_CURRENT: existing API must resolve actual bound session; sanitized refusal={error}"
            ),
        }
    }

    async fn refuses(runtime: &PgPool, verifier: &JwtVerifier, access: &str, ttl: Duration) {
        let mut tx = runtime.begin().await.unwrap();
        let result = live_account_session_in_tx(&mut tx, verifier, access, ttl).await;
        tx.rollback().await.unwrap();
        assert!(
            matches!(result, Err(AccountOperationError::AuthenticationInvalid)),
            "must refuse invalid authentication, not hide unavailable authority"
        );
    }

    #[sqlx::test(migrations = false)]
    async fn business_transaction_resolves_real_native_session_without_credential_access(
        pool: PgPool,
    ) {
        let (app, key) = signed_fixture(&pool).await;
        let (attempt, cookies) = enrolled(&app).await;
        assert_no_company_identity(&pool, attempt.account).await;
        let (verifier, ttl) = verification(&app, &key);
        let before = inventory(&pool, None).await;
        let auth = logout_auth_pool(&pool).await;
        let mut control = auth.begin().await.unwrap();
        eprintln!("native_business_session_probe consumer=Auth boundary=existing_live_api");
        let expected = current(&mut control, &verifier, &cookies.0[ACCESS], ttl).await;
        control.commit().await.unwrap();
        let runtime = business(&pool).await;
        let mut tx = runtime.begin().await.unwrap();
        eprintln!("native_business_session_probe consumer=Business boundary=existing_live_api");
        let actual = current(&mut tx, &verifier, &cookies.0[ACCESS], ttl).await;
        assert!(
            actual.account_id == attempt.account
                && actual.account_id == expected.account_id
                && actual.session_id == expected.session_id
                && actual.security_generation == expected.security_generation
                && actual.auth_time == expected.auth_time
                && actual.assurance == AccountAssurance::PasskeyPrimary
                && actual.assurance == expected.assurance
                && actual.expires_at == expected.expires_at
                && actual.family_expires_at == expected.family_expires_at
        );
        let persisted: (Uuid, String, Option<i64>, Option<OffsetDateTime>, Option<String>, OffsetDateTime, Option<OffsetDateTime>, Option<Uuid>) = sqlx::query_as("SELECT user_id,protocol,account_security_generation,auth_time,assurance,created_at,revoked_at,org_id FROM public.auth_refresh_token_families WHERE id=$1")
            .bind(actual.session_id).fetch_one(&pool).await.unwrap();
        assert!(
            persisted.0 == actual.account_id
                && persisted.1 == "ACCOUNT_V1"
                && persisted.2 == Some(actual.security_generation)
                && persisted.3 == Some(actual.auth_time)
                && persisted.4.as_deref() == Some("PASSKEY_PRIMARY")
                && persisted.5 + ttl == actual.family_expires_at
                && persisted.6.is_none()
                && persisted.7.is_none()
        );
        let claims = signed_claims(&cookies.0[ACCESS], &key).unwrap();
        assert!(claims["exp"].as_i64() == Some(actual.expires_at.unix_timestamp()));
        tx.commit().await.unwrap();
        assert!(
            before == inventory(&pool, None).await,
            "read guard created or changed native/business state"
        );
        assert_no_company_identity(&pool, attempt.account).await;
        runtime.close().await;
        auth.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn business_native_session_rejects_invalid_signed_bindings(pool: PgPool) {
        let (app, key) = signed_fixture(&pool).await;
        let (_, cookies) = enrolled(&app).await;
        let (other, other_cookies) = enrolled(&app).await;
        let (verifier, ttl) = verification(&app, &key);
        let runtime = business(&pool).await;
        let claims = signed_claims(&cookies.0[ACCESS], &key).unwrap();
        let other_claims = signed_claims(&other_cookies.0[ACCESS], &key).unwrap();
        let before = inventory(&pool, None).await;
        let mut tx = runtime.begin().await.unwrap();
        current(&mut tx, &verifier, &cookies.0[ACCESS], ttl).await;
        tx.rollback().await.unwrap();
        let now: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&pool)
            .await
            .unwrap();
        let cases = [
            ("sub", json!(other.account)),
            ("sid", other_claims["sid"].clone()),
            (
                "security_generation",
                json!((claim_generation(&claims) + 1).to_string()),
            ),
            (
                "auth_time",
                json!(claims["auth_time"].as_i64().unwrap() - 1),
            ),
            ("assurance", json!("UNRECOGNIZED")),
            ("token_kind", json!("account_csrf_v1")),
            ("iss", json!("untrusted-issuer")),
            ("aud", json!("untrusted-audience")),
            ("exp", json!(now.unix_timestamp() - 1)),
            ("iat", json!(now.unix_timestamp() + 120)),
            ("nbf", json!(now.unix_timestamp() + 120)),
            ("unexpected_claim", json!(true)),
        ];
        let mut exercised = 0;
        for (field, value) in cases {
            let mut changed = claims.clone();
            changed[field] = value;
            let token = sign_proof_claims(&changed, &key);
            assert!(signed_claims(&token, &key).unwrap() == changed);
            refuses(&runtime, &verifier, &token, ttl).await;
            assert!(
                before == inventory(&pool, None).await,
                "invalid signed binding changed state"
            );
            exercised += 1;
        }
        let wrong_key = SigningKey::random(&mut OsRng);
        refuses(
            &runtime,
            &verifier,
            &sign_proof_claims(&claims, &wrong_key),
            ttl,
        )
        .await;
        assert_eq!(exercised, 12);
        let mut tx = runtime.begin().await.unwrap();
        current(&mut tx, &verifier, &cookies.0[ACCESS], ttl).await;
        tx.commit().await.unwrap();
        assert!(before == inventory(&pool, None).await);
        runtime.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn business_session_checks_current_account_state_and_generation(pool: PgPool) {
        let (app, key) = signed_fixture(&pool).await;
        let (mut attempt, cookies) = enrolled(&app).await;
        let (verifier, ttl) = verification(&app, &key);
        let runtime = business(&pool).await;
        let auth = logout_auth_pool(&pool).await;
        let original = inventory(&pool, None).await;
        for consumer in [&auth, &runtime] {
            let mut tx = consumer.begin().await.unwrap();
            current(&mut tx, &verifier, &cookies.0[ACCESS], ttl).await;
            tx.commit().await.unwrap();
        }
        // Disposable administrator-owned invalidation fixtures only. These do
        // not implement or prove authority for a security-state command.
        for state in [
            "SECURITY_SUSPENDED",
            "RECOVERY_REQUIRED",
            "PENDING_ENROLLMENT",
        ] {
            let changed = sqlx::query("UPDATE public.account_security SET security_state=$2 WHERE account_id=$1 AND security_state='ACTIVE'")
                .bind(attempt.account).bind(state).execute(&pool).await.unwrap().rows_affected();
            assert_eq!(changed, 1);
            let before = inventory(&pool, None).await;
            refuses(&auth, &verifier, &cookies.0[ACCESS], ttl).await;
            refuses(&runtime, &verifier, &cookies.0[ACCESS], ttl).await;
            assert!(
                before == inventory(&pool, None).await,
                "inactive Account reads changed fixture state"
            );
            let restored = sqlx::query("UPDATE public.account_security SET security_state='ACTIVE' WHERE account_id=$1 AND security_state=$2")
                .bind(attempt.account).bind(state).execute(&pool).await.unwrap().rows_affected();
            assert_eq!(restored, 1);
            for consumer in [&auth, &runtime] {
                let mut tx = consumer.begin().await.unwrap();
                current(&mut tx, &verifier, &cookies.0[ACCESS], ttl).await;
                tx.commit().await.unwrap();
            }
            assert!(
                original == inventory(&pool, None).await,
                "restored Account lost original history"
            );
        }
        // The family and original signed token still agree with one another.
        // Only current Account generation advances, so a family-only validator
        // must fail this independent oracle.
        let changed = sqlx::query("UPDATE public.account_security SET security_generation=security_generation+1,revision=revision+1,updated_at=clock_timestamp() WHERE account_id=$1 AND security_state='ACTIVE'")
            .bind(attempt.account).execute(&pool).await.unwrap().rows_affected();
        assert_eq!(changed, 1);
        let advanced = inventory(&pool, None).await;
        for stream in [
            "auth_refresh_token_families",
            "auth_refresh_tokens",
            "auth_webauthn_credentials",
        ] {
            assert!(
                advanced[stream] == original[stream],
                "invalidation fixture rewrote credentials"
            );
        }
        refuses(&auth, &verifier, &cookies.0[ACCESS], ttl).await;
        refuses(&runtime, &verifier, &cookies.0[ACCESS], ttl).await;
        assert!(advanced == inventory(&pool, None).await);
        // Restore usable authority through actual primary login, never by
        // lowering the generation or constructing a verified session object.
        let fresh = fresh_login(&app, &mut attempt).await;
        let new_claims = signed_claims(&fresh.0[ACCESS], &key).unwrap();
        let old_claims = signed_claims(&cookies.0[ACCESS], &key).unwrap();
        assert!(claim_generation(&new_claims) == claim_generation(&old_claims) + 1);
        let after_login = inventory(&pool, None).await;
        for consumer in [&auth, &runtime] {
            refuses(consumer, &verifier, &cookies.0[ACCESS], ttl).await;
            let mut tx = consumer.begin().await.unwrap();
            let valid = current(&mut tx, &verifier, &fresh.0[ACCESS], ttl).await;
            assert!(valid.security_generation == claim_generation(&new_claims));
            tx.commit().await.unwrap();
        }
        assert!(after_login == inventory(&pool, None).await);
        runtime.close().await;
        auth.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn business_native_session_refuses_after_ordinary_logout(pool: PgPool) {
        let (app, key) = signed_fixture(&pool).await;
        let (attempt, cookies) = enrolled(&app).await;
        let (verifier, ttl) = verification(&app, &key);
        let runtime = business(&pool).await;
        let mut tx = runtime.begin().await.unwrap();
        current(&mut tx, &verifier, &cookies.0[ACCESS], ttl).await;
        tx.commit().await.unwrap();
        let csrf = proof(&app, &cookies).await;
        let before = snapshot(&pool, attempt.account).await;
        let untouched = inventory(&pool, Some(attempt.account)).await;
        let response = request(
            &app,
            "POST",
            "/api/v2/auth/logout",
            &cookies,
            Some(json!({})),
            &[("X-Console-CSRF", &csrf)],
        )
        .await;
        response.private();
        assert!(response.json(StatusCode::OK) == json!({"outcome":"COMMITTED"}));
        assert_logout_transition(
            &before,
            &snapshot(&pool, attempt.account).await,
            attempt.account,
        );
        assert!(untouched == inventory(&pool, Some(attempt.account)).await);
        let after = inventory(&pool, None).await;
        let auth = logout_auth_pool(&pool).await;
        refuses(&auth, &verifier, &cookies.0[ACCESS], ttl).await;
        refuses(&runtime, &verifier, &cookies.0[ACCESS], ttl).await;
        assert!(
            after == inventory(&pool, None).await,
            "stale read mutated completed logout"
        );
        auth.close().await;
        runtime.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn business_guard_blocks_ordinary_logout_until_transaction_ends(pool: PgPool) {
        let (app, key) = signed_fixture(&pool).await;
        let (attempt, cookies) = enrolled(&app).await;
        let (verifier, ttl) = verification(&app, &key);
        let csrf = proof(&app, &cookies).await;
        let before = snapshot(&pool, attempt.account).await;
        let untouched = inventory(&pool, Some(attempt.account)).await;
        let runtime = business(&pool).await;
        let mut tx = runtime.begin().await.unwrap();
        current(&mut tx, &verifier, &cookies.0[ACCESS], ttl).await;
        let holder: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        let router = app.service.clone();
        let presented = cookies.clone();
        let mut work = tokio::spawn(async move {
            request(
                &router,
                "POST",
                "/api/v2/auth/logout",
                &presented,
                Some(json!({})),
                &[("X-Console-CSRF", &csrf)],
            )
            .await
        });
        let reached = tokio::time::timeout(Wait::from_secs(4), async {
            loop {
                let blocked: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity a WHERE a.datname=current_database() AND a.usename='console_auth_rt' AND a.query LIKE '%account_security_lock_exclusive_v1%' AND $1=ANY(pg_catalog.pg_blocking_pids(a.pid)))")
                    .bind(holder).fetch_one(&pool).await.unwrap();
                if blocked { break true; }
                if work.is_finished() { break false; }
                tokio::time::sleep(Wait::from_millis(10)).await;
            }
        }).await;
        let unchanged_while_waiting = snapshot(&pool, attempt.account).await == before;
        tx.commit().await.unwrap();
        let completed = tokio::time::timeout(Wait::from_secs(10), &mut work).await;
        if completed.is_err() {
            work.abort();
            let _ = tokio::time::timeout(Wait::from_secs(5), &mut work).await;
        }
        assert!(
            matches!(reached, Ok(true)),
            "actual logout Account waiter must be blocked by this Business backend"
        );
        assert!(
            unchanged_while_waiting,
            "revocation escaped retained Business guard"
        );
        let response = completed
            .expect("logout must finish after Business commit")
            .unwrap();
        assert!(response.json(StatusCode::OK) == json!({"outcome":"COMMITTED"}));
        response.private();
        assert_logout_transition(
            &before,
            &snapshot(&pool, attempt.account).await,
            attempt.account,
        );
        assert!(untouched == inventory(&pool, Some(attempt.account)).await);
        refuses(&runtime, &verifier, &cookies.0[ACCESS], ttl).await;
        runtime.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn business_guard_retains_actual_family_row_lock_until_rollback(pool: PgPool) {
        let (app, key) = signed_fixture(&pool).await;
        let (_, cookies) = enrolled(&app).await;
        let (verifier, ttl) = verification(&app, &key);
        let before = inventory(&pool, None).await;
        let runtime = business(&pool).await;
        let mut tx = runtime.begin().await.unwrap();
        let current = current(&mut tx, &verifier, &cookies.0[ACCESS], ttl).await;
        let holder: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        let mut observer = PgConnection::connect_with(&pool.connect_options())
            .await
            .unwrap();
        let waiter: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut observer)
            .await
            .unwrap();
        let mut work = tokio::spawn(async move {
            let mut probe = observer.begin().await.unwrap();
            sqlx::raw_sql("SET LOCAL statement_timeout='12s'")
                .execute(&mut *probe)
                .await
                .unwrap();
            // Privileged read-lock fixture only. NO KEY UPDATE models the lock
            // taken by revocation of non-key fields; KEY SHARE would not suffice.
            let result: Result<Uuid, _> = sqlx::query_scalar(
                "SELECT id FROM public.auth_refresh_token_families WHERE id=$1 FOR NO KEY UPDATE",
            )
            .bind(current.session_id)
            .fetch_one(&mut *probe)
            .await;
            probe.rollback().await.unwrap();
            result
        });
        let reached = tokio::time::timeout(Wait::from_secs(4), async {
            loop {
                let blocked: bool = sqlx::query_scalar("SELECT $2=ANY(pg_blocking_pids($1)) AND EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity WHERE pid=$1 AND query LIKE '%auth_refresh_token_families%FOR NO KEY UPDATE%')")
                    .bind(waiter).bind(holder).fetch_one(&pool).await.unwrap();
                if blocked { break true; }
                if work.is_finished() { break false; }
                tokio::time::sleep(Wait::from_millis(10)).await;
            }
        }).await;
        tx.rollback().await.unwrap();
        let completed = tokio::time::timeout(Wait::from_secs(5), &mut work).await;
        if completed.is_err() {
            work.abort();
            let _ = tokio::time::timeout(Wait::from_secs(5), &mut work).await;
        }
        assert!(
            matches!(reached, Ok(true)),
            "actual family row waiter must be blocked by this Business transaction"
        );
        assert!(
            completed
                .expect("family probe completes after guard rollback")
                .unwrap()
                .is_ok()
        );
        assert!(before == inventory(&pool, None).await);
        runtime.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn business_guard_refuses_expired_token_after_witnessed_lock_wait(pool: PgPool) {
        let (app, key) = signed_fixture(&pool).await;
        let (attempt, cookies) = enrolled(&app).await;
        let (verifier, ttl) = verification(&app, &key);
        let runtime = business(&pool).await;
        let before = inventory(&pool, None).await;
        let now: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&pool)
            .await
            .unwrap();
        let expires = now.unix_timestamp() + 6;
        let mut claims = signed_claims(&cookies.0[ACCESS], &key).unwrap();
        claims["exp"] = json!(expires);
        let access = sign_proof_claims(&claims, &key);
        let mut positive = runtime.begin().await.unwrap();
        current(&mut positive, &verifier, &access, ttl).await;
        positive.rollback().await.unwrap();
        let mut blocker = pool.begin().await.unwrap();
        let holder: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *blocker)
            .await
            .unwrap();
        sqlx::query(
            "SELECT account_id FROM public.account_security WHERE account_id=$1 FOR UPDATE",
        )
        .bind(attempt.account)
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
        let mut waiting_tx = runtime.begin().await.unwrap();
        let waiter: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *waiting_tx)
            .await
            .unwrap();
        let mut work = tokio::spawn(async move {
            let mut tx = waiting_tx;
            sqlx::raw_sql("SET LOCAL statement_timeout='15s'")
                .execute(&mut *tx)
                .await
                .unwrap();
            let result = live_account_session_in_tx(&mut tx, &verifier, &access, ttl).await;
            tx.rollback().await.unwrap();
            result
        });
        let reached = tokio::time::timeout(Wait::from_secs(3), async {
            loop {
                let blocked: bool = sqlx::query_scalar("SELECT $2=ANY(pg_catalog.pg_blocking_pids($1)) AND EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity a WHERE a.pid=$1 AND a.datname=current_database() AND a.usename='console_rt' AND a.wait_event_type='Lock') AND extract(epoch FROM clock_timestamp()) < $3::bigint")
                    .bind(waiter).bind(holder).bind(expires).fetch_one(&pool).await.unwrap();
                if blocked { break true; }
                if work.is_finished() { break false; }
                tokio::time::sleep(Wait::from_millis(10)).await;
            }
        }).await;
        let elapsed = if matches!(reached, Ok(true)) {
            tokio::time::timeout(Wait::from_secs(7), sqlx::query("SELECT pg_sleep(GREATEST(0.0,$1::double precision-extract(epoch FROM clock_timestamp()))+0.025)")
                .bind(expires as f64).execute(&pool)).await.is_ok_and(|result| result.is_ok())
        } else {
            false
        };
        blocker.rollback().await.unwrap();
        let completed = tokio::time::timeout(Wait::from_secs(5), &mut work).await;
        if completed.is_err() {
            work.abort();
            let _ = tokio::time::timeout(Wait::from_secs(5), &mut work).await;
        }
        assert!(
            matches!(reached, Ok(true)) && elapsed,
            "actual Business guard wait and database-clock expiry are prerequisites"
        );
        let result = completed
            .expect("waited session must complete after guard release")
            .unwrap();
        assert!(
            matches!(result, Err(AccountOperationError::AuthenticationInvalid)),
            "expiry after guard wait must refuse, not return stale or unavailable authority"
        );
        let (verifier, ttl) = verification(&app, &key);
        let mut restored = runtime.begin().await.unwrap();
        current(&mut restored, &verifier, &cookies.0[ACCESS], ttl).await;
        restored.commit().await.unwrap();
        assert!(
            before == inventory(&pool, None).await,
            "expiry refusal changed persistent state"
        );
        runtime.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn business_session_keeps_direct_credential_and_low_level_identity_paths_denied(
        pool: PgPool,
    ) {
        let (app, key) = signed_fixture(&pool).await;
        let (attempt, cookies) = enrolled(&app).await;
        let (verifier, ttl) = verification(&app, &key);
        let runtime = business(&pool).await;
        let before = inventory(&pool, None).await;
        for table in [
            "accounts",
            "account_security",
            "auth_webauthn_credentials",
            "auth_webauthn_ceremonies",
            "auth_refresh_tokens",
            "auth_refresh_token_families",
            "auth_bootstrap_credentials",
            "auth_webauthn_ceremony_bindings",
            "auth_device_login_handoffs",
        ] {
            let qualified = format!("public.{table}");
            let readable: bool = sqlx::query_scalar("SELECT has_table_privilege(current_user,to_regclass($1),'SELECT') OR has_any_column_privilege(current_user,to_regclass($1),'SELECT')")
                .bind(&qualified).fetch_one(&runtime).await.unwrap();
            assert!(
                !readable,
                "narrow session owner must not grant direct or inherited table/column reads"
            );
            let query = format!("SELECT * FROM {qualified} WHERE false");
            let mut tx = runtime.begin().await.unwrap();
            let result = sqlx::query(sqlx::AssertSqlSafe(query))
                .fetch_all(&mut *tx)
                .await;
            tx.rollback().await.unwrap();
            let error = match result {
                Err(error) => error,
                Ok(_) => {
                    panic!("actual direct read must be refused, not hidden by an empty result")
                }
            };
            assert!(
                error
                    .as_database_error()
                    .is_some_and(|e| e.code().as_deref() == Some("42501"))
            );
        }
        for query in [
            "SELECT * FROM public.account_security_lock_shared_v1($1)",
            "SELECT * FROM public.account_security_lock_exclusive_v1($1)",
        ] {
            let mut tx = runtime.begin().await.unwrap();
            let result = sqlx::query(query)
                .bind(attempt.account)
                .fetch_all(&mut *tx)
                .await;
            tx.rollback().await.unwrap();
            let error = match result {
                Err(error) => error,
                Ok(_) => panic!("raw global Account projection is not a Business capability"),
            };
            assert!(
                error
                    .as_database_error()
                    .is_some_and(|e| e.code().as_deref() == Some("42501"))
            );
        }
        let mut tx = runtime.begin().await.unwrap();
        current(&mut tx, &verifier, &cookies.0[ACCESS], ttl).await;
        tx.commit().await.unwrap();
        assert!(before == inventory(&pool, None).await);
        runtime.close().await;
    }

    include!("native_business_session_catalog.rs");
}
