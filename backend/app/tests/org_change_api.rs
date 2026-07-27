#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
//! Authenticated, runtime-role (`console_rt`) story for the org-change lifecycle
//! engine (STORY-ORG-001): draft → preflight → ordered SoD approval →
//! effective-dated apply, plus deny-by-omission authorization, cross-tenant
//! concealment, and audit readback. It crosses the assembled HTTP router.

use axum::body::{Body, to_bytes};
use http::{Request, StatusCode, header};
use console_app::{AppConfig, AppRole, AppState, DatabaseDependency, build_router};
use console_kernel_core::{OrgId, UserId};
use console_platform_auth::{AccessTokenInput, JwtIssuer, JwtSettings};
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use p256::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use time::{Duration, OffsetDateTime, macros::offset};
use tower::ServiceExt;
use uuid::Uuid;

/// `(org_id, actor, anomaly, reason)` as read back from `audit_events` — every
/// column is nullable in the table, so the row shape is wide enough that
/// `clippy::type_complexity` (denied workspace-wide) rejects it inline.
type RefusalAuditRow = (Option<Uuid>, Option<Uuid>, Option<bool>, Option<String>);

const ISSUER: &str = "console-platform-auth";
const AUDIENCE: &str = "console-api";
const CHANGES: &str = "/api/v1/org-changes";

fn today_kst() -> time::Date {
    OffsetDateTime::now_utc().to_offset(offset!(+9)).date()
}

