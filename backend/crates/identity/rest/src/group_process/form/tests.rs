//! External test-only private parser proposal. Parser fixtures are never stored
//! business data and grant no authority. These tests do not establish HTTP
//! admission, signed Auth/CSRF, SQL custody, owner effects or release acceptance.
use super::{Draft, DraftInput, FormError, Input, Target, parse};
use console_identity_application::group_process::*;
use time::{Date, Month, UtcOffset};
use uuid::Uuid;

const METHOD: &str = "ATTENDED_ACCOUNT_AND_DOCUMENTARY_REVIEW_V1";
const PROSE_KEYS: [&str; 8] = [
    "intended_claimant_matching_procedure",
    "account_possession_procedure",
    "physical_human_evidence_procedure",
    "duplicate_contradictory_claim_procedure",
    "qualification_criteria_instruction",
    "escalation_adjudication_procedure",
    "evidence_minimization_retention_description",
    "recipient_responsibility",
];
fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}
fn group() -> GroupId {
    GroupId::from_uuid(id(3)).unwrap()
}
fn incarnation() -> GroupIncarnation {
    GroupIncarnation::from_uuid(id(4)).unwrap()
}
fn requested() -> GroupProcessRouteSelectorV1 {
    GroupProcessRouteSelectorV1::new(group(), id(1)).unwrap()
}
fn target(kind: u8) -> Target {
    match kind {
        0 => Target::Adopt,
        1 => Target::Suspend { process: id(5) },
        2 => Target::Retry {
            requested: requested(),
        },
        _ => panic!("unknown fixture target"),
    }
}
fn fields(kind: u8) -> Vec<(String, String)> {
    let mut pairs = vec![("csrf_proof".into(), "opaque-proof".into())];
    if kind == 2 {
        return pairs;
    }
    pairs.extend([
        ("command_id".into(), id(1).to_string()),
        ("expected_group_revision".into(), "7".into()),
        ("expected_group_incarnation".into(), id(4).to_string()),
        (
            "expected_group_identity_policy_revision".into(),
            if kind == 0 { "0" } else { "1" }.into(),
        ),
    ]);
    if kind == 0 {
        pairs.extend([
            ("process_id".into(), id(5).to_string()),
            ("expected_prior_process_revision".into(), "0".into()),
            ("process_expiry".into(), "2028-02-29T00:15:07".into()),
            ("operator_responsibility".into(), "1".into()),
            ("title".into(), "독립 검증 절차".into()),
            ("method".into(), METHOD.into()),
        ]);
        for (index, name) in PROSE_KEYS.into_iter().enumerate() {
            pairs.push((name.into(), format!("절차 {index}: 원본 입력")));
        }
    } else {
        pairs.extend([
            ("process_version".into(), "1".into()),
            ("process_digest".into(), "ab".repeat(32)),
            ("expected_process_head_revision".into(), "2".into()),
            ("expected_process_head_digest".into(), "cd".repeat(32)),
            ("reason".into(), "원래 요청의 중지 사유".into()),
        ]);
    }
    pairs
}
fn set(pairs: &mut [(String, String)], key: &str, value: impl Into<String>) {
    pairs.iter_mut().find(|(name, _)| name == key).unwrap().1 = value.into();
}
fn value<'a>(pairs: &'a [(String, String)], key: &str) -> &'a str {
    &pairs.iter().find(|(name, _)| name == key).unwrap().1
}
fn encode(pairs: &[(String, String)]) -> Vec<u8> {
    let mut form = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in pairs {
        form.append_pair(key, value);
    }
    form.finish().into_bytes()
}
fn input(kind: u8, pairs: &[(String, String)]) -> Input {
    let parsed = parse(group(), target(kind), &encode(pairs))
        .unwrap_or_else(|_| panic!("declared transport fixture refused"));
    assert_eq!(parsed.proof, value(pairs, "csrf_proof"));
    parsed.input
}
fn command(kind: u8, pairs: &[(String, String)]) -> GroupProcessCommandV1 {
    match input(kind, pairs) {
        Input::Command(command) => command,
        _ => panic!("valid input did not produce owner command"),
    }
}
fn draft(kind: u8, pairs: &[(String, String)]) -> Draft {
    match input(kind, pairs) {
        Input::Validation(draft) => draft,
        _ => panic!("semantic error was not retained draft"),
    }
}
fn rejects(kind: u8, body: &[u8], expected: FormError) {
    match parse(group(), target(kind), body) {
        Err(actual) => assert_eq!(actual, expected),
        Ok(_) => panic!("malformed transport reached command/draft/retry"),
    }
}
fn assert_original(draft: &Draft, pairs: &[(String, String)]) {
    assert_eq!(
        draft.original,
        GroupProcessLocator::new(group(), incarnation(), id(1)).unwrap()
    );
    assert_eq!(draft.process_id, id(5));
    assert_eq!(
        draft.expected_group_revision,
        value(pairs, "expected_group_revision")
            .parse::<u64>()
            .unwrap()
    );
    assert_eq!(
        draft.expected_policy_revision,
        value(pairs, "expected_group_identity_policy_revision")
            .parse::<u64>()
            .unwrap()
    );
    assert!(!draft.errors.is_empty());
}
fn assert_adopt_raw(draft: &Draft, pairs: &[(String, String)]) {
    assert_original(draft, pairs);
    match &draft.input {
        DraftInput::Adopt {
            expected_prior_head_revision,
            expiry,
            content,
        } => {
            assert_eq!(
                *expected_prior_head_revision,
                value(pairs, "expected_prior_process_revision")
                    .parse::<u64>()
                    .unwrap()
            );
            assert_eq!(expiry, value(pairs, "process_expiry"));
            assert_eq!(content.title, value(pairs, "title"));
            assert_eq!(content.method, value(pairs, "method"));
            for (index, key) in PROSE_KEYS.into_iter().enumerate() {
                assert_eq!(content.prose[index], value(pairs, key));
            }
        }
        _ => panic!("adoption validation changed operation"),
    }
}

