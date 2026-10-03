// PROPOSAL ONLY — not admitted, reviewed, compiled, or executed.
// Intended include inside native_policy_startup_tests; uses its actual fixtures.
// Missing successor installation + proposed APIs are listed in accompanying note.
mod native_people_policy_mixed_owner_proposal {
    use super::*;
    use console_identity_application::company_policy::{
        AccountId,
        business::{
            NativeBusinessOperationV1, NativeCompanyBusinessCommandV1,
            PolicyAssignmentExpectationV1,
        },
        people_business::{DirectoryActionV1, NativePeoplePolicyCommandV1},
        workflow::{
            NativePolicyAssignmentState, NativePolicyCommand, NativePolicyCommandRef,
            NativePolicyEffect, NativePolicyOutcome, NativePolicyStatus, NativePolicyTerminalView,
            accept_native_policy_command, execute_native_policy_command,
            native_policy_command_status, native_policy_current, native_policy_form,
            submit_native_policy_command,
        },
    };
    use console_payroll_adapter_postgres::PgNativePayrollRunsReadPort;
    use console_payroll_application::read::{
        ListPayrollRuns, PayrollRunsReadError, list_payroll_runs,
    };
    use std::sync::Arc;

    // Shared closed union and the agreed generic selector/owner APIs.
    // The union already exists; owner/store integration remains a prerequisite.
    fn selector(input: &NativePolicyCommand) -> NativePolicyCommandRef {
        NativePolicyCommandRef::from_command(input)
    }
    fn wire(input: &NativePolicyCommand, actor: AccountId) -> (i16, Vec<u8>) {
        (input.codec_version(), input.encode(actor))
    }

    // Full census: no unnoticed side effects; old rows retain their original JSON
    // bytes through added_rows. Mutable projections are checked separately below.
    fn delta(
        before: &BTreeMap<String, String>,
        after: &BTreeMap<String, String>,
        expected: &[(&str, usize)],
        mutable: &[&str],
    ) {
        assert!(before.keys().eq(after.keys()), "table census changed");
        let counts: BTreeMap<_, _> = expected.iter().copied().collect();
        assert_eq!(counts.len(), expected.len(), "duplicate expected table");
        for name in counts.keys() {
            assert!(before.contains_key(*name));
        }
        for (table, old) in before {
            if mutable.contains(&table.as_str()) {
                continue;
            }
            let added = added_rows(old, &after[table])
                .unwrap_or_else(|| panic!("acknowledged history changed in {table}"));
            assert_eq!(
                added.len(),
                counts.get(table.as_str()).copied().unwrap_or(0),
                "unexpected effects in {table}"
            );
        }
    }

    fn rows(raw: &str) -> BTreeMap<String, Value> {
        serde_json::from_str::<Vec<Value>>(raw)
            .unwrap()
            .into_iter()
            .map(|row| {
                (
                    row["id"]
                        .as_str()
                        .unwrap_or_else(|| row["org_id"].as_str().unwrap())
                        .to_owned(),
                    row,
                )
            })
            .collect()
    }

