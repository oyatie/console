#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Each exact SQLx case requires a fresh owned recovery-supervisor invocation.
//! Initial observer custody precedes the real app database ownership handoff.

use super::{AppConfig, AppError, AppRole, AppState, DatabaseDependency, build_router};
use axum::body::{Body, to_bytes};
use console_kernel_core::{OrgId, UserId, WorkOrderId};
use console_platform_auth::{AccessTokenInput, JwtIssuer, JwtSettings};
use console_platform_request_context::scope_org;
use console_platform_test_support::{
    TestDatabaseLogin, finalize_account_custody, login_test_database_url, login_test_pool,
    prepare_test_migration_owner_url, seed_org_and_super_admin, seed_org_rls_off,
};
use console_workflow_runtime_adapter_postgres::PgWorkflowRuntimeStore;
use http::{Request, StatusCode, header};
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use p256::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use std::time::Duration;
use time::OffsetDateTime;
use tower::ServiceExt;
use uuid::Uuid;

const ISSUER: &str = "console-composition-durability";
const AUDIENCE: &str = "console-api";

fn control(action: &str) {
    let status = std::process::Command::new(
        std::env::var("CONSOLE_RECOVERY_CONTROL").expect("owned two-node harness required"),
    )
    .arg(action)
    .status()
    .expect("invoke owned fixture control");
    assert!(status.success(), "fixture control failed: {action}");
}

struct RestoreReplication;
impl Drop for RestoreReplication {
    fn drop(&mut self) {
        if let Ok(command) = std::env::var("CONSOLE_RECOVERY_CONTROL") {
            for action in ["resume-replay", "restore-sync-policy"] {
                let _ = std::process::Command::new(&command).arg(action).status();
            }
        }
    }
}

fn observer_sql() -> String {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../ops/postgres-install-durability-observer.sql"),
    )
    .expect("real shared observer installer required")
}

async fn observer_identity(owner: &PgPool) -> Value {
    sqlx::query_scalar(
        "SELECT jsonb_build_array(r.oid,p.oid,p.proowner,p.proacl::text,p.prosrc) \
         FROM pg_catalog.pg_roles r CROSS JOIN pg_catalog.pg_proc p \
         WHERE r.rolname='console_durability_observer' \
           AND p.oid=pg_catalog.to_regprocedure('public.console_durability_observation_v1(name,oid)')",
    )
    .fetch_one(owner)
    .await
    .expect("installed observer identity")
}

async fn setup(owner: &PgPool, timeout_ms: u64) -> Value {
    control("assert-topology");
    let identity: (String, i64, bool) = sqlx::query_as(
        "SELECT current_database(),d.oid::bigint, \
         session_user='console_buck_admin' AND current_user=session_user \
         AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' \
         AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user) \
         AND pg_catalog.pg_get_userbyid(d.datdba)=current_user \
         FROM pg_catalog.pg_database d WHERE d.datname=current_database()",
    )
    .fetch_one(owner)
    .await
    .unwrap();
    assert!(
        identity.2,
        "initial observer requires marked admin-owned DB"
    );
    assert_eq!(
        owner.connect_options().get_database(),
        Some(identity.0.as_str())
    );
    let suffix = identity.0.strip_prefix("_sqlx_test_").unwrap();
    assert!(
        suffix.len() == 52
            && suffix
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    );
    let absent: bool = sqlx::query_scalar(
        "SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_durability_observer') \
         AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace \
         WHERE n.nspname='public' AND p.proname='console_durability_observation_v1')",
    ).fetch_one(owner).await.unwrap();
    assert!(
        absent,
        "each exact case needs a fresh dedicated observer cluster"
    );
    let sql = observer_sql();
    let mut tx = owner.begin().await.unwrap();
    sqlx::raw_sql("SET LOCAL statement_timeout='15s'; SET LOCAL lock_timeout='2s'")
        .execute(tx.as_mut())
        .await
        .unwrap();
    sqlx::raw_sql(sqlx::AssertSqlSafe(sql.as_str()))
        .execute(tx.as_mut())
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let installed = observer_identity(owner).await;

    // This public helper retains its exact EMPTY/admin-owned admission, then
    // deliberately transfers this database to the real migration owner.
    let migration_url = prepare_test_migration_owner_url(owner).await;
    let migration = AppConfig::from_pairs([
        ("CONSOLE_APP_ROLE", "migrate".to_owned()),
        ("DATABASE_URL", migration_url),
    ])
    .unwrap();
    super::run_migrations(&migration)
        .await
        .expect("actual numbered AND Apalis migrations");
    finalize_account_custody(owner).await;

    // Closed-state readback after the recorded ownership handoff. Never call
    // the initial admin-owned installer helper with a weakened ownership guard.
    let closed: (String, i64, bool) = sqlx::query_as(
        "SELECT current_database(),d.oid::bigint, \
         session_user='console_buck_admin' AND current_user=session_user \
         AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' \
         AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user) \
         AND pg_catalog.pg_get_userbyid(d.datdba)='console_app' \
         FROM pg_catalog.pg_database d WHERE d.datname=current_database()",
    )
    .fetch_one(owner)
    .await
    .unwrap();
    assert_eq!((&closed.0, closed.1), (&identity.0, identity.1));
    assert!(
        closed.2,
        "same marked DB with expected migration-owner handoff"
    );
    assert_eq!(
        observer_identity(owner).await,
        installed,
        "migrations must preserve observer"
    );
    let mut tx = owner.begin().await.unwrap();
    sqlx::raw_sql("SET LOCAL statement_timeout='15s'; SET LOCAL lock_timeout='2s'")
        .execute(tx.as_mut())
        .await
        .unwrap();
    sqlx::raw_sql(sqlx::AssertSqlSafe(sql.as_str()))
        .execute(tx.as_mut())
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        observer_identity(owner).await,
        installed,
        "closed readback must not repair"
    );
    control("assert-topology");
    let native = sqlx::query(
        "SELECT c.system_identifier::text AS system_id,pg_postmaster_start_time() AS primary_start, \
         r.usesysid::bigint AS role_oid,r.usename::text AS role_name,host(r.client_addr) AS client_addr,x.ssl \
         FROM pg_control_system() c CROSS JOIN pg_replication_slots s \
         JOIN pg_stat_replication r ON r.pid=s.active_pid JOIN pg_stat_ssl x ON x.pid=r.pid \
         WHERE s.slot_name='console_recovery_s1' AND s.slot_type='physical' AND s.active \
         AND r.application_name='console_recovery_s1' AND r.usename='console_fixture_replica' AND r.state='streaming'",
    ).fetch_one(owner).await.unwrap();
    assert!(
        !native.get::<bool, _>("ssl"),
        "fixture is admitted private-network SCRAM"
    );
    let started: OffsetDateTime = native.get("primary_start");
    json!({
        "mode":"required_remote_apply",
        "primary_system_id":native.get::<String,_>("system_id"),
        "primary_started_at":started.format(&time::format_description::well_known::Rfc3339).unwrap(),
        "slot":"console_recovery_s1",
        "replication_role_oid":native.get::<i64,_>("role_oid"),
        "replication_role_name":native.get::<String,_>("role_name"),
        "application_name":"console_recovery_s1",
        "peer":{"mode":"admitted_private_network","client_addr":native.get::<String,_>("client_addr")},
        "timeout_ms":timeout_ms
    })
}

fn config(
    owner: &PgPool,
    role: AppRole,
    descriptor: &Value,
    public_key: Option<&str>,
) -> AppConfig {
    let mut pairs = vec![
        ("CONSOLE_APP_ROLE", role.to_string()),
        ("CONSOLE_HTTP_ADDR", "127.0.0.1:0".to_owned()),
        ("CONSOLE_DATABASE_DURABILITY", descriptor.to_string()),
        (
            "DATABASE_URL",
            login_test_database_url(owner, TestDatabaseLogin::Business),
        ),
    ];
    if role == AppRole::Api {
        for (key, login) in [
            (
                "LEAVE_COMMAND_DATABASE_URL",
                TestDatabaseLogin::LeaveCommand,
            ),
            (
                "ONTOLOGY_COMMAND_DATABASE_URL",
                TestDatabaseLogin::OntologyCommand,
            ),
            (
                "PLATFORM_FORCE_COMMAND_DATABASE_URL",
                TestDatabaseLogin::PlatformForceCommand,
            ),
        ] {
            pairs.push((key, login_test_database_url(owner, login)));
        }
    }
    if let Some(public_key) = public_key {
        pairs.extend([
            ("CONSOLE_JWT_ISSUER", ISSUER.to_owned()),
            ("CONSOLE_JWT_AUDIENCE", AUDIENCE.to_owned()),
            ("CONSOLE_JWT_PUBLIC_KEY_PEM", public_key.to_owned()),
            (
                "AUTH_DATABASE_URL",
                login_test_database_url(owner, TestDatabaseLogin::Auth),
            ),
        ]);
    }
    AppConfig::from_pairs(pairs).expect("explicit real app transports and policy")
}

