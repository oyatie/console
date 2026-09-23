// Include inside company_setup. Shared startup-test fixture owns current-policy
// custody preparation. No direct business writes.
mod native_policy_store_smoke {
    use super::*;
    use console_identity_adapter_postgres::PgOrgStore;
    use console_identity_application::company_policy::{
        AccountId,
        business::NativeCompanyBusinessCommandV1,
        workflow::{
            NativePolicyCommandRef, NativePolicyEffect, NativePolicyOutcome, NativePolicyStatus,
            accept_native_policy_command, execute_native_policy_command,
            native_policy_command_status, native_policy_form,
        },
    };
    use console_platform_auth::account::AccountEnrollmentCredentials;
    use console_platform_auth::{JwtIssuer, JwtSettings, JwtVerifier};
    use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
    use futures::FutureExt;
    use std::panic::AssertUnwindSafe;

    fn additions(
        before: &BTreeMap<String, String>,
        after: &BTreeMap<String, String>,
        expected: &[(&str, usize)],
        mutable_head: bool,
    ) -> BTreeMap<String, Vec<Value>> {
        assert!(before.keys().eq(after.keys()), "table census changed");
        let expected: BTreeMap<_, _> = expected.iter().copied().collect();
        let mut delta = BTreeMap::new();
        for (table, old) in before {
            if mutable_head && table == "company_authority_heads" {
                continue;
            }
            let added = added_rows(old, &after[table])
                .unwrap_or_else(|| panic!("acknowledged history changed in {table}"));
            assert_eq!(
                added.len(),
                *expected.get(table.as_str()).unwrap_or(&0),
                "unexpected effects in {table}"
            );
            if !added.is_empty() {
                delta.insert(table.clone(), added);
            }
        }
        assert_eq!(delta.len(), expected.len());
        delta
    }

    fn same_other_head_fields(
        before: &BTreeMap<String, String>,
        after: &BTreeMap<String, String>,
        company: Uuid,
        receipt: Uuid,
    ) {
        let old: Vec<Value> = serde_json::from_str(&before["company_authority_heads"]).unwrap();
        let mut new: Vec<Value> = serde_json::from_str(&after["company_authority_heads"]).unwrap();
        assert_eq!(old.len(), new.len());
        let selected: Vec<_> = new
            .iter_mut()
            .filter(|r| r["org_id"] == json!(company))
            .collect();
        assert_eq!(selected.len(), 1);
        let row = selected.into_iter().next().unwrap();
        assert_eq!(row["epoch"], json!(2));
        assert_eq!(row["current_policy_receipt_id"], json!(receipt));
        row["epoch"] = json!(1);
        row["current_policy_receipt_id"] = Value::Null;
        let normalize = |rows: Vec<Value>| -> BTreeSet<String> {
            rows.into_iter()
                .map(|v| serde_json::to_string(&v).unwrap())
                .collect()
        };
        assert!(
            normalize(old) == normalize(new),
            "unrelated head fields changed"
        );
    }

