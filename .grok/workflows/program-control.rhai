// program-control — meta productivity manager for the autonomous Console drive.
// Ensures work-manager, implement-lane, and pr-babysit stay productive.
// Soft reds/blocks must be on the queue. Anti-passive. Full autonomy under human supervision.

let meta = #{
    name: "program-control",
    description: "Meta-orchestrator: keep work-manager, implement-lane, pr-babysit productive; soft reds never silent",
    when_to_use: "Primary autonomous heartbeat; session start; when drive may be idle",
    phases: [
        #{ title: "Pulse", detail: "board + open PRs + beads health" },
        #{ title: "WorkManager", detail: "refresh lanes + enqueue soft reds/blocks" },
        #{ title: "Babysit", detail: "repair/review/approve/merge" },
        #{ title: "Implement", detail: "claim ready non-tip-thrash work if capacity" },
        #{ title: "Productivity", detail: "verify no idle/silent soft reds; next wake" },
    ],
};

let pulse_schema = #{
    "type": "object",
    "required": ["open_pr_count", "board_item_count", "soft_red_count", "block_count", "idle_risk", "notes"],
    "properties": #{
        "open_pr_count": #{ "type": "number" },
        "board_item_count": #{ "type": "number" },
        "soft_red_count": #{ "type": "number" },
        "block_count": #{ "type": "number" },
        "idle_risk": #{ "type": "boolean" },
        "notes": #{ "type": "string" },
    },
};

let productivity_schema = #{
    "type": "object",
    "required": [
        "productive",
        "silence_check_ok",
        "workflows_armed",
        "next_wake_actions",
        "summary",
    ],
    "properties": #{
        "productive": #{ "type": "boolean" },
        "silence_check_ok": #{ "type": "boolean" },
        "workflows_armed": #{ "type": "array", "items": #{ "type": "string" } },
        "next_wake_actions": #{ "type": "array", "items": #{ "type": "string" } },
        "summary": #{ "type": "string" },
        "defects": #{ "type": "array", "items": #{ "type": "string" } },
    },
};

let skip_implement = false;
if args != () && args.skip_implement == true {
    skip_implement = true;
}
let skip_babysit = false;
if args != () && args.skip_babysit == true {
    skip_babysit = true;
}

phase("Pulse");
let pulse = agent(
    "Console program-control pulse. Use tools.\n"
        + "1) gh pr list --state open --limit 30 --json number,title,mergeStateStatus\n"
        + "2) Read .grok/harness/lane-board.live.json — count items by kind/status\n"
        + "3) bd ready | head -20\n"
        + "4) Note any open PR that looks red/behind/dirty even if board empty (idle_risk true if board empty but open PR issues exist)\n"
        + "5) Read autonomy-merge.v1.json one-liner policy\n"
        + "6) Read .grok/ultragoal/active-goal.live.json — if active, note objective + dispatch_workflow (Stop hook will loop)\n"
        + "7) If prd.live.json has stories with passes!=true, prefer ralph dispatch next\n"
        + "Return structured counts. soft_red_count/block_count from board + quick PR scan.\n",
    #{
        label: "ctrl-pulse",
        capability_mode: "execute",
        output_schema: pulse_schema,
    },
);
if pulse == () || !pulse.success {
    complete(#{ ok: false, stage: "pulse", error: "pulse failed" });
}

phase("WorkManager");
// Inline work-manager critical path (workflows cannot nest workflows)
let wm_prompt = "";
wm_prompt += "You are work-manager (inlined by program-control). SILENCE OF SOFT REDS/BLOCKS IS FORBIDDEN.\n";
wm_prompt += "Pulse:\n";
wm_prompt += json_encode(pulse.output);
wm_prompt += "\nExecute work-manager duties:\n";
wm_prompt += "- Inventory PRODUCT HOLDs, ROADMAP, beads ready, open PRs + checks, hindsight if available\n";
wm_prompt += "- Enumerate every soft red and block (lane-board.v1.json sources)\n";
wm_prompt += "- Upsert .grok/harness/lane-board.live.json (dedupe source_key)\n";
wm_prompt += "- Propose parallel_ready and serialized_tip_queue\n";
wm_prompt += "- silence_check_ok must be true\n";
wm_prompt += "Return summary of board writes + next_babysit_prs + next_implement_ids.\n";
let wm = agent(wm_prompt, #{
    label: "ctrl-work-manager",
    capability_mode: "all",
});
if wm == () || !wm.success {
    complete(#{ ok: false, stage: "work-manager", error: "work-manager leg failed", pulse: pulse.output });
}

let baby = ();
if !skip_babysit {
    phase("Babysit");
    let baby_prompt = "";
    baby_prompt += "You are pr-babysit (inlined by program-control). Full autonomy: fix → agent review → approve → merge.\n";
    baby_prompt += "Policy file: .grok/harness/autonomy-merge.v1.json\n";
    baby_prompt += "Work-manager output:\n";
    baby_prompt += json_encode(wm.output);
    baby_prompt += "\nRequirements:\n";
    baby_prompt += "1) Address tip-serial queue head first if multiple tip writers.\n";
    baby_prompt += "2) Restack/fix CI/auth/prebind issues (C prebinds T-final blob).\n";
    baby_prompt += "3) Independent review verdict approve|changes_requested.\n";
    baby_prompt += "4) If not approve, fix until approve (max 5 rounds) or board-block with evidence.\n";
    baby_prompt += "5) On approve + Required CI/Security green + mergeable: gh pr merge --squash (no human gate).\n";
    baby_prompt += "6) Every soft red/block remains on board until resolved — never silent.\n";
    baby_prompt += "7) Human is supervising only; do not pause for approval.\n";
    baby = agent(baby_prompt, #{
        label: "ctrl-babysit",
        capability_mode: "all",
    });
    if baby == () || !baby.success {
        complete(#{ ok: false, stage: "babysit", error: "babysit leg failed", wm: wm.output });
    }
}

let impl = ();
if !skip_implement {
    phase("Implement");
    let impl_prompt = "";
    impl_prompt += "You are implement-lane (inlined by program-control) for AT MOST one ready item if capacity allows.\n";
    impl_prompt += "Read lane-board.live.json. Prefer soft_red/hard_block code fixes not owned by open babysit repair,\n";
    impl_prompt += "else path-disjoint product/process that does not add tip writers while tip_writers>=2.\n";
    impl_prompt += "If only tip-serial work remains and babysit is handling it, idle with reason (productive idle).\n";
    impl_prompt += "If you implement: worktree, tests, signed commits, open PR, update board to pr_open.\n";
    impl_prompt += "Do not merge (babysit does).\n";
    impl_prompt += "Work-manager:\n";
    impl_prompt += json_encode(wm.output);
    if baby != () && baby.success {
        impl_prompt += "\nBabysit:\n";
        impl_prompt += json_encode(baby.output);
    }
    impl = agent(impl_prompt, #{
        label: "ctrl-implement",
        capability_mode: "all",
        isolation_worktree: true,
    });
    if impl == () || !impl.success {
        // implement failure is not total control failure if babysit progressed
        log("implement leg failed — continuing to productivity check");
    }
}

