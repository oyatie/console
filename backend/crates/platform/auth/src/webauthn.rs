use console_kernel_core::{AuditAction, AuditEvent, OrgId, TraceContext, UserId};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use time::{Duration, OffsetDateTime};
use url::Url;
use uuid::Uuid;
use webauthn_rs::prelude::{
    CreationChallengeResponse, DiscoverableAuthentication, DiscoverableKey, Passkey,
    PasskeyRegistration, PublicKeyCredential, RegisterPublicKeyCredential,
    RequestChallengeResponse, Webauthn, WebauthnBuilder,
};

use crate::{AuthError, append_legacy_auth_audit_in_tx, guard_legacy_subject_in_tx};

pub type PasskeyRegistrationCredential = RegisterPublicKeyCredential;
pub type PasskeyAuthenticationCredential = PublicKeyCredential;

#[derive(Debug, Clone)]
pub struct WebauthnSettings {
    pub rp_id: String,
    pub rp_origin: Url,
    pub rp_name: String,
    pub extra_allowed_origins: Vec<Url>,
    pub ceremony_ttl: Duration,
}

#[derive(Clone)]
pub struct PasskeyService {
    webauthn: Webauthn,
    ceremony_ttl: Duration,
    native_origin: String,
}

#[derive(Debug, Clone)]
pub struct PasskeyRegistrationStart {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
}

#[derive(Debug)]
pub struct RegistrationCeremony {
    pub ceremony_id: Uuid,
    pub challenge: CreationChallengeResponse,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug)]
pub struct AuthenticationCeremony {
    pub ceremony_id: Uuid,
    pub challenge: RequestChallengeResponse,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MobileStepUpActionKind {
    ApprovalDecision,
    PollVote,
}

impl MobileStepUpActionKind {
    pub const fn as_wire(self) -> &'static str {
        match self {
            Self::ApprovalDecision => "APPROVAL_DECISION",
            Self::PollVote => "POLL_VOTE",
        }
    }

