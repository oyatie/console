//! Actual V4 source prerequisite for the existing manager owner leaf.
//! Root installs the reviewed production successor before this probe. No installer here.
use super::{all_rows, credentials};
use console_app::AppConfig;
use console_identity_application::company_policy::{
    CompanyProjectionRow, CurrentCompanyAuthority, InitialCompanyAction,
    workflow::NativePolicyCommandRef,
};
use console_platform_auth::account::{
    AccountEnrollmentCredentials, ensure_account_session_fresh_in_tx,
};
use futures::FutureExt;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{Connection, PgConnection, PgPool, Postgres, Transaction};
use std::{collections::BTreeMap, time::Duration};
use time::OffsetDateTime;
use uuid::Uuid;

async fn source(
    tx: &mut Transaction<'_, Postgres>,
    actor: Uuid,
    family: Uuid,
    company: Uuid,
    command: Uuid,
    group: Option<Uuid>,
) -> Result<Vec<(Value, OffsetDateTime)>, sqlx::Error> {
    sqlx::query_as("SELECT to_jsonb(s),s.observed_at FROM public.identity_company_information_manager_current_v1($1,$2,$3,$4,$5) s")
        .bind(actor).bind(family).bind(company).bind(command).bind(group)
        .fetch_all(tx.as_mut()).await
}

fn substantive(mut row: Value) -> Value {
    assert_eq!(
        row.as_object().unwrap().len(),
        20,
        "actual Manager output column count"
    );
    assert!(row.as_object_mut().unwrap().remove("observed_at").is_some());
    row
}

fn exact_error(error: sqlx::Error, code: &str, message: &str) {
    let database = error
        .as_database_error()
        .expect("source failure must reach PostgreSQL");
    assert_eq!(database.code().as_deref(), Some(code));
    assert_eq!(database.message(), message);
}

