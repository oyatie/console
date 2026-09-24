// Include inside auth_rest::account_browser. Real frozen228 producer plus
// existing HTTP/WebAuthn fixture seam; no alternate custody implementation.
mod audit_account_transition {
    use super::*;
    use console_kernel_core::{
        AuditAction, AuditClassification, AuditEvent, AuditRequestContext, TraceContext,
    };
    use console_platform_db::insert_audit_event;
    use console_platform_test_support::login_test_pool;
    use sqlx::PgConnection;

    pub(super) const TABLES: &[&str] = &[
        "accounts",
        "account_security",
        "account_security_events",
        "account_terms_acceptances",
        "account_terms_head",
        "account_terms_release_receipts",
        "auth_webauthn_credentials",
        "auth_webauthn_ceremonies",
        "auth_refresh_token_families",
        "auth_refresh_tokens",
        "auth_bootstrap_credentials",
        "auth_webauthn_ceremony_bindings",
        "auth_device_login_handoffs",
        "company_actors",
        "account_context_candidates",
        "deployment_operator_receipts",
        "deployment_operator_head",
        "users",
        "organizations",
        "groups",
        "group_memberships",
        "group_role_grants",
        "audit_events",
        "_sqlx_migrations",
    ];
    const PRIOR: &[(&str, &str)] = &[
        (
            include_str!("fixtures/account-native-deployment228-root-6c884418.sql"),
            "d7b5c26c6e742fdb48c956339420f4761437e52a748a5d5b8c7663807e455cf2",
        ),
        (
            include_str!("fixtures/account-native-deployment228-credentials-6c884418.sql"),
            "9d86424fda1d042d0bd65db8d776d196f2cb5b4723ce05ed17cdd0a6d83b5a79",
        ),
        (
            include_str!("fixtures/account-native-deployment228-verify-6c884418.sql"),
            "4bb128e1c591871db8ebb0d3aaa7133e87ef515caa03a46fe100de1b9c3193fa",
        ),
    ];
    const PRIOR_ROOT: &str =
        include_str!("fixtures/account_custody_state-deployment228-6c884418.sql");
    const PRIOR_CREDENTIAL: &str =
        include_str!("fixtures/account_credential_custody_state-deployment228-6c884418.sql");

