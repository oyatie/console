//! Negative owner controls; no Console business data is manufactured.
use super::manager_policy::ObservedRead;
use super::*;
use console_identity_adapter_postgres::PgOrgStore;
use console_identity_application::company_policy::{
    CurrentCompanyAuthority,
    company_information::{CompanyInformationAssignmentRefV1, NativeCompanyInformationCommandV1},
    workflow::{
        NativePolicyCommand, NativePolicyCommandRef, NativePolicyWorkflowError,
        accept_native_policy_command, execute_native_policy_command, native_policy_command_status,
        native_policy_current, native_policy_form, native_policy_validation_form,
        submit_native_policy_command,
    },
};
use console_kernel_core::TraceContext;
use console_platform_auth::account::AccountEnrollmentCredentials;
use console_platform_authz::company_policy::CompanyPolicy;

pub(super) async fn negatives(
    pool: &PgPool,
    store: &PgOrgStore,
    authority: &CurrentCompanyAuthority,
    reads: [&AccountEnrollmentCredentials; 3],
    command: &NativePolicyCommand,
    before: &BTreeMap<String, String>,
) {
    let [a, b, o] = reads;
    let company = authority.company();
    let selector = NativePolicyCommandRef::from_command(command);
    let parent = CompanyInformationAssignmentRefV1::new(
        authority.assignment_id(),
        authority.assignment_revision(),
    )
    .unwrap();
    let policy = CompanyPolicy::new().unwrap();
    for unrelated in [o, b] {
        assert!(
            matches!(
                native_policy_current(store, &policy, unrelated, selector).await,
                Err(NativePolicyWorkflowError::NotFound)
            ),
            "operator/unenlisted recipient acquired manager context"
        );
        assert!(
            *before == all_rows(pool).await,
            "healthy manager denial changed any table"
        );
    }
    let invalid = AccountEnrollmentCredentials::for_read("invalid-account-session").unwrap();
    assert!(
        matches!(
            native_policy_current(store, &policy, &invalid, selector).await,
            Err(NativePolicyWorkflowError::AuthenticationInvalid)
        ),
        "invalid Auth acquired manager context"
    );
    assert!(*before == all_rows(pool).await);
    for (at, fault) in [(1, false), (2, false), (1, true), (2, true)] {
        let injected = ObservedRead::new(authority.clone(), Some((at, fault)));
        let refused = native_policy_current(store, &injected, a, selector).await;
        assert!(
            if fault {
                matches!(refused, Err(NativePolicyWorkflowError::Unavailable))
            } else {
                matches!(refused, Err(NativePolicyWorkflowError::NotFound))
            },
            "initial/final Cedar refusal released manager context"
        );
        injected.count(at);
        assert!(
            *before == all_rows(pool).await,
            "initial/final Cedar refusal changed any table"
        );
    }
    let revoke = NativePolicyCommand::from(
        NativeCompanyInformationCommandV1::revoke(
            Uuid::new_v4(),
            company,
            authority.epoch(),
            "아직 지원하지 않는 취소 문맥".to_owned(),
            parent,
        )
        .unwrap(),
    );
    assert!(matches!(
        native_policy_current(
            store,
            &policy,
            a,
            NativePolicyCommandRef::from_command(&revoke)
        )
        .await,
        Err(NativePolicyWorkflowError::Unavailable)
    ));
    assert!(matches!(
        native_policy_form(store, &policy, a, selector).await,
        Err(NativePolicyWorkflowError::Unavailable)
    ));
    assert!(matches!(
        native_policy_validation_form(store, &policy, a, selector).await,
        Err(NativePolicyWorkflowError::Unavailable)
    ));
    assert!(matches!(
        accept_native_policy_command(store, &policy, a, command, &TraceContext::generate()).await,
        Err(NativePolicyWorkflowError::Unavailable)
    ));
    assert!(matches!(
        execute_native_policy_command(store, &policy, a, selector, &TraceContext::generate()).await,
        Err(NativePolicyWorkflowError::Unavailable)
    ));
    assert!(matches!(
        native_policy_command_status(store, &policy, a, selector).await,
        Err(NativePolicyWorkflowError::Unavailable)
    ));
    assert!(matches!(
        submit_native_policy_command(store, &policy, a, command, &TraceContext::generate()).await,
        Err(NativePolicyWorkflowError::Unavailable)
    ));
    assert!(
        *before == all_rows(pool).await,
        "inactive mode acquired proof/input/receipt or other durable effect"
    );
}
