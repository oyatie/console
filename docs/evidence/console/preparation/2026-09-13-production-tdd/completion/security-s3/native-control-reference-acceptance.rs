//! Candidate current_controls transport envelope, shared by JSON and SSR forms.
//! No SET may alter server-owned expected values. The actual draft.save binder
//! resolves renderer-issued controls before complete validation/fingerprinting.
use crate::browser_auth_producer::{Browser, login};
use crate::{
    binder_sequence::{completed, explicit_new_intent},
    native_fixture::{NativeDeployment, TestResult, build_native_deployment},
    security_grant_producer::{fresh_context, replace_projection},
};
use console_ontology_adapter_postgres::{action30 as ontology, projection30 as projection};
use console_ontology_application::action30::*;
use console_payroll_adapter_postgres::action30 as payroll;
use console_platform_request_context::account::AuthenticatedCompanyContext;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;
struct Editor {
    binding: RegisteredDraftBinding,
    ack: DraftAck,
    address: FieldAddress,
    handle: String,
    patches: Vec<Patch>,
    input: payroll::RunCalculate,
    run: Uuid,
}
async fn editor(
    d: &NativeDeployment,
    auth: &AuthenticatedCompanyContext,
    run: Uuid,
) -> TestResult<Editor> {
    let f = &d.companies[0];
    let selected = explicit_new_intent::<payroll::RunCalculate>(
        &d.pool,
        auth,
        &UntrustedActionTargetSelection::existing_run(run),
    )
    .await?;
    let binding =
        ontology::read_registered_draft_binding::<payroll::RunCalculate>(&d.pool, auth, &selected)
            .await?;
    let field =
        resolve_registered_field_address(&binding.schema_snapshot, &["expected", "close_basis"])?;
    let input = payroll::RunCalculate {
        expected: payroll::read_run_control(&d.pool, &f.submitter, run).await?,
    };
    assert!(input.expected.close_basis.is_some());
    let ack = completed(
        ontology::draft_start(
            &d.pool,
            auth,
            UnadmittedControl {
                command_id: Uuid::new_v4(),
                input: DraftStart {
                    action: binding.action.clone(),
                    target: binding.target.clone(),
                    schema: binding.schema.clone(),
                    custody: binding.custody.clone(),
                    intent_slot: binding.intent_slot.clone(),
                },
            },
        )
        .await?,
    )?;
    let view = projection::render_registered_draft_editor(&d.pool, auth, ack.draft_id).await?;
    let control = view
        .current_controls
        .iter()
        .find(|c| c.address == field.address)
        .ok_or("actual close-basis control missing")?;
    assert_eq!(control.handle.as_str().len(), 43);
    assert_eq!(
        base64::Engine::decode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            control.handle.as_str()
        )?
        .len(),
        32
    );
    let public = serde_json::to_vec(&view)?;
    let private = serde_json::to_vec(input.expected.close_basis.as_ref().unwrap())?;
    assert!(!public.windows(private.len()).any(|w| w == private));
    let patches = compile_registered_editor_patches(&binding.schema_snapshot, &view.fields)?;
    assert!(
        patches.iter().all(|p| p.address() != &field.address),
        "server-owned expectations cannot become editable patches"
    );
    Ok(Editor {
        binding,
        ack,
        address: field.address,
        handle: control.handle.as_str().into(),
        patches,
        input,
        run,
    })
}
async fn chain(d: &NativeDeployment, e: &Editor) -> TestResult<Value> {
    let f = &d.companies[0];
    let manifest =
        ontology::read_draft_history_manifest(&d.pool, &f.submitter, e.ack.draft_id).await?;
    let mut revisions = Vec::new();
    let mut receipts = Vec::new();
    let mut cursor = None;
    let mut seen = std::collections::BTreeSet::new();
    loop {
        let page =
            ontology::read_draft_history_page(&d.pool, &f.submitter, &manifest, cursor.as_deref())
                .await?;
        assert_eq!(page.manifest, manifest);
        revisions.extend(page.revisions);
        receipts.extend(page.receipts);
        match page.next_cursor {
            None => break,
            Some(next) => {
                assert!(seen.insert(next.clone()) && seen.len() <= manifest.page_budget);
                cursor = Some(next)
            }
        }
    }
    assert_eq!(revisions.len(), manifest.revision_count);
    assert_eq!(receipts.len(), manifest.receipt_count);
    Ok(
        json!({"revisions":revisions,"receipts":receipts,"head":manifest.head_revision_id,"effects":manifest.domain_effects}),
    )
}
fn request(e: &Editor, address: FieldAddress, handle: &str) -> TestResult<DraftEditorSave> {
    Ok(DraftEditorSave {
        save: DraftSave {
            draft_id: e.ack.draft_id,
            expected_revision_id: e.ack.revision_id,
            editing_token: e.ack.editing_token.clone(),
            patches: e.patches.clone(),
        },
        current_controls: vec![SubmittedControlReference {
            address,
            handle: ControlHandle::parse(handle)?,
        }],
    })
}
async fn submit(
    d: &NativeDeployment,
    auth: &AuthenticatedCompanyContext,
    input: DraftEditorSave,
) -> TestResult<OwnerExecution<DraftAck>> {
    Ok(ontology::draft_save_editor(
        &d.pool,
        auth,
        UnadmittedControl {
            command_id: Uuid::new_v4(),
            input,
        },
    )
    .await?)
}
fn refused(r: OwnerExecution<DraftAck>) {
    assert!(matches!(
        r,
        OwnerExecution::Observation(ResultObservation::DefinitiveNoEffect { .. })
    ));
}
async fn setup(
    pool: PgPool,
    label: &str,
) -> TestResult<(NativeDeployment, AuthenticatedCompanyContext, Uuid, Browser)> {
    let mut d = build_native_deployment(pool, label, 2).await?;
    let (run, _) = d.companies[0].closed().await?;
    replace_projection(
        &d,
        0,
        d.accounts.reviewer.account_id,
        &[
            "context.discover",
            "payroll.calculate_run",
            "draft.start",
            "draft.save",
            "draft.seal",
        ],
        "payroll.calculate.scoped_control",
    )
    .await?;
    let browser = login(
        &d.runtime.client,
        &d.runtime.origin,
        &mut d.accounts.reviewer,
    )
    .await?;
    let auth = fresh_context(&d, 0, &browser.cookies["__Host-console_account_session"]).await?;
    Ok((d, auth, run.run_id, browser))
}
#[sqlx::test(migrations = false)]
async fn control_reference_renderer_envelope_seals_exact_private_reference_json_and_html(
    pool: PgPool,
) -> TestResult {
    let (d, auth, run, _browser) = setup(pool, "control-seal").await?;
    let e = editor(&d, &auth, run).await?;
    let before = d.companies[0].snapshot(run).await?;
    let input = request(&e, e.address.clone(), &e.handle)?;
    let json = serde_json::to_vec(&input)?;
    let parsed = console_ontology_rest::editor::parse_json_editor_save(&json)?;
    let html = console_ontology_rest::editor::render_nojs_save_form(&input)?;
    // Parse real generated controls through ordinary HTML form parser. No separate
    // no-JS hidden raw reference, different binder, or browser-only trust exists.
    let parsed_html =
        console_ontology_rest::editor::parse_html_editor_save(&html.submission_fields())?;
    assert_eq!(parsed, parsed_html);
    let saved = completed(submit(&d, &auth, parsed).await?)?;
    let sealed = completed(
        ontology::draft_seal(
            &d.pool,
            &auth,
            UnadmittedControl {
                command_id: Uuid::new_v4(),
                input: DraftSeal {
                    draft_id: saved.draft_id,
                    expected_revision_id: saved.revision_id,
                    editing_token: saved.editing_token,
                    prepared_submission_unit: None,
                    gate_request_intent: e.binding.gate_request_intent,
                },
            },
        )
        .await?,
    )?;
    let retained =
        ontology::read_original_submission(&d.pool, &d.companies[0].submitter, &sealed.attempt)
            .await?;
    let decoded: payroll::RunCalculate = decode_registered_submission(&retained)?;
    assert_eq!(decoded.expected.close_basis, e.input.expected.close_basis);
    assert!(
        !retained
            .windows(e.handle.len())
            .any(|w| w == e.handle.as_bytes())
    );
    assert_eq!(d.companies[0].snapshot(run).await?, before);
    Ok(())
}
#[sqlx::test(migrations = false)]
async fn control_reference_cross_account_scope_field_schema_and_raw_reference_refuse(
    pool: PgPool,
) -> TestResult {
    let (d, auth, run, _browser) = setup(pool, "control-bindings").await?;
    let e = editor(&d, &auth, run).await?;
    let before = chain(&d, &e).await?;
    let business = d.companies[0].snapshot(run).await?;
    for other in [
        &d.companies[0].same_human_other_account,
        &d.companies[1].reviewer,
    ] {
        let r = ontology::draft_save_editor(
            &d.pool,
            other,
            UnadmittedControl {
                command_id: Uuid::new_v4(),
                input: request(&e, e.address.clone(), &e.handle)?,
            },
        )
        .await;
        assert!(matches!(
            r,
            Err(ontology::OwnerActionError::PermissionDenied)
                | Err(ontology::OwnerActionError::StaleContext)
                | Ok(OwnerExecution::Observation(
                    ResultObservation::DefinitiveNoEffect { .. }
                ))
        ));
        assert_eq!(chain(&d, &e).await?, before);
    }
    let field = resolve_registered_field_address(
        &e.binding.schema_snapshot,
        &["expected", "run_revision"],
    )?;
    assert_ne!(field.address, e.address);
    refused(submit(&d, &auth, request(&e, field.address, &e.handle)?).await?);
    assert_eq!(chain(&d, &e).await?, before);
    let schema = crate::lifecycle_producer::publish_upgrade(
        &d.companies[0],
        &e.binding.schema,
        e.address.field_id,
    )
    .await?;
    assert_ne!(schema.target, e.binding.schema);
    let new = editor(&d, &auth, run).await?;
    assert_ne!(new.binding.schema, e.binding.schema);
    refused(submit(&d, &auth, request(&new, new.address.clone(), &e.handle)?).await?);
    let mut raw = serde_json::to_value(request(&e, e.address.clone(), &e.handle)?)?;
    raw["current_controls"][0]["handle"] =
        serde_json::to_value(e.input.expected.close_basis.as_ref().unwrap())?;
    assert!(
        console_ontology_rest::editor::parse_json_editor_save(&serde_json::to_vec(&raw)?).is_err()
    );
    assert_eq!(chain(&d, &e).await?, before);
    assert_eq!(d.companies[0].snapshot(run).await?, business);
    Ok(())
}
#[sqlx::test(migrations = false)]
async fn control_reference_revocation_and_real_fifteen_minute_expiry_require_fresh_render(
    pool: PgPool,
) -> TestResult {
    let (d, mut auth, run, mut browser) = setup(pool, "control-expiry").await?;
    let e = editor(&d, &auth, run).await?;
    let before = chain(&d, &e).await?;
    let (created,expiry):(time::OffsetDateTime,time::OffsetDateTime)=sqlx::query_as("SELECT created_at,expires_at FROM ont_control_reference_handles WHERE handle_sha256=digest($1,'sha256')").bind(e.handle.as_bytes()).fetch_one(&d.readback).await?;
    assert_eq!(expiry - created, time::Duration::minutes(15));
    // Dedicated20minute gate. Server clock query avoids client clock authority.
    let mut last_refresh = tokio::time::Instant::now();
    loop {
        if last_refresh.elapsed() > std::time::Duration::from_secs(60) {
            browser
                .refresh(&d.runtime.client, &d.runtime.origin)
                .await?;
            auth = fresh_context(&d, 0, &browser.cookies["__Host-console_account_session"]).await?;
            last_refresh = tokio::time::Instant::now();
        }
        let now: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&d.readback)
            .await?;
        if now > expiry {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    refused(submit(&d, &auth, request(&e, e.address.clone(), &e.handle)?).await?);
    assert_eq!(chain(&d, &e).await?, before);
    let fresh = editor(&d, &auth, run).await?;
    assert_ne!(fresh.handle, e.handle);
    completed(
        submit(
            &d,
            &auth,
            request(&fresh, fresh.address.clone(), &fresh.handle)?,
        )
        .await?,
    )?;
    replace_projection(
        &d,
        0,
        d.accounts.reviewer.account_id,
        &[],
        "payroll.calculate.scoped_control",
    )
    .await?;
    assert!(matches!(
        projection::render_registered_draft_editor(&d.pool, &auth, fresh.ack.draft_id).await,
        Err(projection::ProjectionError::PermissionDenied)
            | Err(projection::ProjectionError::StaleContext)
    ));
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn control_reference_new_session_and_actual_switch_never_reuse_old_handle(
    pool: PgPool,
) -> TestResult {
    let (mut d, auth, run, _browser) = setup(pool, "control-session-context").await?;
    let e = editor(&d, &auth, run).await?;
    let before = chain(&d, &e).await?;
    let fresh_browser = login(
        &d.runtime.client,
        &d.runtime.origin,
        &mut d.accounts.reviewer,
    )
    .await?;
    let token = &fresh_browser.cookies["__Host-console_account_session"];
    let fresh = fresh_context(&d, 0, token).await?;
    refused(submit(&d, &fresh, request(&e, e.address.clone(), &e.handle)?).await?);
    assert_eq!(chain(&d, &e).await?, before);
    // Same Account has a real allowed B. Switch generation is produced by owner,
    // not a client-provided changed claim or fabricated active context.
    let selected = console_platform_request_context::account::switch_company_context(
        &d.runtime.auth,
        &d.verifier,
        token,
        d.companies[1].org,
        &d.serving,
    )
    .await?;
    let switched = console_platform_request_context::account::resolve_company_context(
        &d.runtime.auth,
        &d.verifier,
        token,
        &selected,
        &d.serving,
    )
    .await?;
    let result = ontology::draft_save_editor(
        &d.pool,
        &switched,
        UnadmittedControl {
            command_id: Uuid::new_v4(),
            input: request(&e, e.address.clone(), &e.handle)?,
        },
    )
    .await;
    assert!(matches!(
        result,
        Err(ontology::OwnerActionError::PermissionDenied)
            | Err(ontology::OwnerActionError::StaleContext)
            | Ok(OwnerExecution::Observation(
                ResultObservation::DefinitiveNoEffect { .. }
            ))
    ));
    assert_eq!(chain(&d, &e).await?, before);
    Ok(())
}
#[sqlx::test(migrations = false)]
async fn control_reference_actual_different_action_draft_cannot_consume_handle(
    pool: PgPool,
) -> TestResult {
    let (d, auth, run, _browser) = setup(pool, "control-action").await?;
    let e = editor(&d, &auth, run).await?;
    let (other_run, _) = d.companies[0].calculated().await?;
    // Give the SAME Account the additional registered action through ordinary
    // policy replacement, then render NEW original action handle at this policy.
    replace_projection(
        &d,
        0,
        d.accounts.reviewer.account_id,
        &[
            "context.discover",
            "payroll.calculate_run",
            "payroll.open_review_cycle",
            "draft.start",
            "draft.save",
            "draft.seal",
        ],
        "payroll.calculate.scoped_control",
    )
    .await?;
    let auth = fresh_context(&d, 0, &d.accounts.reviewer.account_access_token).await?;
    let e = editor(&d, &auth, run).await?;
    let selection = explicit_new_intent::<payroll::ReviewOpen>(
        &d.pool,
        &auth,
        &UntrustedActionTargetSelection::existing_run(other_run.run_id),
    )
    .await?;
    let binding =
        ontology::read_registered_draft_binding::<payroll::ReviewOpen>(&d.pool, &auth, &selection)
            .await?;
    assert_ne!(binding.action, e.binding.action);
    let field =
        resolve_registered_field_address(&binding.schema_snapshot, &["expected", "close_basis"])?;
    let started = completed(
        ontology::draft_start(
            &d.pool,
            &auth,
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
    let original = chain(&d, &e).await?;
    let business = d.companies[0].snapshot(other_run.run_id).await?;
    let input = DraftEditorSave {
        save: DraftSave {
            draft_id: started.draft_id,
            expected_revision_id: started.revision_id,
            editing_token: started.editing_token,
            patches: vec![],
        },
        current_controls: vec![SubmittedControlReference {
            address: field.address,
            handle: ControlHandle::parse(&e.handle)?,
        }],
    };
    refused(submit(&d, &auth, input).await?);
    assert_eq!(chain(&d, &e).await?, original);
    assert_eq!(d.companies[0].snapshot(other_run.run_id).await?, business);
    Ok(())
}
