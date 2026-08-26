//! Company / org / people SSR surfaces. Unauthorized markup is never composed
//! by the shell; this crate only renders authorized snapshots.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod company;
pub mod islands;
pub mod org;
pub mod people;
pub mod ports;

pub use company::CompanyPage;
pub use islands::{
    AppointEmploymentForm, CreateJobPositionForm, CreateOrgUnitForm, CreatePersonForm,
    ReviseCompanyForm, link_islands,
};
pub use org::OrgPage;
pub use people::PeoplePage;
pub use ports::{
    CompanyHead, CompanyReadPort, CompanyWritePort, EmploymentHead, FailClosedOrg, JobPositionHead,
    OrgReadPort, OrgSnapshot, OrgUnitHead, PersonHead, WriteError,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn island_modules_exist() {
        let src = include_str!("islands.rs");
        assert!(src.contains("#[island]"));
        for name in islands::ISLAND_NAMES {
            assert!(src.contains(name), "missing island {name}");
        }
        link_islands();
        for path in [
            ports::POST_COMPANY_REVISE,
            ports::POST_ORG_UNIT_CREATE,
            ports::POST_JOB_POSITION_CREATE,
            ports::POST_PERSON_CREATE,
            ports::POST_EMPLOYMENT_APPOINT,
        ] {
            assert!(
                path.starts_with("/_ui/"),
                "form POST must target /_ui/*, got {path}"
            );
            assert!(!path.starts_with("/ui/"), "stale /ui form path: {path}");
        }
    }

    #[test]
    fn empty_state_is_create_not_import() {
        let src = [
            include_str!("company.rs"),
            include_str!("org.rs"),
            include_str!("people.rs"),
            include_str!("islands.rs"),
        ]
        .join("\n");
        let lower = src.to_ascii_lowercase();
        assert!(src.contains("등록") || src.contains("개정") || src.contains("발령"));
        assert!(!lower.contains("import"));
        assert!(!src.contains("가져오기"));
        assert!(!src.contains("엑셀"));
        assert!(!lower.contains("storeexport"));
        assert!(!FailClosedOrg.head().is_some());
        let company = include_str!("company.rs");
        let empty_arm = company
            .split("None =>")
            .nth(1)
            .expect("empty company None arm")
            .split("Some(company)")
            .next()
            .expect("empty company arm precedes Some");
        assert!(empty_arm.contains("등록된 회사가 없습니다."));
        assert!(
            !empty_arm.contains("ReviseCompanyForm"),
            "empty company must omit revise; no org_id=\"\" version=0 write"
        );
        assert!(!company.contains("org_id=String::new()"));
        assert!(!company.contains("version=0"));
        assert!(company.contains("ReviseCompanyForm"));
    }

    #[test]
    fn write_port_fails_closed() {
        let err = FailClosedOrg
            .revise("actor", "org", 1, "상호", "")
            .unwrap_err();
        assert_eq!(err, WriteError::FailClosed);
    }
}
