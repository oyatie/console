//! Proposed test-composition source, not yet compiled or executable product evidence.
//! All new symbols are approval prerequisites declared in signature-contract.rs.
//! Inputs below are unsealed user choices, never future command identities.
use console_ontology_application::action30::*;
use console_ontology_adapter_postgres::action30 as ontology;
use console_platform_request_context::account::AuthenticatedCompanyContext;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub enum ProducerStop {
    Observation(ResultObservation),
    StaleDraft { draft_id: Uuid },
    InvalidActualResult(&'static str),
    Owner(Box<dyn std::error::Error + Send + Sync>),
}
impl std::fmt::Display for ProducerStop {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {write!(f,"{self:?}")}
}
impl std::error::Error for ProducerStop {
    fn source(&self)->Option<&(dyn std::error::Error+'static)> {
        match self {Self::Owner(error)=>Some(error.as_ref()),_=>None}
    }
}
pub fn owner_error<E:std::error::Error+Send+Sync+'static>(error:E)->ProducerStop {
    ProducerStop::Owner(Box::new(error))
}
pub fn completed<R>(value: OwnerExecution<R>) -> Result<R, ProducerStop> {
    match value {
        OwnerExecution::Completed(result) => Ok(result),
        OwnerExecution::Observation(observation) => Err(ProducerStop::Observation(observation)),
    }
}

pub async fn bind_and_seal<I: SealedRegisteredInput>(
    pool: &PgPool, auth: &AuthenticatedCompanyContext,
    selection: &UntrustedActionTargetSelection, input: &I,
) -> Result<AttemptRef, ProducerStop> {
    // Fixed I::REGISTRATION_KEY is sealed, not arbitrary runtime function dispatch.
    // Owner resolves current registration/schema/target/custody/intent slot and
    // exact gate intent under actual auth. Plain returned data grants no authority.
    let binding = ontology::read_registered_draft_binding::<I>(pool, auth, selection).await.map_err(owner_error)?;
    let started = completed(ontology::draft_start(pool, auth, UnadmittedControl {
        command_id: Uuid::new_v4(),
        input: DraftStart {
            action: binding.action, target: binding.target,
            schema: binding.schema.clone(), custody: binding.custody,
            intent_slot: binding.intent_slot,
        },
    }).await.map_err(owner_error)?)?;
    let patches = compile_registered_input_patches(&binding.schema_snapshot, input).map_err(owner_error)?;
    let saved = completed(ontology::draft_save(pool, auth, UnadmittedControl {
        command_id: Uuid::new_v4(),
        input: DraftSave {
            draft_id: started.draft_id, expected_revision_id: started.revision_id,
            editing_token: started.editing_token, patches,
        },
    }).await.map_err(owner_error)?)?;
    let prepared_submission_unit = if I::SOURCE24_SPECIALIZATION {
        // Sealed registration trait fixes this property for exactly source24 actions.
        let prepared = completed(ontology::source_prepare_submission(pool, auth, UnadmittedControl {
            command_id: Uuid::new_v4(),
            input: SourcePreparationRequest {
                draft_id: saved.draft_id, expected_revision_id: saved.revision_id,
                editing_token: saved.editing_token.clone(), input_schema: binding.schema,
            },
        }).await.map_err(owner_error)?)?;
        Some(prepared.unit)
    } else { None };
    // Preparation may advance continuation. Read the actual owner value; never
    // predict a nonce or silently adopt a concurrently changed draft head.
    let continuation = ontology::read_draft_continuation(pool, auth, saved.draft_id).await.map_err(owner_error)?;
    if continuation.revision_id != saved.revision_id {
        return Err(ProducerStop::StaleDraft { draft_id: saved.draft_id });
    }
    let sealed = completed(ontology::draft_seal(pool, auth, UnadmittedControl {
        command_id: Uuid::new_v4(),
        input: DraftSeal {
            draft_id: saved.draft_id, expected_revision_id: saved.revision_id,
            editing_token: continuation.editing_token,
            prepared_submission_unit, gate_request_intent: binding.gate_request_intent,
        },
    }).await.map_err(owner_error)?)?;
    Ok(sealed.attempt)
}

// Explicit user NEW REQUEST, not randomization during GET/resume. The fixed
// owner issues and persists an intent slot under existing action/target/custody.
// The returned selection carries its actual owner-issued slot into normal binder.
pub async fn explicit_new_intent<I:SealedRegisteredInput>(
    pool:&PgPool,auth:&AuthenticatedCompanyContext,selection:&UntrustedActionTargetSelection,
)->Result<UntrustedActionTargetSelection,ProducerStop> {
    let issued=ontology::request_new_draft_intent::<I>(pool,auth,UnadmittedControl {
        command_id:Uuid::new_v4(), input:ExplicitNewDraftIntent { selection:selection.clone() },
    }).await.map_err(owner_error)?;
    let actual=completed(issued)?;
    Ok(selection.clone().with_owner_issued_intent_slot(actual.intent_slot))
}

pub trait ReceiptedResult { fn receipt(&self)->&ReceiptRef; }
impl ReceiptedResult for console_payroll_adapter_postgres::action30::RunResult {
    fn receipt(&self)->&ReceiptRef { &self.receipt }
}
impl ReceiptedResult for console_payroll_adapter_postgres::action30::ReviewResult {
    fn receipt(&self)->&ReceiptRef { &self.receipt }
}
// If payroll/governance reexport one ReviewResult, the single impl above covers
// both. If separate DTOs exist, an equivalent concrete governance impl is needed.
pub fn completed_attempt<R:ReceiptedResult>(value:OwnerExecution<R>,attempt:&AttemptRef)->Result<R,ProducerStop> {
    let result=completed(value)?;
    match result.receipt() {
        ReceiptRef::Company{command,..} if command.org_id==attempt.org_id && command.command_id==attempt.command_id=>Ok(result),
        _=>Err(ProducerStop::InvalidActualResult("terminal owner receipt not original attempt")),
    }
}
