//! Static, authorized Person and Employment records with native inspectors.
use std::collections::HashMap;

use leptos::prelude::*;

use super::{EmploymentView, PersonView, ScreenSection, day_of, revision_label};

fn name(person: &PersonView) -> String {
    if !person.display_name.is_empty() {
        person.display_name.clone()
    } else if !person.legal_name.is_empty() {
        person.legal_name.clone()
    } else {
        "이름 정보 없음".into()
    }
}

fn available(value: &str) -> String {
    if value.is_empty() {
        "정보 없음".into()
    } else {
        value.into()
    }
}

fn person_inspector(person: &PersonView, related: &[&EmploymentView]) -> AnyView {
    let label = name(person);
    let glyph = person
        .display_name
        .chars()
        .next()
        .or_else(|| person.legal_name.chars().next())
        .unwrap_or('·')
        .to_string();
    let secondary = (!person.legal_name.is_empty() && person.legal_name != label)
        .then(|| person.legal_name.clone());
    let relationships = related
        .iter()
        .map(|record| {
            let href = format!("#employment-{}", record.id);
            let date = available(&day_of(&record.appointed_on));
            let version = revision_label(&record.version);
            view! {
                <li><a href=href>{"고용 기록 · "}{date}{" · "}{version}</a></li>
            }
        })
        .collect_view();
    view! {
        <details class="person-inspector"
            data-person-id=person.id.clone() data-display-name=person.display_name.clone()
            data-legal-name=person.legal_name.clone() data-version=person.version.clone()>
            <summary class="person-summary">
                <span class="person-glyph" aria-hidden="true">{glyph}</span>
                <span class="person-heading"><strong>{label}</strong>
                    {secondary.map(|value| view! { <span class="person-secondary">{value}</span> })}
                </span>
                <span class="person-revision">{revision_label(&person.version)}</span>
                <span class="person-disclosure">"기록 보기"</span>
            </summary>
            <div class="person-detail" id=format!("person-{}", person.id)>
                <dl class="people-facts">
                    <dt>"표시 이름"</dt><dd>{available(&person.display_name)}</dd>
                    <dt>"법적 이름"</dt><dd>{available(&person.legal_name)}</dd>
                    <dt>"기록 버전"</dt><dd>{available(&person.version)}</dd>
                    <dt>"기록 식별자"</dt><dd class="people-identity">{person.id.clone()}</dd>
                </dl>
                {(!related.is_empty()).then(|| view! {
                    <div class="person-relationships"><h4>"연결된 고용 기록"</h4><ul>{relationships}</ul></div>
                })}
                <a class="people-source" href=format!("/api/v1/persons/{}", person.id)>"사람 원본 기록"</a>
            </div>
        </details>
    }.into_any()
}

fn employment_inspector(record: &EmploymentView, person: Option<&PersonView>) -> AnyView {
    let relationship = match person {
        Some(person) => {
            view! { <a href=format!("#person-{}", person.id)>{name(person)}</a> }.into_any()
        }
        None => view! { <span>"연결된 사람은 이 목록에서 확인할 수 없습니다"</span> }.into_any(),
    };
    view! {
        <details class="employment-inspector"
            data-employment-id=record.id.clone() data-version=record.version.clone()
            data-appointed-on=record.appointed_on.clone() data-person-id=record.person_id.clone()
            data-org-unit-id=record.org_unit_id.clone() data-job-position-id=record.job_position_id.clone()>
            <summary class="employment-summary">
                <span class="employment-heading"><strong>"고용 기록"</strong>
                    <span>{available(&day_of(&record.appointed_on))}</span>
                </span>
                <span class="person-revision">{revision_label(&record.version)}</span>
                <span class="person-disclosure">"기록 보기"</span>
            </summary>
            <div class="employment-detail" id=format!("employment-{}", record.id)>
                <dl class="people-facts">
                    <dt>"사람"</dt><dd>{relationship}</dd>
                    <dt>"임명 시점"</dt><dd>{available(&record.appointed_on)}</dd>
                    <dt>"조직 식별자"</dt><dd class="people-identity">{available(&record.org_unit_id)}</dd>
                    <dt>"직무 식별자"</dt><dd class="people-identity">{available(&record.job_position_id)}</dd>
                    <dt>"기록 버전"</dt><dd>{available(&record.version)}</dd>
                    <dt>"기록 식별자"</dt><dd class="people-identity">{record.id.clone()}</dd>
                </dl>
                <a class="people-source" href=format!("/api/v1/employments/{}", record.id)>"고용 원본 기록"</a>
            </div>
        </details>
    }.into_any()
}

