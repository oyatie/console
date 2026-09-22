//! Additive ordinary unit tests for company/native_policy/form.rs.
//! Inputs here are parser fixtures, not browser/business data or Auth evidence.
use super::{FormError, Input, Target, parse};
use console_identity_application::company_policy::{
    AccountId,
    business::{
        NativeBusinessOperationV1, NativeCompanyBusinessCommandV1, PolicyAssignmentExpectationV1,
    },
    workflow::NativePolicyCommandRef,
};
use console_kernel_core::OrgId;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

const COMMAND: &str = "abcdefab-1234-4123-8123-abcdefabcdef";
const RECIPIENT: &str = "fedcbafe-2345-4234-8234-fedcbafedcba";
const ASSIGNMENT: &str = "abcabcab-3456-4345-8345-abcabcabcabc";
const NIL: &str = "00000000-0000-0000-0000-000000000000";

fn company() -> OrgId {
    OrgId::from_uuid(Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap())
}
fn id(value: &str) -> Uuid {
    Uuid::parse_str(value).unwrap()
}
fn recipient() -> AccountId {
    AccountId::from_uuid(id(RECIPIENT)).unwrap()
}
fn assignment() -> PolicyAssignmentExpectationV1 {
    PolicyAssignmentExpectationV1 {
        role_revision: 1,
        assignment_id: id(ASSIGNMENT),
        assignment_revision: 7,
    }
}
fn instant(value: &str) -> OffsetDateTime {
    OffsetDateTime::parse(value, &Rfc3339).unwrap()
}
fn fields(target: Target) -> Vec<(String, String)> {
    let mut result = vec![("csrf_proof".into(), "opaque-proof".into())];
    if matches!(target, Target::Retry { .. }) {
        return result;
    }
    result.extend([
        ("command_id".into(), COMMAND.into()),
        ("expected_company_epoch".into(), "2".into()),
    ]);
    match target {
        Target::Install => (),
        Target::Grant => result.extend([
            ("recipient_account_id".into(), RECIPIENT.into()),
            ("expected_role_revision".into(), "".into()),
            ("assignment_id".into(), "".into()),
            ("expected_assignment_revision".into(), "".into()),
            ("expires_at_local".into(), "2028-02-29T00:15".into()),
        ]),
        Target::Revoke { .. } => result.extend([
            ("expected_role_revision".into(), "1".into()),
            ("expected_assignment_revision".into(), "7".into()),
        ]),
        Target::Retry { .. } => unreachable!(),
    }
    result
}
fn targets() -> [Target; 4] {
    [
        Target::Install,
        Target::Grant,
        Target::Revoke {
            assignment: id(ASSIGNMENT),
        },
        Target::Retry {
            operation: NativeBusinessOperationV1::Grant,
            command_id: id(COMMAND),
        },
    ]
}
fn encode(fields: &[(String, String)]) -> Vec<u8> {
    // Independent existing URL serializer generates ordinary browser form bytes.
    let mut encoded = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in fields {
        encoded.append_pair(key, value);
    }
    encoded.finish().into_bytes()
}
fn set(fields: &mut [(String, String)], key: &str, value: &str) {
    fields.iter_mut().find(|(name, _)| name == key).unwrap().1 = value.into();
}
fn parsed_command(target: Target, pairs: &[(String, String)]) -> NativeCompanyBusinessCommandV1 {
    match parse(company(), target, &encode(pairs)) {
        Ok(parsed) => match parsed.input {
            Input::Command(command) => command,
            Input::Retry(_) => panic!("command unexpectedly became retry"),
        },
        Err(_) => panic!("valid command form rejected"),
    }
}
fn rejects(scope: OrgId, target: Target, body: &[u8], expected: FormError) {
    match parse(scope, target, body) {
        Err(error) => assert_eq!(error, expected),
        Ok(_) => panic!("invalid form admitted"),
    }
}
fn rejects_pairs(target: Target, pairs: &[(String, String)]) {
    rejects(company(), target, &encode(pairs), FormError::Invalid);
}

#[test]
fn install_form_builds_existing_typed_command_without_field_order_dependency() {
    let mut pairs = fields(Target::Install);
    let expected = NativeCompanyBusinessCommandV1::install(id(COMMAND), company(), 2).unwrap();
    assert_eq!(parsed_command(Target::Install, &pairs), expected);
    pairs.reverse();
    assert_eq!(parsed_command(Target::Install, &pairs), expected);
}