/// Full REORG lifecycle: idempotent create, preflight receipt, draft-edit
/// staleness, ordered SoD chain with self-approval + out-of-order refusals,
/// effective-date gate, one-transaction apply, and audit readback.
#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn reorg_lifecycle_runs_draft_to_applied_with_ordered_sod(pool: PgPool) {
    let keys = Keys::generate();
    let rt = runtime_role_pool(&pool).await;
    let org = OrgId::knl();
    let region = seed_region(&pool, org, "리전-개편").await;
    // The drafter is an EXECUTIVE so the self-approval probe below reaches the
    // SoD gate itself instead of stopping at the role floor.
    let drafter = seed_user(&pool, org, "EXECUTIVE").await;
    let approver = seed_user(&pool, org, "EXECUTIVE").await;
    let draft_token = keys.token(drafter, org, &["EXECUTIVE"]);
    let exec_token = keys.token(approver, org, &["EXECUTIVE"]);

    let body = json!({
        "kind": "REORG",
        "target": {"kind": "REGION", "ref": region, "label": "수도권 개편"},
        "effectiveDate": today_kst().to_string(),
        "reason": "지점 신설 및 팀 재배치",
        "proposal": [
            {"op": "CREATE_BRANCH", "regionId": region, "name": "신설지점"},
            {"op": "RENAME_REGION", "regionId": region, "name": "수도권"}
        ]
    });
    let idem = "org-change-story-0001";
    let (status, created) = send(
        &rt,
        &keys,
        "POST",
        CHANGES,
        &draft_token,
        Some(body.clone()),
        Some(idem),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create: {created}");
    let id = created["id"].as_str().unwrap().to_owned();
    assert_eq!(created["status"], "DRAFT");
    assert!(
        created["code"].as_str().unwrap().starts_with("OC-"),
        "server-issued code: {created}"
    );

    // Byte-identical replay returns the SAME request (200, not a second row);
    // a changed body under the same key must conflict.
    let (status, replayed) = send(
        &rt,
        &keys,
        "POST",
        CHANGES,
        &draft_token,
        Some(body.clone()),
        Some(idem),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "idempotent replay: {replayed}");
    assert_eq!(replayed["id"].as_str().unwrap(), id);
    let mut changed = body.clone();
    changed["reason"] = json!("다른 사유");
    let (status, conflicted) = send(
        &rt,
        &keys,
        "POST",
        CHANGES,
        &draft_token,
        Some(changed),
        Some(idem),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "changed replay: {conflicted}");

    // Submit before preflight fails closed; preflight promotes to PRECHECKED.
    let (status, early) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/submit"),
        &draft_token,
        None,
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "submit without receipt: {early}"
    );
    let (status, prechecked) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/preflight"),
        &draft_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "preflight: {prechecked}");
    assert_eq!(prechecked["status"], "PRECHECKED");
    assert_eq!(prechecked["preflight"]["blockers"], json!([]));
    assert_eq!(prechecked["preflight"]["stale"], false);

    // A draft edit knocks the request back to DRAFT and marks the receipt
    // stale — the submit gate recomputes, never trusts the stored receipt.
    let (status, edited) = send(
        &rt,
        &keys,
        "PATCH",
        &format!("{CHANGES}/{id}"),
        &draft_token,
        Some(json!({"reason": "지점 신설 및 팀 재배치 (수정)"})),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "draft edit: {edited}");
    assert_eq!(edited["status"], "DRAFT");
    assert_eq!(edited["preflight"]["stale"], true);
    let (status, reprechecked) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/preflight"),
        &draft_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "re-preflight: {reprechecked}");

    let (status, submitted) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/submit"),
        &draft_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "submit: {submitted}");
    assert_eq!(submitted["status"], "IN_APPROVAL");
    let steps = submitted["approvalSteps"].as_array().unwrap();
    assert_eq!(
        steps
            .iter()
            .map(|s| s["roleKey"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["hr", "finance", "legal", "executive"],
        "ordered SoD chain: {submitted}"
    );

    // Post-submit the draft window is closed.
    let (status, locked) = send(
        &rt,
        &keys,
        "PATCH",
        &format!("{CHANGES}/{id}"),
        &draft_token,
        Some(json!({"reason": "너무 늦은 수정"})),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "post-submit edit: {locked}");

    // Effectuate before approval completes fails closed.
    let (status, premature) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/effectuate"),
        &exec_token,
        None,
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "pre-approval apply: {premature}"
    );

    // The drafter may never approve (SoD), and steps decide strictly in order.
    let step_ids: Vec<String> = steps
        .iter()
        .map(|s| s["id"].as_str().unwrap().to_owned())
        .collect();
    let decision = json!({"decision": "APPROVED"});
    let (status, sod) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/approval-steps/{}/decision", step_ids[0]),
        &draft_token,
        Some(decision.clone()),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "self-approval: {sod}");
    let (status, out_of_order) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/approval-steps/{}/decision", step_ids[2]),
        &exec_token,
        Some(decision.clone()),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "out of order: {out_of_order}");

    let mut last = Value::Null;
    for step_id in &step_ids {
        let (status, decided) = send(
            &rt,
            &keys,
            "POST",
            &format!("{CHANGES}/{id}/approval-steps/{step_id}/decision"),
            &exec_token,
            Some(decision.clone()),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "decide {step_id}: {decided}");
        last = decided;
    }
    assert_eq!(last["status"], "APPROVED", "all four decided: {last}");
    let (status, redecided) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/approval-steps/{}/decision", step_ids[0]),
        &exec_token,
        Some(decision),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "re-decide: {redecided}");

    let (status, applied) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/effectuate"),
        &exec_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "effectuate: {applied}");
    assert_eq!(applied["status"], "APPLIED");
    let (status, terminal) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/effectuate"),
        &exec_token,
        None,
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "terminal re-apply: {terminal}"
    );

    // The proposal really applied, in one transaction, as tenant data.
    let renamed: String = sqlx::query_scalar("SELECT name FROM regions WHERE id = $1")
        .bind(region)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(renamed, "수도권");
    let branches: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM branches WHERE region_id = $1 AND name = '신설지점'",
    )
    .bind(region)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(branches, 1, "CREATE_BRANCH op applied");

    // History layer: the transition chain is readable and complete.
    let (status, detail) = send(
        &rt,
        &keys,
        "GET",
        &format!("{CHANGES}/{id}"),
        &draft_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "detail: {detail}");
    let actions: Vec<&str> = detail["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["action"].as_str().unwrap())
        .collect();
    for expected in [
        "create",
        "preflight",
        "draft.update",
        "submit",
        "step.decide",
        "effectuate",
    ] {
        assert!(
            actions.contains(&expected),
            "event {expected} in {actions:?}"
        );
    }

    // Audit spine readback: every lifecycle mutation appended an audit event.
    for action in [
        "org_change.create",
        "org_change.preflight",
        "org_change.draft.update",
        "org_change.submit",
        "org_change.step.decide",
        "org_change.effectuate",
        "org_change.apply.op",
    ] {
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM audit_events WHERE action = $1")
            .bind(action)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(count >= 1, "audit action {action} recorded");
    }
    // Each SoD decision also passed through the gov_approvals second net.
    let gov: i64 =
        sqlx::query_scalar("SELECT count(*) FROM gov_approvals WHERE kind = 'org_change_step'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(gov, 4, "four step decisions recorded through gov_approvals");
}

/// DISSOLVE defers its ops to archive: effectuate opens the six settlement
/// items, archive fails closed while dependents remain, and the deferred
/// deactivation applies only after settlement genuinely cleared them.
#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn dissolve_settles_then_archives_with_referential_net(pool: PgPool) {
    let keys = Keys::generate();
    let rt = runtime_role_pool(&pool).await;
    let org = OrgId::knl();
    let region = seed_region(&pool, org, "리전-폐지").await;
    let branch = seed_branch(&pool, org, region, "폐지지점").await;
    let resident = seed_user_in_branch(&pool, org, "MEMBER", branch).await;
    let drafter = seed_user(&pool, org, "ADMIN").await;
    let approver = seed_user(&pool, org, "EXECUTIVE").await;
    let draft_token = keys.token(drafter, org, &["ADMIN"]);
    let exec_token = keys.token(approver, org, &["EXECUTIVE"]);

    let (status, created) = send(
        &rt,
        &keys,
        "POST",
        CHANGES,
        &draft_token,
        Some(json!({
            "kind": "DISSOLVE",
            "target": {"kind": "BRANCH", "ref": branch, "label": "폐지지점"},
            "effectiveDate": today_kst().to_string(),
            "reason": "지점 폐지",
            "proposal": [{"op": "DEACTIVATE_BRANCH", "branchId": branch}]
        })),
        Some("org-change-dissolve-01"),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create: {created}");
    let id = created["id"].as_str().unwrap().to_owned();

    // Dissolve dependents surface as settlement WARNINGS, never blockers.
    let (status, prechecked) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/preflight"),
        &draft_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "preflight: {prechecked}");
    assert_eq!(prechecked["status"], "PRECHECKED");
    assert_eq!(prechecked["preflight"]["blockers"], json!([]));
    assert!(
        prechecked["preflight"]["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w["code"] == "ACTIVE_USERS"),
        "resident user surfaces as a settlement warning: {prechecked}"
    );

    let (_, submitted) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/submit"),
        &draft_token,
        None,
        None,
    )
    .await;
    for step in submitted["approvalSteps"].as_array().unwrap() {
        let step_id = step["id"].as_str().unwrap();
        let (status, decided) = send(
            &rt,
            &keys,
            "POST",
            &format!("{CHANGES}/{id}/approval-steps/{step_id}/decision"),
            &exec_token,
            Some(json!({"decision": "APPROVED"})),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "decide: {decided}");
    }

    let (status, settling) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/effectuate"),
        &exec_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "effectuate: {settling}");
    assert_eq!(settling["status"], "SETTLING");
    let items = settling["settlementItems"].as_array().unwrap();
    assert_eq!(items.len(), 6, "six settlement items seeded: {settling}");

    // Archive is double-gated: first on unsettled items, then on the real
    // referential state (the resident user has not actually been cleared).
    let (status, unsettled) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/archive"),
        &exec_token,
        None,
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "unsettled archive: {unsettled}"
    );
    for item in items {
        let item_id = item["id"].as_str().unwrap();
        let (status, done) = send(
            &rt,
            &keys,
            "POST",
            &format!("{CHANGES}/{id}/settlement-items/{item_id}/complete"),
            &exec_token,
            Some(json!({"memo": "정산 완료"})),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "complete item: {done}");
    }
    let (status, dishonest) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/archive"),
        &exec_token,
        None,
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "checked-off settlement cannot beat the referential net: {dishonest}"
    );

    // Actually clear the dependent, then archive applies the deactivation.
    sqlx::query("UPDATE users SET is_active = false WHERE id = $1")
        .bind(*resident.as_uuid())
        .execute(&pool)
        .await
        .unwrap();

    // §3.9.1 동결 창 on the OTHER live-apply path: a DISSOLVE defers its
    // deactivation to archive, so a lock opened after effectuate must still
    // refuse it — and record the attempt.
    let lock = seed_period_lock(
        &pool,
        org,
        "accounting",
        today_kst(),
        today_kst(),
        "결산 중",
    )
    .await;
    let (status, frozen) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/archive"),
        &exec_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "frozen archive: {frozen}");
    assert!(
        frozen["error"]["message"]
            .as_str()
            .unwrap()
            .contains("회계 결산"),
        "refusal names the blocking window: {frozen}"
    );
    let still_active: Option<OffsetDateTime> =
        sqlx::query_scalar("SELECT deactivated_at FROM branches WHERE id = $1")
            .bind(branch)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(still_active.is_none(), "frozen archive applied nothing");
    let refused: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM audit_events WHERE action = 'org_change.archive.refused' \
         AND target_id = $1",
    )
    .bind(&id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(refused, 1, "the refused archive is on the record");
    sqlx::query(
        "UPDATE period_locks SET unlocked_at = now(), unlock_reason = '결산 완료' WHERE id = $1",
    )
    .bind(lock)
    .execute(&pool)
    .await
    .unwrap();

    let (status, archived) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/archive"),
        &exec_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "archive: {archived}");
    assert_eq!(archived["status"], "ARCHIVED");

    // Effective-date gate: a fully-approved change refuses to apply before
    // its effective date (발효일), and the past is rejected at the draft door.
    let (status, past) = send(
        &rt,
        &keys,
        "POST",
        CHANGES,
        &draft_token,
        Some(json!({
            "kind": "REORG",
            "target": {"kind": "REGION", "ref": region, "label": "소급 개편"},
            "effectiveDate": (today_kst() - Duration::days(1)).to_string(),
            "reason": "소급 적용 시도",
            "proposal": []
        })),
        Some("org-change-past-00001"),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "past date: {past}"
    );
    let (status, future) = send(
        &rt,
        &keys,
        "POST",
        CHANGES,
        &draft_token,
        Some(json!({
            "kind": "REORG",
            "target": {"kind": "REGION", "ref": region, "label": "예정 개편"},
            "effectiveDate": (today_kst() + Duration::days(7)).to_string(),
            "reason": "발효일 이전 적용 차단 확인",
            "proposal": []
        })),
        Some("org-change-future-0001"),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "future draft: {future}");
    let future_id = future["id"].as_str().unwrap().to_owned();
    send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{future_id}/preflight"),
        &draft_token,
        None,
        None,
    )
    .await;
    let (_, submitted) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{future_id}/submit"),
        &draft_token,
        None,
        None,
    )
    .await;
    for step in submitted["approvalSteps"].as_array().unwrap() {
        let step_id = step["id"].as_str().unwrap();
        let (status, decided) = send(
            &rt,
            &keys,
            "POST",
            &format!("{CHANGES}/{future_id}/approval-steps/{step_id}/decision"),
            &exec_token,
            Some(json!({"decision": "APPROVED"})),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "decide future: {decided}");
    }
    let (status, early) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{future_id}/effectuate"),
        &exec_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "before 발효일: {early}");
    let deactivated: Option<OffsetDateTime> =
        sqlx::query_scalar("SELECT deactivated_at FROM branches WHERE id = $1")
            .bind(branch)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        deactivated.is_some(),
        "deferred deactivation applied at archive"
    );
}

