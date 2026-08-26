use leptos::prelude::*;

use crate::ports::MyPayslip;

#[component]
pub fn EssPage(payslip: Option<MyPayslip>) -> impl IntoView {
    view! {
        <section class="page">
            <h1>"내 급여"</h1>
            {match payslip {
                None => view! { <p class="empty">"열람할 명세서가 없습니다."</p> }.into_any(),
                Some(doc) => view! {
                    <p>
                        {doc.period_start.clone()}
                        "–"
                        {doc.period_end.clone()}
                        " · "
                        {doc.employee_name.clone()}
                    </p>
                    <h2>"지급"</h2>
                    <ul>
                        {doc.base_pay_won.map(|won| {
                            view! {
                                <li>
                                    "기본급 "
                                    {format!("{won}원")}
                                    <span class="lineage">"계약 임금 · 수정 불가"</span>
                                </li>
                            }
                        })}
                        {doc.earnings
                            .into_iter()
                            .map(|line| {
                                let amount = line
                                    .amount_won
                                    .map(|won| format!("{won}원"))
                                    .unwrap_or_else(|| "—".to_owned());
                                view! {
                                    <li>
                                        {line.label_ko}
                                        " "
                                        {amount}
                                        <span class="lineage">{line.lineage.source_ko}</span>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                    <h2>"공제"</h2>
                    <ul>
                        {doc.deductions
                            .into_iter()
                            .map(|line| {
                                let amount = line
                                    .amount_won
                                    .map(|won| format!("{won}원"))
                                    .unwrap_or_else(|| "—".to_owned());
                                view! {
                                    <li>
                                        {line.label_ko}
                                        " "
                                        {amount}
                                        <span class="lineage">{line.lineage.source_ko}</span>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                    {match doc.net_pay_won {
                        Some(won) => view! {
                            <p>
                                "차인지급액 "
                                {format!("{won}원")}
                                <span class="lineage">"서버 산출"</span>
                            </p>
                        }.into_any(),
                        None => view! {
                            <p>
                                "차인지급액 "
                                <span class="chip">"미산출"</span>
                                <span class="lineage">
                                    {doc.net_pay_unavailable_reason_ko.unwrap_or_else(|| {
                                        "원천징수가 산출되지 않았습니다.".to_owned()
                                    })}
                                </span>
                            </p>
                        }.into_any(),
                    }}
                }.into_any(),
            }}
        </section>
    }
}
