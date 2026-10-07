//! Manager-current owner probe only. Full grant bytes are an unaccepted locator fixture.
use super::*;
use super::{manager_credentials as credentials, manager_policy::ObservedRead};
use console_identity_application::company_policy::{
    AccountId, CompanyPolicyDecision, CompanyPolicyDecisionPort, CompanyPolicyRequest,
    CompanyPolicyScope, CompanyPolicyStore, InitialCompanyAction,
    business::NativeCompanyBusinessCommandV1,
    company_information::{
        CompanyInformationAssignmentRefV1, MANIFEST, NativeCompanyInformationCommandV1,
        NativeCompanyInformationGrantV1,
    },
    workflow::{
        NativePolicyCommand, NativePolicyCommandRef, NativePolicyFormView,
        NativePolicyWorkflowError, native_policy_current,
    },
};
use console_kernel_core::OrgId;
use console_platform_authz::company_policy::CompanyPolicy;

pub(super) async fn probe(
    pool: &PgPool,
    config: &AppConfig,
    captured: &credentials::CapturedCookies,
    administrator: Uuid,
    recipient: Uuid,
    operator: Uuid,
    result: &Committed,
) {
    let runtime = credentials::runtime(pool).await;
    let outcome = std::panic::AssertUnwindSafe(async {
        let before = all_rows(pool).await;
        let a = credentials::credentials(&runtime, config, captured, administrator).await;
        let b = credentials::credentials(&runtime, config, captured, recipient).await;
        let o = credentials::credentials(&runtime, config, captured, operator).await;
        let store = credentials::store(runtime.clone(), config);
        let policy = CompanyPolicy::new().unwrap();
        let company = OrgId::from_uuid(result.company);
        let scope = store
            .lock_current(&a, company)
            .await
            .expect("existing A birth-current source prerequisite");
        let authority = scope
            .authority()
            .expect("actual distinct A manager birth source")
            .clone();
        assert_eq!(*authority.account().as_uuid(), administrator);
        assert_eq!(authority.company(), company);
        assert_eq!(authority.epoch(), 1);
        assert_eq!(authority.assignment_revision(), 1);
        assert_eq!(authority.role_revision(), 1);
        assert_eq!(authority.clauses().len(), 7);
        assert_eq!(
            authority
                .clauses()
                .iter()
                .map(|c| c.fields().len())
                .sum::<usize>(),
            16
        );
        assert_eq!(
            authority.name(),
            "브라우저로 연결한 독립 관리 회사 <연구 & 본사>"
        );
        assert_eq!(authority.slug(), format!("handoff-{}", operator.simple()));
        let read = authority
            .clauses()
            .iter()
            .find(|c| !c.delegable() && c.action_kind() == InitialCompanyAction::ReadPolicy)
            .unwrap();
        assert_eq!(read.fields().len(), 8);
        let request = CompanyPolicyRequest::new(
            company,
            read.action().object_type_id(),
            authority.assignment_id(),
            read.action().clone(),
            read.fields().to_vec(),
        )
        .unwrap();
        assert_eq!(
            policy.decide(&authority, &request).unwrap(),
            CompanyPolicyDecision::Allow,
            "real Cedar ReadPolicy positive prerequisite"
        );
        scope
            .finish()
            .await
            .expect("existing A birth-current scope prerequisite completion");
        let bootstrap =
            NativeCompanyBusinessCommandV1::install(Uuid::new_v4(), company, authority.epoch())
                .unwrap();
        let bootstrap_selector = NativePolicyCommandRef::from_command(&bootstrap);
        let original = native_policy_current(&store, &policy, &o, bootstrap_selector)
            .await
            .expect("actual O bootstrap Current positive prerequisite");
        assert_eq!(original.selector, bootstrap_selector);
        assert_eq!(original.group_id, result.group);
        assert_eq!(*original.acting_account_id.as_uuid(), operator);
        assert_eq!(*original.administrative_account_id.as_uuid(), administrator);
        let discover = authority
            .clauses()
            .iter()
            .find(|c| !c.delegable() && c.action_kind() == InitialCompanyAction::Discover)
            .unwrap();
        let identity = authority
            .clauses()
            .iter()
            .find(|c| !c.delegable() && c.action_kind() == InitialCompanyAction::ReadIdentity)
            .unwrap();
        assert_eq!(discover.fields(), identity.fields());
        assert_eq!(discover.action().manifest_digest(), &MANIFEST);
        assert_eq!(identity.action().manifest_digest(), &MANIFEST);
        let fields = discover
            .fields()
            .to_vec()
            .try_into()
            .expect("actual registered Company identity field pair");
        let now: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&runtime)
            .await
            .unwrap();
        assert_eq!(now.unix_timestamp_nanos() % 1000, 0);
        let parent = CompanyInformationAssignmentRefV1::new(
            authority.assignment_id(),
            authority.assignment_revision(),
        )
        .unwrap();
        let grant = NativeCompanyInformationGrantV1::new(
            AccountId::from_uuid(recipient).unwrap(),
            parent,
            now,
            now + time::Duration::hours(1),
            [discover.action().clone(), identity.action().clone()],
            fields,
        )
        .unwrap();
        let command = NativePolicyCommand::from(
            NativeCompanyInformationCommandV1::grant(
                Uuid::new_v4(),
                company,
                authority.epoch(),
                "회사 정보 접근 업무 문맥 확인".to_owned(),
                grant,
            )
            .unwrap(),
        );
        let encoded = command.encode(authority.account());
        let decoded = NativePolicyCommand::decode(4, &encoded)
            .expect("reviewed codec-four fixture roundtrip prerequisite");
        assert_eq!(decoded, (authority.account(), command.clone()));
        assert_eq!(decoded.1.encode(decoded.0), encoded);
        let selector = NativePolicyCommandRef::from_command(&command);
        assert_eq!(selector.codec_version(), 4);
        assert!(
            before == all_rows(pool).await,
            "prerequisite reads changed any public table"
        );
        let observed = ObservedRead::new(authority.clone(), None);
        let current = native_policy_current(&store, &observed, &a, selector).await;
        assert!(
            before == all_rows(pool).await,
            "manager Current changed any table including limiter/audit"
        );
        assert!(
            current.is_ok() || matches!(current, Err(NativePolicyWorkflowError::Unavailable)),
            "NON_ADMITTING_MANAGER_CURRENT_FAILURE"
        );
        assert!(
            current.is_ok(),
            "COMPANY_INFORMATION_MANAGER_CURRENT_UNAVAILABLE"
        );
        let expected = NativePolicyFormView {
            selector,
            group_id: result.group,
            company_epoch: authority.epoch(),
            acting_account_id: authority.account(),
            administrative_account_id: authority.account(),
            installed_object_type_id: Some(discover.action().object_type_id()),
            assignment: None,
        };
        assert_eq!(current.unwrap(), expected);
        observed.count(2);
        assert_eq!(
            native_policy_current(&store, &observed, &a, selector)
                .await
                .unwrap(),
            expected
        );
        observed.count(4);
        assert!(
            before == all_rows(pool).await,
            "manager reopening changed any table"
        );
        super::manager_controls::negatives(
            pool,
            &store,
            &authority,
            [&a, &b, &o],
            &command,
            &before,
        )
        .await;
    })
    .catch_unwind()
    .await;
    runtime.close().await;
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
}
