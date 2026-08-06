// console-drive — Bun-doctrine autonomous implementation control plane.
// https://bun.com/blog/bun-in-rust
//
// This workflow DRIVES implementation. It is not a ticket board.
// Failures of process must be fixed by editing THIS file / harness / tools —
// not by chat "remember to…". Soft reds are inputs to fix loops, not the product.
//
// Durable phases:
//   1. ProcessHealth  — mechanical ingest + tip contention (tools)
//   2. FleetRepair    — restack/fix/review/merge open PRs (implementation of fixes)
//   3. ProductBuild   — claim path-disjoint product work; code + tests + PR
//   4. ProcessEdit    — if a failure class hit ≥1 this run, patch workflow/tool/harness
//   5. LearnProof     — Hermes learn + ledger evidence that drive did real work
//
// Forbidden: complete() after only narration; skip ProductBuild without tip_serial exclusive reason;
// silent soft reds; PRODUCT HOLD clearance.

let meta = #{
    name: "console-drive",
    description: "Bun-style durable drive: process tools + fleet repair/merge + product implement + process-edit + learn — not board-only",
    when_to_use: "Primary autonomous implementation driver every wake; edit this workflow when the drive class fails",
    phases: [
        #{ title: "ProcessHealth", detail: "mechanical soft-red ingest + tip assess" },
        #{ title: "FleetRepair", detail: "restack fix review merge open PRs" },
        #{ title: "ProductBuild", detail: "implement next product/process allowlist work" },
        #{ title: "ProcessEdit", detail: "edit reusable workflow/tool if class failed" },
        #{ title: "LearnProof", detail: "learn + proof of non-idle drive" },
    ],
};

let health_schema = #{
    "type": "object",
    "required": ["soft_red_ingest_ok", "tip_writers", "open_prs", "fleet_targets", "product_capacity"],
    "properties": #{
        "soft_red_ingest_ok": #{ "type": "boolean" },
        "tip_writers": #{ "type": "number" },
        "open_prs": #{ "type": "array", "items": #{ "type": "string" } },
        "fleet_targets": #{ "type": "array", "items": #{ "type": "string" } },
        "product_capacity": #{ "type": "boolean" },
        "notes": #{ "type": "string" },
    },
};

let fleet_schema = #{
    "type": "object",
    "required": ["actions", "merged", "still_open", "classes_hit"],
    "properties": #{
        "actions": #{ "type": "array", "items": #{ "type": "string" } },
        "merged": #{ "type": "array", "items": #{ "type": "string" } },
        "still_open": #{ "type": "array", "items": #{ "type": "string" } },
        "classes_hit": #{ "type": "array", "items": #{ "type": "string" } },
        "evidence": #{ "type": "string" },
    },
};

let product_schema = #{
    "type": "object",
    "required": ["did_implement", "summary", "paths_touched", "verification", "pr_or_local"],
    "properties": #{
        "did_implement": #{ "type": "boolean" },
        "summary": #{ "type": "string" },
        "paths_touched": #{ "type": "array", "items": #{ "type": "string" } },
        "verification": #{ "type": "string" },
        "pr_or_local": #{ "type": "string" },
        "skip_reason": #{ "type": "string" },
        "classes_hit": #{ "type": "array", "items": #{ "type": "string" } },
    },
};

let process_schema = #{
    "type": "object",
    "required": ["edited_process", "files", "class_ids", "summary"],
    "properties": #{
        "edited_process": #{ "type": "boolean" },
        "files": #{ "type": "array", "items": #{ "type": "string" } },
        "class_ids": #{ "type": "array", "items": #{ "type": "string" } },
        "summary": #{ "type": "string" },
    },
};

let proof_schema = #{
    "type": "object",
    "required": ["productive", "implementation_happened", "silence_check_ok", "summary", "next_drive"],
    "properties": #{
        "productive": #{ "type": "boolean" },
        "implementation_happened": #{ "type": "boolean" },
        "silence_check_ok": #{ "type": "boolean" },
        "summary": #{ "type": "string" },
        "next_drive": #{ "type": "string" },
        "defects": #{ "type": "array", "items": #{ "type": "string" } },
    },
};

let skip_product = false;
if args != () && args.skip_product == true {
    skip_product = true;
}
let allow_merge = true;
if args != () && args.allow_merge == false {
    allow_merge = false;
}