// Full effective trigger roster (including internal/deferred triggers), routines,
// CHECK/FK and index definitions, relation/column ACL and RLS, not statistics.
async fn fault_metadata(connection: &mut PgConnection) -> Value {
    sqlx::query_scalar(r#"
WITH relations AS (
    SELECT * FROM pg_catalog.pg_class WHERE oid IN (
        'public.policy_capability_clause_fields'::regclass,
        'public.company_enrollment_request_events'::regclass,
        'public.company_actors'::regclass)
), triggers AS (
    SELECT t.* FROM pg_catalog.pg_trigger t JOIN relations r ON r.oid=t.tgrelid
), routines AS (
    SELECT p.* FROM pg_catalog.pg_proc p WHERE p.oid IN (SELECT tgfoid FROM triggers)
       OR p.oid IN (
        'public.identity_company_information_manager_current_v1(uuid,uuid,uuid,uuid,uuid)'::regprocedure,
        'public.identity_company_information_group_lock_v1(uuid,uuid)'::regprocedure,
        'public.identity_company_information_selected_lock_v1(uuid,uuid)'::regprocedure,
        'public.identity_company_information_root_material_v1(uuid,uuid)'::regprocedure)
)
SELECT jsonb_build_object(
 'triggers', (SELECT jsonb_agg(to_jsonb(t)||jsonb_build_object('definition',pg_catalog.pg_get_triggerdef(t.oid,false)) ORDER BY t.oid) FROM triggers t),
 'routines', (SELECT jsonb_agg(to_jsonb(p)||jsonb_build_object('definition',pg_catalog.pg_get_functiondef(p.oid)) ORDER BY p.oid) FROM routines p),
 'relations', (SELECT jsonb_agg(jsonb_build_object('oid',r.oid,'name',r.relname,'namespace',r.relnamespace,'owner',r.relowner,'type',r.reltype,'kind',r.relkind,'persistence',r.relpersistence,'checks',r.relchecks,'triggers',r.relhastriggers,'rules',r.relhasrules,'children',r.relhassubclass,'acl',r.relacl,'options',r.reloptions,'rls',r.relrowsecurity,'force_rls',r.relforcerowsecurity,'partition',r.relispartition,'partition_bound',r.relpartbound,'access_method',r.relam) ORDER BY r.oid) FROM relations r),
 'columns', (SELECT jsonb_agg(to_jsonb(a) ORDER BY a.attrelid,a.attnum) FROM pg_catalog.pg_attribute a JOIN relations r ON r.oid=a.attrelid WHERE a.attnum>0),
 'constraints', (SELECT jsonb_agg(to_jsonb(c)||jsonb_build_object('definition',pg_catalog.pg_get_constraintdef(c.oid,false)) ORDER BY c.oid) FROM pg_catalog.pg_constraint c WHERE c.conrelid IN (SELECT oid FROM relations) OR c.confrelid IN (SELECT oid FROM relations)),
 'indexes', (SELECT jsonb_agg(to_jsonb(i)||jsonb_build_object('definition',pg_catalog.pg_get_indexdef(i.indexrelid)) ORDER BY i.indexrelid) FROM pg_catalog.pg_index i WHERE i.indrelid IN (SELECT oid FROM relations)),
 'policies', (SELECT jsonb_agg(to_jsonb(p) ORDER BY p.oid) FROM pg_catalog.pg_policy p JOIN relations r ON r.oid=p.polrelid),
 'inheritance', (SELECT jsonb_agg(to_jsonb(i) ORDER BY i.inhrelid,i.inhseqno) FROM pg_catalog.pg_inherits i WHERE i.inhrelid IN (SELECT oid FROM relations) OR i.inhparent IN (SELECT oid FROM relations)),
 'schemas', (SELECT jsonb_agg(to_jsonb(n) ORDER BY n.oid) FROM pg_catalog.pg_namespace n WHERE n.oid IN (SELECT relnamespace FROM relations) OR n.oid IN (SELECT pronamespace FROM routines)))
"#).fetch_one(connection).await.expect("STOP: complete fault guard metadata capture")
}

async fn fault_settings(connection: &mut PgConnection) -> Value {
    sqlx::query_scalar("SELECT jsonb_build_object('session_user',session_user,'current_user',current_user,'current_org',coalesce(current_setting('app.current_org',true),''),'settings',(SELECT jsonb_object_agg(name,setting ORDER BY name) FROM pg_catalog.pg_settings))")
        .fetch_one(connection).await.expect("STOP: original role/session settings capture")
}

fn ordinary_guard(mut original: Value, selected: &str) -> Value {
    let mut changed = 0;
    for trigger in original["triggers"].as_array_mut().unwrap() {
        if trigger["tgname"] == selected {
            assert_eq!(
                trigger["tgenabled"], "A",
                "STOP: selected guard is not original ALWAYS"
            );
            trigger["tgenabled"] = json!("O");
            changed += 1;
        }
    }
    assert_eq!(changed, 1, "STOP: exact selected guard identity");
    original
}

async fn fault_backend_absent(pool: &PgPool, pid: Option<i32>, marker: &str) {
    // Read-only observation uses a separate direct socket with a server timeout;
    // its failure also closes that socket before reporting STOP.
    let options = pool
        .connect_options()
        .as_ref()
        .clone()
        .options([("statement_timeout", "1s")]);
    let mut checker =
        tokio::time::timeout(Duration::from_secs(5), PgConnection::connect_with(&options))
            .await
            .unwrap()
            .unwrap();
    let absent=std::panic::AssertUnwindSafe(tokio::time::timeout(Duration::from_secs(5),async {
        loop {
            let gone: bool=sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity WHERE pid=$1 OR application_name=$2) AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_locks WHERE pid=$1) AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity WHERE datname=current_database() AND $1=ANY(pg_catalog.pg_blocking_pids(pid)))")
                .bind(pid.unwrap_or(-1)).bind(marker).fetch_one(&mut checker).await.unwrap();
            if gone { break; }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })).catch_unwind().await;
    let closed = tokio::time::timeout(Duration::from_secs(5), checker.close()).await;
    assert!(
        matches!(absent, Ok(Ok(()))) && matches!(closed, Ok(Ok(()))),
        "STOP: marked operator backend/locks/dependent waiters or observer socket remain"
    );
}

