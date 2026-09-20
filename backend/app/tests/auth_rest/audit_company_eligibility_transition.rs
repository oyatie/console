// Include inside business_session_transition. The genuine business-session
// predecessor is replayed from frozen current operator bytes, never fabricated.
mod eligibility_transition {
    use super::*;

    const NAME: &str = "account_company_setup_eligibility_v1";
    const FROZEN_OPERATOR: &[(&str, &str)] = &[
        (
            include_str!("fixtures/account-business-session-c613626e-root.sql"),
            "ee152286a79ceb6432724befe426d90e89dc4c41b40b4cbbb7618b8ce1e5459a",
        ),
        (
            include_str!("fixtures/account-business-session-c613626e-credentials.sql"),
            "3212d5ecf6f9d9310c5d2f3514464e425238b79f00806b5748bd029b33679905",
        ),
        (
            include_str!("fixtures/account-business-session-c613626e-verify.sql"),
            "5bd40cbf253365934d7175b65106b9d8a91b85c3d345182aa9c15c0a64899893",
        ),
    ];

    async fn business_state(connection: &mut PgConnection) {
        operator(connection).await;
        for (source, digest, expected) in [
            (
                include_str!("fixtures/account-business-session-c613626e-root-state.sql"),
                "f67f391d0828aa8b35182b6bc324f46f66f57e027f9ea06a68b6e1d07a86af8e",
                "account_custody.native_finalized",
            ),
            (
                include_str!("fixtures/account-business-session-c613626e-credential-state.sql"),
                "019a4a51caf28e1f744ac254e043f60cc06d2148b495c02350bae1dc4f2d8679",
                "account_credentials.native_finalized",
            ),
        ] {
            assert_eq!(
                hex::encode(Sha256::digest(source)),
                digest,
                "frozen Business-session classifier identity"
            );
            let observed: String = sqlx::query_scalar(sqlx::AssertSqlSafe(source))
                .fetch_one(&mut *connection)
                .await
                .unwrap();
            assert_eq!(
                observed, expected,
                "actual initialized Business-session predecessor required"
            );
        }
        let absent: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_proc WHERE pronamespace='public'::regnamespace AND proname=$1)")
            .bind(NAME).fetch_one(&mut *connection).await.unwrap();
        assert!(
            absent,
            "predecessor must lack every eligibility overload/owner"
        );
        // Actual inherited session capabilities remain present and vetted.
        super::checked_additions(connection).await;
    }

    async fn business_predecessor(
        pool: &PgPool,
    ) -> (Fixture, AppConfig, SigningKey, Attempt, Cookies) {
        let fixture = super::audited228(pool).await;
        let mut tx = pool.begin().await.unwrap();
        operator(&mut tx).await;
        for (source, digest) in FROZEN_OPERATOR {
            assert_eq!(
                hex::encode(Sha256::digest(source)),
                *digest,
                "frozen actual operator source"
            );
            sqlx::raw_sql(*source).execute(&mut *tx).await.unwrap();
        }
        business_state(&mut tx).await;
        tx.commit().await.unwrap();
        fixture
    }

    pub(super) async fn checked_addition(connection: &mut PgConnection) -> Value {
        let records: Vec<(Value, bool)> = sqlx::query_as(r#"
          SELECT to_jsonb(p),
            p.proargtypes='2950'::oidvector AND p.pronargs=1 AND p.pronargdefaults=0
            AND p.proargnames=ARRAY['p_account']::text[] AND p.proallargtypes IS NULL AND p.proargmodes IS NULL
            AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND NOT p.proleakproof
            AND p.provolatile='v' AND p.proparallel='u' AND NOT p.proretset
            AND p.prorettype='bool'::regtype AND p.proowner='console_account_owner'::regrole
            AND p.prolang=(SELECT oid FROM pg_language WHERE lanname='plpgsql')
            AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp','row_security=on']::text[]
            AND (SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),
                CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
                a.privilege_type,a.is_grantable)
                ORDER BY CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END)
                FROM aclexplode(p.proacl) a)
              = '[["console_account_owner","console_account_owner","EXECUTE",false],["console_account_owner","console_auth_rt","EXECUTE",false]]'::jsonb
          FROM pg_proc p WHERE p.pronamespace='public'::regnamespace AND p.proname=$1 ORDER BY p.oid
        "#).bind(NAME).fetch_all(&mut *connection).await.unwrap();
        assert!(
            records.len() == 1 && records[0].1,
            "NATIVE_COMPANY_ELIGIBILITY_TRANSITION: one exact read-only addition required"
        );
        records.into_iter().next().unwrap().0
    }

    fn strip_checked_one(before: &Value, after: &Value, addition: &Value) -> Value {
        adversarial::assert_metadata_shape(before);
        adversarial::assert_metadata_shape(after);
        let prior = before["functions"].as_array().unwrap();
        let final_rows = after["functions"].as_array().unwrap();
        let id = addition["oid"].as_str().expect("vetted catalog OID");
        let numeric = id.parse::<u32>().expect("vetted numeric OID");
        assert!(numeric > 0 && numeric.to_string() == id && addition["proname"] == NAME);
        assert!(
            !prior
                .iter()
                .any(|r| r["oid"] == addition["oid"] || r["proname"] == NAME),
            "predecessor already had helper identity"
        );
        assert_eq!(
            final_rows.iter().filter(|row| *row == addition).count(),
            1,
            "missing/altered/duplicate vetted addition"
        );
        let mut stripped = after.clone();
        stripped["functions"]
            .as_array_mut()
            .unwrap()
            .retain(|row| row != addition);
        stripped
    }

    // Compose with the parent's unchanged two-helper preservation oracle. This
    // strips only the actual independently vetted one-helper row, no old row.
    pub(super) async fn without_checked_addition(
        connection: &mut PgConnection,
        before: &Value,
        after: &Value,
    ) -> Value {
        strip_checked_one(before, after, &checked_addition(connection).await)
    }

    async fn predecessor_equivalent(
        connection: &mut PgConnection,
        before: &Value,
        after: &Value,
    ) -> Value {
        let stripped = without_checked_addition(connection, before, after).await;
        assert_eq!(
            after["functions"].as_array().unwrap().len(),
            before["functions"].as_array().unwrap().len() + 1
        );
        assert!(
            stripped == *before,
            "single read-helper transition changed another metadata fact"
        );
        stripped
    }

    #[test]
    fn eligibility_addition_oracle_rejects_missing_extra_modified_and_stale_evidence() {
        let mut before = adversarial::metadata_control(false);
        before["functions"][0]["proname"] = json!("existing_owner");
        let added = json!({"oid":"900","proname":NAME});
        let mut after = before.clone();
        after["functions"]
            .as_array_mut()
            .unwrap()
            .push(added.clone());
        let oracle = |left: &Value, right: &Value, allowed: &Value| {
            assert_eq!(
                right["functions"].as_array().unwrap().len(),
                left["functions"].as_array().unwrap().len() + 1
            );
            assert!(strip_checked_one(left, right, allowed) == *left);
        };
        oracle(&before, &after, &added);
        for fault in 0..9 {
            let mut left = before.clone();
            let mut right = after.clone();
            let mut allowed = added.clone();
            match fault {
                0 => {
                    right["functions"].as_array_mut().unwrap().pop();
                }
                1 => right["functions"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"oid":"901","proname":"extra"})),
                2 => right["functions"][0]["prosrc"] = json!("altered"),
                3 => {
                    right["functions"].as_array_mut().unwrap().remove(0);
                }
                4 => right["functions"]
                    .as_array_mut()
                    .unwrap()
                    .push(added.clone()),
                5 => allowed["oid"] = json!("902"),
                6 => left["functions"][0] = added.clone(),
                7 => allowed["proname"] = json!("not_the_helper"),
                _ => right["roles"][0]["changed"] = json!(true),
            }
            assert!(
                std::panic::catch_unwind(|| oracle(&left, &right, &allowed)).is_err(),
                "accepted corruption {fault}"
            );
        }
    }

    // Full additional preservation surface, including tables beyond historical
    // TABLES and pg_control_system's ACL. No returned sensitive rows are printed.
    async fn all_business_state(connection: &mut PgConnection) -> Value {
        let tables: Vec<String>=sqlx::query_scalar("SELECT relname::text FROM pg_class WHERE relnamespace='public'::regnamespace AND relkind IN ('r','p') ORDER BY relname COLLATE \"C\"")
            .fetch_all(&mut *connection).await.unwrap();
        assert!(!tables.is_empty() && tables.len() <= 1024);
        let mut content = serde_json::Map::new();
        for table in tables {
            let quoted = table.replace('"', "\"\"");
            let sql = format!(
                "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text COLLATE \"C\"),'[]'::jsonb)::text FROM public.\"{quoted}\" t"
            );
            let rows: String = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
                .fetch_one(&mut *connection)
                .await
                .unwrap();
            assert!(content.insert(table, json!(rows)).is_none());
        }
        let controls:Value=sqlx::query_scalar(r#"SELECT jsonb_build_object(
            'relations',(SELECT jsonb_agg(jsonb_build_object('oid',c.oid,'name',c.relname,'owner',c.relowner,'acl',c.relacl,'rls',c.relrowsecurity,'force',c.relforcerowsecurity,'columns',(SELECT jsonb_agg(to_jsonb(a) ORDER BY a.attnum) FROM pg_attribute a WHERE a.attrelid=c.oid)) ORDER BY c.oid) FROM pg_class c WHERE c.relnamespace='public'::regnamespace AND c.relkind IN ('r','p')),
            'control_system',(SELECT to_jsonb(p) FROM pg_proc p WHERE p.oid='pg_catalog.pg_control_system()'::regprocedure))"#)
            .fetch_one(connection).await.unwrap();
        assert!(controls["relations"].is_array() && controls["control_system"].is_object());
        json!({"tables":content,"controls":controls})
    }

    async fn upgrade(pool: &PgPool, observer: bool) {
        let (app, config, key, attempt, cookies) = business_predecessor(pool).await;
        let mut tx = pool.begin().await.unwrap();
        business_state(&mut tx).await;
        let untouched_rows = rows(&mut tx).await;
        let untouched_meta = metadata(&mut tx).await;
        tx.rollback().await.unwrap();
        let startup = AppState::from_config(config.clone()).await;
        if let Ok(state) = &startup {
            state.shutdown_realtime().await;
        }
        assert!(
            matches!(startup,Err(console_app::AppError::Config(ref code)) if code=="account_custody.native_upgrade_required"),
            "new serving must refuse real current Business-session predecessor"
        );
        let mut tx = pool.begin().await.unwrap();
        operator(&mut tx).await;
        assert_preserved(&untouched_rows, &rows(&mut tx).await);
        assert!(
            untouched_meta == metadata(&mut tx).await,
            "failed startup modified the predecessor"
        );
        if observer {
            let source =
                include_str!("fixtures/account-native-deployment228-observer-6c884418.sql");
            assert!(hex::encode(Sha256::digest(source)) == OBSERVER_SHA256);
            sqlx::raw_sql(source).execute(&mut *tx).await.unwrap();
        }
        business_state(&mut tx).await;
        assert!(state(&mut tx).await == "account_custody.native_upgrade_required");
        let before = metadata(&mut tx).await;
        let before_rows = rows(&mut tx).await;
        let entire_before = all_business_state(&mut tx).await;
        compose(&mut tx).await.unwrap();
        assert_final(&mut tx).await;
        let after = metadata(&mut tx).await;
        assert!(
            predecessor_equivalent(&mut tx, &before, &after).await == before,
            "Business-session transition changed metadata beyond one exact eligibility helper"
        );
        assert_preserved(&before_rows, &rows(&mut tx).await);
        assert!(
            entire_before == all_business_state(&mut tx).await,
            "complete public business state changed"
        );
        compose(&mut tx).await.unwrap();
        assert_final(&mut tx).await;
        assert!(
            after == metadata(&mut tx).await,
            "finalized replay changed metadata"
        );
        assert_preserved(&before_rows, &rows(&mut tx).await);
        assert!(
            entire_before == all_business_state(&mut tx).await,
            "complete public business state changed"
        );
        if observer {
            // Cluster-scoped observer installation is never leaked across tests.
            tx.rollback().await.unwrap();
            let mut check = pool.begin().await.unwrap();
            business_state(&mut check).await;
            assert!(untouched_meta == metadata(&mut check).await);
            assert_preserved(&untouched_rows, &rows(&mut check).await);
            check.rollback().await.unwrap();
        } else {
            tx.commit().await.unwrap();
            let rollback_config = config.clone();
            let actual = AppState::from_config(config.clone()).await.unwrap();
            let service = build_router(actual.clone());
            let me = request(&service, "GET", "/api/v2/accounts/me", &cookies, None, &[]).await;
            projection(&me.json(StatusCode::OK), attempt.account);
            me.private();
            let auth_read = login_test_pool(pool, TestDatabaseLogin::Auth).await;
            let eligible: bool =
                sqlx::query_scalar("SELECT public.account_company_setup_eligibility_v1($1)")
                    .bind(attempt.account)
                    .fetch_one(&auth_read)
                    .await
                    .unwrap();
            assert!(
                eligible,
                "real initialized operator eligibility must survive metadata upgrade"
            );
            auth_read.close().await;
            let jwt = config.jwt.unwrap();
            let verifier = JwtVerifier::from_es256_public_pem(
                JwtSettings {
                    issuer: jwt.issuer,
                    audience: jwt.audience,
                    access_token_ttl: Duration::minutes(15),
                },
                jwt.public_key_pem.as_bytes(),
            )
            .unwrap();
            let runtime = login_test_pool(pool, TestDatabaseLogin::Business).await;
            let mut read = runtime.begin().await.unwrap();
            let live = live_account_session_in_tx(
                &mut read,
                &verifier,
                &cookies.0[ACCESS],
                config.auth_rest.unwrap().refresh_family_absolute_ttl,
            )
            .await
            .unwrap_or_else(|_| {
                panic!("original real session must resolve through Business after upgrade")
            });
            assert!(
                live.account_id == attempt.account
                    && signed_claims(&cookies.0[ACCESS], &key).unwrap()["sid"]
                        == json!(live.session_id)
            );
            read.commit().await.unwrap();
            runtime.close().await;
            actual.shutdown_realtime().await;
            let mut check = pool.acquire().await.unwrap();
            assert_preserved(&before_rows, &rows(&mut check).await);
            assert!(
                entire_before == all_business_state(&mut check).await,
                "complete public business state changed"
            );
            assert!(after == metadata(&mut check).await);
            drop(check);
            drop(service);
            drop(actual);
            assert_session_consumers_drained(pool).await;

            // The committed successor already served the genuine original
            // session. Reuse its actual no-op operator replay to acquire the
            // existing maintenance locks and certify privilege/profile before
            // the exact helper-only inverse. No new rollback framework.
            let mut rollback = pool.begin().await.unwrap();
            compose(&mut rollback).await.unwrap();
            assert_final(&mut rollback).await;
            checked_addition(&mut rollback).await;
            assert!(after == metadata(&mut rollback).await);
            assert_preserved(&before_rows, &rows(&mut rollback).await);
            assert!(
                entire_before == all_business_state(&mut rollback).await,
                "complete public business state changed"
            );
            sqlx::raw_sql(
                "DROP FUNCTION public.account_company_setup_eligibility_v1(uuid) RESTRICT;",
            )
            .execute(&mut *rollback)
            .await
            .unwrap();
            business_state(&mut rollback).await;
            assert!(
                before == metadata(&mut rollback).await,
                "helper inverse must restore exact predecessor metadata"
            );
            assert_preserved(&before_rows, &rows(&mut rollback).await);
            assert!(
                entire_before == all_business_state(&mut rollback).await,
                "complete public business state changed"
            );
            rollback.commit().await.unwrap();

            // Fresh transaction proves the inverse committed; startup is the
            // actual current binary admission path, not a simulated status.
            let mut committed = pool.begin().await.unwrap();
            business_state(&mut committed).await;
            assert!(before == metadata(&mut committed).await);
            assert_preserved(&before_rows, &rows(&mut committed).await);
            assert!(
                entire_before == all_business_state(&mut committed).await,
                "complete public business state changed"
            );
            committed.rollback().await.unwrap();
            let refused = AppState::from_config(rollback_config.clone()).await;
            if let Ok(unexpected) = &refused {
                unexpected.shutdown_realtime().await;
            }
            assert!(
                matches!(&refused, Err(console_app::AppError::Config(code)) if code == "account_custody.native_upgrade_required"),
                "current serving must refuse the committed exact predecessor"
            );
            drop(refused);
            // Old compatible reader still works while eligibility is absent.
            // Real Business authentication, never operator SET ROLE.
            let compatible_runtime = login_test_pool(pool, TestDatabaseLogin::Business).await;
            let mut compatible_read = compatible_runtime.begin().await.unwrap();
            let compatible_live = live_account_session_in_tx(
                &mut compatible_read,
                &verifier,
                &cookies.0[ACCESS],
                rollback_config
                    .auth_rest
                    .as_ref()
                    .unwrap()
                    .refresh_family_absolute_ttl,
            )
            .await
            .unwrap_or_else(|_| {
                panic!("unchanged session reader must work on committed helper-absent predecessor")
            });
            assert!(
                compatible_live.account_id == attempt.account
                    && signed_claims(&cookies.0[ACCESS], &key).unwrap()["sid"]
                        == json!(compatible_live.session_id)
            );
            compatible_read.commit().await.unwrap();
            compatible_runtime.close().await;
            assert_session_consumers_drained(pool).await;

            let mut reapply = pool.begin().await.unwrap();
            business_state(&mut reapply).await;
            assert!(
                before == metadata(&mut reapply).await,
                "refused startup must not repair predecessor metadata"
            );
            assert_preserved(&before_rows, &rows(&mut reapply).await);
            assert!(
                entire_before == all_business_state(&mut reapply).await,
                "complete public business state changed"
            );
            compose(&mut reapply).await.unwrap();
            assert_final(&mut reapply).await;
            let reapplied_metadata = metadata(&mut reapply).await;
            // Dropped function legitimately receives a new pg_proc OID. Vet
            // exactly the one re-created addition; every old row stays exact.
            assert!(
                predecessor_equivalent(&mut reapply, &before, &reapplied_metadata).await == before
            );
            assert_preserved(&before_rows, &rows(&mut reapply).await);
            assert!(
                entire_before == all_business_state(&mut reapply).await,
                "complete public business state changed"
            );
            compose(&mut reapply).await.unwrap();
            assert_final(&mut reapply).await;
            assert!(
                reapplied_metadata == metadata(&mut reapply).await,
                "reapplied successor replay changed metadata"
            );
            assert_preserved(&before_rows, &rows(&mut reapply).await);
            assert!(
                entire_before == all_business_state(&mut reapply).await,
                "complete public business state changed"
            );
            reapply.commit().await.unwrap();

            let restored = AppState::from_config(rollback_config.clone())
                .await
                .unwrap();
            let restored_service = build_router(restored.clone());
            let restored_me = request(
                &restored_service,
                "GET",
                "/api/v2/accounts/me",
                &cookies,
                None,
                &[],
            )
            .await;
            projection(&restored_me.json(StatusCode::OK), attempt.account);
            restored_me.private();
            let restored_runtime = login_test_pool(pool, TestDatabaseLogin::Business).await;
            let mut restored_read = restored_runtime.begin().await.unwrap();
            let restored_live = live_account_session_in_tx(
                &mut restored_read,
                &verifier,
                &cookies.0[ACCESS],
                rollback_config.auth_rest.as_ref().unwrap().refresh_family_absolute_ttl,
            )
            .await
            .unwrap_or_else(|_| panic!("original genuine session must remain usable after committed rollback and reapply"));
            assert!(
                restored_live.account_id == attempt.account
                    && signed_claims(&cookies.0[ACCESS], &key).unwrap()["sid"]
                        == json!(restored_live.session_id)
            );
            restored_read.commit().await.unwrap();
            restored_runtime.close().await;
            restored.shutdown_realtime().await;
            drop(restored_service);
            drop(restored);
            assert_session_consumers_drained(pool).await;
            let mut final_check = pool.acquire().await.unwrap();
            assert_preserved(&before_rows, &rows(&mut final_check).await);
            assert!(
                entire_before == all_business_state(&mut final_check).await,
                "complete public business state changed"
            );
            assert!(reapplied_metadata == metadata(&mut final_check).await);
        }
        drop(app);
    }

    #[sqlx::test(migrations = false)]
    async fn populated_business_session_eligibility_upgrade_preserves_history_and_replays(
        pool: PgPool,
    ) {
        upgrade(&pool, false).await;
    }

    #[sqlx::test(migrations = false)]
    async fn initialized_observer_business_session_eligibility_upgrade_preserves_history(
        pool: PgPool,
    ) {
        upgrade(&pool, true).await;
    }

    #[sqlx::test(migrations = false)]
    async fn malformed_eligibility_helpers_refuse_without_repair_and_actual_install_is_atomic(
        pool: PgPool,
    ) {
        let (app, _, _, _, _) = business_predecessor(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        business_state(&mut tx).await;
        let original = metadata(&mut tx).await;
        let original_rows = rows(&mut tx).await;
        let entire_before = all_business_state(&mut tx).await;
        // Exact and alternative signatures, wrong ownership and behavior. The
        // owner must reject unknown state, never replace a function by its name.
        for signature in ["uuid", "text", "uuid,uuid"] {
            let mut fault = tx.begin().await.unwrap();
            let ddl = format!(
                "CREATE FUNCTION public.{NAME}({signature}) RETURNS boolean LANGUAGE sql SECURITY INVOKER AS 'SELECT false'; ALTER FUNCTION public.{NAME}({signature}) OWNER TO console_app; REVOKE ALL ON FUNCTION public.{NAME}({signature}) FROM PUBLIC"
            );
            sqlx::raw_sql(sqlx::AssertSqlSafe(ddl))
                .execute(&mut *fault)
                .await
                .unwrap();
            assert_eq!(state(&mut fault).await, "account_native.profile_mismatch");
            let fault_meta = metadata(&mut fault).await;
            let mut attempt = fault.begin().await.unwrap();
            let result = compose(&mut attempt).await;
            attempt.rollback().await.unwrap();
            let error = result.expect_err("operator repaired unknown eligibility helper");
            assert!(
                error
                    .as_database_error()
                    .is_some_and(|e| e.code().as_deref() == Some("P0001")
                        && e.message() == "account_native.profile_mismatch")
            );
            assert!(fault_meta == metadata(&mut fault).await);
            assert!(entire_before == all_business_state(&mut fault).await);
            fault.rollback().await.unwrap();
            assert!(original == metadata(&mut tx).await);
            business_state(&mut tx).await;
        }
        let mut fault = tx.begin().await.unwrap();
        sqlx::raw_sql(r#"CREATE SEQUENCE public.eligibility_test_completed_acl_seen;
          CREATE FUNCTION public.eligibility_test_after_complete_acl() RETURNS event_trigger LANGUAGE plpgsql AS $$
          BEGIN IF EXISTS(SELECT 1 FROM pg_proc p
            WHERE p.oid=to_regprocedure('public.account_company_setup_eligibility_v1(uuid)')
              AND p.proowner='console_account_owner'::regrole
              AND p.prosecdef AND p.provolatile='v' AND p.proparallel='u'
              AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp','row_security=on']::text[]
              AND (SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),
                CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
                a.privilege_type,a.is_grantable)
                ORDER BY CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END)
                FROM aclexplode(p.proacl) a)
                ='[["console_account_owner","console_account_owner","EXECUTE",false],["console_account_owner","console_auth_rt","EXECUTE",false]]'::jsonb
          ) THEN
            PERFORM nextval('public.eligibility_test_completed_acl_seen'::regclass);
            RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='ELIGIBILITY_TEST_AFTER_COMPLETE_ACL';
          END IF; END $$;
          CREATE EVENT TRIGGER eligibility_test_after_complete_acl ON ddl_command_end WHEN TAG IN ('GRANT') EXECUTE FUNCTION public.eligibility_test_after_complete_acl()"#)
            .execute(&mut *fault).await.unwrap();
        let fault_meta = metadata(&mut fault).await;
        let mut attempt = fault.begin().await.unwrap();
        let result = compose(&mut attempt).await;
        attempt.rollback().await.unwrap();
        let error = result.expect_err("actual complete owner/ACL fault was not reached");
        assert!(
            error
                .as_database_error()
                .is_some_and(|e| e.code().as_deref() == Some("P0001")
                    && e.message() == "ELIGIBILITY_TEST_AFTER_COMPLETE_ACL"),
            "unrelated failure is not atomicity evidence"
        );
        let witnessed: bool = sqlx::query_scalar(
            "SELECT is_called AND last_value=1 FROM public.eligibility_test_completed_acl_seen",
        )
        .fetch_one(&mut *fault)
        .await
        .unwrap();
        assert!(
            witnessed,
            "exact owner+ACL grant boundary must have fired once before rollback"
        );
        assert!(
            fault_meta == metadata(&mut fault).await,
            "partial helper install escaped rollback"
        );
        assert!(entire_before == all_business_state(&mut fault).await);
        business_state(&mut fault).await;
        fault.rollback().await.unwrap();
        assert!(original == metadata(&mut tx).await);
        compose(&mut tx).await.unwrap();
        assert_final(&mut tx).await;
        let after = metadata(&mut tx).await;
        assert!(predecessor_equivalent(&mut tx, &original, &after).await == original);
        assert_preserved(&original_rows, &rows(&mut tx).await);
        assert!(entire_before == all_business_state(&mut tx).await);
        tx.rollback().await.unwrap();
        drop(app);
    }

    #[sqlx::test(migrations = false)]
    async fn finalized_eligibility_body_acl_and_overload_drift_are_never_resealed(pool: PgPool) {
        let (app, _, _, _, _) = business_predecessor(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        compose(&mut tx).await.unwrap();
        assert_final(&mut tx).await;
        checked_addition(&mut tx).await;
        let original = metadata(&mut tx).await;
        let entire_before = all_business_state(&mut tx).await;
        for fault_sql in [
            "GRANT EXECUTE ON FUNCTION public.account_company_setup_eligibility_v1(uuid) TO PUBLIC",
            "GRANT EXECUTE ON FUNCTION public.account_company_setup_eligibility_v1(uuid) TO console_rt",
            "ALTER FUNCTION public.account_company_setup_eligibility_v1(uuid) OWNER TO console_app",
            "ALTER FUNCTION public.account_company_setup_eligibility_v1(uuid) STABLE",
            "ALTER FUNCTION public.account_company_setup_eligibility_v1(uuid) SET search_path=public,pg_catalog",
            "CREATE OR REPLACE FUNCTION public.account_company_setup_eligibility_v1(p_account uuid) RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE SET search_path=pg_catalog,pg_temp SET row_security=on AS 'BEGIN RETURN true; END;'",
            "CREATE FUNCTION public.account_company_setup_eligibility_v1(text) RETURNS boolean LANGUAGE sql AS 'SELECT false'",
        ] {
            let mut fault = tx.begin().await.unwrap();
            sqlx::raw_sql(fault_sql).execute(&mut *fault).await.unwrap();
            assert_eq!(state(&mut fault).await, "account_native.profile_mismatch");
            let fault_meta = metadata(&mut fault).await;
            let mut attempt = fault.begin().await.unwrap();
            let result = compose(&mut attempt).await;
            attempt.rollback().await.unwrap();
            let error =
                result.expect_err("operator repaired or resealed altered eligibility source");
            assert!(
                error
                    .as_database_error()
                    .is_some_and(|e| e.code().as_deref() == Some("P0001")
                        && e.message() == "account_native.profile_mismatch")
            );
            assert!(fault_meta == metadata(&mut fault).await);
            assert!(entire_before == all_business_state(&mut fault).await);
            fault.rollback().await.unwrap();
            assert!(original == metadata(&mut tx).await);
            assert_final(&mut tx).await;
        }
        tx.rollback().await.unwrap();
        drop(app);
    }

    #[sqlx::test(migrations = false)]
    async fn corrupt_designation_target_or_missing_receipt_is_unavailable_not_ineligible(
        pool: PgPool,
    ) {
        let (app, _, _, account, _) = business_predecessor(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        compose(&mut tx).await.unwrap();
        assert_final(&mut tx).await;
        let healthy = all_business_state(&mut tx).await;
        let healthy_metadata = metadata(&mut tx).await;
        let positive: bool =
            sqlx::query_scalar("SELECT public.account_company_setup_eligibility_v1($1)")
                .bind(account.account)
                .fetch_one(&mut *tx)
                .await
                .unwrap();
        assert!(positive, "genuine initialized designation positive control");
        for change in [
            "UPDATE public.deployment_operator_head SET database_name=database_name||'-WRONG-TARGET' WHERE singleton=1",
            "UPDATE public.deployment_operator_head SET receipt_id=gen_random_uuid() WHERE singleton=1",
        ] {
            let mut fault = tx.begin().await.unwrap();
            // Corruption injection only. Restore every trigger's exact mode
            // before invoking the owner; an unrelated disabled guard or dropped
            // constraint must not be the reason the read refuses.
            let restore: Vec<String> = sqlx::query_scalar(r#"SELECT format(
                'ALTER TABLE public.deployment_operator_head %s TRIGGER %I',
                CASE tgenabled WHEN 'A' THEN 'ENABLE ALWAYS' WHEN 'O' THEN 'ENABLE'
                    WHEN 'R' THEN 'ENABLE REPLICA' ELSE 'DISABLE' END,tgname)
                FROM pg_trigger WHERE tgrelid='public.deployment_operator_head'::regclass ORDER BY oid"#)
                .fetch_all(&mut *fault).await.unwrap();
            assert!(!restore.is_empty());
            sqlx::raw_sql("ALTER TABLE public.deployment_operator_head DISABLE TRIGGER ALL")
                .execute(&mut *fault)
                .await
                .unwrap();
            assert_eq!(
                sqlx::query(change)
                    .execute(&mut *fault)
                    .await
                    .unwrap()
                    .rows_affected(),
                1
            );
            for statement in restore {
                sqlx::raw_sql(sqlx::AssertSqlSafe(statement))
                    .execute(&mut *fault)
                    .await
                    .unwrap();
            }
            assert!(
                healthy_metadata == metadata(&mut fault).await,
                "fault must preserve all metadata before owner read"
            );
            let corrupt = all_business_state(&mut fault).await;
            let corrupt_metadata = metadata(&mut fault).await;
            let mut read = fault.begin().await.unwrap();
            let result: Result<bool, sqlx::Error> =
                sqlx::query_scalar("SELECT public.account_company_setup_eligibility_v1($1)")
                    .bind(account.account)
                    .fetch_one(&mut *read)
                    .await;
            read.rollback().await.unwrap();
            let error = result.expect_err("corrupt designation must not become true or false");
            assert!(
                error
                    .as_database_error()
                    .is_some_and(|e| e.code().as_deref() == Some("P0001")
                        && e.message() == "account.authority_unavailable")
            );
            assert!(corrupt == all_business_state(&mut fault).await);
            assert!(corrupt_metadata == metadata(&mut fault).await);
            fault.rollback().await.unwrap();
            assert!(healthy == all_business_state(&mut tx).await);
            assert!(healthy_metadata == metadata(&mut tx).await);
            assert_final(&mut tx).await;
        }
        tx.rollback().await.unwrap();
        drop(app);
    }
}
