//! Shared workspace acceptance at the existing mounted SSR renderer.
use super::{ScreenSection, ShippingScreens, UiScreen, render_screens};

#[test]
fn workspace_focused_pages_have_a_named_main_and_keyboard_navigation() {
    let screens = ShippingScreens {
        companies: ScreenSection::Empty,
        people: ScreenSection::Empty,
        runs: ScreenSection::Empty,
        ..ShippingScreens::default()
    };
    for (focus, subject) in [
        (UiScreen::Home, "업무 공간"),
        (UiScreen::Organization, "조직"),
        (UiScreen::Hr, "사람과 고용"),
        (UiScreen::Payroll, "급여"),
    ] {
        let html = render_screens(&screens, focus);
        assert_eq!(html.matches("<h1").count(), 1, "one page subject: {html}");
        let heading = html
            .split("<h1")
            .nth(1)
            .unwrap()
            .split("</h1>")
            .next()
            .unwrap();
        assert!(
            heading.contains(subject),
            "focused page subject must be meaningful: {heading}"
        );
        assert!(
            html.contains("href=\"#main-content\""),
            "keyboard users need a working skip link"
        );
        assert!(
            html.contains("<main id=\"main-content\""),
            "skip link must target main"
        );
        let nav = html
            .split("<nav")
            .nth(1)
            .expect("navigation landmark")
            .split('>')
            .next()
            .unwrap();
        assert!(
            nav.contains("aria-label=\"주요 탐색\""),
            "navigation landmark must be named"
        );
        assert!(
            !html.contains("/pkg/"),
            "static workspace must not download hydration"
        );
    }
}

#[test]
fn workspace_navigation_never_advertises_denied_or_unimplemented_destinations() {
    let html = render_screens(
        &ShippingScreens {
            people: ScreenSection::Empty,
            ..ShippingScreens::default()
        },
        UiScreen::Hr,
    );
    let nav = html
        .split("<nav")
        .nth(1)
        .expect("navigation landmark")
        .split("</nav>")
        .next()
        .unwrap();
    assert_eq!(nav.matches("aria-current=\"page\"").count(), 1);
    let at = nav.find("href=\"/hr\"").expect("authorized HR destination");
    let start = nav[..at].rfind("<a ").expect("destination is a link");
    let link = nav[start..].split('>').next().unwrap();
    assert!(
        link.contains("aria-current=\"page\""),
        "current state belongs to focused destination"
    );
    for destination in [
        "/organization",
        "/payroll",
        "/approvals",
        "/messenger",
        "/calendar",
        "/data",
    ] {
        assert!(
            !html.contains(&format!("href=\"{destination}\"")),
            "unoffered destination: {destination}"
        );
    }
    assert!(
        !html.contains("href=\"#\"") && !html.contains("disabled"),
        "no pretend workspace action"
    );
    let denied = render_screens(&ShippingScreens::default(), UiScreen::Hr);
    assert!(
        !denied.contains("<nav") && !denied.contains("사람과 고용"),
        "denied focus stays omitted"
    );
}

// Renderer fixtures only. These values are never product data or browser provisioning.
use super::native_account::{self, Page};
use axum::http::{StatusCode, header};

