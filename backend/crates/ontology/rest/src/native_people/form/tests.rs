use super::*;
const COMPANY: &str = "11111111-1111-4111-8111-111111111111";
const COMMAND: &str = "22222222-2222-4222-8222-222222222222";
fn original() -> String {
    format!(
        "csrf_proof=original-proof&command_id={COMMAND}&expected_company_epoch=7&object_type_id=33333333-3333-4333-8333-333333333333&action_type_id=44444444-4444-4444-8444-444444444444&expected_action_revision=2&expected_schema_revision=3&legal_name_property_id=55555555-5555-4555-8555-555555555555&employee_number_property_id=66666666-6666-4666-8666-666666666666&legal_name=+%EA%B9%80%ED%95%98%EB%8A%98+&employee_number=+K-001+"
    )
}
#[test]
fn native_people_form_keeps_raw_values_proof_expectations_and_uses_shared_validation() {
    let parsed = parse(
        company(COMPANY).unwrap(),
        PostTarget::Prepare,
        original().as_bytes(),
    )
    .unwrap();
    assert_eq!(parsed.proof, "original-proof");
    let Input::Prepare {
        draft,
        submission: Some(input),
    } = parsed.input
    else {
        panic!("typed submission");
    };
    assert_eq!(draft.legal_name, " 김하늘 ");
    assert_eq!(draft.employee_number, " K-001 ");
    assert_eq!(draft.expected, input.expected());
    assert_eq!(draft.expected.company_epoch, 7);
    assert_eq!(draft.locator, input.locator());
    assert_eq!(input.input().legal_name(), "김하늘");
    assert_eq!(input.input().employee_number(), "K-001");
    let invalid = original()
        .replace("+%EA%B9%80%ED%95%98%EB%8A%98+", "%09")
        .replace("+K-001+", "+");
    let parsed = parse(
        company(COMPANY).unwrap(),
        PostTarget::Prepare,
        invalid.as_bytes(),
    )
    .unwrap();
    assert_eq!(parsed.proof, "original-proof");
    let Input::Prepare {
        draft,
        submission: None,
    } = parsed.input
    else {
        panic!("validation draft");
    };
    assert_eq!(draft.legal_name, "\t");
    assert_eq!(draft.employee_number, " ");
    assert_eq!(draft.expected.company_epoch, 7);
    assert_eq!(
        draft.name_error,
        Some(DirectoryInputProblem::ControlCharacter)
    );
    assert_eq!(draft.number_error, Some(DirectoryInputProblem::Required));
}
#[test]
fn native_people_form_refuses_aliases_duplicates_bad_encoding_and_route_command_mismatch() {
    let original = original();
    let company = company(COMPANY).unwrap();
    for hostile in [
        format!("{original}&legal_name=x"),
        format!("{original}&ignored=1"),
        original.replace("legal_name=", "legal%5fname="),
        original.replace("original-proof", "%FF"),
        original.replace("original-proof", "%0"),
        original.replace("expected_company_epoch=7", "expected_company_epoch=+7"),
        original.replace(COMMAND, "22222222-2222-4222-8222-22222222222A"),
        original.replace("csrf_proof=original-proof&", ""),
    ] {
        assert!(parse(company, PostTarget::Prepare, hostile.as_bytes()).is_err());
    }
    assert!(matches!(
        parse(company, PostTarget::Prepare, &vec![b'x'; MAX_BODY + 1]),
        Err(StatusCode::PAYLOAD_TOO_LARGE)
    ));
    let terminal = format!("csrf_proof=original-proof&command_id={COMMAND}");
    assert!(matches!(
        parse(company, PostTarget::Execute(COMMAND), terminal.as_bytes())
            .unwrap()
            .input,
        Input::Execute(_)
    ));
    assert!(matches!(
        parse(company, PostTarget::Cancel(COMMAND), terminal.as_bytes())
            .unwrap()
            .input,
        Input::Cancel(_)
    ));
    assert!(matches!(
        parse(company, PostTarget::Execute(COMPANY), terminal.as_bytes()),
        Err(StatusCode::BAD_REQUEST)
    ));
    assert!(
        parse(
            company,
            PostTarget::Cancel(COMMAND),
            format!("{terminal}&legal_name=x").as_bytes()
        )
        .is_err()
    );
}
#[test]
fn native_people_routes_and_cursor_accept_only_exact_non_nil_selectors() {
    assert_eq!(pagination(None).unwrap().limit(), 25);
    assert_eq!(
        pagination(Some(&format!("after_employee_id={COMMAND}")))
            .unwrap()
            .after(),
        Some(id(COMMAND).unwrap())
    );
    for raw in [
        "",
        "limit=100",
        "after_employee_id=",
        "after_employee_id=00000000-0000-0000-0000-000000000000",
        "after_employee_id=%32",
        "after_employee_id=22222222-2222-4222-8222-222222222222&after_employee_id=22222222-2222-4222-8222-222222222222",
    ] {
        assert!(pagination(Some(raw)).is_err());
    }
    assert!(company("00000000-0000-0000-0000-000000000000").is_err());
    assert!(company(&OrgId::platform().to_string()).is_err());
    assert!(id("22222222222242228222222222222222").is_err());
    assert!(no_query(Some("")).is_err());
    assert!(no_query(None).is_ok());
}

#[test]
fn native_people_exact_number_query_decodes_once_and_rejects_unbounded_or_ambiguous_inputs() {
    for (raw, expected) in [
        ("employee_number=A%2BB", Some("A+B")),
        ("employee_number=A+B", Some("A B")),
        ("employee_number=A%26B", Some("A&B")),
        ("employee_number=%ED%95%9C", Some("한")),
        ("employee_number=", None),
    ] {
        assert_eq!(pagination(Some(raw)).unwrap().employee_number(), expected);
    }
    let page = pagination(Some(&format!(
        "employee_number=A%2BB&after_employee_id={COMMAND}"
    )))
    .unwrap();
    assert_eq!(page.employee_number(), Some("A+B"));
    assert_eq!(page.after(), Some(id(COMMAND).unwrap()));
    assert_eq!(page.limit(), 25);
    for raw in [
        "employee_number=A&employee_number=A",
        "employee_number=A&ignored=1",
        "employee%5Fnumber=A",
        "employee_number=%",
        "employee_number=%GG",
        "employee_number=%FF",
        "employee_number=%00",
        "employee_number=A%0AB",
        "employee_number=%2526",
    ] {
        if raw == "employee_number=%2526" {
            // A single decode produces literal "%26", never an ampersand.
            assert_eq!(
                pagination(Some(raw)).unwrap().employee_number(),
                Some("%26")
            );
        } else {
            assert!(pagination(Some(raw)).is_err(), "accepted {raw}");
        }
    }
    assert!(pagination(Some(&format!("employee_number={}", "x".repeat(65)))).is_err());
    assert!(pagination(Some(&format!("employee_number={}", "x".repeat(1024)))).is_err());
}
