// Domain lane increment lifecycle: specify → design → build → verify → review → handoff.
// Leader merges. Executors must not clear PRODUCT HOLDs or edit docs/current/**.
let meta = #{
    name: "domain-increment",
    description: "SDLC workflow for one backend domain lane increment (admission → design → build → verify → review)",
    phases: [
        #{ title: "Specify", detail: "admission record and acceptance criteria" },
        #{ title: "Design", detail: "architect CLEAR/WATCH/BLOCK" },
        #{ title: "Build", detail: "executor in allowlisted paths" },
        #{ title: "Verify", detail: "run verification commands" },
        #{ title: "Review", detail: "code + optional security review" },
        #{ title: "Handoff", detail: "PR body and candidate SHA for leader merge" },
    ],
};

let admission_schema = #{
    "type": "object",
    "required": ["objective", "non_goals", "allowed_paths", "forbidden_paths", "holds_checked", "verification_commands", "stop_conditions"],
    "properties": #{
        "objective": #{ "type": "string" },
        "non_goals": #{ "type": "string" },
        "allowed_paths": #{ "type": "array", "items": #{ "type": "string" } },
        "forbidden_paths": #{ "type": "array", "items": #{ "type": "string" } },
        "holds_checked": #{ "type": "string" },
        "verification_commands": #{ "type": "array", "items": #{ "type": "string" } },
        "stop_conditions": #{ "type": "string" },
    },
};

let design_schema = #{
    "type": "object",
    "required": ["verdict", "evidence", "blockers"],
    "properties": #{
        "verdict": #{ "type": "string" },
        "evidence": #{ "type": "string" },
        "blockers": #{ "type": "array", "items": #{ "type": "string" } },
    },
};

let handoff_schema = #{
    "type": "object",
    "required": ["summary", "pr_title", "pr_body", "candidate_notes"],
    "properties": #{
        "summary": #{ "type": "string" },
        "pr_title": #{ "type": "string" },
        "pr_body": #{ "type": "string" },
        "candidate_notes": #{ "type": "string" },
    },
};

if args == () { pause("verification", "Pass args: lane_id, issue, base_sha, allowlist_glob, risk_class, objective_hint"); }

let lane_id = args.lane_id;
let issue = args.issue;
let base_sha = args.base_sha;
let allowlist_glob = args.allowlist_glob;
let risk_class = args.risk_class;
let objective_hint = args.objective_hint;
if lane_id == () || issue == () || base_sha == () || allowlist_glob == () {
    pause("verification", "Required args: lane_id, issue, base_sha, allowlist_glob");
}
if risk_class == () { risk_class = "high"; }
if objective_hint == () { objective_hint = "Advance backend-only work under PRODUCT/ROADMAP HOLDs"; }

// --- Specify ---
phase("Specify");
let specify_prompt = "";
specify_prompt += "You are the specify stage for Console domain lane " + lane_id + " (issue " + issue + ").\n";
specify_prompt += "Base SHA (immutable): " + base_sha + ".\n";
specify_prompt += "Allowlist glob: " + allowlist_glob + ".\n";
specify_prompt += "Objective hint: " + objective_hint + ".\n";
specify_prompt += "Read docs/current/PRODUCT.md, ROADMAP.md, DELIVERY.md with read_file/grep.\n";
specify_prompt += "Produce a strict admission record. HOLDs must be listed; none may be cleared by implication.\n";
specify_prompt += "Forbidden always includes: docs/current/**, production exposure, frontend/Leptos unless HOLD cleared, migrations unless this increment owns the migration lease.\n";
specify_prompt += "Do not implement code. Return only the schema fields.\n";
let spec = agent(specify_prompt, #{
    label: "specify:" + lane_id,
    capability_mode: "read-only",
    output_schema: admission_schema,
});
if spec == () || !spec.success || spec.output == () {
    complete(#{ ok: false, stage: "specify", error: "specify agent failed" });
}

// --- Design ---
phase("Design");
let design_prompt = "";
design_prompt += "Architect review for lane " + lane_id + ".\n";
design_prompt += "Admission objective: " + spec.output.objective + "\n";
design_prompt += "Non-goals: " + spec.output.non_goals + "\n";
design_prompt += "HOLDs: " + spec.output.holds_checked + "\n";
design_prompt += "Allowed paths: " + json_encode(spec.output.allowed_paths) + "\n";
design_prompt += "Read the allowlisted code only. verdict must be CLEAR, WATCH, or BLOCK.\n";
design_prompt += "BLOCK if the work would clear a PRODUCT HOLD, open a second write path for payroll, or ship UI under frontend HOLD.\n";
let design = agent(design_prompt, #{
    label: "design:" + lane_id,
    capability_mode: "read-only",
    agent_type: "oh-my-claudecode:architect",
    output_schema: design_schema,
});
if design == () || !design.success || design.output == () {
    complete(#{ ok: false, stage: "design", error: "design agent failed", admission: spec.output });
}
if design.output.verdict == "BLOCK" {
    complete(#{
        ok: false,
        stage: "design",
        verdict: "BLOCK",
        blockers: design.output.blockers,
        admission: spec.output,
    });
}

