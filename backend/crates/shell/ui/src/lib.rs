//! Payroll-vertical Leptos SSR shell. Islands hydrate; nav/lists do not.
//!
//! App composition (HOLD): wire ports in `console-app`, serve WASM same-origin,
//! keep the session in cookies/headers. This crate embeds no access tokens.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod caps;
pub mod csp;
pub mod hub;
pub mod islands;
pub mod login;
pub mod mount;
pub mod nav;
pub mod not_found;
pub mod policy;
pub mod session;
pub mod shell;
pub mod style;

pub use caps::{SurfaceCaps, nav_items, path_allowed};
pub use csp::{CONTENT_SECURITY_POLICY, csp_header};
pub use mount::{apply_csp_to_page, composition_holds};
pub use session::Session;
pub use shell::{RenderedPage, ShellData, render_path};

pub fn link_islands() {
    islands::link_islands();
    console_ontology_ui::link_islands();
    console_payroll_ui::link_islands();
}

#[cfg(feature = "hydrate")]
pub fn hydrate() {
    link_islands();
    leptos::mount::hydrate_islands();
}

#[cfg(not(feature = "hydrate"))]
pub fn hydrate() {
    link_islands();
}

#[cfg(test)]
mod tests {
    use console_ontology_ui::{CompanyHead, FailClosedOrg, OrgReadPort};
    use console_payroll_ui::{FailClosedPayroll, PayRunSummary, PayrollReadPort};

    use super::*;
    use crate::hub::blockers;
    use crate::nav::nav_contains;

    fn admin_session() -> Session {
        Session {
            user_id: "user-admin".into(),
            display_name: "관리자".into(),
            caps: SurfaceCaps::payroll_admin(),
        }
    }

    fn ess_session() -> Session {
        Session {
            user_id: "user-ess".into(),
            display_name: "직원".into(),
            caps: SurfaceCaps::ess_only(),
        }
    }

    #[test]
    fn island_modules_exist() {
        let src = include_str!("islands.rs");
        assert!(src.contains("#[island]"));
        for name in islands::ISLAND_NAMES {
            assert!(src.contains(name), "missing island {name}");
        }
        link_islands();
        assert_eq!(islands::POST_LOGIN, "/_ui/login");
        assert!(islands::POST_LOGIN.starts_with("/_ui/"));
        assert!(!islands::POST_LOGIN.starts_with("/ui/"));
    }

    #[test]
    fn csp_header_helper() {
        let (name, value) = csp_header();
        assert_eq!(name, "content-security-policy");
        assert_eq!(value, CONTENT_SECURITY_POLICY);
        assert!(csp::csp_allows_wasm_eval_not_js_eval());
        assert!(csp::csp_allows_hashed_style_not_unsafe_inline());
        assert!(value.contains("default-src 'self'"));
        assert!(value.contains("style-src 'self'"));
        assert!(value.contains(csp::STYLE_SRC_SHA256));
        assert!(value.contains("frame-ancestors 'none'"));
        assert!(value.contains("object-src 'none'"));
        assert!(!value.contains("unsafe-inline"));
        // Bind the hashed bytes to this CSS: length drift means recompute sha256.
        assert_eq!(crate::style::CSS.len(), 841);
        assert!(!crate::style::CSS.contains("prefers-color-scheme"));
        assert!(!crate::style::CSS.contains("unsafe-inline"));
    }

    #[test]
    fn denied_nav_omitted() {
        let caps = SurfaceCaps::ess_only();
        assert!(nav_contains(&caps, "/payroll/me"));
        assert!(!nav_contains(&caps, "/payroll/runs"));
        assert!(!nav_contains(&caps, "/approvals"));
        assert!(!nav_contains(&caps, "/company"));
        assert!(!path_allowed("/payroll/runs", &caps));
        assert!(!path_allowed("/approvals", &caps));
        assert!(path_allowed("/payroll/me", &caps));
    }

    #[test]
    fn empty_state_is_create_not_import() {
        let org = FailClosedOrg.snapshot();
        let payroll = FailClosedPayroll.snapshot();
        assert!(org.company.is_none());
        assert!(payroll.runs.is_empty());
        assert!(payroll.my_payslip.is_none());
        assert!(payroll.inbox.is_empty());
        let src = include_str!("islands.rs");
        let lower = src.to_ascii_lowercase();
        assert!(src.contains("로그인"));
        assert!(!lower.contains("import"));
        assert!(!src.contains("엑셀"));
    }

