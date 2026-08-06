// pr-babysit — drive open PRs to merge under autonomy-merge policy.
// Fix CI/conflicts/soft reds → agent review → if not approve fix until approve → merge.
// Soft reds and blocks MUST be addressed or re-enqueued (never silent).
// Human supervises; intervenes only if awry. Default: full autonomous.

let meta = #{
    name: "pr-babysit",
    description: "Babysit PRs: fix reds/blocks, agent-review until approve, then merge — soft reds never silent",
    when_to_use: "Whenever open PRs exist; after implement-lane; every program-control tick",
    phases: [
        #{ title: "Select", detail: "pick PR(s) needing action including soft reds" },
        #{ title: "Repair", detail: "restack, CI fix, tip prebind, conflicts" },
        #{ title: "Review", detail: "independent agent review verdict" },
        #{ title: "FixLoop", detail: "until approve or max rounds" },
        #{ title: "Merge", detail: "merge when approve + Required green" },
        #{ title: "Board", detail: "update board; requeue residual soft reds" },
    ],
};

let select_schema = #{
    "type": "object",
    "required": ["targets", "soft_reds_seen", "blocks_seen", "none_idle_ok"],
    "properties": #{
        "targets": #{ "type": "array", "items": #{ "type": "string" } },
        "soft_reds_seen": #{ "type": "array", "items": #{ "type": "string" } },
        "blocks_seen": #{ "type": "array", "items": #{ "type": "string" } },
        "none_idle_ok": #{ "type": "boolean" },
        "notes": #{ "type": "string" },
    },
};

let review_schema = #{
    "type": "object",
    "required": ["pr", "verdict", "summary", "must_fix"],
    "properties": #{
        "pr": #{ "type": "string" },
        "verdict": #{ "type": "string" },
        "summary": #{ "type": "string" },
        "must_fix": #{ "type": "array", "items": #{ "type": "string" } },
        "risk": #{ "type": "string" },
    },
};

let merge_schema = #{
    "type": "object",
    "required": ["pr", "merged", "reason"],
    "properties": #{
        "pr": #{ "type": "string" },
        "merged": #{ "type": "boolean" },
        "reason": #{ "type": "string" },
        "sha": #{ "type": "string" },
    },
};

let only_pr = "";
if args != () && args.pr != () {
    only_pr = args.pr;
}
let max_rounds = 5;
if args != () && args.max_fix_rounds != () {
    // args may be number-like string; agents honor prompt default 5
    max_rounds = 5;
}
let allow_merge = true;
if args != () && args.allow_merge == false {
    allow_merge = false;
}