/// Deny-by-default + deny-by-omission: unauthenticated 401, floor-denied 403
/// with the canonical envelope and zero data, REORG blockers refuse submit,
/// cross-tenant rows are concealed as 404/empty, and the entity read fails
/// closed to an empty list without group grants.
#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn authorization_denies_without_leakage_and_conceals_other_tenants(pool: PgPool) {
    let keys = Keys::generate();
    let rt = runtime_role_pool(&pool).await;
    let org = OrgId::knl();
    let region = seed_region(&pool, org, "리전-차단").await;
    let branch = seed_branch(&pool, org, region, "상주지점").await;
    let _resident = seed_user_in_branch(&pool, org, "MEMBER", branch).await;
    let drafter = seed_user(&pool, org, "ADMIN").await;
    let member = seed_user(&pool, org, "MEMBER").await;
    let draft_token = keys.token(drafter, org, &["ADMIN"]);
    let member_token = keys.token(member, org, &["MEMBER"]);

    let (status, _) = send(&rt, &keys, "GET", CHANGES, "", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "missing bearer");

    let (status, denied) = send(&rt, &keys, "GET", CHANGES, &member_token, None, None).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "member floor: {denied}");
    assert!(denied["error"]["code"].is_string(), "envelope: {denied}");
    assert!(denied.get("items").is_none(), "no data leaks: {denied}");
    let (status, denied) = send(
        &rt,
        &keys,
        "POST",
        CHANGES,
        &member_token,
        Some(json!({
            "kind": "REORG",
            "target": {"kind": "REGION", "ref": region, "label": "차단"},
            "effectiveDate": today_kst().to_string(),
            "reason": "차단 확인",
            "proposal": []
        })),
        Some("org-change-denied-0001"),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "member draft: {denied}");

    // Approve floor is higher than draft floor: ADMIN drafts but cannot decide.
    let (status, created) = send(
        &rt,
        &keys,
        "POST",
        CHANGES,
        &draft_token,
        Some(json!({
            "kind": "REORG",
            "target": {"kind": "BRANCH", "ref": branch, "label": "상주지점 개편"},
            "effectiveDate": today_kst().to_string(),
            "reason": "개편 차단 시나리오",
            "proposal": [{"op": "DEACTIVATE_BRANCH", "branchId": branch}]
        })),
        Some("org-change-blocked-001"),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create: {created}");
    let id = created["id"].as_str().unwrap().to_owned();

    // REORG deactivation with a resident active user is a submit BLOCKER.
    let (status, blocked) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/preflight"),
        &draft_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "preflight: {blocked}");
    assert_eq!(blocked["status"], "DRAFT", "blockers hold DRAFT: {blocked}");
    assert!(
        blocked["preflight"]["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b["code"] == "ACTIVE_USERS"),
        "resident user blocks a REORG deactivation: {blocked}"
    );
    let (status, refused) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/submit"),
        &draft_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "blocked submit: {refused}");

    // Cross-tenant concealment: an ADMIN of another org sees nothing — the
    // detail is 404 (not 403) and the list is RLS-empty.
    let other_org = seed_org(&pool, "other-tenant").await;
    let outsider = seed_user(&pool, other_org, "ADMIN").await;
    let outsider_token = keys.token(outsider, other_org, &["ADMIN"]);
    let (status, concealed) = send(
        &rt,
        &keys,
        "GET",
        &format!("{CHANGES}/{id}"),
        &outsider_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "concealed: {concealed}");
    let (status, listed) = send(&rt, &keys, "GET", CHANGES, &outsider_token, None, None).await;
    assert_eq!(status, StatusCode::OK, "outsider list: {listed}");
    assert_eq!(listed["total"], 0, "RLS-filtered list: {listed}");
    let (status, mutated) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/preflight"),
        &outsider_token,
        None,
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "concealed mutation: {mutated}"
    );

    // Rejection ends the request; revision is a NEW row via supersedesId, and
    // only a REJECTED request may be superseded.
    let exec = seed_user(&pool, org, "EXECUTIVE").await;
    let exec_token = keys.token(exec, org, &["EXECUTIVE"]);
    let (status, clean) = send(
        &rt,
        &keys,
        "POST",
        CHANGES,
        &draft_token,
        Some(json!({
            "kind": "REORG",
            "target": {"kind": "REGION", "ref": region, "label": "반려 시나리오"},
            "effectiveDate": today_kst().to_string(),
            "reason": "반려 후 재기안 확인",
            "proposal": []
        })),
        Some("org-change-reject-0001"),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create: {clean}");
    let clean_id = clean["id"].as_str().unwrap().to_owned();
    send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{clean_id}/preflight"),
        &draft_token,
        None,
        None,
    )
    .await;
    let (_, submitted) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{clean_id}/submit"),
        &draft_token,
        None,
        None,
    )
    .await;
    let first_step = submitted["approvalSteps"][0]["id"].as_str().unwrap();
    let (status, rejected) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{clean_id}/approval-steps/{first_step}/decision"),
        &exec_token,
        Some(json!({"decision": "REJECTED", "memo": "예산 근거 부족"})),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "reject: {rejected}");
    assert_eq!(rejected["status"], "REJECTED");
    let (status, revision) = send(
        &rt,
        &keys,
        "POST",
        CHANGES,
        &draft_token,
        Some(json!({
            "kind": "REORG",
            "target": {"kind": "REGION", "ref": region, "label": "반려 시나리오"},
            "effectiveDate": today_kst().to_string(),
            "reason": "예산 근거 보강 재기안",
            "proposal": [],
            "supersedesId": clean_id
        })),
        Some("org-change-reject-0002"),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "revision: {revision}");
    assert_eq!(revision["supersedesId"].as_str().unwrap(), clean_id);
    let (status, bad_supersede) = send(
        &rt,
        &keys,
        "POST",
        CHANGES,
        &draft_token,
        Some(json!({
            "kind": "REORG",
            "target": {"kind": "REGION", "ref": region, "label": "잘못된 재기안"},
            "effectiveDate": today_kst().to_string(),
            "reason": "반려되지 않은 건 재기안 시도",
            "proposal": [],
            "supersedesId": id
        })),
        Some("org-change-reject-0003"),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "only REJECTED may be superseded: {bad_supersede}"
    );

    // Entity list fails closed to empty without live group grants.
    let (status, entities) = send(
        &rt,
        &keys,
        "GET",
        "/api/v1/org-entities",
        &draft_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "entities: {entities}");
    assert_eq!(entities, json!([]), "fail-closed empty: {entities}");

    // With a live group grant the SECURITY DEFINER resolvers surface the
    // member 법인 — the positive path, not just the fail-closed one.
    let group_id: Uuid = sqlx::query_scalar(
        "INSERT INTO groups (slug, name) VALUES ('knl-holdings', '지주회사') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO group_memberships (group_id, org_id) VALUES ($1, $2)")
        .bind(group_id)
        .bind(*org.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO group_role_grants (group_id, user_id, group_role) \
         VALUES ($1, $2, 'GROUP_VIEWER')",
    )
    .bind(group_id)
    .bind(*drafter.as_uuid())
    .execute(&pool)
    .await
    .unwrap();
    let (status, entities) = send(
        &rt,
        &keys,
        "GET",
        "/api/v1/org-entities",
        &draft_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "granted entities: {entities}");
    let listed = entities.as_array().unwrap();
    assert!(
        listed
            .iter()
            .any(|e| e["orgId"].as_str() == Some(&org.as_uuid().to_string())),
        "granted 법인 listed: {entities}"
    );
    // The outsider's grant-less token still sees nothing.
    let (status, outsider_entities) = send(
        &rt,
        &keys,
        "GET",
        "/api/v1/org-entities",
        &outsider_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "outsider entities");
    assert_eq!(outsider_entities, json!([]), "no grant, no 법인 list");
}

