// program-tick — thin orchestrator that FORCES the dual-track pair.
// Always plans: ci-fleet-tick + product-process-tick (unless args.skip_product true AND no waiting_ci).
// Does not merge. Does not implement. Prevents endless waiting-for-CI trap.
// Doctrine: .grok/programs/BUN-PARALLEL-DISCIPLINE.md

let meta = #{
    name: "program-tick",
    description: "Dual-track orchestrator: always pair CI fleet with product/process work (anti-wait)",
    when_to_use: "Session start and every autonomous wake; replaces single-track CI babysit",
    phases: [
        #{ title: "Fleet", detail: "ci-fleet-tick logic inline" },
        #{ title: "ProductProcess", detail: "product-process-tick when CI wait or always by default" },
        #{ title: "Synthesize", detail: "operator dual-track report" },
    ],
};

let skip_product = false;
if args != () && args.skip_product == true {
    skip_product = true;
}

phase("Fleet");
let fleet = agent(
    "You are the Console ci-fleet-tick inspector+classifier in one pass.\n"
        + "Run gh pr list --state open --limit 25 --json number,title,headRefName,mergeStateStatus,headRefOid,url.\n"
        + "For active PRs, check CI and Security on exact head (gh run list --branch …).\n"
        + "Classify: clean_merge_ready (leader merges), waiting_ci, failed(+urls), behind, blocked.\n"
        + "Set must_run_product_process_tick=true if any waiting_ci OR if nothing is clean and work remains.\n"
        + "next_fleet_action must NOT be bare 'wait'.\n"
        + "Do not merge. Do not edit files. Structured plain text report.\n",
    #{
        label: "tick-fleet",
        capability_mode: "execute",
    },
);
if fleet == () || !fleet.success {
    complete(#{ ok: false, stage: "fleet", error: "fleet leg failed" });
}

phase("ProductProcess");
let run_pp = true;
if skip_product == true {
    run_pp = false;
}
// Always run product/process when fleet output mentions waiting — agent should set flag; we force by default true
let pp = ();
if run_pp {
    let pp_prompt = "";
    pp_prompt += "You are product-process-tick. Fleet report:\n";
    pp_prompt += json_encode(fleet.output);
    pp_prompt += "\n\nWhile any PR is waiting_ci, you MUST select product or process work that does not need that tip writer.\n";
    pp_prompt += "Read docs/current/PRODUCT.md HOLDs + ROADMAP.md; run bd ready | head -30.\n";
    pp_prompt += "Pick track product|process|substrate; allowlist; next_commands; holds_checked.\n";
    pp_prompt += "not_waiting_on_ci must be true — pure WAIT is ops.passive-wait and fails this tick.\n";
    pp_prompt += "If process: cite failure-classes.v1.json class and recommend /workflow process-upgrade.\n";
    pp_prompt += "If product: recommend /workflow domain-increment with lane args sketch.\n";
    pp_prompt += "Plain text dispatch brief.\n";
    pp = agent(pp_prompt, #{
        label: "tick-product-process",
        capability_mode: "execute",
    });
    if pp == () || !pp.success {
        complete(#{
            ok: false,
            stage: "product-process",
            error: "product-process leg failed — do not fall back to CI-only wait",
            fleet: fleet.output,
        });
    }
}

phase("Synthesize");
let syn_prompt = "";
syn_prompt += "Synthesize dual-track operator report.\n\nFLEET:\n";
syn_prompt += json_encode(fleet.output);
syn_prompt += "\n\nPRODUCT/PROCESS:\n";
if pp != () && pp.success {
    syn_prompt += json_encode(pp.output);
} else {
    syn_prompt += "(skipped — only valid if skip_product and no waiting_ci)\n";
}
syn_prompt += "\n\nOutput format:\n";
syn_prompt += "1) Fleet: merge-ready / failed / behind / waiting_ci\n";
syn_prompt += "2) Required pair: workflows ci-fleet-tick + product-process-tick (this tick already did both if needed)\n";
syn_prompt += "3) Next human: leader merge list\n";
syn_prompt += "4) Next agent: exact workflow name + args\n";
syn_prompt += "5) Anti-passive attestation: what productive work happens while CI runs\n";
syn_prompt += "6) Tip-serial assessment: which open PRs touch documentation-manifest, ledger/, or executed-tests-baseline.json\n";
syn_prompt += "   List tip_writers count; if >=2 flag ops.tip-serial-contention and forbid opening another tip PR.\n";
syn_prompt += "7) Wall-tax: estimate if batching pure-test PRs would cut runner hours.\n";
syn_prompt += "8) One process improvement suggestion if any failure class repeated.\n";
syn_prompt += "Plain text.\n";
let syn = agent(syn_prompt, #{
    label: "tick-synthesize",
    capability_mode: "read-only",
});

complete(#{
    ok: true,
    dual_track: true,
    workflows: ["ci-fleet-tick", "product-process-tick"],
    fleet: fleet.output,
    product_process: if pp != () && pp.success { pp.output } else { () },
    report: if syn != () && syn.success { syn.output } else { () },
    note: "Endless waiting-for-CI is a process bug; this tick forbids CI-only completion.",
});
