// Additive, UNACCEPTED diagnostic source proposal. Root owns execution and the
// dedicated disposable cluster/role/schema lease. No phase hash is an oracle.
// Mount inside native_group_process_finalizer_tests after its navigation includes.
mod native_org_unit_account_actor_measurements {
    use super::*;
    use sqlx::Connection as _;
    use std::io::Write as _;

    const ACTOR_EXPANSION: &str = include_str!(
        "../../../../ops/postgres-native-org-unit-account-actor-expansion-v1-owner.sql"
    );
    const ACTOR_VALIDATION: &str = include_str!(
        "../../../../ops/postgres-native-org-unit-account-actor-validation-v1-owner.sql"
    );
    const ACTOR_CLOSED_CAPTURE: &str = include_str!(
        "../../../../ops/postgres-capture-native-org-unit-account-actor-closed76-v1-custody.sql"
    );
    const ACTOR_GROUP_CAPTURE: &str = include_str!(
        "../../../../ops/postgres-capture-native-org-unit-account-actor-group83-v1-custody.sql"
    );
    const OLD_ORG_COLUMNS: [&str; 10] = [
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
    ];
    const ACTOR_CHECK: &str = "((actor_kind='USER' AND actor_id IS NOT NULL AND actor_account_id IS NULL) \
        OR (actor_kind='ACCOUNT' AND actor_id IS NULL AND actor_account_id IS NOT NULL)) IS TRUE";
    const RECEIPT_CHECK: &str = "((actor_kind='USER' AND actor_id IS NOT NULL AND actor_account_id IS NULL) \
        OR (actor_kind='ACCOUNT' AND actor_id IS NULL AND actor_account_id IS NOT NULL \
        AND ((owner='person' AND target='people.create_person' AND action_key='directory_create' AND object_type_id IS NOT NULL) \
        OR (owner='org_unit' AND target='organization.create_org_unit' AND action_key='create_site' AND object_type_id IS NOT NULL) \
        OR (owner='org_unit' AND target='organization.revise_org_unit' AND action_key='correct_site_name' AND object_type_id IS NOT NULL)))) IS TRUE";
    const SUCCESSOR_NAMES: [&str; 3] = [
        "ont_action_receipts_actor_protocol_v2",
        "org_unit_revisions_actor_protocol_v1",
        "org_unit_revisions_native_actor_v1",
    ];

    #[derive(Clone, Copy, Debug)]
    enum ActorFixtureFamily {
        Closed76,
        Group83,
        Navigation83,
    }
    #[derive(Clone, Copy, Debug)]
    enum ActorMeasurementPhase {
        Expansion,
        Validation,
    }

    fn actor_phase_pins() -> Value {
        let mut sources = navigation_finalizer_pins();
        for (name, source, expected) in [
            (
                "actor_expansion",
                ACTOR_EXPANSION,
                "ed2e9df259bbba5b3fbfc9585a42eb210461f20e58cf6b28de6ceed5399391c9",
            ),
            (
                "actor_validation",
                ACTOR_VALIDATION,
                "61630ef6bae75543e1bcab0d3bb760cdbde40429b227df71b1a34285909f01b5",
            ),
            (
                "actor_closed76_capture",
                ACTOR_CLOSED_CAPTURE,
                "6be2e3d095d59bbdb9e1b932dac8da48bde261601455cdcd166c6f2a649e6010",
            ),
            (
                "actor_group83_capture",
                ACTOR_GROUP_CAPTURE,
                "3406bac381896fab4e9a1c079110d3dc770a9d89b0b564e7744034379f60cd3b",
            ),
        ] {
            assert_eq!(
                digest(source),
                expected,
                "unreviewed actor diagnostic source: {name}"
            );
            assert!(
                sources
                    .as_object_mut()
                    .unwrap()
                    .insert(name.into(), json!(expected))
                    .is_none()
            );
        }
        assert_eq!(ACTOR_CLOSED_CAPTURE, WIDER_CAPTURE);
        assert_eq!(ACTOR_GROUP_CAPTURE, GROUP_CAPTURE);
        sources
    }

    async fn actor_capture(connection: &mut PgConnection, family: ActorFixtureFamily) -> Capture {
        sqlx::raw_sql(CLASSIFIER_SESSION)
            .execute(&mut *connection)
            .await
            .unwrap();
        let (source, verdict, expected_rights, expected_tables) = match family {
            ActorFixtureFamily::Closed76 => (
                ACTOR_CLOSED_CAPTURE,
                "native_directory_startup_rights_valid",
                false,
                76,
            ),
            ActorFixtureFamily::Group83 | ActorFixtureFamily::Navigation83 => (
                ACTOR_GROUP_CAPTURE,
                "native_group_process_startup_rights_valid",
                true,
                83,
            ),
        };
        let body = source
            .strip_suffix(";\n")
            .expect("pinned complete capture terminator");
        let query =
            format!("SELECT snapshot::text,snapshot_sha256,{verdict} FROM ({body}) actor_capture");
        let (text, sha256, rights): (String, String, Option<bool>) =
            sqlx::query_as(sqlx::AssertSqlSafe(query))
                .fetch_one(connection)
                .await
                .unwrap();
        assert_eq!(
            digest(&text),
            sha256,
            "actual PostgreSQL snapshot text/digest mismatch"
        );
        assert_eq!(
            rights,
            Some(expected_rights),
            "historical raw rights verdict must remain exact"
        );
        let snapshot: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(
            snapshot["tables"].as_array().unwrap().len(),
            expected_tables
        );
        Capture {
            text,
            sha256,
            rights: expected_rights,
            snapshot,
        }
    }

    // Complete old rows are compared as exact PostgreSQL text. Only the two
    // declared new columns are projected away, after proving the old roster.
    // This is not a replacement for any historical test or custody serializer.
    async fn actor_old_rows(
        connection: &mut PgConnection,
        staged: bool,
    ) -> BTreeMap<String, String> {
        let roster: Vec<String> = sqlx::query_scalar(
            "SELECT attname::text FROM pg_catalog.pg_attribute WHERE attrelid='public.org_unit_revisions'::regclass \
             AND attnum>0 AND NOT attisdropped ORDER BY attnum")
            .fetch_all(&mut *connection).await.unwrap();
        let mut expected: Vec<String> = OLD_ORG_COLUMNS.into_iter().map(str::to_owned).collect();
        if staged {
            expected.extend(["actor_kind".into(), "actor_account_id".into()]);
        }
        assert_eq!(
            roster, expected,
            "exact declared old/new column roster required"
        );
        let mut census = rows(connection).await;
        if staged {
            let raw: String = sqlx::query_scalar(
                "SELECT COALESCE(jsonb_agg(to_jsonb(t)-ARRAY['actor_kind','actor_account_id'] \
                 ORDER BY (to_jsonb(t)-ARRAY['actor_kind','actor_account_id'])::text COLLATE \"C\"),'[]'::jsonb)::text \
                 FROM public.org_unit_revisions t")
                .fetch_one(&mut *connection).await.unwrap();
            assert!(
                census
                    .insert("[\"public\",\"org_unit_revisions\"]".into(), raw)
                    .is_some()
            );
            let defaults: bool = sqlx::query_scalar(
                "SELECT NOT EXISTS(SELECT 1 FROM public.org_unit_revisions \
                 WHERE actor_kind IS DISTINCT FROM 'USER' OR actor_account_id IS NOT NULL)",
            )
            .fetch_one(connection)
            .await
            .unwrap();
            assert!(
                defaults,
                "every historical row must retain USER attribution and NULL Account"
            );
        }
        census
    }

