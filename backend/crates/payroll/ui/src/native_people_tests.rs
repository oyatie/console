//! Renderer fixtures only: no business provisioning or authorization evidence.
use super::native_people::{
    Expectations, Outcome, Page, Record, Registration, Request, Scope, document, render,
};
use axum::http::{StatusCode, header};

const COMPANY: &str = "00000000-0000-0000-0000-000000000101";
const COMMAND: &str = "00000000-0000-0000-0000-000000000102";
const EMPLOYEE: &str = "00000000-0000-0000-0000-000000000103";
const PERSON: &str = "00000000-0000-0000-0000-000000000104";
const PROOF: &str = "unit-form-proof-only";
const NAME: &str = "<img src=x onerror=alert(1)>&김하늘";

fn scope(read: bool, create: bool) -> Scope {
    Scope {
        company: COMPANY.into(),
        company_name: Some("검토 회사".into()),
        company_link: true,
        directory_link: read,
        can_create: create,
        payroll_link: false,
        policy_link: false,
    }
}

fn request(outcome: Outcome) -> Request {
    Request {
        command: COMMAND.into(),
        legal_name: NAME.into(),
        employee_number: "사번&<1>".into(),
        accepted_at: "2026-09-23 10:00".into(),
        deadline: "2026-09-30 10:00".into(),
        intake_receipt: "unit-intake-receipt".into(),
        expected_company_epoch: "3".into(),
        outcome,
    }
}

fn committed() -> Outcome {
    Outcome::Committed {
        employee_id: EMPLOYEE.into(),
        person_id: PERSON.into(),
        receipt: "unit-effect-receipt".into(),
        registered_at: "2026-09-23 10:01".into(),
    }
}

fn input<'a>(html: &'a str, name: &str) -> &'a str {
    let tags: Vec<_> = html
        .split("<input")
        .skip(1)
        .map(|s| s.split('>').next().unwrap())
        .filter(|s| s.contains(&format!("name=\"{name}\"")))
        .collect();
    assert_eq!(tags.len(), 1, "input must be unique: {name}");
    tags[0]
}

#[test]
fn native_people_validation_preserves_input_and_expectations_with_accessible_errors() {
    let html = render(Page::Registration {
        scope: scope(false, true),
        form: Registration {
            command: COMMAND.into(),
            proof: PROOF.into(),
            expected: Expectations {
                company_epoch: "3".into(),
                object_type_id: "unit-object".into(),
                action_type_id: "unit-action".into(),
                action_revision: "4".into(),
                schema_revision: "5".into(),
                legal_name_property_id: "unit-name-property".into(),
                employee_number_property_id: "unit-number-property".into(),
            },
            legal_name: NAME.into(),
            employee_number: "사번&<1>\" autofocus onfocus=alert(1)".into(),
            name_error: Some("이름을 확인하세요"),
            number_error: Some("사번을 확인하세요"),
            form_error: None,
        },
    });
    for (name, value) in [
        ("command_id", COMMAND),
        ("csrf_proof", PROOF),
        ("expected_company_epoch", "3"),
        ("object_type_id", "unit-object"),
        ("action_type_id", "unit-action"),
        ("expected_action_revision", "4"),
        ("expected_schema_revision", "5"),
        ("legal_name_property_id", "unit-name-property"),
        ("employee_number_property_id", "unit-number-property"),
    ] {
        let tag = input(&html, name);
        assert!(tag.contains("type=\"hidden\""));
        assert!(
            tag.contains(&format!("value=\"{value}\"")),
            "original expectation changed: {name}"
        );
    }
    assert!(
        input(&html, "legal_name")
            .contains("value=\"&lt;img src=x onerror=alert(1)&gt;&amp;김하늘\"")
    );
    assert!(
        input(&html, "employee_number")
            .contains("value=\"사번&amp;&lt;1&gt;&quot; autofocus onfocus=alert(1)\"")
    );
    for (name, id) in [
        ("legal_name", "people-name"),
        ("employee_number", "people-number"),
    ] {
        let tag = input(&html, name);
        assert!(tag.contains("aria-invalid=\"true\""));
        assert!(tag.contains(&format!("aria-describedby=\"{id}-help {id}-error\"")));
        for suffix in ["help", "error"] {
            assert_eq!(html.matches(&format!("id=\"{id}-{suffix}\"")).count(), 1);
        }
    }
    let alert = html
        .split("<section")
        .find(|s| s.contains("id=\"people-input-errors\""))
        .unwrap();
    let tag = alert.split('>').next().unwrap();
    assert!(
        tag.contains("role=\"alert\"")
            && tag.contains("tabindex=\"-1\"")
            && tag.contains("autofocus")
    );
    assert!(html.find("등록하면 무엇이 달라지나요?").unwrap() < html.find("<form").unwrap());
    assert!(html.contains(&format!("action=\"/companies/{COMPANY}/people/requests\"")));
    assert!(!html.contains("<img") && !html.contains("<script") && !html.contains("/pkg/"));
    assert!(!html.contains(&format!("href=\"/companies/{COMPANY}/people\"")));
}