/// §3.9.1 변경 동결 창: an APPROVED change may not be applied while the period
/// it takes effect in is closed for 급여 마감 or 회계 결산. The refusal is
/// fail-closed and names the blocking window, the request and the org tree are
/// untouched, the attempt itself is audited even though its transaction rolled
/// back, and another tenant's lock over the same window never bites.
#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn effectuate_is_frozen_inside_a_locked_period_and_records_the_attempt(pool: PgPool) {
    let keys = Keys::generate();
    let rt = runtime_role_pool(&pool).await;
    let org = OrgId::knl();
    let outsider = seed_org(&pool, "freeze-outsider").await;
    let region = seed_region(&pool, org, "리전-동결").await;
    let drafter = seed_user(&pool, org, "ADMIN").await;
    let approver = seed_user(&pool, org, "EXECUTIVE").await;
    let draft_token = keys.token(drafter, org, &["ADMIN"]);
    let exec_token = keys.token(approver, org, &["EXECUTIVE"]);

    let (status, created) = send(
        &rt,
        &keys,
        "POST",
        CHANGES,
        &draft_token,
        Some(json!({
            "kind": "REORG",
            "target": {"kind": "REGION", "ref": region, "label": "동결 검증"},
            "effectiveDate": today_kst().to_string(),
            "reason": "동결 창 적용 확인",
            "proposal": [
                {"op": "CREATE_BRANCH", "regionId": region, "name": "동결지점"},
                {"op": "RENAME_REGION", "regionId": region, "name": "동결후이름"}
            ]
        })),
        Some("org-change-freeze-0001"),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create: {created}");
    let id = created["id"].as_str().unwrap().to_owned();

    // A foreign tenant's lock over the very same window must stay irrelevant,
    // both to the preflight read and to the apply gate.
    seed_period_lock(
        &pool,
        outsider,
        "payroll",
        today_kst(),
        today_kst(),
        "타 법인 급여 마감",
    )
    .await;
    let window_start = today_kst() - Duration::days(3);
    let payroll_lock = seed_period_lock(
        &pool,
        org,
        "payroll",
        window_start,
        today_kst() + Duration::days(3),
        "급여 마감",
    )
    .await;
    let accounting_lock = seed_period_lock(
        &pool,
        org,
        "accounting",
        window_start,
        today_kst() + Duration::days(3),
        "회계 결산",
    )
    .await;

    // Preflight computes the freeze signal from the same locks — it is a read,
    // not a reminder chip, so it names the domains that would refuse the apply.
    // With BOTH domains locked it stays ONE warning: the console keys the
    // warning list by `code`, so a second `FREEZE_WINDOW_REVIEW` row would be a
    // duplicate React key rather than a second signal.
    let (status, prechecked) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/preflight"),
        &draft_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "preflight: {prechecked}");
    let freeze_labels: Vec<&str> = prechecked["preflight"]["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|w| w["code"] == "FREEZE_WINDOW_REVIEW")
        .map(|w| w["label"].as_str().unwrap())
        .collect();
    assert_eq!(
        freeze_labels.len(),
        1,
        "one freeze warning however many domains block: {prechecked}"
    );
    assert!(
        freeze_labels[0].contains("급여 마감") && freeze_labels[0].contains("회계 결산"),
        "the one chip names every blocking domain, in the console's language: {}",
        freeze_labels[0]
    );

    let (status, submitted) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/submit"),
        &draft_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "submit: {submitted}");
    for step in submitted["approvalSteps"].as_array().unwrap() {
        let step_id = step["id"].as_str().unwrap();
        let (status, decided) = send(
            &rt,
            &keys,
            "POST",
            &format!("{CHANGES}/{id}/approval-steps/{step_id}/decision"),
            &exec_token,
            Some(json!({"decision": "APPROVED"})),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "decide: {decided}");
    }

    // Both freeze domains block, and each refusal names its own window. The
    // second pass proves the accounting domain gates on its own, with the
    // payroll lock already lifted.
    for (domain, lock) in [("급여 마감", payroll_lock), ("회계 결산", accounting_lock)] {
        let (status, frozen) = send(
            &rt,
            &keys,
            "POST",
            &format!("{CHANGES}/{id}/effectuate"),
            &exec_token,
            None,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT, "{domain} freeze: {frozen}");
        let message = frozen["error"]["message"].as_str().unwrap();
        assert!(
            message.contains(domain) && message.contains(&today_kst().to_string()),
            "refusal names the blocking {domain} window in the console's language: {message}"
        );
        let (_, still) = send(
            &rt,
            &keys,
            "GET",
            &format!("{CHANGES}/{id}"),
            &draft_token,
            None,
            None,
        )
        .await;
        assert_eq!(still["status"], "APPROVED", "refused apply did not advance");
        sqlx::query(
            "UPDATE period_locks SET unlocked_at = now(), unlock_reason = '검증 해제' \
             WHERE id = $1",
        )
        .bind(lock)
        .execute(&pool)
        .await
        .unwrap();
    }

    // Nothing was applied and no success audit landed.
    let branches: i64 = sqlx::query_scalar("SELECT count(*) FROM branches WHERE name = '동결지점'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(branches, 0, "refused apply wrote no org rows");
    let applied: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM audit_events WHERE action = 'org_change.effectuate'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(applied, 0, "a refused apply is not an apply");

    // The attempt IS on the record — its own committed transaction, anomaly
    // flagged, reason naming the window, scoped to this tenant only.
    let refusals: Vec<RefusalAuditRow> = sqlx::query_as(
        "SELECT org_id, actor, anomaly, reason FROM audit_events \
             WHERE action = 'org_change.effectuate.refused' AND target_id = $1 \
             ORDER BY occurred_at",
    )
    .bind(&id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(refusals.len(), 2, "one detection row per refused attempt");
    for (index, domain) in ["급여 마감", "회계 결산"].iter().enumerate() {
        let (row_org, actor, anomaly, reason) = &refusals[index];
        assert_eq!(row_org.as_ref(), Some(org.as_uuid()), "tenant-scoped");
        assert_eq!(actor.as_ref(), Some(approver.as_uuid()), "actor recorded");
        assert_eq!(*anomaly, Some(true), "freeze refusal is an anomaly");
        assert!(
            reason.as_deref().unwrap().contains(domain),
            "reason names the blocking domain: {reason:?}"
        );
    }

    // Outside every lock the legitimate path is preserved — even with an
    // ACTIVE own-tenant lock over a different period and the outsider's lock
    // still covering today.
    seed_period_lock(
        &pool,
        org,
        "payroll",
        today_kst() - Duration::days(40),
        today_kst() - Duration::days(10),
        "지난달 급여 마감",
    )
    .await;
    let (status, applied) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{id}/effectuate"),
        &exec_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "outside the window: {applied}");
    assert_eq!(applied["status"], "APPLIED");
    let renamed: String = sqlx::query_scalar("SELECT name FROM regions WHERE id = $1")
        .bind(region)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(renamed, "동결후이름", "the unfrozen apply really ran");

    // …and with those same two locks still ACTIVE, a fresh draft whose
    // effective date falls outside them carries NO freeze warning: the signal
    // is computed per effective date, not pushed on every preflight.
    let (_, clean) = send(
        &rt,
        &keys,
        "POST",
        CHANGES,
        &draft_token,
        Some(json!({
            "kind": "REORG",
            "target": {"kind": "REGION", "ref": region, "label": "동결 외"},
            "effectiveDate": today_kst().to_string(),
            "reason": "동결창 밖 확인",
            "proposal": []
        })),
        Some("org-change-freeze-0002"),
    )
    .await;
    let clean_id = clean["id"].as_str().unwrap();
    let (status, clean_report) = send(
        &rt,
        &keys,
        "POST",
        &format!("{CHANGES}/{clean_id}/preflight"),
        &draft_token,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "clean preflight: {clean_report}");
    assert!(
        !clean_report["preflight"]["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w["code"] == "FREEZE_WINDOW_REVIEW"),
        "no lock covers this effective date: {clean_report}"
    );
}

async fn seed_period_lock(
    pool: &PgPool,
    org: OrgId,
    domain: &str,
    start: time::Date,
    end: time::Date,
    reason: &str,
) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO period_locks (org_id, domain, period_start, period_end, reason) \
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(*org.as_uuid())
    .bind(domain)
    .bind(start)
    .bind(end)
    .bind(reason)
    .fetch_one(pool)
    .await
    .unwrap()
}

