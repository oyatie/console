use std::fmt;

use sqlx::PgPool;
use uuid::Uuid;

use crate::JwtVerifier;

/// The token verifier and mandatory Auth transport for current session checks.
///
/// Construction only binds existing dependencies. The composition root admits
/// the Auth connection; a reader never reconnects or substitutes a Business pool.
#[derive(Clone)]
pub struct SessionVerification {
    verifier: JwtVerifier,
    auth_database: PgPool,
}

impl SessionVerification {
    #[must_use]
    pub fn new(verifier: JwtVerifier, auth_database: PgPool) -> Self {
        Self {
            verifier,
            auth_database,
        }
    }

    /// Cryptographic and claim validation only. This does not authenticate a
    /// current session or produce a principal; callers must also check the
    /// verified subject through [`Self::legacy_subject_is_fenced`].
    #[must_use]
    pub fn token_verifier(&self) -> &JwtVerifier {
        &self.verifier
    }

    /// Read the strict Account presence projection through the bound Auth pool.
    /// Only a successful `false` permits a legacy subject. Missing/NULL results
    /// and unavailable transport remain errors, never an unfenced result.
    pub async fn legacy_subject_is_fenced(&self, subject: Uuid) -> Result<bool, sqlx::Error> {
        legacy_subject_is_fenced(&self.auth_database, subject).await
    }
}

impl fmt::Debug for SessionVerification {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SessionVerification")
            .finish_non_exhaustive()
    }
}

pub(crate) async fn legacy_subject_is_fenced(
    auth_pool: &PgPool,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT public.account_legacy_fenced_v1($1)")
        .bind(user_id)
        // rls-arming: ok narrow Account identity projection is global, not Company data
        .fetch_one(auth_pool)
        .await
}
