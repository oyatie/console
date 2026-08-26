use leptos::prelude::*;

use crate::islands::LoginForm;

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <section class="page">
            <h1>"로그인"</h1>
            <p class="note">"세션은 쿠키로만 유지됩니다."</p>
            <LoginForm />
        </section>
    }
}