fn expected_content(pairs: &[(String, String)]) -> ProcessContentV1 {
    // Independently map each declared field into its exact canonical slot.
    // Only new transport normalizes CRLF; no production form helper is used.
    let normalized = |key: &str| value(pairs, key).replace("\r\n", "\n");
    ProcessContentV1::from_projection(ProcessContentProjectionV1 {
        title: normalized("title"),
        method: ProcessMethodV1::AttendedAccountAndDocumentaryReview,
        intended_claimant_matching_procedure: normalized(PROSE_KEYS[0]),
        account_possession_procedure: normalized(PROSE_KEYS[1]),
        physical_human_evidence_procedure: normalized(PROSE_KEYS[2]),
        duplicate_contradictory_claim_procedure: normalized(PROSE_KEYS[3]),
        qualification_criteria_instruction: normalized(PROSE_KEYS[4]),
        escalation_adjudication_procedure: normalized(PROSE_KEYS[5]),
        evidence_minimization_retention_description: normalized(PROSE_KEYS[6]),
        recipient_responsibility: normalized(PROSE_KEYS[7]),
    })
    .unwrap()
}
fn assert_selected_revision(command: &GroupProcessCommandV1, key: &str, expected: u64) {
    let actual = match key {
        "expected_group_revision" => command.expected_group_revision(),
        "expected_group_identity_policy_revision" => command.expected_policy_revision(),
        "expected_prior_process_revision" => {
            command.adoption().unwrap().expected_prior_head_revision()
        }
        "process_version" => command.suspension().unwrap().content_version(),
        "expected_process_head_revision" => command.suspension().unwrap().expected_head_revision(),
        _ => panic!("unknown independently declared revision field"),
    };
    assert_eq!(
        actual, expected,
        "selected revision was substituted or truncated"
    );
}

