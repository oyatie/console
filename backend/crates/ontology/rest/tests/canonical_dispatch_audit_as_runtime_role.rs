#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! RUNTIME proof that a canonical `projected_usecase` success path emits an
//! org-scoped `ontology.canonical.execute` audit row (distinct from the
//! InstanceRevision writeback's `ontology.action.execute`). Canonical ports open
//! raw `pool.begin()` and do not write `audit_events` themselves — the engine
//! must.

use std::collections::BTreeSet;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use console_governance_adapter_postgres::PgGovernanceStore;
use console_kernel_core::{AuditAction, AuditEvent, OrgId, TraceContext, UserId};
use console_ontology_adapter_postgres::instances::PgInstanceStore;
use console_ontology_adapter_postgres::{ActionTypeInput, CreateObjectTypeDraft, PgOntologyStore};
use console_ontology_canonical_domain::{
    CanonicalObject, CanonicalPort, CanonicalQuery, CommandId, CommandReceipt as CanonicalReceipt,
    Company, DispatchTarget, Preflight, ReceiptOwner,
};
use console_ontology_domain::{ActionDispatch, BackingKind, ObjectTypeId};
use console_ontology_rest::{ActionCommand, OntologyRestState, ProjectedDispatchRegistry};
use console_platform_authz::{Principal, Role};
use console_platform_db::{DbError, with_audit};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use time::OffsetDateTime;
use time::macros::datetime;
use uuid::Uuid;

const AT: OffsetDateTime = datetime!(2026-07-10 12:00 UTC);
const DISPATCH_TARGET: &str = "company.revise";

async fn runtime_role_pool(owner_pool: &PgPool) -> PgPool {
    let options = owner_pool.connect_options().as_ref().clone();
    PgPoolOptions::new()
        .max_connections(4)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET ROLE console_rt").execute(conn).await?;
                Ok(())
            })
        })
        .connect_with(options)
        .await
        .unwrap()
}

async fn command_role_pool(owner_pool: &PgPool) -> PgPool {
    let options = owner_pool.connect_options().as_ref().clone();
    PgPoolOptions::new()
        .max_connections(4)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET ROLE console_ontology_cmd")
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .connect_with(options)
        .await
        .unwrap()
}

fn test_audit_event(
    action: &str,
    target_type: &str,
    target_id: impl ToString,
    org: Uuid,
) -> AuditEvent {
    AuditEvent::new(
        None,
        AuditAction::new(action).unwrap(),
        target_type,
        target_id.to_string(),
        TraceContext::generate(),
        OffsetDateTime::now_utc(),
    )
    .with_org(OrgId::from_uuid(org))
}

async fn seed_org(owner_pool: &PgPool, org: Uuid, tag: &str) {
    let event = test_audit_event("test.seed_org", "organization", org, org);
    let tag = tag.to_owned();
    with_audit(owner_pool, event, |tx| {
        Box::pin(async move {
            sqlx::query(
                "INSERT INTO organizations (id, slug, name) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING",
            )
            .bind(org)
            .bind(format!("org-{}", &org.simple().to_string()[..12]))
            .bind(format!("Org {tag}"))
            .execute(tx.as_mut())
            .await
            .map_err(DbError::Sqlx)?;
            Ok::<(), DbError>(())
        })
    })
    .await
    .unwrap();
}

async fn seed_user(owner_pool: &PgPool, org: Uuid, tag: &str) -> UserId {
    let user_id = UserId::new();
    let event = test_audit_event("test.seed_user", "user", *user_id.as_uuid(), org);
    let tag = tag.to_owned();
    with_audit(owner_pool, event, |tx| {
        Box::pin(async move {
            sqlx::query(
                "INSERT INTO users (id, display_name, roles, org_id) VALUES ($1, $2, $3, $4)",
            )
            .bind(*user_id.as_uuid())
            .bind(format!("Admin {tag}"))
            .bind(["SUPER_ADMIN"].as_slice())
            .bind(org)
            .execute(tx.as_mut())
            .await
            .map_err(DbError::Sqlx)?;
            Ok::<(), DbError>(())
        })
    })
    .await
    .unwrap();
    user_id
}

fn super_admin(user_id: UserId, org: OrgId) -> Principal {
    Principal::new(
        user_id,
        org,
        BTreeSet::from([Role::SuperAdmin]),
        console_kernel_core::BranchScope::All,
    )
}

#[derive(Debug, Deserialize)]
struct EchoQuery {
    target: String,
}

impl CanonicalQuery for EchoQuery {
    fn dispatch_target(&self) -> DispatchTarget {
        self.target
            .parse()
            .expect("dispatcher injects a roster member")
    }

    // company.revise is tenant-scoped (`org_id`), not a payload row id — same as
    // CompanyQuery after h3e deleted the CanonicalQuery::subject_id default.
    fn subject_id(&self) -> Option<Uuid> {
        None
    }
}

/// Records what reached the port and writes nothing — isolates the engine's
/// audit obligation from any adapter-side audit (ports use raw `begin()`).
struct StubPort<O> {
    seen: Arc<Mutex<Vec<String>>>,
    object: PhantomData<O>,
}

impl<O> StubPort<O> {
    fn new(seen: &Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            seen: Arc::clone(seen),
            object: PhantomData,
        }
    }
}

