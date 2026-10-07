// External source candidate for the adopted V5 protocol. Each leaf needs the
// root-owned disposable PostgreSQL lease. Observed hashes never grant authority.
mod native_org_unit_account_actor_complete_capture_measurements {
    use super::*;
    use sqlx::Connection as _;
    use std::io::Write as _;

    const COMPLETE_INPUT: &str =
        include_str!("../../../../ops/native-org-unit/account-actor-custody-capture-v1.sql");
    const COMPLETE_EXPORT: &str = include_str!(
        "../../../../ops/postgres-capture-native-org-unit-account-actor-v1-custody.sql"
    );
    const COMPLETE_SHA: &str = "f31c27207f3a9a5c7b1bf8c81149f679f2eec9abd5a14c1e72ca70a5b87f2eb9";
    const EXPANSION: &str = include_str!(
        "../../../../ops/postgres-native-org-unit-account-actor-expansion-v1-owner.sql"
    );
    const VALIDATION: &str = include_str!(
        "../../../../ops/postgres-native-org-unit-account-actor-validation-v1-owner.sql"
    );
    const READER_SESSION: &str = include_str!("../../src/account_custody_session.sql");
    const SUCCESSORS: [&str; 3] = [
        "ont_action_receipts_actor_protocol_v2",
        "org_unit_revisions_actor_protocol_v1",
        "org_unit_revisions_native_actor_v1",
    ];

    #[derive(Clone, Copy, Debug)]
    enum CompleteFixtureFamily {
        Closed76,
        Group83,
        Navigation83,
    }

    #[derive(Clone, Copy, Debug)]
    enum CompleteOwnerPhase {
        Expansion,
        Validation,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct CompleteObservation {
        text: String,
        sha256: String,
        valid: Option<bool>,
        snapshot: Value,
        historical76_text: String,
        historical83_text: String,
    }

    impl CompleteObservation {
        fn record(&self) -> Value {
            json!({"snapshot_text":self.text,"snapshot":self.snapshot,
                "snapshot_sha256":self.sha256,"complete_capture_valid":self.valid,
                "historical76_text":self.historical76_text,
                "historical83_text":self.historical83_text})
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    struct CompleteState {
        target: Target,
        portable: CompleteObservation,
        inverse_and_raw: Value,
        catalog: (String, Value),
        old_rows: BTreeMap<String, String>,
        ledger: Value,
        native: Value,
    }

    fn complete_source_pins() -> Value {
        let mut pins = navigation_finalizer_pins();
        for (name, source, expected) in [
            ("complete_input", COMPLETE_INPUT, COMPLETE_SHA),
            ("complete_export", COMPLETE_EXPORT, COMPLETE_SHA),
            (
                "actor_expansion_export",
                EXPANSION,
                "ed2e9df259bbba5b3fbfc9585a42eb210461f20e58cf6b28de6ceed5399391c9",
            ),
            (
                "actor_validation_export",
                VALIDATION,
                "61630ef6bae75543e1bcab0d3bb760cdbde40429b227df71b1a34285909f01b5",
            ),
        ] {
            assert_eq!(
                digest(source),
                expected,
                "unreviewed complete diagnostic source: {name}"
            );
            pins[name] = json!(expected);
        }
        assert_eq!(COMPLETE_INPUT, COMPLETE_EXPORT);
        assert_eq!(
            READER_SESSION,
            "SET LOCAL search_path=pg_catalog,pg_temp;\nSET LOCAL statement_timeout='3s';\nSET LOCAL jit=off;\n"
        );
        pins
    }

    fn complete_body() -> &'static str {
        assert_eq!(digest(COMPLETE_EXPORT), COMPLETE_SHA);
        COMPLETE_EXPORT
            .strip_suffix(";\n")
            .expect("exact capture terminator")
    }

    fn complete_text_query() -> String {
        format!(
            "SELECT snapshot::text,snapshot_sha256,complete_capture_valid,\
             (snapshot->'historical76')::text,(snapshot->'historical83')::text \
             FROM ({}) exact_complete_capture",
            complete_body()
        )
    }

    // This wrapper retains PostgreSQL's original JSONB text. The source query
    // executes once, under the same reader settings on both principals.
    async fn complete_read(connection: &mut PgConnection) -> CompleteObservation {
        sqlx::raw_sql(READER_SESSION)
            .execute(&mut *connection)
            .await
            .unwrap();
        let settings: (String, String, String) = sqlx::query_as(
            "SELECT current_setting('search_path'),current_setting('statement_timeout'),current_setting('jit')"
        ).fetch_one(&mut *connection).await.unwrap();
        assert_eq!(
            settings,
            ("pg_catalog, pg_temp".into(), "3s".into(), "off".into())
        );
        let query = complete_text_query();
        let start = Instant::now();
        let result: Vec<(String, String, Option<bool>, String, String)> =
            sqlx::query_as(sqlx::AssertSqlSafe(query))
                .fetch_all(&mut *connection).await
                .expect("PREREQUISITE: actual complete query must execute; SQL/auth/permissions/timeout are not semantic RED");
        assert!(
            start.elapsed() <= Duration::from_secs(3),
            "PREREQUISITE: one whole complete statement exceeded the reader deadline"
        );
        assert_eq!(result.len(), 1, "exactly one complete row required");
        let (text, sha256, valid, historical76_text, historical83_text) =
            result.into_iter().next().unwrap();
        assert_eq!(digest(&text), sha256);
        let snapshot: Value = serde_json::from_str(&text).unwrap();
        let keys = snapshot
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            keys,
            [
                "schema",
                "historical76",
                "historical83",
                "historical76_sha256",
                "historical83_sha256",
                "historical76_rights",
                "historical83_rights",
                "original73_rights",
                "supplemental"
            ]
            .into_iter()
            .collect()
        );
        assert_eq!(
            snapshot["schema"],
            "console.native_org_unit.account_actor_complete_capture.v1"
        );
        assert_eq!(snapshot["historical76_sha256"], digest(&historical76_text));
        assert_eq!(snapshot["historical83_sha256"], digest(&historical83_text));
        CompleteObservation {
            text,
            sha256,
            valid,
            snapshot,
            historical76_text,
            historical83_text,
        }
    }

    async fn complete_array_refusal(
        connection: &mut PgConnection,
        missing_array_link: bool,
    ) -> Value {
        sqlx::raw_sql(READER_SESSION)
            .execute(&mut *connection)
            .await
            .unwrap();
        let start = Instant::now();
        let result: Result<Vec<(String, String, Option<bool>, String, String)>, sqlx::Error> =
            sqlx::query_as(sqlx::AssertSqlSafe(complete_text_query()))
                .fetch_all(&mut *connection)
                .await;
        assert!(
            start.elapsed() <= Duration::from_secs(3),
            "PREREQUISITE: negative capture deadline"
        );
        match result {
            Ok(rows) => {
                assert_eq!(rows.len(), 1);
                let (text, sha256, valid, _, _) = &rows[0];
                assert_eq!(digest(text), *sha256);
                assert_eq!(
                    *valid,
                    Some(false),
                    "actual general-array corruption was accepted"
                );
                let snapshot: Value = serde_json::from_str(text).unwrap();
                assert_eq!(snapshot["supplemental"]["resolution_valid"], false);
                json!({"complete_capture_valid":valid,"resolution_valid":false})
            }
            Err(error) => {
                // The pinned query has a literal pg_catalog.uuid[]::regtype.
                // Removing the base's array link can make that exact literal
                // unresolvable. This deliberately induced 42704 is refusal,
                // never a product RED, an accepted snapshot or a fallback.
                let database = error
                    .as_database_error()
                    .expect("PREREQUISITE: transport/auth failure is not an array refusal");
                assert!(missing_array_link, "PREREQUISITE: unexpected capture error");
                assert_eq!(database.code().as_deref(), Some("42704"));
                assert_eq!(
                    database.message(),
                    "type \"pg_catalog.uuid[]\" does not exist"
                );
                json!({"capture_refused":true,"sqlstate":"42704",
                    "reason":"deliberately removed actual UUID canonical array link",
                    "accepted_snapshot":false,"product_RED":false})
            }
        }
    }

    fn complete_positive(observation: &CompleteObservation, family: CompleteFixtureFamily) {
        assert_eq!(
            observation.valid,
            Some(true),
            "complete capture refused actual phase"
        );
        let snapshot = &observation.snapshot;
        assert_eq!(snapshot["historical76_rights"], false);
        match family {
            CompleteFixtureFamily::Closed76 => assert_eq!(snapshot["original73_rights"], true),
            _ => assert_eq!(snapshot["historical83_rights"], true),
        }
        for path in [
            "resolution_valid",
            "org_namespaces",
            "group_namespaces",
            "org_startup_denial",
        ] {
            let field = &snapshot["supplemental"][path];
            assert_eq!(
                if path == "resolution_valid" {
                    field
                } else {
                    &field["valid"]
                },
                &json!(true)
            );
        }
        let checks = snapshot["supplemental"]["org_startup_denial"]["checks"]
            .as_array()
            .unwrap();
        assert!(checks.iter().all(|row| row["allowed"] == false));
        assert_eq!(
            checks.iter().filter(|row| row["kind"] == "table").count(),
            24
        );
        assert!([80, 88].contains(&checks.iter().filter(|row| row["kind"] == "column").count()));
    }

    // The administrator's inverse is a diagnostic suffix of the pinned actual
    // CTE prefix. It uses real allocated RI names, separately from semantic RI
    // identities. It is never part of the portable query or a reader capability.
    fn complete_address_prefix() -> &'static str {
        const BOUNDARY: &str = "), row_type_valid AS (\n";
        assert_eq!(COMPLETE_EXPORT.matches(BOUNDARY).count(), 1);
        COMPLETE_EXPORT.split_once(BOUNDARY).unwrap().0
    }

    fn complete_inverse_sql() -> String {
        let mut union = Vec::new();
        for table in [
            "pg_class",
            "pg_proc",
            "pg_type",
            "pg_collation",
            "pg_attrdef",
            "pg_constraint",
            "pg_trigger",
            "pg_rewrite",
            "pg_policy",
            "pg_namespace",
            "pg_tablespace",
            "pg_language",
            "pg_extension",
            "pg_am",
            "pg_opclass",
            "pg_opfamily",
            "pg_operator",
        ] {
            let raw = if table == "pg_class" {
                "to_jsonb(r)-ARRAY['relpages','reltuples','relallvisible','relallfrozen','relfrozenxid','relminmxid']"
            } else {
                "to_jsonb(r)"
            };
            union.push(format!(
                "SELECT a.classid,a.objid,a.objsubid,{raw} AS raw,r.xmin::text AS xmin,r.ctid::text AS ctid \
                 FROM addresses a JOIN pg_catalog.{table} r ON a.classid='pg_catalog.{table}'::regclass \
                 AND r.oid=a.objid WHERE a.objsubid=0"));
        }
        union.push("SELECT a.classid,a.objid,a.objsubid,to_jsonb(r)-'rolpassword',\
            custody.xmin::text,custody.ctid::text FROM addresses a \
            JOIN pg_catalog.pg_roles r ON a.classid='pg_catalog.pg_authid'::regclass AND r.oid=a.objid \
            JOIN pg_catalog.pg_authid custody ON custody.oid=r.oid WHERE a.objsubid=0".into());
        union.push("SELECT a.classid,a.objid,a.objsubid,to_jsonb(r),r.xmin::text,r.ctid::text \
            FROM addresses a JOIN pg_catalog.pg_attribute r ON a.classid='pg_catalog.pg_class'::regclass \
            AND r.attrelid=a.objid AND r.attnum=a.objsubid WHERE a.objsubid<>0".into());
        format!(
            r#"{})
, inverse AS MATERIALIZED (
 SELECT a.classid,a.objid,a.objsubid,n.native_address,s.address,s.valid,s.ri_flags,
 inverse.classid AS inverse_classid,inverse.objid AS inverse_objid,inverse.objsubid AS inverse_objsubid
 FROM addresses a JOIN native_addresses n USING(classid,objid,objsubid)
 JOIN stable_addresses s USING(classid,objid,objsubid)
 CROSS JOIN LATERAL pg_catalog.pg_get_object_address(n.native_type,n.native_names,n.native_args) inverse
), raw_objects AS MATERIALIZED ({})
SELECT jsonb_build_object(
 'address_count',(SELECT count(*) FROM addresses),
 'inverse',(SELECT jsonb_agg(jsonb_build_object('tuple',jsonb_build_array(classid::text,objid::text,objsubid),
   'inverse_tuple',jsonb_build_array(inverse_classid::text,inverse_objid::text,inverse_objsubid),
   'native',native_address,'portable',address,'valid',valid,'RI_flags',ri_flags)
   ORDER BY classid,objid,objsubid) FROM inverse),
 'objects',(SELECT jsonb_agg(jsonb_build_object('tuple',jsonb_build_array(classid::text,objid::text,objsubid),
   'raw',raw,'xmin',xmin,'ctid',ctid) ORDER BY classid,objid,objsubid) FROM raw_objects),
 'indexes',(SELECT COALESCE(jsonb_agg(to_jsonb(i)||jsonb_build_object('xmin',i.xmin::text,'ctid',i.ctid::text)
   ORDER BY i.indexrelid),'[]'::jsonb) FROM pg_catalog.pg_index i
   WHERE i.indexrelid IN (SELECT indexrelid FROM selected_indexes)),
 'comments',(SELECT COALESCE(jsonb_agg(to_jsonb(d)||jsonb_build_object('xmin',d.xmin::text,'ctid',d.ctid::text)
   ORDER BY d.classoid,d.objoid,d.objsubid),'[]'::jsonb) FROM pg_catalog.pg_description d
   WHERE EXISTS(SELECT 1 FROM metadata_m m WHERE (m.classid,m.objid,m.objsubid)=(d.classoid,d.objoid,d.objsubid))),
 'shared_comments',(SELECT COALESCE(jsonb_agg(to_jsonb(d)||jsonb_build_object('xmin',d.xmin::text,'ctid',d.ctid::text)
   ORDER BY d.classoid,d.objoid),'[]'::jsonb) FROM pg_catalog.pg_shdescription d
   WHERE EXISTS(SELECT 1 FROM metadata_m m WHERE m.classid='pg_catalog.pg_authid'::regclass
    AND (m.classid,m.objid,m.objsubid)=(d.classoid,d.objoid,0))),
 'ordinary_edges',(SELECT COALESCE(jsonb_agg(to_jsonb(d)||jsonb_build_object('xmin',d.xmin::text,'ctid',d.ctid::text)
   ORDER BY to_jsonb(d)::text COLLATE "C",d.ctid),'[]'::jsonb) FROM pg_catalog.pg_depend d
   WHERE EXISTS(SELECT 1 FROM ordinary_edges e WHERE
    (e.classid,e.objid,e.objsubid,e.refclassid,e.refobjid,e.refobjsubid,e.deptype)=
    (d.classid,d.objid,d.objsubid,d.refclassid,d.refobjid,d.refobjsubid,d.deptype))),
 'shared_edges',(SELECT COALESCE(jsonb_agg(to_jsonb(d)||jsonb_build_object('xmin',d.xmin::text,'ctid',d.ctid::text)
   ORDER BY to_jsonb(d)::text COLLATE "C",d.ctid),'[]'::jsonb) FROM pg_catalog.pg_shdepend d
   WHERE EXISTS(SELECT 1 FROM shared_edges e WHERE
    (e.dbid,e.classid,e.objid,e.objsubid,e.refclassid,e.refobjid,e.deptype)=
    (d.dbid,d.classid,d.objid,d.objsubid,d.refclassid,d.refobjid,d.deptype)))
)::text"#,
            complete_address_prefix(),
            union.join(" UNION ALL ")
        )
    }

    async fn complete_inverse_and_raw(connection: &mut PgConnection) -> Value {
        bounds(connection).await;
        let raw: String = sqlx::query_scalar(sqlx::AssertSqlSafe(complete_inverse_sql()))
            .fetch_one(&mut *connection)
            .await
            .expect("PREREQUISITE: administrator must actually invert every native typed address");
        let audit: Value = serde_json::from_str(&raw).unwrap();
        let count = audit["address_count"].as_u64().unwrap() as usize;
        assert!(count > 0);
        for field in ["inverse", "objects"] {
            let entries = audit[field].as_array().unwrap();
            assert_eq!(entries.len(), count, "address/raw cardinality mismatch");
            assert_eq!(
                entries
                    .iter()
                    .map(|entry| entry["tuple"].to_string())
                    .collect::<BTreeSet<_>>()
                    .len(),
                count,
                "duplicate actual tuple"
            );
        }
        let inverse = audit["inverse"].as_array().unwrap();
        assert_eq!(
            inverse
                .iter()
                .map(|entry| entry["portable"].to_string())
                .collect::<BTreeSet<_>>()
                .len(),
            count,
            "ambiguous stable identity"
        );
        for entry in inverse {
            assert_eq!(
                entry["tuple"], entry["inverse_tuple"],
                "native inverse did not return actual tuple"
            );
            assert_eq!(entry["valid"], true, "unresolved tuple cannot be accepted");
            assert!(entry["portable"].is_object());
            if entry["portable"]["type"] == "foreign key RI trigger" {
                assert_eq!(entry["native"]["type"], "trigger");
                assert_eq!(entry["RI_flags"]["generated_name_valid"], true);
                assert!(
                    entry["native"]["object_names"][2]
                        .as_str()
                        .unwrap()
                        .starts_with("RI_ConstraintTrigger_")
                );
            }
        }
        audit
    }

    async fn complete_business(
        connection: &mut PgConnection,
        staged: bool,
    ) -> BTreeMap<String, String> {
        let mut census = rows(connection).await;
        let old: String = sqlx::query_scalar(
            "SELECT COALESCE(jsonb_agg(to_jsonb(old) ORDER BY to_jsonb(old)::text COLLATE \"C\"),'[]'::jsonb)::text \
             FROM (SELECT org_id,id,org_unit_id,version,command_id,actor_id,payload_digest,attributes,receipt,created_at \
               FROM public.org_unit_revisions) old")
            .fetch_one(&mut *connection).await.unwrap();
        let key = "[\"public\",\"org_unit_revisions\"]";
        if !staged {
            assert_eq!(census[key], old);
        }
        assert!(census.insert(key.into(), old).is_some());
        if staged {
            let compatible: bool = sqlx::query_scalar(
                "SELECT NOT EXISTS(SELECT 1 FROM public.org_unit_revisions \
                 WHERE actor_kind IS DISTINCT FROM 'USER' OR actor_account_id IS NOT NULL)",
            )
            .fetch_one(&mut *connection)
            .await
            .unwrap();
            assert!(
                compatible,
                "existing actor bytes must retain USER/NULL attribution"
            );
        }
        census
    }

    async fn complete_native_census(connection: &mut PgConnection) -> Value {
        let actual: (i64, bool, i64, i64, i64, i64) = sqlx::query_as(
            "WITH native AS (SELECT id FROM public.organizations WHERE origin_account_id IS NOT NULL) \
             SELECT (SELECT count(*) FROM native),\
               (SELECT count(*)>0 AND bool_and(public.account_company_provenance_v1(id)=\
                 CASE WHEN origin_account_id IS NULL THEN 'LEGACY' ELSE 'NATIVE' END) FROM public.organizations),\
               (SELECT count(*) FROM public.org_units WHERE org_id IN(SELECT id FROM native)),\
               (SELECT count(*) FROM public.org_unit_revisions WHERE org_id IN(SELECT id FROM native)),\
               (SELECT count(*) FROM public.org_unit_source_bindings WHERE org_id IN(SELECT id FROM native)),\
               (SELECT count(*) FROM public.ont_action_command_receipts WHERE org_id IN(SELECT id FROM native) \
                 AND (owner='org_unit' OR target IN('organization.create_org_unit','organization.revise_org_unit') \
                   OR receipt->>'target' IN('organization.create_org_unit','organization.revise_org_unit')))")
            .fetch_one(connection).await.unwrap();
        assert!(actual.1, "actual accepted Company provenance required");
        assert_eq!((actual.2, actual.3, actual.4, actual.5), (0, 0, 0, 0));
        json!({"native_company_count":actual.0,"all_company_provenance_valid":actual.1,
            "native_org_effect_counts":[actual.2,actual.3,actual.4,actual.5]})
    }

