// ultragoal — durable multi-story plan + native /goal handoff + activate hook loop.
// Ideas from OMC/GJC ultragoal. Grok-native: .grok/ultragoal/* + /goal + Stop hooks.
// Forbidden: omc/omx/gjc/hermes CLIs.

let meta = #{
    name: "ultragoal",
    description: "Create/activate durable ultragoal plan, print /goal handoff, arm Stop-hook loop into program-control/ralph",
    when_to_use: "Large multi-session initiative; pair with ralplan then ralph; keep soft reds on board",
    phases: [
        #{ title: "Brief", detail: "load brief / args objective" },
        #{ title: "Plan", detail: "stories in goals.json + optional ralplan gate" },
        #{ title: "Activate", detail: "active-goal.live.json + ledger + handoff" },
        #{ title: "Dispatch", detail: "next workflow recommendation" },
    ],
};

let out_schema = #{
    "type": "object",
    "required": ["ok", "action", "objective", "handoff", "next_workflow", "native_goal_command"],
    "properties": #{
        "ok": #{ "type": "boolean" },
        "action": #{ "type": "string" },
        "objective": #{ "type": "string" },
        "handoff": #{ "type": "string" },
        "next_workflow": #{ "type": "string" },
        "native_goal_command": #{ "type": "string" },
        "story_ids": #{ "type": "array", "items": #{ "type": "string" } },
        "ralplan_required": #{ "type": "boolean" },
    },
};

let action = "status";
if args != () && args.action != () {
    action = args.action;
}
let objective = "";
if args != () && args.objective != () {
    objective = args.objective;
}
let brief_path = ".grok/ultragoal/brief.md";
if args != () && args.brief_path != () {
    brief_path = args.brief_path;
}
let dispatch_wf = "program-control";
if args != () && args.workflow != () {
    dispatch_wf = args.workflow;
}

phase("Brief");
let brief_prompt = "";
brief_prompt += "Ultragoal Brief phase. action=";
brief_prompt += action;
brief_prompt += "\nRead .grok/harness/ultragoal.v1.json and .grok/ultragoal/*.\n";
brief_prompt += "If action is status: run `.grok/bin/console-goal status` and report.\n";
brief_prompt += "If action is create-goals or activate: read brief at ";
brief_prompt += brief_path;
brief_prompt += " and optional objective arg: ";
brief_prompt += objective;
brief_prompt += "\nRead PRODUCT HOLDs. Do not clear HOLDs.\n";
brief_prompt += "Return plain text: objective candidate + whether ralplan is required (vague large scope).\n";
let brief = agent(brief_prompt, #{
    label: "ug-brief",
    capability_mode: "execute",
});
if brief == () || !brief.success {
    complete(#{ ok: false, stage: "brief", error: "brief failed" });
}

phase("Plan");
let plan_prompt = "";
plan_prompt += "Plan ultragoal stories into .grok/ultragoal/goals.json if action is create-goals or activate.\n";
plan_prompt += "Brief agent output:\n";
plan_prompt += json_encode(brief.output);
plan_prompt += "\nWrite goals as ordered list with id G00x-slug, title, objective, status pending/in_progress.\n";
plan_prompt += "If scope is vague, set ralplan_required=true and do NOT start ralph yet.\n";
plan_prompt += "Append ledger events via `.grok/bin/console-goal ledger-append`.\n";
plan_prompt += "If action is status only, skip writes.\n";
let planned = agent(plan_prompt, #{
    label: "ug-plan",
    capability_mode: "all",
});
if planned == () || !planned.success {
    complete(#{ ok: false, stage: "plan", error: "plan failed", brief: brief.output });
}

phase("Activate");
let act_prompt = "";
act_prompt += "Activate ultragoal for hook loop if action is activate or create-goals.\n";
act_prompt += "Run: `.grok/bin/console-goal activate --objective '…' --workflow ";
act_prompt += dispatch_wf;
act_prompt += "` (add --ralplan-required if needed).\n";
act_prompt += "Then `.grok/bin/console-goal handoff`.\n";
act_prompt += "Seed prd.live.json stories from first executable story if not ralplan-gated (task-specific acceptanceCriteria).\n";
act_prompt += "Plan output:\n";
act_prompt += json_encode(planned.output);
act_prompt += "\nIf action is status: only handoff if already active.\n";
let activated = agent(act_prompt, #{
    label: "ug-activate",
    capability_mode: "all",
    output_schema: out_schema,
});
if activated == () || !activated.success {
    complete(#{ ok: false, stage: "activate", error: "activate failed" });
}

phase("Dispatch");
let disp = "";
disp += "Synthesize operator next steps. Include exact strings:\n";
disp += "- native_goal_command like `/goal …`\n";
disp += "- next_workflow like `/workflow ralplan` or `/workflow ralph` or `/workflow program-control`\n";
disp += "Remind: Stop hook blocks while active-goal.live.json.active; Hermes learn after ticks.\n";
disp += "Activate output:\n";
disp += json_encode(activated.output);
let final = agent(disp, #{
    label: "ug-dispatch",
    capability_mode: "execute",
});

complete(#{
    ok: true,
    workflow: "ultragoal",
    result: activated.output,
    dispatch: if final != () && final.success { final.output } else { () },
});
