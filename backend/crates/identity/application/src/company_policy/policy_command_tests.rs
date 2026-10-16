//! Shared policy protocol dispatch; child of company_policy.
//! Does not grant authority or prove SQL projection/custody validity.
use super::{
    AccountId,
    business::{NativeBusinessOperationV1, NativeCompanyBusinessCommandV1},
    people_business::{DirectoryActionV1, NativePeoplePolicyCommandV1},
    workflow::NativePolicyCommand,
};
use console_kernel_core::OrgId;
use uuid::Uuid;

fn actor() -> AccountId {
    AccountId::from_uuid(Uuid::from_u128(1)).unwrap()
}
fn company() -> OrgId {
    OrgId::from_uuid(Uuid::from_u128(2))
}
fn command() -> Uuid {
    Uuid::from_u128(3)
}
fn recipient() -> AccountId {
    AccountId::from_uuid(Uuid::from_u128(4)).unwrap()
}

#[test]
fn shared_policy_command_preserves_each_existing_codec_and_common_fields() {
    let payroll = NativeCompanyBusinessCommandV1::install(command(), company(), 7).unwrap();
    let people = NativePeoplePolicyCommandV1::install(command(), company(), 7).unwrap();
    let vectors = [
        (
            1,
            NativePolicyCommand::Payroll(payroll.clone()),
            payroll.encode(actor()),
            "native-payroll-collection-read-v1",
            super::business::MANIFEST,
        ),
        (
            2,
            NativePolicyCommand::People(people.clone()),
            people.encode(actor()),
            "native-people-directory-v1",
            super::people_business::MANIFEST,
        ),
    ];
    for (codec, wrapped, original, catalog, manifest) in vectors {
        assert_eq!(wrapped.codec_version(), codec);
        assert_eq!(wrapped.catalog_version(), catalog);
        assert_eq!(wrapped.manifest_digest(), &manifest);
        assert_eq!(wrapped.command_id(), command());
        assert_eq!(wrapped.company(), company());
        assert_eq!(wrapped.expected_company_epoch(), 7);
        assert_eq!(wrapped.operation(), NativeBusinessOperationV1::Install);
        assert!(wrapped.recipient_account_id().is_none());
        assert!(wrapped.assignment_expectation().is_none());
        assert!(wrapped.expires_at().is_none());
        assert_eq!(wrapped.encode(actor()), original);
        let (decoded_actor, decoded) = NativePolicyCommand::decode(codec, &original).unwrap();
        assert_eq!(decoded_actor, actor());
        assert_eq!(decoded, wrapped);
        for wrong in [-1, 0, 3, i16::MAX, if codec == 1 { 2 } else { 1 }] {
            assert!(
                NativePolicyCommand::decode(wrong, &original).is_err(),
                "stored codec must select exactly one grammar"
            );
        }
        let mut trailing = original.clone();
        trailing.push(0);
        assert!(NativePolicyCommand::decode(codec, &trailing).is_err());
        for end in 0..original.len() {
            assert!(NativePolicyCommand::decode(codec, &original[..end]).is_err());
        }
    }
    assert_ne!(
        NativePolicyCommand::Payroll(payroll),
        NativePolicyCommand::People(people)
    );
}

