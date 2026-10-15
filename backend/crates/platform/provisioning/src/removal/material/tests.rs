// Supplemental pure regressions; these do not replace real HTTP/owner admission.
use super::{TenantRemovalError, UserId, Uuid, validate_force};
use sha2::{Digest, Sha256};
use std::borrow::Cow;
// Independent literal derivation: force framing guide SHA-256 8320d1035588bc33….
const FORCE_HEX: &str = "636f6e736f6c652e706c6174666f726d2e666f7263655f72656d6f7665000001000000000000000000000000000000010000000000000000000000000000006c0000000000000000000000000000001e0000000000000000000000000000000a000000000000000000000000000000140000000000000001";

fn fixture() -> Vec<u8> {
    (0..FORCE_HEX.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&FORCE_HEX[i..i + 2], 16).unwrap())
        .collect()
}
fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}
fn identities() -> (UserId, Uuid, Uuid) {
    (UserId::from_uuid(id(1)), id(108), id(30))
}
fn check(bytes: &[u8]) -> Result<(), TenantRemovalError> {
    let (actor, attempt, company) = identities();
    validate_force(bytes, actor, attempt, company)
}
fn invalid(result: Result<(), TenantRemovalError>) {
    assert!(matches!(result, Err(TenantRemovalError::InvalidMaterial)));
}

#[test]
fn force_literal_has_independent_length_digest_and_identity() {
    let bytes = fixture();
    assert_eq!(bytes.len(), 120);
    assert_eq!(
        Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        "7eb5a17e3b74d6f8b6a65272ab6459a1ca4fd937a3a1aa10a047f08bc3b1f356"
    );
    check(&bytes).unwrap();
}

#[test]
fn force_rejects_every_truncation_append_and_prefix_change() {
    let bytes = fixture();
    for length in 0..120 {
        invalid(check(&bytes[..length]));
    }
    let mut appended = bytes.clone();
    appended.push(0);
    invalid(check(&appended));
    for offset in 0..32 {
        let mut changed = bytes.clone();
        changed[offset] ^= 1;
        invalid(check(&changed));
    }
    let mut future_version = bytes;
    future_version[31] = 2;
    invalid(check(&future_version));
}

#[test]
fn force_rejects_nil_in_all_five_uuid_slots_even_matching_selectors() {
    for offset in [32, 48, 64, 80, 96] {
        let mut bytes = fixture();
        bytes[offset..offset + 16].fill(0);
        invalid(check(&bytes));
        let (mut actor, mut attempt, mut company) = identities();
        match offset {
            32 => actor = UserId::from_uuid(Uuid::nil()),
            48 => attempt = Uuid::nil(),
            64 => company = Uuid::nil(),
            _ => continue,
        }
        invalid(validate_force(&bytes, actor, attempt, company));
    }
}

#[test]
fn force_rejects_actor_attempt_company_mismatch_in_bytes_or_expectation() {
    for offset in [32, 48, 64] {
        let mut bytes = fixture();
        bytes[offset + 15] ^= 1;
        invalid(check(&bytes));
    }
    let (actor, attempt, company) = identities();
    let other = id(9);
    let other_actor = UserId::from_uuid(other);
    invalid(validate_force(&fixture(), other_actor, attempt, company));
    invalid(validate_force(&fixture(), actor, other, company));
    invalid(validate_force(&fixture(), actor, attempt, other));
}

#[test]
fn force_accepts_positive_signed_revision_range_and_rejects_sign_overflow() {
    for revision in [1_i64, 2, i64::MAX] {
        let mut bytes = fixture();
        bytes[112..120].copy_from_slice(&revision.to_be_bytes());
        check(&bytes).unwrap();
    }
    for revision in [0_i64, -1, i64::MIN] {
        let mut bytes = fixture();
        bytes[112..120].copy_from_slice(&revision.to_be_bytes());
        invalid(check(&bytes));
    }
}

#[test]
fn force_does_not_accept_the_ordinary_kind8_grammar() {
    let ordinary = concat!(
        "636f6e736f6c652e706c6174666f726d2e746f706f6c6f6779000001",
        "000000000000000000000000000000010000000000000000000000000000006c0801",
        "0000000000000000000000000000000a00000000000000000000000000000014",
        "00000000000000010000000000000000000000000000001e",
    );
    let bytes: Vec<u8> = (0..ordinary.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&ordinary[i..i + 2], 16).unwrap())
        .collect();
    assert_eq!(bytes.len(), 118);
    invalid(check(&bytes));
}

// Error envelopes isolate classification; no database owner or authority is mocked.
#[derive(Debug)]
struct Envelope(Option<&'static str>, &'static str);
impl std::fmt::Display for Envelope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.1)
    }
}
impl std::error::Error for Envelope {}
impl sqlx::error::DatabaseError for Envelope {
    fn message(&self) -> &str {
        self.1
    }
    fn code(&self) -> Option<Cow<'_, str>> {
        self.0.map(Cow::Borrowed)
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
fn mapped(code: Option<&'static str>, message: &'static str) -> TenantRemovalError {
    TenantRemovalError::port(sqlx::Error::Database(Box::new(Envelope(code, message))))
}

#[test]
fn port_classifies_only_two_exact_source_stage_pairs() {
    let codes = [
        "28000", "42501", "40001", "P0001", "55P03", "40P01", "57014", "XX000",
    ];
    for code in codes.map(Some).into_iter().chain([None]) {
        for message in [
            "platform_removal.source_refused",
            "platform_removal.policy_refused",
            "platform_removal.source_refused\n",
            "platform_removal.policy_refused.extra",
            "platform_topology.plan_changed",
            "platform_force.command_input_conflict",
            "platform_force.policy_denied",
            "unknown",
        ] {
            let result = mapped(code, message);
            match (code, message) {
                (Some("28000"), "platform_removal.source_refused") => {
                    assert!(matches!(result, TenantRemovalError::Unauthorized))
                }
                (Some("42501"), "platform_removal.policy_refused") => {
                    assert!(matches!(result, TenantRemovalError::Forbidden))
                }
                _ => {
                    let TenantRemovalError::Storage(error) = result else {
                        panic!("unexpected classification for {code:?}/{message}");
                    };
                    let stored = error.as_database_error().unwrap();
                    assert_eq!(stored.code().as_deref(), code);
                    assert_eq!(stored.message(), message);
                }
            }
        }
    }
}

#[test]
fn port_preserves_non_database_errors_as_storage_failures() {
    assert!(matches!(
        TenantRemovalError::port(sqlx::Error::RowNotFound),
        TenantRemovalError::Storage(sqlx::Error::RowNotFound)
    ));
    assert!(matches!(
        TenantRemovalError::port(sqlx::Error::PoolTimedOut),
        TenantRemovalError::Storage(sqlx::Error::PoolTimedOut)
    ));
}
