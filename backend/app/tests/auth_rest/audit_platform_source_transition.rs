// Include inside eligibility_transition; additive actual finalizer/SQL capability evidence.
mod platform_source_transition {
    use super::*;
    use console_platform_auth::RefreshTokenStore;
    use sqlx::{Acquire, Connection};

    const BRIDGE: &str = "auth_legacy_platform_source_material_v1";
    const CONTEXT: &str = "auth_legacy_session_context_v1";
    const INSTALL: &str = include_str!("fixtures/platform-source-install.sql");
    const SNAPSHOT: &str = include_str!("fixtures/platform-source-prospective.sql");
    const COMPANY_SNAPSHOT: &str = include_str!("fixtures/platform-source-historical.sql");
    const COMPANY_FINAL: [&str; 2] = [
        "ffdd70fc9ea8f96509a9614826415226896d7deed857930b1257f47d920a6131",
        "88a0df75fdb3e18280b9a64951fb12fea491febb1ec3868c8016b86f8ea8f963",
    ];
    const SOURCE_PRIOR: [&str; 2] = [
        "0f2db80a4afeacb1d6d6aaf0bbaef9d8f7968e77500322cf691774f02d88e8ea",
        "93236c9a9e850db234f4dc777553e4eb829bbb76a0f6cc27551f761cc28bffe7",
    ];
    const SOURCE_FINAL: [&str; 2] = [
        "5aee358ff0bea94268d40b5f16f86779952ea95a74cab48cbf99984eedf6e2da",
        "475c1f9a43b402539e0469f31adc6f06848669abfa34c08350aa1ee006664103",
    ];
    const FROZEN_COMPANY: &[(&str, &str)] = &[
        (
            include_str!("fixtures/account-company-eligibility-26d78648-root.sql"),
            "ce195636a182721c027c689a6a021de03eafa17ff30e59607257c963903afb9e",
        ),
        (
            include_str!("fixtures/account-company-eligibility-26d78648-credentials.sql"),
            "f1925281a6e175214c5ff647419a3428fca9faae79f461a161a5938bf6b02253",
        ),
        (
            include_str!("fixtures/account-company-eligibility-26d78648-verify.sql"),
            "a498b59d41bc9c64b82a61a3deeb1f4652bf4441b81e17f538ccca6a04025151",
        ),
        (
            include_str!("fixtures/account-company-eligibility-26d78648-root-state.sql"),
            "89261eaa382aea8e61284878ef4318b556396b0462ea6466fefb858a8910905f",
        ),
        (
            include_str!("fixtures/account-company-eligibility-26d78648-credential-state.sql"),
            "b00737128fe9edd77406fbb68f53a9eb768c16485d22c77efae620f4dab8965d",
        ),
    ];

    async fn profile(connection: &mut PgConnection, query: &str) -> (Value, String) {
        let sql = format!(
            "SELECT snapshot,snapshot::text,snapshot_sha256 FROM ({}) captured",
            query.trim().trim_end_matches(';')
        );
        let (value, text, digest): (Value, String, String) =
            sqlx::query_as(sqlx::AssertSqlSafe(sql))
                .fetch_one(connection)
                .await
                .unwrap();
        assert!(serde_json::from_str::<Value>(&text).unwrap() == value);
        assert_eq!(hex::encode(Sha256::digest(text.as_bytes())), digest);
        (value, digest)
    }

    async fn complete_metadata(connection: &mut PgConnection) -> Value {
        let mut value: Value =
            sqlx::query_scalar(include_str!("fixtures/platform-source-metadata.sql"))
                .fetch_one(&mut *connection)
                .await
                .unwrap();
        value["event_triggers"] = sqlx::query_scalar::<_, Value>("SELECT COALESCE(jsonb_agg(to_jsonb(e) ORDER BY e.oid),'[]'::jsonb) FROM pg_event_trigger e").fetch_one(connection).await.unwrap();
        assert!(
            value["functions"].is_array()
                && value["relations"].as_array().is_some_and(|v| !v.is_empty())
                && value["function_acls"].is_object()
        );
        value
    }

    // Diagnostics reveal structural paths only; equality remains exact.
    fn same_metadata(before: &Value, after: &Value) -> bool {
        fn paths(left: &Value, right: &Value, path: &str, out: &mut Vec<String>) {
            if left == right || out.len() >= 32 {
                return;
            }
            match (left, right) {
                (Value::Object(a), Value::Object(b)) => {
                    for key in a
                        .keys()
                        .chain(b.keys())
                        .collect::<std::collections::BTreeSet<_>>()
                    {
                        if out.len() >= 32 {
                            break;
                        }
                        match (a.get(key), b.get(key)) {
                            (Some(a), Some(b)) => paths(a, b, &format!("{path}.{key}"), out),
                            _ => out.push(format!("{path}.{key}:membership")),
                        }
                    }
                }
                (Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
                    for (i, (a, b)) in a.iter().zip(b).enumerate() {
                        if out.len() >= 32 {
                            break;
                        }
                        paths(a, b, &format!("{path}[{i}]"), out);
                    }
                }
                _ => out.push(path.to_owned()),
            }
        }
        if before != after {
            let mut changed = Vec::new();
            paths(before, after, "metadata", &mut changed);
            eprintln!("PLATFORM_SOURCE_METADATA_DIFFERENCE_PATHS: {changed:?}");
        }
        before == after
    }

