//! In-memory presentation fixtures only; no business provisioning or authority.
use super::native_group_process::*;
use axum::http::{StatusCode, header};

const GROUP: &str = "00000000-0000-0000-0000-000000000003";
const COMMAND: &str = "00000000-0000-0000-0000-000000000002";
const REQUESTED: &str = "00000000-0000-0000-0000-000000000005";
const ACTUAL: &str = "00000000-0000-0000-0000-000000000077";

fn expected() -> Expectations {
    Expectations {
        group_revision: "7".into(),
        group_incarnation: "00000000-0000-0000-0000-000000000004".into(),
        policy_revision: "0".into(),
    }
}

fn reference() -> HeadReference {
    HeadReference {
        process: ACTUAL.into(),
        content_version: "2".into(),
        head_revision: "3".into(),
        content_digest: "11".repeat(32),
        head_digest: "22".repeat(32),
        state: "ACTIVE",
        expires_at: "2026-10-05 10:00:00 KST".into(),
    }
}

fn receipt() -> Page {
    Page::Request(Request {
        original: OriginalRecord {
            group: GROUP.into(),
            incarnation: expected().group_incarnation,
            command: COMMAND.into(),
            actor: "00000000-0000-0000-0000-000000000001".into(),
            process: REQUESTED.into(),
            operation_label: "확인 절차 등록",
            accepted_at: "2026-10-04 10:00:00 KST".into(),
            intake_receipt: "unit-intake-receipt".into(),
            input_digest: "33".repeat(32),
            expected: expected(),
            content: None,
            requested_expiry: Some("2026-10-05 10:00:00 KST".into()),
            suspension_reason: None,
        },
        outcome: Outcome::Terminal {
            code: "REJECTED_STALE_EXPECTATION",
            title: "다른 변경이 먼저 반영되었습니다",
            description: "원래 요청의 확정 결과입니다.",
            receipt: "unit-result-receipt".into(),
            executed_at: "2026-10-04 10:00:01 KST".into(),
            before: Some(reference()),
            after: Some(reference()),
        },
    })
}

#[test]
fn native_group_validation_preserves_original_input_and_links_same_request() {
    let html = render(Page::Form(Form {
        scope: Scope {
            group: GROUP.into(),
            group_name: "현재 그룹".into(),
            incarnation: expected().group_incarnation,
            revision: "9".into(),
            policy_revision: "1".into(),
            operator: "unit-current-operator".into(),
        },
        command: COMMAND.into(),
        expected: expected(),
        proof: "unit-original-proof&\"<".into(),
        input: Input::Suspend {
            process: REQUESTED.into(),
            content_version: "1".into(),
            content_digest: "44".repeat(32),
            expected_head_revision: "1".into(),
            expected_head_digest: "55".repeat(32),
            reason: "<script>fail()</script>\r\n원래 입력".into(),
        },
        current: Some(Head {
            reference: reference(),
            causing_command: "unit-current-command".into(),
            causing_receipt: "unit-current-receipt".into(),
        }),
        errors: vec![FieldError {
            field: "reason",
            message: "사유를 확인하세요",
        }],
        form_error: None,
    }));
    for (name, value) in [
        ("command_id", COMMAND),
        ("expected_group_revision", "7"),
        ("expected_group_identity_policy_revision", "0"),
        ("process_version", "1"),
        ("expected_process_head_revision", "1"),
    ] {
        let tags: Vec<_> = html
            .split("<input")
            .skip(1)
            .map(|s| s.split('>').next().unwrap())
            .filter(|s| s.contains(&format!("name=\"{name}\"")))
            .collect();
        assert_eq!(tags.len(), 1, "{name}");
        assert!(tags[0].contains(&format!("value=\"{value}\"")), "{name}");
    }
    assert!(html.contains("unit-original-proof&amp;&quot;&lt;"));
    assert!(html.contains("&lt;script&gt;fail()&lt;/script&gt;\r\n원래 입력"));
    assert!(!html.contains("<script>fail()"));
    assert!(html.contains(&format!(
        "action=\"/groups/{GROUP}/identity/processes/{REQUESTED}/suspend\""
    )));
    assert!(html.contains(&format!(
        "href=\"/groups/{GROUP}/identity/requests/{COMMAND}\""
    )));
    assert!(html.contains("role=\"alert\""));
    assert!(html.contains("href=\"#reason\""));
    assert!(html.contains("이번 제출의 입력 내용을 확인해 주세요."));
    assert!(!html.contains("요청은 접수되지 않았습니다"));
}

#[test]
fn native_group_historical_result_renders_actual_reference_without_current_provenance() {
    let html = render(receipt());
    assert!(html.contains(&format!("data-group-process-id=\"{ACTUAL}\"")));
    assert!(html.contains("data-group-process-version=\"2\""));
    assert!(html.contains("data-group-process-head-revision=\"3\""));
    assert!(html.contains("unit-result-receipt"));
    assert!(html.contains(REQUESTED));
    assert!(html.contains(&format!(
        "href=\"/groups/{GROUP}/identity/requests/{COMMAND}/retry\""
    )));
    assert!(!html.contains("그룹 업무 탐색"));
    assert!(!html.contains("unit-current-"));
    assert!(!html.contains("csrf_proof"));
}

#[test]
fn native_group_documents_keep_private_static_headers() {
    let response = document(receipt(), StatusCode::OK);
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response.headers()[header::CACHE_CONTROL]
            .to_str()
            .unwrap()
            .contains("no-store")
    );
    assert!(
        response.headers()[header::CONTENT_SECURITY_POLICY]
            .to_str()
            .unwrap()
            .contains("script-src 'none'")
    );
    assert_eq!(response.headers()[header::REFERRER_POLICY], "same-origin");
}
