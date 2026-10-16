//! Adapter state/projection controls only; not authentication/SQL timing proof.
use super::*;
fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}
fn at() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_790_000_000).unwrap()
}
#[test]
fn directory_pending_replay_retains_original_deadline_and_refuses_late_readmission() {
    let command = NativeDirectoryCommandV1::new(
        id(1),
        OrgId::from_uuid(id(2)),
        id(3),
        DirectoryExpectationsV1 {
            company_epoch: 3,
            object_type_id: id(4),
            action_type_id: id(5),
            action_revision: 1,
            schema_revision: 1,
            legal_name_property_id: id(6),
            employee_number_property_id: id(7),
        },
        DirectoryRegistrationInput::new("김하늘", "K-1").unwrap(),
    )
    .unwrap();
    let accepted = AcceptedDirectoryRequestV1::new(
        AccountId::from_uuid(id(8)).unwrap(),
        command,
        id(9),
        at() - Duration::hours(168) + Duration::seconds(1),
    )
    .unwrap();
    // Exact historical replay goes through the same producer as real prepare,
    // never new(now), so its deadline remains one second away.
    let mut until = None;
    let result = pending_status(accepted.clone(), &mut until);
    assert_eq!(result, DirectoryStatus::Pending(accepted.clone()));
    assert_eq!(until, Some(at() + Duration::seconds(1)));
    assert_eq!(check_pending_readmission(until, at()), Ok(()));
    assert_eq!(
        check_pending_readmission(until, at() + Duration::seconds(1)),
        Err(Error::Conflict)
    );
    assert_eq!(
        check_pending_readmission(until, at() + Duration::seconds(2)),
        Err(Error::Conflict)
    );
    assert_eq!(
        check_pending_readmission(None, at() + Duration::days(30)),
        Ok(()),
        "historical terminal disclosure does not reuse expired pending deadline"
    );
    assert_eq!(accepted.execution_not_after(), at() + Duration::seconds(1));
}
fn record(kind: &str, name: serde_json::Value, number: Option<&str>) -> RecordRow {
    RecordRow {
        employee_id: id(1),
        person_id: id(2),
        name_attributes: serde_json::json!({"legal_name":name}),
        employee_number: number.map(str::to_owned),
        source_kind: kind.into(),
        person_version: 1,
        registered_at: at(),
    }
}
#[test]
fn directory_legacy_optional_projection_preserves_real_person_string_semantics() {
    for absent in [
        serde_json::Value::Null,
        serde_json::json!(""),
        serde_json::json!(17),
        serde_json::json!(false),
        serde_json::json!({}),
    ] {
        let view = record("LEGACY", absent, None).view().unwrap();
        assert_eq!(view.legal_name, None);
        assert_eq!(view.employee_number, None);
        assert_eq!(view.employee_id, id(1));
        assert_eq!(view.person_id, id(2));
    }
    let view = record("LEGACY", serde_json::json!(" 김하늘 "), Some("legacy 01"))
        .view()
        .unwrap();
    assert_eq!(view.legal_name.as_deref(), Some(" 김하늘 "));
    assert_eq!(view.employee_number.as_deref(), Some("legacy 01"));
    assert_eq!(
        record("LEGACY", serde_json::json!("김하늘"), Some(""))
            .view()
            .unwrap()
            .employee_number,
        None
    );
}
#[test]
fn directory_native_missing_or_invalid_fields_are_corruption_not_legacy_absence() {
    for (name, number) in [
        (serde_json::Value::Null, Some("K-1")),
        (serde_json::json!(false), Some("K-1")),
        (serde_json::json!("김하늘"), None),
        (serde_json::json!(" 김하늘"), Some("K-1")),
        (serde_json::json!("김하늘"), Some(" K-1")),
    ] {
        assert!(matches!(
            record("NATIVE_DIRECTORY", name, number).view(),
            Err(Error::Unavailable)
        ));
    }
    let native = record("NATIVE_DIRECTORY", serde_json::json!("김하늘"), Some("K-1"))
        .view()
        .unwrap();
    assert_eq!(native.legal_name.as_deref(), Some("김하늘"));
    assert_eq!(native.employee_number.as_deref(), Some("K-1"));
    assert!(matches!(
        record("UNKNOWN", serde_json::json!("김하늘"), Some("K-1")).view(),
        Err(Error::Unavailable)
    ));
}
