//! Test-only coordination and exact row-effect checks; no product owners or grants.

use super::super::*;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration as Wait;
use tokio::task::{JoinHandle, JoinSet};

pub(super) struct Fixture {
    pub owner: PgPool,
    pub target: Uuid,
    pub foreign: Uuid,
    pub target_user: Uuid,
    pub destination: Uuid,
    pub actor: UserId,
    harness: Harness,
    auth_pool: PgPool,
    service: Router,
    token: String,
}

impl Fixture {
    pub async fn new(owner: &PgPool, tag: &str) -> Arc<Self> {
        use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
        let mut harness = Harness::new(owner).await;
        harness.rt_pool.close().await;
        harness.force_pool.close().await;
        harness.rt_pool = login_test_pool(owner, TestDatabaseLogin::Business).await;
        harness.force_pool = login_test_pool(owner, TestDatabaseLogin::PlatformForceCommand).await;
        let actor = seed_platform_admin(owner).await;
        let token = harness.token(actor, OrgId::platform(), true);
        let auth_pool = login_test_pool(owner, TestDatabaseLogin::Auth).await;
        let verifier = JwtVerifier::from_es256_public_pem(
            JwtSettings {
                issuer: TEST_ISSUER.to_owned(),
                audience: TEST_AUDIENCE.to_owned(),
                access_token_ttl: Duration::minutes(15),
            },
            harness.public_pem.as_bytes(),
        )
        .unwrap();
        let service = router(
            PlatformRestState::new(
                harness.rt_pool.clone(),
                Some(console_platform_auth::SessionVerification::new(
                    verifier,
                    auth_pool.clone(),
                )),
                PlatformProvisioner::new(Duration::minutes(15)),
            )
            .with_force_remove_command_pool(Some(harness.force_pool.clone())),
        );
        let target = onboard(&service, &token, &format!("race-{tag}")).await;
        let foreign = onboard(&service, &token, &format!("foreign-{tag}")).await;
        archive_org(&service, &token, target).await;
        let target_user = sqlx::query_scalar("SELECT id FROM public.users WHERE org_id=$1")
            .bind(target)
            .fetch_one(owner)
            .await
            .unwrap();
        let destination =
            sqlx::query_scalar("SELECT group_id FROM public.organizations WHERE id=$1")
                .bind(foreign)
                .fetch_one(owner)
                .await
                .unwrap();
        Arc::new(Self {
            owner: owner.clone(),
            target,
            foreign,
            target_user,
            destination,
            actor,
            harness,
            auth_pool,
            service,
            token,
        })
    }
}

pub(super) async fn require_removal_keys(owner: &PgPool) {
    let ready: bool = sqlx::query_scalar(
        "SELECT has_column_privilege('console_credential_owner','public.auth_webauthn_credentials','org_id','UPDATE')
         AND has_column_privilege('console_account_owner','public.company_actors','org_id','SELECT')
         AND has_column_privilege('console_account_owner','public.company_actors','org_id','UPDATE')",
    ).fetch_one(owner).await.unwrap();
    assert!(
        ready,
        "REMOVAL_RACE_OWNER_PREREQUISITE: missing actual RI key privileges; no race coverage reached"
    );
}

