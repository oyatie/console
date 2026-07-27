# Console north star — confirmed intent (2026-07-24, interview-confirmed)

## Outcome

The console realized as a **corporate operating system on one ontology** — every
entity, operation, action, and state a typed, policy-governed, lifecycle-tracked
object in a single graph ("game engine for the corporation"; benchmark:
Palantir Foundry). The modules are faithful **lenses** over that substrate,
matching the Claude Design prototype's grammar exactly.

## Confirmed decisions

- **User:** every employee of the conglomerate (not just admins), per persona,
  deny-by-omission.
- **Fidelity = grammar + intent, never seed data.** The prototype's layouts,
  cards, steppers, chips, keyboard flows, drill links are the spec; its mock
  rows/counts are illustrations — copying them into the product is a defect.
  Design intent is distilled from the claude_design markdowns
  (design-intent-register), not only from pixels.
- **Backend-thin surfaces → build the backend.** When a module's design section
  is richer than the backend, the missing backend surface is built to full
  production enterprise standard (RLS as `mnt_rt`, deny-by-default PBAC, audit,
  lifecycle, canonical envelopes, idempotency, story-level integration tests) —
  not shipped as a truthful-but-thin screen with a recorded gap.
- **Ontology integration model = (a) projection registration**, per §18/D1/D2:
  domain crates stay the owning writers; every module object type (payroll run,
  posting, applicant, evaluation cycle, rental agreement, ASN, notice, org
  change, …) is REGISTERED in the shared ontology engine as a typed projection
  with its 3 layers (semantic / kinetic / dynamic), links, governed Actions, and
  policy resources — drillable in the graph explorer and 3-layer object cards.
  No re-architecture through the generic instance store.
- **§18 engine residuals close to production grade** — foremost: projected-type
  action dispatch routed through domain use-cases (replace NotWiredYet).
- **Priority: north star + maturity/polish** — substrate depth and production
  maturity over breadth; §4-25 closed-loop polish is part of done.
- **Zero stubs/fillers**, per HANDOFF §0 do-not-ship scaffold + DESIGN §4-12 +
  the program's non-negotiable module completion contract.

## Constraints

Everything stacks onto PR #488 (single stacked integration spine). ADR-0025
dark-mounting governs exposure (`EXPOSED_SCREEN_KEYS` stays evidence-gated).
Codex-fleet coexistence disciplines: hot-check before editing shared crates,
manifests for collision roots, plain-merge before push.

## Out of scope

Pixel-cloning prototype seed rows/counts; re-architecting verified domain
crates through the generic store; production exposure decisions.

---

# Amendment — beyond-prototype enterprise depth (2026-07-25, founder directive)

Appended by wave-4 L-P0-EPOCH. Source of record:
`docs/evidence/console/wave4/inputs/north-star-amendment-beyond-prototype.md`.
This amendment adds lens C (experience/production depth) and lens D
(business-logic depth) to the lenses already implied above (A = substrate,
B = fidelity). It does not replace anything in the 2026-07-24 body.

## Directive

The claude_design UI/UX is itself **mostly intent-level**: it lacks the detailed
polish and implementation depth (deep UX, integrations) that enterprise
production requires. Prototype fidelity is therefore a **floor, not a ceiling**
— the gaps the prototype never closed must be closed by us.

The authority admits this about itself: BENCHMARK.md's per-module honest-gap
column and its structural-gaps section (실시간·멀티유저 simulation-only, no
scale/virtualization/server-pagination, session-state persistence, ko-only +
partial screen-reader). DESIGN §4-21 (3-question benchmark loop) and §4-25
(8-question closed loop) are the authority's own instruction to exceed itself.

## What closing the gaps means (wave-4 lens C, alongside A=substrate, B=fidelity)

1. **Deep UX beyond the mockup, per module** — complete keyboard model (not
   just J/K/Enter: focus management, roving tabindex, escape/undo semantics),
   optimistic-update + conflict UX (409/412 surfaced as merge affordances, not
   toasts), bulk operations, draft autosave + restore, skeleton loading, error
   recovery paths with state preservation, empty states that name reason + next
   action (§4-10), virtualized lists + server pagination for real scale (the
   1,284-roster is seed fiction; production is unbounded), full AA a11y +
   screen-reader flows, responsive to mobile employee-app widths.