    #[test]
    fn hub_blockers_omit_unauthorized_surfaces() {
        let org = console_ontology_ui::OrgSnapshot::default();
        let payroll = console_payroll_ui::PayrollSnapshot::default();
        let ess = blockers(&SurfaceCaps::ess_only(), &org, &payroll);
        assert!(
            ess.is_empty(),
            "ESS must not see admin missing-company chips"
        );
        let admin = blockers(&SurfaceCaps::payroll_admin(), &org, &payroll);
        let chips: Vec<_> = admin.iter().map(|b| b.chip).collect();
        assert!(chips.contains(&"회사 없음"));
        assert!(chips.contains(&"구성원 없음"));
        assert!(chips.contains(&"요율 없음"));
    }

    #[cfg(feature = "ssr")]
    mod ssr {
        use console_payroll_ui::{DecideInboxItem, Lineage, MoneyLine, MyPayslip};

        use super::*;

        #[test]
        fn denied_nav_omitted_from_ssr_html() {
            let mut data = ShellData {
                session: Some(ess_session()),
                ..ShellData::default()
            };
            data.payroll.runs.push(PayRunSummary {
                id: "run-secret".into(),
                period_start: "2026-08-01".into(),
                period_end: "2026-08-31".into(),
                status: "SUBMITTED".into(),
                submitted_by: Some("other".into()),
                decided_by: None,
                exceptions_open: 0,
            });
            data.payroll.inbox.push(DecideInboxItem {
                run_id: "run-secret".into(),
                period_start: "2026-08-01".into(),
                period_end: "2026-08-31".into(),
                submitted_by: "other".into(),
                submitted_at: "2026-08-20T00:00:00Z".into(),
            });
            let page = render_path("/", &data);
            assert_eq!(page.status, 200);
            assert!(page.html.contains("내 급여"));
            assert!(!page.html.contains("href=\"/payroll/runs\""));
            assert!(!page.html.contains("급여 실행"));
            assert!(!page.html.contains("결재 수신함"));
            assert!(!page.html.contains("run-secret"));
            assert!(!page.html.contains("/_ui/approvals/decide"));
            assert!(!page.html.contains("/_ui/payroll/runs"));
            assert!(!page.html.contains("window.__TEST__"));
            assert!(!page.html.contains("Bearer "));
            assert!(!page.html.to_ascii_lowercase().contains("jwt"));

            let ess = render_path("/payroll/me", &data);
            assert_eq!(ess.status, 200);
            assert!(ess.html.contains("열람할 명세서가 없습니다."));
            assert!(ess.html.contains("미발행"));
            assert!(!ess.html.contains("0원"));
            assert!(!ess.html.contains("run-secret"));
            assert!(!ess.html.contains("결재 수신함"));
            assert!(!ess.html.contains("/_ui/approvals/decide"));
            assert!(!ess.html.contains("/_ui/payroll/runs"));

            data.payroll.my_payslip = Some(MyPayslip {
                period_start: "2026-08-01".into(),
                period_end: "2026-08-31".into(),
                employee_name: "직원".into(),
                base_pay_won: None,
                earnings: vec![MoneyLine {
                    code: "OT".into(),
                    label_ko: "연장수당".into(),
                    amount_won: None,
                    lineage: Lineage {
                        label_ko: "연장".into(),
                        source_ko: "서버".into(),
                    },
                    overridable: false,
                }],
                deductions: Vec::new(),
                net_pay_won: None,
                net_pay_unavailable_reason_ko: None,
                citations: Vec::new(),
            });
            let slip = render_path("/payroll/me", &data);
            assert_eq!(slip.status, 200);
            assert!(slip.html.contains("기본급"));
            assert!(slip.html.contains("계산 불가 / 미발행"));
            assert!(!slip.html.contains("0원"));
            assert!(!slip.html.contains("run-secret"));
        }

