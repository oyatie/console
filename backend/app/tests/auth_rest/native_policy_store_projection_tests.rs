// Additive proposal: include inside company_setup, beside existing store smoke.
// Genuine HTTP/WebAuthn Account/Company history; not browser acceptance.
// Real A/B owners; projection corruption
// changes only returned typed composites and always restores source/ACL metadata.
mod native_policy_projection_corruption {
    use super::*;
    use console_identity_adapter_postgres::PgOrgStore;
    use console_identity_application::company_policy::{
        AccountId,
        business::NativeCompanyBusinessCommandV1,
        workflow::{
            NativePolicyCommandRef, NativePolicyEffect, NativePolicyOutcome, NativePolicyRejection,
            NativePolicyStatus, NativePolicyTerminalView, NativePolicyWorkflowError,
            accept_native_policy_command, execute_native_policy_command,
            native_policy_command_status, native_policy_form,
        },
    };
    use console_platform_auth::account::AccountEnrollmentCredentials;
    use console_platform_auth::{JwtIssuer, JwtSettings, JwtVerifier};
    use console_platform_authz::company_policy::CompanyPolicy;
    use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
    use futures::FutureExt;
    use std::panic::AssertUnwindSafe;

    const STATUS_ID: &str = "public.native_company_policy_status_v1(uuid,uuid,uuid,uuid,smallint)";
    const RETURN_ANCHOR: &str =
        " PERFORM set_config('app.current_org',coalesce(prior_org,''),true);\n RETURN NEXT;";