    async fn complete_state(
        connection: &mut PgConnection,
        family: CompleteFixtureFamily,
        staged: bool,
    ) -> CompleteState {
        bounds(connection).await;
        let target = target(connection).await;
        let ledger = applied_ledger(connection).await;
        let native = complete_native_census(connection).await;
        let old_rows = complete_business(connection, staged).await;
        let catalog = catalog(connection).await;
        let portable = complete_read(connection).await;
        complete_positive(&portable, family);
        let denial_checks = portable.snapshot["supplemental"]["org_startup_denial"]["checks"]
            .as_array()
            .unwrap();
        assert_eq!(
            denial_checks
                .iter()
                .filter(|row| row["kind"] == "column")
                .count(),
            if staged { 88 } else { 80 }
        );
        let inverse_and_raw = complete_inverse_and_raw(connection).await;
        CompleteState {
            target,
            portable,
            inverse_and_raw,
            catalog,
            old_rows,
            ledger,
            native,
        }
    }

    async fn complete_fixture(pool: &PgPool, family: CompleteFixtureFamily, variant: usize) {
        assert!(variant < 2);
        match family {
            CompleteFixtureFamily::Closed76 => {
                let before = predecessor(pool, variant).await;
                let closed = group_closed_fixture(pool, &before, variant).await;
                serving_restored(pool, variant, &closed).await;
            }
            _ => {
                let installed = navigation_original_fixture(pool, variant).await;
                if matches!(family, CompleteFixtureFamily::Navigation83) {
                    let mut admin = direct(pool).await;
                    let mut correction = begin_protocol(&mut admin, &installed.target).await;
                    let outcome = AssertUnwindSafe(async {
                        bounds(correction.as_mut()).await;
                        navigation_finalize(correction.as_mut()).await;
                        navigation_snapshot(correction.as_mut(), true, variant).await
                    })
                    .catch_unwind()
                    .await;
                    match outcome {
                        Ok(corrected) => {
                            correction.commit().await.unwrap();
                            admin.close().await.unwrap();
                            navigation_restored(pool, &corrected, true, variant).await;
                        }
                        Err(panic) => {
                            correction.rollback().await.unwrap();
                            admin.close().await.unwrap();
                            navigation_restored(pool, &installed, false, variant).await;
                            std::panic::resume_unwind(panic);
                        }
                    }
                }
            }
        }
    }

    async fn complete_visible(
        pool: &PgPool,
        family: CompleteFixtureFamily,
        staged: bool,
    ) -> CompleteState {
        let mut admin = direct(pool).await;
        let mut tx = sqlx::Connection::begin(&mut admin).await.unwrap();
        let state = complete_state(tx.as_mut(), family, staged).await;
        tx.rollback().await.unwrap();
        admin.close().await.unwrap();
        // Six ancestors lead to deployment_operator_designation::startup.
        // Its accepted helper verifies the externally provisioned actual LOGIN;
        // direct() is deliberately not used for the restricted principal.
        let startup = super::super::super::super::super::super::startup(pool).await;
        let mut reader = PgConnection::connect_with(&startup.connect_options())
            .await
            .expect("PREREQUISITE: fresh actual startup connection");
        let identity: (String, String, String, i64, bool, bool) = sqlx::query_as(
            "SELECT session_user::text,current_user::text,current_database()::text,\
             (SELECT oid::bigint FROM pg_catalog.pg_database WHERE datname=current_database()),\
             (SELECT count(*)=0 FROM pg_catalog.pg_auth_members m JOIN pg_catalog.pg_roles r \
               ON r.oid=m.member OR r.oid=m.roleid WHERE r.rolname='console_auth_startup'),\
             (SELECT count(*)=3 AND bool_and(NOT pg_catalog.has_schema_privilege(session_user,n.oid,'USAGE')) \
               FROM pg_catalog.pg_namespace n WHERE n.nspname IN('ontology_api','leave_api','ont_policy_api'))")
            .fetch_one(&mut reader).await.unwrap();
        assert_eq!(
            identity,
            (
                "console_auth_startup".into(),
                "console_auth_startup".into(),
                state.target.database.clone(),
                state.target.database_oid,
                true,
                true
            )
        );
        let mut read = sqlx::Connection::begin(&mut reader).await.unwrap();
        let actual = complete_read(read.as_mut()).await;
        complete_positive(&actual, family);
        assert_eq!(
            actual, state.portable,
            "fresh startup bytes must equal immediately preceding admin bytes"
        );
        read.rollback().await.unwrap();
        reader.close().await.unwrap();
        startup.close().await;
        state
    }

    async fn complete_locks(connection: &mut PgConnection, phase: CompleteOwnerPhase) -> Value {
        bounds(connection).await;
        let (company, changes, company_readback, changes_readback) = match phase {
            CompleteOwnerPhase::Expansion => (
                "SHARE ROW EXCLUSIVE",
                "ACCESS EXCLUSIVE",
                "ShareRowExclusiveLock",
                "AccessExclusiveLock",
            ),
            CompleteOwnerPhase::Validation => (
                "ROW SHARE",
                "SHARE UPDATE EXCLUSIVE",
                "RowShareLock",
                "ShareUpdateExclusiveLock",
            ),
        };
        let sql = format!(
            "LOCK TABLE ONLY public.company_actors IN {company} MODE; \
            LOCK TABLE ONLY public.ont_action_command_receipts IN {changes} MODE; \
            LOCK TABLE ONLY public.org_unit_revisions IN {changes} MODE"
        );
        sqlx::raw_sql(sqlx::AssertSqlSafe(sql))
            .execute(&mut *connection)
            .await
            .unwrap();
        let actual: Vec<(String, String)> = sqlx::query_as(
            "SELECT c.relname::text,l.mode::text FROM pg_catalog.pg_locks l \
             JOIN pg_catalog.pg_class c ON c.oid=l.relation JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace \
             WHERE l.pid=pg_backend_pid() AND l.locktype='relation' AND l.granted AND n.nspname='public' \
               AND (l.mode NOT IN('AccessShareLock','RowShareLock') \
                 OR c.relname='company_actors' AND l.mode='RowShareLock') \
             ORDER BY c.relname COLLATE \"C\",l.mode COLLATE \"C\"")
            .fetch_all(&mut *connection).await.unwrap();
        assert_eq!(
            actual,
            vec![
                ("_sqlx_migrations".into(), "ShareLock".into()),
                ("company_actors".into(), company_readback.into()),
                (
                    "ont_action_command_receipts".into(),
                    changes_readback.into()
                ),
                ("org_unit_revisions".into(), changes_readback.into())
            ]
        );
        json!(actual)
    }

