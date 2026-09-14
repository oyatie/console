//! Unique owner obligations from action-cases.tsv and WC16/GN16/RP16.
//! Shared draft/hash/auth invariants are not copied once per registration.
use crate::{
    binder_sequence::{ProducerStop, bind_and_seal, completed, explicit_new_intent},
    native_fixture::TestResult,
    native_owner_producer::produce_native_and_review,
    workflow_fixture::{assert_company, assert_group, assert_refusal, fixture, typed, value},
};
use console_ontology_application::action30::*;
use console_payroll_adapter_postgres::action30 as payroll;
use console_platform_group::action30 as group;
use console_workflow_runtime_adapter_postgres::action30 as workflow;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

fn assert_denied_binding(error: ProducerStop) {
    match error {
        ProducerStop::Observation(ResultObservation::DefinitiveNoEffect { reason, .. }) => {
            assert!(matches!(
                reason.code.as_str(),
                "not_found" | "not_authorized" | "cross_company_reference"
            ))
        }
        ProducerStop::Observation(ResultObservation::Unavailable { error }) => assert!(matches!(
            error.code.as_str(),
            "not_found" | "not_authorized"
        )),
        _ => panic!("expected precise owner authorization refusal, not fixture/transport failure"),
    }
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn publication_one_forbidden_company_target_prevents_all_local_publication(
    pool: PgPool,
) -> TestResult {
    let s = fixture(pool, "publication_forbidden_target", 2).await?;
    let local = s.company();
    let other = &s.deployment.companies[1];
    let a = s.approved().await?;
    let b = produce_native_and_review(
        &other.pool,
        &other.submitter,
        &other.reviewer,
        &other.worker,
        other.facts.choices.clone(),
    )
    .await?;
    let mut targets = payroll::read_complete_publication_targets(
        &local.pool,
        &local.submitter,
        a.created.run_id,
        &a.completion,
    )
    .await?;
    let remote = payroll::read_complete_publication_targets(
        &other.pool,
        &other.submitter,
        b.created.run_id,
        &b.completion,
    )
    .await?;
    assert_eq!(targets.len(), 1);
    assert_eq!(remote.len(), 1);
    assert_ne!(local.org, other.org);
    targets.extend(remote); // exact actual but forbidden cross-Company target LAST
    let before = s.publication_snapshot(&[]).await?;
    let input: payroll::PublicationRelease = typed(
        json!({"targets":targets,"projection":s.config.projection,"purpose":"NONPAYABLE_REVIEW"}),
    )?;
    match bind_and_seal(
        &local.pool,
        &local.submitter,
        &UntrustedActionTargetSelection::create_registered::<payroll::PublicationRelease>(),
        &input,
    )
    .await
    {
        Ok(attempt) => assert_refusal(
            &payroll::payroll_review_release(&local.pool, &local.submitter, &attempt).await?,
        ),
        Err(error) => assert_denied_binding(error),
    }
    assert_eq!(
        s.publication_snapshot(&[]).await?,
        before,
        "no first-target partial publication"
    );
    assert_eq!(before["publications"], json!([]));
    assert_eq!(before["slots"], json!([]));
    Ok(())
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn own_case_request_and_withdraw_do_not_follow_general_handler_permission(
    pool: PgPool,
) -> TestResult {
    let s = fixture(pool, "case_own_subject", 1).await?;
    let f = s.company();
    let (own, _, items, publication) = s.case().await?;
    let before:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(c) FROM payroll_review_request_controls c WHERE org_id=$1 ORDER BY request_id")
        .bind(f.org).fetch_all(&f.readback).await?;
    let input: payroll::CaseRequest = typed(
        json!({"target":{"kind":"PUBLICATION","publication":publication},
        "items":items,"predecessor_response_id":null,"narrative":"Handler cannot impersonate another historical subject"}),
    )?;
    match bind_and_seal(
        &f.pool,
        &f.reviewer,
        &UntrustedActionTargetSelection::create_registered::<payroll::CaseRequest>(),
        &input,
    )
    .await
    {
        Ok(a) => assert_refusal(&payroll::payroll_review_request(&f.pool, &f.reviewer, &a).await?),
        Err(e) => assert_denied_binding(e),
    }
    let withdraw = payroll::CaseWithdraw {
        expected: own.case.clone(),
        reason: "Unauthorized handler withdrawal".into(),
    };
    match bind_and_seal(
        &f.pool,
        &f.reviewer,
        &UntrustedActionTargetSelection::existing_case(own.case.case_id),
        &withdraw,
    )
    .await
    {
        Ok(a) => assert_refusal(
            &payroll::payroll_review_request_withdraw(&f.pool, &f.reviewer, &a).await?,
        ),
        Err(e) => assert_denied_binding(e),
    }
    let after:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(c) FROM payroll_review_request_controls c WHERE org_id=$1 ORDER BY request_id")
        .bind(f.org).fetch_all(&f.readback).await?;
    assert_eq!(after, before);
    Ok(())
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn stale_case_handler_attempt_cannot_resolve_after_real_transfer(pool: PgPool) -> TestResult {
    let s = fixture(pool, "case_stale_handler", 1).await?;
    let f = s.company();
    let (plan, _, task) = s.transfer().await?;
    let binding = workflow::read_case_work_binding(&f.pool, &f.reviewer, &task).await?;
    let case = payroll::read_case_control(&f.pool, &f.reviewer, binding.case_id).await?;
    let content = payroll::read_case_content(&f.pool, &f.reviewer, &case).await?;
    let dispositions=content.items.iter().map(|item|json!({"kind":"ANSWERED","item_id":item.item_id,"evidence":[],"answer":"Old handler response"})).collect::<Vec<_>>();
    let response: payroll::CaseRespond = typed(
        json!({"expected":case,"task":task,"dispositions":dispositions,"explanation":"Prepared under original accountable assignment"}),
    )?;
    let old = bind_and_seal(
        &f.pool,
        &f.reviewer,
        &UntrustedActionTargetSelection::existing_case(case.case_id),
        &response,
    )
    .await?;
    let move_attempt = bind_and_seal(
        &f.pool,
        &f.reviewer,
        &UntrustedActionTargetSelection::existing_transfer_plan(plan.plan.plan_id),
        &workflow::TransferExecute {
            plan: plan.plan,
            task: task.clone(),
        },
    )
    .await?;
    let moved =
        completed(workflow::work_transfer_execute(&f.pool, &f.reviewer, &move_attempt).await?)?;
    assert_company(&moved.receipt, &move_attempt);
    let before:Value=sqlx::query_scalar("SELECT to_jsonb(c) FROM payroll_review_request_controls c WHERE org_id=$1 AND request_id=$2")
        .bind(f.org).bind(case.case_id).fetch_one(&f.readback).await?;
    assert_refusal(&payroll::payroll_review_respond(&f.pool, &f.reviewer, &old).await?);
    let after:Value=sqlx::query_scalar("SELECT to_jsonb(c) FROM payroll_review_request_controls c WHERE org_id=$1 AND request_id=$2")
        .bind(f.org).bind(case.case_id).fetch_one(&f.readback).await?;
    assert_eq!(before, after);
    let assignment:Value=sqlx::query_scalar("SELECT to_jsonb(a) FROM workflow_assignment_revisions a JOIN workflow_work_controls c ON (a.org_id,a.task_id,a.epoch)=(c.org_id,c.task_id,c.assignment_epoch) WHERE a.org_id=$1 AND a.task_id=$2")
        .bind(f.org).bind(task.task_id).fetch_one(&f.readback).await?;
    assert_eq!(
        assignment["assignee_account_id"],
        json!(s.deployment.accounts.transfer_recipient.account_id)
    );
    assert!(
        assignment["epoch"]
            .as_i64()
            .ok_or("assignment epoch absent")?
            > task.assignment_epoch
    );
    Ok(())
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn group_create_rejects_another_members_actual_incarnation(pool: PgPool) -> TestResult {
    let s = fixture(pool, "group_wrong_incarnation", 2).await?;
    let (parent, _, _) = s.preparing_group().await?;
    let auth = s
        .group_auth(&s.deployment.accounts.submitter.account_access_token)
        .await?;
    let stored = group::read_group_operation(&s.deployment.pool, &auth, &parent.operation).await?;
    let mut input =
        group::read_original_group_create_input(&s.deployment.pool, &auth, &parent.operation)
            .await?;
    assert_eq!(input.slots.len(), 2);
    assert_ne!(
        input.slots[0].membership_incarnation,
        input.slots[1].membership_incarnation
    );
    input.slots[0].membership_incarnation = input.slots[1].membership_incarnation;
    let before:Value=sqlx::query_scalar("SELECT jsonb_build_object('parents',COALESCE((SELECT jsonb_agg(to_jsonb(p) ORDER BY operation_id) FROM group_operations p WHERE group_id=$1),'[]'::jsonb),'slots',COALESCE((SELECT jsonb_agg(to_jsonb(s) ORDER BY operation_id,slot_id) FROM group_operation_slots s WHERE group_id=$1),'[]'::jsonb))")
        .bind(s.deployment.group_id).fetch_one(&s.deployment.readback).await?;
    let identity = s.group_identity::<group::GroupCreate>(&auth).await?;
    assert_refusal(
        &group::group_operation_create(
            &s.deployment.pool,
            &auth,
            group::GroupRegisteredActionRequest { identity, input },
        )
        .await?,
    );
    let after = group::read_group_operation(&s.deployment.pool, &auth, &parent.operation).await?;
    assert_eq!(value(&after)?, value(&stored)?);
    let persisted:Value=sqlx::query_scalar("SELECT jsonb_build_object('parents',COALESCE((SELECT jsonb_agg(to_jsonb(p) ORDER BY operation_id) FROM group_operations p WHERE group_id=$1),'[]'::jsonb),'slots',COALESCE((SELECT jsonb_agg(to_jsonb(s) ORDER BY operation_id,slot_id) FROM group_operation_slots s WHERE group_id=$1),'[]'::jsonb))")
        .bind(s.deployment.group_id).fetch_one(&s.deployment.readback).await?;
    assert_eq!(
        persisted, before,
        "refused create cannot leave another parent or slot"
    );
    Ok(())
}

async fn blocked_by(pool: &PgPool, blocker: i32) -> TestResult<i32> {
    for _ in 0..200 {
        let pids:Vec<i32>=sqlx::query_scalar("SELECT pid FROM pg_stat_activity WHERE datname=current_database() AND $1=ANY(pg_blocking_pids(pid)) ORDER BY pid")
            .bind(blocker).fetch_all(pool).await?;
        if let Some(pid) = pids.first() {
            return Ok(*pid);
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    Err("no actual Company receipt lookup lock witness".into())
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn group_stop_preserves_unknown_when_authoritative_company_receipt_is_unavailable(
    pool: PgPool,
) -> TestResult {
    let admin = pool.clone();
    let s = fixture(pool, "group_unknown_stop", 2).await?;
    let (parent, _, inputs) = s.preparing_group().await?;
    let attempts = s.prepare_children(&parent, &inputs).await?;
    let auth = s
        .group_auth(&s.deployment.accounts.submitter.account_access_token)
        .await?;
    let identity = s.group_identity::<group::GroupActivate>(&auth).await?;
    let active = completed(
        group::group_operation_activate(
            &s.deployment.pool,
            &auth,
            group::GroupRegisteredActionRequest {
                identity,
                input: group::GroupActivate {
                    operation: parent.operation,
                    prepared_attempts: attempts.clone(),
                },
            },
        )
        .await?,
    )?;
    let actual = s.execute_group_child(&attempts[0]).await?;
    // Actual effect exists, but its Group ACK is not observed. Temporarily block
    // authoritative Company receipt lookup with a real database lock. No UNKNOWN
    // is inserted and no arbitrary canned result is passed to the Group owner.
    let before = s.deployment.companies[0].snapshot(inputs[0].0).await?;
    let mut barrier = admin.begin().await?;
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *barrier)
        .await?;
    sqlx::query("LOCK TABLE public.ont_action_command_receipts IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *barrier)
        .await?;
    let reading = group::read_group_operation(&s.deployment.pool, &auth, &active.operation);
    tokio::pin!(reading);
    tokio::select! {
        r=&mut reading=>{r?;panic!("Group read did not consult unavailable authoritative Company receipt")},
        blocked=blocked_by(&admin,pid)=>{assert_ne!(blocked?,pid);}
    }
    let unavailable =
        tokio::time::timeout(std::time::Duration::from_secs(10), &mut reading).await??;
    let first = unavailable
        .slots
        .iter()
        .find(|x| x.org_id == attempts[0].org_id)
        .ok_or("lost-ACK slot missing")?;
    assert_eq!(value(first)?["state"], "UNKNOWN");
    assert!(first.receipt.is_none());
    let stop = s.group_identity::<group::GroupStop>(&auth).await?;
    // Parent stop must finish with no Company business lock traversal. Current
    // uncertainty remains explicit; it cannot relabel the earlier effect absent.
    let stopped = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        group::group_operation_stop(
            &s.deployment.pool,
            &auth,
            group::GroupRegisteredActionRequest {
                identity: stop.clone(),
                input: group::GroupStop {
                    operation: active.operation.clone(),
                    reason: "Stop while Company reconciliation is unavailable".into(),
                },
            },
        ),
    )
    .await??;
    let stopped = completed(stopped)?;
    assert_group(&stopped.receipt, &stop.command);
    let first = stopped
        .children
        .iter()
        .find(|x| x.org_id == attempts[0].org_id)
        .ok_or("stopped slot missing")?;
    assert_eq!(value(first)?["state"], "UNKNOWN");
    assert!(first.receipt.is_none());
    barrier.rollback().await?;
    let restored =
        group::read_group_operation(&s.deployment.pool, &auth, &stopped.operation).await?;
    let first = restored
        .slots
        .iter()
        .find(|x| x.org_id == attempts[0].org_id)
        .ok_or("reconciled slot missing")?;
    assert_eq!(value(first)?["state"], "SUCCEEDED");
    assert_eq!(first.receipt.as_ref(), Some(&actual.receipt));
    assert_eq!(
        s.deployment.companies[0].snapshot(inputs[0].0).await?,
        before
    );
    Ok(())
}

async fn assignment_snapshot(s: &crate::workflow_fixture::WorkflowFixture) -> TestResult<Value> {
    Ok(sqlx::query_scalar(include_str!("assignment-readback.sql"))
        .bind(s.company().org)
        .bind(s.deployment.group_id)
        .fetch_one(&s.deployment.readback)
        .await?)
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn transfer_rechecks_actual_recipient_availability_after_seal(pool: PgPool) -> TestResult {
    let s = fixture(pool, "transfer_availability", 1).await?;
    let f = s.company();
    let (plan, _, task) = s.transfer().await?;
    let attempt = bind_and_seal(
        &f.pool,
        &f.reviewer,
        &UntrustedActionTargetSelection::existing_transfer_plan(plan.plan.plan_id),
        &workflow::TransferExecute {
            plan: plan.plan,
            task,
        },
    )
    .await?;
    let account = s.deployment.accounts.transfer_recipient.account_id;
    let expected = group::configuration::read_routing_control(
        &s.deployment.pool,
        &s.deployment.operator,
        s.deployment.group_id,
        account,
    )
    .await?;
    let now = time::OffsetDateTime::now_utc();
    group::configuration::publish_capacity_and_availability(&s.deployment.pool,&s.deployment.operator,typed(json!({
        "command_id":Uuid::new_v4(),"group_id":s.deployment.group_id,"account_id":account,"expected":expected,
        "limit_units":20,"availability":"UNAVAILABLE","validity":{"from":crate::workflow_fixture::micros(now),
            "until":crate::workflow_fixture::micros(now+time::Duration::days(1))}
    }))?).await?;
    let before = assignment_snapshot(&s).await?;
    assert_refusal(&workflow::work_transfer_execute(&f.pool, &f.reviewer, &attempt).await?);
    assert_eq!(
        assignment_snapshot(&s).await?,
        before,
        "unavailable nominee retains original accountable assignment and exact charge"
    );
    Ok(())
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn transfer_nomination_never_restores_revoked_recipient_domain_authority(
    pool: PgPool,
) -> TestResult {
    use console_identity_adapter_postgres::account13 as identity;
    let s = fixture(pool, "transfer_no_grant", 1).await?;
    let f = s.company();
    let (plan, _, task) = s.transfer().await?;
    let attempt = bind_and_seal(
        &f.pool,
        &f.reviewer,
        &UntrustedActionTargetSelection::existing_transfer_plan(plan.plan.plan_id),
        &workflow::TransferExecute {
            plan: plan.plan,
            task,
        },
    )
    .await?;
    let nominee = s.deployment.accounts.transfer_recipient.account_id;
    let grants = identity::read_current_company_grants_for_account(
        &s.deployment.pool,
        &s.deployment.operator,
        f.org,
        nominee,
    )
    .await?;
    assert!(!grants.is_empty());
    for grant in grants {
        let expected = identity::read_company_policy_control(
            &s.deployment.pool,
            &s.deployment.operator,
            f.org,
        )
        .await?;
        identity::revoke_company_grant(
            &s.deployment.pool,
            &s.deployment.operator,
            identity::RevokeCompanyGrant {
                command_id: Uuid::new_v4(),
                org_id: f.org,
                grant_id: grant.grant_id,
                expected,
                reason: "Revoke nominee domain authority before actual handover".into(),
            },
        )
        .await?;
    }
    let selection =
        identity::read_current_company_selection(&s.deployment.pool, &s.deployment.operator, f.org)
            .await?;
    let actor = console_platform_request_context::account::resolve_company_context(
        &s.deployment.pool,
        &s.deployment.verifier,
        &s.deployment.accounts.reviewer.account_access_token,
        &selection,
        &s.deployment.serving,
    )
    .await?;
    // Fresh original handler context prevents a stale Company epoch from masking
    // the candidate's independent domain feasibility denial.
    let before = assignment_snapshot(&s).await?;
    assert_refusal(&workflow::work_transfer_execute(&f.pool, &actor, &attempt).await?);
    assert_eq!(assignment_snapshot(&s).await?, before);
    let remaining = identity::read_current_company_grants_for_account(
        &s.deployment.pool,
        &s.deployment.operator,
        f.org,
        nominee,
    )
    .await?;
    assert!(
        remaining.is_empty(),
        "transfer cannot mint recipient rights"
    );
    Ok(())
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn group_command_replay_preserves_allocated_parent_and_rejects_changed_body(
    pool: PgPool,
) -> TestResult {
    let s = fixture(pool, "group_command_identity", 2).await?;
    let (parent, identity, inputs) = s.preparing_group().await?;
    let auth = s
        .group_auth(&s.deployment.accounts.submitter.account_access_token)
        .await?;
    let input =
        group::read_original_group_create_input(&s.deployment.pool, &auth, &parent.operation)
            .await?;
    let before:Value=sqlx::query_scalar("SELECT jsonb_build_object('parents',COALESCE((SELECT jsonb_agg(to_jsonb(p) ORDER BY operation_id) FROM group_operations p WHERE group_id=$1),'[]'::jsonb),'slots',COALESCE((SELECT jsonb_agg(to_jsonb(s) ORDER BY operation_id,slot_id) FROM group_operation_slots s WHERE group_id=$1),'[]'::jsonb),'receipts',COALESCE((SELECT jsonb_agg(to_jsonb(r) ORDER BY command_id) FROM group_operation_command_receipts r WHERE group_id=$1),'[]'::jsonb))")
        .bind(s.deployment.group_id).fetch_one(&s.deployment.readback).await?;
    let replay = completed(
        group::group_operation_create(
            &s.deployment.pool,
            &auth,
            group::GroupRegisteredActionRequest {
                identity: identity.clone(),
                input: input.clone(),
            },
        )
        .await?,
    )?;
    assert_group(&replay.receipt, &identity.command);
    assert_eq!(value(&replay)?, value(&parent)?);
    let mut changed = input;
    changed.reason = "Different body under exact original Group command".into();
    assert_refusal(
        &group::group_operation_create(
            &s.deployment.pool,
            &auth,
            group::GroupRegisteredActionRequest {
                identity,
                input: changed,
            },
        )
        .await?,
    );
    let after:Value=sqlx::query_scalar("SELECT jsonb_build_object('parents',COALESCE((SELECT jsonb_agg(to_jsonb(p) ORDER BY operation_id) FROM group_operations p WHERE group_id=$1),'[]'::jsonb),'slots',COALESCE((SELECT jsonb_agg(to_jsonb(s) ORDER BY operation_id,slot_id) FROM group_operation_slots s WHERE group_id=$1),'[]'::jsonb),'receipts',COALESCE((SELECT jsonb_agg(to_jsonb(r) ORDER BY command_id) FROM group_operation_command_receipts r WHERE group_id=$1),'[]'::jsonb))")
        .bind(s.deployment.group_id).fetch_one(&s.deployment.readback).await?;
    assert_eq!(
        before, after,
        "Group command replay cannot allocate another parent or rewrite its receipt"
    );
    for (f, (run, original)) in s.deployment.companies.iter().zip(inputs.iter()) {
        assert_eq!(
            payroll::read_run_control(&f.pool, &f.submitter, *run).await?,
            original.expected
        );
    }
    Ok(())
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn publication_valid_same_company_forbidden_field_prevents_all_targets(
    pool: PgPool,
) -> TestResult {
    use console_identity_adapter_postgres::account13 as identity;
    let s = crate::workflow_fixture::two_supported_fixture(
        pool,
        "publication_same_company_hidden_field",
    )
    .await?;
    let f = s.company();
    let approved = s.approved().await?;
    let targets = payroll::read_complete_publication_targets(
        &f.pool,
        &f.submitter,
        approved.created.run_id,
        &approved.completion,
    )
    .await?;
    assert_eq!(targets.len(), 2);
    assert_ne!(targets[0].subject, targets[1].subject);
    // Fixed owner read resolves actual stored source resources and immutable
    // registered type/schema/property identities. No ObjectKey variant invented.
    let a = payroll::read_publication_policy_resource(
        &f.pool,
        &f.submitter,
        &targets[0],
        &s.config.projection,
    )
    .await?;
    let b = payroll::read_publication_policy_resource(
        &f.pool,
        &f.submitter,
        &targets[1],
        &s.config.projection,
    )
    .await?;
    assert_eq!(a.object.org_id, f.org);
    assert_eq!(b.object.org_id, f.org);
    assert_ne!(a.object, b.object);
    assert!(!a.required_fields.is_empty());
    assert_eq!(a.required_fields, b.required_fields);
    assert_eq!(a.schema, b.schema);
    let account = s.deployment.accounts.reviewer.account_id;
    let schema =
        identity::read_company_policy_schema(&s.deployment.pool, &s.deployment.operator, f.org)
            .await?;
    let actions = schema.actions_for_publication_release()?;
    let full = schema.projection("native30.standard")?;
    let full_fields = identity::read_field_group_members(
        &s.deployment.pool,
        &s.deployment.operator,
        f.org,
        &full,
    )
    .await?;
    assert!(
        a.required_fields
            .iter()
            .all(|field| full_fields.contains(field))
    );
    let denied = a.required_fields[0].clone();
    let narrow_fields = full_fields
        .iter()
        .filter(|field| **field != denied)
        .cloned()
        .collect::<Vec<_>>();
    let control_fields = full_fields
        .iter()
        .filter(|field| !a.required_fields.contains(field))
        .cloned()
        .collect::<Vec<_>>();
    // The ordinary governed field-group publisher uses current CAS and the same
    // immutable revision store as normal policy authoring. No fixture-only flag.
    let mut projections = Vec::new();
    for (key, fields) in [
        ("release-controls", control_fields),
        ("release-one-field-denied", narrow_fields),
    ] {
        let expected =
            identity::read_field_group_head(&s.deployment.pool, &s.deployment.operator, f.org, key)
                .await?;
        let actual = identity::publish_field_group(
            &s.deployment.pool,
            &s.deployment.operator,
            identity::PublishFieldGroup {
                command_id: Uuid::new_v4(),
                org_id: f.org,
                key: key.into(),
                expected,
                fields,
                reason: "Explicit governed source-field scope for publication".into(),
            },
        )
        .await?;
        projections.push(actual.reference);
    }
    let grants = identity::read_current_company_grants_for_account(
        &s.deployment.pool,
        &s.deployment.operator,
        f.org,
        account,
    )
    .await?;
    assert!(!grants.is_empty());
    for grant in grants {
        let expected = identity::read_company_policy_control(
            &s.deployment.pool,
            &s.deployment.operator,
            f.org,
        )
        .await?;
        identity::revoke_company_grant(
            &s.deployment.pool,
            &s.deployment.operator,
            identity::RevokeCompanyGrant {
                command_id: Uuid::new_v4(),
                org_id: f.org,
                grant_id: grant.grant_id,
                expected,
                reason: "Replace actual broad grant with resource-correlated field grants".into(),
            },
        )
        .await?;
    }
    for (scope, field_projection) in [
        (identity::PolicyScope::Company, projections[0].clone()),
        (
            identity::PolicyScope::ExactResource {
                resource: a.object.clone(),
                schema: a.schema.clone(),
            },
            full,
        ),
        (
            identity::PolicyScope::ExactResource {
                resource: b.object.clone(),
                schema: b.schema.clone(),
            },
            projections[1].clone(),
        ),
    ] {
        let expected = identity::read_company_policy_control(
            &s.deployment.pool,
            &s.deployment.operator,
            f.org,
        )
        .await?;
        identity::apply_company_grant_plan(
            &s.deployment.pool,
            &s.deployment.operator,
            identity::ApplyCompanyGrantPlan {
                command_id: Uuid::new_v4(),
                org_id: f.org,
                expected,
                plan: identity::CompanyGrantPlan {
                    account_id: account,
                    scope,
                    actions: actions.clone(),
                    field_projection,
                    valid_from: None,
                    valid_to: None,
                    reason: "Exact per-resource publication disclosure".into(),
                },
            },
        )
        .await?;
    }
    let selection =
        identity::read_current_company_selection(&s.deployment.pool, &s.deployment.operator, f.org)
            .await?;
    let actor = console_platform_request_context::account::resolve_company_context(
        &s.deployment.pool,
        &s.deployment.verifier,
        &s.deployment.accounts.reviewer.account_access_token,
        &selection,
        &s.deployment.serving,
    )
    .await?;
    let before = s.publication_snapshot(&[]).await?;
    assert_eq!(before["publications"], json!([]));
    let input: payroll::PublicationRelease = typed(
        json!({"targets":targets,"projection":s.config.projection,"purpose":"NONPAYABLE_REVIEW"}),
    )?;
    let negative_selection = explicit_new_intent::<payroll::PublicationRelease>(
        &f.pool,
        &actor,
        &UntrustedActionTargetSelection::create_registered::<payroll::PublicationRelease>(),
    )
    .await?;
    match bind_and_seal(&f.pool, &actor, &negative_selection, &input).await {
        Ok(attempt) => {
            assert_refusal(&payroll::payroll_review_release(&f.pool, &actor, &attempt).await?)
        }
        Err(error) => assert_field_binding_refusal(error),
    }
    assert_eq!(
        s.publication_snapshot(&[]).await?,
        before,
        "valid first target cannot commit ahead of forbidden last source field"
    );
    // Positive control under EXACT SAME current authority: first target really is
    // publishable. This prevents an all-denied or broken-fixture false positive.
    let one: payroll::PublicationRelease = typed(
        json!({"targets":[targets[0].clone()],"projection":s.config.projection,"purpose":"NONPAYABLE_REVIEW"}),
    )?;
    let selected = explicit_new_intent::<payroll::PublicationRelease>(
        &f.pool,
        &actor,
        &UntrustedActionTargetSelection::create_registered::<payroll::PublicationRelease>(),
    )
    .await?;
    let attempt = bind_and_seal(&f.pool, &actor, &selected, &one).await?;
    let released = completed(payroll::payroll_review_release(&f.pool, &actor, &attempt).await?)?;
    assert_company(&released.receipt, &attempt);
    assert_eq!(released.publications.len(), 1);
    let original =
        payroll::read_publication_record(&f.pool, &actor, &released.publications[0]).await?;
    assert_eq!(original.content.subject, targets[0].subject);
    let after = s.publication_snapshot(&[attempt.command_id]).await?;
    assert_eq!(after["publications"].as_array().unwrap().len(), 1);
    assert_eq!(after["slots"].as_array().unwrap().len(), 1);
    Ok(())
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn transfer_capacity_conflict_preserves_two_real_task_charges(pool: PgPool) -> TestResult {
    use console_identity_adapter_postgres::account13 as identity;
    let s = fixture(pool, "transfer_capacity_conflict", 1).await?;
    let f = s.company();
    let (first, _, task) = s.transfer().await?;
    let binding = workflow::read_case_work_binding(&f.pool, &f.reviewer, &task).await?;
    let case = payroll::read_case_control(&f.pool, &f.submitter, binding.case_id).await?;
    let content = payroll::read_case_content(&f.pool, &f.submitter, &case).await?;
    let nominee = s.deployment.accounts.transfer_recipient.account_id;
    let expected = group::configuration::read_routing_control(
        &s.deployment.pool,
        &s.deployment.operator,
        s.deployment.group_id,
        nominee,
    )
    .await?;
    let now = time::OffsetDateTime::now_utc();
    group::configuration::publish_capacity_and_availability(&s.deployment.pool,&s.deployment.operator,typed(json!({
        "command_id":Uuid::new_v4(),"group_id":s.deployment.group_id,"account_id":nominee,"expected":expected,
        "limit_units":1,"availability":"AVAILABLE","validity":{"from":crate::workflow_fixture::micros(now),
            "until":crate::workflow_fixture::micros(now+time::Duration::days(1))}
    }))?).await?;
    let a = bind_and_seal(
        &f.pool,
        &f.reviewer,
        &UntrustedActionTargetSelection::existing_transfer_plan(first.plan.plan_id),
        &workflow::TransferExecute {
            plan: first.plan,
            task: task.clone(),
        },
    )
    .await?;
    let moved = completed(workflow::work_transfer_execute(&f.pool, &f.reviewer, &a).await?)?;
    assert_company(&moved.receipt, &a);
    let selection =
        identity::read_current_company_selection(&s.deployment.pool, &s.deployment.operator, f.org)
            .await?;
    let requester = console_platform_request_context::account::resolve_company_context(
        &s.deployment.pool,
        &s.deployment.verifier,
        &s.deployment.accounts.submitter.account_access_token,
        &selection,
        &s.deployment.serving,
    )
    .await?;
    let handler = console_platform_request_context::account::resolve_company_context(
        &s.deployment.pool,
        &s.deployment.verifier,
        &s.deployment.accounts.reviewer.account_access_token,
        &selection,
        &s.deployment.serving,
    )
    .await?;
    let item =
        s.item("A second explicitly submitted question has its own actual accountable task")?;
    let next: payroll::CaseRequest = typed(
        json!({"target":content.target,"items":[item],"predecessor_response_id":null,
        "narrative":"Separate explicit request; no fabricated capacity charge"}),
    )?;
    let target = explicit_new_intent::<payroll::CaseRequest>(
        &f.pool,
        &requester,
        &UntrustedActionTargetSelection::create_registered::<payroll::CaseRequest>(),
    )
    .await?;
    let a = bind_and_seal(&f.pool, &requester, &target, &next).await?;
    let opened = completed(payroll::payroll_review_request(&f.pool, &requester, &a).await?)?;
    let second = opened.task.ok_or("second actual handler task absent")?;
    assert_ne!(second.task_id, task.task_id);
    let selection =
        identity::read_current_company_selection(&s.deployment.pool, &s.deployment.operator, f.org)
            .await?;
    let handler = console_platform_request_context::account::resolve_company_context(
        &s.deployment.pool,
        &s.deployment.verifier,
        &s.deployment.accounts.reviewer.account_access_token,
        &selection,
        &s.deployment.serving,
    )
    .await?;
    let input: workflow::TransferSubmit = typed(json!({"targets":[{"task":second,
        "selection":{"kind":"NOMINATED","account_id":nominee},
        "responsibility":{"kind":"PERMANENT","from":crate::workflow_fixture::micros(time::OffsetDateTime::now_utc())}}],
        "reason_revision":null,"context_evidence":[],"mandate":null,"explanation":"Explicit next transfer competes for actual occupied capacity"}))?;
    let target = explicit_new_intent::<workflow::TransferSubmit>(
        &f.pool,
        &handler,
        &UntrustedActionTargetSelection::create_registered::<workflow::TransferSubmit>(),
    )
    .await?;
    let before = assignment_snapshot(&s).await?;
    assert_eq!(before["charges"].as_array().unwrap().len(), 2);
    let nominees = before["charges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["account_id"] == json!(nominee))
        .collect::<Vec<_>>();
    assert_eq!(nominees.len(), 1);
    assert_eq!(nominees[0]["units"], 1);
    // Nomination may retain an infeasible explicit plan for repair; it never
    // consumes another unit. A refusal at bind/submit is also precise no effect.
    match bind_and_seal(&f.pool, &handler, &target, &input).await {
        Err(error) => assert_capacity_refusal(error),
        Ok(submission) => {
            match workflow::work_transfer_submit(&f.pool, &handler, &submission).await? {
                OwnerExecution::Completed(plan) => {
                    let execute = bind_and_seal(
                        &f.pool,
                        &handler,
                        &UntrustedActionTargetSelection::existing_transfer_plan(plan.plan.plan_id),
                        &workflow::TransferExecute {
                            plan: plan.plan,
                            task: second,
                        },
                    )
                    .await;
                    match execute {
                        Ok(a) => assert_capacity_owner_refusal(
                            &workflow::work_transfer_execute(&f.pool, &handler, &a).await?,
                        ),
                        Err(e) => assert_capacity_refusal(e),
                    }
                }
                refusal => assert_capacity_owner_refusal(&refusal),
            }
        }
    }
    assert_eq!(
        assignment_snapshot(&s).await?,
        before,
        "capacity conflict cannot lose the first or second real task responsibility/charge"
    );
    Ok(())
}
fn assert_capacity_refusal(error: ProducerStop) {
    match error {
        ProducerStop::Observation(ResultObservation::DefinitiveNoEffect { reason, .. }) => {
            assert!(matches!(
                reason.code.as_str(),
                "capacity_conflict" | "recipient_not_feasible" | "no_capacity"
            ))
        }
        _ => panic!(
            "precise current capacity refusal required; technical/fixture failure is not proof"
        ),
    }
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn transfer_unknown_receipt_retains_actual_assignment_and_capacity_until_reconciled(
    pool: PgPool,
) -> TestResult {
    let admin = pool.clone();
    let s = fixture(pool, "transfer_unknown_receipt", 1).await?;
    let f = s.company();
    let (plan, _, task) = s.transfer().await?;
    let attempt = bind_and_seal(
        &f.pool,
        &f.reviewer,
        &UntrustedActionTargetSelection::existing_transfer_plan(plan.plan.plan_id),
        &workflow::TransferExecute {
            plan: plan.plan,
            task,
        },
    )
    .await?;
    let actual = completed(workflow::work_transfer_execute(&f.pool, &f.reviewer, &attempt).await?)?;
    assert_company(&actual.receipt, &attempt);
    let before = assignment_snapshot(&s).await?;
    let selection = console_identity_adapter_postgres::account13::read_current_company_selection(
        &s.deployment.pool,
        &s.deployment.operator,
        f.org,
    )
    .await?;
    let actor = console_platform_request_context::account::resolve_company_context(
        &s.deployment.pool,
        &s.deployment.verifier,
        &s.deployment.accounts.reviewer.account_access_token,
        &selection,
        &s.deployment.serving,
    )
    .await?;
    let mut barrier = admin.begin().await?;
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *barrier)
        .await?;
    sqlx::query("LOCK TABLE public.ont_action_command_receipts IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *barrier)
        .await?;
    let replay = workflow::work_transfer_execute(&f.pool, &actor, &attempt);
    tokio::pin!(replay);
    tokio::select! {
        r=&mut replay=>{r?;panic!("actual receipt unavailable without a Company lookup witness")},
        blocked=blocked_by(&admin,pid)=>{assert_ne!(blocked?,pid);}
    }
    let observed = tokio::time::timeout(std::time::Duration::from_secs(10), &mut replay).await??;
    match observed {
        OwnerExecution::Observation(ResultObservation::Unknown { subject, .. }) => match subject {
            ReconciliationSubject::Attempt { attempt: observed } => assert_eq!(observed, attempt),
            ReconciliationSubject::CompanyCommand { command } => {
                assert_eq!(command.org_id, attempt.org_id);
                assert_eq!(command.command_id, attempt.command_id);
            }
            _ => panic!("unknown reconciliation must retain the original Company subject"),
        },
        OwnerExecution::Observation(ResultObservation::Unavailable { error }) => assert!(matches!(
            error.code.as_str(),
            "receipt_unavailable" | "reconciliation_unavailable"
        )),
        _ => panic!(
            "an unavailable authoritative receipt cannot become definitive no effect or unverified success"
        ),
    }
    assert_eq!(
        assignment_snapshot(&s).await?,
        before,
        "unknown outcome cannot release capacity or roll back actual assignment"
    );
    barrier.rollback().await?;
    let reconciled = completed(workflow::work_transfer_execute(&f.pool, &actor, &attempt).await?)?;
    assert_eq!(value(&reconciled)?, value(&actual)?);
    assert_eq!(assignment_snapshot(&s).await?, before);
    Ok(())
}

#[sqlx::test(migrations = "../crates/platform/db/migrations")]
async fn group_resume_requires_original_account_even_for_same_human_with_group_grant(
    pool: PgPool,
) -> TestResult {
    use console_identity_adapter_postgres::account13 as identity;
    let s = fixture(pool, "group_resume_account_identity", 2).await?;
    let (parent, _, inputs) = s.preparing_group().await?;
    let attempts = s.prepare_children(&parent, &inputs).await?;
    let auth = s
        .group_auth(&s.deployment.accounts.submitter.account_access_token)
        .await?;
    let activate = s.group_identity::<group::GroupActivate>(&auth).await?;
    let active = completed(
        group::group_operation_activate(
            &s.deployment.pool,
            &auth,
            group::GroupRegisteredActionRequest {
                identity: activate,
                input: group::GroupActivate {
                    operation: parent.operation,
                    prepared_attempts: attempts,
                },
            },
        )
        .await?,
    )?;
    let alias = &s.deployment.accounts.same_human_alias;
    assert_ne!(alias.account_id, s.deployment.accounts.submitter.account_id);
    let schema = identity::read_group_policy_schema(
        &s.deployment.pool,
        &s.deployment.operator,
        s.deployment.group_id,
    )
    .await?;
    let expected = identity::read_group_policy_control(
        &s.deployment.pool,
        &s.deployment.operator,
        s.deployment.group_id,
    )
    .await?;
    identity::apply_group_grant_plan(&s.deployment.pool,&s.deployment.operator,typed(json!({
        "command_id":Uuid::new_v4(),"group_id":s.deployment.group_id,"expected":expected,"plan":{
            "account_id":alias.account_id,"scope":{"kind":"GROUP_CONTROL"},"actions":[schema.action("group.operation.resume")?],
            "field_projection":schema.projection("group.operation.control_and_status")?,"valid_from":null,"valid_to":null,
            "reason":"Explicit Group resume right on another actual Account of the same verified Human"}
    }))?).await?;
    console_platform_auth::account_session::logout_account_session(
        &s.deployment.runtime.auth,
        &s.deployment.verifier,
        &s.deployment.accounts.submitter.account_access_token,
    )
    .await?;
    let alias_auth = s.group_auth(&alias.account_access_token).await?;
    let before =
        group::read_group_operation(&s.deployment.pool, &alias_auth, &active.operation).await?;
    let ready =
        group::read_group_dispatch_readiness(&s.deployment.pool, &alias_auth, &active.operation)
            .await?;
    assert_eq!(value(&ready)?["reason"], "ORIGINAL_SESSION_ENDED");
    let identity = s.group_identity::<group::GroupResume>(&alias_auth).await?;
    assert_refusal(
        &group::group_operation_resume(
            &s.deployment.pool,
            &alias_auth,
            group::GroupRegisteredActionRequest {
                identity,
                input: group::GroupResume {
                    operation: active.operation.clone(),
                    confirm_original_intent: true,
                },
            },
        )
        .await?,
    );
    let after =
        group::read_group_operation(&s.deployment.pool, &alias_auth, &active.operation).await?;
    assert_eq!(value(&after)?, value(&before)?);
    assert_eq!(
        after.original_account_id,
        s.deployment.accounts.submitter.account_id
    );
    Ok(())
}

fn is_field_denial(code: &str) -> bool {
    matches!(
        code,
        "not_authorized" | "not_found" | "required_source_field_denied" | "field_not_authorized"
    )
}
fn assert_field_refusal<T>(result: &OwnerExecution<T>) {
    match result {
        OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect { reason, .. }) => {
            assert!(is_field_denial(&reason.code))
        }
        _ => panic!("precise current source-field authorization refusal required"),
    }
}
fn assert_field_binding_refusal(error: ProducerStop) {
    match error {
        ProducerStop::Observation(ResultObservation::DefinitiveNoEffect { reason, .. }) => {
            assert!(is_field_denial(&reason.code))
        }
        ProducerStop::Observation(ResultObservation::Unavailable { error }) => {
            assert!(is_field_denial(&error.code))
        }
        _ => panic!(
            "precise field-authorization binder refusal required; intent, scope or fixture errors are not proof"
        ),
    }
}
fn assert_capacity_owner_refusal<T>(result: &OwnerExecution<T>) {
    match result {
        OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect { reason, .. }) => {
            assert!(matches!(
                reason.code.as_str(),
                "capacity_conflict" | "recipient_not_feasible" | "no_capacity"
            ))
        }
        _ => panic!("precise capacity owner refusal required; any-no-effect is insufficient"),
    }
}