#[test]
fn native_people_receipts_preserve_command_and_current_read_navigation() {
    let pending = render(Page::Request {
        scope: scope(false, true),
        request: request(Outcome::Pending {
            proof: PROOF.into(),
        }),
    });
    let forms: Vec<_> = pending
        .split("<form")
        .skip(1)
        .map(|tail| tail.split("</form>").next().unwrap())
        .collect();
    assert_eq!(forms.len(), 2);
    for action in ["execute", "cancel"] {
        let expected =
            format!("action=\"/companies/{COMPANY}/people/requests/{COMMAND}/{action}\"");
        let matching: Vec<_> = forms
            .iter()
            .filter(|body| body.split('>').next().unwrap().contains(&expected))
            .collect();
        assert_eq!(matching.len(), 1, "one form per operation");
        let form = matching[0];
        assert!(form.split('>').next().unwrap().contains("method=\"post\""));
        assert_eq!(form.matches("<input").count(), 2);
        for (name, value) in [("command_id", COMMAND), ("csrf_proof", PROOF)] {
            let tag = input(form, name);
            assert!(tag.contains("type=\"hidden\""));
            assert!(
                tag.contains(&format!("value=\"{value}\"")),
                "original form binding changed"
            );
        }
    }
    assert!(pending.contains("data-people-outcome=\"pending\""));
    for read in [false, true] {
        let html = render(Page::Request {
            scope: scope(read, true),
            request: request(committed()),
        });
        assert!(!html.contains("<form") && !html.contains(PROOF));
        assert!(html.contains("data-people-outcome=\"committed\""));
        assert!(html.contains("이 등록으로 고용이나 발령이 생성되지는 않았습니다."));
        assert_eq!(
            html.contains(&format!("href=\"/companies/{COMPANY}/people/{EMPLOYEE}\"")),
            read
        );
        assert!(!html.contains(&format!("href=\"/companies/{COMPANY}/people/{PERSON}\"")));
        assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;&amp;김하늘"));
        assert!(!html.contains("<img"));
    }
    for (outcome, state) in [
        (Outcome::Cancelled, "cancelled"),
        (Outcome::Expired, "expired"),
        (
            Outcome::Rejected {
                reason: "등록하지 않았습니다",
            },
            "rejected",
        ),
        (
            Outcome::Conflicting {
                reason: "새 요청을 작성하세요",
            },
            "conflicting",
        ),
    ] {
        let html = render(Page::Request {
            scope: scope(false, true),
            request: request(outcome),
        });
        assert!(!html.contains("<form") && !html.contains(PROOF));
        assert!(html.contains(&format!("data-people-outcome=\"{state}\"")));
        assert!(html.contains(&format!("href=\"/companies/{COMPANY}/people/new\"")));
        assert!(!html.contains(EMPLOYEE) && !html.contains(PERSON));
    }
    let html = render(Page::Uncertain {
        company: COMPANY.into(),
        command: COMMAND.into(),
    });
    assert!(html.contains(&format!(
        "href=\"/companies/{COMPANY}/people/requests/{COMMAND}\""
    )));
    assert!(!html.contains("<form") && !html.contains("/people/new"));
}

#[test]
fn native_people_directory_uses_entry_identity_and_honest_empty_page() {
    let html = render(Page::Directory {
        scope: scope(true, false),
        records: vec![Record {
            employee_id: EMPLOYEE.into(),
            person_id: PERSON.into(),
            legal_name: NAME.into(),
            employee_number: "사번&<1>".into(),
            person_version: "2".into(),
            registered_at: "2026-09-23 10:01".into(),
        }],
        next_after: Some(EMPLOYEE.into()),
    });
    assert!(html.contains(&format!("href=\"/companies/{COMPANY}/people/{EMPLOYEE}\"")));
    assert!(!html.contains(&format!("href=\"/companies/{COMPANY}/people/{PERSON}\"")));
    assert!(html.contains(&format!("?after_employee_id={EMPLOYEE}")));
    assert!(!html.contains("/people/new") && !html.contains("<form") && !html.contains("<img"));
    assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;&amp;김하늘"));
    for create in [false, true] {
        let html = render(Page::Directory {
            scope: scope(true, create),
            records: vec![],
            next_after: None,
        });
        assert!(html.contains("표시할 사람이 없습니다"));
        assert!(
            !html.contains("등록된 사람이 없습니다")
                && !html.contains("첫 기록")
                && !html.contains("?after_employee_id")
        );
        assert_eq!(html.contains("/people/new"), create);
    }
}

#[test]
fn native_people_denials_omit_identity_and_documents_remain_private_and_static() {
    for page in [Page::Refused, Page::Unavailable] {
        let html = render(page);
        for forbidden in [
            COMPANY,
            COMMAND,
            EMPLOYEE,
            PERSON,
            "검토 회사",
            "<form",
            "data-people-",
            "<script",
            "/pkg/",
        ] {
            assert!(
                !html.contains(forbidden),
                "denied document exposed context or actions"
            );
        }
    }
    for (page, status) in [
        (Page::Refused, StatusCode::NOT_FOUND),
        (Page::Unavailable, StatusCode::SERVICE_UNAVAILABLE),
        (
            Page::Request {
                scope: scope(false, true),
                request: request(Outcome::Pending {
                    proof: PROOF.into(),
                }),
            },
            StatusCode::OK,
        ),
    ] {
        let response = document(page, status);
        assert_eq!(response.status(), status);
        let h = response.headers();
        assert_eq!(h[header::CACHE_CONTROL], "no-store");
        assert_eq!(h[header::VARY], "Authorization, Cookie, Origin");
        assert_eq!(h[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
        assert_eq!(h[header::REFERRER_POLICY], "same-origin");
        assert!(!h.contains_key(header::SET_COOKIE));
        let csp = h[header::CONTENT_SECURITY_POLICY].to_str().unwrap();
        for directive in [
            "script-src 'none'",
            "frame-ancestors 'none'",
            "form-action 'self'",
            "base-uri 'none'",
        ] {
            assert!(csp.split(';').any(|s| s.trim() == directive));
        }
    }
}