phase("Select");
let sel_prompt = "";
sel_prompt += "Select open PRs to babysit. Soft reds/blocks MUST appear in soft_reds_seen/blocks_seen even if not yet fixable.\n";
sel_prompt += "Run gh pr list --state open --limit 30 --json number,title,mergeable,mergeStateStatus,headRefOid,url.\n";
sel_prompt += "Read .grok/harness/lane-board.live.json for fleet_fix/soft_red/review_fix items.\n";
sel_prompt += "Read .grok/harness/autonomy-merge.v1.json.\n";
sel_prompt += "For each PR classify: waiting_ci, failed, behind, dirty, auth_fail, review_pending, merge_ready_candidate.\n";
sel_prompt += "targets: ordered PR numbers as strings (tip-serial order if tip writers).\n";
if only_pr != "" {
    sel_prompt += "Focus only on PR ";
    sel_prompt += only_pr;
    sel_prompt += " but still list ALL soft reds/blocks seen fleet-wide in soft_reds_seen/blocks_seen.\n";
}
sel_prompt += "none_idle_ok=true only if targets non-empty OR truly no open PRs and no board soft reds.\n";
sel_prompt += "SILENCE FORBIDDEN: every BEHIND/DIRTY/fail/auth red must be in soft_reds_seen or blocks_seen.\n";
let selected = agent(sel_prompt, #{
    label: "baby-select",
    capability_mode: "execute",
    output_schema: select_schema,
});
if selected == () || !selected.success {
    complete(#{ ok: false, stage: "select", error: "select failed" });
}

phase("Repair");
let rep_prompt = "";
rep_prompt += "Repair pass for babysit targets. Policy: full autonomy under human supervision.\n";
rep_prompt += "Selection:\n";
rep_prompt += json_encode(selected.output);
rep_prompt += "\nFor each target PR (tip-serial order for manifest/baseline/tip PRs):\n";
rep_prompt += "1) If BEHIND/DIRTY/CONFLICTING: restack onto origin/main with SSH-signed C/T; tip prebind ";
rep_prompt += "(C ledger without ## Authority tip footer; manifest blob_sha = T-final; never naive generate --write after tip).\n";
rep_prompt += "2) If CI/auth/preflight red: diagnose logs, fix code/process on branch, push force-with-lease when needed.\n";
rep_prompt += "3) Soft reds (non-required fails, cancelled, stale): fix or convert to board item with evidence — do not ignore.\n";
rep_prompt += "4) Do not merge in this phase.\n";
rep_prompt += "Report per-PR actions and residual reds.\n";
let repaired = agent(rep_prompt, #{
    label: "baby-repair",
    capability_mode: "all",
});
if repaired == () || !repaired.success {
    complete(#{ ok: false, stage: "repair", error: "repair failed", select: selected.output });
}

phase("Review");
let rev_prompt = "";
rev_prompt += "Independent code review of the primary target PR (first in targets, or only_pr).\n";
rev_prompt += "Use gh pr diff / gh pr view / read files. Adversarial but practical.\n";
rev_prompt += "Selection:\n";
rev_prompt += json_encode(selected.output);
rev_prompt += "\nRepair notes:\n";
rev_prompt += json_encode(repaired.output);
rev_prompt += "\nverdict MUST be exactly one of: approve | changes_requested | comment\n";
rev_prompt += "approve only if: Required-path safe, no HOLD violation, tip/authority sane, tests adequate for change, no serious defect.\n";
rev_prompt += "If not approve, must_fix must be concrete actionable items.\n";
rev_prompt += "This verdict gates autonomous merge (see autonomy-merge.v1.json).\n";
let reviewed = agent(rev_prompt, #{
    label: "baby-review",
    capability_mode: "execute",
    output_schema: review_schema,
});
if reviewed == () || !reviewed.success {
    complete(#{ ok: false, stage: "review", error: "review failed" });
}

phase("FixLoop");
let fix_prompt = "";
fix_prompt += "Fix loop until approve. max_fix_rounds=";
fix_prompt += max_rounds.to_string();
fix_prompt += "\nCurrent review:\n";
fix_prompt += json_encode(reviewed.output);
fix_prompt += "\nIf verdict already approve, do nothing except confirm Required CI/Security green on exact head.\n";
fix_prompt += "If not approve: implement must_fix on the PR branch, push, re-check CI, re-self-review.\n";
fix_prompt += "Loop until approve OR rounds exhausted. On exhaust: leave PR open, board status blocked, list residuals — NEVER silent-drop.\n";
fix_prompt += "Re-state final verdict: approve | changes_requested | blocked_exhausted.\n";
fix_prompt += "Include PR number and head sha.\n";
let fixed = agent(fix_prompt, #{
    label: "baby-fix-loop",
    capability_mode: "all",
});
if fixed == () || !fixed.success {
    complete(#{ ok: false, stage: "fix-loop", error: "fix loop failed", review: reviewed.output });
}

phase("Merge");
let merge_result = #{ merged: false, reason: "skipped", pr: "" };
if allow_merge {
    let merge_prompt = "";
    merge_prompt += "Autonomous merge gate (human supervise-only).\n";
    merge_prompt += "Read .grok/harness/autonomy-merge.v1.json.\n";
    merge_prompt += "Review+fix output:\n";
    merge_prompt += json_encode(fixed.output);
    merge_prompt += "\nEarlier review:\n";
    merge_prompt += json_encode(reviewed.output);
    merge_prompt += "\nMerge with gh pr merge N --squash ONLY IF all hold:\n";
    merge_prompt += "- agent verdict approve (from fix loop final or review)\n";
    merge_prompt += "- Required / CI success on exact head\n";
    merge_prompt += "- Required / Security success on exact head\n";
    merge_prompt += "- mergeable and not BEHIND\n";
    merge_prompt += "If not met: merged=false with reason; enqueue soft red/block on board.\n";
    merge_prompt += "If met: merge and report sha. No human approval step required.\n";
    let merged = agent(merge_prompt, #{
        label: "baby-merge",
        capability_mode: "all",
        output_schema: merge_schema,
    });
    if merged != () && merged.success {
        merge_result = merged.output;
    } else {
        merge_result = #{ merged: false, reason: "merge agent failed", pr: "" };
    }
} else {
    merge_result = #{ merged: false, reason: "allow_merge=false", pr: only_pr };
}

phase("Board");
let board_prompt = "";
board_prompt += "Update .grok/harness/lane-board.live.json after babysit.\n";
board_prompt += "Select soft reds/blocks seen:\n";
board_prompt += json_encode(selected.output);
board_prompt += "\nRepair:\n";
board_prompt += json_encode(repaired.output);
board_prompt += "\nReview:\n";
board_prompt += json_encode(reviewed.output);
board_prompt += "\nFix:\n";
board_prompt += json_encode(fixed.output);
board_prompt += "\nMerge:\n";
board_prompt += json_encode(merge_result);
board_prompt += "\nRules:\n";
board_prompt += "- Every soft_reds_seen and blocks_seen must still have a board item (upsert) unless truly resolved (mark merged/done with evidence).\n";
board_prompt += "- Merged PR → status merged; close related beads if appropriate.\n";
board_prompt += "- Non-approve residual → review_fix ready with fix_rounds incremented.\n";
board_prompt += "- SILENCE FORBIDDEN.\n";
board_prompt += "Write lane-board.live.json. Return brief summary text.\n";
let board = agent(board_prompt, #{
    label: "baby-board",
    capability_mode: "all",
});

complete(#{
    ok: true,
    workflow: "pr-babysit",
    select: selected.output,
    review: reviewed.output,
    fix: fixed.output,
    merge: merge_result,
    board: if board != () && board.success { board.output } else { "board update failed" },
    autonomy: "approve_then_merge",
});
