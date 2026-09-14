//! Candidate acceptance source; no owner implementation or runtime claim.
//! Native fixture is produced through the separately frozen ordinary owner chain.
use console_ontology_adapter_postgres::action30 as ontology;
use console_ontology_application::action30::*;
use console_payroll_adapter_postgres::action30 as payroll;
use crate::binder_sequence::{completed, explicit_new_intent};
use crate::native_fixture::{fixture, Fixture, TestResult};
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::PgPool;

fn control<I>(input: I) -> UnadmittedControl<I> {
    UnadmittedControl { command_id: Uuid::new_v4(), input }
}

async fn editable(f: &Fixture) -> TestResult<(RegisteredDraftBinding, DraftAck)> {
    let (run, _) = f.staged(false).await?;
    let target = UntrustedActionTargetSelection::existing_run(run.run_id);
    let selected = explicit_new_intent::<payroll::NativePrepare>(&f.pool, &f.submitter, &target).await?;
    let binding = ontology::read_registered_draft_binding::<payroll::NativePrepare>(&f.pool, &f.submitter, &selected).await?;
    let ack = completed(ontology::draft_start(&f.pool, &f.submitter, control(DraftStart {
        action: binding.action.clone(), target: binding.target.clone(),
        schema: binding.schema.clone(), custody: binding.custody.clone(),
        intent_slot: binding.intent_slot.clone(),
    })).await?)?;
    let captured=history(f,ack.draft_id).await?;
    oracle(&captured,&[acknowledged(&captured,&ack)])?;
    Ok((binding, ack))
}

async fn populated(f: &Fixture) -> TestResult<(RegisteredDraftBinding, DraftAck)> {
    let (run, choices) = f.staged(false).await?;
    let selected = explicit_new_intent::<payroll::NativePrepare>(&f.pool, &f.submitter, &UntrustedActionTargetSelection::existing_run(run.run_id)).await?;
    let binding = ontology::read_registered_draft_binding::<payroll::NativePrepare>(&f.pool, &f.submitter, &selected).await?;
    let started = completed(ontology::draft_start(&f.pool, &f.submitter, control(DraftStart {
        action: binding.action.clone(), target: binding.target.clone(), schema: binding.schema.clone(),
        custody: binding.custody.clone(), intent_slot: binding.intent_slot.clone(),
    })).await?)?;
    let initial_history=history(f,started.draft_id).await?;
    let initial_ack=acknowledged(&initial_history,&started);
    let input = f.prepare_input(run.run_id, &choices).await?;
    let patches = compile_registered_input_patches(&binding.schema_snapshot, &input)?;
    let saved = completed(ontology::draft_save(&f.pool, &f.submitter, control(DraftSave {
        draft_id: started.draft_id, expected_revision_id: started.revision_id,
        editing_token: started.editing_token, patches,
    })).await?)?;
    let saved_history=history(f,saved.draft_id).await?;
    preserved_receipts(&initial_history,&saved_history);
    oracle(&saved_history,&[initial_ack,acknowledged(&saved_history,&saved)])?;
    Ok((binding, saved))
}

async fn seal(f: &Fixture, binding: &RegisteredDraftBinding, ack: &DraftAck) -> TestResult<SealResult> {
    Ok(completed(ontology::draft_seal(&f.pool, &f.submitter, control(DraftSeal {
        draft_id: ack.draft_id, expected_revision_id: ack.revision_id,
        editing_token: ack.editing_token.clone(), prepared_submission_unit: None,
        gate_request_intent: binding.gate_request_intent.clone(),
    })).await?)?)
}

