// ralplan — consensus planning before heavy execution (Planner → Architect → Critic).
// Ideas from OMC ralplan. Grok-native Rhai; plan pending approval unless args.execute_after_approve.
// Does not mutate product code unless args.execute_after_approve true AND critic APPROVE.

let meta = #{
    name: "ralplan",
    description: "Consensus plan: Planner + Architect + Critic until APPROVE; gate vague ralph/ultragoal execution",
    when_to_use: "Vague large tasks before ralph/program-control fan-out; high-risk use args.deliberate",
    phases: [
        #{ title: "Planner", detail: "principles, drivers, options, plan draft" },
        #{ title: "Architect", detail: "steelman + tradeoffs" },
        #{ title: "Critic", detail: "APPROVE|ITERATE|REJECT loop" },
        #{ title: "Finalize", detail: "write plan artifact; optional handoff to ralph" },
    ],
};

let task = "";
if args != () && args.task != () {
    task = args.task;
}
if task == "" && args != () && args.objective != () {
    task = args.objective;
}
if task == "" {
    pause("verification", "Pass args.task or args.objective for ralplan.");
}

let deliberate = false;
if args != () && args.deliberate == true {
    deliberate = true;
}
let execute_after = false;
if args != () && args.execute_after_approve == true {
    execute_after = true;
}

let plan_schema = #{
    "type": "object",
    "required": ["principles", "decision_drivers", "options", "plan_summary", "acceptance_criteria"],
    "properties": #{
        "principles": #{ "type": "array", "items": #{ "type": "string" } },
        "decision_drivers": #{ "type": "array", "items": #{ "type": "string" } },
        "options": #{ "type": "array", "items": #{ "type": "string" } },
        "plan_summary": #{ "type": "string" },
        "acceptance_criteria": #{ "type": "array", "items": #{ "type": "string" } },
        "premortem": #{ "type": "array", "items": #{ "type": "string" } },
        "allowlist": #{ "type": "array", "items": #{ "type": "string" } },
        "holds_checked": #{ "type": "string" },
    },
};

let critic_schema = #{
    "type": "object",
    "required": ["verdict", "reasons", "must_fix"],
    "properties": #{
        "verdict": #{ "type": "string" },
        "reasons": #{ "type": "string" },
        "must_fix": #{ "type": "array", "items": #{ "type": "string" } },
    },
};

phase("Planner");
let p_prompt = "";
p_prompt += "You are RALPLAN Planner for Console monorepo.\n";
p_prompt += "Task:\n";
p_prompt += task;
p_prompt += "\nRead docs/current/PRODUCT.md HOLDs and ROADMAP.md. Read .grok/programs/BUN-PARALLEL-DISCIPLINE.md.\n";
p_prompt += "Produce RALPLAN-DR: principles (3-5), top 3 decision drivers, >=2 viable options with pros/cons,\n";
p_prompt += "plan_summary, testable acceptance_criteria, allowlist paths, holds_checked.\n";
if deliberate {
    p_prompt += "DELIBERATE: include 3 pre-mortem scenarios and unit/integration/e2e/observability test plan notes in plan_summary.\n";
}
p_prompt += "Planning only — no product file mutations.\n";
let plan = agent(p_prompt, #{
    label: "ral-planner",
    capability_mode: "execute",
    output_schema: plan_schema,
});
if plan == () || !plan.success {
    complete(#{ ok: false, stage: "planner", error: "planner failed" });
}

phase("Architect");
let a_prompt = "";
a_prompt += "You are RALPLAN Architect. Steelman antithesis, at least one real tradeoff, synthesis.\n";
a_prompt += "Plan:\n";
a_prompt += json_encode(plan.output);
a_prompt += "\nFlag HOLD or tip-serial thrash risks. No product mutations.\n";
let arch = agent(a_prompt, #{
    label: "ral-architect",
    capability_mode: "execute",
});
if arch == () || !arch.success {
    complete(#{ ok: false, stage: "architect", error: "architect failed", plan: plan.output });
}

// Critic loop (max 3 in-workflow iterations; agents revise)
let verdict = "ITERATE";
let critic_out = ();
let plan_out = plan.output;
let i = 0;
while i < 3 {
    phase("Critic");
    let c_prompt = "";
    c_prompt += "You are RALPLAN Critic. verdict MUST be APPROVE | ITERATE | REJECT.\n";
    c_prompt += "Enforce principle-option consistency, fair alternatives, testable acceptance criteria,\n";
    c_prompt += "concrete verification, soft-red enqueue rule, no PRODUCT HOLD clearance.\n";
    c_prompt += "Plan:\n";
    c_prompt += json_encode(plan_out);
    c_prompt += "\nArchitect:\n";
    c_prompt += json_encode(arch.output);
    if deliberate {
        c_prompt += "\nDeliberate mode: reject weak pre-mortem or missing expanded test plan.\n";
    }
    critic_out = agent(c_prompt, #{
        label: "ral-critic-" + i.to_string(),
        capability_mode: "execute",
        output_schema: critic_schema,
    });
    if critic_out == () || !critic_out.success {
        complete(#{ ok: false, stage: "critic", error: "critic failed", iteration: i });
    }
    verdict = critic_out.output.verdict;
    if verdict == "APPROVE" {
        break;
    }
    // revise
    let r_prompt = "";
    r_prompt += "Revise plan per Critic must_fix. Keep HOLDs fail-closed.\n";
    r_prompt += json_encode(critic_out.output);
    r_prompt += "\nPrior plan:\n";
    r_prompt += json_encode(plan_out);
    let revised = agent(r_prompt, #{
        label: "ral-revise-" + i.to_string(),
        capability_mode: "execute",
        output_schema: plan_schema,
    });
    if revised != () && revised.success {
        plan_out = revised.output;
    }
    i += 1;
}

phase("Finalize");
let f_prompt = "";
f_prompt += "Write plan artifact to .grok/ultragoal/plans/latest-ralplan.md (create dirs).\n";
f_prompt += "Mark status pending approval unless execute_after_approve.\n";
f_prompt += "Final plan:\n";
f_prompt += json_encode(plan_out);
f_prompt += "\nCritic:\n";
f_prompt += json_encode(critic_out.output);
f_prompt += "\nIf verdict APPROVE and execute_after: update prd.live.json stories from acceptance_criteria ";
f_prompt += "(task-specific), activate ralph via console-goal activate --workflow ralph.\n";
f_prompt += "Else print handoff: /workflow ralph only after user or autonomy approve.\n";
f_prompt += "execute_after_approve=";
f_prompt += if execute_after { "true" } else { "false" };
f_prompt += " verdict=";
f_prompt += verdict;
let fin = agent(f_prompt, #{
    label: "ral-finalize",
    capability_mode: "all",
});

complete(#{
    ok: verdict == "APPROVE",
    workflow: "ralplan",
    verdict: verdict,
    plan: plan_out,
    critic: if critic_out != () && critic_out.success { critic_out.output } else { () },
    finalize: if fin != () && fin.success { fin.output } else { () },
    next: if verdict == "APPROVE" { "/workflow ralph or /workflow program-control" } else { "revise plan / re-run ralplan" },
});
