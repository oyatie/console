//! Native Account documents. Inputs are safe projections composed by the app.
use leptos::prelude::*;

pub struct TermsItem {
    pub kind: String,
    pub title: String,
    pub content_url: String,
    pub content: String,
}

pub enum ContextState {
    Empty,
    Companies(Vec<(String, String)>),
    Unavailable,
}

pub enum CompanySetupEligibility {
    Eligible,
    Ineligible,
    Unavailable,
}

pub enum Page {
    Public,
    SignIn,
    Register {
        version: String,
        items: Vec<TermsItem>,
    },
    Account {
        context: ContextState,
        can_logout: bool,
        company_setup: CompanySetupEligibility,
    },
    CompanySetup {
        account_id: String,
        command_id: Option<String>,
    },
    CompanyCreated {
        company: Option<(String, String)>,
    },
    CompanyPending {
        command_id: String,
        name: String,
        slug: String,
        account_id: String,
    },
    CompanyTerminal {
        expired: bool,
    },
    CompanyUncertain,
    Company {
        org_id: String,
        name: String,
        slug: String,
        show_policy_navigation: bool,
        show_payroll_policy_navigation: bool,
    },
    CompanyPolicy {
        org_id: String,
        action_keys: Vec<&'static str>,
        delegable_action_keys: Vec<&'static str>,
    },
    Refused,
    Unavailable,
}

#[component]
fn AttemptStatus() -> impl IntoView {
    view! {
        <p id="native-status" class="status" role="status" aria-live="polite" tabindex="-1"></p>
        <p id="native-error" class="error" role="alert" hidden></p>
        <button id="native-cancel" class="button secondary" type="button" hidden>"취소"</button>
        <button id="native-recheck" class="button secondary" type="button" hidden>"결과 다시 확인"</button>
        <button id="native-retry-finish" class="button secondary" type="button" hidden>"같은 요청 다시 제출"</button>
        <a id="native-continue" class="text-link" href="/account" hidden>"로그인으로 확인"</a>
        <a id="native-reload-terms" class="text-link" href="/account/register" hidden>"새 약관 확인"</a>
        <noscript><p class="notice">"패스키를 사용하려면 이 브라우저에서 JavaScript를 허용해 주세요. 약관과 안내는 그대로 읽을 수 있습니다."</p></noscript>
    }
}

