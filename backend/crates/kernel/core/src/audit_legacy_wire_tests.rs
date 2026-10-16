// Baseline-compatible tests. Mount as audit::legacy_wire_tests before implementation.
use super::*;
use crate::ids::AuditEventId;

fn fixed_event(actor: Option<UserId>) -> AuditEvent {
    let mut event = AuditEvent::new(
        actor,
        AuditAction::new("audit.read").unwrap(),
        "audit_log",
        "legacy-fixed",
        TraceContext::new("1234567890abcdef1234567890abcdef", "1234567890abcdef").unwrap(),
        time::macros::datetime!(2026-09-21 12:34:56.123456 UTC),
    )
    .with_branch(BranchId::from_uuid(uuid::Uuid::from_u128(3)))
    .with_org(OrgId::from_uuid(uuid::Uuid::from_u128(4)))
    .with_snapshots(
        Some(serde_json::json!({"status": "제출"})),
        Some(serde_json::json!({"items": [3, null, 1]})),
    )
    .with_request_context(AuditRequestContext {
        ip: Some("192.0.2.9".into()),
        user_agent: Some("Console/1.0".into()),
        auth_method: Some("passkey".into()),
        device: Some("desktop".into()),
    })
    .with_classification(AuditClassification {
        badges: Some(vec!["민감정보".into(), "대외비".into()]),
        anomaly: Some(false),
        reason: Some("review\n확인".into()),
    });
    event.id = AuditEventId::from_uuid(uuid::Uuid::from_u128(1));
    event
}

fn exact_legacy_wire(actor: Option<UserId>, expected: &str) {
    let event: AuditEvent = fixed_event(actor);
    let actual = serde_json::to_vec(&event).unwrap();
    assert_eq!(
        actual,
        expected.as_bytes(),
        "legacy field order and bytes changed"
    );
    let roundtrip: AuditEvent = serde_json::from_slice(expected.as_bytes()).unwrap();
    assert_eq!(roundtrip, event);
    assert_eq!(serde_json::to_vec(&roundtrip).unwrap(), actual);
}

#[test]
fn legacy_user_actor_exact_serialized_bytes() {
    exact_legacy_wire(
        Some(UserId::from_uuid(uuid::Uuid::from_u128(2))),
        r##"{"id":"00000000-0000-0000-0000-000000000001","actor":"00000000-0000-0000-0000-000000000002","action":"audit.read","target_type":"audit_log","target_id":"legacy-fixed","branch_id":"00000000-0000-0000-0000-000000000003","org_id":"00000000-0000-0000-0000-000000000004","before":{"status":"제출"},"after":{"items":[3,null,1]},"request_context":{"ip":"192.0.2.9","user_agent":"Console/1.0","auth_method":"passkey","device":"desktop"},"classification":{"badges":["민감정보","대외비"],"anomaly":false,"reason":"review\n확인"},"trace":{"trace_id":"1234567890abcdef1234567890abcdef","span_id":"1234567890abcdef"},"occurred_at":[2026,264,12,34,56,123456000,0,0,0]}"##,
    );
}

#[test]
fn legacy_system_actor_exact_serialized_bytes() {
    exact_legacy_wire(
        None,
        r##"{"id":"00000000-0000-0000-0000-000000000001","actor":null,"action":"audit.read","target_type":"audit_log","target_id":"legacy-fixed","branch_id":"00000000-0000-0000-0000-000000000003","org_id":"00000000-0000-0000-0000-000000000004","before":{"status":"제출"},"after":{"items":[3,null,1]},"request_context":{"ip":"192.0.2.9","user_agent":"Console/1.0","auth_method":"passkey","device":"desktop"},"classification":{"badges":["민감정보","대외비"],"anomaly":false,"reason":"review\n확인"},"trace":{"trace_id":"1234567890abcdef1234567890abcdef","span_id":"1234567890abcdef"},"occurred_at":[2026,264,12,34,56,123456000,0,0,0]}"##,
    );
}
