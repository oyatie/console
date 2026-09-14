//! First fresh primary activation of a preserved migrated Account/key. This is
//! an ordinary account-enrollment use case, not employer reset or recovery bypass.
use super::legacy_transport::{ORIGIN, Transport};
use console_platform_auth::account_session::{VerifiedAccountSession, verify_account_session};
use console_platform_auth::{account_enrollment as accounts, terms_publication as publisher};
use rand::RngCore;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use url::Url;
use webauthn_authenticator_rs::prelude::RequestChallengeResponse;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub async fn activate_and_login(
    pool: &PgPool,
    transport: &mut Transport,
) -> Result<VerifiedAccountSession> {
    // SAME concrete source-authored deployment runtime and independent Ed25519
    // custody producer used by B6/S2. No seeded Account IDs or success callbacks.
    let runtime =
        crate::deployment_runtime_producer::start_runtime(pool, "migration fresh primary").await?;
    let approval = runtime.custody.approve(0)?;
    let operator = runtime
        .custody
        .authenticate(publisher::Environment::TestOnly)
        .await?;
    let published = publisher::publish_terms(
        &operator,
        publisher::OperatorApprovalRef {
            kind: publisher::ApprovalKind::OperatorReleaseApproval,
            approval_sha256: approval.approval_digest,
        },
    )
    .await?;
    assert_eq!(published.previous_revision, None);
    assert_eq!(published.revision, 1);
    let terms = accounts::read_current_account_terms(&runtime.auth).await?;
    assert_eq!(terms.reference.receipt_id, published.id);
    // Actual wrong-key/nonce/terms/binding cases each use a NEW real challenge.
    // Failure may consume its ceremony but cannot activate, assent or mint tokens.
    async fn protected(pool: &PgPool) -> Result<serde_json::Value> {
        let security: Vec<serde_json::Value> =
            sqlx::query_scalar("SELECT to_jsonb(s) FROM account_security s ORDER BY account_id")
                .fetch_all(pool)
                .await?;
        let terms:Vec<serde_json::Value>=sqlx::query_scalar("SELECT to_jsonb(t) FROM account_terms_acceptances t ORDER BY account_id,terms_kind,terms_version").fetch_all(pool).await?;
        let families: Vec<serde_json::Value> =
            sqlx::query_scalar("SELECT to_jsonb(f) FROM auth_refresh_token_families f ORDER BY id")
                .fetch_all(pool)
                .await?;
        let tokens: Vec<serde_json::Value> =
            sqlx::query_scalar("SELECT to_jsonb(t) FROM auth_refresh_tokens t ORDER BY id")
                .fetch_all(pool)
                .await?;
        Ok(json!({"security":security,"terms":terms,"families":families,"tokens":tokens}))
    }
    for fault in ["nonce", "terms", "ceremony", "activation", "wrong_key"] {
        let mut nonce = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
        let before = protected(pool).await?;
        let started = accounts::start_account_primary_activation(
            &runtime.auth,
            &transport.service,
            accounts::PrimaryActivationStart {
                account_id: transport.key.account_id,
                terms: terms.reference.clone(),
                browser_nonce_digest: hex::encode(Sha256::digest(nonce)),
            },
        )
        .await?;
        let key = if fault == "wrong_key" {
            &mut transport.revoked_authenticator
        } else {
            &mut transport.key
        };
        let mut challenge = serde_json::to_value(&started.challenge)?;
        // This is attacker-selected browser input; signer owns the OTHER actual
        // pre-deleted key. Server must ignore this client allow-list substitution.
        challenge["publicKey"]["allowCredentials"] = json!([{"type":"public-key","id":serde_json::from_str::<serde_json::Value>(&key.credential_id)?}]);
        let credential = key.authenticator.do_authentication(
            Url::parse(ORIGIN)?,
            serde_json::from_value::<RequestChallengeResponse>(challenge)?,
        )?;
        let mut request = accounts::PrimaryActivationFinish {
            activation_id: started.activation_id,
            ceremony_id: started.ceremony_id,
            browser_nonce: hex::encode(nonce),
            terms: terms.reference.clone(),
            credential,
        };
        match fault {
            "nonce" => {
                nonce[0] ^= 1;
                request.browser_nonce = hex::encode(nonce);
            }
            "terms" => request.terms.receipt_id = uuid::Uuid::new_v4(),
            "ceremony" => request.ceremony_id = uuid::Uuid::new_v4(),
            "activation" => request.activation_id = uuid::Uuid::new_v4(),
            "wrong_key" => {}
            _ => unreachable!(),
        }
        let result =
            accounts::finish_account_primary_activation(&runtime.auth, &transport.service, request)
                .await;
        assert!(
            matches!(result, Err(accounts::PrimaryActivationError::InvalidProof)),
            "only uniform classified invalid proof counts; storage/fixture error does not"
        );
        assert_eq!(protected(pool).await?, before);
    }
    // The actual pre-deleted-key Account is in RECOVERY_REQUIRED. Its Account
    // identifier alone must not admit a retained-primary activation challenge.
    let before = protected(pool).await?;
    let denied = accounts::start_account_primary_activation(
        &runtime.auth,
        &transport.service,
        accounts::PrimaryActivationStart {
            account_id: transport.revoked_authenticator.account_id,
            terms: terms.reference.clone(),
            browser_nonce_digest: hex::encode(Sha256::digest([7u8; 32])),
        },
    )
    .await;
    assert!(matches!(
        denied,
        Err(accounts::PrimaryActivationError::InvalidProof)
    ));
    assert_eq!(protected(pool).await?, before);
    let mut nonce = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    // Account ID is an UNTRUSTED lookup key. The owner must bind migration's
    // exact preserved Account/key and eligibility under account_security lock;
    // only the fresh cryptographic assertion and current terms can activate it.
    let started = accounts::start_account_primary_activation(
        &runtime.auth,
        &transport.service,
        accounts::PrimaryActivationStart {
            account_id: transport.key.account_id,
            terms: terms.reference.clone(),
            browser_nonce_digest: hex::encode(Sha256::digest(nonce)),
        },
    )
    .await?;
    let mut challenge = serde_json::to_value(&started.challenge)?;
    let allow = challenge["publicKey"]["allowCredentials"]
        .as_array_mut()
        .ok_or("missing real primary challenge")?;
    if allow.is_empty() {
        allow.push(json!({"type":"public-key","id":serde_json::from_str::<serde_json::Value>(&transport.key.credential_id)?}));
    }
    let assertion = transport.key.authenticator.do_authentication(
        Url::parse(ORIGIN)?,
        serde_json::from_value::<RequestChallengeResponse>(challenge)?,
    )?;
    let activated = accounts::finish_account_primary_activation(
        &runtime.auth,
        &transport.service,
        accounts::PrimaryActivationFinish {
            activation_id: started.activation_id,
            ceremony_id: started.ceremony_id,
            browser_nonce: hex::encode(nonce),
            terms: terms.reference,
            credential: assertion,
        },
    )
    .await?;
    assert_eq!(activated.account_id, transport.key.account_id);
    let first = verify_account_session(
        &runtime.auth,
        &runtime.verifier,
        &activated.account_access_token,
    )
    .await?;
    assert_eq!(first.account_id(), transport.key.account_id);
    assert_eq!(
        first.assurance(),
        accounts::AccountAssurance::PasskeyPrimary
    );
    assert_eq!(first.family_id(), activated.session_family_id);
    // Then exercise the common Account13 primary path with a NEW real ceremony.
    let login = accounts::start_account_authentication(&runtime.auth, &transport.service).await?;
    let mut challenge = serde_json::to_value(&login.challenge)?;
    let allow = challenge["publicKey"]["allowCredentials"]
        .as_array_mut()
        .ok_or("missing actual Account login challenge")?;
    assert!(allow.is_empty());
    allow.push(json!({"type":"public-key","id":serde_json::from_str::<serde_json::Value>(&transport.key.credential_id)?}));
    let assertion = transport.key.authenticator.do_authentication(
        Url::parse(ORIGIN)?,
        serde_json::from_value::<RequestChallengeResponse>(challenge)?,
    )?;
    let logged_in = accounts::finish_account_authentication(
        &runtime.auth,
        &transport.service,
        login.ceremony_id,
        assertion,
    )
    .await?;
    let second = verify_account_session(
        &runtime.auth,
        &runtime.verifier,
        &logged_in.account_access_token,
    )
    .await?;
    assert_eq!(second.account_id(), transport.key.account_id);
    assert_eq!(second.family_id(), logged_in.session_family_id);
    assert_eq!(
        second.assurance(),
        accounts::AccountAssurance::PasskeyPrimary
    );
    assert_ne!(second.family_id(), first.family_id());
    Ok(second)
}
