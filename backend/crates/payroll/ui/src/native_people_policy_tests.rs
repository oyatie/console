// Include inside native_policy_validation_tests. Pure renderer fixtures only;
// these are never product data, browser provisioning, or authorization evidence.
use super::native_policy::{OriginalRecord, Outcome, PolicyAction, Subject};

fn people_scope(subject: Subject, installed: bool, active: bool) -> Scope {
    let mut value = scope(active);
    value.subject = subject;
    value.installed = installed;
    value.people_actions = if installed {
        vec![
            PolicyAction {
                subject: Subject::PeopleRead,
                operation: if subject == Subject::PeopleRead && !active {
                    Operation::Grant
                } else {
                    Operation::Revoke
                },
            },
            PolicyAction {
                subject: Subject::PeopleCreate,
                operation: if subject == Subject::PeopleCreate && active {
                    Operation::Revoke
                } else {
                    Operation::Grant
                },
            },
        ]
    } else {
        vec![]
    };
    if let Some(assignment) = value.assignment.as_mut() {
        assignment.state = "ACTIVE";
        assignment.label = "연결된 권한";
        assignment.can_grant = false;
        assignment.can_revoke = true;
    }
    value
}

fn people_form(scope: Scope, operation: Operation) -> String {
    render(Page::Form(Form {
        scope,
        operation,
        command: COMMAND.into(),
        proof: PROOF.into(),
        validation: None,
    }))
}

fn no_payroll_task(html: &str) {
    for wrong in [
        "급여 목록",
        "policy/payroll-read",
        "PayrollReadV1",
        "InstallPayrollReadCatalogV1",
        "href=\"#\"",
        "disabled",
    ] {
        assert!(
            !html.contains(wrong),
            "People task rendered unrelated or dead control"
        );
    }
}

fn actions(html: &str) {
    for (action, next, label, wrong) in [
        ("read", "revoke", "사람 열람 권한 회수", "grant"),
        ("create", "grant", "사람 등록 권한 연결", "revoke"),
    ] {
        assert!(html.contains(&format!(
            "href=\"/companies/{COMPANY}/policy/people-directory/{action}/{next}\""
        )));
        assert!(html.contains(label));
        assert!(!html.contains(&format!(
            "href=\"/companies/{COMPANY}/policy/people-directory/{action}/{wrong}\""
        )));
    }
}

#[test]
fn people_policy_install_prepares_both_without_granting_and_exposes_no_expiry() {
    let html = people_form(
        people_scope(Subject::PeopleCatalog, false, false),
        Operation::Install,
    );
    no_payroll_task(&html);
    assert!(html.contains("사람 등록·열람 권한"));
    assert!(html.contains("어떤 계정에도 권한이 연결되지 않습니다"));
    assert!(html.contains("data-policy-operation=\"InstallPeopleDirectoryCatalogV1\""));
    assert!(html.contains(&format!(
        "action=\"/companies/{COMPANY}/policy/people-directory/catalog\""
    )));
    assert!(html.contains(">권한 설정 준비</button>"));
    assert_eq!(html.matches("<form").count(), 1);
    assert_eq!(html.matches("<input").count(), 3);
    for name in ["command_id", "expected_company_epoch", "csrf_proof"] {
        input(&html, name);
    }
    assert!(!html.contains("expires_at_local") && !html.contains("recipient_account_id"));
}

#[test]
fn people_policy_read_create_forms_use_exact_action_paths_and_distinct_consequences() {
    for (subject, action, label, consequence) in [
        (
            Subject::PeopleRead,
            "read",
            "사람 열람 권한",
            "등록·고용 변경·급여 권한은 포함되지 않습니다",
        ),
        (
            Subject::PeopleCreate,
            "create",
            "사람 등록 권한",
            "사람 목록 열람이나 고용·급여 권한은 별도입니다",
        ),
    ] {
        for (operation, suffix, key, active) in [
            (Operation::Grant, "연결", "GrantPeopleDirectoryV1", false),
            (Operation::Revoke, "회수", "RevokePeopleDirectoryV1", true),
        ] {
            let html = people_form(people_scope(subject, true, active), operation);
            no_payroll_task(&html);
            assert!(html.contains(&format!("<h1>{label}</h1>")));
            assert!(html.contains(&format!(">{label} {suffix}</button>")));
            assert!(html.contains(consequence));
            assert!(html.contains(&format!("data-policy-operation=\"{key}\"")));
            let path = if active {
                format!("grants/{ASSIGNMENT}/revoke")
            } else {
                "grants".into()
            };
            assert!(html.contains(&format!(
                "action=\"/companies/{COMPANY}/policy/people-directory/{action}/{path}\""
            )));
            assert_eq!(html.matches("<form").count(), 1);
            assert_eq!(html.matches("<input").count(), if active { 5 } else { 8 });
            for name in [
                "command_id",
                "expected_company_epoch",
                "csrf_proof",
                "expected_role_revision",
                "expected_assignment_revision",
            ] {
                input(&html, name);
            }
            if !active {
                assert!(input(&html, "expires_at_local").contains("type=\"datetime-local\""));
            }
            if subject == Subject::PeopleCreate {
                assert!(html.contains("본인이 접수한 등록 요청"));
            }
        }
    }
}

