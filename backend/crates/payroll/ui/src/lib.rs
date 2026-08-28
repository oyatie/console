//! Payroll `Layer::Ui` surface. SSR HTML for `/_ui`; no payroll math.
use axum::Router;
use axum::response::Html;
use axum::routing::get;
use leptos::prelude::*;

#[component]
pub fn Shell() -> impl IntoView {
    view! {
        <html>
            <head>
                <meta charset="utf-8" />
            </head>
            <body></body>
        </html>
    }
}

pub fn shell() -> impl IntoView {
    Shell()
}

pub fn render_shell() -> String {
    let mut html = String::from("<!DOCTYPE html>");
    html.push_str(&shell().to_html());
    html
}

async fn get_shell() -> Html<String> {
    Html(render_shell())
}

pub fn router() -> Router {
    Router::new().route("/", get(get_shell))
}