    pub const fn expected_reason_key(self) -> &'static str {
        match self {
            Self::ApprovalDecision => "operations_passkey_approval_decision",
            Self::PollVote => "operations_passkey_poll_vote",
        }
    }

    fn from_wire(raw: &str) -> Result<Self, AuthError> {
        match raw {
            "APPROVAL_DECISION" => Ok(Self::ApprovalDecision),
            "POLL_VOTE" => Ok(Self::PollVote),
            _ => Err(AuthError::InvalidStoredData(format!(
                "unknown mobile step-up action kind: {raw}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobilePasskeyStepUpBinding {
    pub action_kind: MobileStepUpActionKind,
    pub object_id: Uuid,
    pub reason_key: String,
    pub replay_attempt: Option<i32>,
}

impl<'de> Deserialize<'de> for MobilePasskeyStepUpBinding {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BindingVisitor;

        enum Field {
            ActionKind,
            ObjectId,
            ReasonKey,
            ReplayAttempt,
            Ignore,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct FieldVisitor;

                impl serde::de::Visitor<'_> for FieldVisitor {
                    type Value = Field;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        formatter.write_str("a mobile passkey step-up binding field")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok(match value {
                            "action_kind" => Field::ActionKind,
                            "object_id" => Field::ObjectId,
                            "reason_key" => Field::ReasonKey,
                            "replay_attempt" => Field::ReplayAttempt,
                            _ => Field::Ignore,
                        })
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        impl<'de> serde::de::Visitor<'de> for BindingVisitor {
            type Value = MobilePasskeyStepUpBinding;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a mobile passkey step-up binding")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut action_kind = None;
                let mut object_id = None;
                let mut reason_key = None;
                let mut replay_attempt = None;

                while let Some(key) = map.next_key::<Field>()? {
                    match key {
                        Field::ActionKind => {
                            if action_kind.is_some() {
                                return Err(serde::de::Error::duplicate_field("action_kind"));
                            }
                            action_kind = Some(map.next_value()?);
                        }
                        Field::ObjectId => {
                            if object_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("object_id"));
                            }
                            object_id = Some(map.next_value()?);
                        }
                        Field::ReasonKey => {
                            if reason_key.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason_key"));
                            }
                            reason_key = Some(map.next_value()?);
                        }
                        Field::ReplayAttempt => {
                            if replay_attempt.is_some() {
                                return Err(serde::de::Error::duplicate_field("replay_attempt"));
                            }
                            replay_attempt = Some(map.next_value::<Option<i32>>()?);
                        }
                        Field::Ignore => {
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                Ok(MobilePasskeyStepUpBinding {
                    action_kind: action_kind
                        .ok_or_else(|| serde::de::Error::missing_field("action_kind"))?,
                    object_id: object_id
                        .ok_or_else(|| serde::de::Error::missing_field("object_id"))?,
                    reason_key: reason_key
                        .ok_or_else(|| serde::de::Error::missing_field("reason_key"))?,
                    replay_attempt: replay_attempt
                        .ok_or_else(|| serde::de::Error::missing_field("replay_attempt"))?,
                })
            }
        }

        const FIELDS: &[&str] = &["action_kind", "object_id", "reason_key", "replay_attempt"];
        deserializer.deserialize_struct("MobilePasskeyStepUpBinding", FIELDS, BindingVisitor)
    }
}

impl MobilePasskeyStepUpBinding {
    pub fn approval_decision(object_id: Uuid, replay_attempt: Option<i32>) -> Self {
        Self::new(
            MobileStepUpActionKind::ApprovalDecision,
            object_id,
            replay_attempt,
        )
    }

    pub fn poll_vote(object_id: Uuid, replay_attempt: Option<i32>) -> Self {
        Self::new(MobileStepUpActionKind::PollVote, object_id, replay_attempt)
    }

    fn new(
        action_kind: MobileStepUpActionKind,
        object_id: Uuid,
        replay_attempt: Option<i32>,
    ) -> Self {
        Self {
            action_kind,
            object_id,
            reason_key: action_kind.expected_reason_key().to_owned(),
            replay_attempt,
        }
    }

    pub fn validate(&self) -> Result<(), MobileStepUpBindingError> {
        if self.reason_key != self.action_kind.expected_reason_key() {
            return Err(MobileStepUpBindingError::ReasonKeyMismatch);
        }
        if self.replay_attempt.is_some_and(|attempt| attempt < 1) {
            return Err(MobileStepUpBindingError::InvalidReplayAttempt);
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MobileStepUpBindingError {
    #[error("reason_key is not supported for action_kind")]
    ReasonKeyMismatch,

    #[error("replay_attempt must be null or a positive 1-based integer")]
    InvalidReplayAttempt,
}

#[derive(Debug, Deserialize)]
pub struct MobilePasskeyStepUpAssertion {
    pub ceremony_id: Uuid,
    pub credential: PasskeyAuthenticationCredential,
}

#[derive(Debug, Deserialize)]
pub struct MobilePasskeyStepUpEnvelope {
    pub binding: MobilePasskeyStepUpBinding,
    pub assertion: MobilePasskeyStepUpAssertion,
}

#[derive(Debug, thiserror::Error)]
pub enum MobilePasskeyStepUpVerificationError {
    #[error("mobile passkey step-up binding mismatch")]
    BindingMismatch,

    #[error(transparent)]
    Auth(#[from] AuthError),
}

impl From<sqlx::Error> for MobilePasskeyStepUpVerificationError {
    fn from(value: sqlx::Error) -> Self {
        Self::Auth(value.into())
    }
}

impl From<serde_json::Error> for MobilePasskeyStepUpVerificationError {
    fn from(value: serde_json::Error) -> Self {
        Self::Auth(value.into())
    }
}

impl From<webauthn_rs::prelude::WebauthnError> for MobilePasskeyStepUpVerificationError {
    fn from(value: webauthn_rs::prelude::WebauthnError) -> Self {
        Self::Auth(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredPasskey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub credential_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticationOutcome {
    pub user_id: Uuid,
    pub passkey_id: Uuid,
    /// The tenant the asserting credential belongs to, resolved from the
    /// credential id BEFORE the RLS-gated read. The login handler uses it to arm
    /// the GUC for the subsequent `users` read + session mint, since the passkey
    /// login route runs before the tenant middleware.
    pub org_id: OrgId,
}

impl PasskeyService {
    pub fn new(settings: WebauthnSettings) -> Result<Self, AuthError> {
        let mut builder =
            WebauthnBuilder::new(&settings.rp_id, &settings.rp_origin)?.rp_name(&settings.rp_name);
        for origin in &settings.extra_allowed_origins {
            builder = builder.append_allowed_origin(origin);
        }
        Ok(Self {
            webauthn: builder.build()?,
            ceremony_ttl: settings.ceremony_ttl,
            native_origin: settings.rp_origin.origin().ascii_serialization(),
        })
    }

    pub async fn start_registration(
        &self,
        pool: &PgPool,
        org: OrgId,
        input: PasskeyRegistrationStart,
    ) -> Result<RegistrationCeremony, AuthError> {
        let mut tx = pool.begin().await?;
        let ceremony = self.start_registration_in_tx(&mut tx, org, input).await?;
        tx.commit().await?;
        Ok(ceremony)
    }

    pub async fn start_registration_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        org: OrgId,
        input: PasskeyRegistrationStart,
    ) -> Result<RegistrationCeremony, AuthError> {
        guard_legacy_subject_in_tx(tx, org, input.user_id).await?;
        let existing = load_user_passkeys_in_tx(tx, org, input.user_id).await?;
        let exclude_credentials = existing
            .into_iter()
            .map(|passkey| passkey.cred_id().clone())
            .collect::<Vec<_>>();
        let exclude_credentials = if exclude_credentials.is_empty() {
            None
        } else {
            Some(exclude_credentials)
        };

        let (challenge, state) = self.webauthn.start_passkey_registration(
            input.user_id,
            &input.username,
            &input.display_name,
            exclude_credentials,
        )?;
        let ceremony_id = Uuid::new_v4();
        let now = OffsetDateTime::now_utc();
        let expires_at = now + self.ceremony_ttl;

        persist_ceremony(
            tx,
            ceremony_id,
            Some(input.user_id),
            "registration",
            &challenge,
            &state,
            expires_at,
        )
        .await?;

        Ok(RegistrationCeremony {
            ceremony_id,
            challenge,
            expires_at,
        })
    }

    /// Count the authenticated user's existing passkeys (RLS-armed).
    ///
    /// Used by the add-device flow to decide whether a fresh step-up assertion is
    /// REQUIRED: a user with zero passkeys is doing initial enrollment (no
    /// existing credential to assert), while a user with one or more must prove
    /// possession of an existing passkey before a new one is issued.
    pub async fn count_user_passkeys(
        &self,
        pool: &PgPool,
        org: OrgId,
        user_id: Uuid,
    ) -> Result<usize, AuthError> {
        let mut tx = pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_org', $1, true)")
            .bind(org.as_uuid().to_string())
            .execute(tx.as_mut())
            .await?;
        let count: i64 =
            sqlx::query_scalar("SELECT public.auth_legacy_self_passkey_count_v1($1, $2)")
                .bind(*org.as_uuid())
                .bind(user_id)
                .fetch_one(tx.as_mut())
                .await?;
        tx.commit().await?;
        usize::try_from(count)
            .map_err(|_| AuthError::InvalidStoredData("invalid passkey count".to_owned()))
    }

    /// Verify a FRESH step-up assertion of one of `expected_user_id`'s OWN
    /// existing passkeys, with user verification (UV) required.
    ///
    /// This is the anti-silent-add gate for self-service device enrollment: before
    /// a new credential is issued to an already-enrolled user, the caller must
    /// assert an existing passkey of THE SAME user with UV=true, so a stolen
    /// session (bearer token only, no authenticator) cannot add a credential.
    ///
    /// The assertion ceremony is claimed atomically (single-use, like login), the
    /// discoverable assertion is verified against the resolved credential, and the
    /// assertion is rejected unless (a) `user_verified()` is true and (b) the
    /// asserting credential belongs to `expected_user_id`. The credential's org is
    /// resolved + the GUC armed exactly as in `finish_authentication`, but NO
    /// token is minted and NO session is created — this only proves possession.
    pub async fn verify_step_up_for_user(
        &self,
        pool: &PgPool,
        ceremony_id: Uuid,
        credential: PublicKeyCredential,
        expected_user_id: Uuid,
    ) -> Result<(), AuthError> {
        let mut tx = pool.begin().await?;
        self.verify_step_up_for_user_in_tx(&mut tx, ceremony_id, credential, expected_user_id)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn verify_step_up_for_user_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        ceremony_id: Uuid,
        credential: PublicKeyCredential,
        expected_user_id: Uuid,
    ) -> Result<(), AuthError> {
        self.verify_assertion_in_tx(tx, ceremony_id, credential, Some(expected_user_id), None)
            .await?;
        Ok(())
    }

    pub async fn start_mobile_step_up(
        &self,
        pool: &PgPool,
        org: OrgId,
        user_id: Uuid,
        binding: MobilePasskeyStepUpBinding,
    ) -> Result<AuthenticationCeremony, AuthError> {
        binding
            .validate()
            .map_err(|err| AuthError::InvalidStoredData(err.to_string()))?;
        let (challenge, state) = self.webauthn.start_discoverable_authentication()?;
        let ceremony_id = Uuid::new_v4();
        let now = OffsetDateTime::now_utc();
        let expires_at = now + self.ceremony_ttl;

        let mut tx = pool.begin().await?;
        guard_legacy_subject_in_tx(&mut tx, org, user_id).await?;
        sqlx::query(
            r#"
            INSERT INTO auth_webauthn_ceremonies (
                id, user_id, ceremony_kind, challenge_json, state_json, expires_at
            ) VALUES ($1, $2, 'authentication', $3, $4, $5)
            "#,
        )
        .bind(ceremony_id)
        .bind(user_id)
        .bind(serde_json::to_value(&challenge)?)
        .bind(serde_json::to_value(&state)?)
        .bind(expires_at)
        // rls-arming: ok auth_webauthn_ceremonies is a global auth table (no org_id, no RLS)
        .execute(tx.as_mut())
        .await?;
        insert_mobile_step_up_binding_tx(&mut tx, ceremony_id, &binding).await?;
        tx.commit().await?;

        Ok(AuthenticationCeremony {
            ceremony_id,
            challenge,
            expires_at,
        })
    }

    pub async fn verify_mobile_step_up_for_user(
        &self,
        pool: &PgPool,
        envelope: MobilePasskeyStepUpEnvelope,
        expected_user_id: Uuid,
        expected_binding: &MobilePasskeyStepUpBinding,
    ) -> Result<(), MobilePasskeyStepUpVerificationError> {
        if envelope.binding != *expected_binding {
            return Err(MobilePasskeyStepUpVerificationError::BindingMismatch);
        }

        let mut tx = pool.begin().await?;
        self.verify_mobile_step_up_for_user_in_tx(
            &mut tx,
            envelope,
            expected_user_id,
            expected_binding,
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn verify_mobile_step_up_for_user_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        envelope: MobilePasskeyStepUpEnvelope,
        expected_user_id: Uuid,
        expected_binding: &MobilePasskeyStepUpBinding,
    ) -> Result<(), MobilePasskeyStepUpVerificationError> {
        if envelope.binding != *expected_binding {
            return Err(MobilePasskeyStepUpVerificationError::BindingMismatch);
        }
        self.verify_assertion_in_tx(
            tx,
            envelope.assertion.ceremony_id,
            envelope.assertion.credential,
            Some(expected_user_id),
            Some(expected_binding),
        )
        .await
        .map_err(|err| match err {
            AuthError::InvalidStoredData(ref message)
                if message == "mobile step-up binding mismatch" =>
            {
                MobilePasskeyStepUpVerificationError::BindingMismatch
            }
            other => other.into(),
        })?;
        Ok(())
    }

    pub async fn finish_registration(
        &self,
        pool: &PgPool,
        org: OrgId,
        ceremony_id: Uuid,
        credential: RegisterPublicKeyCredential,
    ) -> Result<StoredPasskey, AuthError> {
        let now = OffsetDateTime::now_utc();
        let mut tx = pool.begin().await?;
        let stored = self
            .finish_registration_in_tx(&mut tx, org, ceremony_id, credential, now)
            .await?;
        tx.commit().await?;
        Ok(stored)
    }

    /// Finish a passkey registration inside a caller-provided transaction.
    ///
    /// Performs the atomic ceremony claim, verifies the credential against the
    /// claimed state, inserts the passkey, and appends the audit row — all in the
    /// caller's transaction. The bootstrap cold-start path uses this so the
    /// passkey insert and the single-use bootstrap-credential consume commit (or
    /// roll back) atomically together. The caller owns the `commit`.
    pub async fn finish_registration_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        org: OrgId,
        ceremony_id: Uuid,
        credential: RegisterPublicKeyCredential,
        now: OffsetDateTime,
    ) -> Result<StoredPasskey, AuthError> {
        let user_id: Uuid = sqlx::query_scalar(
            "SELECT user_id FROM public.auth_webauthn_ceremonies WHERE id = $1 AND ceremony_kind = 'registration'",
        ).bind(ceremony_id).fetch_optional(tx.as_mut()).await?
            .ok_or_else(|| AuthError::InvalidStoredData("registration ceremony is unavailable".to_owned()))?;
        guard_legacy_subject_in_tx(tx, org, user_id).await?;
        let claim = claim_ceremony_tx(tx, ceremony_id, "registration", now)
            .await?
            .ok_or_else(|| {
                AuthError::InvalidStoredData("ceremony not found or already consumed".to_owned())
            })?;
        if claim.user_id != Some(user_id) {
            return Err(AuthError::InvalidStoredData(
                "registration ceremony owner changed".to_owned(),
            ));
        }

        // Verify the assertion AFTER the atomic claim using the RETURNING state.
        // On verification failure we return Err, so the transaction rolls back and
        // the claim is undone — a legitimate retry stays possible.
        let state: PasskeyRegistration = serde_json::from_value(claim.state_json)?;
        let passkey = self
            .webauthn
            .finish_passkey_registration(&credential, &state)?;
        let passkey_json = serde_json::to_value(&passkey)?;
        let credential_id = serialize_to_string(passkey.cred_id(), "passkey credential id")?;
        let passkey_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO auth_webauthn_credentials (
                id, user_id, credential_id, passkey_json, created_at, org_id
            ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(passkey_id)
        .bind(user_id)
        .bind(&credential_id)
        .bind(passkey_json)
        .bind(now)
        .bind(*org.as_uuid())
        .execute(tx.as_mut())
        .await?;

        let audit = AuditEvent::new(
            Some(UserId::from_uuid(user_id)),
            AuditAction::new("auth.passkey.register")?,
            "auth_webauthn_credential",
            passkey_id.to_string(),
            TraceContext::generate(),
            now,
        )
        .with_org(org)
        .with_snapshots(
            None,
            Some(serde_json::json!({
                "credential_id": credential_id,
                "user_id": user_id,
            })),
        );
        append_legacy_auth_audit_in_tx(tx, &audit).await?;

        Ok(StoredPasskey {
            id: passkey_id,
            user_id,
            credential_id,
        })
    }

    /// Start a usernameless (discoverable) authentication ceremony.
    ///
    /// The challenge carries an EMPTY `allowCredentials` list: the client
    /// discovers the resident credential to use without the server naming a user.
    /// The persisted ceremony has a NULL `user_id` because the asserting user is
    /// only known once the client returns the credential at finish time.
    pub async fn start_authentication(
        &self,
        pool: &PgPool,
    ) -> Result<AuthenticationCeremony, AuthError> {
        let (challenge, state) = self.webauthn.start_discoverable_authentication()?;
        let ceremony_id = Uuid::new_v4();
        let now = OffsetDateTime::now_utc();
        let expires_at = now + self.ceremony_ttl;

        let mut tx = pool.begin().await?;
        persist_ceremony(
            &mut tx,
            ceremony_id,
            None,
            "authentication",
            &challenge,
            &state,
            expires_at,
        )
        .await?;
        tx.commit().await?;

        Ok(AuthenticationCeremony {
            ceremony_id,
            challenge,
            expires_at,
        })
    }

    /// Finish a usernameless (discoverable) authentication ceremony.
    ///
    /// The user is resolved FROM the asserted credential — by credential id,
    /// which is unique per credential and always present in the assertion — so no
    /// `user_id` is required from the client. When the authenticator returns a
    /// user handle (a true resident key), it is cross-checked against the
    /// resolved credential's owner. The atomic single-use ceremony claim from the
    /// harden-1 fix is preserved verbatim, so a replayed ceremony is rejected.
    pub async fn finish_authentication(
        &self,
        pool: &PgPool,
        ceremony_id: Uuid,
        credential: PublicKeyCredential,
    ) -> Result<AuthenticationOutcome, AuthError> {
        let mut tx = pool.begin().await?;
        let outcome = self
            .finish_authentication_in_tx(&mut tx, ceremony_id, credential)
            .await?;
        tx.commit().await?;
        Ok(outcome)
    }

    pub async fn finish_authentication_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        ceremony_id: Uuid,
        credential: PublicKeyCredential,
    ) -> Result<AuthenticationOutcome, AuthError> {
        self.verify_assertion_in_tx(tx, ceremony_id, credential, None, None)
            .await
    }

    async fn verify_assertion_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        ceremony_id: Uuid,
        credential: PublicKeyCredential,
        expected_user: Option<Uuid>,
        mobile_binding: Option<&MobilePasskeyStepUpBinding>,
    ) -> Result<AuthenticationOutcome, AuthError> {
        let now = OffsetDateTime::now_utc();
        // Resolve the asserting user FROM the credential. The credential id is the
        // stable lookup key (unique in `auth_webauthn_credentials`); it is always
        // present in the assertion even when the authenticator omits the user
        // handle. `raw_id` is the same `Base64UrlSafeData` type stored at
        // registration (`passkey.cred_id()`), so it serializes to the identical
        // base64url string the credential row is keyed by. If a user handle IS
        // present we additionally require it to match the credential's owner.
        let credential_id =
            serialize_to_string(&credential.raw_id, "authentication credential id")?;

        // Resolve the credential's tenant from its credential id FIRST, then arm
        // the GUC, THEN do the RLS-gated read/update. `auth_webauthn_credentials`
        // is FORCE RLS (migration 0035), so as the non-owner `console_rt` role a
        // lookup-by-credential-id returns ZERO rows until `app.current_org` is set
        // — but the org is what we need to set it. The narrow SECURITY DEFINER
        // resolver `platform_resolve_credential_org` (migration 0038) returns only
        // the credential's org_id, breaking that chicken-and-egg so passkey login
        // works for ANY tenant. A NULL means the credential is unknown: keep the
        // existing "not registered" error.
        let Some(org_uuid) = resolve_credential_org(tx, &credential_id).await? else {
            return Err(AuthError::InvalidStoredData(
                "asserted credential is not registered".to_owned(),
            ));
        };
        sqlx::query("SELECT set_config('app.current_org', $1, true)")
            .bind(org_uuid.to_string())
            .execute(tx.as_mut())
            .await?;

        // This first read is correlation only. No credential is claimed until
        // Company/users/Account have been locked and the key is reread.
        let correlated_user: Uuid = sqlx::query_scalar(
            "SELECT user_id FROM public.auth_webauthn_credentials WHERE credential_id = $1 AND org_id = $2",
        ).bind(&credential_id).bind(org_uuid).fetch_optional(tx.as_mut()).await?
            .ok_or_else(|| AuthError::InvalidStoredData("asserted credential is not registered".to_owned()))?;
        if expected_user.is_some_and(|expected| expected != correlated_user) {
            return Err(AuthError::InvalidStoredData(
                "step-up credential does not belong to the authenticated user".to_owned(),
            ));
        }
        guard_legacy_subject_in_tx(tx, OrgId::from_uuid(org_uuid), correlated_user).await?;
        let row = sqlx::query(
            r#"
            SELECT id, user_id, passkey_json
            FROM auth_webauthn_credentials
            WHERE credential_id = $1 AND user_id = $2 AND org_id = $3
            FOR UPDATE
            "#,
        )
        .bind(&credential_id)
        .bind(correlated_user)
        .bind(org_uuid)
        .fetch_optional(tx.as_mut())
        .await?
        .ok_or_else(|| {
            AuthError::InvalidStoredData("asserted credential is not registered".to_owned())
        })?;
        let passkey_id: Uuid = row.try_get("id")?;
        let user_id: Uuid = row.try_get("user_id")?;
        let passkey_json: serde_json::Value = row.try_get("passkey_json")?;

        if let Some(asserted_handle) = credential.get_user_unique_id()
            && Uuid::from_slice(asserted_handle).ok() != Some(user_id)
        {
            return Err(AuthError::InvalidStoredData(
                "asserted user handle does not match the credential owner".to_owned(),
            ));
        }

        let claim = claim_ceremony_tx(tx, ceremony_id, "authentication", now)
            .await?
            .ok_or_else(|| {
                AuthError::InvalidStoredData("ceremony not found or already consumed".to_owned())
            })?;
        if claim.user_id.is_some_and(|owner| owner != user_id) {
            return Err(AuthError::InvalidStoredData(
                "authentication ceremony owner mismatch".to_owned(),
            ));
        }
        let persisted_binding = load_mobile_step_up_binding_tx(tx, ceremony_id).await?;
        match (mobile_binding, persisted_binding.as_ref()) {
            (Some(expected), Some(persisted))
                if expected == persisted && claim.user_id == Some(user_id) => {}
            (None, None) => {}
            _ => {
                return Err(AuthError::InvalidStoredData(
                    "mobile step-up binding mismatch".to_owned(),
                ));
            }
        }

        // Verify the assertion AFTER the atomic claim using the RETURNING state
        // and the resolved credential as the single allowed discoverable key. A
        // verification failure returns Err and rolls back the claim.
        let state: DiscoverableAuthentication = serde_json::from_value(claim.state_json)?;
        let mut passkey: Passkey = serde_json::from_value(passkey_json)?;
        let discoverable_key = DiscoverableKey::from(&passkey);
        let result = self.webauthn.finish_discoverable_authentication(
            &credential,
            state,
            &[discoverable_key],
        )?;
        if expected_user.is_some() && !result.user_verified() {
            return Err(AuthError::InvalidStoredData(
                "step-up assertion did not perform user verification".to_owned(),
            ));
        }
        let changed = passkey.update_credential(&result).unwrap_or(false);

        if changed {
            sqlx::query(
                r#"
                UPDATE auth_webauthn_credentials
                SET passkey_json = $1, last_used_at = $2
                WHERE id = $3
                "#,
            )
            .bind(serde_json::to_value(&passkey)?)
            .bind(now)
            .bind(passkey_id)
            .execute(tx.as_mut())
            .await?;
        } else {
            sqlx::query("UPDATE auth_webauthn_credentials SET last_used_at = $1 WHERE id = $2")
                .bind(now)
                .bind(passkey_id)
                .execute(tx.as_mut())
                .await?;
        }

        Ok(AuthenticationOutcome {
            user_id,
            passkey_id,
            org_id: OrgId::from_uuid(org_uuid),
        })
    }
}

struct CeremonyRow {
    user_id: Option<Uuid>,
    state_json: serde_json::Value,
}

async fn persist_ceremony<C, S>(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    id: Uuid,
    user_id: Option<Uuid>,
    kind: &str,
    challenge: &C,
    state: &S,
    expires_at: OffsetDateTime,
) -> Result<(), AuthError>
where
    C: serde::Serialize,
    S: serde::Serialize,
{
    sqlx::query(
        r#"
        INSERT INTO auth_webauthn_ceremonies (
            id, user_id, ceremony_kind, challenge_json, state_json, expires_at
        ) VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(kind)
    .bind(serde_json::to_value(challenge)?)
    .bind(serde_json::to_value(state)?)
    .bind(expires_at)
    // rls-arming: ok auth_webauthn_ceremonies is a global pre-auth table (no org_id, no RLS)
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

/// Atomically claim a ceremony inside the consuming transaction.
///
/// The `UPDATE ... WHERE consumed_at IS NULL AND expires_at > now() RETURNING`
/// both checks the single-use/expiry invariant and marks the ceremony consumed
/// in one statement. Concurrent finish requests race on this row: exactly one
/// matches and consumes it; the loser matches 0 rows and gets `Ok(None)`, which
/// callers translate into a rejection. Because the claim lives in the caller's
/// transaction, returning `Err` later (e.g. on assertion-verification failure)
/// rolls the claim back, so a committed success is the only permanent consume.
async fn claim_ceremony_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    id: Uuid,
    kind: &str,
    now: OffsetDateTime,
) -> Result<Option<CeremonyRow>, AuthError> {
    let row = sqlx::query(
        r#"
        UPDATE auth_webauthn_ceremonies
        SET consumed_at = $3
        WHERE id = $1
          AND ceremony_kind = $2
          AND consumed_at IS NULL
          AND expires_at > now()
        RETURNING user_id, state_json
        "#,
    )
    .bind(id)
    .bind(kind)
    .bind(now)
    .fetch_optional(tx.as_mut())
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(CeremonyRow {
        user_id: row.try_get("user_id")?,
        state_json: row.try_get("state_json")?,
    }))
}

async fn insert_mobile_step_up_binding_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ceremony_id: Uuid,
    binding: &MobilePasskeyStepUpBinding,
) -> Result<(), AuthError> {
    sqlx::query(
        r#"
        INSERT INTO auth_webauthn_ceremony_bindings (
            ceremony_id, action_kind, object_id, reason_key, replay_attempt
        ) VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(ceremony_id)
    .bind(binding.action_kind.as_wire())
    .bind(binding.object_id)
    .bind(&binding.reason_key)
    .bind(binding.replay_attempt)
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

async fn load_mobile_step_up_binding_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ceremony_id: Uuid,
) -> Result<Option<MobilePasskeyStepUpBinding>, AuthError> {
    let row = sqlx::query(
        r#"
        SELECT action_kind, object_id, reason_key, replay_attempt
        FROM auth_webauthn_ceremony_bindings
        WHERE ceremony_id = $1
        "#,
    )
    .bind(ceremony_id)
    .fetch_optional(tx.as_mut())
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };
    let action_kind: String = row.try_get("action_kind")?;
    Ok(Some(MobilePasskeyStepUpBinding {
        action_kind: MobileStepUpActionKind::from_wire(&action_kind)?,
        object_id: row.try_get("object_id")?,
        reason_key: row.try_get("reason_key")?,
        replay_attempt: row.try_get("replay_attempt")?,
    }))
}

/// Resolve a webauthn credential's tenant from its credential id, via the narrow
/// SECURITY DEFINER resolver `platform_resolve_credential_org` (migration 0038).
///
/// `auth_webauthn_credentials` is FORCE RLS, so the app's non-owner `console_rt` role
/// cannot read a credential row by credential id until `app.current_org` is armed
/// — but the org is exactly what we need to arm it. This resolver returns ONLY the
/// org_id, breaking that chicken-and-egg without widening any read surface.
/// Returns `None` for an unknown credential id.
async fn resolve_credential_org(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    credential_id: &str,
) -> Result<Option<Uuid>, AuthError> {
    Ok(
        sqlx::query_scalar("SELECT platform_resolve_credential_org($1)")
            .bind(credential_id)
            .fetch_one(tx.as_mut())
            .await?,
    )
}

async fn load_user_passkeys_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    org: OrgId,
    user_id: Uuid,
) -> Result<Vec<Passkey>, AuthError> {
    let rows = sqlx::query(
        "SELECT passkey_json FROM auth_webauthn_credentials WHERE user_id = $1 AND org_id = $2 ORDER BY created_at",
    )
    .bind(user_id)
    .bind(*org.as_uuid())
    .fetch_all(tx.as_mut())
    .await?;

    rows.into_iter()
        .map(|row| {
            let value: serde_json::Value = row.try_get("passkey_json")?;
            Ok(serde_json::from_value(value)?)
        })
        .collect()
}

fn serialize_to_string<T>(value: &T, label: &str) -> Result<String, AuthError>
where
    T: serde::Serialize,
{
    let value = serde_json::to_value(value)?;
    value
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| AuthError::InvalidStoredData(format!("{label} did not serialize as string")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn mobile_step_up_binding_replay_attempt_is_required_but_nullable() {
        let object_id = Uuid::nil();
        let missing_replay_attempt = json!({
            "action_kind": "APPROVAL_DECISION",
            "object_id": object_id,
            "reason_key": "operations_passkey_approval_decision"
        });

        let error = serde_json::from_value::<MobilePasskeyStepUpBinding>(missing_replay_attempt)
            .expect_err("replay_attempt must be present, even when null");
        assert!(error.to_string().contains("replay_attempt"));

        let online_binding: MobilePasskeyStepUpBinding = serde_json::from_value(json!({
            "action_kind": "APPROVAL_DECISION",
            "object_id": object_id,
            "reason_key": "operations_passkey_approval_decision",
            "replay_attempt": null
        }))
        .unwrap();
        assert_eq!(online_binding.replay_attempt, None);

        let replay_binding: MobilePasskeyStepUpBinding = serde_json::from_value(json!({
            "action_kind": "POLL_VOTE",
            "object_id": object_id,
            "reason_key": "operations_passkey_poll_vote",
            "replay_attempt": 1
        }))
        .unwrap();
        assert_eq!(replay_binding.replay_attempt, Some(1));
    }

    #[test]
    fn mobile_step_up_binding_rejects_zero_replay_attempt() {
        let binding: MobilePasskeyStepUpBinding = serde_json::from_value(json!({
            "action_kind": "APPROVAL_DECISION",
            "object_id": Uuid::nil(),
            "reason_key": "operations_passkey_approval_decision",
            "replay_attempt": 0
        }))
        .unwrap();

        assert!(matches!(
            binding.validate(),
            Err(MobileStepUpBindingError::InvalidReplayAttempt)
        ));
    }
}

impl PasskeyService {
    pub async fn start_account_registration_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        pending: &crate::account::PendingAccount,
        head: &crate::account::AccountTermsHead,
        origin: &str,
    ) -> Result<crate::account::AccountRegistrationChallenge, crate::account::AccountOperationError>
    {
        use crate::account::{AccountOperationError, AccountRegistrationChallenge};
        self.require_native_origin(origin)?;
        if pending.account_id.is_nil()
            || pending.security_generation != 1
            || pending.revision != 1
            || pending.context_generation != 1
            || head.manifest_sha256.len() != 32
            || head.revision <= 0
            || self.ceremony_ttl <= Duration::ZERO
        {
            return Err(AccountOperationError::AuthorityUnavailable);
        }
        let expires_at = pending
            .created_at
            .checked_add(self.ceremony_ttl.min(Duration::minutes(5)))
            .ok_or(AccountOperationError::AuthorityUnavailable)?;
        let now = crate::account::account_now_in_tx(tx).await?;
        if pending.created_at > now || expires_at <= now {
            return Err(AccountOperationError::EnrollmentInvalid);
        }
        // The handle is the freshly allocated Account's exact 16 bytes. No
        // Company identity, display-name inference or legacy fenced-user path.
        let label = pending.account_id.to_string();
        let (mut challenge, state) = self
            .webauthn
            .start_passkey_registration(pending.account_id, &label, "Console account", None)
            .map_err(|_| AccountOperationError::AuthorityUnavailable)?;
        let selection = challenge
            .public_key
            .authenticator_selection
            .as_mut()
            .ok_or(AccountOperationError::AuthorityUnavailable)?;
        // webauthn-rs hides the enum's module; deserialize only this fixed typed
        // option, never opaque crypto state. Preserve required UV from the owner.
        selection.resident_key = Some(
            serde_json::from_str("\"required\"")
                .map_err(|_| AccountOperationError::AuthorityUnavailable)?,
        );
        selection.require_resident_key = true;
        let ceremony_id = Uuid::new_v4();
        let browser_nonce = crate::refresh::RefreshToken(crate::refresh::generate_refresh_token());
        let challenge_json = serde_json::to_value(&challenge)
            .map_err(|_| AccountOperationError::AuthorityUnavailable)?;
        let state_json = serde_json::to_value(&state)
            .map_err(|_| AccountOperationError::AuthorityUnavailable)?;
        sqlx::query("INSERT INTO public.auth_webauthn_ceremonies (id, user_id, ceremony_kind, challenge_json, state_json, expires_at, created_at, account_browser_flow, browser_nonce_sha256, browser_origin, terms_manifest_sha256, terms_head_revision) VALUES ($1,$2,'registration',$3,$4,$5,$6,'ACCOUNT_REGISTRATION',$7,$8,$9,$10)")
            .bind(ceremony_id).bind(pending.account_id).bind(challenge_json).bind(state_json)
            .bind(expires_at).bind(pending.created_at).bind(crate::refresh::hash_token(browser_nonce.as_str()))
            .bind(origin).bind(&head.manifest_sha256).bind(head.revision)
            .execute(tx.as_mut()).await?;
        if expires_at <= crate::account::account_now_in_tx(tx).await? {
            return Err(AccountOperationError::EnrollmentInvalid);
        }
        Ok(AccountRegistrationChallenge {
            ceremony: RegistrationCeremony {
                ceremony_id,
                challenge,
                expires_at,
            },
            browser_nonce,
        })
    }

    /// Nonlocking correlation only; finish rechecks every binding under guards.
    pub async fn correlate_account_registration_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        ceremony: Uuid,
        nonce: &str,
        origin: &str,
    ) -> Result<crate::account::AccountRegistrationBinding, crate::account::AccountOperationError>
    {
        use crate::account::{AccountOperationError, AccountRegistrationBinding};
        self.require_native_origin(origin)?;
        if ceremony.is_nil() || nonce.is_empty() || nonce.len() > 512 {
            return Err(AccountOperationError::EnrollmentInvalid);
        }
        let row = sqlx::query("SELECT user_id, terms_manifest_sha256, terms_head_revision, created_at, expires_at, consumed_at FROM public.auth_webauthn_ceremonies WHERE id = $1 AND ceremony_kind = 'registration' AND account_browser_flow = 'ACCOUNT_REGISTRATION' AND browser_nonce_sha256 = $2 AND browser_origin = $3")
            .bind(ceremony).bind(crate::refresh::hash_token(nonce)).bind(origin)
            .fetch_optional(tx.as_mut()).await?.ok_or(AccountOperationError::EnrollmentInvalid)?;
        let created_at: OffsetDateTime = row.try_get("created_at")?;
        let expires_at: OffsetDateTime = row.try_get("expires_at")?;
        let consumed_at: Option<OffsetDateTime> = row.try_get("consumed_at")?;
        let now = crate::account::account_now_in_tx(tx).await?;
        if created_at > now
            || expires_at <= now
            || consumed_at.is_some()
            || expires_at > created_at + Duration::minutes(5)
        {
            return Err(AccountOperationError::EnrollmentInvalid);
        }
        let binding = AccountRegistrationBinding {
            account_id: row.try_get("user_id")?,
            terms_manifest_sha256: row.try_get("terms_manifest_sha256")?,
            terms_head_revision: row.try_get("terms_head_revision")?,
        };
        if binding.account_id.is_nil()
            || binding.terms_manifest_sha256.len() != 32
            || binding.terms_head_revision <= 0
        {
            return Err(AccountOperationError::AuthorityUnavailable);
        }
        Ok(binding)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn finish_account_registration_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        binding: &crate::account::AccountRegistrationBinding,
        ceremony: Uuid,
        nonce: &str,
        origin: &str,
        credential: PasskeyRegistrationCredential,
    ) -> Result<Uuid, crate::account::AccountOperationError> {
        use crate::account::AccountOperationError;
        self.require_native_origin(origin)?;
        if ceremony.is_nil() || nonce.is_empty() || nonce.len() > 512 {
            return Err(AccountOperationError::EnrollmentInvalid);
        }
        // The coordinator already holds Account EXCLUSIVE, terms SHARE and the
        // tentative family/token. This is the next lock class, never a new tx.
        let row = sqlx::query("SELECT state_json, created_at, expires_at, consumed_at FROM public.auth_webauthn_ceremonies WHERE id = $1 AND user_id = $2 AND ceremony_kind = 'registration' AND account_browser_flow = 'ACCOUNT_REGISTRATION' AND browser_nonce_sha256 = $3 AND browser_origin = $4 AND terms_manifest_sha256 = $5 AND terms_head_revision = $6 FOR UPDATE")
            .bind(ceremony).bind(binding.account_id).bind(crate::refresh::hash_token(nonce))
            .bind(origin).bind(&binding.terms_manifest_sha256).bind(binding.terms_head_revision)
            .fetch_optional(tx.as_mut()).await?.ok_or(AccountOperationError::EnrollmentInvalid)?;
        let created_at: OffsetDateTime = row.try_get("created_at")?;
        let expires_at: OffsetDateTime = row.try_get("expires_at")?;
        let consumed_at: Option<OffsetDateTime> = row.try_get("consumed_at")?;
        let now = crate::account::account_now_in_tx(tx).await?;
        if created_at > now
            || expires_at <= now
            || consumed_at.is_some()
            || expires_at > created_at + Duration::minutes(5)
        {
            return Err(AccountOperationError::EnrollmentInvalid);
        }
        let state_json: serde_json::Value = row.try_get("state_json")?;
        let state: PasskeyRegistration = serde_json::from_value(state_json)
            .map_err(|_| AccountOperationError::AuthorityUnavailable)?;
        // Legacy may admit extra origins. Native admits the exact configured
        // browser origin in the cryptographically verified clientDataJSON too.
        #[derive(Deserialize)]
        struct NativeClientData {
            origin: String,
            #[serde(rename = "crossOrigin")]
            cross_origin: Option<bool>,
        }
        let client_bytes: &[u8] = credential.response.client_data_json.as_ref();
        if client_bytes.len() > 8192 {
            return Err(AccountOperationError::EnrollmentInvalid);
        }
        let client: NativeClientData = serde_json::from_slice(client_bytes)
            .map_err(|_| AccountOperationError::EnrollmentInvalid)?;
        self.require_native_client_origin(&client.origin, client.cross_origin)?;
        let passkey = self
            .webauthn
            .finish_passkey_registration(&credential, &state)
            .map_err(|_| AccountOperationError::EnrollmentInvalid)?;
        let passkey_json = serde_json::to_value(&passkey)
            .map_err(|_| AccountOperationError::AuthorityUnavailable)?;
        let credential_id = serialize_to_string(passkey.cred_id(), "passkey credential id")
            .map_err(|_| AccountOperationError::AuthorityUnavailable)?;
        let key = Uuid::new_v4();
        let finished_at = crate::account::account_now_in_tx(tx).await?;
        if expires_at <= finished_at {
            return Err(AccountOperationError::EnrollmentInvalid);
        }
        sqlx::query("INSERT INTO public.auth_webauthn_credentials (id, user_id, credential_id, passkey_json, created_at, org_id) VALUES ($1,$2,$3,$4,$5,NULL)")
            .bind(key).bind(binding.account_id).bind(credential_id).bind(passkey_json).bind(finished_at)
            .execute(tx.as_mut()).await.map_err(|error| {
                if error.as_database_error().is_some_and(|db| db.is_unique_violation()) {
                    AccountOperationError::EnrollmentInvalid
                } else { AccountOperationError::AuthorityUnavailable }
            })?;
        sqlx::query("UPDATE public.auth_webauthn_ceremonies SET consumed_at = $2 WHERE id = $1")
            .bind(ceremony)
            .bind(finished_at)
            .execute(tx.as_mut())
            .await?;
        Ok(key)
    }

    fn require_native_client_origin(
        &self,
        origin: &str,
        cross_origin: Option<bool>,
    ) -> Result<(), crate::account::AccountOperationError> {
        if cross_origin == Some(true) {
            return Err(crate::account::AccountOperationError::EnrollmentInvalid);
        }
        // Pinned CollectedClientData serializes its Url with a root slash.
        // Accept only that one representation of the exact configured origin;
        // do not normalize paths, userinfo, query, fragment or extra origins.
        self.require_native_origin(origin.strip_suffix('/').unwrap_or(origin))
    }

    fn require_native_origin(
        &self,
        origin: &str,
    ) -> Result<(), crate::account::AccountOperationError> {
        if !self.native_origin.starts_with("https://") || origin != self.native_origin {
            return Err(crate::account::AccountOperationError::EnrollmentInvalid);
        }
        Ok(())
    }
}

#[cfg(test)]
mod native_client_origin_tests {
    use super::*;

    #[test]
    fn accepts_only_exact_origin_and_one_serialized_root_slash() {
        let origin = Url::parse("https://auth.example.com").unwrap();
        let serialized: String =
            serde_json::from_str(&serde_json::to_string(&origin).unwrap()).unwrap();
        let service = PasskeyService::new(WebauthnSettings {
            rp_id: "example.com".to_owned(),
            rp_origin: origin,
            rp_name: "Console test".to_owned(),
            extra_allowed_origins: vec![Url::parse("https://legacy.example.com").unwrap()],
            ceremony_ttl: Duration::minutes(5),
        })
        .unwrap();
        assert_eq!(serialized, "https://auth.example.com/");
        assert_ne!(serialized, service.native_origin);
        for accepted in [service.native_origin.as_str(), serialized.as_str()] {
            for cross_origin in [None, Some(false)] {
                assert!(
                    service
                        .require_native_client_origin(accepted, cross_origin)
                        .is_ok()
                );
            }
            assert!(matches!(
                service.require_native_client_origin(accepted, Some(true)),
                Err(crate::account::AccountOperationError::EnrollmentInvalid)
            ));
        }
        // The HTTP/stored-flow origin guard remains byte-exact.
        assert!(
            service
                .require_native_origin(&service.native_origin)
                .is_ok()
        );
        assert!(matches!(
            service.require_native_origin(&serialized),
            Err(crate::account::AccountOperationError::EnrollmentInvalid)
        ));
        for denied in [
            "",
            "null",
            "/",
            "https://auth.example.com//",
            "https://auth.example.com/path",
            "https://auth.example.com/path/",
            "https://auth.example.com/.",
            "https://auth.example.com/..",
            "https://auth.example.com?query=1",
            "https://auth.example.com/?query=1",
            "https://auth.example.com#fragment",
            "https://auth.example.com/#fragment",
            "https://user@auth.example.com/",
            "https://user:password@auth.example.com/",
            "https://auth.example.com@evil.example/",
            "http://auth.example.com/",
            "https://sub.auth.example.com/",
            "https://auth.example.com.evil/",
            "https://auth.example.com:444/",
            "https://auth.example.com:443/",
            "https://AUTH.example.com/",
            " https://auth.example.com/",
            "https://auth.example.com/ ",
            "https://auth.example.com/\n",
            "https://auth.example.com/\0",
            "https://auth.example.com\\",
            "https://auth.example.com/%2f",
            "https://legacy.example.com",
            "https://legacy.example.com/",
        ] {
            assert!(matches!(
                service.require_native_client_origin(denied, None),
                Err(crate::account::AccountOperationError::EnrollmentInvalid)
            ));
        }
    }
}

impl PasskeyService {
    pub async fn start_account_login_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        origin: &str,
    ) -> Result<crate::account::AccountLoginChallenge, crate::account::AccountOperationError> {
        use crate::account::{AccountLoginChallenge, AccountOperationError, account_now_in_tx};
        self.require_native_origin(origin)
            .map_err(|_| AccountOperationError::AuthenticationInvalid)?;
        if self.ceremony_ttl <= Duration::ZERO {
            return Err(AccountOperationError::AuthorityUnavailable);
        }
        let (challenge, state) = self
            .webauthn
            .start_discoverable_authentication()
            .map_err(|_| AccountOperationError::AuthorityUnavailable)?;
        let now = account_now_in_tx(tx).await?;
        let expires_at = now
            .checked_add(self.ceremony_ttl.min(Duration::minutes(5)))
            .ok_or(AccountOperationError::AuthorityUnavailable)?;
        let ceremony_id = Uuid::new_v4();
        let browser_nonce = crate::refresh::RefreshToken(crate::refresh::generate_refresh_token());
        let challenge_json = serde_json::to_value(&challenge)
            .map_err(|_| AccountOperationError::AuthorityUnavailable)?;
        let state_json = serde_json::to_value(&state)
            .map_err(|_| AccountOperationError::AuthorityUnavailable)?;
        sqlx::query("INSERT INTO public.auth_webauthn_ceremonies (id, user_id, ceremony_kind, challenge_json, state_json, expires_at, created_at, account_browser_flow, browser_nonce_sha256, browser_origin) VALUES ($1,NULL,'authentication',$2,$3,$4,$5,'ACCOUNT_LOGIN',$6,$7)")
            .bind(ceremony_id).bind(challenge_json).bind(state_json).bind(expires_at).bind(now)
            .bind(crate::refresh::hash_token(browser_nonce.as_str())).bind(origin)
            .execute(tx.as_mut()).await?;
        if expires_at <= account_now_in_tx(tx).await? {
            return Err(AccountOperationError::AuthenticationInvalid);
        }
        Ok(AccountLoginChallenge {
            ceremony: AuthenticationCeremony {
                ceremony_id,
                challenge,
                expires_at,
            },
            browser_nonce,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn finish_account_login_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        ceremony: Uuid,
        nonce: &str,
        origin: &str,
        credential: PasskeyAuthenticationCredential,
        refresh_ttl: Duration,
        absolute_ttl: Duration,
    ) -> Result<crate::account::AccountPrimaryLogin, crate::account::AccountOperationError> {
        use crate::account::{
            AccountOperationError, AccountPrimaryLogin, account_now_in_tx, lock_account_in_tx,
        };
        let invalid = || AccountOperationError::AuthenticationInvalid;
        self.require_native_origin(origin).map_err(|_| invalid())?;
        if ceremony.is_nil() || nonce.is_empty() || nonce.len() > 512 {
            return Err(invalid());
        }
        let credential_id = serialize_to_string(&credential.raw_id, "native credential id")
            .map_err(|_| invalid())?;
        // Neither the credential identifier nor the unsigned userHandle is proof.
        // Correlate only native storage, then lock Account before its new private
        // family/token, the exact key and finally the exact login ceremony.
        let account: Uuid = sqlx::query_scalar("SELECT user_id FROM public.auth_webauthn_credentials WHERE credential_id=$1 AND org_id IS NULL")
            .bind(&credential_id).fetch_optional(tx.as_mut()).await?.ok_or_else(invalid)?;
        let security = lock_account_in_tx(tx, account, false, "ACTIVE").await?;
        if credential.get_user_unique_id() != Some(account.as_bytes().as_slice()) {
            return Err(invalid());
        }
        let family = crate::RefreshTokenStore
            .prepare_account_login_family_in_tx(tx, &security, refresh_ttl, absolute_ttl)
            .await?;
        let key = sqlx::query("SELECT id, passkey_json FROM public.auth_webauthn_credentials WHERE credential_id=$1 AND user_id=$2 AND org_id IS NULL FOR UPDATE")
            .bind(&credential_id).bind(account).fetch_optional(tx.as_mut()).await?.ok_or_else(invalid)?;
        let key_id: Uuid = key.try_get("id")?;
        let row = sqlx::query("SELECT state_json, created_at, expires_at, consumed_at FROM public.auth_webauthn_ceremonies c WHERE id=$1 AND user_id IS NULL AND ceremony_kind='authentication' AND account_browser_flow='ACCOUNT_LOGIN' AND browser_nonce_sha256=$2 AND browser_origin=$3 AND terms_manifest_sha256 IS NULL AND terms_head_revision IS NULL AND NOT EXISTS (SELECT 1 FROM public.auth_webauthn_ceremony_bindings b WHERE b.ceremony_id=c.id) FOR UPDATE")
            .bind(ceremony).bind(crate::refresh::hash_token(nonce)).bind(origin)
            .fetch_optional(tx.as_mut()).await?.ok_or_else(invalid)?;
        let created_at: OffsetDateTime = row.try_get("created_at")?;
        let expires_at: OffsetDateTime = row.try_get("expires_at")?;
        let consumed_at: Option<OffsetDateTime> = row.try_get("consumed_at")?;
        let now = account_now_in_tx(tx).await?;
        if created_at > now
            || expires_at <= now
            || consumed_at.is_some()
            || expires_at > created_at + Duration::minutes(5)
        {
            return Err(invalid());
        }
        let state_json: serde_json::Value = row.try_get("state_json")?;
        let state: DiscoverableAuthentication = serde_json::from_value(state_json)
            .map_err(|_| AccountOperationError::AuthorityUnavailable)?;
        let passkey_json: serde_json::Value = key.try_get("passkey_json")?;
        let mut passkey: Passkey = serde_json::from_value(passkey_json)
            .map_err(|_| AccountOperationError::AuthorityUnavailable)?;
        if serialize_to_string(passkey.cred_id(), "stored native credential id")
            .map_err(|_| AccountOperationError::AuthorityUnavailable)?
            != credential_id
        {
            return Err(AccountOperationError::AuthorityUnavailable);
        }
        #[derive(Deserialize)]
        struct NativeClientData {
            origin: String,
            #[serde(rename = "crossOrigin")]
            cross_origin: Option<bool>,
        }
        let client_bytes: &[u8] = credential.response.client_data_json.as_ref();
        if client_bytes.len() > 8192 {
            return Err(invalid());
        }
        let client: NativeClientData =
            serde_json::from_slice(client_bytes).map_err(|_| invalid())?;
        self.require_native_client_origin(&client.origin, client.cross_origin)
            .map_err(|_| invalid())?;
        // The pinned verifier receives original signed bytes. It enforces RP,
        // challenge, origin, UP/UV and nonzero counter monotonicity; no fake state.
        let verified = self
            .webauthn
            .finish_discoverable_authentication(
                &credential,
                state,
                &[DiscoverableKey::from(&passkey)],
            )
            .map_err(|_| invalid())?;
        if !verified.user_verified() {
            return Err(invalid());
        }
        passkey.update_credential(&verified).ok_or_else(invalid)?;
        let finished_at = account_now_in_tx(tx).await?;
        if expires_at <= finished_at
            || family.token_expires_at <= finished_at
            || family.family_expires_at <= finished_at
        {
            return Err(invalid());
        }
        let passkey_json = serde_json::to_value(&passkey)
            .map_err(|_| AccountOperationError::AuthorityUnavailable)?;
        let changed = sqlx::query("UPDATE public.auth_webauthn_credentials SET passkey_json=$1, last_used_at=$2 WHERE id=$3 AND user_id=$4 AND credential_id=$5 AND org_id IS NULL")
            .bind(passkey_json).bind(finished_at).bind(key_id).bind(account).bind(&credential_id)
            .execute(tx.as_mut()).await?.rows_affected();
        let consumed = sqlx::query("UPDATE public.auth_webauthn_ceremonies SET consumed_at=$2 WHERE id=$1 AND consumed_at IS NULL")
            .bind(ceremony).bind(finished_at).execute(tx.as_mut()).await?.rows_affected();
        if changed != 1 || consumed != 1 {
            return Err(AccountOperationError::AuthorityUnavailable);
        }
        Ok(AccountPrimaryLogin {
            account_id: account,
            security_generation: security.security_generation,
            family,
            ceremony_expires_at: expires_at,
        })
    }
}
