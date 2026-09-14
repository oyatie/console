//! Classified refusal is paired with actual same-actor valid-input execution in callers.
//! This helper proves no effect, not a specific undisclosed refusal cause.
//! A registered semantic refusal may be detected during ordinary preparation,
//! sealing, or final execution. Transport/implementation errors never satisfy it.
#[macro_export]
macro_rules! assert_registered_refusal {
 ($pool:expr,$auth:expr,$target:expr,$input:expr,$owner:path)=>{
  $crate::assert_registered_refusal!($pool,$auth,$target,$input,$owner; reason = None)
 };
 ($pool:expr,$auth:expr,$target:expr,$input:expr,$owner:path; reason = $reason:expr)=>{
  match $crate::binder_sequence::bind_and_seal_with_reason($pool,$auth,$target,$input,$reason).await {
   Ok(attempt)=>assert!(matches!($owner($pool,$auth,&attempt).await?,
    console_ontology_application::action30::OwnerExecution::Observation(
      console_ontology_application::action30::ResultObservation::DefinitiveNoEffect{..}))),
   Err($crate::binder_sequence::ProducerStop::Observation(
     console_ontology_application::action30::ResultObservation::DefinitiveNoEffect{..}))=>{},
   Err(error)=>return Err(error.into()),
  }
 };
}