    // Raw default/comment/dependency observations supplement the existing
    // complete security catalog. They are diagnostic, never a portable hash.
    const ACTOR_EXTRA: &str = r#"WITH relations AS (
        SELECT c.oid FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
        WHERE n.nspname NOT IN ('pg_catalog','information_schema')
         AND NOT starts_with(n.nspname,'pg_toast') AND NOT starts_with(n.nspname,'pg_temp_')
      ), objects AS (
        SELECT 'pg_catalog.pg_class'::regclass AS classid,oid AS objid FROM relations
        UNION SELECT 'pg_catalog.pg_attrdef'::regclass,a.oid FROM pg_catalog.pg_attrdef a JOIN relations r ON r.oid=a.adrelid
        UNION SELECT 'pg_catalog.pg_constraint'::regclass,k.oid FROM pg_catalog.pg_constraint k JOIN relations r ON r.oid=k.conrelid
        UNION SELECT 'pg_catalog.pg_trigger'::regclass,t.oid FROM pg_catalog.pg_trigger t JOIN relations r ON r.oid=t.tgrelid
        UNION SELECT 'pg_catalog.pg_proc'::regclass,p.oid FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
          WHERE n.nspname NOT IN ('pg_catalog','information_schema') AND NOT starts_with(n.nspname,'pg_toast') AND NOT starts_with(n.nspname,'pg_temp_')
        UNION SELECT 'pg_catalog.pg_type'::regclass,t.oid FROM pg_catalog.pg_type t JOIN pg_catalog.pg_namespace n ON n.oid=t.typnamespace
          WHERE n.nspname NOT IN ('pg_catalog','information_schema') AND NOT starts_with(n.nspname,'pg_toast') AND NOT starts_with(n.nspname,'pg_temp_')
      ) SELECT jsonb_build_object(
        'types',(SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.oid),'[]'::jsonb)
          FROM pg_catalog.pg_type t JOIN objects o ON o.classid='pg_catalog.pg_type'::regclass AND o.objid=t.oid),
        'defaults',(SELECT COALESCE(jsonb_agg(to_jsonb(a)||jsonb_build_object('oid',a.oid::text,'adrelid',a.adrelid::text) ORDER BY a.oid),'[]'::jsonb)
          FROM pg_catalog.pg_attrdef a JOIN relations r ON r.oid=a.adrelid),
        'comments',(SELECT COALESCE(jsonb_agg(to_jsonb(d)||jsonb_build_object('classoid',d.classoid::text,'objoid',d.objoid::text) ORDER BY d.classoid,d.objoid,d.objsubid),'[]'::jsonb)
          FROM pg_catalog.pg_description d JOIN objects o ON (o.classid,o.objid)=(d.classoid,d.objoid)),
        'dependencies',(SELECT COALESCE(jsonb_agg(to_jsonb(d)||jsonb_build_object('classid',d.classid::text,'objid',d.objid::text,
          'refclassid',d.refclassid::text,'refobjid',d.refobjid::text) ORDER BY to_jsonb(d)::text COLLATE "C"),'[]'::jsonb)
          FROM pg_catalog.pg_depend d JOIN objects o ON (o.classid,o.objid)=(d.classid,d.objid)),
        'tuples',(SELECT COALESCE(jsonb_agg(jsonb_build_array(kind,oid,xmin,ctid) ORDER BY kind COLLATE "C",oid),'[]'::jsonb) FROM (
          SELECT 'relation'::text kind,c.oid::bigint oid,c.xmin::text xmin,c.ctid::text ctid FROM pg_catalog.pg_class c JOIN relations r ON r.oid=c.oid
          UNION ALL SELECT 'attribute',a.attrelid::bigint*100000+a.attnum,a.xmin::text,a.ctid::text FROM pg_catalog.pg_attribute a JOIN relations r ON r.oid=a.attrelid
          UNION ALL SELECT 'constraint',k.oid::bigint,k.xmin::text,k.ctid::text FROM pg_catalog.pg_constraint k JOIN relations r ON r.oid=k.conrelid
          UNION ALL SELECT 'trigger',t.oid::bigint,t.xmin::text,t.ctid::text FROM pg_catalog.pg_trigger t JOIN relations r ON r.oid=t.tgrelid
          UNION ALL SELECT 'default',a.oid::bigint,a.xmin::text,a.ctid::text FROM pg_catalog.pg_attrdef a JOIN relations r ON r.oid=a.adrelid
          UNION ALL SELECT 'routine',p.oid::bigint,p.xmin::text,p.ctid::text FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
            WHERE n.nspname NOT IN ('pg_catalog','information_schema') AND NOT starts_with(n.nspname,'pg_toast') AND NOT starts_with(n.nspname,'pg_temp_')
        ) tuples)
      )"#;

    async fn actor_extra(connection: &mut PgConnection) -> Value {
        sqlx::query_scalar(ACTOR_EXTRA)
            .fetch_one(connection)
            .await
            .unwrap()
    }

    #[derive(Clone)]
    struct ActorMeasurementBaseline {
        target: Target,
        capture: Capture,
        catalog: (String, Value),
        extra: Value,
        rows: BTreeMap<String, String>,
        ledger: Value,
        native_census: Value,
    }

    async fn actor_native_census(connection: &mut PgConnection) -> Value {
        let origins: Vec<(Uuid, bool, String)> = sqlx::query_as(
            "SELECT id,origin_account_id IS NOT NULL,public.account_company_provenance_v1(id) \
             FROM public.organizations ORDER BY id",
        )
        .fetch_all(&mut *connection)
        .await
        .unwrap();
        assert!(
            !origins.is_empty(),
            "real accepted historical Company fixture required"
        );
        assert!(
            origins
                .iter()
                .all(|(_, native, provenance)| provenance
                    == if *native { "NATIVE" } else { "LEGACY" }),
            "every actual Company must have exact accepted provenance"
        );
        let native: Vec<Uuid> = origins
            .iter()
            .filter(|(_, native, _)| *native)
            .map(|(id, _, _)| *id)
            .collect();
        let effects: (i64, i64, i64, i64) = sqlx::query_as(
            "SELECT (SELECT count(*) FROM public.org_units WHERE org_id=ANY($1)), \
              (SELECT count(*) FROM public.org_unit_revisions WHERE org_id=ANY($1)), \
              (SELECT count(*) FROM public.org_unit_source_bindings WHERE org_id=ANY($1)), \
              (SELECT count(*) FROM public.ont_action_command_receipts WHERE org_id=ANY($1) \
               AND (owner='org_unit' OR target IN('organization.create_org_unit','organization.revise_org_unit') \
                 OR receipt->>'target' IN('organization.create_org_unit','organization.revise_org_unit')))")
            .bind(&native).fetch_one(connection).await.unwrap();
        assert_eq!(
            effects,
            (0, 0, 0, 0),
            "native Organization must remain closed"
        );
        json!({"company_provenance":origins,"native_company_count":native.len(),
            "org_unit_anchor_revision_binding_receipt_counts":[effects.0,effects.1,effects.2,effects.3],
            "ui_created_business_fixture":false})
    }

    async fn actor_snapshot(
        connection: &mut PgConnection,
        family: ActorFixtureFamily,
        staged: bool,
    ) -> ActorMeasurementBaseline {
        let frozen_target = target(connection).await;
        let native_census = actor_native_census(connection).await;
        ActorMeasurementBaseline {
            target: frozen_target,
            capture: actor_capture(connection, family).await,
            catalog: catalog(connection).await,
            extra: actor_extra(connection).await,
            rows: actor_old_rows(connection, staged).await,
            ledger: applied_ledger(connection).await,
            native_census,
        }
    }

    async fn actor_fixture(
        pool: &PgPool,
        family: ActorFixtureFamily,
        variant: usize,
    ) -> ActorMeasurementBaseline {
        actor_phase_pins();
        assert!(variant < 2);
        match family {
            ActorFixtureFamily::Closed76 => {
                let original = predecessor(pool, variant).await;
                let closed = group_closed_fixture(pool, &original, variant).await;
                serving_restored(pool, variant, &closed).await;
            }
            ActorFixtureFamily::Group83 | ActorFixtureFamily::Navigation83 => {
                let original = navigation_original_fixture(pool, variant).await;
                if matches!(family, ActorFixtureFamily::Navigation83) {
                    let mut admin = direct(pool).await;
                    let mut correction = begin_protocol(&mut admin, &original.target).await;
                    let outcome = AssertUnwindSafe(async {
                        navigation_finalize(correction.as_mut()).await;
                        navigation_snapshot(correction.as_mut(), true, variant).await
                    })
                    .catch_unwind()
                    .await;
                    let corrected = match outcome {
                        Ok(value) => {
                            correction.commit().await.unwrap();
                            value
                        }
                        Err(panic) => {
                            correction.rollback().await.unwrap();
                            admin.close().await.unwrap();
                            navigation_restored(pool, &original, false, variant).await;
                            std::panic::resume_unwind(panic)
                        }
                    };
                    admin.close().await.unwrap();
                    navigation_restored(pool, &corrected, true, variant).await;
                }
            }
        }
        let mut fresh = direct(pool).await;
        let mut readback = sqlx::Connection::begin(&mut fresh).await.unwrap();
        bounds(readback.as_mut()).await;
        let baseline = actor_snapshot(readback.as_mut(), family, false).await;
        let expected = match family {
            ActorFixtureFamily::Closed76 => CLOSED76[variant],
            ActorFixtureFamily::Group83 => INSTALLED83[variant],
            ActorFixtureFamily::Navigation83 => NAVIGATION_CORRECTED83[variant],
        };
        assert_eq!(
            baseline.capture.sha256, expected,
            "positive frozen predecessor must pass before measurement"
        );
        relation_and_schema_closure(readback.as_mut(), true).await;
        readback.rollback().await.unwrap();
        fresh.close().await.unwrap();
        baseline
    }

    async fn actor_phase_locks(connection: &mut PgConnection, expansion: bool) -> Value {
        let (company, changes) = if expansion {
            ("SHARE ROW EXCLUSIVE", "ACCESS EXCLUSIVE")
        } else {
            ("ROW SHARE", "SHARE UPDATE EXCLUSIVE")
        };
        // Literal C order; do not turn a capture's AccessShare into a stronger
        // Group business lock. Existing ledger SHARE/role row lock is retained.
        let sql = format!(
            "LOCK TABLE ONLY public.company_actors IN {company} MODE; \
            LOCK TABLE ONLY public.ont_action_command_receipts IN {changes} MODE; \
            LOCK TABLE ONLY public.org_unit_revisions IN {changes} MODE"
        );
        bounds(connection).await;
        sqlx::raw_sql(sqlx::AssertSqlSafe(sql))
            .execute(&mut *connection)
            .await
            .unwrap();
        let actual: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT c.relname::text,c.oid::text,l.mode::text FROM pg_catalog.pg_locks l \
             JOIN pg_catalog.pg_class c ON c.oid=l.relation JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace \
             WHERE l.pid=pg_backend_pid() AND l.locktype='relation' AND l.granted AND n.nspname='public' \
               AND (l.mode NOT IN ('AccessShareLock','RowShareLock') \
                 OR (c.relname='company_actors' AND l.mode='RowShareLock')) \
             ORDER BY c.relname COLLATE \"C\",l.mode COLLATE \"C\"")
            .fetch_all(&mut *connection).await.unwrap();
        let expected: Vec<(&str, &str)> = if expansion {
            vec![
                ("_sqlx_migrations", "ShareLock"),
                ("company_actors", "ShareRowExclusiveLock"),
                ("ont_action_command_receipts", "AccessExclusiveLock"),
                ("org_unit_revisions", "AccessExclusiveLock"),
            ]
        } else {
            vec![
                ("_sqlx_migrations", "ShareLock"),
                ("company_actors", "RowShareLock"),
                ("ont_action_command_receipts", "ShareUpdateExclusiveLock"),
                ("org_unit_revisions", "ShareUpdateExclusiveLock"),
            ]
        };
        assert_eq!(
            actual
                .iter()
                .map(|(name, _, mode)| (name.as_str(), mode.as_str()))
                .collect::<Vec<_>>(),
            expected
        );
        assert!(
            actual
                .iter()
                .all(|(_, oid, _)| oid.parse::<u32>().unwrap() > 0)
        );
        json!(actual)
    }

    fn actor_catalog_records(
        catalog: &Value,
        field: &str,
        identity: &str,
    ) -> BTreeMap<String, Value> {
        catalog[field]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| (value[identity].as_str().unwrap().to_owned(), value.clone()))
            .collect()
    }

    // Remove only insignificant printed parentheses/whitespace/text casts.
    // A separately executable exhaustive truth matrix below verifies grouping,
    // NULL semantics and the IS TRUE boundary against literal declared rules.
    fn actor_check_tokens(expression: &str) -> String {
        expression
            .replace("::text", "")
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '(' && *c != ')')
            .collect()
    }

    async fn actor_check_semantics(connection: &mut PgConnection, name: &str, declared: &str) {
        let expression: String = sqlx::query_scalar(
            "SELECT pg_catalog.pg_get_expr(conbin,conrelid,false) FROM pg_catalog.pg_constraint \
             WHERE conname=$1 AND conrelid=CASE WHEN $1='ont_action_receipts_actor_protocol_v2' \
               THEN 'public.ont_action_command_receipts'::regclass ELSE 'public.org_unit_revisions'::regclass END")
            .bind(name).fetch_one(&mut *connection).await.unwrap();
        assert!(expression.len() <= 4096);
        assert_eq!(
            actor_check_tokens(&expression),
            actor_check_tokens(declared),
            "unexpected CHECK tokens: {name}"
        );
        let query = format!(
            "SELECT count(*)::bigint,bool_and(({expression}) IS NOT DISTINCT FROM ({declared})) \
            FROM (VALUES ('USER'::text),('ACCOUNT'),('OTHER'),(NULL)) k(actor_kind) \
            CROSS JOIN (VALUES ('00000000-0000-0000-0000-000000000001'::uuid),(NULL)) u(actor_id) \
            CROSS JOIN (VALUES ('00000000-0000-0000-0000-000000000002'::uuid),(NULL)) a(actor_account_id) \
            CROSS JOIN (VALUES ('person'::text),('org_unit'),('other'),(NULL)) o(owner) \
            CROSS JOIN (VALUES ('people.create_person'::text),('organization.create_org_unit'),('organization.revise_org_unit'),('other'),(NULL)) t(target) \
            CROSS JOIN (VALUES ('directory_create'::text),('create_site'),('correct_site_name'),('other'),(NULL)) x(action_key) \
            CROSS JOIN (VALUES ('00000000-0000-0000-0000-000000000003'::uuid),(NULL)) b(object_type_id)"
        );
        let (cases, matches): (i64, bool) = sqlx::query_as(sqlx::AssertSqlSafe(query))
            .fetch_one(connection)
            .await
            .unwrap();
        assert_eq!(cases, 3200);
        assert!(
            matches,
            "CHECK grouping/NULL semantics differ from declared source: {name}"
        );
    }

    async fn actor_declared_schema(connection: &mut PgConnection, validated: bool) -> Value {
        let columns: Value = sqlx::query_scalar(
            "SELECT jsonb_agg(jsonb_build_object('name',a.attname,'number',a.attnum,'type',pg_catalog.format_type(a.atttypid,a.atttypmod), \
              'not_null',a.attnotnull,'default',pg_catalog.pg_get_expr(d.adbin,d.adrelid,false),'comment',pg_catalog.col_description(a.attrelid,a.attnum), \
              'acl',a.attacl,'identity',a.attidentity,'generated',a.attgenerated,'dropped',a.attisdropped) ORDER BY a.attnum) \
             FROM pg_catalog.pg_attribute a LEFT JOIN pg_catalog.pg_attrdef d ON (d.adrelid,d.adnum)=(a.attrelid,a.attnum) \
             WHERE a.attrelid='public.org_unit_revisions'::regclass AND a.attname IN('actor_kind','actor_account_id')")
            .fetch_one(&mut *connection).await.unwrap();
        assert_eq!(
            columns,
            json!([
                {"name":"actor_kind","number":11,"type":"text","not_null":true,"default":"'USER'::text",
                    "comment":"pd:personal — canonical OrgUnit actor attribution protocol","acl":null,"identity":"","generated":"","dropped":false},
                {"name":"actor_account_id","number":12,"type":"uuid","not_null":false,"default":null,
                    "comment":"pd:personal — canonical OrgUnit Account actor identity","acl":null,"identity":"","generated":"","dropped":false}
            ])
        );
        let successors: Vec<(String, String, bool, bool, bool, bool, i32, bool)> = sqlx::query_as(
            "SELECT conname::text,contype::text,convalidated,conenforced,condeferrable,condeferred,coninhcount::integer,conislocal \
             FROM pg_catalog.pg_constraint WHERE (conrelid='public.org_unit_revisions'::regclass \
               AND conname IN('org_unit_revisions_actor_protocol_v1','org_unit_revisions_native_actor_v1')) \
              OR (conrelid='public.ont_action_command_receipts'::regclass AND conname='ont_action_receipts_actor_protocol_v2') \
             ORDER BY conname COLLATE \"C\"")
            .fetch_all(&mut *connection).await.unwrap();
        assert_eq!(
            successors,
            vec![
                (
                    SUCCESSOR_NAMES[0].into(),
                    "c".into(),
                    validated,
                    true,
                    false,
                    false,
                    0,
                    true
                ),
                (
                    SUCCESSOR_NAMES[1].into(),
                    "c".into(),
                    validated,
                    true,
                    false,
                    false,
                    0,
                    true
                ),
                (
                    SUCCESSOR_NAMES[2].into(),
                    "f".into(),
                    validated,
                    true,
                    false,
                    false,
                    0,
                    true
                ),
            ]
        );
        let fk: (String,String,Vec<i16>,Vec<i16>,String,String,String,bool,bool,bool) = sqlx::query_as(
            "SELECT sn.nspname||'.'||s.relname,rn.nspname||'.'||r.relname,conkey,confkey,confupdtype::text,confdeltype::text,confmatchtype::text, \
              conpfeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid,'pg_catalog.=(uuid,uuid)'::regoperator::oid], \
              conindid='public.company_actors_pkey'::regclass,conparentid=0 AND contypid=0 AND NOT conperiod \
             FROM pg_catalog.pg_constraint k JOIN pg_catalog.pg_class s ON s.oid=k.conrelid JOIN pg_catalog.pg_namespace sn ON sn.oid=s.relnamespace \
             JOIN pg_catalog.pg_class r ON r.oid=k.confrelid JOIN pg_catalog.pg_namespace rn ON rn.oid=r.relnamespace \
             WHERE conrelid='public.org_unit_revisions'::regclass AND conname='org_unit_revisions_native_actor_v1'")
            .fetch_one(&mut *connection).await.unwrap();
        assert_eq!(
            fk,
            (
                "public.org_unit_revisions".into(),
                "public.company_actors".into(),
                vec![1, 12],
                vec![1, 2],
                "r".into(),
                "r".into(),
                "s".into(),
                true,
                true,
                true
            )
        );
        let ri: Vec<(String,String,String,i16,bool,bool,bool,bool)> = sqlx::query_as(
            "SELECT c.relname::text,n.nspname::text,p.proname::text,t.tgtype,t.tgisinternal,t.tgenabled='O', \
              NOT t.tgdeferrable AND NOT t.tginitdeferred, \
              t.tgconstrrelid=CASE WHEN t.tgrelid='public.org_unit_revisions'::regclass \
                THEN 'public.company_actors'::regclass ELSE 'public.org_unit_revisions'::regclass END \
             FROM pg_catalog.pg_trigger t JOIN pg_catalog.pg_class c ON c.oid=t.tgrelid \
             JOIN pg_catalog.pg_proc p ON p.oid=t.tgfoid JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace \
             WHERE t.tgconstraint=(SELECT oid FROM pg_catalog.pg_constraint WHERE conrelid='public.org_unit_revisions'::regclass \
               AND conname='org_unit_revisions_native_actor_v1') ORDER BY c.relname COLLATE \"C\",p.proname COLLATE \"C\"")
            .fetch_all(&mut *connection).await.unwrap();
        assert_eq!(
            ri,
            vec![
                (
                    "company_actors".into(),
                    "pg_catalog".into(),
                    "RI_FKey_restrict_del".into(),
                    9,
                    true,
                    true,
                    true,
                    true
                ),
                (
                    "company_actors".into(),
                    "pg_catalog".into(),
                    "RI_FKey_restrict_upd".into(),
                    17,
                    true,
                    true,
                    true,
                    true
                ),
                (
                    "org_unit_revisions".into(),
                    "pg_catalog".into(),
                    "RI_FKey_check_ins".into(),
                    5,
                    true,
                    true,
                    true,
                    true
                ),
                (
                    "org_unit_revisions".into(),
                    "pg_catalog".into(),
                    "RI_FKey_check_upd".into(),
                    17,
                    true,
                    true,
                    true,
                    true
                ),
            ]
        );
        let not_null: Vec<(String,Vec<i16>,bool,bool)> = sqlx::query_as(
            "SELECT conname::text,conkey,convalidated,conenforced FROM pg_catalog.pg_constraint \
             WHERE conrelid='public.org_unit_revisions'::regclass AND contype='n' AND conkey=ARRAY[11]::smallint[]")
            .fetch_all(&mut *connection).await.unwrap();
        assert_eq!(
            not_null,
            vec![(
                "org_unit_revisions_actor_kind_not_null".into(),
                vec![11],
                true,
                true
            )]
        );
        actor_check_semantics(connection, SUCCESSOR_NAMES[1], ACTOR_CHECK).await;
        actor_check_semantics(connection, SUCCESSOR_NAMES[0], RECEIPT_CHECK).await;
        json!({"columns":columns,"successors":successors,"foreign_key":fk,"ri_triggers":ri,"implicit_actor_kind_not_null":not_null,
            "actor_check_truth_cases":3200,"receipt_check_truth_cases":3200})
    }

    fn actor_expansion_catalog_delta(before: &Value, after: &Value) {
        let old_relations = actor_catalog_records(before, "relations", "oid");
        let new_relations = actor_catalog_records(after, "relations", "oid");
        assert_eq!(
            old_relations.len(),
            new_relations.len(),
            "no relation or index may be added"
        );
        let revision_oid = old_relations
            .iter()
            .find(|(_, r)| r["relname"] == "org_unit_revisions")
            .unwrap()
            .0
            .clone();
        for (oid, old) in &old_relations {
            let mut expected = old.clone();
            if old["relname"] == "org_unit_revisions" {
                expected["relnatts"] = json!(old["relnatts"].as_i64().unwrap() + 2);
                expected["relchecks"] = json!(old["relchecks"].as_i64().unwrap() + 1);
            } else if old["relname"] == "ont_action_command_receipts" {
                expected["relchecks"] = json!(old["relchecks"].as_i64().unwrap() + 1);
            }
            assert_eq!(
                new_relations.get(oid),
                Some(&expected),
                "unexpected existing relation delta"
            );
        }
        let old_attributes = before["attributes"].as_array().unwrap();
        let new_attributes = after["attributes"].as_array().unwrap();
        assert_eq!(new_attributes.len(), old_attributes.len() + 2);
        assert!(
            old_attributes
                .iter()
                .all(|old| new_attributes.contains(old)),
            "existing attribute changed"
        );
        let added_attributes: Vec<_> = new_attributes
            .iter()
            .filter(|new| !old_attributes.contains(new))
            .collect();
        assert_eq!(
            added_attributes
                .iter()
                .map(|a| a["attname"].as_str().unwrap())
                .collect::<BTreeSet<_>>(),
            ["actor_kind", "actor_account_id"].into_iter().collect()
        );
        assert!(
            added_attributes
                .iter()
                .all(|a| a["attrelid"].as_str() == Some(revision_oid.as_str()))
        );
        let old_constraints = actor_catalog_records(before, "constraints", "oid");
        let new_constraints = actor_catalog_records(after, "constraints", "oid");
        assert_eq!(
            new_constraints.len(),
            old_constraints.len() + 4,
            "three successors plus implicit NOT NULL required"
        );
        assert!(
            old_constraints
                .iter()
                .all(|(oid, old)| new_constraints.get(oid) == Some(old)),
            "old constraint changed"
        );
        let added: Vec<_> = new_constraints
            .iter()
            .filter(|(oid, _)| !old_constraints.contains_key(*oid))
            .map(|(_, c)| c)
            .collect();
        assert_eq!(
            added
                .iter()
                .map(|c| c["conname"].as_str().unwrap())
                .collect::<BTreeSet<_>>(),
            SUCCESSOR_NAMES
                .into_iter()
                .chain(["org_unit_revisions_actor_kind_not_null"])
                .collect()
        );
        let fk = added
            .iter()
            .find(|c| c["conname"] == "org_unit_revisions_native_actor_v1")
            .unwrap();
        let old_triggers = actor_catalog_records(before, "triggers", "oid");
        let new_triggers = actor_catalog_records(after, "triggers", "oid");
        assert_eq!(
            new_triggers.len(),
            old_triggers.len() + 4,
            "only declared FK RI triggers may be added"
        );
        assert!(
            old_triggers
                .iter()
                .all(|(oid, old)| new_triggers.get(oid) == Some(old)),
            "old guard/trigger changed"
        );
        let added_triggers: Vec<_> = new_triggers
            .iter()
            .filter(|(oid, _)| !old_triggers.contains_key(*oid))
            .map(|(_, t)| t)
            .collect();
        let expected: BTreeSet<_> = [
            ("company_actors", 9),
            ("company_actors", 17),
            ("org_unit_revisions", 5),
            ("org_unit_revisions", 17),
        ]
        .into_iter()
        .collect();
        let actual: BTreeSet<_> = added_triggers
            .iter()
            .map(|t| {
                assert_eq!(t["tgconstraint"], fk["oid"]);
                assert_eq!(t["tgenabled"], "O");
                assert_eq!(t["tgisinternal"], true);
                assert_eq!(t["tgdeferrable"], false);
                assert_eq!(t["tginitdeferred"], false);
                let relation =
                    new_relations.get(t["tgrelid"].as_str().unwrap()).unwrap()["relname"]
                        .as_str()
                        .unwrap();
                (relation, t["tgtype"].as_i64().unwrap())
            })
            .collect();
        assert_eq!(actual, expected);
        let mut old_other = before.clone();
        let mut new_other = after.clone();
        for field in ["relations", "attributes", "constraints", "triggers"] {
            old_other.as_object_mut().unwrap().remove(field);
            new_other.as_object_mut().unwrap().remove(field);
        }
        assert_eq!(
            old_other, new_other,
            "any unrelated security/source/ACL/routine metadata change is refused"
        );
    }

    async fn actor_extra_expansion_delta(
        connection: &mut PgConnection,
        before: &Value,
        after: &Value,
    ) {
        assert_eq!(
            after["types"], before["types"],
            "no type metadata change is declared"
        );
        let old_defaults = before["defaults"].as_array().unwrap();
        let new_defaults = after["defaults"].as_array().unwrap();
        assert_eq!(new_defaults.len(), old_defaults.len() + 1);
        assert!(old_defaults.iter().all(|row| new_defaults.contains(row)));
        let added_default = new_defaults
            .iter()
            .find(|row| !old_defaults.contains(row))
            .unwrap();
        let expected_default: (String,i16,String)=sqlx::query_as(
            "SELECT n.nspname||'.'||c.relname,a.adnum,pg_catalog.pg_get_expr(a.adbin,a.adrelid,false) \
             FROM pg_catalog.pg_attrdef a JOIN pg_catalog.pg_class c ON c.oid=a.adrelid \
             JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE a.oid=$1::text::oid")
            .bind(added_default["oid"].as_str().unwrap()).fetch_one(&mut *connection).await.unwrap();
        assert_eq!(
            expected_default,
            (
                "public.org_unit_revisions".into(),
                11,
                "'USER'::text".into()
            )
        );
        let old_comments = before["comments"].as_array().unwrap();
        let new_comments = after["comments"].as_array().unwrap();
        assert_eq!(new_comments.len(), old_comments.len() + 2);
        assert!(old_comments.iter().all(|row| new_comments.contains(row)));
        let comments: Vec<_> = new_comments
            .iter()
            .filter(|row| !old_comments.contains(row))
            .collect();
        assert_eq!(
            comments
                .iter()
                .map(|c| (
                    c["objsubid"].as_i64().unwrap(),
                    c["description"].as_str().unwrap()
                ))
                .collect::<BTreeSet<_>>(),
            [
                (
                    11,
                    "pd:personal — canonical OrgUnit actor attribution protocol"
                ),
                (12, "pd:personal — canonical OrgUnit Account actor identity")
            ]
            .into_iter()
            .collect()
        );
        let old_dependencies = before["dependencies"].as_array().unwrap();
        let new_dependencies = after["dependencies"].as_array().unwrap();
        assert!(
            old_dependencies
                .iter()
                .all(|row| new_dependencies.contains(row)),
            "old dependency removed or changed"
        );
        // Every new dependent object is one of the declared default, four
        // constraints (including NOT NULL), four RI triggers or two columns.
        let allowed:Vec<(String,String)> = sqlx::query_as(
            "SELECT 'pg_catalog.pg_attrdef'::regclass::oid::text,oid::text FROM pg_catalog.pg_attrdef \
               WHERE adrelid='public.org_unit_revisions'::regclass AND adnum=11 \
             UNION ALL SELECT 'pg_catalog.pg_constraint'::regclass::oid::text,oid::text FROM pg_catalog.pg_constraint \
               WHERE conname IN('org_unit_revisions_actor_kind_not_null','org_unit_revisions_actor_protocol_v1', \
                 'org_unit_revisions_native_actor_v1','ont_action_receipts_actor_protocol_v2') \
                AND conrelid IN('public.org_unit_revisions'::regclass,'public.ont_action_command_receipts'::regclass) \
             UNION ALL SELECT 'pg_catalog.pg_trigger'::regclass::oid::text,oid::text FROM pg_catalog.pg_trigger \
               WHERE tgconstraint=(SELECT oid FROM pg_catalog.pg_constraint WHERE conrelid='public.org_unit_revisions'::regclass \
                 AND conname='org_unit_revisions_native_actor_v1')")
            .fetch_all(&mut *connection).await.unwrap();
        assert_eq!(allowed.len(), 9);
        let class_ids:Vec<(String,String)>=sqlx::query_as(
            "SELECT name,(name::regclass::oid)::text FROM (VALUES ('pg_catalog.pg_attrdef'),('pg_catalog.pg_constraint'),('pg_catalog.pg_trigger'),('pg_catalog.pg_class')) c(name)")
            .fetch_all(&mut *connection).await.unwrap();
        let class_map: BTreeMap<_, _> = class_ids.into_iter().collect();
        let allowed: BTreeSet<_> = allowed.into_iter().collect();
        let revision: String =
            sqlx::query_scalar("SELECT 'public.org_unit_revisions'::regclass::oid::text")
                .fetch_one(&mut *connection)
                .await
                .unwrap();
        for dependency in new_dependencies
            .iter()
            .filter(|row| !old_dependencies.contains(row))
        {
            let object = (
                dependency["classid"].as_str().unwrap().to_owned(),
                dependency["objid"].as_str().unwrap().to_owned(),
            );
            let column = object.0 == class_map["pg_catalog.pg_class"]
                && object.1 == revision
                && matches!(dependency["objsubid"].as_i64(), Some(11 | 12));
            assert!(
                allowed.contains(&object) || column,
                "undeclared dependency addition"
            );
        }
        let changed_relations:Vec<i64>=sqlx::query_scalar(
            "SELECT oid::bigint FROM pg_catalog.pg_class WHERE oid IN \
             ('public.org_unit_revisions'::regclass,'public.ont_action_command_receipts'::regclass)")
            .fetch_all(&mut *connection).await.unwrap();
        assert_eq!(changed_relations.len(), 2);
        let old_tuples = before["tuples"].as_array().unwrap();
        let new_tuples = after["tuples"].as_array().unwrap();
        assert_eq!(
            new_tuples.len(),
            old_tuples.len() + 11,
            "only two attributes/four constraints/four RI triggers/one default may add tuples"
        );
        for old in old_tuples {
            let new = new_tuples
                .iter()
                .find(|new| new[0] == old[0] && new[1] == old[1])
                .unwrap();
            if old != new {
                assert_eq!(old[0], "relation");
                assert!(
                    changed_relations.contains(&old[1].as_i64().unwrap()),
                    "expansion touched unrelated old catalog tuple"
                );
            }
        }
    }

    fn actor_validation_delta(before: &ActorMeasurementBaseline, after: &ActorMeasurementBaseline) {
        let mut expected = before.catalog.1.clone();
        let mut changed = BTreeSet::new();
        for constraint in expected["constraints"].as_array_mut().unwrap() {
            let name = constraint["conname"].as_str().unwrap().to_owned();
            if SUCCESSOR_NAMES.contains(&name.as_str()) {
                assert_eq!(constraint["convalidated"], false);
                constraint["convalidated"] = json!(true);
                changed.insert(name);
            }
        }
        assert_eq!(
            changed,
            SUCCESSOR_NAMES.into_iter().map(str::to_owned).collect()
        );
        assert_eq!(
            after.catalog.1, expected,
            "validation changed more than three convalidated bits"
        );
        for field in ["types", "defaults", "comments", "dependencies"] {
            assert_eq!(after.extra[field], before.extra[field]);
        }
        let changed_oids: BTreeSet<_> = before.catalog.1["constraints"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| SUCCESSOR_NAMES.contains(&c["conname"].as_str().unwrap()))
            .map(|c| c["oid"].as_str().unwrap().parse::<i64>().unwrap())
            .collect();
        let old_tuples = before.extra["tuples"].as_array().unwrap();
        let new_tuples = after.extra["tuples"].as_array().unwrap();
        assert_eq!(old_tuples.len(), new_tuples.len());
        let mut changed_tuples = BTreeSet::new();
        for old in old_tuples {
            let new = new_tuples
                .iter()
                .find(|new| new[0] == old[0] && new[1] == old[1])
                .unwrap();
            if new != old {
                assert_eq!(old[0], "constraint");
                assert!(
                    changed_oids.contains(&old[1].as_i64().unwrap()),
                    "validation touched unrelated catalog tuple"
                );
                changed_tuples.insert(old[1].as_i64().unwrap());
            }
        }
        assert_eq!(
            changed_tuples, changed_oids,
            "each validated constraint must show its real metadata transition"
        );
        assert_eq!(after.rows, before.rows);
        assert_eq!(after.ledger, before.ledger);
        assert_eq!(after.native_census, before.native_census);
        assert_eq!(after.target, before.target);
    }

    async fn actor_dormant_refusals(connection: &mut PgConnection) -> Value {
        let legacy:Uuid=sqlx::query_scalar(
            "SELECT id FROM public.organizations WHERE public.account_company_provenance_v1(id)='LEGACY' ORDER BY id LIMIT 1")
            .fetch_one(&mut *connection).await.expect("actual historical LEGACY Company positive prerequisite required");
        let mut results = Vec::new();
        for (name, query, code, message) in [
            (
                "retained_actor_id_not_null",
                "INSERT INTO public.org_unit_revisions \
              (org_id,id,org_unit_id,version,command_id,actor_id,payload_digest,attributes,receipt,actor_kind,actor_account_id) \
              VALUES ($1,$2,$3,1,$4,NULL,decode(repeat('01',32),'hex'),'{}'::jsonb,'{}'::jsonb,'ACCOUNT',$5)",
                "23502",
                "null value in column \"actor_id\" of relation \"org_unit_revisions\" violates not-null constraint",
            ),
            (
                "retained_people_receipt_frame",
                "INSERT INTO public.ont_action_command_receipts \
              (org_id,command_id,actor_id,payload_digest,receipt,created_at,owner,target,action_key,object_type_id,actor_kind,actor_account_id) \
              VALUES ($1,$2,NULL,decode(repeat('01',32),'hex'),'{\"target\":\"organization.create_org_unit\"}'::jsonb, \
              now(),'org_unit','organization.create_org_unit','create_site',$3,'ACCOUNT',$4)",
                "P0001",
                "people.directory.effect_frame_required",
            ),
        ] {
            let before_rows = rows(connection).await;
            let before_catalog = catalog(connection).await;
            sqlx::raw_sql("SAVEPOINT dormant_actor_refusal")
                .execute(&mut *connection)
                .await
                .unwrap();
            bounds(connection).await;
            let mut statement = sqlx::query(query)
                .bind(legacy)
                .bind(Uuid::new_v4())
                .bind(Uuid::new_v4())
                .bind(Uuid::new_v4());
            if name == "retained_actor_id_not_null" {
                statement = statement.bind(Uuid::new_v4());
            }
            let error = statement
                .execute(&mut *connection)
                .await
                .expect_err("dormant ACCOUNT write must refuse");
            let database = error
                .as_database_error()
                .expect("actual PostgreSQL refusal required");
            assert_eq!(database.code().as_deref(), Some(code));
            assert_eq!(database.message(), message);
            results.push(json!({"control":name,"sqlstate":code,"message":message,"accepted_account_fk_execution":false}));
            sqlx::raw_sql("ROLLBACK TO SAVEPOINT dormant_actor_refusal; RELEASE SAVEPOINT dormant_actor_refusal")
                .execute(&mut *connection).await.unwrap();
            assert_eq!(
                rows(connection).await,
                before_rows,
                "refused probe changed any row"
            );
            assert_eq!(
                catalog(connection).await,
                before_catalog,
                "refused probe changed metadata"
            );
        }
        json!(results)
    }

    async fn actor_unregistered_classifier_refusal(
        connection: &mut PgConnection,
        family: ActorFixtureFamily,
    ) -> Value {
        match family {
            ActorFixtureFamily::Closed76 => {
                let observed = state(connection).await;
                assert_eq!(observed, "native_org_unit.profile_mismatch");
                json!({"historical_closed_classifier":observed,"serving_continuity_accepted":false})
            }
            ActorFixtureFamily::Group83 | ActorFixtureFamily::Navigation83 => {
                let group = group_classification(connection).await;
                let navigation = navigation_classification(connection).await;
                assert_eq!(
                    group,
                    ("native_group_process.profile_mismatch".into(), None)
                );
                assert_eq!(
                    navigation,
                    (
                        "native_group_process_navigation.profile_mismatch".into(),
                        None
                    )
                );
                json!({"historical_group_classifier":group,"historical_navigation_classifier":navigation,
                    "serving_continuity_accepted":false})
            }
        }
    }

    async fn actor_restored(
        pool: &PgPool,
        baseline: &ActorMeasurementBaseline,
        family: ActorFixtureFamily,
        staged: bool,
    ) {
        let mut fresh = direct(pool).await;
        assert_eq!(target(&mut fresh).await, baseline.target);
        let mut readback = sqlx::Connection::begin(&mut fresh).await.unwrap();
        bounds(readback.as_mut()).await;
        let after = actor_snapshot(readback.as_mut(), family, staged).await;
        assert_eq!(after.target, baseline.target);
        assert_eq!(after.capture, baseline.capture);
        assert_eq!(
            after.catalog, baseline.catalog,
            "full raw catalog not restored"
        );
        assert_eq!(
            after.extra, baseline.extra,
            "defaults/comments/dependencies/catalog tuples not restored"
        );
        assert_eq!(
            after.rows, baseline.rows,
            "exact old durable rows not restored"
        );
        assert_eq!(after.ledger, baseline.ledger);
        assert_eq!(after.native_census, baseline.native_census);
        readback.rollback().await.unwrap();
        fresh.close().await.unwrap();
    }

    async fn actor_apply_expansion(
        connection: &mut PgConnection,
        baseline: &ActorMeasurementBaseline,
        family: ActorFixtureFamily,
    ) -> (ActorMeasurementBaseline, Value) {
        let preflight = actor_snapshot(connection, family, false).await;
        assert_eq!(preflight.capture, baseline.capture);
        assert_eq!(preflight.catalog, baseline.catalog);
        assert_eq!(preflight.extra, baseline.extra);
        assert_eq!(preflight.rows, baseline.rows);
        assert_eq!(preflight.ledger, baseline.ledger);
        assert_eq!(preflight.native_census, baseline.native_census);
        let locks = actor_phase_locks(connection, true).await;
        let locked = actor_snapshot(connection, family, false).await;
        assert_eq!(locked.capture, baseline.capture);
        assert_eq!(locked.catalog, baseline.catalog);
        assert_eq!(locked.extra, baseline.extra);
        assert_eq!(locked.rows, baseline.rows);
        assert_eq!(locked.ledger, baseline.ledger);
        assert_eq!(locked.native_census, baseline.native_census);
        // Capture helpers change local settings. Reset all four ceilings last.
        bounds(connection).await;
        let started = Instant::now();
        sqlx::raw_sql(ACTOR_EXPANSION)
            .execute(&mut *connection)
            .await
            .unwrap();
        let elapsed = started.elapsed();
        assert!(elapsed <= Duration::from_secs(60));
        let declaration = actor_declared_schema(connection, false).await;
        sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE")
            .execute(&mut *connection)
            .await
            .unwrap();
        let after = actor_snapshot(connection, family, true).await;
        actor_expansion_catalog_delta(&baseline.catalog.1, &after.catalog.1);
        actor_extra_expansion_delta(connection, &baseline.extra, &after.extra).await;
        assert_eq!(after.rows, baseline.rows);
        assert_eq!(after.ledger, baseline.ledger);
        assert_eq!(after.native_census, baseline.native_census);
        assert_eq!(after.target, baseline.target);
        assert_ne!(
            after.capture.sha256, baseline.capture.sha256,
            "declared actor metadata must alter complete capture"
        );
        let refusals = actor_dormant_refusals(connection).await;
        let unregistered = actor_unregistered_classifier_refusal(connection, family).await;
        let evidence = json!({"explicit_lock_readback":locks,"elapsed_us":elapsed.as_micros(),"declaration":declaration,
            "dormant_refusals":refusals,"unregistered_classifier_refusal":unregistered,"accepted_account_fk_execution":false});
        (after, evidence)
    }

    async fn measure_native_org_unit_account_actor_phase_v1(
        pool: PgPool,
        family: ActorFixtureFamily,
        variant: usize,
        phase: ActorMeasurementPhase,
    ) {
        let sources = actor_phase_pins();
        let original = actor_fixture(&pool, family, variant).await;
        let mut admin = direct(&pool).await;
        let mut expansion = begin_protocol(&mut admin, &original.target).await;
        let expansion_result =
            AssertUnwindSafe(actor_apply_expansion(expansion.as_mut(), &original, family))
                .catch_unwind()
                .await;
        let (staged, expansion_evidence) = match expansion_result {
            Ok(value) => value,
            Err(panic) => {
                expansion.rollback().await.unwrap();
                admin.close().await.unwrap();
                actor_restored(&pool, &original, family, false).await;
                std::panic::resume_unwind(panic)
            }
        };
        let mut packet = json!({"schema":"console.native_org_unit.account_actor_phase_diagnostic.v1",
            "design_sha256":"6bb74306149b7b1790c4d89283edc1d7ed542b8101cbdabfe7b01c65ff86d25a",
            "sources":sources,"family":format!("{family:?}"),"variant":if variant==0 {"plain"}else{"observer"},
            "phase":format!("{phase:?}"),"target":{"database":original.target.database,"database_oid":original.target.database_oid,"system_identifier":original.target.system_identifier},
            "predecessor_capture":original.capture.record(),"staged_capture":staged.capture.record(),
            "predecessor_raw_catalog":original.catalog.1,"staged_raw_catalog":staged.catalog.1,
            "predecessor_raw_catalog_text":original.catalog.0,"staged_raw_catalog_text":staged.catalog.0,
            "predecessor_extra_catalog":original.extra,"staged_extra_catalog":staged.extra,
            "old_column_row_census":row_summary(&original.rows),"native_census":original.native_census,"expansion":expansion_evidence,
            "phase_hashes_registered":false,"business_empty":false,"ui_created_business_fixture":false,
            "serving_startup_accepted":false,"database_owner_accepted":false,"organization_ui_accepted":false,
            "mvp_accepted":false,"production_qualified":false});
        match phase {
            ActorMeasurementPhase::Expansion => {
                expansion.rollback().await.unwrap();
                admin.close().await.unwrap();
                actor_restored(&pool, &original, family, false).await;
                packet["predecessor_rollback_fresh_readback_verified"] = json!(true);
            }
            ActorMeasurementPhase::Validation => {
                // This dedicated diagnostic fixture alone commits the expansion.
                // Its later disposal is NOT a production rollback procedure.
                expansion.commit().await.unwrap();
                admin.close().await.unwrap();
                actor_restored(&pool, &staged, family, true).await;
                let mut fresh = direct(&pool).await;
                let mut validation = begin_protocol(&mut fresh, &staged.target).await;
                let outcome=AssertUnwindSafe(async {
                    let positive=actor_snapshot(validation.as_mut(),family,true).await;
                    assert_eq!(positive.capture,staged.capture); assert_eq!(positive.catalog,staged.catalog);
                    assert_eq!(positive.extra,staged.extra); assert_eq!(positive.rows,staged.rows);
                    assert_eq!(positive.ledger,staged.ledger); assert_eq!(positive.native_census,staged.native_census);
                    actor_declared_schema(validation.as_mut(),false).await;
                    let locks=actor_phase_locks(validation.as_mut(),false).await;
                    let locked=actor_snapshot(validation.as_mut(),family,true).await;
                    assert_eq!(locked.capture,staged.capture); assert_eq!(locked.catalog,staged.catalog);
                    assert_eq!(locked.extra,staged.extra); assert_eq!(locked.rows,staged.rows);
                    assert_eq!(locked.ledger,staged.ledger); assert_eq!(locked.native_census,staged.native_census);
                    actor_declared_schema(validation.as_mut(),false).await;
                    bounds(validation.as_mut()).await; let start=Instant::now();
                    sqlx::raw_sql(ACTOR_VALIDATION).execute(validation.as_mut()).await.unwrap();
                    let elapsed=start.elapsed(); assert!(elapsed<=Duration::from_secs(60));
                    let declared=actor_declared_schema(validation.as_mut(),true).await;
                    sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE").execute(validation.as_mut()).await.unwrap();
                    let after=actor_snapshot(validation.as_mut(),family,true).await;
                    actor_validation_delta(&staged,&after);
                    let refusals=actor_dormant_refusals(validation.as_mut()).await;
                    let unregistered=actor_unregistered_classifier_refusal(validation.as_mut(),family).await;
                    json!({"capture":after.capture.record(),"raw_catalog":after.catalog.1,"raw_catalog_text":after.catalog.0,"extra_catalog":after.extra,
                        "explicit_lock_readback":locks,"elapsed_us":elapsed.as_micros(),"declaration":declared,"dormant_refusals":refusals,
                        "unregistered_classifier_refusal":unregistered})
                }).catch_unwind().await;
                validation.rollback().await.unwrap();
                fresh.close().await.unwrap();
                actor_restored(&pool, &staged, family, true).await;
                packet["validation"] = match outcome {
                    Ok(value) => value,
                    Err(panic) => std::panic::resume_unwind(panic),
                };
                packet["staged_rollback_fresh_readback_verified"] = json!(true);
                packet["disposable_expansion_committed"] = json!(true);
                packet["production_rollback_exercised"] = json!(false);
            }
        }
        let mut output = std::io::stderr().lock();
        writeln!(&mut output, "ORG_ACCOUNT_ACTOR_PHASE_DIAGNOSTIC {packet}")
            .expect("diagnostic evidence write failed");
        output.flush().expect("diagnostic evidence flush failed");
    }

    macro_rules! actor_measurement_leaf {
        ($name:ident,$family:ident,$variant:literal,$phase:ident) => {
            #[sqlx::test(migrations = false)]
            async fn $name(pool: PgPool) {
                measure_native_org_unit_account_actor_phase_v1(
                    pool,
                    ActorFixtureFamily::$family,
                    $variant,
                    ActorMeasurementPhase::$phase,
                )
                .await;
            }
        };
    }
    actor_measurement_leaf!(actor_closed76_plain_expansion, Closed76, 0, Expansion);
    actor_measurement_leaf!(actor_closed76_observer_expansion, Closed76, 1, Expansion);
    actor_measurement_leaf!(actor_group83_plain_expansion, Group83, 0, Expansion);
    actor_measurement_leaf!(actor_group83_observer_expansion, Group83, 1, Expansion);
    actor_measurement_leaf!(
        actor_navigation83_plain_expansion,
        Navigation83,
        0,
        Expansion
    );
    actor_measurement_leaf!(
        actor_navigation83_observer_expansion,
        Navigation83,
        1,
        Expansion
    );
    actor_measurement_leaf!(actor_closed76_plain_validation, Closed76, 0, Validation);
    actor_measurement_leaf!(actor_closed76_observer_validation, Closed76, 1, Validation);
    actor_measurement_leaf!(actor_group83_plain_validation, Group83, 0, Validation);
    actor_measurement_leaf!(actor_group83_observer_validation, Group83, 1, Validation);
    actor_measurement_leaf!(
        actor_navigation83_plain_validation,
        Navigation83,
        0,
        Validation
    );
    actor_measurement_leaf!(
        actor_navigation83_observer_validation,
        Navigation83,
        1,
        Validation
    );
}
