// Add inside form::tests after the existing expiry_validation_tests include.
// Calls the shared production parser; pure fixtures are not Auth/SQL evidence.
use super::super::NativePolicySubject;
use console_identity_application::company_policy::{
    people_business::{DirectoryActionV1, NativePeoplePolicyCommandV1},
    workflow::NativePolicyCommand,
};

fn people_subjects() -> [(NativePolicySubject, DirectoryActionV1); 2] {
    [
        (NativePolicySubject::PeopleRead, DirectoryActionV1::Read),
        (NativePolicySubject::PeopleCreate, DirectoryActionV1::Create),
    ]
}

fn subject_command(
    subject: NativePolicySubject,
    target: Target,
    pairs: &[(String, String)],
) -> NativePolicyCommand {
    let parsed = super::parse_document_for(company(), target, &encode(pairs), subject)
        .unwrap_or_else(|_| panic!("valid subject command rejected"));
    assert_eq!(parsed.proof, "opaque-proof");
    match parsed.input {
        super::DocumentInput::Ready(Input::Command(command)) => command,
        _ => panic!("valid subject command lost its command category"),
    }
}

fn subject_rejects(subject: NativePolicySubject, target: Target, body: &[u8], error: FormError) {
    match super::parse_document_for(company(), target, body, subject) {
        Err(actual) => assert_eq!(actual, error),
        Ok(_) => panic!("invalid subject form admitted"),
    }
}

fn exact_people_bytes(actual: NativePolicyCommand, expected: NativePeoplePolicyCommandV1) {
    // The actor is deliberately distinct from the requested grant recipient.
    let actor = AccountId::from_uuid(id("22222222-2222-4222-8222-222222222222")).unwrap();
    assert_eq!(actual, NativePolicyCommand::People(expected.clone()));
    assert_eq!(actual.codec_version(), 2);
    assert_eq!(actual.catalog_version(), "native-people-directory-v1");
    assert_eq!(actual.encode(actor), expected.encode(actor));
    assert_eq!(
        NativePolicyCommand::decode(2, &actual.encode(actor)).unwrap(),
        (actor, NativePolicyCommand::People(expected))
    );
}

#[test]
fn people_catalog_install_preserves_codec2_constructor_bytes_and_field_order() {
    let mut pairs = fields(Target::Install);
    for _ in 0..pairs.len() {
        exact_people_bytes(
            subject_command(NativePolicySubject::PeopleCatalog, Target::Install, &pairs),
            NativePeoplePolicyCommandV1::install(id(COMMAND), company(), 2).unwrap(),
        );
        pairs.rotate_left(1);
    }
}

#[test]
fn people_grants_preserve_requested_action_recipient_expectations_and_kst_bytes() {
    for (subject, action) in people_subjects() {
        for expected in [None, Some(assignment())] {
            let mut pairs = fields(Target::Grant);
            if let Some(a) = expected {
                set(&mut pairs, "expected_role_revision", "1");
                set(&mut pairs, "assignment_id", ASSIGNMENT);
                set(
                    &mut pairs,
                    "expected_assignment_revision",
                    &a.assignment_revision.to_string(),
                );
            }
            for (local, utc) in [
                ("2028-02-29T00:15", "2028-02-28T15:15:00Z"),
                ("2027-01-01T00:00", "2026-12-31T15:00:00Z"),
            ] {
                set(&mut pairs, "expires_at_local", local);
                let wanted = NativePeoplePolicyCommandV1::grant(
                    id(COMMAND),
                    company(),
                    2,
                    action,
                    recipient(),
                    expected,
                    instant(utc),
                )
                .unwrap();
                assert_eq!(wanted.action(), Some(action));
                exact_people_bytes(subject_command(subject, Target::Grant, &pairs), wanted);
                pairs.reverse();
            }
        }
    }
}

#[test]
fn people_revokes_bind_exact_action_route_assignment_and_body_revisions() {
    let target = Target::Revoke {
        assignment: id(ASSIGNMENT),
    };
    for (subject, action) in people_subjects() {
        exact_people_bytes(
            subject_command(subject, target, &fields(target)),
            NativePeoplePolicyCommandV1::revoke(id(COMMAND), company(), 2, action, assignment())
                .unwrap(),
        );
        let mut pairs = fields(target);
        pairs.push(("assignment_id".into(), ASSIGNMENT.into()));
        subject_rejects(subject, target, &encode(&pairs), FormError::Invalid);
        subject_rejects(
            subject,
            Target::Revoke {
                assignment: Uuid::nil(),
            },
            &encode(&fields(target)),
            FormError::Invalid,
        );
    }
}

