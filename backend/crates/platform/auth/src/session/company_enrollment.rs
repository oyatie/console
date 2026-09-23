//! Auth-owned credentials for the Company owner's exclusive lock plan.
//! These values never substitute for current durable Account/family authority.
use super::*;
use crate::{AccountCsrfClaims, AccountCsrfTokenInput, JwtIssuer};

/// Ephemeral proof for one authorized form. Never log or persist this value.
pub struct AccountFormProof {
    token: String,
    expires_at: OffsetDateTime,
}

impl AccountFormProof {
    pub fn as_str(&self) -> &str {
        self.token.as_str()
    }

    pub const fn expires_at(&self) -> OffsetDateTime {
        self.expires_at
    }
}

/// Bounded immutable transport bytes. Construction performs no authentication.
pub struct AccountEnrollmentCredentials {
    access: String,
    csrf: Option<String>,
}

/// The owner must finish successfully after its last wait, then commit the same
/// transaction. Dropping this value is not successful validation or a commit.
#[must_use = "finish the credential guard after owner SQL and before commit"]
pub struct LockedAccountEnrollment<'borrow, 'connection> {
    credentials: &'borrow AccountEnrollmentCredentials,
    tx: &'borrow mut Transaction<'connection, Postgres>,
    verifier: &'borrow JwtVerifier,
    family: NativeFamily,
    planned_recipient: Option<Uuid>,
    planned_input_digest: Option<Vec<u8>>,
}

impl AccountEnrollmentCredentials {
    pub fn for_read(access: &str) -> Result<Self, AccountOperationError> {
        if access.is_empty() || access.len() > 16 * 1024 {
            return Err(AccountOperationError::AuthenticationInvalid);
        }
        Ok(Self {
            access: access.to_owned(),
            csrf: None,
        })
    }

    pub fn for_mutation(access: &str, csrf: &str) -> Result<Self, AccountOperationError> {
        let mut credentials = Self::for_read(access)?;
        if csrf.is_empty() || csrf.len() > 4 * 1024 {
            return Err(AccountOperationError::CsrfInvalid);
        }
        credentials.csrf = Some(csrf.to_owned());
        Ok(credentials)
    }