#[cfg(test)]
async fn start(config: AppConfig) -> AppState {
    let state = tokio::time::timeout(Duration::from_secs(20), AppState::from_config(config))
        .await
        .unwrap()
        .expect("actual custody/identity/observer startup must succeed");
    let pool = business(&state);
    let identity: (String, String) = sqlx::query_as("SELECT session_user::text,current_user::text")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(identity, ("console_rt".into(), "console_rt".into()));
    let response = build_router(state.clone())
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "real startup readiness positive"
    );
    state
}

fn business(state: &AppState) -> PgPool {
    match &state.database {
        DatabaseDependency::Postgres(pool) => pool.clone(),
        DatabaseDependency::NotConfigured => panic!("test requires actual admitted Postgres state"),
    }
}

async fn close_state(state: AppState) {
    state.shutdown_realtime().await;
    for database in [
        &state.database,
        &state.leave_command_database,
        &state.ontology_command_database,
        &state.platform_force_command_database,
    ] {
        if let DatabaseDependency::Postgres(pool) = database {
            pool.close().await;
        }
    }
    if let Some(auth) = state
        .auth_rest
        .as_ref()
        .and_then(|auth| auth.auth_database())
    {
        auth.close().await;
    }
}

async fn startup_refuses(config: AppConfig, marker: &str) {
    let result = tokio::time::timeout(Duration::from_secs(20), AppState::from_config(config))
        .await
        .unwrap();
    match result {
        Ok(state) => {
            close_state(state).await;
            panic!("DURABILITY_COMPOSITION_STARTUP_ACCEPTED: {marker}");
        }
        Err(AppError::Config(message)) => assert!(
            message.starts_with("database durability UNKNOWN:"),
            "specific native durability refusal required for {marker}: {message}"
        ),
        Err(_) => panic!("DURABILITY_COMPOSITION_UNRELATED_STARTUP_FAILURE: {marker}"),
    }
}

#[sqlx::test(migrations = false)]
async fn required_app_startup_admits_only_the_installed_observer(owner: PgPool) {
    let descriptor = setup(&owner, 2_000).await;
    let login = login_test_pool(&owner, TestDatabaseLogin::Business).await;
    login.close().await;
    for role in [AppRole::Api, AppRole::Worker] {
        close_state(start(config(&owner, role, &descriptor, None)).await).await;
    }
    let baseline = observer_identity(&owner).await;
    sqlx::raw_sql("REVOKE EXECUTE ON FUNCTION public.console_durability_observation_v1(name,oid) FROM console_rt")
        .execute(&owner).await.unwrap();
    for role in [AppRole::Api, AppRole::Worker] {
        startup_refuses(
            config(&owner, role, &descriptor, None),
            "missing_business_observer_execute",
        )
        .await;
    }
    sqlx::raw_sql("GRANT EXECUTE ON FUNCTION public.console_durability_observation_v1(name,oid) TO console_rt")
        .execute(&owner).await.unwrap();
    assert_eq!(
        observer_identity(&owner).await,
        baseline,
        "restore exact function identity/ACL/body"
    );
    for role in [AppRole::Api, AppRole::Worker] {
        for (key, wrong) in [
            ("primary_system_id", json!("1")),
            ("primary_started_at", json!("2000-01-01T00:00:00Z")),
        ] {
            let mut rejected = descriptor.clone();
            rejected[key] = wrong;
            startup_refuses(config(&owner, role, &rejected, None), key).await;
        }
        close_state(start(config(&owner, role, &descriptor, None)).await).await;
        let mut missing = config(&owner, role, &descriptor, None);
        missing.database_durability = None;
        missing.database_url =
            Some("postgresql://console_rt:synthetic@127.0.0.1:1/unreachable".into());
        match tokio::time::timeout(Duration::from_secs(1), AppState::from_config(missing))
            .await
            .unwrap()
        {
            Err(AppError::Config(message)) => {
                assert!(message.contains("CONSOLE_DATABASE_DURABILITY"))
            }
            Ok(state) => {
                close_state(state).await;
                panic!("missing policy accepted");
            }
            Err(_) => panic!("missing policy reached transport instead of config gate"),
        }
    }
    println!("durability-composition: actual_api_worker_startup PASS");
}

#[derive(Debug)]
struct HttpResult {
    status: StatusCode,
    etag: Option<String>,
    body: Value,
}

async fn request(
    router: axum::Router,
    method: &str,
    path: &str,
    token: &str,
    etag: Option<&str>,
    body: Value,
) -> HttpResult {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(etag) = etag {
        request = request.header(header::IF_MATCH, etag);
    }
    let response = router
        .oneshot(
            request
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let etag = response
        .headers()
        .get(header::ETAG)
        .map(|v| v.to_str().unwrap().to_owned());
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let body = serde_json::from_slice(&bytes).expect("actual JSON route response");
    HttpResult { status, etag, body }
}

fn issue(issuer: &JwtIssuer, subject: UserId, org: OrgId) -> String {
    issuer
        .issue_access_token(AccessTokenInput {
            subject,
            org_id: org,
            roles: vec!["SUPER_ADMIN".into()],
            branches: Vec::new(),
            platform: false,
            view_as: false,
            read_only: false,
            display_name: None,
            feature_grants: Vec::new(),
            authz_subject_version: 0,
            authz_policy_version: 0,
            session_generation: 0,
            issued_at: OffsetDateTime::now_utc(),
        })
        .unwrap()
}

async fn publish_payroll_type(
    router: &axum::Router,
    author: &str,
    approver: &str,
    actor: UserId,
) -> Value {
    // Reuse the existing real HTTP author/review/approval/publish sequence.
    // No owner UPDATE of lifecycle, forged approval, or manual port registry.
    let created = request(router.clone(), "POST", "/api/v1/ontology/object-types", author, None, json!({
        "stable_key":"canonical.pay_run", "title":"Payroll composition fixture",
        "title_property_key":"label", "backing_kind":"projected",
        "backing_table":"payroll_draft_runs", "primary_key_property":"id",
        "properties":[{"key":"label","title":"Label","field_type":"text","config":{},"required":true}],
        "links":[], "analytics":[], "actions":[{
            "stable_key":"create_run","title":"Create blocked draft","params_schema":{},
            "edits":[],"submission_criteria":[],"side_effects":[],
            "dispatch":"projected_usecase","dispatch_target":"payroll.create_run",
            "control_points":["authority"]
        }]
    })).await;
    assert_eq!(
        created.status,
        StatusCode::CREATED,
        "actual authoring prerequisite: {:?}",
        created.body
    );
    let id = created.body["id"].clone();
    assert!(id.as_str().and_then(|s| Uuid::parse_str(s).ok()).is_some());
    let reviewed = request(
        router.clone(),
        "POST",
        "/api/v1/ontology/object-types/canonical.pay_run/lifecycle",
        author,
        Some(created.etag.as_deref().unwrap()),
        json!({"to_state":"review_pending"}),
    )
    .await;
    assert_eq!(
        reviewed.status,
        StatusCode::OK,
        "real review transition prerequisite: {:?}",
        reviewed.body
    );
    let revision = reviewed.body["key_write_revision"].as_i64().unwrap();
    let request_ref = Uuid::new_v4();
    let requested = request(
        router.clone(),
        "POST",
        "/api/v1/governance/approvals",
        author,
        None,
        json!({
            "request_ref":request_ref,"kind":"ontology.schema.publish","target_ref":id,
            "payload_summary":{"key_revision":revision}
        }),
    )
    .await;
    assert_eq!(
        requested.status,
        StatusCode::CREATED,
        "real approval request: {:?}",
        requested.body
    );
    let decided = request(
        router.clone(),
        "POST",
        "/api/v1/governance/approvals/decide",
        approver,
        None,
        json!({
            "request_ref":request_ref,"kind":"ontology.schema.publish",
            "requested_by":actor.as_uuid(),"decision":"approved"
        }),
    )
    .await;
    assert_eq!(
        decided.status,
        StatusCode::CREATED,
        "distinct principal approval: {:?}",
        decided.body
    );
    let published = request(
        router.clone(),
        "POST",
        "/api/v1/ontology/object-types/canonical.pay_run/lifecycle",
        author,
        Some(reviewed.etag.as_deref().unwrap()),
        json!({"to_state":"published"}),
    )
    .await;
    assert_eq!(
        published.status,
        StatusCode::OK,
        "real publication prerequisite: {:?}",
        published.body
    );
    assert_eq!(published.body["lifecycle_state"], "published");
    id
}

#[derive(Clone, Debug)]
struct Backend {
    pid: i32,
    started: OffsetDateTime,
    application_name: String,
    query_started: OffsetDateTime,
}

async fn database_now(owner: &PgPool) -> OffsetDateTime {
    sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(owner)
        .await
        .unwrap()
}

async fn confirmation_backend(owner: &PgPool, not_before: OffsetDateTime) -> Option<Backend> {
    // Idle is legitimate between the owner's native polling queries. Bound the
    // query timestamp to this operation instead of accepting old query history
    // or sampling only sub-millisecond active windows.
    let rows: Vec<(i32, OffsetDateTime, String, OffsetDateTime)> = sqlx::query_as(
        "SELECT pid,backend_start,application_name,query_start FROM pg_stat_activity \
         WHERE datname=current_database() AND usename='console_rt' \
         AND query_start >= $1 AND query LIKE '%/* console_durability_observe_v1 */%'",
    )
    .bind(not_before)
    .fetch_all(owner)
    .await
    .unwrap();
    assert!(
        rows.len() <= 1,
        "one isolated current-operation owner confirmation backend required"
    );
    rows.into_iter()
        .next()
        .map(|(pid, started, application_name, query_started)| Backend {
            pid,
            started,
            application_name,
            query_started,
        })
}

async fn observes(owner: &PgPool, backend: &Backend) -> bool {
    sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1 AND backend_start=$2 \
         AND application_name=$3 AND datname=current_database() AND query_start >= $4 \
         AND query LIKE '%/* console_durability_observe_v1 */%')",
    )
    .bind(backend.pid)
    .bind(backend.started)
    .bind(&backend.application_name)
    .bind(backend.query_started)
    .fetch_one(owner)
    .await
    .unwrap()
}

