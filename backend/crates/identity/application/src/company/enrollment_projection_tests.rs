#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Pure row-shape checks only; no constructed row establishes live authority.
use super::{CompanyEnrollmentError, CompanyEnrollmentProjection, CompanyEnrollmentStatus};
use crate::CompanyEnrollmentV1;
use console_kernel_core::OrgId;
use uuid::Uuid;

fn input() -> (Uuid, Uuid, CompanyEnrollmentV1) {
    let account = Uuid::from_u128(1);
    let command = Uuid::from_u128(2);
    let input = CompanyEnrollmentV1::from_json_slice(
        &serde_json::to_vec(&serde_json::json!({
            "command_id": command, "group_id": null, "slug": "native-projection",
            "name": "원본 이름", "administrative_account_id": Uuid::from_u128(3)
        }))
        .unwrap(),
    )
    .unwrap();
    (account, command, input)
}

fn row(state: &str) -> CompanyEnrollmentProjection {
    CompanyEnrollmentProjection {
        state: state.to_owned(),
        codec_version: 1,
        input_bytes: None,
        receipt_id: None,
        org_id: None,
        group_id: None,
        administrative_account_id: None,
    }
}

fn rejected(account: Uuid, command: Uuid, row: CompanyEnrollmentProjection) {
    assert!(
        matches!(
            CompanyEnrollmentStatus::from_projection(account, command, row),
            Err(CompanyEnrollmentError::Unavailable)
        ),
        "corrupt owner projection must fail closed as unavailable material"
    );
}

#[test]
fn company_enrollment_projection_pending_preserves_exact_codec_namespace_and_shape() {
    let (account, command, input) = input();
    let encoded = input.encode(account).unwrap();
    let mut valid = row("PENDING");
    valid.input_bytes = Some(encoded.clone());
    let CompanyEnrollmentStatus::Pending(actual) =
        CompanyEnrollmentStatus::from_projection(account, command, valid).unwrap()
    else {
        panic!("valid pending row")
    };
    assert_eq!(actual, input);
    assert_eq!(actual.encode(account).unwrap(), encoded);
    for (a, c) in [
        (Uuid::nil(), command),
        (account, Uuid::nil()),
        (Uuid::from_u128(4), command),
        (account, Uuid::from_u128(5)),
    ] {
        let mut bad = row("PENDING");
        bad.input_bytes = Some(encoded.clone());
        rejected(a, c, bad);
    }
    for codec in [0, 2, -1] {
        let mut bad = row("PENDING");
        bad.codec_version = codec;
        bad.input_bytes = Some(encoded.clone());
        rejected(account, command, bad);
    }
    rejected(account, command, row("PENDING"));
    let mut trailing = encoded.clone();
    trailing.push(0);
    for bytes in [vec![], trailing, b"unrecognized historical codec".to_vec()] {
        let mut bad = row("PENDING");
        bad.input_bytes = Some(bytes);
        rejected(account, command, bad);
    }
    for slot in 0..4 {
        let mut bad = row("PENDING");
        bad.input_bytes = Some(encoded.clone());
        match slot {
            0 => bad.receipt_id = Some(Uuid::from_u128(6)),
            1 => bad.org_id = Some(Uuid::from_u128(7)),
            2 => bad.group_id = Some(Uuid::from_u128(8)),
            _ => bad.administrative_account_id = Some(Uuid::from_u128(9)),
        }
        rejected(account, command, bad);
    }
    for state in ["MISSING", "pending", "UNKNOWN", ""] {
        rejected(account, command, row(state));
    }
}

#[test]
fn company_enrollment_projection_terminal_and_committed_rows_reject_mixed_or_nil_fields() {
    let (account, command, input) = input();
    assert!(matches!(
        CompanyEnrollmentStatus::from_projection(account, command, row("CANCELLED")).unwrap(),
        CompanyEnrollmentStatus::Cancelled
    ));
    assert!(matches!(
        CompanyEnrollmentStatus::from_projection(account, command, row("EXPIRED")).unwrap(),
        CompanyEnrollmentStatus::Expired
    ));
    for state in ["CANCELLED", "EXPIRED"] {
        for slot in 0..6 {
            let mut bad = row(state);
            match slot {
                0 => bad.input_bytes = Some(input.encode(account).unwrap()),
                1 => bad.receipt_id = Some(Uuid::from_u128(6)),
                2 => bad.org_id = Some(Uuid::from_u128(7)),
                3 => bad.group_id = Some(Uuid::from_u128(8)),
                4 => bad.administrative_account_id = Some(Uuid::from_u128(9)),
                _ => bad.codec_version = 2,
            }
            rejected(account, command, bad);
        }
    }
    let committed = || CompanyEnrollmentProjection {
        state: "COMMITTED".into(),
        codec_version: 1,
        input_bytes: None,
        receipt_id: Some(Uuid::from_u128(6)),
        org_id: Some(Uuid::from_u128(7)),
        group_id: Some(Uuid::from_u128(8)),
        administrative_account_id: Some(Uuid::from_u128(9)),
    };
    let CompanyEnrollmentStatus::Committed {
        receipt_id,
        org_id,
        group_id,
        administrative_account_id,
    } = CompanyEnrollmentStatus::from_projection(account, command, committed()).unwrap()
    else {
        panic!("valid future receipt projection shape")
    };
    assert_eq!(
        (receipt_id, org_id, group_id, administrative_account_id),
        (
            Uuid::from_u128(6),
            Uuid::from_u128(7),
            Uuid::from_u128(8),
            Uuid::from_u128(9)
        )
    );
    for slot in 0..4 {
        for value in [None, Some(Uuid::nil())] {
            let mut bad = committed();
            match slot {
                0 => bad.receipt_id = value,
                1 => bad.org_id = value,
                2 => bad.group_id = value,
                _ => bad.administrative_account_id = value,
            }
            rejected(account, command, bad);
        }
    }
    let mut bad = committed();
    bad.org_id = Some(*OrgId::platform().as_uuid());
    rejected(account, command, bad);
    let mut bad = committed();
    bad.input_bytes = Some(input.encode(account).unwrap());
    rejected(account, command, bad);
    let mut bad = committed();
    bad.codec_version = 2;
    rejected(account, command, bad);
    rejected(Uuid::nil(), command, committed());
    rejected(account, Uuid::nil(), committed());
}
