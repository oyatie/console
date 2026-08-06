// Product + process tick — other half of the anti-wait pair.
// Run when ci-fleet-tick reports waiting_ci OR whenever fleet is idle/CLEAN-only.
// Does product/backend progress OR process-upgrade work — never pure CI narration.
// Leader-only merge. PRODUCT HOLDs fail-closed.
// Doctrine: .grok/programs/BUN-PARALLEL-DISCIPLINE.md

let meta = #{
    name: "product-process-tick",
    description: "Product/process half of dual-track drive: claim ready work while CI runs — forbids pure WAIT",
    when_to_use: "Always pair with ci-fleet-tick when any PR is waiting_ci; also when fleet is empty to advance roadmap",
    phases: [
        #{ title: "Context", detail: "authority + beads ready + open leases" },
        #{ title: "Select", detail: "product increment OR process-upgrade class" },
        #{ title: "Dispatch", detail: "concrete next command or admission sketch" },
    ],
};

let select_schema = #{
    "type": "object",
    "required": [
        "track",
        "choice_id",
        "rationale",
        "allowlist",
        "forbidden",
        "holds_checked",
        "next_commands",
        "not_waiting_on_ci",
    ],
    "properties": #{
        "track": #{ "type": "string" },
        "choice_id": #{ "type": "string" },
        "rationale": #{ "type": "string" },
        "allowlist": #{ "type": "array", "items": #{ "type": "string" } },
        "forbidden": #{ "type": "array", "items": #{ "type": "string" } },
        "holds_checked": #{ "type": "string" },
        "next_commands": #{ "type": "array", "items": #{ "type": "string" } },
        "not_waiting_on_ci": #{ "type": "boolean" },
        "failure_class_if_process": #{ "type": "string" },
    },
};

// Optional: fleet summary from paired ci-fleet-tick
let fleet_context = "";
if args != () && args.fleet_json != () {
    fleet_context = args.fleet_json;
}
let prefer = "";
if args != () && args.prefer_track != () {
    prefer = args.prefer_track;
}

phase("Context");
let ctx_prompt = "";
ctx_prompt += "Console dual-track PRODUCT/PROCESS context (execute tools).\n";
ctx_prompt += "1) Read docs/current/PRODUCT.md HOLDs and ROADMAP.md ordered work (read_file).\n";
ctx_prompt += "2) Run: bd ready 2>/dev/null | head -40 ; bd list --status=in_progress 2>/dev/null | head -20.\n";
ctx_prompt += "3) Run: gh pr list --state open --limit 15 --json number,title,headRefName.\n";
ctx_prompt += "4) Note writers that must stay serial: ci.yml, authority tip, migrations, Cargo.lock, docs/current/**.\n";
if fleet_context != "" {
    ctx_prompt += "5) Paired fleet classification JSON:\n";
    ctx_prompt += fleet_context;
    ctx_prompt += "\n";
}
ctx_prompt += "Do not merge. Do not implement product code in this phase. Plain text inventory only.\n";
let ctx = agent(ctx_prompt, #{
    label: "pp-context",
    capability_mode: "execute",
});
if ctx == () || !ctx.success {
    complete(#{ ok: false, stage: "context", error: "context failed" });
}

phase("Select");
let sel_prompt = "";
sel_prompt += "Select exactly ONE next unit of work that is NOT 'wait for CI'.\n\n";
sel_prompt += "Context inventory:\n";
sel_prompt += json_encode(ctx.output);
sel_prompt += "\n\nPrefer track hint (may be empty): ";
sel_prompt += prefer;
sel_prompt += "\n\nTracks (pick one):\n";
sel_prompt += "- product: path-disjoint backend lane under PRODUCT (ontology/policy/approval foundations, or next ROADMAP backend item not HOLD-blocked). Use domain-increment next.\n";
sel_prompt += "- process: failure-class control upgrade (workflows/harness/tools/ci admit). Use process-upgrade next. Prefer if fleet shows repeated hosted reds fixable locally (lens, doc-manifest, residual buck, skip-admit).\n";
sel_prompt += "- substrate: DN-0005 / cargo-nextest / residual cutover when CI tip free; not while second ci.yml writer fights an open CI PR.\n\n";
sel_prompt += "Rules:\n";
sel_prompt += "- not_waiting_on_ci MUST be true. Choosing wait is forbidden (ops.passive-wait).\n";
sel_prompt += "- holds_checked must list PRODUCT HOLDs and confirm none cleared by this choice.\n";
sel_prompt += "- forbidden always includes docs/current/** edits unless track=docs with explicit docs lane (default forbid).\n";
sel_prompt += "- allowlist must be concrete globs.\n";
sel_prompt += "- next_commands: 1-5 shell/workflow invocations (e.g. /workflow process-upgrade, bd update, worktree add).\n";
sel_prompt += "- If fleet waiting_ci non-empty, still pick product or process work that does not require that PR's tip writer.\n";
sel_prompt += "- Read .grok/harness/failure-classes.v1.json if process track.\n";
let selected = agent(sel_prompt, #{
    label: "pp-select",
    capability_mode: "read-only",
    output_schema: select_schema,
});
if selected == () || !selected.success || selected.output == () {
    complete(#{ ok: false, stage: "select", error: "select failed", context: ctx.output });
}
if selected.output.not_waiting_on_ci != true {
    complete(#{
        ok: false,
        stage: "select",
        error: "ops.passive-wait: selection resolved to waiting on CI",
        selection: selected.output,
    });
}

phase("Dispatch");
let disp_prompt = "";
disp_prompt += "Turn this selection into a concrete dispatch brief for the operator/agent.\n";
disp_prompt += json_encode(selected.output);
disp_prompt += "\n\nInclude:\n";
disp_prompt += "1) Which workflow to run next: domain-increment | process-upgrade | manual admit steps.\n";
disp_prompt += "2) Worktree base: origin/main fetch SHA.\n";
disp_prompt += "3) Success criteria (tests/commands).\n";
disp_prompt += "4) Explicit: do not babysit CI as the only action; ci-fleet-tick owns fleet.\n";
disp_prompt += "Plain text.\n";
let dispatch = agent(disp_prompt, #{
    label: "pp-dispatch",
    capability_mode: "read-only",
});

complete(#{
    ok: true,
    pair_workflow: "ci-fleet-tick",
    selection: selected.output,
    dispatch: if dispatch != () && dispatch.success { dispatch.output } else { () },
    context: ctx.output,
    doctrine: "While CI runs, this tick must advance product or process — never pure WAIT",
});