#[test]
fn group_form_positive_exact_rosters_reordered_fields_and_route_only_suspend_process() {
    assert_eq!(fields(0).len(), 19);
    assert_eq!(fields(1).len(), 10);
    assert_eq!(fields(2).len(), 1);
    for kind in 0..=2 {
        let mut pairs = fields(kind);
        pairs.reverse();
        match input(kind, &pairs) {
            Input::Command(command) => {
                assert_ne!(kind, 2);
                assert_eq!(command.group(), group());
                assert_eq!(command.incarnation(), incarnation());
                assert_eq!(command.command_id(), id(1));
                assert_eq!(command.expected_group_revision(), 7);
                if kind == 0 {
                    let adopt = command.adoption().unwrap();
                    assert_eq!(adopt.process_id(), id(5));
                    assert_eq!(
                        adopt.responsibility(),
                        AcceptedOperatorResponsibilityV1::from_form("yes").unwrap()
                    );
                    assert_eq!(adopt.content().title(), "독립 검증 절차");
                    assert_eq!(adopt.content(), &expected_content(&pairs));
                } else {
                    let suspend = command.suspension().unwrap();
                    assert_eq!(suspend.process_id(), id(5));
                    assert_eq!(suspend.content_digest(), &[0xab; 32]);
                    assert_eq!(suspend.expected_head_digest(), &[0xcd; 32]);
                    assert_eq!(suspend.reason(), "원래 요청의 중지 사유");
                }
            }
            Input::Retry(actual) => {
                assert_eq!(kind, 2);
                assert_eq!(actual, requested());
            }
            _ => panic!("valid fixed fixture became validation"),
        }
    }
}
#[test]
fn group_form_rejects_every_omitted_duplicate_encoded_duplicate_and_unknown_key() {
    for kind in 0..=2 {
        let pairs = fields(kind);
        for index in 0..pairs.len() {
            let mut omitted = pairs.clone();
            omitted.remove(index);
            rejects(kind, &encode(&omitted), FormError::Invalid);
            let mut duplicate = pairs.clone();
            duplicate.push(pairs[index].clone());
            rejects(kind, &encode(&duplicate), FormError::Invalid);
            let (key, val) = &pairs[index];
            let mut encoded_duplicate = encode(&pairs);
            encoded_duplicate.push(b'&');
            encoded_duplicate
                .extend(format!("%{:02X}{}=", key.as_bytes()[0], &key[1..]).as_bytes());
            let encoded_val = encode(&[("x".into(), val.clone())]);
            encoded_duplicate.extend_from_slice(&encoded_val[2..]);
            rejects(kind, &encoded_duplicate, FormError::Invalid);
        }
        for key in [
            "actor",
            "account_id",
            "group_id",
            "current_org",
            "mode",
            "bootstrap_stage",
            "command_bytes",
            "input_digest",
            "csrf_header",
            "process_id_from_route",
        ] {
            let mut unknown = pairs.clone();
            unknown.push((key.into(), "1".into()));
            rejects(kind, &encode(&unknown), FormError::Invalid);
        }
    }
    let mut suspend = fields(1);
    suspend.push(("process_id".into(), id(5).to_string()));
    rejects(1, &encode(&suspend), FormError::Invalid);
    for key in [
        "command_id",
        "expected_group_incarnation",
        "reason",
        "title",
        "process_id",
    ] {
        let mut retry = fields(2);
        retry.push((key.into(), "replacement".into()));
        rejects(2, &encode(&retry), FormError::Invalid);
    }
}
#[test]
fn group_form_rejects_malformed_urlencoded_wire_and_invalid_utf8() {
    for kind in 0..=2 {
        for malformed in [
            b"".as_slice(),
            b"csrf_proof",
            b"=value",
            b"&csrf_proof=ok",
            b"csrf_proof=ok&",
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
            b"csrf%_proof=ok",
            b"csrf+proof=ok",
        ] {
            rejects(kind, malformed, FormError::Invalid);
        }
    }
    let literal = parse(group(), target(2), b"csrf_proof=ok;command_id=replacement").unwrap();
    assert_eq!(literal.proof, "ok;command_id=replacement");
    assert!(matches!(literal.input, Input::Retry(actual) if actual == requested()));
}
#[test]
fn group_form_proof_is_raw_unnormalized_and_bounded_in_decoded_utf8_bytes() {
    for proof in [
        "a b+c%20",
        " \t한글\r\n+%25 ",
        "p=literal&value",
        "\0opaque",
    ] {
        for kind in 0..=2 {
            let mut pairs = fields(kind);
            set(&mut pairs, "csrf_proof", proof);
            input(kind, &pairs);
        }
    }
    let parsed = parse(group(), target(2), b"csrf_proof=a+b%2Bc%2520").unwrap();
    assert_eq!(parsed.proof, "a b+c%20");
    for proof in ["p".repeat(4096), format!("{}x", "한".repeat(1365))] {
        assert_eq!(proof.len(), 4096);
        for kind in 0..=2 {
            let mut pairs = fields(kind);
            set(&mut pairs, "csrf_proof", proof.clone());
            input(kind, &pairs);
            set(&mut pairs, "csrf_proof", format!("{proof}x"));
            rejects(kind, &encode(&pairs), FormError::TooLarge);
            set(&mut pairs, "csrf_proof", "");
            rejects(kind, &encode(&pairs), FormError::Invalid);
        }
    }
    let body = format!("csrf_proof={}", "%70".repeat(4096));
    let parsed = parse(group(), target(2), body.as_bytes()).unwrap();
    assert_eq!(parsed.proof, "p".repeat(4096));
}
#[test]
fn group_form_encoded_body_limits_admit_exact_cap_for_grammar_check_and_refuse_overflow() {
    for (kind, cap) in [(0, 131072), (1, 32768), (2, 32768)] {
        // Exact-size malformed wire must reach grammar refusal. No valid form
        // is claimed at this cap; independent decoded bounds are stricter.
        rejects(kind, &vec![b'x'; cap], FormError::Invalid);
        rejects(kind, &vec![b'x'; cap + 1], FormError::TooLarge);
    }
}
#[test]
fn group_form_identifiers_are_canonical_nonnil_and_transport_errors_never_validation() {
    for (kind, key) in [
        (0, "command_id"),
        (0, "expected_group_incarnation"),
        (0, "process_id"),
        (1, "command_id"),
        (1, "expected_group_incarnation"),
    ] {
        for invalid in [
            "",
            "not-a-uuid",
            "00000000-0000-0000-0000-000000000000",
            "00000000000000000000000000000001",
            "00000000-0000-0000-0000-00000000000A",
            "{00000000-0000-0000-0000-000000000001}",
            "urn:uuid:00000000-0000-0000-0000-000000000001",
            " 00000000-0000-0000-0000-000000000001",
        ] {
            let mut pairs = fields(kind);
            set(&mut pairs, key, invalid);
            rejects(kind, &encode(&pairs), FormError::Invalid);
        }
    }
    assert!(matches!(
        parse(
            group(),
            Target::Suspend {
                process: Uuid::nil()
            },
            &encode(&fields(1))
        ),
        Err(FormError::Invalid)
    ));
    assert!(matches!(
        parse(
            GroupId::from_uuid(id(23)).unwrap(),
            target(2),
            &encode(&fields(2))
        ),
        Err(FormError::Invalid)
    ));
}
#[test]
fn group_form_revisions_have_exact_bigint_grammar_with_independent_zero_expectations() {
    for (kind, key, zero) in [
        (0, "expected_group_revision", false),
        (0, "expected_group_identity_policy_revision", true),
        (0, "expected_prior_process_revision", true),
        (1, "expected_group_revision", false),
        (1, "expected_group_identity_policy_revision", false),
        (1, "process_version", false),
        (1, "expected_process_head_revision", false),
    ] {
        for invalid in [
            "",
            "+1",
            "-1",
            "01",
            "00",
            " 1",
            "1 ",
            "1.0",
            "1e0",
            "１",
            "9223372036854775808",
            "18446744073709551615",
            "999999999999999999999999999",
        ] {
            let mut pairs = fields(kind);
            set(&mut pairs, key, invalid);
            rejects(kind, &encode(&pairs), FormError::Invalid);
        }
        let mut pairs = fields(kind);
        set(&mut pairs, key, i64::MAX.to_string());
        let max = command(kind, &pairs);
        assert_selected_revision(&max, key, i64::MAX as u64);
        set(&mut pairs, key, "0");
        if zero {
            let zero_command = command(kind, &pairs);
            assert_selected_revision(&zero_command, key, 0);
        } else {
            rejects(kind, &encode(&pairs), FormError::Invalid);
        }
    }
    for (policy, prior) in [("0", "1"), ("1", "0")] {
        let mut pairs = fields(0);
        set(
            &mut pairs,
            "expected_group_identity_policy_revision",
            policy,
        );
        set(&mut pairs, "expected_prior_process_revision", prior);
        let command = command(0, &pairs);
        assert_eq!(
            command.expected_policy_revision(),
            policy.parse::<u64>().unwrap()
        );
        assert_eq!(
            command.adoption().unwrap().expected_prior_head_revision(),
            prior.parse::<u64>().unwrap()
        );
    }
}
#[test]
fn group_form_suspend_digest_has_exact_lowercase_hex64_grammar() {
    for key in ["process_digest", "expected_process_head_digest"] {
        for invalid in [
            String::new(),
            "ab".repeat(31),
            format!("{}a", "ab".repeat(32)),
            "AB".repeat(32),
            "gg".repeat(32),
            format!("0x{}", "ab".repeat(32)),
            format!(" {}", "ab".repeat(32)),
        ] {
            let mut pairs = fields(1);
            set(&mut pairs, key, invalid);
            rejects(1, &encode(&pairs), FormError::Invalid);
        }
        let mut pairs = fields(1);
        set(&mut pairs, key, "00".repeat(32));
        command(1, &pairs);
    }
}
#[test]
fn group_form_responsibility_requires_exact_checkbox1_and_maps_existing_owner_yes() {
    let pairs = fields(0);
    let actual = command(0, &pairs);
    assert_eq!(
        actual.adoption().unwrap().responsibility(),
        AcceptedOperatorResponsibilityV1::from_form("yes").unwrap()
    );
    for invalid in ["", "yes", "true", "on", "0", "01", "1 ", "１"] {
        let mut pairs = fields(0);
        set(&mut pairs, "operator_responsibility", invalid);
        rejects(0, &encode(&pairs), FormError::Invalid);
    }
}
#[test]
fn group_form_kst_seconds_calendar_and_year_endpoints_convert_once() {
    let cases = [
        ("0001-01-01T00:00:00", -62_135_629_200_000_000i64),
        ("9999-12-31T23:59:59", 253_402_268_399_000_000i64),
    ];
    for (local, micros) in cases {
        let mut pairs = fields(0);
        set(&mut pairs, "process_expiry", local);
        let actual = command(0, &pairs);
        assert_eq!(
            exact_time_us(actual.adoption().unwrap().expiry()).unwrap(),
            micros
        );
    }
    let expected = Date::from_calendar_date(2028, Month::February, 29)
        .unwrap()
        .with_hms(0, 15, 7)
        .unwrap()
        .assume_offset(UtcOffset::from_hms(9, 0, 0).unwrap())
        .to_offset(UtcOffset::UTC);
    assert_eq!(
        command(0, &fields(0)).adoption().unwrap().expiry(),
        expected
    );
}
#[test]
fn group_form_invalid_expiry_preserves_every_raw_input_and_original_pin() {
    for invalid in [
        "",
        "0000-01-01T00:00:00",
        "10000-01-01T00:00:00",
        "1900-02-29T00:00:00",
        "2027-02-29T00:00:00",
        "2028-00-01T00:00:00",
        "2028-13-01T00:00:00",
        "2028-01-00T00:00:00",
        "2028-01-32T00:00:00",
        "2028-01-01T24:00:00",
        "2028-01-01T00:60:00",
        "2028-01-01T00:00:60",
        "2028-01-01T00:00",
        "2028-01-01t00:00:00",
        "2028-01-01T00:00:00Z",
        "2028-01-01T00:00:00+09:00",
        "2028-01-01T00:00:00.000",
        "２０２８-01-01T00:00:00",
    ] {
        let mut pairs = fields(0);
        set(&mut pairs, "process_expiry", invalid);
        set(&mut pairs, "expected_group_identity_policy_revision", "1");
        set(&mut pairs, "expected_prior_process_revision", "0");
        let actual = draft(0, &pairs);
        assert_adopt_raw(&actual, &pairs);
    }
}
#[test]
fn group_form_content_semantic_failure_preserves_all_raw_including_crlf_whitespace_and_pins() {
    for (key, invalid) in [
        ("title", " \t "),
        ("title", ""),
        ("method", "UNKNOWN_METHOD"),
        ("method", ""),
        ("title", "control\0title"),
        (PROSE_KEYS[0], "lone\rcarriage"),
    ]
    .into_iter()
    .chain(PROSE_KEYS.into_iter().map(|key| (key, " \n\t ")))
    {
        let mut pairs = fields(0);
        set(&mut pairs, key, invalid);
        if key != PROSE_KEYS[7] {
            set(&mut pairs, PROSE_KEYS[7], "원문\r\n  다음 줄\t보존 ");
        }
        let actual = draft(0, &pairs);
        assert_adopt_raw(&actual, &pairs);
    }
}
#[test]
fn group_form_raw_and_canonical_text_limits_use_utf8_bytes_and_exact_aggregate() {
    let mut pairs = fields(0);
    set(&mut pairs, "title", "한".repeat(40));
    assert_eq!(value(&pairs, "title").len(), 120);
    command(0, &pairs);
    set(&mut pairs, "title", "한".repeat(41));
    assert_adopt_raw(&draft(0, &pairs), &pairs);
    set(&mut pairs, "title", "한".repeat(80));
    assert_eq!(value(&pairs, "title").len(), 240);
    assert_adopt_raw(&draft(0, &pairs), &pairs);
    set(&mut pairs, "title", "한".repeat(81));
    rejects(0, &encode(&pairs), FormError::TooLarge);
    for key in PROSE_KEYS {
        let mut pairs = fields(0);
        set(&mut pairs, key, format!("{}xy", "한".repeat(682)));
        command(0, &pairs);
        set(&mut pairs, key, "한".repeat(683));
        assert_adopt_raw(&draft(0, &pairs), &pairs);
        set(&mut pairs, key, format!("{}x", "한".repeat(1365)));
        assert_eq!(value(&pairs, key).len(), 4096);
        assert_adopt_raw(&draft(0, &pairs), &pairs);
        set(&mut pairs, key, "한".repeat(1366));
        rejects(0, &encode(&pairs), FormError::TooLarge);
    }
    let mut pairs = fields(0);
    set(&mut pairs, "title", "t".repeat(120));
    for key in &PROSE_KEYS[..7] {
        set(&mut pairs, key, "p".repeat(2048));
    }
    set(&mut pairs, PROSE_KEYS[7], "p".repeat(1886));
    assert_eq!(
        value(&pairs, "title").len()
            + METHOD.len()
            + PROSE_KEYS
                .iter()
                .map(|k| value(&pairs, k).len())
                .sum::<usize>(),
        16384
    );
    command(0, &pairs);
    set(&mut pairs, PROSE_KEYS[7], "p".repeat(1887));
    assert_adopt_raw(&draft(0, &pairs), &pairs);
    let mut pairs = fields(0);
    set(&mut pairs, "title", "t");
    for key in &PROSE_KEYS[..7] {
        set(&mut pairs, key, "p".repeat(4096));
    }
    set(&mut pairs, PROSE_KEYS[7], "p".repeat(4053));
    assert_eq!(
        1 + METHOD.len()
            + PROSE_KEYS
                .iter()
                .map(|k| value(&pairs, k).len())
                .sum::<usize>(),
        32768
    );
    assert_adopt_raw(&draft(0, &pairs), &pairs);
    set(&mut pairs, PROSE_KEYS[7], "p".repeat(4054));
    rejects(0, &encode(&pairs), FormError::TooLarge);
}
#[test]
fn group_form_new_crlf_normalizes_once_and_stored_codec_preserves_original_bytes() {
    let mut pairs = fields(0);
    set(&mut pairs, "title", " 원문\r\n제목 ");
    for key in PROSE_KEYS {
        set(&mut pairs, key, " 첫 줄\r\n다음 줄\t보존 ");
    }
    let actual = command(0, &pairs);
    let content = actual.adoption().unwrap().content();
    assert_eq!(content, &expected_content(&pairs));
    assert_eq!(content.title(), " 원문\n제목 ");
    assert_eq!(
        content.intended_claimant_matching_procedure(),
        " 첫 줄\n다음 줄\t보존 "
    );
    let actor = AccountId::from_uuid(id(1)).unwrap();
    let wire = actual.encode(actor).unwrap();
    let (returned_actor, reopened) = GroupProcessCommandV1::decode(&wire).unwrap();
    assert_eq!(returned_actor, actor);
    assert_eq!(reopened, actual);
    assert_eq!(reopened.encode(actor).unwrap(), wire);
    // Historical constructor/codec remains exact; only new transport performs
    // CRLF normalization. Existing control rejection must not become repair.
    let historical = ProcessContentV1::from_projection(ProcessContentProjectionV1 {
        title: "historical\r\ntitle".into(),
        method: ProcessMethodV1::AttendedAccountAndDocumentaryReview,
        intended_claimant_matching_procedure: "old".into(),
        account_possession_procedure: "old".into(),
        physical_human_evidence_procedure: "old".into(),
        duplicate_contradictory_claim_procedure: "old".into(),
        qualification_criteria_instruction: "old".into(),
        escalation_adjudication_procedure: "old".into(),
        evidence_minimization_retention_description: "old".into(),
        recipient_responsibility: "old".into(),
    });
    // The existing stored text contract rejects CR as a control, rather than
    // silently reinterpreting or normalizing those historical bytes.
    assert!(historical.is_err());
}
#[test]
fn group_form_suspend_semantic_error_preserves_raw_reason_and_all_exact_head_pins() {
    for reason in ["", " \t\n ", "lone\rreturn", "control\0reason"] {
        let mut pairs = fields(1);
        set(&mut pairs, "reason", reason);
        let actual = draft(1, &pairs);
        assert_original(&actual, &pairs);
        match actual.input {
            DraftInput::Suspend {
                content_version,
                content_digest,
                expected_head_revision,
                expected_head_digest,
                reason: retained,
            } => {
                assert_eq!(content_version, 1);
                assert_eq!(content_digest, [0xab; 32]);
                assert_eq!(expected_head_revision, 2);
                assert_eq!(expected_head_digest, [0xcd; 32]);
                assert_eq!(retained, reason);
            }
            _ => panic!("suspension semantic error changed operation"),
        }
    }
    let mut pairs = fields(1);
    set(&mut pairs, "reason", " 이유\r\n  원문 ");
    assert_eq!(
        command(1, &pairs).suspension().unwrap().reason(),
        " 이유\n  원문 "
    );
    set(&mut pairs, "reason", "r".repeat(2048));
    command(1, &pairs);
    set(&mut pairs, "reason", "r".repeat(2049));
    draft(1, &pairs);
    set(&mut pairs, "reason", "r".repeat(4096));
    draft(1, &pairs);
    set(&mut pairs, "reason", "r".repeat(4097));
    rejects(1, &encode(&pairs), FormError::TooLarge);
}
#[test]
fn group_form_semantic_draft_cannot_hide_malformed_pins_keys_or_proof() {
    let mut base = fields(0);
    set(&mut base, "title", " ");
    for (key, bad) in [
        ("command_id", "nil"),
        ("expected_group_incarnation", "bad"),
        ("process_id", "bad"),
        ("expected_group_revision", "0"),
        ("expected_group_identity_policy_revision", "-1"),
        ("expected_prior_process_revision", "01"),
        ("operator_responsibility", "yes"),
        ("csrf_proof", ""),
    ] {
        let mut pairs = base.clone();
        set(&mut pairs, key, bad);
        rejects(0, &encode(&pairs), FormError::Invalid);
    }
    let mut duplicate = base.clone();
    duplicate.push(("title".into(), "replacement".into()));
    rejects(0, &encode(&duplicate), FormError::Invalid);
    let mut unknown = base;
    unknown.push(("actor".into(), id(9).to_string()));
    rejects(0, &encode(&unknown), FormError::Invalid);
    let mut suspension = fields(1);
    set(&mut suspension, "reason", " ");
    set(&mut suspension, "process_digest", "bad");
    rejects(1, &encode(&suspension), FormError::Invalid);
}