async fn standby(owner: &PgPool) -> PgPool {
    let port = std::env::var("CONSOLE_RECOVERY_STANDBY_PORT")
        .unwrap()
        .parse()
        .unwrap();
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect_with(owner.connect_options().as_ref().clone().port(port))
        .await
        .unwrap()
}

async fn assert_paused_below(owner: &PgPool, standby: &PgPool) {
    let bound: String = sqlx::query_scalar("SELECT pg_current_wal_insert_lsn()::text")
        .fetch_one(owner)
        .await
        .unwrap();
    let below: bool = sqlx::query_scalar(
        "SELECT pg_is_in_recovery() AND pg_get_wal_replay_pause_state()='paused' \
         AND pg_last_wal_replay_lsn() < $1::pg_lsn",
    )
    .bind(bound)
    .fetch_one(standby)
    .await
    .unwrap();
    assert!(
        below,
        "actual paused standby must be below the local effect bound"
    );
}

#[cfg(test)]
async fn canonical_snapshot(pool: &PgPool, org: OrgId, command: Uuid) -> Value {
    sqlx::query_scalar(
        "SELECT jsonb_build_object( \
         'receipts',(SELECT coalesce(jsonb_agg(to_jsonb(r) ORDER BY command_id),'[]') FROM ont_action_command_receipts r WHERE org_id=$1 AND command_id=$2), \
         'drafts',(SELECT coalesce(jsonb_agg(to_jsonb(d) ORDER BY id),'[]') FROM payroll_draft_runs d WHERE org_id=$1), \
         'audits',(SELECT coalesce(jsonb_agg(to_jsonb(a) ORDER BY id),'[]') FROM audit_events a \
                    WHERE org_id=$1 AND target_id=($2::uuid)::text AND action='ontology.canonical.execute'))",
    ).bind(org.as_uuid()).bind(command).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = false)]
async fn required_api_projected_payroll_dispatch_waits_for_remote_apply(owner: PgPool) {
    let descriptor = setup(&owner, 15_000).await;
    let org = OrgId::from_uuid(Uuid::new_v4());
    let actor = seed_org_and_super_admin(&owner, *org.as_uuid(), "durability-api-author").await;
    let approver =
        seed_org_and_super_admin(&owner, *org.as_uuid(), "durability-api-approver").await;
    assert_ne!(actor, approver);
    let auth = login_test_pool(&owner, TestDatabaseLogin::Auth).await;
    for subject in [actor, approver] {
        let fenced: bool = sqlx::query_scalar("SELECT public.account_legacy_fenced_v1($1)")
            .bind(subject.as_uuid())
            .fetch_one(&auth)
            .await
            .unwrap();
        assert!(
            !fenced,
            "real Auth projection must admit each actual fixture subject"
        );
    }
    auth.close().await;
    let key = SigningKey::random(&mut OsRng);
    let private_key = key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_key = key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let issuer = JwtIssuer::from_es256_pem(
        JwtSettings {
            issuer: ISSUER.into(),
            audience: AUDIENCE.into(),
            access_token_ttl: time::Duration::minutes(15),
        },
        private_key.as_bytes(),
        public_key.as_bytes(),
    )
    .unwrap();
    let author_token = issue(&issuer, actor, org);
    let approver_token = issue(&issuer, approver, org);
    let state = start(config(&owner, AppRole::Api, &descriptor, Some(&public_key))).await;
    let router = build_router(state.clone());
    let object_type = publish_payroll_type(&router, &author_token, &approver_token, actor).await;
    let command = Uuid::new_v4();
    let run = Uuid::new_v4();
    let payload = json!({
        "object_type_id":object_type,"instance_id":run,"command_id":command,
        "reason":"durability composition","valid_from":"2026-06-01T00:00:00Z",
        "params":{"run_id":run,"period_start":[2026,152],"period_end":[2026,181],"connector":"m2","job":"payroll_draft"}
    });
    let replica = standby(&owner).await;
    assert_eq!(
        canonical_snapshot(&owner, org, command).await["drafts"],
        json!([])
    );
    control("pause-replay");
    let restore = RestoreReplication;
    // Audit COMMIT must remain executable, so only the real owner's explicit
    // remote confirmation can hold back HTTP success and its success audit.
    control("clear-sync-policy");
    let observation_not_before = database_now(&owner).await;
    let route = router.clone();
    let token = author_token.clone();
    let sent = payload.clone();
    let mut call = tokio::spawn(async move {
        request(
            route,
            "POST",
            "/api/v1/ontology/actions/create_run/execute",
            &token,
            None,
            sent,
        )
        .await
    });
    let local = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let local = canonical_snapshot(&owner, org, command).await;
            if local["receipts"].as_array().unwrap().len() == 1 {
                break local;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("real local canonical receipt");
    assert_eq!(local["drafts"].as_array().unwrap().len(), 1);
    assert_paused_below(&owner, &replica).await;
    let observed = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Some(backend) = confirmation_backend(&owner, observation_not_before).await {
                break observes(&owner, &backend).await;
            }
            if call.is_finished() {
                break false;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let early = tokio::time::timeout(Duration::from_millis(300), &mut call).await;
    let premature = matches!(&early, Ok(Ok(response)) if response.status.is_success());
    let before_resume = canonical_snapshot(&owner, org, command).await;
    control("resume-replay");
    control("restore-sync-policy");
    drop(restore);
    let response = match early {
        Ok(result) => result.unwrap(),
        Err(_) => tokio::time::timeout(Duration::from_secs(20), call)
            .await
            .unwrap()
            .unwrap(),
    };
    assert_eq!(
        response.status,
        StatusCode::OK,
        "post-resume actual API success: {:?}",
        response.body
    );
    assert!(
        observed && !premature,
        "app registry must retain Required through native confirmation"
    );
    assert_eq!(
        before_resume["audits"],
        json!([]),
        "no canonical success audit before remote confirmation"
    );
    let receipt = &local["receipts"][0];
    assert_eq!(response.body["projected"]["owner"], receipt["owner"]);
    assert_eq!(response.body["projected"]["target"], "payroll.create_run");
    assert_eq!(response.body["projected"]["command_id"], json!(command));
    assert_eq!(response.body["projected"]["result"], receipt["receipt"]);
    let replay = request(
        router.clone(),
        "POST",
        "/api/v1/ontology/actions/create_run/execute",
        &author_token,
        None,
        payload,
    )
    .await;
    assert_eq!(replay.status, StatusCode::OK);
    assert_eq!(replay.body["projected"], response.body["projected"]);
    let final_rows = canonical_snapshot(&owner, org, command).await;
    assert_eq!(final_rows["receipts"], local["receipts"]);
    assert_eq!(final_rows["drafts"], local["drafts"]);
    assert_eq!(final_rows["drafts"][0]["status"], "BLOCKED_LEGAL_GATE");
    assert_eq!(final_rows["drafts"][0]["calculation_enabled"], false);
    assert_eq!(final_rows["audits"].as_array().unwrap().len(), 1);
    assert_eq!(canonical_snapshot(&replica, org, command).await, final_rows);
    drop(router);
    close_state(state).await;
    replica.close().await;
    println!("durability-composition: actual_projected_api_dispatch PASS");
}

#[cfg(test)]
async fn seed_completion(pool: &PgPool, org: OrgId) -> (Uuid, Uuid) {
    // This is the existing M2 fixture shape, executed with actual Business
    // LOGIN. The engine, rather than SQL event inserts, emits the payroll job.
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(org.to_string())
        .execute(tx.as_mut())
        .await
        .unwrap();
    let definition: Uuid = sqlx::query_scalar(
        "INSERT INTO workflow_definitions \
         (org_id,workflow_key,display_name,object_type,status,latest_version,active_version) \
         VALUES ($1,'work_order.completion','Completion fixture','work_order','ACTIVE',1,1) RETURNING id",
    ).bind(org.as_uuid()).fetch_one(tx.as_mut()).await.unwrap();
    sqlx::query(
        "INSERT INTO workflow_definition_versions \
         (org_id,definition_id,version,status,definition,required_approval_line,required_payment_line) \
         VALUES ($1,$2,1,'PUBLISHED',$3,TRUE,TRUE)",
    ).bind(org.as_uuid()).bind(definition)
        .bind(json!({"schema_version":"wf.exec.v1","template":"work_order_completion"}))
        .execute(tx.as_mut()).await.unwrap();
    tx.commit().await.unwrap();
    let store = PgWorkflowRuntimeStore::new(pool.clone());
    scope_org(
        org,
        console_workorder_rest::m2_strangler::drive_completion_tail(
            &store,
            org,
            WorkOrderId::new(),
            None,
            definition,
            1,
            Vec::new(),
        ),
    )
    .await
    .expect("real completion engine emits the pending payroll job");
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(org.to_string())
        .execute(tx.as_mut())
        .await
        .unwrap();
    let rows: Vec<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT id,run_id FROM workflow_outbox_events WHERE org_id=$1 \
         AND channel='JOB' AND payload->>'job'='payroll_draft'",
    )
    .bind(org.as_uuid())
    .fetch_all(tx.as_mut())
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        rows.len(),
        1,
        "real engine emitted exactly one payroll event"
    );
    rows[0]
}

