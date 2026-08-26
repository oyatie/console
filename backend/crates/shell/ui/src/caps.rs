//! Deny-by-omission capabilities. The full matrix never enters an island.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceCaps {
    pub company: bool,
    pub org: bool,
    pub people: bool,
    pub policy: bool,
    pub attendance: bool,
    pub payroll: bool,
    pub payroll_manage: bool,
    pub ess: bool,
    pub approvals: bool,
}

impl SurfaceCaps {
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn ess_only() -> Self {
        Self {
            ess: true,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn payroll_admin() -> Self {
        Self {
            company: true,
            org: true,
            people: true,
            policy: true,
            attendance: true,
            payroll: true,
            payroll_manage: true,
            ess: true,
            approvals: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavItem {
    pub href: &'static str,
    pub label_ko: &'static str,
}

#[must_use]
pub fn nav_items(caps: &SurfaceCaps) -> Vec<NavItem> {
    let mut items = vec![NavItem {
        href: "/",
        label_ko: "작업 허브",
    }];
    if caps.company {
        items.push(NavItem {
            href: "/company",
            label_ko: "회사",
        });
    }
    if caps.org {
        items.push(NavItem {
            href: "/org",
            label_ko: "조직",
        });
    }
    if caps.people {
        items.push(NavItem {
            href: "/people",
            label_ko: "구성원",
        });
    }
    if caps.policy {
        items.push(NavItem {
            href: "/policy",
            label_ko: "권한 폴드",
        });
    }
    if caps.attendance {
        items.push(NavItem {
            href: "/attendance",
            label_ko: "근태 인수",
        });
    }
    if caps.payroll {
        items.push(NavItem {
            href: "/payroll/runs",
            label_ko: "급여 실행",
        });
    }
    if caps.ess {
        items.push(NavItem {
            href: "/payroll/me",
            label_ko: "내 급여",
        });
    }
    if caps.approvals {
        items.push(NavItem {
            href: "/approvals",
            label_ko: "결재 수신함",
        });
    }
    items
}

#[must_use]
pub fn path_allowed(path: &str, caps: &SurfaceCaps) -> bool {
    let path = normalize_path(path);
    if path == "/" {
        return true;
    }
    if path == "/company" {
        return caps.company;
    }
    if path == "/org" {
        return caps.org;
    }
    if path == "/people" {
        return caps.people;
    }
    if path == "/policy" {
        return caps.policy;
    }
    if path == "/attendance" {
        return caps.attendance;
    }
    if path == "/payroll/me" {
        return caps.ess;
    }
    if path == "/payroll/runs" || path.starts_with("/payroll/runs/") {
        return caps.payroll;
    }
    if path == "/approvals" {
        return caps.approvals;
    }
    false
}

#[must_use]
pub fn normalize_path(path: &str) -> &str {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() { "/" } else { trimmed }
}
