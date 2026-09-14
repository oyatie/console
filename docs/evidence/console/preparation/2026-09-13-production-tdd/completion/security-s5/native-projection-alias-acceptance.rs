//! Ordinary owner projection over actual native state. This is the owner alias
//! matrix, not a claim of HTTP/SSR/live adapter enrollment or clock coupling.
use crate::{
    binder_sequence::{bind_and_seal, completed},
    native_fixture::{NativeDeployment, TestResult, build_native_deployment},
    security_grant_producer::{fresh_context, replace_projection},
};
use console_ontology_application::action30::*;
use console_payroll_adapter_postgres::action30 as payroll;
use console_platform_request_context::account::AuthenticatedCompanyContext;
use serde_json::{Value, json};
use sqlx::PgPool;
const ALIASES: [&str; 11] = [
    "list",
    "detail",
    "history",
    "search",
    "graph",
    "link_title",
    "count",
    "conflict",
    "preflight",
    "replay",
    "current",
];
pub(crate) async fn read_all(
    d: &NativeDeployment,
    auth: &AuthenticatedCompanyContext,
    run: uuid::Uuid,
) -> TestResult<(Vec<Value>, Vec<String>)> {
    let schema = payroll::read_native_projection_schema(&d.pool, auth, run).await?;
    let mut actual = Vec::new();
    let mut handles = Vec::new();
    for key in ALIASES {
        let purpose = schema.resolve_registered_purpose(key)?;
        let projection =
            payroll::read_native_authorized_projection(&d.pool, auth, run, purpose).await?;
        // Only authorized output serializer, never serde(private RunResult).
        let mut encoded =
            console_ontology_rest::projection::encode_authorized_projection(&projection)?;
        // OR30 explicitly permits independently random handles. Compare semantic
        // authorized objects; separately revalidate every actual earlier handle.
        let handle = encoded
            .as_object_mut()
            .ok_or("authorized object")?
            .remove("snapshot_revision")
            .ok_or("actual scoped projection handle")?;
        let handle = handle.as_str().ok_or("opaque handle")?.to_string();
        assert_eq!(handle.len(), 43);
        handles.push(handle);
        actual.push(encoded);
    }
    Ok((actual, handles))
}
pub(crate) async fn grant_visible_line(d: &NativeDeployment, run: uuid::Uuid) -> TestResult {
    use console_identity_adapter_postgres::account13 as identity;
    let f = &d.companies[0];
    let account = d.accounts.reviewer.account_id;
    replace_projection(
        d,
        0,
        account,
        &["context.discover"],
        "context.identity_only",
    )
    .await?;
    let current_submitter = fresh_context(d, 0, &d.accounts.submitter.account_access_token).await?;
    let resource = payroll::read_native_line_policy_resource(
        &d.pool,
        &current_submitter,
        run,
        f.facts.employee_id,
    )
    .await?;
    let schema = identity::read_company_policy_schema(&d.pool, &d.operator, f.org).await?;
    let gross = resource.resolve_property_path(&["gross_won"])?;
    let key = "test.native.visible_gross";
    let expected = identity::read_field_group_head(&d.pool, &d.operator, f.org, key).await?;
    let group = identity::publish_field_group(
        &d.pool,
        &d.operator,
        identity::PublishFieldGroup {
            command_id: uuid::Uuid::new_v4(),
            org_id: f.org,
            key: key.into(),
            expected,
            fields: vec![gross],
            reason: "TEST_ONLY actual line gross only".into(),
        },
    )
    .await?;
    // ExactResource is existing PA1.2 typed ObjectRef/schema scope, shared with
    // Workflow15 release tests. Neither UUID nor returned ref confers authority.
    for (object, field_projection) in [
        (resource.line.clone(), group.reference),
        (
            resource.run.clone(),
            schema.projection("payroll.run.identity")?,
        ),
    ] {
        let expected = identity::read_company_policy_control(&d.pool, &d.operator, f.org).await?;
        identity::apply_company_grant_plan(
            &d.pool,
            &d.operator,
            identity::ApplyCompanyGrantPlan {
                command_id: uuid::Uuid::new_v4(),
                org_id: f.org,
                expected,
                plan: identity::CompanyGrantPlan {
                    account_id: account,
                    scope: identity::PolicyScope::ExactResource {
                        resource: object.object,
                        schema: object.schema,
                    },
                    actions: schema.actions_for_native_projection()?,
                    field_projection,
                    valid_from: None,
                    valid_to: None,
                    reason: "TEST_ONLY current native projection".into(),
                },
            },
        )
        .await?;
    }
    Ok(())
}
async fn make_hidden_wage(
    d: &NativeDeployment,
    company: usize,
    amount: &str,
) -> TestResult<uuid::Uuid> {
    let f = &d.companies[company];
    let worker = &f.facts.unprepared_workforce;
    let control = payroll::read_contract_wage_creation_control(
        &d.pool,
        &f.submitter,
        worker.employee_id,
        worker.employment.employment_id,
    )
    .await?;
    let evidence = console_docs_adapter_postgres::action30::read_governed_artifact_revision(
        &d.pool,
        &f.submitter,
        &f.facts.sources.evidence,
    )
    .await?;
    let input: payroll::WageCreate = serde_json::from_value(
        json!({"employee_id":worker.employee_id,"employment_id":worker.employment.employment_id,"amount_won":amount,"wage_kind":"MONTHLY","monthly_standard_hours":209,"effective_from":"2026-06-01","source_note":"TEST_ONLY hidden employee salary","evidence_refs":[evidence],"subject_source_generation":control.subject_source_generation}),
    )?;
    let attempt = bind_and_seal(&d.pool, &f.submitter, &control.selection, &input).await?;
    let made =
        completed(payroll::payroll_create_contract_wage(&d.pool, &f.submitter, &attempt).await?)?;
    // Real authorized owner read supplies nonvacuity; literal request alone is no
    // proof that hidden-world mutation reached persisted state.
    let actual = payroll::read_contract_wage(&d.pool, &f.submitter, &made.fact).await?;
    assert_eq!(actual.amount_won.to_string(), amount);
    Ok(made.fact.base_wage_id)
}
#[sqlx::test(migrations = false)]
async fn native_aliases_hide_other_company_changes_while_emitting_permitted_salary(
    pool: PgPool,
) -> TestResult {
    let d = build_native_deployment(pool, "native-projection-aliases", 2).await?;
    let (run, _) = d.companies[0].calculated().await?;
    grant_visible_line(&d, run.run_id).await?;
    let auth = fresh_context(&d, 0, &d.accounts.reviewer.account_access_token).await?;
    let (before, handles) = read_all(&d, &auth, run.run_id).await?;
    assert!(
        serde_json::to_string(&before)?.contains("3000000"),
        "permitted salary must actually emit"
    );
    let protected = d.companies[0].snapshot(run.run_id).await?;
    make_hidden_wage(&d, 1, "7313371").await?;
    let (other, _) = d.companies[1].calculated().await?;
    assert_ne!(other.run_id, run.run_id);
    assert_eq!(
        read_all(&d, &auth, run.run_id).await?.0,
        before,
        "cross-Company private state influenced A"
    );
    let (after, _) = read_all(&d, &auth, run.run_id).await?;
    assert_eq!(after, before);
    for handle in handles {
        payroll::revalidate_native_projection_handle(&d.pool, &auth, &handle).await?;
    }
    let output = serde_json::to_string(&after)?;
    for forbidden in ["7313371", "8424482", "TEST_ONLY hidden employee salary"] {
        assert!(!output.contains(forbidden));
    }
    assert_eq!(
        d.companies[0].snapshot(run.run_id).await?,
        protected,
        "B-owned command changed untouched A run"
    );
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn native_same_run_two_real_members_expose_only_actual_granted_line_gross(
    pool: PgPool,
) -> TestResult {
    let d = crate::two_subject_producer::build_two_supported_deployment(
        pool,
        "same-run-field-projection",
    )
    .await?;
    let f = &d.companies[0];
    let second = f
        .facts
        .second_supported
        .as_ref()
        .ok_or("real second supported source")?;
    let (run, _) = f.calculated().await?;
    let before = f.snapshot(run.run_id).await?;
    let lines = before["lines"].as_array().ok_or("actual line rows")?;
    assert_eq!(lines.len(), 2);
    let ids = lines
        .iter()
        .map(|l| serde_json::from_value::<uuid::Uuid>(l["employee_id"].clone()))
        .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
    assert_eq!(
        ids,
        std::collections::BTreeSet::from([f.facts.employee_id, second.workforce.employee_id])
    );
    assert_eq!(
        before["money"]
            .as_array()
            .ok_or("actual calculation rows")?
            .len(),
        2
    );
    grant_visible_line(&d, run.run_id).await?;
    let auth = fresh_context(&d, 0, &d.accounts.reviewer.account_access_token).await?;
    let (projected, _) = read_all(&d, &auth, run.run_id).await?;
    let bytes = serde_json::to_string(&projected)?;
    assert!(bytes.contains("3000000"));
    assert!(!bytes.contains(&second.workforce.employee_id.to_string()));
    // Scalar names come from actual registered schema; no serialized current/global
    // totals or hidden tax fields can stand in for the permitted gross field.
    let current_submitter =
        fresh_context(&d, 0, &d.accounts.submitter.account_access_token).await?;
    let resource = payroll::read_native_line_policy_resource(
        &d.pool,
        &current_submitter,
        run.run_id,
        f.facts.employee_id,
    )
    .await?;
    let gross = resource.resolve_property_path(&["gross_won"])?;
    assert!(contains_field_id(
        &projected,
        &resource.public_field_address(&gross)?.field_id
    ));
    for path in [["net_won"], ["total_deductions_won"]] {
        let field = resource.resolve_property_path(&path)?;
        let address = resource.public_field_address(&field)?;
        assert!(!contains_field_id(&projected, &address.field_id));
    }
    assert_eq!(f.snapshot(run.run_id).await?, before);
    Ok(())
}

// Inspect the actual projected field node key; PropertyRef serialization is not
// the transport shape and would allow a false negative.
fn contains_field_id(values: &[Value], field_id: &str) -> bool {
    fn visit(v: &Value, id: &str) -> bool {
        match v {
            Value::Object(m) => {
                m.get("field_id").and_then(Value::as_str) == Some(id)
                    || m.values().any(|x| visit(x, id))
            }
            Value::Array(a) => a.iter().any(|x| visit(x, id)),
            _ => false,
        }
    }
    values.iter().any(|v| visit(v, field_id))
}