fn people_section(
    people: &ScreenSection<PersonView>,
    related: &HashMap<&str, Vec<&EmploymentView>>,
) -> AnyView {
    let (state, content) = match people {
        ScreenSection::Omitted => return ().into_any(),
        ScreenSection::Failure => ("failure", view! {
            <p class="state">"사람 목록을 불러오지 못했습니다"</p>
            <a class="people-retry" href="/hr">"다시 불러오기"</a>
        }.into_any()),
        ScreenSection::Rows(rows) if !rows.is_empty() => ("rows", view! {
            <p class="people-count">{format!("{}개 기록 표시 중", rows.len())}</p>
            <div class="people-records">{rows.iter().map(|person| {
                person_inspector(person, related.get(person.id.as_str()).map(Vec::as_slice).unwrap_or(&[]))
            }).collect_view()}</div>
        }.into_any()),
        ScreenSection::Empty | ScreenSection::Rows(_) => ("empty", view! {
            <p class="state">"표시할 사람이 없습니다"</p>
        }.into_any()),
    };
    view! {
        <section class="panel people-section" data-section="people" data-state=state aria-label="사람 목록">
            <h3>"사람"</h3>{content}
        </section>
    }.into_any()
}

fn employment_section(
    employments: &ScreenSection<EmploymentView>,
    people: &HashMap<&str, &PersonView>,
) -> AnyView {
    let (state, content) = match employments {
        ScreenSection::Omitted => return ().into_any(),
        ScreenSection::Failure => (
            "failure",
            view! {
                <p class="state">"고용 기록을 불러오지 못했습니다"</p>
                <a class="people-retry" href="/hr">"다시 불러오기"</a>
            }
            .into_any(),
        ),
        ScreenSection::Rows(rows) if !rows.is_empty() => (
            "rows",
            view! {
                <p class="people-count">{format!("{}개 기록 표시 중", rows.len())}</p>
                <div class="people-records">{rows.iter().map(|record| {
                    employment_inspector(record, people.get(record.person_id.as_str()).copied())
                }).collect_view()}</div>
            }
            .into_any(),
        ),
        ScreenSection::Empty | ScreenSection::Rows(_) => (
            "empty",
            view! {
                <p class="state">"표시할 고용 기록이 없습니다"</p>
            }
            .into_any(),
        ),
    };
    view! {
        <section class="panel people-section" data-section="employments" data-state=state aria-label="고용 기록 목록">
            <h3>"고용 기록"</h3>{content}
        </section>
    }.into_any()
}

pub(crate) fn body(
    people: ScreenSection<PersonView>,
    employments: ScreenSection<EmploymentView>,
) -> AnyView {
    if !people.is_offered() && !employments.is_offered() {
        return ().into_any();
    }
    // Build once: exact identities, not names or an all-pairs record scan.
    let mut person_index = HashMap::new();
    if let ScreenSection::Rows(rows) = &people {
        for person in rows {
            person_index.insert(person.id.as_str(), person);
        }
    }
    let mut related: HashMap<&str, Vec<&EmploymentView>> = HashMap::new();
    if let ScreenSection::Rows(rows) = &employments {
        for record in rows {
            related
                .entry(record.person_id.as_str())
                .or_default()
                .push(record);
        }
    }
    let people = people_section(&people, &related);
    let employments = employment_section(&employments, &person_index);
    view! {
        <div class="people-workspace" data-screen="hr">
            <div class="people-introduction"><h2>"사람과 고용"</h2>
                <p>"현재 업무 범위에서 볼 수 있는 사람과 고용 기록입니다. 기록을 열어 연결 관계와 원본을 확인하세요."</p>
            </div>
            <div class="people-layout">{people}{employments}</div>
        </div>
    }.into_any()
}
