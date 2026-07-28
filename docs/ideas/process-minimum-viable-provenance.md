# Minimum Viable Provenance

**Status: SUPERSEDED 2026-07-28 by `execution-plan-DRAFT.md` §0. NOT authorized for execution.**

> This document records rounds 1–3 of the register-strip investigation. Its final position
> ("Keep 2 of 390, add one assertion line") was **refuted in round 4**: the surviving
> `?.candidate_sha` comparisons at `validate-console-truth-ledger.mjs:297` and `:301` double as
> payload-existence checks, so stripping weakens the merge gate at three sites covering **324 of 357
> leaves (91%)**. The strip is **deleted, not reduced**. `.gitattributes linguist-generated=true`
> delivers the entire benefit at zero risk.
>
> Kept because the reasoning chain — and four rounds of being confidently wrong — is the reusable
> lesson.
Date: 2026-07-28 · From `/idea-refine`, grounded in measurements taken this session.

## Problem Statement

**How might we keep signature-verified provenance and adversarial verification, while deleting every
artifact whose only reader is the thing that wrote it?**

## The finding that drives this

The capability and jurisdiction registers carry **~390 `candidate_sha` bindings**. Their production
readers are exactly two: `rebind-candidate.mjs`, which *writes* them, and
`validate-console-truth-ledger.mjs`, which asserts they all equal `candidate.sha` — the value they
were copied from.

**The validator's job is to verify that 390 copies of a value equal the value they were copied
from.** It is a tautology. It cannot fail unless the rebind tool has a bug, and it carries no
information about the world.

Worse, the repo is **squash-only**, so the commit those 390 references name is destroyed by the merge
they authorize. Meanwhile `bind-merged-console-authority-squash` already runs on close, binds the
**surviving** squash SHA, and emits a receipt — verified green on #506, #507 and #508 today.

