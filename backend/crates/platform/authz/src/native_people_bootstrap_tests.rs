//! People maintenance decision vectors, not directory access or SQL custody proof.
//! Child of the existing native_business_policy_unit_tests module; reuse its
//! verified-source fixture and keep every Payroll test and frozen byte oracle.
use super::*;

const PEOPLE_DIGEST: &str = "591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e";
fn people_digest() -> [u8; 32] {
    hex::decode(PEOPLE_DIGEST).unwrap().try_into().unwrap()
}
fn people_request(
    operation: NativeBusinessOperationV1,
    changed: Option<usize>,
) -> NativeBootstrapRequestV1 {
    let mut manifest = people_digest();
    if let Some(byte) = changed.filter(|n| *n >= 3) {
        manifest[byte - 3] ^= 1;
    }
    NativeBootstrapRequestV1::new(
        OrgId::from_uuid(id(if changed == Some(0) { 91 } else { 11 })),
        id(if changed == Some(1) { 92 } else { 12 }),
        AccountId::from_uuid(id(if changed == Some(2) { 93 } else { 34 })).unwrap(),
        operation,
        manifest,
    )
    .unwrap()
}

#[test]
fn native_people_maintenance_uses_cedar_for_all_operations_and_valid_successors() {
    let policy = CompanyPolicy::new().unwrap();
    for same_birth_actor in [true, false] {
        let mut row = bootstrap_row();
        if same_birth_actor {
            row.origin_account_id = id(1);
        }
        let authority =
            CurrentNativeBootstrapAuthority::from_retained_projection(&binding(), row).unwrap();
        for operation in [
            NativeBusinessOperationV1::Install,
            NativeBusinessOperationV1::Grant,
            NativeBusinessOperationV1::Revoke,
        ] {
            policy.sdk_calls.store(0, Ordering::SeqCst);
            assert_eq!(
                policy.decide_native_bootstrap(&authority, &people_request(operation, None)),
                Ok(CompanyPolicyDecision::Allow)
            );
            assert_eq!(
                policy.sdk_calls.load(Ordering::SeqCst),
                1,
                "People maintenance bypassed Cedar"
            );
        }
    }
}

#[test]
fn native_people_maintenance_denies_cross_scope_recipient_and_every_corrupted_digest_byte() {
    let policy = CompanyPolicy::new().unwrap();
    let authority = bootstrap();
    for operation in [
        NativeBusinessOperationV1::Install,
        NativeBusinessOperationV1::Grant,
        NativeBusinessOperationV1::Revoke,
    ] {
        for changed in 0..35 {
            policy.sdk_calls.store(0, Ordering::SeqCst);
            assert_eq!(
                policy
                    .decide_native_bootstrap(&authority, &people_request(operation, Some(changed))),
                Ok(CompanyPolicyDecision::Deny),
                "bad People request vector {changed}"
            );
            assert_eq!(
                policy.sdk_calls.load(Ordering::SeqCst),
                1,
                "denial must reach actual Cedar"
            );
        }
    }
}

#[test]
fn native_people_maintenance_never_substitutes_for_payroll_read_material() {
    let policy = CompanyPolicy::new().unwrap();
    let authority = bootstrap();
    assert_eq!(
        policy.decide_native_bootstrap(
            &authority,
            &people_request(NativeBusinessOperationV1::Grant, None)
        ),
        Ok(CompanyPolicyDecision::Allow)
    );
    let mut row = payroll_row();
    row.action =
        ActionRef::new(OrgId::from_uuid(id(11)), id(21), id(20), 1, people_digest()).unwrap();
    assert!(matches!(
        CurrentPayrollReadAuthority::from_retained_projection(&binding(), row),
        Err(CompanyPolicyError::MaterialUnavailable)
    ));
    assert_eq!(
        policy.decide_native_payroll_collection(&payroll()),
        Ok(CompanyPolicyDecision::Allow)
    );
}
