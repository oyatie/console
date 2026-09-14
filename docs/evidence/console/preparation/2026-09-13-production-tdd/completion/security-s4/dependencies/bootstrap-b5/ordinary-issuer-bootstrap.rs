//! Ordinary provider registration, supervised identity-root admission, Company
//! enrollment and current policy grants. TEST_ONLY applies to deployment config,
//! issuer keys and synthetic fixture claims; no alternate test business owner.
use console_platform_provisioning::PlatformProvisioner;
use console_platform_provisioning::account13 as provisioning;
use console_identity_adapter_postgres::account13 as identity;
use console_platform_auth::account_session::VerifiedAccountSession;
use jsonwebtoken::{Algorithm,EncodingKey,Header};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;
use crate::{account_enrollment_producer::AccountBootstrapPhase,native_fixture::TestResult};

pub struct FixtureIssuer { issuer:String,key_id:String,public_pem:Vec<u8>,key:EncodingKey }
impl FixtureIssuer {
    pub fn from_config(issuer:String,key_id:String,public_pem:Vec<u8>,private_pem:&[u8])->TestResult<Self> {
        Ok(Self{issuer,key_id,public_pem,key:EncodingKey::from_ec_pem(private_pem)?})
    }
    pub fn sign<T:Serialize>(&self,claims:&T)->TestResult<String> {
        let mut header=Header::new(Algorithm::ES256);header.kid=Some(self.key_id.clone());
        Ok(jsonwebtoken::encode(&header,claims,&self.key)?)
    }
}
#[derive(Serialize)]
pub struct SubjectBinding { account_id:Uuid,provider_subject:String }
#[derive(Serialize)]
pub struct IdentityRootClaims {
    iss:String,aud:String,iat:i64,nbf:i64,exp:i64,jti:Uuid,
    deployment_id:Uuid,group_id:Uuid,provider:identity::IdentityProviderRevision,evidence_classification:&'static str,
    bindings:Vec<SubjectBinding>,
}
pub struct BootstrapOutputs {
    pub group:provisioning::GroupEnrollmentReceipt,
    pub provider:identity::IdentityProviderRevision,
    pub identity_root:identity::SupervisedIdentityRootReceipt,
    pub companies:Vec<provisioning::CompanyEnrollmentReceipt>,
}

pub async fn enroll_authority_and_companies(
    pool:&PgPool,operator:&VerifiedAccountSession,accounts:&AccountBootstrapPhase,
    issuer:&FixtureIssuer,scenario_names:&[String],choices:&[identity::CompanyGrantChoices],
)->TestResult<BootstrapOutputs> {
    let provisioner=PlatformProvisioner::new(time::Duration::minutes(5));
    // Account-aware siblings in SAME normal provisioner as create_group and
    // onboard_tenant; no legacy admin_user_id/OTP returned as Account session.
    let group=provisioner.create_account_group(pool,operator,
        provisioning::GroupEnrollmentInput{command_id:Uuid::new_v4(),slug:format!("acceptance-{}",Uuid::new_v4()),name:"Synthetic acceptance Group".into()}).await?;
    let provider=identity::register_identity_provider(pool,operator,
        identity::IdentityProviderRegistration {
            command_id:Uuid::new_v4(),group_id:group.group_id,issuer:issuer.issuer.clone(),key_id:issuer.key_id.clone(),
            public_es256_pem:issuer.public_pem.clone(),accepted_audience:"console.identity-root.v1".into(),
            evidence_classification:identity::EvidenceClassification::Synthetic,
        }).await?;
    let now=time::OffsetDateTime::now_utc().unix_timestamp();
    let claims=IdentityRootClaims{iss:issuer.issuer.clone(),aud:"console.identity-root.v1".into(),iat:now,nbf:now,exp:now+300,jti:Uuid::new_v4(),deployment_id:group.deployment_id,group_id:group.group_id,provider:provider.clone(),evidence_classification:"SYNTHETIC",bindings:vec![
        SubjectBinding{account_id:accounts.submitter.account_id,provider_subject:"SYNTHETIC_SUBJECT_A".into()},
        SubjectBinding{account_id:accounts.same_human_alias.account_id,provider_subject:"SYNTHETIC_SUBJECT_A".into()},
        SubjectBinding{account_id:accounts.reviewer.account_id,provider_subject:"SYNTHETIC_SUBJECT_B".into()},
    ]};
    // Ordinary initial supervised root path, including exact currently configured
    // provider verification and operator authority. Test signature is real ES256.
    let identity_root=identity::admit_supervised_identity_root(pool,operator,
        identity::SupervisedIdentityRootAdmission{command_id:Uuid::new_v4(),group_id:group.group_id,provider:provider.clone(),signed_evidence:issuer.sign(&claims)?}).await?;
    assert_eq!(identity_root.group_id,group.group_id);
    let mut companies=Vec::new();
    for scenario in scenario_names {
        let command_id=Uuid::new_v4();
        let company=provisioner.onboard_account_company(pool,operator,
            provisioning::CompanyEnrollmentInput{command_id,group_id:group.group_id,slug:format!("{}-{}",scenario,Uuid::new_v4()),name:scenario.clone(),administrative_account_id:operator.account_id()}).await?;
        assert_eq!(company.original_command_id,command_id);
        assert!(companies.iter().all(|c:&provisioning::CompanyEnrollmentReceipt|c.org_id!=company.org_id));
        // Ordinary current grant owner, independently authorized by operator.
        // Grant plan is untrusted fixture DATA, not accepted authority, and is
        // bound to actual new Company here. Owner checks issuer scope, actual
        // registered actions/fields/validity and current expected policy head.
        for choice in choices {
            // Resolve Company-scoped action/field/scope refs ONLY AFTER this
            // actual Company exists. Choices contain keys/intent, no future refs.
            let plan=identity::read_registered_company_grant_plan(pool,operator,company.org_id,choice).await?;
            let expected=identity::read_company_policy_control(pool,operator,company.org_id).await?;
            let granted=identity::apply_company_grant_plan(pool,operator,
                identity::ApplyCompanyGrantPlan{command_id:Uuid::new_v4(),org_id:company.org_id,expected,plan}).await?;
            assert_eq!(granted.org_id,company.org_id);
        }
        companies.push(company);
    }
    Ok(BootstrapOutputs{group,provider,identity_root,companies})
}
