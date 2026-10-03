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
        show_people_navigation: false,
        show_people_create_navigation: false,
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
        assert!(html.contains("native-workspace-header"));
        assert!(html.contains("aria-label=\"계정 탐색\""));
        let (desktop, mobile) = company_header_presentations(&html);
        for region in [desktop, mobile] {
            let current: Vec<_> = region
                .split("<a ")
                .skip(1)
                .filter(|link| {
                    link.split('>')
                        .next()
                        .unwrap()
                        .contains("aria-current=\"page\"")
                })
                .collect();
            assert_eq!(current.len(), 1);
            assert!(current[0].contains(&format!("href=\"/companies/{COMPANY}\"")));
            let account: Vec<_> = links(region)
                .into_iter()
                .filter(|(href, _)| *href == "/account")
                .collect();
            assert_eq!(account, vec![("/account", "내 계정 · 업무 공간 선택")]);
        }
        assert!(
            !html.contains("<script")
                && !html.contains("<leptos-island")
                && !html.contains("native-account.js")
        );
        let main = company_main_region(&html);
        let actual = links(main);
        assert!(has_expected_cards(main, flags));
        assert_eq!(
            links(main)
                .iter()
                .filter(|(href, _)| *href == "/account")
                .count(),
            0
        );
        assert_eq!(
            actual
                .iter()
                .filter(|(href, _)| *href == "/account")
                .count(),
            0
        );
        assert_eq!(
            links(&html)
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
            0
        );
        for (href, _) in links(&html) {
            assert!(
                ["/", "/account", "#main-content"].contains(&href)
                    || href == format!("/companies/{COMPANY}")
                    || allowed
                        .iter()
                        .any(|(bit, path, _)| flags & bit != 0 && href == path.as_str())
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

// Renderer fixtures only. These inputs do not grant rights or qualify a real browser state.
fn company_main_region(html: &str) -> &str {
    assert_eq!(html.matches("<main ").count(), 1);
    html.split_once("<main ")
        .unwrap()
        .1
        .split_once("</main>")
        .unwrap()
        .0
}

fn company_header_presentations(html: &str) -> (&str, &str) {
    assert_eq!(html.matches("<header ").count(), 1);
    let header = html
        .split_once("<header ")
        .unwrap()
        .1
        .split_once("</header>")
        .unwrap()
        .0;
    let desktop_marker = "data-native-navigation=\"desktop\"";
    let mobile_marker = "data-native-navigation=\"mobile\"";
    assert_eq!(
        header.matches(desktop_marker).count(),
        1,
        "Company must use the shared desktop header"
    );
    assert_eq!(
        header.matches(mobile_marker).count(),
        1,
        "Company must use the shared mobile header"
    );
    let desktop = header.split_once(desktop_marker).unwrap().1;
    desktop.split_once(mobile_marker).unwrap()
}

fn company_shared_header_matrix_case(flags: u8) {
    use super::native_policy::{Operation, PolicyAction, Subject};
    let root = format!("/companies/{COMPANY}");
    let read = flags & 1 != 0;
    let create = flags & 2 != 0;
    let policy = flags & 4 != 0;
    let payroll = flags & 8 != 0;
    for people_policy_case in 0..3 {
        let people_policy = match people_policy_case {
            0 => None,
            1 => Some(vec![]),
            _ => Some(vec![
                PolicyAction {
                    subject: Subject::PeopleRead,
                    operation: Operation::Grant,
                },
                PolicyAction {
                    subject: Subject::PeopleCreate,
                    operation: Operation::Grant,
                },
            ]),
        };
        let html = native_account::render(Page::Company {
            org_id: COMPANY.into(),
            name: "회사 <연구 & 본사>".into(),
            slug: "unit-scope".into(),
            show_people_navigation: read,
            show_people_create_navigation: create,
            show_policy_navigation: policy,
            show_payroll_navigation: payroll,
            show_payroll_policy_navigation: false,
            people_policy,
        });
        let main = company_main_region(&html);
        let (desktop, mobile) = company_header_presentations(&html);
        let mut expected = vec!["/account".to_owned(), root.clone()];
        for (visible, suffix) in [
            (read, "/people"),
            (create, "/people/new"),
            (policy, "/policy"),
            (payroll, "/payroll"),
        ] {
            let href = format!("{root}{suffix}");
            assert_eq!(
                destination_href_present(&html, &href),
                visible,
                "raw href flags={flags} policy_case={people_policy_case}"
            );
            assert_eq!(
                links(main).iter().filter(|(path, _)| *path == href).count(),
                usize::from(visible),
                "main destination must be unique"
            );
            if visible {
                expected.push(href);
            }
        }
        expected.sort();
        for region in [desktop, mobile] {
            let mut actual: Vec<_> = links(region)
                .into_iter()
                .map(|(href, _)| href.to_owned())
                .collect();
            actual.sort();
            assert_eq!(
                actual, expected,
                "header projections flags={flags} policy_case={people_policy_case}"
            );
            assert_eq!(region.matches("aria-current=\"page\"").count(), 1);
            let current = region
                .split("<a ")
                .skip(1)
                .find(|link| {
                    link.split('>')
                        .next()
                        .unwrap()
                        .contains("aria-current=\"page\"")
                })
                .unwrap();
            assert!(
                current
                    .split('>')
                    .next()
                    .unwrap()
                    .contains(&format!("href=\"{root}\""))
            );
            assert!(!region.contains("<form") && !region.contains("<input"));
            let labels: Vec<_> = format!("{region}{main}")
                .split("<nav ")
                .skip(1)
                .map(|part| {
                    part.split_once("aria-label=\"")
                        .unwrap()
                        .1
                        .split('"')
                        .next()
                        .unwrap()
                        .to_owned()
                })
                .collect();
            assert_eq!(
                labels.len(),
                labels
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len(),
                "visible landmark names must differ"
            );
        }
        assert!(desktop.contains("aria-label=\"회사 업무 탐색\""));
        assert_eq!(
            desktop.contains("aria-label=\"사람과 조직 탐색\""),
            read || create || payroll
        );
        assert_eq!(
            mobile.matches("data-native-payroll-shortcut").count(),
            usize::from(payroll)
        );
        let menu = mobile.split_once("<details ").unwrap().1;
        assert!(!destination_href_present(menu, &format!("{root}/payroll")));
        assert!(main.contains("<summary>업무 공간 식별자</summary>"));
        assert!(main.contains("tabindex=\"-1\""));
        assert!(!html.contains("<script") && !html.contains("disabled"));
        assert!(
            !destination_href_present(&html, "#")
                && !html.contains("/organization")
                && !html.contains("/employment")
                && !html.contains("/approval")
        );
        let shortcut = if create {
            Some(format!("{root}/people/new"))
        } else if people_policy_case == 1 {
            Some(format!("{root}/policy/people-directory/install"))
        } else if people_policy_case == 2 {
            Some(format!("{root}/policy/people-directory/create/grant"))
        } else {
            None
        };
        assert_eq!(
            main.contains("id=\"company-next-task-title\">바로가기</h2>"),
            shortcut.is_some()
        );
        assert!(!main.contains("id=\"company-next-task-title\">다음 업무</h2>"));
        if let Some(target) = shortcut {
            assert_eq!(
                links(main)
                    .iter()
                    .filter(|(href, _)| *href == target)
                    .count(),
                1,
                "shortcut must have no competing main action"
            );
        }
    }
}

macro_rules! company_header_case {
    ($name:ident, $flags:literal) => {
        #[test]
        fn $name() {
            company_shared_header_matrix_case($flags);
        }
    };
}
company_header_case!(company_shared_header_flags_0000, 0);
company_header_case!(company_shared_header_flags_0001, 1);
company_header_case!(company_shared_header_flags_0010, 2);
company_header_case!(company_shared_header_flags_0011, 3);
company_header_case!(company_shared_header_flags_0100, 4);
company_header_case!(company_shared_header_flags_0101, 5);
company_header_case!(company_shared_header_flags_0110, 6);
company_header_case!(company_shared_header_flags_0111, 7);
company_header_case!(company_shared_header_flags_1000, 8);
company_header_case!(company_shared_header_flags_1001, 9);
company_header_case!(company_shared_header_flags_1010, 10);
company_header_case!(company_shared_header_flags_1011, 11);
company_header_case!(company_shared_header_flags_1100, 12);
company_header_case!(company_shared_header_flags_1101, 13);
company_header_case!(company_shared_header_flags_1110, 14);
company_header_case!(company_shared_header_flags_1111, 15);

#[test]
fn company_policy_root_clauses_are_truthful_static_rows() {
    let html = native_account::render(Page::CompanyPolicy {
        org_id: COMPANY.into(),
        action_keys: vec!["company.identity.read"],
        delegable_action_keys: vec!["company.identity.read"],
    });
    let main = company_main_region(&html);
    let (desktop, mobile) = company_header_presentations(&html);
    for region in [desktop, mobile] {
        assert_eq!(region.matches("aria-current=\"page\"").count(), 1);
        assert!(region.contains(&format!("href=\"/companies/{COMPANY}/policy\"")));
    }
    assert!(main.contains("연결된 사용 조항") && main.contains("위임 상한에 포함된 조항"));
    assert!(
        !main.contains("사용할 수 있는 기능") && !main.contains("다른 계정에 연결할 수 있는 범위")
    );
    assert!(main.contains("실행") && main.contains("현재 정책"));
    assert!(main.contains("회사 이름과 식별자 열람"));
    assert!(!html.contains("<script") && !main.contains("<form") && !main.contains("<button"));
    for href in [
        format!("/companies/{COMPANY}/people"),
        format!("/companies/{COMPANY}/people/new"),
        format!("/companies/{COMPANY}/payroll"),
    ] {
        assert!(
            !destination_href_present(&html, &href),
            "clauses alone must not create action destinations"
        );
    }
    let response = native_account::document(
        Page::CompanyPolicy {
            org_id: COMPANY.into(),
            action_keys: vec![],
            delegable_action_keys: vec![],
        },
        StatusCode::OK,
    );
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(
        response.headers()[header::CONTENT_SECURITY_POLICY],
        "default-src 'self'; script-src 'none'; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'"
    );
}

#[test]
fn company_policy_root_refusal_preserves_status_csp_and_omits_protected_context() {
    let html = native_account::render(Page::Refused);
    for forbidden in [
        COMPANY,
        "会社",
        "/companies/",
        "<form",
        "data-company-id",
        "unit-scope",
    ] {
        assert!(!html.contains(forbidden));
    }
    let response = native_account::document(Page::Refused, StatusCode::NOT_FOUND);
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(
        response.headers()[header::X_CONTENT_TYPE_OPTIONS],
        "nosniff"
    );
    assert_eq!(
        response.headers()[header::VARY],
        "Authorization, Cookie, Origin"
    );
    assert_eq!(
        response.headers()[header::CONTENT_SECURITY_POLICY],
        "default-src 'self'; script-src 'self'; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'"
    );
}
