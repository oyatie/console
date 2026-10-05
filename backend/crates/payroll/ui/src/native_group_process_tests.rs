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
fn native_group_validation_preserves_original_input_and_links_correction() {
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
    let summary = invalid_summary(&html);
    assert!(!summary.contains(&format!(
        "href=\"/groups/{GROUP}/identity/requests/{COMMAND}\""
    )));
    assert!(!summary.contains("원래 요청 결과 확인"));
    assert_eq!(correction_href(summary), "#reason");
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

// Presentation-only fixtures for the separately approved invalid-feedback
// amendment. Real HTTP/owner/focus acceptance belongs to the native browser leaf.
fn invalid_summary(html: &str) -> &str {
    html.split("class=\"group-validation\"")
        .nth(1)
        .expect("one real invalid feedback summary")
        .split("</div>")
        .next()
        .unwrap()
}

fn correction_href(summary: &str) -> &str {
    let before_label = summary.split("입력 내용 수정으로 이동").next().unwrap();
    assert!(summary.contains("입력 내용 수정으로 이동"));
    let anchor = before_label.rsplit("<a ").next().unwrap();
    anchor
        .split("href=\"")
        .nth(1)
        .expect("native correction href")
        .split('"')
        .next()
        .unwrap()
}

fn adoption_validation(
    errors: Vec<FieldError>,
    form_error: Option<&'static str>,
    replacing: bool,
) -> Form {
    Form {
        scope: Scope {
            group: GROUP.into(),
            group_name: "현재 그룹".into(),
            incarnation: expected().group_incarnation,
            revision: "9".into(),
            policy_revision: "1".into(),
            operator: "unit-current-operator".into(),
        },
        // Deliberately the same identity as receipt(): invalid feedback cannot
        // assert this command has never been accepted.
        command: COMMAND.into(),
        expected: expected(),
        proof: "unit-original-proof&\"<".into(),
        input: Input::Adopt {
            process: REQUESTED.into(),
            expected_prior_head_revision: "3".into(),
            replacing,
            content: Content {
                title: "한".repeat(41),
                method: "ATTENDED_ACCOUNT_AND_DOCUMENTARY_REVIEW_V1".into(),
                intended_claimant_matching_procedure: "신청자를 직접 대조한다.".into(),
                account_possession_procedure: "원래 입력 <계정 & 소유>".into(),
                physical_human_evidence_procedure: "실제로 확인한 범위만 기록한다.".into(),
                duplicate_contradictory_claim_procedure: "중복 주장을 보류한다.".into(),
                qualification_criteria_instruction: "확인한 근거를 검토한다.".into(),
                escalation_adjudication_procedure: "독립 담당자에게 요청한다.".into(),
                evidence_minimization_retention_description: "최소 범위만 보관한다.".into(),
                recipient_responsibility: "불확실성과 확인한 사실을 구분한다.".into(),
            },
            expires_at_local: "2026-11-06T15:23:42".into(),
            responsibility_accepted: true,
        },
        current: Some(Head {
            reference: reference(),
            causing_command: "unit-current-command".into(),
            causing_receipt: "unit-current-receipt".into(),
        }),
        errors,
        form_error,
    }
}

#[test]
fn native_group_invalid_feedback_uses_first_error_in_server_order() {
    let html = render(Page::Form(adoption_validation(
        vec![
            FieldError {
                field: "account_possession_procedure",
                message: "계정 확인 내용을 확인하세요",
            },
            FieldError {
                field: "title",
                message: "제목을 확인하세요",
            },
        ],
        None,
        false,
    )));
    let summary = invalid_summary(&html);
    assert_eq!(correction_href(summary), "#account_possession_procedure");
    assert!(
        summary.find("계정 확인 내용을 확인하세요").unwrap()
            < summary.find("제목을 확인하세요").unwrap()
    );
    assert!(html.contains("id=\"account_possession_procedure\""));
    assert!(html.contains("id=\"title\""));
    assert!(html.contains("원래 입력 &lt;계정 &amp; 소유&gt;"));
    assert!(html.contains("unit-original-proof&amp;&quot;&lt;"));
    assert!(summary.contains("href=\"#title\""));
    assert!(!summary.contains("원래 요청 결과 확인"));
}

#[test]
fn native_group_aggregate_adopt_and_replace_errors_have_editable_title_destination() {
    for replacing in [false, true] {
        let html = render(Page::Form(adoption_validation(
            vec![],
            Some("제출한 내용 전체를 확인하세요"),
            replacing,
        )));
        let summary = invalid_summary(&html);
        assert_eq!(correction_href(summary), "#title");
        assert!(html.contains("id=\"title\""));
        assert!(summary.contains("제출한 내용 전체를 확인하세요"));
        assert!(summary.contains("이번 제출의 입력 내용을 확인해 주세요."));
        assert!(!summary.contains("요청은 접수되지 않았습니다"));
        assert!(!summary.contains("원래 요청 결과 확인"));
        assert!(html.contains(&"한".repeat(41)));
        assert!(html.contains("2026-11-06T15:23:42"));
    }
}

#[test]
fn native_group_aggregate_suspension_error_has_reason_destination() {
    let mut form = adoption_validation(vec![], Some("제출한 내용 전체를 확인하세요"), false);
    form.input = Input::Suspend {
        process: REQUESTED.into(),
        content_version: "1".into(),
        content_digest: "44".repeat(32),
        expected_head_revision: "1".into(),
        expected_head_digest: "55".repeat(32),
        reason: "원래 중단 사유".into(),
    };
    let html = render(Page::Form(form));
    assert_eq!(correction_href(invalid_summary(&html)), "#reason");
    assert!(html.contains("id=\"reason\""));
    assert!(html.contains("원래 중단 사유"));
    assert!(html.contains("unit-original-proof&amp;&quot;&lt;"));
}
