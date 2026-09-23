// Include inside native_policy_startup_tests. Real HTTP/WebAuthn Company
// fixture and actual native policy owner; no direct business-row insertion.
// The short-lived JWT below is an explicit expiration fault, not fixture truth.
use console_identity_application::company_policy::{
    CompanyPolicyDecision as ValidationDecision,
    CompanyPolicyDecisionPort as ValidationDecisionPort,
    NativeBootstrapRequestV1 as ValidationRequest,
    business::{MANIFEST as VALIDATION_MANIFEST, NativeBusinessOperationV1 as ValidationOperation},
    workflow::{
        NativePolicyScopeRequest as ValidationScopeRequest,
        NativePolicyWorkflowScope as ValidationScope, NativePolicyWorkflowStore as ValidationStore,
        native_policy_command_status as validation_status, native_policy_validation_form,
    },
};

#[sqlx::test(migrations = false)]
async fn native_policy_validation_form_retains_original_proof_and_rechecks_expiry_without_writes(
    pool: PgPool,
) {
    let (app, key, state) = configured_fixture(&pool, true).await;
    let mut cleanup_runtime = None;
    let outcome = AssertUnwindSafe(async {
        let (app, cookies, created) = create_owned_company(&pool, app).await;
        let config = account_browser_config(&pool, app._artifacts.root.clone(), &key);
        let (verifier, issuer, ttl) = bindings(&config);
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        cleanup_runtime = Some(runtime.clone());
        let identity: (String, String, bool, bool) = sqlx::query_as(
            "SELECT session_user::text,current_user::text,rolsuper,rolbypassrls FROM pg_roles WHERE rolname=current_user",
        ).fetch_one(&runtime).await.unwrap();
        assert_eq!(identity, ("console_rt".into(), "console_rt".into(), false, false));
        let store = PgOrgStore::new(runtime.clone()).with_native_account_policy(
            verifier.clone(), issuer.clone(), ttl,
        );
        let policy = CompanyPolicy::new().unwrap();
        let read = read_credentials(&cookies);
        let company = OrgId::from_uuid(created.company);
        let install = NativeCompanyBusinessCommandV1::install(Uuid::new_v4(), company, 1).unwrap();
        let install_form = native_policy_form(
            &store, &policy, &read, NativePolicyCommandRef::from_command(&install),
        ).await.unwrap();
        let install_credentials = AccountEnrollmentCredentials::for_mutation(
            &cookies.0[ACCESS], install_form.proof.as_str(),
        ).unwrap();
        let installed = submit_native_policy_command(
            &store, &policy, &install_credentials, &install, &TraceContext::generate(),
        ).await.unwrap();
        assert!(matches!(installed.terminal.outcome,
            NativePolicyOutcome::Committed(console_identity_application::company_policy::workflow::NativePolicyEffect::Installed { .. })));

        let selector = NativePolicyCommandRef::new(company, Uuid::new_v4(), ValidationOperation::Grant).unwrap();
        let original = native_policy_form(&store, &policy, &read, selector).await.unwrap();
        assert_eq!(original.view.company_epoch, 2);
        assert!(original.view.installed_object_type_id.is_some());
        assert!(original.view.assignment.is_none());
        let credentials = AccountEnrollmentCredentials::for_mutation(
            &cookies.0[ACCESS], original.proof.as_str(),
        ).unwrap();
        let before = all_rows(&pool).await;
        let retained = native_policy_validation_form(&store, &policy, &credentials, selector).await.unwrap();
        assert_eq!(retained.view, original.view, "validation replaced original selector/current scope");
        assert!(retained.proof.as_str() == original.proof.as_str(), "validation minted/replaced submitted proof");
        assert_eq!(retained.proof.expires_at(), original.proof.expires_at());
        assert!(before == all_rows(&pool).await, "validation wrote durable state");
        assert!(matches!(native_policy_form(&store, &policy, &credentials, selector).await,
            Err(NativePolicyWorkflowError::CsrfInvalid)), "ordinary GET Form must not mint from mutation credentials");
        assert!(matches!(validation_status(&store, &policy, &read, selector).await.unwrap(),
            console_identity_application::company_policy::workflow::NativePolicyStatus::NotVisible));
        assert!(before == all_rows(&pool).await, "validation produced accepted/terminal history");

        // Real cryptographic/live-owner checks: these are not transport-only refusals.
        assert!(matches!(native_policy_validation_form(&store, &policy, &read, selector).await,
            Err(NativePolicyWorkflowError::CsrfInvalid)));
        let corrupt = AccountEnrollmentCredentials::for_mutation(&cookies.0[ACCESS], "not-a-signed-proof").unwrap();
        assert!(matches!(native_policy_validation_form(&store, &policy, &corrupt, selector).await,
            Err(NativePolicyWorkflowError::CsrfInvalid)));
        assert!(before == all_rows(&pool).await);

        // Another actual Account's signed proof cannot bind to the operator session.
        let (_, other_cookies) = enrolled(&app).await;
        let other_proof = proof(&app, &other_cookies).await;
        let mismatched = AccountEnrollmentCredentials::for_mutation(&cookies.0[ACCESS], &other_proof).unwrap();
        let before_mismatch = all_rows(&pool).await;
        assert!(matches!(native_policy_validation_form(&store, &policy, &mismatched, selector).await,
            Err(NativePolicyWorkflowError::CsrfInvalid)));
        assert!(before_mismatch == all_rows(&pool).await);

        // Establish genuine live session material in a completed read scope;
        // no Account lock is carried into the following Group-first owner scope.
        let before_expiry = all_rows(&pool).await;
        let mut material_tx = runtime.begin().await.unwrap();
        let session = read.read_session_in_tx(&mut material_tx, &verifier, ttl).await.unwrap();
        let now: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(material_tx.as_mut()).await.unwrap();
        material_tx.rollback().await.unwrap();
        let expires = time::OffsetDateTime::from_unix_timestamp(now.unix_timestamp() + 8).unwrap();
        assert!(expires < session.family_expires_at);
        let short = issuer.issue_account_csrf_token(console_platform_auth::AccountCsrfTokenInput {
            account_id: session.account_id,
            session_id: session.session_id,
            security_generation: session.security_generation,
            issued_at: now,
            family_expires_at: expires,
        }).unwrap();
        let expiring = AccountEnrollmentCredentials::for_mutation(&cookies.0[ACCESS], short.as_str()).unwrap();
        let mut scope = store.lock(&expiring, ValidationScopeRequest::ValidationForm(selector)).await.unwrap();
        let source = scope.authority().source();
        let request = ValidationRequest::new(company, source.current_group_id,
            console_identity_application::company_policy::AccountId::from_uuid(source.administrative_account_id).unwrap(),
            ValidationOperation::Grant, VALIDATION_MANIFEST).unwrap();
        assert_eq!(policy.decide_native_bootstrap(scope.authority(), &request).unwrap(), ValidationDecision::Allow);
        let form = scope.form().await.unwrap();
        assert!(form.proof.as_str() == short.as_str(), "initial owner changed original expiring proof");
        assert_eq!(form.proof.expires_at(), expires);
        // The positive control proves the real form owner returned before expiry.
        let observed: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()").fetch_one(&pool).await.unwrap();
        assert!(observed < expires, "expiration prerequisite reached too late");
        tokio::time::timeout(std::time::Duration::from_secs(12), async {
            loop {
                let observed: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()").fetch_one(&pool).await.unwrap();
                if observed >= expires { break; }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        }).await.expect("actual database clock did not cross checked proof expiry");
        assert!(matches!(scope.finish(&policy, Some(&form.proof)).await,
            Err(NativePolicyWorkflowError::CsrfInvalid)), "retained owner accepted expired original proof at finish");
        assert!(before_expiry == all_rows(&pool).await, "expired validation changed durable state");

        // Recovery obtains a normal authorized GET proof; validation still returns
        // that exact proof and selector, with no command acceptance or mutation.
        let fresh = native_policy_form(&store, &policy, &read, selector).await.unwrap();
        let recovered_credentials = AccountEnrollmentCredentials::for_mutation(&cookies.0[ACCESS], fresh.proof.as_str()).unwrap();
        let recovered = native_policy_validation_form(&store, &policy, &recovered_credentials, selector).await.unwrap();
        assert_eq!(recovered.view.selector, selector);
        assert!(recovered.proof.as_str() == fresh.proof.as_str(), "recovery changed original proof");
        assert!(before_expiry == all_rows(&pool).await);
    }).catch_unwind().await;
    if let Some(runtime) = cleanup_runtime {
        runtime.close().await;
    }
    close_states(&[state], outcome).await;
}
