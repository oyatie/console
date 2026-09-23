//! SSR unit fixtures only; never a product data source or authentication evidence.
use super::native_policy::{Assignment, Form, GrantValidation, Operation, Page, Scope, render};

const COMPANY: &str = "00000000-0000-0000-0000-000000000101";
const RECIPIENT: &str = "00000000-0000-0000-0000-000000000102";
const OTHER: &str = "00000000-0000-0000-0000-000000000103";
const COMMAND: &str = "00000000-0000-0000-0000-000000000104";
const ASSIGNMENT: &str = "00000000-0000-0000-0000-000000000105";
const PROOF: &str = "original-proof-unit-fixture-only";
const INVALID_EXPIRY: &str = "<img src=x onerror=alert(1)>&입력";

fn scope(with_assignment: bool) -> Scope {
    Scope {
        company_name: Some("검토 회사".to_owned()),
        group: "00000000-0000-0000-0000-000000000106".to_owned(),
        company_link: true,
        policy_link: true,
        payroll_link: false,
        company: COMPANY.to_owned(),
        operator: OTHER.to_owned(),
        recipient: RECIPIENT.to_owned(),
        epoch: "9".to_owned(),
        installed: true,
        assignment: with_assignment.then(|| Assignment {
            id: ASSIGNMENT.to_owned(),
            role_revision: "1".to_owned(),
            revision: "3".to_owned(),
            state: "REVOKED",
            label: "회수된 열람 권한",
            from: "2026-09-01 09:00".to_owned(),
            until: "2026-09-10 18:00".to_owned(),
            can_grant: true,
            can_revoke: false,
        }),
    }
}

fn validation(with_assignment: bool, current_matches: bool) -> GrantValidation {
    GrantValidation {
        expected_epoch: "9".to_owned(),
        recipient: RECIPIENT.to_owned(),
        assignment: with_assignment
            .then(|| ("1".to_owned(), ASSIGNMENT.to_owned(), "3".to_owned())),
        expires_at_local: INVALID_EXPIRY.to_owned(),
        current_matches,
    }
}

fn page(scope: Scope, validation: Option<GrantValidation>) -> String {
    render(Page::Form(Form {
        scope,
        operation: Operation::Grant,
        command: COMMAND.to_owned(),
        proof: PROOF.to_owned(),
        validation,
    }))
}

// Follow existing SSR tests: inspect actual serialized elements, no new DOM dependency.
fn input<'a>(html: &'a str, name: &str) -> &'a str {
    let matches: Vec<_> = html
        .split("<input")
        .skip(1)
        .map(|tail| tail.split('>').next().unwrap())
        .filter(|tag| tag.contains(&format!("name=\"{name}\"")))
        .collect();
    assert_eq!(matches.len(), 1, "expected one input named {name}");
    matches[0]
}

fn original_inputs(html: &str, with_assignment: bool) {
    assert_eq!(html.matches("<input").count(), 8);
    for (name, value) in [
        ("command_id", COMMAND),
        ("expected_company_epoch", "9"),
        ("csrf_proof", PROOF),
        ("recipient_account_id", RECIPIENT),
        (
            "assignment_id",
            if with_assignment { ASSIGNMENT } else { "" },
        ),
        (
            "expected_role_revision",
            if with_assignment { "1" } else { "" },
        ),
        (
            "expected_assignment_revision",
            if with_assignment { "3" } else { "" },
        ),
    ] {
        let tag = input(html, name);
        assert!(
            tag.contains("type=\"hidden\""),
            "original field must remain hidden"
        );
        // Never print form proof bytes or the entire rendered document on failure.
        assert!(
            tag.contains(&format!("value=\"{value}\"")),
            "original field value changed: {name}"
        );
    }
    let form = html
        .split("<form")
        .nth(1)
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    assert!(form.contains("method=\"post\""));
    assert!(form.contains(&format!(
        "action=\"/companies/{COMPANY}/policy/payroll-read/grants\""
    )));
    assert_eq!(html.matches("type=\"submit\"").count(), 1);
}

