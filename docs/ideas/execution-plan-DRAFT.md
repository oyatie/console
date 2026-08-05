# EXECUTION PLAN — from proven substrate to a demonstrable company

> **§0 REVERSED 2026-08-01. Everything outside §0 stands.** §0's verdict ("the register strip is
> DELETED, not deferred") and the `.gitattributes` remedy it recommended are both reversed:
> `.gitattributes` is deleted and the strip is built across two pull requests (expand, then
> contract). Neither is merged at the time of writing.
>
> §0's refutation was **correct on the mechanism** and was answered rather than ignored. The three
> `?.candidate_sha` comparisons were doubling as payload-existence checks; each site now carries an
> explicit `object(value, label)` check, asserted by `validate-console-truth-ledger.test.mjs`. The
> other half of the refutation — that the leaves catch a *partial* rebind — dissolved: there is no
> rebind step and no rebind tool left. The residual cost is a wholesale register revert validating
> clean; it is stated and bounded in
> `docs/program/ledger/2026-08-01-candidate-sha-leaves-the-registers.md`.

**Status: APPROVED 2026-07-28 — execution authorized.**
Date: 2026-07-28 · Mode: deliberate (`/ralplan --deliberate`) · **Rev 3**
Rev 1 → Architect: gate-weakening found. Rev 2 → Critic: **REJECT**, weakening survived the fix.
Rev 3 deletes the change rather than defending it a fourth time.

---

## 0. The decision, and why it reversed

**The register strip is DELETED, not deferred.** Four rounds of evidence killed it, each after I had
declared it proven:

| round | claim | how it died |
|---|---|---|
| 1 | "390 bindings are a tautology; delete them" | codex: a *partial rebind* is caught. Verified — `.test.mjs:45-47` |
| 2 | "safe — 11/11 red-first controls" | Architect: `delete control.candidate_evidence` → **ACCEPTS**. `:297`'s `?.` was doubling as an existence check. Reproduced independently |
| 3 | "safe with 5 preconditions" | Critic: the same class survives at `:297` trace and `:301` binding — **324 of 357 leaves (91%)**. I hardened the 6-occurrence site and left the 324-occurrence site open |
| 4 | "retain `independent_outcome_review.candidate_sha` ×27" | **that field exists on 0 of 27 capabilities.** I invented a count from codex's phrasing and never checked it |

Two further self-inflicted errors, both found by my own measurement:
- **I corrected a correct number.** `390` was right — it was the count stated in the header comment of
  `scripts/console/rebind-authority-train.mjs`, a file this train deletes, so the citation is
  historical and no longer resolvable in the tree; `426` counts all 40-hex strings, a different
  quantity. The `candidate_sha` key occurs 358 times.
- **The absence-ratchet I proposed cannot go green** — `registry.source_inventory.candidate_sha`
  legitimately carries that key, so an unscoped assertion rejects the clean baseline.

**Root cause, unchanged across all four rounds:** enumerating what a line *semantically asserts*
instead of what it *structurally does*. Identical to the header-comment failure that killed the
fan-out premise. Four rounds is enough evidence that the enumeration method is the defect, not any
particular enumeration.

### What replaces it — one line, no risk

The entire objective was reviewable authority diffs. Measured: **780 of 780 changed lines contain a
SHA; 0 are semantic.**

```gitattributes
docs/program/console-capability-registry.json   linguist-generated=true
docs/program/console-jurisdiction-register.json linguist-generated=true
```

GitHub collapses both files to one expandable line in the PR diff — where the adversarial review this
program depends on actually happens. Verified: the repo has **no `.gitattributes` today**, and no
script textually diffs the registers (all use `git show` or `git diff --raw`), so CI is unaffected.

| approach | reviewable diff | works where | risk |
|---|---|---|---|
| today | 891 lines | — | — |
| local pathspec | 111 lines | **local only** — not GitHub | none |
| the strip | ~66 residual lines | everywhere | **gate-weakening ×3 sites** |
| **`linguist-generated`** | **collapsed to 1 line, expandable** | local **and** GitHub | **none** |

It beats the strip on its own metric, and avoids the trade the strip forced: the strip moved the
invariant from **signature-covered data** to **freely-editable code in C**
(`validate-console-truth-ledger.mjs` is not an authority document and is not signature-bound). Round 2
was a mistake made while writing exactly that replacement code.

## 0.1 Evidence base

Every row produced by running something. Retractions marked.