    /// Signed namespace only; no Account/family rows or shared locks are read.
    pub async fn account_id_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        verifier: &JwtVerifier,
        absolute_ttl: Duration,
    ) -> Result<Uuid, AccountOperationError> {
        if absolute_ttl <= Duration::ZERO {
            return Err(AccountOperationError::AuthorityUnavailable);
        }
        Ok(self.verify_at(verifier, account_now_in_tx(tx).await?)?.sub)
    }

    /// Signed namespace only; no Account/family rows or shared locks are read.
    pub async fn session_ids_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        verifier: &JwtVerifier,
        absolute_ttl: Duration,
    ) -> Result<(Uuid, Uuid), AccountOperationError> {
        if absolute_ttl <= Duration::ZERO {
            return Err(AccountOperationError::AuthorityUnavailable);
        }
        let claims = self.verify_at(verifier, account_now_in_tx(tx).await?)?;
        Ok((claims.sub, claims.sid))
    }

    /// Resolve current read authority and retain the existing Account/family
    /// shared guards until the caller ends this transaction. This read does not
    /// consume or validate a captured mutation CSRF proof.
    pub async fn read_session_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        verifier: &JwtVerifier,
        absolute_ttl: Duration,
    ) -> Result<AccountLiveSession, AccountOperationError> {
        live_account_session_in_tx(tx, verifier, &self.access, absolute_ttl).await
    }

    /// Check the originally captured mutation proof in the retained owner scope.
    /// Call only after the owner's Group-first lock plan, and again after waits.
    pub async fn validate_mutation_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        verifier: &JwtVerifier,
        absolute_ttl: Duration,
    ) -> Result<AccountLiveSession, AccountOperationError> {
        let proof = self
            .csrf
            .as_deref()
            .ok_or(AccountOperationError::CsrfInvalid)?;
        let session = self.read_session_in_tx(tx, verifier, absolute_ttl).await?;
        let now = ensure_account_session_fresh_in_tx(tx, &session).await?;
        verify_form_proof(verifier, proof, &session, now)?;
        Ok(session)
    }

    /// Issue only for an already authorized, rate-limited form read. Mutation
    /// credentials cannot mint a replacement proof during command execution.
    pub async fn issue_form_proof_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        verifier: &JwtVerifier,
        issuer: &JwtIssuer,
        absolute_ttl: Duration,
    ) -> Result<AccountFormProof, AccountOperationError> {
        if self.csrf.is_some() {
            return Err(AccountOperationError::CsrfInvalid);
        }
        let session = self.read_session_in_tx(tx, verifier, absolute_ttl).await?;
        let issued_at = ensure_account_session_fresh_in_tx(tx, &session).await?;
        let token = issuer
            .issue_account_csrf_token(AccountCsrfTokenInput {
                account_id: session.account_id,
                session_id: session.session_id,
                security_generation: session.security_generation,
                issued_at,
                family_expires_at: session.family_expires_at,
            })
            .map_err(|_| AccountOperationError::AuthorityUnavailable)?;
        let now = ensure_account_session_fresh_in_tx(tx, &session).await?;
        let claims = verify_form_proof(verifier, token.as_str(), &session, now)
            .map_err(|_| AccountOperationError::AuthorityUnavailable)?;
        let expires_at = OffsetDateTime::from_unix_timestamp(claims.exp)
            .map_err(|_| AccountOperationError::AuthorityUnavailable)?;
        Ok(AccountFormProof {
            token: token.as_str().to_owned(),
            expires_at,
        })
    }

    /// Return the original submitted proof only after current verification.
    /// Validation redisplay cannot mint or substitute a new credential.
    pub async fn retain_submitted_form_proof_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        verifier: &JwtVerifier,
        absolute_ttl: Duration,
    ) -> Result<AccountFormProof, AccountOperationError> {
        let proof = self
            .csrf
            .as_deref()
            .ok_or(AccountOperationError::CsrfInvalid)?;
        let session = self.read_session_in_tx(tx, verifier, absolute_ttl).await?;
        let now = ensure_account_session_fresh_in_tx(tx, &session).await?;
        let claims = verify_form_proof(verifier, proof, &session, now)?;
        let expires_at = OffsetDateTime::from_unix_timestamp(claims.exp)
            .map_err(|_| AccountOperationError::CsrfInvalid)?;
        Ok(AccountFormProof {
            token: proof.to_owned(),
            expires_at,
        })
    }

    /// Recheck after the last owner wait; this never replaces the issued token.
    pub async fn recheck_form_proof_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        verifier: &JwtVerifier,
        absolute_ttl: Duration,
        proof: &AccountFormProof,
    ) -> Result<(), AccountOperationError> {
        let session = self.read_session_in_tx(tx, verifier, absolute_ttl).await?;
        let now = ensure_account_session_fresh_in_tx(tx, &session).await?;
        verify_form_proof(verifier, proof.as_str(), &session, now)?;
        Ok(())
    }

    pub async fn lock_submit_in_tx<'b, 'c>(
        &'b self,
        tx: &'b mut Transaction<'c, Postgres>,
        verifier: &'b JwtVerifier,
        absolute_ttl: Duration,
        command: Uuid,
        input: &[u8],
    ) -> Result<LockedAccountEnrollment<'b, 'c>, AccountOperationError> {
        self.lock_in_tx(tx, verifier, absolute_ttl, command, Some(input), true)
            .await
    }

    pub async fn lock_status_in_tx<'b, 'c>(
        &'b self,
        tx: &'b mut Transaction<'c, Postgres>,
        verifier: &'b JwtVerifier,
        absolute_ttl: Duration,
        command: Uuid,
    ) -> Result<LockedAccountEnrollment<'b, 'c>, AccountOperationError> {
        self.lock_in_tx(tx, verifier, absolute_ttl, command, None, false)
            .await
    }

    pub async fn lock_cancel_in_tx<'b, 'c>(
        &'b self,
        tx: &'b mut Transaction<'c, Postgres>,
        verifier: &'b JwtVerifier,
        absolute_ttl: Duration,
        command: Uuid,
    ) -> Result<LockedAccountEnrollment<'b, 'c>, AccountOperationError> {
        self.lock_in_tx(tx, verifier, absolute_ttl, command, None, true)
            .await
    }

    async fn lock_in_tx<'b, 'c>(
        &'b self,
        tx: &'b mut Transaction<'c, Postgres>,
        verifier: &'b JwtVerifier,
        absolute_ttl: Duration,
        command: Uuid,
        input: Option<&[u8]>,
        mutation: bool,
    ) -> Result<LockedAccountEnrollment<'b, 'c>, AccountOperationError> {
        if absolute_ttl <= Duration::ZERO {
            return Err(AccountOperationError::AuthorityUnavailable);
        }
        if mutation && self.csrf.is_none() {
            return Err(AccountOperationError::CsrfInvalid);
        }
        let claims = self.verify_at(verifier, account_now_in_tx(tx).await?)?;
        let rows =
            sqlx::query("SELECT * FROM public.company_enrollment_session_material_v1($1,$2,$3,$4)")
                .bind(claims.sub)
                .bind(claims.sid)
                .bind(command)
                .bind(input)
                .fetch_all(tx.as_mut())
                .await
                .map_err(|error| match error.as_database_error() {
                    Some(db)
                        if db.code().as_deref() == Some("P0001")
                            && db.message() == "account.authentication_invalid" =>
                    {
                        AccountOperationError::AuthenticationInvalid
                    }
                    _ => AccountOperationError::AuthorityUnavailable,
                })?;
        if rows.len() != 1 {
            return Err(AccountOperationError::AuthorityUnavailable);
        }
        let row = &rows[0];
        let control = account_security_from_row(row, claims.sub, "ACTIVE")?;
        let family = family_from_row(row, &control, claims.sid, absolute_ttl)?;
        let planned_recipient: Option<Uuid> = row.try_get("planned_recipient")?;
        let planned_input_digest: Option<Vec<u8>> = row.try_get("planned_input_digest")?;
        if planned_recipient.is_some_and(|id| id.is_nil())
            || (input.is_some() && planned_input_digest.is_none() && planned_recipient.is_none())
            || (input.is_none() && planned_recipient.is_some())
            || planned_input_digest
                .as_ref()
                .is_some_and(|digest| digest.len() != 32)
        {
            return Err(AccountOperationError::AuthorityUnavailable);
        }
        self.verify_family_at(verifier, &family, account_now_in_tx(tx).await?)?;
        Ok(LockedAccountEnrollment {
            credentials: self,
            tx,
            verifier,
            family,
            planned_recipient,
            planned_input_digest,
        })
    }

    fn verify_at(
        &self,
        verifier: &JwtVerifier,
        now: OffsetDateTime,
    ) -> Result<AccountAccessClaims, AccountOperationError> {
        let AccountAccessVerification::Current(claims) = verifier
            .verify_account_access_token(&self.access, now)
            .map_err(|_| AccountOperationError::AuthenticationInvalid)?
        else {
            return Err(AccountOperationError::AuthenticationInvalid);
        };
        if let Some(proof) = &self.csrf {
            let proof = verifier
                .verify_account_csrf_token(proof, now)
                .map_err(|_| AccountOperationError::CsrfInvalid)?;
            if proof.sub != claims.sub
                || proof.sid != claims.sid
                || proof.security_generation != claims.security_generation
            {
                return Err(AccountOperationError::CsrfInvalid);
            }
        }
        Ok(claims)
    }

    fn verify_family_at(
        &self,
        verifier: &JwtVerifier,
        family: &NativeFamily,
        now: OffsetDateTime,
    ) -> Result<(), AccountOperationError> {
        let claims = self.verify_at(verifier, now)?;
        if claims.sub != family.account_id
            || claims.sid != family.family_id
            || claims.security_generation != family.generation
            || claims.auth_time != family.auth_time.unix_timestamp()
            || claims.assurance != AccountAssurance::PasskeyPrimary
            || claims.iat > now.unix_timestamp()
            || claims.nbf > now.unix_timestamp()
            || claims.iat < family.created_at.unix_timestamp()
            || claims.exp > family.expires_at.unix_timestamp()
            || family.created_at > now
            || family.auth_time > now
            || family.expires_at <= now
        {
            return Err(AccountOperationError::AuthenticationInvalid);
        }
        if let Some(proof) = &self.csrf {
            let proof = verifier
                .verify_account_csrf_token(proof, now)
                .map_err(|_| AccountOperationError::CsrfInvalid)?;
            if proof.exp > family.expires_at.unix_timestamp() {
                return Err(AccountOperationError::CsrfInvalid);
            }
        }
        Ok(())
    }
}