#[test]
fn shared_policy_command_retains_people_action_and_replay_distinctions() {
    let expires = time::macros::datetime!(2026-10-01 00:00:00 UTC);
    let read = NativePeoplePolicyCommandV1::grant(
        command(),
        company(),
        7,
        DirectoryActionV1::Read,
        recipient(),
        None,
        expires,
    )
    .unwrap();
    let create = NativePeoplePolicyCommandV1::grant(
        command(),
        company(),
        7,
        DirectoryActionV1::Create,
        recipient(),
        None,
        expires,
    )
    .unwrap();
    let read = NativePolicyCommand::People(read);
    let create = NativePolicyCommand::People(create);
    assert_ne!(read, create);
    assert_ne!(read.encode(actor()), create.encode(actor()));
    for (wrapped, expected_action) in [
        (read, DirectoryActionV1::Read),
        (create, DirectoryActionV1::Create),
    ] {
        assert_eq!(wrapped.recipient_account_id(), Some(recipient()));
        assert_eq!(wrapped.expires_at(), Some(expires));
        assert_eq!(wrapped.operation(), NativeBusinessOperationV1::Grant);
        let (decoded_actor, decoded) =
            NativePolicyCommand::decode(2, &wrapped.encode(actor())).unwrap();
        assert_eq!(decoded_actor, actor());
        assert_eq!(decoded, wrapped);
        let NativePolicyCommand::People(original) = decoded else {
            panic!("People became Payroll");
        };
        assert_eq!(original.action(), Some(expected_action));
        let foreign = AccountId::from_uuid(Uuid::from_u128(90)).unwrap();
        let (decoded_foreign, same_input) =
            NativePolicyCommand::decode(2, &wrapped.encode(foreign)).unwrap();
        assert_ne!(decoded_foreign, actor());
        assert_eq!(decoded_foreign, foreign);
        assert_eq!(same_input, wrapped);
    }
}

#[test]
fn shared_policy_command_preserves_assignment_witnesses_for_both_protocols() {
    let witness = super::business::PolicyAssignmentExpectationV1 {
        role_revision: 1,
        assignment_id: Uuid::from_u128(5),
        assignment_revision: 9,
    };
    let expires = time::macros::datetime!(2026-10-01 00:00:00 UTC);
    let payroll_grant = NativeCompanyBusinessCommandV1::grant(
        command(),
        company(),
        7,
        recipient(),
        Some(witness),
        expires,
    )
    .unwrap();
    let payroll_revoke =
        NativeCompanyBusinessCommandV1::revoke(command(), company(), 7, witness).unwrap();
    let people_grant = NativePeoplePolicyCommandV1::grant(
        command(),
        company(),
        7,
        DirectoryActionV1::Create,
        recipient(),
        Some(witness),
        expires,
    )
    .unwrap();
    let people_revoke = NativePeoplePolicyCommandV1::revoke(
        command(),
        company(),
        7,
        DirectoryActionV1::Create,
        witness,
    )
    .unwrap();
    for (wrapped, bytes, codec, operation) in [
        (
            NativePolicyCommand::Payroll(payroll_grant.clone()),
            payroll_grant.encode(actor()),
            1,
            NativeBusinessOperationV1::Grant,
        ),
        (
            NativePolicyCommand::Payroll(payroll_revoke.clone()),
            payroll_revoke.encode(actor()),
            1,
            NativeBusinessOperationV1::Revoke,
        ),
        (
            NativePolicyCommand::People(people_grant.clone()),
            people_grant.encode(actor()),
            2,
            NativeBusinessOperationV1::Grant,
        ),
        (
            NativePolicyCommand::People(people_revoke.clone()),
            people_revoke.encode(actor()),
            2,
            NativeBusinessOperationV1::Revoke,
        ),
    ] {
        assert_eq!(wrapped.assignment_expectation(), Some(witness));
        assert_eq!(wrapped.operation(), operation);
        assert_eq!(wrapped.encode(actor()), bytes);
        assert_eq!(
            NativePolicyCommand::decode(codec, &bytes).unwrap(),
            (actor(), wrapped.clone())
        );
        let grant = operation == NativeBusinessOperationV1::Grant;
        assert_eq!(wrapped.recipient_account_id(), grant.then_some(recipient()));
        assert_eq!(wrapped.expires_at(), grant.then_some(expires));
    }
}

// Organization V12 exact codec3 contract, design appendix SHA256
// c26751c6a49b759de50ee9e9dd42c7255b2403d1c874673f7b51b91d23dd3086.
// Test-only literals; no catalog installation, authority, owner or serving claim.
#[path = "org_unit_codec3_literal_fixtures.rs"]
mod org_unit_codec3_literals;

fn org_unit_codec3_literal_bytes(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0, "literal fixture must contain whole bytes");
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn org_unit_codec3_positive_literals() -> Vec<serde_json::Value> {
    let literals: Vec<serde_json::Value> =
        serde_json::from_str(org_unit_codec3_literals::POSITIVE_JSON).unwrap();
    assert_eq!(
        literals.len(),
        10,
        "reviewed positive literal census changed"
    );
    literals
}

