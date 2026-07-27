# Wave-4 evidence base

Committed by **L-P0-EPOCH**, the wave's admission lane, on 2026-07-25.
Every other wave-4 lane depends on what is in here.

Wave-4 baseline HEAD: `4cabe239673e132765a003de04fd9dce5a86bfe2`
(`test(notif): assert the ADR-0025 invariant, not an exposure snapshot`), which is
also the tip of `origin/codex/operational-object-runtime-progress`.

---

## 1. The epoch contract is amended — plain-merge train

**This is the line that makes wave 4 admissible at all.**

`docs/program/console-fanout-epoch-contract.md:115-118` admitted only *"a serialized
rebase/cherry-pick admission train"*. **Rebase is classifier-blocked on this spine**, so
under the contract as written every fan-out lane was formally inadmissible.

Per founder decision **D-2** the clause is **amended to a plain-merge train**: a lane
integrates with `git merge origin/<spine>` before it pushes. This is what every
successful landing in this program has actually used — wave-1, wave-2/3, and the openapi
integration.

- The **rebase/cherry-pick clause is struck** — not carried forward, not escalated.
- **No lane may rebase. No lane may cherry-pick.**
- Serialization, receipt-only review commits, and the integrator-manifest rule for shared
  roots (openapi, clients, migrations, route/nav, tokens, generated Buck faces) are all
  **unchanged**.

The exact replacement text is in
[`manifests/epoch-contract-plain-merge.json`](manifests/epoch-contract-plain-merge.json),
anchored to the pre-edit file digest. The contract file itself is integrator-owned and
outside this lane's roots, so the edit is handed over as a manifest rather than applied
here.

## 2. What is in this directory

| Path | What it is |
|---|---|
| [`backend-blocked-index.json`](backend-blocked-index.json) | **The wave's main honesty artifact.** 69 machine-readable rows, one per backend-blocked fidelity finding. |
| [`manifests/`](manifests) | Edits to integrator-owned files that this lane may not apply itself, plus the harness that proves them. |
| [`inputs/`](inputs) | The frozen decision, charter, lens, scout and research inputs, so every DoD anchor is a repo path instead of a dead scratchpad path. |

### `inputs/`

| File | Role |
|---|---|
| `DECISIONS.md` | Founder decisions D-0…D-6 and standing risks R-1/R-2. **Binding.** |
| `WAVE4-CHARTER-DEPTH.md` | The operative charter: 19 lanes, 5 Phase-0 + 14 CRM. |
| `WAVE4-CHARTER.md` | The superseded 60-lane charter. Its §4 collision map, §6 DoD template and §7 truthfulness doctrine remain normative and are cited, not restated. |
| `charter-lens-{A,B,C,D}.md` | Substrate · fidelity floor · experience depth · business-logic depth. |
| `design-intent-register.md` | Distilled design intent; CRM-1…CRM-6 and WFL-9 are the operative `[>190]` set for this wave. |
| `north-star-amendment-beyond-prototype.md` | Source of record for the doctrine appended to `docs/intent/console-north-star.md`. |
| `fidelity-registers.json` | The 15-module adversarial fidelity audit: 180 findings, 69 backend-blocked. Source of the index. |
| `research-statutory-params.md` | Statutory parameter research. Any rule with yearly/regulatory parameters is re-verified from a live authoritative source at build time and appended here, producing a reviewable diff. |
| `research-depth-patterns-{backend,frontend}.md` | Depth patterns the lanes are held to. |
| `scout-{ontology-engine,shared-grammar,shared-grammar-b,spine-delta}.md` | Pre-wave scout briefs. |
| `build-backend-blocked-index.py` | Generator for the index. Row bodies come from `fidelity-registers.json`; only the classification is hand-authored, and it lives here where it can be reviewed against the register text. |
| `patch-capability-registry.py` | The additive registry patch, which asserts every pre-existing capability row is unchanged before it writes. |

## 3. `backend-blocked-index.json` — read this before believing any coverage claim

Depth-first (D-0) takes CRM to production depth by **not touching thirteen modules**.
The ledger of what is therefore *not* being done is the wave's main honesty artifact and
the integrator's completion checklist.

69 rows — 6 blocker, 44 major, 19 minor. 30 are CRM-relevant.

Each row carries the verbatim `design_says` / `code_does` / `fix` from the audit, the
owning **backend surface**, and a `wave4_disposition`:

| Disposition | Rows | Meaning |
|---|---|---|
| `deferred_no_wave4_lane` | 39 | No wave-4 lane touches this at all. |
| `gap_manifested_only` | 13 | A CRM lane names the same gap with anchors but does not close it. |
| `pattern_proven_by_wave4_crm` | 11 | Wave 4 builds the same primitive for sales; this row still needs its own contract. |
| `unblocked_by_wave4_registration` | 4 | **No endpoint is missing.** `instance_acting` / `object_type_acting` are already live routes (`backend/crates/ontology/rest/src/lib.rs:1562-1588`); the module needs type *registration*. L-X7 proves the path. |
| `unblocked_by_wave4_exposure` | 2 | D-1 exposing `sales` gives the row its first reachable target. |

`wave4_owning_lane` is `null` on all 69 rows, recorded explicitly rather than left blank:
depth-first assigns no lane to the thirteen undeepened modules.

**Four rows a reviewer should look at first,** because they are defects rather than depth
gaps and they are being deferred: `recruiting.findings[0]` (an OFFER-stage pool applicant
has no primary CTA anywhere — the pipeline dead-ends), `directory.findings[0]` (a
directory that renders no contact channel at all), `docs.findings[0]` (the module ships
as the EV- archive only), and `logistics.findings[6]` (a write-only pilot router, so every
queue starts empty each session).

## 4. Migration slots

Slot numbers come from [`../../../program/wave4-migration-slot-ledger.md`](../../../program/wave4-migration-slot-ledger.md).
A lane **requests**; the integrator **appends and assigns**. A lane never picks its own by
listing the directory — the ledger records `0197` as a live duplicate across three
unmerged branches and twelve more slots holding two distinct subjects each.

## 5. Registry state, stated plainly

`docs/program/console-capability-registry.json` gained `CAP-SALES-CRM` and
`CAP-ONTOLOGY-ENGINE` and a refreshed `source_revision`. Both rows are HOLD across truth,
candidate evidence and benchmark verdict, with `REQUIRED_UNRESOLVED` Buck delivery — no
target was invented.

**The full ledger does not validate today, before or after this lane's change:**

1. `npm run check:console-truth-ledger` fails at candidate resolution —
   `integration tip changes product path after candidate: backend/Cargo.lock`. The pinned
   candidate `88c57a1d` is far behind the tip and only authority-control paths may differ.
2. With that bypassed, the first per-row failure is
   `CAP-LOGISTICS-PILOT truth must be an object` — twelve pre-existing rows carry neither
   `truth` nor `route_presentation`.

Both are integrator work (candidate rebind, then row completion), and both were verified
to fail identically on the **unmodified** registry. This lane repaired neither, because
other agents' truthful row states are not this lane's to rewrite.

What *is* proven: [`manifests/verify-new-rows.mjs`](manifests/verify-new-rows.mjs) runs
the real validator over a subset containing the two new rows plus their dependency
targets and returns `STRUCTURALLY_VALID_HOLD_PRESERVED`, with the jurisdiction traces
from [`manifests/jurisdiction-register-traces.json`](manifests/jurisdiction-register-traces.json)
applied. Those twelve traces must be appended for the bijection to hold.
