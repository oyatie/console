// work-manager — discover parallel lanes + enqueue soft reds/blocks (no silence).
// Sources: Beads, ROADMAP/PRODUCT, durable goals, open PRs, Hindsight, live board.
// Does not implement product code. Does not merge.
// Doctrine: BUN-PARALLEL-DISCIPLINE + autonomy-merge.v1.json + lane-board.v1.json

let meta = #{
    name: "work-manager",
    description: "Discover parallel lanes from backlog/roadmap/beads/hindsight and enqueue soft reds/blocks — never silent-drop",
    when_to_use: "Every program-control tick; start of autonomous drive; when board may be stale",
    phases: [
        #{ title: "Inventory", detail: "authority + beads + open PRs + hindsight + live board" },
        #{ title: "SoftRedsBlocks", detail: "every soft red and block becomes a board item" },
        #{ title: "Lanes", detail: "path-disjoint ready lanes from roadmap/beads" },
        #{ title: "WriteBoard", detail: "update lane-board.live.json + optional beads creates" },
    ],
};

let board_schema = #{
    "type": "object",
    "required": [
        "items_added",
        "items_updated",
        "soft_reds_enqueued",
        "blocks_enqueued",
        "parallel_ready",
        "serialized_tip_queue",
        "silence_check_ok",
        "summary",
    ],
    "properties": #{
        "items_added": #{ "type": "array", "items": #{ "type": "string" } },
        "items_updated": #{ "type": "array", "items": #{ "type": "string" } },
        "soft_reds_enqueued": #{ "type": "array", "items": #{ "type": "string" } },
        "blocks_enqueued": #{ "type": "array", "items": #{ "type": "string" } },
        "parallel_ready": #{ "type": "array", "items": #{ "type": "string" } },
        "serialized_tip_queue": #{ "type": "array", "items": #{ "type": "string" } },
        "silence_check_ok": #{ "type": "boolean" },
        "summary": #{ "type": "string" },
        "next_implement_ids": #{ "type": "array", "items": #{ "type": "string" } },
        "next_babysit_prs": #{ "type": "array", "items": #{ "type": "string" } },
    },
};