pub(super) async fn prerequisite(
    pool: &PgPool,
    runtime: &PgPool,
    config: &AppConfig,
    reads: [&AccountEnrollmentCredentials; 3],
    authority: &CurrentCompanyAuthority,
    selector: NativePolicyCommandRef,
    group: Uuid,
    before: &BTreeMap<String, String>,
) {
    // Preserve pin_profile; bind the independently accepted V4 source bytes separately.
    for (sql, digest) in [
        (
            include_str!("../../../../ops/native-company-information/group-lock-v1.sql"),
            "daee9a6d7f2e0b1e8992c327500f0e93a9641501c467bdaddac3bfaa19fac397",
        ),
        (
            include_str!("../../../../ops/native-company-information/selected-lock-v1.sql"),
            "f60cc97a92df9ca446dc4bec2964981a3957cf18b4d81fd60f9c32826065598f",
        ),
        (
            include_str!("../../../../ops/native-company-information/root-material-v1.sql"),
            "f75d12df521b2d0ec6c69271ff6dfac4663e207890af0ccdeefdd39763dacbc6",
        ),
        (
            include_str!("../../../../ops/native-company-information/manager-current-v1.sql"),
            "607635bd51942c6c04f1fbdbe39c7a8ab414c9c6f19af94632a2f92e40980265",
        ),
        (
            include_str!("../../../../ops/native-company-information/acl-v1.sql"),
            "1e74e8bb66e9f61242c3ef79c122f99c8953a4495ba03ab56ac1d9121cd81ade",
        ),
    ] {
        assert_eq!(
            hex::encode(Sha256::digest(sql.as_bytes())),
            digest,
            "V4 source drift is not owner RED"
        );
    }
    let abi: Value = sqlx::query_scalar("SELECT jsonb_agg(jsonb_build_array(a.name,t.typname) ORDER BY a.ordinal) FROM pg_catalog.pg_proc p CROSS JOIN LATERAL unnest(p.proargnames,p.proargmodes,p.proallargtypes) WITH ORDINALITY a(name,mode,type_oid,ordinal) JOIN pg_catalog.pg_type t ON t.oid=a.type_oid WHERE p.oid='public.identity_company_information_manager_current_v1(uuid,uuid,uuid,uuid,uuid)'::regprocedure AND a.mode IN ('o','t')")
        .fetch_one(runtime).await.expect("installed production Manager ABI prerequisite");
    assert_eq!(
        abi,
        json!([
            ["actor_account_id", "uuid"],
            ["session_id", "uuid"],
            ["org_id", "uuid"],
            ["command_id", "uuid"],
            ["current_group_id", "uuid"],
            ["company_epoch", "int8"],
            ["current_policy_receipt_id", "uuid"],
            ["context_generation", "int8"],
            ["assignment_id", "uuid"],
            ["assignment_revision", "int8"],
            ["role_id", "uuid"],
            ["role_revision", "int8"],
            ["registered_clauses", "jsonb"],
            ["company_name", "text"],
            ["company_slug", "text"],
            ["installed_object_type_id", "uuid"],
            ["observed_at", "timestamptz"],
            ["source_xid", "xid8"],
            ["source_backend_pid", "int4"],
            ["source_material", "jsonb"]
        ])
    );
    let rights: (bool, bool, bool, bool) = sqlx::query_as("SELECT has_function_privilege(current_user,'public.identity_company_information_manager_current_v1(uuid,uuid,uuid,uuid,uuid)','EXECUTE'),has_function_privilege(current_user,'public.identity_company_information_group_lock_v1(uuid,uuid)','EXECUTE'),has_function_privilege(current_user,'public.identity_company_information_selected_lock_v1(uuid,uuid)','EXECUTE'),has_function_privilege(current_user,'public.identity_company_information_root_material_v1(uuid,uuid)','EXECUTE')")
        .fetch_one(runtime).await.unwrap();
    assert_eq!(
        rights,
        (true, false, false, false),
        "genuine console_rt finite source rights"
    );
    let (verifier, _, ttl) = credentials::bindings(config);
    let company = *selector.company().as_uuid();
    let command = selector.command_id();
    let [a, b, o] = reads;
    let mut original = None;
    for (credential, positive) in [(a, true), (o, false), (b, false)] {
        let mut tx = runtime.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL statement_timeout='10s'; SET LOCAL lock_timeout='1s'")
            .execute(tx.as_mut()).await.unwrap();
        let (actor, family) = credential
            .session_ids_in_tx(&mut tx, &verifier, ttl)
            .await
            .unwrap();
        let prior_org: String =
            sqlx::query_scalar("SELECT coalesce(current_setting('app.current_org',true),'')")
                .fetch_one(tx.as_mut())
                .await
                .unwrap();
        let rows = source(&mut tx, actor, family, company, command, None)
            .await
            .expect("healthy actual Manager source prerequisite; SQL failure is not owner RED");
        if positive {
            let [(row, observed)] = rows.as_slice() else {
                panic!("actual A source must return exactly one row")
            };
            let ids = [
                ("actor_account_id", actor),
                ("session_id", family),
                ("org_id", company),
                ("command_id", command),
                ("current_group_id", group),
                ("assignment_id", authority.assignment_id()),
                ("role_id", authority.role_id()),
            ];
            for (key, value) in ids {
                assert_eq!(row[key], json!(value));
            }
            assert_eq!(row["company_epoch"], json!(authority.epoch()));
            assert_eq!(
                row["context_generation"],
                json!(authority.context_generation())
            );
            assert_eq!(row["assignment_revision"], json!(1));
            assert_eq!(row["role_revision"], json!(1));
            assert_eq!(
                row["current_policy_receipt_id"],
                json!(authority.current_policy_receipt_id())
            );
            assert_eq!(row["company_name"], json!(authority.name()));
            assert_eq!(row["company_slug"], json!(authority.slug()));
            let workspace = authority
                .clauses()
                .iter()
                .find(|c| !c.delegable() && c.action_kind() == InitialCompanyAction::Discover)
                .unwrap()
                .action()
                .object_type_id();
            assert_eq!(row["installed_object_type_id"], json!(workspace));
            let (xid, pid): (String, i32) =
                sqlx::query_as("SELECT pg_current_xact_id()::text,pg_backend_pid()")
                    .fetch_one(tx.as_mut())
                    .await
                    .unwrap();
            assert_eq!(row["source_xid"], json!(xid));
            assert_eq!(row["source_backend_pid"], json!(pid));
            assert_eq!(
                row["source_material"]["kind"],
                "COMPANY_INFORMATION_MANAGER_CURRENT_SOURCE_V1"
            );
            assert_eq!(
                row["source_material"]["request"],
                json!({"codec_version":4,"operation":"Grant","account_id":actor,"session_id":family,"org_id":company,"command_id":command})
            );
            let parsed = CurrentCompanyAuthority::from_current_projection(
                authority.account(),
                authority.company(),
                *observed,
                CompanyProjectionRow {
                    company_epoch: row["company_epoch"].as_i64().unwrap(),
                    context_generation: row["context_generation"].as_i64().unwrap(),
                    assignment_id: authority.assignment_id(),
                    assignment_revision: 1,
                    role_id: authority.role_id(),
                    role_revision: 1,
                    registered_clauses: row["registered_clauses"].to_string(),
                    company_name: row["company_name"].as_str().unwrap().to_owned(),
                    company_slug: row["company_slug"].as_str().unwrap().to_owned(),
                },
                authority.current_policy_receipt_id(),
            )
            .expect("unchanged birth7/16 parser on actual Manager rows");
            assert_eq!(parsed.clauses(), authority.clauses());
            let final_rows = source(&mut tx, actor, family, company, command, Some(group))
                .await
                .unwrap();
            let [(final_row, final_observed)] = final_rows.as_slice() else {
                panic!("retained A source disappeared")
            };
            assert!(*final_observed >= *observed);
            assert_eq!(
                substantive(row.clone()),
                substantive(final_row.clone()),
                "actual source changed inside retained scope"
            );
            original = Some((actor, family));
        } else {
            assert!(rows.is_empty(), "O/B acquired A manager source");
        }
        let restored_org: String =
            sqlx::query_scalar("SELECT coalesce(current_setting('app.current_org',true),'')")
                .fetch_one(tx.as_mut())
                .await
                .unwrap();
        assert_eq!(restored_org, prior_org, "Manager source leaked current_org");
        let session = credential
            .read_session_in_tx(&mut tx, &verifier, ttl)
            .await
            .unwrap();
        assert_eq!((session.account_id, session.session_id), (actor, family));
        ensure_account_session_fresh_in_tx(&mut tx, &session)
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }
    let (actor, family) = original.unwrap();
    // Same genuine runtime LOGIN: healthy absence, planned-Group refusal and helpers.
    let mut tx = runtime.begin().await.unwrap();
    sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
        .execute(tx.as_mut())
        .await
        .unwrap();
    assert!(
        source(&mut tx, actor, family, Uuid::new_v4(), command, None)
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        source(&mut tx, actor, family, company, command, None)
            .await
            .unwrap()
            .len(),
        1
    );
    for (selected_command, expected_group, code, message) in [
        (
            command,
            Some(Uuid::nil()),
            "P0001",
            "company_information.material_unavailable",
        ),
        (
            command,
            Some(Uuid::new_v4()),
            "40001",
            "company_information.lock_plan_changed",
        ),
        (
            Uuid::nil(),
            None,
            "P0001",
            "company_information.material_unavailable",
        ),
    ] {
        sqlx::raw_sql("SAVEPOINT manager_parameter")
            .execute(tx.as_mut())
            .await
            .unwrap();
        exact_error(
            source(
                &mut tx,
                actor,
                family,
                company,
                selected_command,
                expected_group,
            )
            .await
            .unwrap_err(),
            code,
            message,
        );
        sqlx::raw_sql(
            "ROLLBACK TO SAVEPOINT manager_parameter; RELEASE SAVEPOINT manager_parameter",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();
        assert_eq!(
            source(&mut tx, actor, family, company, command, Some(group))
                .await
                .unwrap()
                .len(),
            1
        );
    }
    for sql in [
        "SELECT * FROM public.identity_company_information_group_lock_v1($1,$2)",
        "SELECT * FROM public.identity_company_information_selected_lock_v1($1,$2)",
        "SELECT * FROM public.identity_company_information_root_material_v1($1,$2)",
    ] {
        sqlx::raw_sql("SAVEPOINT manager_helper")
            .execute(tx.as_mut())
            .await
            .unwrap();
        let error = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(company)
            .bind(group)
            .fetch_all(tx.as_mut())
            .await
            .unwrap_err();
        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some("42501"),
            "helper must fail at privilege boundary"
        );
        sqlx::raw_sql("ROLLBACK TO SAVEPOINT manager_helper; RELEASE SAVEPOINT manager_helper")
            .execute(tx.as_mut())
            .await
            .unwrap();
        assert_eq!(
            source(&mut tx, actor, family, company, command, Some(group))
                .await
                .unwrap()
                .len(),
            1
        );
    }
    let session = a.read_session_in_tx(&mut tx, &verifier, ttl).await.unwrap();
    ensure_account_session_fresh_in_tx(&mut tx, &session)
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    // Root-adopted bounded fault transport only; no fixture birth DML in this
    // fresh operator transaction. Restore ALWAYS and origin before real source.
    let marker = format!("manager-source-fault-{}", Uuid::new_v4().simple());
    let options = pool
        .connect_options()
        .as_ref()
        .clone()
        .application_name(&marker)
        .options([
            ("statement_timeout", "5s"),
            ("lock_timeout", "500ms"),
            ("idle_in_transaction_session_timeout", "10s"),
            ("transaction_timeout", "60s"),
        ]);
    let mut connection =
        tokio::time::timeout(Duration::from_secs(5), PgConnection::connect_with(&options))
            .await
            .unwrap()
            .unwrap();
    let mut pid = None;
    let mut outer_metadata = None;
    let mut outer_settings = None;
    // All setup after this owned socket exists belongs to the caught/bounded
    // scope. External whole-leaf cancellation belongs to root's container supervisor.
    let outcome=std::panic::AssertUnwindSafe(tokio::time::timeout(Duration::from_secs(45),async {
    pid=Some(sqlx::query_scalar::<_,i32>("SELECT pg_backend_pid()").fetch_one(&mut connection).await.unwrap());
    let metadata_before=fault_metadata(&mut connection).await;
    outer_metadata=Some(metadata_before.clone());
    let settings_before=fault_settings(&mut connection).await;
    outer_settings=Some(settings_before.clone());
    assert_eq!(settings_before["settings"]["session_replication_role"],"origin","STOP: finite original operator replication role");
    let mut tx=connection.begin().await.unwrap();
    sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL statement_timeout='5s'; SET LOCAL lock_timeout='500ms'; SET LOCAL idle_in_transaction_session_timeout='10s'")
        .execute(tx.as_mut()).await.unwrap();
    let marked: String=sqlx::query_scalar("SELECT set_config('application_name',$1,true)").bind(&marker).fetch_one(tx.as_mut()).await.unwrap();
    assert_eq!(marked,marker,"STOP: isolated operator transaction mark");
    assert!(sqlx::query_scalar::<_,bool>("SELECT pg_current_xact_id_if_assigned() IS NULL").fetch_one(tx.as_mut()).await.unwrap(),"STOP: fault transaction must begin without birth DML or pending events");
    let metadata=fault_metadata(tx.as_mut()).await;
    assert_eq!(metadata,metadata_before,"STOP: initial guard metadata changed");
    let roster: Vec<(String,String,String,i16,String)>=sqlx::query_as("SELECT c.relname::text,t.tgname::text,p.proname::text,t.tgtype,t.tgenabled::text FROM pg_catalog.pg_trigger t JOIN pg_catalog.pg_class c ON c.oid=t.tgrelid JOIN pg_catalog.pg_proc p ON p.oid=t.tgfoid WHERE (t.tgrelid,t.tgname) IN (('public.policy_capability_clause_fields'::regclass,'policy_capability_clause_fields_immutable_v1'),('public.company_enrollment_request_events'::regclass,'company_enrollment_event_immutable_v1'),('public.company_actors'::regclass,'company_actor_birth_guard_v1')) ORDER BY c.relname")
        .fetch_all(tx.as_mut()).await.unwrap();
    assert_eq!(roster,vec![
        ("company_actors".into(),"company_actor_birth_guard_v1".into(),"identity_company_actor_birth_guard_v1".into(),31,"A".into()),
        ("company_enrollment_request_events".into(),"company_enrollment_event_immutable_v1".into(),"company_enrollment_event_immutable_v1".into(),58,"A".into()),
        ("policy_capability_clause_fields".into(),"policy_capability_clause_fields_immutable_v1".into(),"identity_native_immutable_v1".into(),58,"A".into()),
    ],"STOP: exact effective three-guard UPDATE roster");
    let settings=fault_settings(tx.as_mut()).await;
    let operator: bool = sqlx::query_scalar("SELECT session_user=current_user AND current_user='console_buck_admin' AND starts_with(current_database(),'_sqlx_test_') AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' AND r.rolsuper FROM pg_catalog.pg_roles r WHERE r.rolname=session_user")
        .fetch_one(tx.as_mut()).await.unwrap();
    assert!(
        operator,
        "corruption transport needs isolated test operator"
    );
    sqlx::raw_sql("SET LOCAL ROLE console_rt")
        .execute(tx.as_mut())
        .await
        .unwrap();
    assert_eq!(
        source(&mut tx, actor, family, company, command, None)
            .await
            .unwrap()
            .len(),
        1
    );
    sqlx::raw_sql("RESET ROLE")
        .execute(tx.as_mut())
        .await
        .unwrap();
    let field = authority
        .clauses()
        .iter()
        .find(|c| !c.delegable() && c.action_kind() == InitialCompanyAction::ReadPolicy)
        .unwrap()
        .fields()[0]
        .property_id();
    for (label, sql) in [
        (
            "field content",
            "UPDATE public.policy_capability_clause_fields SET content_digest=decode(repeat('00',32),'hex') WHERE org_id=$1 AND role_id=$2 AND role_revision=1 AND clause_index=3 AND property_id=$3 AND content_digest<>decode(repeat('00',32),'hex')",
        ),
        (
            "field manifest",
            "UPDATE public.policy_capability_clause_fields SET manifest_digest=decode(repeat('00',32),'hex') WHERE org_id=$1 AND role_id=$2 AND role_revision=1 AND clause_index=3 AND property_id=$3 AND manifest_digest<>decode(repeat('00',32),'hex')",
        ),
        (
            "PREPARED time",
            "UPDATE public.company_enrollment_request_events e SET occurred_at=e.occurred_at+interval '1 microsecond' FROM public.company_enrollment_receipts r WHERE r.org_id=$1 AND e.account_id=r.account_id AND e.command_id=r.command_id AND e.event_revision=1 AND $2::uuid IS NOT NULL AND $3::uuid IS NOT NULL",
        ),
        (
            "actor attribution time",
            "UPDATE public.company_actors SET created_at=created_at+interval '1 microsecond' WHERE org_id=$1 AND account_id=$2 AND $3::uuid IS NOT NULL",
        ),
    ] {
        let (guard,ordinary,always)=match label {
            "field content" | "field manifest" => (
                "policy_capability_clause_fields_immutable_v1",
                "ALTER TABLE ONLY public.policy_capability_clause_fields ENABLE TRIGGER policy_capability_clause_fields_immutable_v1",
                "ALTER TABLE ONLY public.policy_capability_clause_fields ENABLE ALWAYS TRIGGER policy_capability_clause_fields_immutable_v1"),
            "PREPARED time" => (
                "company_enrollment_event_immutable_v1",
                "ALTER TABLE ONLY public.company_enrollment_request_events ENABLE TRIGGER company_enrollment_event_immutable_v1",
                "ALTER TABLE ONLY public.company_enrollment_request_events ENABLE ALWAYS TRIGGER company_enrollment_event_immutable_v1"),
            "actor attribution time" => (
                "company_actor_birth_guard_v1",
                "ALTER TABLE ONLY public.company_actors ENABLE TRIGGER company_actor_birth_guard_v1",
                "ALTER TABLE ONLY public.company_actors ENABLE ALWAYS TRIGGER company_actor_birth_guard_v1"),
            _ => panic!("STOP: unreviewed fault guard label"),
        };
        assert_eq!(fault_metadata(tx.as_mut()).await,metadata,"STOP: pre-fault full metadata");
        assert_eq!(fault_settings(tx.as_mut()).await,settings,"STOP: pre-fault operator settings");
        sqlx::raw_sql("SAVEPOINT manager_fault; SET LOCAL session_replication_role='replica'")
            .execute(tx.as_mut())
            .await
            .unwrap();
        sqlx::raw_sql(ordinary).execute(tx.as_mut()).await.expect("STOP: selected A-to-O guard transport");
        assert_eq!(fault_metadata(tx.as_mut()).await,ordinary_guard(metadata.clone(),guard),"STOP: only selected guard may become ordinary");
        let mut replica_settings=settings.clone();
        replica_settings["settings"]["session_replication_role"]=json!("replica");
        assert_eq!(fault_settings(tx.as_mut()).await,replica_settings,"STOP: only replica setting may change for UPDATE");
        let second = if label == "actor attribution time" {
            actor
        } else {
            authority.role_id()
        };
        let changed = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(company)
            .bind(second)
            .bind(field)
            .execute(tx.as_mut())
            .await
            .unwrap_or_else(|error| {
                panic!("{label}: injection failed before actual source: {error}")
            });
        assert_eq!(
            changed.rows_affected(),
            1,
            "{label}: exact real source-row calibration"
        );
        sqlx::raw_sql(always).execute(tx.as_mut()).await.expect("STOP: restore ALWAYS before actual source; pending events cannot be bypassed");
        sqlx::raw_sql("SET LOCAL session_replication_role='origin'").execute(tx.as_mut()).await.unwrap();
        assert_eq!(fault_metadata(tx.as_mut()).await,metadata,"STOP: exact full guard metadata before source");
        assert_eq!(fault_settings(tx.as_mut()).await,settings,"STOP: original operator settings before source");
        sqlx::raw_sql("SET LOCAL ROLE console_rt")
            .execute(tx.as_mut())
            .await
            .unwrap();
        exact_error(
            source(&mut tx, actor, family, company, command, Some(group))
                .await
                .unwrap_err(),
            "P0001",
            "company_information.material_unavailable",
        );
        sqlx::raw_sql("ROLLBACK TO SAVEPOINT manager_fault; RELEASE SAVEPOINT manager_fault; RESET ROLE").execute(tx.as_mut()).await.unwrap();
        assert_eq!(fault_metadata(tx.as_mut()).await,metadata,"STOP: full metadata after savepoint rollback");
        assert_eq!(fault_settings(tx.as_mut()).await,settings,"STOP: original settings after savepoint rollback");
        sqlx::raw_sql("SET LOCAL ROLE console_rt").execute(tx.as_mut()).await.unwrap();
        assert_eq!(
            source(&mut tx, actor, family, company, command, Some(group))
                .await
                .unwrap()
                .len(),
            1,
            "{label}: restored actual-source positive"
        );
        sqlx::raw_sql("RESET ROLE")
            .execute(tx.as_mut())
            .await
            .unwrap();
    }
    tx.rollback().await.unwrap();
    })).catch_unwind().await;
    // Attempt rollback, full restoration, connection close, and backend absence
    // before propagating any panic/timeout. No cleanup failure can become owner RED.
    let rolled_back = tokio::time::timeout(
        Duration::from_secs(5),
        sqlx::raw_sql("ROLLBACK").execute(&mut connection),
    )
    .await;
    let restored =
        std::panic::AssertUnwindSafe(tokio::time::timeout(Duration::from_secs(5), async {
            if let (Some(metadata), Some(settings)) = (&outer_metadata, &outer_settings) {
                assert_eq!(
                    fault_metadata(&mut connection).await,
                    *metadata,
                    "STOP: full metadata after outer rollback"
                );
                assert_eq!(
                    fault_settings(&mut connection).await,
                    *settings,
                    "STOP: original settings after outer rollback"
                );
            }
        }))
        .catch_unwind()
        .await;
    let closed = tokio::time::timeout(Duration::from_secs(5), connection.close()).await;
    let absent = std::panic::AssertUnwindSafe(fault_backend_absent(pool, pid, &marker))
        .catch_unwind()
        .await;
    assert!(
        matches!(rolled_back, Ok(Ok(_)))
            && matches!(restored, Ok(Ok(())))
            && matches!(closed, Ok(Ok(())))
            && absent.is_ok(),
        "STOP: exact marked operator rollback/metadata/settings/backend/lock cleanup not proven"
    );
    match outcome {
        Ok(Ok(())) => {}
        Ok(Err(_)) => {
            panic!("STOP: bounded fault transport timed out before actual source completion")
        }
        Err(panic) => std::panic::resume_unwind(panic),
    }
    let mut tx = runtime.begin().await.unwrap();
    sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
        .execute(tx.as_mut())
        .await
        .unwrap();
    assert_eq!(
        source(&mut tx, actor, family, company, command, None)
            .await
            .unwrap()
            .len(),
        1,
        "genuine LOGIN positive after all fault rollbacks"
    );
    let session = a.read_session_in_tx(&mut tx, &verifier, ttl).await.unwrap();
    ensure_account_session_fresh_in_tx(&mut tx, &session)
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    assert!(
        *before == all_rows(pool).await,
        "source prerequisites/fault restoration changed any public table"
    );
}
