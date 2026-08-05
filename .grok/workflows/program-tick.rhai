// Program control-plane tick: report open PR merge readiness and next actions.
// Does not merge automatically (leader confirmation). Does not clear HOLDs.
let meta = #{
    name: "program-tick",
    description: "Read-only program heartbeat: open PRs, ultragoal status hints, lease reminders",
    phases: [
        #{ title: "Inspect", detail: "PR and issue state" },
        #{ title: "Report", detail: "next actions without auto-merge" },
    ],
};

phase("Inspect");
let inspect = agent(
    "Inspect the Console program control plane. Run: gh pr list --state open --limit 20; "
        + "gh pr list --state open --json number,title,mergeStateStatus,headRefName --jq .; "
        + "for key PRs 574 and 575 if open, run gh pr checks N. "
        + "Summarize which PRs are CLEAN vs BLOCKED and why. "
        + "Remind: migrations/CI/lockfile leases are single-writer; PRODUCT frontend and projection HOLDs remain. "
        + "Do not merge anything. Do not edit files.",
    #{
        label: "inspect-prs",
        capability_mode: "execute",
    },
);

phase("Report");
let report = agent(
    "Using the prior inspection output, produce a short operator report: "
        + "(1) ready to merge, (2) waiting on CI, (3) blocked, (4) suggested next lane dispatch under Wave 0 "
        + "only if substrate PRs are not hogging CI/lockfile writers. Plain text.",
    #{
        label: "operator-report",
        capability_mode: "read-only",
    },
);

complete(#{
    ok: true,
    inspect: if inspect != () && inspect.success { inspect.output } else { () },
    report: if report != () && report.success { report.output } else { () },
});
