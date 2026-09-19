// Included inside native_business_session; existing fixtures/owners only.
mod catalog {
    use super::super::audit_account_transition::metadata;
    use super::*;
    use sqlx::Acquire;

    const OUTER: &str = "public.account_session_shared_material_v1(uuid,uuid)";
    const INNER: &str = "public.auth_account_session_shared_material_v1(uuid,uuid)";
    const NAMES: &[&str] = &[
        "account_session_shared_material_v1",
        "auth_account_session_shared_material_v1",
    ];
    const FAMILY_FIELDS: &[(&str, &str)] = &[
        ("user_id", "uuid"),
        ("protocol", "text"),
        ("account_security_generation", "bigint"),
        ("auth_time", "timestamp with time zone"),
        ("assurance", "text"),
        ("created_at", "timestamp with time zone"),
        ("revoked_at", "timestamp with time zone"),
        ("org_id", "uuid"),
    ];
    const SECURITY_FIELDS: &[(&str, &str)] = &[
        ("security_state", "text"),
        ("security_generation", "bigint"),
        ("revision", "bigint"),
        ("context_generation", "bigint"),
    ];

    async fn required(pool: &PgPool) {
        let identities: Vec<String> = sqlx::query_scalar("SELECT n.nspname||'.'||p.proname||'('||replace(oidvectortypes(p.proargtypes),' ','')||')' FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname='public' AND p.proname=ANY($1) ORDER BY p.proname")
            .bind(NAMES).fetch_all(pool).await.unwrap();
        assert!(
            identities == [OUTER.to_owned(), INNER.to_owned()],
            "SESSION_CATALOG_PREREQUISITE: exact two UUID helpers required; deeper owner/fault assertions unreached"
        );
    }

    async fn classify(connection: &mut PgConnection) -> (String, String) {
        sqlx::raw_sql(include_str!("../../src/account_custody_session.sql"))
            .execute(&mut *connection)
            .await
            .unwrap();
        let root = sqlx::query_scalar(include_str!("../../src/account_custody_state.sql"))
            .fetch_one(&mut *connection)
            .await
            .unwrap();
        let credential = sqlx::query_scalar(include_str!(
            "../../src/account_credential_custody_state.sql"
        ))
        .fetch_one(&mut *connection)
        .await
        .unwrap();
        (root, credential)
    }

    fn finalized(observed: (String, String)) {
        assert!(
            observed
                == (
                    "account_custody.native_finalized".into(),
                    "account_credentials.native_finalized".into()
                ),
            "real generated classifiers must admit the exact positive profile"
        );
    }

    fn expected_shape(outer: bool) -> Value {
        let owner = if outer {
            "console_account_owner"
        } else {
            "console_credential_owner"
        };
        let executors = if outer {
            vec!["console_account_owner", "console_auth_rt", "console_rt"]
        } else {
            vec!["console_account_owner", "console_credential_owner"]
        };
        let fields: Vec<_> = if outer {
            SECURITY_FIELDS
                .iter()
                .chain(FAMILY_FIELDS)
                .copied()
                .collect()
        } else {
            FAMILY_FIELDS.to_vec()
        };
        let mut names = vec!["p_account", "p_family"];
        let mut kinds = vec!["uuid", "uuid"];
        let mut modes = vec!["i", "i"];
        for (name, kind) in fields {
            names.push(name);
            kinds.push(kind);
            modes.push("t");
        }
        json!({"owner":owner,"language":"plpgsql","kind":"f","definer":true,
            "strict":false,"leakproof":false,"parallel":"u","volatility":"v",
            "returns_set":true,"return_record":true,"defaults":0,"input_types":"uuid, uuid",
            "arg_names":names,"arg_modes":modes,"all_types":kinds,
            "config":["search_path=pg_catalog, pg_temp","row_security=on"],
            "acl":executors.into_iter().map(|role| json!([owner,role,"EXECUTE",false])).collect::<Vec<_>>()})
    }

    async fn actual_denial(
        runtime: &PgPool,
        statement: &'static str,
        account: Uuid,
        family: Uuid,
        function: &str,
    ) {
        let mut tx = runtime.begin().await.unwrap();
        let result = sqlx::query(statement)
            .bind(account)
            .bind(family)
            .fetch_all(&mut *tx)
            .await;
        tx.rollback().await.unwrap();
        let Err(error) = result else {
            panic!("forbidden executor reached session material");
        };
        let database = error
            .as_database_error()
            .expect("actual SQL permission refusal");
        assert!(
            database.code().as_deref() == Some("42501")
                && database.message() == format!("permission denied for function {function}"),
            "exact EXECUTE refusal required, not a body's unrelated error"
        );
    }

