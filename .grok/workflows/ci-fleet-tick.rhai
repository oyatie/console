// CI fleet tick — one half of the anti-wait pair.
// NEVER treat waiting_ci as a terminal idle state.
// Pair with: product-process-tick (must run when this reports waiting_ci).
// Leader-only merge. Process edits go through process-upgrade, not here.
// Doctrine: .grok/programs/BUN-PARALLEL-DISCIPLINE.md

let meta = #{
    name: "ci-fleet-tick",
    description: "Fleet half of dual-track drive: PR/CI status, tip-sync needs, CLEAN report — never end as pure WAIT",
    when_to_use: "Every autonomous wake while open PRs exist; always pair with product-process-tick if any PR is waiting_ci",
    phases: [
        #{ title: "Inspect", detail: "open PRs and exact-head CI/Security" },
        #{ title: "Classify", detail: "CLEAN / waiting_ci / failed / behind / blocked" },
        #{ title: "Act", detail: "report merge-ready, fail URLs, tip-sync list; arm next action" },
    ],
};

let fleet_schema = #{
    "type": "object",
    "required": [
        "open_prs",
        "clean_merge_ready",
        "waiting_ci",
        "failed",
        "behind",
        "blocked",
        "tip_sync_needed",
        "next_fleet_action",
        "must_run_product_process_tick",
        "anti_passive_ok",
    ],
    "properties": #{
        "open_prs": #{ "type": "array", "items": #{ "type": "string" } },
        "clean_merge_ready": #{ "type": "array", "items": #{ "type": "string" } },
        "waiting_ci": #{ "type": "array", "items": #{ "type": "string" } },
        "failed": #{ "type": "array", "items": #{ "type": "string" } },
        "behind": #{ "type": "array", "items": #{ "type": "string" } },
        "blocked": #{ "type": "array", "items": #{ "type": "string" } },
        "tip_sync_needed": #{ "type": "array", "items": #{ "type": "string" } },
        "next_fleet_action": #{ "type": "string" },
        "must_run_product_process_tick": #{ "type": "boolean" },
        "anti_passive_ok": #{ "type": "boolean" },
        "notes": #{ "type": "string" },
    },
};

phase("Inspect");
let inspect = agent(
    "Console CI fleet inspect (read tools + gh). "
        + "Run: gh pr list --state open --limit 25 --json number,title,headRefName,mergeStateStatus,headRefOid,url. "
        + "For each OPEN PR that looks substrate/CI/docs (or all open if few), get CI and Security workflow conclusions for exact headRefOid "
        + "(gh run list --branch HEAD_REF --limit 8 --json name,status,conclusion,headSha). "
        + "Do not merge. Do not edit files. Do not claim success without exact head SHA match. "
        + "Return plain text: table of PR | sha9 | mergeState | CI | Security | url.",
    #{
        label: "fleet-inspect",
        capability_mode: "execute",
    },
);
if inspect == () || !inspect.success {
    complete(#{ ok: false, stage: "inspect", error: "fleet inspect failed" });
}

phase("Classify");
let classify_prompt = "";
classify_prompt += "Using this fleet inspect output, classify every open PR into buckets.\n\n";
classify_prompt += json_encode(inspect.output);
classify_prompt += "\n\nRules:\n";
classify_prompt += "- clean_merge_ready: CI success + Security success + MERGEABLE + not BEHIND (leader merges; do not merge yourself).\n";
classify_prompt += "- waiting_ci: CI or Security in_progress/queued/pending on exact head.\n";
classify_prompt += "- failed: CI or Security failure on exact head (include run URL in notes).\n";
classify_prompt += "- behind: mergeStateStatus BEHIND (tip-sync/restack needed).\n";
classify_prompt += "- blocked: other blockers (draft, conflicts, auth bootstrap fail, etc.).\n";
classify_prompt += "- tip_sync_needed: BEHIND or base main moved under open head.\n";
classify_prompt += "- next_fleet_action: ONE concrete next action for fleet (e.g. 'leader merge #585', 'diagnose fail #N url', 'restack #N onto main', 're-poll in 3m after arming monitor').\n";
classify_prompt += "- must_run_product_process_tick: TRUE if waiting_ci is non-empty OR clean_merge_ready is empty and failed is empty — i.e. CI wait must not idle product/process track.\n";
classify_prompt += "- anti_passive_ok: TRUE only if next_fleet_action is non-empty AND (failed/behind/clean have actions OR must_run_product_process_tick is true with explicit pairing note).\n";
classify_prompt += "Forbidden: next_fleet_action = 'wait' or 'still running' alone. That is class ops.passive-wait.\n";
let classified = agent(classify_prompt, #{
    label: "fleet-classify",
    capability_mode: "read-only",
    output_schema: fleet_schema,
});
if classified == () || !classified.success || classified.output == () {
    complete(#{ ok: false, stage: "classify", error: "classify failed", inspect: inspect.output });
}

phase("Act");
let act_prompt = "";
act_prompt += "Produce a short operator-facing fleet report from this classification.\n";
act_prompt += json_encode(classified.output);
act_prompt += "\n\nHard rules:\n";
act_prompt += "1) If clean_merge_ready non-empty: list exact PRs for LEADER squash-merge (agents do not merge).\n";
act_prompt += "2) If failed non-empty: list fail URLs and map to a failure class id from .grok/harness/failure-classes.v1.json if possible.\n";
act_prompt += "3) If behind non-empty: tip-sync list (restack in PR worktree, force-with-lease only after signed tip if authority).\n";
act_prompt += "4) If waiting_ci non-empty: you MUST state: run workflow product-process-tick now (do not end session as WAIT).\n";
act_prompt += "5) Never recommend mid-run push thrash (ops.mid-run-push).\n";
act_prompt += "Plain text only.\n";
let report = agent(act_prompt, #{
    label: "fleet-report",
    capability_mode: "read-only",
});

let must_pair = false;
if classified.output.must_run_product_process_tick == true {
    must_pair = true;
}
if classified.output.waiting_ci != () {
    // schema array — if present and we could not read bool, still pair when waiting listed
    must_pair = true;
}

complete(#{
    ok: classified.output.anti_passive_ok == true,
    pair_workflow: "product-process-tick",
    must_run_product_process_tick: must_pair,
    fleet: classified.output,
    report: if report != () && report.success { report.output } else { () },
    inspect: inspect.output,
    doctrine: "BUN-PARALLEL-DISCIPLINE dual-track: ci-fleet-tick + product-process-tick",
});
