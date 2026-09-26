//! Shared presentation for the native task pages. Callers supply authorized links.
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum NavigationMode {
    Desktop,
    MobileMenu,
}

pub(super) fn render(
    navigation: impl Fn(NavigationMode) -> AnyView,
    payroll_shortcut: Option<(String, bool)>,
) -> AnyView {
    let desktop = navigation(NavigationMode::Desktop);
    let mobile = navigation(NavigationMode::MobileMenu);
    view! {
        <header class="app native-workspace-header">
            <a class="brand" href="/"><span class="brand-mark" aria-hidden="true">"C"</span>"Console"</a>
            <div data-native-navigation="desktop">
                {desktop}
                <nav class="native-account-nav" aria-label="계정 탐색">
                    <a href="/account">"내 계정 · 업무 공간 선택"</a>
                </nav>
            </div>
            <div data-native-navigation="mobile">
                {payroll_shortcut.map(|(path, current)| view! {
                    <a class="native-payroll-shortcut" data-native-payroll-shortcut
                        href=path aria-current=current.then_some("page")>"급여"</a>
                })}
                <details class="native-mobile-menu">
                    <summary>"업무 탐색"</summary>
                    <div class="native-mobile-menu-body">
                        {mobile}
                        <nav class="native-account-nav" aria-label="계정 탐색">
                            <a href="/account">"내 계정 · 업무 공간 선택"</a>
                        </nav>
                    </div>
                </details>
            </div>
        </header>
    }
    .into_any()
}
