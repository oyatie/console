//! Native workspace presentation. The isolated preview receives sample projections;
//! no account, policy, database, calculation or business command runs here.

#[derive(Clone, Debug)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub number: String,
    pub email: String,
    pub status: String,
    pub joined: String,
}

#[derive(Clone, Debug)]
pub struct Unit {
    pub id: String,
    pub name: String,
    pub parent: Option<String>,
    pub manager: String,
}

#[derive(Clone, Debug)]
pub struct Employment {
    pub id: String,
    pub person_id: String,
    pub unit_id: String,
    pub job: String,
    pub kind: String,
    pub start: String,
    pub end: Option<String>,
    pub status: String,
    pub history: Vec<Activity>,
}

#[derive(Clone, Debug)]
pub struct Activity {
    pub date: String,
    pub actor: String,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct Difference {
    pub label: String,
    pub before: String,
    pub after: String,
}

#[derive(Clone, Debug)]
pub struct Evidence {
    pub title: String,
    pub revision: String,
    pub detail: String,
}

#[derive(Clone, Debug)]
pub struct Approval {
    pub id: String,
    pub title: String,
    pub employment_id: String,
    pub requester: String,
    pub reviewer: String,
    pub effective: String,
    pub due: String,
    pub status: String,
    pub differences: Vec<Difference>,
    pub evidence: Vec<Evidence>,
    pub consequence: String,
    pub history: Vec<Activity>,
}

#[derive(Clone, Debug)]
pub struct PayrollLine {
    pub person_id: String,
    pub gross: i64,
    pub deductions: i64,
    pub net: i64,
    pub status: String,
}

#[derive(Clone, Debug)]
pub struct PayrollIssue {
    pub person_id: String,
    pub title: String,
    pub detail: String,
    pub owner: String,
    pub approval_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PayRun {
    pub id: String,
    pub period: String,
    pub payday: String,
    pub owner: String,
    pub revision: String,
    pub status: String,
    pub gross: i64,
    pub deductions: i64,
    pub net: i64,
    pub lines: Vec<PayrollLine>,
    pub issues: Vec<PayrollIssue>,
    pub history: Vec<Activity>,
}

#[derive(Clone, Debug)]
pub struct WorkspaceData {
    pub company: String,
    pub group: String,
    pub operator: String,
    pub people: Vec<Person>,
    pub units: Vec<Unit>,
    pub employments: Vec<Employment>,
    pub approvals: Vec<Approval>,
    pub payroll: PayRun,
}

#[derive(Clone, Debug)]
pub enum Page {
    Work,
    People {
        query: String,
    },
    Person {
        id: String,
    },
    Employments,
    Employment {
        id: String,
    },
    EmploymentChange {
        id: String,
        unit: String,
        job: String,
        effective: String,
        reason: String,
        compare: bool,
    },
    Organization {
        unit: Option<String>,
    },
    Approvals {
        outbox: bool,
    },
    Approval {
        id: String,
    },
    Payroll,
    Missing,
}

/// Unmounted test-first boundary. Implementation follows the reviewed semantic RED.
/// The real Console application never calls this sample-preview renderer.
pub fn render_preview(_data: &WorkspaceData, _page: &Page) -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn story() -> WorkspaceData {
        WorkspaceData {
            company: "한빛서비스".into(),
            group: "한빛그룹".into(),
            operator: "이지원".into(),
            people: vec![Person {
                id: "p-101".into(),
                name: "김민지".into(),
                number: "HB-0101".into(),
                email: "minji@example.test".into(),
                status: "재직".into(),
                joined: "2023-03-02".into(),
            }],
            units: vec![
                Unit {
                    id: "u-operations".into(),
                    name: "운영지원팀".into(),
                    parent: None,
                    manager: "박서준".into(),
                },
                Unit {
                    id: "u-people".into(),
                    name: "인사팀".into(),
                    parent: None,
                    manager: "이지원".into(),
                },
            ],
            employments: vec![Employment {
                id: "e-101".into(),
                person_id: "p-101".into(),
                unit_id: "u-operations".into(),
                job: "운영 담당".into(),
                kind: "정규직".into(),
                start: "2023-03-02".into(),
                end: None,
                status: "재직".into(),
                history: vec![Activity {
                    date: "2023-03-02".into(),
                    actor: "이지원".into(),
                    description: "입사 및 운영지원팀 배정".into(),
                }],
            }],
            approvals: vec![Approval {
                id: "ap-2041".into(),
                title: "김민지 부서 이동".into(),
                employment_id: "e-101".into(),
                requester: "박서준".into(),
                reviewer: "이지원".into(),
                effective: "2026-10-01".into(),
                due: "2026-09-25".into(),
                status: "검토 대기".into(),
                differences: vec![Difference {
                    label: "소속 조직".into(),
                    before: "운영지원팀".into(),
                    after: "인사팀".into(),
                }],
                evidence: vec![Evidence {
                    title: "인사 발령안".into(),
                    revision: "제안 2".into(),
                    detail: "김민지의 인사팀 이동 제안".into(),
                }],
                consequence: "승인 후 발효일에 소속이 변경됩니다. 현재 급여는 변경하지 않습니다."
                    .into(),
                history: vec![Activity {
                    date: "2026-09-21".into(),
                    actor: "박서준".into(),
                    description: "부서 이동 검토 요청".into(),
                }],
            }],
            payroll: PayRun {
                id: "pr-202609".into(),
                period: "2026년 9월".into(),
                payday: "2026-09-25".into(),
                owner: "이지원".into(),
                revision: "계산안 3".into(),
                status: "예외 검토".into(),
                gross: 3200000,
                deductions: 320000,
                net: 2880000,
                lines: vec![PayrollLine {
                    person_id: "p-101".into(),
                    gross: 3200000,
                    deductions: 320000,
                    net: 2880000,
                    status: "확인 필요".into(),
                }],
                issues: vec![PayrollIssue {
                    person_id: "p-101".into(),
                    title: "부서 이동 발효일 확인".into(),
                    detail: "10월 발령의 9월 급여 적용 여부를 확인합니다.".into(),
                    owner: "박서준".into(),
                    approval_id: Some("ap-2041".into()),
                }],
                history: vec![],
            },
        }
    }

