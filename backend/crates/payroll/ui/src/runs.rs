use leptos::prelude::*;

use crate::islands::{
    CalculateRunForm, CloseAttendanceForm, CreateRunForm, IssuePayslipsForm, ResolveExceptionForm,
    SubmitRunForm,
};
use crate::ports::{PayRunDetail, PayRunSummary};
use crate::status::{exception_chip, run_chip};

#[component]
pub fn RunsPage(runs: Vec<PayRunSummary>) -> impl IntoView {
    let empty = runs.is_empty();
    view! {
        <section class="page">
            <h1>"급여 실행"</h1>
            {empty.then(|| view! { <p class="empty">"등록된 급여 실행이 없습니다."</p> })}
            <ul>
                {runs
                    .into_iter()
                    .map(|run| {
                        let href = format!("/payroll/runs/{}", run.id);
                        view! {
                            <li>
                                <a href=href>
                                    {run.period_start.clone()}
                                    "–"
                                    {run.period_end.clone()}
                                </a>
                                " "
                                <span class="chip">{run_chip(&run.status)}</span>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
            <CreateRunForm />
        </section>
    }
}

#[component]
pub fn RunDetailPage(detail: PayRunDetail, actor_id: String, can_manage: bool) -> impl IntoView {
    let status = detail.run.status.clone();
    let run_id = detail.run.id.clone();
    let show_close = can_manage && matches!(status.as_str(), "STAGED" | "READY_FOR_REVIEW");
    let show_calc = can_manage && status == "ATTENDANCE_CLOSED";
    let show_submit = can_manage && status == "CALCULATED" && detail.exceptions.iter().all(|row| row.status != "OPEN");
    let show_issue = can_manage && status == "APPROVED";
    let is_submitter = detail
        .run
        .submitted_by
        .as_ref()
        .is_some_and(|id| id == &actor_id);
    view! {
        <section class="page">
            <h1>"급여 실행 상세"</h1>
            <p>
                <span class="chip">{run_chip(&status)}</span>
                " "
                {detail.run.period_start.clone()}
                "–"
                {detail.run.period_end.clone()}
            </p>
            {detail.total_net_won.map(|won| {
                let lineage = detail
                    .total_net_lineage
                    .as_ref()
                    .map(|line| line.source_ko.clone())
                    .unwrap_or_else(|| "서버 산출".to_owned());
                view! {
                    <p>
                        "차인지급액 합계 "
                        {format!("{won}원")}
                        <span class="lineage">{lineage}</span>
                    </p>
                }
            })}
            <p class="note">"결재는 수신함에서 합니다. 상신자와 결재자는 같을 수 없습니다."</p>
            {is_submitter.then(|| view! { <p class="note">"이 실행을 상신한 당사자는 결재할 수 없습니다."</p> })}
            {show_close.then(|| view! { <CloseAttendanceForm run_id=run_id.clone() /> })}
            {show_calc.then(|| view! { <CalculateRunForm run_id=run_id.clone() /> })}
            <h2>"예외"</h2>
            <ul>
                {detail
                    .exceptions
                    .into_iter()
                    .map(|ex| {
                        let open = ex.status == "OPEN" && can_manage;
                        view! {
                            <li>
                                <span class="chip">{exception_chip(&ex.status)}</span>
                                " "
                                {ex.summary_ko}
                                {open.then(|| {
                                    view! {
                                        <ResolveExceptionForm
                                            run_id=run_id.clone()
                                            exception_id=ex.id.clone()
                                        />
                                    }
                                })}
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
            {show_submit.then(|| view! { <SubmitRunForm run_id=run_id.clone() /> })}
            {show_issue.then(|| view! { <IssuePayslipsForm run_id=run_id.clone() /> })}
        </section>
    }
}
