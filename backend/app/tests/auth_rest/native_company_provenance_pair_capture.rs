// Unaccepted test-source proposal: metadata captures only, both variants roll back.
// The production migration baseline contains historical business data. This is
// neither a business-empty fixture nor a serving profile or workflow acceptance.
mod native_company_provenance_pair_capture {
    use super::*;
    use sqlx::{Acquire, PgConnection, Postgres, Transaction};
    use std::io::Write as _;

    const OWNER: &str = include_str!("../../../../ops/postgres-company-provenance-v1-owner.sql");
    const CAPTURE: &str =
        include_str!("../../../../ops/postgres-capture-company-provenance-v1-custody.sql");
    const PREDECESSOR_CAPTURE: &str =
        include_str!("../../../../ops/postgres-capture-native-people-directory-custody.sql");
    const ROW_LOCK_STATE: &str =
        include_str!("../../../../ops/postgres-native-people-directory-row-lock-custody-state.sql");
    const ACCOUNT: &str = include_str!("../../../../ops/postgres-finalize-account-custody.sql");
    const CREDENTIALS: &str =
        include_str!("../../../../ops/postgres-finalize-account-credentials.sql");
    const COMPANY: &str = include_str!("../../../../ops/postgres-finalize-company-enrollment.sql");
    const POLICY_V2: &str =
        include_str!("../../../../ops/postgres-finalize-native-company-policy-v2.sql");
    const ROW_LOCK: &str =
        include_str!("../../../../ops/postgres-finalize-native-people-directory-row-lock.sql");
    const OBSERVER: &str = include_str!("../../../../ops/postgres-install-durability-observer.sql");
    const LEDGER: &str = include_str!("../../../../ops/account-custody-migrations.sha384");
    static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../crates/platform/db/migrations");
    const CORRECTED: [&str; 2] = [
        "b0d8ced14929a0c4ef041dfceb57519c64663cb87f39c1dccf61d61227e2278e",
        "2c69786d88b784ca348725dc85730b9d8be1bc835e1069f64de7b7d3ec6ac80d",
    ];

    fn digest(text: &str) -> String {
        hex::encode(Sha256::digest(text.as_bytes()))
    }

    fn source_pins() -> Value {
        let mut pins = BTreeMap::new();
        for (name, source, expected) in [
            (
                "owner",
                OWNER,
                "e813293ace00c46358a0c1c62093e0911aae4da1b93c4dd1577dcb351c5398c2",
            ),
            (
                "capture",
                CAPTURE,
                "0fc02c2bd70375acb0b6ddc86b66892af4069003c88dc58455347887cdf28ab2",
            ),
            (
                "predecessor_capture",
                PREDECESSOR_CAPTURE,
                "bc8a1f87f57cd676ca1a3deae12263b1cca4a290c71a12e9a1749d189e090ca7",
            ),
            (
                "row_lock_state",
                ROW_LOCK_STATE,
                "781cc446ce26525cbad6cd281c66239d1a7366eb9369d914068f39886cad3d6e",
            ),
            (
                "account",
                ACCOUNT,
                "fb532ef3b82f6d03b58d6e164a26567683039444098cf1a0a98d33e27e5dd4f8",
            ),
            (
                "credentials",
                CREDENTIALS,
                "2f960163c9bd8832cdd3a058a6ae69d7503cf1009c476ef292af9443036624d9",
            ),
            (
                "company",
                COMPANY,
                "bc35b52d5692e474a3c890dde73112f56e7b85a241ab390075d58b5d5b43a92f",
            ),
            (
                "policy_v1",
                POLICY_INSTALLER,
                "4e7fc41b1d2ed6c2155d44d43347c18815ed9e70611eb9285f21e1c590bf996a",
            ),
            (
                "policy_v2",
                POLICY_V2,
                "ec945607e209b93843116ae2b2a20772797dce38ff7884fb96081f09651f7d8e",
            ),
            (
                "row_lock",
                ROW_LOCK,
                "040652e06b9ae491514898964ff9cd6db03ffe860db3e14dfd9ccf8ea85b221f",
            ),
            (
                "observer",
                OBSERVER,
                "ffe7038b43de0207d6ae3e3ce0487498edc4d061c21e4249abacc91abaacdbbc",
            ),
            (
                "ledger231",
                LEDGER,
                "42079d3f1b8077e163960adc65f35f1959c22a67bf42acf43d6b816721ba1357",
            ),
            (
                "classifier_session",
                CLASSIFIER_SESSION,
                "728593aa17d371220a9f62bfe5bf4ecab5096d41eba7c8852907845ae65a87bd",
            ),
        ] {
            assert_eq!(digest(source), expected, "unreviewed {name} source");
            assert!(pins.insert(name, expected).is_none());
        }
        json!(pins)
    }

