#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Maker–checker (four-eyes / SoD) proofs for the shared lifecycle chokepoint,
//! executed as the genuine non-owner runtime role `console_rt` (NOSUPERUSER,
//! NOBYPASSRLS, FORCE RLS) — a superuser session would BYPASSRLS and mask both
//! the tenant policy and the `governance_findings` write.
//!
//! Requirement (DESIGN §3.9.1 / §3.10 ③④, §3.9.3, HANDOFF §15/§16): 기안자 ≠
//! 승인자 on the approval step, fail closed, override only for the org 대표 /
//! SUPER_ADMIN and only with a governance finding. Authoring, publication and
//! archival steps stay single-actor.
//!
//! 1. The maker cannot approve their own 기안; a second actor can.
//! 2. Legitimate same-actor NON-checker transitions still pass (regression).
//! 3. The org 대표 and SUPER_ADMIN overrides are allowed **and recorded**.
//! 4. The gate is declarative, not `document`-specific: `benefit_catalog_item`
//!    pending→finalized is gated by the same rule.
//! 5. A checker transition with no recorded 기안 fails closed.

use console_kernel_core::{ErrorKind, OrgId};
use console_platform_db::{lifecycle, with_org_conn};
use sqlx::PgPool;
use sqlx::Row;
use sqlx::postgres::PgPoolOptions;
use time::macros::date;
use uuid::Uuid;

const TODAY: time::Date = date!(2026 - 07 - 25);

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

async fn seed_org(owner_pool: &PgPool, org: Uuid) {
    sqlx::query(
        "INSERT INTO organizations (id, slug, name) VALUES ($1, 'org-a', 'Org A') \
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(org)
    .execute(owner_pool)
    .await
    .unwrap();
}

async fn seed_user(
    owner_pool: &PgPool,
    org: Uuid,
    display_name: &str,
    roles: &[&str],
    is_org_lead: bool,
) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO users (org_id, display_name, roles, is_org_lead) \
         VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(org)
    .bind(display_name)
    .bind(roles.iter().map(|r| (*r).to_owned()).collect::<Vec<_>>())
    .bind(is_org_lead)
    .fetch_one(owner_pool)
    .await
    .unwrap()
}

/// One transition through the shared chokepoint, as `console_rt` with the tenant
/// GUC armed, in its own transaction.
async fn transition(
    rt_pool: &PgPool,
    org: Uuid,
    object_type: &'static str,
    object_id: Uuid,
    to_state: &'static str,
    actor: Option<Uuid>,
) -> Result<lifecycle::LifecycleRecord, console_kernel_core::KernelError> {
    with_org_conn::<_, _, console_platform_db::DbError>(rt_pool, OrgId::from_uuid(org), move |tx| {
        Box::pin(async move {
            Ok(lifecycle::transition_lifecycle(
                tx,
                org,
                object_type,
                object_id,
                to_state,
                actor,
                "전이 사유",
                TODAY,
            )
            .await)
        })
    })
    .await
    .unwrap()
}