const COMPANY: &str = "00000000-0000-4000-8000-000000000101";
fn page(flags: u8) -> Page {
    Page::Company {
        org_id: COMPANY.into(),
        name: "긴 회사 이름 <연구 & 본사> 주식회사".into(),
        slug: "seoul-<research&production>-".repeat(12),
        show_payroll_navigation: flags & 1 != 0,
        show_policy_navigation: flags & 2 != 0,
        show_payroll_policy_navigation: flags & 4 != 0,
        people_policy: None,
    }
}
fn links(html: &str) -> Vec<(&str, &str)> {
    html.split("<a ")
        .skip(1)
        .map(|part| {
            let (attributes, rest) = part.split_once('>').expect("anchor opening");
            let href = attributes
                .split("href=\"")
                .nth(1)
                .expect("anchor href")
                .split('"')
                .next()
                .unwrap();
            let label = rest.split_once("</a>").expect("anchor closing").0;
            assert!(
                !attributes.contains(" hidden") && !attributes.contains("aria-hidden=\"true\"")
            );
            (href, label)
        })
        .collect()
}
fn destination_href_present(html: &str, href: &str) -> bool {
    html.contains(&format!("href=\"{href}\""))
}
#[test]
fn destination_href_oracle_distinguishes_independently_authorized_descendant() {
    let parent = "/companies/id/policy";
    let descendant = "/companies/id/policy/payroll-read/install";
    let only_descendant = format!("<a href=\"{descendant}\">급여 목록 열람 권한</a>");
    assert!(destination_href_present(&only_descendant, descendant));
    assert!(!destination_href_present(&only_descendant, parent));
    let with_parent = format!("{only_descendant}<a href=\"{parent}\">권한 관리</a>");
    assert!(destination_href_present(&with_parent, parent));
    assert!(!destination_href_present("", parent));
}
fn has_expected_cards(html: &str, flags: u8) -> bool {
    let cards: Vec<_> = html.split("<article ").skip(1).collect();
    cards.len() == flags.count_ones() as usize
        && [(1, "payroll"), (2, "policy"), (4, "payroll-policy")]
            .iter()
            .all(|(bit, key)| {
                cards
                    .iter()
                    .filter(|card| {
                        card.split_once('>').is_some_and(|(attrs, _)| {
                            attrs.contains(&format!("data-company-destination=\"{key}\""))
                                && !attrs.contains("hidden")
                        })
                    })
                    .count()
                    == usize::from(flags & bit != 0)
            })
}
fn one_account_destination(html: &str) -> bool {
    let links = links(html);
    let account: Vec<_> = links
        .iter()
        .filter(|(href, _)| *href == "/account")
        .collect();
    account.len() == 1 && account[0].1 == "내 업무 공간 목록"
}
#[test]
fn company_workspace_oracles_reject_unoffered_cards_and_duplicate_account_links() {
    let account = "<a href=\"/account\">내 업무 공간 목록</a>";
    assert!(one_account_destination(account));
    assert!(!one_account_destination(&format!(
        "{account}<a href=\"/account\">다른 이름</a>"
    )));
    assert!(!one_account_destination(
        "<a href=\"/account\">다른 이름</a>"
    ));
    assert!(!one_account_destination(""));
    let payroll = "<article data-company-destination=\"payroll\"><a href=\"/companies/x/payroll\">급여</a></article>";
    assert!(has_expected_cards(payroll, 1));
    assert!(has_expected_cards("", 0));
    assert!(!has_expected_cards("", 1));
    assert!(!has_expected_cards(payroll, 0));
    assert!(!has_expected_cards(
        "<article data-company-destination=\"payroll\"><h2>급여</h2></article>",
        0
    ));
    assert!(!has_expected_cards(&format!("{payroll}{payroll}"), 1));
    assert!(!has_expected_cards(
        &payroll.replace("payroll\"", "wrong\""),
        1
    ));
    assert!(!has_expected_cards(
        &payroll.replace("<article ", "<article hidden "),
        1
    ));
}
#[test]
fn company_workspace_keeps_each_destination_independently_authorized() {
    for flags in 0..8 {
        let html = native_account::render(page(flags));
        assert!(html.contains("class=\"workspace company-workspace\""));
        assert!(html.contains("href=\"/assets/workspace.css\""));
        assert!(html.contains("<html lang=\"ko\""));
        assert!(html.contains("id=\"main-content\" tabindex=\"-1\""));
        assert!(html.contains(&format!("data-company-id=\"{COMPANY}\"")));
        assert!(html.contains("긴 회사 이름 &lt;연구 &amp; 본사&gt; 주식회사"));
        assert!(html.contains(&"seoul-&lt;research&amp;production&gt;-".repeat(12)));
        let headings: Vec<_> = html.split("<h1").skip(1).collect();
        assert_eq!(headings.len(), 1);
        assert_eq!(
            headings[0]
                .split_once('>')
                .unwrap()
                .1
                .split_once("</h1>")
                .unwrap()
                .0,
            "긴 회사 이름 &lt;연구 &amp; 본사&gt; 주식회사"
        );
        assert_eq!(html.matches("<main ").count(), 1);
        assert_eq!(html.matches("<header ").count(), 1);
        assert!(html.contains("<header class=\"app\""));
        assert!(html.contains("aria-label=\"계정 탐색\""));
        assert_eq!(html.matches("aria-current=\"page\"").count(), 1);
        assert_eq!(
            html.split("aria-current=\"page\"")
                .nth(1)
                .unwrap()
                .split_once('>')
                .unwrap()
                .1
                .split('<')
                .next()
                .unwrap(),
            "회사 업무 공간"
        );
        assert!(
            !html.contains("<script")
                && !html.contains("<leptos-island")
                && !html.contains("native-account.js")
        );
        let actual = links(&html);
        assert!(has_expected_cards(&html, flags));
        assert!(one_account_destination(&html));
        assert_eq!(
            actual
                .iter()
                .filter(|(href, _)| *href == "/account")
                .count(),
            1
        );
        assert_eq!(
            actual
                .iter()
                .filter(|(href, _)| *href == "#main-content")
                .count(),
            1
        );
        let allowed = [
            (1, format!("/companies/{COMPANY}/payroll"), "급여"),
            (2, format!("/companies/{COMPANY}/policy"), "권한 관리"),
            (
                4,
                format!("/companies/{COMPANY}/policy/payroll-read/install"),
                "급여 목록 열람 권한",
            ),
        ];
        for (bit, href, label) in &allowed {
            let matches: Vec<_> = actual.iter().filter(|(target, _)| target == href).collect();
            assert_eq!(matches.len(), usize::from(flags & bit != 0));
            if let Some((_, actual_label)) = matches.first() {
                assert_eq!(*actual_label, *label);
            }
            if flags & bit == 0 {
                assert!(!destination_href_present(&html, href));
            }
        }
        assert_eq!(
            actual
                .iter()
                .filter(|(href, label)| *href == "/account" && *label == "내 업무 공간 목록")
                .count(),
            1
        );
        for (href, _) in &actual {
            assert!(
                ["/", "/account", "#main-content"].contains(href)
                    || allowed
                        .iter()
                        .any(|(bit, path, _)| flags & bit != 0 && href == path)
            );
        }
        assert_eq!(
            html.contains("data-company-destinations=\"empty\""),
            flags == 0
        );
        assert_eq!(
            html.contains("이 화면에서 열 수 있는 업무가 없습니다"),
            flags == 0
        );
        if flags == 0 {
            assert!(html.contains(
                "내 업무 공간 목록에서 다른 회사를 선택하거나 담당자에게 업무 권한을 확인해 주세요."
            ));
        }
        assert!(
            !html.contains("disabled")
                && !html.contains("/hr\"")
                && !html.contains("/organization\"")
        );
    }
}
#[test]
fn static_company_csp_preserves_private_headers_and_interactive_account_script() {
    let response = native_account::document(page(7), StatusCode::OK);
    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers();
    assert_eq!(headers[header::CACHE_CONTROL], "no-store");
    assert_eq!(headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    assert_eq!(
        headers[header::CONTENT_SECURITY_POLICY],
        "default-src 'self'; script-src 'none'; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'"
    );
    let sign_in = native_account::render(Page::SignIn);
    assert!(sign_in.contains("src=\"/assets/native-account.js\""));
    assert!(sign_in.contains("data-native-action=\"login\""));
    for id in [
        "native-status",
        "native-error",
        "native-cancel",
        "native-recheck",
        "native-retry-finish",
        "native-continue",
        "native-reload-terms",
    ] {
        assert_eq!(sign_in.matches(&format!("id=\"{id}\"")).count(), 1);
    }
    for label in [
        "취소",
        "결과 다시 확인",
        "같은 요청 다시 제출",
        "로그인으로 확인",
        "새 약관 확인",
    ] {
        assert!(sign_in.contains(label));
    }
    assert_eq!(
        native_account::document(Page::SignIn, StatusCode::OK).headers()
            [header::CONTENT_SECURITY_POLICY],
        "default-src 'self'; script-src 'self'; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'"
    );
}