#[test]
fn uninstalled_people_actions_link_only_to_real_shared_catalog_install() {
    for subject in [Subject::PeopleRead, Subject::PeopleCreate] {
        let html = people_form(people_scope(subject, false, false), Operation::Grant);
        assert!(!html.contains("<form") && !html.contains(PROOF));
        assert!(html.contains(&format!(
            "href=\"/companies/{COMPANY}/policy/people-directory/install\""
        )));
        for wrong in ["/read/install", "/create/install"] {
            assert!(!html.contains(wrong));
        }
        no_payroll_task(&html);
    }
}

#[test]
fn installed_people_catalog_keeps_each_authorized_action_without_false_no_grants() {
    let current = people_scope(Subject::PeopleCatalog, true, false);
    assert!(current.assignment.is_none());
    let html = people_form(current, Operation::Install);
    assert!(!html.contains("<form") && !html.contains(PROOF));
    assert!(!html.contains("연결된 권한 없음") && !html.contains("연결된 열람 권한 없음"));
    assert!(!html.contains("data-policy-current-state=\"NONE\""));
    actions(&html);
    no_payroll_task(&html);
}

#[test]
fn people_pending_results_retry_only_original_command_without_action_in_receipt_url() {
    for subject in [Subject::PeopleRead, Subject::PeopleCreate] {
        let html = render(Page::Result {
            scope: people_scope(subject, true, false),
            operation: Operation::Grant,
            command: COMMAND.into(),
            outcome: Outcome::Pending {
                accepted_at: "2026-09-23 09:00 KST".into(),
                deadline: "2026-09-23 10:00 KST".into(),
                proof: PROOF.into(),
            },
            original: OriginalRecord {
                accepted_at: "2026-09-23 09:00 KST".into(),
                deadline: "2026-09-23 10:00 KST".into(),
                intake_receipt: OTHER.into(),
                expected_company_epoch: "8".into(),
                expected_assignment: None,
                requested_until: None,
                effect_period: None,
                effect_epochs: None,
            },
        });
        let path = format!("/companies/{COMPANY}/policy/people-directory/requests/grant/{COMMAND}");
        assert!(html.contains(&format!("action=\"{path}/retry\"")));
        assert!(html.contains(&format!("href=\"{path}\"")));
        assert!(html.contains("data-policy-outcome=\"pending\""));
        assert_eq!(html.matches("<form").count(), 1);
        assert_eq!(html.matches("<input").count(), 1);
        assert!(input(&html, "csrf_proof").contains(&format!("value=\"{PROOF}\"")));
        for wrong in [
            "/read/requests",
            "/create/requests",
            "name=\"command_id\"",
            "name=\"action\"",
            "name=\"recipient_account_id\"",
        ] {
            assert!(!html.contains(wrong));
        }
        no_payroll_task(&html);
    }
}

#[test]
fn people_validation_preserves_original_action_and_disables_stale_resubmission() {
    for (subject, action) in [
        (Subject::PeopleRead, "read"),
        (Subject::PeopleCreate, "create"),
    ] {
        for current_matches in [true, false] {
            let html = page(
                people_scope(subject, true, false),
                Some(validation(false, current_matches)),
            );
            escaped_visible_expiry(&html);
            assert!(html.contains("role=\"alert\"") && html.contains("tabindex=\"-1\""));
            if current_matches {
                assert!(html.contains(&format!(
                    "action=\"/companies/{COMPANY}/policy/people-directory/{action}/grants\""
                )));
                assert!(input(&html, "command_id").contains(&format!("value=\"{COMMAND}\"")));
                assert!(
                    input(&html, "recipient_account_id")
                        .contains(&format!("value=\"{RECIPIENT}\""))
                );
                assert!(input(&html, "expires_at_local").contains("aria-invalid=\"true\""));
            } else {
                assert!(
                    !html.contains("<form") && !html.contains("<input") && !html.contains(PROOF)
                );
                assert!(html.contains(&format!(
                    "href=\"/companies/{COMPANY}/policy/people-directory/{action}/grant\""
                )));
            }
            no_payroll_task(&html);
        }
    }
}

#[test]
fn people_policy_workspace_projection_controls_real_links_and_empty_state() {
    use super::native_account;
    for visible in [false, true] {
        let html = native_account::render(native_account::Page::Company {
            org_id: COMPANY.into(),
            name: "회사 <연구 & 인사>".into(),
            slug: "scope".into(),
            show_policy_navigation: false,
            show_payroll_policy_navigation: false,
            show_payroll_navigation: false,
            people_policy: visible
                .then(|| people_scope(Subject::PeopleCatalog, true, false).people_actions),
        });
        assert_eq!(
            html.contains("data-company-destinations=\"empty\""),
            !visible
        );
        assert_eq!(
            html.contains("data-company-destination=\"people-policy\""),
            visible
        );
        if visible {
            assert!(html.contains(&format!(
                "href=\"/companies/{COMPANY}/policy/people-directory/install\""
            )));
            assert!(html.contains("사람 등록·열람 권한"));
            actions(&html);
        } else {
            assert!(!html.contains("/policy/people-directory"));
        }
        // Policy administration alone must not invent a business-directory link.
        assert!(!html.contains(&format!("href=\"/companies/{COMPANY}/people\"")));
        assert!(!html.contains("<연구") && html.contains("&lt;연구 &amp; 인사&gt;"));
    }
}