| Claim | Result |
|---|---|
| Lanes isolate under concurrency | 4 codex lanes, 37s, **0 of 4** out-of-slice, 4/4 correct by content |
| The harness works | `fanout.py run`, 2 lanes, ~19s, 0 out-of-slice |
| The probe guard works | both polarities — a probe that passes on bad input is **rejected, exit 1** |
| Churn is pure noise | **780/780** changed lines contain a SHA; **0** semantic |
| Registers have no `.gitattributes` | verified; `linguist-generated` is a clean addition |
| No script textually diffs the registers | all use `git show` / `git diff --raw` |
| ~~Provenance reduction is safe~~ | **RETRACTED** — gate-weakening at 3 sites, 91% of leaves |
| ~~426 occurrences~~ | **RETRACTED** — 390 was correct; 426 counts all 40-hex strings |
| ~~retain `independent_outcome_review.candidate_sha` ×27~~ | **RETRACTED** — 0 of 27 have it |
| `rebind-candidate.test.mjs` runs nowhere | **dead test**; preflight runs exactly 4 (`ci.yml:137,140,143,146`) |
| `route_keys = 0` across all 27 capabilities | the conformance target genuinely cannot move |
| REST surface is type-agnostic | `{key}`, `{id}`, `{action_key}` — a new Instance type needs zero routes |
| Batch 3 harness exists | 48 `*_as_runtime_role.rs` suites under `backend/` |
| Gate-crate cost anchor | `ci/gates/rls-arming` = 340 + 40 lines |
| No ordered multi-version install array exists | `seed.rs:68, 1259-1273` — single version, single manifest |

## 1. Principles

1. **Prove by execution before planning on it** — and enumerate what a line *does*, not what it says.
2. **Parallelize only what is structurally disjoint**, demonstrated by a dry run.
3. **Never repair a gate by weakening it; never move the target.**
4. **Take the highest rung that works.** A tooling problem is not a data problem.
5. **Scope is load-bearing.** Org, employee, HR, payroll.

## 2. Decision drivers

1. **The substrate is proven and idle** — isolation, harness and delegation all work. The only blocker
   is that no immutable target exists.
2. **Authority diffs are unreviewable at 780 lines of noise per PR**, and that is fixable with one
   line of `.gitattributes` at zero risk.
3. **Fan-out cannot start without a conformance suite**, which does not exist and is the long pole.

## 3. Options

### Option A — strip the registers *(rejected, four times)*
Gate-weakening at three sites covering 91% of the leaves; moves the invariant off the signed side;
requires 5 collateral files, a hand-edited ledger in T, and a ratchet that cannot go green.

### Option B — `linguist-generated` + go straight to the product *(recommended)*
One 2-line file. Delivers more readability than the strip, at no risk, with no train.

### Option C — do nothing about readability
Defensible — reviewers already skip the noise — but the one-line fix is cheaper than the argument.

## 4. Plan

### Batch 1 · Now, zero risk, no train
- **`.gitattributes`** with the two `linguist-generated` lines.
- **Probe-red-first into the lane contract.** `fanout.py verify` already enforces it, both polarities
  proven. This addresses the failure mode that has cost this session the most: three broken probes,
  each of which could only have returned GREEN.
- **`FACTS.generated.md`** — *only* if it has a named required-check host. Restricted to
  tree-derivable counts (migrations, workspace members, capabilities, controls). **Excluded:**
  `include_str!` line numbers (they move on unrelated edits — pinning them reintroduces the churn this
  removes), branch-protection settings (GitHub API inside a required check), worktree count
  (machine-local). **Must land in C** — `verify-console-authority-train.mjs:28` rejects any status but
  `M` in T. *If no required-check host is identified, drop it: an unenforced gate is the false-green
  class fixed in #506.*

### Batch 2 · Phase 0 pre-reservation (one commit)
Pre-reserve the five `mod` lines and per-lane migration blocks. **The ordered-install-array half moves
to Batch 4** — that array does not exist (`seed.rs:68, 1259-1273`), and Rev 1 dropped that caveat when
compressing Phase 0.

### Batch 3 · Conformance suite — the immutable target *(serial, blocking)*
Scenario logic once; two drivers, both against surfaces that exist today (generic REST + store action
dispatch). Cheaper than it looks: 48 `*_as_runtime_role.rs` suites already exist and `route_keys = 0`
means the target cannot move. Owned outside the lanes; fixtures lane-addable.
**Deliverable, not a follow-up: the per-phase expected-red ledger.** The suite is red for the entire
program ("run a pay cycle" cannot pass until Lane 4), so without it nothing distinguishes correct red
from regression on any later PR.
**Red for the right reason:** positive control (`customer` → 200) alongside `org_unit` → the specific
unknown-type error, same run.