    #[test]
    fn connected_workspace_exposes_task_context_and_all_five_workflows() {
        let data = story();
        let pages = [
            Page::Work,
            Page::People {
                query: String::new(),
            },
            Page::Person { id: "p-101".into() },
            Page::Employments,
            Page::Employment { id: "e-101".into() },
            Page::Organization { unit: None },
            Page::Approvals { outbox: false },
            Page::Approval {
                id: "ap-2041".into(),
            },
            Page::Payroll,
        ];
        for page in &pages {
            let html = render_preview(&data, page);
            assert!(
                html.contains("data-workspace-preview=\"synthetic\""),
                "preview truth missing on {page:?}"
            );
            assert!(
                html.contains("실제 업무에 반영되지 않습니다"),
                "preview must not imply persistence"
            );
            assert!(html.contains("한빛서비스"), "Company scope missing");
            assert!(
                html.contains("href=\"#main-content\""),
                "keyboard skip link missing"
            );
            assert!(
                html.contains("aria-label=\"주요 탐색\""),
                "navigation landmark missing"
            );
            for route in [
                "/work",
                "/people",
                "/employment",
                "/organization",
                "/approvals",
                "/payroll",
            ] {
                assert!(
                    html.contains(&format!("href=\"{route}\"")),
                    "missing connected destination {route} on {page:?}"
                );
            }
            assert!(
                !html.contains("/api/"),
                "entity navigation must not lead to JSON"
            );
            assert!(
                !html.contains("<script"),
                "SSR preview needs no browser runtime"
            );
        }
        let work = render_preview(&data, &Page::Work);
        for text in [
            "김민지 부서 이동",
            "이지원",
            "2026-09-25",
            "부서 이동 발효일 확인",
        ] {
            assert!(work.contains(text), "work task lacks {text}");
        }
        let employments = render_preview(&data, &Page::Employments);
        for text in [
            "김민지",
            "정규직",
            "운영지원팀",
            "href=\"/employment/e-101\"",
        ] {
            assert!(employments.contains(text), "employment list lacks {text}");
        }
        let organization = render_preview(
            &data,
            &Page::Organization {
                unit: Some("u-operations".into()),
            },
        );
        for text in ["운영지원팀", "박서준", "김민지", "href=\"/people/p-101\""] {
            assert!(
                organization.contains(text),
                "selected organization lacks {text}"
            );
        }
        let person = render_preview(&data, &Page::Person { id: "p-101".into() });
        for text in [
            "김민지",
            "HB-0101",
            "정규직",
            "2023-03-02",
            "입사 및 운영지원팀 배정",
        ] {
            assert!(person.contains(text), "person lacks {text}");
        }
        let employment = render_preview(&data, &Page::Employment { id: "e-101".into() });
        for text in [
            "정규직",
            "운영지원팀",
            "운영 담당",
            "2023-03-02",
            "2026-10-01",
            "인사팀",
            "인사 발령안",
        ] {
            assert!(employment.contains(text), "employment lacks {text}");
        }
        assert!(person.contains("href=\"/employment/e-101\""));
        assert!(employment.contains("href=\"/approvals/ap-2041\""));
        let approval = render_preview(
            &data,
            &Page::Approval {
                id: "ap-2041".into(),
            },
        );
        for text in [
            "소속 조직",
            "운영지원팀",
            "인사팀",
            "인사 발령안",
            "제안 2",
            "박서준",
            "이지원",
            "현재 급여는 변경하지 않습니다",
        ] {
            assert!(approval.contains(text), "decision lacks {text}");
        }
        assert!(
            approval.find("인사 발령안").unwrap() < approval.find("판단 결과 미리보기").unwrap(),
            "evidence must precede decision"
        );
        let pay = render_preview(&data, &Page::Payroll);
        for text in [
            "계산안 3",
            "부서 이동 발효일 확인",
            "3,200,000",
            "320,000",
            "2,880,000",
            "지급 지시",
            "대사",
        ] {
            assert!(pay.contains(text), "payroll lacks {text}");
        }
        assert!(pay.contains("href=\"/approvals/ap-2041\""));
    }

