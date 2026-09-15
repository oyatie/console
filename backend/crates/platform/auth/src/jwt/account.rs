//! Account protocol cryptography only. Verified claims do not establish a live
//! Account, current generation, family ownership, revocation state, or session.
//! Callers must load that authority and enforce the family's absolute lifetime.
//! Primary WebAuthn proof and browser/CSRF binding belong to those callers too.

use std::fmt;

use jsonwebtoken::{Algorithm, Header, Validation, decode, encode, errors::ErrorKind};
use serde::{Deserialize, Deserializer, Serialize, de};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use super::{JwtIssuer, JwtVerifier};
use crate::AuthError;

const ACCESS_AUDIENCE: &str = "console.account.v1";
const ACCESS_KIND: &str = "account_v1";
const CSRF_AUDIENCE: &str = "console.account.csrf.v1";
const CSRF_KIND: &str = "account_csrf_v1";
const ACCESS_MAX_SECONDS: i64 = 900;
const CSRF_MAX_SECONDS: i64 = 300;

/// Protocol metadata supplied only after actual primary authentication proof.
#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AccountAssurance {
    #[serde(rename = "PASSKEY_PRIMARY")]
    PasskeyPrimary,
}

impl<'de> Deserialize<'de> for AccountAssurance {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match String::deserialize(deserializer)?.as_str() {
            "PASSKEY_PRIMARY" => Ok(Self::PasskeyPrimary),
            _ => Err(de::Error::custom("invalid account assurance")),
        }
    }
}

pub struct AccountAccessTokenInput {
    pub account_id: Uuid,
    pub session_id: Uuid,
    pub security_generation: i64,
    pub auth_time: OffsetDateTime,
    pub assurance: AccountAssurance,
    pub issued_at: OffsetDateTime,
    pub family_expires_at: OffsetDateTime,
}

pub struct AccountCsrfTokenInput {
    pub account_id: Uuid,
    pub session_id: Uuid,
    pub security_generation: i64,
    pub issued_at: OffsetDateTime,
    pub family_expires_at: OffsetDateTime,
}

/// Signed credential bytes are available only through explicit access.
pub struct SignedAccountToken(String);

impl SignedAccountToken {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SignedAccountToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SignedAccountToken([REDACTED])")
    }
}

/// Closed signed claims; possession alone conveys no current Account authority.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountAccessClaims {
    pub iss: String,
    pub aud: String,
    pub token_kind: String,
    #[serde(deserialize_with = "canonical_uuid")]
    pub sub: Uuid,
    #[serde(deserialize_with = "canonical_uuid")]
    pub sid: Uuid,
    #[serde(deserialize_with = "canonical_uuid")]
    pub jti: Uuid,
    pub iat: i64,
    pub nbf: i64,
    pub exp: i64,
    #[serde(with = "positive_generation")]
    pub security_generation: i64,
    pub auth_time: i64,
    pub assurance: AccountAssurance,
}

impl fmt::Debug for AccountAccessClaims {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AccountAccessClaims([REDACTED])")
    }
}

/// Closed proof claims; callers must enforce browser binding and matching
/// Account, family, and generation against separately verified access claims.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountCsrfClaims {
    pub iss: String,
    pub aud: String,
    pub token_kind: String,
    #[serde(deserialize_with = "canonical_uuid")]
    pub sub: Uuid,
    #[serde(deserialize_with = "canonical_uuid")]
    pub sid: Uuid,
    pub iat: i64,
    pub nbf: i64,
    pub exp: i64,
    #[serde(with = "positive_generation")]
    pub security_generation: i64,
}

impl fmt::Debug for AccountCsrfClaims {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AccountCsrfClaims([REDACTED])")
    }
}

/// Both variants require live Account and family checks. `Expired` permits no
/// fallback until that authority and a matching CSRF proof have been checked.
#[derive(Debug)]
pub enum AccountAccessVerification {
    Current(AccountAccessClaims),
    Expired(AccountAccessClaims),
}