### Batch 4 · OrgUnit end to end — the long pole
Carries OrgUnit (registry + instance store + event log + effective dating + action dispatch + Cedar
authorize/residual + audit + as-of), **the three CI gates** (~380 lines each), and **the ordered
multi-version bootstrap** Batch 2 depends on. 1 implementer + 2 adversarial reviewers, cross-family.

### Batch 5 · Fan-out
Only after Batch 3 is green on Batch 4 and `fanout.py run` reports zero out-of-slice on a two-lane
real dry run.

## 5. Pre-mortem (deliberate)

**S1 — Batch 1's facts gate becomes a false-green.** If `FACTS.generated.md` has no required-check
host it gates nothing while looking like a gate. *Mitigation:* the batch item is conditional on naming
the host; drop it otherwise. *Early warning:* the gate never goes red when a count is hand-edited.

**S2 — The conformance suite is written against a surface that then moves.** *Mitigation:* both
drivers bind to surfaces that exist today; criterion 6 is split, with residual row-filtering (6b)
explicitly deferred to L-WIRE. *Early warning:* any change to `ontology/rest/src/lib.rs:1645-1648`.

**S3 — The strip returns.** It is seductive: the tautology argument is *partly* true, which is why it
survived three refutations. *Mitigation:* this document records that it weakens the gate at three
sites and that `linguist-generated` beats it on its own metric. *Early warning:* anyone citing "390
tautological bindings" without citing `:297`/`:301`.

## 6. Test plan (expanded — deliberate)

- **Unit:** per-crate; domain logic DB-free where possible.
- **Integration:** `#[sqlx::test]` on disposable Postgres (always `--rm`), asserted **as the
  non-superuser runtime role**, `--test-threads=1`.
- **Conformance:** Batch 3 scenarios through both drivers — the definition of done — plus the
  expected-red ledger.
- **Authorization:** 6a at the REST door (denial without `RoleManage`; armed vs unarmed
  `app.current_org`). 6b deferred with L-WIRE named.
- **Temporal:** as-of reconstruction diffed against the event log.
- **Contract:** `openapi_drift` (`ci.yml:597`) — no new routes expected, so it should not move.
- **Observability:** lanes write findings to disk as they go; `fanout.py run` emits per-lane
  out-of-slice counts, so "stuck" and "died" are distinguishable.
- **Rollback:** every batch states its revert before it lands. Batch 1's is deleting a 2-line file.

## 7. Acceptance criteria

1. `.gitattributes` lands; the next authority PR shows both registers collapsed on GitHub.
2. The lane contract requires probe-red-first, and `fanout.py verify` rejects a probe that passes on a
   known-bad input.
3. `FACTS.generated.md` either has a named required-check host and goes red on a hand-edited count, or
   is not shipped.
4. Batch 3 fails red **for the right reason**, with a passing positive control in the same run, and
   publishes the expected-red ledger.
5. Batch 4 passes its named assertion ids through both drivers, reviewed cross-family.
6. `fanout.py run` reports zero out-of-slice across a two-lane real dry run.
7. A non-privileged principal is denied at the REST door, and unarmed `app.current_org` is caught (6a).

## 8. ADR

- **Decision:** Do **not** strip the registers. Add `linguist-generated=true`, adopt probe-red-first,
  and go straight to the conformance suite and OrgUnit. Fan-out last.
- **Drivers:** the parallel substrate is proven and idle; authority diffs are unreviewable and fixable
  with one line; the conformance suite is the real blocker.
- **Alternatives:** the strip — rejected four times, gate-weakening at three sites covering 91% of
  leaves, and it moves the invariant off the signed side; doing nothing — defensible but the fix is
  cheaper than the argument.
- **Consequences:** the registers stay verbose but signature-covered and collapsed in review;
  `rebind-authority-train.mjs` keeps rewriting the bindings every cycle — mechanical, automated, and
  no longer in anyone's way.
  *(2026-08-01: reversed. Both rebind tools and `.gitattributes` are deleted; a routine authority
  commit now changes zero lines in either register.)*
- **Follow-ups:** release-signing or branch-exemption (the ledger's own named throughput fix, still
  unbuilt — **this is the real throughput item, not the strip**); `rebind-candidate.test.mjs` is a dead
  test and needs wiring or deleting *(2026-08-01: deleted with its tool)*;
  `rebind-authority-train.mjs:43-49` can emit a 2-file T that CI rejects *(2026-08-01: moot — the
  tool is gone and a 1-document T is now admissible)*; `source_revision` ancestry under squash;
  L-WIRE charter for 6b.
