//! Static presentation fixtures only; never business provisioning or auth evidence.
//! Contract: adopted design d11e6f8c1755ef08e3a654d356b37b42d5298a0f34636f89f8733f24032d791a.
#![cfg(feature = "ssr")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use axum::http::{StatusCode, header};
use console_payroll_ui::native_people::{
    Expectations, Outcome, Page, Record, Registration, Request, Scope, document, render,
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    future::Future,
    task::{Context, Poll, Waker},
};

const COMPANY: &str = "00000000-0000-0000-0000-000000000101";
const COMMAND: &str = "00000000-0000-0000-0000-000000000102";
const EMPLOYEE: &str = "00000000-0000-0000-0000-000000000103";
const PERSON: &str = "00000000-0000-0000-0000-000000000104";
const RECEIPT: &str = "00000000-0000-0000-0000-000000000105";
const PROOF: &str = "presentation-fixture-proof-only";
const NUMBER: &str = "사람&001";
const HOSTILE: &str = "</script><img src=x onerror=alert(1)>&\"김하늘\u{2028}\u{2029}";
const BASELINE_NAME: &str = "김하늘";
const BOOTSTRAP_ID: &str = "console-people-bootstrap";
const FALLBACK_ID: &str = "console-people-fallback";
const GUARD_PATH: &str = "/assets/people-guard.js";
const RUNTIME_PATH: &str = "/assets/people.js";

fn scope(read: bool, create: bool) -> Scope {
    Scope {
        company: COMPANY.into(),
        company_name: Some("현재 검토 회사".into()),
        company_link: true,
        directory_link: read,
        can_create: create,
        payroll_link: false,
        policy_link: false,
    }
}
fn record(name: &str) -> Record {
    Record {
        employee_id: EMPLOYEE.into(),
        person_id: PERSON.into(),
        legal_name: Some(name.into()),
        employee_number: Some(NUMBER.into()),
        person_version: "9007199254740993".into(),
        registered_at: "2026-10-07 10:00:00 KST".into(),
    }
}
fn registration(name: &str) -> Page {
    Page::Registration {
        scope: scope(true, true),
        form: Registration {
            command: COMMAND.into(),
            proof: PROOF.into(),
            expected: Expectations {
                company_epoch: "9007199254740993".into(),
                object_type_id: "00000000-0000-0000-0000-000000000201".into(),
                action_type_id: "00000000-0000-0000-0000-000000000202".into(),
                action_revision: "4".into(),
                schema_revision: "5".into(),
                legal_name_property_id: "00000000-0000-0000-0000-000000000203".into(),
                employee_number_property_id: "00000000-0000-0000-0000-000000000204".into(),
            },
            legal_name: name.into(),
            employee_number: NUMBER.into(),
            name_error: None,
            number_error: None,
            form_error: None,
        },
    }
}
fn request(outcome: Outcome) -> Page {
    Page::Request {
        scope: scope(true, true),
        request: Request {
            command: COMMAND.into(),
            legal_name: BASELINE_NAME.into(),
            employee_number: NUMBER.into(),
            accepted_at: "2026-10-07 10:00:00 KST".into(),
            deadline: "2026-10-08 10:00:00 KST".into(),
            intake_receipt: RECEIPT.into(),
            expected_company_epoch: "3".into(),
            outcome,
        },
    }
}
fn directory() -> Page {
    Page::Directory {
        scope: scope(true, true),
        records: vec![record(BASELINE_NAME)],
        search_number: Some(NUMBER.into()),
        next_href: Some(format!(
            "/companies/{COMPANY}/people?employee_number=%EC%82%AC%EB%9E%8C%26001&after_employee_id={EMPLOYEE}"
        )),
        after_cursor: false,
    }
}
fn html(page: Page) -> String {
    // private_document uses a buffered HTML body. A pending/streaming body fails
    // explicitly; no executor, runtime or extra test dependency is necessary.
    let response = document(page, StatusCode::OK);
    let mut future = std::pin::pin!(axum::body::to_bytes(response.into_body(), 512 * 1024));
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(Ok(bytes)) => String::from_utf8(bytes.to_vec()).unwrap(),
        Poll::Ready(Err(_)) => panic!("REACT_PEOPLE_DOCUMENT_BODY_UNREADABLE"),
        Poll::Pending => panic!("REACT_PEOPLE_DOCUMENT_BODY_NOT_BUFFERED"),
    }
}
fn script<'a>(html: &'a str, attribute: &str) -> (&'a str, &'a str) {
    let matches: Vec<_> = html
        .split("<script")
        .skip(1)
        .filter_map(|tail| {
            let (tag, rest) = tail.split_once('>')?;
            tag.contains(attribute)
                .then(|| (tag, rest.split_once("</script>").unwrap().0))
        })
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "REACT_PEOPLE_BOOTSTRAP_MISSING_OR_DUPLICATED: {attribute}"
    );
    matches[0]
}
fn bootstrap(html: &str) -> (&str, Value) {
    let (tag, raw) = script(html, &format!("id=\"{BOOTSTRAP_ID}\""));
    assert!(
        tag.contains("type=\"application/json\""),
        "REACT_PEOPLE_BOOTSTRAP_NOT_INERT"
    );
    let value: Value = serde_json::from_str(raw).expect("REACT_PEOPLE_BOOTSTRAP_INVALID_JSON");
    keys(&value, &["version", "page"]);
    assert_eq!(value["version"], 1, "REACT_PEOPLE_BOOTSTRAP_VERSION");
    (raw, value)
}
fn keys(value: &Value, expected: &[&str]) {
    let actual: BTreeSet<_> = value
        .as_object()
        .expect("object contract")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        actual,
        expected.iter().copied().collect(),
        "REACT_PEOPLE_CLOSED_PROJECTION_FIELDS"
    );
}
fn body(html: &str) -> &str {
    html.split_once("<body")
        .unwrap()
        .1
        .split_once('>')
        .unwrap()
        .1
        .split_once("</body>")
        .unwrap()
        .0
}
fn no_proof(value: &Value) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                assert!(
                    !matches!(key.as_str(), "proof" | "csrf_proof"),
                    "REACT_PEOPLE_PROOF_ON_NONACTIONABLE_VARIANT"
                );
                no_proof(value);
            }
        }
        Value::Array(values) => {
            for value in values {
                no_proof(value);
            }
        }
        _ => {}
    }
}