    #[sqlx::test(migrations = false)]
    async fn shared_session_catalog_has_exact_shape_acl_and_executor_chain(pool: PgPool) {
        let (app, key) = signed_fixture(&pool).await;
        let (attempt, cookies) = enrolled(&app).await;
        required(&pool).await;
        let before = inventory(&pool, None).await;
        let runtime = business(&pool).await;
        let auth = logout_auth_pool(&pool).await;
        let (verifier, ttl) = verification(&app, &key);
        let mut tx = runtime.begin().await.unwrap();
        let session = current(&mut tx, &verifier, &cookies.0[ACCESS], ttl).await;
        tx.commit().await.unwrap();
        for (signature, outer) in [(OUTER, true), (INNER, false)] {
            let shape: Value = sqlx::query_scalar(r#"SELECT jsonb_build_object(
                'owner',pg_get_userbyid(p.proowner),'language',l.lanname,'kind',p.prokind::text,
                'definer',p.prosecdef,'strict',p.proisstrict,'leakproof',p.proleakproof,
                'parallel',p.proparallel::text,'volatility',p.provolatile::text,
                'returns_set',p.proretset,'return_record',p.prorettype='record'::regtype,
                'defaults',p.pronargdefaults,'input_types',oidvectortypes(p.proargtypes),
                'arg_names',p.proargnames,
                'arg_modes',ARRAY(SELECT mode::text FROM unnest(p.proargmodes) WITH ORDINALITY t(mode,ordinal) ORDER BY ordinal),
                'all_types',ARRAY(SELECT format_type(kind,NULL) FROM unnest(p.proallargtypes) WITH ORDINALITY t(kind,ordinal) ORDER BY ordinal),
                'config',p.proconfig,
                'acl',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable) ORDER BY CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable) FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a))
                FROM pg_proc p JOIN pg_language l ON l.oid=p.prolang WHERE p.oid=to_regprocedure($1)"#)
                .bind(signature).fetch_one(&pool).await.unwrap();
            assert!(
                shape == expected_shape(outer),
                "exact helper metadata/ACL contract differs"
            );
            let effective: Vec<(String, bool, bool)> = sqlx::query_as("SELECT rolname::text,has_function_privilege(oid,to_regprocedure($1),'EXECUTE'),has_function_privilege(oid,to_regprocedure($1),'EXECUTE WITH GRANT OPTION') FROM pg_roles WHERE NOT rolsuper ORDER BY rolname")
                .bind(signature).fetch_all(&pool).await.unwrap();
            assert!(!effective.is_empty());
            for (role, execute, grantable) in effective {
                let owner = if outer {
                    "console_account_owner"
                } else {
                    "console_credential_owner"
                };
                let permitted = if outer {
                    ["console_account_owner", "console_auth_rt", "console_rt"]
                        .contains(&role.as_str())
                } else {
                    ["console_account_owner", "console_credential_owner"].contains(&role.as_str())
                };
                assert!(
                    execute == permitted && grantable == (role == owner),
                    "unexpected effective helper execution/delegation"
                );
            }
        }
        for consumer in [&auth, &runtime] {
            let mut tx = consumer.begin().await.unwrap();
            let count: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM public.account_session_shared_material_v1($1,$2)",
            )
            .bind(attempt.account)
            .bind(session.session_id)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
            assert_eq!(count, 1, "actual outer projection positive control");
            tx.rollback().await.unwrap();
            actual_denial(
                consumer,
                "SELECT * FROM public.auth_account_session_shared_material_v1($1,$2)",
                attempt.account,
                session.session_id,
                "auth_account_session_shared_material_v1",
            )
            .await;
        }
        // Explicit owner-chain fixture, never a browser identity or grant.
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql("SET LOCAL ROLE console_account_owner")
            .execute(&mut *tx)
            .await
            .unwrap();
        let owner: String = sqlx::query_scalar("SELECT current_user::text")
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        assert!(owner == "console_account_owner");
        let count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM public.auth_account_session_shared_material_v1($1,$2)",
        )
        .bind(attempt.account)
        .bind(session.session_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert_eq!(count, 1);
        let direct: bool = sqlx::query_scalar("SELECT has_table_privilege(current_user,'public.auth_refresh_token_families','SELECT') OR has_any_column_privilege(current_user,'public.auth_refresh_token_families','SELECT')")
            .fetch_one(&mut *tx).await.unwrap();
        assert!(!direct, "Account owner must not receive credential SELECT");
        let result = sqlx::query("SELECT * FROM public.auth_refresh_token_families WHERE false")
            .fetch_all(&mut *tx)
            .await;
        tx.rollback().await.unwrap();
        let Err(error) = result else {
            panic!("Account owner unexpectedly read credential table");
        };
        assert!(
            error
                .as_database_error()
                .is_some_and(|e| e.code().as_deref() == Some("42501"))
        );
        assert!(before == inventory(&pool, None).await);
        runtime.close().await;
        auth.close().await;
    }

    const FAULTS: &[(&str, &str, &str)] = &[
        (
            "missing_inner",
            "DROP FUNCTION public.auth_account_session_shared_material_v1(uuid,uuid)",
            "SELECT to_regprocedure('public.auth_account_session_shared_material_v1(uuid,uuid)') IS NULL",
        ),
        (
            "missing_outer",
            "DROP FUNCTION public.account_session_shared_material_v1(uuid,uuid)",
            "SELECT to_regprocedure('public.account_session_shared_material_v1(uuid,uuid)') IS NULL",
        ),
        (
            "inner_wrong_owner",
            "ALTER FUNCTION public.auth_account_session_shared_material_v1(uuid,uuid) OWNER TO console_app",
            "SELECT proowner='console_app'::regrole FROM pg_proc WHERE oid='public.auth_account_session_shared_material_v1(uuid,uuid)'::regprocedure",
        ),
        (
            "outer_search_path",
            "ALTER FUNCTION public.account_session_shared_material_v1(uuid,uuid) SET search_path=public,pg_catalog",
            "SELECT proconfig @> ARRAY['search_path=public, pg_catalog']::text[] FROM pg_proc WHERE oid='public.account_session_shared_material_v1(uuid,uuid)'::regprocedure",
        ),
        (
            "inner_public_execute",
            "GRANT EXECUTE ON FUNCTION public.auth_account_session_shared_material_v1(uuid,uuid) TO PUBLIC",
            "SELECT EXISTS(SELECT 1 FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a WHERE p.oid='public.auth_account_session_shared_material_v1(uuid,uuid)'::regprocedure AND a.grantee=0 AND a.privilege_type='EXECUTE')",
        ),
        (
            "inner_business_execute",
            "GRANT EXECUTE ON FUNCTION public.auth_account_session_shared_material_v1(uuid,uuid) TO console_rt",
            "SELECT has_function_privilege('console_rt','public.auth_account_session_shared_material_v1(uuid,uuid)','EXECUTE')",
        ),
        (
            "outer_business_grant_option",
            "GRANT EXECUTE ON FUNCTION public.account_session_shared_material_v1(uuid,uuid) TO console_rt WITH GRANT OPTION",
            "SELECT has_function_privilege('console_rt','public.account_session_shared_material_v1(uuid,uuid)','EXECUTE WITH GRANT OPTION')",
        ),
        (
            "outer_body",
            r#"DO $probe$ DECLARE body text; definition text; BEGIN
            SELECT prosrc,pg_get_functiondef(oid) INTO STRICT body,definition FROM pg_proc WHERE oid='public.account_session_shared_material_v1(uuid,uuid)'::regprocedure;
            IF length(body)=0 OR length(definition)-length(replace(definition,body,''))<>length(body) THEN RAISE EXCEPTION 'session_catalog.body_probe_unavailable'; END IF;
            EXECUTE replace(definition,body,body || E'\n-- session_catalog_body_probe\n'); END $probe$"#,
            "SELECT right(prosrc,length(E'\n-- session_catalog_body_probe\n'))=E'\n-- session_catalog_body_probe\n' FROM pg_proc WHERE oid='public.account_session_shared_material_v1(uuid,uuid)'::regprocedure",
        ),
    ];

    #[sqlx::test(migrations = false)]
    async fn shared_session_profile_refuses_exact_metadata_drift_and_preserves_rollback(
        pool: PgPool,
    ) {
        let (app, _) = signed_fixture(&pool).await;
        enrolled(&app).await;
        required(&pool).await;
        let before_rows = inventory(&pool, None).await;
        let mut tx = pool.begin().await.unwrap();
        finalized(classify(&mut tx).await);
        let before = metadata(&mut tx).await;
        let mut executed = 0;
        for &(name, mutation, witness) in FAULTS {
            let mut fault = tx.begin().await.unwrap();
            sqlx::raw_sql(sqlx::AssertSqlSafe(mutation))
                .execute(&mut *fault)
                .await
                .unwrap();
            let changed: bool = sqlx::query_scalar(sqlx::AssertSqlSafe(witness))
                .fetch_one(&mut *fault)
                .await
                .unwrap();
            assert!(changed, "declared catalog fault did not happen: {name}");
            // These are actual serving classifier statements in the fault's
            // transaction. A separate LOGIN cannot see uncommitted DDL.
            let observed = classify(&mut fault).await;
            assert!(
                observed
                    == (
                        "account_native.profile_mismatch".into(),
                        "account_native.profile_mismatch".into()
                    ),
                "generated classifiers admitted session helper drift: {name}"
            );
            fault.rollback().await.unwrap();
            finalized(classify(&mut tx).await);
            assert!(
                before == metadata(&mut tx).await,
                "catalog fault rollback changed retained metadata"
            );
            executed += 1;
        }
        assert_eq!(executed, 8);
        tx.rollback().await.unwrap();
        assert!(before_rows == inventory(&pool, None).await);
    }

    #[sqlx::test(migrations = false)]
    async fn shared_session_foreign_overloads_refuse_actual_startup_then_recover(pool: PgPool) {
        let (app, key) = signed_fixture(&pool).await;
        let (_, cookies) = enrolled(&app).await;
        required(&pool).await;
        let before_rows = inventory(&pool, None).await;
        let mut observer = pool.acquire().await.unwrap();
        let before = metadata(&mut observer).await;
        for name in NAMES {
            // Fixed declared names only. Foreign owner + SECURITY INVOKER is
            // deliberately outside an owner-only / definer-only routine census.
            let ddl = format!(
                "CREATE FUNCTION public.{name}(p_account text,p_family text) RETURNS boolean LANGUAGE sql SECURITY INVOKER AS 'SELECT false'; ALTER FUNCTION public.{name}(text,text) OWNER TO console_app; REVOKE ALL ON FUNCTION public.{name}(text,text) FROM PUBLIC"
            );
            sqlx::raw_sql(sqlx::AssertSqlSafe(ddl))
                .execute(&pool)
                .await
                .unwrap();
            let added = format!("public.{name}(text,text)");
            let witness: bool = sqlx::query_scalar("SELECT proowner='console_app'::regrole AND NOT prosecdef AND oidvectortypes(proargtypes)='text, text' FROM pg_proc WHERE oid=to_regprocedure($1)")
                .bind(&added).fetch_one(&pool).await.unwrap();
            assert!(witness, "foreign overload fixture must be real");
            let fault = metadata(&mut observer).await;
            let result = AppState::from_config(account_browser_config(
                &pool,
                app._artifacts.root.clone(),
                &key,
            ))
            .await;
            if let Ok(state) = &result {
                state.shutdown_realtime().await;
            }
            let unchanged = fault == metadata(&mut observer).await;
            // Remove only the exact disposable overload this case created, even
            // when admission unexpectedly succeeded. Never modify real owners.
            let cleanup = format!("DROP FUNCTION public.{name}(text,text)");
            sqlx::raw_sql(sqlx::AssertSqlSafe(cleanup))
                .execute(&pool)
                .await
                .unwrap();
            assert!(
                matches!(result, Err(console_app::AppError::Config(ref code)) if code == "account_native.profile_mismatch"),
                "actual startup must refuse foreign session overload with exact profile error"
            );
            assert!(unchanged, "startup repaired or changed hostile metadata");
            assert!(before == metadata(&mut observer).await);
            assert!(before_rows == inventory(&pool, None).await);
            let recovered = AppState::from_config(account_browser_config(
                &pool,
                app._artifacts.root.clone(),
                &key,
            ))
            .await
            .unwrap_or_else(|_| {
                panic!("same real configuration must start after exact overload removal")
            });
            recovered.shutdown_realtime().await;
        }
        // Original enrolled identity remains usable after both real restarts.
        let (verifier, ttl) = verification(&app, &key);
        let runtime = business(&pool).await;
        let mut tx = runtime.begin().await.unwrap();
        current(&mut tx, &verifier, &cookies.0[ACCESS], ttl).await;
        tx.commit().await.unwrap();
        assert!(before_rows == inventory(&pool, None).await);
        runtime.close().await;
    }
}