    async fn status_definition(pool: &PgPool) -> String {
        sqlx::query_scalar("SELECT pg_get_functiondef(to_regprocedure($1))")
            .bind(STATUS_ID)
            .fetch_one(pool)
            .await
            .unwrap()
    }
    async fn status_metadata(pool: &PgPool) -> Value {
        sqlx::query_scalar("SELECT jsonb_build_object('definition',pg_get_functiondef(p.oid),'owner',pg_get_userbyid(p.proowner),'acl',p.proacl::text,'config',p.proconfig,'security_definer',p.prosecdef,'language',p.prolang,'volatility',p.provolatile,'parallel',p.proparallel,'result',pg_get_function_result(p.oid)) FROM pg_proc p WHERE p.oid=to_regprocedure($1)")
            .bind(STATUS_ID).fetch_one(pool).await.unwrap()
    }
    async fn policy_profile(pool: &PgPool) -> String {
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql("SET LOCAL search_path=pg_catalog,pg_temp")
            .execute(tx.as_mut())
            .await
            .unwrap();
        let result = sqlx::query_scalar(include_str!(
            "../../../../ops/postgres-native-company-policy-custody-state.sql"
        ))
        .fetch_one(tx.as_mut())
        .await
        .unwrap();
        tx.rollback().await.unwrap();
        result
    }
    async fn mutation_proof(
        store: &PgOrgStore,
        policy: &CompanyPolicy,
        cookies: &Cookies,
        command: &NativeCompanyBusinessCommandV1,
    ) -> AccountEnrollmentCredentials {
        let read = AccountEnrollmentCredentials::for_read(&cookies.0[ACCESS]).unwrap();
        let form = native_policy_form(
            store,
            policy,
            &read,
            NativePolicyCommandRef::from_command(command),
        )
        .await
        .unwrap();
        AccountEnrollmentCredentials::for_mutation(&cookies.0[ACCESS], form.proof.as_str()).unwrap()
    }
    async fn accept(
        store: &PgOrgStore,
        policy: &CompanyPolicy,
        cookies: &Cookies,
        command: &NativeCompanyBusinessCommandV1,
    ) {
        let credentials = mutation_proof(store, policy, cookies, command).await;
        let accepted = accept_native_policy_command(
            store,
            policy,
            &credentials,
            command,
            &TraceContext::generate(),
        )
        .await
        .unwrap();
        assert!(accepted.inserted);
        assert!(matches!(
            accepted.status,
            NativePolicyStatus::AcceptedPending(_)
        ));
    }
    async fn execute(
        store: &PgOrgStore,
        policy: &CompanyPolicy,
        cookies: &Cookies,
        command: &NativeCompanyBusinessCommandV1,
    ) -> NativePolicyTerminalView {
        let credentials = mutation_proof(store, policy, cookies, command).await;
        let executed = execute_native_policy_command(
            store,
            policy,
            &credentials,
            NativePolicyCommandRef::from_command(command),
            &TraceContext::generate(),
        )
        .await
        .unwrap();
        assert!(executed.inserted);
        executed.terminal
    }
    async fn refused(
        pool: &PgPool,
        store: &PgOrgStore,
        policy: &CompanyPolicy,
        cookies: &Cookies,
        command: &NativeCompanyBusinessCommandV1,
        expected: NativePolicyWorkflowError,
        label: &str,
    ) {
        let before = all_rows(pool).await;
        let credentials = mutation_proof(store, policy, cookies, command).await;
        match accept_native_policy_command(
            store,
            policy,
            &credentials,
            command,
            &TraceContext::generate(),
        )
        .await
        {
            Err(actual) => assert_eq!(actual, expected, "{label}: wrong known refusal"),
            Ok(_) => panic!("{label}: invalid admission unexpectedly succeeded"),
        }
        assert!(
            before == all_rows(pool).await,
            "{label}: refusal wrote intake/audit/business state"
        );
    }
    // No generic injection framework: one exact known status owner, one exact
    // return anchor, one fixed test-supplied composite mutation, one restore.
    async fn projected_case(
        pool: &PgPool,
        store: &PgOrgStore,
        policy: &CompanyPolicy,
        read: &AccountEnrollmentCredentials,
        terminal: &NativePolicyTerminalView,
        label: &str,
        mutation: &str,
        positive: bool,
    ) {
        let original = status_definition(pool).await;
        let metadata = status_metadata(pool).await;
        let before = all_rows(pool).await;
        assert_eq!(
            original.matches(RETURN_ANCHOR).count(),
            1,
            "owner return shape drift"
        );
        let injected = original.replacen(
            RETURN_ANCHOR,
            &format!(
                " IF terminal.command_id='{}'::uuid THEN\n {}\n END IF;\n{}",
                terminal.accepted.input.command_id(),
                mutation,
                RETURN_ANCHOR
            ),
            1,
        );
        let outcome = AssertUnwindSafe(async {
            sqlx::raw_sql(sqlx::AssertSqlSafe(injected.as_str()))
                .execute(pool)
                .await
                .unwrap();
            assert_eq!(
                status_definition(pool).await,
                injected,
                "wrong function was injected"
            );
            assert_eq!(
                policy_profile(pool).await,
                "native_company_policy.profile_mismatch",
                "metadata fault must be observable"
            );
            let result = native_policy_command_status(
                store,
                policy,
                read,
                NativePolicyCommandRef::from_command(&terminal.accepted.input),
            )
            .await;
            if positive {
                assert_eq!(
                    result.unwrap(),
                    NativePolicyStatus::Terminal(terminal.clone()),
                    "{label}"
                );
            } else {
                assert!(
                    matches!(result, Err(NativePolicyWorkflowError::Unavailable)),
                    "{label}: decoder accepted corrupt typed projection"
                );
            }
            assert!(
                before == all_rows(pool).await,
                "{label}: status fault wrote business state"
            );
        })
        .catch_unwind()
        .await;
        // Restore even when injection/query/assertion panics. CREATE OR REPLACE
        // keeps function identity and ACL; compare complete original metadata.
        let restore = sqlx::raw_sql(sqlx::AssertSqlSafe(original.as_str()))
            .execute(pool)
            .await;
        assert!(
            restore.is_ok(),
            "failed to restore exact status owner: {label}"
        );
        assert_eq!(
            status_metadata(pool).await,
            metadata,
            "source/owner/ACL restore mismatch"
        );
        assert_eq!(
            policy_profile(pool).await,
            "native_company_policy.finalized"
        );
        assert!(
            before == all_rows(pool).await,
            "{label}: recovery changed business rows"
        );
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
        assert_eq!(
            native_policy_command_status(
                store,
                policy,
                read,
                NativePolicyCommandRef::from_command(&terminal.accepted.input)
            )
            .await
            .unwrap(),
            NativePolicyStatus::Terminal(terminal.clone()),
            "{label}: positive recovery failed"
        );
        assert!(
            before == all_rows(pool).await,
            "{label}: positive recovery wrote state"
        );
    }

