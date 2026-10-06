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
}
