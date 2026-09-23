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