fn org_unit_codec3_assert_positive(
    literal: &serde_json::Value,
    decoded_actor: AccountId,
    decoded: &NativePolicyCommand,
) {
    use super::business::PolicyAssignmentExpectationV1;
    use time::OffsetDateTime;

    let expected = &literal["expected"];
    let bytes = org_unit_codec3_literal_bytes(literal["hex"].as_str().unwrap());
    assert_eq!(
        bytes.len(),
        usize::try_from(literal["bytes"].as_u64().unwrap()).unwrap()
    );
    assert_eq!(literal["codec"].as_i64(), Some(3));
    assert_eq!(decoded.codec_version(), 3);
    assert_eq!(
        decoded_actor,
        AccountId::from_uuid(
            Uuid::parse_str(expected["actor_account_id"].as_str().unwrap()).unwrap()
        )
        .unwrap()
    );
    assert_eq!(
        decoded.company(),
        OrgId::from_uuid(Uuid::parse_str(expected["company_id"].as_str().unwrap()).unwrap())
    );
    assert_eq!(
        decoded.command_id(),
        Uuid::parse_str(expected["command_id"].as_str().unwrap()).unwrap()
    );
    assert_eq!(
        decoded.expected_company_epoch(),
        expected["expected_company_epoch"].as_u64().unwrap()
    );
    assert_eq!(
        decoded.catalog_version(),
        expected["catalog_version"].as_str().unwrap()
    );
    assert_eq!(decoded.catalog_version(), "native-org-unit-work-v1");
    let manifest = org_unit_codec3_literal_bytes(expected["manifest_digest"].as_str().unwrap());
    assert_eq!(manifest.len(), 32);
    assert_eq!(decoded.manifest_digest().as_slice(), manifest.as_slice());
    assert_eq!(
        manifest,
        org_unit_codec3_literal_bytes(
            "e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158"
        )
    );
    let operation = match expected["operation"].as_str().unwrap() {
        "Install" => NativeBusinessOperationV1::Install,
        "Grant" => NativeBusinessOperationV1::Grant,
        "Revoke" => NativeBusinessOperationV1::Revoke,
        _ => panic!("reviewed fixture contains an unknown operation"),
    };
    assert_eq!(decoded.operation(), operation);
    let recipient = expected["recipient_account_id"]
        .as_str()
        .map(|id| AccountId::from_uuid(Uuid::parse_str(id).unwrap()).unwrap());
    assert_eq!(decoded.recipient_account_id(), recipient);
    let assignment = (!expected["assignment_expectation"].is_null()).then(|| {
        let witness = &expected["assignment_expectation"];
        PolicyAssignmentExpectationV1 {
            role_revision: witness["role_revision"].as_u64().unwrap(),
            assignment_id: Uuid::parse_str(witness["assignment_id"].as_str().unwrap()).unwrap(),
            assignment_revision: witness["assignment_revision"].as_u64().unwrap(),
        }
    });
    assert_eq!(decoded.assignment_expectation(), assignment);
    let expiry = expected["expires_at_microseconds"].as_i64().map(|micros| {
        OffsetDateTime::from_unix_timestamp_nanos(i128::from(micros) * 1_000).unwrap()
    });
    assert_eq!(decoded.expires_at(), expiry);
    let encoded = decoded.encode(decoded_actor);
    assert_eq!(
        encoded, bytes,
        "decoded command changed the accepted literal"
    );
    if let Some(action) = expected["action_byte"].as_u64() {
        assert_eq!(encoded[123], u8::try_from(action).unwrap());
        assert!(matches!(action, 1..=3));
    } else {
        assert_eq!(operation, NativeBusinessOperationV1::Install);
        assert_eq!(
            encoded.len(),
            123,
            "install must not contain an action suffix"
        );
    }
}

