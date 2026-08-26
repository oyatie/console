use leptos::prelude::*;

use console_ontology_ui::OrgSnapshot;
use console_payroll_ui::PayrollSnapshot;

use crate::caps::SurfaceCaps;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blocker {
    pub chip: &'static str,
}

#[must_use]
pub fn blockers(
    caps: &SurfaceCaps,
    org: &OrgSnapshot,
    payroll: &PayrollSnapshot,
) -> Vec<Blocker> {
    let mut out = Vec::new();
    if caps.company && org.company.is_none() {
        out.push(Blocker { chip: "회사 없음" });
    }
    if caps.people && org.people.is_empty() {
        out.push(Blocker { chip: "구성원 없음" });
    }
    if caps.payroll && !payroll.rates_present {
        out.push(Blocker { chip: "요율 없음" });
    }
    if caps.payroll && payroll.runs.iter().any(|run| run.exceptions_open > 0) {
        out.push(Blocker { chip: "예외 미결" });
    }
    if caps.approvals && !payroll.inbox.is_empty() {
        out.push(Blocker { chip: "결재 대기" });
    }
    out
}

#[component]
pub fn WorkHubPage(
    caps: SurfaceCaps,
    org: OrgSnapshot,
    payroll: PayrollSnapshot,
) -> impl IntoView {
    let items = blockers(&caps, &org, &payroll);
    let ready = items.is_empty();
    view! {
        <section class="page">
            <h1>"작업 허브"</h1>
            {if ready {
                view! { <p><span class="chip">"준비됨"</span></p> }.into_any()
            } else {
                view! {
                    <ul class="blockers">
                        {items
                            .into_iter()
                            .map(|item| view! { <li><span class="chip">{item.chip}</span></li> })
                            .collect_view()}
                    </ul>
                }
                .into_any()
            }}
        </section>
    }
}