#[test]
fn react_people_document_binds_guard_projection_and_runtime() {
    let result = html(registration(BASELINE_NAME));
    bootstrap(&result); // Actual document boundary: semantic RED on unchanged source.
    assert_eq!(result.matches(&format!("id=\"{FALLBACK_ID}\"")).count(), 1);
    assert!(
        result.contains(body(&render(registration(BASELINE_NAME)))),
        "REACT_PEOPLE_FALLBACK_CHANGED"
    );
    let (guard, _) = script(&result, &format!("src=\"{GUARD_PATH}\""));
    assert!(
        !guard.contains("async") && !guard.contains("defer") && !guard.contains("type=\"module\""),
        "REACT_PEOPLE_GUARD_NOT_PARSER_BLOCKING"
    );
    assert!(
        result.find(GUARD_PATH).unwrap() < result.find("<body").unwrap(),
        "REACT_PEOPLE_GUARD_AFTER_INTERACTIVE_BODY"
    );
    script(&result, &format!("src=\"{RUNTIME_PATH}\""));
    assert_eq!(
        result.matches("<script").count(),
        3,
        "REACT_PEOPLE_UNDECLARED_SCRIPT"
    );
    assert!(!result.contains("dangerouslySetInnerHTML") && !result.contains("src=\"https://"));
}

#[test]
fn react_people_projection_roundtrips_exact_authorized_inputs_and_revisions() {
    let result = html(registration(BASELINE_NAME));
    let (_, value) = bootstrap(&result);
    let page = &value["page"];
    keys(page, &["kind", "scope", "form"]);
    assert_eq!(page["kind"], "registration");
    assert_eq!(page["scope"]["company"], COMPANY);
    assert_eq!(page["form"]["legal_name"], BASELINE_NAME);
    assert_eq!(page["form"]["employee_number"], NUMBER);
    assert_eq!(page["form"]["command"], COMMAND);
    assert_eq!(page["form"]["proof"], PROOF);
    assert_eq!(
        page["form"]["expected"]["company_epoch"],
        "9007199254740993"
    );
    assert_eq!(page["form"]["expected"]["action_revision"], "4");
    assert_eq!(page["form"]["expected"]["schema_revision"], "5");
}

