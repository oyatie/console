//! Durable intake and closed terminal outcome invariants.
use super::*;
use crate::{people::DirectoryExpectationsV1, people::DirectoryRegistrationInput};
use console_kernel_core::OrgId;
fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}
fn accepted() -> AcceptedDirectoryRequestV1 {
    let command = NativeDirectoryCommandV1::new(
        id(3),
        OrgId::from_uuid(id(2)),
        id(4),
        DirectoryExpectationsV1 {
            company_epoch: 1,
            object_type_id: id(5),
            action_type_id: id(6),
            action_revision: 1,
            schema_revision: 1,
            legal_name_property_id: id(7),
            employee_number_property_id: id(8),
        },
        DirectoryRegistrationInput::new("김하늘", "K-1").unwrap(),
    )
    .unwrap();
    AcceptedDirectoryRequestV1::new(
        AccountId::from_uuid(id(1)).unwrap(),
        command,
        id(9),
        OffsetDateTime::from_unix_timestamp(1_790_000_000).unwrap(),
    )
    .unwrap()
}
#[test]
fn directory_receipt_outcome_codes_are_closed_and_cross_pairs_refuse() {
    let cases = [
        (
            "COMMITTED",
            "registered",
            DirectoryTerminalOutcomeV1::Committed,
        ),
        (
            "REJECTED",
            "revision_conflict",
            DirectoryTerminalOutcomeV1::Rejected(DirectoryRejectionV1::RevisionConflict),
        ),
        (
            "REJECTED",
            "employee_number_conflict",
            DirectoryTerminalOutcomeV1::Rejected(DirectoryRejectionV1::EmployeeNumberConflict),
        ),
        (
            "REJECTED",
            "command_conflict",
            DirectoryTerminalOutcomeV1::Rejected(DirectoryRejectionV1::CommandConflict),
        ),
        (
            "CANCELLED",
            "cancelled",
            DirectoryTerminalOutcomeV1::Cancelled,
        ),
        (
            "EXPIRED",
            "intake_expired",
            DirectoryTerminalOutcomeV1::Expired,
        ),
    ];
    for (label, code, expected) in cases {
        assert_eq!(
            DirectoryTerminalOutcomeV1::from_storage(label, code).unwrap(),
            expected
        );
        assert_eq!(expected.outcome(), label);
        assert_eq!(expected.result_code(), code);
        for other in [
            "COMMITTED",
            "REJECTED",
            "CANCELLED",
            "EXPIRED",
            "UNCERTAIN",
            "PENDING",
            "committed",
            "",
        ] {
            assert_eq!(
                DirectoryTerminalOutcomeV1::from_storage(other, code).is_ok(),
                other == label
            );
        }
        assert!(DirectoryTerminalOutcomeV1::from_storage(label, "unknown").is_err());
    }
}
#[test]
fn directory_receipt_preserves_accepted_command_and_has_only_committed_links() {
    let request = accepted();
    let original = request.command().encode(request.actor());
    let at = request.accepted_at() + Duration::seconds(1);
    for outcome in [
        DirectoryTerminalOutcomeV1::Committed,
        DirectoryTerminalOutcomeV1::Rejected(DirectoryRejectionV1::EmployeeNumberConflict),
        DirectoryTerminalOutcomeV1::Cancelled,
        DirectoryTerminalOutcomeV1::Expired,
    ] {
        let end = if outcome == DirectoryTerminalOutcomeV1::Expired {
            request.execution_not_after()
        } else {
            at
        };
        let receipt = DirectoryTerminalV1::new(request.clone(), outcome, end).unwrap();
        assert_eq!(receipt.accepted(), &request);
        assert_eq!(
            receipt
                .accepted()
                .command()
                .encode(receipt.accepted().actor()),
            original
        );
        if outcome == DirectoryTerminalOutcomeV1::Committed {
            assert_eq!(receipt.employee_id(), Some(id(4)));
            assert_eq!(receipt.person_id(), Some(id(4)));
            assert_eq!(receipt.canonical_command_id(), Some(id(3)));
            assert_eq!(
                receipt.canonical_result(),
                Some(
                    serde_json::json!({"person_id":id(4),"version":1,"target":"people.create_person"})
                )
            );
        } else {
            assert_eq!(receipt.employee_id(), None);
            assert_eq!(receipt.person_id(), None);
            assert_eq!(receipt.canonical_command_id(), None);
            assert_eq!(receipt.canonical_result(), None);
        }
    }
}
#[test]
fn directory_receipt_precision_deadline_and_terminal_time_are_exact() {
    let r = accepted();
    assert_eq!(
        r.execution_not_after() - r.accepted_at(),
        Duration::hours(168)
    );
    assert!(
        AcceptedDirectoryRequestV1::new(
            r.actor(),
            r.command().clone(),
            Uuid::nil(),
            r.accepted_at()
        )
        .is_err()
    );
    assert!(
        AcceptedDirectoryRequestV1::new(
            r.actor(),
            r.command().clone(),
            id(9),
            r.accepted_at() + Duration::nanoseconds(1)
        )
        .is_err()
    );
    for outcome in [
        DirectoryTerminalOutcomeV1::Committed,
        DirectoryTerminalOutcomeV1::Rejected(DirectoryRejectionV1::RevisionConflict),
        DirectoryTerminalOutcomeV1::Cancelled,
    ] {
        assert!(
            DirectoryTerminalV1::new(
                r.clone(),
                outcome,
                r.accepted_at() - Duration::microseconds(1)
            )
            .is_err()
        );
        assert!(
            DirectoryTerminalV1::new(
                r.clone(),
                outcome,
                r.accepted_at() + Duration::nanoseconds(1)
            )
            .is_err()
        );
        assert!(
            DirectoryTerminalV1::new(
                r.clone(),
                outcome,
                r.execution_not_after() - Duration::microseconds(1)
            )
            .is_ok()
        );
        assert!(DirectoryTerminalV1::new(r.clone(), outcome, r.execution_not_after()).is_err());
    }
    assert!(
        DirectoryTerminalV1::new(
            r.clone(),
            DirectoryTerminalOutcomeV1::Expired,
            r.execution_not_after() - Duration::microseconds(1)
        )
        .is_err()
    );
    assert!(
        DirectoryTerminalV1::new(
            r.clone(),
            DirectoryTerminalOutcomeV1::Expired,
            r.execution_not_after()
        )
        .is_ok()
    );
}
