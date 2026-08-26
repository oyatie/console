use leptos::prelude::*;

use crate::caps::{nav_items, SurfaceCaps};
use crate::session::Session;

#[component]
pub fn NavBar(session: Session, current: String) -> impl IntoView {
    let items = nav_items(&session.caps);
    view! {
        <header>
            <strong>"콘솔"</strong>
            <nav>
                {items
                    .into_iter()
                    .map(|item| {
                        let current_page = current == item.href;
                        view! {
                            <a
                                href=item.href
                                aria-current=if current_page { "page" } else { "" }
                            >
                                {item.label_ko}
                            </a>
                        }
                    })
                    .collect_view()}
            </nav>
            <span class="chip">{session.display_name}</span>
        </header>
    }
}

#[must_use]
pub fn nav_contains(caps: &SurfaceCaps, href: &str) -> bool {
    nav_items(caps).iter().any(|item| item.href == href)
}