fn body(page: Page) -> AnyView {
    match page {
        Page::Public => view! {
            <section class="entry-card welcome">
                <p class="eyebrow">"CONSOLE"</p>
                <h1>"하나의 계정으로,"<br/>"업무를 이어가세요."</h1>
                <p class="lead">"로그인하면 지금 사용할 수 있는 업무 공간을 확인할 수 있습니다."</p>
                <div class="actions">
                    <a class="button primary" href="/account">"로그인"</a>
                    <a class="button secondary" href="/account/register">"계정 만들기"</a>
                </div>
                <p class="supporting">"아직 연결된 회사나 업무가 없어도 계정을 만들 수 있습니다."</p>
            </section>
        }.into_any(),
        Page::SignIn => view! {
            <section class="entry-card" data-account-state="anonymous">
                <p class="eyebrow">"다시 만나서 반갑습니다"</p>
                <h1>"Console에 로그인"</h1>
                <p class="lead">"기기에 저장된 패스키로 본인의 계정에 로그인하세요."</p>
                <form data-native-action="login">
                    <label for="native-account-choice">"패스키 계정 선택"</label>
                    <input id="native-account-choice" type="text" autocomplete="username webauthn" aria-describedby="native-choice-help"/>
                    <p id="native-choice-help" class="supporting">"아래 버튼을 누른 뒤 이 입력란에서 브라우저가 제안하는 패스키를 선택하세요. 이름이나 이메일을 입력할 필요는 없습니다."</p>
                    <button class="button primary wide" type="button" data-native-submit disabled>"패스키로 로그인"</button>
                    <AttemptStatus/>
                </form>
                <p class="other-path">"처음 방문하셨나요? "<a href="/account/register">"계정 만들기"</a></p>
            </section>
        }.into_any(),
        Page::Register { version, items } => {
            let terms = items.into_iter().enumerate().map(|(index, item)| {
                let id = format!("native-terms-{index}");
                view! {
                    <article class="terms-item">
                        <label class="check-label" for=id.clone()>
                            <input id=id.clone() type="checkbox" data-terms-kind=item.kind required/>
                            <span>{item.title}</span>
                        </label>
                        <details>
                            <summary>"약관 내용 보기"</summary>
                            <div class="terms-content">{item.content}</div>
                            <a href=item.content_url target="_blank" rel="noopener noreferrer">"문서 새 창으로 열기"</a>
                        </details>
                    </article>
                }
            }).collect_view();
            view! {
                <section class="entry-card registration" data-account-state="anonymous">
                    <p class="eyebrow">"내 계정의 시작"</p>
                    <h1>"계정 만들기"</h1>
                    <p class="lead">"약관을 확인하고 패스키를 만드세요. 회사 소속이나 직책은 지금 입력하지 않습니다."</p>
                    <form data-native-action="register" data-terms-version=version>
                        <fieldset>
                            <legend>"필수 약관을 각각 확인해 주세요"</legend>
                            {terms}
                        </fieldset>
                        <p class="supporting">"다음 단계에서 기기의 화면 잠금이나 보안 키로 패스키를 만듭니다."</p>
                        <button class="button primary wide" type="button" data-native-submit disabled>"패스키로 계정 만들기"</button>
                        <AttemptStatus/>
                    </form>
                    <p class="other-path">"이미 계정이 있나요? "<a href="/account">"로그인"</a></p>
                </section>
            }.into_any()
        },
        Page::Account { context, can_logout, company_setup } => {
            let setup = match company_setup {
                CompanySetupEligibility::Eligible => view! {
                    <a class="button primary" href="/account/companies/new">"회사 업무 공간 만들기"</a>
                }.into_any(),
                CompanySetupEligibility::Ineligible => ().into_any(),
                CompanySetupEligibility::Unavailable => view! {
                    <section class="workspace-state" role="alert">
                        <h2>"업무 공간 등록을 확인할 수 없습니다"</h2>
                        <p>"잠시 후 다시 확인해 주세요."</p>
                        <a href="">"다시 확인"</a>
                    </section>
                }.into_any(),
            };
            let workspace = match context {
                ContextState::Companies(companies) => view! {
                    <section class="workspace-state" data-context-state="populated">
                        <h2>"내 업무 공간"</h2>
                        <ul>{companies.into_iter().map(|(id, name)| view! {
                            <li><a href=format!("/companies/{id}")>{name}</a></li>
                        }).collect_view()}</ul>
                    </section>
                }.into_any(),
                ContextState::Empty => view! {
                    <section class="workspace-state" data-context-state="empty">
                        <span class="state-label">"연결된 업무 공간 없음"</span>
                        <h2>"계정은 준비되었습니다"</h2>
                        <p>"현재 이 계정으로 접근할 수 있는 업무 공간이 없습니다. 업무 접근 권한이 연결되면 이곳에서 이어갈 수 있습니다."</p>
                        <a href="/account">"다시 확인"</a>
                    </section>
                }.into_any(),
                ContextState::Unavailable => view! {
                    <section class="workspace-state" data-context-state="unavailable" role="alert">
                        <span class="state-label">"다시 확인이 필요합니다"</span>
                        <h2>"업무 공간을 확인할 수 없습니다"</h2>
                        <p>"계정 로그인은 확인했지만 업무 접근 정보를 불러오지 못했습니다. 잠시 후 다시 확인해 주세요."</p>
                        <a href="/account">"다시 확인"</a>
                    </section>
                }.into_any(),
            };
            view! {
                <section class="entry-card" data-account-state="active">
                    <p class="eyebrow">"내 CONSOLE"</p>
                    <h1>"계정에 로그인했습니다"</h1>
                    {workspace}
                    {setup}
                    {can_logout.then(|| view! {
                        <form data-native-action="logout" class="logout-form">
                            <button class="button secondary" type="button" data-native-submit disabled>"로그아웃"</button>
                            <AttemptStatus/>
                        </form>
                    })}
                </section>
            }.into_any()
        },
        Page::CompanySetup { account_id, command_id } => view! {
            <section class="entry-card" data-company-setup="" aria-labelledby="company-setup-title">
                <p class="eyebrow">"내 CONSOLE"</p>
                <h1 id="company-setup-title">"회사 업무 공간 만들기"</h1>
                <p class="lead">"기존 회사가 사용할 콘솔 업무 공간을 등록합니다."</p>
                {command_id.as_ref().map(|_| view! {
                    <p class="notice">"이 계정에 저장된 요청을 찾지 못했습니다. 입력 내용을 다시 확인하고 제출하면 현재 주소의 같은 요청 번호를 사용합니다."</p>
                })}
                <form data-company-enrollment="" data-account-id=account_id data-command-id=command_id>
                    <fieldset>
                        <legend>"회사 업무 공간 정보"</legend>
                        <label for="company-name">"회사 이름"</label>
                        <input id="company-name" name="name" type="text" autocomplete="organization" required maxlength="256" aria-describedby="company-name-help"/>
                        <p id="company-name-help" class="supporting">"업무에서 사용하는 회사 이름을 입력하세요."</p>
                        <label for="company-slug">"업무 공간 식별자"</label>
                        <input id="company-slug" name="slug" type="text" autocomplete="off" autocapitalize="none" spellcheck="false" required maxlength="63" pattern=r"[a-z0-9](?:[a-z0-9\-]{0,61}[a-z0-9])?" aria-describedby="company-slug-help"/>
                        <p id="company-slug-help" class="supporting">"영문 소문자와 숫자, 하이픈(-)으로 입력하세요. 처음과 끝에는 하이픈을 사용할 수 없습니다."</p>
                    </fieldset>
                    <section class="workspace-state" aria-labelledby="company-recipient-title">
                        <h2 id="company-recipient-title">"관리할 계정"</h2>
                        <p>"내 계정"</p>
                        <p>"이 계정에 이 회사의 정보 열람과 제한된 권한 관리 기능을 연결합니다. 급여·인사 권한은 포함되지 않습니다."</p>
                    </section>
                    <button class="button primary wide company-submit" type="submit" disabled>"회사 업무 공간 만들기"</button>
                    <p id="company-status" class="status" role="status" aria-live="polite" tabindex="-1"></p>
                    <p id="company-error" class="error" role="alert" hidden></p>
                    <a id="company-result" class="text-link" hidden>"요청 결과 확인"</a>
                    <noscript><p class="notice">"업무 공간을 등록하려면 이 브라우저에서 JavaScript를 허용해 주세요."</p></noscript>
                </form>
                <a href="/account">"내 계정으로"</a>
            </section>
        }.into_any(),
        Page::CompanyCreated { company } => view! {
            <section class="entry-card" data-company-outcome="committed">
                <p class="eyebrow">"회사 업무 공간"</p>
                <h1>"생성 완료"</h1>
                <p class="lead">"요청이 처리되었습니다. 이 주소에서 결과를 다시 확인할 수 있습니다."</p>
                {company.map(|(id, name)| view! {
                    <section class="workspace-state"><h2>{name}</h2>
                        <a class="button primary" href=format!("/companies/{id}")>"업무 공간 열기"</a>
                    </section>
                })}
                <a href="/account">"내 계정으로"</a>
            </section>
        }.into_any(),
        Page::CompanyPending { command_id, name, slug, account_id } => view! {
            <section class="entry-card" data-company-outcome="pending">
                <p class="eyebrow">"저장된 등록 요청"</p><h1>"아직 생성이 완료되지 않았습니다"</h1>
                <p class="lead">"입력한 내용이 저장되어 있습니다. 같은 요청으로 다시 시도하면 중복으로 만들지 않습니다."</p>
                <form data-company-enrollment="" data-account-id=account_id data-command-id=command_id>
                    <label for="company-name">"회사 이름"</label>
                    <input id="company-name" name="name" value=name readonly/>
                    <label for="company-slug">"업무 공간 식별자"</label>
                    <input id="company-slug" name="slug" value=slug readonly/>
                    <button class="button primary wide company-submit" type="submit" disabled>"같은 요청으로 다시 시도"</button>
                    <p class="supporting">"등록 요청을 취소하면 다시 제출할 수 없습니다. 이미 생성이 완료된 업무 공간은 취소되지 않습니다."</p>
                    <button class="button secondary" type="button" data-company-cancel disabled>"등록 요청 취소"</button>
                    <p id="company-status" class="status" role="status" aria-live="polite" tabindex="-1"></p>
                    <p id="company-error" class="error" role="alert" hidden></p>
                    <a id="company-result" class="text-link" hidden>"요청 결과 확인"</a>
                    <noscript><p class="notice">"다시 제출하려면 이 브라우저에서 JavaScript를 허용해 주세요."</p></noscript>
                </form>
                <a href="/account">"내 계정으로"</a>
            </section>
        }.into_any(),
        Page::CompanyUncertain => view! {
            <section class="entry-card" data-company-outcome="uncertain">
                <p class="eyebrow">"원래 요청 확인"</p><h1>"아직 결과를 확인할 수 없습니다"</h1>
                <p class="lead">"요청이 처리 중일 수 있습니다. 새로 등록하지 말고 이 주소에서 같은 요청의 결과를 다시 확인해 주세요."</p>
                <a class="button primary" href="">"같은 요청 결과 다시 확인"</a>
                <a class="text-link" href="/account">"내 계정으로"</a>
            </section>
        }.into_any(),
        Page::CompanyTerminal { expired } => view! {
            <section class="entry-card" data-company-outcome=if expired { "expired" } else { "cancelled" }>
                <p class="eyebrow">"회사 등록 요청"</p>
                <h1>{if expired { "요청이 만료되었습니다" } else { "요청이 취소되었습니다" }}</h1>
                <p class="lead">"이 요청으로 생성된 회사 업무 공간은 없습니다."</p>
                <a class="button primary" href="/account">"내 계정으로"</a>
            </section>
        }.into_any(),
        Page::Company { org_id, name, slug, show_policy_navigation, show_payroll_policy_navigation } => view! {
            <section class="entry-card" data-company-id=org_id.clone()>
                <p class="eyebrow">"회사 업무 공간"</p><h1>{name}</h1>
                <dl><dt>"업무 공간 식별자"</dt><dd>{slug}</dd></dl>
                {show_policy_navigation.then(|| view! {
                    <a class="button primary" href=format!("/companies/{org_id}/policy")>"권한 관리"</a>
                })}
                {show_payroll_policy_navigation.then(|| view! {
                    <a class="button secondary" href=format!("/companies/{org_id}/policy/payroll-read/install")>"급여 목록 열람 권한"</a>
                })}
                <a class="text-link" href="/account">"내 업무 공간 목록"</a>
            </section>
        }.into_any(),
        Page::CompanyPolicy { org_id, action_keys, delegable_action_keys } => view! {
            <section class="entry-card">
                <p class="eyebrow">"회사 업무 공간"</p><h1>"권한 관리"</h1>
                <p class="lead">"이 회사에서 현재 계정에 연결된 권한입니다."</p>
                <h2>"사용할 수 있는 기능"</h2>
                <ul>{action_keys.into_iter().map(|key| view! { <li>{company_action_label(key)}</li> }).collect_view()}</ul>
                <h2>"다른 계정에 연결할 수 있는 범위"</h2>
                <ul>{delegable_action_keys.into_iter().map(|key| view! { <li>{company_action_label(key)}</li> }).collect_view()}</ul>
                <p>"회사 정보는 이름과 업무 공간 식별자만 포함됩니다. 급여·인사 정보와 다른 회사의 권한은 포함되지 않습니다."</p>
                <a class="button secondary" href=format!("/companies/{org_id}")>"회사 업무 공간으로"</a>
            </section>
        }.into_any(),
        Page::Refused => view! {
            <section class="entry-card">
                <p class="eyebrow">"요청 확인"</p><h1>"이 요청을 열 수 없습니다"</h1>
                <p class="lead">"Console의 로그인 화면에서 다시 시작해 주세요."</p>
                <a class="button primary" href="/account">"로그인 화면으로"</a>
            </section>
        }.into_any(),
        Page::Unavailable => view! {
            <section class="entry-card" role="alert">
                <p class="eyebrow">"잠시 기다려 주세요"</p><h1>"지금은 계정을 확인할 수 없습니다"</h1>
                <p class="lead">"계정이나 약관 정보를 불러오지 못했습니다. 잠시 후 다시 시도해 주세요."</p>
                <a class="button primary" href="">"다시 시도"</a>
                <a class="text-link" href="/">"시작 화면으로"</a>
            </section>
        }.into_any(),
    }
}