pub(super) async fn finish(fixture: Arc<Fixture>, mut task: JoinHandle<()>) {
    let outcome = tokio::time::timeout(Wait::from_secs(60), &mut task).await;
    if outcome.is_err() {
        task.abort();
        let _ = task.await;
    }
    let closed = tokio::time::timeout(Wait::from_secs(15), async {
        fixture.harness.rt_pool.close().await;
        fixture.harness.force_pool.close().await;
        fixture.auth_pool.close().await;
    })
    .await;
    assert!(
        closed.is_ok(),
        "owned restricted pools must close after task/lock cleanup"
    );
    match outcome {
        Ok(Ok(())) => (),
        Ok(Err(error)) if error.is_panic() => std::panic::resume_unwind(error.into_panic()),
        Ok(Err(error)) => panic!("owned race task failed: {error}"),
        Err(_) => panic!("race exceeded its bound; cancellation is not race acceptance"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Kind {
    Reactivate,
    Assign,
    Remove(bool),
}

impl Kind {
    fn marker(self) -> &'static str {
        match self {
            Self::Reactivate => "platform_set_organization_status",
            Self::Assign => "platform_assign_org_to_group",
            Self::Remove(false) => "platform_remove_organization",
            Self::Remove(true) => "platform_force_remove_organization_command",
        }
    }
}

pub(super) struct Reply {
    pub kind: Kind,
    pub status: StatusCode,
    pub body: Value,
}

pub(super) async fn request(f: &Fixture, kind: Kind, target: Uuid) -> Reply {
    let (status, body) = match kind {
        Kind::Remove(true) => force_delete_org(&f.service, &f.token, target).await,
        Kind::Remove(false) => delete_org(&f.service, &f.token, target).await,
        _ => {
            let (method, uri, body) = match kind {
                Kind::Reactivate => (
                    "PATCH",
                    format!("/api/platform/orgs/{target}"),
                    Body::from(r#"{"status":"ACTIVE"}"#),
                ),
                Kind::Assign => (
                    "PUT",
                    format!(
                        "/api/platform/groups/{}/organizations/{target}",
                        f.destination
                    ),
                    Body::empty(),
                ),
                Kind::Remove(_) => unreachable!(),
            };
            let request = Request::builder()
                .method(method)
                .uri(uri)
                .header(header::AUTHORIZATION, format!("Bearer {}", f.token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(body)
                .unwrap();
            let response = f.service.clone().oneshot(request).await.unwrap();
            let status = response.status();
            let bytes = to_bytes(response.into_body(), 256 * 1024).await.unwrap();
            (status, serde_json::from_slice(&bytes).unwrap())
        }
    };
    Reply { kind, status, body }
}

pub(super) fn start(tasks: &mut JoinSet<Reply>, f: &Arc<Fixture>, kind: Kind) {
    let f = Arc::clone(f);
    tasks.spawn(async move { request(&f, kind, f.target).await });
}

pub(super) async fn next(tasks: &mut JoinSet<Reply>) -> Reply {
    tokio::time::timeout(Wait::from_secs(12), tasks.join_next())
        .await
        .expect("command must finish within the interleaving bound; timeout is not success")
        .expect("expected a live command")
        .expect("HTTP command task must finish normally")
}

pub(super) async fn drain(tasks: &mut JoinSet<Reply>) -> BTreeMap<Kind, Reply> {
    let mut result = BTreeMap::new();
    while !tasks.is_empty() {
        let reply = next(tasks).await;
        assert!(result.insert(reply.kind, reply).is_none());
    }
    result
}

pub(super) async fn still_waiting(owner: &PgPool, pid: i32, blocker: i32) -> bool {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity a WHERE a.pid=$1 AND a.datname=current_database() AND a.state='active' AND $2=ANY(pg_catalog.pg_blocking_pids(a.pid)))")
        .bind(pid).bind(blocker).fetch_one(owner).await.unwrap()
}

pub(super) async fn waiting(
    f: &Fixture,
    tasks: &mut JoinSet<Reply>,
    blocker: i32,
    kind: Kind,
) -> i32 {
    tokio::time::timeout(Wait::from_secs(12), async {
        loop {
            let pid = sqlx::query_scalar("SELECT a.pid FROM pg_catalog.pg_stat_activity a WHERE a.datname=current_database() AND a.state='active' AND position($2 IN a.query)>0 AND $1=ANY(pg_catalog.pg_blocking_pids(a.pid))")
                .bind(blocker).bind(kind.marker()).fetch_optional(&f.owner).await.unwrap();
            if let Some(pid) = pid { return pid; }
            if let Some(reply) = tasks.try_join_next() {
                let reply = reply.unwrap();
                panic!("expected {kind:?} owner lock witness, but {:?} completed {}", reply.kind, reply.status);
            }
            tokio::time::sleep(Wait::from_millis(10)).await;
        }
    }).await.expect("actual PostgreSQL blocking witness required; timing alone is not ordering")
}

pub(super) enum HeldRow {
    Company,
    TargetUser,
    DestinationGroup,
}

pub(super) async fn lock_row(
    f: &Fixture,
    row: HeldRow,
) -> (sqlx::Transaction<'static, sqlx::Postgres>, i32) {
    let mut tx = f.owner.begin().await.unwrap();
    let pid = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(tx.as_mut())
        .await
        .unwrap();
    let (sql, id) = match row {
        HeldRow::Company => (
            "SELECT id FROM public.organizations WHERE id=$1 FOR UPDATE",
            f.target,
        ),
        HeldRow::TargetUser => (
            "SELECT id FROM public.users WHERE id=$1 FOR UPDATE",
            f.target_user,
        ),
        HeldRow::DestinationGroup => (
            "SELECT id FROM public.groups WHERE id=$1 FOR UPDATE",
            f.destination,
        ),
    };
    let locked: Uuid = sqlx::query_scalar(sql)
        .bind(id)
        .fetch_one(tx.as_mut())
        .await
        .unwrap();
    assert_eq!(locked, id);
    (tx, pid)
}

pub(super) async fn membership_is_locked(f: &Fixture) {
    let mut tx = f.owner.begin().await.unwrap();
    let result: Result<Uuid, sqlx::Error> = sqlx::query_scalar(
        "SELECT org_id FROM public.group_memberships WHERE org_id=$1 FOR UPDATE NOWAIT",
    )
    .bind(f.target)
    .fetch_one(tx.as_mut())
    .await;
    let tuple_busy = result.err().and_then(|e| {
        e.as_database_error()
            .and_then(|d| d.code())
            .map(|c| c.into_owned())
    });
    tx.rollback().await.unwrap();
    assert_eq!(
        tuple_busy.as_deref(),
        Some("55P03"),
        "real assignment must already own the membership tuple"
    );
}

pub(super) fn expect(reply: &Reply, status: StatusCode, code: Option<&str>) {
    assert_eq!(
        reply.status, status,
        "{:?} must report its accountable outcome",
        reply.kind
    );
    if let Some(code) = code {
        assert_eq!(reply.body["error"]["code"], code);
        let message = reply.body["error"]["message"].as_str().unwrap();
        assert!(!message.is_empty());
        assert!(!message.contains("auth_legacy") && !message.contains("SQLSTATE"));
    }
}