    fn complete_record_map<'a>(audit: &'a Value, field: &str) -> BTreeMap<String, &'a Value> {
        let array = audit[field].as_array().unwrap();
        let map: BTreeMap<_, _> = array
            .iter()
            .map(|entry| (entry["tuple"].to_string(), entry))
            .collect();
        assert_eq!(map.len(), array.len());
        map
    }

    fn complete_preserved_rows(before: &CompleteState, after: &CompleteState) {
        assert_eq!(before.target, after.target);
        assert_eq!(before.old_rows, after.old_rows);
        assert_eq!(before.ledger, after.ledger);
        assert_eq!(before.native, after.native);
        assert_ne!(before.portable.sha256, after.portable.sha256);
    }

    fn complete_expansion_delta(before: &CompleteState, after: &CompleteState) {
        complete_preserved_rows(before, after);
        let old = complete_record_map(&before.inverse_and_raw, "objects");
        let new = complete_record_map(&after.inverse_and_raw, "objects");
        let mut changed_relation_names = BTreeSet::new();
        for (identity, record) in &old {
            let actual = new
                .get(identity)
                .expect("expansion removed captured object");
            let name = record["raw"]["relname"].as_str();
            if record["tuple"][2] == 0
                && matches!(
                    name,
                    Some("org_unit_revisions" | "ont_action_command_receipts")
                )
            {
                let mut expected = record["raw"].clone();
                expected["relchecks"] = json!(expected["relchecks"].as_i64().unwrap() + 1);
                if name == Some("org_unit_revisions") {
                    expected["relnatts"] = json!(expected["relnatts"].as_i64().unwrap() + 2);
                }
                assert_eq!(
                    actual["raw"], expected,
                    "undeclared existing relation change"
                );
                assert_ne!(
                    (record["xmin"].clone(), record["ctid"].clone()),
                    (actual["xmin"].clone(), actual["ctid"].clone())
                );
                changed_relation_names.insert(name.unwrap());
            } else {
                assert_eq!(
                    *actual, *record,
                    "expansion altered an existing captured tuple"
                );
            }
        }
        assert_eq!(
            changed_relation_names,
            ["org_unit_revisions", "ont_action_command_receipts"]
                .into_iter()
                .collect()
        );
        let added: Vec<_> = new
            .iter()
            .filter(|(key, _)| !old.contains_key(*key))
            .map(|(_, row)| *row)
            .collect();
        let attributes: Vec<_> = added
            .iter()
            .filter(|row| row["raw"].get("attname").is_some())
            .collect();
        let constraints: Vec<_> = added
            .iter()
            .filter(|row| row["raw"].get("conname").is_some())
            .collect();
        let triggers: Vec<_> = added
            .iter()
            .filter(|row| row["raw"].get("tgname").is_some())
            .collect();
        let defaults: Vec<_> = added
            .iter()
            .filter(|row| row["raw"].get("adbin").is_some())
            .collect();
        assert_eq!(
            (
                added.len(),
                attributes.len(),
                constraints.len(),
                triggers.len(),
                defaults.len()
            ),
            (11, 2, 4, 4, 1)
        );
        assert_eq!(
            attributes
                .iter()
                .map(|row| row["raw"]["attname"].as_str().unwrap())
                .collect::<BTreeSet<_>>(),
            ["actor_kind", "actor_account_id"].into_iter().collect()
        );
        assert_eq!(
            constraints
                .iter()
                .map(|row| row["raw"]["conname"].as_str().unwrap())
                .collect::<BTreeSet<_>>(),
            SUCCESSORS
                .into_iter()
                .chain(["org_unit_revisions_actor_kind_not_null"])
                .collect()
        );
        let fk = constraints
            .iter()
            .find(|row| row["raw"]["conname"] == "org_unit_revisions_native_actor_v1")
            .unwrap();
        assert!(triggers.iter().all(|row| row["raw"]["tgisinternal"] == true
            && row["raw"]["tgconstraint"] == fk["raw"]["oid"]));
        let added_ids: BTreeSet<_> = added.iter().map(|row| row["tuple"].to_string()).collect();
        for field in [
            "ordinary_edges",
            "shared_edges",
            "comments",
            "shared_comments",
            "indexes",
        ] {
            let prior = before.inverse_and_raw[field].as_array().unwrap();
            let current = after.inverse_and_raw[field].as_array().unwrap();
            let mut multiplicities: BTreeMap<String, usize> = BTreeMap::new();
            for row in current {
                *multiplicities.entry(row.to_string()).or_default() += 1;
            }
            for row in prior {
                let count = multiplicities.get_mut(&row.to_string()).expect(
                    "expansion changed/removed existing raw edge/comment/index or MVCC witness",
                );
                assert!(
                    *count > 0,
                    "expansion removed an existing duplicate metadata row"
                );
                *count -= 1;
            }
            let mut remaining = Vec::new();
            for (text, count) in multiplicities {
                let row: Value = serde_json::from_str(&text).unwrap();
                remaining.extend(std::iter::repeat_n(row, count));
            }
            if matches!(field, "shared_comments" | "indexes") {
                assert!(remaining.is_empty());
            }
            for row in &remaining {
                let identity = if field == "comments" {
                    json!([
                        row["classoid"].as_str().unwrap(),
                        row["objoid"].as_str().unwrap(),
                        row["objsubid"]
                    ])
                } else {
                    json!([
                        row["classid"].as_str().unwrap(),
                        row["objid"].as_str().unwrap(),
                        row["objsubid"]
                    ])
                };
                assert!(
                    added_ids.contains(&identity.to_string()),
                    "new metadata edge/comment belongs to an undeclared object"
                );
            }
            if field == "comments" {
                assert_eq!(remaining.len(), 2);
            }
        }
        for (key, value) in before.catalog.1.as_object().unwrap() {
            if !["relations", "attributes", "constraints", "triggers"].contains(&key.as_str()) {
                assert_eq!(
                    after.catalog.1[key], *value,
                    "expansion changed unrelated security metadata: {key}"
                );
            }
        }
        // The broader unchanged catalog helper also contains objects outside
        // the finite portable roster. Preserve those rows independently.
        for (field, growth) in [
            ("relations", 0),
            ("attributes", 2),
            ("constraints", 4),
            ("triggers", 4),
        ] {
            let identity = |row: &Value| {
                if field == "attributes" {
                    json!([row["attrelid"], row["attnum"]]).to_string()
                } else {
                    row["oid"].to_string()
                }
            };
            let prior = before.catalog.1[field].as_array().unwrap();
            let current = after.catalog.1[field].as_array().unwrap();
            let lookup: BTreeMap<_, _> = current.iter().map(|row| (identity(row), row)).collect();
            assert_eq!(
                lookup.len(),
                current.len(),
                "duplicate raw catalog identity"
            );
            assert_eq!(current.len(), prior.len() + growth);
            for row in prior {
                let mut expected = row.clone();
                if field == "relations"
                    && ["org_unit_revisions", "ont_action_command_receipts"]
                        .contains(&row["relname"].as_str().unwrap())
                {
                    expected["relchecks"] = json!(row["relchecks"].as_i64().unwrap() + 1);
                    if row["relname"] == "org_unit_revisions" {
                        expected["relnatts"] = json!(row["relnatts"].as_i64().unwrap() + 2);
                    }
                }
                assert_eq!(
                    lookup.get(&identity(row)).copied(),
                    Some(&expected),
                    "expansion altered existing broad catalog row: {field}"
                );
            }
        }
        // Complete source predicates independently check phase columns, missing
        // value/default/comments, FK keys/index/RI flags and startup denials.
    }

    fn complete_validation_delta(before: &CompleteState, after: &CompleteState) {
        complete_preserved_rows(before, after);
        let mut expected = before.catalog.1.clone();
        let mut flipped = BTreeSet::new();
        for row in expected["constraints"].as_array_mut().unwrap() {
            let name = row["conname"].as_str().unwrap().to_owned();
            if SUCCESSORS.contains(&name.as_str()) {
                assert_eq!(row["convalidated"], false);
                row["convalidated"] = json!(true);
                flipped.insert(name);
            }
        }
        assert_eq!(flipped, SUCCESSORS.into_iter().map(str::to_owned).collect());
        assert_eq!(after.catalog.1, expected);
        let old = complete_record_map(&before.inverse_and_raw, "objects");
        let new = complete_record_map(&after.inverse_and_raw, "objects");
        assert_eq!(
            old.keys().collect::<Vec<_>>(),
            new.keys().collect::<Vec<_>>()
        );
        let mut raw_flipped = BTreeSet::new();
        for (key, prior) in old {
            let actual = new[&key];
            let name = prior["raw"]["conname"].as_str();
            if name.is_some_and(|name| SUCCESSORS.contains(&name)) {
                let mut raw = prior["raw"].clone();
                assert_eq!(raw["convalidated"], false);
                raw["convalidated"] = json!(true);
                assert_eq!(actual["raw"], raw);
                assert_ne!(
                    (prior["xmin"].clone(), prior["ctid"].clone()),
                    (actual["xmin"].clone(), actual["ctid"].clone())
                );
                raw_flipped.insert(name.unwrap());
            } else {
                assert_eq!(actual, prior);
            }
        }
        assert_eq!(raw_flipped, SUCCESSORS.into_iter().collect());
        for field in [
            "inverse",
            "indexes",
            "comments",
            "shared_comments",
            "ordinary_edges",
            "shared_edges",
        ] {
            assert_eq!(before.inverse_and_raw[field], after.inverse_and_raw[field]);
        }
    }

    async fn complete_attempt(
        pool: &PgPool,
        before: &CompleteState,
        family: CompleteFixtureFamily,
        phase: CompleteOwnerPhase,
        commit: bool,
    ) -> (CompleteState, Value) {
        let staged_before = matches!(phase, CompleteOwnerPhase::Validation);
        let mut admin = direct(pool).await;
        let mut tx = begin_protocol(&mut admin, &before.target).await;
        let outcome = AssertUnwindSafe(async {
            let preflight = complete_state(tx.as_mut(), family, staged_before).await;
            assert_eq!(
                &preflight, before,
                "fresh precondition must reproduce full previous state"
            );
            let locks = complete_locks(tx.as_mut(), phase).await;
            let locked = complete_state(tx.as_mut(), family, staged_before).await;
            assert_eq!(&locked, before);
            bounds(tx.as_mut()).await;
            assert!(matches_target(tx.as_mut(), &before.target).await);
            assert_eq!(applied_ledger(tx.as_mut()).await, before.ledger);
            // This must remain the last settings operation before owner DDL.
            bounds(tx.as_mut()).await;
            let start = Instant::now();
            sqlx::raw_sql(match phase {
                CompleteOwnerPhase::Expansion => EXPANSION,
                CompleteOwnerPhase::Validation => VALIDATION,
            })
            .execute(tx.as_mut())
            .await
            .unwrap();
            let elapsed = start.elapsed();
            assert!(elapsed <= Duration::from_secs(60));
            sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE")
                .execute(tx.as_mut())
                .await
                .unwrap();
            let after = complete_state(tx.as_mut(), family, true).await;
            match phase {
                CompleteOwnerPhase::Expansion => complete_expansion_delta(before, &after),
                CompleteOwnerPhase::Validation => complete_validation_delta(before, &after),
            }
            (
                after,
                json!({"phase":format!("{phase:?}"),"explicit_locks":locks,
                "owner_ddl_elapsed_us":elapsed.as_micros(),"committed_disposable_fixture":commit}),
            )
        })
        .catch_unwind()
        .await;
        match outcome {
            Ok(value) => {
                if commit {
                    tx.commit().await.unwrap();
                } else {
                    tx.rollback().await.unwrap();
                }
                admin.close().await.unwrap();
                value
            }
            Err(panic) => {
                tx.rollback().await.unwrap();
                admin.close().await.unwrap();
                let restored = complete_visible(pool, family, staged_before).await;
                assert_eq!(
                    &restored, before,
                    "failed trial must restore all raw/MVCC/row evidence"
                );
                std::panic::resume_unwind(panic)
            }
        }
    }

    fn complete_reallocated(trial: &CompleteState, reapplied: &CompleteState) {
        assert_eq!(
            trial.portable, reapplied.portable,
            "portable state changed when allocated OIDs/RI names changed"
        );
        let by_address = |state: &CompleteState| -> BTreeMap<String, Value> {
            state.inverse_and_raw["inverse"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| (row["portable"].to_string(), row.clone()))
                .collect()
        };
        let first = by_address(trial);
        let second = by_address(reapplied);
        assert_eq!(
            first.keys().collect::<Vec<_>>(),
            second.keys().collect::<Vec<_>>()
        );
        let new_ri: Vec<_> = first
            .iter()
            .filter(|(_, row)| {
                row["portable"]["type"] == "foreign key RI trigger"
                    && row["portable"]["foreign_key"]["object_names"][2]
                        == "org_unit_revisions_native_actor_v1"
            })
            .collect();
        assert_eq!(new_ri.len(), 4);
        for (address, row) in new_ri {
            let fresh = &second[address];
            assert_ne!(
                row["tuple"], fresh["tuple"],
                "FK RI OID was not actually reallocated"
            );
            assert_ne!(
                row["native"], fresh["native"],
                "allocated RI name was not actually changed"
            );
            assert_eq!(row["RI_flags"], fresh["RI_flags"]);
        }
    }

    // Actual database controls: no synthetic JSON, no grants, SET ROLE, fallback,
    // captured-set duplicate or relaxed timeout can satisfy these assertions.
    async fn complete_scope_array_controls(
        pool: &PgPool,
        baseline: &CompleteState,
        family: CompleteFixtureFamily,
    ) -> Value {
        let mut admin = direct(pool).await;
        let mut tx = begin_protocol(&mut admin, &baseline.target).await;
        let result = AssertUnwindSafe(async {
            bounds(tx.as_mut()).await;
            sqlx::raw_sql("SAVEPOINT complete_scope").execute(tx.as_mut()).await.unwrap();
            sqlx::raw_sql("CREATE TABLE public.complete_capture_unrelated_v1 (id uuid)")
                .execute(tx.as_mut()).await.unwrap();
            let unrelated = complete_read(tx.as_mut()).await;
            assert_eq!(unrelated, baseline.portable, "resource/schema fan-in annexed an unrelated table");
            bounds(tx.as_mut()).await;
            sqlx::raw_sql("ROLLBACK TO SAVEPOINT complete_scope").execute(tx.as_mut()).await.unwrap();
            bounds(tx.as_mut()).await;
            sqlx::raw_sql("CREATE VIEW public.complete_capture_incoming_v1 AS SELECT org_id FROM public.org_units")
                .execute(tx.as_mut()).await.unwrap();
            let incoming = complete_read(tx.as_mut()).await;
            assert_eq!(incoming.snapshot["historical76_sha256"], baseline.portable.snapshot["historical76_sha256"]);
            assert_eq!(incoming.snapshot["historical83_sha256"], baseline.portable.snapshot["historical83_sha256"]);
            assert_ne!(incoming.sha256, baseline.portable.sha256, "incoming column/rule dependency was omitted");
            assert!(incoming.snapshot["supplemental"]["ordinary_dependencies"].as_array().unwrap()
                .iter().any(|row| row.to_string().contains("complete_capture_incoming_v1")));
            bounds(tx.as_mut()).await;
            sqlx::raw_sql("ROLLBACK TO SAVEPOINT complete_scope").execute(tx.as_mut()).await.unwrap();
            bounds(tx.as_mut()).await;
            sqlx::raw_sql("COMMENT ON TABLE public.org_units IS 'complete capture comment-only control'")
                .execute(tx.as_mut()).await.unwrap();
            let comment = complete_read(tx.as_mut()).await;
            assert_eq!(comment.historical76_text, baseline.portable.historical76_text);
            assert_eq!(comment.historical83_text, baseline.portable.historical83_text);
            assert_ne!(comment.sha256, baseline.portable.sha256, "comment-only corruption was invisible");
            bounds(tx.as_mut()).await;
            sqlx::raw_sql("ROLLBACK TO SAVEPOINT complete_scope").execute(tx.as_mut()).await.unwrap();
            bounds(tx.as_mut()).await;
            sqlx::raw_sql("CREATE SCHEMA complete_capture_control; CREATE TYPE complete_capture_control.unused_enum AS ENUM ('x')")
                .execute(tx.as_mut()).await.unwrap();
            let extra = complete_read(tx.as_mut()).await;
            assert_eq!(extra, baseline.portable);
            let address_sql = format!(r#"{})
SELECT (SELECT oid::text FROM pg_catalog.pg_type WHERE oid='pg_catalog.uuid[]'::regtype),
 (SELECT t.typarray::text FROM pg_catalog.pg_type t WHERE t.oid='complete_capture_control.unused_enum'::regtype),
 (SELECT count(*) FROM addresses WHERE classid='pg_catalog.pg_type'::regclass AND objid='pg_catalog.uuid[]'::regtype),
 (SELECT count(*) FROM addresses WHERE classid='pg_catalog.pg_type'::regclass AND objid IN
  (SELECT t.oid FROM pg_catalog.pg_type t JOIN pg_catalog.pg_namespace n ON n.oid=t.typnamespace
   WHERE n.nspname='complete_capture_control'))"#, complete_address_prefix());
            bounds(tx.as_mut()).await;
            let (array_oid, alternate_oid, array_in_roster, alternate_in_roster): (String, String, i64, i64) =
                sqlx::query_as(sqlx::AssertSqlSafe(address_sql)).fetch_one(tx.as_mut()).await.unwrap();
            assert_eq!((array_in_roster,alternate_in_roster), (1,0),
                "control must target a real general array; no alternate array may cause captured-set duplication");
            let relationship: (bool, bool) = sqlx::query_as(
                "SELECT a.typelem=b.oid AND b.typarray=a.oid,\
                 NOT EXISTS(SELECT 1 FROM pg_catalog.pg_class c WHERE c.reltype=b.oid) \
                 FROM pg_catalog.pg_type a JOIN pg_catalog.pg_type b ON b.oid=a.typelem WHERE a.oid='pg_catalog.uuid[]'::regtype")
                .fetch_one(tx.as_mut()).await.unwrap();
            assert_eq!(relationship, (true,true));
            let controls = [
                ("actual_array_missing_namespace", format!("UPDATE pg_catalog.pg_type SET typnamespace=0 WHERE oid={array_oid}::oid")),
                ("actual_array_temp_namespace", format!("UPDATE pg_catalog.pg_type SET typnamespace=pg_catalog.pg_my_temp_schema() WHERE oid={array_oid}::oid")),
                ("actual_array_toast_namespace", format!("UPDATE pg_catalog.pg_type SET typnamespace='pg_toast'::regnamespace WHERE oid={array_oid}::oid")),
                ("base_missing_canonical_array", "UPDATE pg_catalog.pg_type SET typarray=0 WHERE oid='pg_catalog.uuid'::regtype".into()),
                ("base_different_uncaptured_array", format!("UPDATE pg_catalog.pg_type SET typarray={alternate_oid}::oid WHERE oid='pg_catalog.uuid'::regtype")),
            ];
            // Obtain an actual temporary namespace through PostgreSQL, with
            // an unrelated uncaptured temp object; no reserved-name DDL.
            bounds(tx.as_mut()).await;
            sqlx::raw_sql("CREATE TEMP TABLE complete_capture_temp_control (id integer)")
                .execute(tx.as_mut()).await.unwrap();
            let mut control_records = Vec::new();
            for (name, mutation) in controls {
                bounds(tx.as_mut()).await;
                sqlx::raw_sql("SAVEPOINT complete_array").execute(tx.as_mut()).await.unwrap();
                let changed = sqlx::raw_sql(sqlx::AssertSqlSafe(mutation))
                    .execute(tx.as_mut()).await.unwrap();
                assert_eq!(changed.rows_affected(), 1);
                let forward: String = sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT pg_catalog.format_type({array_oid}::oid,NULL)")))
                    .fetch_one(tx.as_mut()).await.unwrap();
                assert_eq!(forward, "uuid[]", "negative must retain ordinary forward spelling");
                let refusal = complete_array_refusal(tx.as_mut(), name == "base_missing_canonical_array").await;
                control_records.push(json!({"case":name,"refusal":refusal,
                    "unchanged_forward_spelling":forward,"alternate_array_in_roster":alternate_in_roster}));
                // Recovery must precede settings statements if the deliberate
                // missing-type refusal has put the transaction in abort state.
                sqlx::raw_sql("ROLLBACK TO SAVEPOINT complete_array").execute(tx.as_mut()).await.unwrap();
                bounds(tx.as_mut()).await;
                assert_eq!(complete_read(tx.as_mut()).await, baseline.portable);
                complete_inverse_and_raw(tx.as_mut()).await;
            }
            json!({"unrelated_resource_scope_exact":true,"incoming_I_edge_observed":true,
                "comment_only_historical_bytes_exact":true,"general_array_controls":control_records})
        }).catch_unwind().await;
        tx.rollback().await.unwrap();
        admin.close().await.unwrap();
        let restored = complete_visible(pool, family, false).await;
        assert_eq!(
            &restored, baseline,
            "controls failed fresh complete/raw/MVCC/row restoration"
        );
        match result {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }

    async fn measure_native_org_unit_account_actor_complete_custody_v1(
        pool: PgPool,
        fixture_family: CompleteFixtureFamily,
        observer_variant: usize,
    ) {
        let sources = complete_source_pins();
        complete_fixture(&pool, fixture_family, observer_variant).await;
        let p = complete_visible(&pool, fixture_family, false).await;
        let frozen = match fixture_family {
            CompleteFixtureFamily::Closed76 => (
                &p.portable.snapshot["historical76_sha256"],
                CLOSED76[observer_variant],
            ),
            CompleteFixtureFamily::Group83 => (
                &p.portable.snapshot["historical83_sha256"],
                INSTALLED83[observer_variant],
            ),
            CompleteFixtureFamily::Navigation83 => (
                &p.portable.snapshot["historical83_sha256"],
                NAVIGATION_CORRECTED83[observer_variant],
            ),
        };
        assert_eq!(
            frozen.0,
            &json!(frozen.1),
            "actual frozen predecessor required before complete measurement"
        );
        let controls = complete_scope_array_controls(&pool, &p, fixture_family).await;
        let (trial_s, expansion_trial) = complete_attempt(
            &pool,
            &p,
            fixture_family,
            CompleteOwnerPhase::Expansion,
            false,
        )
        .await;
        assert_eq!(
            complete_visible(&pool, fixture_family, false).await,
            p,
            "expansion rollback must restore P including actual tuples/rows"
        );
        let (reapplied_s, expansion_commit) = complete_attempt(
            &pool,
            &p,
            fixture_family,
            CompleteOwnerPhase::Expansion,
            true,
        )
        .await;
        complete_reallocated(&trial_s, &reapplied_s);
        let s = complete_visible(&pool, fixture_family, true).await;
        assert_eq!(
            s, reapplied_s,
            "committed S requires fresh administrator inverse and actual reader parity"
        );
        let (trial_v, validation_trial) = complete_attempt(
            &pool,
            &s,
            fixture_family,
            CompleteOwnerPhase::Validation,
            false,
        )
        .await;
        assert_eq!(
            complete_visible(&pool, fixture_family, true).await,
            s,
            "validation rollback must restore committed S including actual tuples/rows"
        );
        let (reapplied_v, validation_commit) = complete_attempt(
            &pool,
            &s,
            fixture_family,
            CompleteOwnerPhase::Validation,
            true,
        )
        .await;
        assert_eq!(trial_v.portable, reapplied_v.portable);
        let v = complete_visible(&pool, fixture_family, true).await;
        assert_eq!(
            v, reapplied_v,
            "committed V requires fresh administrator inverse and actual reader parity"
        );
        let packet = json!({"schema":"console.native_org_unit.account_actor_complete_measurement.v1",
            "design_sha256":"8e39f70157e5a7b32ccbca8ab288657f5084599c1a48d639174686f107400511",
            "sources":sources,"family":format!("{fixture_family:?}"),
            "variant":if observer_variant==0 {"plain"} else {"observer"},
            "target":{"database":p.target.database,"database_oid":p.target.database_oid,
                "system_identifier":p.target.system_identifier},
            "canonical_states":{"P":p.portable.record(),"S":s.portable.record(),"V":v.portable.record()},
            "raw_and_native_inverse":{"P":p.inverse_and_raw,"S":s.inverse_and_raw,"V":v.inverse_and_raw},
            "security_catalog":{"P":p.catalog.1,"S":s.catalog.1,"V":v.catalog.1},
            "old_column_row_summary":row_summary(&p.old_rows),"ledger":p.ledger,"native_census":p.native,
            "trial_S":trial_s.portable.record(),"trial_V":trial_v.portable.record(),"controls":controls,
            "expansion_trial":expansion_trial,"expansion_commit":expansion_commit,
            "validation_trial":validation_trial,"validation_commit":validation_commit,
            "trial_phases_have_administrator_inverse":true,"startup_parity_only_after_commit":true,
            "expansion_rollback_fresh_P_exact":true,"validation_rollback_fresh_S_exact":true,
            "reallocated_RI_portable_equality":true,"canonical_complete_states_observed":3,
            "phase_hashes_registered":false,"database_owner_accepted":false,
            "serving_startup_accepted":false,"organization_ui_accepted":false,"mvp_accepted":false,
            "production_qualified":false,"production_rollback_exercised":false});
        let mut output = std::io::stderr().lock();
        writeln!(
            &mut output,
            "ORG_ACCOUNT_ACTOR_COMPLETE_MEASUREMENT {packet}"
        )
        .unwrap();
        output.flush().unwrap();
    }

    macro_rules! complete_measurement_leaf {
        ($name:ident,$family:ident,$variant:literal) => {
            #[sqlx::test(migrations = false)]
            async fn $name(pool: PgPool) {
                measure_native_org_unit_account_actor_complete_custody_v1(
                    pool,
                    CompleteFixtureFamily::$family,
                    $variant,
                )
                .await;
            }
        };
    }
    complete_measurement_leaf!(actor_complete_closed76_plain_three_phases, Closed76, 0);
    complete_measurement_leaf!(actor_complete_closed76_observer_three_phases, Closed76, 1);
    complete_measurement_leaf!(actor_complete_group83_plain_three_phases, Group83, 0);
    complete_measurement_leaf!(actor_complete_group83_observer_three_phases, Group83, 1);
    complete_measurement_leaf!(
        actor_complete_navigation83_plain_three_phases,
        Navigation83,
        0
    );
    complete_measurement_leaf!(
        actor_complete_navigation83_observer_three_phases,
        Navigation83,
        1
    );

    // Adopted portable TOAST V2, test-only successor. V1 helpers stay unchanged.
    mod portable_toast_v2_successor_tests {
        use super::*;
        use sqlx::Connection as _;
        use std::io::Write as _;

        const TOAST_V2_DESIGN: &str =
            "075d904c3e5b4bdd6e8f1bae1dca6b1c891f6471b5598f018cc89be0fb572be0";
        const OLD_INVERSE: &str = r#", inverse AS MATERIALIZED (
 SELECT a.classid,a.objid,a.objsubid,n.native_address,s.address,s.valid,s.ri_flags,
 inverse.classid AS inverse_classid,inverse.objid AS inverse_objid,inverse.objsubid AS inverse_objsubid
 FROM addresses a JOIN native_addresses n USING(classid,objid,objsubid)
 JOIN stable_addresses s USING(classid,objid,objsubid)
 CROSS JOIN LATERAL pg_catalog.pg_get_object_address(n.native_type,n.native_names,n.native_args) inverse
)"#;

        // $1 is [] in the admitted audit. Controls may replace private original
        // input components for one actual tuple; they do not rewrite catalogs,
        // the portable map, or native/raw evidence. Lookup never selects an OID.
        const CHECKED_INVERSE: &str = r#", toast_original_inputs AS MATERIALIZED (
     SELECT a.classid,a.objid,a.objsubid,n.native_address,n.native_type,n.native_names,n.native_args,
     c.catalog_valid,s.address,s.valid,s.ri_flags,
     CASE WHEN o.patch ? 'classid' THEN (o.patch->>'classid')::oid ELSE a.classid END AS input_classid,
     CASE WHEN o.patch ? 'objid' THEN (o.patch->>'objid')::oid ELSE a.objid END AS input_objid,
     CASE WHEN o.patch ? 'objsubid' THEN (o.patch->>'objsubid')::integer ELSE a.objsubid END AS input_objsubid,
     CASE WHEN o.patch ? 'type' THEN o.patch->>'type' ELSE n.native_type END AS input_type,
     CASE WHEN o.patch ? 'names' THEN CASE WHEN o.patch->'names'='null'::jsonb THEN NULL ELSE
      ARRAY(SELECT jsonb_array_elements_text(o.patch->'names')) END ELSE n.native_names END AS input_names,
     CASE WHEN o.patch ? 'args' THEN CASE WHEN o.patch->'args'='null'::jsonb THEN NULL ELSE
      ARRAY(SELECT jsonb_array_elements_text(o.patch->'args')) END ELSE n.native_args END AS input_args
     FROM addresses a LEFT JOIN native_addresses n USING(classid,objid,objsubid)
     LEFT JOIN checked_native c USING(classid,objid,objsubid)
     LEFT JOIN stable_addresses s USING(classid,objid,objsubid)
     LEFT JOIN LATERAL (
      SELECT e->'patch' AS patch FROM jsonb_array_elements($1::jsonb) e
      WHERE e->'tuple'=jsonb_build_array(a.classid::text,a.objid::text,a.objsubid)
     ) o ON true
    ), toast_inputs AS MATERIALIZED (
     SELECT * FROM toast_original_inputs WHERE native_type='toast table' OR input_type='toast table'
    ), toast_name_lookups AS MATERIALIZED (
     SELECT t.*,found.lookup_count,found.observed_oid
     FROM toast_inputs t LEFT JOIN LATERAL (
      SELECT count(*) AS lookup_count,(array_agg(child.oid ORDER BY child.oid))[1] AS observed_oid
      FROM pg_catalog.pg_class child JOIN pg_catalog.pg_namespace ns ON ns.oid=child.relnamespace
      WHERE ns.nspname::text=t.input_names[1] AND child.relname::text=t.input_names[2]
     ) found ON true
    ), toast_found_children AS MATERIALIZED (
     SELECT t.*,child.relname,child.relkind,child.relpersistence,child.relispartition,
     child.reltoastrelid,child.relowner,ns.nspname,child.oid AS found_child_oid,
     id.type AS found_type,id.object_names AS found_names,id.object_args AS found_args
     FROM toast_name_lookups t LEFT JOIN pg_catalog.pg_class child
     ON t.lookup_count=1 AND child.oid=t.observed_oid
     LEFT JOIN pg_catalog.pg_namespace ns ON ns.oid=child.relnamespace
     LEFT JOIN LATERAL pg_catalog.pg_identify_object_as_address(
      'pg_catalog.pg_class'::regclass::oid,child.oid,0) id ON child.oid IS NOT NULL
    ), toast_parent_lookups AS MATERIALIZED (
     SELECT t.*,parents.global_parent_count,parents.parent_oid,selected.selected_parent_count,
     owners.global_internal_count,canonical.canonical_count,bag.incoming_canonical_count
     FROM toast_found_children t LEFT JOIN LATERAL (
      SELECT count(*) AS global_parent_count,(array_agg(p.oid ORDER BY p.oid))[1] AS parent_oid
      FROM pg_catalog.pg_class p WHERE p.reltoastrelid=t.found_child_oid
     ) parents ON true LEFT JOIN LATERAL (
      SELECT count(*) AS selected_parent_count FROM selected_relations p
      WHERE p.reltoastrelid=t.found_child_oid
     ) selected ON true LEFT JOIN LATERAL (
      SELECT count(*) AS global_internal_count FROM pg_catalog.pg_depend d
      WHERE d.classid='pg_catalog.pg_class'::regclass AND d.objid=t.found_child_oid AND d.deptype='i'
     ) owners ON true LEFT JOIN LATERAL (
      SELECT count(*) AS canonical_count FROM pg_catalog.pg_depend d
      WHERE d.classid='pg_catalog.pg_class'::regclass AND d.objid=t.found_child_oid AND d.objsubid=0
      AND d.refclassid='pg_catalog.pg_class'::regclass AND d.refobjid=parents.parent_oid
      AND d.refobjsubid=0 AND d.deptype='i'
     ) canonical ON true LEFT JOIN LATERAL (
      SELECT count(*) AS incoming_canonical_count FROM ordinary_edges d
      WHERE d.classid='pg_catalog.pg_class'::regclass AND d.objid=t.found_child_oid AND d.objsubid=0
      AND d.refclassid='pg_catalog.pg_class'::regclass AND d.refobjid=parents.parent_oid
      AND d.refobjsubid=0 AND d.deptype='i'
     ) bag ON true
    ), toast_parents AS MATERIALIZED (
     SELECT t.*,p.oid AS found_parent_oid,p.relkind AS parent_relkind,p.relpersistence AS parent_persistence,
     p.relispartition AS parent_partition,p.relowner AS parent_owner,p.reltoastrelid AS parent_toast,
     cp.native_valid AS parent_native_valid,cp.native_address AS checked_parent_native,
     cp.native_type AS checked_parent_type,cp.native_names AS checked_parent_names,
     cp.native_args AS checked_parent_args,role.oid AS actual_owner_oid,
     ident.type AS parent_type,ident.object_names AS parent_names,ident.object_args AS parent_args,
     EXISTS(SELECT 1 FROM metadata_m m WHERE
      (m.classid,m.objid,m.objsubid)=('pg_catalog.pg_class'::regclass::oid,p.oid,0)) AS parent_in_m,
     EXISTS(SELECT 1 FROM incoming_i i WHERE
      (i.classid,i.objid,i.objsubid)=('pg_catalog.pg_class'::regclass::oid,p.oid,0)) AS parent_in_i
     FROM toast_parent_lookups t LEFT JOIN pg_catalog.pg_class p
     ON t.global_parent_count=1 AND p.oid=t.parent_oid
     LEFT JOIN checked_relation_addresses cp ON
     (cp.classid,cp.objid,cp.objsubid)=('pg_catalog.pg_class'::regclass::oid,p.oid,0)
     LEFT JOIN pg_catalog.pg_roles role ON role.oid=t.relowner
     LEFT JOIN LATERAL pg_catalog.pg_identify_object_as_address(
      'pg_catalog.pg_class'::regclass::oid,p.oid,0) ident ON p.oid IS NOT NULL
    ), toast_eligible_parent_inputs AS MATERIALIZED (
     SELECT * FROM toast_parents WHERE found_parent_oid IS NOT NULL
     AND parent_relkind='r' AND parent_persistence='p' AND parent_partition=false
     AND parent_native_valid IS TRUE AND checked_parent_type='table'
     AND parent_type='table' AND parent_names=checked_parent_names AND parent_args=checked_parent_args
     AND parent_names IS NOT NULL AND parent_args=ARRAY[]::text[]
     AND NOT EXISTS(SELECT 1 FROM unnest(parent_names||parent_args) part WHERE part IS NULL)
    ), toast_parent_native_inverses AS MATERIALIZED (
     SELECT p.classid,p.objid,p.objsubid,
     inverse.classid AS parent_inverse_classid,inverse.objid AS parent_inverse_oid,
     inverse.objsubid AS parent_inverse_subid
     FROM toast_eligible_parent_inputs p CROSS JOIN LATERAL
     pg_catalog.pg_get_object_address(p.parent_type,p.parent_names,p.parent_args) inverse
    ), toast_guard_witnesses AS MATERIALIZED (
     SELECT t.*,
     ARRAY[
      (t.classid='pg_catalog.pg_class'::regclass AND t.objsubid=0
       AND t.input_classid='pg_catalog.pg_class'::regclass AND t.input_objsubid=0
       AND t.lookup_count=1 AND t.found_child_oid IS NOT NULL AND t.relkind='t'
       AND t.relpersistence='p' AND t.relispartition=false AND t.nspname='pg_toast'
       AND t.reltoastrelid=0) IS TRUE,
      (t.native_type='toast table' AND t.input_type='toast table' AND t.catalog_valid IS TRUE
       AND t.input_names IS NOT NULL AND array_ndims(t.input_names)=1 AND array_lower(t.input_names,1)=1
       AND cardinality(t.input_names)=2 AND t.input_names=ARRAY[t.nspname::text,t.relname::text]
       AND t.input_args IS NOT NULL AND t.input_args=ARRAY[]::text[]
       AND NOT EXISTS(SELECT 1 FROM unnest(t.input_names||t.input_args) part WHERE part IS NULL)
       AND t.found_type=t.input_type AND t.found_names=t.input_names AND t.found_args=t.input_args
       AND t.found_child_oid=t.input_objid) IS TRUE,
      (t.relname::text ~ '^pg_toast_[0-9]+$') IS TRUE,
      (t.global_parent_count=1 AND t.selected_parent_count=1 AND t.parent_in_m AND t.parent_in_i
       AND t.parent_toast=t.found_child_oid) IS TRUE,
      (t.parent_relkind='r' AND t.parent_persistence='p' AND t.parent_partition=false
       AND t.parent_native_valid IS TRUE AND t.parent_type='table'
       AND t.parent_names IS NOT NULL AND t.parent_args=ARRAY[]::text[]
       AND t.checked_parent_native=jsonb_build_object('type',t.parent_type,
         'object_names',to_jsonb(t.parent_names),'object_args',to_jsonb(t.parent_args))
       AND (pi.parent_inverse_classid,pi.parent_inverse_oid,pi.parent_inverse_subid)=
         ('pg_catalog.pg_class'::regclass::oid,t.found_parent_oid,0)) IS TRUE,
      (t.relowner=t.parent_owner AND t.actual_owner_oid=t.relowner) IS TRUE,
      (t.global_internal_count=1 AND t.canonical_count=1 AND t.incoming_canonical_count=1) IS TRUE,
      ((SELECT count(*) FROM native_addresses n WHERE
        (n.classid,n.objid,n.objsubid)=(t.classid,t.objid,t.objsubid))=1
       AND (SELECT count(*) FROM stable_addresses s WHERE
        (s.classid,s.objid,s.objsubid)=(t.classid,t.objid,t.objsubid))=1) IS TRUE
     ] AS toast_guards,
     jsonb_build_object('type','table TOAST storage','parent',jsonb_build_object(
      'type',t.parent_type,'object_names',to_jsonb(t.parent_names),'object_args',to_jsonb(t.parent_args))) AS expected_portable,
     jsonb_build_array('pg_catalog.pg_class'::regclass::oid::text,t.found_parent_oid::text,0) AS parent_tuple
     FROM toast_parents t LEFT JOIN toast_parent_native_inverses pi USING(classid,objid,objsubid)
    ), ordinary_native_inverse_inputs AS MATERIALIZED (
     SELECT * FROM toast_original_inputs WHERE
     native_type IS DISTINCT FROM 'toast table' AND input_type IS DISTINCT FROM 'toast table'
    ), inverse AS MATERIALIZED (
     SELECT t.classid,t.objid,t.objsubid,t.native_address,t.address,t.valid,t.ri_flags,
     CASE WHEN true=ALL(t.toast_guards) THEN 'pg_catalog.pg_class'::regclass::oid END AS inverse_classid,
     CASE WHEN true=ALL(t.toast_guards) THEN t.found_child_oid END AS inverse_objid,
     CASE WHEN true=ALL(t.toast_guards) THEN 0 END AS inverse_objsubid,
     to_jsonb(t.toast_guards) AS toast_guard_checks,t.expected_portable,t.parent_tuple,t.lookup_count,
     t.global_parent_count,t.selected_parent_count,t.global_internal_count,t.canonical_count,t.incoming_canonical_count
     FROM toast_guard_witnesses t
     UNION ALL
     SELECT t.classid,t.objid,t.objsubid,t.native_address,t.address,t.valid,t.ri_flags,
     inverse.classid,inverse.objid,inverse.objsubid,NULL::jsonb,NULL::jsonb,NULL::jsonb,NULL::bigint,
     NULL::bigint,NULL::bigint,NULL::bigint,NULL::bigint,NULL::bigint
     FROM ordinary_native_inverse_inputs t LEFT JOIN LATERAL
     pg_catalog.pg_get_object_address(t.native_type,t.native_names,t.native_args) inverse ON true
    )"#;

        fn toast_audit_sql() -> String {
            let original = complete_inverse_sql();
            assert_eq!(original.matches(OLD_INVERSE).count(), 1);
            let query = original.replace(OLD_INVERSE, CHECKED_INVERSE);
            const END: &str = "'RI_flags',ri_flags)";
            assert_eq!(query.matches(END).count(), 1);
            let query = query.replace(END, "'RI_flags',ri_flags,'TOAST_guards',toast_guard_checks,\
                    'TOAST_expected',expected_portable,'TOAST_parent_tuple',parent_tuple,\
                    'TOAST_lookup_count',lookup_count,'TOAST_parent_global_count',global_parent_count,\
                    'TOAST_parent_selected_count',selected_parent_count,'TOAST_internal_global_count',global_internal_count,\
                    'TOAST_canonical_count',canonical_count,'TOAST_incoming_canonical_count',incoming_canonical_count)");
            const COUNT: &str = "'address_count',(SELECT count(*) FROM addresses),";
            assert_eq!(query.matches(COUNT).count(), 1);
            query.replace(COUNT, r#"'address_count',(SELECT count(*) FROM addresses),
     'scope',jsonb_build_object(
      'M',(SELECT jsonb_agg(jsonb_build_array(classid::text,objid::text,objsubid) ORDER BY classid,objid,objsubid) FROM metadata_m),
      'I',(SELECT jsonb_agg(jsonb_build_array(classid::text,objid::text,objsubid) ORDER BY classid,objid,objsubid) FROM incoming_i),
      'addresses',(SELECT jsonb_agg(jsonb_build_array(classid::text,objid::text,objsubid) ORDER BY classid,objid,objsubid) FROM addresses),
      'selected_relations',(SELECT jsonb_agg(jsonb_build_object('oid',oid::text,'name',relname,'toast',reltoastrelid::text)
        ORDER BY oid) FROM selected_relations),
      'ordinary_edge_count',(SELECT count(*) FROM ordinary_edges),
      'shared_edge_count',(SELECT count(*) FROM shared_edges)),"#)
        }

        async fn toast_audit_with(
            connection: &mut PgConnection,
            query: String,
            overrides: Value,
        ) -> Value {
            sqlx::raw_sql(READER_SESSION)
                .execute(&mut *connection)
                .await
                .unwrap();
            let start = Instant::now();
            let raw: String = sqlx::query_scalar(sqlx::AssertSqlSafe(query))
                    .bind(sqlx::types::Json(overrides))
                    .fetch_one(&mut *connection)
                    .await
                    .expect("PREREQUISITE: checked catalog inverse must execute; SQL/build/schema/permission/timeout is not semantic TOAST RED");
            assert!(
                start.elapsed() <= Duration::from_secs(3),
                "PREREQUISITE: focused address audit exceeded 3s"
            );
            serde_json::from_str(&raw).unwrap()
        }

        async fn toast_audit(connection: &mut PgConnection) -> Value {
            toast_audit_with(connection, toast_audit_sql(), json!([])).await
        }

        fn census(audit: &Value) -> Result<(), String> {
            let count = audit["address_count"]
                .as_u64()
                .ok_or("missing address count")? as usize;
            if count == 0 {
                return Err("empty address census".into());
            }
            let mut keys = Vec::new();
            for field in ["inverse", "objects"] {
                let entries = audit[field].as_array().ok_or(format!("missing {field}"))?;
                let actual: BTreeSet<_> = entries.iter().map(|e| e["tuple"].to_string()).collect();
                if entries.len() != count || actual.len() != count {
                    return Err(format!("{field} cardinality/uniqueness"));
                }
                keys.push(actual);
            }
            let scope = audit["scope"]["addresses"]
                .as_array()
                .ok_or("missing original addresses")?;
            let scope_keys: BTreeSet<_> = scope.iter().map(Value::to_string).collect();
            if scope.len() != count
                || scope_keys.len() != count
                || keys[0] != scope_keys
                || keys[1] != scope_keys
            {
                return Err("omitted or changed address/raw endpoint".into());
            }
            for (raw, count_key) in [
                ("ordinary_edges", "ordinary_edge_count"),
                ("shared_edges", "shared_edge_count"),
            ] {
                if audit[raw].as_array().ok_or(format!("missing {raw}"))?.len() as u64
                    != audit["scope"][count_key]
                        .as_u64()
                        .ok_or(format!("missing {count_key}"))?
                {
                    return Err(format!("{raw} bag multiplicity"));
                }
            }
            Ok(())
        }

        fn inverse_positive(audit: &Value) -> Result<(), String> {
            census(audit)?;
            let mut inverses = BTreeSet::new();
            let mut expected = BTreeSet::new();
            let mut toast_count = 0;
            for e in audit["inverse"].as_array().unwrap() {
                if e["tuple"] != e["inverse_tuple"]
                    || !inverses.insert(e["inverse_tuple"].to_string())
                {
                    return Err("independent inverse tuple mismatch/collision".into());
                }
                if e["native"]["type"] == "toast table" {
                    toast_count += 1;
                    if e["TOAST_guards"] != json!(vec![true; 8])
                        || e["TOAST_lookup_count"] != 1
                        || e["TOAST_parent_global_count"] != 1
                        || e["TOAST_parent_selected_count"] != 1
                        || e["TOAST_internal_global_count"] != 1
                        || e["TOAST_canonical_count"] != 1
                        || e["TOAST_incoming_canonical_count"] != 1
                        || !e["TOAST_expected"].is_object()
                        || !expected.insert(e["TOAST_expected"].to_string())
                    {
                        return Err("checked TOAST inverse/parent/owner/edge precondition".into());
                    }
                    // These generated storage children remain endpoint-only.
                    for field in ["M", "I"] {
                        if audit["scope"][field]
                            .as_array()
                            .unwrap()
                            .contains(&e["tuple"])
                        {
                            return Err("TOAST child expanded metadata/incoming scope".into());
                        }
                    }
                } else if e["valid"] != true || !e["portable"].is_object() {
                    return Err("ordinary/RI/builtin identity changed or refused".into());
                }
            }
            if toast_count == 0 {
                return Err("no real TOAST fixture witness".into());
            }
            Ok(())
        }

        fn semantic_positive(audit: &Value) -> Result<(), String> {
            inverse_positive(audit)?;
            let mut portable = BTreeSet::new();
            for e in audit["inverse"].as_array().unwrap() {
                if e["valid"] != true
                    || !e["portable"].is_object()
                    || !portable.insert(e["portable"].to_string())
                {
                    return Err("unresolved/duplicate stable identity".into());
                }
                if e["native"]["type"] == "toast table" && e["portable"] != e["TOAST_expected"] {
                    return Err(
                        "TOAST semantic identity differs from independent actual-parent inverse"
                            .into(),
                    );
                }
            }
            Ok(())
        }

        fn machinery_controls(baseline: &Value) {
            inverse_positive(baseline)
                .expect("PREREQUISITE: true inverse positive before injected evidence faults");
            // Evidence-only positive; this does not accept a product capture.
            let mut oracle_positive = baseline.clone();
            for entry in oracle_positive["inverse"].as_array_mut().unwrap() {
                if entry["native"]["type"] == "toast table" {
                    entry["portable"] = entry["TOAST_expected"].clone();
                    entry["valid"] = json!(true);
                }
            }
            semantic_positive(&oracle_positive)
                .expect("private expected-map oracle positive must actually pass");
            for fault in ["NULL_map", "wrong_type", "map_collision", "omitted_TOAST"] {
                let mut damaged = oracle_positive.clone();
                let indices: Vec<_> = damaged["inverse"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .enumerate()
                    .filter(|(_, entry)| entry["native"]["type"] == "toast table")
                    .map(|(index, _)| index)
                    .collect();
                assert!(
                    indices.len() >= 2,
                    "PREREQUISITE: two real storage identity witnesses"
                );
                match fault {
                    "NULL_map" => damaged["inverse"][indices[0]]["portable"] = Value::Null,
                    "wrong_type" => {
                        damaged["inverse"][indices[0]]["portable"]["type"] = json!("table")
                    }
                    "map_collision" => {
                        damaged["inverse"][indices[0]]["portable"] =
                            damaged["inverse"][indices[1]]["portable"].clone()
                    }
                    "omitted_TOAST" => {
                        damaged["inverse"]
                            .as_array_mut()
                            .unwrap()
                            .remove(indices[0]);
                    }
                    _ => unreachable!(),
                }
                assert!(
                    semantic_positive(&damaged).is_err(),
                    "corrupted map evidence accepted: {fault}"
                );
            }

            for field in ["inverse", "objects", "ordinary_edges", "shared_edges"] {
                let entries = baseline[field].as_array().unwrap();
                assert!(
                    !entries.is_empty(),
                    "PREREQUISITE: nonempty corruption witness {field}"
                );
                let mut omitted = baseline.clone();
                omitted[field].as_array_mut().unwrap().pop();
                assert!(
                    census(&omitted).is_err(),
                    "omitted evidence accepted: {field}"
                );
                let mut duplicate = baseline.clone();
                duplicate[field]
                    .as_array_mut()
                    .unwrap()
                    .push(entries[0].clone());
                assert!(
                    census(&duplicate).is_err(),
                    "duplicate evidence accepted: {field}"
                );
            }
            let mut changed = baseline.clone();
            changed["inverse"][0]["inverse_tuple"][1] = json!("0");
            assert!(
                inverse_positive(&changed).is_err(),
                "copied expected inverse accepted"
            );
            let mut missing = baseline.clone();
            let toast = missing["inverse"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|e| e["native"]["type"] == "toast table")
                .unwrap();
            toast["TOAST_guards"][5] = json!(false);
            assert!(
                inverse_positive(&missing).is_err(),
                "owner guard omitted from oracle"
            );
            let mut dropped = baseline.clone();
            dropped["scope"]["addresses"].as_array_mut().unwrap().pop();
            assert!(census(&dropped).is_err(), "omitted endpoint accepted");
        }

        async fn platform_refusals(connection: &mut PgConnection, child: &Value) {
            let names: Vec<String> = child["native"]["object_names"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_owned())
                .collect();
            for (kind, code) in [("toast table", "22023"), ("table", "42809")] {
                sqlx::raw_sql("SAVEPOINT toast_platform_probe")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                let result = sqlx::query(
                    "SELECT * FROM pg_catalog.pg_get_object_address($1,$2::text[],ARRAY[]::text[])",
                )
                .bind(kind)
                .bind(&names)
                .fetch_all(&mut *connection)
                .await;
                let error = result.expect_err(
                    "pinned unsupported TOAST platform inverse unexpectedly became usable",
                );
                assert_eq!(
                    error.as_database_error().and_then(|e| e.code()).as_deref(),
                    Some(code)
                );
                sqlx::raw_sql("ROLLBACK TO SAVEPOINT toast_platform_probe; RELEASE SAVEPOINT toast_platform_probe")
                        .execute(&mut *connection).await.unwrap();
            }
        }

        async fn original_input_controls(
            connection: &mut PgConnection,
            baseline: &Value,
            child: &Value,
        ) {
            let tuple = child["tuple"].clone();
            let names = child["native"]["object_names"].clone();
            let cases = [
                (
                    "missing raw name",
                    json!({"names":["pg_toast","pg_toast_999999999999999999999999999999"]}),
                ),
                (
                    "missing raw namespace",
                    json!({"names":["console_absent_toast_namespace",names[1]]}),
                ),
                ("alias namespace", json!({"names":["pg_catalog",names[1]]})),
                (
                    "extra native name",
                    json!({"names":[names[0],names[1],"extra"]}),
                ),
                ("partial native names", json!({"names":[names[0]]})),
                ("NULL name part", json!({"names":[names[0],null]})),
                ("NULL names", json!({"names":null})),
                ("nonempty arguments", json!({"args":["extra"]})),
                ("NULL argument part", json!({"args":[null]})),
                ("NULL arguments", json!({"args":null})),
                ("native type disagreement", json!({"type":"table"})),
                ("wrong original tuple", json!({"objid":"0"})),
                ("wrong class", json!({"classid":"0"})),
                ("positive subobject", json!({"objsubid":1})),
                ("negative subobject", json!({"objsubid":-1})),
            ];
            for (name, patch) in cases {
                let audit = toast_audit_with(
                    connection,
                    toast_audit_sql(),
                    json!([{"tuple":tuple,"patch":patch}]),
                )
                .await;
                census(&audit)
                    .expect("private input fault must retain every original address/raw record");
                assert!(
                    inverse_positive(&audit).is_err(),
                    "private inverse input accepted: {name}"
                );
                assert_eq!(audit["objects"], baseline["objects"]);
                assert_eq!(audit["scope"], baseline["scope"]);
            }
            // Deliberately corrupt the test lookup: return a copied expected OID
            // without discovering a name. Exact re-identification must still refuse.
            const FIND: &str = r#"SELECT count(*) AS lookup_count,(array_agg(child.oid ORDER BY child.oid))[1] AS observed_oid
      FROM pg_catalog.pg_class child JOIN pg_catalog.pg_namespace ns ON ns.oid=child.relnamespace
      WHERE ns.nspname::text=t.input_names[1] AND child.relname::text=t.input_names[2]"#;
            let query = toast_audit_sql();
            assert_eq!(query.matches(FIND).count(), 1);
            let forged = query.replace(
                FIND,
                "SELECT 1::bigint AS lookup_count,t.input_objid AS observed_oid",
            );
            let rejected = toast_audit_with(
                connection,
                forged,
                json!([{"tuple":tuple,
                    "patch":{"names":["pg_toast","pg_toast_999999999999999999999999999999"]}}]),
            )
            .await;
            assert!(
                inverse_positive(&rejected).is_err(),
                "expected-OID fabrication bypassed independent re-identification"
            );
            let duplicate = query.replace(FIND,r#"SELECT count(*) AS lookup_count,(array_agg(child.oid ORDER BY child.oid))[1] AS observed_oid
      FROM (SELECT c.oid,c.relnamespace,c.relname FROM pg_catalog.pg_class c
       UNION ALL SELECT c.oid,c.relnamespace,c.relname FROM pg_catalog.pg_class c) child
      JOIN pg_catalog.pg_namespace ns ON ns.oid=child.relnamespace
      WHERE ns.nspname::text=t.input_names[1] AND child.relname::text=t.input_names[2]"#);
            let rejected = toast_audit_with(connection, duplicate, json!([])).await;
            census(&rejected).unwrap();
            assert!(
                inverse_positive(&rejected).is_err(),
                "ambiguous lookup accepted or deduplicated"
            );
            assert_eq!(
                toast_audit(connection).await,
                *baseline,
                "input/lookup corruption changed actual state"
            );
        }

        async fn catalog_control(
            connection: &mut PgConnection,
            baseline: &Value,
            child: &Value,
            name: &str,
            mutation: &str,
            expected_rows: u64,
            expected_guard: usize,
        ) {
            bounds(connection).await;
            sqlx::raw_sql("SAVEPOINT toast_catalog_control")
                .execute(&mut *connection)
                .await
                .unwrap();
            let result = AssertUnwindSafe(async {
                let rows = sqlx::query(sqlx::AssertSqlSafe(mutation.to_owned()))
                    .bind(child["tuple"][1].as_str().unwrap())
                    .bind(child["TOAST_parent_tuple"][1].as_str().unwrap())
                    .execute(&mut *connection)
                    .await
                    .expect("PREREQUISITE: exact administrator catalog mutation must execute")
                    .rows_affected();
                assert_eq!(rows, expected_rows, "mutation row-count admission: {name}");
                let original_names = if name == "malformed generated name" {
                    json!(["pg_toast", "console_toast_malformed"])
                } else {
                    child["native"]["object_names"].clone()
                };
                let audit = toast_audit_with(
                    connection,
                    toast_audit_sql(),
                    json!([{"tuple":child["tuple"],
                        "patch":{"type":child["native"]["type"],"names":original_names,
                        "args":child["native"]["object_args"]}}]),
                )
                .await;
                census(&audit).expect("catalog fault must retain complete address/raw census");
                let actual = audit["inverse"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|e| e["tuple"] == child["tuple"])
                    .expect("catalog fault omitted the original child endpoint");
                assert_eq!(
                    actual["TOAST_guards"][expected_guard], false,
                    "guard {expected_guard} accepted {name}"
                );
                assert_ne!(
                    actual["inverse_tuple"], actual["tuple"],
                    "catalog fault gained usable inverse: {name}"
                );
                assert!(
                    actual["valid"] != true && actual["portable"].is_null(),
                    "invalid storage got map fallback: {name}"
                );
            })
            .catch_unwind()
            .await;
            sqlx::raw_sql(
                "ROLLBACK TO SAVEPOINT toast_catalog_control; RELEASE SAVEPOINT toast_catalog_control",
            )
            .execute(&mut *connection)
            .await
            .expect("mandatory catalog control savepoint recovery");
            assert_eq!(
                toast_audit(connection).await,
                *baseline,
                "fresh raw/native/census readback failed: {name}"
            );
            if let Err(panic) = result {
                std::panic::resume_unwind(panic);
            }
        }

        async fn noninternal_edge_control(
            connection: &mut PgConnection,
            baseline: &Value,
            child: &Value,
        ) {
            bounds(connection).await;
            sqlx::raw_sql("SAVEPOINT toast_noninternal_control")
                .execute(&mut *connection)
                .await
                .unwrap();
            let result = AssertUnwindSafe(async {
                let inserted = sqlx::query("INSERT INTO pg_catalog.pg_depend \
                    SELECT classid,objid,objsubid,refclassid,refobjid,refobjsubid,'n' \
                    FROM pg_catalog.pg_depend WHERE classid='pg_catalog.pg_class'::regclass \
                    AND objid=$1::text::oid AND objsubid=0 AND refclassid='pg_catalog.pg_class'::regclass \
                    AND refobjid=$2::text::oid AND refobjsubid=0 AND deptype='i'")
                    .bind(child["tuple"][1].as_str().unwrap()).bind(child["TOAST_parent_tuple"][1].as_str().unwrap())
                    .execute(&mut *connection).await.unwrap().rows_affected();
                assert_eq!(inserted,1,"exact one real noninternal incoming edge admission");
                let extra=toast_audit(connection).await;
                inverse_positive(&extra).expect("noninternal edge must preserve canonical storage inverse");
                assert_eq!(extra["inverse"],baseline["inverse"]);
                assert_eq!(extra["objects"],baseline["objects"]);
                for field in ["M","I","addresses","selected_relations","shared_edge_count"] {
                    assert_eq!(extra["scope"][field],baseline["scope"][field]);
                }
                assert_eq!(extra["ordinary_edges"].as_array().unwrap().len(),baseline["ordinary_edges"].as_array().unwrap().len()+1);
                let mut bag:BTreeMap<String,usize>=BTreeMap::new();
                for row in extra["ordinary_edges"].as_array().unwrap() { *bag.entry(row.to_string()).or_default()+=1; }
                for row in baseline["ordinary_edges"].as_array().unwrap() {
                    let copies=bag.get_mut(&row.to_string()).expect("lost prior raw/MVCC edge");
                    assert!(*copies>0); *copies-=1;
                }
                assert_eq!(bag.values().sum::<usize>(),1,"exact one-row bag delta");
                // Keep a genuine incoming edge while deleting the canonical one,
                // so endpoint disappearance cannot mask guard-seven refusal.
                let removed=sqlx::query("DELETE FROM pg_catalog.pg_depend \
                    WHERE classid='pg_catalog.pg_class'::regclass AND objid=$1::text::oid \
                    AND objsubid=0 AND refclassid='pg_catalog.pg_class'::regclass \
                    AND refobjid=$2::text::oid AND refobjsubid=0 AND deptype='i'")
                    .bind(child["tuple"][1].as_str().unwrap()).bind(child["TOAST_parent_tuple"][1].as_str().unwrap())
                    .execute(&mut *connection).await.unwrap().rows_affected();
                assert_eq!(removed,1,"exact missing canonical ownership admission");
                let refused=toast_audit(connection).await;
                census(&refused).unwrap();
                assert_eq!(refused["scope"]["addresses"],baseline["scope"]["addresses"]);
                let observed=refused["inverse"].as_array().unwrap().iter().find(|e|e["tuple"]==child["tuple"]).unwrap();
                assert_eq!(observed["TOAST_guards"][6],false);
                assert_ne!(observed["inverse_tuple"],observed["tuple"]);
                assert!(observed["valid"]!=true && observed["portable"].is_null());
            }).catch_unwind().await;
            sqlx::raw_sql("ROLLBACK TO SAVEPOINT toast_noninternal_control; RELEASE SAVEPOINT toast_noninternal_control")
                .execute(&mut *connection).await.unwrap();
            assert_eq!(
                toast_audit(connection).await,
                *baseline,
                "noninternal/missing-internal rollback failed exact raw/MVCC restoration"
            );
            if let Err(panic) = result {
                std::panic::resume_unwind(panic);
            }
        }

        #[sqlx::test(migrations = false)]
        async fn actor_complete_closed76_toast_v2_catalog_inverse_then_semantic_map(pool: PgPool) {
            let sources = complete_source_pins();
            complete_fixture(&pool, CompleteFixtureFamily::Closed76, 0).await;
            let mut admin = direct(&pool).await;
            let expected_target = target(&mut admin).await;
            let administrator:bool=sqlx::query_scalar("SELECT current_user=session_user AND \
                (SELECT count(*)=1 AND bool_and(rolsuper) FROM pg_catalog.pg_roles WHERE rolname=current_user)")
                .fetch_one(&mut admin).await.unwrap();
            assert!(
                administrator,
                "PREREQUISITE: private inverse requires the actual administrator, without role switching"
            );
            let engine: i32 =
                sqlx::query_scalar("SELECT current_setting('server_version_num')::integer")
                    .fetch_one(&mut admin)
                    .await
                    .unwrap();
            assert_eq!(
                engine, 180004,
                "PREREQUISITE: exact characterized PostgreSQL 18.4 required"
            );
            let mut tx = begin_protocol(&mut admin, &expected_target).await;
            let baseline = toast_audit(tx.as_mut()).await;
            // THIS IS A PREREQUISITE, separate from the intended semantic RED.
            inverse_positive(&baseline).expect("PREREQUISITE: genuine checked-catalog TOAST positive plus unchanged ordinary/RI/builtin inverse");
            let child = baseline["inverse"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["native"]["type"] == "toast table")
                .unwrap()
                .clone();
            let original_ledger = applied_ledger(tx.as_mut()).await;
            let original_rows = complete_business(tx.as_mut(), false).await;
            let result=AssertUnwindSafe(async {
                    platform_refusals(tx.as_mut(),&child).await;
                    assert_eq!(toast_audit(tx.as_mut()).await,baseline,"platform probe recovery changed raw custody");
                    machinery_controls(&baseline);
                    original_input_controls(tx.as_mut(),&baseline,&child).await;
                    for (name,mutation,guard) in [
                        ("wrong child relkind","UPDATE pg_catalog.pg_class SET relkind='r' WHERE oid=$1::text::oid AND $2::text::oid>0",0),
                        ("unlogged child","UPDATE pg_catalog.pg_class SET relpersistence='u' WHERE oid=$1::text::oid AND $2::text::oid>0",0),
                        ("partitioned child","UPDATE pg_catalog.pg_class SET relispartition=true WHERE oid=$1::text::oid AND $2::text::oid>0",0),
                        ("recursive child storage","UPDATE pg_catalog.pg_class SET reltoastrelid=oid WHERE oid=$1::text::oid AND $2::text::oid>0",0),
                        ("namespace alias","UPDATE pg_catalog.pg_class SET relnamespace='pg_catalog'::regnamespace WHERE oid=$1::text::oid AND $2::text::oid>0",0),
                        ("malformed generated name","UPDATE pg_catalog.pg_class SET relname='console_toast_malformed' WHERE oid=$1::text::oid AND $2::text::oid>0",2),
                        ("missing child owner","UPDATE pg_catalog.pg_class SET relowner=0 WHERE oid=$1::text::oid AND $2::text::oid>0",5),
                        ("owner mismatch","UPDATE pg_catalog.pg_class SET relowner=(SELECT oid FROM pg_catalog.pg_roles WHERE rolname=CASE WHEN relowner=(SELECT oid FROM pg_catalog.pg_roles WHERE rolname='console_app') THEN 'console_account_owner' ELSE 'console_app' END) WHERE oid=$1::text::oid AND $2::text::oid>0",5),
                        ("no parent","UPDATE pg_catalog.pg_class SET reltoastrelid=0 WHERE oid=$2::text::oid AND $1::text::oid>0",3),
                        ("wrong parent kind","UPDATE pg_catalog.pg_class SET relkind='v' WHERE oid=$2::text::oid AND $1::text::oid>0",4),
                        ("unlogged parent","UPDATE pg_catalog.pg_class SET relpersistence='u' WHERE oid=$2::text::oid AND $1::text::oid>0",4),
                        ("partitioned parent","UPDATE pg_catalog.pg_class SET relispartition=true WHERE oid=$2::text::oid AND $1::text::oid>0",4),
                        ("wrong internal dependency type","UPDATE pg_catalog.pg_depend SET deptype='n' WHERE classid='pg_catalog.pg_class'::regclass AND objid=$1::text::oid AND objsubid=0 AND refclassid='pg_catalog.pg_class'::regclass AND refobjid=$2::text::oid AND refobjsubid=0 AND deptype='i'",6),
                        ("duplicate internal dependency","INSERT INTO pg_catalog.pg_depend SELECT * FROM pg_catalog.pg_depend WHERE classid='pg_catalog.pg_class'::regclass AND objid=$1::text::oid AND objsubid=0 AND refclassid='pg_catalog.pg_class'::regclass AND refobjid=$2::text::oid AND refobjsubid=0 AND deptype='i'",6),
                    ] {
                        catalog_control(tx.as_mut(),&baseline,&child,name,mutation,1,guard).await;
                    }
                    noninternal_edge_control(tx.as_mut(),&baseline,&child).await;
                    assert_eq!(applied_ledger(tx.as_mut()).await,original_ledger);
                    assert_eq!(complete_business(tx.as_mut(),false).await,original_rows);
                    let after=toast_audit(tx.as_mut()).await;
                    assert_eq!(after,baseline,"all controls must restore exact M/I/addresses/raw/MVCC/edge bags");
                    writeln!(&mut std::io::stderr().lock(),"ORG_TOAST_V2_INVERSE_PREREQUISITE {}",json!({
                        "design_sha256":TOAST_V2_DESIGN,"sources":sources,"address_count":baseline["address_count"],
                        "toast_count":baseline["inverse"].as_array().unwrap().iter().filter(|e|e["native"]["type"]=="toast table").count(),
                        "catalog_controls":15,"original_input_controls":15,"lookup_corruption_controls":2,
                        "complete_raw_inverse_census":true,"all_eight_inverse_guards_positive":true,
                        "semantic_map_accepted":false,"full_PSV_packet_accepted":false,
                        "database_owner_accepted":false,"mvp_accepted":false,"production_qualified":false})).unwrap();
                    // First and sole intended product RED: absent semantic map.
                    semantic_positive(&after).expect("TOAST_V2_SEMANTIC_MAP_MISSING_OR_INVALID: actual parent-derived identity required for every admitted TOAST address");
                }).catch_unwind().await;
            tx.rollback()
                .await
                .expect("mandatory complete diagnostic rollback");
            assert!(matches_target(&mut admin, &expected_target).await);
            let mut fresh = sqlx::Connection::begin(&mut admin).await.unwrap();
            assert_eq!(
                toast_audit(fresh.as_mut()).await,
                baseline,
                "fresh transaction rollback readback"
            );
            assert_eq!(applied_ledger(fresh.as_mut()).await, original_ledger);
            assert_eq!(
                complete_business(fresh.as_mut(), false).await,
                original_rows
            );
            fresh.rollback().await.unwrap();
            admin.close().await.unwrap();
            if let Err(panic) = result {
                std::panic::resume_unwind(panic);
            }
        }
        // V3 successor: fixed unregistered V2 candidate source, with the V1
        // source/measurement/TOAST diagnostic bytes preserved above.
        mod finite_relation_column_v3_successor_tests {
            use super::*;

            const DESIGN: &str = "c0c96a5d6c69fdb8b26a2d4c23666d46106ccb0cb54b8b171b995812b53ef1fc";
            const CANDIDATE_INPUT: &str = include_str!(
                "../../../../ops/native-org-unit/account-actor-custody-capture-v2.sql"
            );
            const CANDIDATE_EXPORT: &str = include_str!(
                "../../../../ops/postgres-capture-native-org-unit-account-actor-v2-custody.sql"
            );
            const CANDIDATE_ORACLE: &str = include_str!(
                "../../../../ops/fixtures/native-org-unit-account-actor-capture-export-contract-v2.json"
            );
            const COLUMN_TYPES: [&str; 6] = [
                "table column",
                "foreign table column",
                "view column",
                "materialized view column",
                "composite type column",
                "sequence column",
            ];
            const IS_COLUMN: &str = "classid='pg_catalog.pg_class'::regclass AND objsubid>0 AND native_type IN ('table column','foreign table column','view column','materialized view column','composite type column','sequence column')";

            // Private inverse API namespace only. Neither original raw type nor
            // source catalog/native/portable validity is changed by this helper.
            const COLUMN_CTES: &str = r#"column_name_lookups AS MATERIALIZED (
 SELECT t.*,ns.namespace_match_count,rel.relation_match_count,found.column_match_count,
 found.observed_oid,found.observed_attnum
 FROM column_inputs t LEFT JOIN LATERAL (
  SELECT count(*) AS namespace_match_count FROM pg_catalog.pg_namespace n
  WHERE n.nspname::text COLLATE "C"=t.input_names[1] COLLATE "C"
 ) ns ON true LEFT JOIN LATERAL (
  SELECT count(*) AS relation_match_count FROM pg_catalog.pg_class c
  JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
  WHERE n.nspname::text COLLATE "C"=t.input_names[1] COLLATE "C" AND c.relname::text COLLATE "C"=t.input_names[2] COLLATE "C"
 ) rel ON true LEFT JOIN LATERAL (
  SELECT count(*) AS column_match_count,
   (array_agg(c.oid ORDER BY c.oid,a.attnum))[1] AS observed_oid,
   (array_agg(a.attnum ORDER BY c.oid,a.attnum))[1] AS observed_attnum
  FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
  JOIN pg_catalog.pg_attribute a ON a.attrelid=c.oid
  WHERE n.nspname::text COLLATE "C"=t.input_names[1] COLLATE "C" AND c.relname::text COLLATE "C"=t.input_names[2] COLLATE "C"
   AND a.attname::text COLLATE "C"=t.input_names[3] COLLATE "C"
 ) found ON true
), column_found AS MATERIALIZED (
 SELECT t.*,c.oid AS found_oid,c.relkind,n.nspname,c.relname,
 a.attrelid,a.attnum,a.attname,a.attisdropped,a.atttypid,
 CASE c.relkind WHEN 'r' THEN 'table column' WHEN 'p' THEN 'table column'
  WHEN 'f' THEN 'foreign table column' WHEN 'v' THEN 'view column'
  WHEN 'm' THEN 'materialized view column' WHEN 'c' THEN 'composite type column'
  WHEN 'S' THEN 'sequence column' END AS catalog_type,
 ARRAY[n.nspname::text,c.relname::text,a.attname::text] AS catalog_names
 FROM column_name_lookups t LEFT JOIN pg_catalog.pg_class c
 ON t.column_match_count=1 AND c.oid=t.observed_oid
 LEFT JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
 LEFT JOIN pg_catalog.pg_attribute a ON a.attrelid=c.oid AND a.attnum=t.observed_attnum
), column_catalog_guards AS MATERIALIZED (
 SELECT t.*,ARRAY[
  (t.classid='pg_catalog.pg_class'::regclass AND t.objsubid>0
   AND t.input_classid='pg_catalog.pg_class'::regclass AND t.input_objsubid>0) IS TRUE,
  (t.input_names IS NOT NULL AND array_ndims(t.input_names)=1
   AND array_lower(t.input_names,1)=1 AND cardinality(t.input_names)=3
   AND t.input_args IS NOT NULL AND t.input_args=ARRAY[]::text[]
   AND NOT EXISTS(SELECT 1 FROM unnest(t.input_names||t.input_args) part WHERE part IS NULL)) IS TRUE,
  (t.namespace_match_count=1 AND t.relation_match_count=1 AND t.column_match_count=1) IS TRUE,
  (t.catalog_type IS NOT NULL AND t.native_type=t.catalog_type AND t.input_type=t.catalog_type
   AND t.relkind IN ('r','p','f','v','m','c','S') AND t.nspname IS NOT NULL
   AND t.nspname NOT IN ('pg_temp','pg_toast') AND NOT starts_with(t.nspname,'pg_temp_')
   AND NOT starts_with(t.nspname,'pg_toast_temp_')) IS TRUE,
  (t.found_oid IS NOT NULL AND t.attrelid=t.found_oid AND t.attnum>0
   AND NOT t.attisdropped AND t.atttypid<>0) IS TRUE,
  ((t.classid,t.objid,t.objsubid)=('pg_catalog.pg_class'::regclass::oid,t.found_oid,t.attnum::integer)
   AND (t.input_classid,t.input_objid,t.input_objsubid)=(t.classid,t.objid,t.objsubid)
   AND t.native_names=t.catalog_names AND t.input_names=t.catalog_names
   AND t.native_args=ARRAY[]::text[] AND t.input_args=t.native_args) IS TRUE
 ] AS catalog_checks FROM column_found t
), column_identification_inputs AS MATERIALIZED (
 SELECT * FROM column_catalog_guards WHERE true=ALL(catalog_checks)
), column_native_observations AS MATERIALIZED (
 SELECT t.classid,t.objid,t.objsubid,ident.type AS found_native_type,
 ident.object_names AS found_native_names,ident.object_args AS found_native_args
 FROM column_identification_inputs t CROSS JOIN LATERAL
 pg_catalog.pg_identify_object_as_address('pg_catalog.pg_class'::regclass::oid,t.found_oid,t.attnum::integer) ident
), column_guard_witnesses AS MATERIALIZED (
 SELECT t.*,o.found_native_type,o.found_native_names,o.found_native_args,
 t.catalog_checks || ARRAY[(o.found_native_type=t.native_type AND o.found_native_type=t.input_type
  AND o.found_native_names=t.native_names AND o.found_native_names=t.input_names
  AND o.found_native_args=t.native_args AND o.found_native_args=t.input_args) IS TRUE] AS column_checks
 FROM column_catalog_guards t LEFT JOIN column_native_observations o USING(classid,objid,objsubid)
), column_function_inputs AS MATERIALIZED (
 SELECT * FROM column_guard_witnesses WHERE true=ALL(column_checks)
), column_function_observations AS MATERIALIZED (
 SELECT t.classid,t.objid,t.objsubid,inverse.classid AS function_classid,
 inverse.objid AS function_objid,inverse.objsubid AS function_objsubid
 FROM column_function_inputs t CROSS JOIN LATERAL
 pg_catalog.pg_get_object_address('table column',t.input_names,t.input_args) inverse
), column_inverse AS MATERIALIZED (
 SELECT t.*,
 CASE WHEN (o.function_classid,o.function_objid,o.function_objsubid)=(t.classid,t.objid,t.objsubid)
  THEN o.function_classid END AS inverse_classid,
 CASE WHEN (o.function_classid,o.function_objid,o.function_objsubid)=(t.classid,t.objid,t.objsubid)
  THEN o.function_objid END AS inverse_objid,
 CASE WHEN (o.function_classid,o.function_objid,o.function_objsubid)=(t.classid,t.objid,t.objsubid)
  THEN o.function_objsubid END AS inverse_objsubid,
 jsonb_build_object('checks',to_jsonb(t.column_checks),'kind',t.relkind,
  'namespace_count',t.namespace_match_count,'relation_count',t.relation_match_count,'column_count',t.column_match_count,
  'found_tuple',jsonb_build_array('pg_catalog.pg_class'::regclass::oid::text,t.found_oid::text,t.attnum::integer),
  'found_native',jsonb_build_object('type',t.found_native_type,'object_names',to_jsonb(t.found_native_names),'object_args',to_jsonb(t.found_native_args)),
  'inverse_namespace','table column','function_tuple',jsonb_build_array(o.function_classid::text,o.function_objid::text,o.function_objsubid)) AS column_witness
 FROM column_guard_witnesses t LEFT JOIN column_function_observations o USING(classid,objid,objsubid)
)"#;

            fn candidate_byte_pins(
                input: &str,
                export: &str,
                oracle: &Value,
            ) -> Result<(), String> {
                if input != export
                    || oracle["source_sha256"] != json!(digest(input))
                    || oracle["source_bytes"].as_u64() != Some(input.len() as u64)
                {
                    return Err("stale/mismatched candidate source/export/oracle bytes".into());
                }
                Ok(())
            }

            fn candidate_source_pins() -> Value {
                let old = complete_source_pins();
                let oracle: Value = serde_json::from_str(CANDIDATE_ORACLE)
                    .expect("PREREQUISITE: valid reviewed V2 oracle");
                assert_eq!(
                    oracle["schema"],
                    "console.native_org_unit.account_actor_capture_export_contract.v2"
                );
                assert_eq!(oracle["design_sha256"], DESIGN);
                assert_eq!(oracle["predecessor_source_sha256"], COMPLETE_SHA);
                assert_eq!(
                    oracle["predecessor_oracle_sha256"],
                    "71223f87d2c3e6283e90ee1c20f63099faccc8876f2171dd9f8c012512170660"
                );
                assert_eq!(
                    oracle["source"],
                    "ops/native-org-unit/account-actor-custody-capture-v2.sql"
                );
                assert_eq!(
                    oracle["output"],
                    "ops/postgres-capture-native-org-unit-account-actor-v2-custody.sql"
                );
                assert_eq!(
                    oracle["snapshot_schema"],
                    "console.native_org_unit.account_actor_complete_capture.v2"
                );
                for field in [
                    "phase_hashes_registered",
                    "database_owner_accepted",
                    "source_release_accepted",
                    "MVP_accepted",
                ] {
                    assert_eq!(
                        oracle[field], false,
                        "PREREQUISITE: unregistered source/test custody only"
                    );
                }
                candidate_byte_pins(CANDIDATE_INPUT, CANDIDATE_EXPORT, &oracle)
                    .expect("PREREQUISITE: exact reviewed candidate bytes");
                let mut stale = oracle.clone();
                stale["source_sha256"] = json!("0".repeat(64));
                assert!(candidate_byte_pins(CANDIDATE_INPUT, CANDIDATE_EXPORT, &stale).is_err());
                let mut wrong_length = oracle.clone();
                wrong_length["source_bytes"] = json!(CANDIDATE_INPUT.len() + 1);
                assert!(
                    candidate_byte_pins(CANDIDATE_INPUT, CANDIDATE_EXPORT, &wrong_length).is_err()
                );
                assert!(
                    candidate_byte_pins(CANDIDATE_INPUT, &format!("{CANDIDATE_EXPORT} "), &oracle)
                        .is_err()
                );
                assert_eq!(
                    CANDIDATE_INPUT, CANDIDATE_EXPORT,
                    "PREREQUISITE: exact source/export parity"
                );
                assert_eq!(oracle["source_sha256"], digest(CANDIDATE_INPUT));
                assert_eq!(
                    oracle["source_bytes"].as_u64(),
                    Some(CANDIDATE_INPUT.len() as u64)
                );
                assert!(CANDIDATE_INPUT.ends_with(";\n"));
                assert_eq!(
                    CANDIDATE_INPUT
                        .matches(
                            "'schema','console.native_org_unit.account_actor_complete_capture.v2'"
                        )
                        .count(),
                    1
                );
                for name in ["historical76", "historical83", "original73"] {
                    let start = format!("{name}_query AS MATERIALIZED (\n SELECT * FROM (\n");
                    let end = format!("\n ) frozen_{name}\n)");
                    let extract = |source: &'static str| {
                        assert_eq!(source.matches(&start).count(), 1);
                        let after = source.split_once(&start).unwrap().1;
                        assert_eq!(after.matches(&end).count(), 1);
                        after.split_once(&end).unwrap().0
                    };
                    assert_eq!(
                        extract(CANDIDATE_INPUT),
                        extract(COMPLETE_INPUT),
                        "historical body bytes changed: {name}"
                    );
                }
                json!({"old":old,"candidate_oracle":oracle,"candidate_oracle_sha256":digest(CANDIDATE_ORACLE)})
            }

            fn candidate_prefix() -> &'static str {
                const BOUNDARY: &str = "), row_type_valid AS (\n";
                assert_eq!(CANDIDATE_INPUT.matches(BOUNDARY).count(), 1);
                CANDIDATE_INPUT.split_once(BOUNDARY).unwrap().0
            }

            fn column_owner_audit_sql() -> String {
                let original = toast_audit_sql();
                assert_eq!(original.matches(complete_address_prefix()).count(), 1);
                let query = original.replacen(complete_address_prefix(), candidate_prefix(), 1);
                const ORDINARY: &str = r#"), ordinary_native_inverse_inputs AS MATERIALIZED (
     SELECT * FROM toast_original_inputs WHERE
     native_type IS DISTINCT FROM 'toast table' AND input_type IS DISTINCT FROM 'toast table'
    )"#;
                assert_eq!(query.matches(ORDINARY).count(), 1);
                let replacement = format!(
                    "), column_inputs AS MATERIALIZED (SELECT * FROM toast_original_inputs WHERE {IS_COLUMN}),\n{COLUMN_CTES}, ordinary_native_inverse_inputs AS MATERIALIZED (SELECT * FROM toast_original_inputs WHERE native_type IS DISTINCT FROM 'toast table' AND input_type IS DISTINCT FROM 'toast table' AND NOT ({IS_COLUMN}))"
                );
                let query = query.replace(ORDINARY, &replacement);
                const OPEN: &str = ", inverse AS MATERIALIZED (\n     SELECT t.classid";
                assert_eq!(query.matches(OPEN).count(), 1);
                let query = query.replace(
                    OPEN,
                    ", toast_and_ordinary_inverse AS MATERIALIZED (\n     SELECT t.classid",
                );
                const END: &str = "pg_catalog.pg_get_object_address(t.native_type,t.native_names,t.native_args) inverse ON true\n    )";
                assert_eq!(query.matches(END).count(), 1);
                let query = query.replace(
                    END,
                    &format!(
                        r#"{END}, inverse AS MATERIALIZED (
 SELECT t.*,NULL::jsonb AS column_witness FROM toast_and_ordinary_inverse t
 UNION ALL
 SELECT c.classid,c.objid,c.objsubid,c.native_address,c.address,c.valid,c.ri_flags,
 c.inverse_classid,c.inverse_objid,c.inverse_objsubid,NULL::jsonb,NULL::jsonb,NULL::jsonb,
 NULL::bigint,NULL::bigint,NULL::bigint,NULL::bigint,NULL::bigint,NULL::bigint,c.column_witness
 FROM column_inverse c
)"#
                    ),
                );
                const FIELD: &str = "'TOAST_incoming_canonical_count',incoming_canonical_count)";
                assert_eq!(query.matches(FIELD).count(), 1);
                query.replace(FIELD, "'TOAST_incoming_canonical_count',incoming_canonical_count,'COLUMN_witness',column_witness)")
            }

            async fn owner_audit(connection: &mut PgConnection) -> Value {
                toast_audit_with(connection, column_owner_audit_sql(), json!([])).await
            }

            fn column_entry(e: &Value) -> bool {
                e["tuple"][2].as_i64().is_some_and(|n| n > 0)
                    && COLUMN_TYPES.iter().any(|kind| e["native"]["type"] == *kind)
            }

            fn checked_column(e: &Value) -> Result<(), String> {
                let w = &e["COLUMN_witness"];
                let expected_type = match w["kind"].as_str() {
                    Some("r" | "p") => Some("table column"),
                    Some("f") => Some("foreign table column"),
                    Some("v") => Some("view column"),
                    Some("m") => Some("materialized view column"),
                    Some("c") => Some("composite type column"),
                    Some("S") => Some("sequence column"),
                    _ => None,
                };
                if expected_type.is_none()
                    || e["native"]["type"].as_str() != expected_type
                    || e["tuple"] != e["inverse_tuple"]
                    || w["checks"] != json!(vec![true; 7])
                    || w["namespace_count"] != 1
                    || w["relation_count"] != 1
                    || w["column_count"] != 1
                    || w["found_tuple"] != e["tuple"]
                    || w["function_tuple"] != e["tuple"]
                    || w["found_native"] != e["native"]
                    || w["inverse_namespace"] != "table column"
                {
                    return Err(
                        "independent finite catalog/native/namespace inverse mismatch".into(),
                    );
                }
                Ok(())
            }

            // The prerequisite deliberately does not assert the unresolved
            // positive-column portable facts. No audit field is rewritten.
            // Existing inverse_positive remains byte-identical and strict.
            fn owner_inverse_prerequisite(audit: &Value) -> Result<(), String> {
                census(audit)?;
                let mut tuples = BTreeSet::new();
                let mut columns = 0;
                let mut toast = 0;
                let mut expected_toast = BTreeSet::new();
                for e in audit["inverse"].as_array().ok_or("missing inverse")? {
                    if e["tuple"] != e["inverse_tuple"]
                        || !tuples.insert(e["inverse_tuple"].to_string())
                    {
                        return Err("complete inverse tuple mismatch/collision".into());
                    }
                    if column_entry(e) {
                        checked_column(e)?;
                        columns += 1;
                    } else if e["native"]["type"] == "toast table" {
                        toast += 1;
                        if e["TOAST_guards"] != json!(vec![true; 8])
                            || e["TOAST_lookup_count"] != 1
                            || e["TOAST_parent_global_count"] != 1
                            || e["TOAST_parent_selected_count"] != 1
                            || e["TOAST_internal_global_count"] != 1
                            || e["TOAST_canonical_count"] != 1
                            || e["TOAST_incoming_canonical_count"] != 1
                            || !e["TOAST_expected"].is_object()
                            || !expected_toast.insert(e["TOAST_expected"].to_string())
                        {
                            return Err("all eight retained TOAST inverse guards required".into());
                        }
                        for field in ["M", "I"] {
                            if audit["scope"][field]
                                .as_array()
                                .ok_or("missing M/I")?
                                .contains(&e["tuple"])
                            {
                                return Err("TOAST child metadata/incoming scope expansion".into());
                            }
                        }
                    } else if e["valid"] != true || !e["portable"].is_object() {
                        return Err("unrelated ordinary/RI/builtin identity prerequisite".into());
                    }
                }
                if columns == 0 || toast == 0 {
                    return Err("missing actual owner column/TOAST witness".into());
                }
                Ok(())
            }

            fn portable_columns_required(audit: &Value) -> Result<(), String> {
                owner_inverse_prerequisite(audit)?;
                for e in audit["inverse"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|e| column_entry(e))
                {
                    if e["valid"] != true
                        || e["portable"] != e["native"]
                        || !e["portable"].is_object()
                    {
                        return Err(format!(
                            "actual positive column {} has valid={} portable={}",
                            e["native"], e["valid"], e["portable"]
                        ));
                    }
                }
                Ok(())
            }

            const JSON_COLUMN_INPUTS: &str = r#"WITH column_json AS MATERIALIZED (
 SELECT x,x->'native' AS native,
 (x->'tuple'->>0)::oid AS classid,(x->'tuple'->>1)::oid AS objid,(x->'tuple'->>2)::integer AS objsubid
 FROM jsonb_array_elements($1::jsonb) x
), column_inputs AS MATERIALIZED (
 SELECT classid,objid,objsubid,native AS native_address,native->>'type' AS native_type,
 ARRAY(SELECT jsonb_array_elements_text(native->'object_names')) AS native_names,
 ARRAY(SELECT jsonb_array_elements_text(native->'object_args')) AS native_args,
 NULL::jsonb AS address,NULL::boolean AS valid,NULL::jsonb AS ri_flags,
 CASE WHEN x ? 'input_classid' THEN (x->>'input_classid')::oid ELSE classid END AS input_classid,
 CASE WHEN x ? 'input_objid' THEN (x->>'input_objid')::oid ELSE objid END AS input_objid,
 CASE WHEN x ? 'input_objsubid' THEN (x->>'input_objsubid')::integer ELSE objsubid END AS input_objsubid,
 CASE WHEN x ? 'input_type' THEN x->>'input_type' ELSE native->>'type' END AS input_type,
 CASE WHEN x ? 'input_names' THEN CASE WHEN x->'input_names'='null'::jsonb THEN NULL
 ELSE ARRAY(SELECT jsonb_array_elements_text(x->'input_names')) END
 ELSE ARRAY(SELECT jsonb_array_elements_text(native->'object_names')) END AS input_names,
 CASE WHEN x ? 'input_args' THEN CASE WHEN x->'input_args'='null'::jsonb THEN NULL
 ELSE ARRAY(SELECT jsonb_array_elements_text(x->'input_args')) END
 ELSE ARRAY(SELECT jsonb_array_elements_text(native->'object_args')) END AS input_args
 FROM column_json
)"#;

            fn family_query(ctes: &str, inputs: &str) -> String {
                format!(
                    r#"{inputs},{ctes}
SELECT jsonb_agg(jsonb_build_object('tuple',jsonb_build_array(classid::text,objid::text,objsubid),
 'native',native_address,'inverse_tuple',jsonb_build_array(inverse_classid::text,inverse_objid::text,inverse_objsubid),
 'COLUMN_witness',column_witness) ORDER BY classid,objid,objsubid)::text FROM column_inverse"#
                )
            }

            async fn family_inverse(
                connection: &mut PgConnection,
                input: &Value,
                ctes: &str,
                inputs: &str,
            ) -> Value {
                sqlx::raw_sql(READER_SESSION)
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                let started = Instant::now();
                let raw: String = sqlx::query_scalar(sqlx::AssertSqlSafe(family_query(ctes, inputs)))
                    .bind(sqlx::types::Json(input)).fetch_one(&mut *connection).await
                    .expect("PREREQUISITE: actual finite column inverse must execute; SQL/timeout is not RED");
                assert!(started.elapsed() <= Duration::from_secs(3));
                serde_json::from_str(&raw).unwrap()
            }

            fn family_positive(audit: &Value) -> Result<(), String> {
                let rows = audit.as_array().ok_or("missing seven-kind array")?;
                if rows.len() != 7 {
                    return Err("seven exact actual kind witnesses required".into());
                }
                let mut kinds = BTreeSet::new();
                let mut tuples = BTreeSet::new();
                for e in rows {
                    checked_column(e)?;
                    if !kinds.insert(e["COLUMN_witness"]["kind"].to_string())
                        || !tuples.insert(e["tuple"].to_string())
                    {
                        return Err("duplicate catalog kind or tuple".into());
                    }
                }
                let expected: BTreeSet<_> = ["r", "p", "f", "v", "m", "c", "S"]
                    .into_iter()
                    .map(|x| json!(x).to_string())
                    .collect();
                if kinds != expected {
                    return Err("wrong finite catalog family".into());
                }
                Ok(())
            }

            async fn family_raw(connection: &mut PgConnection) -> Value {
                let raw: String = sqlx::query_scalar(r#"WITH n AS (SELECT oid FROM pg_catalog.pg_namespace WHERE nspname='org_column_v3_probe'),
 c AS (SELECT oid FROM pg_catalog.pg_class WHERE relnamespace IN(SELECT oid FROM n))
SELECT jsonb_build_object('namespace',(SELECT jsonb_agg(to_jsonb(x)||jsonb_build_object('xmin',x.xmin::text,'ctid',x.ctid::text) ORDER BY x.oid) FROM pg_catalog.pg_namespace x WHERE x.oid IN(SELECT oid FROM n)),
 'relations',(SELECT jsonb_agg((to_jsonb(x)-ARRAY['relpages','reltuples','relallvisible','relallfrozen','relfrozenxid','relminmxid'])||jsonb_build_object('xmin',x.xmin::text,'ctid',x.ctid::text) ORDER BY x.oid) FROM pg_catalog.pg_class x WHERE x.oid IN(SELECT oid FROM c)),
 'attributes',(SELECT jsonb_agg(to_jsonb(x)||jsonb_build_object('xmin',x.xmin::text,'ctid',x.ctid::text) ORDER BY x.attrelid,x.attnum) FROM pg_catalog.pg_attribute x WHERE x.attrelid IN(SELECT oid FROM c)),
 'types',(SELECT jsonb_agg(to_jsonb(x)||jsonb_build_object('xmin',x.xmin::text,'ctid',x.ctid::text) ORDER BY x.oid) FROM pg_catalog.pg_type x WHERE x.typnamespace IN(SELECT oid FROM n)),
 'dependencies',(SELECT jsonb_agg(to_jsonb(x)||jsonb_build_object('xmin',x.xmin::text,'ctid',x.ctid::text) ORDER BY to_jsonb(x)::text COLLATE "C",x.ctid) FROM pg_catalog.pg_depend x WHERE (x.classid='pg_catalog.pg_class'::regclass AND x.objid IN(SELECT oid FROM c)) OR (x.refclassid='pg_catalog.pg_class'::regclass AND x.refobjid IN(SELECT oid FROM c))))::text"#)
                    .fetch_one(connection).await.unwrap();
                serde_json::from_str(&raw).unwrap()
            }

            fn source_family_query() -> String {
                const INPUT: &str = "), addresses(classid,objid,objsubid) AS MATERIALIZED (\n";
                const NEXT: &str = "), type_format_inputs AS MATERIALIZED (\n";
                let source = candidate_prefix();
                assert_eq!(source.matches(INPUT).count(), 1);
                assert_eq!(source.matches(NEXT).count(), 1);
                let (before, input_and_after) = source.split_once(INPUT).unwrap();
                let (_, after) = input_and_after.split_once(NEXT).unwrap();
                // Only the addresses-input body receives genuinely observed
                // isolated catalog tuples. The complete source prefix and all
                // native/reconstruction/checked/RI/stable expressions and their
                // real dependencies remain unchanged. This characterizes the
                // finite seam, not full source scope or business-browser work.
                format!(
                    r#"{before}{INPUT}
 SELECT (x->'tuple'->>0)::oid,(x->'tuple'->>1)::oid,(x->'tuple'->>2)::integer
 FROM jsonb_array_elements($1::jsonb) x
{NEXT}{after})
SELECT jsonb_agg(jsonb_build_object(
 'tuple',jsonb_build_array(a.classid::text,a.objid::text,a.objsubid),
 'kind',c.relkind,'native',n.native_address,
 'expected_type',r.expected_type,'expected_names',to_jsonb(r.expected_names),
 'expected_args',to_jsonb(r.expected_args),'catalog_valid',r.catalog_valid,
 'native_valid',checked.native_valid,'valid',s.valid,'portable',s.address)
 ORDER BY a.classid,a.objid,a.objsubid)::text
FROM addresses a LEFT JOIN native_addresses n USING(classid,objid,objsubid)
LEFT JOIN reconstruction r USING(classid,objid,objsubid)
LEFT JOIN checked_native checked USING(classid,objid,objsubid)
LEFT JOIN stable_addresses s USING(classid,objid,objsubid)
LEFT JOIN pg_catalog.pg_class c ON a.classid='pg_catalog.pg_class'::regclass AND c.oid=a.objid"#
                )
            }

            async fn source_family_observation(
                connection: &mut PgConnection,
                inputs: &Value,
            ) -> Value {
                sqlx::raw_sql(READER_SESSION)
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                let started = Instant::now();
                let raw: String = sqlx::query_scalar(sqlx::AssertSqlSafe(source_family_query()))
                    .bind(sqlx::types::Json(inputs)).fetch_one(&mut *connection).await
                    .expect("PREREQUISITE: pinned source seven-kind reconstruction must execute; SQL/timeout is not RED");
                assert!(
                    started.elapsed() <= Duration::from_secs(3),
                    "PREREQUISITE: source seven-kind seam exceeded 3s"
                );
                serde_json::from_str(&raw).unwrap()
            }

            fn source_family_census(source: &Value, inverses: &Value) -> Result<(), String> {
                family_positive(inverses)?;
                let rows = source.as_array().ok_or("missing actual source family")?;
                if rows.len() != 7 {
                    return Err("source family must retain seven actual addresses".into());
                }
                let mut tuples = BTreeSet::new();
                for row in rows {
                    if !tuples.insert(row["tuple"].to_string()) {
                        return Err("duplicate source family tuple".into());
                    }
                    let independent = inverses
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|e| e["tuple"] == row["tuple"])
                        .ok_or("source row not in independently inverted family")?;
                    if row["kind"] != independent["COLUMN_witness"]["kind"]
                        || row["native"] != independent["native"]
                    {
                        return Err(
                            "source/native/catalog witness differs from actual independent family"
                                .into(),
                        );
                    }
                    for field in ["catalog_valid", "native_valid", "valid"] {
                        if !row[field].is_boolean() {
                            return Err(format!("source family missing boolean {field}"));
                        }
                    }
                    let object = row
                        .as_object()
                        .ok_or("source family row is not an object")?;
                    for field in [
                        "expected_type",
                        "expected_names",
                        "expected_args",
                        "portable",
                    ] {
                        if !object.contains_key(field) {
                            return Err(format!("source family omitted {field}"));
                        }
                    }
                }
                Ok(())
            }

            fn source_family_positive(source: &Value, inverses: &Value) -> Result<(), String> {
                source_family_census(source, inverses)?;
                for row in source.as_array().unwrap() {
                    let independent = inverses
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|e| e["tuple"] == row["tuple"])
                        .unwrap();
                    if row["catalog_valid"] != true
                        || row["native_valid"] != true
                        || row["valid"] != true
                        || row["expected_type"] != independent["native"]["type"]
                        || row["expected_names"] != independent["native"]["object_names"]
                        || row["expected_args"] != independent["native"]["object_args"]
                        || row["portable"] != independent["native"]
                        || !row["portable"].is_object()
                    {
                        return Err(format!(
                            "actual source kind {} lacks exact native reconstruction/validity/portable identity: {}",
                            row["kind"], row
                        ));
                    }
                }
                Ok(())
            }

            fn source_family_machinery_controls(inverses: &Value) {
                family_positive(inverses)
                    .expect("PREREQUISITE: actual seven-kind inverse before evidence controls");
                // A separate normative evidence model, derived from genuine
                // independent catalog/native/function observations. It is not
                // a source observation and never qualifies the candidate.
                let oracle_positive = Value::Array(inverses.as_array().unwrap().iter().map(|e| json!({
                    "tuple":e["tuple"],"kind":e["COLUMN_witness"]["kind"],"native":e["native"],
                    "expected_type":e["native"]["type"],"expected_names":e["native"]["object_names"],
                    "expected_args":e["native"]["object_args"],"catalog_valid":true,"native_valid":true,
                    "valid":true,"portable":e["native"]
                })).collect());
                source_family_positive(&oracle_positive, inverses)
                    .expect("EVIDENCE_ONLY: uncorrupted seven-kind predicate model must pass");
                for selected in 0..7 {
                    for field in [
                        "catalog_valid",
                        "native_valid",
                        "valid",
                        "portable",
                        "expected_type",
                        "expected_names",
                        "expected_args",
                        "native",
                    ] {
                        let mut damaged = oracle_positive.clone();
                        damaged[selected][field] =
                            if ["catalog_valid", "native_valid", "valid"].contains(&field) {
                                json!(false)
                            } else {
                                Value::Null
                            };
                        assert!(
                            source_family_positive(&damaged, inverses).is_err(),
                            "source predicate ignored {field} for actual kind {}",
                            damaged[selected]["kind"]
                        );
                    }
                }
                let mut omitted = oracle_positive.clone();
                omitted.as_array_mut().unwrap().pop();
                assert!(source_family_positive(&omitted, inverses).is_err());
                let mut duplicate = oracle_positive.clone();
                duplicate[6] = duplicate[0].clone();
                assert!(source_family_positive(&duplicate, inverses).is_err());
                source_family_positive(&oracle_positive, inverses).unwrap();
            }
            async fn seven_kind_positive_and_controls(
                connection: &mut PgConnection,
            ) -> (Value, Value) {
                let before = catalog(connection).await;
                let ledger = applied_ledger(connection).await;
                let business = complete_business(connection, false).await;
                assert!(
                    sqlx::query_scalar::<_, bool>(
                        "SELECT to_regnamespace('org_column_v3_probe') IS NULL"
                    )
                    .fetch_one(&mut *connection)
                    .await
                    .unwrap()
                );
                sqlx::raw_sql("SAVEPOINT column_family")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                let mut observation = None;
                let result = AssertUnwindSafe(async {
                    sqlx::raw_sql(r#"CREATE SCHEMA org_column_v3_probe;
CREATE TABLE org_column_v3_probe.source ("값" integer);
CREATE TABLE org_column_v3_probe.partitioned_source ("값" integer) PARTITION BY RANGE("값");
CREATE VIEW org_column_v3_probe.projection AS SELECT "값" FROM org_column_v3_probe.source;
CREATE MATERIALIZED VIEW org_column_v3_probe.materialized_projection AS SELECT "값" FROM org_column_v3_probe.source WITH NO DATA;
CREATE TYPE org_column_v3_probe.record_type AS ("값" integer);
CREATE SEQUENCE org_column_v3_probe.sequence_value;
CREATE EXTENSION IF NOT EXISTS file_fdw;
CREATE SERVER org_column_v3_server FOREIGN DATA WRAPPER file_fdw;
CREATE FOREIGN TABLE org_column_v3_probe.foreign_source ("값" integer) SERVER org_column_v3_server OPTIONS (filename '/dev/null', format 'csv');"#)
                        .execute(&mut *connection).await.unwrap();
                    let mut fixture_business = business.clone();
                    for identity in [
                        r#"["org_column_v3_probe","source"]"#,
                        r#"["org_column_v3_probe","partitioned_source"]"#,
                    ] {
                        assert!(
                            fixture_business.insert(identity.into(), "[]".into()).is_none(),
                            "fixture business identity must be absent from original census"
                        );
                    }
                    assert_eq!(complete_business(connection, false).await, fixture_business);
                    let raw: String = sqlx::query_scalar(r#"SELECT jsonb_agg(jsonb_build_object('tuple',jsonb_build_array('pg_catalog.pg_class'::regclass::oid::text,c.oid::text,a.attnum::integer),
 'native',jsonb_build_object('type',ident.type,'object_names',to_jsonb(ident.object_names),'object_args',to_jsonb(ident.object_args))) ORDER BY c.oid)::text
FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
JOIN pg_catalog.pg_attribute a ON a.attrelid=c.oid AND a.attnum=1 AND NOT a.attisdropped
CROSS JOIN LATERAL pg_catalog.pg_identify_object_as_address('pg_catalog.pg_class'::regclass::oid,c.oid,a.attnum::integer) ident
WHERE n.nspname='org_column_v3_probe' AND c.relkind IN('r','p','f','v','m','c','S')"#)
                        .fetch_one(&mut *connection).await.unwrap();
                    let inputs: Value = serde_json::from_str(&raw).unwrap();
                    let baseline = family_inverse(connection, &inputs, COLUMN_CTES, JSON_COLUMN_INPUTS).await;
                    family_positive(&baseline).expect("PREREQUISITE: seven actual catalog/native/function positives");
                    let source = source_family_observation(connection, &inputs).await;
                    source_family_census(&source, &baseline).expect("PREREQUISITE: actual pinned source seven-kind native/census observation");
                    source_family_machinery_controls(&baseline);
                    let saved = family_raw(connection).await;
                    let selected = inputs.as_array().unwrap().iter().position(|x|x["native"]["type"]=="view column").unwrap();
                    let selected_tuple = inputs[selected]["tuple"].clone();
                    for (field,value) in [
                        ("input_classid",json!("0")),("input_objid",json!("0")),
                        ("input_objsubid",json!(0)),("input_objsubid",json!(-1)),
                        ("input_type",json!("table column")),("input_type",Value::Null),
                        ("input_names",Value::Null),("input_names",json!([])),
                        ("input_names",json!(["org_column_v3_probe","projection"])),
                        ("input_names",json!(["org_column_v3_probe","projection","값","extra"])),
                        ("input_names",json!(["org_column_v3_probe","projection",null])),
                        ("input_names",json!(["org_column_v3_probe","absent","값"])),
                        ("input_names",json!(["absent","projection","값"])),
                        ("input_args",Value::Null),("input_args",json!(["extra"])),("input_args",json!([null])),
                    ] {
                        let mut bad = inputs.clone(); bad[selected][field] = value;
                        let refused = family_inverse(connection, &bad, COLUMN_CTES, JSON_COLUMN_INPUTS).await;
                        assert_eq!(refused.as_array().unwrap().len(), 7);
                        let e=refused.as_array().unwrap().iter().find(|e|e["tuple"]==selected_tuple).unwrap();
                        assert!(checked_column(e).is_err(), "malformed input passed: {field}");
                        assert_ne!(e["tuple"],e["inverse_tuple"],"no usable malformed inverse");
                        assert_eq!(family_raw(connection).await,saved);
                        assert_eq!(family_inverse(connection,&inputs,COLUMN_CTES,JSON_COLUMN_INPUTS).await,baseline);
                    }
                    for (from,to) in [
                        ("count(*) AS column_match_count","count(*)+1 AS column_match_count"),
                        ("(array_agg(c.oid ORDER BY c.oid,a.attnum))[1] AS observed_oid","t.input_objid AS observed_oid"),
                    ] {
                        assert_eq!(COLUMN_CTES.matches(from).count(),1);
                        let corrupted=COLUMN_CTES.replace(from,to);
                        let mut missing=inputs.clone(); missing[selected]["input_names"]=json!(["org_column_v3_probe","absent","값"]);
                        let refused=family_inverse(connection,&missing,&corrupted,JSON_COLUMN_INPUTS).await;
                        assert!(family_positive(&refused).is_err(),"lookup corruption hid missing catalog discovery");
                        assert_eq!(family_inverse(connection,&inputs,COLUMN_CTES,JSON_COLUMN_INPUTS).await,baseline);
                    }
                    const NAMES: &str = "ELSE ARRAY(SELECT jsonb_array_elements_text(native->'object_names')) END AS input_names";
                    assert_eq!(JSON_COLUMN_INPUTS.matches(NAMES).count(),1);
                    for expression in ["array_fill('unused'::text,ARRAY[3],ARRAY[0])","array_fill('unused'::text,ARRAY[1,3])"] {
                        let changed=JSON_COLUMN_INPUTS.replace(NAMES,&format!("ELSE {expression} END AS input_names"));
                        let refused=family_inverse(connection,&inputs,COLUMN_CTES,&changed).await;
                        assert_eq!(refused.as_array().unwrap().len(),7);
                        assert!(family_positive(&refused).is_err(),"malformed SQL array shape passed");
                        for entry in refused.as_array().unwrap() { assert_ne!(entry["tuple"],entry["inverse_tuple"]); }
                        assert_eq!(family_raw(connection).await,saved);
                        assert_eq!(family_inverse(connection,&inputs,COLUMN_CTES,JSON_COLUMN_INPUTS).await,baseline);
                    }
                    const NAME_FIND: &str = r#"SELECT count(*) AS column_match_count,
   (array_agg(c.oid ORDER BY c.oid,a.attnum))[1] AS observed_oid,
   (array_agg(a.attnum ORDER BY c.oid,a.attnum))[1] AS observed_attnum
  FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
  JOIN pg_catalog.pg_attribute a ON a.attrelid=c.oid
  WHERE n.nspname::text COLLATE "C"=t.input_names[1] COLLATE "C" AND c.relname::text COLLATE "C"=t.input_names[2] COLLATE "C"
   AND a.attname::text COLLATE "C"=t.input_names[3] COLLATE "C""#;
                    assert_eq!(COLUMN_CTES.matches(NAME_FIND).count(),1);
                    let expected_oid_without_lookup=COLUMN_CTES.replace(NAME_FIND,"SELECT 1::bigint AS column_match_count,t.input_objid AS observed_oid,t.input_objsubid::smallint AS observed_attnum");
                    let mut nonexistent_attribute=inputs.clone();
                    nonexistent_attribute[selected]["input_names"]=json!(["org_column_v3_probe","projection","absent"]);
                    let rejected=family_inverse(connection,&nonexistent_attribute,&expected_oid_without_lookup,JSON_COLUMN_INPUTS).await;
                    assert!(family_positive(&rejected).is_err(),"fabricated lookup count and expected OID hid missing attribute discovery");
                    let selected_entry=rejected.as_array().unwrap().iter().find(|e|e["tuple"]==selected_tuple).unwrap();
                    assert_ne!(selected_entry["tuple"],selected_entry["inverse_tuple"]);
                    let mut forged=baseline.clone();
                    let chosen=forged.as_array_mut().unwrap().iter_mut().find(|e|e["tuple"]==selected_tuple).unwrap();
                    chosen["COLUMN_witness"]["column_count"]=json!(0);
                    chosen["inverse_tuple"]=chosen["tuple"].clone();
                    assert!(family_positive(&forged).is_err(),"copied expected inverse hid omitted lookup");
                    // A genuine unsupported index supplies its own catalog/native
                    // address. Never fabricate an index by changing a view's kind.
                    let index_raw: String = sqlx::query_scalar(r#"SELECT jsonb_build_object(
 'tuple',jsonb_build_array('pg_catalog.pg_class'::regclass::oid::text,c.oid::text,a.attnum::integer),
 'native',jsonb_build_object('type',ident.type,'object_names',to_jsonb(ident.object_names),'object_args',to_jsonb(ident.object_args)))::text
FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
JOIN pg_catalog.pg_index i ON i.indexrelid=c.oid AND i.indrelid='pg_catalog.pg_class'::regclass
JOIN pg_catalog.pg_attribute a ON a.attrelid=c.oid AND a.attnum=1 AND NOT a.attisdropped AND a.atttypid<>0
CROSS JOIN LATERAL pg_catalog.pg_identify_object_as_address('pg_catalog.pg_class'::regclass::oid,c.oid,a.attnum::integer) ident
WHERE c.oid='pg_catalog.pg_class_oid_index'::regclass AND c.relkind='i'
 AND i.indisvalid AND i.indisready AND i.indislive AND n.nspname='pg_catalog'"#)
                        .fetch_one(&mut *connection).await
                        .expect("PREREQUISITE: genuine unsupported index-column catalog/native observation must execute");
                    let index_input: Value = serde_json::from_str(&index_raw).unwrap();
                    let mut unsupported_index = inputs.clone();
                    unsupported_index[selected] = index_input.clone();
                    let refused = family_inverse(connection,&unsupported_index,COLUMN_CTES,JSON_COLUMN_INPUTS).await;
                    assert_eq!(refused.as_array().unwrap().len(),7);
                    let e = refused.as_array().unwrap().iter().find(|e|e["tuple"]==index_input["tuple"]).unwrap();
                    let w = &e["COLUMN_witness"];
                    assert_eq!(w["kind"],"i");
                    assert_eq!(w["namespace_count"],1); assert_eq!(w["relation_count"],1); assert_eq!(w["column_count"],1);
                    assert_eq!(w["found_tuple"],index_input["tuple"]);
                    assert_eq!(w["checks"],json!([true,true,true,false,true,true,false]));
                    assert_eq!(e["native"],index_input["native"]);
                    assert_eq!(e["inverse_tuple"],json!([null,null,null]));
                    assert!(checked_column(e).is_err());
                    for original_entry in baseline.as_array().unwrap().iter().filter(|e|e["tuple"]!=selected_tuple) {
                        let retained = refused.as_array().unwrap().iter().find(|e|e["tuple"]==original_entry["tuple"]).unwrap();
                        assert_eq!(retained,original_entry,"unsupported index changed another actual witness");
                    }
                    assert_eq!(family_raw(connection).await,saved,"unsupported index input changed raw/MVCC custody");
                    assert_eq!(family_inverse(connection,&inputs,COLUMN_CTES,JSON_COLUMN_INPUTS).await,baseline);
                    assert_eq!(source_family_observation(connection,&inputs).await,source,"unsupported index changed pinned seven-kind source observation");
                    // Materialized views already have valid table AM/storage.
                    // Changing only this genuine relation's kind isolates mismatch.
                    let materialized = inputs.as_array().unwrap().iter().position(|x|x["native"]["type"]=="materialized view column").unwrap();
                    let materialized_tuple = inputs[materialized]["tuple"].clone();
                    for (mutation,mutation_tuple) in [
                        ("UPDATE pg_catalog.pg_class SET relname='renamed_column_v3' WHERE oid=$1::text::oid AND $2::integer>0",&selected_tuple),
                        ("UPDATE pg_catalog.pg_class SET relkind='r' WHERE oid=$1::text::oid AND relkind='m' AND $2::integer>0",&materialized_tuple),
                        ("UPDATE pg_catalog.pg_class SET relnamespace='pg_catalog'::regnamespace WHERE oid=$1::text::oid AND $2::integer>0",&selected_tuple),
                        ("UPDATE pg_catalog.pg_attribute SET attname='renamed_column_v3' WHERE attrelid=$1::text::oid AND attnum=$2::integer",&selected_tuple),
                        ("UPDATE pg_catalog.pg_attribute SET attisdropped=true WHERE attrelid=$1::text::oid AND attnum=$2::integer",&selected_tuple),
                        ("UPDATE pg_catalog.pg_attribute SET atttypid=0 WHERE attrelid=$1::text::oid AND attnum=$2::integer",&selected_tuple),
                    ] {
                        sqlx::raw_sql("SAVEPOINT column_catalog_control").execute(&mut *connection).await.unwrap();
                        let trial=AssertUnwindSafe(async {
                            let count=sqlx::query(sqlx::AssertSqlSafe(mutation.to_owned()))
                                .bind(mutation_tuple[1].as_str().unwrap()).bind(mutation_tuple[2].as_i64().unwrap() as i32)
                                .execute(&mut *connection).await.unwrap().rows_affected();
                            assert_eq!(count,1,"exact catalog mutation admission");
                            let refused=family_inverse(connection,&inputs,COLUMN_CTES,JSON_COLUMN_INPUTS).await;
                            assert_eq!(refused.as_array().unwrap().len(),7);
                            let e=refused.as_array().unwrap().iter().find(|e|e["tuple"]==*mutation_tuple).unwrap();
                            assert!(checked_column(e).is_err()); assert_ne!(e["tuple"],e["inverse_tuple"]);
                        }).catch_unwind().await;
                        sqlx::raw_sql("ROLLBACK TO SAVEPOINT column_catalog_control; RELEASE SAVEPOINT column_catalog_control").execute(&mut *connection).await.unwrap();
                        assert_eq!(family_raw(connection).await,saved,"fresh complete raw/MVCC family restoration");
                        assert_eq!(family_inverse(connection,&inputs,COLUMN_CTES,JSON_COLUMN_INPUTS).await,baseline);
                        assert_eq!(source_family_observation(connection,&inputs).await,source,"fresh pinned source seven-kind restoration");
                        if let Err(panic)=trial { std::panic::resume_unwind(panic); }
                    }
                    assert_eq!(applied_ledger(connection).await,ledger);
                    assert_eq!(complete_business(connection,false).await,fixture_business);
                    observation = Some((source, baseline));
                }).catch_unwind().await;
                sqlx::raw_sql(
                    "ROLLBACK TO SAVEPOINT column_family; RELEASE SAVEPOINT column_family",
                )
                .execute(&mut *connection)
                .await
                .unwrap();
                assert!(
                    sqlx::query_scalar::<_, bool>(
                        "SELECT to_regnamespace('org_column_v3_probe') IS NULL"
                    )
                    .fetch_one(&mut *connection)
                    .await
                    .unwrap()
                );
                assert_eq!(
                    catalog(connection).await,
                    before,
                    "catalog family rollback must restore existing custody"
                );
                assert_eq!(applied_ledger(connection).await, ledger);
                assert_eq!(complete_business(connection, false).await, business);
                if let Err(panic) = result {
                    std::panic::resume_unwind(panic);
                }
                observation.expect(
                    "PREREQUISITE: actual source family observation retained before rollback",
                )
            }

            const OWNER_VIEW_ABSENT: &str = "SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_namespace WHERE nspname='org_column_v3_owner_probe') AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_proc WHERE proname='native_org_unit_column_witness_v3')";
            const OWNER_VIEW_FIXTURE: &str = r#"CREATE SCHEMA org_column_v3_owner_probe;
