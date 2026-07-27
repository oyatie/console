# North-star amendment — beyond-prototype enterprise depth (2026-07-25, user directive)

> To be appended to `docs/intent/console-north-star.md` at the post-consolidation
> pass (worktree owned by the integrator right now).

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
