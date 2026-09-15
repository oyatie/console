//! COMPOSED boundary proof: persisted two-member sources and actual decisions;
//! denied-scalar counterfactual at the real pure projector. This does not claim
//! a second persisted same-line tax/calculation world or a replacement evaluator.
use crate::{
    native_fixture::TestResult,
    native_projection_alias_acceptance::grant_visible_line,
    projector_observation_collector::{equal, observe},
    security_grant_producer::fresh_context,
};
use console_ontology_application::projection::{
    self, OwnerProjectionSource, PublicObservationInputs,
};
use console_payroll_adapter_postgres::action30 as payroll;
use serde_json::{Value, json};
use sqlx::PgPool;
fn replace_one(
    original: &OwnerProjectionSource,
    pointer: &str,
    value: Value,
) -> TestResult<OwnerProjectionSource> {
    let mut changed = serde_json::to_value(original)?;
    let old = changed
        .pointer(pointer)
        .ok_or("source-bound scalar path")?
        .clone();
    assert_ne!(old, value);
    *changed.pointer_mut(pointer).ok_or("source scalar")? = value;
    let typed: OwnerProjectionSource = serde_json::from_value(changed.clone())?;
    // Reverse exactly one scalar and compare the entire source tree. No payroll
    // fields, revision, provenance, ordering, absence or public shape silently vary.
    *changed.pointer_mut(pointer).unwrap() = old;
    assert_eq!(changed, serde_json::to_value(original)?);
    Ok(typed)
}
#[sqlx::test(migrations = false)]
async fn proj_actual_two_member_decisions_hide_denied_scalar_across_all_concrete_sink_encoders(
    pool: PgPool,
) -> TestResult {
    let d = crate::two_subject_producer::build_two_supported_deployment(
        pool,
        "projector-two-member-counterfactual",
    )
    .await?;
    let f = &d.companies[0];
    let second = f
        .facts
        .second_supported
        .as_ref()
        .ok_or("real two-member producer")?;
    let (run, _) = f.calculated().await?;
    let snapshot = f.snapshot(run.run_id).await?;
    assert_eq!(snapshot["lines"].as_array().ok_or("lines")?.len(), 2);
    assert_eq!(snapshot["money"].as_array().ok_or("money")?.len(), 2);
    grant_visible_line(&d, run.run_id).await?;
    let auth = fresh_context(&d, 0, &d.accounts.reviewer.account_access_token).await?;
    let schema = payroll::read_native_projection_schema(&d.pool, &auth, run.run_id).await?;
    // Public clock, random tape and incident ID are explicit ordinary request
    // inputs. Same tape is replayed into each encoder, never redacted afterwards.
    let env = PublicObservationInputs::from_request_inputs(
        time::OffsetDateTime::parse(
            "2026-09-14T12:00:00Z",
            &time::format_description::well_known::Rfc3339,
        )?,
        [19u8; 32],
        uuid::Uuid::parse_str("e328fb61-b387-44d8-80a7-a172b26bfc3f")?,
    );
    for key in [
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
    ] {
        let purpose = schema.resolve_registered_purpose(key)?;
        // Current owner loads private plain source operands and creates nonforgeable
        // current authority. Only the source is cloneable/deserializable.
        let loaded =
            payroll::load_native_projection_operands(&d.pool, &auth, run.run_id, purpose.clone())
                .await?;
        assert_eq!(
            loaded.source.member_ids(),
            std::collections::BTreeSet::from([f.facts.employee_id, second.workforce.employee_id])
        );
        for employee in [f.facts.employee_id, second.workforce.employee_id] {
            let line = snapshot["lines"]
                .as_array()
                .unwrap()
                .iter()
                .find(|l| l["employee_id"] == json!(employee))
                .ok_or("persisted source line")?;
            assert_eq!(
                loaded.source.member_line_id(employee)?,
                serde_json::from_value::<uuid::Uuid>(line["id"].clone())?
            );
        }
        let allowed = loaded
            .source
            .resolve_scalar(f.facts.employee_id, &["gross_won"])?;
        let hidden = loaded
            .source
            .resolve_scalar(second.workforce.employee_id, &["gross_won"])?;
        let same_line_hidden = loaded
            .source
            .resolve_scalar(f.facts.employee_id, &["net_won"])?;
        assert!(loaded.authority.permits(&allowed.address));
        assert!(!loaded.authority.permits(&hidden.address));
        assert!(!loaded.authority.permits(&same_line_hidden.address));
        assert_eq!(
            serde_json::to_value(&loaded.source)?.pointer(&allowed.json_pointer),
            Some(&json!("3000000"))
        );
        let baseline =
            projection::project_authorized_result(&auth, &loaded.source, &loaded.authority, &env)?;
        // Actual public owner caller must use this exact loader/projector, evidenced
        // by same-input equality, plus required exact-source caller binding gate.
        let caller = payroll::read_native_authorized_projection_with_public_inputs(
            &d.pool, &auth, run.run_id, purpose, &env,
        )
        .await?;
        assert_eq!(
            console_ontology_rest::projection::encode_authorized_projection(&caller)?,
            console_ontology_rest::projection::encode_authorized_projection(&baseline)?
        );
        let before = observe(&baseline, &env).await?;
        assert!(!before.live.is_empty());
        assert!(!before.events.is_empty());
        assert!(!before.egress.is_empty());
        for secret in [&hidden, &same_line_hidden] {
            let changed = replace_one(&loaded.source, &secret.json_pointer, json!("7313371"))?;
            let projected =
                projection::project_authorized_result(&auth, &changed, &loaded.authority, &env)?;
            let after = observe(&projected, &env).await?;
            assert!(
                equal(&before, &after),
                "denied scalar affected {key} logical bytes/count/order"
            );
            assert!(!format!("{after:?}").contains("7313371"));
        }
        let visible = replace_one(&loaded.source, &allowed.json_pointer, json!("8424482"))?;
        let permitted =
            projection::project_authorized_result(&auth, &visible, &loaded.authority, &env)?;
        let after = observe(&permitted, &env).await?;
        if key == "detail" || key == "current" {
            assert!(!equal(&before, &after), "vacuous projector or collector");
        }
        let json = console_ontology_rest::projection::encode_authorized_projection(&permitted)?;
        if key == "detail" || key == "current" {
            assert!(serde_json::to_string(&json)?.contains("8424482"));
        }
        // Negative oracle control uses one extra ACTUAL individually safe event;
        // no checker can treat per-record sanitization as schedule equivalence.
        let mut extra = before.clone();
        let mut event = extra.events[0].clone();
        event["ordinal"] = json!(extra.events.len());
        extra.events.push(event);
        assert!(!equal(&before, &extra));
    }
    assert_eq!(f.snapshot(run.run_id).await?, snapshot);
    Ok(())
}