#[test]
fn group_form_raw_expiry_exact64_preserves_validation_draft_and_all_raw_inputs() {
    let mut pairs = fields(0);
    set(
        &mut pairs,
        "process_expiry",
        format!("2028-02-29T00:15:07{}", "x".repeat(45)),
    );
    assert_eq!(value(&pairs, "process_expiry").len(), 64);
    set(&mut pairs, "expected_group_revision", i64::MAX.to_string());
    set(
        &mut pairs,
        "expected_group_identity_policy_revision",
        i64::MAX.to_string(),
    );
    set(
        &mut pairs,
        "expected_prior_process_revision",
        i64::MAX.to_string(),
    );
    set(&mut pairs, PROSE_KEYS[7], "원문\r\n  다음 줄\t보존 ");
    let actual = draft(0, &pairs);
    assert_eq!(actual.errors, vec!["process_expiry"]);
    assert_adopt_raw(&actual, &pairs);
}

#[test]
fn group_form_raw_expiry65_refuses_transport_before_validation() {
    let mut pairs = fields(0);
    set(
        &mut pairs,
        "process_expiry",
        format!("2028-02-29T00:15:07{}", "x".repeat(46)),
    );
    assert_eq!(value(&pairs, "process_expiry").len(), 65);
    assert!(encode(&pairs).len() < 131_072);
    rejects(0, &encode(&pairs), FormError::TooLarge);
}

#[test]
fn group_form_raw_aggregate_counts_unknown_method_at_exact_cap_and_overflow() {
    let mut pairs = fields(0);
    set(&mut pairs, "title", "t");
    for key in PROSE_KEYS {
        set(&mut pairs, key, "p");
    }
    set(&mut pairs, "method", "m".repeat(32_759));
    let raw_sum = |pairs: &[(String, String)]| {
        value(pairs, "title").len()
            + value(pairs, "method").len()
            + PROSE_KEYS
                .iter()
                .map(|key| value(pairs, key).len())
                .sum::<usize>()
    };
    assert_eq!(raw_sum(&pairs), 32_768);
    assert!(encode(&pairs).len() < 131_072);
    let actual = draft(0, &pairs);
    assert_eq!(actual.errors, vec!["method", "content"]);
    assert_adopt_raw(&actual, &pairs);
    set(&mut pairs, "method", "m".repeat(32_760));
    assert_eq!(raw_sum(&pairs), 32_769);
    assert!(encode(&pairs).len() < 131_072);
    rejects(0, &encode(&pairs), FormError::TooLarge);
}