impl JwtIssuer {
    pub fn issue_account_access_token(
        &self,
        input: AccountAccessTokenInput,
    ) -> Result<SignedAccountToken, AuthError> {
        require_issuer(&self.settings.issuer)?;
        validate_identity(
            input.account_id,
            input.session_id,
            input.security_generation,
        )?;
        if input.auth_time.unix_timestamp() < 0 || input.auth_time > input.issued_at {
            return Err(invalid_issuance());
        }
        let (iat, exp) = issuance_times(
            input.issued_at,
            input.family_expires_at,
            self.settings.access_token_ttl,
            ACCESS_MAX_SECONDS,
        )?;
        let claims = AccountAccessClaims {
            iss: self.settings.issuer.clone(),
            aud: ACCESS_AUDIENCE.to_owned(),
            token_kind: ACCESS_KIND.to_owned(),
            sub: input.account_id,
            sid: input.session_id,
            jti: Uuid::new_v4(),
            iat,
            nbf: iat,
            exp,
            security_generation: input.security_generation,
            auth_time: input.auth_time.unix_timestamp(),
            assurance: input.assurance,
        };
        encode(&Header::new(Algorithm::ES256), &claims, &self.encoding_key)
            .map(SignedAccountToken)
            .map_err(|_| signing_failed())
    }

    pub fn issue_account_csrf_token(
        &self,
        input: AccountCsrfTokenInput,
    ) -> Result<SignedAccountToken, AuthError> {
        require_issuer(&self.settings.issuer)?;
        validate_identity(
            input.account_id,
            input.session_id,
            input.security_generation,
        )?;
        let (iat, exp) = issuance_times(
            input.issued_at,
            input.family_expires_at,
            Duration::seconds(CSRF_MAX_SECONDS),
            CSRF_MAX_SECONDS,
        )?;
        let claims = AccountCsrfClaims {
            iss: self.settings.issuer.clone(),
            aud: CSRF_AUDIENCE.to_owned(),
            token_kind: CSRF_KIND.to_owned(),
            sub: input.account_id,
            sid: input.session_id,
            iat,
            nbf: iat,
            exp,
            security_generation: input.security_generation,
        };
        encode(&Header::new(Algorithm::ES256), &claims, &self.encoding_key)
            .map(SignedAccountToken)
            .map_err(|_| signing_failed())
    }
}

impl JwtVerifier {
    /// `now` must come from the trusted server clock. An error never authorizes
    /// refresh fallback; an expired result still needs all live authority checks.
    pub fn verify_account_access_token(
        &self,
        token: &str,
        now: OffsetDateTime,
    ) -> Result<AccountAccessVerification, AuthError> {
        let claims: AccountAccessClaims = self.decode_account_claims(token, ACCESS_AUDIENCE)?;
        if claims.token_kind != ACCESS_KIND || claims.auth_time < 0 || claims.auth_time > claims.iat
        {
            return Err(invalid_token());
        }
        validate_times(claims.iat, claims.nbf, claims.exp, now, ACCESS_MAX_SECONDS)?;
        if claims.exp > now.unix_timestamp() {
            Ok(AccountAccessVerification::Current(claims))
        } else {
            Ok(AccountAccessVerification::Expired(claims))
        }
    }

    /// `now` must come from the trusted server clock. Expiry is classified only
    /// after the signature and every other required claim check have succeeded.
    pub fn verify_account_csrf_token(
        &self,
        token: &str,
        now: OffsetDateTime,
    ) -> Result<AccountCsrfClaims, AuthError> {
        let claims: AccountCsrfClaims = self.decode_account_claims(token, CSRF_AUDIENCE)?;
        if claims.token_kind != CSRF_KIND {
            return Err(invalid_token());
        }
        validate_times(claims.iat, claims.nbf, claims.exp, now, CSRF_MAX_SECONDS)?;
        if claims.exp <= now.unix_timestamp() {
            return Err(AuthError::Jwt(ErrorKind::ExpiredSignature.into()));
        }
        Ok(claims)
    }