    #[sqlx::test(migrations = false)]
    async fn native_policy_store_commits_acceptance_then_effect_and_reopens_without_writes(
        pool: PgPool,
    ) {
        // Shared prerequisite must perform the exact reviewed atomic successor
        // installation before AppState startup. This test never self-installs SQL.
        super::native_policy_startup_tests::prepare_policy_ready_database(&pool).await;
        let artifacts = Artifacts::new();
        let key = SigningKey::random(&mut OsRng);
        let state = state_with_key(&pool, artifacts.root.clone(), &key).await;
        let app = Fixture {
            service: build_router(state.clone()),
            _artifacts: artifacts,
            pool: pool.clone(),
        };
        let outcome = AssertUnwindSafe(async {
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
        let actor = AccountId::from_uuid(operator.account).unwrap();
        let command = NativeCompanyBusinessCommandV1::install(
            Uuid::new_v4(),
            OrgId::from_uuid(created.company),
            1,
        )
        .unwrap();
        let selector = NativePolicyCommandRef::from_command(&command);
        let read = AccountEnrollmentCredentials::for_read(&cookies.0[ACCESS]).unwrap();
        let family: Uuid = signed_claims(&cookies.0[ACCESS], &key).unwrap()["sid"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let before = all_rows(&pool).await;
        let form = native_policy_form(&new_store(), &policy, &read, selector)
            .await
            .unwrap();
        assert_eq!(form.view.acting_account_id, actor);
        assert_eq!(
            *form.view.administrative_account_id.as_uuid(),
            admin.account
        );
        assert_eq!(form.view.company_epoch, 1);
        assert!(form.view.installed_object_type_id.is_none());
        assert!(form.view.assignment.is_none());
        assert!(before == all_rows(&pool).await, "form wrote durable state");
        let mutation =
            AccountEnrollmentCredentials::for_mutation(&cookies.0[ACCESS], form.proof.as_str())
                .unwrap();
        let trace = TraceContext::generate();
        let accepted =
            accept_native_policy_command(&new_store(), &policy, &mutation, &command, &trace)
                .await
                .unwrap();
        assert!(accepted.inserted);
        let pending = match accepted.status {
            NativePolicyStatus::AcceptedPending(view) => view,
            _ => panic!("new acceptance did not remain pending"),
        };
        assert_eq!(pending.input, command.clone().into());
        let after_a = all_rows(&pool).await;
        let a = additions(
            &before,
            &after_a,
            &[("native_company_policy_inputs_v1", 1), ("audit_events", 1)],
            false,
        );
        let input = &a["native_company_policy_inputs_v1"][0];
        assert_eq!(input.as_object().unwrap().len(), 13);
        assert_eq!(input["actor_account_id"], json!(operator.account));
        assert_eq!(input["command_id"], json!(command.command_id()));
        assert_eq!(input["org_id"], json!(created.company));
        assert_eq!(input["operation"], json!(1));
        assert_eq!(input["codec_version"], json!(1));
        assert_eq!(
            input["input_bytes"],
            json!(format!("\\x{}", hex::encode(command.encode(actor))))
        );
        assert_eq!(
            input["input_digest"],
            json!(format!(
                "\\x{}",
                hex::encode(sha2::Sha256::digest(command.encode(actor)))
            ))
        );
        assert_eq!(input["intake_receipt_id"], json!(pending.intake_receipt_id));
        assert_eq!(input["accepting_session_id"], json!(family));
        assert!(input["acceptance_backend_pid"].as_i64().unwrap() > 0);
        assert_eq!(
            a["audit_events"][0]["action"],
            json!("policy.company_command.accept")
        );
        let a_binding: bool = sqlx::query_scalar("SELECT i.accepted_at=$3 AND i.execution_not_after=$4 AND i.execution_not_after=i.accepted_at+interval '168 hours' AND EXISTS(SELECT 1 FROM public.audit_events e WHERE e.org_id=i.org_id AND e.actor=i.actor_account_id AND e.action='policy.company_command.accept' AND e.target_id=i.intake_receipt_id::text AND e.occurred_at=i.accepted_at AND e.xmin=i.acceptance_xid::xid) FROM public.native_company_policy_inputs_v1 i WHERE i.actor_account_id=$1 AND i.command_id=$2")
            .bind(operator.account).bind(command.command_id()).bind(pending.accepted_at).bind(pending.execution_not_after).fetch_one(&pool).await.unwrap();
        assert!(a_binding);
        assert_eq!(
            native_policy_command_status(&new_store(), &policy, &read, selector)
                .await
                .unwrap(),
            NativePolicyStatus::AcceptedPending(pending.clone())
        );
        assert!(
            after_a == all_rows(&pool).await,
            "pending reopen wrote durable state"
        );

        let executed =
            execute_native_policy_command(&new_store(), &policy, &mutation, selector, &trace)
                .await
                .unwrap();
        assert!(executed.inserted);
        let terminal = executed.terminal;
        assert_eq!(terminal.accepted, pending);
        assert_eq!((terminal.epoch_before, terminal.epoch_after), (1, 2));
        let object_type = match &terminal.outcome {
            NativePolicyOutcome::Committed(NativePolicyEffect::Installed { object_type_id }) => {
                *object_type_id
            }
            _ => panic!("installation did not commit"),
        };
        let after_b = all_rows(&pool).await;
        let b = additions(
            &after_a,
            &after_b,
            &[
                ("native_company_policy_receipts_v1", 1),
                ("audit_events", 2),
                ("ont_object_type_key_revisions", 1),
                ("ont_object_types", 1),
                ("ont_property_defs", 18),
                ("ont_action_types", 1),
                ("ont_builtin_catalog_installs", 1),
                ("native_company_catalog_installs", 1),
                ("native_company_object_refs", 1),
                ("native_company_action_refs", 1),
                ("native_company_property_refs", 18),
            ],
            true,
        );
        same_other_head_fields(&after_a, &after_b, created.company, terminal.receipt_id);
        let receipt = &b["native_company_policy_receipts_v1"][0];
        assert_eq!(receipt.as_object().unwrap().len(), 30);
        assert_eq!(receipt["actor_account_id"], json!(operator.account));
        assert_eq!(receipt["command_id"], json!(command.command_id()));
        assert_eq!(receipt["receipt_id"], json!(terminal.receipt_id));
        assert_eq!(receipt["intake_receipt_id"], input["intake_receipt_id"]);
        assert_eq!(receipt["input_digest"], input["input_digest"]);
        assert_eq!(receipt["execution_session_id"], json!(family));
        assert_ne!(
            receipt["effect_xid"], input["acceptance_xid"],
            "A/B shared a transaction"
        );
        assert!(receipt["effect_backend_pid"].as_i64().unwrap() > 0);
        assert_eq!(receipt["installed_object_type_id"], json!(object_type));
        assert_eq!(receipt["outcome"], json!("COMMITTED"));
        assert_eq!(receipt["result_code"], json!("installed"));
        let b_binding: bool = sqlx::query_scalar("SELECT r.executed_at=$3 AND r.epoch_before=1 AND r.epoch_after=2 AND r.committed_epoch=2 AND r.predecessor_receipt_id IS NULL AND r.catalog_version='native-payroll-collection-read-v1' AND r.manifest_digest=decode('07781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd','hex') AND num_nonnulls(r.recipient_account_id,r.role_id,r.role_revision,r.assignment_id,r.assignment_revision_before,r.assignment_revision_after,r.assignment_state_after,r.assignment_valid_from,r.assignment_valid_until)=0 AND (SELECT count(*) FROM public.audit_events e WHERE e.org_id=r.org_id AND e.actor=r.actor_account_id AND e.occurred_at=r.executed_at AND e.xmin=r.effect_xid::xid AND e.action IN ('policy.company_command.complete','ontology.object_type.builtin_install'))=2 FROM public.native_company_policy_receipts_v1 r WHERE r.actor_account_id=$1 AND r.command_id=$2")
            .bind(operator.account).bind(command.command_id()).bind(terminal.executed_at).fetch_one(&pool).await.unwrap();
        assert!(b_binding);
        let actions: BTreeSet<_> = b["audit_events"]
            .iter()
            .map(|r| r["action"].as_str().unwrap())
            .collect();
        assert_eq!(
            actions,
            BTreeSet::from([
                "policy.company_command.complete",
                "ontology.object_type.builtin_install"
            ])
        );
        let reopened = native_policy_command_status(&new_store(), &policy, &read, selector)
            .await
            .unwrap();
        assert_eq!(reopened, NativePolicyStatus::Terminal(terminal.clone()));
        let replay =
            execute_native_policy_command(&new_store(), &policy, &mutation, selector, &trace)
                .await
                .unwrap();
        assert!(!replay.inserted);
        assert_eq!(replay.terminal, terminal);
        assert!(
            after_b == all_rows(&pool).await,
            "terminal reopen/replay changed state"
        );
        // The complete census also proves original birth, assignments, roles,
        // Account generations and payroll business rows remained unchanged.
        runtime.close().await;
        startup.close().await;
        }).catch_unwind().await;
        state.shutdown_realtime().await;
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }
}