fn company_action_label(key: &str) -> &'static str {
    match key {
        "context.discover" => "업무 공간 찾기",
        "company.identity.read" => "회사 이름과 식별자 열람",
        "company.policy.read" => "회사 권한 열람",
        "company.policy.assign" => "제한된 열람 권한 연결",
        "company.policy.revoke" => "연결한 열람 권한 해제",
        _ => "등록된 기능",
    }
}

pub fn render(page: Page) -> String {
    let title = match &page {
        Page::Public => "Console · 업무의 연결",
        Page::SignIn => "로그인 · Console",
        Page::Register { .. } => "계정 만들기 · Console",
        Page::Account { .. } => "내 계정 · Console",
        Page::CompanySetup { .. } => "회사 업무 공간 만들기 · Console",
        Page::CompanyCreated { .. }
        | Page::CompanyPending { .. }
        | Page::CompanyTerminal { .. }
        | Page::CompanyUncertain => "회사 등록 요청 · Console",
        Page::Company { .. } => "회사 업무 공간 · Console",
        Page::CompanyPolicy { .. } => "권한 관리 · Console",
        Page::Refused => "요청 확인 · Console",
        Page::Unavailable => "다시 시도 · Console",
    };
    let content = body(page);
    let html = view! {
        <html lang="ko">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <title>{title}</title>
                <link rel="stylesheet" href="/assets/native-account.css"/>
                <script type="module" src="/assets/native-account.js"></script>
            </head>
            <body>
                <a class="skip-link" href="#main-content">"본문 바로가기"</a>
                <header class="entry-header"><a class="brand" href="/" aria-label="Console 시작 화면">"Console"<span class="brand-dot" aria-hidden="true"></span></a></header>
                <main id="main-content" tabindex="-1">{content}</main>
                <footer class="entry-footer">"필요한 업무와 정보가 한곳에."</footer>
            </body>
        </html>
    }.to_html();
    format!("<!DOCTYPE html>{html}")
}

#[cfg(feature = "ssr")]
pub fn document(page: Page, status: axum::http::StatusCode) -> axum::response::Response {
    super::ssr::private_document(
        render(page),
        status,
        "default-src 'self'; script-src 'self'; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'",
    )
}
