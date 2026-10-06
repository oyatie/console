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
        let comparison_started = Instant::now();
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
        eprintln!(
            "ORG_ACCOUNT_ACTOR_COMPARISON {}",
            json!({
                "schema":"console.native_org_unit.account_actor_comparison_observation.v1",
                "function":"actor_expansion_catalog_delta","region":"complete_catalog_comparison",
                "elapsed_us":comparison_started.elapsed().as_micros(),
                "old_relations":before["relations"].as_array().unwrap().len(),
                "new_relations":after["relations"].as_array().unwrap().len(),
                "old_attributes":old_attributes.len(),"new_attributes":new_attributes.len(),
                "old_constraints":before["constraints"].as_array().unwrap().len(),
                "new_constraints":after["constraints"].as_array().unwrap().len(),
                "old_triggers":before["triggers"].as_array().unwrap().len(),
                "new_triggers":after["triggers"].as_array().unwrap().len()
            })
        );
    }

    async fn actor_extra_expansion_delta(
        connection: &mut PgConnection,
        before: &Value,
        after: &Value,
    ) {
        let defaults_started = Instant::now();
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
        // Emit before the next SQL call, including on an idle-expiry failure.
        eprintln!(
            "ORG_ACCOUNT_ACTOR_COMPARISON {}",
            json!({
                "schema":"console.native_org_unit.account_actor_comparison_observation.v1",
                "function":"actor_extra_expansion_delta","region":"defaults_before_sql",
                "elapsed_us":defaults_started.elapsed().as_micros(),
                "old_defaults":old_defaults.len(),"new_defaults":new_defaults.len()
            })
        );
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
        let membership_started = Instant::now();
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
        eprintln!(
            "ORG_ACCOUNT_ACTOR_COMPARISON {}",
            json!({
                "schema":"console.native_org_unit.account_actor_comparison_observation.v1",
                "function":"actor_extra_expansion_delta","region":"comments_dependencies_before_sql",
                "elapsed_us":membership_started.elapsed().as_micros(),
                "old_comments":old_comments.len(),"new_comments":new_comments.len(),
                "old_dependencies":old_dependencies.len(),"new_dependencies":new_dependencies.len()
            })
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
        let additions_started = Instant::now();
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
        eprintln!(
            "ORG_ACCOUNT_ACTOR_COMPARISON {}",
            json!({
                "schema":"console.native_org_unit.account_actor_comparison_observation.v1",
                "function":"actor_extra_expansion_delta","region":"dependency_additions_before_sql",
                "elapsed_us":additions_started.elapsed().as_micros(),
                "old_dependencies":old_dependencies.len(),"new_dependencies":new_dependencies.len()
            })
        );
        let changed_relations:Vec<i64>=sqlx::query_scalar(
            "SELECT oid::bigint FROM pg_catalog.pg_class WHERE oid IN \
             ('public.org_unit_revisions'::regclass,'public.ont_action_command_receipts'::regclass)")
            .fetch_all(&mut *connection).await.unwrap();
        assert_eq!(changed_relations.len(), 2);
        let tuples_started = Instant::now();
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
        eprintln!(
            "ORG_ACCOUNT_ACTOR_COMPARISON {}",
            json!({
                "schema":"console.native_org_unit.account_actor_comparison_observation.v1",
                "function":"actor_extra_expansion_delta","region":"complete_tuple_comparison",
                "elapsed_us":tuples_started.elapsed().as_micros(),
                "old_tuples":old_tuples.len(),"new_tuples":new_tuples.len()
            })
        );
    }

    fn actor_validation_delta(before: &ActorMeasurementBaseline, after: &ActorMeasurementBaseline) {
        let comparison_started = Instant::now();
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
        eprintln!(
            "ORG_ACCOUNT_ACTOR_COMPARISON {}",
            json!({
                "schema":"console.native_org_unit.account_actor_comparison_observation.v1",
                "function":"actor_validation_delta","region":"complete_validation_comparison",
                "elapsed_us":comparison_started.elapsed().as_micros(),
                "old_constraints":before.catalog.1["constraints"].as_array().unwrap().len(),
                "new_constraints":after.catalog.1["constraints"].as_array().unwrap().len(),
                "old_tuples":old_tuples.len(),"new_tuples":new_tuples.len()
            })
        );
    }

    // Pure comparator inputs below are independent assertion controls, not
    // database/business fixtures or custody evidence. Every case calls the
    // existing comparison boundary; no replacement index is tested here.
    #[test]
    fn actor_expansion_catalog_membership_controls() {
        let before = json!({
            "relations": [
                {"oid":"1","relname":"org_unit_revisions","relnatts":10,"relchecks":0},
                {"oid":"2","relname":"ont_action_command_receipts","relnatts":13,"relchecks":1},
                {"oid":"3","relname":"company_actors","relnatts":4,"relchecks":0}
            ],
            "attributes": [{"attrelid":"1","attname":"kept","attnum":1,
                "atttypid":"25","metadata":{"left":1,"right":2}}],
            "constraints": [{"oid":"10","conname":"kept_constraint","convalidated":true}],
            "triggers": [{"oid":"20","tgname":"kept_trigger","tgenabled":"O"}],
            "unrelated_metadata": {"source":"retained","acl":["retained"]}
        });
        let after = json!({
            "relations": [
                {"oid":"1","relname":"org_unit_revisions","relnatts":12,"relchecks":1},
                {"oid":"2","relname":"ont_action_command_receipts","relnatts":13,"relchecks":2},
                {"oid":"3","relname":"company_actors","relnatts":4,"relchecks":0}
            ],
            "attributes": [
                {"attrelid":"1","attname":"kept","attnum":1,"atttypid":"25",
                    "metadata":{"left":1,"right":2}},
                {"attrelid":"1","attname":"actor_kind","attnum":11,"atttypid":"25"},
                {"attrelid":"1","attname":"actor_account_id","attnum":12,"atttypid":"2950"}
            ],
            "constraints": [
                {"oid":"10","conname":"kept_constraint","convalidated":true},
                {"oid":"11","conname":"ont_action_receipts_actor_protocol_v2","convalidated":false},
                {"oid":"12","conname":"org_unit_revisions_actor_protocol_v1","convalidated":false},
                {"oid":"13","conname":"org_unit_revisions_native_actor_v1","convalidated":false},
                {"oid":"14","conname":"org_unit_revisions_actor_kind_not_null","convalidated":true}
            ],
            "triggers": [
                {"oid":"20","tgname":"kept_trigger","tgenabled":"O"},
                {"oid":"21","tgconstraint":"13","tgenabled":"O","tgisinternal":true,
                    "tgdeferrable":false,"tginitdeferred":false,"tgrelid":"3","tgtype":9},
                {"oid":"22","tgconstraint":"13","tgenabled":"O","tgisinternal":true,
                    "tgdeferrable":false,"tginitdeferred":false,"tgrelid":"3","tgtype":17},
                {"oid":"23","tgconstraint":"13","tgenabled":"O","tgisinternal":true,
                    "tgdeferrable":false,"tginitdeferred":false,"tgrelid":"1","tgtype":5},
                {"oid":"24","tgconstraint":"13","tgenabled":"O","tgisinternal":true,
                    "tgdeferrable":false,"tginitdeferred":false,"tgrelid":"1","tgtype":17}
            ],
            "unrelated_metadata": {"source":"retained","acl":["retained"]}
        });
        let accepts = |old: &Value, new: &Value| {
            std::panic::catch_unwind(|| actor_expansion_catalog_delta(old, new)).is_ok()
        };
        assert!(
            accepts(&before, &after),
            "independent allowed catalog control"
        );

        let mut reordered = after.clone();
        reordered["attributes"].as_array_mut().unwrap().reverse();
        assert!(
            accepts(&before, &reordered),
            "attribute iteration order is not identity"
        );
        let mut key_order = after.clone();
        key_order["attributes"][0] = serde_json::from_str(
            r#"{"metadata":{"right":2,"left":1},"atttypid":"25","attnum":1,"attname":"kept","attrelid":"1"}"#,
        ).unwrap();
        assert!(
            accepts(&before, &key_order),
            "full Value equality ignores object key order"
        );

        let mut duplicate_before = before.clone();
        let mut duplicate_after = after.clone();
        duplicate_before["attributes"]
            .as_array_mut()
            .unwrap()
            .push(before["attributes"][0].clone());
        duplicate_after["attributes"]
            .as_array_mut()
            .unwrap()
            .push(before["attributes"][0].clone());
        assert!(
            accepts(&duplicate_before, &duplicate_after),
            "raw duplicate counts remain +2"
        );
        duplicate_after["attributes"].as_array_mut().unwrap().pop();
        assert!(
            !accepts(&duplicate_before, &duplicate_after),
            "set cardinality cannot replace raw count"
        );

        let mut distinct_before = before.clone();
        let mut distinct_after = after.clone();
        let mut same_selectors = before["attributes"][0].clone();
        same_selectors["metadata"]["left"] = json!(9);
        distinct_before["attributes"]
            .as_array_mut()
            .unwrap()
            .push(same_selectors.clone());
        distinct_after["attributes"]
            .as_array_mut()
            .unwrap()
            .push(same_selectors);
        assert!(
            accepts(&distinct_before, &distinct_after),
            "same identity fields do not deduplicate full rows"
        );
        distinct_after["attributes"].as_array_mut().unwrap().pop();
        distinct_after["attributes"]
            .as_array_mut()
            .unwrap()
            .push(before["attributes"][0].clone());
        assert!(
            !accepts(&distinct_before, &distinct_after),
            "changed full row is not retained by partial equality"
        );

        for (label, field, replacement) in [
            (
                "attribute_metadata",
                "attributes",
                json!({"attrelid":"1","attname":"kept","attnum":1,
                "atttypid":"25","metadata":{"left":9,"right":2}}),
            ),
            (
                "relation_metadata",
                "relations",
                json!({"oid":"1","relname":"org_unit_revisions","relnatts":12,"relchecks":2}),
            ),
            (
                "constraint_metadata",
                "constraints",
                json!({"oid":"10","conname":"kept_constraint","convalidated":false}),
            ),
            (
                "trigger_metadata",
                "triggers",
                json!({"oid":"20","tgname":"kept_trigger","tgenabled":"D"}),
            ),
        ] {
            let mut changed = after.clone();
            changed[field][0] = replacement;
            assert!(
                !accepts(&before, &changed),
                "full catalog corruption must refuse: {label}"
            );
        }
        let mut missing = after.clone();
        missing["attributes"][0]["attname"] = json!("missing_kept_attribute");
        assert!(
            !accepts(&before, &missing),
            "missing original full attribute refuses"
        );
        let mut unrelated = after.clone();
        unrelated["unrelated_metadata"]["acl"] = json!(["broader"]);
        assert!(
            !accepts(&before, &unrelated),
            "unrelated catalog metadata remains exact"
        );
    }

    #[test]
    fn actor_validation_tuple_controls() {
        // Equality sentinels only: no database, row writer or custody claim.
        let before = ActorMeasurementBaseline {
            target: Target {
                database: "pure-comparison-control".into(),
                database_oid: 1,
                system_identifier: "1".into(),
            },
            capture: Capture {
                text: String::new(),
                sha256: String::new(),
                rights: false,
                snapshot: Value::Null,
            },
            catalog: (
                String::new(),
                json!({"constraints":[
                    {"oid":"101","conname":"ont_action_receipts_actor_protocol_v2","convalidated":false},
                    {"oid":"102","conname":"org_unit_revisions_actor_protocol_v1","convalidated":false},
                    {"oid":"103","conname":"org_unit_revisions_native_actor_v1","convalidated":false}
                ]}),
            ),
            extra: json!({"types":[],"defaults":[],"comments":[],"dependencies":[],"tuples":[
                ["constraint",101,"old-a","(1,1)"],
                ["constraint",102,"old-b","(1,2)"],
                ["constraint",103,"old-c","(1,3)"],
                ["attribute",201,"retained","(2,1)"]
            ]}),
            rows: BTreeMap::new(),
            ledger: json!({"retained":true}),
            native_census: json!({"retained":true}),
        };
        let mut after = before.clone();
        after.catalog.1 = json!({"constraints":[
            {"oid":"101","conname":"ont_action_receipts_actor_protocol_v2","convalidated":true},
            {"oid":"102","conname":"org_unit_revisions_actor_protocol_v1","convalidated":true},
            {"oid":"103","conname":"org_unit_revisions_native_actor_v1","convalidated":true}
        ]});
        after.extra["tuples"] = json!([
            ["constraint", 101, "new-a", "(3,1)"],
            ["constraint", 102, "new-b", "(3,2)"],
            ["constraint", 103, "new-c", "(3,3)"],
            ["attribute", 201, "retained", "(2,1)"]
        ]);
        let accepts = |old: &ActorMeasurementBaseline, new: &ActorMeasurementBaseline| {
            std::panic::catch_unwind(|| actor_validation_delta(old, new)).is_ok()
        };
        assert!(
            accepts(&before, &after),
            "three complete constraint-tuple transitions"
        );
        let mut reordered = after.clone();
        reordered.extra["tuples"].as_array_mut().unwrap().reverse();
        assert!(
            accepts(&before, &reordered),
            "lookup retains original old-array iteration"
        );
        for slot in [2, 3] {
            let mut changed = after.clone();
            changed.extra["tuples"][3][slot] = json!("corrupted");
            assert!(
                !accepts(&before, &changed),
                "unchanged tuple field {slot} cannot be omitted from equality"
            );
        }
        let mut missing = after.clone();
        missing.extra["tuples"][3][1] = json!(999);
        assert!(!accepts(&before, &missing), "missing exact key refuses");
        let mut extra = after.clone();
        extra.extra["tuples"]
            .as_array_mut()
            .unwrap()
            .push(json!(["unused", 900, "x", "y"]));
        assert!(!accepts(&before, &extra), "raw tuple counts remain equal");
        let mut no_transition = after.clone();
        no_transition.extra["tuples"][1] = before.extra["tuples"][1].clone();
        assert!(
            !accepts(&before, &no_transition),
            "each validated constraint requires a changed tuple"
        );

        let mut duplicate_before = before.clone();
        let mut duplicate_after = after.clone();
        duplicate_before.extra["tuples"]
            .as_array_mut()
            .unwrap()
            .push(before.extra["tuples"][3].clone());
        let bad_duplicate = json!(["attribute", 201, "corrupted", "(2,1)"]);
        duplicate_after.extra["tuples"]
            .as_array_mut()
            .unwrap()
            .push(bad_duplicate.clone());
        assert!(
            accepts(&duplicate_before, &duplicate_after),
            "first matching duplicate wins over later corruption"
        );
        duplicate_after.extra["tuples"][3] = bad_duplicate;
        duplicate_after.extra["tuples"][4] = before.extra["tuples"][3].clone();
        assert!(
            !accepts(&duplicate_before, &duplicate_after),
            "first corruption refuses despite later equal duplicate"
        );

        let distinct = json!([
            ["ab","c","first","(4,1)"], ["a","bc","second","(4,2)"],
            ["attribute",1,"number","(4,3)"], ["attribute","1","string","(4,4)"],
            [{"left":1,"right":2},{"key":1},"object","(4,5)"],
            ["relation",1,"same_oid_other_kind","(4,6)"]
        ]);
        let mut distinct_before = before.clone();
        let mut distinct_after = after.clone();
        distinct_before.extra["tuples"]
            .as_array_mut()
            .unwrap()
            .extend(distinct.as_array().unwrap().iter().cloned());
        distinct_after.extra["tuples"]
            .as_array_mut()
            .unwrap()
            .extend(distinct.as_array().unwrap().iter().cloned());
        distinct_after.extra["tuples"][8][0] =
            serde_json::from_str(r#"{"right":2,"left":1}"#).unwrap();
        assert!(
            accepts(&distinct_before, &distinct_after),
            "pair boundaries, JSON kinds and object key order stay exact"
        );
        distinct_after.extra["tuples"][5][2] = json!("corrupted");
        assert!(
            !accepts(&distinct_before, &distinct_after),
            "distinct-key corruption is not hidden by collapsed keys"
        );

        // Pinned serde_json has distinct Eq for integer/float zero, but the
        // numeric hash input is zero for both. This proves the collision witness;
        // verdicts below still invoke the real tuple comparator, not an index.
        use std::hash::{Hash as _, Hasher as _};
        let integer_key = json!(["numeric_hash_collision", 0]);
        let float_key = json!(["numeric_hash_collision", 0.0]);
        assert_ne!(integer_key, float_key);
        let mut integer_hash = std::collections::hash_map::DefaultHasher::new();
        let mut float_hash = std::collections::hash_map::DefaultHasher::new();
        (&integer_key[0], &integer_key[1]).hash(&mut integer_hash);
        (&float_key[0], &float_key[1]).hash(&mut float_hash);
        assert_eq!(integer_hash.finish(), float_hash.finish());
        let collisions = json!([
            ["numeric_hash_collision", 0, "integer", "(5,1)"],
            ["numeric_hash_collision", 0.0, "float", "(5,2)"]
        ]);
        let mut collision_before = before.clone();
        let mut collision_after = after.clone();
        collision_before.extra["tuples"]
            .as_array_mut()
            .unwrap()
            .extend(collisions.as_array().unwrap().iter().cloned());
        collision_after.extra["tuples"]
            .as_array_mut()
            .unwrap()
            .extend(collisions.as_array().unwrap().iter().cloned());
        assert!(
            accepts(&collision_before, &collision_after),
            "hash collisions retain distinct full Value keys"
        );
        collision_after.extra["tuples"][5][2] = json!("corrupted");
        assert!(
            !accepts(&collision_before, &collision_after),
            "a colliding-key full-row corruption still refuses"
        );

        for malformed in [json!(null), json!({}), json!([]), json!([null])] {
            let mut old = before.clone();
            let mut new = after.clone();
            old.extra["tuples"]
                .as_array_mut()
                .unwrap()
                .push(malformed.clone());
            new.extra["tuples"]
                .as_array_mut()
                .unwrap()
                .push(malformed.clone());
            assert!(
                accepts(&old, &new),
                "existing Null selectors accept a fully equal malformed row: {malformed}"
            );
            let other = if malformed == json!(null) {
                json!([])
            } else {
                json!(null)
            };
            new.extra["tuples"][4] = other;
            assert!(
                !accepts(&old, &new),
                "same Null selectors do not authorize unequal full rows"
            );
        }
    }

    #[sqlx::test(migrations = false)]
    async fn actor_extra_expansion_membership_and_tuple_controls(pool: PgPool) {
        // Reuse the retained disposable fixture and the existing owner sequence.
        // Existing twelve full-catalog journeys and all their oracles stay intact.
        // This compact control leaf does not qualify those journeys or startup.
        #[derive(Clone, Copy, Debug)]
        enum ExpectedRefusal {
            ConditionAssertion,
            EqualityAssertion,
            MissingTuple,
            Exact(&'static str),
        }
        let matches_refusal = |payload: &(dyn std::any::Any + Send), expected: ExpectedRefusal| {
            let message = payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&str>().copied());
            let Some(message) = message else {
                return false;
            };
            // A failed query/client Result is never a comparator refusal, even
            // if its error text embeds a valid assertion/custom-message prefix.
            if message.contains("called `Result::unwrap()`") {
                return false;
            }
            match expected {
                ExpectedRefusal::ConditionAssertion => message.starts_with("assertion failed: "),
                ExpectedRefusal::EqualityAssertion => {
                    message.starts_with("assertion `left == right` failed\n")
                        || message.starts_with("assertion `left == right` failed: ")
                }
                ExpectedRefusal::MissingTuple => {
                    message == "called `Option::unwrap()` on a `None` value"
                }
                ExpectedRefusal::Exact(expected) => message == expected,
            }
        };
        let classifier_controls: Vec<(&str, Box<dyn std::any::Any + Send>, ExpectedRefusal, bool)> = vec![
            (
                "condition_string",
                Box::new(String::from("assertion failed: retained_membership")),
                ExpectedRefusal::ConditionAssertion,
                true,
            ),
            (
                "condition_str",
                Box::new("assertion failed: retained_membership"),
                ExpectedRefusal::ConditionAssertion,
                true,
            ),
            (
                "equality",
                Box::new("assertion `left == right` failed: retained tuple\n  left: 1\n right: 2"),
                ExpectedRefusal::EqualityAssertion,
                true,
            ),
            (
                "missing_tuple",
                Box::new("called `Option::unwrap()` on a `None` value"),
                ExpectedRefusal::MissingTuple,
                true,
            ),
            (
                "retained_dependency",
                Box::new("old dependency removed or changed"),
                ExpectedRefusal::Exact("old dependency removed or changed"),
                true,
            ),
            (
                "undeclared_dependency",
                Box::new(String::from("undeclared dependency addition")),
                ExpectedRefusal::Exact("undeclared dependency addition"),
                true,
            ),
            (
                "different_custom_assertion",
                Box::new("undeclared dependency addition"),
                ExpectedRefusal::Exact("old dependency removed or changed"),
                false,
            ),
            (
                "result_embedded_condition",
                Box::new(
                    "called `Result::unwrap()` on an `Err` value: Database(assertion failed: injected)",
                ),
                ExpectedRefusal::ConditionAssertion,
                false,
            ),
            (
                "result_embedded_equality",
                Box::new(
                    "called `Result::unwrap()` on an `Err` value: Database(assertion `left == right` failed)",
                ),
                ExpectedRefusal::EqualityAssertion,
                false,
            ),
            (
                "result_embedded_option",
                Box::new(
                    "called `Result::unwrap()` on an `Err` value: called `Option::unwrap()` on a `None` value",
                ),
                ExpectedRefusal::MissingTuple,
                false,
            ),
            (
                "result_embedded_custom",
                Box::new(
                    "called `Result::unwrap()` on an `Err` value: old dependency removed or changed",
                ),
                ExpectedRefusal::Exact("old dependency removed or changed"),
                false,
            ),
            (
                "assertion_embedded_result",
                Box::new("assertion failed: called `Result::unwrap()` on an `Err` value: Client"),
                ExpectedRefusal::ConditionAssertion,
                false,
            ),
            (
                "client_embedded_assertion",
                Box::new("client error: assertion failed: injected"),
                ExpectedRefusal::ConditionAssertion,
                false,
            ),
            (
                "sql_embedded_assertion",
                Box::new("SQL error: assertion `left == right` failed"),
                ExpectedRefusal::EqualityAssertion,
                false,
            ),
            (
                "unknown_text",
                Box::new("unknown failure with assertion text"),
                ExpectedRefusal::ConditionAssertion,
                false,
            ),
            (
                "unknown_payload",
                Box::new(17u64),
                ExpectedRefusal::ConditionAssertion,
                false,
            ),
            (
                "option_trailing_error",
                Box::new("called `Option::unwrap()` on a `None` value: SQL/client failure"),
                ExpectedRefusal::MissingTuple,
                false,
            ),
            (
                "wrong_assertion_category",
                Box::new("assertion failed: retained_membership"),
                ExpectedRefusal::EqualityAssertion,
                false,
            ),
            (
                "invalid_equality_prefix",
                Box::new("assertion `left == right` failedUnexpected SQL/client text"),
                ExpectedRefusal::EqualityAssertion,
                false,
            ),
        ];
        let classifier_control_count = classifier_controls.len();
        for (label, payload, expected, accepted) in classifier_controls {
            assert_eq!(
                matches_refusal(payload.as_ref(), expected),
                accepted,
                "refusal classifier control: {label}"
            );
        }
        let family = ActorFixtureFamily::Closed76;
        let original = actor_fixture(&pool, family, 0).await;
        let mut admin = direct(&pool).await;
        let mut transaction = begin_protocol(&mut admin, &original.target).await;
        let outcome = AssertUnwindSafe(async {
            actor_phase_locks(transaction.as_mut(), true).await;
            bounds(transaction.as_mut()).await;
            let started = Instant::now();
            sqlx::raw_sql(ACTOR_EXPANSION).execute(transaction.as_mut()).await.unwrap();
            assert!(started.elapsed() <= Duration::from_secs(60));
            actor_declared_schema(transaction.as_mut(), false).await;
            sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE").execute(transaction.as_mut()).await.unwrap();
            let staged = actor_extra(transaction.as_mut()).await;
            let revision_oid = original.catalog.1["relations"].as_array().unwrap().iter()
                .find(|row| row["relname"] == "org_unit_revisions").unwrap()["oid"].clone();
            let default = staged["defaults"].as_array().unwrap().iter()
                .find(|row| row["adrelid"] == revision_oid && row["adnum"] == 11).unwrap().clone();
            let retained_default = original.extra["defaults"].as_array().unwrap().first().unwrap().clone();
            assert_ne!(retained_default["oid"], default["oid"]);
            let retained_comment = original.extra["comments"].as_array().unwrap().first().unwrap().clone();
            let comments: Vec<_> = staged["comments"].as_array().unwrap().iter()
                .filter(|row| row["objoid"] == revision_oid && matches!(row["objsubid"].as_i64(), Some(11 | 12)))
                .cloned().collect();
            assert_eq!(comments.len(), 2);
            let retained_dependency = original.extra["dependencies"].as_array().unwrap().first().unwrap().clone();
            let added_dependency = staged["dependencies"].as_array().unwrap().iter()
                .find(|row| row["objid"] == default["oid"]).unwrap().clone();
            let before = json!({"types":[],"defaults":[retained_default],"comments":[retained_comment],
                "dependencies":[retained_dependency],"tuples":[["attribute",7,"retained","(1,1)"]]});
            let mut tuples = vec![json!(["attribute",7,"retained","(1,1)"])];
            tuples.extend((0..11).map(|n| json!(["unselected_addition",n,"added","(2,1)"])));
            let after = json!({"types":[],"defaults":[retained_default,default],
                "comments":[retained_comment,comments[0],comments[1]],
                "dependencies":[retained_dependency,added_dependency],"tuples":tuples});
            let mut controls = vec![("allowed_actual_owner_metadata", before.clone(), after.clone(), None)];

            let mut reordered = after.clone();
            for field in ["defaults","comments","dependencies","tuples"] {
                reordered[field].as_array_mut().unwrap().reverse();
            }
            controls.push(("array_reordering", before.clone(), reordered, None));
            let mut object_order = after.clone();
            for field in ["defaults","comments","dependencies"] {
                let object = object_order[field][0].as_object().unwrap();
                let mut reordered_object = serde_json::Map::new();
                for (key, value) in object.iter().rev() { reordered_object.insert(key.clone(), value.clone()); }
                object_order[field][0] = Value::Object(reordered_object);
            }
            controls.push(("full_row_object_order", before.clone(), object_order, None));
            for field in ["defaults","comments","dependencies"] {
                let mut changed = after.clone();
                changed[field][0]["retained_full_row_control"] = json!("corruption");
                controls.push((match field { "defaults" => "default_full_row", "comments" => "comment_full_row", _ => "dependency_full_row" },
                    before.clone(), changed, Some(match field {
                        "defaults" | "comments" => ExpectedRefusal::ConditionAssertion,
                        _ => ExpectedRefusal::Exact("old dependency removed or changed"),
                    })));
                let mut old = before.clone();
                let mut new = after.clone();
                old[field].as_array_mut().unwrap().push(before[field][0].clone());
                new[field].as_array_mut().unwrap().push(before[field][0].clone());
                controls.push((match field { "defaults" => "default_raw_duplicates", "comments" => "comment_raw_duplicates", _ => "dependency_raw_duplicates" }, old, new, None));
            }
            for field in ["defaults","comments","tuples"] {
                let mut changed = after.clone();
                changed[field].as_array_mut().unwrap().pop();
                controls.push((match field { "defaults" => "default_raw_count", "comments" => "comment_raw_count", _ => "tuple_raw_count" },
                    before.clone(), changed, Some(ExpectedRefusal::EqualityAssertion)));
            }
            let mut wrong_default = retained_default.clone();
            wrong_default["distinct_full_row_control"] = json!(true);
            let mut duplicate_default_before = before.clone();
            duplicate_default_before["defaults"] = json!([retained_default,retained_default]);
            let mut default_first = after.clone();
            default_first["defaults"] = json!([retained_default,default,wrong_default]);
            controls.push(("first_added_default_selected", duplicate_default_before.clone(), default_first.clone(), None));
            default_first["defaults"] = json!([retained_default,wrong_default,default]);
            controls.push(("first_wrong_default_refuses", duplicate_default_before, default_first, Some(ExpectedRefusal::EqualityAssertion)));
            let mut comment_changed = after.clone();
            comment_changed["comments"][1]["description"] = json!("undeclared comment");
            controls.push(("declared_comment_value", before.clone(), comment_changed, Some(ExpectedRefusal::EqualityAssertion)));
            let mut forbidden_dependency = after.clone();
            forbidden_dependency["dependencies"][1]["objid"] = json!("0");
            controls.push(("undeclared_dependency_object", before.clone(), forbidden_dependency,
                Some(ExpectedRefusal::Exact("undeclared dependency addition"))));

            for slot in [2,3] {
                let mut changed = after.clone();
                changed["tuples"][0][slot] = json!("corrupted");
                controls.push((if slot == 2 { "tuple_xmin" } else { "tuple_ctid" }, before.clone(), changed, Some(ExpectedRefusal::EqualityAssertion)));
            }
            let mut missing = after.clone();
            missing["tuples"][0][1] = json!(999);
            controls.push(("missing_tuple_match", before.clone(), missing, Some(ExpectedRefusal::MissingTuple)));
            let bad = json!(["attribute",7,"corrupted","(1,1)"]);
            let mut duplicate = after.clone();
            duplicate["tuples"][1] = bad.clone();
            controls.push(("first_matching_duplicate_selected", before.clone(), duplicate.clone(), None));
            duplicate["tuples"][0] = bad;
            duplicate["tuples"][1] = before["tuples"][0].clone();
            controls.push(("first_matching_corruption_refuses", before.clone(), duplicate, Some(ExpectedRefusal::EqualityAssertion)));
            let mut duplicate_tuple_before = before.clone();
            let mut duplicate_tuple_after = after.clone();
            duplicate_tuple_before["tuples"].as_array_mut().unwrap().push(before["tuples"][0].clone());
            duplicate_tuple_after["tuples"].as_array_mut().unwrap().push(before["tuples"][0].clone());
            controls.push(("tuple_raw_duplicates", duplicate_tuple_before, duplicate_tuple_after, None));

            let distinct = json!([
                ["ab","c","first","(4,1)"], ["a","bc","second","(4,2)"],
                ["attribute",1,"number","(4,3)"], ["attribute","1","string","(4,4)"],
                [{"left":1,"right":2},{"key":1},"object","(4,5)"],
                ["relation",1,"same_oid_other_kind","(4,6)"]
            ]);
            let mut distinct_before = before.clone();
            let mut distinct_after = after.clone();
            distinct_before["tuples"].as_array_mut().unwrap().extend(distinct.as_array().unwrap().iter().cloned());
            distinct_after["tuples"].as_array_mut().unwrap().extend(distinct.as_array().unwrap().iter().cloned());
            distinct_after["tuples"][16][0] = serde_json::from_str(r#"{"right":2,"left":1}"#).unwrap();
            controls.push(("pair_boundaries_json_kinds_object_order", distinct_before.clone(), distinct_after.clone(), None));
            distinct_after["tuples"][13][2] = json!("corrupted");
            controls.push(("distinct_key_corruption", distinct_before, distinct_after, Some(ExpectedRefusal::EqualityAssertion)));
            use std::hash::{Hash as _, Hasher as _};
            let integer_key = json!(["numeric_hash_collision",0]);
            let float_key = json!(["numeric_hash_collision",0.0]);
            assert_ne!(integer_key, float_key);
            let mut integer_hash = std::collections::hash_map::DefaultHasher::new();
            let mut float_hash = std::collections::hash_map::DefaultHasher::new();
            (&integer_key[0], &integer_key[1]).hash(&mut integer_hash);
            (&float_key[0], &float_key[1]).hash(&mut float_hash);
            assert_eq!(integer_hash.finish(), float_hash.finish());
            let collisions = json!([
                ["numeric_hash_collision",0,"integer","(5,1)"],
                ["numeric_hash_collision",0.0,"float","(5,2)"]
            ]);
            let mut collision_before = before.clone();
            let mut collision_after = after.clone();
            collision_before["tuples"].as_array_mut().unwrap().extend(collisions.as_array().unwrap().iter().cloned());
            collision_after["tuples"].as_array_mut().unwrap().extend(collisions.as_array().unwrap().iter().cloned());
            controls.push(("hash_collision_distinct_keys", collision_before.clone(), collision_after.clone(), None));
            collision_after["tuples"][13][2] = json!("corrupted");
            controls.push(("hash_collision_full_row_corruption", collision_before, collision_after, Some(ExpectedRefusal::EqualityAssertion)));
            for malformed in [json!(null),json!({}),json!([]),json!([null])] {
                let mut old = before.clone();
                let mut new = after.clone();
                old["tuples"].as_array_mut().unwrap().push(malformed.clone());
                new["tuples"].as_array_mut().unwrap().push(malformed.clone());
                controls.push(("equal_malformed_null_selectors", old.clone(), new.clone(), None));
                new["tuples"][12] = if malformed == json!(null) { json!([]) } else { json!(null) };
                controls.push(("unequal_malformed_null_selectors", old, new, Some(ExpectedRefusal::EqualityAssertion)));
            }
            let control_count = controls.len();
            for (label, old, new, expected) in controls {
                // Positive calls bracket each corruption: a query/session error
                // cannot masquerade as a semantic refusal or poison later cases.
                actor_extra_expansion_delta(transaction.as_mut(), &before, &after).await;
                let observed = AssertUnwindSafe(actor_extra_expansion_delta(transaction.as_mut(), &old, &new))
                    .catch_unwind().await;
                assert_eq!(observed.is_ok(), expected.is_none(), "frozen extra-expansion outcome: {label}");
                if let Some(expected) = expected {
                    assert!(matches_refusal(observed.as_ref().err().unwrap().as_ref(), expected),
                        "exact comparator refusal required for {label}: {expected:?}; SQL/client/unknown failure is not proof");
                }
                actor_extra_expansion_delta(transaction.as_mut(), &before, &after).await;
            }
            eprintln!("ORG_ACCOUNT_ACTOR_COMPARISON_CONTROLS {}", json!({
                "schema":"console.native_org_unit.account_actor_comparison_controls.v1",
                "function":"actor_extra_expansion_delta","controls_executed":control_count,
                "classifier_controls_executed":classifier_control_count,
                "business_fixture_created":false,"complete_capture_accepted":false
            }));
        }).catch_unwind().await;
        transaction.rollback().await.unwrap();
        admin.close().await.unwrap();
        actor_restored(&pool, &original, family, false).await;
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
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