#[test]
fn org_unit_codec3_install_literal_preserves_catalog_and_roundtrip() {
    let literals = org_unit_codec3_positive_literals();
    let install = &literals[0];
    assert_eq!(install["name"].as_str(), Some("install"));
    assert_eq!(install["expected"]["operation"].as_str(), Some("Install"));
    let bytes = org_unit_codec3_literal_bytes(install["hex"].as_str().unwrap());
    let (decoded_actor, decoded) = NativePolicyCommand::decode(3, &bytes).expect(
        "ORG_UNIT_CODEC3_INSTALL: exact reviewed install literal must reach the existing decoder",
    );
    org_unit_codec3_assert_positive(install, decoded_actor, &decoded);
}

#[test]
fn org_unit_codec3_grant_and_revoke_literals_bind_each_action_and_coordinate() {
    let literals = org_unit_codec3_positive_literals();
    let mut checked = 0;
    for literal in &literals[1..] {
        let bytes = org_unit_codec3_literal_bytes(literal["hex"].as_str().unwrap());
        let (decoded_actor, decoded) = NativePolicyCommand::decode(3, &bytes)
            .expect("ORG_UNIT_CODEC3_ACTION: exact reviewed grant/revoke literal rejected");
        org_unit_codec3_assert_positive(literal, decoded_actor, &decoded);
        checked += 1;
    }
    assert_eq!(
        checked, 9,
        "three actions need fresh/replacement grant and revoke"
    );
}

#[test]
fn org_unit_codec3_finite_literal_refusals_remain_closed() {
    let refusals: Vec<serde_json::Value> =
        serde_json::from_str(org_unit_codec3_literals::REFUSAL_JSON).unwrap();
    assert_eq!(
        refusals.len(),
        2_581,
        "reviewed finite refusal census changed"
    );
    let mut names = std::collections::BTreeSet::new();
    for refusal in &refusals {
        let name = refusal["name"].as_str().unwrap();
        assert!(names.insert(name), "duplicate refusal name: {name}");
        assert_eq!(
            refusal["expected_error"].as_str(),
            Some("invalid Company business policy input")
        );
        let codec = i16::try_from(refusal["codec"].as_i64().unwrap()).unwrap();
        let bytes = org_unit_codec3_literal_bytes(refusal["hex"].as_str().unwrap());
        assert_eq!(
            NativePolicyCommand::decode(codec, &bytes).unwrap_err(),
            console_kernel_core::KernelError::validation("invalid Company business policy input"),
            "literal refusal admitted or changed its typed result: {name}"
        );
    }
    assert_eq!(names.len(), 2_581);
}

#[test]
fn org_unit_codec3_existing_literal_protocols_remain_byte_exact() {
    let preserved: Vec<serde_json::Value> =
        serde_json::from_str(org_unit_codec3_literals::PREDECESSOR_JSON).unwrap();
    assert_eq!(
        preserved.len(),
        11,
        "four Payroll and seven People literals required"
    );
    let mut counts = [0_usize; 2];
    for literal in &preserved {
        let codec = i16::try_from(literal["codec"].as_i64().unwrap()).unwrap();
        let bytes = org_unit_codec3_literal_bytes(literal["hex"].as_str().unwrap());
        let (decoded_actor, decoded) = NativePolicyCommand::decode(codec, &bytes).unwrap();
        assert_eq!(decoded.codec_version(), codec);
        match codec {
            1 => {
                counts[0] += 1;
                assert_eq!(
                    decoded.catalog_version(),
                    "native-payroll-collection-read-v1"
                );
                assert_eq!(decoded.manifest_digest(), &super::business::MANIFEST);
            }
            2 => {
                counts[1] += 1;
                assert_eq!(decoded.catalog_version(), "native-people-directory-v1");
                assert_eq!(decoded.manifest_digest(), &super::people_business::MANIFEST);
            }
            _ => panic!("predecessor fixture contains a new codec"),
        }
        assert_eq!(decoded.encode(decoded_actor), bytes);
        assert_eq!(
            NativePolicyCommand::decode(3, &bytes).unwrap_err(),
            console_kernel_core::KernelError::validation("invalid Company business policy input")
        );
    }
    assert_eq!(counts, [4, 7]);
}

