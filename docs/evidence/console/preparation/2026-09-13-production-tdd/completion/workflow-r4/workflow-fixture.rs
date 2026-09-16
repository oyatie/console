//! Source proposal over ordinary owners. No product code, SQL seed, verified ctor,
//! fake owner result or ignored fixture failure. See packet-contract.md.
use crate::{
    binder_sequence::{bind_and_seal, completed},
    native_fixture::{Fixture, NativeDeployment, TestResult, build_native_deployment},
    native_owner_producer::{ActualNativeOwnerResults, produce_native_and_review},
};
use console_governance_adapter_postgres::action30 as governance;
use console_ontology_adapter_postgres::action30 as ontology;
use console_ontology_application::action30::*;
use console_payroll_adapter_postgres::action30 as payroll;
use console_platform_group::action30 as group;
use console_platform_request_context::account::{
    AuthenticatedCompanyContext, AuthenticatedGroupContext,
};
use console_workflow_runtime_adapter_postgres::action30 as workflow;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

pub fn micros(t: time::OffsetDateTime) -> String {
    (t.unix_timestamp_nanos() / 1000).to_string()
}
pub fn typed<T: DeserializeOwned>(value: Value) -> TestResult<T> {
    Ok(serde_json::from_value(value)?)
}
pub fn value<T: Serialize>(input: &T) -> TestResult<Value> {
    Ok(serde_json::to_value(input)?)
}
pub fn assert_company(receipt: &ReceiptRef, attempt: &AttemptRef) {
    match receipt {
        ReceiptRef::Company { command, .. } => {
            assert_eq!(command.org_id, attempt.org_id);
            assert_eq!(command.command_id, attempt.command_id);
        }
        _ => panic!("Company effect returned a Group receipt"),
    }
}
pub fn assert_group(receipt: &ReceiptRef, command: &GroupCommandRef) {
    match receipt {
        ReceiptRef::Group {
            command: actual, ..
        } => assert_eq!(actual, command),
        _ => panic!("Group control returned Company receipt"),
    }
}
pub fn assert_refusal<T>(result: &OwnerExecution<T>) {
    // A technical error or PENDING/UNKNOWN is not the expected business refusal.
    match result {
        OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect { .. }) => {}
        _ => panic!("expected terminal no-effect refusal"),
    }
}