    async fn marked(connection: &mut PgConnection, require_unmigrated: bool) {
        let valid: bool = sqlx::query_scalar(
            "SELECT session_user=current_user AND current_user='console_buck_admin' \
             AND starts_with(current_database(),'_sqlx_test_') \
             AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' \
             AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user) \
             AND (NOT $1 OR to_regclass('public._sqlx_migrations') IS NULL)",
        )
        .bind(require_unmigrated)
        .fetch_one(connection)
        .await
        .unwrap();
        assert!(
            valid,
            "capture requires directly authenticated marked disposable admin"
        );
    }

    async fn applied_ledger(connection: &mut PgConnection) -> Value {
        let records: Vec<_> = LEDGER.split_inclusive('\n').collect();
        assert_eq!(records.len(), 231);
        assert_eq!(
            digest(&records[..230].concat()),
            "25e02488cdaf864f6d15ee21d62df98eb263bb82de1e2a283470ca659d160325"
        );
        let expected: Vec<_> = MIGRATOR
            .iter()
            .map(|migration| {
                (
                    migration.version,
                    migration.description.to_string(),
                    true,
                    migration.checksum.to_vec(),
                )
            })
            .collect();
        assert_eq!(expected.len(), 231);
        for (index, (migration, line)) in MIGRATOR.iter().zip(&records).enumerate() {
            let (version, checksum) = line.trim_end_matches('\n').split_once('\t').unwrap();
            assert_eq!(version.parse::<i64>().unwrap(), index as i64 + 1);
            assert_eq!(migration.version, index as i64 + 1);
            assert_eq!(hex::encode(migration.checksum.as_ref()), checksum);
            assert_eq!(
                hex::encode(sha2::Sha384::digest(migration.sql.as_str().as_bytes())),
                checksum
            );
        }
        let owner: String = sqlx::query_scalar("SELECT pg_get_userbyid(relowner) FROM pg_catalog.pg_class WHERE oid='public._sqlx_migrations'::regclass AND relkind='r' AND NOT relispartition")
            .fetch_one(&mut *connection).await.unwrap();
        assert_eq!(owner, "console_app");
        let actual: Vec<(i64, String, bool, Vec<u8>)> = sqlx::query_as(
            "SELECT version,description,success,checksum FROM public._sqlx_migrations ORDER BY version",
        ).fetch_all(connection).await.unwrap();
        assert_eq!(
            actual, expected,
            "actual complete applied migration ledger differs"
        );
        let actual_records: Vec<_> = actual.iter().map(|(version,description,success,checksum)|
            json!({"version":version,"description":description,"success":success,"sha384":hex::encode(checksum)})).collect();
        json!({"owner":owner,"records":records.len(),"raw_ledger_text":LEDGER,
            "sha256":digest(LEDGER),"historical230_sha256":digest(&records[..230].concat()),
            "actual":actual_records})
    }

    // Same lossless PostgreSQL row ordering as company_setup::all_rows, on this
    // retained transaction. Include every non-system base-table schema, not a
    // hand-selected business roster. Raw business/credential rows are never emitted.
    async fn rows(connection: &mut PgConnection) -> BTreeMap<String, String> {
        let tables: Vec<(String, String)> = sqlx::query_as(
            "SELECT n.nspname::text,c.relname::text FROM pg_catalog.pg_class c \
             JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace \
             WHERE c.relkind IN ('r','p') AND n.nspname NOT IN ('pg_catalog','information_schema') \
             AND NOT starts_with(n.nspname,'pg_toast') AND NOT starts_with(n.nspname,'pg_temp_') \
             ORDER BY n.nspname COLLATE \"C\",c.relname COLLATE \"C\"",
        )
        .fetch_all(&mut *connection)
        .await
        .unwrap();
        assert!(
            !tables.is_empty() && tables.len() <= 1024,
            "bounded complete base-table census required"
        );
        let mut result = BTreeMap::new();
        let mut total_bytes = 0_usize;
        for (schema, table) in &tables {
            let schema_sql = schema.replace('"', "\"\"");
            let table_sql = table.replace('"', "\"\"");
            let query = format!(
                "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text COLLATE \"C\"),'[]'::jsonb)::text FROM \"{schema_sql}\".\"{table_sql}\" t"
            );
            let raw: String = sqlx::query_scalar(sqlx::AssertSqlSafe(query))
                .fetch_one(&mut *connection)
                .await
                .unwrap();
            assert!(
                raw.len() <= 16 * 1024 * 1024,
                "fixture row census exceeds reviewed per-table bound"
            );
            total_bytes = total_bytes.checked_add(raw.len()).unwrap();
            assert!(
                total_bytes <= 64 * 1024 * 1024,
                "complete fixture row census exceeds reviewed total bound"
            );
            let parsed: Value = serde_json::from_str(&raw).unwrap();
            assert!(
                parsed
                    .as_array()
                    .is_some_and(|values| values.iter().all(Value::is_object))
            );
            // JSON tuple identity cannot collide when identifiers contain dots.
            let identity = serde_json::to_string(&(schema, table)).unwrap();
            assert!(result.insert(identity, raw).is_none());
        }
        assert_eq!(result.len(), tables.len());
        assert!(result.contains_key("[\"public\",\"_sqlx_migrations\"]"));
        result
    }

