//! Authorized organization projections. Company membership is not inferred.
use super::{CompanyView, OrgUnitView, ScreenSection, revision_label};
use leptos::prelude::*;
use std::collections::HashMap;

pub(crate) fn body(
    companies: ScreenSection<CompanyView>,
    units: ScreenSection<OrgUnitView>,
) -> AnyView {
    if matches!(
        (&companies, &units),
        (ScreenSection::Omitted, ScreenSection::Omitted)
    ) {
        return ().into_any();
    }
    view! {
        <div class="workflow-layout" data-screen="organization">
            <div class="workflow-main">
                {company_section(companies)}
                {unit_section(units)}
            </div>
            <aside class="workflow-guide" aria-label="조직 변경 안내">
                <section class="panel">
                    <h2>"조직 변경 시 확인할 사항"</h2>
                    <p class="section-intro">"변경을 검토할 때 다음 자료를 함께 확인하세요."</p>
                    <ul class="guidance-list">
                        <li>"현재 상위 조직과 변경할 상위 조직"</li>
                        <li>"변경 적용일과 해당 시점의 조직 버전"</li>
                        <li>"영향을 받는 고용과 배치"</li>
                        <li>"필요한 검토, 승인과 변경 이력"</li>
                    </ul>
                </section>
            </aside>
        </div>
    }
    .into_any()
}

fn company_section(companies: ScreenSection<CompanyView>) -> AnyView {
    match companies {
        ScreenSection::Omitted => ().into_any(),
        ScreenSection::Failure => view! {
            <section class="panel" data-state="failure" aria-label="법인 목록 불러오기 오류">
                <h2>"법인"</h2>
                <p class="state">"법인 목록을 불러오지 못했습니다"</p>
                <a class="recovery-link" href="/organization">"다시 불러오기"</a>
            </section>
        }
        .into_any(),
        ScreenSection::Empty => empty_companies(),
        ScreenSection::Rows(rows) if rows.is_empty() => empty_companies(),
        ScreenSection::Rows(rows) => view! {
            <section class="panel" aria-label="법인 목록">
                <h2>"법인"</h2>
                <ul class="record-list">
                    {rows.into_iter().map(company_card).collect_view()}
                </ul>
            </section>
        }
        .into_any(),
    }
}

fn empty_companies() -> AnyView {
    view! {
        <section class="panel" data-state="empty" aria-label="법인 목록">
            <h2>"법인"</h2>
            <p class="state">"표시할 법인이 없습니다"</p>
        </section>
    }
    .into_any()
}

fn company_card(company: CompanyView) -> impl IntoView {
    let href = format!("/api/v1/companies/{}", company.org_id);
    let label = if company.legal_name.is_empty() {
        company.org_id.clone()
    } else {
        company.legal_name.clone()
    };
    let id = company.org_id.clone();
    let registration = if company.reg_no.is_empty() {
        "정보 없음".to_owned()
    } else {
        company.reg_no.clone()
    };
    let version = revision_label(&company.version);
    view! {
        <li class="record-card"
            data-org-id=company.org_id
            data-legal-name=company.legal_name
            data-reg-no=company.reg_no
            data-version=company.version
        >
            <h3 class="record-title"><a href=href>{label}</a></h3>
            <dl class="record-meta">
                <dt>"등록번호"</dt><dd>{registration}</dd>
                <dt>"법인 식별자"</dt><dd class="record-id">{id}</dd>
                <dt>"버전"</dt><dd class="record-revision">{version}</dd>
            </dl>
        </li>
    }
}

fn unit_section(units: ScreenSection<OrgUnitView>) -> AnyView {
    match units {
        ScreenSection::Omitted => ().into_any(),
        ScreenSection::Failure => view! {
            <section class="panel" data-state="failure" aria-label="조직 목록 불러오기 오류">
                <h2>"조직 구조"</h2>
                <p class="state">"조직 목록을 불러오지 못했습니다"</p>
                <a class="recovery-link" href="/organization">"다시 불러오기"</a>
            </section>
        }
        .into_any(),
        ScreenSection::Empty => empty_units(),
        ScreenSection::Rows(rows) if rows.is_empty() => empty_units(),
        ScreenSection::Rows(rows) => hierarchy(rows),
    }
}

fn empty_units() -> AnyView {
    view! {
        <section class="panel" data-state="empty">
            <h2>"조직 구조"</h2>
            <p class="state">"표시할 조직이 없습니다"</p>
        </section>
    }
    .into_any()
}

