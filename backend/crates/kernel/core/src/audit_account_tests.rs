// Mount as audit::account_tests only with the admitted typed implementation.
use super::*;
use crate::AccountId;

#[test]
fn native_account_constructor_keeps_typed_actor_and_all_builders() {
    let account = AccountId::from_uuid(uuid::Uuid::from_u128(17)).unwrap();
    let branch = BranchId::from_uuid(uuid::Uuid::from_u128(18));
    let org = OrgId::from_uuid(uuid::Uuid::from_u128(19));
    let trace = TraceContext::new("0123456789abcdef0123456789abcdef", "0123456789abcdef").unwrap();
    let at = time::macros::datetime!(2026-09-21 12:34:56.123456 UTC);
    let context = AuditRequestContext {
        ip: Some("192.0.2.20".into()),
        user_agent: Some("Console native audit".into()),
        auth_method: Some("passkey".into()),
        device: Some("desktop".into()),
    };
    let classification = AuditClassification {
        badges: Some(vec!["확인".into()]),
        anomaly: Some(false),
        reason: Some("typed actor".into()),
    };
    let event: AuditEvent<AccountId> = AuditEvent::new_account(
        account,
        AuditAction::new("payroll_run.list_read").unwrap(),
        "payroll_draft_run",
        "query",
        trace.clone(),
        at,
    )
    .with_branch(branch)
    .with_org(org)
    .with_snapshots(None, Some(serde_json::Value::Null))
    .with_request_context(context.clone())
    .with_classification(classification.clone());
    assert!(!event.id.as_uuid().is_nil());
    assert_eq!(event.actor, Some(account));
    assert_eq!(event.action.as_str(), "payroll_run.list_read");
    assert_eq!(event.target_type, "payroll_draft_run");
    assert_eq!(event.target_id, "query");
    assert_eq!(event.branch_id, Some(branch));
    assert_eq!(event.org_id, Some(org));
    assert_eq!(event.before, None);
    assert_eq!(event.after, Some(serde_json::Value::Null));
    assert_eq!(event.request_context, context);
    assert_eq!(event.classification, classification);
    assert_eq!(event.trace, trace);
    assert_eq!(event.occurred_at, at);
}

#[test]
fn native_account_identity_retains_nil_rejection() {
    assert!(AccountId::from_uuid(uuid::Uuid::nil()).is_err());
    let raw = uuid::Uuid::from_u128(20);
    let account = AccountId::from_uuid(raw).unwrap();
    assert_eq!(*account.as_uuid(), raw);
}
