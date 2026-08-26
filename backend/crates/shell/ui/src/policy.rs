//! Read-only grant fold. Not a client Cedar engine.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyGrant {
    pub action: String,
    pub principal: String,
    pub effect: String,
    pub status: String,
}

#[component]
pub fn PolicyFoldPage(grants: Vec<PolicyGrant>) -> impl IntoView {
    let empty = grants.is_empty();
    view! {
        <section class="page">
            <h1>"권한 폴드"</h1>
            <p class="note">"서버가 투영한 허용만 표시합니다. 클라이언트는 정책을 평가하지 않습니다."</p>
            {empty.then(|| view! { <p class="empty">"표시할 권한이 없습니다."</p> })}
            <ul>
                {grants
                    .into_iter()
                    .map(|grant| {
                        view! {
                            <li>
                                <span class="chip">{grant.effect}</span>
                                " "
                                <span class="chip">{grant.status}</span>
                                " "
                                {grant.action}
                                " · "
                                {grant.principal}
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </section>
    }
}
