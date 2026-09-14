//! Actual ordinary deployment publisher precedes Account root/bootstrap.
//! operator_custody_producer is the independently authored real Ed25519 custody
//! producer from the security acceptance packet, copied byte-for-byte at integration.
use console_platform_auth::{terms_publication as publisher,PasskeyService,JwtVerifier};
use sqlx::PgPool;
use url::Url;
use crate::{operator_custody_producer::CustodyFixture,deployment_root_producer::enroll_deployment_operator,native_fixture::TestResult};
pub async fn publish_genesis_then_enroll_operator(
    custody:&CustodyFixture,auth_pool:&PgPool,service:&PasskeyService,
    verifier:&JwtVerifier,origin:&Url,configured_root_secret:&str,
)->TestResult<console_platform_auth::account_session::VerifiedAccountSession> {
    // Genuine signed operator approval for expected genesis0→head1 is resolved
    // by the real restricted ordinary publisher, using its dedicated DB role.
    let approved=custody.approve(0)?;
    let operator=custody.authenticate(publisher::Environment::TestOnly).await?;
    let published=publisher::publish_terms(&operator,publisher::OperatorApprovalRef {
        kind:publisher::ApprovalKind::OperatorReleaseApproval,
        approval_sha256:approved.approval_digest,
    }).await?;
    assert_eq!(published.previous_revision,None);assert_eq!(published.revision,1);
    assert_eq!(published.manifest_sha256,approved.manifest_digest);
    // Only after committed actual publication can Account terms read, root proof
    // redemption, real WebAuthn and ACCOUNT_V1 verification start.
    enroll_deployment_operator(auth_pool,service,verifier,origin,configured_root_secret,&published).await
}
