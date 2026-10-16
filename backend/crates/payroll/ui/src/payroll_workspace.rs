//! Payroll task context. Record values remain exclusively in the working island.
use super::{AuthorizedRuns, RunSummary, ScreenSection};
use leptos::prelude::*;

pub(crate) fn body(runs: ScreenSection<RunSummary>) -> AnyView {
    if matches!(&runs, ScreenSection::Omitted) {
        return ().into_any();
    }
    view! {
        <div data-screen="payroll">
            <section class="panel" aria-label="급여 처리 안내">
                <h2>"급여 처리 안내"</h2>
                <p class="section-intro">"각 실행 기록에서 원천 자료와 검토 결과를 확인하세요."</p>
                <ol class="process-guide" aria-label="급여 처리 순서">
                    <li>"원천 확인"</li>
                    <li>"계산"</li>
                    <li>"예외 검토"</li>
                    <li>"독립 검토"</li>
                    <li>"명세서·지급 지시"</li>
                    <li>"지급 대사"</li>
                </ol>
            </section>
            <div class="workflow-layout">
                <div class="workflow-main">{records(runs)}</div>
                <aside class="workflow-guide" aria-label="급여 검토 안내">
                    <section class="panel">
                        <h2>"검토 전 확인"</h2>
                        <ul class="guidance-list">
                            <li>"검토할 기간, 원천과 버전이 같은지 확인하세요."</li>
                            <li>"원천의 불일치와 예외를 확인하고 처리 근거를 남기세요."</li>
                            <li>"작성자와 독립된 검토자의 검토 기록을 확인하세요."</li>
                        </ul>
                        <p class="review-notice">"승인은 지급 완료를 의미하지 않습니다"</p>
                        <p class="section-intro">"지급 지시를 내보내거나 내려받은 뒤에도 외부 지급 결과와 정산 증빙을 대사해야 합니다."</p>
                    </section>
                    <section class="panel">
                        <h2>"검토 자료"</h2>
                        <p class="section-intro">"실행 기록을 열어 다음 근거를 확인하세요."</p>
                        <ul class="guidance-list">
                            <li>"대상 기간과 원천 기록"</li>
                            <li>"계산에 사용된 버전과 적용 근거"</li>
                            <li>"예외, 보정 사유와 관련 증빙"</li>
                            <li>"검토 이력과 검토한 결과의 버전"</li>
                        </ul>
                    </section>
                </aside>
            </div>
        </div>
    }
    .into_any()
}

fn records(runs: ScreenSection<RunSummary>) -> AnyView {
    match runs {
        ScreenSection::Omitted => ().into_any(),
        ScreenSection::Failure => view! {
            <section class="panel" data-state="failure" aria-label="급여 목록 불러오기 오류">
                <h2>"실행 기록"</h2>
                <p class="state">"급여 목록을 불러오지 못했습니다"</p>
                <a class="recovery-link" href="/payroll">"다시 불러오기"</a>
            </section>
        }
        .into_any(),
        ScreenSection::Empty => empty_records(),
        ScreenSection::Rows(rows) if rows.is_empty() => empty_records(),
        ScreenSection::Rows(runs) => view! {
            <section class="panel" aria-label="급여 실행 기록">
                <h2>"실행 기록"</h2>
                <AuthorizedRuns runs=runs />
            </section>
        }
        .into_any(),
    }
}

fn empty_records() -> AnyView {
    view! {
        <section class="panel" data-state="empty" aria-label="급여 실행 기록">
            <h2>"실행 기록"</h2>
            <p class="state">"표시할 급여 이력이 없습니다"</p>
        </section>
    }
    .into_any()
}
