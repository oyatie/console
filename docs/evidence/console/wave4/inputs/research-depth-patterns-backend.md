# Lens D — Best-in-class business-logic depth patterns (backend)

Wave-4 pre-planning brief. Accessed 2026-07-24. Read-only research; no repo changes.

Grounding already established internally (cited, not re-derived):
- `.omc/research/be-ontology-engine-arch.md` (main repo) — §18 registry incl. `ont_action_types.{submission_criteria, side_effects, edits}` JSONB (L89-96), guardrail gate chain (L170-186), effective-dated append-only instance store `valid_from/valid_to` + fixity chain (L43-63), request composition RLS∧Cedar∧audit (L241-257).
- `.omc/research/benchmark-brief.md` — Workday BP framework + effective-dated time-sliced rows (L365-371); Temporal effectively-once side effects, "already-recorded Activities are not re-executed" (L386); "current state = fold over append-only effective-dated log" (L447).
- `.omc/research/foundry-domain-research.md` — CAP-5 action-engine ACs (L47-53); KR payroll compute surface + effective-dated `(rate_kind, effective_from, effective_to)` rate tables + July-1 NPS boundary (L72-102); golden-case/노무사 gate (L111-115).
- `.omc/research/palantir-blueprint.md` — closed-loop audited Actions doctrine (L7, L21).

This brief adds the *engine-shape* patterns those docs don't cover: how production systems structure the hard parts, and the minimal production-grade version for our scale (single conglomerate, Postgres, RLS + `with_audit`, tens-of-thousands of rows — not SaaS-at-Workday-scale).

---

## 1. Payroll engines (Workday/ADP-class)