fn verify_form_proof(
    verifier: &JwtVerifier,
    proof: &str,
    session: &AccountLiveSession,
    now: OffsetDateTime,
) -> Result<AccountCsrfClaims, AccountOperationError> {
    let claims = verifier
        .verify_account_csrf_token(proof, now)
        .map_err(|_| AccountOperationError::CsrfInvalid)?;
    if claims.sub != session.account_id
        || claims.sid != session.session_id
        || claims.security_generation != session.security_generation
        || claims.exp > session.family_expires_at.unix_timestamp()
    {
        return Err(AccountOperationError::CsrfInvalid);
    }
    Ok(claims)
}

impl LockedAccountEnrollment<'_, '_> {
    pub fn account_id(&self) -> Uuid {
        self.family.account_id
    }
    pub fn session_id(&self) -> Uuid {
        self.family.family_id
    }
    pub fn connection(&mut self) -> &mut sqlx::PgConnection {
        self.tx.as_mut()
    }
    pub fn planned_recipient(&self) -> Option<Uuid> {
        self.planned_recipient
    }
    pub fn planned_input_digest(&self) -> Option<&[u8]> {
        self.planned_input_digest.as_deref()
    }

    pub async fn finish(self) -> Result<OffsetDateTime, AccountOperationError> {
        let now = account_now_in_tx(self.tx).await?;
        self.credentials
            .verify_family_at(self.verifier, &self.family, now)?;
        Ok(now)
    }
}