#[test]
fn org_unit_codec3_three_actions_remain_pairwise_distinct() {
    let literals = org_unit_codec3_positive_literals();
    for operation_offset in 0..3 {
        let mut decoded_actions = Vec::new();
        for start in [1, 4, 7] {
            let literal = &literals[start + operation_offset];
            let bytes = org_unit_codec3_literal_bytes(literal["hex"].as_str().unwrap());
            let (decoded_actor, decoded) = NativePolicyCommand::decode(3, &bytes)
                .expect("ORG_UNIT_CODEC3_DISTINCT: exact reviewed action literal rejected");
            org_unit_codec3_assert_positive(literal, decoded_actor, &decoded);
            decoded_actions.push((decoded_actor, decoded));
        }
        assert_eq!(decoded_actions.len(), 3);
        for (left, right) in [(0, 1), (0, 2), (1, 2)] {
            let (left_actor, left_command) = &decoded_actions[left];
            let (right_actor, right_command) = &decoded_actions[right];
            assert_eq!(left_actor, right_actor);
            assert_eq!(left_command.company(), right_command.company());
            assert_eq!(left_command.command_id(), right_command.command_id());
            assert_eq!(
                left_command.expected_company_epoch(),
                right_command.expected_company_epoch()
            );
            assert_eq!(left_command.operation(), right_command.operation());
            assert_eq!(
                left_command.recipient_account_id(),
                right_command.recipient_account_id()
            );
            assert_eq!(
                left_command.assignment_expectation(),
                right_command.assignment_expectation()
            );
            assert_eq!(left_command.expires_at(), right_command.expires_at());
            assert_ne!(
                left_command, right_command,
                "different actions collapsed into one command"
            );
            assert_ne!(
                left_command.encode(*left_actor),
                right_command.encode(*right_actor)
            );
        }
    }
}

#[test]
fn org_unit_codec3_request_refs_preserve_family_and_distinct_actions() {
    use super::workflow::{NativePolicyCommandRef, NativePolicyWorkflowError};

    let literals = org_unit_codec3_positive_literals();
    let mut references = Vec::new();
    for literal in &literals {
        let bytes = org_unit_codec3_literal_bytes(literal["hex"].as_str().unwrap());
        let (decoded_actor, decoded) = NativePolicyCommand::decode(3, &bytes)
            .expect("ORG_UNIT_CODEC3_REQUEST_REF: exact reviewed literal rejected");
        org_unit_codec3_assert_positive(literal, decoded_actor, &decoded);
        let reference = NativePolicyCommandRef::from_command(&decoded);
        assert_eq!(reference.codec_version(), 3);
        assert_eq!(reference.manifest_digest(), decoded.manifest_digest());
        assert_eq!(reference.company(), decoded.company());
        assert_eq!(reference.command_id(), decoded.command_id());
        assert_eq!(reference.operation(), decoded.operation());
        assert_eq!(reference.directory_action(), None);
        assert_eq!(reference.resolve(reference), Ok(reference));
        references.push(reference);
    }
    assert_eq!(references.len(), 10);

    for operation_offset in 0..3 {
        let selected = [
            references[1 + operation_offset],
            references[4 + operation_offset],
            references[7 + operation_offset],
        ];
        for (left, right) in [(0, 1), (0, 2), (1, 2)] {
            let left = selected[left];
            let right = selected[right];
            assert_eq!(left.company(), right.company());
            assert_eq!(left.command_id(), right.command_id());
            assert_eq!(left.operation(), right.operation());
            assert_ne!(
                left, right,
                "request reference collapsed distinct Organization actions"
            );
            assert_eq!(
                left.resolve(right),
                Err(NativePolicyWorkflowError::Unavailable)
            );
            assert_eq!(
                right.resolve(left),
                Err(NativePolicyWorkflowError::Unavailable)
            );
        }
    }
}

// Company-information V2 exact pure-codec contract; database owner remains fenced.
#[path = "company_information_codec4_contract_tests.rs"]
mod company_information_codec4_contract;