    fn decode_account_claims<T: de::DeserializeOwned>(
        &self,
        token: &str,
        audience: &str,
    ) -> Result<T, AuthError> {
        require_issuer(&self.settings.issuer)?;
        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_issuer(&[self.settings.issuer.as_str()]);
        validation.set_audience(&[audience]);
        validation.set_required_spec_claims(&["iss", "aud", "sub", "nbf", "exp"]);
        // Keep cryptographic/issuer/audience checks enabled. All time checks use
        // the explicit trusted clock after closed typed parsing, without grace.
        validation.validate_exp = false;
        validation.validate_nbf = false;
        validation.leeway = 0;
        // Decode directly into the closed struct so duplicate members survive
        // until serde rejects them. Discard all input-bearing library errors.
        decode::<T>(token, &self.decoding_key, &validation)
            .map(|decoded| decoded.claims)
            .map_err(|_| invalid_token())
    }
}

fn require_issuer(issuer: &str) -> Result<(), AuthError> {
    if issuer.is_empty() {
        return Err(AuthError::InvalidStoredData(
            "account token issuer must be configured".to_owned(),
        ));
    }
    Ok(())
}

fn validate_identity(account: Uuid, session: Uuid, generation: i64) -> Result<(), AuthError> {
    if account.is_nil() || session.is_nil() || generation <= 0 {
        return Err(invalid_issuance());
    }
    Ok(())
}

fn issuance_times(
    issued_at: OffsetDateTime,
    family_expires_at: OffsetDateTime,
    ttl: Duration,
    max_seconds: i64,
) -> Result<(i64, i64), AuthError> {
    let iat = issued_at.unix_timestamp();
    if iat < 0 || !ttl.is_positive() || family_expires_at <= issued_at {
        return Err(invalid_issuance());
    }
    // Two representable datetimes have a representable Duration difference.
    // Clamp to the family ceiling BEFORE adding even the protocol maximum: the
    // addition otherwise can overflow near the greatest representable date.
    let lifetime = ttl
        .min(Duration::seconds(max_seconds))
        .min(family_expires_at - issued_at);
    let exp = issued_at
        .checked_add(lifetime)
        .ok_or_else(invalid_issuance)?
        .unix_timestamp();
    if exp <= iat {
        return Err(invalid_issuance());
    }
    Ok((iat, exp))
}

fn validate_times(
    iat: i64,
    nbf: i64,
    exp: i64,
    now: OffsetDateTime,
    max_seconds: i64,
) -> Result<(), AuthError> {
    let now = now.unix_timestamp();
    if iat < 0 || nbf < 0 || exp < 0 || exp <= iat || nbf > exp || iat > now || nbf > now {
        return Err(invalid_token());
    }
    if exp.checked_sub(iat).ok_or_else(invalid_token)? > max_seconds {
        return Err(invalid_token());
    }
    Ok(())
}

fn canonical_uuid<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Uuid, D::Error> {
    let wire = String::deserialize(deserializer)?;
    let uuid =
        Uuid::parse_str(&wire).map_err(|_| de::Error::custom("invalid account token UUID"))?;
    if uuid.is_nil() || uuid.to_string() != wire {
        return Err(de::Error::custom("invalid account token UUID"));
    }
    Ok(uuid)
}

mod positive_generation {
    use serde::{Deserialize, Deserializer, Serializer, de, ser};

    pub fn serialize<S: Serializer>(value: &i64, serializer: S) -> Result<S::Ok, S::Error> {
        if *value <= 0 {
            return Err(ser::Error::custom("invalid account security generation"));
        }
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i64, D::Error> {
        let wire = String::deserialize(deserializer)?;
        let value: i64 = wire
            .parse()
            .map_err(|_| de::Error::custom("invalid account security generation"))?;
        if value <= 0 || value.to_string() != wire {
            return Err(de::Error::custom("invalid account security generation"));
        }
        Ok(value)
    }
}

fn invalid_issuance() -> AuthError {
    AuthError::InvalidStoredData("invalid account token issuance input".to_owned())
}

fn signing_failed() -> AuthError {
    AuthError::InvalidStoredData("account token signing failed".to_owned())
}

fn invalid_token() -> AuthError {
    AuthError::Jwt(ErrorKind::InvalidToken.into())
}