fn escaped_visible_expiry(html: &str) {
    let retained = html
        .split("data-policy-retained-expiry")
        .nth(1)
        .expect("retained text exists");
    let text = retained
        .split_once('>')
        .unwrap()
        .1
        .split("</code>")
        .next()
        .unwrap();
    assert!(text.contains("&lt;img src=x onerror=alert(1)&gt;&amp;입력"));
    assert!(
        !html.contains("<img"),
        "expiry text created an HTML element"
    );
    assert!(
        !html.contains(INVALID_EXPIRY),
        "expiry text escaped its text/attribute context"
    );
}

#[test]
fn native_policy_regular_grant_form_preserves_all_original_inputs() {
    for with_assignment in [false, true] {
        let html = page(scope(with_assignment), None);
        original_inputs(&html, with_assignment);
        let expiry = input(&html, "expires_at_local");
        assert!(expiry.contains("type=\"datetime-local\"") && expiry.contains("value=\"\""));
        assert!(expiry.contains("aria-invalid=\"false\""));
        assert!(expiry.contains("aria-describedby=\"expiry-help\""));
        assert!(!html.contains("role=\"alert\"") && !html.contains("id=\"expiry-error\""));
    }
}

#[test]
fn native_policy_invalid_expiry_retains_original_inputs_and_accessible_escaped_error() {
    for with_assignment in [false, true] {
        let html = page(
            scope(with_assignment),
            Some(validation(with_assignment, true)),
        );
        original_inputs(&html, with_assignment);
        escaped_visible_expiry(&html);
        let expiry = input(&html, "expires_at_local");
        assert!(
            expiry.contains("id=\"policy-expiry\"") && expiry.contains("type=\"datetime-local\"")
        );
        assert!(expiry.contains("aria-invalid=\"true\""));
        assert!(expiry.contains("aria-describedby=\"expiry-help expiry-error\""));
        assert!(expiry.contains("value=\"&lt;img src=x onerror=alert(1)&gt;&amp;입력\""));
        for id in ["expiry-help", "expiry-error"] {
            assert_eq!(html.matches(&format!("id=\"{id}\"")).count(), 1);
        }
        assert!(html.contains("for=\"policy-expiry\""));
        let alert = html.split("role=\"alert\"").nth(1).unwrap();
        assert!(alert.split('>').next().unwrap().contains("tabindex=\"-1\""));
        assert!(alert.contains("이 입력으로 요청은 접수되지 않았습니다."));
    }
}

#[test]
fn native_policy_stale_validation_preserves_original_context_without_resubmit() {
    // App compares these three expectations; rendering consumes that authorized result.
    for changed in ["epoch", "recipient", "assignment"] {
        let mut current = scope(true);
        let fresh_path = match changed {
            "epoch" => {
                current.epoch = "10".to_owned();
                "grant"
            }
            "recipient" => {
                current.recipient = OTHER.to_owned();
                "grant"
            }
            "assignment" => {
                let assignment = current.assignment.as_mut().unwrap();
                assignment.id = OTHER.to_owned();
                assignment.revision = "4".to_owned();
                assignment.state = "ACTIVE";
                assignment.label = "현재 연결된 열람 권한";
                assignment.can_grant = false;
                assignment.can_revoke = true;
                "revoke"
            }
            _ => unreachable!(),
        };
        let html = page(current, Some(validation(true, false)));
        escaped_visible_expiry(&html);
        assert!(
            !html.contains("<form")
                && !html.contains("<input")
                && !html.contains("type=\"submit\"")
        );
        assert!(
            !html.contains(PROOF),
            "stale view leaked a resubmission proof"
        );
        let original = html
            .split("원래 요청")
            .nth(1)
            .unwrap()
            .split("</dl>")
            .next()
            .unwrap();
        for value in [
            COMMAND,
            RECIPIENT,
            ASSIGNMENT,
            "원래 회사 버전",
            "원래 권한 대상",
            "원래 연결",
        ] {
            assert!(original.contains(value), "original context disappeared");
        }
        assert!(
            original.contains(">9<"),
            "original company expectation was replaced"
        );
        assert!(
            original.contains(" / ") && original.contains(">3<"),
            "original assignment revision disappeared"
        );
        assert!(html.contains(&format!(
            "href=\"/companies/{COMPANY}/policy/payroll-read/{fresh_path}\""
        )));
        assert!(!html.contains("href=\"#\"") && !html.contains("disabled"));
    }
}

