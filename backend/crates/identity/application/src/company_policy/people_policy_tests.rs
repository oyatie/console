//! Exact native People policy protocol. These tests do not grant authority.
use super::business::{
    NativeBusinessOperationV1 as Operation, NativeCompanyBusinessCommandV1 as Payroll,
    PolicyAssignmentExpectationV1 as Expectation,
};
use super::people_business::{
    DirectoryActionV1 as Action, MANIFEST, NativePeoplePolicyCommandV1 as Command,
};
use console_identity_domain::AccountId;
use console_kernel_core::{KernelError, OrgId};
use time::OffsetDateTime;
use uuid::Uuid;

const ACTOR: &str = "11111111-1111-4111-8111-111111111111";
const COMPANY: &str = "33333333-3333-4333-8333-333333333333";
const COMMAND: &str = "22222222-2222-4222-8222-222222222222";
const RECIPIENT: &str = "44444444-4444-4444-8444-444444444444";
const ASSIGNMENT: &str = "55555555-5555-4555-8555-555555555555";
const EXPIRY: i64 = 1_790_607_600_000_000;
fn uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).unwrap()
}
fn account(value: &str) -> AccountId {
    AccountId::from_uuid(uuid(value)).unwrap()
}
fn company() -> OrgId {
    OrgId::from_uuid(uuid(COMPANY))
}
fn instant(micros: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(micros) * 1_000).unwrap()
}
fn witness() -> Expectation {
    Expectation {
        role_revision: 1,
        assignment_id: uuid(ASSIGNMENT),
        assignment_revision: 9,
    }
}
fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn rejected<T: std::fmt::Debug>(value: Result<T, KernelError>) {
    assert_eq!(
        value.unwrap_err(),
        KernelError::validation("invalid Company business policy input")
    );
}
const VECTORS: [&str; 7] = [
    "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e01",
    "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e0201444444444444444484444444444444440000065c8c51ee1c00",
    "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e02014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e03010000000000000001555555555555455585555555555555550000000000000009",
    "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e0202444444444444444484444444444444440000065c8c51ee1c00",
    "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e02024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e03020000000000000001555555555555455585555555555555550000000000000009",
];
fn commands() -> Vec<Command> {
    let mut result = vec![Command::install(uuid(COMMAND), company(), 7).unwrap()];
    for action in [Action::Read, Action::Create] {
        result.push(
            Command::grant(
                uuid(COMMAND),
                company(),
                7,
                action,
                account(RECIPIENT),
                None,
                instant(EXPIRY),
            )
            .unwrap(),
        );
        result.push(
            Command::grant(
                uuid(COMMAND),
                company(),
                7,
                action,
                account(RECIPIENT),
                Some(witness()),
                instant(EXPIRY),
            )
            .unwrap(),
        );
        result.push(Command::revoke(uuid(COMMAND), company(), 7, action, witness()).unwrap());
    }
    result
}
#[test]
fn independent_people_vectors_bind_operation_action_and_all_selectors() {
    assert_eq!(
        MANIFEST.as_slice(),
        unhex("591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e")
    );
    assert_eq!(Action::Read.as_str(), "people.directory.read");
    assert_eq!(Action::Create.as_str(), "people.directory.create");
    for (i, command) in commands().into_iter().enumerate() {
        let bytes = unhex(VECTORS[i]);
        assert_eq!(bytes.len(), [121, 147, 179, 154, 147, 179, 154][i]);
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
                Operation::Revoke,
                Operation::Grant,
                Operation::Grant,
                Operation::Revoke
            ][i]
        );
        assert_eq!(
            decoded.action(),
            if i == 0 {
                None
            } else if i < 4 {
                Some(Action::Read)
            } else {
                Some(Action::Create)
            }
        );
        assert_eq!(
            decoded.recipient_account_id(),
            if [1, 2, 4, 5].contains(&i) {
                Some(account(RECIPIENT))
            } else {
                None
            }
        );
        assert_eq!(
            decoded.assignment_expectation(),
            if [2, 3, 5, 6].contains(&i) {
                Some(witness())
            } else {
                None
            }
        );
        assert_eq!(
            decoded.expires_at(),
            if [1, 2, 4, 5].contains(&i) {
                Some(instant(EXPIRY))
            } else {
                None
            }
        );
    }
}
#[test]
fn people_and_payroll_bytes_are_disjoint_and_never_reinterpreted() {
    let payroll = [
        Payroll::install(uuid(COMMAND), company(), 7).unwrap(),
        Payroll::grant(
            uuid(COMMAND),
            company(),
            7,
            account(RECIPIENT),
            None,
            instant(EXPIRY),
        )
        .unwrap(),
        Payroll::grant(
            uuid(COMMAND),
            company(),
            7,
            account(RECIPIENT),
            Some(witness()),
            instant(EXPIRY),
        )
        .unwrap(),
        Payroll::revoke(uuid(COMMAND), company(), 7, witness()).unwrap(),
    ];
    for command in payroll {
        let bytes = command.encode(account(ACTOR));
        rejected(Command::decode(&bytes));
        assert_eq!(Payroll::decode(&bytes).unwrap(), (account(ACTOR), command));
    }
    for vector in VECTORS {
        rejected(Payroll::decode(&unhex(vector)));
    }
}
#[test]
fn people_codec_rejects_truncation_suffix_and_unknown_discriminants() {
    for vector in VECTORS {
        let bytes = unhex(vector);
        for length in 0..bytes.len() {
            rejected(Command::decode(&bytes[..length]));
        }
        for suffix in [&[0][..], &[1, 2, 3][..]] {
            let mut bad = bytes.clone();
            bad.extend_from_slice(suffix);
            rejected(Command::decode(&bad));
        }
        for at in (0..32).chain(88..120) {
            let mut bad = bytes.clone();
            bad[at] ^= 1;
            rejected(Command::decode(&bad));
        }
        for operation in [0, 4, 255] {
            let mut bad = bytes.clone();
            bad[120] = operation;
            rejected(Command::decode(&bad));
        }
        for operation in [1, 2, 3] {
            if bytes[120] != operation {
                let mut bad = bytes.clone();
                bad[120] = operation;
                rejected(Command::decode(&bad));
            }
        }
        if bytes[120] != 1 {
            for action in [0, 3, 255] {
                let mut bad = bytes.clone();
                bad[121] = action;
                rejected(Command::decode(&bad));
            }
            let mut other = bytes.clone();
            other[121] = if bytes[121] == 1 { 2 } else { 1 };
            let (actor, decoded) = Command::decode(&other).unwrap();
            assert_eq!(decoded.encode(actor), other);
            assert_ne!(
                decoded.action(),
                Command::decode(&bytes).unwrap().1.action()
            );
        }
    }
    for (index, wrong_witness) in [(1, 1), (2, 0), (4, 1), (5, 0)] {
        for tag in [wrong_witness, 2, 255] {
            let mut bad = unhex(VECTORS[index]);
            bad[138] = tag;
            rejected(Command::decode(&bad));
        }
    }
}
#[test]
fn people_codec_rejects_invalid_ids_revisions_and_exact_time_loss() {
    for (i, vector) in VECTORS.into_iter().enumerate() {
        let bytes = unhex(vector);
        let mut ids = vec![32, 48, 64];
        if [1, 2, 4, 5].contains(&i) {
            ids.push(122);
        }
        if [2, 5].contains(&i) {
            ids.push(147);
        }
        if [3, 6].contains(&i) {
            ids.push(130);
        }
        for at in ids {
            let mut bad = bytes.clone();
            bad[at..at + 16].fill(0);
            rejected(Command::decode(&bad));
        }
        let mut bad = bytes.clone();
        bad[48..64].copy_from_slice(OrgId::platform().as_uuid().as_bytes());
        rejected(Command::decode(&bad));
        let mut revisions = vec![80];
        if [2, 5].contains(&i) {
            revisions.push(163);
        }
        if [3, 6].contains(&i) {
            revisions.push(146);
        }
        for at in revisions {
            for value in [0u64, i64::MAX as u64 + 1, u64::MAX] {
                let mut bad = bytes.clone();
                bad[at..at + 8].copy_from_slice(&value.to_be_bytes());
                rejected(Command::decode(&bad));
            }
        }
        let role = if [2, 5].contains(&i) {
            Some(139)
        } else if [3, 6].contains(&i) {
            Some(122)
        } else {
            None
        };
        if let Some(at) = role {
            for value in [0u64, 2, u64::MAX] {
                let mut bad = bytes.clone();
                bad[at..at + 8].copy_from_slice(&value.to_be_bytes());
                rejected(Command::decode(&bad));
            }
        }
        if [1, 2, 4, 5].contains(&i) {
            for micros in [
                i64::MIN,
                -62_135_596_860_000_000,
                EXPIRY - 1,
                EXPIRY + 1,
                253_402_268_400_000_000,
                i64::MAX,
            ] {
                let mut bad = bytes.clone();
                let start = bad.len() - 8;
                bad[start..].copy_from_slice(&micros.to_be_bytes());
                rejected(Command::decode(&bad));
            }
        }
    }
    for action in [Action::Read, Action::Create] {
        for micros in [
            -62_135_596_800_000_000,
            -60_000_000,
            0,
            EXPIRY,
            253_402_268_340_000_000,
        ] {
            let command = Command::grant(
                uuid(COMMAND),
                company(),
                7,
                action,
                account(RECIPIENT),
                None,
                instant(micros),
            )
            .unwrap();
            let bytes = command.encode(account(ACTOR));
            assert_eq!(&bytes[bytes.len() - 8..], &micros.to_be_bytes());
            assert_eq!(Command::decode(&bytes).unwrap().1, command);
        }
        for delta in [-1, 1] {
            rejected(Command::grant(
                uuid(COMMAND),
                company(),
                7,
                action,
                account(RECIPIENT),
                None,
                OffsetDateTime::from_unix_timestamp_nanos(i128::from(EXPIRY) * 1000 + delta)
                    .unwrap(),
            ));
        }
        for revision in [0, i64::MAX as u64 + 1, u64::MAX] {
            rejected(Command::install(uuid(COMMAND), company(), revision));
            rejected(Command::grant(
                uuid(COMMAND),
                company(),
                revision,
                action,
                account(RECIPIENT),
                None,
                instant(EXPIRY),
            ));
            rejected(Command::revoke(
                uuid(COMMAND),
                company(),
                revision,
                action,
                witness(),
            ));
        }
        for org in [OrgId::platform(), OrgId::from_uuid(Uuid::nil())] {
            rejected(Command::install(uuid(COMMAND), org, 7));
        }
        rejected(Command::install(Uuid::nil(), company(), 7));
    }
}