    #[test]
    fn employment_preview_preserves_context_inputs_and_distinguishes_comparison_from_commit() {
        let data = story();
        let page = Page::EmploymentChange {
            id: "e-101".into(),
            unit: "u-people".into(),
            job: "인사 운영 담당".into(),
            effective: "2026-10-01".into(),
            reason: "조직 개편 <script>alert(1)</script>".into(),
            compare: true,
        };
        let html = render_preview(&data, &page);
        for text in [
            "김민지",
            "운영지원팀",
            "인사팀",
            "인사 운영 담당",
            "2026-10-01",
            "변경 전",
            "변경 후",
            "실제 업무에 반영되지 않습니다",
            "입력으로 돌아가기",
        ] {
            assert!(html.contains(text), "comparison lacks {text}");
        }
        assert!(
            html.contains("&lt;script&gt;"),
            "entered reason must be escaped"
        );
        assert!(!html.contains("<script>"));
        assert!(
            !html.contains("저장 완료") && !html.contains("상신 완료"),
            "preview is not acknowledgement"
        );
        let form = render_preview(
            &data,
            &Page::EmploymentChange {
                id: "e-101".into(),
                unit: String::new(),
                job: String::new(),
                effective: String::new(),
                reason: String::new(),
                compare: false,
            },
        );
        for text in [
            "method=\"get\"",
            "name=\"unit\"",
            "name=\"job\"",
            "name=\"effective\"",
            "name=\"reason\"",
            "운영 담당",
            "변경 내용 비교",
        ] {
            assert!(form.contains(text), "contextual form lacks {text}");
        }
    }

    #[test]
    fn people_search_and_unknown_entities_have_truthful_results() {
        let data = story();
        let found = render_preview(
            &data,
            &Page::People {
                query: "김민지".into(),
            },
        );
        assert!(found.contains("href=\"/people/p-101\""));
        let missing = render_preview(
            &data,
            &Page::People {
                query: "없는이름".into(),
            },
        );
        assert!(missing.contains("검색 결과가 없습니다"));
        assert!(!missing.contains("href=\"/people/p-101\""));
        for page in [
            Page::Person {
                id: "absent".into(),
            },
            Page::Employment {
                id: "absent".into(),
            },
            Page::Approval {
                id: "absent".into(),
            },
            Page::Missing,
        ] {
            let html = render_preview(&data, &page);
            assert!(html.contains("찾을 수 없는 화면"));
            assert!(html.contains("data-workspace-preview=\"synthetic\""));
            assert!(!html.contains("minji@example.test"));
        }
    }
}