    // A metadata-only operator phase must not race maintenance updating index
    // statistics. Preserve the complete snapshot and compare every original field.
    async fn lock_metadata_relations(connection: &mut PgConnection) {
        let relations: Vec<(i64, String)> = sqlx::query_as(
            "SELECT c.oid::bigint,format('LOCK TABLE ONLY %I.%I IN SHARE MODE',n.nspname,c.relname) FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relkind IN ('r','p','m') ORDER BY c.oid"
        ).fetch_all(&mut *connection).await.unwrap();
        assert!(!relations.is_empty());
        for (_, sql) in &relations {
            sqlx::raw_sql(sqlx::AssertSqlSafe(sql.as_str()))
                .execute(&mut *connection)
                .await
                .unwrap();
        }
        let expected: Vec<i64> = relations.iter().map(|(id, _)| *id).collect();
        let actual: Vec<i64> = sqlx::query_scalar(
            "SELECT relation::bigint FROM pg_catalog.pg_locks WHERE pid=pg_backend_pid() AND locktype='relation' AND granted AND mode='ShareLock' AND relation::bigint=ANY($1) ORDER BY relation"
        ).bind(&expected).fetch_all(&mut *connection).await.unwrap();
        assert_eq!(
            actual, expected,
            "same-backend granted ShareLock for every captured relation"
        );
        let current: Vec<i64> = sqlx::query_scalar(
            "SELECT oid::bigint FROM pg_catalog.pg_class WHERE relnamespace='public'::regnamespace AND relkind IN ('r','p','m') ORDER BY oid"
        ).fetch_all(&mut *connection).await.unwrap();
        assert_eq!(
            current, expected,
            "complete relation identities must remain stable"
        );
        let uncovered: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_catalog.pg_index i JOIN pg_catalog.pg_class c ON c.oid=i.indexrelid WHERE c.relnamespace='public'::regnamespace AND NOT(i.indrelid::bigint=ANY($1))"
        ).bind(&expected).fetch_one(connection).await.unwrap();
        assert_eq!(
            uncovered, 0,
            "every captured public index belongs to a locked relation"
        );
    }

    fn canonical(rows: &Value) -> Vec<String> {
        let mut values = rows
            .as_array()
            .unwrap()
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>();
        values.sort();
        values
    }

    fn exact_two_object_delta(before: &Value, after: &Value) {
        let old = before["functions"].as_array().unwrap();
        let new = after["functions"].as_array().unwrap();
        assert_eq!(new.len(), old.len() + 1);
        assert!(!old.iter().any(|r| r["proname"] == BRIDGE));
        let added = new
            .iter()
            .filter(|r| r["proname"] == BRIDGE)
            .collect::<Vec<_>>();
        assert_eq!(added.len(), 1);
        let added = added[0];
        assert_eq!(added["prosrc"], INSTALL.split("$body$").nth(1).unwrap());
        let old_context = old
            .iter()
            .filter(|r| r["proname"] == CONTEXT)
            .collect::<Vec<_>>();
        let new_context = new
            .iter()
            .filter(|r| r["proname"] == CONTEXT)
            .collect::<Vec<_>>();
        assert_eq!((old_context.len(), new_context.len()), (1, 1));
        let (old_context, new_context) = (old_context[0], new_context[0]);
        assert_eq!(old_context["proargtypes"], json!(["2950", "2950"]));
        let mut unchanged = new_context.clone();
        unchanged["proacl"] = old_context["proacl"].clone();
        assert!(unchanged == *old_context, "old context definition changed");
        assert!(old_context["proacl"] != new_context["proacl"]);
        let owner = before["roles"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["oid"] == old_context["proowner"])
            .collect::<Vec<_>>();
        assert_eq!(owner.len(), 1);
        let context_key = old_context["oid"].as_str().unwrap();
        let added_key = added["oid"].as_str().unwrap();
        assert_ne!(context_key, added_key);
        let mut acl = before["function_acls"][context_key].clone();
        assert!(
            !acl.as_array()
                .unwrap()
                .iter()
                .any(|r| r[1] == "console_credential_owner")
        );
        acl.as_array_mut().unwrap().push(json!([
            owner[0]["rolname"],
            "console_credential_owner",
            "EXECUTE",
            false
        ]));
        assert!(
            canonical(&acl) == canonical(&after["function_acls"][context_key]),
            "context ACL must add exactly one non-grantable owner grant"
        );
        assert!(
            canonical(&after["function_acls"][added_key])
                == canonical(&json!([
                    [
                        "console_credential_owner",
                        "console_credential_owner",
                        "EXECUTE",
                        false
                    ],
                    ["console_credential_owner", "console_rt", "EXECUTE", false]
                ]))
        );
        let mut stripped = after.clone();
        stripped["functions"]
            .as_array_mut()
            .unwrap()
            .retain(|r| r["oid"] != added["oid"]);
        let context = stripped["functions"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|r| r["oid"] == old_context["oid"])
            .unwrap();
        context["proacl"] = old_context["proacl"].clone();
        stripped["function_acls"]
            .as_object_mut()
            .unwrap()
            .remove(added_key);
        stripped["function_acls"][context_key] = before["function_acls"][context_key].clone();
        assert!(
            stripped == *before,
            "metadata changed beyond exact source bridge and exact context ACL"
        );
    }

    async fn checked_bridge(connection: &mut PgConnection) {
        let checked: Vec<Option<bool>> =
            sqlx::query_scalar(include_str!("fixtures/platform-source-checked.sql"))
                .fetch_all(&mut *connection)
                .await
                .unwrap();
        assert_eq!(
            checked,
            [Some(true)],
            "exact source bridge signature/owner/ACL/effective privilege required"
        );
        let body: String = sqlx::query_scalar("SELECT prosrc FROM pg_proc WHERE oid='public.auth_legacy_platform_source_material_v1(uuid,uuid)'::regprocedure").fetch_one(&mut *connection).await.unwrap();
        assert_eq!(body, INSTALL.split("$body$").nth(1).unwrap());
        let granted: bool = sqlx::query_scalar("SELECT has_function_privilege('console_credential_owner','public.auth_legacy_session_context_v1(uuid,uuid)','EXECUTE')").fetch_one(connection).await.unwrap();
        assert!(granted);
    }

    async fn prior(connection: &mut PgConnection, observer: bool) {
        operator(connection).await;
        for (source, expected_hash) in FROZEN_COMPANY {
            assert_eq!(hex::encode(Sha256::digest(source)), *expected_hash);
        }
        for (source, expected) in [
            (FROZEN_COMPANY[3].0, "account_custody.native_finalized"),
            (FROZEN_COMPANY[4].0, "account_credentials.native_finalized"),
        ] {
            let observed: String = sqlx::query_scalar(source)
                .fetch_one(&mut *connection)
                .await
                .unwrap();
            assert_eq!(
                observed, expected,
                "exact historical Company finalized classifier"
            );
        }
        assert_eq!(
            profile(connection, COMPANY_SNAPSHOT).await.1,
            COMPANY_FINAL[usize::from(observer)]
        );
        let source = profile(connection, SNAPSHOT).await;
        assert_eq!(source.1, SOURCE_PRIOR[usize::from(observer)]);
        assert_eq!(
            source.0["deployment_operator_boundary"]["startup_final_rights_valid"],
            false
        );
        let absent: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_proc WHERE pronamespace='public'::regnamespace AND proname='auth_legacy_platform_source_material_v1') AND NOT has_function_privilege('console_credential_owner','public.auth_legacy_session_context_v1(uuid,uuid)','EXECUTE')").fetch_one(connection).await.unwrap();
        assert!(absent);
    }

    async fn final_profile(connection: &mut PgConnection, observer: bool) {
        operator(connection).await;
        assert_final(connection).await;
        checked_bridge(connection).await;
        let p = profile(connection, SNAPSHOT).await;
        assert_eq!(p.1, SOURCE_FINAL[usize::from(observer)]);
        assert_eq!(
            p.0["deployment_operator_boundary"]["startup_final_rights_valid"],
            true
        );
    }

    async fn company_predecessor(
        pool: &PgPool,
        populated: bool,
    ) -> Option<(Fixture, AppConfig, SigningKey, Attempt, Cookies)> {
        let existing = if populated {
            Some(business_predecessor(pool).await)
        } else {
            prepare_http_database_staging(pool).await;
            None
        };
        let mut tx = pool.begin().await.unwrap();
        operator(&mut tx).await;
        for (source, hash) in &FROZEN_COMPANY[..3] {
            assert_eq!(hex::encode(Sha256::digest(source)), *hash);
            sqlx::raw_sql(*source).execute(&mut *tx).await.unwrap();
        }
        prior(&mut tx, false).await;
        tx.commit().await.unwrap();
        existing
    }

    async fn observer_fixture(connection: &mut PgConnection) {
        let source = include_str!("fixtures/account-native-deployment228-observer-6c884418.sql");
        assert_eq!(hex::encode(Sha256::digest(source)), OBSERVER_SHA256);
        sqlx::raw_sql(source).execute(connection).await.unwrap();
    }

    // Finish fixture maintenance before freezing exact catalog bytes. This is
    // disposable-test preparation, not a change to production custody profiles.
    async fn finish_fixture_maintenance(pool: &PgPool) {
        let mut connection = PgConnection::connect_with(&pool.connect_options())
            .await
            .unwrap();
        sqlx::raw_sql("SET statement_timeout='60s'; SET lock_timeout='5s'")
            .execute(&mut connection)
            .await
            .unwrap();
        let bounded: bool = sqlx::query_scalar(
            "SELECT current_setting('statement_timeout')::interval=interval '60 seconds' AND current_setting('lock_timeout')::interval=interval '5 seconds' AND session_user='console_buck_admin' AND current_user='console_buck_admin'"
        ).fetch_one(&mut connection).await.unwrap();
        assert!(
            bounded,
            "dedicated fixture-maintenance login and bounded waits"
        );
        let relations: Vec<(i64, String)> = sqlx::query_as(
            "SELECT c.oid::bigint,format('VACUUM (ANALYZE) %I.%I',n.nspname,c.relname) FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relkind IN ('r','p','m') ORDER BY c.oid"
        ).fetch_all(&mut connection).await.unwrap();
        assert!(!relations.is_empty());
        for (_, sql) in &relations {
            sqlx::raw_sql(sqlx::AssertSqlSafe(sql.as_str()))
                .execute(&mut connection)
                .await
                .unwrap();
        }
        let expected: Vec<i64> = relations.iter().map(|(id, _)| *id).collect();
        let actual: Vec<i64> = sqlx::query_scalar(
            "SELECT oid::bigint FROM pg_catalog.pg_class WHERE relnamespace='public'::regnamespace AND relkind IN ('r','p','m') ORDER BY oid"
        ).fetch_all(&mut connection).await.unwrap();
        assert_eq!(
            actual, expected,
            "exact fixture maintenance relation coverage"
        );
        let uncovered: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_catalog.pg_index i JOIN pg_catalog.pg_class c ON c.oid=i.indexrelid WHERE c.relnamespace='public'::regnamespace AND NOT(i.indrelid::bigint=ANY($1))"
        ).bind(&expected).fetch_one(&mut connection).await.unwrap();
        assert_eq!(uncovered, 0, "all captured public index parents maintained");
        connection.close().await.unwrap();
    }

    async fn upgrade_case(pool: &PgPool, populated: bool, observer: bool) {
        let existing = company_predecessor(pool, populated).await;
        if let Some((_, config, _, _, _)) = &existing {
            let result = AppState::from_config(config.clone()).await;
            if let Ok(app) = &result {
                app.shutdown_realtime().await;
            }
            assert!(
                matches!(result, Err(console_app::AppError::Config(ref code)) if code=="account_custody.native_upgrade_required")
            );
        }
        finish_fixture_maintenance(pool).await;
        let mut tx = pool.begin().await.unwrap();
        operator(&mut tx).await;
        lock_metadata_relations(&mut tx).await;
        let original_meta = complete_metadata(&mut tx).await;
        let original_rows = all_business_state(&mut tx).await;
        if observer {
            observer_fixture(&mut tx).await;
        }
        prior(&mut tx, observer).await;
        let before = complete_metadata(&mut tx).await;
        let before_rows = all_business_state(&mut tx).await;
        compose(&mut tx).await.unwrap();
        final_profile(&mut tx, observer).await;
        let after = complete_metadata(&mut tx).await;
        exact_two_object_delta(&before, &after);
        assert!(
            before_rows == all_business_state(&mut tx).await,
            "upgrade changed public rows"
        );
        compose(&mut tx).await.unwrap();
        final_profile(&mut tx, observer).await;
        assert!(
            same_metadata(&after, &complete_metadata(&mut tx).await),
            "replay wrote metadata"
        );
        assert!(
            before_rows == all_business_state(&mut tx).await,
            "replay wrote public rows"
        );
        if observer {
            tx.rollback().await.unwrap();
            let mut restored = pool.begin().await.unwrap();
            prior(&mut restored, false).await;
            assert!(same_metadata(
                &original_meta,
                &complete_metadata(&mut restored).await
            ));
            assert!(original_rows == all_business_state(&mut restored).await);
            restored.rollback().await.unwrap();
        } else {
            tx.commit().await.unwrap();
            let mut reopened = pool.begin().await.unwrap();
            final_profile(&mut reopened, false).await;
            assert!(same_metadata(
                &after,
                &complete_metadata(&mut reopened).await
            ));
            assert!(before_rows == all_business_state(&mut reopened).await);
            compose(&mut reopened).await.unwrap();
            assert!(same_metadata(
                &after,
                &complete_metadata(&mut reopened).await
            ));
            assert!(before_rows == all_business_state(&mut reopened).await);
            reopened.commit().await.unwrap();
            if let Some((_, config, _, account, cookies)) = &existing {
                let app = AppState::from_config(config.clone()).await.unwrap();
                let service = build_router(app.clone());
                let me = request(&service, "GET", "/api/v2/accounts/me", cookies, None, &[]).await;
                projection(&me.json(StatusCode::OK), account.account, pool).await;
                me.private();
                app.shutdown_realtime().await;
            }
        }
        drop(existing);
    }

    #[sqlx::test(migrations = false)]
    async fn platform_source_fresh_plain_upgrade_and_replay(pool: PgPool) {
        upgrade_case(&pool, false, false).await;
    }
    #[sqlx::test(migrations = false)]
    async fn platform_source_fresh_observer_upgrade_and_rollback(pool: PgPool) {
        upgrade_case(&pool, false, true).await;
    }
    #[sqlx::test(migrations = false)]
    async fn platform_source_populated_plain_upgrade_reopens_original_session(pool: PgPool) {
        upgrade_case(&pool, true, false).await;
    }
    #[sqlx::test(migrations = false)]
    async fn platform_source_populated_observer_upgrade_preserves_history(pool: PgPool) {
        upgrade_case(&pool, true, true).await;
    }

    fn exact_error(error: sqlx::Error, code: &str, message: &str) {
        assert!(
            error
                .as_database_error()
                .is_some_and(|e| e.code().as_deref() == Some(code) && e.message() == message),
            "unexpected SQL refusal category"
        );
    }

    async fn finalized_corruptions(pool: &PgPool, observer: bool) {
        let existing = company_predecessor(pool, true).await;
        let mut tx = pool.begin().await.unwrap();
        operator(&mut tx).await;
        if observer {
            observer_fixture(&mut tx).await;
        }
        compose(&mut tx).await.unwrap();
        final_profile(&mut tx, observer).await;
        lock_metadata_relations(&mut tx).await;
        let clean = complete_metadata(&mut tx).await;
        let clean_rows = all_business_state(&mut tx).await;
        let body = INSTALL.split("$body$").nth(1).unwrap();
        let wrong_body = INSTALL
            .replacen("CREATE FUNCTION", "CREATE OR REPLACE FUNCTION", 1)
            .replacen(body, "BEGIN RETURN; END\n", 1);
        let statements = [
            "GRANT EXECUTE ON FUNCTION public.auth_legacy_platform_source_material_v1(uuid,uuid) TO PUBLIC",
            "GRANT EXECUTE ON FUNCTION public.auth_legacy_platform_source_material_v1(uuid,uuid) TO console_auth_rt",
            "GRANT EXECUTE ON FUNCTION public.auth_legacy_platform_source_material_v1(uuid,uuid) TO console_rt WITH GRANT OPTION",
            "ALTER FUNCTION public.auth_legacy_platform_source_material_v1(uuid,uuid) OWNER TO console_app",
            "ALTER FUNCTION public.auth_legacy_platform_source_material_v1(uuid,uuid) SET search_path=public,pg_catalog",
            "ALTER FUNCTION public.auth_legacy_platform_source_material_v1(uuid,uuid) STABLE",
            "CREATE FUNCTION public.auth_legacy_platform_source_material_v1(text) RETURNS boolean LANGUAGE sql AS 'SELECT false'",
            "DROP FUNCTION public.auth_legacy_platform_source_material_v1(uuid,uuid) RESTRICT",
            "REVOKE EXECUTE ON FUNCTION public.auth_legacy_session_context_v1(uuid,uuid) FROM console_credential_owner",
            "GRANT console_credential_owner TO console_auth_rt WITH ADMIN FALSE, INHERIT TRUE, SET TRUE",
            "ALTER DEFAULT PRIVILEGES FOR ROLE console_credential_owner GRANT EXECUTE ON FUNCTIONS TO console_auth_rt",
            wrong_body.as_str(),
        ];
        for (index, statement) in statements.into_iter().enumerate() {
            let mut fault = tx.begin().await.unwrap();
            sqlx::raw_sql(sqlx::AssertSqlSafe(statement))
                .execute(&mut *fault)
                .await
                .unwrap();
            let corrupt = complete_metadata(&mut fault).await;
            assert!(
                corrupt != clean,
                "fault {index} must actually change metadata"
            );
            assert_eq!(
                state(&mut fault).await,
                "account_native.profile_mismatch",
                "fault {index}"
            );
            let corrupt_rows = all_business_state(&mut fault).await;
            let mut attempt = fault.begin().await.unwrap();
            let result = compose(&mut attempt).await;
            attempt.rollback().await.unwrap();
            exact_error(
                result.expect_err("finalizer repaired or resealed corrupt profile"),
                "P0001",
                "account_native.profile_mismatch",
            );
            assert!(same_metadata(
                &corrupt,
                &complete_metadata(&mut fault).await
            ));
            assert!(corrupt_rows == all_business_state(&mut fault).await);
            fault.rollback().await.unwrap();
            assert!(same_metadata(&clean, &complete_metadata(&mut tx).await));
            assert!(clean_rows == all_business_state(&mut tx).await);
            final_profile(&mut tx, observer).await;
        }
        tx.rollback().await.unwrap();
        drop(existing);
    }

    #[sqlx::test(migrations = false)]
    async fn platform_source_plain_corrupt_profiles_refuse_without_reseal(pool: PgPool) {
        finalized_corruptions(&pool, false).await;
    }
    #[sqlx::test(migrations = false)]
    async fn platform_source_observer_corrupt_profiles_refuse_without_reseal(pool: PgPool) {
        finalized_corruptions(&pool, true).await;
    }

    async fn interrupted_upgrade(pool: &PgPool, observer: bool) {
        let existing = company_predecessor(pool, true).await;
        let mut tx = pool.begin().await.unwrap();
        operator(&mut tx).await;
        if observer {
            observer_fixture(&mut tx).await;
        }
        prior(&mut tx, observer).await;
        lock_metadata_relations(&mut tx).await;
        let before = complete_metadata(&mut tx).await;
        let before_rows = all_business_state(&mut tx).await;
        for signature in ["text", "uuid", "uuid,uuid"] {
            let mut fault = tx.begin().await.unwrap();
            let ddl = format!(
                "CREATE FUNCTION public.{BRIDGE}({signature}) RETURNS boolean LANGUAGE sql AS 'SELECT false'"
            );
            sqlx::raw_sql(sqlx::AssertSqlSafe(ddl))
                .execute(&mut *fault)
                .await
                .unwrap();
            assert_eq!(state(&mut fault).await, "account_native.profile_mismatch");
            let malformed = complete_metadata(&mut fault).await;
            let mut attempt = fault.begin().await.unwrap();
            let result = compose(&mut attempt).await;
            attempt.rollback().await.unwrap();
            exact_error(
                result.expect_err("unknown helper was repaired"),
                "P0001",
                "account_native.profile_mismatch",
            );
            assert!(malformed == complete_metadata(&mut fault).await);
            assert!(before_rows == all_business_state(&mut fault).await);
            fault.rollback().await.unwrap();
            assert!(same_metadata(&before, &complete_metadata(&mut tx).await));
        }
        for (tag, condition, marker) in [
            (
                "CREATE FUNCTION",
                "to_regprocedure('public.auth_legacy_platform_source_material_v1(uuid,uuid)') IS NOT NULL",
                "PLATFORM_SOURCE_TEST_CREATED",
            ),
            (
                "GRANT",
                "to_regprocedure('public.auth_legacy_platform_source_material_v1(uuid,uuid)') IS NOT NULL AND has_function_privilege('console_credential_owner','public.auth_legacy_session_context_v1(uuid,uuid)','EXECUTE') AND has_function_privilege('console_rt','public.auth_legacy_platform_source_material_v1(uuid,uuid)','EXECUTE')",
                "PLATFORM_SOURCE_TEST_CONTEXT_GRANT",
            ),
        ] {
            let mut fault = tx.begin().await.unwrap();
            let ddl = format!(
                "CREATE FUNCTION public.platform_source_test_interrupt() RETURNS event_trigger LANGUAGE plpgsql AS $hook$ BEGIN IF {condition} THEN RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='{marker}'; END IF; END $hook$; CREATE EVENT TRIGGER platform_source_test_interrupt ON ddl_command_end WHEN TAG IN ('{tag}') EXECUTE FUNCTION public.platform_source_test_interrupt()"
            );
            sqlx::raw_sql(sqlx::AssertSqlSafe(ddl))
                .execute(&mut *fault)
                .await
                .unwrap();
            prior(&mut fault, observer).await;
            let instrumented = complete_metadata(&mut fault).await;
            let mut attempt = fault.begin().await.unwrap();
            let result = compose(&mut attempt).await;
            attempt.rollback().await.unwrap();
            exact_error(
                result.expect_err("actual finalizer interruption not reached"),
                "P0001",
                marker,
            );
            assert!(
                same_metadata(&instrumented, &complete_metadata(&mut fault).await),
                "partial DDL or ACL escaped rollback"
            );
            assert!(before_rows == all_business_state(&mut fault).await);
            prior(&mut fault, observer).await;
            fault.rollback().await.unwrap();
            assert!(same_metadata(&before, &complete_metadata(&mut tx).await));
        }
        compose(&mut tx).await.unwrap();
        final_profile(&mut tx, observer).await;
        exact_two_object_delta(&before, &complete_metadata(&mut tx).await);
        assert!(before_rows == all_business_state(&mut tx).await);
        tx.rollback().await.unwrap();
        drop(existing);
    }

    #[sqlx::test(migrations = false)]
    async fn platform_source_plain_partial_helpers_and_interrupted_finalizer_rollback(
        pool: PgPool,
    ) {
        interrupted_upgrade(&pool, false).await;
    }
    #[sqlx::test(migrations = false)]
    async fn platform_source_observer_partial_helpers_and_interrupted_finalizer_rollback(
        pool: PgPool,
    ) {
        interrupted_upgrade(&pool, true).await;
    }

    struct SqlFixture {
        business: PgPool,
        auth: PgPool,
        subject: Uuid,
        other: Uuid,
        first: Uuid,
        second: Uuid,
        other_family: Uuid,
    }

    async fn sql_fixture(pool: &PgPool) -> SqlFixture {
        assert!(company_predecessor(pool, false).await.is_none());
        let mut tx = pool.begin().await.unwrap();
        compose(&mut tx).await.unwrap();
        final_profile(&mut tx, false).await;
        tx.commit().await.unwrap();
        let business = login_test_pool(pool, TestDatabaseLogin::Business).await;
        let auth = login_test_pool(pool, TestDatabaseLogin::Auth).await;
        for (pool, role) in [(&business, "console_rt"), (&auth, "console_auth_rt")] {
            let identity: (String, String) =
                sqlx::query_as("SELECT session_user::text,current_user::text")
                    .fetch_one(pool)
                    .await
                    .unwrap();
            assert_eq!(identity, (role.to_owned(), role.to_owned()));
        }
        let subject = Uuid::new_v4();
        let other = Uuid::new_v4();
        for id in [subject, other] {
            sqlx::query("INSERT INTO public.users(id,display_name,roles,org_id) VALUES($1,'플랫폼 원본 사실 검증',ARRAY['SUPER_ADMIN']::text[],$2)").bind(id).bind(OrgId::platform().as_uuid()).execute(pool).await.unwrap();
            let exists: bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.accounts WHERE id=$1) AND NOT EXISTS(SELECT 1 FROM public.account_security WHERE account_id=$1)").bind(id).fetch_one(pool).await.unwrap();
            assert!(exists);
        }
        let mut families = Vec::new();
        for id in [subject, subject, other] {
            let now: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                .fetch_one(&auth)
                .await
                .unwrap();
            let mut issued = auth.begin().await.unwrap();
            let family = RefreshTokenStore
                .issue_family_in_tx(&mut issued, id, OrgId::platform(), now, Duration::hours(1))
                .await
                .unwrap();
            issued.commit().await.unwrap();
            families.push(family.family_id);
        }
        assert!(families.iter().copied().collect::<BTreeSet<_>>().len() == 3);
        SqlFixture {
            business,
            auth,
            subject,
            other,
            first: families[0],
            second: families[1],
            other_family: families[2],
        }
    }

    async fn bridge(
        pool: &PgPool,
        subject: Option<Uuid>,
        family: Option<Uuid>,
        org: &str,
        isolation: &str,
    ) -> Result<Value, sqlx::Error> {
        let mut tx = pool.begin().await.unwrap();
        let isolation = match isolation {
            "read committed" => "SET TRANSACTION ISOLATION LEVEL READ COMMITTED",
            "repeatable read" => "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ",
            "serializable" => "SET TRANSACTION ISOLATION LEVEL SERIALIZABLE",
            _ => panic!("closed test isolation roster"),
        };
        sqlx::raw_sql(isolation).execute(&mut *tx).await.unwrap();
        sqlx::query("SELECT set_config('app.current_org',$1,true)")
            .bind(org)
            .execute(&mut *tx)
            .await
            .unwrap();
        let result = sqlx::query_scalar(
            "SELECT to_jsonb(s) FROM public.auth_legacy_platform_source_material_v1($1,$2) s",
        )
        .bind(subject)
        .bind(family)
        .fetch_one(&mut *tx)
        .await;
        tx.rollback().await.unwrap();
        result
    }

    fn exact_projection(actual: &Value, expected: &Value) {
        let keys = [
            "roles",
            "family_id",
            "family_user_id",
            "family_org_id",
            "family_protocol",
            "family_created_at",
            "family_revoked_at",
            "family_account_security_generation",
            "family_auth_time",
            "family_assurance",
        ];
        assert_eq!(
            actual
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>(),
            keys.into_iter().collect()
        );
        assert!(
            actual == expected,
            "source bridge must expose only exact current requested facts"
        );
    }

    async fn expected_family(pool: &PgPool, family: Uuid, roles: &[&str]) -> Value {
        sqlx::query_scalar("SELECT jsonb_build_object('roles',$2::text[],'family_id',id,'family_user_id',user_id,'family_org_id',org_id,'family_protocol',protocol,'family_created_at',created_at,'family_revoked_at',revoked_at,'family_account_security_generation',account_security_generation,'family_auth_time',auth_time,'family_assurance',assurance) FROM public.auth_refresh_token_families WHERE id=$1")
            .bind(family).bind(roles).fetch_one(pool).await.unwrap()
    }

    fn same_owner_snapshot(before: &(Value, Value), after: &(Value, Value)) -> bool {
        let metadata_equal = same_metadata(&before.0, &after.0);
        if before.1 != after.1 {
            eprintln!("PLATFORM_SOURCE_OWNER_BUSINESS_STATE_CHANGED");
        }
        metadata_equal && before.1 == after.1
    }

    async fn owner_snapshot(pool: &PgPool) -> (Value, Value) {
        let mut tx = pool.begin().await.unwrap();
        operator(&mut tx).await;
        let result = (
            complete_metadata(&mut tx).await,
            all_business_state(&mut tx).await,
        );
        tx.rollback().await.unwrap();
        result
    }

    #[sqlx::test(migrations = false)]
    async fn platform_source_actual_login_acl_has_no_wider_read_or_write_capability(pool: PgPool) {
        let f = sql_fixture(&pool).await;
        let before = owner_snapshot(&pool).await;
        assert!(
            bridge(
                &f.business,
                Some(f.subject),
                None,
                &OrgId::platform().to_string(),
                "read committed"
            )
            .await
            .is_ok()
        );
        let error = bridge(
            &f.auth,
            Some(f.subject),
            None,
            &OrgId::platform().to_string(),
            "read committed",
        )
        .await
        .expect_err("Auth acquired Business-only source bridge");
        assert_eq!(
            error.as_database_error().and_then(|e| e.code()).as_deref(),
            Some("42501")
        );
        for sql in [
            "SELECT * FROM public.auth_refresh_token_families LIMIT 1",
            "SELECT * FROM public.auth_legacy_session_context_v1('00000000-0000-0000-0000-00000000face'::uuid,gen_random_uuid())",
            "UPDATE public.auth_refresh_token_families SET revoked_at=clock_timestamp() WHERE false",
        ] {
            let mut tx = f.business.begin().await.unwrap();
            let result = sqlx::raw_sql(sql).execute(&mut *tx).await;
            tx.rollback().await.unwrap();
            assert_eq!(
                result
                    .expect_err("Business acquired wider credential capability")
                    .as_database_error()
                    .and_then(|e| e.code())
                    .as_deref(),
                Some("42501")
            );
        }
        for sql in [
            "UPDATE public.organizations SET name=name WHERE false",
            "UPDATE public.users SET is_active=false WHERE false",
            "SET ROLE console_credential_owner",
            "SET ROLE console_rt",
        ] {
            let mut tx = f.auth.begin().await.unwrap();
            let result = sqlx::raw_sql(sql).execute(&mut *tx).await;
            tx.rollback().await.unwrap();
            assert_eq!(
                result
                    .expect_err("Auth acquired business mutation/role capability")
                    .as_database_error()
                    .and_then(|e| e.code())
                    .as_deref(),
                Some("42501")
            );
        }
        assert!(
            same_owner_snapshot(&before, &owner_snapshot(&pool).await),
            "capability probes changed metadata or rows"
        );
        f.auth.close().await;
        f.business.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn platform_source_sql_exact_current_roles_and_requested_family_projection(pool: PgPool) {
        let f = sql_fixture(&pool).await;
        assert_ne!(f.subject, f.other);
        for rs in [vec!["SUPER_ADMIN"], vec!["MEMBER"], vec!["ADMIN"]] {
            sqlx::query("UPDATE public.users SET roles=$1 WHERE id=$2")
                .bind(&rs)
                .bind(f.subject)
                .execute(&pool)
                .await
                .unwrap();
            let before = owner_snapshot(&pool).await;
            let mut absent = json!({"roles":rs});
            for key in [
                "family_id",
                "family_user_id",
                "family_org_id",
                "family_protocol",
                "family_created_at",
                "family_revoked_at",
                "family_account_security_generation",
                "family_auth_time",
                "family_assurance",
            ] {
                absent[key] = Value::Null;
            }
            exact_projection(
                &bridge(
                    &f.business,
                    Some(f.subject),
                    None,
                    &OrgId::platform().to_string(),
                    "read committed",
                )
                .await
                .unwrap(),
                &absent,
            );
            for family in [f.first, f.second, f.other_family] {
                let expected = expected_family(&pool, family, &rs).await;
                let actual = bridge(
                    &f.business,
                    Some(f.subject),
                    Some(family),
                    &OrgId::platform().to_string(),
                    "read committed",
                )
                .await
                .unwrap();
                exact_projection(&actual, &expected);
            }
            assert!(
                same_owner_snapshot(&before, &owner_snapshot(&pool).await),
                "facts reads changed state"
            );
        }
        // The bridge deliberately returns raw selected revoked facts; Auth owns
        // exact-family authority refusal and must not substitute the live sibling.
        sqlx::query("UPDATE public.auth_refresh_token_families SET revoked_at=clock_timestamp(),revoked_reason='TEST_ONLY selected family evidence' WHERE id=$1").bind(f.first).execute(&pool).await.unwrap();
        let before = owner_snapshot(&pool).await;
        let expected = expected_family(&pool, f.first, &["ADMIN"]).await;
        assert!(!expected["family_revoked_at"].is_null());
        exact_projection(
            &bridge(
                &f.business,
                Some(f.subject),
                Some(f.first),
                &OrgId::platform().to_string(),
                "read committed",
            )
            .await
            .unwrap(),
            &expected,
        );
        assert!(same_owner_snapshot(&before, &owner_snapshot(&pool).await));
        f.auth.close().await;
        f.business.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn platform_source_sql_nil_scope_isolation_missing_and_fenced_refusals(pool: PgPool) {
        let f = sql_fixture(&pool).await;
        let platform = OrgId::platform().to_string();
        let ordinary = OrgId::knl().to_string();
        let cases = [
            (
                None,
                None,
                platform.as_str(),
                "read committed",
                "22023",
                "auth_legacy_platform.invalid_identity",
            ),
            (
                Some(Uuid::nil()),
                None,
                platform.as_str(),
                "read committed",
                "22023",
                "auth_legacy_platform.invalid_identity",
            ),
            (
                Some(f.subject),
                Some(Uuid::nil()),
                platform.as_str(),
                "read committed",
                "22023",
                "auth_legacy_platform.invalid_identity",
            ),
            (
                Some(f.subject),
                None,
                "",
                "read committed",
                "42501",
                "auth_legacy.company_context_mismatch",
            ),
            (
                Some(f.subject),
                None,
                ordinary.as_str(),
                "read committed",
                "42501",
                "auth_legacy.company_context_mismatch",
            ),
            (
                Some(f.subject),
                None,
                platform.as_str(),
                "repeatable read",
                "P0001",
                "auth_legacy.unsupported_isolation",
            ),
            (
                Some(f.subject),
                None,
                platform.as_str(),
                "serializable",
                "P0001",
                "auth_legacy.unsupported_isolation",
            ),
            (
                Some(f.subject),
                Some(Uuid::new_v4()),
                platform.as_str(),
                "read committed",
                "P0002",
                "auth_legacy_platform.family_not_found",
            ),
            (
                Some(Uuid::new_v4()),
                None,
                platform.as_str(),
                "read committed",
                "P0002",
                "account_company_deactivation.subject_not_found",
            ),
        ];
        let before = owner_snapshot(&pool).await;
        for (subject, family, org, isolation, code, message) in cases {
            exact_error(
                bridge(&f.business, subject, family, org, isolation)
                    .await
                    .expect_err("invalid source SQL material accepted"),
                code,
                message,
            );
        }
        assert!(same_owner_snapshot(&before, &owner_snapshot(&pool).await));
        for (mutation, code, message) in [
            (
                "UPDATE public.users SET is_active=false WHERE id=$1",
                "28000",
                "auth_legacy_platform.source_inactive",
            ),
            (
                "UPDATE public.users SET roles=ARRAY[]::text[] WHERE id=$1",
                "28000",
                "auth_legacy.subject_has_no_roles",
            ),
            (
                "INSERT INTO public.account_security(account_id,security_state,security_generation,revision,updated_at,context_generation) VALUES($1,'PENDING_ENROLLMENT',1,1,clock_timestamp(),1)",
                "28000",
                "auth_legacy_platform.source_fenced",
            ),
        ] {
            sqlx::query("UPDATE public.users SET is_active=true,roles=ARRAY['SUPER_ADMIN']::text[] WHERE id=$1").bind(f.subject).execute(&pool).await.unwrap();
            sqlx::query(mutation)
                .bind(f.subject)
                .execute(&pool)
                .await
                .unwrap();
            let mutated = owner_snapshot(&pool).await;
            exact_error(
                bridge(
                    &f.business,
                    Some(f.subject),
                    None,
                    &platform,
                    "read committed",
                )
                .await
                .expect_err("unavailable/ineligible source accepted"),
                code,
                message,
            );
            assert!(same_owner_snapshot(&mutated, &owner_snapshot(&pool).await));
        }
        f.auth.close().await;
        f.business.close().await;
    }

    fn oracle_fixture() -> (Value, Value) {
        // Minimal metadata fixture for the delta oracle, not a substitute for
        // checked_bridge's real pg_proc signature and effective-privilege SQL.
        let context = json!({"oid":"100","proname":CONTEXT,"proowner":"10",
            "proargtypes":["2950","2950"],"prosrc":"retained exact context",
            "proacl":["console_account_owner=X/console_account_owner","console_auth_rt=X/console_account_owner"]});
        let before = json!({"functions":[context,{"oid":"101","proname":"unrelated","prosrc":"retained"}],
            "roles":[{"oid":"10","rolname":"console_account_owner"},{"oid":"11","rolname":"console_credential_owner"}],
            "function_acls":{"100":[["console_account_owner","console_account_owner","EXECUTE",false],["console_account_owner","console_auth_rt","EXECUTE",false]],"101":[]},
            "relations":[{"name":"users","rls":true}],"memberships":[],"default_acls":[],"ledger":[{"version":228}],"event_triggers":[]});
        let mut after = before.clone();
        after["functions"][0]["proacl"]
            .as_array_mut()
            .unwrap()
            .push(json!("console_credential_owner=X/console_account_owner"));
        after["function_acls"]["100"]
            .as_array_mut()
            .unwrap()
            .push(json!([
                "console_account_owner",
                "console_credential_owner",
                "EXECUTE",
                false
            ]));
        after["functions"].as_array_mut().unwrap().push(
            json!({"oid":"102","proname":BRIDGE,"prosrc":INSTALL.split("$body$").nth(1).unwrap()}),
        );
        after["function_acls"]["102"] = json!([
            [
                "console_credential_owner",
                "console_credential_owner",
                "EXECUTE",
                false
            ],
            ["console_credential_owner", "console_rt", "EXECUTE", false]
        ]);
        (before, after)
    }

    #[test]
    fn platform_source_delta_oracle_accepts_exact_pair_without_mutating_evidence() {
        let (before, after) = oracle_fixture();
        let retained = (before.clone(), after.clone());
        exact_two_object_delta(&before, &after);
        assert!((before, after) == retained);
    }

    #[test]
    fn platform_source_delta_oracle_rejects_omissions_and_unrelated_corruption() {
        let (before, after) = oracle_fixture();
        for corruption in 0..20 {
            let mut bad = after.clone();
            match corruption {
                0 => {
                    bad["functions"].as_array_mut().unwrap().pop();
                }
                1 => {
                    bad["functions"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!({"oid":"103","proname":"unexpected"}));
                }
                2 => {
                    bad["functions"][2]["prosrc"] = json!("BEGIN RETURN; END");
                }
                3 => {
                    bad["functions"][0]["prosrc"] = json!("changed context");
                }
                4 => {
                    bad["functions"][0]["proowner"] = json!("11");
                }
                5 => {
                    bad["functions"][0]["proargtypes"] = json!(["25", "2950"]);
                }
                6 => {
                    bad["functions"][0]["proacl"] = before["functions"][0]["proacl"].clone();
                }
                7 => {
                    bad["function_acls"]["100"] = before["function_acls"]["100"].clone();
                }
                8 => {
                    bad["function_acls"]["100"][2][3] = json!(true);
                }
                9 => {
                    bad["function_acls"]["100"][2][0] = json!("console_credential_owner");
                }
                10 => {
                    bad["function_acls"]["100"]
                        .as_array_mut()
                        .unwrap()
                        .remove(1);
                }
                11 => {
                    bad["function_acls"]["102"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!([
                            "console_credential_owner",
                            "PUBLIC",
                            "EXECUTE",
                            false
                        ]));
                }
                12 => {
                    bad["function_acls"]["102"][1][3] = json!(true);
                }
                13 => {
                    bad["functions"][1]["prosrc"] = json!("unrelated drift");
                }
                14 => {
                    bad["relations"][0]["rls"] = json!(false);
                }
                15 => {
                    bad["roles"][1]["rolsuper"] = json!(true);
                }
                16 => {
                    bad["memberships"] = json!([{"roleid":"11","member":"99"}]);
                }
                17 => {
                    bad["default_acls"] = json!(["PUBLIC=EXECUTE"]);
                }
                18 => {
                    bad["ledger"] = json!([]);
                }
                19 => {
                    bad["event_triggers"] = json!([{"evtenabled":"D"}]);
                }
                _ => unreachable!(),
            }
            assert!(
                std::panic::catch_unwind(|| exact_two_object_delta(&before, &bad)).is_err(),
                "delta oracle missed corruption {corruption}"
            );
        }
        exact_two_object_delta(&before, &after);
    }

    #[test]
    fn platform_source_frozen_fixture_and_historical_migration_identity() {
        for (source, expected) in FROZEN_COMPANY {
            assert_eq!(hex::encode(Sha256::digest(source)), *expected);
        }
        for (source, expected) in [
            (
                include_str!("fixtures/platform-source-historical.sql"),
                "74c8e2a912e75b3b4e2dc85ca6c1acb4893fd513483ee1d83ede14ba88ff2f46",
            ),
            (
                include_str!("fixtures/platform-source-prospective.sql"),
                "5ae32e8583ed7a8072f33a1fc82c4ad7c9f89725d67137c60919b7d210e827ac",
            ),
            (
                include_str!("fixtures/platform-source-metadata.sql"),
                "8752adaad8ccae80c6e4e88624c61b826610d1f6fa4522fc277b9e8225aa65c8",
            ),
            (
                include_str!("fixtures/platform-source-checked.sql"),
                "d217984b3a0c6a94dc3552ef5c124f54b8d62ac3b4c004a6e664aa3c993488a2",
            ),
            (
                include_str!("fixtures/platform-source-install.sql"),
                "6965be5e8d2b873bcbb4f49d72399850ef3c58dfaabd3c4dfdb56877b178eb2a",
            ),
        ] {
            assert_eq!(hex::encode(Sha256::digest(source)), expected);
        }
        for (path, expected) in [
            (
                "backend/crates/platform/db/migrations/0226_account_terms_catalog.sql",
                "3f03683d7f117cf094c58e1230283de1dfa816be33de5573aa55e761985a8afc",
            ),
            (
                "backend/crates/platform/db/migrations/0227_native_account_identity_catalog.sql",
                "0a3f709b31ee0ce4378d17f2ba2f16596b2c19ff2489d86c3dbdd2fb63712206",
            ),
            (
                "backend/crates/platform/db/migrations/0228_deployment_operator_catalog.sql",
                "dae02e362e71ebea609621ec0c6377bcbea69d146e4a9d62caa4ed0379cb1148",
            ),
        ] {
            assert_eq!(
                hex::encode(Sha256::digest(artifact(path))),
                expected,
                "historical migration bytes changed"
            );
        }
    }

    include!("audit_platform_source_predecessor.rs");
}