#[test]
fn react_people_bootstrap_escapes_script_terminators_and_unicode_without_data_loss() {
    let result = html(registration(HOSTILE));
    let (raw, value) = bootstrap(&result);
    for forbidden in ['<', '>', '&', '\u{2028}', '\u{2029}'] {
        assert!(
            !raw.contains(forbidden),
            "REACT_PEOPLE_UNESCAPED_BOOTSTRAP_CHARACTER: {forbidden:?}"
        );
    }
    assert_eq!(value["page"]["form"]["legal_name"], HOSTILE);
    assert!(
        !result.contains("<img src=x") && !result.contains("<script>"),
        "REACT_PEOPLE_EXECUTABLE_PERSONAL_VALUE"
    );
}

#[test]
fn react_people_authorized_and_recovery_page_variants_use_closed_versioned_projection() {
    let cases = [
        (
            directory(),
            "directory",
            vec![
                "kind",
                "scope",
                "records",
                "search_number",
                "next_href",
                "after_cursor",
            ],
        ),
        (
            registration(BASELINE_NAME),
            "registration",
            vec!["kind", "scope", "form"],
        ),
        (
            Page::RegistrationConflict {
                scope: scope(true, true),
                command: COMMAND.into(),
                legal_name: BASELINE_NAME.into(),
                employee_number: NUMBER.into(),
            },
            "registration_conflict",
            vec!["kind", "scope", "command", "legal_name", "employee_number"],
        ),
        (
            request(Outcome::Pending {
                proof: PROOF.into(),
            }),
            "request",
            vec!["kind", "scope", "request"],
        ),
        (
            Page::RequestNotVisible {
                scope: scope(false, true),
                command: COMMAND.into(),
            },
            "request_not_visible",
            vec!["kind", "scope", "command"],
        ),
        (
            Page::Detail {
                scope: scope(true, false),
                record: record(BASELINE_NAME),
            },
            "detail",
            vec!["kind", "scope", "record"],
        ),
        (
            Page::Uncertain {
                company: COMPANY.into(),
                command: COMMAND.into(),
            },
            "uncertain",
            vec!["kind", "company", "command"],
        ),
    ];
    for (page, kind, fields) in cases {
        let result = html(page);
        let (_, value) = bootstrap(&result);
        assert_eq!(value["page"]["kind"], kind);
        keys(&value["page"], &fields);
        if kind != "registration" && kind != "request" {
            no_proof(&value);
        }
    }
}

#[test]
fn react_people_only_registration_and_pending_expose_current_form_proof() {
    let (_, value) = bootstrap(&html(request(Outcome::Pending {
        proof: PROOF.into(),
    })));
    let outcome = &value["page"]["request"]["outcome"];
    keys(outcome, &["kind", "proof"]);
    assert_eq!(outcome["kind"], "pending");
    assert_eq!(outcome["proof"], PROOF);
    let cases = [
        (
            Outcome::Committed {
                employee_id: EMPLOYEE.into(),
                person_id: PERSON.into(),
                receipt: RECEIPT.into(),
                registered_at: "2026-10-07 10:01:00 KST".into(),
            },
            "committed",
            vec![
                "kind",
                "employee_id",
                "person_id",
                "receipt",
                "registered_at",
            ],
        ),
        (
            Outcome::Rejected {
                reason: "이미 등록된 사번입니다.",
            },
            "rejected",
            vec!["kind", "reason"],
        ),
        (
            Outcome::Conflicting {
                reason: "회사 설정이 변경되었습니다.",
            },
            "conflicting",
            vec!["kind", "reason"],
        ),
        (Outcome::Cancelled, "cancelled", vec!["kind"]),
        (Outcome::Expired, "expired", vec!["kind"]),
    ];
    for (outcome, kind, fields) in cases {
        let result = html(request(outcome));
        let (_, value) = bootstrap(&result);
        assert_eq!(value["page"]["request"]["outcome"]["kind"], kind);
        keys(&value["page"]["request"]["outcome"], &fields);
        no_proof(&value);
        assert!(!result.contains(PROOF) && !result.contains("name=\"csrf_proof\""));
    }
}