struct Keys {
    private_pem: String,
    public_pem: String,
}

impl Keys {
    fn generate() -> Self {
        let key = SigningKey::random(&mut OsRng);
        Self {
            private_pem: key.to_pkcs8_pem(LineEnding::LF).unwrap().to_string(),
            public_pem: key
                .verifying_key()
                .to_public_key_pem(LineEnding::LF)
                .unwrap(),
        }
    }

    fn token(&self, user: UserId, org: OrgId, roles: &[&str]) -> String {
        JwtIssuer::from_es256_pem(
            JwtSettings {
                issuer: ISSUER.into(),
                audience: AUDIENCE.into(),
                access_token_ttl: Duration::minutes(15),
            },
            self.private_pem.as_bytes(),
            self.public_pem.as_bytes(),
        )
        .unwrap()
        .issue_access_token(AccessTokenInput {
            subject: user,
            org_id: org,
            roles: roles.iter().map(|r| (*r).to_owned()).collect(),
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
}

async fn runtime_role_pool(owner: &PgPool) -> PgPool {
    PgPoolOptions::new()
        .max_connections(8)
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("SET ROLE console_rt").execute(conn).await?;
                Ok(())
            })
        })
        .connect_with(owner.connect_options().as_ref().clone())
        .await
        .unwrap()
}