// --- 1 ProcessHealth (mechanical tools preferred) ---
phase("ProcessHealth");
let health_prompt = "";
health_prompt += "You are Console-drive ProcessHealth. This phase is MECHANICAL + brief report.\n";
health_prompt += "MUST run (use shell):\n";
health_prompt += "1) `node tools/ci/ingest-soft-reds.mjs` then `node tools/ci/ingest-soft-reds.mjs --check`\n";
health_prompt += "2) `gh pr list --state open --limit 30 --json number,title,mergeable,mergeStateStatus,headRefOid`\n";
health_prompt += "3) If present: `node tools/ci/assess-tip-contention.mjs` or count tip-touching open PRs (manifest/baseline/ledger tip)\n";
health_prompt += "4) `bd ready | head -20`\n";
health_prompt += "5) Read .grok/ultragoal/active-goal.live.json\n";
health_prompt += "product_capacity=true if tip_writers<=1 OR there is path-disjoint non-tip work (pure domain tests batch, process allowlist).\n";
health_prompt += "fleet_targets = PR numbers needing restack/fix/merge in tip-serial order.\n";
health_prompt += "soft_red_ingest_ok=false if --check fails.\n";
health_prompt += "Do not implement product code here.\n";
let health = agent(health_prompt, #{
    label: "drive-health",
    capability_mode: "all",
    output_schema: health_schema,
});
if health == () || !health.success {
    complete(#{ ok: false, stage: "process-health", error: "health failed — edit console-drive.rhai ProcessHealth if agents skip tools" });
}
if health.output.soft_red_ingest_ok != true {
    complete(#{
        ok: false,
        stage: "process-health",
        error: "ops.soft-red-silence — ingest/check failed; fix tools/ci/ingest-soft-reds.mjs or board",
        health: health.output,
    });
}

// --- 2 FleetRepair (implementation of fixes) ---
phase("FleetRepair");
let fleet_prompt = "";
fleet_prompt += "You are Console-drive FleetRepair. IMPLEMENT fixes — do not only report.\n";
fleet_prompt += "Health:\n";
fleet_prompt += json_encode(health.output);
fleet_prompt += "\nFor each fleet_target in tip-serial order (tip writers first):\n";
fleet_prompt += "1) Restack onto origin/main with SSH-signed C/T; tip prebind (C prebinds T-final ledger blob).\n";
fleet_prompt += "2) Fix CI/auth/preflight reds on the branch; push --force-with-lease.\n";
fleet_prompt += "3) Independent mini-review: if approve-worthy and Required CI+Security green and mergeable:\n";
if allow_merge {
    fleet_prompt += "   `gh pr merge N --squash` (autonomy-merge policy).\n";
} else {
    fleet_prompt += "   report CLEAN only (allow_merge=false).\n";
}
fleet_prompt += "4) Record failure classes hit (auth.unsigned-tip, docs.tip-blob-prebind, ops.missed-tip-sync, …).\n";
fleet_prompt += "If no open PRs, actions=['none'] merged=[] still_open=[].\n";
fleet_prompt += "Evidence must include commands run and outcomes — not vibes.\n";
let fleet = agent(fleet_prompt, #{
    label: "drive-fleet",
    capability_mode: "all",
    output_schema: fleet_schema,
});
if fleet == () || !fleet.success {
    complete(#{ ok: false, stage: "fleet-repair", error: "fleet repair failed", health: health.output });
}

// --- 3 ProductBuild (real implementation) ---
let product = #{
    did_implement: false,
    summary: "skipped",
    paths_touched: [],
    verification: "",
    pr_or_local: "none",
    skip_reason: "",
    classes_hit: [],
};
if !skip_product {
    phase("ProductBuild");
    let prod_prompt = "";
    prod_prompt += "You are Console-drive ProductBuild. You MUST produce code or an explicit capacity skip.\n";
    prod_prompt += "Health:\n";
    prod_prompt += json_encode(health.output);
    prod_prompt += "\nFleet:\n";
    prod_prompt += json_encode(fleet.output);
    prod_prompt += "\nRules (Bun prep → trial → fan-out):\n";
    prod_prompt += "- If tip_writers>=2: only implement process fixes for the tip head OR pure local dual-clock work WITHOUT opening a new tip PR.\n";
    prod_prompt += "- Else implement ONE of:\n";
    prod_prompt += "  A) Path-disjoint pure domain unit tests (prefer sparse crates: inspection already done; benefit/dispatch/finance-gl/logistics).\n";
    prod_prompt += "  B) Process allowlist hardening already scoped (admit gates, assess tools) as a single tip PR after fleet drain.\n";
    prod_prompt += "  C) Residual Buck company-conformance cargo cutover ONLY if no other ci.yml writer open.\n";
    prod_prompt += "- Read PRODUCT HOLDs; never clear HOLDs.\n";
    prod_prompt += "- Worktree; rustfmt; cargo test -p <crate> --lib; signed commits; open PR if tip capacity allows.\n";
    prod_prompt += "- Authority tip C/T when required; prebind tip blob correctly.\n";
    prod_prompt += "did_implement=true only if files changed with verification evidence.\n";
    prod_prompt += "skip_reason only if fleet still owns exclusive tip serial AND no local dual-clock possible — justify.\n";
    let prod = agent(prod_prompt, #{
        label: "drive-product",
        capability_mode: "all",
        isolation_worktree: true,
        output_schema: product_schema,
    });
    if prod == () || !prod.success {
        complete(#{ ok: false, stage: "product-build", error: "product build agent failed", fleet: fleet.output });
    }
    product = prod.output;
    // Fail closed: idle product without capacity justification is process defect
    if product.did_implement != true {
        let cap = health.output.product_capacity;
        if cap == true && (product.skip_reason == () || product.skip_reason == "") {
            complete(#{
                ok: false,
                stage: "product-build",
                error: "ops.passive-wait: product_capacity true but did_implement false without skip_reason — edit console-drive ProductBuild prompts/gates",
                product: product,
            });
        }
    }
}

