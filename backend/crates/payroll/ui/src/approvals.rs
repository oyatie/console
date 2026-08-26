use leptos::prelude::*;

use crate::islands::DecideRunForm;
use crate::ports::DecideInboxItem;

#[component]
pub fn ApprovalsPage(items: Vec<DecideInboxItem>, actor_id: String) -> impl IntoView {
    let visible: Vec<DecideInboxItem> = items
        .into_iter()
        .filter(|item| item.submitted_by != actor_id)
        .collect();
    let empty = visible.is_empty();
    view! {
        <section class="page">
            <h1>"결재 수신함"</h1>
            {empty.then(|| view! { <p class="empty">"대기 중인 결재가 없습니다."</p> })}
            <ul>
                {visible
                    .into_iter()
                    .map(|item| {
                        view! {
                            <li>
                                <span class="chip">"상신"</span>
                                " "
                                {item.period_start.clone()}
                                "–"
                                {item.period_end.clone()}
                                <DecideRunForm run_id=item.run_id />
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </section>
    }
}
