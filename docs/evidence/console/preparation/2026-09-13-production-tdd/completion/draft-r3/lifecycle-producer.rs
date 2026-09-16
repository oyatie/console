//! Ordinary schema/retention configuration producer, not direct database setup.
//! All arguments are untrusted choices; the owning configuration APIs enforce
//! current Cedar schema/retention administration authority and append receipts.
use console_ontology_adapter_postgres::{action30 as ontology, schema_lifecycle as schema, retention};
use console_ontology_application::action30::*;
use crate::native_fixture::{Fixture, TestResult};
use uuid::Uuid;

pub struct UpgradeFacts {
    pub original: InputSchemaRef,
    pub target: InputSchemaRef,
    pub mapping: GovernedRevision,
    pub intentionally_partial_mapping: GovernedRevision,
}

pub async fn publish_upgrade(f: &Fixture, current: &InputSchemaRef, populated_field: Uuid) -> TestResult<UpgradeFacts> {
    // Read the full current administrative schema, not a policy-redacted user
    // projection. Native bootstrap grants this scoped operation through normal
    // policy administration; a job title supplies no such privilege.
    let original = schema::read_schema_for_administration(&f.pool, &f.submitter, current).await?;
    assert_eq!(&original.reference, current);
    let mut fields = original.fields.clone();
    assert!(!fields.is_empty());
    assert!(fields.iter().any(|field|field.field_id==populated_field && field.required),"exact populated required field belongs to actual schema");
    fields[0].label = format!("{} (clarified)", fields[0].label);
    let proposal = schema::SchemaRevisionProposal {
        predecessor: current.clone(), fields,
        action_registration: original.action_registration.clone(),
        expected_policy_revision: original.current_policy_revision,
    };
    let target = schema::publish_schema_revision(&f.pool, &f.submitter,
        UnadmittedControl { command_id: Uuid::new_v4(), input: proposal }).await?;
    assert_ne!(&target.reference, current);
    // Stable field IDs and exact kinds are copied; no semantic inference or LLM.
    let copies: Vec<_> = original.fields.iter().map(|field| schema::FieldMapping::Copy {
        from_field_id: field.field_id, to_field_id: field.field_id,
        expected_kind: field.kind.clone(),
    }).collect();
    let mapping = schema::publish_mapping_revision(&f.pool, &f.submitter, UnadmittedControl {
        command_id: Uuid::new_v4(), input: schema::MappingProposal {
            source: current.clone(), target: target.reference.clone(), fields: copies.clone(),
        },
    }).await?;
    // Partial mappings are legal proposed configuration because input-dependent
    // mapping may refuse. They are never advertised as total; upgrade must retain
    // complete content or refuse. Publishing one does not rewrite any draft.
    let partial = schema::publish_mapping_revision(&f.pool, &f.submitter, UnadmittedControl {
        command_id: Uuid::new_v4(), input: schema::MappingProposal {
            source: current.clone(), target: target.reference.clone(), fields: copies.into_iter().filter(|mapping| !matches!(mapping, schema::FieldMapping::Copy { from_field_id, .. } if *from_field_id == populated_field)).collect(),
        },
    }).await?;
    Ok(UpgradeFacts { original: current.clone(), target: target.reference, mapping: mapping.reference, intentionally_partial_mapping: partial.reference })
}

pub async fn install_short_editable_retention(f: &Fixture, draft: Uuid) -> TestResult<GovernedRevision> {
    let current=retention::read_draft_policy_control(&f.pool,&f.submitter,draft).await?;
    // Synthetic fixture Company only; this does not assert a legal default.
    // One-second duration is ordinary configurable data under current authority.
    let policy=retention::publish_policy_revision(&f.pool,&f.submitter,UnadmittedControl {
        command_id:Uuid::new_v4(),input:retention::PolicyProposal {
            expected:current.policy_revision, scope:retention::PolicyScope::ExactDraft(draft),
            editable_inactivity_seconds:1, permit_payload_purge:false,
        },
    }).await?;
    Ok(policy.reference)
}

pub async fn hold(f:&Fixture,draft:Uuid)->TestResult<GovernedRevision> {
    let current=retention::read_draft_policy_control(&f.pool,&f.submitter,draft).await?;
    let result=retention::place_hold(&f.pool,&f.submitter,UnadmittedControl {
        command_id:Uuid::new_v4(), input:retention::HoldProposal {
            target:UntrustedActionTargetSelection::existing_draft(draft),
            expected_policy_revision:current.policy_revision,
            reason:"Synthetic preservation exercise".into(),
        },
    }).await?;
    Ok(result.reference)
}

pub async fn wait_until_due(f:&Fixture,draft:Uuid,policy:&GovernedRevision)->TestResult {
    let deadline=tokio::time::Instant::now()+std::time::Duration::from_secs(5);
    loop {
        let read=retention::read_draft_expiry_eligibility(&f.pool,&f.worker,draft,policy).await?;
        assert_eq!(&read.policy,policy);
        // Eligibility is current owner observation, not a caller-written clock.
        if read.server_now>=read.expiry_at {return Ok(());}
        if tokio::time::Instant::now()>=deadline {return Err("fixture server clock never reached expiry deadline".into());}
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
}