phase("Inventory");
let inv_prompt = "";
inv_prompt += "You are Console work-manager Inventory. Use tools (shell, read_file, gh, bd, hindsight MCP if available).\n";
inv_prompt += "Collect and report as plain structured text:\n";
inv_prompt += "1) docs/current/PRODUCT.md HOLDs (list exact HOLD lines).\n";
inv_prompt += "2) docs/current/ROADMAP.md ordered work (summarize open items).\n";
inv_prompt += "3) .grok/programs/ROADMAP-DURABLE-GOAL.md G001–G009 status hints.\n";
inv_prompt += "4) bd ready + bd list --status=open + bd list --status=in_progress (run bd prime first if needed).\n";
inv_prompt += "5) gh pr list --state open --limit 30 --json number,title,headRefName,mergeable,mergeStateStatus,headRefOid,url.\n";
inv_prompt += "6) For each open PR: Required CI + Security + notable fail/pending checks (gh pr checks N).\n";
inv_prompt += "7) Read .grok/harness/lane-board.live.json and .grok/harness/lane-board.v1.json soft_red_sources.\n";
inv_prompt += "8) Hindsight recall if available: query 'console tip serial soft red CI fail process blockers'.\n";
inv_prompt += "9) Tip-serial writers: PRs touching documentation-manifest, executed-tests-baseline, ci.yml, authority tip.\n";
inv_prompt += "Do NOT edit product code. You MAY draft board updates in the next phases.\n";
let inventory = agent(inv_prompt, #{
    label: "wm-inventory",
    capability_mode: "execute",
});
if inventory == () || !inventory.success {
    complete(#{ ok: false, stage: "inventory", error: "inventory failed" });
}

phase("SoftRedsBlocks");
let soft_prompt = "";
soft_prompt += "From this inventory, enumerate EVERY soft red and EVERY hard/soft block. SILENCE IS FORBIDDEN.\n\n";
soft_prompt += json_encode(inventory.output);
soft_prompt += "\n\nRead soft_red_sources and hard_block_sources from .grok/harness/lane-board.v1.json.\n";
soft_prompt += "For each observation produce a board item draft:\n";
soft_prompt += "- source_key: stable (e.g. soft_red:pr:591:auth_bootstrap_fail, block:pr:592:conflicting)\n";
soft_prompt += "- kind: soft_red | hard_block | fleet_fix | review_fix\n";
soft_prompt += "- title, priority P0-P2, pr_number if any, evidence (check URLs), status ready or fixing\n";
soft_prompt += "- If already on live board with same source_key, mark update not duplicate silent skip.\n";
soft_prompt += "Include: BEHIND, DIRTY, CONFLICTING, auth fail, preflight fail, non-required reds, cancelled runs,";
soft_prompt += " stale waiting_ci, tip_writers>=2, unsigned tip, review not approve, beads blocked without owner.\n";
soft_prompt += "Return a clear list. No empty claim without stating zero after exhaustive check.\n";
let softs = agent(soft_prompt, #{
    label: "wm-soft-reds",
    capability_mode: "execute",
});
if softs == () || !softs.success {
    complete(#{ ok: false, stage: "soft-reds", error: "soft red enumeration failed", inventory: inventory.output });
}

phase("Lanes");
let lanes_prompt = "";
lanes_prompt += "Propose path-disjoint parallel READY lanes from beads/roadmap that do NOT thrash tip-serial files ";
lanes_prompt += "while tip queue is non-empty. Prefer: pure domain tests on disjoint crates (batch), process allowlist, ";
lanes_prompt += "read-only audit, residual Buck design only if no other ci.yml writer.\n\n";
lanes_prompt += "Inventory:\n";
lanes_prompt += json_encode(inventory.output);
lanes_prompt += "\n\nSoft reds/blocks (must stay higher priority than new product fan-out):\n";
lanes_prompt += json_encode(softs.output);
lanes_prompt += "\n\nRules from BUN-PARALLEL-DISCIPLINE: serialize tip/manifest/baseline/ci.yml; parallel path-disjoint crates.\n";
lanes_prompt += "PRODUCT HOLDs fail-closed. Max 4 new product ready items unless tip_writers==0.\n";
lanes_prompt += "Each lane: id, kind product|process|substrate, allowlist, forbidden, beads_id or goal_id, status ready.\n";
let lanes = agent(lanes_prompt, #{
    label: "wm-lanes",
    capability_mode: "execute",
});
if lanes == () || !lanes.success {
    complete(#{ ok: false, stage: "lanes", error: "lane planning failed" });
}

phase("WriteBoard");
let write_prompt = "";
write_prompt += "Update the Console lane board. SILENCE CHECK is mandatory.\n\n";
write_prompt += "Soft reds/blocks drafts:\n";
write_prompt += json_encode(softs.output);
write_prompt += "\n\nParallel lanes drafts:\n";
write_prompt += json_encode(lanes.output);
write_prompt += "\n\nActions:\n";
write_prompt += "1) Read .grok/harness/lane-board.live.json.\n";
write_prompt += "2) Merge items by source_key (upsert). Never drop an observed soft red/block without writing it.\n";
write_prompt += "3) Write the full updated JSON to .grok/harness/lane-board.live.json (valid JSON, version 1.0.0, updated_at ISO-ish, updated_by work-manager).\n";
write_prompt += "4) Optionally bd create for new durable soft_red/block items not yet in beads (type=task, priority by P0/P1).\n";
write_prompt += "5) silence_check_ok=true only if every soft red/block from step SoftReds is present on the board after write.\n";
write_prompt += "6) next_implement_ids: ready items safe to implement (not tip-serial thrash).\n";
write_prompt += "7) next_babysit_prs: PR numbers needing babysit/fix/review.\n";
write_prompt += "Do not merge PRs. Do not expand PRODUCT HOLDs.\n";
let written = agent(write_prompt, #{
    label: "wm-write-board",
    capability_mode: "all",
    output_schema: board_schema,
});
if written == () || !written.success {
    complete(#{ ok: false, stage: "write-board", error: "board write failed", softs: softs.output, lanes: lanes.output });
}
if written.output.silence_check_ok != true {
    complete(#{
        ok: false,
        stage: "silence-check",
        error: "soft reds/blocks would be silent — board write rejected",
        board: written.output,
    });
}

complete(#{
    ok: true,
    workflow: "work-manager",
    board: written.output,
});
