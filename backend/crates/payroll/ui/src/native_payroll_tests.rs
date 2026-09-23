//! Renderer contract fixtures only: not product records or populated journey evidence.
use super::native_payroll::{Collection, CompanyIdentity, Page, Run, document, render};
use axum::http::{StatusCode, header};

const COMPANY: &str = "00000000-0000-0000-0000-000000000101";
fn collection() -> Collection {
    Collection {
        company: COMPANY.into(),
        identity: Some(CompanyIdentity {
            name: "서울 제조 주식회사".into(),
            slug: "seoul-manufacturing".into(),
        }),
        items: vec![],
        total: 0,
        limit: 100,
        offset: 0,
    }
}
fn run() -> Run {
    Run {
        id: "00000000-0000-0000-0000-000000000201".into(),
        period_start: "2026-09-01".into(),
        period_end: "2026-09-30".into(),
        source_label: "9월 정기 급여 — 생산1팀".into(),
        status: "SUBMITTED".into(),
        calculation_enabled: true,
        created_by: Some("00000000-0000-0000-0000-000000000202".into()),
        approved_by: Some("00000000-0000-0000-0000-000000000203".into()),
        approved_at: Some("2026-10-01T09:00:00+09:00".into()),
        close_receipt: Some(
            serde_json::json!({"checks":[{"key":"attendance-closed", "label_ko":"근태 마감 확인",
            "ok":false,"warn":true,"note":"근태 정정 확인 필요", "blocking_refs":["attendance-case-123"]}],
            "attested_by":"00000000-0000-0000-0000-000000000204", "attested_at":"2026-09-30T18:00:00+09:00",
            "additional_evidence":{"future-field":"원본 추가 증빙","absent_reason":null}}),
        ),
        submitted_by: Some("00000000-0000-0000-0000-000000000205".into()),
        submitted_at: Some("2026-10-01T08:00:00+09:00".into()),
        decided_by: Some("00000000-0000-0000-0000-000000000206".into()),
        decided_at: Some("2026-10-01T10:00:00+09:00".into()),
        decision_reason: Some("장문 결정 사유 — 정정 자료를 검토했습니다. ".repeat(80)),
        approval_ref: Some("00000000-0000-0000-0000-000000000207".into()),
        created_at: "2026-09-29T09:00:00+09:00".into(),
        updated_at: "2026-10-01T11:00:00+09:00".into(),
    }
}
fn hrefs(html: &str) -> Vec<&str> {
    html.split("<a ")
        .skip(1)
        .filter_map(|tail| {
            tail.split('>')
                .next()?
                .split("href=\"")
                .nth(1)?
                .split('"')
                .next()
        })
        .collect()
}