    fn artifact(path: &str) -> String {
        std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(path),
        )
        .expect("AUDIT_TRANSITION_ARTIFACT_PREREQUISITE: actual generated owner SQL")
    }

    pub(super) async fn operator(connection: &mut PgConnection) {
        let actual: (String, String, bool) = sqlx::query_as(
            "SELECT session_user::text,current_user::text,current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user)"
        ).fetch_one(&mut *connection).await.unwrap();
        assert_eq!(
            actual,
            (
                "console_buck_admin".into(),
                "console_buck_admin".into(),
                true
            )
        );
        sqlx::raw_sql("SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL statement_timeout='60s'; SET LOCAL lock_timeout='5s'; SET LOCAL jit=off")
            .execute(connection).await.unwrap();
    }

    async fn prior_state(connection: &mut PgConnection) {
        for (query, digest, expected) in [
            (
                PRIOR_ROOT,
                "0f5b3b817cc505c73773b172b289f0cd4076347f82feada3b3c09deb4d86e61f",
                "account_custody.native_finalized",
            ),
            (
                PRIOR_CREDENTIAL,
                "dbdedb13e899c4bea7f92a1b7e3e03b120a0f1950748519c531f689088aae7b8",
                "account_credentials.native_finalized",
            ),
        ] {
            assert_eq!(
                hex::encode(Sha256::digest(query)),
                digest,
                "frozen228 classifier source"
            );
            let state: String = sqlx::query_scalar(query)
                .fetch_one(&mut *connection)
                .await
                .unwrap();
            assert_eq!(state, expected, "genuine complete228 predecessor");
        }
    }

    pub(super) async fn prior228(pool: &PgPool) {
        prepare_http_database_staging(pool).await;
        let mut tx = pool.begin().await.unwrap();
        operator(&mut tx).await;
        for (sql, digest) in PRIOR {
            assert_eq!(
                hex::encode(Sha256::digest(sql)),
                *digest,
                "frozen228 producer source"
            );
            sqlx::raw_sql(*sql).execute(&mut *tx).await.unwrap();
        }
        prior_state(&mut tx).await;
        tx.commit().await.unwrap();
        seed_history(pool).await;
    }

    pub(super) async fn compose(connection: &mut PgConnection) -> Result<(), sqlx::Error> {
        operator(connection).await;
        // Read every artifact before executing; caller owns the one transaction.
        let statements = [
            account_custody_finalizer_sql(),
            artifact("ops/postgres-finalize-account-credentials.sql"),
            artifact("ops/postgres-verify-account-native.sql"),
        ];
        for sql in statements {
            sqlx::raw_sql(sqlx::AssertSqlSafe(sql.as_str()))
                .execute(&mut *connection)
                .await?;
        }
        Ok(())
    }

    pub(super) async fn state(connection: &mut PgConnection) -> String {
        // Metadata fingerprints use qualified regclass output. Every caller
        // owns a transaction, including adversarial checks before compose().
        sqlx::raw_sql("SET LOCAL search_path=pg_catalog,pg_temp")
            .execute(&mut *connection)
            .await
            .unwrap();
        sqlx::query_scalar(include_str!("../../src/account_custody_state.sql"))
            .fetch_one(connection)
            .await
            .unwrap()
    }

    pub(super) async fn assert_final(connection: &mut PgConnection) {
        assert_eq!(state(connection).await, "account_custody.native_finalized");
        let credential: String = sqlx::query_scalar(include_str!(
            "../../src/account_credential_custody_state.sql"
        ))
        .fetch_one(&mut *connection)
        .await
        .unwrap();
        assert_eq!(credential, "account_credentials.native_finalized");
        exact_actor_fk(connection).await;
    }

    pub(super) async fn rows(connection: &mut PgConnection) -> Value {
        let mut result = serde_json::Map::new();
        for table in TABLES {
            let columns: Value = sqlx::query_scalar("SELECT jsonb_agg(attname ORDER BY attnum) FROM pg_catalog.pg_attribute WHERE attrelid=pg_catalog.to_regclass($1) AND attnum>0 AND NOT attisdropped")
                .bind(format!("public.{table}")).fetch_one(&mut *connection).await.unwrap();
            let query = format!(
                "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text COLLATE \"C\"),'[]'::jsonb) FROM public.{table} t"
            );
            let mut content: Value = sqlx::query_scalar(sqlx::AssertSqlSafe(query.as_str()))
                .fetch_one(&mut *connection)
                .await
                .unwrap();
            content
                .as_array_mut()
                .expect("actual complete rows")
                .sort_by_cached_key(Value::to_string);
            result.insert((*table).into(), json!({"columns":columns,"rows":content}));
        }
        let result = Value::Object(result);
        assert!(
            preservation_holds(&result, &result),
            "complete row/column roster required"
        );
        result
    }

    pub(super) fn preservation_holds(before: &Value, after: &Value) -> bool {
        let valid = |value: &Value| {
            let Some(tables) = value.as_object() else {
                return false;
            };
            if tables.len() != TABLES.len()
                || TABLES.iter().any(|table| !tables.contains_key(*table))
            {
                return false;
            }
            for table in TABLES {
                let Some(entry) = tables.get(*table).and_then(Value::as_object) else {
                    return false;
                };
                if entry.len() != 2 {
                    return false;
                }
                let Some(columns) = entry.get("columns").and_then(Value::as_array) else {
                    return false;
                };
                let Some(rows) = entry.get("rows").and_then(Value::as_array) else {
                    return false;
                };
                let names: Option<BTreeSet<&str>> = columns.iter().map(Value::as_str).collect();
                let Some(names) = names else { return false };
                if names.is_empty() || names.len() != columns.len() {
                    return false;
                }
                let fixed_columns: Option<&[&str]> = match *table {
                    "accounts" => Some(&["id", "created_at"]),
                    "audit_events" => Some(&[
                        "id",
                        "actor",
                        "action",
                        "target_type",
                        "target_id",
                        "branch_id",
                        "before_snap",
                        "after_snap",
                        "trace_id",
                        "span_id",
                        "occurred_at",
                        "created_at",
                        "org_id",
                        "ip",
                        "user_agent",
                        "auth_method",
                        "device",
                        "classification_badges",
                        "anomaly",
                        "reason",
                    ]),
                    _ => None,
                };
                if fixed_columns.is_some_and(|required| names != required.iter().copied().collect())
                {
                    return false;
                }
                for row in rows {
                    let Some(fields) = row.as_object() else {
                        return false;
                    };
                    if fields.len() != names.len()
                        || names.iter().any(|name| !fields.contains_key(*name))
                    {
                        return false;
                    }
                }
            }
            true
        };
        valid(before) && valid(after) && before == after
    }

    pub(super) fn assert_preserved(before: &Value, after: &Value) {
        assert!(
            preservation_holds(before, after),
            "complete retained rows/columns/ledger changed; payloads intentionally omitted"
        );
    }

    pub(super) async fn metadata(connection: &mut PgConnection) -> Value {
        let observed: Value = sqlx::query_scalar(r#"SELECT jsonb_build_object(
          'relations',(SELECT jsonb_agg(jsonb_build_object('name',c.relname,'oid',c.oid,'owner',c.relowner,'acl',c.relacl,
            'rls',c.relrowsecurity,'force',c.relforcerowsecurity,
            'columns',(SELECT jsonb_agg(to_jsonb(a) ORDER BY a.attnum) FROM pg_catalog.pg_attribute a WHERE a.attrelid=c.oid),
            'defaults',(SELECT COALESCE(jsonb_agg(to_jsonb(d) ORDER BY d.oid),'[]'::jsonb) FROM pg_catalog.pg_attrdef d WHERE d.adrelid=c.oid),
            'constraints',(SELECT COALESCE(jsonb_agg(to_jsonb(k) ORDER BY k.oid),'[]'::jsonb) FROM pg_catalog.pg_constraint k WHERE k.conrelid=c.oid),
            'triggers',(SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.oid),'[]'::jsonb) FROM pg_catalog.pg_trigger t WHERE t.tgrelid=c.oid),
            'indexes',(SELECT COALESCE(jsonb_agg(to_jsonb(i) ORDER BY i.indexrelid),'[]'::jsonb) FROM pg_catalog.pg_index i WHERE i.indrelid=c.oid),
            'policies',(SELECT COALESCE(jsonb_agg(to_jsonb(p) ORDER BY p.oid),'[]'::jsonb) FROM pg_catalog.pg_policy p WHERE p.polrelid=c.oid)) ORDER BY c.relname)
            FROM pg_catalog.pg_class c WHERE c.relnamespace='public'::regnamespace AND c.relname=ANY($1)),
          'functions',(SELECT COALESCE(jsonb_agg(to_jsonb(p) ORDER BY p.oid),'[]'::jsonb) FROM pg_catalog.pg_proc p
            WHERE p.pronamespace='public'::regnamespace OR p.oid IN (SELECT tgfoid FROM pg_catalog.pg_trigger WHERE tgrelid='public.audit_events'::regclass AND NOT tgisinternal)),
          'roles',(SELECT jsonb_agg(to_jsonb(r) ORDER BY r.oid) FROM pg_catalog.pg_roles r),
          'memberships',(SELECT COALESCE(jsonb_agg(to_jsonb(m) ORDER BY m.oid),'[]'::jsonb) FROM pg_catalog.pg_auth_members m),
          'ledger',(SELECT jsonb_agg(to_jsonb(m) ORDER BY version) FROM public._sqlx_migrations m))"#)
            .bind(TABLES).fetch_one(connection).await.unwrap();
        adversarial::assert_metadata_shape(&observed);
        observed
    }

    pub(super) async fn exact_actor_fk(connection: &mut PgConnection) {
        let exact: bool = sqlx::query_scalar(r#"WITH actor AS (
          SELECT attnum,atttypid='uuid'::regtype AND NOT attnotnull AND NOT attisdropped AND attidentity='' AND attgenerated='' AS valid
          FROM pg_catalog.pg_attribute WHERE attrelid='public.audit_events'::regclass AND attname='actor' AND attnum>0
        ), edges AS (
          SELECT k.* FROM pg_catalog.pg_constraint k,actor a
          WHERE k.contype='f' AND k.conrelid='public.audit_events'::regclass AND a.attnum=ANY(k.conkey)
        ) SELECT (SELECT count(*)=1 AND bool_and(valid) FROM actor)
          AND (SELECT count(*)=1 AND bool_and(conname='audit_events_actor_fkey' AND convalidated AND conenforced
            AND confrelid='public.accounts'::regclass AND conkey=ARRAY[(SELECT attnum FROM actor)]::smallint[]
            AND confkey=ARRAY[(SELECT attnum FROM pg_catalog.pg_attribute WHERE attrelid='public.accounts'::regclass AND attname='id')]::smallint[]
            AND confupdtype='a' AND confdeltype='r' AND confmatchtype='s' AND NOT condeferrable AND NOT condeferred
            AND conislocal AND coninhcount=0 AND conparentid=0) FROM edges)
          AND (SELECT count(*)=4 AND bool_and(t.tgisinternal AND t.tgenabled='O' AND NOT t.tgdeferrable AND NOT t.tginitdeferred)
            FROM edges k JOIN pg_catalog.pg_trigger t ON t.tgconstraint=k.oid)"#)
            .fetch_one(connection).await.unwrap();
        assert!(
            exact,
            "exact nullable Account actor FK; no leftover single/composite actor edge"
        );
    }

    async fn append_history(pool: &PgPool, actor: Option<UserId>, tag: &str) {
        let runtime = login_test_pool(pool, TestDatabaseLogin::Business).await;
        let login: (String, String) =
            sqlx::query_as("SELECT session_user::text,current_user::text")
                .fetch_one(&runtime)
                .await
                .unwrap();
        assert_eq!(login, ("console_rt".into(), "console_rt".into()));
        let now: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(pool)
            .await
            .unwrap();
        let event = AuditEvent::new(
            actor,
            AuditAction::new("platform.audit_transition").unwrap(),
            "audit_transition_fixture",
            tag,
            TraceContext::generate(),
            now,
        )
        .with_snapshots(
            Some(json!({"revision":1,"tag":tag,"nullable":null})),
            Some(json!({"revision":2,"tag":tag,"list":[3,1,4]})),
        )
        .with_request_context(AuditRequestContext {
            ip: Some("192.0.2.19".into()),
            user_agent: Some(format!("audit-transition/{tag}")),
            auth_method: Some("synthetic-data-only".into()),
            device: Some("fixture-device".into()),
        })
        .with_classification(AuditClassification {
            badges: Some(vec!["fixture".into(), tag.into()]),
            anomaly: Some(false),
            reason: Some(format!("preserve-{tag}")),
        });
        let mut tx = runtime.begin().await.unwrap();
        insert_audit_event(&mut tx, &event).await.unwrap();
        tx.commit().await.unwrap();
        runtime.close().await;
    }

    async fn seed_history(pool: &PgPool) -> UserId {
        let legacy = UserId::new();
        sqlx::query("INSERT INTO public.users(id,display_name,roles,org_id,created_at) VALUES($1,'AUDIT TRANSITION DATA FIXTURE',ARRAY['MECHANIC']::text[],$2,'2026-09-16T12:34:56.123456Z')")
            .bind(*legacy.as_uuid()).bind(*OrgId::knl().as_uuid()).execute(pool).await.unwrap();
        append_history(pool, Some(legacy), "legacy").await;
        append_history(pool, None, "system-null").await;
        legacy
    }

    async fn designate_real_account(pool: &PgPool, account: Uuid) {
        let raw = std::env::var("CONSOLE_STARTUP_AUTH_DATABASE_URL")
            .expect("PREREQUISITE: actual distinct startup LOGIN");
        let mut url = Url::parse(&raw).expect("configured startup URL");
        let options = pool.connect_options();
        assert_eq!(url.username(), "console_auth_startup");
        assert!(
            url.password().is_some_and(|p| !p.is_empty())
                && url.query().is_none()
                && url.fragment().is_none()
        );
        assert_eq!(url.host_str(), Some(options.get_host()));
        assert_eq!(url.port().unwrap_or(5432), options.get_port());
        let database = options.get_database().unwrap();
        assert!(database.starts_with("_sqlx_test_"));
        url.set_path(database);
        let startup = PgPool::connect(url.as_str())
            .await
            .unwrap_or_else(|_| panic!("startup LOGIN prerequisite failed"));
        let actual: (String,String,bool) = sqlx::query_as("SELECT session_user::text,current_user::text,NOT rolsuper AND NOT rolinherit AND NOT rolbypassrls AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication FROM pg_catalog.pg_roles WHERE rolname=session_user")
            .fetch_one(&startup).await.unwrap();
        assert_eq!(
            actual,
            (
                "console_auth_startup".into(),
                "console_auth_startup".into(),
                true
            )
        );
        let (system,database,oid):(String,String,i64) = sqlx::query_as("SELECT (SELECT system_identifier::text FROM pg_catalog.pg_control_system()),current_database()::text,(SELECT oid::bigint FROM pg_catalog.pg_database WHERE datname=current_database())")
            .fetch_one(pool).await.unwrap();
        let generation:i64 = sqlx::query_scalar("SELECT security_generation FROM public.account_security WHERE account_id=$1 AND security_state='ACTIVE'")
            .bind(account).fetch_one(pool).await.unwrap();
        let receipt:(Uuid,i64,bool) = sqlx::query_as("SELECT receipt_id,revision,replayed FROM public.deployment_operator_designate_v1($1,$2,$3,$4,$5,$6,0)")
            .bind(system).bind(database).bind(oid).bind(Uuid::new_v4()).bind(account).bind(generation).fetch_one(&startup).await.unwrap();
        assert_eq!((receipt.1, receipt.2), (1, false));
        startup.close().await;
    }

    async fn populated_upgrade(pool: &PgPool, observer: bool) {
        prior228(pool).await;
        seed_terms(pool).await;
        let artifacts = Artifacts::new();
        let key = SigningKey::random(&mut OsRng);
        let config = account_browser_config(pool, artifacts.root.clone(), &key);
        let business = login_test_pool(pool, TestDatabaseLogin::Business).await;
        let auth = login_test_pool(pool, TestDatabaseLogin::Auth).await;
        // Same explicit test DI seam as retained native_extension_cases. Current
        // production startup is independently required to refuse this predecessor.
        let injected = AppState::new(
            config.clone(),
            console_app::DatabaseDependency::Postgres(business.clone()),
        )
        .unwrap()
        .with_auth_database(auth.clone());
        let predecessor = Fixture {
            service: build_router(injected.clone()),
            _artifacts: artifacts,
            pool: pool.clone(),
        };
        let (attempt, cookies) = historical_enrolled(&predecessor, &key).await;
        assert_committed(pool, &attempt).await;
        designate_real_account(pool, attempt.account).await;
        let native_without_user: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM public.users WHERE id=$1) AND EXISTS(SELECT 1 FROM public.deployment_operator_receipts WHERE account_id=$1)")
            .bind(attempt.account).fetch_one(pool).await.unwrap();
        assert!(
            native_without_user,
            "genuine native root and nonempty designation history"
        );
        injected.shutdown_realtime().await;
        let Fixture {
            service: old_service,
            _artifacts,
            pool: _,
        } = predecessor;
        drop(old_service);
        drop(injected);
        let startup = AppState::from_config(config.clone()).await;
        if let Ok(state) = &startup {
            state.shutdown_realtime().await;
        }
        assert!(
            matches!(startup,Err(console_app::AppError::Config(ref code)) if code=="account_custody.native_upgrade_required"),
            "current production startup must refuse the genuine prior228 profile"
        );
        let mut tx = pool.begin().await.unwrap();
        operator(&mut tx).await;
        if observer {
            let sql = include_str!("fixtures/account-native-deployment228-observer-6c884418.sql");
            assert_eq!(
                hex::encode(Sha256::digest(sql)),
                OBSERVER_SHA256,
                "frozen observer producer source"
            );
            sqlx::raw_sql(sql).execute(&mut *tx).await.unwrap();
        }
        prior_state(&mut tx).await;
        assert_eq!(
            state(&mut tx).await,
            "account_custody.native_upgrade_required",
            "prior228 is recognized but cannot admit new serving"
        );
        let before_rows = rows(&mut tx).await;
        let before_metadata = metadata(&mut tx).await;
        compose(&mut tx).await.unwrap();
        assert_final(&mut tx).await;
        assert_preserved(&before_rows, &rows(&mut tx).await);
        let after_metadata = metadata(&mut tx).await;
        let predecessor_equivalent = business_session_transition::predecessor_equivalent(
            &mut tx,
            &before_metadata,
            &after_metadata,
        )
        .await;
        adversarial::assert_transition_metadata(&before_metadata, &predecessor_equivalent);
        compose(&mut tx).await.unwrap();
        assert_final(&mut tx).await;
        assert_preserved(&before_rows, &rows(&mut tx).await);
        assert!(
            metadata(&mut tx).await == after_metadata,
            "replay changed metadata; payloads omitted"
        );
        if observer {
            // Do not leak a cluster-scoped observer role to other SQLx fixtures.
            tx.rollback().await.unwrap();
            let mut tx = pool.begin().await.unwrap();
            operator(&mut tx).await;
            prior_state(&mut tx).await;
            tx.rollback().await.unwrap();
        } else {
            tx.commit().await.unwrap();
            let actual = AppState::from_config(config).await.unwrap();
            let service = build_router(actual.clone());
            let me = request(&service, "GET", "/api/v2/accounts/me", &cookies, None, &[]).await;
            projection(&me.json(StatusCode::OK), attempt.account, &pool).await;
            me.private();
            actual.shutdown_realtime().await;
        }
        drop(_artifacts);
        auth.close().await;
        business.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn populated_prior228_upgrade_preserves_native_legacy_designation_and_replay(
        pool: PgPool,
    ) {
        populated_upgrade(&pool, false).await;
    }

    #[sqlx::test(migrations = false)]
    async fn populated_prior228_observer_upgrade_preserves_rows_and_replays(pool: PgPool) {
        populated_upgrade(&pool, true).await;
    }

    #[sqlx::test(migrations = false)]
    async fn fresh_chain_backfills_exact_roots_and_retains_legacy_null_audit_writers(pool: PgPool) {
        prepare_http_database_staging(&pool).await;
        seed_history(&pool).await;
        let empty: i64 = sqlx::query_scalar("SELECT count(*) FROM public.accounts")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(empty, 0, "numbered migrations do not backfill Accounts");
        let expected =
            super::super::account_root_transition::expected_roots_after_backfill(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        operator(&mut tx).await;
        let mut expected_rows = rows(&mut tx).await;
        *expected_rows
            .get_mut("accounts")
            .unwrap()
            .get_mut("rows")
            .unwrap() = expected;
        // Exact additive schema/default delta in the frozen227 NATIVE_INSTALL.
        // Full snapshots are still compared; no general projection or dropped columns.
        for (table, additions) in [
            (
                "auth_refresh_token_families",
                vec![
                    ("protocol", json!("LEGACY_COMPANY")),
                    ("account_security_generation", Value::Null),
                    ("auth_time", Value::Null),
                    ("assurance", Value::Null),
                ],
            ),
            (
                "auth_webauthn_ceremonies",
                vec![
                    ("account_browser_flow", Value::Null),
                    ("browser_nonce_sha256", Value::Null),
                    ("browser_origin", Value::Null),
                    ("terms_manifest_sha256", Value::Null),
                    ("terms_head_revision", Value::Null),
                ],
            ),
        ] {
            let entry = expected_rows
                .get_mut(table)
                .unwrap()
                .as_object_mut()
                .unwrap();
            let columns = entry.get_mut("columns").unwrap().as_array_mut().unwrap();
            for (name, _) in &additions {
                assert!(
                    !columns.iter().any(|column| column.as_str() == Some(*name)),
                    "exact staged predecessor lacks native columns"
                );
                columns.push(json!(name));
            }
            let records = entry.get_mut("rows").unwrap().as_array_mut().unwrap();
            for row in &mut *records {
                let fields = row.as_object_mut().unwrap();
                for (name, value) in &additions {
                    assert!(fields.insert((*name).into(), value.clone()).is_none());
                }
            }
            records.sort_by_cached_key(Value::to_string);
        }
        compose(&mut tx).await.unwrap();
        assert_final(&mut tx).await;
        assert_preserved(&expected_rows, &rows(&mut tx).await);
        let metadata_before_replay = metadata(&mut tx).await;
        compose(&mut tx).await.unwrap();
        assert_preserved(&expected_rows, &rows(&mut tx).await);
        assert!(
            metadata(&mut tx).await == metadata_before_replay,
            "fresh replay changed metadata"
        );
        tx.commit().await.unwrap();
        let legacy = seed_history(&pool).await;
        let exact:bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.users u JOIN public.accounts a ON a.id=u.id AND a.created_at=u.created_at WHERE u.id=$1) AND NOT EXISTS(SELECT 1 FROM public.account_security WHERE account_id=$1) AND EXISTS(SELECT 1 FROM public.audit_events WHERE actor=$1 AND action='platform.audit_transition')")
            .bind(*legacy.as_uuid()).fetch_one(&pool).await.unwrap();
        assert!(
            exact,
            "actual legacy bridge plus existing business audit writer survives cutover without creating security authority"
        );
        let nulls:i64 = sqlx::query_scalar("SELECT count(*) FROM public.audit_events WHERE actor IS NULL AND action='platform.audit_transition'").fetch_one(&pool).await.unwrap();
        assert_eq!(
            nulls, 2,
            "both pre/post NULL attribution appended through actual writer"
        );
    }

    // Execution settings belong to the operator transaction, not pooled callers.
    #[sqlx::test(migrations = false)]
    async fn operator_jit_is_disabled_locally_with_original_timeouts(pool: PgPool) {
        let mut connection = pool.acquire().await.unwrap();
        let original_jit: String = sqlx::query_scalar("SELECT current_setting('jit')")
            .fetch_one(&mut *connection)
            .await
            .unwrap();
        sqlx::raw_sql("SET SESSION jit=on")
            .execute(&mut *connection)
            .await
            .unwrap();
        let settings = "SELECT current_setting('jit'),current_setting('statement_timeout'),current_setting('lock_timeout'),current_setting('search_path')";
        let before: (String, String, String, String) = sqlx::query_as(settings)
            .fetch_one(&mut *connection)
            .await
            .unwrap();
        assert_eq!(before.0, "on", "positive control must not inherit jit=off");
        let mut tx = sqlx::Connection::begin(&mut *connection).await.unwrap();
        operator(&mut tx).await;
        let inside: (String, String, String, String) =
            sqlx::query_as(settings).fetch_one(&mut *tx).await.unwrap();
        tx.rollback().await.unwrap();
        let after: (String, String, String, String) = sqlx::query_as(settings)
            .fetch_one(&mut *connection)
            .await
            .unwrap();
        // Restore this checked-out connection even when the final assertion is RED.
        sqlx::query("SELECT set_config('jit',$1,false)")
            .bind(original_jit)
            .execute(&mut *connection)
            .await
            .unwrap();
        assert_eq!(after, before, "operator settings escaped their transaction");
        assert_eq!(
            inside,
            (
                "off".into(),
                "1min".into(),
                "5s".into(),
                "pg_catalog, pg_temp".into()
            ),
            "operator must disable JIT without changing execution budgets or search path"
        );
    }

    // Filled from immutable source bytes before packet publication.
    const OBSERVER_SHA256: &str =
        "ffe7038b43de0207d6ae3e3ce0487498edc4d061c21e4249abacc91abaacdbbc";
    include!("audit_account_transition_adversarial.rs");
    include!("audit_business_session_transition.rs");
}
