use leptos::prelude::*;

pub const POST_LOGIN: &str = "/ui/login";

#[island]
pub fn LoginForm() -> impl IntoView {
    view! {
        <form method="post" action=POST_LOGIN class="editor">
            <label>
                "아이디"
                <input type="text" name="username" required autocomplete="username" />
            </label>
            <label>
                "비밀번호"
                <input type="password" name="password" required autocomplete="current-password" />
            </label>
            <button type="submit">"로그인"</button>
        </form>
    }
}

pub fn link_islands() {
    let _ = LoginForm;
}

pub const ISLAND_NAMES: &[&str] = &["LoginForm"];