pub struct WorkflowFixture {
    pub deployment: NativeDeployment,
    pub config: Configuration,
}
pub struct Configuration {
    pub projection: GovernedRevision,
    pub replacement_projection: GovernedRevision,
    pub category: GovernedRevision,
    pub gate_policy: GovernedRevision,
    pub definition: GovernedRevision,
    pub coverage: GovernedRevision,
}
pub async fn fixture(pool: PgPool, scenario: &str, count: usize) -> TestResult<WorkflowFixture> {
    let deployment = build_native_deployment(pool, scenario, count).await?;
    assert_eq!(deployment.companies.len(), count);
    configure(deployment).await
}
pub async fn two_supported_fixture(pool: PgPool, scenario: &str) -> TestResult<WorkflowFixture> {
    let deployment =
        crate::two_subject_producer::build_two_supported_deployment(pool, scenario).await?;
    assert_eq!(deployment.companies.len(), 1);
    configure(deployment).await
}
async fn configure(mut deployment: NativeDeployment) -> TestResult<WorkflowFixture> {
    let company = &deployment.companies[0];
    let now = time::OffsetDateTime::now_utc();
    let until = now + time::Duration::days(1);
    let interval = json!({"from":micros(now),"until":micros(until)});
    // Exact action/property refs exist only after Company profile/schema enrollment.
    // These fixed ordinary reads return untrusted refs, never authority.
    let handler_action = ontology::read_registered_action::<payroll::CaseRespond>(
        &deployment.pool,
        &company.submitter,
    )
    .await?;
    let calculate_action = ontology::read_registered_action::<payroll::RunCalculate>(
        &deployment.pool,
        &company.submitter,
    )
    .await?;
    let fields =
        payroll::read_registered_review_projection_fields(&deployment.pool, &company.submitter)
            .await?;
    assert!(!fields.is_empty());
    let config_auth = &deployment.operator;
    // Ordinary current configuration CAS. The bootstrap has no preassigned heads:
    // reads return explicit ABSENT/current expected values, owners lock absence.
    let expected = payroll::configuration::read_projection_head(
        &deployment.pool,
        config_auth,
        company.org,
        "review-full",
    )
    .await?;
    let projection=payroll::configuration::publish_projection(&deployment.pool,config_auth,typed(json!({
        "command_id":Uuid::new_v4(),"org_id":company.org,"key":"review-full","expected":expected,
        "purpose":"NONPAYABLE_REVIEW","fields":fields,"validity":interval
    }))?).await?;
    let expected = payroll::configuration::read_projection_head(
        &deployment.pool,
        config_auth,
        company.org,
        "review-first-field",
    )
    .await?;
    let replacement=payroll::configuration::publish_projection(&deployment.pool,config_auth,typed(json!({
        "command_id":Uuid::new_v4(),"org_id":company.org,"key":"review-first-field","expected":expected,
        "purpose":"NONPAYABLE_REVIEW","fields":[fields[0].clone()],"validity":interval
    }))?).await?;
    let expected = payroll::configuration::read_issue_category_head(
        &deployment.pool,
        config_auth,
        company.org,
        "explanation",
    )
    .await?;
    let category=payroll::configuration::publish_issue_category(&deployment.pool,config_auth,typed(json!({
        "command_id":Uuid::new_v4(),"org_id":company.org,"key":"explanation","expected":expected,
        "label":"Explain the published calculation","allowed_claim_units":[],"validity":interval
    }))?).await?;
    let expected = governance::configuration::read_gate_policy_head(
        &deployment.pool,
        config_auth,
        company.org,
        "calculate-permit",
    )
    .await?;
    let gate=governance::configuration::publish_gate_policy(&deployment.pool,config_auth,typed(json!({
        "command_id":Uuid::new_v4(),"org_id":company.org,"key":"calculate-permit","expected":expected,
        "action":calculate_action,"purpose":"EXECUTION_GATE","independent_human_required":true,
        "decider_accounts":[deployment.accounts.submitter.account_id,deployment.accounts.reviewer.account_id],"max_validity_seconds":300,"validity":interval
    }))?).await?;
    let expected = workflow::configuration::read_definition_head(
        &deployment.pool,
        config_auth,
        company.org,
        "case-explanation",
    )
    .await?;
    let definition=workflow::configuration::publish_work_definition(&deployment.pool,config_auth,typed(json!({
        "command_id":Uuid::new_v4(),"org_id":company.org,"key":"case-explanation","expected":expected,
        "action":handler_action,"target_kind":"payroll_review_case","charge_units":1,
        "completion_criterion":"EXACT_COMPLETE_CASE_RESPONSE","max_obligation_seconds":86400,
        "validity":interval
    }))?).await?;
    let expected = workflow::configuration::read_coverage_head(
        &deployment.pool,
        config_auth,
        company.org,
        "case-handlers",
    )
    .await?;
    // Both real Accounts already have ordinary scoped handler permission. Coverage
    // and capacity choose feasibility; they confer no new action/field permission.
    let coverage=workflow::configuration::publish_work_coverage(&deployment.pool,config_auth,typed(json!({
        "command_id":Uuid::new_v4(),"org_id":company.org,"key":"case-handlers","expected":expected,
        "definition":definition.reference,"target_scope":{"kind":"COMPANY","org_id":company.org},
        "accounts":[{"account_id":deployment.accounts.reviewer.account_id,"priority":1},
                    {"account_id":deployment.accounts.transfer_recipient.account_id,"priority":2}],
        "validity":interval
    }))?).await?;
    for account in [
        deployment.accounts.reviewer.account_id,
        deployment.accounts.transfer_recipient.account_id,
    ] {
        let expected = group::configuration::read_routing_control(
            &deployment.pool,
            config_auth,
            deployment.group_id,
            account,
        )
        .await?;
        group::configuration::publish_capacity_and_availability(&deployment.pool,config_auth,typed(json!({
            "command_id":Uuid::new_v4(),"group_id":deployment.group_id,"account_id":account,"expected":expected,
            "limit_units":20,"availability":"AVAILABLE","validity":interval
        }))?).await?;
    }
    // This ordinary selection maps request handling to actual published definition;
    // it does not precreate cases, tasks, assignments or charges.
    let expected =
        payroll::configuration::read_case_routing_head(&deployment.pool, config_auth, company.org)
            .await?;
    payroll::configuration::publish_case_routing(
        &deployment.pool,
        config_auth,
        typed(json!({
            "command_id":Uuid::new_v4(),"org_id":company.org,"expected":expected,
            "definition":definition.reference,"coverage":coverage.reference
        }))?,
    )
    .await?;
    // Explicit Group controls are a separate ordinary policy grant; Company
    // business grants do not imply Group authority.
    let group_schema = console_identity_adapter_postgres::account13::read_group_policy_schema(
        &deployment.pool,
        config_auth,
        deployment.group_id,
    )
    .await?;
    let expected = console_identity_adapter_postgres::account13::read_group_policy_control(
        &deployment.pool,
        config_auth,
        deployment.group_id,
    )
    .await?;
    console_identity_adapter_postgres::account13::apply_group_grant_plan(&deployment.pool,config_auth,typed(json!({
        "command_id":Uuid::new_v4(),"group_id":deployment.group_id,"expected":expected,"plan":{
            "account_id":deployment.accounts.submitter.account_id,"scope":{"kind":"GROUP_CONTROL"},
            "actions":[group_schema.action("group.operation.create")?,group_schema.action("group.operation.activate")?,
                       group_schema.action("group.operation.resume")?,group_schema.action("group.operation.stop")?],
            "field_projection":group_schema.projection("group.operation.control_and_status")?,
            "valid_from":null,"valid_to":null,"reason":"Explicit selected Group operation control"}
    }))?).await?;
    // All configuration/policy writes precede refreshed opaque Company contexts.
    // Reuse actual owner tokens, never patch their generations in memory.
    for f in &mut deployment.companies {
        let selection =
            console_identity_adapter_postgres::account13::read_current_company_selection(
                &deployment.pool,
                &deployment.operator,
                f.org,
            )
            .await?;
        f.submitter = console_platform_request_context::account::resolve_company_context(
            &deployment.pool,
            &deployment.verifier,
            &deployment.accounts.submitter.account_access_token,
            &selection,
            &deployment.serving,
        )
        .await?;
        f.reviewer = console_platform_request_context::account::resolve_company_context(
            &deployment.pool,
            &deployment.verifier,
            &deployment.accounts.reviewer.account_access_token,
            &selection,
            &deployment.serving,
        )
        .await?;
    }
    Ok(WorkflowFixture {
        deployment,
        config: Configuration {
            projection: projection.reference,
            replacement_projection: replacement.reference,
            category: category.reference,
            gate_policy: gate.reference,
            definition: definition.reference,
            coverage: coverage.reference,
        },
    })
}
impl WorkflowFixture {
    pub async fn publication_snapshot(&self, commands: &[Uuid]) -> TestResult<Value> {
        let f = self.company();
        let snapshot: Value = sqlx::query_scalar(include_str!("publication-readback.sql"))
            .bind(f.org)
            .bind(commands)
            .fetch_one(&f.readback)
            .await?;
        let rows = snapshot["original_receipts"]
            .as_array()
            .ok_or("receipt rows missing")?;
        assert_eq!(
            rows.len(),
            commands.len(),
            "every original command must have exactly one actual ledger row"
        );
        let unique: std::collections::BTreeSet<_> = rows
            .iter()
            .map(|r| r["command_id"].as_str().expect("command UUID"))
            .collect();
        assert_eq!(unique.len(), rows.len());
        Ok(snapshot)
    }
    pub async fn execute_group_child(
        &self,
        attempt: &AttemptRef,
    ) -> TestResult<payroll::RunResult> {
        let f = self
            .deployment
            .companies
            .iter()
            .find(|f| f.org == attempt.org_id)
            .ok_or("actual child Company missing")?;
        let child = workflow::read_group_child_by_attempt(&f.pool, &f.submitter, attempt).await?;
        let claim = workflow::claim_group_child(&f.pool, &f.worker, &child.reference).await?;
        // Ordinary worker transport resolves original human+stored parent+claim,
        // then fixed-dispatches the SAME registered payroll owner with AttemptRef.
        // Opaque claim is actual owner-issued; no client constructs its generation.
        let initial =
            workflow::dispatch_claimed_group_calculation(&f.pool, &f.worker, &claim).await?;
        let result = crate::native_owner_producer::complete_native(
            &f.pool,
            &f.submitter,
            &f.worker,
            attempt,
            initial,
        )
        .await?;
        assert_company(&result.receipt, attempt);
        Ok(result)
    }
    pub async fn fresh_submitter_login(&mut self) -> TestResult<String> {
        let r = &self.deployment.runtime;
        crate::account_enrollment_producer::fresh_primary_login(
            &r.auth,
            &r.passkeys,
            &r.verifier,
            &r.origin,
            &mut self.deployment.accounts.submitter,
        )
        .await
    }
    pub fn company(&self) -> &Fixture {
        &self.deployment.companies[0]
    }
    pub async fn approved(&self) -> TestResult<ActualNativeOwnerResults> {
        let f = self.company();
        let actual = produce_native_and_review(
            &f.pool,
            &f.submitter,
            &f.reviewer,
            &f.worker,
            f.facts.choices.clone(),
        )
        .await?;
        assert!(!actual.decisions.is_empty());
        let current =
            payroll::read_run_control(&f.pool, &f.submitter, actual.created.run_id).await?;
        assert_eq!(current.review_cycle_id, Some(actual.completion.cycle_id));
        Ok(actual)
    }
    pub async fn publication(
        &self,
    ) -> TestResult<(
        payroll::PublicationResult,
        AttemptRef,
        ActualNativeOwnerResults,
    )> {
        let f = self.company();
        let approved = self.approved().await?;
        let targets = payroll::read_complete_publication_targets(
            &f.pool,
            &f.submitter,
            approved.created.run_id,
            &approved.completion,
        )
        .await?;
        assert!(!targets.is_empty());
        // Native subject producer binds the requester to this historical subject.
        // Resolve entitlement from actual owner; Account ID!=Person ID.
        let own =
            payroll::read_current_own_publication_targets(&f.pool, &f.submitter, &targets).await?;
        assert_eq!(
            own.len(),
            1,
            "fixture requires one actual own historical subject"
        );
        let input: payroll::PublicationRelease = typed(
            json!({"targets":own,"projection":self.config.projection,"purpose":"NONPAYABLE_REVIEW"}),
        )?;
        let selection =
            UntrustedActionTargetSelection::create_registered::<payroll::PublicationRelease>();
        let attempt = bind_and_seal(&f.pool, &f.submitter, &selection, &input).await?;
        let result =
            completed(payroll::payroll_review_release(&f.pool, &f.submitter, &attempt).await?)?;
        assert_company(&result.receipt, &attempt);
        assert_eq!(result.publications.len(), 1);
        Ok((result, attempt, approved))
    }
    pub fn item(&self, label: &str) -> TestResult<payroll::IssueItem> {
        typed(json!({
            "item_id":Uuid::new_v4(),"field_id":null,"category_revision":self.config.category,
            "description":label,"evidence":[],"expected_claim":null,"selected_line":null
        }))
    }
    pub async fn case(
        &self,
    ) -> TestResult<(
        payroll::CaseResult,
        AttemptRef,
        Vec<payroll::IssueItem>,
        payroll::PublicationRef,
    )> {
        let f = self.company();
        let (published, _, _) = self.publication().await?;
        let publication = published.publications[0].clone();
        let items = vec![self.item("Explain the first published amount")?];
        let input: payroll::CaseRequest = typed(
            json!({"target":{"kind":"PUBLICATION","publication":publication},
            "items":items,"predecessor_response_id":null,"narrative":"Please explain this immutable reviewed result."}),
        )?;
        let a = bind_and_seal(
            &f.pool,
            &f.submitter,
            &UntrustedActionTargetSelection::create_registered::<payroll::CaseRequest>(),
            &input,
        )
        .await?;
        let result = completed(payroll::payroll_review_request(&f.pool, &f.submitter, &a).await?)?;
        assert_company(&result.receipt, &a);
        Ok((result, a, items, publication))
    }
    pub async fn transfer(&self) -> TestResult<(workflow::TransferResult, AttemptRef, TaskRef)> {
        let f = self.company();
        let (case, _, _, _) = self.case().await?;
        let task = case.task.ok_or("configured real handler task missing")?;
        let assignment = workflow::read_assignment(&f.pool, &f.reviewer, &task).await?;
        assert_eq!(
            assignment.account_id,
            self.deployment.accounts.reviewer.account_id
        );
        let input: workflow::TransferSubmit = typed(json!({"targets":[{"task":task,
            "selection":{"kind":"NOMINATED","account_id":self.deployment.accounts.transfer_recipient.account_id},
            "responsibility":{"kind":"PERMANENT","from":micros(time::OffsetDateTime::now_utc())}}],
            "reason_revision":null,"context_evidence":[],"mandate":null,"explanation":"Explicit handover of this one accountable task"}))?;
        let a = bind_and_seal(
            &f.pool,
            &f.reviewer,
            &UntrustedActionTargetSelection::create_registered::<workflow::TransferSubmit>(),
            &input,
        )
        .await?;
        let result = completed(workflow::work_transfer_submit(&f.pool, &f.reviewer, &a).await?)?;
        assert_company(&result.receipt, &a);
        // Direct execution capability is granted normally at bootstrap. Submit's
        // ordinary policy path may authorize immediately; no test approve hook.
        Ok((result, a, task))
    }
    pub async fn editable_calculation(&self) -> TestResult<(DraftAck, Uuid)> {
        let f = self.company();
        let (run, _) = f.closed().await?;
        let input = payroll::RunCalculate {
            expected: payroll::read_run_control(&f.pool, &f.submitter, run.run_id).await?,
        };
        let binding = ontology::read_registered_draft_binding::<payroll::RunCalculate>(
            &f.pool,
            &f.submitter,
            &UntrustedActionTargetSelection::existing_run(run.run_id),
        )
        .await?;
        let started = completed(
            ontology::draft_start(
                &f.pool,
                &f.submitter,
                UnadmittedControl {
                    command_id: Uuid::new_v4(),
                    input: DraftStart {
                        action: binding.action,
                        target: binding.target,
                        schema: binding.schema,
                        custody: binding.custody,
                        intent_slot: binding.intent_slot,
                    },
                },
            )
            .await?,
        )?;
        let saved = completed(
            ontology::draft_save(
                &f.pool,
                &f.submitter,
                UnadmittedControl {
                    command_id: Uuid::new_v4(),
                    input: DraftSave {
                        draft_id: started.draft_id,
                        expected_revision_id: started.revision_id,
                        editing_token: started.editing_token,
                        patches: compile_registered_input_patches(
                            &binding.schema_snapshot,
                            &input,
                        )?,
                    },
                },
            )
            .await?,
        )?;
        Ok((saved, run.run_id))
    }
    pub async fn gate_request(
        &self,
    ) -> TestResult<(governance::GateResult, AttemptRef, Uuid, Uuid)> {
        let f = self.company();
        let (saved, run) = self.editable_calculation().await?;
        let now = time::OffsetDateTime::now_utc();
        let input: governance::GateRequest = typed(
            json!({"draft_id":saved.draft_id,"expected_revision_id":saved.revision_id,
            "editing_token":saved.editing_token,"gate_policy":self.config.gate_policy,"evidence":[],
            "requested_validity":{"from":micros(now),"until":micros(now+time::Duration::minutes(5))}}),
        )?;
        let a = bind_and_seal(
            &f.pool,
            &f.submitter,
            &UntrustedActionTargetSelection::create_registered::<governance::GateRequest>(),
            &input,
        )
        .await?;
        let result = completed(
            governance::governance_execution_gate_request(&f.pool, &f.submitter, &a).await?,
        )?;
        assert_company(&result.receipt, &a);
        Ok((result, a, saved.draft_id, run))
    }
    pub async fn group_auth(&self, token: &str) -> TestResult<AuthenticatedGroupContext> {
        use console_platform_request_context::account::resolve_group_context;
        Ok(resolve_group_context(
            &self.deployment.pool,
            &self.deployment.verifier,
            token,
            &UntrustedGroupSelection::existing(self.deployment.group_id),
            &self.deployment.serving,
        )
        .await?)
    }
    pub async fn group_identity<I: group::SealedGroupInput>(
        &self,
        auth: &AuthenticatedGroupContext,
    ) -> TestResult<group::GroupRequestIdentity> {
        Ok(group::GroupRequestIdentity {
            command: GroupCommandRef {
                group_id: self.deployment.group_id,
                command_id: Uuid::new_v4(),
            },
            action: group::read_registered_group_action::<I>(&self.deployment.pool, auth).await?,
            original_account_id: auth.account_id(),
        })
    }
    pub async fn preparing_group(
        &self,
    ) -> TestResult<(
        group::GroupResult,
        group::GroupRequestIdentity,
        Vec<(Uuid, payroll::RunCalculate)>,
    )> {
        let auth = self
            .group_auth(&self.deployment.accounts.submitter.account_access_token)
            .await?;
        let mut slots = Vec::new();
        let mut inputs = Vec::new();
        for f in &self.deployment.companies {
            let (run, _) = f.closed().await?;
            let input = payroll::RunCalculate {
                expected: payroll::read_run_control(&f.pool, &f.submitter, run.run_id).await?,
            };
            let binding = ontology::read_registered_draft_binding::<payroll::RunCalculate>(
                &f.pool,
                &f.submitter,
                &UntrustedActionTargetSelection::existing_run(run.run_id),
            )
            .await?;
            let membership =
                group::read_current_member_incarnation(&self.deployment.pool, &auth, f.org).await?;
            slots.push(typed::<group::GroupSlot>(json!({"slot_id":Uuid::new_v4(),"org_id":f.org,
                "membership_incarnation":membership.incarnation,"target":binding.target.existing_object()?,
                "action":binding.action,"expected":input.expected}))?);
            inputs.push((run.run_id, input));
        }
        let identity = self.group_identity::<group::GroupCreate>(&auth).await?;
        let input = group::GroupCreate {
            group_id: self.deployment.group_id,
            slots,
            reason: "Explicit selected Company calculations".into(),
        };
        let result = completed(
            group::group_operation_create(
                &self.deployment.pool,
                &auth,
                group::GroupRegisteredActionRequest {
                    identity: identity.clone(),
                    input,
                },
            )
            .await?,
        )?;
        assert_group(&result.receipt, &identity.command);
        Ok((result, identity, inputs))
    }
    pub async fn prepare_children(
        &self,
        parent: &group::GroupResult,
        inputs: &[(Uuid, payroll::RunCalculate)],
    ) -> TestResult<Vec<AttemptRef>> {
        let mut attempts = Vec::new();
        for (index, f) in self.deployment.companies.iter().enumerate() {
            let slot = parent
                .children
                .iter()
                .find(|c| c.org_id == f.org)
                .ok_or("actual Group slot missing")?;
            let selection = UntrustedActionTargetSelection::existing_run(inputs[index].0)
                .with_group_slot(parent.operation.clone(), slot.slot_id);
            let a = bind_and_seal(&f.pool, &f.submitter, &selection, &inputs[index].1).await?;
            // Observe actual prepared receipt through normal Group control owner
            // after Company transaction. Parent receives no fabricated ACK/result.
            group::observe_prepared_child(
                &self.deployment.pool,
                &self
                    .group_auth(&self.deployment.accounts.submitter.account_access_token)
                    .await?,
                &parent.operation,
                &a,
            )
            .await?;
            attempts.push(a);
        }
        Ok(attempts)
    }
}