2. **Integration depth** — the C-3 standard chain 1-click traversable in the
   REAL console (not just within a module): cross-module drills carry object
   identity end-to-end; notifications wired to real events; rail↔main
   promotion; object-card ubiquity (every noun opens the 3-layer card);
   automation writebacks visible where they land (e.g. AT- approval updating a
   payroll exception).
3. **Enterprise-production concerns the prototype only gestures at** — audited
   sensitive views + masking defaults (§4-27), DLP affordances, SoD as
   deny-by-omission in the UI (not server-only), session fencing, real-time
   multi-user semantics where the design implies them (presence, unread,
   live status) built on real transport, i18n expansion tolerance, perf
   budgets.
4. **Benchmark loop as a mandated gate** — per module, the §4-21 pass ("what
   would Palantir/Workday/Slack/Greenhouse/SAP do better?") grounded in
   BENCHMARK.md's gap column produces a ranked register; top items become build
   lanes. No module is "done" at prototype parity.

## Lens D — business-logic depth (2026-07-25, user directive)

The implemented business logic is itself shallow and needs deeper polish. The
FSMs/validations the lanes shipped are skeletons of the real domain rules.

- **Statutory-deterministic rules get IMPLEMENTED, with citations and tests** —
  근로기준법 주52 computation (연장/야간/휴일 가산 and their overlaps), 연차
  accrual (1년 미만 월단위 + 15일+ tiers, 촉진 procedure, expiry), 주휴수당,
  일할계산/proration conventions (중도입퇴사), 대근 pay chain (SR-206),
  inventory valuation, double-entry invariants, rental
  proration/deposit/assessment pricing, SLA math. These are computable law and
  arithmetic — leaving them as thin gates is the shallowness being corrected.
- **The truthfulness line moves, precisely:** externally-certified artifacts
  (final tax filings, 노무사/세무사 sign-off, bank transfer confirmations)
  remain gated attestations — never estimated. Everything deterministically
  computable upstream of those artifacts is real engine logic.
- **Cross-domain invariants are first-class logic**, not integration
  afterthoughts: attendance exception approval → payroll exception amount
  writeback; substitution → C-D contract issuance → pay reflection; org change
  effective-dating cascades; evaluation finalization → person ledger.
- **Edge-case matrices are part of DoD**: mid-period joins/leaves, backdated
  corrections + effective-dated recalculation, concurrent transition races,
  reversal/compensation paths, boundary dates (DST-free KST, month ends,
  회계연도).
- Per-domain deepening registers (business-logic depth audit) drive lens-D
  lanes exactly as the fidelity registers drive lens B.

## Consequence for scoring

The fidelity audit's 0–100 scores measure distance to the FLOOR. A module at
100 fidelity is not done; wave-4 DoD = fidelity floor + lens-C register worked
off (or truthfully deferred with named items) + §4-25 closed loop run.

## Wave-4 decisions that bind this doctrine (2026-07-25)

Full text, with the reasoning and the standing risks, is committed at
`docs/evidence/console/wave4/inputs/DECISIONS.md`. The ones that change how the
north star is read:

- **D-0 · Depth-first.** C-64's CRM → WMS → MES order is obeyed, not waived. A
  module reaches genuine production depth before the next one starts; breadth
  across the remaining thirteen is bought only through shared Phase-0 fixes.
  What that costs is itemised, not implied:
  `docs/evidence/console/wave4/backend-blocked-index.json`.
- **D-1 · Exposure is an evidence chain, not a flag.** `EXPOSED_SCREEN_KEYS` is
  `[]` today because `b9e7fd74` emptied it for want of evidence. An entry lands
  only with runtime proof, a committed browser user-story replay, an a11y
  matrix and an ops observation. If the evidence does not hold, the entry does
  not land and we say so.
- **D-2 · Plain-merge train.** The integration protocol is
  `git merge origin/<spine>` before push. Rebase and cherry-pick are struck —
  they are classifier-blocked on this spine. No lane may rebase.
- **D-5 · Dead code is deleted.** A documented control with no implementation,
  or a field nothing reads, is removed unless the same lane wires it for real.
  This is the operational form of the zero-stubs line above.

**R-1, recorded here because it is a standing constraint on the doctrine, not a
wave item:** payroll's wage-law layer is shallow (연장 ×1.5 is gate-only,
야간/휴일 hours are dead columns, overlap is structurally unrecoverable, 주휴수당
is absent, the golden-case gate never executes). Nothing is exposed, so there is
no live liability — but **payroll must not be exposed, and must not process real
runs, until the wage-law lanes land.**