    #[sqlx::test(migrations = false)]
    async fn native_policy_store_rejects_corrupt_terminal_projection_and_restores_owner(
        pool: PgPool,
    ) {
        super::native_policy_startup_tests::prepare_policy_ready_database(&pool).await;
        let artifacts = Artifacts::new();
        let key = SigningKey::random(&mut OsRng);
        let state = state_with_key(&pool, artifacts.root.clone(), &key).await;
        let app = Fixture {
            service: build_router(state.clone()),
            _artifacts: artifacts,
            pool: pool.clone(),
        };
        let outcome=AssertUnwindSafe(async {
        let (app, operator, cookies, startup, _) = designated_fixture(&pool, app).await;
        let (admin, _admin_cookies) = enrolled(&app).await;
        assert_ne!(operator.account, admin.account);
        let create_command = Uuid::new_v4();
        let enrollment_input = enrollment(create_command, admin.account);
        let enrollment_proof = proof(&app, &cookies).await;
        let created = committed(
            &submit(&app, &cookies, &enrollment_proof, &enrollment_input).await,
            StatusCode::CREATED,
            create_command,
            admin.account,
            false,
        );
        durable(&pool, &created, &enrollment_input, operator.account).await;
        let config = account_browser_config(&pool, app._artifacts.root.clone(), &key);
        let auth = config.auth_rest.unwrap();
        let ttl = auth.refresh_family_absolute_ttl;
        let settings = || JwtSettings {
            issuer: auth.jwt_issuer.clone(),
            audience: auth.jwt_audience.clone(),
            access_token_ttl: Duration::minutes(15),
        };
        let verifier =
            JwtVerifier::from_es256_public_pem(settings(), auth.jwt_public_key_pem.as_bytes()).unwrap();
        let issuer = JwtIssuer::from_es256_pem(
            settings(),
            auth.jwt_private_key_pem.as_bytes(),
            auth.jwt_public_key_pem.as_bytes(),
        )
        .unwrap();
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let identity: (String, String, bool, bool) = sqlx::query_as(
            "SELECT session_user::text,current_user::text,rolsuper,rolbypassrls FROM pg_roles WHERE rolname=current_user",
        ).fetch_one(&runtime).await.unwrap();
        assert_eq!(
            identity,
            ("console_rt".into(), "console_rt".into(), false, false)
        );
        let new_store = || {
            PgOrgStore::new(runtime.clone()).with_native_account_policy(
                verifier.clone(),
                issuer.clone(),
                ttl,
            )
        };
        let policy = console_platform_authz::company_policy::CompanyPolicy::new().unwrap();

        let store=new_store();
        let read=AccountEnrollmentCredentials::for_read(&cookies.0[ACCESS]).unwrap();
        let company=OrgId::from_uuid(created.company);
        let now:OffsetDateTime=sqlx::query_scalar("SELECT clock_timestamp()").fetch_one(&pool).await.unwrap();
        let expires=OffsetDateTime::from_unix_timestamp((now.unix_timestamp()/60+1440)*60).unwrap();
        let recipient=AccountId::from_uuid(admin.account).unwrap();
        let before_catalog=NativeCompanyBusinessCommandV1::grant(Uuid::new_v4(),company,1,recipient,None,expires).unwrap();
        refused(&pool,&store,&policy,&cookies,&before_catalog,NativePolicyWorkflowError::Conflict,"catalog_required").await;
        // Two real pending install commands; first execution wins. The other
        // actual B commits revision_conflict, without test-seeded receipts.
        let stale_install=NativeCompanyBusinessCommandV1::install(Uuid::new_v4(),company,1).unwrap();
        let install=NativeCompanyBusinessCommandV1::install(Uuid::new_v4(),company,1).unwrap();
        accept(&store,&policy,&cookies,&stale_install).await;
        accept(&store,&policy,&cookies,&install).await;
        let installed=execute(&store,&policy,&cookies,&install).await;
        assert!(matches!(installed.outcome,NativePolicyOutcome::Committed(NativePolicyEffect::Installed{..})));
        let rejected_install=execute(&store,&policy,&cookies,&stale_install).await;
        assert!(matches!(rejected_install.outcome,NativePolicyOutcome::Rejected(NativePolicyRejection::RevisionConflict)));
        let duplicate_catalog=NativeCompanyBusinessCommandV1::install(Uuid::new_v4(),company,2).unwrap();
        refused(&pool,&store,&policy,&cookies,&duplicate_catalog,NativePolicyWorkflowError::Conflict,"catalog_already_installed").await;
        // Whole-minute real future expiry; immutable bytes are never rounded later.
        let stale_grant=NativeCompanyBusinessCommandV1::grant(Uuid::new_v4(),company,2,recipient,None,expires).unwrap();
        let grant=NativeCompanyBusinessCommandV1::grant(Uuid::new_v4(),company,2,recipient,None,expires).unwrap();
        accept(&store,&policy,&cookies,&stale_grant).await;
        accept(&store,&policy,&cookies,&grant).await;
        let granted=execute(&store,&policy,&cookies,&grant).await;
        let assignment=match &granted.outcome {
            NativePolicyOutcome::Committed(NativePolicyEffect::Granted{assignment,recipient:actual,..})=>{assert_eq!(*actual,recipient);assignment.clone()},
            _=>panic!("real grant did not commit"),
        };
        let active_again=NativeCompanyBusinessCommandV1::grant(Uuid::new_v4(),company,3,recipient,Some(assignment.expectation),expires).unwrap();
        refused(&pool,&store,&policy,&cookies,&active_again,NativePolicyWorkflowError::Conflict,"assignment_active").await;
        let rejected_grant=execute(&store,&policy,&cookies,&stale_grant).await;
        assert!(matches!(rejected_grant.outcome,NativePolicyOutcome::Rejected(NativePolicyRejection::RevisionConflict)));
        let revoke=NativeCompanyBusinessCommandV1::revoke(Uuid::new_v4(),company,3,assignment.expectation).unwrap();
        accept(&store,&policy,&cookies,&revoke).await;
        let revoked=execute(&store,&policy,&cookies,&revoke).await;
        match &revoked.outcome {
            NativePolicyOutcome::Committed(NativePolicyEffect::Revoked{assignment:actual,recipient:actual_recipient,..})=>{
                assert_eq!(*actual_recipient,recipient);assert_eq!(actual.valid_from,assignment.valid_from);assert_eq!(actual.valid_until,assignment.valid_until);
            },_=>panic!("real revoke did not commit"),
        }
        let revoked_assignment=match &revoked.outcome {
            NativePolicyOutcome::Committed(NativePolicyEffect::Revoked{assignment,..})=>assignment.expectation,
            _=>unreachable!("checked actual revoke above"),
        };
        let revoke_again=NativeCompanyBusinessCommandV1::revoke(Uuid::new_v4(),company,4,revoked_assignment).unwrap();
        refused(&pool,&store,&policy,&cookies,&revoke_again,NativePolicyWorkflowError::Conflict,"assignment_not_active").await;
        let at:OffsetDateTime=sqlx::query_scalar("SELECT clock_timestamp()").fetch_one(&pool).await.unwrap();
        let past_minute=OffsetDateTime::from_unix_timestamp(at.unix_timestamp()/60*60).unwrap();
        let invalid_expiry=NativeCompanyBusinessCommandV1::grant(Uuid::new_v4(),company,4,recipient,Some(revoked_assignment),past_minute).unwrap();
        refused(&pool,&store,&policy,&cookies,&invalid_expiry,NativePolicyWorkflowError::InvalidInput,"grant_expiry_invalid").await;
        let baseline=all_rows(&pool).await;
        for terminal in [&installed,&rejected_install,&granted,&rejected_grant,&revoked] {
            assert_eq!(native_policy_command_status(&store,&policy,&read,NativePolicyCommandRef::from_command(&terminal.accepted.input)).await.unwrap(),NativePolicyStatus::Terminal(terminal.clone()));
        }
        assert!(baseline==all_rows(&pool).await,"positive status controls wrote state");
        // No-op injected projections prove metadata alteration alone is not the
        // refusal oracle; the actual typed row must reach the decoder.
        projected_case(&pool,&store,&policy,&read,&rejected_install,"unchanged rejection","NULL;",true).await;
        projected_case(&pool,&store,&policy,&read,&revoked,"unchanged revoke","NULL;",true).await;
        for (label,code) in [("install cannot have grant expiry rejection","grant_expiry_invalid"),("install cannot have recipient rejection","recipient_ineligible"),("intake expiry before deadline","intake_expired")] {
            projected_case(&pool,&store,&policy,&read,&rejected_install,label,&format!("terminal.result_code:='{code}';"),false).await;
        }
        projected_case(&pool,&store,&policy,&read,&rejected_grant,"grant-specific reason cannot skip epoch conflict","terminal.result_code:='recipient_ineligible';",false).await;
        let invalid_expiry=format!("terminal.result_code:='grant_expiry_invalid'; terminal.epoch_before:=2; terminal.epoch_after:=2; terminal.predecessor_receipt_id:='{}'::uuid;",installed.receipt_id);
        projected_case(&pool,&store,&policy,&read,&rejected_grant,"valid future grant expiry cannot be expiry-invalid",&invalid_expiry,false).await;
        let wrong_recipient=format!("terminal.recipient_account_id:='{}'::uuid;",operator.account);
        projected_case(&pool,&store,&policy,&read,&revoked,"revoke recipient must be original administrative Account",&wrong_recipient,false).await;
        projected_case(&pool,&store,&policy,&read,&revoked,"revoke cannot inherit future interval","terminal.assignment_valid_from:=terminal.executed_at+interval '1 microsecond';",false).await;
        assert!(baseline==all_rows(&pool).await,"corruption/recovery series changed acknowledged history");
        runtime.close().await;startup.close().await;
        }).catch_unwind().await;
        state.shutdown_realtime().await;
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }
}