// --- Build ---
phase("Build");
let build_prompt = "";
build_prompt += "Implement the admission for lane " + lane_id + " issue " + issue + ".\n";
build_prompt += "Objective: " + spec.output.objective + "\n";
build_prompt += "Non-goals: " + spec.output.non_goals + "\n";
build_prompt += "ONLY modify paths matching allowlist: " + allowlist_glob + "\n";
build_prompt += "Allowed path list: " + json_encode(spec.output.allowed_paths) + "\n";
build_prompt += "Forbidden: " + json_encode(spec.output.forbidden_paths) + "\n";
build_prompt += "Base SHA " + base_sha + ". Work in the current workspace; keep commits focused.\n";
build_prompt += "Do not edit docs/current/**, .gjc/**, or parent dirty deploy files.\n";
build_prompt += "Run verification commands when done: " + json_encode(spec.output.verification_commands) + "\n";
build_prompt += "Return a short summary of files changed and test results as plain text.\n";
let build = agent(build_prompt, #{
    label: "build:" + lane_id,
    capability_mode: "execute",
    isolation_worktree: true,
});
if build == () || !build.success {
    complete(#{ ok: false, stage: "build", error: "build agent failed", admission: spec.output, design: design.output });
}

// --- Verify ---
phase("Verify");
let verify_prompt = "";
verify_prompt += "Verify the build for lane " + lane_id + " against the admission.\n";
verify_prompt += "Run these commands and report discovered vs executed counts where applicable:\n";
verify_prompt += json_encode(spec.output.verification_commands) + "\n";
verify_prompt += "Fail closed if any command fails or if tests were skipped/deleted without receipt.\n";
verify_prompt += "Return plain text evidence only.\n";
let verify = agent(verify_prompt, #{
    label: "verify:" + lane_id,
    capability_mode: "execute",
    agent_type: "agent-skills:test-engineer",
});
if verify == () || !verify.success {
    complete(#{ ok: false, stage: "verify", error: "verify agent failed", admission: spec.output });
}

// --- Review ---
phase("Review");
let review_jobs = [];
review_jobs.push(#{
    prompt: "Code review the current changes for lane " + lane_id + ". Focus on correctness, allowlist violations, HOLD breaches, and missing tests. Read the diff with tools.",
    label: "review-code:" + lane_id,
    capability_mode: "read-only",
    agent_type: "agent-skills:code-reviewer",
});
if risk_class == "high" {
    review_jobs.push(#{
        prompt: "Security review for lane " + lane_id + " focusing on authz fail-closed, tenant isolation, PII logging, and injection. Read the diff with tools.",
        label: "review-sec:" + lane_id,
        capability_mode: "read-only",
        agent_type: "agent-skills:security-auditor",
    });
}
let reviews = parallel(review_jobs);

// --- Handoff ---
phase("Handoff");
let handoff_prompt = "";
handoff_prompt += "Assemble a GitHub PR title and body for lane " + lane_id + " issue " + issue + ".\n";
handoff_prompt += "Admission objective: " + spec.output.objective + "\n";
handoff_prompt += "Design verdict: " + design.output.verdict + "\n";
handoff_prompt += "Build notes: " + json_encode(build.output) + "\n";
handoff_prompt += "Verify notes: " + json_encode(verify.output) + "\n";
handoff_prompt += "Include Closes " + issue + " if appropriate. State HOLDs not cleared. Leader will merge.\n";
let handoff = agent(handoff_prompt, #{
    label: "handoff:" + lane_id,
    capability_mode: "read-only",
    output_schema: handoff_schema,
});

complete(#{
    ok: true,
    lane_id: lane_id,
    issue: issue,
    base_sha: base_sha,
    admission: spec.output,
    design: design.output,
    build: build.output,
    verify: verify.output,
    reviews: reviews,
    handoff: if handoff != () && handoff.success { handoff.output } else { () },
    note: "Leader owns git push, gh pr create/merge, and beads updates.",
});
