use leptos::prelude::*;

use crate::islands::AttendanceHandoffForm;
use crate::ports::AttendancePeriod;

#[component]
pub fn AttendanceHandoffPage(period: Option<AttendancePeriod>) -> impl IntoView {
    view! {
        <section class="page">
            <h1>"근태 인수"</h1>
            {match period {
                None => view! {
                    <p class="empty">"넘길 근태 기간이 없습니다."</p>
                    <AttendanceHandoffForm
                        period_start=String::new()
                        period_end=String::new()
                    />
                }.into_any(),
                Some(period) => view! {
                    <dl>
                        <dt>"기간"</dt>
                        <dd>{format!("{}–{}", period.period_start, period.period_end)}</dd>
                        <dt>"출근일"</dt>
                        <dd>
                            {period.worked_days.to_string()}
                            <span class="lineage">"근태 기록 · 서버"</span>
                        </dd>
                    </dl>
                    <AttendanceHandoffForm
                        period_start=period.period_start
                        period_end=period.period_end
                    />
                }.into_any(),
            }}
        </section>
    }
}
