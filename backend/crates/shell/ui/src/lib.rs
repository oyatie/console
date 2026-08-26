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

pub use caps::{nav_items, path_allowed, SurfaceCaps};
pub use csp::{csp_header, CONTENT_SECURITY_POLICY};
pub use mount::{apply_csp_to_page, composition_holds};
pub use session::Session;
pub use shell::{render_path, RenderedPage, ShellData};

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
    }

    #[test]
    fn csp_header_helper() {
        let (name, value) = csp_header();
        assert_eq!(name, "content-security-policy");
        assert_eq!(value, CONTENT_SECURITY_POLICY);
        assert!(csp::csp_allows_wasm_eval_not_js_eval());
        assert!(value.contains("default-src 'self'"));
        assert!(value.contains("frame-ancestors 'none'"));
        assert!(value.contains("object-src 'none'"));
        assert!(!value.contains("unsafe-inline"));
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
        assert!(ess.is_empty(), "ESS must not see admin missing-company chips");
        let admin = blockers(&SurfaceCaps::payroll_admin(), &org, &payroll);
        let chips: Vec<_> = admin.iter().map(|b| b.chip).collect();
        assert!(chips.contains(&"회사 없음"));
        assert!(chips.contains(&"구성원 없음"));
        assert!(chips.contains(&"요율 없음"));
    }

    #[cfg(feature = "ssr")]
    mod ssr {
        use super::*;

        #[test]
        fn denied_nav_omitted_from_ssr_html() {
            let data = ShellData {
                session: Some(ess_session()),
                ..ShellData::default()
            };
            let page = render_path("/", &data);
            assert_eq!(page.status, 200);
            assert!(page.html.contains("내 급여"));
            assert!(!page.html.contains("href=\"/payroll/runs\""));
            assert!(!page.html.contains("급여 실행"));
            assert!(!page.html.contains("결재 수신함"));
            assert!(!page.html.contains("window.__TEST__"));
            assert!(!page.html.contains("Bearer "));
            assert!(!page.html.to_ascii_lowercase().contains("jwt"));
        }

        #[test]
        fn unauthorized_path_is_404_omit_not_403() {
            let data = ShellData {
                session: Some(ess_session()),
                ..ShellData::default()
            };
            let denied = render_path("/payroll/runs", &data);
            let unknown = render_path("/erp/maintenance", &data);
            assert_eq!(denied.status, 404);
            assert_eq!(unknown.status, 404);
            assert_eq!(denied.html, unknown.html);
            assert!(!denied.html.contains("403"));
            assert!(!denied.html.contains("급여 실행"));

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
        }

        #[test]
        fn csp_applied_to_rendered_page() {
            let page = render_path("/", &ShellData::default());
            let (status, [name, value], _html) = apply_csp_to_page(page);
            assert_eq!(status, 200);
            assert_eq!(name, "content-security-policy");
            assert!(value.contains("'wasm-unsafe-eval'"));
        }
    }
}
