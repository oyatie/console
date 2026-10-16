//! Included by legacy.rs under cfg(test), after the mounted HTTP owner is RED.
//! Isolated tuple/time/error controls; this file is not a credential constructor.
use super::*;
use crate::AccessClaims;
use console_kernel_core::ErrorKind;
use serde_json::json;
use std::borrow::Cow;
use time::{Duration, OffsetDateTime};

type ReadError = LegacySelfPasskeyReadError;

fn case() -> (AccessClaims, LegacySelfReadFamily, Duration, OffsetDateTime) {
    let now =
        OffsetDateTime::from_unix_timestamp(1_790_000_020).unwrap() + Duration::milliseconds(500);
    let created = now - Duration::seconds(10) + Duration::microseconds(123);
    let subject = Uuid::from_u128(0x11111111111141118111111111111111);
    let home = *OrgId::knl().as_uuid();
    let family = Uuid::from_u128(0x22222222222242228222222222222222);
    let claims = serde_json::from_value(json!({
        "iss":"unit-self-read", "aud":"unit-self-read", "sub":subject.to_string(),
        "org":home.to_string(), "iat":created.unix_timestamp(), "nbf":created.unix_timestamp(),
        "exp":now.unix_timestamp()+300, "jti":"unit-correlation", "roles":["MECHANIC"],
        "branches":[], "alg":"ES256", "legacy_session":{
            "version":1,"family_id":family,"home_org":home,"kind":"direct"
        }
    }))
    .unwrap();
    let stored = LegacySelfReadFamily {
        id: family,
        user_id: subject,
        org_id: Some(home),
        protocol: "LEGACY_COMPANY".to_owned(),
        created_at: created,
        revoked_at: None,
        account_security_generation: None,
        auth_time: None,
        assurance: None,
    };
    (claims, stored, Duration::minutes(10), now)
}

fn unauthorized(result: Result<(), ReadError>) {
    assert!(
        matches!(result, Err(ReadError::Unauthorized)),
        "known refusal must be401 category"
    );
}
fn unavailable(result: Result<(), ReadError>) {
    assert!(
        matches!(result, Err(ReadError::Unavailable)),
        "invalid storage/policy must be503 category"
    );
}

#[test]
fn exact_tuple_rejects_each_component_and_native_metadata_independently() {
    let (claims, family, ttl, now) = case();
    assert!(validate_legacy_self_read_family(&claims, &family, ttl, now).is_ok());
    let changes: [fn(&mut LegacySelfReadFamily); 10] = [
        |f| f.id = Uuid::new_v4(),
        |f| f.user_id = Uuid::new_v4(),
        |f| f.org_id = None,
        |f| f.org_id = Some(Uuid::new_v4()),
        |f| f.protocol = "ACCOUNT_V1".to_owned(),
        |f| f.protocol = "unknown".to_owned(),
        |f| f.revoked_at = Some(f.created_at),
        |f| f.account_security_generation = Some(1),
        |f| f.auth_time = Some(f.created_at),
        |f| f.assurance = Some("PASSKEY_PRIMARY".to_owned()),
    ];
    for change in changes {
        let (claims, mut family, ttl, now) = case();
        change(&mut family);
        unauthorized(validate_legacy_self_read_family(&claims, &family, ttl, now));
    }
    // ACCOUNT_V1 differs ONLY in protocol; unlike HTTP native fixtures, this
    // isolates that guard without counterfeit rows or weakened DB constraints.
}

#[test]
fn signed_correlation_cannot_select_another_subject_home_or_family() {
    let changes: [fn(&mut AccessClaims); 6] = [
        |c| c.sub = Uuid::new_v4().to_string(),
        |c| c.org = Uuid::new_v4().to_string(),
        |c| c.sub = "invalid-subject".to_owned(),
        |c| c.org = "invalid-home".to_owned(),
        |c| c.legacy_session.as_mut().unwrap().family_id = Uuid::new_v4(),
        |c| c.legacy_session.as_mut().unwrap().home_org = Uuid::new_v4(),
    ];
    for change in changes {
        let (mut claims, family, ttl, now) = case();
        change(&mut claims);
        unauthorized(validate_legacy_self_read_family(&claims, &family, ttl, now));
    }
}