// ===========================================================================
// 1 + 2. The 기안자 is refused, a second actor is accepted, and the
//        single-actor steps around the approval keep working.
// ===========================================================================
#[sqlx::test(migrations = "./migrations")]
async fn checker_transition_refuses_the_maker_and_accepts_a_second_actor(owner_pool: PgPool) {
    let org = *OrgId::knl().as_uuid();
    seed_org(&owner_pool, org).await;
    let maker = seed_user(&owner_pool, org, "기안자", &["ADMIN"], false).await;
    let checker = seed_user(&owner_pool, org, "승인자", &["ADMIN"], false).await;
    let rt_pool = runtime_role_pool(&owner_pool).await;
    let object_id = Uuid::new_v4();

    // draft → submitted by the author: a legitimate same-actor 상신.
    let submitted = transition(
        &rt_pool,
        org,
        "document",
        object_id,
        "submitted",
        Some(maker),
    )
    .await
    .expect("the author must be able to submit their own draft");
    assert_eq!(submitted.current_state, "submitted");

    // submitted → approved by the same person: SoD violation, hard stop.
    let self_approval = transition(
        &rt_pool,
        org,
        "document",
        object_id,
        "approved",
        Some(maker),
    )
    .await
    .expect_err("the 기안자 must not be able to approve their own 기안");
    assert_eq!(self_approval.kind, ErrorKind::Forbidden);
    assert!(
        self_approval
            .message
            .contains("본인이 기안한 건은 승인할 수 없습니다"),
        "refusal must use the shared SoD message: {}",
        self_approval.message
    );

    // The refusal left no trace of an approval: still `submitted`.
    let state = with_org_conn::<_, _, console_platform_db::DbError>(
        &rt_pool,
        OrgId::from_uuid(org),
        move |tx| {
            Box::pin(async move { lifecycle::get_lifecycle(tx, "document", object_id).await })
        },
    )
    .await
    .unwrap()
    .expect("the lifecycle row must exist");
    assert_eq!(state.current_state, "submitted");

    // A second actor approves: four eyes satisfied.
    let approved = transition(
        &rt_pool,
        org,
        "document",
        object_id,
        "approved",
        Some(checker),
    )
    .await
    .expect("a different actor must be able to approve");
    assert_eq!(approved.current_state, "approved");

    // Regression: the steps around the approval are single-actor by design and
    // must NOT be caught by the guard.
    // - approved → active: publication by the approver themselves.
    transition(
        &rt_pool,
        org,
        "document",
        object_id,
        "active",
        Some(checker),
    )
    .await
    .expect("publication by the approver is a legitimate single-actor act");
    // - active → revised → archived: authoring/archival by one person.
    transition(&rt_pool, org, "document", object_id, "revised", Some(maker))
        .await
        .expect("opening a revision is a legitimate single-actor act");
    let archived = transition(
        &rt_pool,
        org,
        "document",
        object_id,
        "archived",
        Some(maker),
    )
    .await
    .expect("archiving is a legitimate single-actor act");
    assert_eq!(archived.current_state, "archived");

    // No exemption was claimed anywhere in this walk, so no finding was written.
    let findings: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM governance_findings WHERE detector_id = 'anomaly.self_approval'",
    )
    .fetch_one(&owner_pool)
    .await
    .unwrap();
    assert_eq!(
        findings, 0,
        "a blocked self-approval is a refusal, not an exempted override"
    );
}

// ===========================================================================
// 3. 대표 / SUPER_ADMIN override: allowed, but never invisible.
// ===========================================================================
#[sqlx::test(migrations = "./migrations")]
async fn org_lead_and_super_admin_self_approval_is_allowed_and_recorded(owner_pool: PgPool) {
    let org = *OrgId::knl().as_uuid();
    seed_org(&owner_pool, org).await;
    let lead = seed_user(&owner_pool, org, "대표", &["ADMIN"], true).await;
    let root = seed_user(&owner_pool, org, "시스템관리자", &["SUPER_ADMIN"], false).await;
    let rt_pool = runtime_role_pool(&owner_pool).await;

    for (actor, expected_reason) in [(lead, "org_lead_exempt"), (root, "super_admin_exempt")] {
        let object_id = Uuid::new_v4();
        transition(
            &rt_pool,
            org,
            "document",
            object_id,
            "submitted",
            Some(actor),
        )
        .await
        .expect("submit must succeed");
        let approved = transition(
            &rt_pool,
            org,
            "document",
            object_id,
            "approved",
            Some(actor),
        )
        .await
        .expect("대표/SUPER_ADMIN have no higher approver and may self-approve");
        assert_eq!(approved.current_state, "approved");

        // The override is recorded as an OPEN governance finding, readable by
        // the same runtime role that wrote it (RLS-scoped).
        let row = with_org_conn::<_, _, console_platform_db::DbError>(
            &rt_pool,
            OrgId::from_uuid(org),
            move |tx| {
                Box::pin(async move {
                    sqlx::query(
                        "SELECT subject_user_id, severity, status, evidence \
                         FROM governance_findings \
                         WHERE detector_id = 'anomaly.self_approval' \
                           AND entity_type = 'object_lifecycle' AND entity_id = $1",
                    )
                    .bind(format!("document:{object_id}"))
                    .fetch_one(tx.as_mut())
                    .await
                    .map_err(console_platform_db::DbError::Sqlx)
                })
            },
        )
        .await
        .expect("the exempted self-approval must write exactly one finding");

        let subject: Option<Uuid> = row.try_get("subject_user_id").unwrap();
        assert_eq!(subject, Some(actor));
        let severity: String = row.try_get("severity").unwrap();
        assert_eq!(severity, "HIGH");
        let status: String = row.try_get("status").unwrap();
        assert_eq!(status, "OPEN");
        let evidence: serde_json::Value = row.try_get("evidence").unwrap();
        assert_eq!(
            evidence.get("exemption_reason").and_then(|v| v.as_str()),
            Some(expected_reason)
        );
        assert_eq!(
            evidence.get("to_state").and_then(|v| v.as_str()),
            Some("approved")
        );
    }
}