#[test]
fn react_people_denied_and_withheld_navigation_never_gain_company_actions() {
    for page in [Page::Refused, Page::Unavailable] {
        let result = html(page);
        for forbidden in [BOOTSTRAP_ID, GUARD_PATH, RUNTIME_PATH, "<script", "<form"] {
            assert!(
                !result.contains(forbidden),
                "REACT_PEOPLE_DENIED_DOCUMENT_GAINED_CLIENT: {forbidden}"
            );
        }
        for secret in [
            COMPANY,
            COMMAND,
            PERSON,
            EMPLOYEE,
            RECEIPT,
            BASELINE_NAME,
            PROOF,
            "현재 검토 회사",
        ] {
            assert!(
                !result.contains(secret),
                "REACT_PEOPLE_DENIED_PROJECTION_DISCLOSURE: {secret}"
            );
        }
    }
    let result = html(Page::Directory {
        scope: scope(false, false),
        records: vec![],
        search_number: None,
        next_href: None,
        after_cursor: false,
    });
    let (_, value) = bootstrap(&result);
    for flag in [
        "directory_link",
        "can_create",
        "payroll_link",
        "policy_link",
    ] {
        assert_eq!(value["page"]["scope"][flag], false);
    }
    for suffix in ["/people/new", "/policy", "/payroll"] {
        assert!(!result.contains(&format!("href=\"/companies/{COMPANY}{suffix}\"")));
    }
    no_proof(&value);
}

#[test]
fn react_people_document_retains_private_headers_and_strict_same_origin_csp() {
    for (page, status) in [
        (directory(), StatusCode::OK),
        (Page::Refused, StatusCode::NOT_FOUND),
        (Page::Unavailable, StatusCode::SERVICE_UNAVAILABLE),
    ] {
        let response = document(page, status);
        assert_eq!(response.status(), status);
        let h = response.headers();
        assert_eq!(h[header::CACHE_CONTROL], "no-store");
        assert_eq!(h[header::PRAGMA], "no-cache");
        assert_eq!(h[header::VARY], "Authorization, Cookie, Origin");
        assert_eq!(h[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
        assert_eq!(h[header::REFERRER_POLICY], "same-origin");
        assert!(!h.contains_key(header::SET_COOKIE));
        let csp = h[header::CONTENT_SECURITY_POLICY].to_str().unwrap();
        let script_directive = if status == StatusCode::OK {
            "script-src 'self'"
        } else {
            "script-src 'none'"
        };
        for directive in [
            "default-src 'self'",
            script_directive,
            "style-src 'self'",
            "object-src 'none'",
            "frame-ancestors 'none'",
            "base-uri 'none'",
            "form-action 'self'",
        ] {
            assert!(
                csp.split(';').any(|part| part.trim() == directive),
                "REACT_PEOPLE_REQUIRED_CSP_MISSING: {directive}"
            );
        }
        for forbidden in ["'unsafe-inline'", "'unsafe-eval'", "https:", "http:", "*"] {
            assert!(
                !csp.contains(forbidden),
                "REACT_PEOPLE_UNSAFE_CSP: {forbidden}"
            );
        }
    }
}

#[test]
fn react_people_existing_pure_ssr_fallback_keeps_real_forms_and_oversized_drafts() {
    let oversized = "한".repeat(201);
    let result = render(registration(&oversized));
    assert!(
        result.contains(&format!("value=\"{oversized}\"")),
        "REACT_PEOPLE_INVALID_DRAFT_TRUNCATED"
    );
    assert!(result.contains(&format!("action=\"/companies/{COMPANY}/people/requests\"")));
    assert!(result.contains("method=\"post\"") && result.contains("name=\"csrf_proof\""));
    assert!(
        !result.contains("<script"),
        "REACT_PEOPLE_PURE_FALLBACK_NOT_PRESERVED"
    );
}
