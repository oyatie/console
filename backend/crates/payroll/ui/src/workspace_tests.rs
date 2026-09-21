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