#[test]
fn first_grant_three_empty_expectations_mean_absent_and_kst_crosses_utc_date() {
    let expected = NativeCompanyBusinessCommandV1::grant(
        id(COMMAND),
        company(),
        2,
        recipient(),
        None,
        instant("2028-02-28T15:15:00Z"),
    )
    .unwrap();
    let actual = parsed_command(Target::Grant, &fields(Target::Grant));
    assert_eq!(actual, expected);
    assert_eq!(actual.assignment_expectation(), None);
    // Existing immutable codec is unchanged; parser does not own or reseal it.
    assert_eq!(actual.encode(recipient()), expected.encode(recipient()));
}

#[test]
fn renewal_grant_preserves_exact_existing_assignment_expectations() {
    let mut pairs = fields(Target::Grant);
    set(&mut pairs, "expected_role_revision", "1");
    set(&mut pairs, "assignment_id", ASSIGNMENT);
    set(&mut pairs, "expected_assignment_revision", "7");
    let expected = NativeCompanyBusinessCommandV1::grant(
        id(COMMAND),
        company(),
        2,
        recipient(),
        Some(assignment()),
        instant("2028-02-28T15:15:00Z"),
    )
    .unwrap();
    assert_eq!(parsed_command(Target::Grant, &pairs), expected);
}

#[test]
fn revoke_uses_only_route_assignment_and_exact_body_revisions() {
    let target = Target::Revoke {
        assignment: id(ASSIGNMENT),
    };
    let expected =
        NativeCompanyBusinessCommandV1::revoke(id(COMMAND), company(), 2, assignment()).unwrap();
    assert_eq!(parsed_command(target, &fields(target)), expected);
    let mut pairs = fields(target);
    pairs.push(("assignment_id".into(), ASSIGNMENT.into()));
    rejects_pairs(target, &pairs);
}

#[test]
fn retry_accepts_only_proof_and_retains_each_exact_original_selector() {
    for operation in [
        NativeBusinessOperationV1::Install,
        NativeBusinessOperationV1::Grant,
        NativeBusinessOperationV1::Revoke,
    ] {
        let target = Target::Retry {
            operation,
            command_id: id(COMMAND),
        };
        let expected = NativePolicyCommandRef::new(company(), id(COMMAND), operation).unwrap();
        match parse(company(), target, &encode(&fields(target))) {
            Ok(parsed) => {
                assert!(parsed.proof == "opaque-proof");
                match parsed.input {
                    Input::Retry(selector) => assert_eq!(selector, expected),
                    Input::Command(_) => panic!("retry created replacement command"),
                }
            }
            Err(_) => panic!("valid original retry selector rejected"),
        }
    }
}

#[test]
fn proof_decode_is_strict_utf8_single_pass_and_preserves_whitespace() {
    let target = targets()[3];
    for (body, expected) in [
        (b"csrf_proof=a+b%2Bc%2520".as_slice(), "a b+c%20"),
        (b"csrf_proof=%ED%95%9C%EA%B8%80".as_slice(), "한글"),
        (b"csrf_proof=+%09opaque+".as_slice(), " \topaque "),
        (b"%63srf_proof=%2f%2F".as_slice(), "//"),
        (b"csrf_proof=a=b".as_slice(), "a=b"),
        ("csrf_proof=한글".as_bytes(), "한글"),
    ] {
        let parsed =
            parse(company(), target, body).unwrap_or_else(|_| panic!("valid decode rejected"));
        assert!(parsed.proof == expected, "decoded proof was normalized");
    }
}

#[test]
fn every_plain_or_percent_encoded_duplicate_key_is_rejected() {
    for target in targets() {
        let pairs = fields(target);
        for (key, value) in &pairs {
            let mut body = encode(&pairs);
            body.push(b'&');
            body.extend(encode(&[(key.clone(), value.clone())]));
            rejects(company(), target, &body, FormError::Invalid);
            let mut body = encode(&pairs);
            body.extend_from_slice(
                format!("&%{:02X}{}=different", key.as_bytes()[0], &key[1..]).as_bytes(),
            );
            rejects(company(), target, &body, FormError::Invalid);
        }
    }
}

#[test]
fn every_required_field_is_required_and_unknown_fields_never_grant_authority() {
    for target in targets() {
        let pairs = fields(target);
        for index in 0..pairs.len() {
            let mut missing = pairs.clone();
            missing.remove(index);
            rejects_pairs(target, &missing);
        }
        for key in [
            "company",
            "actor",
            "group",
            "operation",
            "manifest",
            "receipt",
            "return_url",
            "csrf_header",
            "submit",
        ] {
            let mut extra = pairs.clone();
            extra.push((key.into(), "untrusted".into()));
            rejects_pairs(target, &extra);
        }
    }
}