#[test]
fn changed_valid_inputs_remain_distinct_and_constructors_reject_bad_shapes() {
    let other = uuid("66666666-6666-4666-8666-666666666666");
    let other_account = AccountId::from_uuid(other).unwrap();
    for action in [Action::Read, Action::Create] {
        let original = Command::grant(
            uuid(COMMAND),
            company(),
            7,
            action,
            account(RECIPIENT),
            Some(witness()),
            instant(EXPIRY),
        )
        .unwrap();
        let bytes = original.encode(account(ACTOR));
        assert_ne!(original.encode(other_account), bytes);
        let cases = [
            Command::grant(
                other,
                company(),
                7,
                action,
                account(RECIPIENT),
                Some(witness()),
                instant(EXPIRY),
            )
            .unwrap(),
            Command::grant(
                uuid(COMMAND),
                OrgId::from_uuid(other),
                7,
                action,
                account(RECIPIENT),
                Some(witness()),
                instant(EXPIRY),
            )
            .unwrap(),
            Command::grant(
                uuid(COMMAND),
                company(),
                8,
                action,
                account(RECIPIENT),
                Some(witness()),
                instant(EXPIRY),
            )
            .unwrap(),
            Command::grant(
                uuid(COMMAND),
                company(),
                7,
                action,
                other_account,
                Some(witness()),
                instant(EXPIRY),
            )
            .unwrap(),
            Command::grant(
                uuid(COMMAND),
                company(),
                7,
                action,
                account(RECIPIENT),
                Some(Expectation {
                    assignment_id: other,
                    ..witness()
                }),
                instant(EXPIRY),
            )
            .unwrap(),
            Command::grant(
                uuid(COMMAND),
                company(),
                7,
                action,
                account(RECIPIENT),
                Some(Expectation {
                    assignment_revision: 10,
                    ..witness()
                }),
                instant(EXPIRY),
            )
            .unwrap(),
            Command::grant(
                uuid(COMMAND),
                company(),
                7,
                action,
                account(RECIPIENT),
                None,
                instant(EXPIRY),
            )
            .unwrap(),
            Command::grant(
                uuid(COMMAND),
                company(),
                7,
                action,
                account(RECIPIENT),
                Some(witness()),
                instant(EXPIRY + 60_000_000),
            )
            .unwrap(),
        ];
        for changed in cases {
            let encoded = changed.encode(account(ACTOR));
            assert_ne!(encoded, bytes);
            assert_eq!(Command::decode(&encoded).unwrap().1, changed);
        }
        for org in [OrgId::platform(), OrgId::from_uuid(Uuid::nil())] {
            rejected(Command::grant(
                uuid(COMMAND),
                org,
                7,
                action,
                account(RECIPIENT),
                None,
                instant(EXPIRY),
            ));
            rejected(Command::revoke(uuid(COMMAND), org, 7, action, witness()));
        }
        rejected(Command::grant(
            Uuid::nil(),
            company(),
            7,
            action,
            account(RECIPIENT),
            None,
            instant(EXPIRY),
        ));
        rejected(Command::revoke(
            Uuid::nil(),
            company(),
            7,
            action,
            witness(),
        ));
        for expected in [
            Expectation {
                assignment_id: Uuid::nil(),
                ..witness()
            },
            Expectation {
                role_revision: 0,
                ..witness()
            },
            Expectation {
                role_revision: 2,
                ..witness()
            },
            Expectation {
                assignment_revision: 0,
                ..witness()
            },
            Expectation {
                assignment_revision: i64::MAX as u64 + 1,
                ..witness()
            },
        ] {
            rejected(Command::grant(
                uuid(COMMAND),
                company(),
                7,
                action,
                account(RECIPIENT),
                Some(expected),
                instant(EXPIRY),
            ));
            rejected(Command::revoke(
                uuid(COMMAND),
                company(),
                7,
                action,
                expected,
            ));
        }
        let command = Command::grant(
            other,
            OrgId::from_uuid(other),
            i64::MAX as u64,
            action,
            other_account,
            Some(Expectation {
                assignment_revision: i64::MAX as u64,
                ..witness()
            }),
            instant(EXPIRY),
        )
        .unwrap();
        assert_eq!(
            Command::decode(&command.encode(other_account)).unwrap(),
            (other_account, command)
        );
    }
}
