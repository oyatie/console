// Additive parser-contract proposal. Include inside existing form::tests.
// Reuses existing parser fixtures; no database/browser/authentication claim.
// Proposed API: parse_document -> { proof, input: DocumentInput }, where
// DocumentInput::{Ready(Input), GrantValidation(NativePolicyGrantDraft)}.

fn retained_expiry(pairs: &[(String, String)]) -> super::NativePolicyGrantDraft {
    let parsed = super::parse_document(company(), Target::Grant, &encode(pairs))
        .unwrap_or_else(|_| panic!("well-formed envelope lost recoverable expiry"));
    assert_eq!(parsed.proof, "opaque-proof");
    match parsed.input {
        super::DocumentInput::GrantValidation(draft) => draft,
        super::DocumentInput::Ready(_) => panic!("invalid expiry became executable input"),
    }
}

#[test]
fn document_expiry_validation_preserves_original_typed_scope_and_text() {
    for value in [
        "",
        "2027-02-29T00:15",
        "2028-02-29T24:00",
        "2028-02-29T00:15Z",
        "0001-01-01T08:59",
        "２０２８-02-29T00:15",
        "<img src=x onerror=alert(1)>",
    ] {
        for expected in [None, Some(assignment())] {
            let mut pairs = fields(Target::Grant);
            set(&mut pairs, "expected_company_epoch", "17");
            set(&mut pairs, "expires_at_local", value);
            if let Some(expected) = expected {
                set(&mut pairs, "expected_role_revision", "1");
                set(&mut pairs, "assignment_id", ASSIGNMENT);
                set(
                    &mut pairs,
                    "expected_assignment_revision",
                    &expected.assignment_revision.to_string(),
                );
            }
            let draft = retained_expiry(&pairs);
            assert_eq!(
                draft.selector,
                NativePolicyCommandRef::new(
                    company(),
                    id(COMMAND),
                    NativeBusinessOperationV1::Grant
                )
                .unwrap()
            );
            assert_eq!(draft.expected_company_epoch, 17);
            assert_eq!(draft.recipient_account_id, recipient());
            assert_eq!(draft.assignment, expected);
            assert_eq!(draft.expires_at_local, value);
            // The existing strict command parser must remain strict.
            rejects_pairs(Target::Grant, &pairs);
        }
    }
}

#[test]
fn document_valid_grant_preserves_the_existing_command_bytes() {
    for local in ["2028-02-29T00:15", "2001-01-01T09:00", "2099-12-31T23:59"] {
        let mut pairs = fields(Target::Grant);
        set(&mut pairs, "expires_at_local", local);
        let expected = parsed_command(Target::Grant, &pairs);
        let parsed = super::parse_document(company(), Target::Grant, &encode(&pairs)).unwrap();
        match parsed.input {
            super::DocumentInput::Ready(Input::Command(NativePolicyCommand::Payroll(actual))) => {
                assert_eq!(actual, expected);
                assert_eq!(actual.encode(recipient()), expected.encode(recipient()));
            }
            _ => panic!("valid expiry changed input category"),
        }
    }
}

#[test]
fn document_expiry_retention_cannot_hide_other_invalid_grant_fields() {
    for (key, value) in [
        ("command_id", NIL),
        ("command_id", "not-a-uuid"),
        ("expected_company_epoch", "0"),
        ("expected_company_epoch", "01"),
        ("expected_company_epoch", "9223372036854775808"),
        ("recipient_account_id", NIL),
        ("recipient_account_id", "not-a-uuid"),
        ("expected_role_revision", "2"),
        ("assignment_id", NIL),
        ("expected_assignment_revision", "0"),
        ("expected_assignment_revision", "01"),
        ("csrf_proof", ""),
    ] {
        let mut pairs = fields(Target::Grant);
        set(&mut pairs, "expected_role_revision", "1");
        set(&mut pairs, "assignment_id", ASSIGNMENT);
        set(&mut pairs, "expected_assignment_revision", "7");
        set(&mut pairs, "expires_at_local", "invalid expiry");
        set(&mut pairs, key, value);
        assert!(matches!(
            super::parse_document(company(), Target::Grant, &encode(&pairs)),
            Err(FormError::Invalid)
        ));
    }
    for scope in [OrgId::from_uuid(Uuid::nil()), OrgId::platform()] {
        let mut pairs = fields(Target::Grant);
        set(&mut pairs, "expires_at_local", "invalid expiry");
        assert!(matches!(
            super::parse_document(scope, Target::Grant, &encode(&pairs)),
            Err(FormError::Invalid)
        ));
    }
}

#[test]
fn document_expiry_retention_preserves_closed_envelope_and_assignment_tuple() {
    let mut original = fields(Target::Grant);
    set(&mut original, "expires_at_local", "invalid expiry");
    for (key, value) in original
        .iter()
        .cloned()
        .chain([("unexpected".into(), "x".into())])
    {
        let mut pairs = original.clone();
        pairs.push((key, value));
        assert!(matches!(
            super::parse_document(company(), Target::Grant, &encode(&pairs)),
            Err(FormError::Invalid)
        ));
    }
    for missing in 0..original.len() {
        let mut pairs = original.clone();
        pairs.remove(missing);
        assert!(matches!(
            super::parse_document(company(), Target::Grant, &encode(&pairs)),
            Err(FormError::Invalid)
        ));
    }
    for mask in 1..7 {
        let mut pairs = original.clone();
        for (index, (key, value)) in [
            ("expected_role_revision", "1"),
            ("assignment_id", ASSIGNMENT),
            ("expected_assignment_revision", "7"),
        ]
        .iter()
        .enumerate()
        {
            if mask & (1 << index) != 0 {
                set(&mut pairs, key, value);
            }
        }
        assert!(matches!(
            super::parse_document(company(), Target::Grant, &encode(&pairs)),
            Err(FormError::Invalid)
        ));
    }
}

#[test]
fn document_retention_bounds_decoded_utf8_bytes_without_truncation() {
    for text in ["x".repeat(64), "가".repeat(21)] {
        let mut pairs = fields(Target::Grant);
        set(&mut pairs, "expires_at_local", &text);
        assert_eq!(retained_expiry(&pairs).expires_at_local, text);
    }
    for text in ["x".repeat(65), "가".repeat(22)] {
        let mut pairs = fields(Target::Grant);
        set(&mut pairs, "expires_at_local", &text);
        assert!(matches!(
            super::parse_document(company(), Target::Grant, &encode(&pairs)),
            Err(FormError::TooLarge)
        ));
    }
    for text in ["bad\0date", "bad\ndate", "bad\rdate", "bad\tdate"] {
        let mut pairs = fields(Target::Grant);
        set(&mut pairs, "expires_at_local", text);
        assert!(matches!(
            super::parse_document(company(), Target::Grant, &encode(&pairs)),
            Err(FormError::Invalid)
        ));
    }
}

#[test]
fn document_retention_does_not_depend_on_field_order() {
    let mut pairs = fields(Target::Grant);
    set(&mut pairs, "expires_at_local", "invalid expiry");
    for _ in 0..pairs.len() {
        let draft = retained_expiry(&pairs);
        assert_eq!(draft.selector.command_id(), id(COMMAND));
        assert_eq!(draft.expected_company_epoch, 2);
        assert_eq!(draft.recipient_account_id, recipient());
        assert!(draft.assignment.is_none());
        assert_eq!(draft.expires_at_local, "invalid expiry");
        pairs.rotate_left(1);
    }
}