#[test]
fn same_second_issuance_is_valid_but_earlier_whole_second_is_not() {
    let (mut claims, family, ttl, now) = case();
    assert!(family.created_at.nanosecond() != 0);
    assert_eq!(family.created_at.unix_timestamp(), claims.iat);
    assert!(OffsetDateTime::from_unix_timestamp(claims.iat).unwrap() < family.created_at);
    assert!(validate_legacy_self_read_family(&claims, &family, ttl, now).is_ok());
    claims.iat -= 1;
    unauthorized(validate_legacy_self_read_family(&claims, &family, ttl, now));
}

#[test]
fn current_family_and_signed_deadlines_are_strict_and_neither_extends_the_other() {
    let (mut claims, family, ttl, _) = case();
    let deadline = family.created_at + ttl;
    claims.exp = deadline.unix_timestamp() + 300;
    assert!(
        validate_legacy_self_read_family(
            &claims,
            &family,
            ttl,
            deadline - Duration::microseconds(1)
        )
        .is_ok()
    );
    unauthorized(validate_legacy_self_read_family(
        &claims, &family, ttl, deadline,
    ));
    unauthorized(validate_legacy_self_read_family(
        &claims,
        &family,
        ttl,
        deadline + Duration::microseconds(1),
    ));
    let (claims, family, ttl, _) = case();
    let expiry = OffsetDateTime::from_unix_timestamp(claims.exp).unwrap();
    assert!(expiry < family.created_at + ttl);
    assert!(
        validate_legacy_self_read_family(&claims, &family, ttl, expiry - Duration::microseconds(1))
            .is_ok()
    );
    unauthorized(validate_legacy_self_read_family(
        &claims, &family, ttl, expiry,
    ));
    unauthorized(validate_legacy_self_read_family(
        &claims,
        &family,
        ttl,
        expiry + Duration::microseconds(1),
    ));
}

#[test]
fn bound_signed_times_reject_future_bad_order_and_unrepresentable_values() {
    let changes: [fn(&mut AccessClaims, OffsetDateTime); 9] = [
        |c, now| c.iat = now.unix_timestamp() + 1,
        |c, now| c.nbf = now.unix_timestamp() + 1,
        |c, _| c.nbf = c.iat - 1,
        |c, _| c.nbf = c.exp,
        |c, _| c.nbf = c.exp + 1,
        |c, _| c.iat = i64::MIN,
        |c, _| c.nbf = i64::MAX,
        |c, _| c.exp = i64::MAX,
        |c, _| c.exp = i64::MIN,
    ];
    for change in changes {
        let (mut claims, family, ttl, now) = case();
        change(&mut claims, now);
        unauthorized(validate_legacy_self_read_family(&claims, &family, ttl, now));
    }
    let (mut claims, mut family, ttl, now) = case();
    family.created_at = now + Duration::microseconds(1);
    claims.iat = now.unix_timestamp();
    claims.nbf = claims.iat;
    unauthorized(validate_legacy_self_read_family(&claims, &family, ttl, now));
}

#[test]
fn invalid_or_overflowing_server_family_policy_is_unavailable() {
    let (claims, family, _, now) = case();
    for ttl in [Duration::ZERO, Duration::seconds(-1), Duration::MAX] {
        unavailable(validate_legacy_self_read_family(&claims, &family, ttl, now));
    }
}