// Ordinary projection owner; exact manifest pins all returned pages to the same
// control/content/policy revisions. A mutation invalidates continuation rather
// than mixing pages. This read grants nothing and is shared with the history UI.
async fn history(f: &Fixture, id: Uuid) -> TestResult<Value> {
    let manifest = ontology::read_draft_history_manifest(&f.pool, &f.submitter, id).await?;
    let mut revisions = Vec::new();
    let mut receipts = Vec::new();
    let mut cursor = None;
    let mut seen = std::collections::BTreeSet::new();
    loop {
        let page = ontology::read_draft_history_page(&f.pool, &f.submitter, &manifest, cursor.as_deref()).await?;
        assert_eq!(page.manifest, manifest);
        revisions.extend(page.revisions);
        receipts.extend(page.receipts);
        match page.next_cursor {
            None => break,
            Some(next) => {
                assert!(seen.insert(next.clone()), "cyclic history pagination");
                assert!(seen.len() <= manifest.page_budget, "history exceeds declared budget");
                cursor = Some(next);
            }
        }
    }
    assert_eq!(revisions.len(), manifest.revision_count);
    assert_eq!(receipts.len(), manifest.receipt_count);
    let value = json!({"complete":true,"remaining_cursor":null,
        "draft_id":id,"company_id":f.org,"head_revision_id":manifest.head_revision_id,
        "declared_revision_count":manifest.revision_count,"declared_receipt_count":manifest.receipt_count,
        "revisions":revisions,"receipts":receipts,"domain_effects":manifest.domain_effects});
    oracle(&value, &[])?;
    Ok(value)
}

fn oracle(h: &Value, a: &[Value]) -> TestResult {
    use std::io::Write;
    let checker = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/draft/history_oracle.py");
    let mut child = std::process::Command::new("python3").arg(checker)
        .stdin(std::process::Stdio::piped()).spawn()?;
    child.stdin.take().ok_or("checker input unavailable")?.write_all(&serde_json::to_vec(&json!({"history":h,"acknowledged":a}))?)?;
    assert!(child.wait()?.success(), "independent draft history oracle rejected readback");
    Ok(())
}

