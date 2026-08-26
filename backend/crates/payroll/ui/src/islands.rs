//! Island editors. Islands never receive a capability matrix or payroll math.

use leptos::prelude::*;

use crate::ports::{
    POST_ATTENDANCE_HANDOFF, POST_CALCULATE, POST_CLOSE_ATTENDANCE, POST_DECIDE, POST_ISSUE,
    POST_RESOLVE_EXCEPTION, POST_RUN_CREATE, POST_SUBMIT,
};

#[island]
pub fn CreateRunForm() -> impl IntoView {
    view! {
        <form method="post" action=POST_RUN_CREATE class="editor">
            <label>
                "시작일"
                <input type="date" name="period_start" required />
            </label>
            <label>
                "종료일"
                <input type="date" name="period_end" required />
            </label>
            <button type="submit">"등록"</button>
        </form>
    }
}

#[island]
pub fn AttendanceHandoffForm(period_start: String, period_end: String) -> impl IntoView {
    view! {
        <form method="post" action=POST_ATTENDANCE_HANDOFF class="editor">
            <input type="hidden" name="period_start" value=period_start.clone() />
            <input type="hidden" name="period_end" value=period_end.clone() />
            <p>
                "대상 기간 "
                <span class="chip">{period_start}</span>
                "–"
                <span class="chip">{period_end}</span>
            </p>
            <button type="submit">"급여 실행으로 넘기기"</button>
        </form>
    }
}

#[island]
pub fn CloseAttendanceForm(run_id: String) -> impl IntoView {
    view! {
        <form method="post" action=POST_CLOSE_ATTENDANCE class="editor">
            <input type="hidden" name="run_id" value=run_id />
            <button type="submit">"근태 마감"</button>
        </form>
    }
}

#[island]
pub fn CalculateRunForm(run_id: String) -> impl IntoView {
    view! {
        <form method="post" action=POST_CALCULATE class="editor">
            <input type="hidden" name="run_id" value=run_id />
            <button type="submit">"산출"</button>
        </form>
    }
}

#[island]
pub fn ResolveExceptionForm(run_id: String, exception_id: String) -> impl IntoView {
    view! {
        <form method="post" action=POST_RESOLVE_EXCEPTION class="editor">
            <input type="hidden" name="run_id" value=run_id />
            <input type="hidden" name="exception_id" value=exception_id />
            <label>
                "사유"
                <input type="text" name="reason" required />
            </label>
            <button type="submit">"해소"</button>
        </form>
    }
}

#[island]
pub fn SubmitRunForm(run_id: String) -> impl IntoView {
    view! {
        <form method="post" action=POST_SUBMIT class="editor">
            <input type="hidden" name="run_id" value=run_id />
            <button type="submit">"상신"</button>
        </form>
    }
}

#[island]
pub fn IssuePayslipsForm(run_id: String) -> impl IntoView {
    view! {
        <form method="post" action=POST_ISSUE class="editor">
            <input type="hidden" name="run_id" value=run_id />
            <button type="submit">"명세서 발행"</button>
        </form>
    }
}

#[island]
pub fn DecideRunForm(run_id: String) -> impl IntoView {
    let (decision, set_decision) = signal("APPROVE".to_string());
    view! {
        <form method="post" action=POST_DECIDE class="editor">
            <input type="hidden" name="run_id" value=run_id />
            <label>
                "결정"
                <select
                    name="decision"
                    on:change=move |ev| set_decision.set(event_target_value(&ev))
                >
                    <option value="APPROVE" selected>"승인"</option>
                    <option value="REJECT">"반려"</option>
                </select>
            </label>
            <label>
                "사유"
                <input type="text" name="reason" prop:required=move || decision.get() == "REJECT" />
            </label>
            <button type="submit">"결재"</button>
        </form>
    }
}

pub fn link_islands() {
    let _ = (
        CreateRunForm,
        AttendanceHandoffForm,
        CloseAttendanceForm,
        CalculateRunForm,
        ResolveExceptionForm,
        SubmitRunForm,
        IssuePayslipsForm,
        DecideRunForm,
    );
}

pub const ISLAND_NAMES: &[&str] = &[
    "CreateRunForm",
    "AttendanceHandoffForm",
    "CloseAttendanceForm",
    "CalculateRunForm",
    "ResolveExceptionForm",
    "SubmitRunForm",
    "IssuePayslipsForm",
    "DecideRunForm",
];