async fn send(
    pool: &PgPool,
    keys: &Keys,
    method: &str,
    uri: &str,
    token: &str,
    body: Option<Value>,
    idempotency_key: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if !token.is_empty() {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    if let Some(key) = idempotency_key {
        builder = builder.header("Idempotency-Key", key);
    }
    let request = builder
        .body(
            body.map(|v| Body::from(serde_json::to_vec(&v).unwrap()))
                .unwrap_or_else(Body::empty),
        )
        .unwrap();
    let response = build_router(app_state(pool.clone(), keys.public_pem.clone()).unwrap())
        .oneshot(request)
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        if bytes.is_empty() {
            Value::Null
        } else {
            // The request-context layer's own rejections are not JSON; keep
            // the raw text so status assertions still see the real body.
            serde_json::from_slice(&bytes)
                .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()))
        },
    )
}

fn app_state(pool: PgPool, public_key: String) -> Result<AppState, console_app::AppError> {
    AppState::new(
        AppConfig::from_pairs([
            ("CONSOLE_APP_ROLE", AppRole::Api.to_string()),
            ("CONSOLE_HTTP_ADDR", "127.0.0.1:0".into()),
            ("CONSOLE_JWT_ISSUER", ISSUER.into()),
            ("CONSOLE_JWT_AUDIENCE", AUDIENCE.into()),
            ("CONSOLE_JWT_PUBLIC_KEY_PEM", public_key),
        ])?,
        DatabaseDependency::Postgres(pool),
    )
}