// Synthetic error envelopes isolate mapper semantics ONLY. They never replace
// a database function, fixture authority, query, pool, or actual HTTP oracle.
#[derive(Debug)]
struct StoredError {
    code: &'static str,
    message: &'static str,
}
impl std::fmt::Display for StoredError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message)
    }
}
impl std::error::Error for StoredError {}
impl sqlx::error::DatabaseError for StoredError {
    fn message(&self) -> &str {
        self.message
    }
    fn code(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed(self.code))
    }
    fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        self
    }
    fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
        self
    }
    fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
        self
    }
    fn kind(&self) -> sqlx::error::ErrorKind {
        sqlx::error::ErrorKind::Other
    }
}
fn storage(code: &'static str, message: &'static str) -> sqlx::Error {
    sqlx::Error::Database(Box::new(StoredError { code, message }))
}
fn mapped_unauthorized(error: AuthError) {
    assert!(matches!(
        legacy_self_read_guard_error(error),
        ReadError::Unauthorized
    ));
}
fn mapped_unavailable(error: AuthError) {
    assert!(matches!(
        legacy_self_read_guard_error(error),
        ReadError::Unavailable
    ));
}

#[test]
fn guard_mapper_accepts_only_exact_known_storage_code_and_message_pairs() {
    for message in [
        "auth_legacy.company_not_found",
        "account_company_deactivation.subject_not_found",
        "auth_legacy.subject_not_found",
    ] {
        mapped_unauthorized(AuthError::Sqlx(storage("P0002", message)));
        for code in ["P0001", "42501", "28000", "40001", "XX000"] {
            mapped_unavailable(AuthError::Sqlx(storage(code, message)));
        }
        // Matching prose from a different error origin is not the guard path.
        mapped_unavailable(AuthError::InvalidStoredData(message.to_owned()));
        mapped_unavailable(AuthError::Kernel(KernelError::not_found(message)));
        mapped_unavailable(AuthError::Db(console_platform_db::DbError::Sqlx(storage(
            "P0002", message,
        ))));
    }
    for message in [
        "unknown",
        "auth_legacy.subject_not_found extra",
        "auth_legacy.subject_inactive",
        "auth_legacy.subject_has_no_roles",
        "auth_legacy.fenced",
        "account.root_missing",
    ] {
        mapped_unavailable(AuthError::Sqlx(storage("P0002", message)));
    }
}

#[test]
fn guard_mapper_does_not_turn_outage_null_or_unexpected_origin_into_revocation() {
    const FENCE: &str = "legacy credential operation is unavailable";
    const INACTIVE: &str = "비활성화된 사용자는 인증 정보를 변경할 수 없습니다.";
    mapped_unauthorized(AuthError::InvalidStoredData(FENCE.to_owned()));
    mapped_unauthorized(AuthError::Kernel(KernelError::conflict(INACTIVE)));
    mapped_unavailable(AuthError::InvalidStoredData(format!("{FENCE} extra")));
    mapped_unavailable(AuthError::Kernel(KernelError::conflict("unknown conflict")));
    for kind in [
        ErrorKind::Validation,
        ErrorKind::NotFound,
        ErrorKind::Forbidden,
        ErrorKind::InvalidTransition,
        ErrorKind::Internal,
    ] {
        mapped_unavailable(AuthError::Kernel(KernelError::new(kind, INACTIVE)));
    }
    mapped_unavailable(AuthError::InvalidStoredData(INACTIVE.to_owned()));
    mapped_unavailable(AuthError::Sqlx(storage("P0002", FENCE)));
    mapped_unavailable(AuthError::Sqlx(storage("P0002", INACTIVE)));
    for error in [
        sqlx::Error::PoolClosed,
        sqlx::Error::PoolTimedOut,
        sqlx::Error::RowNotFound,
        sqlx::Error::ColumnDecode {
            index: "fenced".to_owned(),
            source: Box::new(sqlx::error::UnexpectedNullError),
        },
    ] {
        mapped_unavailable(AuthError::Sqlx(error));
    }
    for code in ["42883", "42P01", "42501", "40P01", "57014", "40001"] {
        mapped_unavailable(AuthError::Sqlx(storage(code, "operation unavailable")));
    }
    // The owner handles verifier failures separately before the guarded storage
    // mapper. A JWT-shaped unexpected guard error still means unavailable.
    mapped_unavailable(AuthError::Jwt(
        jsonwebtoken::errors::ErrorKind::InvalidToken.into(),
    ));
}
