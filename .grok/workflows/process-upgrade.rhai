// Process-upgrade — Bun "edit the process" for one failure class.
// Allowlist: workflows, harness, tools/ci, check scripts — NOT product crates.
// After 2 repeats of a class, this workflow should land a permanent control.
// Doctrine: .grok/programs/BUN-PARALLEL-DISCIPLINE.md + learning-loop.v1.json

let meta = #{
    name: "process-upgrade",
    description: "Map a failure class to a durable workflow/tool control (not a one-shot PR fix)",
    when_to_use: "Hosted red mapped to failure-classes.v1.json; or repeat class >= 2; never for product features",
    phases: [
        #{ title: "Map", detail: "class id + evidence URL + missing control" },
        #{ title: "Design", detail: "permanent control in admit/workflow/hook class" },
        #{ title: "Implement", detail: "edit process allowlist only" },
        #{ title: "Verify", detail: "detector fails open bug / passes fixed" },
        #{ title: "Handoff", detail: "PR body for process-only change" },
    ],
};

let map_schema = #{
    "type": "object",
    "required": ["class_id", "evidence", "missing_control", "proposed_control", "one_shot_rejected"],
    "properties": #{
        "class_id": #{ "type": "string" },
        "evidence": #{ "type": "string" },
        "missing_control": #{ "type": "string" },
        "proposed_control": #{ "type": "string" },
        "one_shot_rejected": #{ "type": "string" },
    },
};

let class_id = "";
let evidence = "";
if args != () {
    if args.class_id != () { class_id = args.class_id; }
    if args.evidence != () { evidence = args.evidence; }
}
if class_id == () {
    pause("verification", "Pass args.class_id (from failure-classes.v1.json) and optional args.evidence (CI URL)");
}

phase("Map");
let map_prompt = "";
map_prompt += "Process-upgrade MAP for Console failure class: " + class_id + "\n";
map_prompt += "Evidence: " + evidence + "\n";
map_prompt += "Read .grok/harness/failure-classes.v1.json and .grok/programs/BUN-PARALLEL-DISCIPLINE.md with read_file.\n";
map_prompt += "State what permanent control is missing so agents cannot repeat this class.\n";
map_prompt += "one_shot_rejected: describe the forbidden one-shot fix (e.g. only fixing one PR lens JSON without admit gate).\n";
map_prompt += "proposed_control must be reusable (workflow phase, npm admit command, class-based hook matcher, harness catalog row).\n";
let mapped = agent(map_prompt, #{
    label: "proc-map",
    capability_mode: "read-only",
    output_schema: map_schema,
});
if mapped == () || !mapped.success || mapped.output == () {
    complete(#{ ok: false, stage: "map", error: "map failed" });
}

phase("Design");
let design = agent(
    "Design the minimal permanent control for class " + class_id + ".\n"
        + "Map result: " + json_encode(mapped.output) + "\n"
        + "Allowlist only: .grok/workflows/**, .grok/harness/**, .grok/programs/**, tools/ci/**, scripts/check-*.mjs, scripts/console/**, package.json.\n"
        + "Forbidden: docs/current/**, backend/crates/**, deploy/**, product features.\n"
        + "Return plain text: files to touch, detector command, how domain-increment or ci-fleet-tick will call it.\n",
    #{
        label: "proc-design",
        capability_mode: "read-only",
    },
);
if design == () || !design.success {
    complete(#{ ok: false, stage: "design", error: "design failed", map: mapped.output });
}

phase("Implement");
let impl = agent(
    "Implement the process control for class " + class_id + ".\n"
        + "Design:\n" + json_encode(design.output) + "\n"
        + "Map:\n" + json_encode(mapped.output) + "\n"
        + "ONLY edit process allowlist paths. Update failure-classes.v1.json if detector/control text changes.\n"
        + "If adding a hook, it must be class-based (path/tool matcher), never named for a single PR.\n"
        + "Add or update a tip under .grok/memory/tips/ if useful.\n"
        + "Run detector commands to show fail/pass behavior when possible.\n",
    #{
        label: "proc-impl",
        capability_mode: "execute",
        isolation_worktree: true,
    },
);
if impl == () || !impl.success {
    complete(#{ ok: false, stage: "implement", error: "implement failed", map: mapped.output });
}

phase("Verify");
let ver = agent(
    "Verify process-upgrade for " + class_id + ".\n"
        + "Confirm controls are permanent (workflow/harness/tool), not a one-off content fix.\n"
        + "Run relevant admit_commands from failure-classes.v1.json.\n"
        + "Fail if only product files changed or docs/current edited.\n"
        + "Plain text evidence.\n",
    #{
        label: "proc-verify",
        capability_mode: "execute",
        agent_type: "agent-skills:test-engineer",
    },
);

phase("Handoff");
let handoff = agent(
    "Write PR title and body for process-upgrade class " + class_id + ".\n"
        + "State: process-only; Bun fix-the-process; links evidence " + evidence + ".\n"
        + "Leader merges. Include files changed summary from implement.\n"
        + "JSON fields: summary, pr_title, pr_body, candidate_notes.\n",
    #{
        label: "proc-handoff",
        capability_mode: "read-only",
        output_schema: #{
            "type": "object",
            "required": ["summary", "pr_title", "pr_body", "candidate_notes"],
            "properties": #{
                "summary": #{ "type": "string" },
                "pr_title": #{ "type": "string" },
                "pr_body": #{ "type": "string" },
                "candidate_notes": #{ "type": "string" },
            },
        },
    },
);

complete(#{
    ok: true,
    class_id: class_id,
    map: mapped.output,
    design: design.output,
    implement: impl.output,
    verify: if ver != () && ver.success { ver.output } else { () },
    handoff: if handoff != () && handoff.success { handoff.output } else { () },
    note: "Leader owns push/PR/merge. Pair fleet waits with product-process-tick, not more WAIT.",
});