fn field<'a>(html: &'a str, name: &str, label: &str) -> &'a str {
    let marker = format!("data-field=\"{name}\"");
    let rows: Vec<_> = html
        .split("</dd>")
        .filter(|row| row.contains(&marker))
        .collect();
    assert_eq!(rows.len(), 1, "one visible labeled value per field: {name}");
    let row = rows[0].rsplit("<dt").next().unwrap();
    assert!(
        row.contains(label) && row.contains("</dt>") && row.contains("<dd"),
        "unlabeled field: {name}"
    );
    let value = row
        .split(&marker)
        .nth(1)
        .unwrap()
        .split_once('>')
        .unwrap()
        .1;
    for hidden in [
        " hidden",
        "aria-hidden",
        "type=\"hidden\"",
        "display:none",
        "visibility:hidden",
    ] {
        assert!(!row.contains(hidden), "hidden field: {name}");
    }
    value
}
fn only_backed_links(html: &str, identity: bool) {
    let base = format!("/companies/{COMPANY}/payroll");
    for href in hrefs(html) {
        if ["/", "/account", "#main-content"].contains(&href) || href == base {
            continue;
        }
        if identity && href == format!("/companies/{COMPANY}") {
            continue;
        }
        let query = href
            .strip_prefix(&format!("{base}?limit="))
            .expect("unbacked route");
        let (limit, offset) = query
            .split_once("&amp;offset=")
            .expect("unexpected pagination query");
        assert!((1..=500).contains(&limit.parse::<i64>().unwrap()));
        assert!(offset.parse::<i64>().unwrap() >= 0);
    }
    for forbidden in [
        "<script",
        "<leptos-island",
        "disabled",
        "onclick=",
        "href=\"#\"",
        "지급 완료",
    ] {
        assert!(
            !html.contains(forbidden),
            "dead/unbacked action: {forbidden}"
        );
    }
    // The only permitted form in this collection is real read pagination.
    for form in html.split("<form").skip(1) {
        let tag = form.split('>').next().unwrap();
        assert!(tag.contains("method=\"get\"") && tag.contains(&format!("action=\"{base}\"")));
    }
}
#[test]
fn native_payroll_empty_is_a_real_scoped_ssr_collection_without_dead_actions() {
    let html = render(Page::Runs(collection()));
    for marker in [
        "<html lang=\"ko\"",
        "data-screen=\"payroll\"",
        "data-state=\"empty\"",
        "<h1",
        "급여",
        "서울 제조 주식회사",
        "seoul-manufacturing",
        "등록된 급여 회차가 없습니다",
        "href=\"#main-content\"",
        "id=\"main-content\"",
        "aria-current=\"page\"",
    ] {
        assert!(html.contains(marker), "missing {marker}");
    }
    assert!(!html.contains("data-run-id"));
    only_backed_links(&html, true);
    assert!(!html.contains("rel=\"next\"") && !html.contains("rel=\"prev\""));
}
#[test]
fn native_payroll_all_eighteen_fields_and_full_receipt_have_visible_evidence() {
    let mut page = collection();
    let r = run();
    let reason = r.decision_reason.clone().unwrap();
    page.items.push(r);
    page.total = 1;
    let html = render(Page::Runs(page));
    for (name, label, value) in [
        ("id", "회차 식별자", "00000000-0000-0000-0000-000000000201"),
        ("period_start", "산정 시작", "2026-09-01"),
        ("period_end", "산정 종료", "2026-09-30"),
        ("source_label", "자료 이름", "9월 정기 급여 — 생산1팀"),
        ("status", "상태", "SUBMITTED"),
        ("calculation_enabled", "산정 가능 여부", "산정 가능"),
        (
            "created_by",
            "작성 계정",
            "00000000-0000-0000-0000-000000000202",
        ),
        (
            "approved_by",
            "승인 계정",
            "00000000-0000-0000-0000-000000000203",
        ),
        ("approved_at", "승인 시각", "2026-10-01T09:00:00+09:00"),
        (
            "submitted_by",
            "제출 계정",
            "00000000-0000-0000-0000-000000000205",
        ),
        ("submitted_at", "제출 시각", "2026-10-01T08:00:00+09:00"),
        (
            "decided_by",
            "결정 계정",
            "00000000-0000-0000-0000-000000000206",
        ),
        ("decided_at", "결정 시각", "2026-10-01T10:00:00+09:00"),
        ("decision_reason", "결정 사유", reason.as_str()),
        (
            "approval_ref",
            "승인 기록",
            "00000000-0000-0000-0000-000000000207",
        ),
        ("created_at", "작성 시각", "2026-09-29T09:00:00+09:00"),
        ("updated_at", "최근 변경", "2026-10-01T11:00:00+09:00"),
    ] {
        assert!(
            field(&html, name, label).contains(value),
            "missing visible field: {name}"
        );
    }
    let receipt = field(&html, "close_receipt", "근태 마감 증빙");
    for value in [
        "attendance-closed",
        "근태 마감 확인",
        "통과하지 않음",
        "주의",
        "근태 정정 확인 필요",
        "attendance-case-123",
        "00000000-0000-0000-0000-000000000204",
        "2026-09-30T18:00:00+09:00",
        "additional_evidence",
        "원본 추가 증빙",
        "absent_reason",
        "null",
    ] {
        assert!(
            receipt.contains(value),
            "missing receipt semantics: {value}"
        );
    }
    assert!(html.contains("<details") && html.contains("<summary"));
    assert!(!html.contains("data-state=\"empty\""));
    only_backed_links(&html, true);
}
#[test]
fn native_payroll_escapes_every_text_surface_and_preserves_unknown_receipts() {
    let mut page = collection();
    page.identity = Some(CompanyIdentity {
        name: "<img name>&\"".into(),
        slug: "<img slug>&\"".into(),
    });
    let mut r = run();
    r.source_label = "<img source>&\"".into();
    r.status = "<img status>&\"".into();
    r.decision_reason = Some("<img reason>&\"".into());
    r.close_receipt = Some(
        serde_json::json!({"checks":"unexpected historical shape","opaque":["<img receipt>&\"",null,false,91]}),
    );
    page.items.push(r);
    page.total = 1;
    let html = render(Page::Runs(page));
    assert!(!html.contains("<img"));
    assert!(!html.contains("<script"));

    for surface in ["name", "slug", "source", "status", "reason"] {
        let prefix = format!("&lt;img {surface}&gt;&amp;");
        assert!(
            html.contains(&format!("{prefix}\"")) || html.contains(&format!("{prefix}&quot;")),
            "quote lost from escaped field"
        );
    }
    assert!(html.contains("&lt;img name&gt;&amp;") && html.contains("&lt;img slug&gt;&amp;"));
    for (name, label, value) in [
        ("source_label", "자료 이름", "&lt;img source&gt;&amp;"),
        ("status", "상태", "&lt;img status&gt;&amp;"),
        ("decision_reason", "결정 사유", "&lt;img reason&gt;&amp;"),
    ] {
        assert!(field(&html, name, label).contains(value));
    }
    let receipt = field(&html, "close_receipt", "근태 마감 증빙");
    for value in [
        "&lt;img receipt&gt;&amp;",
        "unexpected historical shape",
        "opaque",
        "null",
        "false",
        "91",
    ] {
        assert!(receipt.contains(value));
    }
    let mut page = collection();
    page.identity = None;
    let mut r = run();
    r.close_receipt = None;
    r.created_by = None;
    r.approved_by = None;
    r.approved_at = None;
    r.submitted_by = None;
    r.submitted_at = None;
    r.decided_by = None;
    r.decided_at = None;
    r.decision_reason = None;
    r.approval_ref = None;
    r.calculation_enabled = false;
    page.items.push(r);
    page.total = 1;
    let html = render(Page::Runs(page));
    assert!(!html.contains("서울 제조 주식회사") && !html.contains("seoul-manufacturing"));
    only_backed_links(&html, false);
    for (name, label) in [
        ("created_by", "작성 계정"),
        ("approved_by", "승인 계정"),
        ("approved_at", "승인 시각"),
        ("submitted_by", "제출 계정"),
        ("submitted_at", "제출 시각"),
        ("decided_by", "결정 계정"),
        ("decided_at", "결정 시각"),
        ("decision_reason", "결정 사유"),
        ("approval_ref", "승인 기록"),
        ("close_receipt", "근태 마감 증빙"),
    ] {
        assert!(field(&html, name, label).contains("기록 없음"));
    }
    assert!(field(&html, "calculation_enabled", "산정 가능 여부").contains("산정 불가"));
    let mut page = collection();
    let mut r = run();
    r.close_receipt = Some(serde_json::Value::Null);
    page.items.push(r);
    page.total = 1;
    let html = render(Page::Runs(page));
    let receipt = field(&html, "close_receipt", "근태 마감 증빙");
    assert!(receipt.contains("null") && !receipt.contains("기록 없음"));
}
#[test]
fn native_payroll_pagination_reenters_same_owner_and_handles_empty_windows_and_overflow() {
    let mut page = collection();
    page.total = 301;
    page.limit = 100;
    page.offset = 100;
    page.items.push(run());
    let html = render(Page::Runs(page));
    let links = hrefs(&html);
    only_backed_links(&html, true);
    assert!(
        links
            .iter()
            .any(|h| *h == format!("/companies/{COMPANY}/payroll?limit=100&amp;offset=0"))
    );
    assert!(
        links
            .iter()
            .any(|h| *h == format!("/companies/{COMPANY}/payroll?limit=100&amp;offset=200"))
    );

    let summary = html
        .split("data-pagination-summary")
        .nth(1)
        .unwrap()
        .split('>')
        .nth(1)
        .unwrap()
        .split("</p>")
        .next()
        .unwrap();
    assert!(
        summary.contains("301") && summary.contains("101–101"),
        "missing actual count/window text"
    );
    let forms: Vec<_> = html.split("<form").skip(1).collect();
    assert_eq!(forms.len(), 1);
    let form = forms[0].split("</form>").next().unwrap();
    for bypass in ["formaction", "formmethod", " form="] {
        assert!(!form.contains(bypass));
    }
    assert_eq!(form.matches("<select").count(), 1);
    assert_eq!(form.matches("<input").count(), 1);
    assert_eq!(form.matches("<button").count(), 1);
    assert!(!form.contains("<textarea"));
    assert_eq!(form.matches("<label").count(), 1);
    assert!(form.contains("for=\"payroll-page-size\""));
    let select = form
        .split("<select")
        .nth(1)
        .unwrap()
        .split("</select>")
        .next()
        .unwrap();
    let tag = select.split('>').next().unwrap();
    assert!(tag.contains("id=\"payroll-page-size\"") && tag.contains("name=\"limit\""));
    let selected: Vec<_> = select
        .split("<option")
        .skip(1)
        .map(|x| x.split('>').next().unwrap())
        .filter(|x| x.contains("selected"))
        .collect();
    assert_eq!(selected.len(), 1);
    assert!(selected[0].contains("value=\"100\""));
    let offset = form
        .split("<input")
        .nth(1)
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    assert!(
        offset.contains("name=\"offset\"")
            && offset.contains("type=\"hidden\"")
            && offset.contains("value=\"0\"")
    );
    let button = form
        .split("<button")
        .nth(1)
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    assert!(button.contains("type=\"submit\"") && !button.contains("name="));
    for (total, offset, previous_offset, next_offset) in [
        (301, 0, None, Some(100)),
        (301, 300, Some(200), None),
        (300, 200, Some(100), None),
        (25, 100, Some(0), None),
    ] {
        let mut page = collection();
        page.total = total;
        page.offset = offset;
        let html = render(Page::Runs(page));
        only_backed_links(&html, true);
        assert_eq!(html.contains("rel=\"prev\""), previous_offset.is_some());
        assert_eq!(html.contains("rel=\"next\""), next_offset.is_some());
        let expected: Vec<_> = [previous_offset, next_offset]
            .into_iter()
            .flatten()
            .map(|value| format!("/companies/{COMPANY}/payroll?limit=100&amp;offset={value}"))
            .collect();
        let queries: Vec<_> = hrefs(&html)
            .into_iter()
            .filter(|h| h.contains('?'))
            .collect();
        for destination in &expected {
            assert!(queries.contains(&destination.as_str()));
        }
        assert!(
            queries
                .iter()
                .all(|href| expected.iter().any(|value| value == href)),
            "unexpected boundary page destination"
        );
        if offset >= total {
            assert!(!html.contains("등록된 급여 회차가 없습니다"));
            assert!(html.contains("첫 페이지"));
            assert!(
                hrefs(&html)
                    .iter()
                    .any(|h| *h == format!("/companies/{COMPANY}/payroll?limit=100&amp;offset=0"))
            );
        }
    }
    let mut page = collection();
    page.total = i64::MAX;
    page.limit = 500;
    page.offset = i64::MAX - 10;
    let html = render(Page::Runs(page));
    only_backed_links(&html, true);
    assert!(!html.contains("offset=-") && !html.contains("rel=\"next\""));
    assert!(html.contains("rel=\"prev\""));

    let expected_previous = format!(
        "/companies/{COMPANY}/payroll?limit=500&amp;offset={}",
        i64::MAX - 510
    );
    let first = format!("/companies/{COMPANY}/payroll?limit=500&amp;offset=0");
    let queries: Vec<_> = hrefs(&html)
        .into_iter()
        .filter(|h| h.contains('?'))
        .collect();
    assert!(queries.contains(&expected_previous.as_str()));
    assert!(
        queries
            .iter()
            .all(|h| *h == expected_previous || *h == first)
    );
}
#[test]
fn native_payroll_errors_have_no_scoped_context_and_all_documents_are_private() {
    for page in [
        Page::NotVisible,
        Page::AuthenticationRequired,
        Page::Unavailable,
        Page::InvalidRequest,
    ] {
        let html = render(page);
        for secret in [
            COMPANY,
            "서울 제조 주식회사",
            "seoul-manufacturing",
            "data-run-id",
            "data-state=\"empty\"",
            "data-screen=\"payroll\"",
            "<form",
        ] {
            assert!(!html.contains(secret));
        }
        assert!(html.contains("/account") && !html.contains("<script"));
    }
    for (page, status) in [
        (Page::Runs(collection()), StatusCode::OK),
        (Page::NotVisible, StatusCode::NOT_FOUND),
        (Page::AuthenticationRequired, StatusCode::UNAUTHORIZED),
        (Page::Unavailable, StatusCode::SERVICE_UNAVAILABLE),
        (Page::InvalidRequest, StatusCode::BAD_REQUEST),
    ] {
        let response = document(page, status);
        assert_eq!(response.status(), status);
        for (key, value) in [
            (header::CACHE_CONTROL, "no-store"),
            (header::PRAGMA, "no-cache"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::REFERRER_POLICY, "no-referrer"),
            (header::VARY, "Authorization, Cookie, Origin"),
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
        ] {
            assert_eq!(response.headers().get_all(&key).iter().count(), 1);
            assert_eq!(response.headers()[key], value);
        }
        assert!(!response.headers().contains_key(header::SET_COOKIE));
        assert_eq!(
            response
                .headers()
                .get_all(header::CONTENT_SECURITY_POLICY)
                .iter()
                .count(),
            1
        );
        assert_eq!(
            response.headers()[header::CONTENT_SECURITY_POLICY],
            "default-src 'self'; script-src 'none'; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'"
        );
    }
}
