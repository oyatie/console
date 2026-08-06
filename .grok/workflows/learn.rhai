// learn — Hermes-inspired closed learning tick (no hermes CLI).
// Maps failures → reflection/tip/skill draft → process-upgrade recommendation.

let meta = #{
    name: "learn",
    description: "Closed learning loop: reflections, tips, skill drafts, process-upgrade prompts from failure classes",
    when_to_use: "After program-control, ralph, or any hard fail; promotes process edits not chat folklore",
    phases: [
        #{ title: "Collect", detail: "open soft reds, recent fails, board" },
        #{ title: "Persist", detail: "console-learn + memory" },
        #{ title: "Promote", detail: "process-upgrade if class repeats" },
    ],
};

let out_schema = #{
    "type": "object",
    "required": ["ok", "classes", "wrote", "promote_process_upgrade", "summary"],
    "properties": #{
        "ok": #{ "type": "boolean" },
        "classes": #{ "type": "array", "items": #{ "type": "string" } },
        "wrote": #{ "type": "array", "items": #{ "type": "string" } },
        "promote_process_upgrade": #{ "type": "boolean" },
        "summary": #{ "type": "string" },
    },
};

phase("Collect");
let c_prompt = "";
c_prompt += "Collect learning signals. Read lane-board.live.json, failure-classes.v1.json, hermes-learning.v1.json,\n";
c_prompt += "memory/reflections (today), ultragoal ledger tail if present.\n";
c_prompt += "List failure classes observed (soft reds count as ops.soft-red-silence if unowned).\n";
c_prompt += "gh pr list open with fails → classes.\n";
if args != () && args.classes != () {
    c_prompt += "Forced classes: ";
    c_prompt += args.classes;
    c_prompt += "\n";
}
if args != () && args.summary != () {
    c_prompt += "Summary: ";
    c_prompt += args.summary;
    c_prompt += "\n";
}
let collected = agent(c_prompt, #{
    label: "learn-collect",
    capability_mode: "execute",
});
if collected == () || !collected.success {
    complete(#{ ok: false, stage: "collect", error: "collect failed" });
}

phase("Persist");
let p_prompt = "";
p_prompt += "Run console-learn and ensure MEMORY/tips/trajectories updated.\n";
p_prompt += "Collected:\n";
p_prompt += json_encode(collected.output);
p_prompt += "\nExample: `.grok/bin/console-learn from-event --id learn-tick --summary '…' --classes 'a,b'`\n";
p_prompt += "If hard or repeated class, --promote-skill.\n";
let persisted = agent(p_prompt, #{
    label: "learn-persist",
    capability_mode: "all",
    output_schema: out_schema,
});
if persisted == () || !persisted.success {
    complete(#{ ok: false, stage: "persist", error: "persist failed" });
}

phase("Promote");
let pr_prompt = "";
pr_prompt += "If promote_process_upgrade or class repeats >=2, recommend `/workflow process-upgrade` with class_id.\n";
pr_prompt += "Do not expand product scope. Soft reds must remain owned on board.\n";
pr_prompt += json_encode(persisted.output);
let promoted = agent(pr_prompt, #{
    label: "learn-promote",
    capability_mode: "execute",
});

complete(#{
    ok: true,
    workflow: "learn",
    result: persisted.output,
    promote: if promoted != () && promoted.success { promoted.output } else { () },
});