        #[test]
        fn unauthorized_path_is_404_omit_not_403() {
            let mut data = ShellData {
                session: Some(ess_session()),
                ..ShellData::default()
            };
            data.payroll.runs.push(PayRunSummary {
                id: "run-hidden".into(),
                period_start: "2026-08-01".into(),
                period_end: "2026-08-31".into(),
                status: "SUBMITTED".into(),
                submitted_by: Some("other".into()),
                decided_by: None,
                exceptions_open: 0,
            });
            data.payroll.my_payslip = Some(MyPayslip {
                period_start: "2026-08-01".into(),
                period_end: "2026-08-31".into(),
                employee_name: "숨긴직원".into(),
                base_pay_won: Some(1),
                earnings: Vec::new(),
                deductions: Vec::new(),
                net_pay_won: Some(1),
                net_pay_unavailable_reason_ko: None,
                citations: Vec::new(),
            });
            let denied = render_path("/payroll/runs", &data);
            let unknown = render_path("/erp/maintenance", &data);
            assert_eq!(denied.status, 404);
            assert_eq!(unknown.status, 404);
            assert_eq!(denied.html, unknown.html);
            assert!(!denied.html.contains("403"));
            assert!(!denied.html.contains("급여 실행"));
            assert!(!denied.html.contains("run-hidden"));
            assert!(!denied.html.contains("숨긴직원"));
            assert!(!denied.html.contains("1원"));

            let authorized = ShellData {
                session: Some(admin_session()),
                ..ShellData::default()
            };
            let missing = render_path("/payroll/runs/missing", &authorized);
            assert_eq!(missing.status, 404);
            assert_eq!(missing.html, unknown.html);
            assert!(!missing.html.contains("급여 실행"));
        }

        #[test]
        fn unauthenticated_fails_closed_to_login() {
            let data = ShellData::default();
            let page = render_path("/payroll/me", &data);
            assert_eq!(page.status, 404);
            let login = render_path("/", &data);
            assert_eq!(login.status, 200);
            assert!(login.html.contains("로그인"));
            assert!(!login.html.contains("급여 실행"));
            assert!(login.html.contains("/_ui/login"));
        }

        #[test]
        fn empty_runs_page_has_create_not_import() {
            let mut data = ShellData {
                session: Some(admin_session()),
                ..ShellData::default()
            };
            data.org.company = Some(CompanyHead {
                org_id: "org-1".into(),
                legal_name: "주식회사 예시".into(),
                reg_no: String::new(),
                version: 1,
            });
            let page = render_path("/payroll/runs", &data);
            assert_eq!(page.status, 200);
            assert!(page.html.contains("등록"));
            let lower = page.html.to_ascii_lowercase();
            assert!(!lower.contains("import"));
            assert!(!page.html.contains("가져오기"));
            assert!(!page.html.contains("엑셀"));
            assert!(page.html.contains("/_ui/payroll/runs"));
            assert!(!page.html.contains("action=\"/ui/payroll/runs\""));
        }

        #[test]
        fn run_detail_has_no_inline_decide() {
            let mut data = ShellData {
                session: Some(admin_session()),
                ..ShellData::default()
            };
            data.payroll.runs.push(PayRunSummary {
                id: "run-1".into(),
                period_start: "2026-08-01".into(),
                period_end: "2026-08-31".into(),
                status: "SUBMITTED".into(),
                submitted_by: Some("other".into()),
                decided_by: None,
                exceptions_open: 0,
            });
            let page = render_path("/payroll/runs/run-1", &data);
            assert_eq!(page.status, 200);
            assert!(page.html.contains("결재는 수신함에서"));
            assert!(!page.html.contains("/ui/approvals/decide"));
            assert!(!page.html.contains("/_ui/approvals/decide"));
            assert!(!page.html.contains("action=\"/_ui/approvals/decide\""));
        }

        #[test]
        fn csp_applied_to_rendered_page() {
            let page = render_path("/", &ShellData::default());
            let (status, [name, value], html) = apply_csp_to_page(page);
            assert_eq!(status, 200);
            assert_eq!(name, "content-security-policy");
            assert!(value.contains("'wasm-unsafe-eval'"));
            assert!(value.contains("style-src 'self'"));
            assert!(value.contains(csp::STYLE_SRC_SHA256));
            assert!(!value.contains("unsafe-inline"));
            let style_tag = format!("<style>{}</style>", crate::style::CSS);
            assert!(
                html.contains(&style_tag),
                "inline CSS must match the hashed style-src bytes"
            );
            assert!(!html.contains("prefers-color-scheme"));
            assert!(html.contains("/_ui/login"));
            assert!(!html.contains("action=\"/ui/login\""));
            assert!(html.contains("/ui/pkg/console_shell_hydrate.js"));
        }
    }
}