#[test]
fn people_retry_is_proof_only_and_preserves_resolved_or_actionless_recovery() {
    for (subject, action) in [
        (NativePolicySubject::PeopleCatalog, None),
        (
            NativePolicySubject::PeopleRead,
            Some(DirectoryActionV1::Read),
        ),
        (
            NativePolicySubject::PeopleCreate,
            Some(DirectoryActionV1::Create),
        ),
    ] {
        for operation in [
            NativeBusinessOperationV1::Install,
            NativeBusinessOperationV1::Grant,
            NativeBusinessOperationV1::Revoke,
        ] {
            let target = Target::Retry {
                operation,
                command_id: id(COMMAND),
            };
            let body = encode(&fields(target));
            if action.is_some() && operation == NativeBusinessOperationV1::Install {
                subject_rejects(subject, target, &body, FormError::Invalid);
                continue;
            }
            let parsed = super::parse_document_for(company(), target, &body, subject)
                .unwrap_or_else(|_| panic!("valid original People retry rejected"));
            assert_eq!(parsed.proof, "opaque-proof");
            let expected =
                NativePolicyCommandRef::new_people(company(), id(COMMAND), operation, action)
                    .unwrap();
            match parsed.input {
                super::DocumentInput::Ready(Input::Retry(actual)) => assert_eq!(actual, expected),
                _ => panic!("People retry created replacement input"),
            }
            for (key, value) in fields(Target::Grant)
                .into_iter()
                .chain([
                    ("action".into(), "read".into()),
                    ("catalog".into(), "native-people-directory-v1".into()),
                ])
                .filter(|(key, _)| key != "csrf_proof")
            {
                let mut pairs = fields(target);
                pairs.push((key, value));
                subject_rejects(subject, target, &encode(&pairs), FormError::Invalid);
            }
            subject_rejects(
                subject,
                Target::Retry {
                    operation,
                    command_id: Uuid::nil(),
                },
                &body,
                FormError::Invalid,
            );
        }
    }
}

#[test]
fn people_subject_operation_matrix_refuses_catalog_grants_and_action_installs() {
    for target in [
        Target::Grant,
        Target::Revoke {
            assignment: id(ASSIGNMENT),
        },
    ] {
        subject_rejects(
            NativePolicySubject::PeopleCatalog,
            target,
            &encode(&fields(target)),
            FormError::Invalid,
        );
    }
    for (subject, _) in people_subjects() {
        subject_rejects(
            subject,
            Target::Install,
            &encode(&fields(Target::Install)),
            FormError::Invalid,
        );
    }
}

#[test]
fn people_forms_keep_exact_required_fields_and_reject_plain_encoded_duplicates() {
    let mut cases = vec![(NativePolicySubject::PeopleCatalog, Target::Install)];
    for (subject, _) in people_subjects() {
        cases.extend([
            (subject, Target::Grant),
            (
                subject,
                Target::Revoke {
                    assignment: id(ASSIGNMENT),
                },
            ),
        ]);
    }
    cases.push((
        NativePolicySubject::PeopleCatalog,
        Target::Retry {
            operation: NativeBusinessOperationV1::Grant,
            command_id: id(COMMAND),
        },
    ));
    for (subject, target) in cases {
        let pairs = fields(target);
        for index in 0..pairs.len() {
            let mut missing = pairs.clone();
            missing.remove(index);
            subject_rejects(subject, target, &encode(&missing), FormError::Invalid);
        }
        for (key, value) in &pairs {
            let mut duplicate = pairs.clone();
            duplicate.push((key.clone(), value.clone()));
            subject_rejects(subject, target, &encode(&duplicate), FormError::Invalid);
            let mut encoded_duplicate = encode(&pairs);
            encoded_duplicate.extend_from_slice(
                format!("&%{:02X}{}=untrusted", key.as_bytes()[0], &key[1..]).as_bytes(),
            );
            subject_rejects(subject, target, &encoded_duplicate, FormError::Invalid);
        }
        for key in [
            "actor",
            "company",
            "group",
            "action",
            "subject",
            "catalog",
            "codec_version",
            "manifest",
            "operation",
            "receipt",
            "return_url",
        ] {
            let mut extra = pairs.clone();
            extra.push((key.into(), "untrusted".into()));
            subject_rejects(subject, target, &encode(&extra), FormError::Invalid);
        }
    }
}