#[test]
fn native_policy_documents_preserve_post_origin_and_private_security_headers() {
    use super::native_policy::{document, OriginalRecord, Outcome};
    use axum::http::{header, StatusCode};
    let mut pages = vec![];
    for operation in [Operation::Install, Operation::Grant, Operation::Revoke] {
        let mut current = scope(true);
        let action = match operation {
            Operation::Install => {
                current.installed = false;
                current.assignment = None;
                format!("/companies/{COMPANY}/policy/payroll-read/catalog")
            }
            Operation::Grant => format!("/companies/{COMPANY}/policy/payroll-read/grants"),
            Operation::Revoke => {
                let assignment = current.assignment.as_mut().unwrap();
                assignment.state = "ACTIVE";
                assignment.label = "연결됨";
                assignment.can_revoke = true;
                assignment.can_grant = false;
                format!("/companies/{COMPANY}/policy/payroll-read/grants/{ASSIGNMENT}/revoke")
            }
        };
        pages.push((Page::Form(Form {
            scope: current, operation, command: COMMAND.into(), proof: PROOF.into(), validation: None,
        }), StatusCode::OK, Some(action)));
    }
    pages.push((Page::Form(Form {
        scope: scope(true), operation: Operation::Grant, command: COMMAND.into(), proof: PROOF.into(),
        validation: Some(validation(true, true)),
    }), StatusCode::UNPROCESSABLE_ENTITY, Some(format!("/companies/{COMPANY}/policy/payroll-read/grants"))));
    pages.push((Page::Result {
        scope: scope(true), operation: Operation::Grant, command: COMMAND.into(),
        outcome: Outcome::Pending { accepted_at: "2026-09-23 09:00 KST".into(),
            deadline: "2026-09-30 09:00 KST".into(), proof: PROOF.into() },
        original: OriginalRecord { accepted_at: "2026-09-23 09:00 KST".into(),
            deadline: "2026-09-30 09:00 KST".into(), intake_receipt: COMMAND.into(),
            expected_company_epoch: "9".into(), expected_assignment: None, requested_until: None,
            effect_period: None, effect_epochs: None },
    }, StatusCode::OK, Some(format!("/companies/{COMPANY}/policy/payroll-read/requests/grant/{COMMAND}/retry"))));
    pages.push((Page::Unavailable, StatusCode::SERVICE_UNAVAILABLE, None));
    for (page, status, action) in pages {
        let response = document(page, status);
        assert_eq!(response.status(), status);
        let headers = response.headers();
        for (name, value) in [
            (header::REFERRER_POLICY, "same-origin"),
            (header::CACHE_CONTROL, "no-store"),
            (header::PRAGMA, "no-cache"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::VARY, "Authorization, Cookie, Origin"),
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CONTENT_SECURITY_POLICY, "default-src 'self'; script-src 'none'; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'"),
        ] {
            let values: Vec<_> = headers.get_all(name.clone()).iter().collect();
            assert_eq!(values.len(), 1, "missing or duplicate {name}");
            assert_eq!(values[0], value, "unexpected {name}");
        }
        assert!(!headers.contains_key(header::SET_COOKIE));
        use std::{future::Future, task::{Context, Poll, Waker}};
        let mut body = std::pin::pin!(axum::body::to_bytes(response.into_body(), 256 * 1024));
        let Poll::Ready(bytes) = body.as_mut().poll(&mut Context::from_waker(Waker::noop())) else {
            panic!("buffered native policy response unexpectedly pending");
        };
        let html = String::from_utf8(bytes.unwrap().to_vec()).unwrap();
        let forms: Vec<_> = html.split("<form").skip(1).map(|s| s.split('>').next().unwrap()).collect();
        if let Some(action) = action {
            assert_eq!(forms.len(), 1, "fixture must expose a real POST form");
            assert!(forms[0].contains("method=\"post\""));
            assert!(forms[0].contains(&format!("action=\"{action}\"")));
        } else {
            assert!(forms.is_empty(), "error document exposed a mutation form");
        }
    }
}