// --- 4 ProcessEdit (Bun: fix the process, not the anecdote) ---
phase("ProcessEdit");
let pe_prompt = "";
pe_prompt += "You are Console-drive ProcessEdit. Bun doctrine: edit reusable workflow/harness/tool.\n";
pe_prompt += "Classes from fleet:\n";
pe_prompt += json_encode(fleet.output.classes_hit);
pe_prompt += "\nProduct classes:\n";
pe_prompt += json_encode(product.classes_hit);
pe_prompt += "\nIf any class_id present OR fleet still has auth/prebind/behind issues:\n";
pe_prompt += "1) Map to failure-classes.v1.json\n";
pe_prompt += "2) Patch the CONTROL: prefer editing .grok/workflows/console-drive.rhai, pr-babysit.rhai, tools/ci/*, harness/*\n";
pe_prompt += "3) Do NOT leave only a chat note. If no code change needed because control already exists, edited_process=false with summary citing control path.\n";
pe_prompt += "If no classes and fleet clean: edited_process=false, class_ids=[], summary='no process edit needed'.\n";
pe_prompt += "Allowlist: .grok/**, tools/ci/**, package.json scripts only.\n";
let process_edit = agent(pe_prompt, #{
    label: "drive-process-edit",
    capability_mode: "all",
    output_schema: process_schema,
});
if process_edit == () || !process_edit.success {
    complete(#{ ok: false, stage: "process-edit", error: "process edit failed", product: product, fleet: fleet.output });
}

// --- 5 LearnProof ---
phase("LearnProof");
let learn_prompt = "";
learn_prompt += "Prove this drive was productive and non-silent.\n";
learn_prompt += "Run: `node tools/ci/ingest-soft-reds.mjs --check`\n";
learn_prompt += "Run: `.grok/bin/console-learn from-event --id console-drive --summary '…' --classes '…'`\n";
learn_prompt += "implementation_happened = fleet merged any OR product.did_implement OR process_edit.edited_process\n";
learn_prompt += "productive = implementation_happened OR (fleet actions non-empty with concrete restack/push)\n";
learn_prompt += "If only status narration: productive=false (ops.passive-wait).\n";
learn_prompt += "Fleet:\n";
learn_prompt += json_encode(fleet.output);
learn_prompt += "\nProduct:\n";
learn_prompt += json_encode(product);
learn_prompt += "\nProcessEdit:\n";
learn_prompt += json_encode(process_edit.output);
learn_prompt += "\nAppend ultragoal ledger via console-goal ledger-append if goal active.\n";
learn_prompt += "next_drive must be concrete `/workflow console-drive` or specific follow-up.\n";
let proof = agent(learn_prompt, #{
    label: "drive-proof",
    capability_mode: "all",
    output_schema: proof_schema,
});
if proof == () || !proof.success {
    complete(#{ ok: false, stage: "learn-proof", error: "proof failed" });
}
if proof.output.productive != true || proof.output.silence_check_ok != true {
    complete(#{
        ok: false,
        stage: "learn-proof",
        error: "drive unproductive or soft-red silence — fix process (console-drive.rhai / ingest tool)",
        proof: proof.output,
        fleet: fleet.output,
        product: product,
        process_edit: process_edit.output,
    });
}

complete(#{
    ok: true,
    workflow: "console-drive",
    doctrine: "bun-edit-the-process",
    health: health.output,
    fleet: fleet.output,
    product: product,
    process_edit: process_edit.output,
    proof: proof.output,
});