async fn seed_org(pool: &PgPool, slug: &str) -> OrgId {
    let id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (slug, name) VALUES ($1, $2) RETURNING id")
            .bind(slug)
            .bind(format!("org-{slug}"))
            .fetch_one(pool)
            .await
            .unwrap();
    OrgId::from_uuid(id)
}

async fn seed_region(pool: &PgPool, org: OrgId, name: &str) -> Uuid {
    sqlx::query_scalar("INSERT INTO regions (name, org_id) VALUES ($1, $2) RETURNING id")
        .bind(name)
        .bind(*org.as_uuid())
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn seed_branch(pool: &PgPool, org: OrgId, region: Uuid, name: &str) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO branches (region_id, name, org_id) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(region)
    .bind(name)
    .bind(*org.as_uuid())
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn seed_user(pool: &PgPool, org: OrgId, role: &str) -> UserId {
    let user = UserId::new();
    sqlx::query(
        "INSERT INTO users (id, display_name, roles, is_active, org_id) \
         VALUES ($1, $2, $3, true, $4)",
    )
    .bind(*user.as_uuid())
    .bind(format!("story-{user}"))
    .bind(vec![role])
    .bind(*org.as_uuid())
    .execute(pool)
    .await
    .unwrap();
    user
}

async fn seed_user_in_branch(pool: &PgPool, org: OrgId, role: &str, branch: Uuid) -> UserId {
    let user = seed_user(pool, org, role).await;
    sqlx::query("INSERT INTO user_branches (user_id, branch_id, org_id) VALUES ($1, $2, $3)")
        .bind(*user.as_uuid())
        .bind(branch)
        .bind(*org.as_uuid())
        .execute(pool)
        .await
        .unwrap();
    user
}
