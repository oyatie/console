use leptos::prelude::*;

use crate::islands::ReviseCompanyForm;
use crate::ports::CompanyHead;

#[component]
pub fn CompanyPage(head: Option<CompanyHead>) -> impl IntoView {
    view! {
        <section class="page">
            <h1>"회사"</h1>
            {match head {
                None => view! {
                    <p class="empty">"등록된 회사가 없습니다."</p>
                }.into_any(),
                Some(company) => view! {
                    <dl>
                        <dt>"상호"</dt>
                        <dd>{company.legal_name.clone()}</dd>
                        <dt>"사업자등록번호"</dt>
                        <dd>{company.reg_no.clone()}</dd>
                        <dt>"개정"</dt>
                        <dd>
                            <span class="chip">{format!("v{}", company.version)}</span>
                        </dd>
                    </dl>
                    <ReviseCompanyForm
                        org_id=company.org_id
                        legal_name=company.legal_name
                        reg_no=company.reg_no
                        version=company.version
                    />
                }.into_any(),
            }}
        </section>
    }
}
