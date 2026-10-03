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