### Pattern A — rules-as-data, effective-dated
Every rate/threshold/bracket is a row keyed by validity interval, never a constant; the engine resolves config *as of the pay period*, not as of "now". Workday stores all HCM/payroll records as time-sliced effective-dated versions (benchmark-brief L365-371; [Mastering Effective Dating in Workday HCM](https://www.linkedin.com/pulse/mastering-effective-dating-workday-hcm-kalyani-ghule-sxafc)). **Already fully spec'd for us** in foundry-domain-research §2b (KR rate table family incl. the July-1 NPS 상·하한 boundary). Nothing new to derive — the lane implements that table family verbatim.

### Pattern B — pay run as an FSM over immutable results
Production engines never mutate a computed pay result. The unit is a **pay run** (pay group × period × run type ∈ {regular, off-cycle, retro}) moving through `draft → calculated → reviewed/approved → committed/completed`. Committed results are frozen artifacts; every result row records *which config versions and which input versions* produced it (calc versioning). Why: auditability (reproduce any payslip), retro diffing (below), and the legal artifact (급여명세서) must be bit-stable after issuance.

### Pattern C — retro recalculation as diff-and-carry-forward
Workday Retro ([Kognitiv: Should Your Company Activate Workday Retro](https://kognitivinc.com/blog/should-your-company-activate-workday-retro), [WSU Modernization note](https://modernization.wsu.edu/2025/06/30/new-retroactive-payroll-functionality-available-7-7/)):
0. Retro is a **periodic batch run, not a per-event trigger**: "every period, the payroll team will run a retro calculation that will look for employees with supported retroactive events and create a retro result similar to a payroll result" ([UW–Madison Retro Payroll Process](https://hr.wisc.edu/hr-guides/for-hr-professionals/retro-payroll-process/)). Verified 2026-07-25.
1. A backdated pay-impacting event (comp change, time entry, absence, benefits) whose effective date falls inside the pay group's **retro lookback window** flags the employee. The **reprocess date = the earliest date across all retro notifications for that payroll relationship**, and reprocessing runs across *all* runs from that date forward — "as payroll calculations are cumulative" ([Oracle: Recalculate Payroll for Retroactive Changes](https://docs.oracle.com/en/cloud/saas/human-resources/falzi/recalculate-payroll-for-retroactive-changes.html)). This cumulative property is the reason you cannot recompute a single period in isolation: YTD-dependent bases (in KR: 4대보험 정산, 연말정산 carry) shift every later period.
2. The retro engine **recalculates all results from the reprocess date to the present** using the corrected inputs — producing *new* result versions, never editing the originals.
3. It **diffs** new vs original per earning/deduction and **brings the differences forward** as retro delta lines into the current on-cycle run (or an on-demand payment).
4. Caveat class: **unsupported retro transactions** — Workday explicitly classifies backdated changes it cannot auto-recalculate (employees with multiple jobs in *different pay groups*, or multiple jobs in *multiple companies*; multiple changes to job/FTE) and *surfaces them for manual adjustment* rather than silently skipping ([WA OFM: Unsupported Retro Transaction](https://ofm.wa.gov/unsupported-retro-transaction-workday-payroll)). Unsupported events halt retro and force manual handling — a retro engine needs an explicit "unsupported ⇒ surface as exception" path, not silent skips. Retro is operationally "a second payroll cycle within the cycle".
Why it exists: the alternative (mutating history) destroys the audit trail, the issued payslips, and the GL postings already made from the original run.

### Pattern D — gross-to-net as an ordered pipeline
Fixed stage order, because each stage's base depends on the previous: earnings (regular + 가산수당) → pre-tax exclusions (비과세: 식대/자가운전) → statutory contributions on their own clamped bases (4대보험 — each with a *different* base definition) → income tax (간이세액표 lookup) → local tax (derived 10%) → post-tax deductions → garnishments computed on **disposable earnings, not gross** → net ([ADP payroll deductions](https://adp.com/resources/articles-and-insights/articles/p/payroll-deductions.aspx), [Patriot: how to calculate net pay](https://www.patriotsoftware.com/blog/payroll/how-to-calculate-net-pay-from-gross-pay/)). The stage order is itself config-versioned data in big engines; for us it can be code, but each stage's inputs/outputs must be persisted per result line (the 급여명세서 legally requires per-item 계산방법 — foundry-domain-research L82).

### Minimal production-grade version for us
- Tables: `pay_runs (id, org, pay_group, period, run_type CHECK IN ('regular','off_cycle','retro'), state FSM, config_snapshot_ref)`, `pay_results (run_id, employee, line_kind, base, rate_ref, amount, source_input_version)` — results **immutable once run committed** (reuse the `audit_events` REVOKE UPDATE/DELETE + trigger pattern, be-ontology-engine-arch L16).
- Retro v1: no automatic event detection. A backdated correction to a committed input (timecard, comp) *requires* opening a `retro` run scoped to (employee, affected periods); engine recomputes those periods against corrected inputs, writes delta lines referencing the original result rows, deltas pay out in the next regular run. Detection can stay a query ("committed inputs newer than the run that consumed them" — see §5 version pointers), surfaced as a work-queue item, not a background daemon.
- Calc versioning v1 = `config_snapshot_ref` (the resolved rate-table rows' ids/hash) + `source_input_version` per line. That is enough to reproduce any payslip byte-for-byte.
- Skip: multi-pay-group retro, forecasting runs, continuous "always-on" calculation. Golden-case + 노무사 gate already binding (foundry-domain-research §2d).

---

## 2. Time & attendance engines (Kronos/UKG-class)

### Pattern A — raw punches vs derived interpretation, strictly separated
The punch stream is append-only ground truth; everything payable is a **derived layer** computed by applying a versioned **pay rule** (a bundle of "work rule building blocks": punch-round rules, grace, breaks, day divide, zones) to the punch pairs ([UKG: Configure Work Rule building blocks](https://customer2.kronos.com/support/KOL/onlinehelp/Subsystems/Help-STP/Content/SetupHelp/ConfigWorkRuleBldgBlocks.htm), [UKG Glossary](https://communityfiles.ukg.com/support/kol/onlinehelp-workforcedimensions/en-us/content/MasterTopics/Glossary.htm)). Why: reinterpretation must be possible (rule fixed, rounding policy changed, judge disagrees) without touching evidence; and the same punch can be non-payable under one rule and payable under another.

### Pattern B — rounding as configured policy with a legality constraint
UKG models rounding as: division of the hour into equal segments + a **grace** determining forward/backward rounding at the boundary; either per-punch or per-interval (interval round: 8:07–16:25 → 8:15 under a 15/7 rule) ([UKG rounding configurations guide](https://ukg.cloudapper.ai/time-capture/a-guide-to-ukg-time-clock-rounding-configurations/), [Configure Work Rule building blocks](https://customer2.kronos.com/support/KOL/onlinehelp/Subsystems/Help-STP/Content/SetupHelp/ConfigWorkRuleBldgBlocks.htm)). Config is reached at Setup → Pay Policies → Work Rule Building Blocks → Punch Round Rules, and carries one hard structural invariant: **"a round must exceed its corresponding grace"** (verified 2026-07-25). That is a free `CHECK (increment > grace)` for us — a policy row violating it rounds ambiguously at the boundary. The legal envelope (US analog, same logic applies to KR practice): rounding is only lawful if **neutral over time** — never systematically employer-favoring (FLSA 29 CFR 785.48(b), the "7-minute rule": 1-7 down, 8-14 up — [My Hours: 7-minute rule explained](https://myhours.com/articles/7-minute-rule-explained)). So the rounding policy must be (a) data, (b) effective-dated, (c) symmetric by construction.

### Pattern C — day divide, overlap, and exceptions as first-class objects
- **Day divide**: a configured boundary (e.g. 04:00) attributing an overnight shift's hours; UKG optionally credits the whole shift to the "day actually worked" by splitting a span at the divide. Needed the moment any 야간 shift exists (and 야간수당 22:00-06:00 windows make split-at-boundary mandatory for us).
- **Overlap/duplicate resolution**: a punch pair overlapping an existing paid segment, double IN, missing OUT — production systems do **not** silently infer; they emit typed **exceptions** (missed punch, unexpected punch, early/late) that block timecard approval until a supervisor resolves them with an audited edit.
- **Sign-off = lock**: the pay-period timecard is approved then signed-off, which freezes it as the version payroll consumes; post-sign-off edits are historical corrections that trigger retro (→ §1C) ([UKG Dimensions supervisor guide](https://www.bsu.edu/-/media/www/departmentalcontent/payroll/pdfs/ukg/ukg-dimensions-users-guidefinal.pdf?sc_lang=en&hash=29AA5B7571A6A93B5547C91E5182A90A3C97A6E6)).

### Minimal production-grade version for us
- `raw_punches (org, employee, ts, direction, source, device_meta)` append-only, immutability-triggered. Derived `timecard_segments (employee, work_date, start, end, segment_kind ∈ {regular, overtime, night, holiday}, punch_refs[], rule_version)` — **deterministically recomputable** from (punches × effective-dated work rule); recompute-on-change instead of stored mutation.
- Rounding v1: one org-level effective-dated policy row `{increment, grace}` applied per punch; symmetric by construction. KR gotcha: rounding *down* against the worker is 임금체불 exposure — default policy = no rounding (pay actual minutes), keep the knob as data.
- Exceptions: reuse the §18 ontology — `TimecardException` as an object type with resolve actions, feeding the existing work-queue surface. Approval/sign-off = the instance lifecycle `active → locked` (be-ontology-engine-arch L138) — do not invent a second lock mechanism.
- Skip: schedule-vs-actual variance engine, attestation prompts, geofencing. Add when a real branch needs them.

---

## 3. Ledger / ERP (SAP-class + Modern Treasury-class)

### Pattern A — double-entry enforced at write time, entries immutable once posted
The invariant: a journal transaction's entries **sum to zero and commit all-or-nothing**; posted entries are never edited ([Modern Treasury Ledgers](https://www.moderntreasury.com/products/ledgers), [How to Scale a Ledger Part V](https://www.moderntreasury.com/journal/how-to-scale-a-ledger-part-v)). Modern Treasury's nuance: a **pending** transaction is mutable (and discardable/archivable), a **posted** one is immutable; business objects (orders, requests) stay mutable and *feed* the immutable accounting layer ([Enforcing Immutability in your Double-Entry Ledger](https://www.moderntreasury.com/journal/enforcing-immutability-in-your-double-entry-ledger)). Why: balances must be a pure fold over entries; any in-place edit silently corrupts every downstream report and closes the audit trail.

### Pattern B — reversal-vs-adjustment doctrine (SAP)
Never edit a posted document. Two sanctioned corrections, both *new documents linked to the original*:
1. **Reversal**: post the mirror document (debits↔credits). SAP distinguishes **normal reversal** — posts the wrong debit as a credit and vice versa, which *increases* the transaction figures on both sides — from **negative posting**, where the amount "is not added to the transaction figures, but is subtracted from the transaction figures on the other side of the account", resetting them to their pre-error value (verified 2026-07-25: [SAP Help: Executing Negative Postings](https://help.sap.com/docs/SAP_S4HANA_ON-PREMISE/99fb46a79ab241d5984df80fe7a9aa32/93019b2d2f7e4db9ac563beba5688606.html), [SAP blog: Negative Postings in Journal Entries](https://community.sap.com/t5/enterprise-resource-planning-blog-posts-by-sap/negative-postings-in-journal-entries-reversals-adjustments-and-more/ba-p/13570785)). Negative posting is **doubly gated**: the company code must permit it *and* the reversal reason must be configured to allow it. The distinction exists purely so period totals stay meaningful ([SAP Help: Reversal of Documents](https://help.sap.com/docs/SAP_S4HANA_ON-PREMISE/651d8af3ea974ad1a4d74449122c620e/99a6f42c861d4f85b047cce702a8d7cd.pdf), [SAP Learning: Configuring Document Reversal](https://learning.sap.com/courses/customizing-core-settings-in-financial-accounting-in-sap-s4hana/configuring-document-reversal)). Reversal requires a **reversal reason** (config-controlled, may permit an alternative posting date).
2. **Adjustment/delta**: post only the difference (Modern Treasury's "difference posting", SAP accrual/deferral auto-reversing docs). Full-reverse-and-repost is the auditable default; delta posting is for accruals.
Reversal into a **closed period** is forbidden — the reversal posts into the current open period with the original linked ([SAP Community: reversing from closed period](https://community.sap.com/t5/enterprise-resource-planning-q-a/reversing-document-fb08-from-closed-period/qaq-p/12547904)).

### Pattern C — period close as a gate plus an orchestrated task list
Two separable mechanisms:
1. **Posting-period control**: an open/closed flag per (entity, period, module); every posting validates against it ([SAP Help: Opening and Closing Posting Periods](https://help.sap.com/docs/SAP_S4HANA_CLOUD/0fa84c9d9c634132b7c4abb9ffdd8f06/a11940e4f97143d98a82e4741827e580.html)). This is the *enforcement point* and is cheap.
2. **Close orchestration**: SAP Advanced Financial Closing models the close as a dependency-ordered task list per entity/period — tasks auto-launch when prerequisites finish, mixing automated jobs and audited manual tasks, with escalation and a progress dashboard ([SAP blog: task orchestration with AFC](https://community.sap.com/t5/financial-management-blog-posts-by-sap/unify-your-close-task-orchestration-with-sap-advanced-financial-closing-and/ba-p/14274476)). This is *workflow*, not ledger.

### Minimal production-grade version for us
- `journal_transactions (org, id, period_id, status CHECK IN ('pending','posted','archived'), reversal_of NULL, reversal_reason NULL)` + `journal_entries (txn_id, account_id, direction, amount NUMERIC)`; sum-zero enforced inside the `with_audit` tx (app-level assert + a `CONSTRAINT TRIGGER DEFERRABLE INITIALLY DEFERRED` checking per-txn sum — DB backstop, since a CHECK can't span rows). Posted rows get the existing REVOKE UPDATE/DELETE + trigger immutability treatment.
- `accounting_periods (org, period, status open/closed, closed_by, closed_at)`; posting validates period open; close = an action with four-eyes (reuse `gov_approvals`, be-ontology-engine-arch L155). Reopen = its own audited four-eyes action, not an UPDATE.
- Corrections: reversal action only (mirror txn + bidirectional link + mandatory reason). **Skip negative posting** — our reports can compute "net of reversals" at read time; that is SAP's problem at SAP's transaction volumes. `ponytail:` note the ceiling.
- Close orchestration v1: a per-period checklist object on the §18 ontology (tasks + dependencies + the existing approval/work-queue surface) — not a new engine. AFC is the pattern ceiling if multi-entity close ever hurts.

---

## 4. Foundry ontology/actions — precision check vs our §18

Our §18 spec (be-ontology-engine-arch §2, §4) is already Foundry-faithful in shape. What their public docs add in *semantics*:

### Rules ([Action types • Rules](https://www.palantir.com/docs/foundry/action-types/rules))
- Rule kinds: create/modify/delete object, create/delete many-to-many link (one-to-many links are edited *as FK properties*, not link rules — matches our projected-FK approach).
- **Compile semantics**: multiple rules compile into a single edit per entity; ordering constraints (no delete-before-add, no modify-before-add); conflicting property writes resolve to the **final rule's value** (last-write-wins). Our `edits` JSONB needs this stated: dedupe-per-entity + last-write-wins, or reject conflicts at authoring time.
- **Function-backed actions are exclusive**: when a function rule is present, *no other rule may be configured* — the function does everything. This maps exactly to our `dispatch` enum: `projected_usecase` (the Rust use-case = the edit function, owns everything) vs `instance_revision` (declarative edits). The enum already enforces exclusivity structurally. No change needed; document the equivalence.

### Submission criteria ([Action types • Submission criteria](https://www.palantir.com/docs/foundry/action-types/submission-criteria))
Conditions gating submit, verified 2026-07-25 against the doc: the current user's **id**, **group memberships** (direct *or inherited*), or "any other multipass attribute available (such as the user's Organization)"; combinable with **parameter values**, letting builders compare against object properties (e.g. "amount < X unless finance group"). Evaluation happens "at the moment that the action is submitted", and "actions can only be submitted if all the submission criteria are met."

> **Correction to an earlier draft of this brief.** A previous revision asserted that "failed submission criteria also suppress all side effects" as a documented distinct semantic. The submission-criteria page does **not** state this. It is true only *by construction* — criteria block submission, an unsubmitted action runs no rules, and side effects are rules ([Rules](https://www.palantir.com/docs/foundry/action-types/rules) lists notification/webhook/schedule as rule kinds). Treat it as an inference from the rule model, not a cited guarantee. The practical consequence for us is unchanged, and it is the important one: **criteria must be evaluated before anything is dispatched — never after edits, never in parallel with side-effect emission.**

For us: principal-side criteria belong in Cedar (gate 1), but *parameter-dependent* criteria (amount thresholds, state preconditions) are the `submission_criteria` JSONB's real job — evaluate inside the execute tx before edits, and preflight must evaluate them too (our §4 preflight already does).

### Side effects ([Side effects overview](https://www.palantir.com/docs/foundry/action-types/side-effects-overview), [Webhooks](https://www.palantir.com/docs/foundry/action-types/webhooks))
Two webhook modes with different guarantees:
- **Writeback webhook**: runs *before* ontology edits; failure **aborts the whole action**; max one per action; its outputs are usable in subsequent rules (write the external system's response into object properties). This is Foundry's transactional integration with an external system-of-record.
- **Side-effect webhook**: runs *after* edits; best-effort; multiple allowed; user sees success before side effects complete; failures "won't prevent ontology modifications or affect the success message shown to users"; outputs are *not* consumable by rules.
- **The honest limit Palantir itself documents** (verified 2026-07-25): writeback gives only "some degree of transactionality between Foundry and the external system" — the external-fails-then-no-edit direction is guaranteed, but **the reverse is not**: the external call can succeed and the ontology write then fail. This is the classic dual-write problem, and it is the single best argument for our deferral decision below. Even the reference implementation cannot make a remote HTTP call and a local commit atomic.
- Notifications are the other side-effect kind.
Our §6 currently has only post-commit side effects ("after commit, idempotent", L253). Decision for chartering: **defer the writeback-webhook slot** (we have no external system-of-record yet; KNL integrations are inbound). Keep `side_effects` JSONB post-commit only, but implement delivery via a transactional outbox (§5) so "idempotent, after commit" is a mechanism, not a comment.

### What we deliberately don't copy
Branching action types, interface/polymorphic rules v1, Automate-style scheduled rules (our automation enters through the same execute path per §4 — keep that invariant).

---

## 5. Idempotency + effective-dating + bitemporality for cross-domain writeback chains

### Pattern A — idempotency keys (Stripe-class)
Client-generated `Idempotency-Key` header; server persists the **first outcome (status + body)** keyed by it and replays that outcome on retry — including error outcomes; reusing a key with different parameters is rejected; keys expire (Stripe: 24h) ([Stripe blog: Designing robust and predictable APIs with idempotency](https://stripe.com/blog/idempotency), [Stripe API reference: Idempotent requests](https://docs.stripe.com/api/idempotent_requests)). The reference Postgres implementation (single `idempotency_keys` table: key, request_hash, recovery_point, response_code, response_body, locked_at) is [brandur.org/idempotency-keys](https://brandur.org/idempotency-keys). Why: any writeback chain crossing a network boundary (mobile punch clock → API; automation → action execute) retries, and retries must not double-execute an action.

### Pattern B — transactional outbox for side-effect delivery
Side-effect *intent* rows are written in the **same transaction** as the mutation (ours: inside `with_audit`); a worker delivers them at-least-once; receivers are idempotent. This is the concrete mechanism behind both Foundry's post-edit side effects and Temporal's effectively-once replay already noted in benchmark-brief L386. Without it, "fire webhook after commit" either loses effects (crash between commit and send) or double-fires.

### Pattern C — bitemporality: valid time vs transaction time
Valid time = when the fact is true in the world; transaction time = when the system recorded it. Retroactive correction = a new record with `valid_from` in the past and `recorded_at` = now — recorded history is never rewritten ([XTDB: Bitemporality](https://v1-docs.xtdb.com/concepts/bitemporality/), [Software Patterns Lexicon: Retroactive Changes Handling](https://softwarepatternslexicon.com/103/8/1/)). **Our §1b store is already bitemporal in effect**: `valid_from/valid_to` (valid time) + append-only revisions with `created_at` + fixity chain (transaction time + tamper evidence) — be-ontology-engine-arch L50-62. No new storage needed; the pattern to *add* is the query discipline: "as-of(valid_time)" vs "as-recorded(revision)" are different questions and both must be answerable.

### Pattern D — consumed-version pointers for cross-domain chains (attendance→payroll→GL)
The chain-safe composition, assembled from A-C:
1. Producer domain commits an **immutable version** of its fact set (signed-off timecard vN; committed pay run vN).
2. Consumer records **which version it consumed** (`pay_results.source_input_version`; GL posting references `pay_run_id`).
3. A retro/backdated correction produces vN+1 — it never touches vN.
4. "Needs recalculation" = the pure query *"committed input versions newer than the version my committed output consumed"* — no event bus, no listener, no missed-message failure mode. Surfaced as a work-queue item; the human (or automation) opens the retro run (§1C) / reversal posting (§3B).
5. Every hop's write is an §18 action execute carrying an idempotency key; every hop's outbound effect goes through the outbox.
This makes the whole attendance→payroll→ledger chain **re-derivable and auditable at every joint**, which is the actual point.

### Minimal production-grade version for us
- One platform-level `idempotency_keys (org, key, request_hash, response_code, response_body, created_at, expires_at)` table + a check at the top of action `execute` (inside the tx: insert-or-return-stored). No recovery-point machinery v1 — our executes are single-tx, so first-write-wins + stored-response replay covers it. `ponytail:` add recovery points only if an execute ever spans multiple txs.
- One `outbox (org, id, kind, payload, created_at, delivered_at NULL, attempts)` table written inside `with_audit`; one worker loop; receivers idempotent by (kind, source id).
- No new bitemporal store — §1b already is one. Add the `as_of` vs `revision` query distinction to the instance REST (the `?as_of=` param exists; history endpoint covers as-recorded).
- Consumed-version pointer = a column convention, not infrastructure.

---

## Summary of deltas vs the existing §18/engine spec

| Area | Already spec'd (cite) | New from this brief |
|---|---|---|
| Payroll rates | effective-dated rate family (foundry-domain-research §2b) | pay-run FSM, immutable committed results, retro = diff-and-carry-forward run, calc-version refs |
| T&A | — (compute list only, §2a items 4) | raw-punch/derived-segment split, effective-dated symmetric rounding policy, exceptions-as-objects, sign-off lock reusing instance lifecycle |
| Ledger | audit immutability pattern (be-ontology L16) | sum-zero deferred constraint trigger, pending/posted states, reversal-link doctrine (no negative posting), posting-period gate + checklist-based close |
| Actions | §18 registry + gate chain (be-ontology §2,§4) | last-write-wins edit compile rule, param-dependent submission criteria in-tx, writeback-webhook slot explicitly deferred, outbox as the side-effect mechanism |
| Cross-domain | Temporal effectively-once note (benchmark L386), §1b bitemporal store | idempotency_keys table + execute check, outbox table, consumed-version-pointer convention, recalc-needed as a query not an event bus |

---

## 7. Verification log (2026-07-25 re-verification pass)

Every load-bearing external claim above was re-fetched. Status:

| Claim | Source | Status |
|---|---|---|
| Rule kinds; "compiles rules to generate a single edit per object"; later rules overwrite earlier; conflicting property write resolves to last rule's value | palantir.com/docs/foundry/action-types/rules | **verified verbatim** |
| Function rule exclusivity: "no other rule may be configured since function code alone is capable of handling everything" | same | **verified verbatim** |
| 1-to-many links edited as FK on Modify Object; many-to-many via dedicated link rules | same | **verified** |
| Writeback webhook: before edits, max 1, failure ⇒ no changes + user error, outputs usable by later rules | palantir.com/docs/foundry/action-types/webhooks | **verified** |
| Side-effect webhook: after edits, multiple allowed, best-effort, failures invisible to submitter | same | **verified** |
| Dual-write hazard (external success + ontology failure remains possible) | same | **verified — newly added** |
| Submission criteria: user id / group (direct or inherited) / any multipass attribute / parameter values; evaluated at submit | palantir.com/docs/foundry/action-types/submission-criteria | **verified** |
| "Failed criteria suppress side effects" | — | **CORRECTED — not documented; inference only** (see §4) |
| Normal reversal inflates transaction figures; negative posting subtracts on the other side to reset them; reversal reason mandatory; company code **and** reversal reason must both permit negative posting | help.sap.com Executing Negative Postings + SAP community blog | **verified, sharpened** |
| Modern Treasury pending (mutable) / posted (immutable) / archived; corrections = full reversal-and-repost or difference posting; business objects mutable, accounting objects immutable | moderntreasury.com/journal/enforcing-immutability-in-your-double-entry-ledger | **verified verbatim** |
| Retro is a periodic run creating "a retro result similar to a payroll result" | hr.wisc.edu retro payroll process | **verified — newly added** |
| Reprocess date = earliest notification date; reprocessing spans all runs from it "as payroll calculations are cumulative" | docs.oracle.com Recalculate Payroll for Retroactive Changes | **verified — newly added** |
| Unsupported retro = multi-job across pay groups / companies, surfaced for manual adjustment | ofm.wa.gov Unsupported Retro Transaction | **verified** |
| UKG punch rounding = equal hour segments + grace deciding forward/backward; Setup → Pay Policies → Work Rule Building Blocks → Punch Round Rules | customer2.kronos.com ConfigWorkRuleBldgBlocks | **verified** |
| **"A round must exceed its corresponding grace"** | same | **verified — newly added, becomes a CHECK constraint** |
| Internal: `ont_action_types` JSONB columns, effective-dated append-only instance store + fixity chain, RLS∧Cedar∧audit composition | `/Users/jasonlee/Developer/maintenance/.omc/research/be-ontology-engine-arch.md` L43-63, L89-96, L241-257 | **verified against file** |

Not re-verified this pass (carried from the prior draft, confidence high but unfetched): Stripe idempotency-key semantics, brandur.org reference schema, XTDB bitemporality, FLSA 29 CFR 785.48(b) 7-minute rule, SAP AFC close orchestration, SAP posting-period control.

---

## 8. What this means for wave-4 lane chartering

1. **Three engines share one skeleton** — immutable committed artifact + version pointer + correction-by-new-document. Payroll `pay_runs`, T&A `timecard_segments`+sign-off, ledger `journal_transactions` are the *same* pattern three times. Charter the shared primitives (immutability trigger reuse, `config_snapshot_ref`, consumed-version pointer convention) **once, before** the three domain lanes, or three lanes will invent three incompatible versions.
2. **The cumulative property kills naive per-period recompute.** Any payroll lane charter must state that retro reprocesses *forward from the reprocess date*, not the single corrected period. A lane that ships single-period recompute has shipped a bug, not a slice.
3. **No event bus.** "Needs recalculation" is a query over version pointers (§5D). Charter it as a work-queue-backed query; explicitly forbid a listener/daemon design in the lane text, since that is the reflex answer and it introduces missed-message failure modes we do not need at our scale.
4. **The §18 action schema needs two documented semantics before domain lanes consume it**: edit compilation is dedupe-per-entity + last-write-wins (or reject conflicts at authoring), and parameter-dependent submission criteria evaluate *in-tx before dispatch*. Both are one-paragraph spec additions, not code — but they are collision-prone if each lane assumes differently.
5. **Defer the writeback-webhook slot explicitly** (not silently). Rationale is now citable: Palantir's own docs concede writeback is not truly transactional, and we have no external system-of-record. `side_effects` stays post-commit, delivered via the outbox.
6. **Two cheap DB-level invariants belong in the schema from day one**, not in review comments: deferred constraint trigger for per-transaction Dr=Cr sum-zero, and `CHECK (increment > grace)` on the rounding policy row.
7. **KR default for rounding is "none".** Ship the policy row as data with the knob present and the default set to pay actual minutes; rounding down against the worker is 임금체불 exposure. This is a legal default, not a simplification to revisit.