fn acknowledged(h: &Value, ack: &DraftAck) -> Value {
    let immutable = revisions(h).iter().find(|r| r["revision_id"] == json!(ack.revision_id)).expect("owner acknowledged revision must exist").clone();
    match &ack.receipt {
        ReceiptRef::Company { command, receipt_id } => json!({"command_id":command.command_id,"receipt_id":receipt_id,"revision_id":ack.revision_id,"immutable_revision":immutable}),
        ReceiptRef::Group { .. } => panic!("Company draft received Group receipt"),
    }
}
fn required_field<'a>(h: &'a Value, binding: &RegisteredDraftBinding, path: &[&str]) -> TestResult<&'a Value> {
    let defined = resolve_registered_field_address(&binding.schema_snapshot,path)?;
    assert!(defined.required, "fixture targets a required schema field");
    assert!(defined.is_scalar(), "fixture targets a scalar field");
    let address=serde_json::to_value(&defined.address)?;
    let field=revisions(h).last().unwrap()["fields"].as_array().unwrap().iter()
        .find(|f|f["typed_address"]==address).ok_or("required registered field absent from actual draft")?;
    assert_eq!(field["state"],"COMPLETE");
    Ok(field)
}
fn revisions(h: &Value) -> &Vec<Value> { h["revisions"].as_array().expect("revision array") }
fn unchanged_content(a: &Value, b: &Value) {
    assert_eq!(a["revisions"], b["revisions"]);
    assert_eq!(a["head_revision_id"], b["head_revision_id"]);
    assert_eq!(a["domain_effects"], b["domain_effects"]);
    preserved_receipts(a,b);
}
fn preserved_receipts(a:&Value,b:&Value) {
    for receipt in a["receipts"].as_array().unwrap() {
        let actual=b["receipts"].as_array().unwrap().iter().find(|row|row["command_id"]==receipt["command_id"]).expect("prior receipt disappeared");
        assert_eq!(actual,receipt,"prior receipt rewritten");
    }
}
fn no_effect<R: std::fmt::Debug>(r: OwnerExecution<R>) {
    assert!(matches!(r, OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect { .. })), "expected classified definitive refusal: {r:?}");
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn draft_start_replay_preserves_identity_and_has_no_business_effect(pool:PgPool) -> TestResult {
    let f=fixture(pool,"draft_start_replay").await?;let (binding, ack)=editable(&f).await?;
    let h=history(&f,ack.draft_id).await?;assert_eq!(revisions(&h).len(),1);
    assert!(h["domain_effects"].as_array().unwrap().is_empty());
    // Same intent with a new delivery/control command must resume the existing draft.
    let again=completed(ontology::draft_start(&f.pool,&f.submitter,control(DraftStart{
        action:binding.action,target:binding.target,schema:binding.schema,custody:binding.custody,intent_slot:binding.intent_slot,
    })).await?)?;
    assert_eq!(ack.draft_id,again.draft_id);assert_eq!(ack.revision_id,again.revision_id);
    unchanged_content(&h,&history(&f,ack.draft_id).await?);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn draft_get_binding_does_not_mint_intent_or_revision(pool:PgPool) -> TestResult {
    let f=fixture(pool,"draft_get_binding_no_mutation").await?;let (binding,ack)=populated(&f).await?;
    let before=history(&f,ack.draft_id).await?;
    for _ in 0..3 {let current=ontology::read_draft_continuation(&f.pool,&f.submitter,ack.draft_id).await?;assert_eq!(current.revision_id,ack.revision_id);}
    assert_eq!(before,history(&f,ack.draft_id).await?);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn draft_save_stale_head_does_not_overwrite_acknowledged_input(pool:PgPool) -> TestResult {
    let f=fixture(pool,"draft_save_stale_head").await?;let (binding,ack)=populated(&f).await?;
    let before=history(&f,ack.draft_id).await?;
    let field=required_field(&before,&binding,&["expected","run_revision"])?;
    let patch:Patch=serde_json::from_value(json!({"kind":"SET_INCOMPLETE","address":field["typed_address"],"editor_text":"300만?"}))?;
    let stale=ontology::draft_save(&f.pool,&f.submitter,control(DraftSave{draft_id:ack.draft_id,expected_revision_id:serde_json::from_value(revisions(&before)[revisions(&before).len()-2]["revision_id"].clone())?,editing_token:ack.editing_token.clone(),patches:vec![patch]})).await?;
    no_effect(stale);unchanged_content(&before,&history(&f,ack.draft_id).await?);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn draft_incomplete_text_survives_replay_and_refuses_seal(pool:PgPool) -> TestResult {
    let f=fixture(pool,"draft_incomplete_text").await?;let (binding,ack)=populated(&f).await?;
    let before=history(&f,ack.draft_id).await?;
    let field=required_field(&before,&binding,&["expected","run_revision"])?;
    let address=field["typed_address"].clone();
    let patch:Patch=serde_json::from_value(json!({"kind":"SET_INCOMPLETE","address":address,"editor_text":"입력 중 300만?"}))?;
    let request=control(DraftSave{draft_id:ack.draft_id,expected_revision_id:ack.revision_id,editing_token:ack.editing_token.clone(),patches:vec![patch]});
    let saved=completed(ontology::draft_save(&f.pool,&f.submitter,request.clone()).await?)?;
    let after=history(&f,ack.draft_id).await?;preserved_receipts(&before,&after);oracle(&after,&[acknowledged(&before,&ack),acknowledged(&after,&saved)])?;assert_eq!(revisions(&after).len(),revisions(&before).len()+1);
    assert_eq!(&revisions(&after)[..revisions(&before).len()],revisions(&before));
    let field=revisions(&after).last().unwrap()["fields"].as_array().unwrap().iter().find(|x|x["typed_address"]==address).expect("preserved field");
    assert_eq!(field["editor_text"],"입력 중 300만?");assert_eq!(field["state"],"INCOMPLETE");
    let replay=completed(ontology::draft_save(&f.pool,&f.submitter,request).await?)?;
    assert_eq!(replay.revision_id,saved.revision_id);assert_eq!(replay.receipt,saved.receipt);
    let refused=ontology::draft_seal(&f.pool,&f.submitter,control(DraftSeal{draft_id:saved.draft_id,expected_revision_id:saved.revision_id,editing_token:saved.editing_token.clone(),prepared_submission_unit:None,gate_request_intent:binding.gate_request_intent})).await?;
    no_effect(refused);unchanged_content(&after,&history(&f,ack.draft_id).await?);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn draft_restore_appends_selected_history_without_rewriting_old_revisions(pool:PgPool) -> TestResult {
    let f=fixture(pool,"draft_restore_selected").await?;let (binding,ack)=populated(&f).await?;
    let before=history(&f,ack.draft_id).await?;
    let fields=[required_field(&before,&binding,&["expected","run_revision"])?,required_field(&before,&binding,&["expected_membership_revision"])?];
    let first=fields[0]["typed_address"].clone();let second=fields[1]["typed_address"].clone();
    let patches:Vec<Patch>=serde_json::from_value(json!([
        {"kind":"SET_INCOMPLETE","address":first,"editor_text":"first edit"},
        {"kind":"SET_INCOMPLETE","address":second,"editor_text":"keep second edit"}]))?;
    let saved=completed(ontology::draft_save(&f.pool,&f.submitter,control(DraftSave{draft_id:ack.draft_id,expected_revision_id:ack.revision_id,editing_token:ack.editing_token.clone(),patches})).await?)?;
    let middle=history(&f,ack.draft_id).await?;preserved_receipts(&before,&middle);
    let request=control(DraftRestore{draft_id:saved.draft_id,expected_revision_id:saved.revision_id,editing_token:saved.editing_token.clone(),source_revision_id:ack.revision_id,selected_field_ids:vec![serde_json::from_value(first["field_id"].clone())?]});
    let restored=completed(ontology::draft_restore(&f.pool,&f.submitter,request.clone()).await?)?;
    let after=history(&f,ack.draft_id).await?;preserved_receipts(&middle,&after);
    assert_ne!(restored.revision_id,ack.revision_id);assert_ne!(restored.revision_id,saved.revision_id);
    assert_eq!(&revisions(&after)[..revisions(&middle).len()],revisions(&middle));
    assert_eq!(revisions(&after).len(),revisions(&middle).len()+1);
    let current=revisions(&after).last().unwrap()["fields"].as_array().unwrap();
    assert_eq!(current.iter().find(|x|x["typed_address"]==first).unwrap(),fields[0]);
    oracle(&after,&[acknowledged(&before,&ack),acknowledged(&middle,&saved),acknowledged(&after,&restored)])?;
    assert_eq!(current.iter().find(|x|x["typed_address"]==second).unwrap()["editor_text"],"keep second edit");
    let replay=completed(ontology::draft_restore(&f.pool,&f.submitter,request).await?)?;
    assert_eq!(restored.receipt,replay.receipt);unchanged_content(&after,&history(&f,ack.draft_id).await?);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn draft_two_saves_same_head_have_one_winner_and_keep_original_history(pool:PgPool) -> TestResult {
    let f=fixture(pool,"draft_save_conflict").await?;let (binding,ack)=populated(&f).await?;
    let before=history(&f,ack.draft_id).await?;
    let address=required_field(&before,&binding,&["expected","run_revision"])?["typed_address"].clone();
    let patch=|text:&str| -> TestResult<Patch>{Ok(serde_json::from_value(json!({"kind":"SET_INCOMPLETE","address":address,"editor_text":text}))?)};
    let a=control(DraftSave{draft_id:ack.draft_id,expected_revision_id:ack.revision_id,editing_token:ack.editing_token.clone(),patches:vec![patch("A")?]});
    let b=control(DraftSave{draft_id:ack.draft_id,expected_revision_id:ack.revision_id,editing_token:ack.editing_token.clone(),patches:vec![patch("B")?]});
    assert_ne!(a.command_id,b.command_id);
    let (a,b)=tokio::join!(ontology::draft_save(&f.pool,&f.submitter,a),ontology::draft_save(&f.pool,&f.submitter,b));
    let mut wins=Vec::new();for result in [a?,b?] {match result {OwnerExecution::Completed(ack)=>wins.push(ack),other=>no_effect(other)}}assert_eq!(wins.len(),1);
    let after=history(&f,ack.draft_id).await?;preserved_receipts(&before,&after);oracle(&after,&[acknowledged(&after,&wins[0])])?;
    assert_eq!(revisions(&after).len(),revisions(&before).len()+1);assert_eq!(&revisions(&after)[..revisions(&before).len()],revisions(&before));
    assert_eq!(before["domain_effects"],after["domain_effects"]);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn draft_discard_keeps_acknowledged_content_and_refuses_later_seal(pool:PgPool) -> TestResult {
    let f=fixture(pool,"draft_discard_retains_history").await?;let (binding,ack)=populated(&f).await?;
    let before=history(&f,ack.draft_id).await?;
    let request=control(DraftDiscard{draft_id:ack.draft_id,expected_revision_id:ack.revision_id,editing_token:ack.editing_token.clone(),reason:"User canceled this draft".into()});
    let first=completed(ontology::draft_discard(&f.pool,&f.submitter,request.clone()).await?)?;
    let replay=completed(ontology::draft_discard(&f.pool,&f.submitter,request).await?)?;assert_eq!(first,replay);
    let refused=ontology::draft_seal(&f.pool,&f.submitter,control(DraftSeal{draft_id:ack.draft_id,expected_revision_id:ack.revision_id,editing_token:ack.editing_token.clone(),prepared_submission_unit:None,gate_request_intent:binding.gate_request_intent})).await?;
    no_effect(refused);unchanged_content(&before,&history(&f,ack.draft_id).await?);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn draft_seal_replay_has_one_original_attempt_without_business_effect(pool:PgPool) -> TestResult {
    let f=fixture(pool,"draft_seal_replay").await?;let (binding,ack)=populated(&f).await?;
    let request=control(DraftSeal{draft_id:ack.draft_id,expected_revision_id:ack.revision_id,editing_token:ack.editing_token.clone(),prepared_submission_unit:None,gate_request_intent:binding.gate_request_intent});
    let before=history(&f,ack.draft_id).await?;
    let sealed=completed(ontology::draft_seal(&f.pool,&f.submitter,request.clone()).await?)?;
    let replay=completed(ontology::draft_seal(&f.pool,&f.submitter,request).await?)?;
    assert_eq!(sealed.attempt,replay.attempt);assert_eq!(sealed.receipt,replay.receipt);
    unchanged_content(&before,&history(&f,ack.draft_id).await?);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn cancel_ready_attempt_is_terminal_no_effect_and_cannot_execute(pool:PgPool) -> TestResult {
    let f=fixture(pool,"attempt_cancel_ready").await?;let (binding,ack)=populated(&f).await?;
    let sealed=seal(&f,&binding,&ack).await?;
    let current=ontology::read_attempt_control(&f.pool,&f.submitter,&sealed.attempt).await?;
    assert_eq!(current.state,AttemptState::Ready);
    let request=control(AttemptCancel{attempt:sealed.attempt.clone(),expected_state_revision:current.state_revision,reason:"User changed intent".into()});
    let canceled=completed(ontology::attempt_cancel(&f.pool,&f.submitter,request.clone()).await?)?;
    let replay=completed(ontology::attempt_cancel(&f.pool,&f.submitter,request).await?)?;assert_eq!(canceled,replay);
    let observed=ontology::read_attempt_control(&f.pool,&f.submitter,&sealed.attempt).await?;assert_eq!(observed.state,AttemptState::Cancelled);
    let result=ontology::attempt_execute(&f.pool,&f.submitter,control(AttemptExecute{attempt:sealed.attempt})).await?;
    no_effect(result);assert!(history(&f,ack.draft_id).await?["domain_effects"].as_array().unwrap().is_empty());Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn attempt_execute_replay_reuses_original_domain_receipt(pool:PgPool) -> TestResult {
    let f=fixture(pool,"attempt_execute_original_receipt").await?;
    let (run,attempt)=f.prepared(false).await?;
    let before=f.snapshot(run.run_id).await?;
    let request=control(AttemptExecute{attempt:attempt.clone()});
    assert_ne!(request.command_id,attempt.command_id);
    let first=completed(ontology::attempt_execute(&f.pool,&f.submitter,request.clone()).await?)?;
    let replay=completed(ontology::attempt_execute(&f.pool,&f.submitter,request).await?)?;
    match &first {
        ResultObservation::Succeeded{receipt,..} => assert_eq!(receipt,&run.receipt),
        other=>panic!("terminal original prepare must reconcile: {other:?}"),
    }
    assert_eq!(first,replay);assert_eq!(before,f.snapshot(run.run_id).await?);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn attempt_cancel_cannot_erase_an_already_committed_effect(pool:PgPool) -> TestResult {
    let f=fixture(pool,"attempt_cancel_committed").await?;
    let (run,attempt)=f.prepared(false).await?;let before=f.snapshot(run.run_id).await?;
    let current=ontology::read_attempt_control(&f.pool,&f.submitter,&attempt).await?;
    no_effect(ontology::attempt_cancel(&f.pool,&f.submitter,control(AttemptCancel{
        attempt:attempt.clone(),expected_state_revision:current.state_revision,reason:"Cancel after completion".into(),
    })).await?);
    assert_eq!(before,f.snapshot(run.run_id).await?);
    let replay=completed(payroll::payroll_prepare_inputs(&f.pool,&f.submitter,&attempt).await?)?;
    assert_eq!(replay.receipt,run.receipt);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn attempt_cancel_pending_owner_work_requires_definitive_owner_evidence(pool:PgPool) -> TestResult {
    let f=fixture(pool,"attempt_cancel_pending").await?;let (binding,ack)=populated(&f).await?;
    let sealed=seal(&f,&binding,&ack).await?;
    let actual=payroll::payroll_prepare_inputs(&f.pool,&f.submitter,&sealed.attempt).await?;
    assert!(matches!(actual,OwnerExecution::Observation(ResultObservation::Pending{..})),"fixture must reach real BUILDING work");
    let current=ontology::read_attempt_control(&f.pool,&f.submitter,&sealed.attempt).await?;
    let refusal=ontology::attempt_cancel(&f.pool,&f.submitter,control(AttemptCancel{attempt:sealed.attempt.clone(),expected_state_revision:current.state_revision,reason:"Work may already be in progress".into()})).await?;
    match refusal {
        OwnerExecution::Observation(ResultObservation::Unknown{..}) => {},
        OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect{..}) => {},
        other=>panic!("pending owner cannot be reported canceled absent no-effect proof: {other:?}"),
    }
    let after=ontology::read_attempt_control(&f.pool,&f.submitter,&sealed.attempt).await?;
    assert_ne!(after.state,AttemptState::Cancelled);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn draft_schema_upgrade_preserves_identity_fields_and_old_schema(pool:PgPool) -> TestResult {
    let f=fixture(pool,"draft_upgrade_preserve").await?;let (binding,ack)=populated(&f).await?;
    let before=history(&f,ack.draft_id).await?;
    let facts=crate::lifecycle_producer::publish_upgrade(&f,&binding.schema,serde_json::from_value(required_field(&before,&binding,&["expected","run_revision"])?["typed_address"]["field_id"].clone())?).await?;
    let request=control(DraftUpgrade{draft_id:ack.draft_id,expected_revision_id:ack.revision_id,editing_token:ack.editing_token.clone(),target_schema:facts.target.clone(),mapping_revision:facts.mapping});
    let upgraded=completed(ontology::draft_upgrade_schema(&f.pool,&f.submitter,request.clone()).await?)?;
    let after=history(&f,ack.draft_id).await?;preserved_receipts(&before,&after);
    assert_eq!(upgraded.draft_id,ack.draft_id);assert_ne!(upgraded.revision_id,ack.revision_id);
    assert_eq!(revisions(&after).len(),revisions(&before).len()+1);assert_eq!(&revisions(&after)[..revisions(&before).len()],revisions(&before));
    assert_eq!(revisions(&before).last().unwrap()["fields"],revisions(&after).last().unwrap()["fields"]);
    assert_ne!(revisions(&before).last().unwrap()["schema_revision_id"],revisions(&after).last().unwrap()["schema_revision_id"]);
    oracle(&after,&[acknowledged(&before,&ack),acknowledged(&after,&upgraded)])?;
    let replay=completed(ontology::draft_upgrade_schema(&f.pool,&f.submitter,request).await?)?;
    assert_eq!(upgraded.receipt,replay.receipt);unchanged_content(&after,&history(&f,ack.draft_id).await?);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn draft_schema_upgrade_partial_mapping_cannot_drop_complete_input(pool:PgPool) -> TestResult {
    let f=fixture(pool,"draft_upgrade_partial").await?;let (binding,ack)=populated(&f).await?;
    let before=history(&f,ack.draft_id).await?;let facts=crate::lifecycle_producer::publish_upgrade(&f,&binding.schema,serde_json::from_value(required_field(&before,&binding,&["expected","run_revision"])?["typed_address"]["field_id"].clone())?).await?;
    no_effect(ontology::draft_upgrade_schema(&f.pool,&f.submitter,control(DraftUpgrade{
        draft_id:ack.draft_id,expected_revision_id:ack.revision_id,editing_token:ack.editing_token,target_schema:facts.target,mapping_revision:facts.intentionally_partial_mapping,
    })).await?);
    unchanged_content(&before,&history(&f,ack.draft_id).await?);Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn draft_expiry_uses_current_server_policy_and_retains_history(pool:PgPool) -> TestResult {
    let f=fixture(pool,"draft_expiry").await?;let (binding,ack)=populated(&f).await?;
    let policy=crate::lifecycle_producer::install_short_editable_retention(&f,ack.draft_id).await?;
    crate::lifecycle_producer::wait_until_due(&f,ack.draft_id,&policy).await?;
    let current=ontology::read_draft_continuation(&f.pool,&f.submitter,ack.draft_id).await?;
    let before=history(&f,ack.draft_id).await?;
    let request=UnadmittedWorkerControl{command_id:Uuid::new_v4(),input:DraftExpire{draft_id:ack.draft_id,expected_control_revision:current.control_revision,retention_policy:policy}};
    let first=completed(ontology::draft_expire(&f.pool,&f.worker,request.clone()).await?)?;
    let replay=completed(ontology::draft_expire(&f.pool,&f.worker,request).await?)?;assert_eq!(first,replay);
    unchanged_content(&before,&history(&f,ack.draft_id).await?);
    assert!(matches!(ontology::read_draft_continuation(&f.pool,&f.submitter,ack.draft_id).await,Err(ontology::OwnerActionError::DraftNotEditable{..})));
    Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn draft_expiry_ends_editing_but_preserves_held_evidence(pool:PgPool) -> TestResult {
    let f=fixture(pool,"draft_expiry_held").await?;let (binding,ack)=populated(&f).await?;
    let policy=crate::lifecycle_producer::install_short_editable_retention(&f,ack.draft_id).await?;
    let _held=crate::lifecycle_producer::hold(&f,ack.draft_id).await?;
    crate::lifecycle_producer::wait_until_due(&f,ack.draft_id,&policy).await?;
    let current=ontology::read_draft_continuation(&f.pool,&f.submitter,ack.draft_id).await?;
    let before=history(&f,ack.draft_id).await?;
    let expired=completed(ontology::draft_expire(&f.pool,&f.worker,UnadmittedWorkerControl{command_id:Uuid::new_v4(),input:DraftExpire{draft_id:ack.draft_id,expected_control_revision:current.control_revision,retention_policy:policy}}).await?)?;
    assert!(matches!(expired,ResultObservation::Succeeded{..}));
    unchanged_content(&before,&history(&f,ack.draft_id).await?);
    assert!(matches!(ontology::read_draft_continuation(&f.pool,&f.submitter,ack.draft_id).await,Err(ontology::OwnerActionError::DraftNotEditable{..})));Ok(())
}
