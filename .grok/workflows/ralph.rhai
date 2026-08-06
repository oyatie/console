// ralph — PRD story loop until all passes + independent review APPROVE.
// Ideas from OMC ralph ("the boulder never stops"). Grok-native.
// Soft reds remain on board. Merge via autonomy-merge policy after APPROVE.

let meta = #{
    name: "ralph",
    description: "PRD-driven execution loop: story-by-story until passes + reviewer APPROVE; hooks keep Stop looping while active",
    when_to_use: "After ralplan APPROVE or well-specified task; pairs with ultragoal active-goal",
    phases: [
        #{ title: "Prd", detail: "ensure task-specific acceptance criteria" },
        #{ title: "Execute", detail: "implement next failing story" },
        #{ title: "Verify", detail: "criteria + tests" },
        #{ title: "Review", detail: "independent APPROVE gate" },
        #{ title: "Learn", detail: "Hermes-style console-learn + progress.txt" },
    ],
};

let review_schema = #{
    "type": "object",
    "required": ["verdict", "summary", "must_fix"],
    "properties": #{
        "verdict": #{ "type": "string" },
        "summary": #{ "type": "string" },
        "must_fix": #{ "type": "array", "items": #{ "type": "string" } },
    },
};

phase("Prd");
let prd_prompt = "";
prd_prompt += "Ralph PRD setup. Read .grok/ultragoal/prd.live.json and active-goal.live.json.\n";
prd_prompt += "If stories empty or acceptanceCriteria are generic ('implementation complete'), REFINE into task-specific criteria.\n";
prd_prompt += "If args.task present, build/refine stories from it.\n";
if args != () && args.task != () {
    prd_prompt += "Task: ";
    prd_prompt += args.task;
    prd_prompt += "\n";
}
prd_prompt += "Write prd.live.json. Ensure console-goal activate --workflow ralph if ultragoal should loop Stop hooks.\n";
prd_prompt += "Append progress.txt header for this run.\n";
prd_prompt += "No PRODUCT HOLD clearance. Tip-serial: do not open tip thrash PRs if tip_writers>=2 unless serial head.\n";
let prd = agent(prd_prompt, #{
    label: "ralph-prd",
    capability_mode: "all",
});
if prd == () || !prd.success {
    complete(#{ ok: false, stage: "prd", error: "prd setup failed" });
}

phase("Execute");
let ex_prompt = "";
ex_prompt += "Pick highest-priority story with passes!=true from prd.live.json. Implement it (worktree ok).\n";
ex_prompt += "PRD context:\n";
ex_prompt += json_encode(prd.output);
ex_prompt += "\nRun tests for touched crates. Open/update PR if needed. Do not merge here.\n";
ex_prompt += "If blocked by soft red on another PR, enqueue board item and still progress this story if path-disjoint.\n";
let executed = agent(ex_prompt, #{
    label: "ralph-execute",
    capability_mode: "all",
    isolation_worktree: true,
});
if executed == () || !executed.success {
    complete(#{ ok: false, stage: "execute", error: "execute failed", prd: prd.output });
}

phase("Verify");
let v_prompt = "";
v_prompt += "Verify EACH acceptance criterion for the story with fresh evidence (commands/output).\n";
v_prompt += "Execute output:\n";
v_prompt += json_encode(executed.output);
v_prompt += "\nIf all met, set passes:true on that story in prd.live.json and append progress.txt learnings.\n";
v_prompt += "If not met, leave passes false and list gaps — do not claim done.\n";
let verified = agent(v_prompt, #{
    label: "ralph-verify",
    capability_mode: "all",
});
if verified == () || !verified.success {
    complete(#{ ok: false, stage: "verify", error: "verify failed" });
}

phase("Review");
let r_prompt = "";
r_prompt += "Independent reviewer against SPECIFIC prd acceptance criteria (not vague done).\n";
r_prompt += "verdict MUST be APPROVE or CHANGES_REQUESTED.\n";
r_prompt += "Verify:\n";
r_prompt += json_encode(verified.output);
r_prompt += "\nIf all stories passes:true and APPROVE: say ready_for_babysit_merge=true.\n";
r_prompt += "If not all stories complete: say continue_ralph=true (boulder never stops).\n";
let reviewed = agent(r_prompt, #{
    label: "ralph-review",
    capability_mode: "execute",
    output_schema: review_schema,
});
if reviewed == () || !reviewed.success {
    complete(#{ ok: false, stage: "review", error: "review failed" });
}

phase("Learn");
let l_prompt = "";
l_prompt += "Hermes-style learn. Run:\n";
l_prompt += "`.grok/bin/console-learn from-event --id ralph-$(date +%Y%m%d) --summary '…' --classes '…'`\n";
l_prompt += "If review not APPROVE, class ops.review-not-approve-stall.\n";
l_prompt += "Review:\n";
l_prompt += json_encode(reviewed.output);
l_prompt += "\nRecommend next: /workflow ralph (continue) or /workflow pr-babysit (merge path) or /workflow program-control.\n";
l_prompt += "Do not deactivate ultragoal unless all stories pass AND quality gate met.\n";
let learned = agent(l_prompt, #{
    label: "ralph-learn",
    capability_mode: "all",
});

complete(#{
    ok: true,
    workflow: "ralph",
    review: reviewed.output,
    verify: verified.output,
    learn: if learned != () && learned.success { learned.output } else { () },
    boulder: "continue until prd all passes + APPROVE + Required green merge",
});