    fn row_summary(rows: &BTreeMap<String, String>) -> Value {
        let records: Vec<_> = rows
            .iter()
            .map(|(identity, raw)| {
                let count = serde_json::from_str::<Value>(raw)
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .len();
                json!({"identity":identity,"rows":count,"raw_row_text_sha256":digest(raw)})
            })
            .collect();
        json!({"tables":records,"table_count":rows.len(),
            "complete_raw_text_map_sha256":hex::encode(Sha256::digest(serde_json::to_vec(rows).unwrap()))})
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Capture {
        text: String,
        sha256: String,
        rights: bool,
        snapshot: Value,
    }

    impl Capture {
        fn record(&self) -> Value {
            json!({"snapshot_text":self.text,"snapshot":self.snapshot,
                "snapshot_sha256":self.sha256,"startup_rights_valid":self.rights})
        }
    }

    async fn capture(connection: &mut PgConnection, source: &'static str) -> Capture {
        sqlx::raw_sql(CLASSIFIER_SESSION)
            .execute(&mut *connection)
            .await
            .unwrap();
        // Only wrap the exact declared query to retain PostgreSQL's original
        // JSON text. No clause, routine selector, historical bytes or hashes change.
        let body = source
            .strip_suffix(";\n")
            .expect("exact reviewed capture terminator");
        let query = format!(
            "SELECT snapshot::text,snapshot_sha256,native_directory_startup_rights_valid FROM ({body}) original_capture"
        );
        let (text, sha256, rights): (String, String, Option<bool>) =
            sqlx::query_as(sqlx::AssertSqlSafe(query))
                .fetch_one(connection)
                .await
                .unwrap();
        assert_eq!(
            digest(&text),
            sha256,
            "database digest does not match exact UTF-8 snapshot text"
        );
        assert_eq!(
            rights,
            Some(true),
            "complete original startup privilege matrix invalid"
        );
        let snapshot: Value = serde_json::from_str(&text).unwrap();
        assert!(snapshot.is_object());
        Capture {
            text,
            sha256,
            rights: true,
            snapshot,
        }
    }

    // Catalog identities/OIDs are diagnostic evidence, never portable profile
    // fingerprints. Password verifiers and raw business data are excluded.
    // Relation permission/structure fields remain exact. Autovacuum estimates and
    // horizons (six explicitly listed pg_class fields) are not security metadata;
    // neither original complete custody serializer is filtered or rewritten.
    const CATALOG: &str = r#"WITH namespaces AS (
      SELECT * FROM pg_catalog.pg_namespace WHERE nspname NOT IN ('pg_catalog','information_schema')
       AND NOT starts_with(nspname,'pg_toast') AND NOT starts_with(nspname,'pg_temp_')
    ), relations AS (SELECT c.* FROM pg_catalog.pg_class c JOIN namespaces n ON n.oid=c.relnamespace),
    routines AS (
      SELECT p.oid,jsonb_build_object('schema',n.nspname,'name',p.proname,
       'type_arguments',pg_catalog.oidvectortypes(p.proargtypes),
       'identity_arguments',pg_catalog.pg_get_function_identity_arguments(p.oid),
       'result',pg_catalog.pg_get_function_result(p.oid),'owner',pg_catalog.pg_get_userbyid(p.proowner),
       'language',l.lanname,'support_oid',p.prosupport::oid,'source_sha256',encode(sha256(convert_to(p.prosrc,'UTF8')),'hex'),
       'raw',to_jsonb(p),'definition',CASE WHEN p.prokind='a' THEN NULL ELSE pg_catalog.pg_get_functiondef(p.oid) END,
       'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(pg_catalog.pg_get_userbyid(a.grantor),
          CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_catalog.pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable)
          ORDER BY pg_catalog.pg_get_userbyid(a.grantor) COLLATE "C",
          CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_catalog.pg_get_userbyid(a.grantee) END COLLATE "C",a.privilege_type)
          FROM pg_catalog.aclexplode(COALESCE(p.proacl,pg_catalog.acldefault('f',p.proowner))) a),'[]'::jsonb),
       'effective_execute',(SELECT jsonb_agg(jsonb_build_array(r.rolname,
          pg_catalog.has_function_privilege(r.oid,p.oid,'EXECUTE'),
          pg_catalog.has_function_privilege(r.oid,p.oid,'EXECUTE WITH GRANT OPTION')) ORDER BY r.rolname COLLATE "C") FROM pg_catalog.pg_roles r),
       'dependencies',COALESCE((SELECT jsonb_agg(to_jsonb(d) ORDER BY to_jsonb(d)::text COLLATE "C") FROM pg_catalog.pg_depend d
          WHERE d.classid='pg_catalog.pg_proc'::regclass AND d.objid=p.oid),'[]'::jsonb),
       'shared_dependencies',COALESCE((SELECT jsonb_agg(to_jsonb(d) ORDER BY to_jsonb(d)::text COLLATE "C") FROM pg_catalog.pg_shdepend d
          WHERE d.dbid=(SELECT oid FROM pg_catalog.pg_database WHERE datname=current_database())
           AND d.classid='pg_catalog.pg_proc'::regclass AND d.objid=p.oid),'[]'::jsonb)) AS record
      FROM pg_catalog.pg_proc p JOIN namespaces n ON n.oid=p.pronamespace JOIN pg_catalog.pg_language l ON l.oid=p.prolang
    ) SELECT jsonb_build_object(
      'roles',(SELECT jsonb_agg(to_jsonb(r)-'rolpassword' ORDER BY r.rolname COLLATE "C") FROM pg_catalog.pg_roles r),
      'memberships',(SELECT COALESCE(jsonb_agg(to_jsonb(m) ORDER BY to_jsonb(m)::text COLLATE "C"),'[]'::jsonb) FROM pg_catalog.pg_auth_members m),
      'database',(SELECT jsonb_build_object('name',datname,'owner',pg_catalog.pg_get_userbyid(datdba),'acl',datacl) FROM pg_catalog.pg_database WHERE datname=current_database()),
      'database_role_settings',(SELECT COALESCE(jsonb_agg(to_jsonb(d) ORDER BY to_jsonb(d)::text COLLATE "C"),'[]'::jsonb) FROM pg_catalog.pg_db_role_setting d
        WHERE d.setdatabase IN (0,(SELECT oid FROM pg_catalog.pg_database WHERE datname=current_database()))),
      'schemas',(SELECT jsonb_agg(to_jsonb(n) ORDER BY n.nspname COLLATE "C") FROM namespaces n),
      'relations',(SELECT jsonb_agg(to_jsonb(c)-ARRAY['relpages','reltuples','relallvisible','relallfrozen','relfrozenxid','relminmxid'] ORDER BY c.relnamespace,c.relname COLLATE "C") FROM relations c),
      'attributes',(SELECT jsonb_agg(to_jsonb(a) ORDER BY a.attrelid,a.attnum) FROM pg_catalog.pg_attribute a JOIN relations c ON c.oid=a.attrelid),
      'constraints',(SELECT COALESCE(jsonb_agg(to_jsonb(k) ORDER BY k.oid),'[]'::jsonb) FROM pg_catalog.pg_constraint k JOIN relations c ON c.oid=k.conrelid),
      'triggers',(SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.oid),'[]'::jsonb) FROM pg_catalog.pg_trigger t JOIN relations c ON c.oid=t.tgrelid),
      'policies',(SELECT COALESCE(jsonb_agg(to_jsonb(p) ORDER BY p.oid),'[]'::jsonb) FROM pg_catalog.pg_policy p JOIN relations c ON c.oid=p.polrelid),
      'default_acls',(SELECT COALESCE(jsonb_agg(to_jsonb(a) ORDER BY a.oid),'[]'::jsonb) FROM pg_catalog.pg_default_acl a),
      'event_triggers',(SELECT COALESCE(jsonb_agg(to_jsonb(e) ORDER BY e.oid),'[]'::jsonb) FROM pg_catalog.pg_event_trigger e),
      'routines',(SELECT jsonb_agg(record ORDER BY record->>'schema' COLLATE "C",record->>'name' COLLATE "C",record->>'type_arguments' COLLATE "C") FROM routines)
    )::text"#;

    async fn catalog(connection: &mut PgConnection) -> (String, Value) {
        sqlx::raw_sql("SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL statement_timeout='30s'; SET LOCAL jit=off")
            .execute(&mut *connection).await.unwrap();
        let text: String = sqlx::query_scalar(CATALOG)
            .fetch_one(connection)
            .await
            .unwrap();
        assert!(
            text.len() <= 32 * 1024 * 1024,
            "raw security catalog exceeds reviewed bound"
        );
        let parsed: Value = serde_json::from_str(&text).unwrap();
        assert!(parsed.is_object());
        (text, parsed)
    }

    async fn namespace(connection: &mut PgConnection) -> Vec<(String, String, String)> {
        // Prefix detection includes every schema. It never accepts a routine by
        // wildcard: the only accepted roster is the exact two typed public ABIs.
        sqlx::query_as("SELECT n.nspname::text,p.proname::text,pg_catalog.oidvectortypes(p.proargtypes) FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace WHERE starts_with(p.proname,'account_company_provenance_') ORDER BY n.nspname COLLATE \"C\",p.proname COLLATE \"C\",pg_catalog.oidvectortypes(p.proargtypes) COLLATE \"C\"")
            .fetch_all(connection).await.unwrap()
    }

    fn exact_namespace() -> Vec<(String, String, String)> {
        vec![
            (
                "public".into(),
                "account_company_provenance_lock_v1".into(),
                "uuid, uuid".into(),
            ),
            (
                "public".into(),
                "account_company_provenance_v1".into(),
                "uuid".into(),
            ),
        ]
    }

    fn routine_map(catalog: &Value) -> BTreeMap<String, Value> {
        let mut result = BTreeMap::new();
        for routine in catalog["routines"].as_array().unwrap() {
            let identity = format!(
                "{}.{}({})",
                routine["schema"].as_str().unwrap(),
                routine["name"].as_str().unwrap(),
                routine["type_arguments"].as_str().unwrap()
            );
            assert!(result.insert(identity, routine.clone()).is_none());
        }
        result
    }

    fn exact_routines(catalog: &Value) {
        let routines = routine_map(catalog);
        for (identity, owner, result, arguments, body, acl) in [
            (
                "public.account_company_provenance_lock_v1(uuid, uuid)",
                "console_app",
                "boolean",
                json!(["p_company", "p_group"]),
                "2a373c67aae7ed89804b8daf2f013b116d9c7a7c6046d4d574d46c766c67a959",
                json!([
                    ["console_app", "console_account_owner", "EXECUTE", false],
                    ["console_app", "console_app", "EXECUTE", false]
                ]),
            ),
            (
                "public.account_company_provenance_v1(uuid)",
                "console_account_owner",
                "text",
                json!(["p_company"]),
                "4720cf26c2d86b6843986275465ee8794f54e69fc11d77fb600c82f704ebf9dd",
                json!([
                    [
                        "console_account_owner",
                        "console_account_owner",
                        "EXECUTE",
                        false
                    ],
                    ["console_account_owner", "console_rt", "EXECUTE", false]
                ]),
            ),
        ] {
            let routine = &routines[identity];
            assert_eq!(routine["owner"], owner);
            assert_eq!(routine["result"], result);
            assert_eq!(routine["language"], "plpgsql");
            assert_eq!(routine["source_sha256"], body);
            assert_eq!(digest(routine["raw"]["prosrc"].as_str().unwrap()), body);
            assert_eq!(routine["acl"], acl);
            let raw = &routine["raw"];
            assert_eq!(raw["prokind"], "f");
            assert_eq!(raw["provolatile"], "v");
            assert_eq!(raw["proparallel"], "u");
            assert_eq!(raw["prosecdef"], true);
            assert_eq!(raw["proisstrict"], false);
            assert_eq!(raw["proretset"], false);
            assert_eq!(raw["proleakproof"], false);
            assert_eq!(raw["proargnames"], arguments);
            assert_eq!(
                raw["proconfig"],
                json!(["search_path=pg_catalog, pg_temp", "row_security=on"])
            );
            assert_eq!(raw["procost"], 100);
            assert_eq!(raw["prorows"], 0);
            for field in [
                "proargdefaults",
                "proallargtypes",
                "proargmodes",
                "probin",
                "prosqlbody",
                "protrftypes",
            ] {
                assert!(raw[field].is_null(), "unexpected routine field: {field}");
            }
            assert_eq!(raw["provariadic"], "0");
            assert_eq!(raw["pronargdefaults"], 0);
            assert_eq!(routine["support_oid"], "0");
            assert_eq!(raw["proacl"].as_array().unwrap().len(), 2);
            let rights: BTreeMap<_, _> = routine["effective_execute"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| {
                    (
                        r[0].as_str().unwrap(),
                        (r[1].as_bool().unwrap(), r[2].as_bool().unwrap()),
                    )
                })
                .collect();
            assert_eq!(rights["console_account_owner"].0, true);
            if owner != "console_account_owner" {
                assert_eq!(rights["console_account_owner"].1, false);
            }
            assert_eq!(rights["console_auth_rt"], (false, false));
            assert_eq!(rights["console_auth_startup"], (false, false));
            assert_eq!(
                rights["console_rt"],
                (identity.ends_with("_v1(uuid)"), false)
            );
        }
    }

    fn source_catalog_delta(before: &Value, after: &Value) {
        let old = routine_map(before);
        let new = routine_map(after);
        assert_eq!(new.len(), old.len() + 2);
        assert!(
            old.iter()
                .all(|(identity, record)| new.get(identity) == Some(record)),
            "source changed an existing routine/catalog dependency"
        );
        let additions: Vec<_> = new
            .keys()
            .filter(|identity| !old.contains_key(*identity))
            .cloned()
            .collect();
        assert_eq!(
            additions,
            vec![
                "public.account_company_provenance_lock_v1(uuid, uuid)",
                "public.account_company_provenance_v1(uuid)"
            ]
        );
        // Routines are checked in full above. Every other captured catalog field
        // must remain exact; nothing is removed from either custody serializer.
        let mut old_other = before.clone();
        old_other.as_object_mut().unwrap().remove("routines");
        let mut new_other = after.clone();
        new_other.as_object_mut().unwrap().remove("routines");
        assert!(
            old_other == new_other,
            "source changed non-routine security/catalog metadata"
        );
        exact_routines(after);
    }

    async fn execute(tx: &mut Transaction<'_, Postgres>, source: &'static str) {
        sqlx::raw_sql("SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='120s'; SET LOCAL jit=off")
            .execute(tx.as_mut()).await.unwrap();
        sqlx::raw_sql(sqlx::AssertSqlSafe(source))
            .execute(tx.as_mut())
            .await
            .unwrap();
        sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE")
            .execute(tx.as_mut())
            .await
            .unwrap();
    }

    async fn row_lock_state(connection: &mut PgConnection) -> String {
        sqlx::raw_sql(CLASSIFIER_SESSION)
            .execute(&mut *connection)
            .await
            .unwrap();
        sqlx::query_scalar(ROW_LOCK_STATE)
            .fetch_one(connection)
            .await
            .unwrap()
    }

    async fn controls(
        tx: &mut Transaction<'_, Postgres>,
        accepted: &Capture,
        accepted_catalog: &(String, Value),
        accepted_rows: &BTreeMap<String, String>,
    ) -> Value {
        let mut results = Vec::new();
        for (name, fault, capture_changes, namespace_changes) in [
            (
                "helper_runtime_acl",
                "GRANT EXECUTE ON FUNCTION public.account_company_provenance_lock_v1(uuid,uuid) TO console_rt",
                true,
                false,
            ),
            (
                "helper_setting",
                "ALTER FUNCTION public.account_company_provenance_lock_v1(uuid,uuid) SET row_security=off",
                true,
                false,
            ),
            (
                "helper_body",
                "CREATE OR REPLACE FUNCTION public.account_company_provenance_lock_v1(p_company uuid,p_group uuid) RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE SET search_path=pg_catalog,pg_temp SET row_security=on AS $fault$ BEGIN RETURN false; END; $fault$",
                true,
                false,
            ),
            (
                "overload",
                "CREATE FUNCTION public.account_company_provenance_v1(text) RETURNS text LANGUAGE sql IMMUTABLE AS 'SELECT ''UNKNOWN''::text'; ALTER FUNCTION public.account_company_provenance_v1(text) OWNER TO console_app; REVOKE ALL ON FUNCTION public.account_company_provenance_v1(text) FROM PUBLIC",
                true,
                true,
            ),
            (
                "unselected_invoker_family",
                "CREATE FUNCTION public.account_company_provenance_pair_unknown_v1() RETURNS integer LANGUAGE sql IMMUTABLE SECURITY INVOKER AS 'SELECT 1'; ALTER FUNCTION public.account_company_provenance_pair_unknown_v1() OWNER TO console_app; REVOKE ALL ON FUNCTION public.account_company_provenance_pair_unknown_v1() FROM PUBLIC",
                false,
                true,
            ),
            (
                "partial_presence",
                "DROP FUNCTION public.account_company_provenance_v1(uuid)",
                true,
                true,
            ),
        ] {
            let mut attempt = tx.begin().await.unwrap();
            let outcome = AssertUnwindSafe(async {
                execute(&mut attempt, fault).await;
                let observed = capture(attempt.as_mut(), CAPTURE).await;
                assert_eq!(observed != *accepted, capture_changes, "capture visibility control: {name}");
                let family = namespace(attempt.as_mut()).await;
                assert_eq!(family != exact_namespace(), namespace_changes, "namespace visibility control: {name}");
                let corrupted_catalog = catalog(attempt.as_mut()).await;
                assert!(corrupted_catalog != *accepted_catalog, "raw catalog omitted mutation: {name}");
                assert!(rows(attempt.as_mut()).await == *accepted_rows, "metadata control changed business/history rows");
                json!({"name":name,"capture_changed":capture_changes,"namespace_changed":namespace_changes,
                    "observed_capture_sha256":observed.sha256,"observed_namespace":family,
                    "observed_catalog_text_sha256":digest(&corrupted_catalog.0)})
            }).catch_unwind().await;
            attempt.rollback().await.unwrap();
            assert!(
                capture(tx.as_mut(), CAPTURE).await == *accepted,
                "control rollback changed complete successor capture"
            );
            assert!(
                catalog(tx.as_mut()).await == *accepted_catalog,
                "control rollback changed raw catalog"
            );
            assert_eq!(namespace(tx.as_mut()).await, exact_namespace());
            assert!(
                rows(tx.as_mut()).await == *accepted_rows,
                "control rollback changed durable rows"
            );
            match outcome {
                Ok(result) => results.push(result),
                Err(panic) => std::panic::resume_unwind(panic),
            }
        }
        json!(results)
    }

    #[sqlx::test(migrations = false)]
    async fn captures_actual_plain_and_observer_successors_preserves_rows_and_rolls_back(
        pool: PgPool,
    ) {
        let pins = source_pins();
        let mut initial = pool.begin().await.unwrap();
        marked(initial.as_mut(), true).await;
        initial.rollback().await.unwrap();
        let mut identity_tx = pool.begin().await.unwrap();
        let database_identity: Value = sqlx::query_scalar("SELECT jsonb_build_object('database',current_database(),'database_oid',(SELECT oid FROM pg_catalog.pg_database WHERE datname=current_database()),'session_user',session_user,'current_user',current_user,'fixture_marker',current_setting('console.sqlx_test_bootstrap',true),'system_identifier',(pg_catalog.pg_control_system()).system_identifier::text,'server_version_num',current_setting('server_version_num'),'server_encoding',current_setting('server_encoding'),'collation',(SELECT datcollate FROM pg_catalog.pg_database WHERE datname=current_database()),'ctype',(SELECT datctype FROM pg_catalog.pg_database WHERE datname=current_database()),'tls',COALESCE((SELECT ssl FROM pg_catalog.pg_stat_ssl WHERE pid=pg_backend_pid()),false),'fsync',current_setting('fsync'),'synchronous_commit',current_setting('synchronous_commit'),'full_page_writes',current_setting('full_page_writes'))")
            .fetch_one(identity_tx.as_mut()).await.unwrap();
        identity_tx.rollback().await.unwrap();
        // Actual production migrations plus jobs owner; historical migration data
        // remains present. No new seeds or UI/business acceptance are claimed.
        prepare_http_database_staging(&pool).await;
        let mut initial = pool.begin().await.unwrap();
        marked(initial.as_mut(), false).await;
        let ledger = applied_ledger(initial.as_mut()).await;
        let baseline_rows = rows(initial.as_mut()).await;
        let baseline_catalog = catalog(initial.as_mut()).await;
        assert!(namespace(initial.as_mut()).await.is_empty());
        let observer_absent: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_durability_observer') AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname='public' AND p.proname='console_durability_observation_v1')")
            .fetch_one(initial.as_mut()).await.unwrap();
        assert!(
            observer_absent,
            "dedicated capture cluster requires actually absent observer"
        );
        initial.rollback().await.unwrap();
        let mut packets = Vec::new();
        for variant in 0..2 {
            let mut tx = pool.begin().await.unwrap();
            let outcome = AssertUnwindSafe(async {
                sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='120s'; SET LOCAL search_path=pg_catalog,pg_temp")
                    .execute(tx.as_mut()).await.unwrap();
                marked(tx.as_mut(), false).await;
                // Match the existing dedicated two-variant capture coordination.
                let _: String = sqlx::query_scalar("SELECT oid::text FROM pg_catalog.pg_authid WHERE rolname='console_account_owner' FOR UPDATE")
                    .fetch_one(tx.as_mut()).await.unwrap();
                if variant == 1 { execute(&mut tx, OBSERVER).await; }
                for source in [ACCOUNT, CREDENTIALS, COMPANY, POLICY_INSTALLER, POLICY_V2, ROW_LOCK] { execute(&mut tx, source).await; }
                let old = capture(tx.as_mut(), PREDECESSOR_CAPTURE).await;
                assert_eq!(old.sha256, CORRECTED[variant], "canonical corrected predecessor differs; never normalize/reseal");
                assert_eq!(row_lock_state(tx.as_mut()).await, "native_people_directory.finalized");
                assert!(namespace(tx.as_mut()).await.is_empty());
                assert!(capture(tx.as_mut(), CAPTURE).await == old, "extended capture changed absent-source predecessor bytes");
                let before_rows = rows(tx.as_mut()).await;
                let before_catalog = catalog(tx.as_mut()).await;
                assert_eq!(applied_ledger(tx.as_mut()).await, ledger);
                marked(tx.as_mut(), false).await;
                execute(&mut tx, OWNER).await;
                let successor = capture(tx.as_mut(), CAPTURE).await;
                assert!(successor != old, "full capture omitted actual source installation");
                assert!(capture(tx.as_mut(), CAPTURE).await == successor, "successor raw capture is not deterministic");
                assert_eq!(namespace(tx.as_mut()).await, exact_namespace());
                let after_catalog = catalog(tx.as_mut()).await;
                source_catalog_delta(&before_catalog.1, &after_catalog.1);
                assert!(catalog(tx.as_mut()).await == after_catalog, "raw catalog is not deterministic");
                assert!(rows(tx.as_mut()).await == before_rows, "source DDL changed any business/history/ledger rows");
                assert_eq!(applied_ledger(tx.as_mut()).await, ledger);
                assert_eq!(row_lock_state(tx.as_mut()).await, "native_people_directory.profile_mismatch", "historical classifier must reject new source metadata");
                for name in ["account_company_provenance_v1","account_company_provenance_lock_v1"] {
                    assert_eq!(successor.snapshot["routines"].as_array().unwrap().iter().filter(|r| r["metadata"]["schema"]=="public" && r["metadata"]["name"]==name).count(), 1, "complete capture omitted exact ABI: {name}");
                }
                let control_evidence = controls(&mut tx, &successor, &after_catalog, &before_rows).await;
                json!({"schema":"console.company_provenance_pair_capture.v1","variant":if variant==0 {"plain"} else {"observer"},
                    "sources":pins,"database_identity":database_identity,"applied_ledger":ledger,"predecessor":old.record(),"successor":successor.record(),
                    "raw_catalog_before_text":before_catalog.0,"raw_catalog_after_text":after_catalog.0,
                    "raw_catalog_before_sha256":digest(&before_catalog.0),"raw_catalog_after_sha256":digest(&after_catalog.0),
                    "namespace":exact_namespace(),"business_rows":row_summary(&before_rows),"metadata_controls":control_evidence,
                    "business_history_unchanged":true,"successor_hash_is_observed_not_accepted":true,
                    "business_empty":false,"workflow_accepted":false,"production_qualified":false})
            }).catch_unwind().await;
            tx.rollback()
                .await
                .expect("all owner/source/observer DDL must roll back");
            let mut restored = pool.begin().await.unwrap();
            assert!(
                rows(restored.as_mut()).await == baseline_rows,
                "variant rollback changed complete durable rows"
            );
            assert!(
                catalog(restored.as_mut()).await == baseline_catalog,
                "variant rollback changed complete raw catalog"
            );
            assert_eq!(applied_ledger(restored.as_mut()).await, ledger);
            assert!(namespace(restored.as_mut()).await.is_empty());
            restored.rollback().await.unwrap();
            match outcome {
                Ok(mut packet) => {
                    packet["bootstrap_and_source_rollback_verified"] = json!(true);
                    packets.push(packet);
                }
                Err(panic) => std::panic::resume_unwind(panic),
            }
        }
        assert_eq!(packets.len(), 2);
        assert_ne!(
            packets[0]["successor"]["snapshot_sha256"],
            packets[1]["successor"]["snapshot_sha256"]
        );
        // Direct writes retain successful diagnostic records outside libtest's
        // print-macro capture. Publication failure invalidates this evidence run.
        let mut output = std::io::stderr().lock();
        for packet in packets {
            writeln!(&mut output, "COMPANY_PROVENANCE_PAIR_CAPTURE {packet}")
                .expect("paired capture record write failed");
        }
        writeln!(&mut output, "COMPANY_PROVENANCE_PAIR_CAPTURE_COMPLETE variants=2 controls=12 ledger231=verified historical230=verified rows=unchanged rollback=verified acceptance=not_claimed")
            .expect("paired capture completion write failed");
        output
            .flush()
            .expect("paired capture evidence flush failed");
    }
}
