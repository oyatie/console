//! Payroll `Layer::Ui` surface. Leptos is pinned here; `/_ui` is not mounted.
use leptos::prelude::*;

#[component]
pub fn Shell() -> impl IntoView {
    view! { <div></div> }
}

pub fn shell() -> impl IntoView {
    Shell()
}