#[test]
fn malformed_pair_percent_and_utf8_sequences_are_not_lossily_repaired() {
    let target = targets()[3];
    for body in [
        b"".as_slice(),
        b"csrf_proof",
        b"=proof",
        b"csrf_proof=ok&",
        b"&csrf_proof=ok",
        b"csrf_proof=ok&&",
        b"csrf_proof=%",
        b"csrf_proof=%0",
        b"csrf_proof=%GG",
        b"csrf_proof=%0g",
        b"csrf_proof=%FF",
        b"csrf_proof=%C0%AF",
        b"csrf_proof=%ED%A0%80",
        b"csrf_proof=%E2%82",
        b"csrf_proof=\xff",
        b"%FF=proof",
        b"csrf%_proof=proof",
        b"csrf+proof=proof",
        b"csrf_proof=ok;command_id=bad&unknown=x",
    ] {
        rejects(company(), target, body, FormError::Invalid);
    }
}

#[test]
fn decoded_proof_byte_boundary_preserves_existing_ascii_and_multibyte_contract() {
    let target = targets()[3];
    for proof in [
        "p".into(),
        "p".repeat(4096),
        format!("{}p", "한".repeat(1365)),
    ] {
        let body = encode(&[("csrf_proof".into(), proof.clone())]);
        let parsed = parse(company(), target, &body)
            .unwrap_or_else(|_| panic!("admitted proof boundary rejected"));
        assert!(parsed.proof == proof);
    }
    for proof in ["p".repeat(4097), format!("{}pp", "한".repeat(1365))] {
        rejects(
            company(),
            target,
            &encode(&[("csrf_proof".into(), proof)]),
            FormError::TooLarge,
        );
    }
    rejects(company(), target, b"csrf_proof=", FormError::Invalid);
}

#[test]
fn encoded_body_limit_is_checked_before_decode_and_allows_full_percent_expansion() {
    let target = targets()[3];
    let expanded = format!("csrf_proof={}", "%70".repeat(4096));
    let parsed = parse(company(), target, expanded.as_bytes())
        .unwrap_or_else(|_| panic!("full permitted encoded proof rejected"));
    assert!(parsed.proof == "p".repeat(4096));
    // Same malformed grammar, distinct boundary classification. Over-limit
    // body must never be parsed first and classified as ordinary Invalid.
    rejects(company(), target, &vec![b'x'; 16384], FormError::Invalid);
    rejects(company(), target, &vec![b'x'; 16385], FormError::TooLarge);
}

#[test]
fn numeric_aliases_zero_negative_and_overflow_are_rejected_without_coercion() {
    let invalid = [
        "",
        "0",
        "00",
        "01",
        "+1",
        "-1",
        " 1",
        "1 ",
        "1.0",
        "1e0",
        "１",
        "١",
        "9223372036854775808",
        "18446744073709551616",
    ];
    for target in [
        Target::Install,
        Target::Revoke {
            assignment: id(ASSIGNMENT),
        },
    ] {
        for value in invalid {
            let mut pairs = fields(target);
            set(&mut pairs, "expected_company_epoch", value);
            rejects_pairs(target, &pairs);
        }
    }
    let target = Target::Revoke {
        assignment: id(ASSIGNMENT),
    };
    for key in ["expected_role_revision", "expected_assignment_revision"] {
        for value in invalid {
            let mut pairs = fields(target);
            set(&mut pairs, key, value);
            rejects_pairs(target, &pairs);
        }
    }
    let mut pairs = fields(target);
    set(&mut pairs, "expected_role_revision", "2");
    rejects_pairs(target, &pairs);
    let mut pairs = fields(target);
    set(&mut pairs, "expected_company_epoch", "9223372036854775807");
    set(
        &mut pairs,
        "expected_assignment_revision",
        "9223372036854775807",
    );
    let parsed = parsed_command(target, &pairs);
    assert_eq!(parsed.expected_company_epoch(), i64::MAX as u64);
    assert_eq!(
        parsed.assignment_expectation().unwrap().assignment_revision,
        i64::MAX as u64
    );
}

#[test]
fn uuid_aliases_nil_and_malformed_values_do_not_become_commands() {
    for (target, key, canonical) in [
        (Target::Install, "command_id", COMMAND),
        (Target::Grant, "recipient_account_id", RECIPIENT),
    ] {
        for value in [
            "".to_owned(),
            NIL.into(),
            canonical.to_uppercase(),
            canonical.replace('-', ""),
            format!("{{{canonical}}}"),
            format!("urn:uuid:{canonical}"),
            format!(" {canonical}"),
            format!("{canonical} "),
            "not-a-uuid".into(),
        ] {
            let mut pairs = fields(target);
            set(&mut pairs, key, &value);
            rejects_pairs(target, &pairs);
        }
    }
}

