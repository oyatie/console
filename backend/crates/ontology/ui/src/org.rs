use leptos::prelude::*;

use crate::islands::{CreateJobPositionForm, CreateOrgUnitForm};
use crate::ports::{JobPositionHead, OrgUnitHead};

#[component]
pub fn OrgPage(units: Vec<OrgUnitHead>, jobs: Vec<JobPositionHead>) -> impl IntoView {
    let units_empty = units.is_empty();
    let jobs_empty = jobs.is_empty();
    view! {
        <section class="page">
            <h1>"조직"</h1>
            <h2>"조직 단위"</h2>
            {units_empty.then(|| view! { <p class="empty">"등록된 조직이 없습니다."</p> })}
            <ul>
                {units
                    .into_iter()
                    .map(|unit| {
                        view! {
                            <li>
                                <span>{unit.name}</span>
                                " "
                                <span class="chip">{format!("v{}", unit.version)}</span>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
            <CreateOrgUnitForm />
            <h2>"직위"</h2>
            {jobs_empty.then(|| view! { <p class="empty">"등록된 직위가 없습니다."</p> })}
            <ul>
                {jobs
                    .into_iter()
                    .map(|job| {
                        view! {
                            <li>
                                <span>{job.title}</span>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
            <CreateJobPositionForm />
        </section>
    }
}
