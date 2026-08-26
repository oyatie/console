use leptos::prelude::*;

/// Identical body for unknown paths and unauthorized paths (404 omit).
#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <section class="page">
            <h1>"페이지 없음"</h1>
            <p class="empty">"요청한 경로가 없습니다."</p>
        </section>
    }
}
