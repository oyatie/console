use leptos::prelude::*;

use crate::islands::{AppointEmploymentForm, CreatePersonForm};
use crate::ports::{EmploymentHead, PersonHead};

#[component]
pub fn PeoplePage(people: Vec<PersonHead>, employments: Vec<EmploymentHead>) -> impl IntoView {
    let people_empty = people.is_empty();
    view! {
        <section class="page">
            <h1>"구성원"</h1>
            {people_empty.then(|| view! { <p class="empty">"등록된 구성원이 없습니다."</p> })}
            <ul>
                {people
                    .into_iter()
                    .map(|person| {
                        view! {
                            <li>
                                <span>{person.display_name}</span>
                                " · "
                                <span>{person.legal_name}</span>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
            <CreatePersonForm />
            <h2>"발령"</h2>
            <ul>
                {employments
                    .into_iter()
                    .map(|row| {
                        view! {
                            <li>
                                <span>{row.person_id}</span>
                                " → "
                                <span>{row.org_unit_id}</span>
                                " · "
                                <span>{row.appointed_on}</span>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
            <AppointEmploymentForm />
        </section>
    }
}