#[test]
fn people_invalid_expiry_preserves_exact_action_command_scope_and_original_text() {
    for (subject, action) in people_subjects() {
        for expected in [None, Some(assignment())] {
            for expiry in [
                "",
                "2027-02-29T00:15",
                "2028-02-29T24:00",
                "2028-02-29T00:15Z",
                "0001-01-01T08:59",
                "２０２８-02-29T00:15",
                "<img src=x onerror=alert(1)>",
            ] {
                let mut pairs = fields(Target::Grant);
                set(&mut pairs, "expected_company_epoch", "17");
                set(&mut pairs, "expires_at_local", expiry);
                if expected.is_some() {
                    set(&mut pairs, "expected_role_revision", "1");
                    set(&mut pairs, "assignment_id", ASSIGNMENT);
                    set(&mut pairs, "expected_assignment_revision", "7");
                }
                let parsed =
                    super::parse_document_for(company(), Target::Grant, &encode(&pairs), subject)
                        .unwrap_or_else(|_| panic!("recoverable People expiry lost"));
                assert_eq!(parsed.proof, "opaque-proof");
                let draft = match parsed.input {
                    super::DocumentInput::GrantValidation(draft) => draft,
                    _ => panic!("invalid People expiry became an executable command"),
                };
                assert_eq!(
                    draft.selector,
                    NativePolicyCommandRef::new_people(
                        company(),
                        id(COMMAND),
                        NativeBusinessOperationV1::Grant,
                        Some(action)
                    )
                    .unwrap()
                );
                assert_eq!(draft.expected_company_epoch, 17);
                assert_eq!(draft.recipient_account_id, recipient());
                assert_eq!(draft.assignment, expected);
                assert_eq!(draft.expires_at_local, expiry);
            }
        }
    }
}

#[test]
fn people_recoverable_expiry_does_not_admit_invalid_identity_expectations_or_limits() {
    for (subject, _) in people_subjects() {
        let mut original = fields(Target::Grant);
        set(&mut original, "expected_role_revision", "1");
        set(&mut original, "assignment_id", ASSIGNMENT);
        set(&mut original, "expected_assignment_revision", "7");
        set(&mut original, "expires_at_local", "invalid expiry");
        for (key, value) in [
            ("command_id", NIL),
            ("recipient_account_id", NIL),
            ("expected_company_epoch", "01"),
            ("expected_company_epoch", "9223372036854775808"),
            ("expected_role_revision", "2"),
            ("assignment_id", NIL),
            ("expected_assignment_revision", "0"),
            ("csrf_proof", ""),
        ] {
            let mut pairs = original.clone();
            set(&mut pairs, key, value);
            subject_rejects(subject, Target::Grant, &encode(&pairs), FormError::Invalid);
        }
        for mask in 1..7 {
            let mut pairs = fields(Target::Grant);
            set(&mut pairs, "expires_at_local", "invalid expiry");
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
            subject_rejects(subject, Target::Grant, &encode(&pairs), FormError::Invalid);
        }
        for expiry in ["bad\0date", "bad\ndate", "bad\tdate"] {
            let mut pairs = original.clone();
            set(&mut pairs, "expires_at_local", expiry);
            subject_rejects(subject, Target::Grant, &encode(&pairs), FormError::Invalid);
        }
        for expiry in ["x".repeat(65), "가".repeat(22)] {
            let mut pairs = original.clone();
            set(&mut pairs, "expires_at_local", &expiry);
            subject_rejects(subject, Target::Grant, &encode(&pairs), FormError::TooLarge);
        }
        for scope in [OrgId::from_uuid(Uuid::nil()), OrgId::platform()] {
            assert!(matches!(
                super::parse_document_for(scope, Target::Grant, &encode(&original), subject),
                Err(FormError::Invalid)
            ));
        }
    }
    subject_rejects(
        NativePolicySubject::PeopleCatalog,
        Target::Install,
        &vec![b'x'; 16385],
        FormError::TooLarge,
    );
    let retry = Target::Retry {
        operation: NativeBusinessOperationV1::Grant,
        command_id: id(COMMAND),
    };
    for invalid in [
        b"csrf_proof=%FF".as_slice(),
        b"csrf_proof=%",
        b"csrf_proof=%ED%A0%80",
        b"csrf_proof",
    ] {
        subject_rejects(
            NativePolicySubject::PeopleCatalog,
            retry,
            invalid,
            FormError::Invalid,
        );
    }
    subject_rejects(
        NativePolicySubject::PeopleCatalog,
        retry,
        &encode(&[("csrf_proof".into(), "x".repeat(4097))]),
        FormError::TooLarge,
    );
}

#[test]
fn explicit_payroll_subject_and_legacy_wrapper_preserve_codec1_commands() {
    for target in [
        Target::Install,
        Target::Grant,
        Target::Revoke {
            assignment: id(ASSIGNMENT),
        },
    ] {
        let pairs = fields(target);
        let expected = parsed_command(target, &pairs);
        let actual = subject_command(NativePolicySubject::PayrollRead, target, &pairs);
        assert_eq!(actual.codec_version(), 1);
        assert_eq!(actual, NativePolicyCommand::Payroll(expected.clone()));
        assert_eq!(actual.encode(recipient()), expected.encode(recipient()));
    }
}