REVOKE ALL ON SCHEMA org_column_v3_owner_probe FROM PUBLIC;
CREATE VIEW org_column_v3_owner_probe.projection AS SELECT 1::integer AS "값";
CREATE FUNCTION org_column_v3_owner_probe.native_org_unit_column_witness_v3()
RETURNS integer LANGUAGE sql SECURITY INVOKER
BEGIN ATOMIC
 SELECT "값" FROM org_column_v3_owner_probe.projection;
END;
REVOKE ALL ON FUNCTION org_column_v3_owner_probe.native_org_unit_column_witness_v3() FROM PUBLIC;"#;
            const OWNER_VIEW_WITNESS: &str = r#"SELECT jsonb_build_object(
 'routine_tuple',jsonb_build_array('pg_catalog.pg_proc'::regclass::oid::text,p.oid::text,0),
 'view_tuple',jsonb_build_array('pg_catalog.pg_class'::regclass::oid::text,c.oid::text,a.attnum::integer),
 'native',jsonb_build_object('type',ident.type,'object_names',to_jsonb(ident.object_names),'object_args',to_jsonb(ident.object_args)),
 'parsed_invoker_admin',p.prosqlbody IS NOT NULL AND NOT p.prosecdef AND l.lanname='sql'
   AND p.proowner=(SELECT oid FROM pg_catalog.pg_roles WHERE rolname=current_user),
 'no_PUBLIC_execute',NOT EXISTS(SELECT 1 FROM pg_catalog.aclexplode(COALESCE(p.proacl,pg_catalog.acldefault('f',p.proowner))) acl
   WHERE acl.grantee=0 AND acl.privilege_type='EXECUTE'),
 'no_nonsuper_execute',NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles r WHERE NOT r.rolsuper
   AND pg_catalog.has_function_privilege(r.oid,p.oid,'EXECUTE')),
 'dependencies',(SELECT COALESCE(jsonb_agg(to_jsonb(d)||jsonb_build_object('xmin',d.xmin::text,'ctid',d.ctid::text)
   ORDER BY to_jsonb(d)::text COLLATE "C",d.ctid),'[]'::jsonb)
   FROM pg_catalog.pg_depend d WHERE d.classid='pg_catalog.pg_proc'::regclass AND d.objid=p.oid AND d.objsubid=0
    AND d.refclassid='pg_catalog.pg_class'::regclass AND d.refobjid=c.oid AND d.refobjsubid=a.attnum))::text
