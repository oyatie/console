// implement-lane — claim one board/beads item and drive implementation to a PR.
// Grabs highest-priority ready work that is path-safe. Soft reds/fleet_fix of type
// "code fix" may be claimed here; pure CI restack belongs to pr-babysit.
// Opens PR with signed C/T when required. Does not merge (pr-babysit does after approve).

let meta = #{
    name: "implement-lane",
    description: "Claim ready board/beads work and implement through PR open — no silent skip of soft-red fix items",
    when_to_use: "After work-manager; when board has ready implementable items",
    phases: [
        #{ title: "Claim", detail: "select and claim one item" },
        #{ title: "Admit", detail: "allowlist HOLDs verification" },
        #{ title: "Build", detail: "implement in worktree" },
        #{ title: "Verify", detail: "local tests/gates" },
        #{ title: "Handoff", detail: "signed tip if needed + open PR + board update" },
    ],
};

let claim_schema = #{
    "type": "object",
    "required": ["claimed", "item_id", "kind", "title", "allowlist", "forbidden", "holds_checked", "rationale"],
    "properties": #{
        "claimed": #{ "type": "boolean" },
        "item_id": #{ "type": "string" },
        "kind": #{ "type": "string" },
        "title": #{ "type": "string" },
        "allowlist": #{ "type": "array", "items": #{ "type": "string" } },
        "forbidden": #{ "type": "array", "items": #{ "type": "string" } },
        "holds_checked": #{ "type": "string" },
        "rationale": #{ "type": "string" },
        "pr_number_if_fix": #{ "type": "string" },
        "skip_reason": #{ "type": "string" },
    },
};

let handoff_schema = #{
    "type": "object",
    "required": ["ok", "summary", "pr_url_or_number", "board_updated", "next", "paths_touched", "verification"],
    "properties": #{
        "ok": #{ "type": "boolean" },
        "summary": #{ "type": "string" },
        "pr_url_or_number": #{ "type": "string" },
        "board_updated": #{ "type": "boolean" },
        "next": #{ "type": "string" },
        "paths_touched": #{ "type": "array", "items": #{ "type": "string" } },
        "verification": #{ "type": "string" },
    },
};

let force_id = "";
if args != () && args.item_id != () {
    force_id = args.item_id;
}

phase("Claim");
let claim_prompt = "";
claim_prompt += "Claim ONE implementable item from Console lane board / beads.\n";
claim_prompt += "Read .grok/harness/lane-board.live.json and .grok/harness/lane-board.v1.json priority_order.\n";
claim_prompt += "Run bd ready. Prefer priority: hard_block/soft_red/review_fix that need CODE fixes, then process, then product.\n";
claim_prompt += "Do NOT claim pure restack/CI-wait items (those are pr-babysit) unless the board marks them kind fleet_fix needing code.\n";
claim_prompt += "Do NOT claim tip-serial thrashing product while tip_writers>=2 unless item is the serial tip head fix.\n";
claim_prompt += "PRODUCT HOLDs fail-closed. If force item_id provided use it: ";
claim_prompt += force_id;
claim_prompt += "\nClaim via bd update ID --claim when beads_id present. Set board item status claimed/implementing.\n";
claim_prompt += "If nothing claimable, claimed=false with skip_reason (must not ignore open soft reds — say they are babysit-owned if so).\n";
let claim = agent(claim_prompt, #{
    label: "impl-claim",
    capability_mode: "all",
    output_schema: claim_schema,
});
if claim == () || !claim.success {
    complete(#{ ok: false, stage: "claim", error: "claim failed" });
}
if claim.output.claimed != true {
    complete(#{
        ok: true,
        workflow: "implement-lane",
        idle: true,
        skip_reason: claim.output.skip_reason,
        note: "No implementable claim — soft reds must still be on board for pr-babysit",
    });
}

phase("Admit");
let admit_prompt = "";
admit_prompt += "Admission for claimed item. Enforce allowlist/forbidden/HOLDs.\n";
admit_prompt += json_encode(claim.output);
admit_prompt += "\nRead PRODUCT HOLDs. Write short admission: objective, non_goals, verification_commands, stop_conditions.\n";
admit_prompt += "If HOLD would be violated, stop and requeue as blocked on board.\n";
let admit = agent(admit_prompt, #{
    label: "impl-admit",
    capability_mode: "execute",
});
if admit == () || !admit.success {
    complete(#{ ok: false, stage: "admit", error: "admit failed", claim: claim.output });
}

phase("Build");
let build_prompt = "";
build_prompt += "IMPLEMENT code for the claimed item — this workflow drives implementation, not ticket text.\n";
build_prompt += "Isolated worktree required. Produce real file diffs.\n";
build_prompt += "Claim:\n";
build_prompt += json_encode(claim.output);
build_prompt += "\nAdmission:\n";
build_prompt += json_encode(admit.output);
build_prompt += "\nRules:\n";
build_prompt += "- Stay inside allowlist; never docs/current/** product authority edits.\n";
build_prompt += "- Authority tip C/T SSH-signed when PR needs tip; C prebinds T-final ledger blob_sha (docs.tip-blob-prebind).\n";
build_prompt += "- Do not open a second tip-writing PR if tip_writers>=2 unless this item IS the serial tip drain fix.\n";
build_prompt += "- Prefer batch pure domain tests on one branch.\n";
build_prompt += "- rustfmt + cargo test for touched crates; admit gates if process lane.\n";
build_prompt += "- Fail closed if you only write docs/ledger without product/process allowlist code when claim is product/process code.\n";
build_prompt += "Commit signed (gpg.format ssh). Push branch. Open PR with gh pr create if tip capacity allows; else local branch only and say so.\n";
build_prompt += "Return PR number/url, paths_touched[], and verification command output summary.\n";
let built = agent(build_prompt, #{
    label: "impl-build",
    capability_mode: "all",
    isolation_worktree: true,
});
if built == () || !built.success {
    complete(#{ ok: false, stage: "build", error: "build failed — edit implement-lane Build phase if agents narrate without code", claim: claim.output });
}

phase("Verify");
let ver_prompt = "";
ver_prompt += "Re-verify implementation evidence from build output. Run key verification commands.\n";
ver_prompt += json_encode(built.output);
ver_prompt += "\nIf red, fix in place (same branch) up to 2 tight loops. Report pass/fail.\n";
let verified = agent(ver_prompt, #{
    label: "impl-verify",
    capability_mode: "all",
});
if verified == () || !verified.success {
    complete(#{ ok: false, stage: "verify", error: "verify failed", build: built.output });
}

phase("Handoff");
let hand_prompt = "";
hand_prompt += "Handoff: ensure PR is open; update lane-board.live.json item status to pr_open or review_pending;\n";
hand_prompt += "include pr_number; clear claim locks appropriately; bd update if needed.\n";
hand_prompt += "Claim:\n";
hand_prompt += json_encode(claim.output);
hand_prompt += "\nBuild:\n";
hand_prompt += json_encode(built.output);
hand_prompt += "\nVerify:\n";
hand_prompt += json_encode(verified.output);
hand_prompt += "\nDo NOT merge. next must say run pr-babysit for this PR.\n";
let hand = agent(hand_prompt, #{
    label: "impl-handoff",
    capability_mode: "all",
    output_schema: handoff_schema,
});
if hand == () || !hand.success {
    complete(#{ ok: false, stage: "handoff", error: "handoff failed", build: built.output });
}

complete(#{
    ok: hand.output.ok,
    workflow: "implement-lane",
    claim: claim.output,
    handoff: hand.output,
});