#[test]
fn grant_assignment_is_exactly_absent_or_complete_never_a_partial_tuple() {
    let keys = [
        "expected_role_revision",
        "assignment_id",
        "expected_assignment_revision",
    ];
    let values = ["1", ASSIGNMENT, "7"];
    for mask in 1..7 {
        let mut pairs = fields(Target::Grant);
        for index in 0..3 {
            if mask & (1 << index) != 0 {
                set(&mut pairs, keys[index], values[index]);
            }
        }
        rejects_pairs(Target::Grant, &pairs);
    }
    for (key, bad) in [
        (keys[0], "0"),
        (keys[0], "2"),
        (keys[1], NIL),
        (keys[1], "none"),
        (keys[2], "0"),
        (keys[2], "01"),
    ] {
        let mut pairs = fields(Target::Grant);
        for index in 0..3 {
            set(&mut pairs, keys[index], values[index]);
        }
        set(&mut pairs, key, bad);
        rejects_pairs(Target::Grant, &pairs);
    }
}

#[test]
fn kst_datetime_maps_exactly_across_leap_and_year_boundaries() {
    for (local, utc) in [
        ("2028-02-29T00:15", "2028-02-28T15:15:00Z"),
        ("2028-03-01T08:59", "2028-02-29T23:59:00Z"),
        ("2027-01-01T00:00", "2026-12-31T15:00:00Z"),
        ("2000-02-29T09:00", "2000-02-29T00:00:00Z"),
        ("0001-01-01T09:00", "0001-01-01T00:00:00Z"),
        ("9999-12-31T23:59", "9999-12-31T14:59:00Z"),
    ] {
        let mut pairs = fields(Target::Grant);
        set(&mut pairs, "expires_at_local", local);
        assert_eq!(
            parsed_command(Target::Grant, &pairs).expires_at(),
            Some(instant(utc))
        );
    }
}

#[test]
fn datetime_rejects_noncanonical_timezones_seconds_dates_and_codec_underflow() {
    for value in [
        "",
        "2028-2-29T00:15",
        "2028-02-29t00:15",
        "2028-02-29 00:15",
        "2028-02-29T0:15",
        "2028-02-29T00:1",
        "2028-02-29T00:15:00",
        "2028-02-29T00:15Z",
        "2028-02-29T00:15+09:00",
        " 2028-02-29T00:15",
        "2028-02-29T00:15 ",
        "2027-02-29T00:15",
        "1900-02-29T00:15",
        "2028-04-31T00:15",
        "2028-00-01T00:15",
        "2028-13-01T00:15",
        "2028-01-00T00:15",
        "2028-02-29T24:00",
        "2028-02-29T23:60",
        "0000-01-01T09:00",
        "10000-01-01T09:00",
        "0001-01-01T08:59",
        "２０２８-02-29T00:15",
        "2028-02-29T00:15\0",
    ] {
        let mut pairs = fields(Target::Grant);
        set(&mut pairs, "expires_at_local", value);
        rejects_pairs(Target::Grant, &pairs);
    }
}

#[test]
fn parser_does_not_replace_database_execution_clock_for_grant_applicability() {
    for local in ["2001-01-01T09:00", "2099-12-31T23:59"] {
        let mut pairs = fields(Target::Grant);
        set(&mut pairs, "expires_at_local", local);
        let command = parsed_command(Target::Grant, &pairs);
        assert_eq!(command.operation(), NativeBusinessOperationV1::Grant);
    }
}

#[test]
fn retry_rejects_every_replacement_command_field_even_when_matching_original() {
    let target = targets()[3];
    let grant = fields(Target::Grant);
    for (key, value) in grant.iter().filter(|(key, _)| key != "csrf_proof") {
        let mut pairs = fields(target);
        pairs.push((key.clone(), value.clone()));
        rejects_pairs(target, &pairs);
    }
    let mut pairs = fields(target);
    pairs.push(("operation".into(), "grant".into()));
    rejects_pairs(target, &pairs);
}

#[test]
fn invalid_internal_scope_and_nil_route_targets_cannot_bypass_typed_guards() {
    for scope in [OrgId::from_uuid(Uuid::nil()), OrgId::platform()] {
        for target in targets() {
            rejects(scope, target, &encode(&fields(target)), FormError::Invalid);
        }
    }
    for target in [
        Target::Revoke {
            assignment: Uuid::nil(),
        },
        Target::Retry {
            operation: NativeBusinessOperationV1::Install,
            command_id: Uuid::nil(),
        },
    ] {
        rejects(
            company(),
            target,
            &encode(&fields(target)),
            FormError::Invalid,
        );
    }
}
