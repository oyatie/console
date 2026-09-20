// Include inside existing auth_rest::account_browser. No new Rust API imports.
// Catalog RED admits the DB owner only. Later cases invoke real owner functions
// as actual configured LOGINs and use existing real HTTP/WebAuthn enrollment.
mod deployment_operator_designation {
    use super::*;
    use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
    use sqlx::{Connection, Executor, Row};

    type ForeignKeyShape = (Vec<String>, Vec<String>, String, String, bool, bool);

    const DESIGNATE: &str =
        "public.deployment_operator_designate_v1(text,text,bigint,uuid,uuid,bigint,bigint)";
    const REVOKE: &str =
        "public.deployment_operator_revoke_v1(text,text,bigint,uuid,uuid,bigint,text)";
    const TABLES: &[&str] = &[
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
        "users",
        "organizations",
        "groups",
    ];
    #[derive(Clone)]
    struct Target {
        system: String,
        database: String,
        oid: i64,
    }
    #[derive(Clone)]
    struct Designation {
        target: Target,
        command: Uuid,
        account: Uuid,
        generation: i64,
        revision: i64,
    }
    type Receipt = (Uuid, i64, bool);

    async fn target(pool: &PgPool) -> Target {
        let (system, database, oid): (String, String, i64) = sqlx::query_as(
            "SELECT (SELECT system_identifier::text FROM pg_catalog.pg_control_system()), current_database()::text, (SELECT oid::bigint FROM pg_catalog.pg_database WHERE datname=current_database())"
        ).fetch_one(pool).await.unwrap();
        assert!(
            database.starts_with("_sqlx_test_"),
            "only owned disposable database"
        );
        Target {
            system,
            database,
            oid,
        }
    }
    async fn startup(pool: &PgPool) -> PgPool {
        let raw = std::env::var("CONSOLE_STARTUP_AUTH_DATABASE_URL").expect(
            "PREREQUISITE: externally provisioned startup LOGIN; never fixture GRANT/SET ROLE",
        );
        let mut url = Url::parse(&raw).expect("PREREQUISITE: configured startup URL");
        assert!(matches!(url.scheme(), "postgres" | "postgresql"));
        assert_eq!(url.username(), "console_auth_startup");
        assert!(
            url.password().is_some_and(|p| !p.is_empty())
                && url.query().is_none()
                && url.fragment().is_none()
        );
        let options = pool.connect_options();
        assert_eq!(url.host_str(), Some(options.get_host()));
        assert_eq!(url.port().unwrap_or(5432), options.get_port());
        let database = options.get_database().unwrap();
        assert!(database.starts_with("_sqlx_test_"));
        url.set_path(database);
        let runtime = PgPool::connect(url.as_str())
            .await
            .unwrap_or_else(|_| panic!("PREREQUISITE: actual startup login connection failed"));
        let actual: (String, String, bool, bool, bool, bool, bool, bool) = sqlx::query_as(
            "SELECT session_user::text,current_user::text,rolsuper,rolbypassrls,rolcreaterole,rolcreatedb,rolreplication,rolinherit FROM pg_catalog.pg_roles WHERE rolname=session_user"
        ).fetch_one(&runtime).await.unwrap();
        assert_eq!(
            actual,
            (
                "console_auth_startup".into(),
                "console_auth_startup".into(),
                false,
                false,
                false,
                false,
                false,
                false
            )
        );
        let memberships: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_catalog.pg_auth_members WHERE member=(SELECT oid FROM pg_catalog.pg_roles WHERE rolname=session_user)"
        ).fetch_one(&runtime).await.unwrap();
        assert_eq!(memberships, 0);
        runtime
    }
    async fn designation(pool: &PgPool, account: Uuid) -> Designation {
        let generation: i64 = sqlx::query_scalar("SELECT security_generation FROM public.account_security WHERE account_id=$1 AND security_state='ACTIVE'")
            .bind(account).fetch_one(pool).await.expect("actual enrolled ACTIVE Account prerequisite");
        Designation {
            target: target(pool).await,
            command: Uuid::new_v4(),
            account,
            generation,
            revision: 0,
        }
    }
    async fn designate<'e, E>(executor: E, input: &Designation) -> Result<Receipt, sqlx::Error>
    where
        E: Executor<'e, Database = sqlx::Postgres>,
    {
        sqlx::query_as("SELECT receipt_id,revision,replayed FROM public.deployment_operator_designate_v1($1,$2,$3,$4,$5,$6,$7)")
            .bind(&input.target.system).bind(&input.target.database).bind(input.target.oid)
            .bind(input.command).bind(input.account).bind(input.generation).bind(input.revision)
            .fetch_one(executor).await
    }
    async fn revoke(
        pool: &PgPool,
        input: &Designation,
        command: Uuid,
        revision: i64,
        reason: &str,
    ) -> Result<Receipt, sqlx::Error> {
        sqlx::query_as("SELECT receipt_id,revision,replayed FROM public.deployment_operator_revoke_v1($1,$2,$3,$4,$5,$6,$7)")
            .bind(&input.target.system).bind(&input.target.database).bind(input.target.oid)
            .bind(command).bind(input.account).bind(revision).bind(reason).fetch_one(pool).await
    }
    fn refused<T>(result: Result<T, sqlx::Error>, state: &str, code: Option<&str>) {
        let Err(error) = result else {
            panic!("owner unexpectedly permitted mutation")
        };
        let database = error
            .as_database_error()
            .expect("actual SQL owner refusal, not transport");
        assert!(
            database.code().as_deref() == Some(state),
            "unexpected SQLSTATE; missing schema is not denial"
        );
        if let Some(code) = code {
            assert!(database.message() == code, "wrong bounded owner refusal");
        }
    }
    async fn rows(pool: &PgPool, tables: &[&'static str]) -> Vec<String> {
        let mut result = Vec::new();
        for table in tables {
            // Static trusted table roster only. Compare exact PostgreSQL text,
            // retaining numeric precision and never logging protected row values.
            let sql = format!(
                "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text COLLATE \"C\"),'[]'::jsonb)::text FROM public.{table} t"
            );
            result.push(
                sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
                    .fetch_one(pool)
                    .await
                    .unwrap(),
            );
        }
        result
    }
    async fn operator_rows(pool: &PgPool) -> Vec<String> {
        rows(
            pool,
            &["deployment_operator_head", "deployment_operator_receipts"],
        )
        .await
    }
    async fn owner_exists(pool: &PgPool) {
        for name in [DESIGNATE, REVOKE] {
            let exists: bool =
                sqlx::query_scalar("SELECT pg_catalog.to_regprocedure($1) IS NOT NULL")
                    .bind(name)
                    .fetch_one(pool)
                    .await
                    .unwrap();
            assert!(
                exists,
                "PREREQUISITE: exact designation owner is not installed; only catalog test can admit it"
            );
        }
    }

    async fn direct_authority_denied(owner: &PgPool, runtime: &PgPool) {
        let session: String = sqlx::query_scalar("SELECT session_user::text")
            .fetch_one(runtime)
            .await
            .unwrap();
        for role in [
            "console_account_owner",
            "console_terms_owner",
            "console_credential_owner",
            "console_app",
            "console_auth_startup",
            "pg_monitor",
        ] {
            if role == session {
                continue;
            }
            let access:bool=sqlx::query_scalar("SELECT pg_catalog.pg_has_role(session_user,$1,'MEMBER') OR pg_catalog.pg_has_role(session_user,$1,'SET') OR pg_catalog.pg_has_role(session_user,$1,'USAGE')")
                .bind(role).fetch_one(runtime).await.unwrap();
            assert!(!access, "no indirect authority role membership");
            let mut tx = runtime.begin().await.unwrap();
            let result = sqlx::query(sqlx::AssertSqlSafe(format!("SET LOCAL ROLE {role}")))
                .execute(&mut *tx)
                .await;
            tx.rollback().await.unwrap();
            refused(result, "42501", None);
        }
        for table in ["deployment_operator_head", "deployment_operator_receipts"] {
            let columns:Vec<String>=sqlx::query_scalar("SELECT attname::text FROM pg_catalog.pg_attribute WHERE attrelid=pg_catalog.to_regclass($1) AND attnum>0 AND NOT attisdropped ORDER BY attnum")
                .bind(format!("public.{table}")).fetch_all(owner).await.unwrap();
            for column in columns {
                // Catalog names must match the fixed declared columns; quoted
                // identifiers are escaped even for private fault observations.
                let column = column.replace('"', "\"\"");
                refused(
                    sqlx::query(sqlx::AssertSqlSafe(format!(
                        "SELECT \"{column}\" FROM public.{table}"
                    )))
                    .fetch_all(runtime)
                    .await,
                    "42501",
                    None,
                );
            }
            for statement in [
                format!("INSERT INTO public.{table} DEFAULT VALUES"),
                format!("UPDATE public.{table} SET account_id=account_id WHERE false"),
                format!("DELETE FROM public.{table} WHERE false"),
            ] {
                let mut tx = runtime.begin().await.unwrap();
                let result = sqlx::query(sqlx::AssertSqlSafe(statement))
                    .execute(&mut *tx)
                    .await;
                tx.rollback().await.unwrap();
                refused(result, "42501", None);
            }
        }
        let mut tx = runtime.begin().await.unwrap();
        let result = sqlx::query(
            "TRUNCATE public.deployment_operator_head,public.deployment_operator_receipts",
        )
        .execute(&mut *tx)
        .await;
        tx.rollback().await.unwrap();
        refused(result, "42501", None);
    }
    async fn blocking_witness(pool: &PgPool, waiter: i32, holder: i32) -> bool {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                let blockers: Vec<i32> =
                    sqlx::query_scalar("SELECT pg_catalog.pg_blocking_pids($1)")
                        .bind(waiter)
                        .fetch_one(pool)
                        .await
                        .unwrap();
                if blockers.contains(&holder) {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .is_ok()
    }
    #[sqlx::test(migrations = false)]
    async fn catalog_enrolls_exact_designation_owner_and_no_runtime_capability(pool: PgPool) {
        prepare_http_database(&pool).await;
        // This currently reaches an actual catalog assertion without new imports.
        // It is the DB-owner declaration RED, not a missing-SQL exception.
        for signature in [DESIGNATE, REVOKE] {
            let row = sqlx::query("SELECT pg_catalog.pg_get_userbyid(p.proowner) AS owner,p.prosecdef,p.proconfig,p.provolatile::text AS volatility,p.proparallel::text AS parallel FROM pg_catalog.pg_proc p WHERE p.oid=pg_catalog.to_regprocedure($1)")
                .bind(signature).fetch_optional(&pool).await.unwrap();
            assert!(
                row.is_some(),
                "required deployment designation DB owner is absent"
            );
            let row = row.unwrap();
            let extra: bool = sqlx::query_scalar("SELECT p.prokind='f' AND l.lanname='plpgsql' AND NOT p.proisstrict AND NOT p.proleakproof AND p.proretset AND p.prosupport=0 AND p.provariadic=0 AND p.pronargdefaults=0 AND p.proargdefaults IS NULL AND p.prosqlbody IS NULL AND p.protrftypes IS NULL AND p.probin IS NULL AND p.pronargs=7 AND pg_catalog.pg_get_function_result(p.oid)='TABLE(receipt_id uuid, revision bigint, replayed boolean)' AND (SELECT count(*) FROM pg_catalog.pg_proc other WHERE other.pronamespace=p.pronamespace AND other.proname=p.proname)=1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_language l ON l.oid=p.prolang WHERE p.oid=pg_catalog.to_regprocedure($1)")
                .bind(signature).fetch_one(&pool).await.unwrap();
            assert!(
                extra,
                "one exact non-strict registered owner, no overload/default/hidden execution flags"
            );

            assert_eq!(row.get::<String, _>("owner"), "console_account_owner");
            assert!(row.get::<bool, _>("prosecdef"));
            assert_eq!(
                row.get::<Vec<String>, _>("proconfig"),
                vec!["search_path=pg_catalog, pg_temp"]
            );
            assert_eq!(row.get::<String, _>("volatility"), "v");
            assert_eq!(row.get::<String, _>("parallel"), "u");
            let grants: Vec<(String,String,bool)> = sqlx::query_as("SELECT COALESCE(r.rolname,'PUBLIC')::text,a.privilege_type,a.is_grantable FROM pg_catalog.pg_proc p CROSS JOIN LATERAL pg_catalog.aclexplode(COALESCE(p.proacl,pg_catalog.acldefault('f',p.proowner))) a LEFT JOIN pg_catalog.pg_roles r ON r.oid=a.grantee WHERE p.oid=pg_catalog.to_regprocedure($1) ORDER BY 1,2,3")
                .bind(signature).fetch_all(&pool).await.unwrap();
            assert_eq!(
                grants,
                vec![
                    ("console_account_owner".into(), "EXECUTE".into(), false),
                    ("console_auth_startup".into(), "EXECUTE".into(), false)
                ]
            );
        }
        for table in ["deployment_operator_head", "deployment_operator_receipts"] {
            let columns: Vec<(String,String,bool)> = sqlx::query_as("SELECT attname::text,pg_catalog.format_type(atttypid,atttypmod),attnotnull FROM pg_catalog.pg_attribute WHERE attrelid=pg_catalog.to_regclass($1) AND attnum>0 AND NOT attisdropped ORDER BY attnum")
                .bind(format!("public.{table}")).fetch_all(&pool).await.unwrap();
            let expected: &[(&str, &str, bool)] = if table == "deployment_operator_head" {
                &[
                    ("singleton", "smallint", true),
                    ("system_identifier", "text", true),
                    ("database_name", "text", true),
                    ("database_oid", "bigint", true),
                    ("account_id", "uuid", true),
                    ("revision", "bigint", true),
                    ("receipt_id", "uuid", true),
                ]
            } else {
                &[
                    ("receipt_id", "uuid", true),
                    ("command_id", "uuid", true),
                    ("system_identifier", "text", true),
                    ("database_name", "text", true),
                    ("database_oid", "bigint", true),
                    ("account_id", "uuid", true),
                    ("kind", "text", true),
                    ("expected_revision", "bigint", true),
                    ("revision", "bigint", true),
                    ("expected_security_generation", "bigint", false),
                    ("reason", "text", false),
                    ("recorded_at", "timestamp with time zone", true),
                ]
            };
            assert_eq!(
                columns,
                expected
                    .iter()
                    .map(|(name, kind, nonnull)| (name.to_string(), kind.to_string(), *nonnull))
                    .collect::<Vec<_>>()
            );
            let primary: Vec<String> = sqlx::query_scalar("SELECT a.attname::text FROM pg_catalog.pg_constraint c CROSS JOIN LATERAL unnest(c.conkey) WITH ORDINALITY k(attnum,ordering) JOIN pg_catalog.pg_attribute a ON a.attrelid=c.conrelid AND a.attnum=k.attnum WHERE c.conrelid=pg_catalog.to_regclass($1) AND c.contype='p' ORDER BY k.ordering")
                .bind(format!("public.{table}")).fetch_all(&pool).await.unwrap();
            assert_eq!(
                primary,
                vec![if table == "deployment_operator_head" {
                    "singleton"
                } else {
                    "receipt_id"
                }]
            );
        }
        let account_fk: Vec<ForeignKeyShape> = sqlx::query_as("SELECT ARRAY(SELECT a.attname::text FROM unnest(c.conkey) WITH ORDINALITY k(attnum,n) JOIN pg_catalog.pg_attribute a ON a.attrelid=c.conrelid AND a.attnum=k.attnum ORDER BY k.n),ARRAY(SELECT a.attname::text FROM unnest(c.confkey) WITH ORDINALITY k(attnum,n) JOIN pg_catalog.pg_attribute a ON a.attrelid=c.confrelid AND a.attnum=k.attnum ORDER BY k.n),c.confupdtype::text,c.confdeltype::text,c.convalidated,c.condeferrable FROM pg_catalog.pg_constraint c WHERE c.conrelid='public.deployment_operator_receipts'::regclass AND c.confrelid='public.accounts'::regclass AND c.contype='f'")
            .fetch_all(&pool).await.unwrap();
        assert_eq!(
            account_fk,
            vec![(
                vec!["account_id".into()],
                vec!["id".into()],
                "r".into(),
                "r".into(),
                true,
                false
            )]
        );
        let head_fk: Vec<ForeignKeyShape> = sqlx::query_as("SELECT ARRAY(SELECT a.attname::text FROM unnest(c.conkey) WITH ORDINALITY k(attnum,n) JOIN pg_catalog.pg_attribute a ON a.attrelid=c.conrelid AND a.attnum=k.attnum ORDER BY k.n),ARRAY(SELECT a.attname::text FROM unnest(c.confkey) WITH ORDINALITY k(attnum,n) JOIN pg_catalog.pg_attribute a ON a.attrelid=c.confrelid AND a.attnum=k.attnum ORDER BY k.n),c.confupdtype::text,c.confdeltype::text,c.convalidated,c.condeferrable FROM pg_catalog.pg_constraint c WHERE c.conrelid='public.deployment_operator_head'::regclass AND c.confrelid='public.deployment_operator_receipts'::regclass AND c.contype='f'")
            .fetch_all(&pool).await.unwrap();
        let tuple: Vec<String> = [
            "receipt_id",
            "revision",
            "account_id",
            "system_identifier",
            "database_name",
            "database_oid",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        assert_eq!(
            head_fk,
            vec![(tuple.clone(), tuple, "r".into(), "r".into(), true, false)]
        );
        let command_unique: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_catalog.pg_constraint c WHERE c.conrelid='public.deployment_operator_receipts'::regclass AND c.contype='u' AND c.conkey=ARRAY[(SELECT attnum FROM pg_catalog.pg_attribute WHERE attrelid=c.conrelid AND attname='command_id')]::smallint[]")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(command_unique, 1);
        let guarded: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_catalog.pg_trigger WHERE tgrelid='public.deployment_operator_receipts'::regclass AND NOT tgisinternal AND tgenabled='A' AND (tgtype::integer & 2)=2 AND (tgtype::integer & 1)=0 AND (tgtype::integer & 8)=8 AND (tgtype::integer & 16)=16 AND (tgtype::integer & 32)=32")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(
            guarded, 1,
            "one always-on statement guard covers UPDATE/DELETE/TRUNCATE"
        );
        let checks:Vec<(String,String,bool,bool)>=sqlx::query_as("SELECT conname::text,pg_catalog.pg_get_expr(conbin,conrelid),convalidated,connoinherit FROM pg_catalog.pg_constraint WHERE conrelid='public.deployment_operator_head'::regclass AND contype='c' ORDER BY conname")
            .fetch_all(&pool).await.unwrap();
        assert_eq!(
            checks,
            vec![
                (
                    "deployment_operator_head_revision_check".into(),
                    "(revision > 0)".into(),
                    true,
                    false
                ),
                (
                    "deployment_operator_head_singleton_check".into(),
                    "(singleton = 1)".into(),
                    true,
                    false
                )
            ]
        );
        let receipt_checks:Vec<(String,bool,bool)>=sqlx::query_as("SELECT conname::text,convalidated,connoinherit FROM pg_catalog.pg_constraint WHERE conrelid='public.deployment_operator_receipts'::regclass AND contype='c' ORDER BY conname")
            .fetch_all(&pool).await.unwrap();
        assert_eq!(
            receipt_checks,
            vec![
                (
                    "deployment_operator_receipts_expected_revision_check".to_owned(),
                    true,
                    false
                ),
                (
                    "deployment_operator_receipts_generation_kind_check".to_owned(),
                    true,
                    false
                ),
                (
                    "deployment_operator_receipts_kind_check".to_owned(),
                    true,
                    false
                ),
                (
                    "deployment_operator_receipts_reason_kind_check".to_owned(),
                    true,
                    false
                ),
                (
                    "deployment_operator_receipts_revision_check".to_owned(),
                    true,
                    false
                ),
            ]
        );
        let control_access: bool = sqlx::query_scalar("SELECT pg_catalog.has_function_privilege('console_account_owner','pg_catalog.pg_control_system()','EXECUTE')")
            .fetch_one(&pool).await.unwrap();
        assert!(
            control_access,
            "descriptor verifier needs its explicitly enrolled narrow builtin grant"
        );
        let builtin_grant:i64=sqlx::query_scalar("SELECT count(*) FROM pg_catalog.pg_proc p CROSS JOIN LATERAL pg_catalog.aclexplode(p.proacl) a WHERE p.oid='pg_catalog.pg_control_system()'::regprocedure AND a.grantee='console_account_owner'::regrole AND a.privilege_type='EXECUTE' AND NOT a.is_grantable").fetch_one(&pool).await.unwrap();
        assert_eq!(
            builtin_grant, 1,
            "explicit narrow builtin capability, no inherited-role substitute"
        );
        let role: Option<(bool,bool,bool,bool,bool,bool,bool)> = sqlx::query_as("SELECT rolcanlogin,rolsuper,rolbypassrls,rolcreaterole,rolcreatedb,rolreplication,rolinherit FROM pg_catalog.pg_roles WHERE rolname='console_auth_startup'")
            .fetch_optional(&pool).await.unwrap();
        assert_eq!(role, Some((true, false, false, false, false, false, false)));
        for table in ["deployment_operator_head", "deployment_operator_receipts"] {
            let owner: Option<String> = sqlx::query_scalar("SELECT pg_catalog.pg_get_userbyid(relowner) FROM pg_catalog.pg_class WHERE oid=pg_catalog.to_regclass($1) AND relkind='r'")
                .bind(format!("public.{table}")).fetch_optional(&pool).await.unwrap();
            assert_eq!(owner.as_deref(), Some("console_account_owner"));
            let grants:Vec<(String,String,bool)>=sqlx::query_as("SELECT COALESCE(r.rolname,'PUBLIC')::text,a.privilege_type,a.is_grantable FROM pg_catalog.pg_class c CROSS JOIN LATERAL pg_catalog.aclexplode(COALESCE(c.relacl,pg_catalog.acldefault('r',c.relowner))) a LEFT JOIN pg_catalog.pg_roles r ON r.oid=a.grantee WHERE c.oid=pg_catalog.to_regclass($1) ORDER BY 1,2,3")
                .bind(format!("public.{table}")).fetch_all(&pool).await.unwrap();
            let privileges: &[&str] = if table == "deployment_operator_head" {
                &["INSERT", "SELECT", "UPDATE"]
            } else {
                &["INSERT", "SELECT"]
            };
            assert_eq!(
                grants,
                privileges
                    .iter()
                    .map(|p| ("console_account_owner".to_owned(), p.to_string(), false))
                    .collect::<Vec<_>>()
            );
            let column_acls:i64=sqlx::query_scalar("SELECT count(*) FROM pg_catalog.pg_attribute WHERE attrelid=pg_catalog.to_regclass($1) AND attnum>0 AND NOT attisdropped AND attacl IS NOT NULL")
                .bind(format!("public.{table}")).fetch_one(&pool).await.unwrap();
            assert_eq!(
                column_acls,
                if table == "deployment_operator_receipts" {
                    1
                } else {
                    0
                },
                "only the exact receipt key-share owner grant"
            );
            let column_grants:Vec<(String,String,String,bool)>=sqlx::query_as("SELECT a.attname::text,COALESCE(r.rolname,'PUBLIC')::text,p.privilege_type,p.is_grantable FROM pg_catalog.pg_attribute a CROSS JOIN LATERAL pg_catalog.aclexplode(a.attacl) p LEFT JOIN pg_catalog.pg_roles r ON r.oid=p.grantee WHERE a.attrelid=pg_catalog.to_regclass($1) AND a.attnum>0 AND NOT a.attisdropped ORDER BY 1,2,3,4")
                .bind(format!("public.{table}")).fetch_all(&pool).await.unwrap();
            let expected: Vec<(String, String, String, bool)> =
                if table == "deployment_operator_receipts" {
                    vec![(
                        "receipt_id".into(),
                        "console_account_owner".into(),
                        "UPDATE".into(),
                        false,
                    )]
                } else {
                    vec![]
                };
            assert_eq!(
                column_grants, expected,
                "native FK key-share requires only owner UPDATE(receipt_id), never serving rights"
            );
            for role in [
                "console_auth_startup",
                "console_rt",
                "console_auth_rt",
                "console_leave_cmd",
                "console_ontology_cmd",
                "console_platform_force_cmd",
                "console_app",
            ] {
                let any: bool = sqlx::query_scalar("SELECT pg_catalog.has_table_privilege($1,$2,'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN') OR pg_catalog.has_any_column_privilege($1,$2,'SELECT,INSERT,UPDATE,REFERENCES')")
                    .bind(role).bind(format!("public.{table}")).fetch_one(&pool).await.unwrap();
                assert!(
                    !any,
                    "global operational authority cannot be direct runtime table access"
                );
            }
        }
    }

    #[sqlx::test(migrations = false)]
    async fn active_account_designation_replays_without_identity_or_company_effects(pool: PgPool) {
        let app = fixture(&pool).await;
        owner_exists(&pool).await;
        let (a, cookies) = enrolled(&app).await;
        let (b, _) = enrolled(&app).await;
        let me = request(&app, "GET", "/api/v2/accounts/me", &cookies, None, &[]).await;
        projection(&me.json(StatusCode::OK), a.account);
        let runtime = startup(&pool).await;
        let input = designation(&pool, a.account).await;
        let untouched = rows(&pool, TABLES).await;
        assert!(operator_rows(&pool).await.iter().all(|value| value == "[]"));
        let command_started: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&pool)
            .await
            .unwrap();
        let first = designate(&runtime, &input)
            .await
            .expect("actual startup owner designation");
        assert!(!first.0.is_nil() && first.1 == 1 && !first.2);
        let stored: (Uuid,Uuid,i64,String) = sqlx::query_as("SELECT h.account_id,r.command_id,h.revision,r.kind FROM public.deployment_operator_head h JOIN public.deployment_operator_receipts r ON r.receipt_id=h.receipt_id WHERE h.singleton=1 AND r.receipt_id=$1")
            .bind(first.0).fetch_one(&pool).await.unwrap();
        assert_eq!(stored, (a.account, input.command, 1, "DESIGNATE".into()));
        let exact:bool=sqlx::query_scalar("SELECT r.command_id=$2 AND r.system_identifier=$3 AND r.database_name=$4 AND r.database_oid=$5 AND r.account_id=$6 AND r.kind='DESIGNATE' AND r.expected_revision=0 AND r.revision=1 AND r.expected_security_generation=$7 AND r.reason IS NULL AND r.recorded_at<=clock_timestamp() AND r.recorded_at>=$8 AND h.account_id=r.account_id AND h.system_identifier=r.system_identifier AND h.database_name=r.database_name AND h.database_oid=r.database_oid AND h.revision=r.revision FROM public.deployment_operator_receipts r JOIN public.deployment_operator_head h ON h.receipt_id=r.receipt_id WHERE r.receipt_id=$1")
            .bind(first.0).bind(input.command).bind(&input.target.system).bind(&input.target.database).bind(input.target.oid).bind(input.account).bind(input.generation).bind(command_started).fetch_one(&pool).await.unwrap();
        assert!(
            exact,
            "full original typed designation receipt and head must match command"
        );

        let committed = operator_rows(&pool).await;
        let replay = designate(&runtime, &input).await.unwrap();
        assert_eq!(replay, (first.0, 1, true));
        assert!(operator_rows(&pool).await == committed);
        let mut conflict = input.clone();
        conflict.account = b.account;
        refused(
            designate(&runtime, &conflict).await,
            "P0001",
            Some("deployment_operator.command_conflict"),
        );
        conflict = input.clone();
        conflict.generation += 1;
        refused(
            designate(&runtime, &conflict).await,
            "P0001",
            Some("deployment_operator.command_conflict"),
        );
        conflict = input.clone();
        conflict.revision = 1;
        refused(
            designate(&runtime, &conflict).await,
            "P0001",
            Some("deployment_operator.command_conflict"),
        );
        conflict = input.clone();
        conflict.command = Uuid::new_v4();
        conflict.account = b.account;
        refused(
            designate(&runtime, &conflict).await,
            "P0001",
            Some("deployment_operator.already_initialized"),
        );
        assert!(operator_rows(&pool).await == committed && rows(&pool, TABLES).await == untouched);
        assert_no_company_identity(&pool, a.account).await;
        runtime.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn unknown_pending_stale_target_and_wrong_deployment_refuse_without_effect(pool: PgPool) {
        let app = fixture(&pool).await;
        owner_exists(&pool).await;
        let (active, _) = enrolled(&app).await;
        let pending = start(&app).await;
        let runtime = startup(&pool).await;
        let good = designation(&pool, active.account).await;
        let before = operator_rows(&pool).await;
        let untouched = rows(&pool, TABLES).await;
        for variant in 0..6 {
            let mut bad = good.clone();
            bad.command = Uuid::new_v4();
            let code = match variant {
                0 => {
                    bad.account = Uuid::new_v4();
                    "deployment_operator.target_ineligible"
                }
                1 => {
                    bad.account = pending.account;
                    "deployment_operator.target_ineligible"
                }
                2 => {
                    bad.generation += 1;
                    "deployment_operator.target_ineligible"
                }
                3 => {
                    bad.target.system = if bad.target.system == "1" { "2" } else { "1" }.into();
                    "deployment_operator.target_mismatch"
                }
                4 => {
                    bad.target.database.replace_range(..1, "x");
                    assert_eq!(bad.target.database.len(), good.target.database.len());
                    assert_ne!(bad.target.database, good.target.database);
                    "deployment_operator.target_mismatch"
                }
                _ => {
                    bad.target.oid += 1;
                    "deployment_operator.target_mismatch"
                }
            };
            refused(designate(&runtime, &bad).await, "P0001", Some(code));
            assert!(operator_rows(&pool).await == before && rows(&pool, TABLES).await == untouched);
        }
        for variant in 0..7 {
            let mut bad = good.clone();
            bad.command = Uuid::new_v4();
            match variant {
                0 => bad.account = Uuid::nil(),
                1 => bad.command = Uuid::nil(),
                2 => bad.target.system.clear(),
                3 => bad.target.database.clear(),
                4 => bad.target.oid = 0,
                5 => bad.generation = 0,
                _ => bad.revision = -1,
            }
            refused(
                designate(&runtime, &bad).await,
                "P0001",
                Some("deployment_operator.invalid_command"),
            );
            assert!(operator_rows(&pool).await == before && rows(&pool, TABLES).await == untouched);
        }
        assert!(
            designate(&runtime, &good).await.is_ok(),
            "same genuine owner positive after all faults"
        );
        runtime.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn ordinary_login_roles_cannot_designate_revoke_or_read_authority(pool: PgPool) {
        let app = fixture(&pool).await;
        owner_exists(&pool).await;
        let (a, _) = enrolled(&app).await;
        let startup = startup(&pool).await;
        let input = designation(&pool, a.account).await;
        let initial = designate(&startup, &input).await.unwrap();
        let before = operator_rows(&pool).await;
        let untouched = rows(&pool, TABLES).await;
        for role in [
            TestDatabaseLogin::Business,
            TestDatabaseLogin::Auth,
            TestDatabaseLogin::LeaveCommand,
            TestDatabaseLogin::OntologyCommand,
            TestDatabaseLogin::PlatformForceCommand,
        ] {
            let runtime = login_test_pool(&pool, role).await;
            refused(designate(&runtime, &input).await, "42501", None);
            refused(
                revoke(
                    &runtime,
                    &input,
                    Uuid::new_v4(),
                    initial.1,
                    "TEST_ONLY revoke",
                )
                .await,
                "42501",
                None,
            );
            for table in ["deployment_operator_head", "deployment_operator_receipts"] {
                refused(
                    sqlx::query(sqlx::AssertSqlSafe(format!("SELECT * FROM public.{table}")))
                        .fetch_all(&runtime)
                        .await,
                    "42501",
                    None,
                );
            }
            assert!(operator_rows(&pool).await == before && rows(&pool, TABLES).await == untouched);
            direct_authority_denied(&pool, &runtime).await;
            runtime.close().await;
        }
        direct_authority_denied(&pool, &startup).await;
        refused(
            sqlx::query("SELECT * FROM pg_catalog.pg_control_system()")
                .fetch_all(&startup)
                .await,
            "42501",
            None,
        );

        assert!(
            revoke(
                &startup,
                &input,
                Uuid::new_v4(),
                initial.1,
                "TEST_ONLY authorized revoke"
            )
            .await
            .is_ok()
        );
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn revocation_tombstone_survives_old_replay_and_cas_conflicts(pool: PgPool) {
        let app = fixture(&pool).await;
        owner_exists(&pool).await;
        let (a, _) = enrolled(&app).await;
        let runtime = startup(&pool).await;
        let input = designation(&pool, a.account).await;
        let untouched = rows(&pool, TABLES).await;
        let original = designate(&runtime, &input).await.unwrap();
        let before = operator_rows(&pool).await;
        refused(
            revoke(
                &runtime,
                &input,
                Uuid::new_v4(),
                0,
                "TEST_ONLY wrong revision",
            )
            .await,
            "P0001",
            Some("deployment_operator.stale_revision"),
        );
        assert!(operator_rows(&pool).await == before);
        let command = Uuid::new_v4();
        let command_started: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&pool)
            .await
            .unwrap();
        let revoked = revoke(&runtime, &input, command, 1, "TEST_ONLY revoke")
            .await
            .unwrap();
        assert!(!revoked.0.is_nil() && revoked.0 != original.0 && revoked.1 == 2 && !revoked.2);
        let current:(Uuid,i64,String)=sqlx::query_as("SELECT h.account_id,h.revision,r.kind FROM public.deployment_operator_head h JOIN public.deployment_operator_receipts r ON r.receipt_id=h.receipt_id WHERE h.singleton=1")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(current, (a.account, 2, "REVOKE".into()));
        let exact:bool=sqlx::query_scalar("SELECT r.command_id=$2 AND r.system_identifier=$3 AND r.database_name=$4 AND r.database_oid=$5 AND r.account_id=$6 AND r.kind='REVOKE' AND r.expected_revision=1 AND r.revision=2 AND r.expected_security_generation IS NULL AND r.reason=$7 AND r.recorded_at<=clock_timestamp() AND r.recorded_at>=$8 FROM public.deployment_operator_receipts r WHERE r.receipt_id=$1")
            .bind(revoked.0).bind(command).bind(&input.target.system).bind(&input.target.database).bind(input.target.oid).bind(input.account).bind("TEST_ONLY revoke").bind(command_started).fetch_one(&pool).await.unwrap();
        assert!(
            exact,
            "full original typed revocation receipt must match command"
        );
        refused(
            revoke(&runtime, &input, input.command, 1, "TEST_ONLY cross-kind").await,
            "P0001",
            Some("deployment_operator.command_conflict"),
        );
        let mut other_kind = input.clone();
        other_kind.command = command;
        refused(
            designate(&runtime, &other_kind).await,
            "P0001",
            Some("deployment_operator.command_conflict"),
        );
        let mut wrong = input.clone();
        wrong.target.database.replace_range(..1, "x");
        assert_eq!(wrong.target.database.len(), input.target.database.len());
        assert_ne!(wrong.target.database, input.target.database);
        refused(
            designate(&runtime, &wrong).await,
            "P0001",
            Some("deployment_operator.target_mismatch"),
        );
        refused(
            revoke(&runtime, &wrong, command, 1, "TEST_ONLY revoke").await,
            "P0001",
            Some("deployment_operator.target_mismatch"),
        );
        let tombstone = operator_rows(&pool).await;
        assert_eq!(
            designate(&runtime, &input).await.unwrap(),
            (original.0, 1, true)
        );
        assert_eq!(
            revoke(&runtime, &input, command, 1, "TEST_ONLY revoke")
                .await
                .unwrap(),
            (revoked.0, 2, true)
        );
        refused(
            revoke(&runtime, &input, command, 1, "TEST_ONLY changed reason").await,
            "P0001",
            Some("deployment_operator.command_conflict"),
        );
        let mut fresh = input.clone();
        fresh.command = Uuid::new_v4();
        refused(
            designate(&runtime, &fresh).await,
            "P0001",
            Some("deployment_operator.already_initialized"),
        );
        refused(
            revoke(&runtime, &input, Uuid::new_v4(), 2, "TEST_ONLY repeat").await,
            "P0001",
            Some("deployment_operator.already_revoked"),
        );
        assert!(operator_rows(&pool).await == tombstone && rows(&pool, TABLES).await == untouched);
        runtime.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn mandatory_receipt_failure_rolls_back_then_same_command_retries(pool: PgPool) {
        let app = fixture(&pool).await;
        owner_exists(&pool).await;
        let (a, _) = enrolled(&app).await;
        let runtime = startup(&pool).await;
        let input = designation(&pool, a.account).await;
        let before = operator_rows(&pool).await;
        let untouched = rows(&pool, TABLES).await;
        // Deliberate AFTER INSERT failure in owned disposable database. Exact
        // marker proves the real receipt insertion path was reached.
        sqlx::raw_sql("CREATE FUNCTION public.test_only_designation_receipt_failure() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'TEST_ONLY.designation_receipt_failure'; END $$; CREATE TRIGGER test_only_designation_receipt_failure AFTER INSERT ON public.deployment_operator_receipts FOR EACH ROW EXECUTE FUNCTION public.test_only_designation_receipt_failure();")
            .execute(&pool).await.unwrap();
        let failed = designate(&runtime, &input).await;
        sqlx::raw_sql("DROP TRIGGER test_only_designation_receipt_failure ON public.deployment_operator_receipts; DROP FUNCTION public.test_only_designation_receipt_failure();")
            .execute(&pool).await.unwrap();
        refused(
            failed,
            "P0001",
            Some("TEST_ONLY.designation_receipt_failure"),
        );
        assert!(operator_rows(&pool).await == before && rows(&pool, TABLES).await == untouched);
        let result = designate(&runtime, &input).await.unwrap();
        assert_eq!(result.1, 1);
        assert!(!result.2);
        runtime.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn distinct_accounts_contend_at_one_actual_designation_slot(pool: PgPool) {
        let app = fixture(&pool).await;
        owner_exists(&pool).await;
        let (a, _) = enrolled(&app).await;
        let (b, _) = enrolled(&app).await;
        let runtime = startup(&pool).await;
        let left = designation(&pool, a.account).await;
        let right = designation(&pool, b.account).await;
        let untouched = rows(&pool, TABLES).await;
        let mut first = runtime.acquire().await.unwrap();
        let mut second = runtime.acquire().await.unwrap();
        let first_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *first)
            .await
            .unwrap();
        let second_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *second)
            .await
            .unwrap();
        let mut transaction = first.begin().await.unwrap();
        let initial = designate(&mut *transaction, &left).await.unwrap();
        let second_task = tokio::spawn(async move { designate(&mut *second, &right).await });
        let witnessed = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                let blockers: Vec<i32> =
                    sqlx::query_scalar("SELECT pg_catalog.pg_blocking_pids($1)")
                        .bind(second_pid)
                        .fetch_one(&pool)
                        .await
                        .unwrap();
                if blockers.contains(&first_pid) {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await;
        transaction.commit().await.unwrap();
        let outcome = tokio::time::timeout(std::time::Duration::from_secs(5), second_task)
            .await
            .unwrap()
            .unwrap();
        assert!(
            witnessed.is_ok(),
            "actual competing startup backend must block on first writer before release"
        );
        refused(
            outcome,
            "P0001",
            Some("deployment_operator.already_initialized"),
        );
        let count:(i64,i64)=sqlx::query_as("SELECT (SELECT count(*) FROM public.deployment_operator_head),(SELECT count(*) FROM public.deployment_operator_receipts)")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(count, (1, 1));
        let winner: Uuid = sqlx::query_scalar(
            "SELECT account_id FROM public.deployment_operator_head WHERE receipt_id=$1",
        )
        .bind(initial.0)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(winner, a.account);
        assert!(rows(&pool, TABLES).await == untouched);
        drop(first);
        runtime.close().await;
    }
    #[sqlx::test(migrations = false)]
    async fn receipt_history_and_original_head_binding_are_immutable(pool: PgPool) {
        let app = fixture(&pool).await;
        owner_exists(&pool).await;
        let (a, _) = enrolled(&app).await;
        let (b, _) = enrolled(&app).await;
        let runtime = startup(&pool).await;
        let input = designation(&pool, a.account).await;
        designate(&runtime, &input).await.unwrap();
        let original = operator_rows(&pool).await;
        let untouched = rows(&pool, TABLES).await;
        for statement in [
            "UPDATE public.deployment_operator_receipts SET command_id=command_id",
            "UPDATE public.deployment_operator_receipts SET command_id=command_id WHERE false",
            "DELETE FROM public.deployment_operator_receipts",
            "DELETE FROM public.deployment_operator_receipts WHERE false",
        ] {
            let mut transaction = pool.begin().await.unwrap();
            let result = sqlx::query(statement).execute(&mut *transaction).await;
            transaction.rollback().await.unwrap();
            refused(
                result,
                "P0001",
                Some("deployment_operator.receipt_immutable"),
            );
            assert!(operator_rows(&pool).await == original);
        }
        for statement in [
            "UPDATE public.deployment_operator_receipts SET receipt_id=receipt_id",
            "UPDATE public.deployment_operator_receipts SET receipt_id=receipt_id WHERE false",
        ] {
            // Negative owner misuse only: nonlogin owner needs narrow UPDATE
            // for native FK key-share; the ALWAYS statement guard refuses DML.
            let mut tx = pool.begin().await.unwrap();
            sqlx::query("SET LOCAL ROLE console_account_owner")
                .execute(&mut *tx)
                .await
                .unwrap();
            let result = sqlx::query(statement).execute(&mut *tx).await;
            tx.rollback().await.unwrap();
            refused(
                result,
                "P0001",
                Some("deployment_operator.receipt_immutable"),
            );
            assert!(
                operator_rows(&pool).await == original && rows(&pool, TABLES).await == untouched
            );
        }
        let mut transaction = pool.begin().await.unwrap();
        sqlx::query("SET LOCAL session_replication_role=replica")
            .execute(&mut *transaction)
            .await
            .unwrap();
        let result = sqlx::query(
            "UPDATE public.deployment_operator_receipts SET command_id=command_id WHERE false",
        )
        .execute(&mut *transaction)
        .await;
        transaction.rollback().await.unwrap();
        refused(
            result,
            "P0001",
            Some("deployment_operator.receipt_immutable"),
        );
        let mut transaction = pool.begin().await.unwrap();
        let result = sqlx::query("UPDATE public.deployment_operator_head SET account_id=$1")
            .bind(b.account)
            .execute(&mut *transaction)
            .await;
        transaction.rollback().await.unwrap();
        refused(result, "P0001", Some("deployment_operator.head_immutable"));
        assert!(operator_rows(&pool).await == original && rows(&pool, TABLES).await == untouched);
        runtime.close().await;
    }
    #[sqlx::test(migrations = false)]
    async fn null_malformed_inputs_and_revocation_bounds_have_no_partial_effect(pool: PgPool) {
        let app = fixture(&pool).await;
        owner_exists(&pool).await;
        let (a, _) = enrolled(&app).await;
        let runtime = startup(&pool).await;
        let good = designation(&pool, a.account).await;
        let before = operator_rows(&pool).await;
        let identity = rows(&pool, TABLES).await;
        for null in 0..7 {
            let result:Result<Receipt,_>=sqlx::query_as("SELECT receipt_id,revision,replayed FROM public.deployment_operator_designate_v1($1,$2,$3,$4,$5,$6,$7)")
                .bind((null!=0).then_some(good.target.system.as_str())).bind((null!=1).then_some(good.target.database.as_str()))
                .bind((null!=2).then_some(good.target.oid)).bind((null!=3).then_some(good.command)).bind((null!=4).then_some(good.account))
                .bind((null!=5).then_some(good.generation)).bind((null!=6).then_some(good.revision)).fetch_one(&runtime).await;
            refused(result, "P0001", Some("deployment_operator.invalid_command"));
            assert!(operator_rows(&pool).await == before && rows(&pool, TABLES).await == identity);
        }
        for bad_system in ["0", "-1", "+1", "01", "1 ", "18446744073709551616"] {
            let mut bad = good.clone();
            bad.target.system = bad_system.into();
            refused(
                designate(&runtime, &bad).await,
                "P0001",
                Some("deployment_operator.invalid_command"),
            );
        }
        for bad_oid in [-1, 4294967296] {
            let mut bad = good.clone();
            bad.target.oid = bad_oid;
            refused(
                designate(&runtime, &bad).await,
                "P0001",
                Some("deployment_operator.invalid_command"),
            );
        }
        let mut bad = good.clone();
        bad.target.database = "x".repeat(64);
        refused(
            designate(&runtime, &bad).await,
            "P0001",
            Some("deployment_operator.invalid_command"),
        );
        assert!(operator_rows(&pool).await == before && rows(&pool, TABLES).await == identity);
        designate(&runtime, &good).await.unwrap();
        let current = operator_rows(&pool).await;
        for null in 0..7 {
            let result:Result<Receipt,_>=sqlx::query_as("SELECT receipt_id,revision,replayed FROM public.deployment_operator_revoke_v1($1,$2,$3,$4,$5,$6,$7)")
                .bind((null!=0).then_some(good.target.system.as_str())).bind((null!=1).then_some(good.target.database.as_str()))
                .bind((null!=2).then_some(good.target.oid)).bind((null!=3).then_some(Uuid::new_v4())).bind((null!=4).then_some(good.account))
                .bind((null!=5).then_some(1_i64)).bind((null!=6).then_some("TEST_ONLY revoke")).fetch_one(&runtime).await;
            refused(result, "P0001", Some("deployment_operator.invalid_command"));
            assert!(operator_rows(&pool).await == current && rows(&pool, TABLES).await == identity);
        }
        for reason in [
            String::new(),
            " \t\n\r".into(),
            "x".repeat(513),
            "가".repeat(171),
        ] {
            refused(
                revoke(&runtime, &good, Uuid::new_v4(), 1, &reason).await,
                "P0001",
                Some("deployment_operator.invalid_command"),
            );
            assert!(operator_rows(&pool).await == current && rows(&pool, TABLES).await == identity);
        }
        refused(
            revoke(&runtime, &good, Uuid::nil(), 1, "TEST_ONLY revoke").await,
            "P0001",
            Some("deployment_operator.invalid_command"),
        );
        refused(
            revoke(&runtime, &good, Uuid::new_v4(), -1, "TEST_ONLY revoke").await,
            "P0001",
            Some("deployment_operator.invalid_command"),
        );
        let reason = "가".repeat(170) + "xy";
        assert_eq!(reason.len(), 512);
        let command = Uuid::new_v4();
        let result = revoke(&runtime, &good, command, 1, &reason).await.unwrap();
        let stored: String = sqlx::query_scalar(
            "SELECT reason FROM public.deployment_operator_receipts WHERE receipt_id=$1",
        )
        .bind(result.0)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(
            stored == reason,
            "bounded UTF-8 reason is preserved exactly, never truncated"
        );
        assert_eq!(
            revoke(&runtime, &good, command, 1, &reason).await.unwrap(),
            (result.0, 2, true)
        );
        assert!(rows(&pool, TABLES).await == identity);
        runtime.close().await;
    }
    #[sqlx::test(migrations = false)]
    async fn same_command_wait_timeout_then_exact_retry_preserves_one_receipt(pool: PgPool) {
        let app = fixture(&pool).await;
        owner_exists(&pool).await;
        let (a, _) = enrolled(&app).await;
        let runtime = startup(&pool).await;
        let input = designation(&pool, a.account).await;
        let identity = rows(&pool, TABLES).await;
        let mut first = runtime.acquire().await.unwrap();
        let first_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *first)
            .await
            .unwrap();
        let mut holder = first.begin().await.unwrap();
        let committed = designate(&mut *holder, &input).await.unwrap();
        for (setting, code) in [
            ("SET LOCAL lock_timeout='2s'", "55P03"),
            ("SET LOCAL statement_timeout='2s'", "57014"),
        ] {
            let mut second = runtime.acquire().await.unwrap();
            let second_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
                .fetch_one(&mut *second)
                .await
                .unwrap();
            let competing = input.clone();
            let waiter = tokio::spawn(async move {
                let mut tx = second.begin().await.unwrap();
                sqlx::query(setting).execute(&mut *tx).await.unwrap();
                let result = designate(&mut *tx, &competing).await;
                tx.rollback().await.unwrap();
                result
            });
            let witnessed = blocking_witness(&pool, second_pid, first_pid).await;
            let outcome = tokio::time::timeout(std::time::Duration::from_secs(5), waiter)
                .await
                .unwrap()
                .unwrap();
            refused(outcome, code, None);
            assert!(witnessed, "actual same-command wait before timeout");
            assert!(
                operator_rows(&pool).await.iter().all(|v| v == "[]"),
                "uncommitted authority has no visible partial state"
            );
        }
        let mut second = runtime.acquire().await.unwrap();
        let second_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *second)
            .await
            .unwrap();
        let competing = input.clone();
        let waiter = tokio::spawn(async move { designate(&mut *second, &competing).await });
        let witnessed = blocking_witness(&pool, second_pid, first_pid).await;
        holder.commit().await.unwrap();
        drop(first);
        let result = tokio::time::timeout(std::time::Duration::from_secs(5), waiter)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(
            witnessed,
            "actual waiting replay must complete only after original commit"
        );
        assert_eq!(result, (committed.0, 1, true));
        let before = operator_rows(&pool).await;
        assert_eq!(
            designate(&runtime, &input).await.unwrap(),
            (committed.0, 1, true)
        );
        let counts:(i64,i64)=sqlx::query_as("SELECT (SELECT count(*) FROM public.deployment_operator_head),(SELECT count(*) FROM public.deployment_operator_receipts)").fetch_one(&pool).await.unwrap();
        assert_eq!(counts, (1, 1));
        assert!(operator_rows(&pool).await == before && rows(&pool, TABLES).await == identity);
        runtime.close().await;
    }
    #[sqlx::test(migrations = false)]
    async fn security_invalidation_serializes_and_historical_replay_does_not_reactivate(
        pool: PgPool,
    ) {
        let app = fixture(&pool).await;
        owner_exists(&pool).await;
        let (a, _) = enrolled(&app).await;
        let (b, _) = enrolled(&app).await;
        let runtime = startup(&pool).await;
        let blocked = designation(&pool, a.account).await;
        let good = designation(&pool, b.account).await;
        // Explicit negative invalidation injection after genuine WebAuthn. This
        // proves designation serialization, not an implemented suspension flow.
        let mut invalidator = pool.acquire().await.unwrap();
        let mut caller = runtime.acquire().await.unwrap();
        let invalidator_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *invalidator)
            .await
            .unwrap();
        let caller_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *caller)
            .await
            .unwrap();
        let mut change = invalidator.begin().await.unwrap();
        assert_eq!(sqlx::query("UPDATE public.account_security SET security_state='SECURITY_SUSPENDED',security_generation=security_generation+1,revision=revision+1,updated_at=clock_timestamp() WHERE account_id=$1 AND security_state='ACTIVE'").bind(a.account).execute(&mut *change).await.unwrap().rows_affected(),1);
        let call = tokio::spawn(async move { designate(&mut *caller, &blocked).await });
        let witnessed = blocking_witness(&pool, caller_pid, invalidator_pid).await;
        change.commit().await.unwrap();
        let result = tokio::time::timeout(std::time::Duration::from_secs(5), call)
            .await
            .unwrap()
            .unwrap();
        assert!(witnessed);
        refused(
            result,
            "P0001",
            Some("deployment_operator.target_ineligible"),
        );
        assert!(operator_rows(&pool).await.iter().all(|v| v == "[]"));
        // Reverse order: owner holds the current ACTIVE guard until commit.
        let mut caller = runtime.acquire().await.unwrap();
        let caller_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *caller)
            .await
            .unwrap();
        let mut tx = caller.begin().await.unwrap();
        let original = designate(&mut *tx, &good).await.unwrap();
        let invalidating = tokio::spawn(async move {
            sqlx::query("UPDATE public.account_security SET security_state='SECURITY_SUSPENDED',security_generation=security_generation+1,revision=revision+1,updated_at=clock_timestamp() WHERE account_id=$1 AND security_state='ACTIVE'").bind(b.account).execute(&mut *invalidator).await
        });
        let witnessed = blocking_witness(&pool, invalidator_pid, caller_pid).await;
        tx.commit().await.unwrap();
        drop(caller);
        let changed = tokio::time::timeout(std::time::Duration::from_secs(5), invalidating)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(changed.rows_affected(), 1);
        assert!(witnessed);
        let identity = rows(&pool, TABLES).await;
        let history = operator_rows(&pool).await;
        assert_eq!(
            designate(&runtime, &good).await.unwrap(),
            (original.0, 1, true)
        );
        assert!(operator_rows(&pool).await == history && rows(&pool, TABLES).await == identity);
        let revoked = revoke(
            &runtime,
            &good,
            Uuid::new_v4(),
            1,
            "TEST_ONLY revoke suspended operator",
        )
        .await
        .unwrap();
        assert_eq!(revoked.1, 2);
        assert!(rows(&pool, TABLES).await == identity);
        let tombstone = operator_rows(&pool).await;
        assert_eq!(
            designate(&runtime, &good).await.unwrap(),
            (original.0, 1, true)
        );
        assert!(operator_rows(&pool).await == tombstone && rows(&pool, TABLES).await == identity);
        runtime.close().await;
    }
    #[sqlx::test(migrations = false)]
    async fn receipt_checks_refuse_malformed_copies_without_creating_authority(pool: PgPool) {
        let app = fixture(&pool).await;
        owner_exists(&pool).await;
        let (a, _) = enrolled(&app).await;
        let runtime = startup(&pool).await;
        let input = designation(&pool, a.account).await;
        let initial = designate(&runtime, &input).await.unwrap();
        let revoked = revoke(
            &runtime,
            &input,
            Uuid::new_v4(),
            1,
            "TEST_ONLY immutable source",
        )
        .await
        .unwrap();
        let history = operator_rows(&pool).await;
        let identity = rows(&pool, TABLES).await;
        let checks = [
            (
                initial.0,
                "kind",
                "'UNKNOWN'",
                "deployment_operator_receipts_kind_check",
            ),
            (
                initial.0,
                "expected_revision",
                "-1",
                "deployment_operator_receipts_expected_revision_check",
            ),
            (
                initial.0,
                "revision",
                "0",
                "deployment_operator_receipts_revision_check",
            ),
            (
                initial.0,
                "expected_security_generation",
                "NULL",
                "deployment_operator_receipts_generation_kind_check",
            ),
            (
                initial.0,
                "expected_security_generation",
                "0",
                "deployment_operator_receipts_generation_kind_check",
            ),
            (
                initial.0,
                "expected_security_generation",
                "-1",
                "deployment_operator_receipts_generation_kind_check",
            ),
            (
                revoked.0,
                "expected_security_generation",
                "1",
                "deployment_operator_receipts_generation_kind_check",
            ),
            (
                initial.0,
                "reason",
                "'unexpected'",
                "deployment_operator_receipts_reason_kind_check",
            ),
            (
                revoked.0,
                "reason",
                "NULL",
                "deployment_operator_receipts_reason_kind_check",
            ),
            (
                revoked.0,
                "reason",
                "''",
                "deployment_operator_receipts_reason_kind_check",
            ),
            (
                revoked.0,
                "reason",
                r"E' \t\n\r\f\013'",
                "deployment_operator_receipts_reason_kind_check",
            ),
            (
                revoked.0,
                "reason",
                "repeat('x',513)",
                "deployment_operator_receipts_reason_kind_check",
            ),
            (
                revoked.0,
                "reason",
                "repeat('가',171)",
                "deployment_operator_receipts_reason_kind_check",
            ),
        ];
        for (source, column, replacement, expected_constraint) in checks {
            let fields = [
                "receipt_id",
                "command_id",
                "system_identifier",
                "database_name",
                "database_oid",
                "account_id",
                "kind",
                "expected_revision",
                "revision",
                "expected_security_generation",
                "reason",
                "recorded_at",
            ];
            let values = fields
                .iter()
                .map(|name| {
                    if *name == column {
                        replacement.to_owned()
                    } else if matches!(*name, "receipt_id" | "command_id") {
                        "pg_catalog.gen_random_uuid()".into()
                    } else {
                        format!("r.{name}")
                    }
                })
                .collect::<Vec<_>>()
                .join(",");
            // Fixed expressions/identifiers only. Copy actual receipts solely
            // as a hostile constraint fault; rollback even if wrongly accepted.
            let sql = format!(
                "INSERT INTO public.deployment_operator_receipts({}) SELECT {values} FROM public.deployment_operator_receipts r WHERE r.receipt_id=$1",
                fields.join(",")
            );
            let mut tx = pool.begin().await.unwrap();
            let result = sqlx::query(sqlx::AssertSqlSafe(sql))
                .bind(source)
                .execute(&mut *tx)
                .await;
            tx.rollback().await.unwrap();
            let Err(error) = result else {
                panic!("malformed receipt copy incorrectly passed its CHECK")
            };
            let database = error
                .as_database_error()
                .expect("CHECK refusal is a database error");
            assert!(
                database.code().as_deref() == Some("23514"),
                "wrong constraint refusal; missing schema/FK/permission is not CHECK evidence"
            );
            assert!(
                database.constraint() == Some(expected_constraint),
                "wrong exact receipt CHECK caught the fault"
            );
            assert!(operator_rows(&pool).await == history && rows(&pool, TABLES).await == identity);
        }
        runtime.close().await;
    }
    include!("startup_effective_privileges.rs");
    #[cfg(feature = "test-operator-transport")]
    include!("deployment_operator_transport.rs");
    include!("native_company_setup.rs");
}