phase("Productivity");
let prod_prompt = "";
prod_prompt += "Productivity auditor for Console autonomous drive. FAIL if soft reds/blocks silent or all legs pure WAIT.\n\n";
prod_prompt += "Pulse:\n";
prod_prompt += json_encode(pulse.output);
prod_prompt += "\nWork-manager:\n";
prod_prompt += json_encode(wm.output);
prod_prompt += "\nBabysit:\n";
if baby != () && baby.success {
    prod_prompt += json_encode(baby.output);
} else {
    prod_prompt += "(skipped or failed)\n";
}
prod_prompt += "\nImplement:\n";
if impl != () && impl.success {
    prod_prompt += json_encode(impl.output);
} else {
    prod_prompt += "(skipped or failed)\n";
}
prod_prompt += "\nRead final .grok/harness/lane-board.live.json.\n";
prod_prompt += "silence_check_ok=false if any open PR is BEHIND/DIRTY/failed/auth-red without a board item source_key covering it.\n";
prod_prompt += "productive=true if any of: merge happened, PR fixed/pushed, board soft reds enqueued+owned, implement opened PR, or explicit tip-serial wait with active babysit actions.\n";
prod_prompt += "productive=false if only narration / pure wait / silent soft reds.\n";
prod_prompt += "workflows_armed: list which of work-manager|pr-babysit|implement-lane should run next wake.\n";
prod_prompt += "next_wake_actions: concrete commands (e.g. /workflow pr-babysit, /workflow work-manager).\n";
let prod = agent(prod_prompt, #{
    label: "ctrl-productivity",
    capability_mode: "execute",
    output_schema: productivity_schema,
});
if prod == () || !prod.success {
    complete(#{
        ok: false,
        stage: "productivity",
        error: "productivity audit failed",
        wm: wm.output,
        baby: if baby != () && baby.success { baby.output } else { () },
    });
}

complete(#{
    ok: prod.output.productive == true && prod.output.silence_check_ok == true,
    workflow: "program-control",
    pulse: pulse.output,
    work_manager: wm.output,
    babysit: if baby != () && baby.success { baby.output } else { () },
    implement: if impl != () && impl.success { impl.output } else { () },
    productivity: prod.output,
    autonomy: "approve_then_merge_human_supervises",
});
