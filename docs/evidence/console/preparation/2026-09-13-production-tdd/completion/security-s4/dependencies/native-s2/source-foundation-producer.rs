//! Actual synthetic evidence is signed, uploaded, verified, admitted and reviewed.
//! Installed kernel custody is verified against the running build by its ordinary
//! deployment owner before any qualification/profile may bind its references.
use console_ontology_application::action30::*;
use console_payroll_adapter_postgres::action30 as payroll;
use console_docs_adapter_postgres::action30 as docs;
use console_identity_adapter_postgres::account13 as identity;
use console_platform_auth::account_session::VerifiedAccountSession;
use console_platform_request_context::{account::AuthenticatedCompanyContext,worker::AuthenticatedWorkerContext};
use serde_json::{json,Value};
use sqlx::PgPool;
use uuid::Uuid;
use crate::{artifact_owner_producer::{upload_and_verify,VerifiedArtifact},ordinary_issuer_bootstrap::FixtureIssuer,
 native_fixture::TestResult,workforce_owner_producer::WorkforceFacts,
 source_owner_producer::produce_verified_qualification};

pub fn signed_issuer_document(compact:&str)->TestResult<Vec<u8>> {
 let parts=compact.split('.').collect::<Vec<_>>();
 if parts.len()!=3||parts.iter().any(|p|p.is_empty()){return Err("invalid actual compact JWS".into());}
 // RFC7515 flattened JWS JSON serialization preserves original signing input;
 // the ordinary pinned issuer verifier checks protected alg/kid, exact payload,
 // signature and three-member closed envelope. MIME is genuinely JSON.
 Ok(serde_json::to_vec(&json!({"protected":parts[0],"payload":parts[1],"signature":parts[2]}))?)
}
pub struct ArtifactChannel<'a> {pub client:&'a reqwest::Client,pub origin:&'a url::Url,pub token:&'a str}
pub async fn upload_source(
 pool:&PgPool,auth:&AuthenticatedCompanyContext,worker:&AuthenticatedWorkerContext,
 channel:&ArtifactChannel<'_>,mime:&str,bytes:Vec<u8>,
)->TestResult<VerifiedArtifact> {
 let control=docs::read_upload_request_control(pool,auth,"payroll.source.evidence").await?;
 upload_and_verify(pool,auth,worker,channel.client,channel.origin,channel.token,&control.selection,
  control.purpose,mime,bytes).await
}
#[derive(Clone)]
pub struct SourceFoundations {
 pub credential:payroll::QualificationAdmissionControl,
 pub submitter_credential:payroll::QualificationAdmissionControl,
 pub calendar:payroll::QualificationAdmissionControl,
 pub applicability:payroll::QualificationAdmissionControl,
 pub tax:payroll::QualificationAdmissionControl,
 pub bundle:payroll::QualificationAdmissionControl,
 pub profile:payroll::ProfileRef,
 pub evidence:payroll::ArtifactRef,
 pub tax_artifact:payroll::ArtifactRef,
 pub calendar_payload:payroll::AdmissionPayload,
 pub installed:payroll::InstalledKernelBuild,
 pub rule_bundle:payroll::RuleBundleRef,
}
fn admission(payload:Value)->TestResult<payroll::AdmissionPayload> {
 Ok(serde_json::from_value(json!({"family":"QUALIFICATION","payload":payload,"supersedes_ref":null}))?)
}
pub async fn produce_source_foundations(
 pool:&PgPool,submitter:&AuthenticatedCompanyContext,reviewer:&AuthenticatedCompanyContext,
 worker:&AuthenticatedWorkerContext,operator:&VerifiedAccountSession,
 workforce:&WorkforceFacts,issuer:&FixtureIssuer,provider:&identity::IdentityProviderRevision,
 review_scope:&payroll::ScopeBasisRef,channel:&ArtifactChannel<'_>,compiled_manifest:&std::path::Path,
)->TestResult<SourceFoundations> {
 // Ordinary build-admission owner reads signed build custody and hashes actual
 // installed code/rule bodies. It rejects a manifest for a different executable.
 let installed=payroll::admit_installed_kernel_build(pool,operator,compiled_manifest).await?;
 assert_eq!(installed.domain_source_sha256,"34176ef3b404ee744edf1da8ea233e5a7fa909192707143356df2ef65adb4a92");
 assert_eq!(installed.build_ref_test,"builds_employee_deduction_draft_from_effective_rates_and_supplied_nts_row");
 assert_eq!(installed.tax_table_display_name,"NTS-간이세액표-fixture-row-v1");
 assert_eq!(installed.tax_table_edition,"NTS_FIXTURE_ROW_V1"); // closed registered ASCII ID
 assert_eq!(installed.environment,payroll::EvidenceEnvironment::TestOnly);
 let issuer_profile=identity::register_professional_issuer_profile(pool,operator,
  identity::ProfessionalIssuerRegistration{command_id:Uuid::new_v4(),org_id:submitter.company_id(),
   provider:provider.clone(),profession_id:"TEST_PAYROLL_REVIEWER".into(),jurisdiction_id:"TEST_KR".into(),
   purpose_ids:vec!["SOURCE_REVIEW".into(),"PAYROLL_REVIEW".into()],scope_basis:review_scope.clone(),
   evidence_classification:identity::EvidenceClassification::Synthetic,
   evidence_serialization:identity::IssuerEvidenceSerialization::JwsFlattenedJsonEs256V1}).await?;
 let now=time::OffsetDateTime::now_utc().unix_timestamp();
 let issuer_credential_id=format!("SYNTHETIC-{}",Uuid::new_v4());
 let claims=json!({"iss":issuer.issuer(),"aud":"console.source-credential.v1","iat":now,"nbf":now,"exp":now+3600,"jti":Uuid::new_v4(),
  "classification":"SYNTHETIC","org_id":submitter.company_id(),"provider":provider,"issuer_trust_profile":issuer_profile.reference,
  "subject_person_id":workforce.reviewer_person_id,"issuer_credential_id":issuer_credential_id,"profession_id":"TEST_PAYROLL_REVIEWER","jurisdiction_id":"TEST_KR",
  "purpose_ids":["SOURCE_REVIEW","PAYROLL_REVIEW"],"scope_basis_ref":review_scope});
 let signed=issuer.sign(&claims)?;
 let credential_artifact=upload_source(pool,submitter,worker,channel,"application/json",signed_issuer_document(&signed)?).await?;
 let collection=payroll::read_source_collection_selection(pool,submitter,"QUALIFICATION").await?;
 let credential=produce_verified_qualification(pool,submitter,reviewer,&collection,admission(json!({
  "kind":"REVIEWER_CREDENTIAL","subject_person_id":workforce.reviewer_person_id,
  "issuer_id":issuer_profile.issuer_id,"issuer_credential_id":issuer_credential_id,
  "profession_id":"TEST_PAYROLL_REVIEWER","jurisdiction_id":"TEST_KR","purpose_ids":["SOURCE_REVIEW","PAYROLL_REVIEW"],
  "scope_basis_ref":review_scope,"valid_from_us":(now*1_000_000).to_string(),"valid_to_us":((now+3600)*1_000_000).to_string(),
  "issuer_trust_profile_ref":issuer_profile.reference,"verification_method":"SIGNED_ISSUER_RESPONSE",
  "issuer_evidence_refs":[credential_artifact.source_artifact] }))?).await?;
 // The submitter's Human also has a real professional credential. Same-Human
 // negative tests therefore reach independence guards with otherwise qualified
 // evidence; absence of a credential cannot accidentally satisfy the oracle.
 let submitter_credential_id=format!("SYNTHETIC-{}",Uuid::new_v4());
 let mut submitter_claims=claims.clone();
 submitter_claims["subject_person_id"]=json!(workforce.person_id);
 submitter_claims["issuer_credential_id"]=json!(submitter_credential_id);
 submitter_claims["jti"]=json!(Uuid::new_v4());
 let submitter_artifact=upload_source(pool,submitter,worker,channel,"application/json",signed_issuer_document(&issuer.sign(&submitter_claims)?)?).await?;
 let collection=payroll::read_source_collection_selection(pool,reviewer,"QUALIFICATION").await?;
 let submitter_credential=produce_verified_qualification(pool,reviewer,submitter,&collection,admission(json!({
  "kind":"REVIEWER_CREDENTIAL","subject_person_id":workforce.person_id,
  "issuer_id":issuer_profile.issuer_id,"issuer_credential_id":submitter_credential_id,
  "profession_id":"TEST_PAYROLL_REVIEWER","jurisdiction_id":"TEST_KR","purpose_ids":["SOURCE_REVIEW","PAYROLL_REVIEW"],
  "scope_basis_ref":review_scope,"valid_from_us":(now*1_000_000).to_string(),"valid_to_us":((now+3600)*1_000_000).to_string(),
  "issuer_trust_profile_ref":issuer_profile.reference,"verification_method":"SIGNED_ISSUER_RESPONSE",
  "issuer_evidence_refs":[submitter_artifact.source_artifact] }))?).await?;
 // Qualified review is signed input evidence for a concrete installed build and
 // intended source statements. The source owner still verifies current reviewer
 // credential, professional scope and Human/task independence at admission.
 let source_statement=json!({"iss":issuer.issuer(),"aud":"console.payroll-source.v1","iat":now,"nbf":now,"exp":now+3600,"classification":"SYNTHETIC","org_id":submitter.company_id(),
  "employee_id":workforce.employee_id,"employment":workforce.employment,"kernel":installed.build_ref,
  "period":{"start":"2026-06-01","end":"2026-06-30"},"gross_won":"3000000","standard_hours":"209",
  "pension_basis_won":"3000000","remuneration_won":"3000000","taxable_won":"3000000",
  "dependent_count":"1","eligible_child_count":"0","withholding_option":"P100",
  "insurance":["NATIONAL_PENSION","HEALTH","LONG_TERM_CARE","EMPLOYMENT"],
  "income_tax_won":"74350","local_tax_won":"7430","issuer":issuer.issuer(),"jti":Uuid::new_v4()});
 let source_bytes=signed_issuer_document(&issuer.sign(&source_statement)?)?;
 let evidence=upload_source(pool,submitter,worker,channel,"application/json",source_bytes).await?.source_artifact;
 let review_claim=json!({"iss":issuer.issuer(),"aud":"console.source-review.v1","iat":now,"exp":now+3600,"jti":Uuid::new_v4(),
  "org_id":submitter.company_id(),"credential":credential.admission_ref,"artifact":evidence,
  "kernel_build":installed.build_ref,"classification":"SYNTHETIC","decision":"SUPPORTED_FOR_TEST"});
 let reviewed=upload_source(pool,submitter,worker,channel,"application/json",signed_issuer_document(&issuer.sign(&review_claim)?)?).await?.source_artifact;
 let citation=json!({"instrument":"Synthetic existing domain regression","edition":"NATIVE_DOMAIN_2026_V1","source_url":null,
  "artifact_refs":[evidence],"qualified_review_refs":[reviewed],"applicability":{"from":"2026-05-31","to_exclusive":"2026-07-01"}});
 let calendar_payload=admission(json!({"kind":"WORK_CALENDAR_RULE","citation":citation,
  "time_rules":[installed.rule("CALENDAR_PAID_TIME")?,installed.rule("PAID_ABSENCE_TREATMENT")?]}))?;
 let collection=payroll::read_source_collection_selection(pool,submitter,"QUALIFICATION").await?;
 let calendar=produce_verified_qualification(pool,submitter,reviewer,&collection,calendar_payload.clone()).await?;
 let bindings=["TAXABLE_INCOME_DEFINITION","INSURANCE_APPLICABILITY","NO_ADJUSTMENT_COVERAGE","PENSION_BASIS_DEFINITION","INSURANCE_REMUNERATION_DEFINITION"]
  .iter().map(|purpose|installed.rule(purpose)).collect::<Result<Vec<_>,_>>()?;
 let collection=payroll::read_source_collection_selection(pool,submitter,"QUALIFICATION").await?;
 let applicability=produce_verified_qualification(pool,submitter,reviewer,&collection,
  admission(json!({"kind":"APPLICABILITY_RULE","citation":citation,"bindings":bindings}))?).await?;
 let table=b"lower_won,upper_won,dependents,children,option,income_tax_won,local_tax_won\n3000000,3010000,1,0,P100,74350,7430\n".to_vec();
 let tax_artifact=upload_source(pool,submitter,worker,channel,"text/csv",table).await?.source_artifact;
 // Layout/support are actual registered immutable rule metadata, not fabricated
 // locators. Assert exact CSV columns/record and supported selection below.
 assert_eq!(installed.tax_columns,vec!["lower_won","upper_won","dependents","children","option","income_tax_won","local_tax_won"]);
 let tax_citation=json!({"instrument":"Synthetic NTS row","edition":"NTS_FIXTURE_ROW_V1","source_url":null,
  "artifact_refs":[tax_artifact],"qualified_review_refs":[reviewed],"applicability":{"from":"2026-05-31","to_exclusive":"2026-07-01"}});
 let collection=payroll::read_source_collection_selection(pool,submitter,"QUALIFICATION").await?;
 let tax=produce_verified_qualification(pool,submitter,reviewer,&collection,admission(json!({
  "kind":"TAX_TABLE","citation":tax_citation,"layout":installed.tax_layout,
  "support":{"dependent_counts":["1"],"eligible_child_counts":["0"],"withholding_options":["P100"],
   "combination_rule":installed.rule("TAX_ROW_SELECTION")?,"note_map_artifact_ref":tax_artifact,"normalizer_ref":null},
  "selection_rule":installed.rule("TAX_ROW_SELECTION")?}))?).await?;
 let collection=payroll::read_source_collection_selection(pool,submitter,"QUALIFICATION").await?;
 let qualified=payroll::bind_qualified_rule_bundle(pool,submitter,payroll::BindQualifiedRuleBundle {
  command_id:Uuid::new_v4(),kernel_build:installed.build_ref.clone(),
  qualification_refs:vec![calendar.admission_ref.clone(),applicability.admission_ref.clone(),tax.admission_ref.clone()],
 }).await?;
 // citation_set_sha256 is computed over ACTUAL committed qualified source content
 // in this owner, never embedded with future Company refs in deployment metadata.
 let bundle=produce_verified_qualification(pool,submitter,reviewer,&collection,admission(json!({
  "kind":"RULE_BUNDLE","citation":citation,"bundle_ref":qualified.reference,
  "source_qualification_refs":[calendar.admission_ref,applicability.admission_ref,tax.admission_ref]}))?).await?;
 let profile=payroll::create_supported_profile(pool,submitter,payroll::CreateSupportedProfile {
  command_id:Uuid::new_v4(),rule_bundle:qualified.reference.clone(),qualification:bundle.admission_ref.clone(),
  scope_basis:workforce.scope.clone(),period:payroll::InclusivePeriod{start:time::macros::date!(2026-06-01),end:time::macros::date!(2026-06-30)},
  pay_date:time::macros::date!(2026-06-27),currency:"KRW".into(),monthly_standard_hours:209,
 }).await?;
 assert_eq!(profile.reference.rule_bundle,qualified.reference);
 Ok(SourceFoundations{credential,submitter_credential,calendar,applicability,tax,bundle,profile:profile.reference,evidence,tax_artifact,calendar_payload,installed,rule_bundle:qualified.reference})
}
