//! Private source proposal for the actual Identity application codec boundary.
//! Register as business::tests only after exact source/API admission. No stand-in
//! implementation or missing import is semantic RED; no DB or Auth claims.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::{
    NativeBusinessOperationV1 as Operation, NativeCompanyBusinessCommandV1 as Command,
    PolicyAssignmentExpectationV1 as Expectation,
};
use console_identity_domain::AccountId;
use console_kernel_core::{KernelError, OrgId};
use time::{OffsetDateTime, UtcOffset};
use uuid::Uuid;

const ACTOR: &str = "11111111-1111-4111-8111-111111111111";
const COMPANY: &str = "33333333-3333-4333-8333-333333333333";
const COMMAND: &str = "22222222-2222-4222-8222-222222222222";
const RECIPIENT: &str = "44444444-4444-4444-8444-444444444444";
const ASSIGNMENT: &str = "55555555-5555-4555-8555-555555555555";
const EXPIRY: i64 = 1_790_607_600_000_000;
const MIN_EXPIRY: i64 = -62_135_596_800_000_000;
const MAX_EXPIRY: i64 = 253_402_268_340_000_000;
const MINUTE: i64 = 60_000_000;
fn uuid(s: &str) -> Uuid {
    Uuid::parse_str(s).unwrap()
}
fn account(s: &str) -> AccountId {
    AccountId::from_uuid(uuid(s)).unwrap()
}
fn company() -> OrgId {
    OrgId::from_uuid(uuid(COMPANY))
}
fn instant(micros: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(micros) * 1_000).unwrap()
}
fn expectation() -> Expectation {
    Expectation {
        role_revision: 1,
        assignment_id: uuid(ASSIGNMENT),
        assignment_revision: 9,
    }
}
fn grant(expiry: OffsetDateTime, witness: Option<Expectation>) -> Result<Command, KernelError> {
    Command::grant(
        uuid(COMMAND),
        company(),
        7,
        account(RECIPIENT),
        witness,
        expiry,
    )
}
fn unhex(s: &str) -> Vec<u8> {
    assert_eq!(s.len() % 2, 0);
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
fn rejected<T: std::fmt::Debug>(result: Result<T, KernelError>) {
    assert_eq!(
        result.unwrap_err(),
        KernelError::validation("invalid Company business policy input")
    );
}

// Literal vectors independently verified field-by-field, not owner-generated.
const VECTORS: [&str; 4] = [
    "636f6e736f6c652e636f6d70616e792e627573696e6573732d706f6c696379000001111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000000707781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd01",
    "636f6e736f6c652e636f6d70616e792e627573696e6573732d706f6c696379000001111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000000707781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd02444444444444444484444444444444440000065c8c51ee1c00",
    "636f6e736f6c652e636f6d70616e792e627573696e6573732d706f6c696379000001111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000000707781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "636f6e736f6c652e636f6d70616e792e627573696e6573732d706f6c696379000001111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000000707781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd030000000000000001555555555555455585555555555555550000000000000009",
];
fn commands() -> [Command; 4] {
    [
        Command::install(uuid(COMMAND), company(), 7).unwrap(),
        grant(instant(EXPIRY), None).unwrap(),
        grant(instant(EXPIRY), Some(expectation())).unwrap(),
        Command::revoke(uuid(COMMAND), company(), 7, expectation()).unwrap(),
    ]
}

#[test]
fn all_four_independent_vectors_match_constructor_decoder_and_supplied_accessors() {
    for (i, command) in commands().into_iter().enumerate() {
        let bytes = unhex(VECTORS[i]);
        assert_eq!(bytes.len(), [123, 148, 180, 155][i]);
        assert_eq!(command.encode(account(ACTOR)), bytes);
        let (actor, decoded) = Command::decode(&bytes).unwrap();
        assert_eq!(actor, account(ACTOR));
        assert_eq!(decoded, command);
        assert_eq!(decoded.encode(actor), bytes);
        assert_eq!(decoded.command_id(), uuid(COMMAND));
        assert_eq!(decoded.company(), company());
        assert_eq!(decoded.expected_company_epoch(), 7);
        assert_eq!(
            decoded.operation(),
            [
                Operation::Install,
                Operation::Grant,
                Operation::Grant,
                Operation::Revoke
            ][i]
        );
        assert_eq!(
            decoded.recipient_account_id(),
            if i == 1 || i == 2 {
                Some(account(RECIPIENT))
            } else {
                None
            }
        );
        assert_eq!(
            decoded.assignment_expectation(),
            if i >= 2 { Some(expectation()) } else { None }
        );
        assert_eq!(
            decoded.expires_at(),
            if i == 1 || i == 2 {
                Some(instant(EXPIRY))
            } else {
                None
            }
        );
    }
}

#[test]
fn every_truncation_and_any_extra_suffix_is_rejected_with_fixed_error() {
    for hex in VECTORS {
        let bytes = unhex(hex);
        for length in 0..bytes.len() {
            rejected(Command::decode(&bytes[..length]));
        }
        for tail in [&[0][..], &[1, 2, 3][..], &b" "[..]] {
            let mut bad = bytes.clone();
            bad.extend_from_slice(tail);
            rejected(Command::decode(&bad));
        }
    }
}

#[test]
fn fixed_prefix_version_manifest_operation_and_witness_are_closed() {
    for hex in VECTORS {
        let bytes = unhex(hex);
        for at in (0..34).chain(90..122) {
            let mut bad = bytes.clone();
            bad[at] ^= 1;
            rejected(Command::decode(&bad));
        }
        for operation in [0, 4, 255] {
            let mut bad = bytes.clone();
            bad[122] = operation;
            rejected(Command::decode(&bad));
        }
    }
    for i in [1, 2] {
        for tag in [2, 255, if i == 1 { 1 } else { 0 }] {
            let mut bad = unhex(VECTORS[i]);
            bad[139] = tag;
            rejected(Command::decode(&bad));
        }
    }
    // Changing between known operations cannot reinterpret another suffix.
    for (i, operation) in [
        (0, 2),
        (0, 3),
        (1, 1),
        (1, 3),
        (2, 1),
        (2, 3),
        (3, 1),
        (3, 2),
    ] {
        let mut bad = unhex(VECTORS[i]);
        bad[122] = operation;
        rejected(Command::decode(&bad));
    }
}

#[test]
fn nil_identifiers_and_platform_company_reject_but_non_v4_ids_remain_valid() {
    for (i, offsets) in [
        (0, vec![34, 50, 66]),
        (1, vec![34, 50, 66, 123]),
        (2, vec![34, 50, 66, 123, 148]),
        (3, vec![34, 50, 66, 131]),
    ] {
        for at in offsets {
            let mut bad = unhex(VECTORS[i]);
            bad[at..at + 16].fill(0);
            rejected(Command::decode(&bad));
        }
        let mut bad = unhex(VECTORS[i]);
        bad[50..66].copy_from_slice(OrgId::platform().as_uuid().as_bytes());
        rejected(Command::decode(&bad));
    }
    for company in [OrgId::from_uuid(Uuid::nil()), OrgId::platform()] {
        rejected(Command::install(uuid(COMMAND), company, 7));
        rejected(Command::grant(
            uuid(COMMAND),
            company,
            7,
            account(RECIPIENT),
            None,
            instant(EXPIRY),
        ));
        rejected(Command::revoke(uuid(COMMAND), company, 7, expectation()));
    }
    rejected(Command::install(Uuid::nil(), company(), 7));
    rejected(Command::grant(
        Uuid::nil(),
        company(),
        7,
        account(RECIPIENT),
        None,
        instant(EXPIRY),
    ));
    rejected(Command::revoke(Uuid::nil(), company(), 7, expectation()));
    let bad = Expectation {
        assignment_id: Uuid::nil(),
        ..expectation()
    };
    rejected(grant(instant(EXPIRY), Some(bad)));
    rejected(Command::revoke(uuid(COMMAND), company(), 7, bad));
    // Existing AccountId rejects nil before the infallible encoder can be called.
    assert!(AccountId::from_uuid(Uuid::nil()).is_err());
    let v7 = uuid("01234567-89ab-7def-8123-456789abcdef");
    let selected = OrgId::from_uuid(v7);
    let actor = AccountId::from_uuid(v7).unwrap();
    let command = Command::grant(
        v7,
        selected,
        7,
        actor,
        Some(Expectation {
            assignment_id: v7,
            ..expectation()
        }),
        instant(EXPIRY),
    )
    .unwrap();
    assert_eq!(
        Command::decode(&command.encode(actor)).unwrap(),
        (actor, command)
    );
}

#[test]
fn positive_signed_i64_limits_and_permanent_role_one_are_enforced() {
    for revision in [0, i64::MAX as u64 + 1, u64::MAX] {
        rejected(Command::install(uuid(COMMAND), company(), revision));
        rejected(Command::grant(
            uuid(COMMAND),
            company(),
            revision,
            account(RECIPIENT),
            None,
            instant(EXPIRY),
        ));
        rejected(Command::revoke(
            uuid(COMMAND),
            company(),
            revision,
            expectation(),
        ));
        let e = Expectation {
            assignment_revision: revision,
            ..expectation()
        };
        rejected(grant(instant(EXPIRY), Some(e)));
        rejected(Command::revoke(uuid(COMMAND), company(), 7, e));
    }
    for revision in [0, 2, i64::MAX as u64, u64::MAX] {
        let e = Expectation {
            role_revision: revision,
            ..expectation()
        };
        rejected(grant(instant(EXPIRY), Some(e)));
        rejected(Command::revoke(uuid(COMMAND), company(), 7, e));
    }
    for hex in VECTORS {
        for revision in [0i64, -1, i64::MIN] {
            let mut bad = unhex(hex);
            bad[82..90].copy_from_slice(&revision.to_be_bytes());
            rejected(Command::decode(&bad));
        }
    }
    for (i, role, assignment) in [(2, 140, 164), (3, 123, 147)] {
        for revision in [0i64, -1, i64::MIN] {
            let mut bad = unhex(VECTORS[i]);
            bad[assignment..assignment + 8].copy_from_slice(&revision.to_be_bytes());
            rejected(Command::decode(&bad));
        }
        for revision in [0i64, -1, 2, i64::MAX] {
            let mut bad = unhex(VECTORS[i]);
            bad[role..role + 8].copy_from_slice(&revision.to_be_bytes());
            rejected(Command::decode(&bad));
        }
    }
    for revision in [1, i64::MAX as u64] {
        let e = Expectation {
            assignment_revision: revision,
            ..expectation()
        };
        let c = Command::grant(
            uuid(COMMAND),
            company(),
            revision,
            account(RECIPIENT),
            Some(e),
            instant(EXPIRY),
        )
        .unwrap();
        assert_eq!(Command::decode(&c.encode(account(ACTOR))).unwrap().1, c);
    }
}

#[test]
fn exact_time_range_whole_minutes_and_historical_decode_do_not_round_or_use_now() {
    for micros in [MIN_EXPIRY, -MINUTE, 0, EXPIRY, MAX_EXPIRY] {
        for witness in [None, Some(expectation())] {
            let command = grant(instant(micros), witness).unwrap();
            let bytes = command.encode(account(ACTOR));
            assert_eq!(&bytes[bytes.len() - 8..], &micros.to_be_bytes());
            assert_eq!(
                Command::decode(&bytes).unwrap().1.expires_at(),
                Some(instant(micros))
            );
        }
    }
    for micros in [
        MIN_EXPIRY - MINUTE,
        MAX_EXPIRY + MINUTE,
        EXPIRY - 1,
        EXPIRY + 1,
        i64::MIN,
        i64::MAX,
    ] {
        for (i, at) in [(1, 140), (2, 172)] {
            let mut bad = unhex(VECTORS[i]);
            bad[at..at + 8].copy_from_slice(&micros.to_be_bytes());
            rejected(Command::decode(&bad));
        }
    }
    for nanos in [
        i128::from(EXPIRY) * 1000 - 1,
        i128::from(EXPIRY) * 1000 + 1,
        i128::from(EXPIRY + 1) * 1000,
        i128::from(MIN_EXPIRY - MINUTE) * 1000,
        i128::from(MAX_EXPIRY + MINUTE) * 1000,
    ] {
        let time = OffsetDateTime::from_unix_timestamp_nanos(nanos).unwrap();
        rejected(grant(time, None));
        rejected(grant(time, Some(expectation())));
    }
    // Outside time's representable range cannot be a typed input; decoder still
    // rejects its complete i64 wire value above, without casts/panic/rounding.
    assert!(OffsetDateTime::from_unix_timestamp_nanos(i128::from(i64::MAX) * 1000).is_err());
}

#[test]
fn korea_offset_normalizes_to_utc_without_losing_original_instant() {
    let korea = UtcOffset::from_hms(9, 0, 0).unwrap();
    let local = instant(EXPIRY).to_offset(korea);
    assert_eq!(
        (
            local.year(),
            u8::from(local.month()),
            local.day(),
            local.hour(),
            local.minute()
        ),
        (2026, 9, 29, 0, 0)
    );
    let command = grant(local, None).unwrap();
    assert_eq!(command.encode(account(ACTOR)), unhex(VECTORS[1]));
    assert_eq!(command.expires_at().unwrap().offset(), UtcOffset::UTC);
    // Earliest supported Korean local minute is0001-01-01T09:00.
    let first = instant(MIN_EXPIRY).to_offset(korea);
    assert_eq!(
        (first.year(), first.month(), first.day(), first.hour()),
        (1, time::Month::January, 1, 9)
    );
    assert_eq!(
        grant(first, None).unwrap().expires_at(),
        Some(instant(MIN_EXPIRY))
    );
    // The upper bound is the last four-digit Korean local minute,14:59UTC.
    let last = instant(MAX_EXPIRY).to_offset(korea);
    assert_eq!(
        (
            last.year(),
            last.month(),
            last.day(),
            last.hour(),
            last.minute()
        ),
        (9999, time::Month::December, 31, 23, 59)
    );
    assert_eq!(
        grant(last, None).unwrap().expires_at(),
        Some(instant(MAX_EXPIRY))
    );
}

#[test]
fn every_changed_supplied_input_is_bound_and_namespace_swaps_do_not_authenticate() {
    let original = commands()[2].clone();
    let bytes = original.encode(account(ACTOR));
    let other = uuid("66666666-6666-4666-8666-666666666666");
    let other_account = AccountId::from_uuid(other).unwrap();
    assert_ne!(original.encode(other_account), bytes);
    let cases = [
        Command::grant(
            other,
            company(),
            7,
            account(RECIPIENT),
            Some(expectation()),
            instant(EXPIRY),
        )
        .unwrap(),
        Command::grant(
            uuid(COMMAND),
            OrgId::from_uuid(other),
            7,
            account(RECIPIENT),
            Some(expectation()),
            instant(EXPIRY),
        )
        .unwrap(),
        Command::grant(
            uuid(COMMAND),
            company(),
            8,
            account(RECIPIENT),
            Some(expectation()),
            instant(EXPIRY),
        )
        .unwrap(),
        Command::grant(
            uuid(COMMAND),
            company(),
            7,
            other_account,
            Some(expectation()),
            instant(EXPIRY),
        )
        .unwrap(),
        grant(
            instant(EXPIRY),
            Some(Expectation {
                assignment_id: other,
                ..expectation()
            }),
        )
        .unwrap(),
        grant(
            instant(EXPIRY),
            Some(Expectation {
                assignment_revision: 10,
                ..expectation()
            }),
        )
        .unwrap(),
        grant(instant(EXPIRY + MINUTE), Some(expectation())).unwrap(),
        grant(instant(EXPIRY), None).unwrap(),
    ];
    for changed in cases {
        let encoded = changed.encode(account(ACTOR));
        assert_ne!(encoded, bytes);
        assert_eq!(Command::decode(&encoded).unwrap().1, changed);
    }
    // UUID selector swaps are different valid input, not a codec authentication
    // failure. Retained execution/recovery must compare actual authorized keys.
    for (left, right) in [(34, 50), (34, 66), (50, 66)] {
        let mut changed = bytes.clone();
        for i in 0..16 {
            changed.swap(left + i, right + i);
        }
        let (actor, c) = Command::decode(&changed).unwrap();
        assert_ne!(changed, bytes);
        assert_eq!(c.encode(actor), changed);
        assert!(
            actor != account(ACTOR) || c.company() != company() || c.command_id() != uuid(COMMAND)
        );
    }
}