    async fn transition(
        pool: &PgPool,
        (store, policy): (&PgOrgStore, &CompanyPolicy),
        cookies: &Cookies,
        actor: AccountId,
        input: NativePolicyCommand,
        epoch: u64,
        predecessor: Option<Uuid>,
    ) -> NativePolicyTerminalView {
        let request = selector(&input);
        let before_form = all_rows(pool).await;
        let read = read_credentials(cookies);
        let form = native_policy_form(store, policy, &read, request)
            .await
            .unwrap();
        assert_eq!(form.view.company_epoch, epoch);
        assert_eq!(form.view.acting_account_id, actor);
        assert!(
            before_form == all_rows(pool).await,
            "form wrote durable state"
        );
        let mutation =
            AccountEnrollmentCredentials::for_mutation(&cookies.0[ACCESS], form.proof.as_str())
                .unwrap();
        let trace = TraceContext::generate();
        let acceptance = accept_native_policy_command(store, policy, &mutation, &input, &trace)
            .await
            .unwrap();
        assert!(acceptance.inserted);
        let pending = match acceptance.status {
            NativePolicyStatus::AcceptedPending(v) => v,
            _ => panic!("acceptance did not remain pending"),
        };
        assert_eq!(pending.input, input);
        let accepted = all_rows(pool).await;
        delta(
            &before_form,
            &accepted,
            &[("native_company_policy_inputs_v1", 1), ("audit_events", 1)],
            &[],
        );
        let (codec, expected_bytes) = wire(&input, actor);
        let stored: (i16, Vec<u8>, Vec<u8>) = sqlx::query_as(
            "SELECT codec_version,input_bytes,input_digest FROM public.native_company_policy_inputs_v1 WHERE actor_account_id=$1 AND command_id=$2 AND org_id=$3")
            .bind(*actor.as_uuid()).bind(request.command_id()).bind(*request.company().as_uuid())
            .fetch_one(pool).await.unwrap();
        assert_eq!(stored.0, codec);
        assert!(stored.1 == expected_bytes, "wrong accepted command bytes");
        assert!(
            stored.2 == Sha256::digest(&expected_bytes).as_slice(),
            "wrong accepted digest"
        );
        assert_eq!(
            native_policy_command_status(store, policy, &read, request)
                .await
                .unwrap(),
            NativePolicyStatus::AcceptedPending(pending.clone())
        );
        assert!(
            accepted == all_rows(pool).await,
            "pending reopen wrote rows"
        );

        // submit re-accepts the identical command then executes its existing
        // durable acceptance. Exactly one input and one terminal may result.
        let execution = submit_native_policy_command(store, policy, &mutation, &input, &trace)
            .await
            .unwrap();
        assert!(execution.inserted);
        let terminal = execution.terminal;
        assert_eq!(terminal.accepted, pending);
        assert_eq!(
            (terminal.epoch_before, terminal.epoch_after),
            (epoch, epoch + 1)
        );
        // The request is the oracle: a wrong effect must not choose its own
        // permitted census. In particular, Create revoke cannot return Granted.
        let assignment_effect = match (input.operation(), &terminal.outcome) {
            (
                NativeBusinessOperationV1::Install,
                NativePolicyOutcome::Committed(NativePolicyEffect::Installed { object_type_id }),
            ) => {
                assert!(!object_type_id.is_nil());
                None
            }
            (
                NativeBusinessOperationV1::Grant,
                NativePolicyOutcome::Committed(NativePolicyEffect::Granted {
                    recipient,
                    assignment,
                    assignment_revision_before,
                }),
            ) => {
                assert_eq!(assignment.state, NativePolicyAssignmentState::Active);
                assert_eq!(Some(*recipient), input.recipient_account_id());
                assert_eq!(assignment.valid_from, terminal.executed_at);
                assert_eq!(Some(assignment.valid_until), input.expires_at());
                Some((*recipient, assignment, *assignment_revision_before))
            }
            (
                NativeBusinessOperationV1::Revoke,
                NativePolicyOutcome::Committed(NativePolicyEffect::Revoked {
                    recipient,
                    assignment,
                    assignment_revision_before,
                }),
            ) => {
                assert_eq!(assignment.state, NativePolicyAssignmentState::Revoked);
                let previous = form
                    .view
                    .assignment
                    .as_ref()
                    .expect("revoke needs existing assignment");
                assert_eq!(
                    (assignment.valid_from, assignment.valid_until),
                    (previous.valid_from, previous.valid_until)
                );
                Some((*recipient, assignment, Some(*assignment_revision_before)))
            }
            _ => panic!("returned effect does not match requested operation"),
        };
        if let Some((recipient, assignment, before)) = assignment_effect {
            assert_eq!(recipient, form.view.administrative_account_id);
            assert!(!assignment.role_id.is_nil());
            assert!(!assignment.expectation.assignment_id.is_nil());
            assert_eq!(assignment.expectation.role_revision, 1);
            let requested = input.assignment_expectation();
            assert_eq!(before, requested.map(|a| a.assignment_revision));
            assert_eq!(
                assignment.expectation.assignment_revision,
                requested.map_or(1, |a| a.assignment_revision + 1)
            );
            if let Some(requested) = requested {
                assert_eq!(
                    assignment.expectation.assignment_id,
                    requested.assignment_id
                );
                assert_eq!(
                    assignment.role_id,
                    form.view.assignment.as_ref().unwrap().role_id
                );
            }
        }
        let mut expected = vec![
            ("native_company_policy_receipts_v1", 1),
            ("audit_events", 1),
        ];
        let changed_assignment = match &terminal.outcome {
            NativePolicyOutcome::Committed(NativePolicyEffect::Installed { .. }) => {
                expected[1].1 = 2;
                expected.extend([
                    ("ont_object_type_key_revisions", 1),
                    ("ont_object_types", 1),
                    ("ont_property_defs", if codec == 1 { 18 } else { 6 }),
                    ("ont_action_types", if codec == 1 { 1 } else { 2 }),
                    ("ont_builtin_catalog_installs", 1),
                    ("native_company_catalog_installs", 1),
                    ("native_company_object_refs", 1),
                    ("native_company_action_refs", if codec == 1 { 1 } else { 2 }),
                    (
                        "native_company_property_refs",
                        if codec == 1 { 18 } else { 6 },
                    ),
                ]);
                None
            }
            NativePolicyOutcome::Committed(NativePolicyEffect::Granted {
                assignment,
                assignment_revision_before,
                ..
            }) => {
                expected.push(("policy_assignment_revisions", 1));
                if assignment_revision_before.is_none() {
                    let fields = match &input {
                        NativePolicyCommand::Payroll(_) => 18,
                        NativePolicyCommand::People(c)
                            if c.action() == Some(DirectoryActionV1::Read) =>
                        {
                            6
                        }
                        NativePolicyCommand::People(_) => 2,
                    };
                    expected.extend([
                        ("policy_roles", 1),
                        ("policy_role_revisions", 1),
                        ("policy_capability_clauses", 1),
                        ("policy_capability_clause_fields", fields),
                        ("user_role_assignments", 1),
                    ]);
                    None
                } else {
                    Some((
                        assignment.expectation.assignment_id,
                        assignment.expectation.assignment_revision,
                    ))
                }
            }
            NativePolicyOutcome::Committed(NativePolicyEffect::Revoked { assignment, .. }) => {
                expected.push(("policy_assignment_revisions", 1));
                Some((
                    assignment.expectation.assignment_id,
                    assignment.expectation.assignment_revision,
                ))
            }
            _ => panic!("expected committed transition"),
        };
        let after = all_rows(pool).await;
        let mutable = if changed_assignment.is_some() {
            vec!["company_authority_heads", "user_role_assignments"]
        } else {
            vec!["company_authority_heads"]
        };
        delta(&accepted, &after, &expected, &mutable);
        let mut new_heads = rows(&after["company_authority_heads"]);
        let row = new_heads
            .get_mut(&request.company().as_uuid().to_string())
            .unwrap();
        assert_eq!(row["epoch"], json!(epoch + 1));
        assert_eq!(row["current_policy_receipt_id"], json!(terminal.receipt_id));
        row["epoch"] = json!(epoch);
        row["current_policy_receipt_id"] = json!(predecessor);
        assert_eq!(
            new_heads,
            rows(&accepted["company_authority_heads"]),
            "unrelated head changed"
        );
        if let Some((id, revision)) = changed_assignment {
            let mut new_assignments = rows(&after["user_role_assignments"]);
            let row = new_assignments.get_mut(&id.to_string()).unwrap();
            assert_eq!(row["native_current_revision"], json!(revision));
            row["native_current_revision"] = json!(revision - 1);
            assert_eq!(
                new_assignments,
                rows(&accepted["user_role_assignments"]),
                "unrelated assignment changed"
            );
        }
        let binding: bool = sqlx::query_scalar(
            "SELECT r.codec_version=$4 AND r.predecessor_receipt_id IS NOT DISTINCT FROM $5 AND r.epoch_before=$6 AND r.epoch_after=$6+1 AND r.committed_epoch=r.epoch_after AND r.input_digest=i.input_digest AND r.intake_receipt_id=i.intake_receipt_id AND r.effect_xid<>i.acceptance_xid AND r.catalog_version=CASE $4 WHEN 1 THEN 'native-payroll-collection-read-v1' ELSE 'native-people-directory-v1' END FROM public.native_company_policy_receipts_v1 r JOIN public.native_company_policy_inputs_v1 i USING(actor_account_id,command_id,org_id) WHERE r.actor_account_id=$1 AND r.command_id=$2 AND r.org_id=$3")
            .bind(*actor.as_uuid()).bind(request.command_id()).bind(*request.company().as_uuid())
            .bind(codec).bind(predecessor).bind(epoch as i64).fetch_one(pool).await.unwrap();
        assert!(
            binding,
            "receipt/input/predecessor transaction binding failed"
        );
        let replay = execute_native_policy_command(store, policy, &mutation, request, &trace)
            .await
            .unwrap();
        assert!(!replay.inserted);
        assert_eq!(replay.terminal, terminal);
        assert!(
            after == all_rows(pool).await,
            "terminal execution replay wrote rows"
        );
        terminal
    }