FROM pg_catalog.pg_namespace n JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid
JOIN pg_catalog.pg_attribute a ON a.attrelid=c.oid AND a.attnum=1 AND NOT a.attisdropped AND a.atttypid<>0
JOIN pg_catalog.pg_proc p ON p.pronamespace=n.oid AND p.proname='native_org_unit_column_witness_v3'
 AND p.prokind='f' AND p.pronargs=0
JOIN pg_catalog.pg_language l ON l.oid=p.prolang
CROSS JOIN LATERAL pg_catalog.pg_identify_object_as_address('pg_catalog.pg_class'::regclass::oid,c.oid,a.attnum::integer) ident
WHERE n.nspname='org_column_v3_owner_probe' AND c.relname='projection' AND c.relkind='v'"#;

            async fn owner_view_observation(
                connection: &mut PgConnection,
                original_audit: &Value,
            ) -> Value {
                let before = catalog(connection).await;
                let ledger = applied_ledger(connection).await;
                let business = complete_business(connection, false).await;
                assert!(
                    sqlx::query_scalar::<_, bool>(OWNER_VIEW_ABSENT)
                        .fetch_one(&mut *connection)
                        .await
                        .unwrap(),
                    "PREREQUISITE: diagnostic view/routine identities must be absent"
                );
                sqlx::raw_sql("SAVEPOINT column_owner_view")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                let mut observation = None;
                let result = AssertUnwindSafe(async {
                    // Transient diagnostic custody only. The parsed helper is
                    // selected by the existing owner prefix; it is never called.
                    // Its real pg_depend edge supplies the endpoint, not a union
                    // of fabricated addresses or a new selected business table.
                    sqlx::raw_sql(OWNER_VIEW_FIXTURE)
                        .execute(&mut *connection).await.unwrap();
                    let witness_raw: String = sqlx::query_scalar(OWNER_VIEW_WITNESS)
                        .fetch_one(&mut *connection).await
                        .expect("PREREQUISITE: real parsed-helper/view dependency witness must execute");
                    let witness: Value = serde_json::from_str(&witness_raw).unwrap();
                    for guard in ["parsed_invoker_admin", "no_PUBLIC_execute", "no_nonsuper_execute"] {
                        assert_eq!(witness[guard], true,
                            "PREREQUISITE: temporary helper must preserve invoker/privilege containment: {guard}");
                    }
                    assert_eq!(witness["native"], json!({"type":"view column",
                        "object_names":["org_column_v3_owner_probe","projection","값"],"object_args":[]}));
                    let dependencies = witness["dependencies"].as_array().unwrap();
                    assert_eq!(dependencies.len(), 1,
                        "PREREQUISITE: one genuine direct parsed-routine-to-positive-view-column dependency");
                    assert_eq!(dependencies[0]["deptype"], "n");
                    let actual = owner_audit(connection).await;
                    let selected = &actual["scope"]["selected_relations"];
                    assert_eq!(selected, &original_audit["scope"]["selected_relations"],
                        "diagnostic fixture must not expand selected business relations");
                    assert_eq!(selected.as_array().unwrap().len(), 76);
                    let ordinary: (i64, bool) = sqlx::query_as(r#"SELECT count(*),bool_and(c.relkind='r' AND NOT c.relispartition)
FROM pg_catalog.pg_class c WHERE c.oid IN(SELECT (x->>'oid')::oid FROM jsonb_array_elements($1::jsonb) x)"#)
                        .bind(sqlx::types::Json(selected)).fetch_one(&mut *connection).await.unwrap();
                    assert_eq!(ordinary, (76, true),
                        "PREREQUISITE: actual selected relations remain the 76 ordinary relations");
                    assert!(actual["scope"]["M"].as_array().unwrap().contains(&witness["routine_tuple"]),
                        "PREREQUISITE: actual owning M must select the real parsed helper");
                    for field in ["M", "I"] {
                        assert!(!actual["scope"][field].as_array().unwrap().contains(&witness["view_tuple"]),
                            "diagnostic view column must remain endpoint-only");
                    }
                    assert!(actual["scope"]["addresses"].as_array().unwrap().contains(&witness["view_tuple"]),
                        "PREREQUISITE: actual owner addresses must include the real view column");
                    assert!(actual["ordinary_edges"].as_array().unwrap().contains(&dependencies[0]),
                        "PREREQUISITE: actual owner must retain the exact raw/MVCC dependency");
                    let captured: Vec<_> = actual["inverse"].as_array().unwrap().iter()
                        .filter(|entry| entry["tuple"] == witness["view_tuple"]).collect();
                    assert_eq!(captured.len(), 1);
                    assert_eq!(captured[0]["native"], witness["native"],
                        "PREREQUISITE: captured native type/names/arguments remain unchanged");
                    assert_eq!(applied_ledger(connection).await, ledger);
                    assert_eq!(complete_business(connection, false).await, business);
                    observation = Some(actual);
                }).catch_unwind().await;
                sqlx::raw_sql(
                    "ROLLBACK TO SAVEPOINT column_owner_view; RELEASE SAVEPOINT column_owner_view",
                )
                .execute(&mut *connection)
                .await
                .unwrap();
                assert!(
                    sqlx::query_scalar::<_, bool>(OWNER_VIEW_ABSENT)
                        .fetch_one(&mut *connection)
                        .await
                        .unwrap(),
                    "mandatory diagnostic view/routine removal"
                );
                assert_eq!(
                    catalog(connection).await,
                    before,
                    "diagnostic view/routine rollback must restore complete catalog custody"
                );
                assert_eq!(applied_ledger(connection).await, ledger);
                assert_eq!(complete_business(connection, false).await, business);
                assert_eq!(
                    owner_audit(connection).await,
                    *original_audit,
                    "immediate original raw/MVCC/M/I/census restoration after diagnostic fixture"
                );
                if let Err(panic) = result {
                    std::panic::resume_unwind(panic);
                }
                observation.expect("PREREQUISITE: retain the genuine transient owner observation")
            }

            fn audit_machinery_controls(baseline: &Value) {
                owner_inverse_prerequisite(baseline).unwrap();
                for field in ["inverse", "objects"] {
                    let mut omitted = baseline.clone();
                    omitted[field].as_array_mut().unwrap().pop();
                    assert!(owner_inverse_prerequisite(&omitted).is_err());
                    let mut duplicate = baseline.clone();
                    let copy = duplicate[field][0].clone();
                    duplicate[field].as_array_mut().unwrap().push(copy);
                    assert!(owner_inverse_prerequisite(&duplicate).is_err());
                }
                for field in ["ordinary_edges", "shared_edges"] {
                    let mut changed = baseline.clone();
                    let rows = changed[field].as_array_mut().unwrap();
                    if rows.is_empty() {
                        rows.push(Value::Null);
                    } else {
                        rows.pop();
                    }
                    assert!(
                        owner_inverse_prerequisite(&changed).is_err(),
                        "edge bag multiplicity must remain complete"
                    );
                }
                let selected = baseline["inverse"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .position(column_entry)
                    .unwrap();
                for (field, value) in [
                    ("inverse_tuple", Value::Null),
                    ("COLUMN_witness", Value::Null),
                ] {
                    let mut bad = baseline.clone();
                    bad["inverse"][selected][field] = value;
                    assert!(owner_inverse_prerequisite(&bad).is_err());
                }
                // Evidence-only positive model. It leaves the actual
                // source observation untouched and supplies no product pass.
                // Every column in this independent model has its separately
                // checked raw native address as the expected portable identity.
                let mut oracle_positive = baseline.clone();
                let columns: Vec<_> = oracle_positive["inverse"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| column_entry(e))
                    .map(|(i, _)| i)
                    .collect();
                for &index in &columns {
                    oracle_positive["inverse"][index]["valid"] = json!(true);
                    oracle_positive["inverse"][index]["portable"] =
                        oracle_positive["inverse"][index]["native"].clone();
                }
                portable_columns_required(&oracle_positive)
                    .expect("EVIDENCE_ONLY: uncorrupted owner column predicate model must pass");
                inverse_positive(&oracle_positive)
                    .expect("EVIDENCE_ONLY: predecessor strict predicate model must pass");
                for &index in &columns {
                    for field in ["valid", "portable"] {
                        let mut bad = oracle_positive.clone();
                        bad["inverse"][index][field] = if field == "valid" {
                            json!(false)
                        } else {
                            Value::Null
                        };
                        assert!(
                            portable_columns_required(&bad).is_err(),
                            "successful inverse cannot confer portable validity"
                        );
                        assert!(
                            inverse_positive(&bad).is_err(),
                            "predecessor strict portability assertion must remain enforced"
                        );
                    }
                }
                portable_columns_required(&oracle_positive).unwrap();
                inverse_positive(&oracle_positive).unwrap();
            }

            #[sqlx::test(migrations = false)]
            async fn actor_complete_closed76_v3_finite_column_inverse_then_portable_validity(
                pool: PgPool,
            ) {
                let sources = candidate_source_pins();
                complete_fixture(&pool, CompleteFixtureFamily::Closed76, 0).await;
                let mut admin = direct(&pool).await;
                let expected_target = target(&mut admin).await;
                let administrator:bool=sqlx::query_scalar("SELECT current_user=session_user AND (SELECT count(*)=1 AND bool_and(rolsuper) FROM pg_catalog.pg_roles WHERE rolname=current_user)")
                    .fetch_one(&mut admin).await.unwrap();
                assert!(
                    administrator,
                    "PREREQUISITE: actual administrator, no role switching"
                );
                let engine: i32 =
                    sqlx::query_scalar("SELECT current_setting('server_version_num')::integer")
                        .fetch_one(&mut admin)
                        .await
                        .unwrap();
                assert_eq!(
                    engine, 180004,
                    "PREREQUISITE: pinned characterized PostgreSQL 18.4"
                );
                let mut tx = begin_protocol(&mut admin, &expected_target).await;
                let original_ledger = applied_ledger(tx.as_mut()).await;
                let original_rows = complete_business(tx.as_mut(), false).await;
                let original_catalog = catalog(tx.as_mut()).await;
                let mut baseline = None;
                let result=AssertUnwindSafe(async {
                    baseline=Some(owner_audit(tx.as_mut()).await);
                    let (source_family, family_inverses)=seven_kind_positive_and_controls(tx.as_mut()).await;
                    let actual=owner_view_observation(tx.as_mut(),baseline.as_ref().unwrap()).await;
                    owner_inverse_prerequisite(&actual).expect("PREREQUISITE: complete independent column/TOAST/native inverse positive before portable-validity RED");
                    assert!(actual["inverse"].as_array().unwrap().iter().any(|e|column_entry(e) && e["native"]["type"]=="view column"),"PREREQUISITE: real selected owner view-column witness");
                    audit_machinery_controls(&actual);
                    assert_eq!(applied_ledger(tx.as_mut()).await,original_ledger);
                    assert_eq!(complete_business(tx.as_mut(),false).await,original_rows);
                    assert_eq!(catalog(tx.as_mut()).await,original_catalog);
                    writeln!(&mut std::io::stderr().lock(),"ORG_COLUMN_V3_INVERSE_PREREQUISITE {}",json!({
                        "design_sha256":DESIGN,"sources":sources,"address_count":actual["address_count"],
                        "actual_seven_kind_inverse_positive":true,"actual_pinned_seven_kind_source_observation":&source_family,
                        "evidence_only_predicate_models":true,"source_family_predicate_corruptions":58,"malformed_input_controls":16,"SQL_array_shape_controls":2,"lookup_corruption_controls":4,"actual_catalog_controls":7,"actual_catalog_mutations":6,"actual_unsupported_index_inputs":1,
                        "all_eight_TOAST_guards_retained":true,"complete_inverse_raw_census":true,"source_flags_unchanged_by_helper":true,
                        "semantic_column_validity_accepted":false,"TOAST_map_accepted":false,"full_PSV_packet_accepted":false,
                        "database_owner_accepted":false,"MVP_accepted":false,"production_qualified":false})).unwrap();
                    portable_columns_required(&actual)
                        .and_then(|_| source_family_positive(&source_family, &family_inverses))
                        .expect("COLUMN_V3_PORTABLE_VALIDITY_MISSING_OR_INVALID: actual finite positive columns must retain exact native portable identities");
                }).catch_unwind().await;
                tx.rollback()
                    .await
                    .expect("mandatory candidate diagnostic rollback");
                assert!(matches_target(&mut admin, &expected_target).await);
                let mut fresh = sqlx::Connection::begin(&mut admin).await.unwrap();
                assert_eq!(applied_ledger(fresh.as_mut()).await, original_ledger);
                assert_eq!(
                    complete_business(fresh.as_mut(), false).await,
                    original_rows
                );
                assert_eq!(catalog(fresh.as_mut()).await, original_catalog);
                assert!(
                    sqlx::query_scalar::<_, bool>(OWNER_VIEW_ABSENT)
                        .fetch_one(fresh.as_mut())
                        .await
                        .unwrap(),
                    "fresh diagnostic view/routine removal"
                );
                if let Some(before) = baseline {
                    assert_eq!(
                        owner_audit(fresh.as_mut()).await,
                        before,
                        "fresh raw/MVCC/census readback after semantic refusal"
                    );
                }
                fresh.rollback().await.unwrap();
                admin.close().await.unwrap();
                if let Err(panic) = result {
                    std::panic::resume_unwind(panic);
                }
            }
        }
    }
}
