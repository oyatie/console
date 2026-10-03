// Diagnostic metadata collection only. Both variants roll back. No serving
// profile/finalizer/workflow acceptance or Organization activation is implied.
mod native_org_unit_closed_phase_capture {
    use super::*;
    use std::collections::BTreeSet;

    const CLOSED_OWNER: &str =
        include_str!("../../../../ops/postgres-native-org-unit-closed-perimeter-v1-owner.sql");
    const WIDER_CAPTURE: &str = include_str!(
        "../../../../ops/postgres-capture-native-org-unit-closed-perimeter-v1-custody.sql"
    );
    const CLASSIFIER_INSTALLED: [&str; 2] = [
        "de87fafa527398d64a1930288ef1a0a56d017db6b56bc877f8b716c714afd90a",
        "8011bd8141ec1a0497319773d73bc1df97a924f99e8aa84b8ef7c9cb4c821b37",
    ];

    // Independent additive effective-rights oracle. The historical complete
    // 73-relation positive predicate is executed unchanged through capture().
    const DENIED_RIGHTS: &str = r#"WITH startup AS (
      SELECT oid,rolname FROM pg_catalog.pg_roles WHERE rolname='console_auth_startup'
    ), required(name) AS (
      VALUES ('org_units'::text),('org_unit_revisions'),('org_unit_source_bindings')
    ), relations AS (
      SELECT required.name,c.oid,c.relkind,c.relispartition
      FROM required LEFT JOIN pg_catalog.pg_namespace n ON n.nspname='public'
      LEFT JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
    ), columns AS (
      SELECT r.name,r.oid,a.attnum,a.attname,a.atttypid,a.atttypmod,a.attnotnull
      FROM relations r JOIN pg_catalog.pg_attribute a ON a.attrelid=r.oid
      WHERE a.attnum>0 AND NOT a.attisdropped
    ), table_privileges(privilege) AS (
      VALUES ('SELECT'::text),('INSERT'),('UPDATE'),('DELETE'),
             ('TRUNCATE'),('REFERENCES'),('TRIGGER'),('MAINTAIN')
    ), column_privileges(privilege) AS (
      VALUES ('SELECT'::text),('INSERT'),('UPDATE'),('REFERENCES')
    ), table_checks AS (
      SELECT r.name,r.oid,p.privilege,
       pg_catalog.has_table_privilege(s.oid,r.oid,p.privilege) AS allowed
      FROM startup s CROSS JOIN relations r CROSS JOIN table_privileges p
    ), column_checks AS (
      SELECT c.name,c.oid,c.attnum,c.attname,p.privilege,
       pg_catalog.has_column_privilege(s.oid,c.oid,c.attnum,p.privilege) AS allowed
      FROM startup s CROSS JOIN columns c CROSS JOIN column_privileges p
    ) SELECT pg_catalog.jsonb_build_object(
      'role',(SELECT COALESCE(jsonb_agg(jsonb_build_object('oid',oid::text,'name',rolname)),'[]'::jsonb) FROM startup),
      'relations',(SELECT jsonb_agg(jsonb_build_object('name',name,'oid',oid::text,'kind',relkind,'partition',relispartition) ORDER BY name COLLATE "C") FROM relations),
      'columns',(SELECT COALESCE(jsonb_agg(jsonb_build_object('relation',name,'relation_oid',oid::text,'attnum',attnum,'name',attname,'type_oid',atttypid::text,'type_modifier',atttypmod,'not_null',attnotnull) ORDER BY name COLLATE "C",attnum),'[]'::jsonb) FROM columns),
      'table_checks',(SELECT COALESCE(jsonb_agg(jsonb_build_object('relation',name,'relation_oid',oid::text,'privilege',privilege,'allowed',allowed) ORDER BY name COLLATE "C",privilege COLLATE "C"),'[]'::jsonb) FROM table_checks),
      'column_checks',(SELECT COALESCE(jsonb_agg(jsonb_build_object('relation',name,'relation_oid',oid::text,'attnum',attnum,'name',attname,'privilege',privilege,'allowed',allowed) ORDER BY name COLLATE "C",attnum,privilege COLLATE "C"),'[]'::jsonb) FROM column_checks)
    )"#;

    async fn added_three_denied_rights(connection: &mut PgConnection) -> Value {
        let result: Value = sqlx::query_scalar(DENIED_RIGHTS)
            .fetch_one(connection)
            .await
            .unwrap();
        let roles = result["role"].as_array().unwrap();
        assert_eq!(roles.len(), 1, "one actual startup role is required");
        assert_eq!(roles[0]["name"], "console_auth_startup");
        assert!(roles[0]["oid"].as_str().unwrap().parse::<u32>().unwrap() > 0);
        let relations = result["relations"].as_array().unwrap();
        let names = [
            "org_unit_revisions",
            "org_unit_source_bindings",
            "org_units",
        ];
        assert_eq!(relations.len(), names.len());
        let mut relation_oids = BTreeMap::new();
        for (relation, expected) in relations.iter().zip(names) {
            assert_eq!(relation["name"], expected);
            assert_eq!(relation["kind"], "r", "ordinary public relation required");
            assert_eq!(relation["partition"], false);
            let oid = relation["oid"].as_str().unwrap();
            assert!(oid.parse::<u32>().unwrap() > 0);
            assert!(relation_oids.insert(expected, oid).is_none());
        }
        assert_eq!(relation_oids.values().collect::<BTreeSet<_>>().len(), 3);
        let expected_columns: Vec<(&str, &str)> = [
            (
                "org_unit_revisions",
                &[
                    "org_id",
                    "id",
                    "org_unit_id",
                    "version",
                    "command_id",
                    "actor_id",
                    "payload_digest",
                    "attributes",
                    "receipt",
                    "created_at",
                ][..],
            ),
            (
                "org_unit_source_bindings",
                &[
                    "org_id",
                    "source_kind",
                    "source_id",
                    "org_unit_id",
                    "actor_id",
                    "payload_digest",
                    "created_at",
                ][..],
            ),
            ("org_units", &["org_id", "id", "created_at"][..]),
        ]
        .into_iter()
        .flat_map(|(relation, columns)| columns.iter().map(move |column| (relation, *column)))
        .collect();
        let columns = result["columns"].as_array().unwrap();
        assert_eq!(columns.len(), expected_columns.len());
        let mut column_keys = BTreeSet::new();
        for (column, (relation, name)) in columns.iter().zip(expected_columns) {
            assert_eq!(column["relation"], relation);
            assert_eq!(column["name"], name);
            assert_eq!(
                column["relation_oid"].as_str(),
                relation_oids.get(relation).copied()
            );
            let attnum = column["attnum"].as_i64().unwrap();
            assert!(attnum > 0);
            assert!(column_keys.insert((relation.to_owned(), attnum)));
        }
        let table_privileges = [
            "SELECT",
            "INSERT",
            "UPDATE",
            "DELETE",
            "TRUNCATE",
            "REFERENCES",
            "TRIGGER",
            "MAINTAIN",
        ];
        let expected_tables: BTreeSet<_> = names
            .into_iter()
            .flat_map(|relation| {
                table_privileges
                    .into_iter()
                    .map(move |privilege| (relation.to_owned(), privilege.to_owned()))
            })
            .collect();
        let table_checks = result["table_checks"].as_array().unwrap();
        assert_eq!(table_checks.len(), 24);
        let mut checked_tables = BTreeSet::new();
        for check in table_checks {
            let relation = check["relation"].as_str().unwrap();
            assert_eq!(
                check["relation_oid"].as_str(),
                relation_oids.get(relation).copied()
            );
            assert_eq!(
                check["allowed"], false,
                "NULL and effective grants both fail"
            );
            assert!(checked_tables.insert((
                relation.to_owned(),
                check["privilege"].as_str().unwrap().to_owned()
            )));
        }
        assert_eq!(checked_tables, expected_tables);
        let column_privileges = ["SELECT", "INSERT", "UPDATE", "REFERENCES"];
        let expected_checks: BTreeSet<_> = column_keys
            .iter()
            .flat_map(|(relation, attnum)| {
                column_privileges
                    .into_iter()
                    .map(move |privilege| (relation.clone(), *attnum, privilege.to_owned()))
            })
            .collect();
        let column_checks = result["column_checks"].as_array().unwrap();
        assert_eq!(column_checks.len(), 80);
        let mut checked_columns = BTreeSet::new();
        for check in column_checks {
            let relation = check["relation"].as_str().unwrap();
            assert_eq!(
                check["relation_oid"].as_str(),
                relation_oids.get(relation).copied()
            );
            assert_eq!(
                check["allowed"], false,
                "NULL and effective grants both fail"
            );
            assert!(checked_columns.insert((
                relation.to_owned(),
                check["attnum"].as_i64().unwrap(),
                check["privilege"].as_str().unwrap().to_owned()
            )));
        }
        assert_eq!(checked_columns, expected_checks);
        result
    }

    async fn raw_wider_capture(connection: &mut PgConnection) -> Value {
        sqlx::raw_sql(CLASSIFIER_SESSION)
            .execute(&mut *connection)
            .await
            .unwrap();
        let body = WIDER_CAPTURE
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
            "exact PostgreSQL UTF-8 text digest differs"
        );
        assert_eq!(
            rights,
            Some(false),
            "frozen historical counters with wider76 roster must remain raw FALSE"
        );
        let snapshot: Value = serde_json::from_str(&text).unwrap();
        assert!(snapshot.is_object());
        json!({"snapshot_text":text,"snapshot":snapshot,"snapshot_sha256":sha256,
            "raw_native_directory_startup_rights_valid":rights,"accepted_profile":false})
    }

    #[sqlx::test(migrations = false)]
    async fn collects_closed_plain_observer_raw_metadata_and_complete_effective_denials(
        pool: PgPool,
    ) {
        let prior_pins = source_pins();
        assert_eq!(
            digest(CLOSED_OWNER),
            "aba221ad2cfa03390eca638ccd290901877fef615450b17cf14a96304e9f2609"
        );
        assert_eq!(
            digest(WIDER_CAPTURE),
            "6be2e3d095d59bbdb9e1b932dac8da48bde261601455cdcd166c6f2a649e6010"
        );
        let mut initial = pool.begin().await.unwrap();
        marked(initial.as_mut(), true).await;
        initial.rollback().await.unwrap();
        prepare_http_database_staging(&pool).await;
        let mut initial = pool.begin().await.unwrap();
        marked(initial.as_mut(), false).await;
        let identity: Value = sqlx::query_scalar("SELECT jsonb_build_object('database',current_database(),'database_oid',(SELECT oid FROM pg_catalog.pg_database WHERE datname=current_database()),'session_user',session_user,'current_user',current_user,'fixture_marker',current_setting('console.sqlx_test_bootstrap',true),'system_identifier',(pg_catalog.pg_control_system()).system_identifier::text,'server_version_num',current_setting('server_version_num'),'server_encoding',current_setting('server_encoding'),'collation',(SELECT datcollate FROM pg_catalog.pg_database WHERE datname=current_database()),'ctype',(SELECT datctype FROM pg_catalog.pg_database WHERE datname=current_database()),'tls',COALESCE((SELECT ssl FROM pg_catalog.pg_stat_ssl WHERE pid=pg_backend_pid()),false),'fsync',current_setting('fsync'),'synchronous_commit',current_setting('synchronous_commit'),'full_page_writes',current_setting('full_page_writes'))").fetch_one(initial.as_mut()).await.unwrap();
        let ledger = applied_ledger(initial.as_mut()).await;
        let baseline_rows = rows(initial.as_mut()).await;
        let baseline_catalog = catalog(initial.as_mut()).await;
        assert!(namespace(initial.as_mut()).await.is_empty());
        let observer_absent: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_durability_observer') AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p WHERE p.proname='console_durability_observation_v1')").fetch_one(initial.as_mut()).await.unwrap();
        assert!(
            observer_absent,
            "dedicated cluster must start without observer"
        );
        initial.rollback().await.unwrap();
        let mut packets = Vec::new();
        for variant in 0..2 {
            let mut tx = pool.begin().await.unwrap();
            let outcome = AssertUnwindSafe(async {
                sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='120s'; SET LOCAL search_path=pg_catalog,pg_temp").execute(tx.as_mut()).await.unwrap();
                marked(tx.as_mut(), false).await;
                let _: String = sqlx::query_scalar("SELECT oid::text FROM pg_catalog.pg_authid WHERE rolname='console_account_owner' FOR UPDATE").fetch_one(tx.as_mut()).await.unwrap();
                if variant == 1 { execute(&mut tx, OBSERVER).await; }
                for source in [ACCOUNT, CREDENTIALS, COMPANY, POLICY_INSTALLER, POLICY_V2, ROW_LOCK] { execute(&mut tx, source).await; }
                let predecessor = capture(tx.as_mut(), PREDECESSOR_CAPTURE).await;
                assert_eq!(predecessor.sha256, CORRECTED[variant]);
                assert_eq!(row_lock_state(tx.as_mut()).await, "native_people_directory.finalized");
                let pre_provenance_catalog = catalog(tx.as_mut()).await;
                marked(tx.as_mut(), false).await;
                execute(&mut tx, OWNER).await;
                let original73_before = capture(tx.as_mut(), CAPTURE).await;
                assert_eq!(original73_before.sha256, CLASSIFIER_INSTALLED[variant], "exact observed classifier-installed phase required");
                assert_eq!(namespace(tx.as_mut()).await, exact_namespace());
                let before_catalog = catalog(tx.as_mut()).await;
                source_catalog_delta(&pre_provenance_catalog.1, &before_catalog.1);
                let before_rows = rows(tx.as_mut()).await;
                let raw76_before = raw_wider_capture(tx.as_mut()).await;
                let denied_before = added_three_denied_rights(tx.as_mut()).await;
                marked(tx.as_mut(), false).await;
                execute(&mut tx, CLOSED_OWNER).await;
                let raw76_after = raw_wider_capture(tx.as_mut()).await;
                assert_ne!(raw76_before["snapshot_text"], raw76_after["snapshot_text"], "wider capture omitted installed closed metadata");
                assert_eq!(raw_wider_capture(tx.as_mut()).await, raw76_after, "raw text/hash/flag must be deterministic");
                let original73_after = capture(tx.as_mut(), CAPTURE).await;
                let denied_after = added_three_denied_rights(tx.as_mut()).await;
                assert_eq!(denied_before, denied_after, "closed source changed effective denied rights or relation columns");
                let after_catalog = catalog(tx.as_mut()).await;
                assert_ne!(before_catalog, after_catalog);
                assert_eq!(catalog(tx.as_mut()).await, after_catalog, "full raw catalog must be deterministic");
                assert_eq!(rows(tx.as_mut()).await, before_rows, "closed DDL changed complete durable rows");
                assert_eq!(applied_ledger(tx.as_mut()).await, ledger);
                assert_eq!(row_lock_state(tx.as_mut()).await, "native_people_directory.profile_mismatch");
                json!({"schema":"console.org_unit_closed_phase_diagnostic.v1","variant":if variant==0 {"plain"} else {"observer"},
                    "prior_sources":prior_pins,"closed_owner_sha256":digest(CLOSED_OWNER),"wider_capture_sha256":digest(WIDER_CAPTURE),
                    "database_identity":identity,"applied_ledger":ledger,"predecessor":predecessor.record(),
                    "original73_before":original73_before.record(),"original73_after":original73_after.record(),
                    "wider76_before":raw76_before,"wider76_after":raw76_after,"added_three_denied_rights_before":denied_before,"added_three_denied_rights_after":denied_after,
                    "raw_catalog_before_text":before_catalog.0,"raw_catalog_after_text":after_catalog.0,
                    "raw_catalog_before_sha256":digest(&before_catalog.0),"raw_catalog_after_sha256":digest(&after_catalog.0),
                    "business_rows":row_summary(&before_rows),"business_history_unchanged":true,
                    "observed_not_accepted":true,"closed_corruption_controls_qualified":false,"measured_finalizer_qualified":false,
                    "serving_profile_accepted":false,"native_guard_accepted":false,"organization_workflow_accepted":false,"production_qualified":false})
            }).catch_unwind().await;
            tx.rollback()
                .await
                .expect("all finalizer/source/observer DDL must roll back");
            let mut restored = pool.begin().await.unwrap();
            assert_eq!(
                rows(restored.as_mut()).await,
                baseline_rows,
                "variant rollback changed complete durable rows"
            );
            assert_eq!(
                catalog(restored.as_mut()).await,
                baseline_catalog,
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
        let mut output = std::io::stderr().lock();
        for packet in packets {
            writeln!(&mut output, "ORG_UNIT_CLOSED_PHASE_DIAGNOSTIC {packet}")
                .expect("diagnostic record write failed");
        }
        writeln!(&mut output, "ORG_UNIT_CLOSED_PHASE_DIAGNOSTIC_COMPLETE variants=2 original73_rights=true wider76_raw_rights=false added3_table_checks=24 added3_column_checks=80 rollback=verified acceptance=not_claimed").expect("completion write failed");
        output.flush().expect("diagnostic evidence flush failed");
    }
}

// Actual isolated metadata protocol tests, separate from the frozen diagnostic.
// Production migrations contain historical rows. No UI/business-empty evidence,
// serving acceptance, native OrgUnit activation or production execution is claimed.
mod native_org_unit_closed_finalizer_tests {
    use super::*;
    use sqlx::Connection as _;
    use std::collections::BTreeSet;
    use std::time::{Duration, Instant};

    const FINALIZER: &str =
        include_str!("../../../../ops/postgres-finalize-native-org-unit-closed-perimeter-v1.sql");
    const CLOSED_STATE: &str = include_str!(
        "../../../../ops/postgres-native-org-unit-closed-perimeter-v1-custody-state.sql"
    );
    const WIDER_CAPTURE: &str = include_str!(
        "../../../../ops/postgres-capture-native-org-unit-closed-perimeter-v1-custody.sql"
    );
    const PREDECESSOR73: [&str; 2] = [
        "de87fafa527398d64a1930288ef1a0a56d017db6b56bc877f8b716c714afd90a",
        "8011bd8141ec1a0497319773d73bc1df97a924f99e8aa84b8ef7c9cb4c821b37",
    ];
    const PREDECESSOR76: [&str; 2] = [
        "efc7f14dee39011c6ed5e68b97bd8374543b1307afe3d936b828ef5a52af51e7",
        "a47330ee0efb705f72744a93263e65d400fbe525d70b829d2c7eedf5bfa75bdf",
    ];
    const CLOSED73: [&str; 2] = [
        "fe0f65aebe362a969202e13d79c21d4e49f75834fd7b254a16a85a270e4e3c98",
        "be18fc18f7bc6df0b5371ff01140d6a06c0a6a0438e7323d3eddaa7b070851e1",
    ];
    const CLOSED76: [&str; 2] = [
        "be4e86175dcd561150beb68db36de84fd3f224ca39d3128b1dcc958fa319a46d",
        "7bf64f46c07608b2fba7a39be765a80e45db727b424caa55342103dc180dfcc9",
    ];
    const ENTRY_BOUNDS: &str = "SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL jit=off; \
        SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='60s'; \
        SET LOCAL idle_in_transaction_session_timeout='30s'; SET LOCAL transaction_timeout='120s'";
    // Independent literal roster from the reviewed fixture contract, never
    // inferred from a successful finalizer result or fabricated lock records.
    const RELATIONS76: [&str; 76] = [
        "account_context_candidates",
        "account_security",
        "account_security_events",
        "account_terms_acceptances",
        "account_terms_head",
        "account_terms_release_receipts",
        "accounts",
        "audit_events",
        "auth_bootstrap_credentials",
        "auth_device_login_handoffs",
        "auth_refresh_token_families",
        "auth_refresh_tokens",
        "auth_webauthn_ceremonies",
        "auth_webauthn_ceremony_bindings",
        "auth_webauthn_credentials",
        "cedar_policy_catalog_entries",
        "company_actors",
        "company_authority_heads",
        "company_enrollment_effect_bindings",
        "company_enrollment_receipts",
        "company_enrollment_request_events",
        "company_enrollment_requests",
        "deployment_operator_head",
        "deployment_operator_receipts",
        "employee_employment_profiles",
        "employee_lifecycle_events",
        "employee_person_bindings",
        "employees",
        "employment_revisions",
        "employment_source_bindings",
        "group_authority_heads",
        "group_membership_revisions",
        "group_memberships",
        "group_role_grants",
        "groups",
        "leave_balance_import_receipts",
        "native_company_action_refs",
        "native_company_catalog_installs",
        "native_company_object_refs",
        "native_company_policy_inputs_v1",
        "native_company_policy_receipts_v1",
        "native_company_property_refs",
        "native_people_inputs_v1",
        "native_people_terminals_v1",
        "ont_action_command_receipts",
        "ont_action_types",
        "ont_analytics",
        "ont_builtin_catalog_allowlist",
        "ont_builtin_catalog_installs",
        "ont_link_types",
        "ont_object_policies",
        "ont_object_type_key_revisions",
        "ont_object_types",
        "ont_property_defs",
        "org_unit_revisions",
        "org_unit_source_bindings",
        "org_units",
        "organizations",
        "person_revisions",
        "persons",
        "platform_force_removal_effect_bindings",
        "platform_force_removal_receipts",
        "platform_legacy_catalog_effect_bindings",
        "platform_legacy_membership_effect_bindings",
        "platform_legacy_topology_effect_bindings",
        "platform_legacy_topology_receipts",
        "platform_legacy_user_birth_witnesses",
        "policy_assignment_revisions",
        "policy_capability_clause_fields",
        "policy_capability_clauses",
        "policy_role_conditions",
        "policy_role_permissions",
        "policy_role_revisions",
        "policy_roles",
        "user_role_assignments",
        "users",
    ];

    // Independent additive effective-rights oracle. The historical complete
    // 73-relation positive predicate is executed unchanged through capture().
    const DENIED_RIGHTS: &str = r#"WITH startup AS (
      SELECT oid,rolname FROM pg_catalog.pg_roles WHERE rolname='console_auth_startup'
    ), required(name) AS (
      VALUES ('org_units'::text),('org_unit_revisions'),('org_unit_source_bindings')
    ), relations AS (
      SELECT required.name,c.oid,c.relkind,c.relispartition
      FROM required LEFT JOIN pg_catalog.pg_namespace n ON n.nspname='public'
      LEFT JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
    ), columns AS (
      SELECT r.name,r.oid,a.attnum,a.attname,a.atttypid,a.atttypmod,a.attnotnull
      FROM relations r JOIN pg_catalog.pg_attribute a ON a.attrelid=r.oid
      WHERE a.attnum>0 AND NOT a.attisdropped
    ), table_privileges(privilege) AS (
      VALUES ('SELECT'::text),('INSERT'),('UPDATE'),('DELETE'),
             ('TRUNCATE'),('REFERENCES'),('TRIGGER'),('MAINTAIN')
    ), column_privileges(privilege) AS (
      VALUES ('SELECT'::text),('INSERT'),('UPDATE'),('REFERENCES')
    ), table_checks AS (
      SELECT r.name,r.oid,p.privilege,
       pg_catalog.has_table_privilege(s.oid,r.oid,p.privilege) AS allowed
      FROM startup s CROSS JOIN relations r CROSS JOIN table_privileges p
    ), column_checks AS (
      SELECT c.name,c.oid,c.attnum,c.attname,p.privilege,
       pg_catalog.has_column_privilege(s.oid,c.oid,c.attnum,p.privilege) AS allowed
      FROM startup s CROSS JOIN columns c CROSS JOIN column_privileges p
    ) SELECT pg_catalog.jsonb_build_object(
      'role',(SELECT COALESCE(jsonb_agg(jsonb_build_object('oid',oid::text,'name',rolname)),'[]'::jsonb) FROM startup),
      'relations',(SELECT jsonb_agg(jsonb_build_object('name',name,'oid',oid::text,'kind',relkind,'partition',relispartition) ORDER BY name COLLATE "C") FROM relations),
      'columns',(SELECT COALESCE(jsonb_agg(jsonb_build_object('relation',name,'relation_oid',oid::text,'attnum',attnum,'name',attname,'type_oid',atttypid::text,'type_modifier',atttypmod,'not_null',attnotnull) ORDER BY name COLLATE "C",attnum),'[]'::jsonb) FROM columns),
      'table_checks',(SELECT COALESCE(jsonb_agg(jsonb_build_object('relation',name,'relation_oid',oid::text,'privilege',privilege,'allowed',allowed) ORDER BY name COLLATE "C",privilege COLLATE "C"),'[]'::jsonb) FROM table_checks),
      'column_checks',(SELECT COALESCE(jsonb_agg(jsonb_build_object('relation',name,'relation_oid',oid::text,'attnum',attnum,'name',attname,'privilege',privilege,'allowed',allowed) ORDER BY name COLLATE "C",attnum,privilege COLLATE "C"),'[]'::jsonb) FROM column_checks)
    )"#;

    async fn added_three_denied_rights(connection: &mut PgConnection) -> Value {
        let result: Value = sqlx::query_scalar(DENIED_RIGHTS)
            .fetch_one(connection)
            .await
            .unwrap();
        let roles = result["role"].as_array().unwrap();
        assert_eq!(roles.len(), 1, "one actual startup role is required");
        assert_eq!(roles[0]["name"], "console_auth_startup");
        assert!(roles[0]["oid"].as_str().unwrap().parse::<u32>().unwrap() > 0);
        let relations = result["relations"].as_array().unwrap();
        let names = [
            "org_unit_revisions",
            "org_unit_source_bindings",
            "org_units",
        ];
        assert_eq!(relations.len(), names.len());
        let mut relation_oids = BTreeMap::new();
        for (relation, expected) in relations.iter().zip(names) {
            assert_eq!(relation["name"], expected);
            assert_eq!(relation["kind"], "r", "ordinary public relation required");
            assert_eq!(relation["partition"], false);
            let oid = relation["oid"].as_str().unwrap();
            assert!(oid.parse::<u32>().unwrap() > 0);
            assert!(relation_oids.insert(expected, oid).is_none());
        }
        assert_eq!(relation_oids.values().collect::<BTreeSet<_>>().len(), 3);
        let expected_columns: Vec<(&str, &str)> = [
            (
                "org_unit_revisions",
                &[
                    "org_id",
                    "id",
                    "org_unit_id",
                    "version",
                    "command_id",
                    "actor_id",
                    "payload_digest",
                    "attributes",
                    "receipt",
                    "created_at",
                ][..],
            ),
            (
                "org_unit_source_bindings",
                &[
                    "org_id",
                    "source_kind",
                    "source_id",
                    "org_unit_id",
                    "actor_id",
                    "payload_digest",
                    "created_at",
                ][..],
            ),
            ("org_units", &["org_id", "id", "created_at"][..]),
        ]
        .into_iter()
        .flat_map(|(relation, columns)| columns.iter().map(move |column| (relation, *column)))
        .collect();
        let columns = result["columns"].as_array().unwrap();
        assert_eq!(columns.len(), expected_columns.len());
        let mut column_keys = BTreeSet::new();
        for (column, (relation, name)) in columns.iter().zip(expected_columns) {
            assert_eq!(column["relation"], relation);
            assert_eq!(column["name"], name);
            assert_eq!(
                column["relation_oid"].as_str(),
                relation_oids.get(relation).copied()
            );
            let attnum = column["attnum"].as_i64().unwrap();
            assert!(attnum > 0);
            assert!(column_keys.insert((relation.to_owned(), attnum)));
        }
        let table_privileges = [
            "SELECT",
            "INSERT",
            "UPDATE",
            "DELETE",
            "TRUNCATE",
            "REFERENCES",
            "TRIGGER",
            "MAINTAIN",
        ];
        let expected_tables: BTreeSet<_> = names
            .into_iter()
            .flat_map(|relation| {
                table_privileges
                    .into_iter()
                    .map(move |privilege| (relation.to_owned(), privilege.to_owned()))
            })
            .collect();
        let table_checks = result["table_checks"].as_array().unwrap();
        assert_eq!(table_checks.len(), 24);
        let mut checked_tables = BTreeSet::new();
        for check in table_checks {
            let relation = check["relation"].as_str().unwrap();
            assert_eq!(
                check["relation_oid"].as_str(),
                relation_oids.get(relation).copied()
            );
            assert_eq!(
                check["allowed"], false,
                "NULL and effective grants both fail"
            );
            assert!(checked_tables.insert((
                relation.to_owned(),
                check["privilege"].as_str().unwrap().to_owned()
            )));
        }
        assert_eq!(checked_tables, expected_tables);
        let column_privileges = ["SELECT", "INSERT", "UPDATE", "REFERENCES"];
        let expected_checks: BTreeSet<_> = column_keys
            .iter()
            .flat_map(|(relation, attnum)| {
                column_privileges
                    .into_iter()
                    .map(move |privilege| (relation.clone(), *attnum, privilege.to_owned()))
            })
            .collect();
        let column_checks = result["column_checks"].as_array().unwrap();
        assert_eq!(column_checks.len(), 80);
        let mut checked_columns = BTreeSet::new();
        for check in column_checks {
            let relation = check["relation"].as_str().unwrap();
            assert_eq!(
                check["relation_oid"].as_str(),
                relation_oids.get(relation).copied()
            );
            assert_eq!(
                check["allowed"], false,
                "NULL and effective grants both fail"
            );
            assert!(checked_columns.insert((
                relation.to_owned(),
                check["attnum"].as_i64().unwrap(),
                check["privilege"].as_str().unwrap().to_owned()
            )));
        }
        assert_eq!(checked_columns, expected_checks);
        result
    }

    async fn raw_wider_capture(connection: &mut PgConnection) -> Value {
        sqlx::raw_sql(CLASSIFIER_SESSION)
            .execute(&mut *connection)
            .await
            .unwrap();
        let body = WIDER_CAPTURE
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
            "exact PostgreSQL UTF-8 text digest differs"
        );
        assert_eq!(
            rights,
            Some(false),
            "frozen historical counters with wider76 roster must remain raw FALSE"
        );
        let snapshot: Value = serde_json::from_str(&text).unwrap();
        assert!(snapshot.is_object());
        json!({"snapshot_text":text,"snapshot":snapshot,"snapshot_sha256":sha256,
            "raw_native_directory_startup_rights_valid":rights,"accepted_profile":false})
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Target {
        database: String,
        database_oid: i64,
        system_identifier: String,
    }

    async fn target(connection: &mut PgConnection) -> Target {
        let (database, database_oid, system_identifier): (String, i64, String) = sqlx::query_as(
            "SELECT current_database()::text, \
                (SELECT oid::bigint FROM pg_catalog.pg_database WHERE datname=current_database()), \
                (pg_catalog.pg_control_system()).system_identifier::text",
        )
        .fetch_one(connection)
        .await
        .unwrap();
        assert!(database_oid > 0);
        assert!(system_identifier.parse::<u64>().unwrap() > 0);
        Target {
            database,
            database_oid,
            system_identifier,
        }
    }

    async fn matches_target(connection: &mut PgConnection, expected: &Target) -> bool {
        target(connection).await == *expected
    }

    async fn direct(pool: &PgPool) -> PgConnection {
        let mut connection = PgConnection::connect_with(&pool.connect_options())
            .await
            .unwrap();
        marked(&mut connection, false).await;
        connection
    }

    fn pins() -> Value {
        let mut result = source_pins();
        for (name, source, expected) in [
            (
                "closed_finalizer",
                FINALIZER,
                "2596a0f43b2e81d87302fbbe92354922558a61fce62bbba06c75731c94ca8c7b",
            ),
            (
                "closed_state",
                CLOSED_STATE,
                "138b3179b41b15bfbbaae4d49b93888ff92a662547a14725ccf52a36dc3d28ff",
            ),
            (
                "wider76_capture",
                WIDER_CAPTURE,
                "6be2e3d095d59bbdb9e1b932dac8da48bde261601455cdcd166c6f2a649e6010",
            ),
        ] {
            assert_eq!(
                digest(source),
                expected,
                "unreviewed native prerequisite: {name}"
            );
            result[name] = json!(expected);
        }
        assert_eq!(
            RELATIONS76.iter().copied().collect::<BTreeSet<_>>().len(),
            76
        );
        assert!(RELATIONS76.windows(2).all(|pair| pair[0] < pair[1]));
        result
    }

    async fn bounds(connection: &mut PgConnection) {
        // A separate statement BEFORE finalizer DO, including after catalog()
        // and capture() helpers which deliberately change local settings.
        sqlx::raw_sql(ENTRY_BOUNDS)
            .execute(&mut *connection)
            .await
            .unwrap();
        let actual: Vec<(String, i64)> = sqlx::query_as(
            "SELECT name::text,setting::bigint FROM pg_catalog.pg_settings \
             WHERE name IN ('lock_timeout','statement_timeout', \
               'idle_in_transaction_session_timeout','transaction_timeout') ORDER BY name COLLATE \"C\"")
            .fetch_all(&mut *connection).await.unwrap();
        assert_eq!(
            actual,
            vec![
                ("idle_in_transaction_session_timeout".into(), 30_000),
                ("lock_timeout".into(), 1_000),
                ("statement_timeout".into(), 60_000),
                ("transaction_timeout".into(), 120_000),
            ]
        );
        let settings: (String, String, String) = sqlx::query_as(
            "SELECT current_setting('transaction_isolation'), \
             current_setting('search_path'),current_setting('jit')",
        )
        .fetch_one(connection)
        .await
        .unwrap();
        assert_eq!(
            settings,
            (
                "read committed".into(),
                "pg_catalog, pg_temp".into(),
                "off".into()
            )
        );
    }

    async fn no_prior_user_locks(connection: &mut PgConnection) {
        let held: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_catalog.pg_locks l \
             JOIN pg_catalog.pg_class c ON c.oid=l.relation \
             JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace \
             WHERE l.pid=pg_backend_pid() AND l.locktype='relation' \
               AND n.nspname NOT IN ('pg_catalog','information_schema') \
               AND NOT starts_with(n.nspname,'pg_toast')",
        )
        .fetch_one(connection)
        .await
        .unwrap();
        assert_eq!(
            held, 0,
            "fresh direct entry must not inherit setup relation locks"
        );
    }

    async fn begin_protocol<'a>(
        connection: &'a mut PgConnection,
        expected: &Target,
    ) -> Transaction<'a, Postgres> {
        marked(connection, false).await;
        assert!(
            matches_target(connection, expected).await,
            "caller target mismatch before BEGIN"
        );
        no_prior_user_locks(connection).await;
        let mut tx = sqlx::Connection::begin(&mut *connection).await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
            .execute(tx.as_mut())
            .await
            .unwrap();
        bounds(tx.as_mut()).await;
        assert!(
            matches_target(tx.as_mut(), expected).await,
            "caller target changed after BEGIN"
        );
        let ledger_shape: bool = sqlx::query_scalar(
            "SELECT (count(*)=1 AND bool_and(c.oid>0 AND c.relkind='r' \
               AND NOT c.relispartition AND r.rolname='console_app') IS TRUE) IS TRUE \
             FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace \
             JOIN pg_catalog.pg_roles r ON r.oid=c.relowner \
             WHERE n.nspname='public' AND c.relname='_sqlx_migrations'",
        )
        .fetch_one(tx.as_mut())
        .await
        .unwrap();
        assert!(
            ledger_shape,
            "exact actual ordinary console_app ledger required"
        );
        sqlx::raw_sql("LOCK TABLE ONLY public._sqlx_migrations IN SHARE MODE")
            .execute(tx.as_mut())
            .await
            .unwrap();
        applied_ledger(tx.as_mut()).await;
        let role_oids: Vec<String> = sqlx::query_scalar(
            "SELECT oid::text FROM pg_catalog.pg_authid WHERE rolname='console_account_owner' FOR UPDATE")
            .fetch_all(tx.as_mut()).await.unwrap();
        assert_eq!(
            role_oids.len(),
            1,
            "one actual account-owner role row must be retained"
        );
        assert!(role_oids[0].parse::<u32>().unwrap() > 0);
        tx
    }

    async fn state(connection: &mut PgConnection) -> String {
        sqlx::raw_sql(CLASSIFIER_SESSION)
            .execute(&mut *connection)
            .await
            .unwrap();
        sqlx::query_scalar(CLOSED_STATE)
            .fetch_one(connection)
            .await
            .unwrap()
    }

    async fn relation_and_schema_closure(connection: &mut PgConnection, closed: bool) -> Value {
        let reserved_relations: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT n.nspname::text,c.relname::text,c.relkind::text \
             FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace \
             WHERE c.relname='native_org_unit' OR starts_with(c.relname,'native_org_unit_') \
             ORDER BY n.nspname COLLATE \"C\",c.relname COLLATE \"C\",c.relkind",
        )
        .fetch_all(&mut *connection)
        .await
        .unwrap();
        let reserved_schemas: Vec<String> = sqlx::query_scalar(
            "SELECT nspname::text FROM pg_catalog.pg_namespace \
             WHERE nspname='native_org_unit' OR starts_with(nspname,'native_org_unit_') \
             ORDER BY nspname COLLATE \"C\"",
        )
        .fetch_all(&mut *connection)
        .await
        .unwrap();
        assert!(reserved_relations.is_empty());
        assert!(reserved_schemas.is_empty());
        let guarded: Vec<(String, i64, String, bool, String)> = sqlx::query_as(
            "SELECT c.relname::text,c.oid::bigint,c.relkind::text,c.relispartition,r.rolname::text \
             FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace \
             JOIN pg_catalog.pg_roles r ON r.oid=c.relowner \
             WHERE n.nspname='public' AND c.relname IN \
               ('ont_action_command_receipts','org_unit_revisions','org_unit_source_bindings','org_units') \
             ORDER BY c.relname COLLATE \"C\"")
            .fetch_all(&mut *connection).await.unwrap();
        assert_eq!(guarded.len(), 4);
        assert_eq!(
            guarded.iter().map(|r| r.0.as_str()).collect::<Vec<_>>(),
            vec![
                "ont_action_command_receipts",
                "org_unit_revisions",
                "org_unit_source_bindings",
                "org_units"
            ]
        );
        assert_eq!(
            guarded.iter().map(|r| r.1).collect::<BTreeSet<_>>().len(),
            4
        );
        assert!(
            guarded
                .iter()
                .all(|r| r.1 > 0 && r.2 == "r" && !r.3 && r.4 == "console_app")
        );
        let routines: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT n.nspname::text,p.proname::text,p.prokind::text, \
             pg_catalog.oidvectortypes(p.proargtypes) FROM pg_catalog.pg_proc p \
             JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace \
             WHERE starts_with(p.proname,'native_org_unit_') \
             ORDER BY n.nspname COLLATE \"C\",p.proname COLLATE \"C\",p.proargtypes",
        )
        .fetch_all(&mut *connection)
        .await
        .unwrap();
        let expected = if closed {
            vec![(
                "public".into(),
                "native_org_unit_closed_guard_v1".into(),
                "f".into(),
                "".into(),
            )]
        } else {
            Vec::new()
        };
        assert_eq!(routines, expected);
        assert_eq!(namespace(connection).await, exact_namespace());
        json!({"reserved_relations":reserved_relations,"reserved_schemas":reserved_schemas,
            "guarded_four":guarded,"native_routines":routines})
    }

    struct Baseline {
        target: Target,
        rows: BTreeMap<String, String>,
        catalog: (String, Value),
        ledger: Value,
        denied: Value,
    }

    async fn predecessor(pool: &PgPool, variant: usize) -> Baseline {
        assert!(variant < 2);
        pins();
        let mut empty = PgConnection::connect_with(&pool.connect_options())
            .await
            .unwrap();
        marked(&mut empty, true).await;
        empty.close().await.unwrap();
        // Exactly existing migrations and owner prerequisite installer. Historical
        // rows remain; no new synthetic persons, Companies or business workflow.
        prepare_http_database_staging(pool).await;
        let mut setup = direct(pool).await;
        let mut tx = sqlx::Connection::begin(&mut setup).await.unwrap();
        let absent: bool = sqlx::query_scalar(
            "SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_durability_observer') \
             AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_proc WHERE proname='console_durability_observation_v1')")
            .fetch_one(tx.as_mut()).await.unwrap();
        assert!(
            absent,
            "each named leaf needs its dedicated root-owned disposable cluster"
        );
        let ledger_before = applied_ledger(tx.as_mut()).await;
        sqlx::raw_sql("SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='120s'; SET LOCAL search_path=pg_catalog,pg_temp")
            .execute(tx.as_mut()).await.unwrap();
        let _: String = sqlx::query_scalar(
            "SELECT oid::text FROM pg_catalog.pg_authid WHERE rolname='console_account_owner' FOR UPDATE")
            .fetch_one(tx.as_mut()).await.unwrap();
        if variant == 1 {
            execute(&mut tx, OBSERVER).await;
        }
        for source in [
            ACCOUNT,
            CREDENTIALS,
            COMPANY,
            POLICY_INSTALLER,
            POLICY_V2,
            ROW_LOCK,
        ] {
            execute(&mut tx, source).await;
        }
        assert_eq!(
            capture(tx.as_mut(), PREDECESSOR_CAPTURE).await.sha256,
            CORRECTED[variant]
        );
        assert_eq!(
            row_lock_state(tx.as_mut()).await,
            "native_people_directory.finalized"
        );
        assert!(namespace(tx.as_mut()).await.is_empty());
        // Prerequisite Company owners legitimately migrate historical Group/
        // membership/catalog rows. Freeze every row AFTER those accepted owner
        // transitions; only provenance and closed source effects must be zero.
        let rows_before = rows(tx.as_mut()).await;
        execute(&mut tx, OWNER).await;
        assert_eq!(
            capture(tx.as_mut(), CAPTURE).await.sha256,
            PREDECESSOR73[variant]
        );
        assert_eq!(
            raw_wider_capture(tx.as_mut()).await["snapshot_sha256"],
            PREDECESSOR76[variant]
        );
        assert_eq!(
            state(tx.as_mut()).await,
            "native_org_unit.closed_perimeter_required"
        );
        added_three_denied_rights(tx.as_mut()).await;
        relation_and_schema_closure(tx.as_mut(), false).await;
        assert_eq!(rows(tx.as_mut()).await, rows_before);
        assert_eq!(applied_ledger(tx.as_mut()).await, ledger_before);
        // Committing the explicitly historical prerequisite fixture gives the
        // finalizer a fresh connection without inherited DDL/relation/role locks.
        // Accepted prerequisite owners may have changed historical business
        // rows; none were hand-seeded or attributed to a UI-created journey.
        tx.commit().await.unwrap();
        setup.close().await.unwrap();
        let mut fresh = direct(pool).await;
        let frozen_target = target(&mut fresh).await; // before BEGIN
        let mut readback = sqlx::Connection::begin(&mut fresh).await.unwrap();
        assert_eq!(
            capture(readback.as_mut(), CAPTURE).await.sha256,
            PREDECESSOR73[variant]
        );
        assert_eq!(
            raw_wider_capture(readback.as_mut()).await["snapshot_sha256"],
            PREDECESSOR76[variant]
        );
        let baseline = Baseline {
            target: frozen_target,
            rows: rows(readback.as_mut()).await,
            catalog: catalog(readback.as_mut()).await,
            ledger: applied_ledger(readback.as_mut()).await,
            denied: added_three_denied_rights(readback.as_mut()).await,
        };
        assert_eq!(baseline.rows, rows_before);
        assert_eq!(baseline.ledger, ledger_before);
        readback.rollback().await.unwrap();
        fresh.close().await.unwrap();
        baseline
    }

    async fn restored(pool: &PgPool, baseline: &Baseline, variant: usize) {
        let mut fresh = direct(pool).await;
        assert_eq!(target(&mut fresh).await, baseline.target);
        let mut tx = sqlx::Connection::begin(&mut fresh).await.unwrap();
        assert_eq!(
            rows(tx.as_mut()).await,
            baseline.rows,
            "outer rollback changed any durable row"
        );
        assert_eq!(
            catalog(tx.as_mut()).await,
            baseline.catalog,
            "outer rollback changed raw metadata"
        );
        assert_eq!(applied_ledger(tx.as_mut()).await, baseline.ledger);
        assert_eq!(
            added_three_denied_rights(tx.as_mut()).await,
            baseline.denied
        );
        assert_eq!(
            capture(tx.as_mut(), CAPTURE).await.sha256,
            PREDECESSOR73[variant]
        );
        assert_eq!(
            raw_wider_capture(tx.as_mut()).await["snapshot_sha256"],
            PREDECESSOR76[variant]
        );
        assert_eq!(
            state(tx.as_mut()).await,
            "native_org_unit.closed_perimeter_required"
        );
        relation_and_schema_closure(tx.as_mut(), false).await;
        tx.rollback().await.unwrap();
        fresh.close().await.unwrap();
    }

    async fn retained_locks(connection: &mut PgConnection) -> Value {
        let names: Vec<String> = sqlx::query_scalar(
            "SELECT c.relname::text FROM pg_catalog.pg_locks l \
             JOIN pg_catalog.pg_class c ON c.oid=l.relation \
             JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace \
             WHERE l.pid=pg_backend_pid() AND l.locktype='relation' \
               AND l.mode='AccessExclusiveLock' AND l.granted AND n.nspname='public' \
             ORDER BY c.relname COLLATE \"C\"",
        )
        .fetch_all(&mut *connection)
        .await
        .unwrap();
        assert_eq!(
            names, RELATIONS76,
            "actual retained lock set must equal the independent roster"
        );
        let ledger_share: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_catalog.pg_locks WHERE pid=pg_backend_pid() \
             AND locktype='relation' AND relation='public._sqlx_migrations'::regclass \
             AND mode='ShareLock' AND granted",
        )
        .fetch_one(&mut *connection)
        .await
        .unwrap();
        assert_eq!(ledger_share, 1);
        let transaction: (i32, i64) =
            sqlx::query_as("SELECT pg_backend_pid(),txid_current()::bigint")
                .fetch_one(connection)
                .await
                .unwrap();
        // Sorting proves the retained set, not lock acquisition order. The
        // separately synchronized prefix test observes the actual order prefix.
        json!({"relation_names":names,"ledger_share":ledger_share,
            "backend_pid":transaction.0,"transaction_id":transaction.1})
    }

    fn raw_oid(value: &Value) -> u64 {
        let oid = value.as_str().unwrap().parse::<u64>().unwrap();
        assert!(oid > 0);
        oid
    }

    fn closed_delta(before: &Value, after: &Value) {
        let old = routine_map(before);
        let new = routine_map(after);
        assert_eq!(new.len(), old.len() + 1);
        assert!(
            old.iter().all(|(key, value)| new.get(key) == Some(value)),
            "changed existing routine"
        );
        let additions: Vec<_> = new.keys().filter(|key| !old.contains_key(*key)).collect();
        assert_eq!(additions, vec!["public.native_org_unit_closed_guard_v1()"]);
        let guard = &new["public.native_org_unit_closed_guard_v1()"];
        assert_eq!(guard["owner"], "console_account_owner");
        assert_eq!(guard["language"], "plpgsql");
        assert_eq!(guard["result"], "trigger");
        assert_eq!(
            guard["source_sha256"],
            "afda867ce859a820e3cebe033e1b1597532f99bb07a31174be494f480770dd3f"
        );
        assert_eq!(
            digest(guard["raw"]["prosrc"].as_str().unwrap()),
            guard["source_sha256"].as_str().unwrap()
        );
        assert_eq!(
            guard["raw"]["proconfig"],
            json!([
                "search_path=pg_catalog, pg_temp",
                "row_security=on",
                "lock_timeout=100ms"
            ])
        );
        assert_eq!(
            guard["acl"],
            json!([[
                "console_account_owner",
                "console_account_owner",
                "EXECUTE",
                false
            ]])
        );
        assert_eq!(guard["raw"]["prokind"], "f");
        assert_eq!(guard["raw"]["prosecdef"], true);
        assert_eq!(guard["raw"]["provolatile"], "v");
        assert_eq!(guard["raw"]["proparallel"], "u");
        assert_eq!(guard["raw"]["pronargs"], 0);
        let guard_oid = guard["raw"]["oid"].clone();
        let old_triggers: BTreeMap<_, _> = before["triggers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| (raw_oid(&value["oid"]), value.clone()))
            .collect();
        let new_triggers: BTreeMap<_, _> = after["triggers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| (raw_oid(&value["oid"]), value.clone()))
            .collect();
        assert_eq!(new_triggers.len(), old_triggers.len() + 8);
        let promoted = [
            "trg_org_unit_revisions_immutable",
            "trg_org_unit_source_bindings_immutable",
            "trg_ont_action_command_receipts_immutable",
        ];
        let mut promoted_seen = BTreeSet::new();
        for (oid, old_record) in &old_triggers {
            let mut expected = old_record.clone();
            let name = old_record["tgname"].as_str().unwrap();
            if promoted.contains(&name) {
                assert_eq!(old_record["tgenabled"], "O");
                expected["tgenabled"] = json!("A");
                assert!(promoted_seen.insert(name));
            }
            assert_eq!(
                new_triggers.get(oid),
                Some(&expected),
                "unexpected full trigger delta: {name}"
            );
        }
        assert_eq!(promoted_seen, promoted.into_iter().collect());
        let guarded = [
            "ont_action_command_receipts",
            "org_unit_revisions",
            "org_unit_source_bindings",
            "org_units",
        ];
        let relation_names: BTreeMap<_, _> = after["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| guarded.contains(&r["relname"].as_str().unwrap()))
            .map(|r| (raw_oid(&r["oid"]), r["relname"].as_str().unwrap()))
            .collect();
        assert_eq!(relation_names.len(), 4);
        let mut additions_seen = BTreeSet::new();
        for record in new_triggers
            .values()
            .filter(|r| !old_triggers.contains_key(&raw_oid(&r["oid"])))
        {
            let relation = relation_names[&raw_oid(&record["tgrelid"])];
            let name = record["tgname"].as_str().unwrap();
            assert_eq!(record["tgfoid"], guard_oid);
            let tgtype = match name {
                "native_org_unit_closed_row_v1" => 31,
                "native_org_unit_closed_truncate_v1" => 34,
                _ => panic!("unexpected new trigger {name}"),
            };
            let mut raw = record.clone();
            for field in ["oid", "tgrelid", "tgfoid"] {
                raw.as_object_mut().unwrap().remove(field);
            }
            assert_eq!(
                raw,
                json!({"tgparentid":"0","tgname":name,"tgtype":tgtype,"tgenabled":"A",
                "tgisinternal":false,"tgconstrrelid":"0","tgconstrindid":"0","tgconstraint":"0",
                "tgdeferrable":false,"tginitdeferred":false,"tgnargs":0,"tgattr":[],"tgargs":"\\x",
                "tgqual":null,"tgoldtable":null,"tgnewtable":null})
            );
            assert!(additions_seen.insert((relation, name)));
        }
        let expected_additions: BTreeSet<_> = guarded
            .into_iter()
            .flat_map(|relation| {
                [
                    "native_org_unit_closed_row_v1",
                    "native_org_unit_closed_truncate_v1",
                ]
                .into_iter()
                .map(move |name| (relation, name))
            })
            .collect();
        assert_eq!(additions_seen, expected_additions);
        let mut old_other = before.clone();
        let mut new_other = after.clone();
        for field in ["routines", "triggers"] {
            old_other.as_object_mut().unwrap().remove(field);
            new_other.as_object_mut().unwrap().remove(field);
        }
        assert_eq!(
            old_other, new_other,
            "unexpected non-routine/trigger catalog delta"
        );
    }

    async fn run_finalizer(connection: &mut PgConnection) -> Duration {
        bounds(connection).await;
        let start = Instant::now();
        sqlx::raw_sql(FINALIZER)
            .execute(&mut *connection)
            .await
            .unwrap();
        let elapsed = start.elapsed();
        assert!(
            elapsed <= Duration::from_secs(60),
            "initial finalizer envelope exceeded; no automatic increase"
        );
        sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE")
            .execute(connection)
            .await
            .unwrap();
        elapsed
    }

    async fn wait_for_blocker(observer: &mut PgConnection, waiting: i32, blocker: i32) -> Duration {
        let start = Instant::now();
        loop {
            let blocked: bool =
                sqlx::query_scalar("SELECT $2=ANY(pg_catalog.pg_blocking_pids($1))")
                    .bind(waiting)
                    .bind(blocker)
                    .fetch_one(&mut *observer)
                    .await
                    .unwrap();
            if blocked {
                return start.elapsed();
            }
            assert!(
                start.elapsed() < Duration::from_millis(750),
                "real wait was not observed before bounded stop"
            );
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }

    async fn transition(pool: PgPool, variant: usize) {
        let baseline = predecessor(&pool, variant).await;
        let mut connection = direct(&pool).await;
        let frozen = target(&mut connection).await;
        assert_eq!(frozen, baseline.target);
        let mut tx = begin_protocol(&mut connection, &frozen).await;
        let outcome = AssertUnwindSafe(async {
            let install_elapsed = run_finalizer(tx.as_mut()).await;
            assert_eq!(state(tx.as_mut()).await, "native_org_unit.closed_perimeter_compatible");
            let original = capture(tx.as_mut(), CAPTURE).await;
            let wider = raw_wider_capture(tx.as_mut()).await;
            assert_eq!(original.sha256, CLOSED73[variant]);
            assert_eq!(wider["snapshot_sha256"], CLOSED76[variant]);
            assert_eq!(added_three_denied_rights(tx.as_mut()).await, baseline.denied);
            let closure = relation_and_schema_closure(tx.as_mut(), true).await;
            let installed_catalog = catalog(tx.as_mut()).await;
            closed_delta(&baseline.catalog.1, &installed_catalog.1);
            assert_eq!(rows(tx.as_mut()).await, baseline.rows);
            assert_eq!(applied_ledger(tx.as_mut()).await, baseline.ledger);
            let installed_locks = retained_locks(tx.as_mut()).await;
            let replay_elapsed = run_finalizer(tx.as_mut()).await;
            assert_eq!(capture(tx.as_mut(), CAPTURE).await, original);
            assert_eq!(raw_wider_capture(tx.as_mut()).await, wider);
            assert_eq!(catalog(tx.as_mut()).await, installed_catalog, "replay changed full raw metadata");
            assert_eq!(rows(tx.as_mut()).await, baseline.rows, "replay changed any durable row");
            assert_eq!(applied_ledger(tx.as_mut()).await, baseline.ledger);
            assert_eq!(retained_locks(tx.as_mut()).await, installed_locks, "replay lost transaction/locks");
            assert_eq!(added_three_denied_rights(tx.as_mut()).await, baseline.denied);
            assert_eq!(relation_and_schema_closure(tx.as_mut(), true).await, closure);
            json!({"variant":if variant==0 {"plain"} else {"observer"},"source_pins":pins(),
                "database":frozen.database,"database_oid":frozen.database_oid,"system_identifier":frozen.system_identifier,
                "installed73":original.sha256,"installed76":wider["snapshot_sha256"],
                "install_elapsed_ms":install_elapsed.as_millis(),"replay_elapsed_ms":replay_elapsed.as_millis(),
                "retained_locks":installed_locks,"strict_denials":{"table":24,"column":80},
                "row_summary":row_summary(&baseline.rows),"ledger_records":231,
                "closed_source_metadata_only":true,"historical_prerequisite_owner_effects":true,"business_empty":false,"serving_accepted":false,
                "organization_workflow_accepted":false,"production_qualified":false})
        }).catch_unwind().await;
        tx.rollback()
            .await
            .expect("entire closed installation and replay must roll back");
        connection.close().await.unwrap();
        restored(&pool, &baseline, variant).await;
        let packet = match outcome {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        };
        let mut output = std::io::stderr().lock();
        writeln!(&mut output, "ORG_CLOSED_FINALIZER_PROTOCOL {packet}").unwrap();
        output.flush().unwrap();
    }

    #[sqlx::test(migrations = false)]
    async fn closed_finalizer_plain_transition_replay_and_rollback(pool: PgPool) {
        transition(pool, 0).await;
    }

    #[sqlx::test(migrations = false)]
    async fn closed_finalizer_observer_transition_replay_and_rollback(pool: PgPool) {
        transition(pool, 1).await;
    }

    #[sqlx::test(migrations = false)]
    async fn closed_finalizer_malformed_ledger_and_replay_revalidation_refuse_atomically(
        pool: PgPool,
    ) {
        let baseline = predecessor(&pool, 0).await;
        // Actual isolated ledger corruptions only; historical SQL/ledger source
        // bytes are unchanged. Every attempt rolls back its entire transaction.
        for replay in [false, true] {
            for (name, fault) in [
                (
                    "checksum_null",
                    "ALTER TABLE public._sqlx_migrations ALTER COLUMN checksum DROP NOT NULL; UPDATE public._sqlx_migrations SET checksum=NULL WHERE version=231",
                ),
                (
                    "historical_checksum_mismatch",
                    "UPDATE public._sqlx_migrations SET checksum=decode('00','hex') WHERE version=1",
                ),
                (
                    "success_false",
                    "UPDATE public._sqlx_migrations SET success=false WHERE version=231",
                ),
                (
                    "missing_record",
                    "DELETE FROM public._sqlx_migrations WHERE version=231",
                ),
            ] {
                let mut connection = direct(&pool).await;
                let frozen = target(&mut connection).await;
                let mut tx = begin_protocol(&mut connection, &frozen).await;
                let outcome = AssertUnwindSafe(async {
                    if replay {
                        run_finalizer(tx.as_mut()).await;
                    }
                    sqlx::raw_sql(fault).execute(tx.as_mut()).await.unwrap();
                    bounds(tx.as_mut()).await;
                    let error = sqlx::raw_sql(FINALIZER)
                        .execute(tx.as_mut())
                        .await
                        .unwrap_err();
                    let database = error
                        .as_database_error()
                        .expect("actual database refusal required");
                    assert_eq!(database.code().as_deref(), Some("P0001"));
                    assert_eq!(
                        database.message(),
                        "native_org_unit.migration_ledger_mismatch",
                        "wrong refusal for {name} replay={replay}"
                    );
                })
                .catch_unwind()
                .await;
                tx.rollback().await.unwrap(); // never continue the aborted transaction
                connection.close().await.unwrap();
                restored(&pool, &baseline, 0).await;
                if let Err(panic) = outcome {
                    std::panic::resume_unwind(panic);
                }
            }
        }
    }

    #[sqlx::test(migrations = false)]
    async fn closed_finalizer_entry_target_identity_isolation_bounds_refuse_atomically(
        pool: PgPool,
    ) {
        let baseline = predecessor(&pool, 0).await;
        // Caller descriptor custody is a separate executable boundary: finalizer
        // DO does not pretend to validate an unfurnished expected target.
        let mut descriptor_connection = direct(&pool).await;
        for field in ["database", "database_oid", "system_identifier"] {
            let mut wrong = baseline.target.clone();
            match field {
                "database" => wrong.database.push_str("_wrong"),
                "database_oid" => wrong.database_oid += 1,
                "system_identifier" => wrong.system_identifier.push('0'),
                _ => unreachable!(),
            }
            assert!(
                !matches_target(&mut descriptor_connection, &wrong).await,
                "caller must reject wrong {field} before BEGIN/DO"
            );
        }
        descriptor_connection.close().await.unwrap();
        for (name, fault, message) in [
            (
                "lock_zero",
                "SET LOCAL lock_timeout='0'",
                "native_org_unit.entry_bounds_mismatch",
            ),
            (
                "lock_over",
                "SET LOCAL lock_timeout='1001ms'",
                "native_org_unit.entry_bounds_mismatch",
            ),
            (
                "statement_zero",
                "SET LOCAL statement_timeout='0'",
                "native_org_unit.entry_bounds_mismatch",
            ),
            (
                "statement_over",
                "SET LOCAL statement_timeout='60001ms'",
                "native_org_unit.entry_bounds_mismatch",
            ),
            (
                "idle_zero",
                "SET LOCAL idle_in_transaction_session_timeout='0'",
                "native_org_unit.entry_bounds_mismatch",
            ),
            (
                "idle_over",
                "SET LOCAL idle_in_transaction_session_timeout='30001ms'",
                "native_org_unit.entry_bounds_mismatch",
            ),
            (
                "transaction_zero",
                "SET LOCAL transaction_timeout='0'",
                "native_org_unit.entry_bounds_mismatch",
            ),
            (
                "transaction_over",
                "SET LOCAL transaction_timeout='120001ms'",
                "native_org_unit.entry_bounds_mismatch",
            ),
            (
                "wrong_search_path",
                "SET LOCAL search_path=public,pg_catalog",
                "native_org_unit.entry_settings_mismatch",
            ),
            (
                "jit_on",
                "SET LOCAL jit=on",
                "native_org_unit.entry_settings_mismatch",
            ),
            (
                "marker_mismatch",
                "SET LOCAL console.sqlx_test_bootstrap='wrong'",
                "native_org_unit.operator_identity_mismatch",
            ),
            (
                "set_role",
                "SET LOCAL ROLE console_account_owner",
                "native_org_unit.operator_identity_mismatch",
            ),
        ] {
            let mut connection = direct(&pool).await;
            let frozen = target(&mut connection).await;
            let mut tx = begin_protocol(&mut connection, &frozen).await;
            let outcome = AssertUnwindSafe(async {
                sqlx::raw_sql(fault).execute(tx.as_mut()).await.unwrap();
                let error = sqlx::raw_sql(FINALIZER)
                    .execute(tx.as_mut())
                    .await
                    .unwrap_err();
                let database = error.as_database_error().unwrap();
                assert_eq!(database.code().as_deref(), Some("P0001"));
                assert_eq!(database.message(), message, "wrong refusal for {name}");
            })
            .catch_unwind()
            .await;
            tx.rollback().await.unwrap();
            connection.close().await.unwrap();
            restored(&pool, &baseline, 0).await;
            if let Err(panic) = outcome {
                std::panic::resume_unwind(panic);
            }
        }
        // Unsupported isolation is set before the first transaction query.
        for isolation in ["REPEATABLE READ", "SERIALIZABLE"] {
            let mut connection = direct(&pool).await;
            assert_eq!(target(&mut connection).await, baseline.target);
            let mut tx = sqlx::Connection::begin(&mut connection).await.unwrap();
            let outcome = AssertUnwindSafe(async {
                let statement = format!("SET TRANSACTION ISOLATION LEVEL {isolation}");
                sqlx::raw_sql(sqlx::AssertSqlSafe(statement))
                    .execute(tx.as_mut())
                    .await
                    .unwrap();
                sqlx::raw_sql(ENTRY_BOUNDS)
                    .execute(tx.as_mut())
                    .await
                    .unwrap();
                let error = sqlx::raw_sql(FINALIZER)
                    .execute(tx.as_mut())
                    .await
                    .unwrap_err();
                let database = error.as_database_error().unwrap();
                assert_eq!(database.code().as_deref(), Some("P0001"));
                assert_eq!(
                    database.message(),
                    "native_org_unit.entry_settings_mismatch"
                );
            })
            .catch_unwind()
            .await;
            tx.rollback().await.unwrap();
            connection.close().await.unwrap();
            restored(&pool, &baseline, 0).await;
            if let Err(panic) = outcome {
                std::panic::resume_unwind(panic);
            }
        }
    }

    #[sqlx::test(migrations = false)]
    async fn closed_finalizer_actual_prefix_wait_and_cancellation_roll_back(pool: PgPool) {
        let baseline = predecessor(&pool, 0).await;
        let mut blocker = direct(&pool).await;
        let blocker_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut blocker)
            .await
            .unwrap();
        let mut blocking = sqlx::Connection::begin(&mut blocker).await.unwrap();
        bounds(blocking.as_mut()).await;
        sqlx::raw_sql("LOCK TABLE ONLY public.account_security IN ACCESS SHARE MODE")
            .execute(blocking.as_mut())
            .await
            .unwrap();
        let mut installer = direct(&pool).await;
        let frozen = target(&mut installer).await;
        let installer_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut installer)
            .await
            .unwrap();
        let task = tokio::spawn(async move {
            let mut tx = begin_protocol(&mut installer, &frozen).await;
            bounds(tx.as_mut()).await;
            let result = sqlx::raw_sql(FINALIZER).execute(tx.as_mut()).await;
            tx.rollback().await.unwrap();
            installer.close().await.unwrap();
            result
        });
        let mut observer = direct(&pool).await;
        let outcome=AssertUnwindSafe(async {
            let wait=wait_for_blocker(&mut observer,installer_pid,blocker_pid).await;
            let locks:Vec<(String,String,bool)>=sqlx::query_as(
                "SELECT c.relname::text,l.mode::text,l.granted FROM pg_catalog.pg_locks l \
                 JOIN pg_catalog.pg_class c ON c.oid=l.relation \
                 JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace \
                 WHERE l.pid=$1 AND n.nspname='public' \
                   AND l.mode='AccessExclusiveLock' ORDER BY c.relname COLLATE \"C\"")
                .bind(installer_pid).fetch_all(&mut observer).await.unwrap();
            assert_eq!(locks,vec![("account_context_candidates".into(),"AccessExclusiveLock".into(),true),
                ("account_security".into(),"AccessExclusiveLock".into(),false)],
                "actual finalizer must acquire first C relation and wait on second before later relations");
            assert!(wait<Duration::from_millis(750));
            let cancelled:bool=sqlx::query_scalar("SELECT pg_catalog.pg_cancel_backend($1)")
                .bind(installer_pid).fetch_one(&mut observer).await.unwrap();
            assert!(cancelled);
        }).catch_unwind().await;
        // Release owned blocker even on assertion panic, and join/close the real
        // cancelled operation before collecting fresh restoration evidence.
        blocking.rollback().await.unwrap();
        blocker.close().await.unwrap();
        let result = tokio::time::timeout(Duration::from_secs(5), task)
            .await
            .expect("bounded cancellation did not finish")
            .unwrap();
        observer.close().await.unwrap();
        restored(&pool, &baseline, 0).await;
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
        let error = result.unwrap_err();
        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some("57014")
        );
    }

    #[sqlx::test(migrations = false)]
    async fn closed_finalizer_actual_account_owner_lock_contends_across_databases_until_rollback(
        pool: PgPool,
    ) {
        let baseline = predecessor(&pool, 0).await;
        let mut creator = direct(&pool).await;
        let other_database = format!("_sqlx_test_org_role_{}", Uuid::new_v4().simple());
        let create = format!("CREATE DATABASE \"{other_database}\" TEMPLATE template0");
        sqlx::raw_sql(sqlx::AssertSqlSafe(create))
            .execute(&mut creator)
            .await
            .unwrap();
        let options = pool
            .connect_options()
            .as_ref()
            .clone()
            .database(&other_database);
        let mut other = PgConnection::connect_with(&options).await.unwrap();
        let other_target = target(&mut other).await;
        assert_ne!(other_target.database_oid, baseline.target.database_oid);
        assert_eq!(
            other_target.system_identifier,
            baseline.target.system_identifier
        );
        assert_eq!(other_target.database, other_database);
        marked(&mut other, false).await;
        let other_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut other)
            .await
            .unwrap();
        let mut installer = direct(&pool).await;
        let frozen = target(&mut installer).await;
        let mut tx = begin_protocol(&mut installer, &frozen).await;
        let installer_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(tx.as_mut())
            .await
            .unwrap();
        // Real finalizer success and replay keep the same actual role row lock.
        run_finalizer(tx.as_mut()).await;
        let locks_before = retained_locks(tx.as_mut()).await;
        run_finalizer(tx.as_mut()).await;
        assert_eq!(retained_locks(tx.as_mut()).await, locks_before);
        let task = tokio::spawn(async move {
            let mut waiter = sqlx::Connection::begin(&mut other).await.unwrap();
            sqlx::raw_sql(ENTRY_BOUNDS)
                .execute(waiter.as_mut())
                .await
                .unwrap();
            let result:Result<String,_>=sqlx::query_scalar(
                "SELECT oid::text FROM pg_catalog.pg_authid WHERE rolname='console_account_owner' FOR UPDATE")
                .fetch_one(waiter.as_mut()).await;
            waiter.rollback().await.unwrap();
            other.close().await.unwrap();
            result
        });
        let outcome = AssertUnwindSafe(async {
            let wait = wait_for_blocker(&mut creator, other_pid, installer_pid).await;
            assert!(wait < Duration::from_millis(750));
            // Observe actual shared transaction wait. No fabricated tuple-lock
            // census substitutes for the contending FOR UPDATE operation.
            let transaction_wait: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_locks WHERE pid=$1 \
                   AND locktype='transactionid' AND mode='ShareLock' AND NOT granted)",
            )
            .bind(other_pid)
            .fetch_one(&mut creator)
            .await
            .unwrap();
            assert!(
                transaction_wait,
                "actual account-owner row waiter must wait on holding transaction"
            );
            assert_eq!(retained_locks(tx.as_mut()).await, locks_before);
        })
        .catch_unwind()
        .await;
        tx.rollback().await.unwrap();
        installer.close().await.unwrap();
        let waiter_result = tokio::time::timeout(Duration::from_secs(5), task)
            .await
            .expect("role waiter did not finish after rollback")
            .unwrap();
        let drop_statement = format!("DROP DATABASE \"{other_database}\" WITH (FORCE)");
        sqlx::raw_sql(sqlx::AssertSqlSafe(drop_statement))
            .execute(&mut creator)
            .await
            .unwrap();
        let remains: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_database WHERE datname=$1)",
        )
        .bind(&other_database)
        .fetch_one(&mut creator)
        .await
        .unwrap();
        assert!(!remains, "owned contention database was not removed");
        creator.close().await.unwrap();
        restored(&pool, &baseline, 0).await;
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
        assert!(
            waiter_result
                .expect("actual role row could not be acquired after rollback")
                .parse::<u32>()
                .unwrap()
                > 0
        );
    }
}