#[cfg(test)]
async fn workflow_snapshot(pool: &PgPool, org: OrgId, event: Uuid, run: Uuid) -> Value {
    // Independent witness only: all serving work uses the actual Business pool.
    sqlx::query_scalar(
        "SELECT jsonb_build_object( \
         'event',(SELECT to_jsonb(e) FROM workflow_outbox_events e WHERE org_id=$1 AND id=$2 AND run_id=$3), \
         'runs',(SELECT coalesce(jsonb_agg(to_jsonb(r) ORDER BY id),'[]') FROM workflow_runs r WHERE org_id=$1), \
         'nodes',(SELECT coalesce(jsonb_agg(to_jsonb(n) ORDER BY id),'[]') FROM workflow_node_runs n WHERE org_id=$1 AND run_id=$3), \
         'drafts',(SELECT coalesce(jsonb_agg(to_jsonb(d) ORDER BY id),'[]') FROM payroll_draft_runs d WHERE org_id=$1), \
         'receipts',(SELECT coalesce(jsonb_agg(to_jsonb(r) ORDER BY command_id),'[]') FROM ont_action_command_receipts r WHERE org_id=$1), \
         'audits',(SELECT coalesce(jsonb_agg(to_jsonb(a) ORDER BY id),'[]') FROM audit_events a \
                    WHERE org_id=$1 AND target_id=($2::uuid)::text AND action='workflow_runtime.outbox_drain'))",
    ).bind(org.as_uuid()).bind(event).bind(run).fetch_one(pool).await.unwrap()
}