// ===========================================================================
// 4. Not document-specific: the benefit catalog 승인 is gated by the same rule.
// ===========================================================================
#[sqlx::test(migrations = "./migrations")]
async fn benefit_catalog_finalization_is_checker_gated_too(owner_pool: PgPool) {
    let org = *OrgId::knl().as_uuid();
    seed_org(&owner_pool, org).await;
    let maker = seed_user(&owner_pool, org, "제도 기안자", &["ADMIN"], false).await;
    let checker = seed_user(&owner_pool, org, "제도 승인자", &["ADMIN"], false).await;
    let rt_pool = runtime_role_pool(&owner_pool).await;
    let object_id = Uuid::new_v4();

    transition(
        &rt_pool,
        org,
        "benefit_catalog_item",
        object_id,
        "pending",
        Some(maker),
    )
    .await
    .expect("draft → pending (승인 대기 상신) is the author's own act");

    let blocked = transition(
        &rt_pool,
        org,
        "benefit_catalog_item",
        object_id,
        "finalized",
        Some(maker),
    )
    .await
    .expect_err("확정 by the 기안자 must be refused");
    assert_eq!(blocked.kind, ErrorKind::Forbidden);

    let finalized = transition(
        &rt_pool,
        org,
        "benefit_catalog_item",
        object_id,
        "finalized",
        Some(checker),
    )
    .await
    .expect("확정 by a second actor must succeed");
    assert_eq!(finalized.current_state, "finalized");

    // 시행(finalized → implemented) is the preset effective date firing, not a
    // second approval: the same actor may carry it out.
    let implemented = transition(
        &rt_pool,
        org,
        "benefit_catalog_item",
        object_id,
        "implemented",
        Some(checker),
    )
    .await
    .expect("시행 is a single-actor publication step");
    assert_eq!(implemented.current_state, "implemented");
}

// ===========================================================================
// 5. Fail closed when the control cannot be evaluated at all.
// ===========================================================================
#[sqlx::test(migrations = "./migrations")]
async fn checker_transition_without_a_recorded_maker_fails_closed(owner_pool: PgPool) {
    let org = *OrgId::knl().as_uuid();
    seed_org(&owner_pool, org).await;
    let actor = seed_user(&owner_pool, org, "승인자", &["ADMIN"], false).await;
    let rt_pool = runtime_role_pool(&owner_pool).await;
    let object_id = Uuid::new_v4();

    // A lifecycle parked in `submitted` with an empty transition log — the
    // shape a backfill/data migration can produce.
    sqlx::query(
        "INSERT INTO object_lifecycles (org_id, object_type, object_id, current_state) \
         VALUES ($1, 'document', $2, 'submitted')",
    )
    .bind(org)
    .bind(object_id)
    .execute(&owner_pool)
    .await
    .unwrap();

    let err = transition(
        &rt_pool,
        org,
        "document",
        object_id,
        "approved",
        Some(actor),
    )
    .await
    .expect_err("no recorded 기안 → the four-eyes control cannot be evaluated");
    assert_eq!(err.kind, ErrorKind::Forbidden);

    // Same for an unattributed (system) approval of an unattributed 기안.
    let anonymous_object = Uuid::new_v4();
    transition(
        &rt_pool,
        org,
        "document",
        anonymous_object,
        "submitted",
        None,
    )
    .await
    .expect("an unattributed submit is still a legal authoring step");
    let anonymous = transition(
        &rt_pool,
        org,
        "document",
        anonymous_object,
        "approved",
        None,
    )
    .await
    .expect_err("an unattributed actor cannot supply the second pair of eyes");
    assert_eq!(anonymous.kind, ErrorKind::Forbidden);
}