impl<O> CanonicalPort for StubPort<O>
where
    O: CanonicalObject + Send + Sync + 'static,
{
    type Object = O;
    type Query = EchoQuery;
    type Command = (OrgId, CommandId, UserId, DispatchTarget);
    type Error = std::convert::Infallible;

    fn preflight(_query: &Self::Query) -> Preflight {
        Preflight::ok()
    }

    fn command(
        org_id: OrgId,
        command_id: CommandId,
        actor_id: UserId,
        query: Self::Query,
        _action_key: &str,
        _object_type_id: uuid::Uuid,
    ) -> Self::Command {
        (org_id, command_id, actor_id, query.dispatch_target())
    }

    fn execute(&self, command: &Self::Command) -> Result<CanonicalReceipt, Self::Error> {
        let (org_id, command_id, actor_id, target) = command;
        self.seen
            .lock()
            .expect("stub mutex")
            .push(target.as_str().to_owned());
        Ok(CanonicalReceipt::new(
            *org_id,
            *command_id,
            ReceiptOwner::Canonical(<O as CanonicalObject>::KEY),
            *target,
            *actor_id,
            [0_u8; 32],
            json!({ "stub": true }),
            OffsetDateTime::UNIX_EPOCH,
        ))
    }
}

async fn seed_canonical_projected_action(
    owner_pool: &PgPool,
    org: OrgId,
    actor: UserId,
    key: &str,
    action_key: &str,
) -> ObjectTypeId {
    console_platform_request_context::scope_org(org, async {
        let store = PgOntologyStore::new(owner_pool.clone())
            .with_command_pool(command_role_pool(owner_pool).await);
        let draft = CreateObjectTypeDraft {
            stable_key: key.to_owned(),
            title: "회사".to_owned(),
            title_property_key: None,
            backing_kind: BackingKind::Projected,
            backing_table: Some("company_revisions".to_owned()),
            primary_key_property: Some("org_id".to_owned()),
            properties: Vec::new(),
            links: Vec::new(),
            actions: vec![ActionTypeInput {
                stable_key: action_key.to_owned(),
                title: "회사 개정".to_owned(),
                params_schema: json!({}),
                edits: json!([]),
                submission_criteria: json!([]),
                side_effects: json!([]),
                dispatch: ActionDispatch::ProjectedUsecase,
                dispatch_target: Some(DISPATCH_TARGET.to_owned()),
                control_points: json!(["authority"]),
            }],
            analytics: Vec::new(),
        };
        store
            .create_object_type(actor, draft, TraceContext::generate(), AT)
            .await
            .expect("create projected object type")
            .id
    })
    .await
}

async fn count_execute_audits(owner_pool: &PgPool, org: OrgId) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_events WHERE org_id = $1 AND action = 'ontology.canonical.execute'",
    )
    .bind(*org.as_uuid())
    .fetch_one(owner_pool)
    .await
    .unwrap()
}

fn state_with_company_stub(
    pool: &PgPool,
    command_pool: &PgPool,
    seen: &Arc<Mutex<Vec<String>>>,
) -> OntologyRestState {
    let registry = ProjectedDispatchRegistry::new().register_port(StubPort::<Company>::new(seen));
    OntologyRestState::new(
        PgOntologyStore::new(pool.clone()).with_command_pool(command_pool.clone()),
        PgInstanceStore::new(pool.clone()),
        PgGovernanceStore::new(pool.clone()),
        None,
    )
    .with_projected_dispatch(registry)
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn canonical_projected_success_emits_ontology_canonical_execute_audit(owner_pool: PgPool) {
    let rt = runtime_role_pool(&owner_pool).await;
    let cmd = command_role_pool(&owner_pool).await;
    let org = OrgId::knl();
    let org_uuid = *org.as_uuid();
    seed_org(&owner_pool, org_uuid, "a").await;
    let actor = seed_user(&owner_pool, org_uuid, "a").await;
    let type_id =
        seed_canonical_projected_action(&owner_pool, org, actor, "company.proj", "revise").await;

    let seen = Arc::new(Mutex::new(Vec::new()));
    let command_id = Uuid::new_v4();
    let outcome = console_platform_request_context::scope_org(org, async {
        state_with_company_stub(&rt, &cmd, &seen)
            .execute_action(
                &super_admin(actor, org),
                "revise",
                ActionCommand {
                    object_type_id: type_id,
                    instance_id: None,
                    title: None,
                    params: json!({}),
                    reason: Some("canonical audit probe".to_owned()),
                    valid_from: Some(AT),
                    checklist_all_acknowledged: None,
                    four_eyes_request_ref: None,
                    command_id: Some(command_id),
                    expected_revision: None,
                },
            )
            .await
    })
    .await
    .expect("canonical projected dispatch must succeed");

    assert!(outcome.instance.is_none());
    assert_eq!(
        outcome.projected.as_ref().and_then(|v| v.get("target")),
        Some(&Value::String(DISPATCH_TARGET.to_owned()))
    );
    assert_eq!(
        seen.lock().unwrap().as_slice(),
        &[DISPATCH_TARGET.to_owned()],
        "stub port must have executed"
    );

    let audits = count_execute_audits(&owner_pool, org).await;
    assert_eq!(
        audits, 1,
        "canonical projected success must write one ontology.canonical.execute audit_events row; got {audits}"
    );

    let target_id: String = sqlx::query_scalar(
        "SELECT target_id FROM audit_events \
         WHERE org_id = $1 AND action = 'ontology.canonical.execute' \
         ORDER BY created_at, id LIMIT 1",
    )
    .bind(org_uuid)
    .fetch_one(&owner_pool)
    .await
    .unwrap();
    assert_eq!(
        target_id,
        command_id.to_string(),
        "audit target_id must be the tenant-global command_id"
    );
}