Cost of the tautology, measured: **3 train rebuilds in one session**, O(N²) across lanes (every merge
invalidates every other lane's bindings), and the program ledger blames it for **four consecutive
releases losing verified work**. `pipeline-ceremony-bottleneck` measured 3:1 docs-to-code, 12:1 in
integration sessions.

Confirmed with the owner: this provenance is **self-imposed engineering discipline**, not an external
audit obligation. Nothing outside this repo reads these registers.

## ⚠️ Rev 1 was REFUTED by adversarial challenge — read §Consensus below

**Verdict from cross-family challenge (codex `gpt-5.6-sol`, read-only): BLOCK as written.**
Independently verified before acceptance.

**The checks are NOT a tautology.** A signed T can contain a **partial rebind** — top-level
`candidate.sha` correct while *one* leaf stays stale. The validator catches exactly that, and there
is an explicit regression test for it: `validate-console-truth-ledger.test.mjs:45-47` corrupts a
single capability's `candidate_evidence.candidate_sha` and asserts `/candidate-bound/` throws.
Verified by reading it. The refuting sentence:

> *"Signatures authenticate inconsistent bytes; they do not make them semantically consistent."*

Rev 1's "390 copies of one value" framing assumed the failure mode was *all copies wrong*. The real
failure mode is **one copy stale among 389 correct ones**, which a tree signature cannot detect.

Two further corrections it established:
- **Promotion bindings ≠ HOLD-row duplicates.** A non-HOLD review's `candidate_sha` originates in a
  *separate immutable signed review receipt* and is compared against C and both authority-document
  digests (`validate-console-truth-ledger.mjs:164, :182`). That is genuine **cross-candidate replay
  protection**, not a copy.
- **Raw deletion stops fan-out immediately** — `plan-fanout.mjs:294, :577` require the validator's
  mutation-sensitive attestation.

What it **conceded**, which reframes the question: the bindings add **no cryptographic** protection.
T's signature binds the registry tree, T must descend directly from C, only authority documents may
change (`verify-console-authority-train.mjs:35`), and the squash check requires S's tree to equal T
exactly (`verify-console-pr-authority-bootstrap.mjs:92`). Their value is narrower than the design
implies: **detecting a mistake by our own rebind tool inside an otherwise authentic T.**

## Consensus (both challengers in, disagreement resolved)

**codex `gpt-5.6-sol`: BLOCK as written. Claude critic: SOUND-WITH-CHANGES.** They disagreed on the
central question, which is precisely why the challenge was run cross-family.

**Resolution — both are right about different things.** codex proved a *corrupted* leaf is caught
(`.test.mjs:45-47`). Claude proved a *deleted* leaf loses nothing, by stripping both registers and
re-running every real check: the bijection, TOCTOU attestation, exposure and HOLD gates all still
fail correctly. The bijection's SHA component **cancels** — `.mjs:297` already forces every
`trace.candidate_sha === candidate.sha`, making it a constant on both sides of the set comparison.
A leaf whose only job is detecting its own staleness takes both the risk and the check with it.
Claude's evidence is stronger because it tested the actual proposal rather than a corruption.

### Two corrections to Rev 1's *case* — both mine, both material

**1. The cost attribution was backwards.** Rev 1 claimed the ledger blames the authority train for
four consecutive releases losing work. **It blames the opposite.** The ledger says release-please
emits an *unsigned bot commit* that the train rejects, forcing a hand-rebuild that opens the
merge-timing window — and names the only two fixes: *"Either release commits are signed at source, or
the release branch pattern is exempted from the train by policy; nothing else removes the
hand-rebuild."* MVP keeps the signature requirement, so **deleting bindings does not fix this at
all.** The economic case did not survive its own evidence.

**2. The O(N²) throughput win is not delivered either.** Rebinding is O(N²) because the candidate SHA
changes on every merge — driven by `registry.candidate.sha`, `source_revision`, and the C→T train,
**all of which this proposal keeps**. Whether a rebind rewrites 1 field or 390 does not change the
number of rebind *cycles*, and lanes still collide on the same lines. The ledger already says so:
*"It does not dissolve the shared-file mutex that serializes concurrent lanes on these documents; it
removes the manual step."*

**The real, measured win is churn, and it is large:** per authority commit (#506/#507/#508) the
capability registry changes **440 lines** and the jurisdiction register **340**, of which **zero are
non-SHA**. ~780 lines of pure noise per PR, dropping to ~6. Worth doing for reviewability. **Do not
book it as throughput.**

### One defect that would have blocked every merge

`verify-console-authority-train.mjs:31` fails closed unless C..T modifies **all three** authority
documents. Rev 1's *"keep one `candidate.sha`"* is ambiguous, and the one-total reading leaves the
jurisdiction register with **no per-candidate content to change** — measured: *"per-candidate fields
left in jurisdiction register: NONE → C..T 3-file requirement unsatisfiable."* Every PR would fail
the required `authenticate-console-authority` check on a repo with `enforce_admins`.

**Keep two — one per register.**

### A gap deletion would have opened

Measured on the post-deletion validator: a jurisdiction register left stale by a bad merge or
cherry-pick — same control IDs, same trace set, stale `freshness.status`/`unhold_authority` — **PASSES**
after deletion. Today it fails because the stale file carries an older SHA. Closed by one line:
`if (jurisdiction.candidate.sha !== candidate.sha) fail('jurisdiction register is not candidate-bound')`.
**One field replaces 169.** Blast radius was bounded anyway — `.mjs:294` forces every control to
`release_disposition: HOLD` — but the fix is free since finding #1 already requires keeping that field.

### The argument for doing it now that Rev 1 never made

All 27 capabilities are `HOLD` with **zero review receipts in existence**, so the
`promotionAuthorityDigests` re-minting cost of changing register content is **zero today, and rises
the moment the first review lands.**

### Agreed disposition

**Keep 2 of 390. Add one assertion line.** Removes 388 copies and ~99% of the churn, keeps the C→T
train satisfiable, keeps `rebind-candidate.mjs` working unchanged (verified empirically), and closes
the stale-register gap.

Also in scope, previously unbudgeted: `rebind-authority-train.mjs` (a second writer),
`docs/evidence/console/wave4/manifests/verify-new-rows.mjs`, and two CI-gating test files
(`validate-console-truth-ledger.test.mjs`, `rebind-candidate.test.mjs`) that assert on binding shape
and run in the required `preflight` job. Changes must land **atomically** — C carrying the new
validator, T the stripped registers — or CI is red mid-flight.

**Explicitly excluded from deletion:** `benchmark.independent_outcome_review.candidate_sha` — dark
today at 27/27 HOLD, but it is what stops a reviewer's MEET verdict being replayed onto a different
candidate the moment the first review lands.

**Method note:** a same-family reviewer would likely have shared the "signature implies consistency"
blind spot that produced Rev 1. The disagreement between challengers *was* the value.

---

### Superseded interim reasoning (kept for the record)

Both opening positions were partly wrong, and the deciding fact belongs to neither.

**`rebind-authority-train.mjs:109-110` already scans both registers and fails if *any* stale SHA
remains.** The partial-rebind class codex defends is already prevented at the source. The 390 stored
copies are a second, far more expensive way to detect what one whole-file scan already prevents.

**Agreed direction — replace 390 stored copies with one derived invariant, retain what carries
independent information:**

| | Disposition | Why |
|---|---|---|
| HOLD-row duplicate `candidate_sha` leaves | **replace** with a single scan invariant: *no SHA other than `candidate.sha` appears in either register* | catches the identical partial-rebind class in one check; no per-leaf storage semantics; O(1) not O(N) per lane |
| immutable-review `candidate_sha` (non-HOLD promotions) | **retain** | separate signed receipt, cross-candidate replay protection — not a copy of anything |
| meaningful `source_sha` / `contract.source_sha` | **retain** | asserted nonempty, not equal (`:232`) — carries real content |
| C→T shape + both SSH signatures | **retain** | the actual provenance anchor |
| post-merge squash receipt | **retain** | binds the surviving commit |
| per-lane rebind (the O(N²) cost) | **delete** | one rebind per *batch*, not per lane — this was always the real cost, and it is untouched by the above |

Net effect: the same defect classes remain detectable, the per-lane serialization that caps
parallelism is gone, and the security properties codex identified are preserved rather than traded.

**Method note worth keeping:** this consensus exists because the challenge was run **cross-family**.
A same-family reviewer would likely have shared the "signature implies consistency" blind spot that
produced Rev 1.

## ~~Recommended Direction (Rev 1 — superseded)~~

**Delete the 390 bindings. Keep the signature.**

Provenance is already delivered, in full, by three things that survive the merge: the **SSH signature
on C and T**, the **post-merge squash receipt**, and **adversarial diff-only review**. The bindings
add no evidence on top of these — they only add serialization.

What an auditor would actually ask is *who signed this, what changed, who reviewed it, what test
proved it*. All four are answerable from git and CI without a single `candidate_sha` copy.

The second change follows from the same principle: **facts that can be derived must never be
restated.** Migration count, `include_str!` line numbers, worktree count and crate count were
simultaneously wrong in three planning documents — eleven corrections. Hand-maintained numbers drift
faster than anyone re-reads them.

The third is a harness rule rather than a document: **a probe must prove RED on a known-bad input
before its GREEN is trusted.** My own verification scripts broke three times this session while the
code under test was correct — `.length` on an object, `root//pkg:name` vs `//pkg:name`, zsh
1-indexed arrays. Each would have produced a confident wrong conclusion. This is too reliable a
failure to leave to discipline.

## Key Assumptions to Validate

- [ ] **Nothing outside the two scripts reads the bindings.** *Test:* delete them on a branch; run the
      full CI suite plus `plan-fanout` and the truth-ledger validator. Anything that breaks is a
      reader I missed.
- [ ] **The squash receipt alone satisfies the provenance goal.** *Test:* take one merged PR and
      reconstruct — signer, diff, reviewers, proving test — using only git and the receipt. If any of
      the four is unanswerable, the bindings were load-bearing after all.
- [ ] **Removing the rebind actually unlocks throughput.** *Test:* run two lanes to a real merge with
      the rebind removed and compare wall-clock to today's serialized landing. The 4-lane isolation
      experiment (37s, zero out-of-slice writes) proves the lanes work; it does **not** prove the
      landing does.
- [ ] **A generated facts block stays green.** *Test:* land `FACTS.generated.md` with a
      regenerate-and-diff gate, then deliberately change a migration count and confirm CI goes red.

## MVP Scope

**In:**
1. Delete `jurisdiction_bindings[].candidate_sha`, `candidate_evidence.*_sha`, and the per-capability
   SHA copies. Keep **one** `candidate.sha` and the signature checks.
2. Keep the C→T shape check and both SSH signature verifications — those are the real anchor.
3. Keep `bind-merged-console-authority-squash` as the post-merge provenance record.
4. `docs/program/FACTS.generated.md` emitted from the tree; CI regenerates and `git diff --exit-code`.
5. Lane contract gains one required step: **probe proves RED on a known-bad input, and both results
   are reported.**

**Out of MVP:** the bijection/traceability check between capabilities and Korea controls stays for
now — it asserts something real (completeness), unlike the SHA copies. Revisit separately.

## Not Doing (and Why)

- **Deriving capability state from conformance results** (`VERIFIED` iff its slice is green) — the
  right end state, and genuinely better than a hand-maintained field. But no conformance suite exists
  yet, so it would be building on nothing. Revisit once phase 1 lands.
- **Enabling merge commits so C survives** — would make the bindings meaningful, but changes history
  shape for every consumer and the squash receipt already solves the problem. Wrong direction.
- **Deleting the program ledger** — it is the one artifact with a human reader and real narrative
  value; it is where this session's reasoning errors are recorded so they are not repeated. Keep.
- **Rewriting the registers into a database** — moves the problem without shrinking it.
- **Touching the Korea control traceability** — it asserts completeness, not a tautology. Different
  question, different evidence.

## Open Questions

- Does `plan-fanout.mjs`'s `assertSourceRevision` depend on a binding we plan to remove? It already
  fails when `source_revision` is not an ancestor — and squash orphans that too.
- Should `rebind-authority-train.mjs` be deleted with the bindings, or retained for the single
  remaining `candidate.sha`? A one-reference rebind may not justify a 110-line tool.
- The registers are `include_str!`-adjacent authority documents in CI's required checks. Which of the
  12 required contexts actually depend on binding content versus register *shape*?