    #[derive(Clone, Copy)]
    enum Step {
        PayrollInstall,
        PayrollGrant,
        PayrollRevoke,
        PeopleInstall,
        PeopleReadGrant,
        PeopleCreateGrant,
        PeopleCreateRevoke,
    }

    #[sqlx::test(migrations = false)]
    async fn mixed_people_payroll_policy_histories_preserve_bytes_reopen_and_current_reads(
        pool: PgPool,
    ) {
        let (app, key, state) =
            super::native_people_codec2_install_probe::configured_successor_fixture(&pool).await;
        let (app, operator, operator_cookies, startup, _) = designated_fixture(&pool, app).await;
        let (verifier, issuer, ttl) = bindings(&account_browser_config(
            &pool,
            app._artifacts.root.clone(),
            &key,
        ));
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let identity: (String, String, bool, bool) = sqlx::query_as(
            "SELECT session_user::text,current_user::text,rolsuper,rolbypassrls FROM pg_roles WHERE rolname=current_user")
            .fetch_one(&runtime).await.unwrap();
        assert_eq!(
            identity,
            ("console_rt".into(), "console_rt".into(), false, false)
        );
        let store = PgOrgStore::new(runtime.clone()).with_native_account_policy(
            verifier.clone(),
            issuer,
            ttl,
        );
        let policy = Arc::new(CompanyPolicy::new().unwrap());
        let actor = AccountId::from_uuid(operator.account).unwrap();
        let outcome = AssertUnwindSafe(async {
            use Step::*;
            let histories = [
                vec![PayrollInstall, PayrollGrant, PeopleInstall, PeopleReadGrant, PeopleCreateGrant,
                     PayrollRevoke, PayrollGrant, PeopleCreateRevoke, PeopleCreateGrant],
                vec![PeopleInstall, PeopleCreateGrant, PeopleReadGrant, PayrollInstall, PayrollGrant,
                     PeopleCreateRevoke, PayrollRevoke, PayrollGrant, PeopleCreateGrant],
            ];
            for steps in histories {
                let (admin, admin_cookies) = enrolled(&app).await;
                assert_ne!(admin.account, operator.account);
                let create_id = Uuid::new_v4();
                let enrollment_input = enrollment(create_id, admin.account);
                let csrf = proof(&app, &operator_cookies).await;
                let created = committed(&submit(&app, &operator_cookies, &csrf, &enrollment_input).await,
                    StatusCode::CREATED, create_id, admin.account, false);
                durable(&pool, &created, &enrollment_input, operator.account).await;
                let company = OrgId::from_uuid(created.company);
                let recipient = AccountId::from_uuid(admin.account).unwrap();
                let mut payroll_assignment: Option<PolicyAssignmentExpectationV1> = None;
                let mut people_create_assignment: Option<PolicyAssignmentExpectationV1> = None;
                let mut people_read_assignment: Option<PolicyAssignmentExpectationV1> = None;
                let mut people_installed = false;
                let mut people_create_active = false;
                let mut payroll_allowed = false;
                let mut expected_fields: BTreeMap<String, Vec<String>> = BTreeMap::new();
                let mut epoch = 1;
                let mut predecessor = None;
                let mut history: Vec<(NativePolicyCommandRef, NativePolicyTerminalView)> = Vec::new();
                for step in steps {
                    let now: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()").fetch_one(&pool).await.unwrap();
                    let until = OffsetDateTime::from_unix_timestamp((now.unix_timestamp() / 60 + 1440) * 60).unwrap();
                    let id = Uuid::new_v4();
                    let input = match step {
                        PayrollInstall => NativePolicyCommand::Payroll(NativeCompanyBusinessCommandV1::install(id, company, epoch).unwrap()),
                        PayrollGrant => NativePolicyCommand::Payroll(NativeCompanyBusinessCommandV1::grant(id, company, epoch, recipient, payroll_assignment, until).unwrap()),
                        PayrollRevoke => NativePolicyCommand::Payroll(NativeCompanyBusinessCommandV1::revoke(id, company, epoch, payroll_assignment.unwrap()).unwrap()),
                        PeopleInstall => NativePolicyCommand::People(NativePeoplePolicyCommandV1::install(id, company, epoch).unwrap()),
                        PeopleReadGrant => NativePolicyCommand::People(NativePeoplePolicyCommandV1::grant(id, company, epoch, DirectoryActionV1::Read, recipient, None, until).unwrap()),
                        PeopleCreateGrant => NativePolicyCommand::People(NativePeoplePolicyCommandV1::grant(id, company, epoch, DirectoryActionV1::Create, recipient, people_create_assignment, until).unwrap()),
                        PeopleCreateRevoke => NativePolicyCommand::People(NativePeoplePolicyCommandV1::revoke(id, company, epoch, DirectoryActionV1::Create, people_create_assignment.unwrap()).unwrap()),
                    };
                    let request = selector(&input);
                    let terminal = transition(&pool, (&store, policy.as_ref()), &operator_cookies,
                        actor, input, epoch, predecessor).await;
                    if let NativePolicyOutcome::Committed(NativePolicyEffect::Granted { assignment, .. }
                        | NativePolicyEffect::Revoked { assignment, .. }) = &terminal.outcome {
                        match step {
                            PayrollGrant | PayrollRevoke => payroll_assignment = Some(assignment.expectation),
                            PeopleCreateGrant | PeopleCreateRevoke => people_create_assignment = Some(assignment.expectation),
                            PeopleReadGrant => people_read_assignment = Some(assignment.expectation),
                            _ => {}
                        }
                    }
                    match step {
                        PayrollGrant => payroll_allowed = true,
                        PayrollRevoke => payroll_allowed = false,
                        PeopleInstall => people_installed = true,
                        PeopleCreateGrant => people_create_active = true,
                        PeopleCreateRevoke => people_create_active = false,
                        _ => {}
                    }
                    match step {
                        PayrollGrant => {
                            let mut fields: Vec<String> = ["id", "period_start", "period_end", "source_label", "status", "calculation_enabled", "created_by", "approved_by", "approved_at", "close_receipt", "submitted_by", "submitted_at", "decided_by", "decided_at", "decision_reason", "approval_ref", "created_at", "updated_at"].into_iter().map(|p| format!("pay_run.{p}")).collect();
                            fields.sort();
                            expected_fields.insert("payroll.collection.read".into(), fields);
                        }
                        PeopleReadGrant => {
                            expected_fields.insert("people.directory.read".into(), vec![
                                "person.directory_registered_at".into(), "person.employee_id".into(),
                                "person.employee_number".into(), "person.legal_name".into(),
                                "person.person_id".into(), "person.person_version".into()]);
                        }
                        PeopleCreateGrant => {
                            expected_fields.insert("people.directory.create".into(),
                                vec!["person.employee_number".into(), "person.legal_name".into()]);
                        }
                        _ => {}
                    }
                    // Assert exact field membership, not only the cardinalities
                    // checked by the census. Revocation preserves historical roles.
                    let fields: Vec<(String, Vec<String>)> = sqlx::query_as(
                        "SELECT a.action_key,array_agg(p.property_key ORDER BY p.property_key COLLATE \"C\") FROM public.policy_capability_clauses c JOIN public.policy_roles r ON r.org_id=c.org_id AND r.id=c.role_id JOIN public.native_company_action_refs a ON a.org_id=c.org_id AND a.object_type_id=c.action_object_type_id AND a.action_type_id=c.action_type_id JOIN public.policy_capability_clause_fields f ON (f.org_id,f.role_id,f.role_revision,f.clause_index)=(c.org_id,c.role_id,c.role_revision,c.clause_index) JOIN public.native_company_property_refs p ON p.org_id=f.org_id AND p.object_type_id=f.object_type_id AND p.property_id=f.property_id WHERE c.org_id=$1 AND r.policy_receipt_id IS NOT NULL GROUP BY r.id,a.action_key ORDER BY a.action_key")
                        .bind(created.company).fetch_all(&pool).await.unwrap();
                    assert_eq!(fields.len(), expected_fields.len(), "duplicate/missing business roles");
                    assert_eq!(fields.into_iter().collect::<BTreeMap<_, _>>(), expected_fields);
                    predecessor = Some(terminal.receipt_id);
                    epoch += 1;
                    history.push((request, terminal));

                    let before_reads = all_rows(&pool).await;
                    if people_installed {
                        let mut actual_ids = Vec::new();
                        for (action, expected_assignment, expected_state) in [
                            (DirectoryActionV1::Read, people_read_assignment, NativePolicyAssignmentState::Active),
                            (DirectoryActionV1::Create, people_create_assignment,
                                if people_create_active { NativePolicyAssignmentState::Active } else { NativePolicyAssignmentState::Revoked }),
                        ] {
                            let requested = NativePolicyCommand::People(NativePeoplePolicyCommandV1::grant(
                                Uuid::new_v4(), company, epoch, action, recipient, expected_assignment, until).unwrap());
                            let current = native_policy_current(&store, policy.as_ref(),
                                &read_credentials(&operator_cookies), NativePolicyCommandRef::from_command(&requested)).await.unwrap();
                            assert_eq!(current.company_epoch, epoch);
                            assert!(current.installed_object_type_id.is_some());
                            match (expected_assignment, current.assignment) {
                                (None, None) => {}
                                (Some(expected), Some(actual)) => {
                                    assert_eq!(actual.expectation, expected);
                                    assert_eq!(actual.state, expected_state);
                                    actual_ids.push((actual.role_id, actual.expectation.assignment_id));
                                }
                                _ => panic!("independent current action assignment differs from requested history"),
                            }
                        }
                        if actual_ids.len() == 2 {
                            assert_ne!(actual_ids[0].0, actual_ids[1].0, "Read/Create shared one role");
                            assert_ne!(actual_ids[0].1, actual_ids[1].1, "Read/Create shared one assignment");
                        }
                    }
                    for (old_request, old_terminal) in &history {
                        assert_eq!(native_policy_command_status(&store, policy.as_ref(),
                            &read_credentials(&operator_cookies), *old_request).await.unwrap(),
                            NativePolicyStatus::Terminal(old_terminal.clone()));
                    }
                    let identity = read_company_identity(&store, policy.as_ref(),
                        &read_credentials(&admin_cookies), company).await.unwrap();
                    assert_eq!(identity.org_id, company);
                    assert!(before_reads == all_rows(&pool).await, "reopening/Company read changed history");
                    let mut reader = PgNativePayrollRunsReadPort::new(runtime.clone(), verifier.clone(), ttl,
                        policy.clone(), read_credentials(&admin_cookies), company);
                    let page = list_payroll_runs(&mut reader, ListPayrollRuns { limit: None, offset: None }).await;
                    if payroll_allowed {
                        let page = page.unwrap();
                        assert_eq!(page.company.unwrap().id, company);
                        assert!(page.page.items.is_empty());
                        assert_eq!(page.page.total, 0);
                        let after = all_rows(&pool).await;
                        delta(&before_reads, &after, &[("audit_events", 1)], &[]);
                        let audits = added_rows(&before_reads["audit_events"], &after["audit_events"]).unwrap();
                        assert_eq!(audits[0]["action"], "payroll_run.list_read");
                        assert_eq!(audits[0]["actor"], json!(admin.account));
                        assert_eq!(audits[0]["org_id"], json!(created.company));
                    } else {
                        match page {
                            Err(PayrollRunsReadError::Authorization(error)) =>
                                assert_eq!(error.kind, console_kernel_core::ErrorKind::NotFound),
                            _ => panic!("expected policy denial, not missing prerequisites or malformed projection"),
                        }
                        assert!(before_reads == all_rows(&pool).await, "denied read changed rows");
                    }
                    let chain: Vec<(Uuid, i64, i64, Option<Uuid>, i16)> = sqlx::query_as(
                        "SELECT receipt_id,epoch_before,epoch_after,predecessor_receipt_id,codec_version FROM public.native_company_policy_receipts_v1 WHERE org_id=$1 AND outcome='COMMITTED' ORDER BY committed_epoch")
                        .bind(created.company).fetch_all(&pool).await.unwrap();
                    assert_eq!(chain.len(), history.len());
                    for (n, (receipt, before, after, prior, codec)) in chain.iter().enumerate() {
                        assert_eq!(*receipt, history[n].1.receipt_id);
                        assert_eq!((*before, *after), (n as i64 + 1, n as i64 + 2));
                        assert_eq!(*prior, n.checked_sub(1).map(|i| history[i].1.receipt_id));
                        assert_eq!(*codec, wire(&history[n].1.accepted.input, actor).0);
                    }
                }
            }
        }).catch_unwind().await;
        runtime.close().await;
        startup.close().await;
        state.shutdown_realtime().await;
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }
}
