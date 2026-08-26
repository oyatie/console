//! Island editors. Each island receives only the fields it edits.

use leptos::prelude::*;

use crate::ports::{
    POST_COMPANY_REVISE, POST_EMPLOYMENT_APPOINT, POST_JOB_POSITION_CREATE, POST_ORG_UNIT_CREATE,
    POST_PERSON_CREATE,
};

#[island]
pub fn ReviseCompanyForm(
    org_id: String,
    legal_name: String,
    reg_no: String,
    version: i32,
) -> impl IntoView {
    view! {
        <form method="post" action=POST_COMPANY_REVISE class="editor">
            <input type="hidden" name="org_id" value=org_id />
            <input type="hidden" name="version" value=version.to_string() />
            <label>
                "상호"
                <input type="text" name="legal_name" value=legal_name required />
            </label>
            <label>
                "사업자등록번호"
                <input type="text" name="reg_no" value=reg_no />
            </label>
            <button type="submit">"개정"</button>
        </form>
    }
}

#[island]
pub fn CreateOrgUnitForm() -> impl IntoView {
    view! {
        <form method="post" action=POST_ORG_UNIT_CREATE class="editor">
            <label>
                "조직 이름"
                <input type="text" name="name" required />
            </label>
            <label>
                "상위 조직 ID"
                <input type="text" name="parent_id" />
            </label>
            <button type="submit">"등록"</button>
        </form>
    }
}

#[island]
pub fn CreateJobPositionForm() -> impl IntoView {
    view! {
        <form method="post" action=POST_JOB_POSITION_CREATE class="editor">
            <label>
                "직위"
                <input type="text" name="title" required />
            </label>
            <label>
                "조직 ID"
                <input type="text" name="org_unit_id" required />
            </label>
            <button type="submit">"등록"</button>
        </form>
    }
}

#[island]
pub fn CreatePersonForm() -> impl IntoView {
    view! {
        <form method="post" action=POST_PERSON_CREATE class="editor">
            <label>
                "표시 이름"
                <input type="text" name="display_name" required />
            </label>
            <label>
                "성명"
                <input type="text" name="legal_name" required />
            </label>
            <button type="submit">"등록"</button>
        </form>
    }
}

#[island]
pub fn AppointEmploymentForm() -> impl IntoView {
    view! {
        <form method="post" action=POST_EMPLOYMENT_APPOINT class="editor">
            <label>
                "구성원 ID"
                <input type="text" name="person_id" required />
            </label>
            <label>
                "조직 ID"
                <input type="text" name="org_unit_id" required />
            </label>
            <label>
                "직위 ID"
                <input type="text" name="job_position_id" required />
            </label>
            <label>
                "발령일"
                <input type="date" name="appointed_on" required />
            </label>
            <button type="submit">"발령"</button>
        </form>
    }
}

pub fn link_islands() {
    let _ = (
        ReviseCompanyForm,
        CreateOrgUnitForm,
        CreateJobPositionForm,
        CreatePersonForm,
        AppointEmploymentForm,
    );
}

pub const ISLAND_NAMES: &[&str] = &[
    "ReviseCompanyForm",
    "CreateOrgUnitForm",
    "CreateJobPositionForm",
    "CreatePersonForm",
    "AppointEmploymentForm",
];