async fn staging_observer(
    owner: &PgPool,
    org: OrgId,
    event: Uuid,
    run: Uuid,
    not_before: OffsetDateTime,
) -> Backend {
    tokio::time::timeout(Duration::from_secs(4), async {
        loop {
            if let Some(backend) = confirmation_backend(owner, not_before).await {
                return backend;
            }
            let snapshot = workflow_snapshot(owner, org, event, run).await;
            assert_eq!(
                snapshot["event"]["status"], "PENDING",
                "DURABILITY_COMPOSITION_WORKER_PREMATURE_ACK: no Required observer before delivery"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("actual staging owner's native confirmation query")
}

async fn backend_gone(owner: &PgPool, backend: &Backend) {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1 AND backend_start=$2 \
                 AND application_name=$3 AND datname=current_database())",
            )
            .bind(backend.pid)
            .bind(backend.started)
            .bind(&backend.application_name)
            .fetch_one(owner)
            .await
            .unwrap();
            if !exists {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("actual unconfirmed owner backend discarded after bounded UNKNOWN");
}

#[sqlx::test(migrations = false)]
async fn required_workflow_spawn_keeps_unknown_staging_pending_until_retry(owner: PgPool) {
    let descriptor = setup(&owner, 5_000).await;
    let org = OrgId::from_uuid(Uuid::new_v4());
    seed_org_rls_off(&owner, *org.as_uuid(), "durability-workflow").await;
    let state = start(config(&owner, AppRole::Worker, &descriptor, None)).await;
    let pool = business(&state);
    let (event, run) = seed_completion(&pool, org).await;
    let before = workflow_snapshot(&owner, org, event, run).await;
    assert_eq!(before["runs"].as_array().unwrap().len(), 1);
    assert_eq!(before["runs"][0]["id"], json!(run));
    assert_eq!(before["runs"][0]["status"], "SUCCEEDED");
    assert_eq!(before["nodes"].as_array().unwrap().len(), 2);
    assert_eq!(before["event"]["status"], "PENDING");
    assert_eq!(before["event"]["delivered_at"], Value::Null);
    assert_eq!(before["event"]["attempt_count"], 0);
    for key in ["drafts", "receipts", "audits"] {
        assert_eq!(before[key], json!([]));
    }
    let replica = standby(&owner).await;
    control("pause-replay");
    let restore = RestoreReplication;
    // Phase 1 SELECT FOR UPDATE may itself wait in SyncRep. Clear policy before
    // the real spawn, then prove Required using the staging owner's native
    // confirmation. Neither claim nor ACK COMMIT may hide a Local regression.
    control("clear-sync-policy");
    let observation_not_before = database_now(&owner).await;
    let drain = super::workflow_drain::spawn(pool.clone(), state.postgres_durability());
    let backend = staging_observer(&owner, org, event, run, observation_not_before).await;
    let staged = workflow_snapshot(&owner, org, event, run).await;
    assert_eq!(
        staged["drafts"].as_array().unwrap().len(),
        1,
        "UNKNOWN permits the genuine local draft; it must not authorize ACK"
    );
    assert_eq!(
        staged["drafts"][0]["source_label"],
        format!("workflow_runtime_m2:run:{run}")
    );
    assert_eq!(staged["drafts"][0]["status"], "BLOCKED_LEGAL_GATE");
    assert_eq!(staged["drafts"][0]["calculation_enabled"], false);
    assert_eq!(staged["event"], before["event"]);
    assert_eq!(staged["audits"], json!([]));
    assert_eq!(staged["receipts"], json!([]));
    assert_paused_below(&owner, &replica).await;
    assert!(observes(&owner, &backend).await);
    backend_gone(&owner, &backend).await;
    let unknown = workflow_snapshot(&owner, org, event, run).await;
    assert_eq!(
        unknown, staged,
        "completed unconfirmed stage must leave event and all evidence unchanged"
    );
    drain.shutdown();
    close_state(state).await;
    drop(pool);
    // shutdown signals the existing loop; closing its pool waits for active
    // leases and prevents this instance from writing during the fresh retry.
    let after_shutdown = workflow_snapshot(&owner, org, event, run).await;
    assert_eq!(after_shutdown, staged);
    control("resume-replay");
    control("restore-sync-policy");
    drop(restore);
    control("assert-topology");
    let retry_state = start(config(&owner, AppRole::Worker, &descriptor, None)).await;
    let retry_pool = business(&retry_state);
    let retry = super::workflow_drain::spawn(retry_pool.clone(), retry_state.postgres_durability());
    let delivered = tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let rows = workflow_snapshot(&owner, org, event, run).await;
            if rows["event"]["status"] == "DELIVERED" {
                break rows;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("actual fresh spawn immediately restages and delivers same pending event");
    retry.shutdown();
    close_state(retry_state).await;
    drop(retry_pool);
    assert_eq!(
        delivered["drafts"], staged["drafts"],
        "same-provenance retry must preserve the one local draft"
    );
    assert_eq!(delivered["runs"], before["runs"]);
    assert_eq!(delivered["nodes"], before["nodes"]);
    assert_eq!(delivered["event"]["attempt_count"], 1);
    assert!(delivered["event"]["delivered_at"].is_string());
    let mut original_event = before["event"].clone();
    let mut delivered_event = delivered["event"].clone();
    for key in ["status", "attempt_count", "delivered_at", "updated_at"] {
        original_event.as_object_mut().unwrap().remove(key);
        delivered_event.as_object_mut().unwrap().remove(key);
    }
    assert_eq!(
        delivered_event, original_event,
        "ACK must retain exact event identity and payload"
    );
    assert_eq!(
        delivered["receipts"],
        json!([]),
        "staging never creates canonical receipts"
    );
    assert_eq!(delivered["audits"].as_array().unwrap().len(), 1);
    assert_eq!(workflow_snapshot(&owner, org, event, run).await, delivered);
    assert_eq!(
        workflow_snapshot(&replica, org, event, run).await,
        delivered
    );
    replica.close().await;
    println!("durability-composition: actual_workflow_spawn_unknown_retry PASS");
}

async fn publish_unknown_payroll_type(
    router: &axum::Router,
    author: &str,
    approver: &str,
    actor: UserId,
    control_points: Value,
) -> Value {
    // Reuse the existing real HTTP author/review/approval/publish sequence.
    // No owner UPDATE of lifecycle, forged approval, or manual port registry.
    let created = request(router.clone(), "POST", "/api/v1/ontology/object-types", author, None, json!({
        "stable_key":"canonical.pay_run", "title":"Payroll composition fixture",
        "title_property_key":"label", "backing_kind":"projected",
        "backing_table":"payroll_draft_runs", "primary_key_property":"id",
        "properties":[{"key":"label","title":"Label","field_type":"text","config":{},"required":true}],
        "links":[], "analytics":[], "actions":[{
            "stable_key":"create_run","title":"Create blocked draft","params_schema":{},
            "edits":[],"submission_criteria":[],"side_effects":[],
            "dispatch":"projected_usecase","dispatch_target":"payroll.create_run",
            "control_points":control_points
        }]
    })).await;
    assert_eq!(
        created.status,
        StatusCode::CREATED,
        "actual authoring prerequisite: {:?}",
        created.body
    );
    let id = created.body["id"].clone();
    assert!(id.as_str().and_then(|s| Uuid::parse_str(s).ok()).is_some());
    let reviewed = request(
        router.clone(),
        "POST",
        "/api/v1/ontology/object-types/canonical.pay_run/lifecycle",
        author,
        Some(created.etag.as_deref().unwrap()),
        json!({"to_state":"review_pending"}),
    )
    .await;
    assert_eq!(
        reviewed.status,
        StatusCode::OK,
        "real review transition prerequisite: {:?}",
        reviewed.body
    );
    let revision = reviewed.body["key_write_revision"].as_i64().unwrap();
    let request_ref = Uuid::new_v4();
    let requested = request(
        router.clone(),
        "POST",
        "/api/v1/governance/approvals",
        author,
        None,
        json!({
            "request_ref":request_ref,"kind":"ontology.schema.publish","target_ref":id,
            "payload_summary":{"key_revision":revision}
        }),
    )
    .await;
    assert_eq!(
        requested.status,
        StatusCode::CREATED,
        "real approval request: {:?}",
        requested.body
    );
    let decided = request(
        router.clone(),
        "POST",
        "/api/v1/governance/approvals/decide",
        approver,
        None,
        json!({
            "request_ref":request_ref,"kind":"ontology.schema.publish",
            "requested_by":actor.as_uuid(),"decision":"approved"
        }),
    )
    .await;
    assert_eq!(
        decided.status,
        StatusCode::CREATED,
        "distinct principal approval: {:?}",
        decided.body
    );
    let published = request(
        router.clone(),
        "POST",
        "/api/v1/ontology/object-types/canonical.pay_run/lifecycle",
        author,
        Some(reviewed.etag.as_deref().unwrap()),
        json!({"to_state":"published"}),
    )
    .await;
    assert_eq!(
        published.status,
        StatusCode::OK,
        "real publication prerequisite: {:?}",
        published.body
    );
    assert_eq!(published.body["lifecycle_state"], "published");
    id
}

async fn unknown_staging_observer(
    owner: &PgPool,
    org: OrgId,
    event: Uuid,
    run: Uuid,
    not_before: OffsetDateTime,
    prior_status: &str,
) -> Backend {
    tokio::time::timeout(Duration::from_secs(4), async {
        loop {
            if let Some(backend) = confirmation_backend(owner, not_before).await {
                return backend;
            }
            let snapshot = workflow_snapshot(owner, org, event, run).await;
            assert_eq!(
                snapshot["event"]["status"], prior_status,
                "DURABILITY_COMPOSITION_WORKER_PREMATURE_ACK: no Required observer before delivery"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("actual staging owner's native confirmation query")
}

// Next reviewed boundary slice. Existing three composition cases above remain unchanged.
const COMPLETION_UNKNOWN_MESSAGE: &str = "Completion could not be confirmed. Retry after service recovery with the original command_id and unchanged business input. A consumed approval may need renewal for the same action and target.";

struct UnknownApi {
    state: AppState,
    router: axum::Router,
    org: OrgId,
    author: String,
    actor: UserId,
    approver: String,
    payload: Value,
    command: Uuid,
    run: Uuid,
}

async fn unknown_api(owner: &PgPool, timeout_ms: u64, four_eyes: bool) -> UnknownApi {
    let descriptor = setup(owner, timeout_ms).await;
    let org = OrgId::from_uuid(Uuid::new_v4());
    let actor = seed_org_and_super_admin(owner, *org.as_uuid(), "unknown-author").await;
    let approver = seed_org_and_super_admin(owner, *org.as_uuid(), "unknown-approver").await;
    assert_ne!(actor, approver);
    let auth = login_test_pool(owner, TestDatabaseLogin::Auth).await;
    for subject in [actor, approver] {
        assert!(
            !sqlx::query_scalar::<_, bool>("SELECT public.account_legacy_fenced_v1($1)")
                .bind(subject.as_uuid())
                .fetch_one(&auth)
                .await
                .unwrap()
        );
    }
    auth.close().await;
    let key = SigningKey::random(&mut OsRng);
    let private_key = key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_key = key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let issuer = JwtIssuer::from_es256_pem(
        JwtSettings {
            issuer: ISSUER.into(),
            audience: AUDIENCE.into(),
            access_token_ttl: time::Duration::minutes(15),
        },
        private_key.as_bytes(),
        public_key.as_bytes(),
    )
    .unwrap();
    let author_token = issue(&issuer, actor, org);
    let approver_token = issue(&issuer, approver, org);
    let state = start(config(owner, AppRole::Api, &descriptor, Some(&public_key))).await;
    let router = build_router(state.clone());
    let object_type = publish_unknown_payroll_type(
        &router,
        &author_token,
        &approver_token,
        actor,
        if four_eyes {
            json!(["authority", "four_eyes"])
        } else {
            json!(["authority"])
        },
    )
    .await;
    let command = Uuid::new_v4();
    let run = Uuid::new_v4();
    let payload = json!({
        "object_type_id":object_type,"instance_id":run,"command_id":command,
        "reason":"typed completion boundary","valid_from":"2026-06-01T00:00:00Z",
        "params":{"run_id":run,"period_start":[2026,152],"period_end":[2026,181],"connector":"m2","job":"payroll_draft"}
    });
    UnknownApi {
        state,
        router,
        org,
        author: author_token,
        actor,
        approver: approver_token,
        payload,
        command,
        run,
    }
}

async fn unknown_execute(api: &UnknownApi, payload: Value) -> HttpResult {
    tokio::time::timeout(
        Duration::from_secs(8),
        request(
            api.router.clone(),
            "POST",
            "/api/v1/ontology/actions/create_run/execute",
            &api.author,
            None,
            payload,
        ),
    )
    .await
    .expect("bounded actual action response")
}

fn assert_completion_unknown(response: &HttpResult) {
    assert_eq!(
        response.status,
        StatusCode::SERVICE_UNAVAILABLE,
        "typed owner uncertainty must have its public outcome: {:?}",
        response.body
    );
    assert_eq!(
        response.body,
        json!({"error": {
            "code":"completion_unknown", "message":COMPLETION_UNKNOWN_MESSAGE
        }}),
        "existing error envelope only; no diagnostics or invented command field"
    );
}

async fn unknown_native_waiter(owner: &PgPool, boundary: OffsetDateTime) -> Backend {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            // Relation locks distinguish the actual payroll owner transaction
            // from authentication/governance/audit COMMITs on the same app pool.
            let rows: Vec<(i32, OffsetDateTime, String, OffsetDateTime)> = sqlx::query_as(
                "SELECT a.pid,a.backend_start,a.application_name,a.query_start FROM pg_stat_activity a \
                 WHERE a.datname=current_database() AND a.usename='console_rt' AND a.query_start >= $1 \
                   AND a.state='active' AND a.wait_event_type='IPC' AND a.wait_event='SyncRep' \
                   AND EXISTS(SELECT 1 FROM pg_locks l WHERE l.pid=a.pid AND l.granted \
                     AND l.relation='public.payroll_draft_runs'::regclass AND l.mode='RowExclusiveLock') \
                   AND EXISTS(SELECT 1 FROM pg_locks l WHERE l.pid=a.pid AND l.granted \
                     AND l.relation='public.ont_action_command_receipts'::regclass AND l.mode='RowExclusiveLock')",
            ).bind(boundary).fetch_all(owner).await.unwrap();
            assert!(rows.len() <= 1, "one isolated native payroll transaction");
            if let Some((pid, started, application_name, query_started)) = rows.into_iter().next() {
                return Backend { pid, started, application_name, query_started };
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("positive active native payroll SyncRep witness")
}

#[sqlx::test(migrations = false)]
async fn required_api_completion_unknown_reconciles_same_command_after_replay(owner: PgPool) {
    let api = unknown_api(&owner, 3_000, false).await;
    let replica = standby(&owner).await;
    let original = api.payload.clone();
    control("pause-replay");
    let restore = RestoreReplication;
    let boundary = database_now(&owner).await;
    let route = api.router.clone();
    let token = api.author.clone();
    let sent = original.clone();
    let call = tokio::spawn(async move {
        request(
            route,
            "POST",
            "/api/v1/ontology/actions/create_run/execute",
            &token,
            None,
            sent,
        )
        .await
    });
    let backend = unknown_native_waiter(&owner, boundary).await;
    let response = tokio::time::timeout(Duration::from_secs(5), call)
        .await
        .unwrap()
        .unwrap();
    assert_completion_unknown(&response);
    assert_eq!(
        canonical_snapshot(&owner, api.org, api.command).await,
        json!({"receipts":[],"drafts":[],"audits":[]}),
        "native incomplete COMMIT is not snapshot-visible or a success acknowledgment"
    );
    let still_waiting: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1 AND backend_start=$2 \
         AND datname=current_database() AND state='active' AND wait_event_type='IPC' AND wait_event='SyncRep')")
        .bind(backend.pid).bind(backend.started).fetch_one(&owner).await.unwrap();
    assert!(
        still_waiting,
        "UNKNOWN must not claim cancellation or rollback"
    );
    control("resume-replay");
    drop(restore);
    backend_gone(&owner, &backend).await;
    let committed = canonical_snapshot(&owner, api.org, api.command).await;
    assert_eq!(committed["receipts"].as_array().unwrap().len(), 1);
    assert_eq!(committed["drafts"].as_array().unwrap().len(), 1);
    assert_eq!(committed["audits"], json!([]));
    let replay = unknown_execute(&api, original.clone()).await;
    assert_eq!(replay.status, StatusCode::OK, "{:?}", replay.body);
    assert_eq!(replay.body["projected"]["command_id"], json!(api.command));
    let complete = canonical_snapshot(&owner, api.org, api.command).await;
    assert_eq!(complete["receipts"], committed["receipts"]);
    assert_eq!(complete["drafts"], committed["drafts"]);
    assert_eq!(complete["audits"].as_array().unwrap().len(), 1);
    assert_eq!(unknown_execute(&api, original).await.body, replay.body);
    assert_eq!(
        canonical_snapshot(&owner, api.org, api.command).await,
        complete
    );
    assert_eq!(
        canonical_snapshot(&replica, api.org, api.command).await,
        complete
    );
    replica.close().await;
    close_state(api.state).await;
}

async fn unknown_approval(api: &UnknownApi, request_ref: Uuid) {
    let requested = request(
        api.router.clone(),
        "POST",
        "/api/v1/governance/approvals",
        &api.author,
        None,
        json!({"request_ref":request_ref,"kind":"create_run",
            "target_ref":api.run,"payload_summary":{"command_id":api.command}}),
    )
    .await;
    assert_eq!(
        requested.status,
        StatusCode::CREATED,
        "{:?}",
        requested.body
    );
    let decided = request(api.router.clone(), "POST", "/api/v1/governance/approvals/decide",
        &api.approver, None, json!({"request_ref":request_ref,"kind":"create_run","requested_by":api.actor.as_uuid(),"decision":"approved"})).await;
    assert_eq!(decided.status, StatusCode::CREATED, "{:?}", decided.body);
}

async fn unknown_approval_history(owner: &PgPool, org: OrgId, first: Uuid, second: Uuid) -> Value {
    sqlx::query_scalar(
        "SELECT jsonb_build_object( \
         'decisions',(SELECT coalesce(jsonb_agg(to_jsonb(a) ORDER BY request_ref),'[]') FROM gov_approvals a \
           WHERE org_id=$1 AND request_ref IN ($2,$3)), \
         'consumptions',(SELECT coalesce(jsonb_agg(to_jsonb(c) ORDER BY c.id),'[]') FROM gov_approval_consumptions c \
           JOIN gov_approvals a ON (a.id,a.org_id)=(c.approval_id,c.org_id) \
           WHERE a.org_id=$1 AND a.request_ref IN ($2,$3)))")
        .bind(org.as_uuid()).bind(first).bind(second).fetch_one(owner).await.unwrap()
}

#[sqlx::test(migrations = false)]
async fn required_api_receipt_absent_unknown_renews_approval_with_same_command(owner: PgPool) {
    let mut api = unknown_api(&owner, 3_000, true).await;
    let first = Uuid::new_v4();
    let second = Uuid::new_v4();
    unknown_approval(&api, first).await;
    api.payload["four_eyes_request_ref"] = json!(first);
    let original = api.payload.clone();
    let observer_before = observer_identity(&owner).await;
    // Actual admission failure after the route's committed approval consumption;
    // the real observer function remains installed and its body is never forged.
    sqlx::raw_sql("REVOKE EXECUTE ON FUNCTION public.console_durability_observation_v1(name,oid) FROM console_rt")
        .execute(&owner).await.unwrap();
    assert!(!sqlx::query_scalar::<_, bool>("SELECT has_function_privilege('console_rt','public.console_durability_observation_v1(name,oid)','EXECUTE')")
        .fetch_one(&owner).await.unwrap());
    let response = unknown_execute(&api, original.clone()).await;
    sqlx::raw_sql("GRANT EXECUTE ON FUNCTION public.console_durability_observation_v1(name,oid) TO console_rt")
        .execute(&owner).await.unwrap();
    assert_eq!(observer_identity(&owner).await, observer_before);
    assert_completion_unknown(&response);
    let absent = canonical_snapshot(&owner, api.org, api.command).await;
    assert_eq!(absent, json!({"receipts":[],"drafts":[],"audits":[]}));
    let spent = unknown_approval_history(&owner, api.org, first, second).await;
    assert_eq!(spent["decisions"].as_array().unwrap().len(), 1);
    assert_eq!(spent["consumptions"].as_array().unwrap().len(), 1);
    let refusal = unknown_execute(&api, original.clone()).await;
    assert_eq!(refusal.status, StatusCode::FORBIDDEN);
    assert_eq!(refusal.body["error"]["code"], "gate_denied");
    assert_eq!(
        canonical_snapshot(&owner, api.org, api.command).await,
        absent
    );
    unknown_approval(&api, second).await;
    let mut renewed = original.clone();
    renewed["four_eyes_request_ref"] = json!(second);
    let mut business_before = original.clone();
    let mut business_after = renewed.clone();
    business_before
        .as_object_mut()
        .unwrap()
        .remove("four_eyes_request_ref");
    business_after
        .as_object_mut()
        .unwrap()
        .remove("four_eyes_request_ref");
    assert_eq!(
        business_before, business_after,
        "only the approval witness changes"
    );
    let accepted = unknown_execute(&api, renewed.clone()).await;
    assert_eq!(accepted.status, StatusCode::OK, "{:?}", accepted.body);
    let complete = canonical_snapshot(&owner, api.org, api.command).await;
    for key in ["receipts", "drafts", "audits"] {
        assert_eq!(complete[key].as_array().unwrap().len(), 1);
    }
    let history = unknown_approval_history(&owner, api.org, first, second).await;
    assert_eq!(history["decisions"].as_array().unwrap().len(), 2);
    assert_eq!(history["consumptions"].as_array().unwrap().len(), 2);
    assert!(
        history["decisions"]
            .as_array()
            .unwrap()
            .contains(&spent["decisions"][0])
    );
    assert!(
        history["consumptions"]
            .as_array()
            .unwrap()
            .contains(&spent["consumptions"][0])
    );
    let replay = unknown_execute(&api, renewed).await;
    assert_eq!(replay.status, StatusCode::OK);
    assert_eq!(replay.body["projected"], accepted.body["projected"]);
    assert_eq!(
        canonical_snapshot(&owner, api.org, api.command).await,
        complete
    );
    assert_eq!(
        unknown_approval_history(&owner, api.org, first, second).await,
        history
    );
    let replica = standby(&owner).await;
    assert_eq!(
        canonical_snapshot(&replica, api.org, api.command).await,
        complete
    );
    assert_eq!(
        unknown_approval_history(&replica, api.org, first, second).await,
        history
    );
    replica.close().await;
    close_state(api.state).await;
}

#[cfg(test)]
#[sqlx::test(migrations = false)]
async fn required_api_confirmed_owner_audit_failure_replays_and_repairs(owner: PgPool) {
    let api = unknown_api(&owner, 3_000, false).await;
    let acl: String = sqlx::query_scalar(
        "SELECT relacl::text FROM pg_class WHERE oid='public.audit_events'::regclass",
    )
    .fetch_one(&owner)
    .await
    .unwrap();
    sqlx::raw_sql("REVOKE INSERT ON public.audit_events FROM console_rt")
        .execute(&owner)
        .await
        .unwrap();
    assert!(
        !sqlx::query_scalar::<_, bool>(
            "SELECT has_table_privilege('console_rt','public.audit_events','INSERT')"
        )
        .fetch_one(&owner)
        .await
        .unwrap()
    );
    let response = unknown_execute(&api, api.payload.clone()).await;
    sqlx::raw_sql("GRANT INSERT ON public.audit_events TO console_rt")
        .execute(&owner)
        .await
        .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, String>(
            "SELECT relacl::text FROM pg_class WHERE oid='public.audit_events'::regclass"
        )
        .fetch_one(&owner)
        .await
        .unwrap(),
        acl
    );
    assert_eq!(response.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response.body["error"]["code"], "internal",
        "audit failure after confirmed owner success is not owner completion_unknown"
    );
    let committed = canonical_snapshot(&owner, api.org, api.command).await;
    assert_eq!(committed["receipts"].as_array().unwrap().len(), 1);
    assert_eq!(committed["drafts"].as_array().unwrap().len(), 1);
    assert_eq!(committed["audits"], json!([]));
    let replica = standby(&owner).await;
    assert_eq!(
        canonical_snapshot(&replica, api.org, api.command).await,
        committed,
        "ordinary HTTP failure can follow a remotely confirmed owner effect"
    );
    let replay = unknown_execute(&api, api.payload.clone()).await;
    assert_eq!(replay.status, StatusCode::OK, "{:?}", replay.body);
    let repaired = canonical_snapshot(&owner, api.org, api.command).await;
    assert_eq!(repaired["receipts"], committed["receipts"]);
    assert_eq!(repaired["drafts"], committed["drafts"]);
    assert_eq!(repaired["audits"].as_array().unwrap().len(), 1);
    assert_eq!(
        unknown_execute(&api, api.payload.clone()).await.body,
        replay.body
    );
    assert_eq!(
        canonical_snapshot(&owner, api.org, api.command).await,
        repaired
    );
    assert_eq!(
        canonical_snapshot(&replica, api.org, api.command).await,
        repaired
    );
    replica.close().await;
    close_state(api.state).await;
}

#[derive(Clone, Default)]
struct UnknownOutcomes(
    std::sync::Arc<std::sync::Mutex<Vec<std::collections::BTreeMap<String, String>>>>,
);

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for UnknownOutcomes {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        #[derive(Default)]
        struct Fields(std::collections::BTreeMap<String, String>);
        impl tracing::field::Visit for Fields {
            fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
                self.0.insert(field.name().to_owned(), value.to_owned());
            }
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                self.0.insert(field.name().to_owned(), format!("{value:?}"));
            }
        }
        let mut fields = Fields::default();
        event.record(&mut fields);
        if fields.0.contains_key("outcome")
            || fields.0.get("message").is_some_and(|message| {
                message.starts_with("payroll draft staging failed;")
                    || message == "workflow payroll outbox drainer stopping"
            })
        {
            self.0.lock().unwrap().push(fields.0);
        }
    }
}

impl UnknownOutcomes {
    fn saw_unknown(&self, event: Uuid, run: Uuid) -> bool {
        self.0.lock().unwrap().iter().any(|fields| {
            fields.get("outcome").map(String::as_str) == Some("unknown")
                && fields.get("event_id") == Some(&event.to_string())
                && fields.get("run_id") == Some(&run.to_string())
                && fields.get("source_label") == Some(&format!("workflow_runtime_m2:run:{run}"))
        })
    }
}

#[cfg(test)]
#[sqlx::test(migrations = false)]
async fn required_workflow_typed_unknown_preserves_pending_and_failed_events(owner: PgPool) {
    use tracing_subscriber::prelude::*;
    let outcomes = UnknownOutcomes::default();
    tracing::subscriber::set_global_default(tracing_subscriber::registry().with(outcomes.clone()))
        .expect("one exact case in a fresh supervised process");
    let descriptor = setup(&owner, 3_000).await;
    for prior_status in ["PENDING", "FAILED"] {
        let org = OrgId::from_uuid(Uuid::new_v4());
        let tag = format!("typed-unknown-{}", prior_status.to_lowercase());
        seed_org_rls_off(&owner, *org.as_uuid(), &tag).await;
        let state = start(config(&owner, AppRole::Worker, &descriptor, None)).await;
        let pool = business(&state);
        let (event, run) = seed_completion(&pool, org).await;
        if prior_status == "FAILED" {
            // Explicit input history fixture; the real engine emitted the event.
            // No staging result, ACK or audit is supplied by this preparation.
            sqlx::query("UPDATE workflow_outbox_events SET status='FAILED',attempt_count=2 WHERE id=$1 AND org_id=$2")
                .bind(event).bind(org.as_uuid()).execute(&owner).await.unwrap();
        }
        let before = workflow_snapshot(&owner, org, event, run).await;
        assert_eq!(before["event"]["status"], prior_status);
        let replica = standby(&owner).await;
        control("pause-replay");
        let restore = RestoreReplication;
        control("clear-sync-policy");
        let boundary = database_now(&owner).await;
        let drain = super::workflow_drain::spawn(pool.clone(), state.postgres_durability());
        let backend =
            unknown_staging_observer(&owner, org, event, run, boundary, prior_status).await;
        let local = workflow_snapshot(&owner, org, event, run).await;
        assert_eq!(local["event"], before["event"]);
        assert_eq!(local["drafts"].as_array().unwrap().len(), 1);
        assert_eq!(local["audits"], json!([]));
        assert_eq!(local["receipts"], json!([]));
        assert_paused_below(&owner, &replica).await;
        backend_gone(&owner, &backend).await;
        tokio::time::timeout(Duration::from_secs(2), async {
            while !outcomes.saw_unknown(event, run) {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("typed stage Unknown must be distinguished at the real drainer boundary");
        assert_eq!(workflow_snapshot(&owner, org, event, run).await, local);
        drain.shutdown();
        close_state(state).await;
        drop(pool);
        control("resume-replay");
        control("restore-sync-policy");
        drop(restore);
        let retry_state = start(config(&owner, AppRole::Worker, &descriptor, None)).await;
        let retry_pool = business(&retry_state);
        let retry =
            super::workflow_drain::spawn(retry_pool.clone(), retry_state.postgres_durability());
        let delivered = tokio::time::timeout(Duration::from_secs(15), async {
            loop {
                let rows = workflow_snapshot(&owner, org, event, run).await;
                if rows["event"]["status"] == "DELIVERED" {
                    break rows;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        retry.shutdown();
        close_state(retry_state).await;
        drop(retry_pool);
        assert_eq!(delivered["drafts"], local["drafts"]);
        assert_eq!(delivered["runs"], before["runs"]);
        assert_eq!(delivered["nodes"], before["nodes"]);
        assert_eq!(delivered["receipts"], json!([]));
        assert_eq!(delivered["audits"].as_array().unwrap().len(), 1);
        assert_eq!(
            delivered["event"]["attempt_count"].as_i64().unwrap(),
            before["event"]["attempt_count"].as_i64().unwrap() + 1
        );
        let mut old_event = before["event"].clone();
        let mut new_event = delivered["event"].clone();
        for key in ["status", "attempt_count", "delivered_at", "updated_at"] {
            old_event.as_object_mut().unwrap().remove(key);
            new_event.as_object_mut().unwrap().remove(key);
        }
        assert_eq!(
            old_event, new_event,
            "same event identity and business input after retry"
        );
        assert_eq!(
            workflow_snapshot(&replica, org, event, run).await,
            delivered
        );
        replica.close().await;
    }
}

impl UnknownOutcomes {
    fn saw_provenance_operation(&self, event: Uuid, run: Uuid) -> bool {
        self.0.lock().unwrap().iter().any(|fields| {
            fields.get("event_id") == Some(&event.to_string())
                && fields.get("source_label") == Some(&format!("workflow_runtime_m2:run:{run}"))
                && fields.get("message").is_some_and(|message| {
                    message.starts_with("payroll draft staging failed;")
                })
                && fields.get("error").map(String::as_str)
                    == Some("Conflict: a payroll draft for this run and period already exists with different provenance")
                && fields.get("outcome").map(String::as_str) != Some("unknown")
        })
    }

    fn saw_drainer_stop(&self) -> bool {
        self.0.lock().unwrap().iter().any(|fields| {
            fields.get("message").map(String::as_str)
                == Some("workflow payroll outbox drainer stopping")
        })
    }

    fn unknown_for_stage(&self, event: Uuid, run: Uuid) -> bool {
        self.0.lock().unwrap().iter().any(|fields| {
            fields.get("outcome").map(String::as_str) == Some("unknown")
                && (fields.get("event_id") == Some(&event.to_string())
                    || fields.get("run_id") == Some(&run.to_string())
                    || fields.get("source_label")
                        == Some(&format!("workflow_runtime_m2:run:{run}")))
        })
    }
}

#[sqlx::test(migrations = false)]
async fn required_workflow_provenance_refusal_is_operation_not_unknown(owner: PgPool) {
    use console_payroll_adapter_postgres::pay_run::PgPayRunPort;
    use console_workflow_domain::{PayrollDraftStaging, StagePayrollDraft};
    use tracing_subscriber::prelude::*;

    let outcomes = UnknownOutcomes::default();
    tracing::subscriber::set_global_default(tracing_subscriber::registry().with(outcomes.clone()))
        .expect("one exact case in a fresh supervised process");
    let descriptor = setup(&owner, 3_000).await;
    let org = OrgId::from_uuid(Uuid::new_v4());
    seed_org_rls_off(&owner, *org.as_uuid(), "operation-provenance").await;
    let state = start(config(&owner, AppRole::Worker, &descriptor, None)).await;
    let pool = business(&state);
    let (event, run) = seed_completion(&pool, org).await;
    let emitted = workflow_snapshot(&owner, org, event, run).await;
    assert_eq!(emitted["event"]["status"], "PENDING");
    assert_eq!(emitted["drafts"], json!([]));
    assert_eq!(emitted["receipts"], json!([]));
    assert_eq!(emitted["audits"], json!([]));

    // Use the real owner's public staging port. Only connector provenance
    // differs from the engine's request; the event, run, period and job remain.
    let period: (Option<time::Date>, Option<time::Date>) = sqlx::query_as(
        "SELECT (payload->>'period_start')::date,(payload->>'period_end')::date \
         FROM workflow_outbox_events WHERE id=$1 AND org_id=$2 AND run_id=$3",
    )
    .bind(event)
    .bind(org.as_uuid())
    .bind(run)
    .fetch_one(&owner)
    .await
    .unwrap();
    let period = period.0.zip(period.1);
    let connector = "ordinary-operation-conflict";
    assert_ne!(emitted["event"]["payload"]["connector"], json!(connector));
    let draft = StagePayrollDraft {
        org,
        outbox_event_id: event,
        run_id: run,
        period_start: period.map(|(start, _)| start),
        period_end: period.map(|(_, end)| end),
        connector: Some(connector.to_owned()),
        job: emitted["event"]["payload"]["job"]
            .as_str()
            .map(str::to_owned),
    };
    let staging = PgPayRunPort::new(
        pool.clone(),
        tokio::runtime::Handle::current(),
        state.postgres_durability(),
    );
    assert!(
        tokio::time::timeout(Duration::from_secs(5), scope_org(org, staging.stage(draft)))
            .await
            .expect("bounded real Required owner seed")
            .expect("real owner seed must complete before drainer refusal")
    );
    drop(staging);
    let seeded = workflow_snapshot(&owner, org, event, run).await;
    assert_eq!(seeded["drafts"].as_array().unwrap().len(), 1);
    assert_eq!(
        seeded["drafts"][0]["source_label"],
        format!("workflow_runtime_m2:run:{run}")
    );
    assert_eq!(
        seeded["drafts"][0]["source_summary"]["connector"],
        connector
    );
    assert_eq!(
        seeded["drafts"][0]["source_summary"]["outbox_event_id"],
        json!(event)
    );
    assert_eq!(seeded["drafts"][0]["status"], "BLOCKED_LEGAL_GATE");
    assert_eq!(seeded["drafts"][0]["calculation_enabled"], false);
    let mut expected_seed = emitted;
    expected_seed["drafts"] = seeded["drafts"].clone();
    assert_eq!(
        seeded, expected_seed,
        "owner seed changes only the real draft"
    );
    let replica = standby(&owner).await;
    assert_eq!(workflow_snapshot(&replica, org, event, run).await, seeded);

    let drain = super::workflow_drain::spawn(pool.clone(), state.postgres_durability());
    let observed = tokio::time::timeout(Duration::from_secs(5), async {
        while !outcomes.saw_provenance_operation(event, run) {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await;
    // The real stop log follows run_tick completion. Wait for it before closing
    // the pool, so shutdown cannot mask an incorrect late ACK in that pass.
    drain.shutdown();
    let stopped = tokio::time::timeout(Duration::from_secs(5), async {
        while !outcomes.saw_drainer_stop() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await;
    close_state(state).await;
    drop(pool);
    let after = workflow_snapshot(&owner, org, event, run).await;
    let remote = workflow_snapshot(&replica, org, event, run).await;
    replica.close().await;

    assert!(
        observed.is_ok(),
        "ORDINARY_STAGE_OPERATION_NOT_OBSERVED: exact provenance conflict must reach the real drainer error branch"
    );
    assert!(
        stopped.is_ok(),
        "ordinary refusal drainer must finish its pass before pool shutdown"
    );
    assert!(
        !outcomes.unknown_for_stage(event, run),
        "ORDINARY_STAGE_MISCLASSIFIED_UNKNOWN: known provenance refusal must not emit unknown for its event, run or source"
    );
    assert_eq!(
        after, seeded,
        "ordinary refusal preserves event, draft and observed workflow history without ACK or success audit"
    );
    assert_eq!(remote, seeded);
}