fn hierarchy(units: Vec<OrgUnitView>) -> AnyView {
    // None marks an ambiguous identity, never a winning duplicate. Only unique
    // roots and edges enter the forest, so ambiguous descendants stay unresolved.
    let mut identities: HashMap<&str, Option<usize>> = HashMap::new();
    for (index, unit) in units.iter().enumerate() {
        identities
            .entry(unit.id.as_str())
            .and_modify(|entry| *entry = None)
            .or_insert(Some(index));
    }
    let mut children = vec![Vec::new(); units.len()];
    let mut roots = Vec::new();
    for (index, unit) in units.iter().enumerate() {
        if identities[unit.id.as_str()].is_none() {
            continue;
        }
        if unit.parent_id.is_empty() {
            roots.push(index);
        } else if let Some(Some(parent)) = identities.get(unit.parent_id.as_str()) {
            children[*parent].push(index);
        }
    }
    // Iterative preorder preserves input sibling order and visits parents first.
    // The DOM stays flat, including for thousands of levels of source hierarchy.
    let mut stack: Vec<_> = roots.into_iter().rev().collect();
    let mut visited = vec![false; units.len()];
    let mut ordered = Vec::new();
    while let Some(index) = stack.pop() {
        if visited[index] {
            continue;
        }
        visited[index] = true;
        ordered.push(index);
        stack.extend(children[index].iter().rev().copied());
    }
    let parent_label = |unit: &OrgUnitView| {
        if unit.parent_id.is_empty() {
            "상위 조직 없음".to_owned()
        } else {
            match identities.get(unit.parent_id.as_str()) {
                Some(Some(index)) => {
                    let parent = &units[*index];
                    let name = if parent.name.is_empty() {
                        "이름 정보 없음"
                    } else {
                        parent.name.as_str()
                    };
                    format!("상위 조직: {name} · {}", parent.id)
                }
                _ => format!("상위 조직 확인 필요 · 식별자: {}", unit.parent_id),
            }
        }
    };
    let clean = ordered
        .into_iter()
        .map(|index| unit_card(&units[index], parent_label(&units[index]), false))
        .collect_view();
    let has_unresolved = visited.iter().any(|seen| !seen);
    let unresolved = units
        .iter()
        .enumerate()
        .filter(|(index, _)| !visited[*index])
        .map(|(_, unit)| {
            unit_card(
                unit,
                parent_label(unit),
                identities[unit.id.as_str()].is_none(),
            )
        })
        .collect_view();
    view! {
        <section class="panel">
            <h2>"조직 구조"</h2>
            <p class="section-intro">"조직 이름과 상위 조직을 함께 확인하세요."</p>
            <ol class="record-list" aria-label="조직 구조">{clean}</ol>
            {has_unresolved.then(|| view! {
                <section aria-label="상위 조직 확인 필요">
                    <h3 class="hierarchy-note">"상위 조직 확인 필요"</h3>
                    <p class="section-intro">"상위 관계를 확인할 수 없는 기록입니다. 원본과 변경 이력을 확인하세요."</p>
                    <ol class="record-list" aria-label="상위 관계 확인이 필요한 조직">{unresolved}</ol>
                </section>
            })}
        </section>
    }
    .into_any()
}

fn unit_card(unit: &OrgUnitView, parent: String, duplicate: bool) -> AnyView {
    let label = if unit.name.is_empty() {
        unit.id.clone()
    } else {
        unit.name.clone()
    };
    let title = if duplicate {
        view! { <span>{label}</span> }.into_any()
    } else {
        let href = format!("/api/v1/org-units/{}", unit.id);
        view! { <a href=href>{label}</a> }.into_any()
    };
    let version = revision_label(&unit.version);
    view! {
        <li class="record-card"
            data-org-unit-id=unit.id.clone()
            data-name=unit.name.clone()
            data-parent-id=unit.parent_id.clone()
            data-version=unit.version.clone()
        >
            <h3 class="record-title">{title}</h3>
            {duplicate.then(|| view! { <p class="conflict-note">"중복 식별자 확인 필요"</p> })}
            <p class="hierarchy-note">{parent}</p>
            <dl class="record-meta">
                <dt>"조직 식별자"</dt><dd class="record-id">{unit.id.clone()}</dd>
                <dt>"버전"</dt><dd class="record-revision">{version}</dd>
            </dl>
        </li>
    }
    .into_any()
}
