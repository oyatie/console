# Wave-4 charter — Lens D: business-logic depth

Evidence base (every lane cites into these; no lane invents a requirement):

- Depth registers, 13 domains, scores 30–58: `tasks/w1pxjxdpk.output` → `result.registers[]`
- LIVE statutory brief (accessed 2026-07-25): `scratchpad/wave4/research-statutory-params.md`
- Backend depth patterns: `scratchpad/wave4/research-depth-patterns-backend.md`
- Spine delta (hot zones, collision roots, migration high-water): `scratchpad/wave4/scout-spine-delta.md`
- North-star lens-D doctrine: `scratchpad/intent/north-star-amendment-beyond-prototype.md` §"Lens D"

Domain scores (the ranking backbone): benefits 30 · equipment-3r 35 · payroll 42 · attendance 42 ·
finance 42 · leave 45 (1 wrongly-fabricated) · org 52 · maintenance-workorder 52 · field-support 52 ·
inventory 55 · notif-board-routing 56 · recruiting 58 · evaluation 58.

---

## 0. Standing rules for every lens-D lane

**Truthfulness line (binding, from the north-star amendment).**
Statutory-deterministic arithmetic is IMPLEMENTED with a citation and a test.
Externally-certified artifacts stay gated attestations, never estimated: final NTS 간이세액표 rows,
노무사/세무사 sign-off, 공단 고지액, bank transfer confirmations, 산재 업종별 요율 table,
전자세금계산서 relay. A lane that computes a gated artifact has failed, and so has a lane that
gates a computable one.

**param_verify_live.** Any lane implementing a rule with a yearly/regulatory parameter sets
`param_verify_live: true`, cites the live source URL from `research-statutory-params.md`, and —
where the brief does not carry the parameter — fetches it live and appends it to the brief's §8
table and §9 register BEFORE writing the rule. Never from model memory. Every seeded parameter row
carries `source_url` + `retrieved_on` NOT NULL. Every §9 `needs-verification` item resolves to
`Err(ParamUnverified)`, never a default (grep gate: no `unwrap_or` on a statutory resolve).

**Disjoint roots.** Lanes own the globs in `roots` and nothing else. Serialized single-owner
surfaces — `web/src/i18n/ko.ts`, `web/src/console/shell/nav.ts`, `web/src/console/screens/registry.ts`,
`backend/openapi/openapi.yaml`, `clients/**`, `backend/app/src/lib.rs` (beyond appended router-register
lines), `backend/app/src/objects.rs`, `**/BUCK` (generated face) — are **manifest-only**: the lane emits
`docs/evidence/console/CAP-<X>/api/openapi-fragment.yaml` and, if any UI copy is implied,
`docs/evidence/console/CAP-<X>/frontend/manifests/mount.json` + an i18n key inventory. The integrator
applies them. Mechanical fragment splicing of `openapi.yaml` is a proven failure mode
(`ee277e16` reverted whole by `9bb877c6`) — emit, do not splice.

**No new crates.** New shared code lands as a MODULE inside an existing crate
(`backend/crates/kernel/core/src/<name>.rs` + one appended `pub mod` line). A new crate forces
`tools/buck/gen_first_party.py` regeneration across ~10 BUCK files — a serialized generated face,
not a leaf-lane write.

**Migrations.** High-water on the branch is `0202_notification_policies_and_object_agg.sql`; `0201`
is a reserved gap already allocated to docs evidence-retention. Slots below are PROVISIONAL from
0210; the integrator renumbers at merge. Additive new files only — never edit an existing migration.
Re-check the high-water immediately before push.

**Codex-fleet hazards.** Tier-1 hot crates: `platform` (205 hits/48h, integrator-owned),
`attendance` (106, **active writer**, CAP-ATTENDANCE-CONSOLE = writer_assigned_gap_closure_in_progress,
3 live codex/attendance-* lanes), `backend/app/{src,tests}`, `production`, `logistics`, `inventory` (35).
Tier-3 safest landing zones: `kernel` (3), `erp` (2), `finance-gl` (6), `orgchange` (7), `governance`,
`policy`, `registry`, `todos`, `sales`. Lanes touching Tier-1 crates must confirm ownership before
starting and **plain-merge before every push (rebase is classifier-blocked)**.

**Lens boundary.** Lens D is backend business logic. No `web/**` edits in any lens-D lane — UI depth
is lens C. Lens-D lanes emit manifests only.

**Enterprise bar — implicit in every DoD, restated once:**
RLS FORCE and tested as `mnt_rt` (superuser tests are void — a BYPASSRLS pass proves nothing);
deny-by-default PBAC; audit row in the SAME transaction as every mutation; canonical error envelope
(`22x` validation vs `409` conflict correctly separated — a 500 for a DB CHECK is a defect);
idempotency key + canonical fingerprint on every create/decide; story-level app integration test;
Buck2 targets green (`buck2 test` is the completion evidence, cargo is dependency metadata);
AA a11y n/a (no UI); **no stubs, no fillers, no TODO, no `test.skip`/`.only`, no unimplemented
branches**; every deliberate simplification carries a `ponytail:` comment naming its ceiling and
upgrade path.

**Standard DoD preamble (assumed on every lane; the per-lane `dod` lists only what is lane-specific):**
1. `cargo fmt` clean; `cargo clippy -p <owned crates> -- -D warnings` clean.
2. Named `cargo test -p …` commands green.
3. Named app story test green as `mnt_rt` on the Buck2 PG harness
   (`mnt_buck_admin` superuser bootstraps, assertions run as `mnt_rt`).
4. `buck2 test //backend/crates/<owned>/...` green.
5. CI gates: `backend/ci/gates/{rls-arming, migration-safety, audit-coverage, layer-boundary,
   tenant-isolation, pii-no-logs}` as applicable.
6. Edge-case matrix table in the lane's evidence doc, one test per row, covering:
   **mid-period join/leave · backdated correction + effective-dated recalculation · concurrent
   transition race · reversal/compensation path · boundary dates (KST month-end, 회계연도 boundary,
   the 2026-07-01 NPS cap boundary, week-start Monday).** A row without a test is a blocker.
7. openapi fragment manifest emitted for any route change.
8. Every statutory constant carries its citation in the test name.

---

## 1. Execution order

```
FOUNDATION (must complete before fan-out; L-D0 → L-D1 → L-D2 serialized on kernel/core/src/lib.rs)
  L-D0  statutory parameter registry
  L-D1  calc-artifact spine
  L-D2  worktime interval engine

PARALLEL-WITH-FOUNDATION (independent roots, urgent)
  L-D3  leave 촉진 truthfulness repair   ← the ONLY wrongly-fabricated finding; fix FIRST
  L-D12 equipment handover custody repair ← P0, equipment_3r_api.rs is RED at HEAD

FAN-OUT (ranked by value)
  L-D4  → L-D5            payroll (domain, then lifecycle)
  L-D6  → L-D7            leave (accrual+촉진, then integrity edges)
  L-D8                    attendance close integrity
  L-D9  → L-D10           finance GL, then derivation chains
  L-D11                   cross-domain writeback invariants
  L-D13                   equipment rental economics
  L-D14                   benefits engine
  L-D15                   org effective-dating cascade
  L-D16                   evaluation scoring
  L-D17                   recruiting statutory obligations
  L-D18                   inventory valuation
  L-D19                   field-support SLA engine
  L-D20                   maintenance WO settlement + PM
  L-D21                   notifications/notices depth
  L-D22                   attendance planned timetable (largest, lowest rank)
```

Full per-lane scope, roots, DoD and migration slots are carried in the structured lane set returned
to the orchestrator; this document records the doctrine, ordering, and hazards that the structured
set compresses.

---

## 2. Three decisions this lens needs resolved before fan-out

**D-1. Where the statutory registry lives.** Recommendation taken: a module in `kernel/core`
(pure Rust, effective-dated, source-cited, fail-closed) — NOT a DB table. Rationale: the existing
`payroll/domain::statutory_contribution_rates()` already is an effective-dated source-cited table and
is tested; the real defect the registers name is *duplication and divergence* (benefits hand-types
`employer_rate_bps`; finance has two divergent VAT implementations, one of them a correct unwired
effective-dated table in `erp/domain`; attendance re-derives the 209h divisor). One shared Rust
registry fixes divergence without inventing infrastructure. The DB carries only what is genuinely
per-org and attested, not law: `org_statutory_facts` (상시근로자수 / 우선지원대상기업 / 업종코드) —
three of the four insurances band on these — plus the org-configurable conventions
(일할계산 역일수 vs 209h, rounding policy, 연차년도 basis, 산입범위 pending V1).

**D-2. 제11조 <5인 applicability must be a data table, not an `if`.** MOEL's 2026 업무보고 reportedly
proposes phased extension of 근기법 to <5인 사업장 (statutory brief §3.6 — **secondary sources only,
no gov URL, treat as unenacted**). Mitigation is cheap and must be taken now:
`article_applicability (article, min_headcount, effective_from)`. Flipping a row must flip the outcome
with no code change — asserted by a test in L-D6.

**D-3. Retro is a run type, not a trigger; recalc-needed is a query, not an event bus.**
Both are collision-prone reflexes. `research-depth-patterns-backend.md` §8.2–8.3 is explicit:
reprocessing runs **forward from the reprocess date across all subsequent runs** because payroll is
cumulative (a lane that ships single-period recompute has shipped a bug), and "needs recalculation"
is the pure query *"committed input versions newer than the version my committed output consumed"* —
no listener, no daemon, no missed-message failure mode. L-D1 fixes both shapes once; L-D5, L-D8,
L-D10 and L-D11 consume them. Every one of those lanes carries a grep gate asserting no
`tokio::spawn`/LISTEN-NOTIFY was introduced for recalc detection.

---

## 3. Statutory traps that will silently produce wrong money

Copied forward from the live brief so no lane re-derives them wrong:

1. **연장 + 야간 stacks to 2.0×** — MOEL: "야간근로가 휴일·연장근로와 중복될 경우 야간근로가산수당은
   추가지급되어야 합니다." Premiums are **additive independent flags on a time segment**, not an
   enum. Any design with one premium type per hour silently underpays — 임금체불 exposure, not a
   rounding bug. The registers confirm the current model cannot even represent it: hour buckets are
   pre-flattened, so overlap is *structurally unrecoverable at calc time*. This is why L-D2 exists.
2. **휴일 and 연장 do NOT stack** for the same hour — 휴일 has its own 50/100 ladder; 야간 is the
   only orthogonal add-on. Stacking them is the double-count the 2018 개정 settled.
3. **장기요양 = 건강보험료 × 0.9448/7.19**, computed on the *rounded 건강보험료*, not on income.
   The register confirms the code does it the income way; the divergence shows up won-level against
   every NHIS EDI 고지서.
4. **NPS caps run July→June, the rate runs calendar.** A single "2026 row" is wrong; both change
   inside 2026. Effective-dating is per parameter, not per year.
5. **제53조제3항 (<30인 +8h) expired 2022-12-31.** A 60h/week cap in 2026 is wrong and punishable.
   L-D2 carries a test asserting 60h is unreachable.
6. **휴게 must be interior to the shift** ("근로시간 도중") — a trailing break does not satisfy §54.
7. **연차 미사용수당 defaults to PAY.** The statute gives only the negative (§61 removes the
   obligation *when* 촉진 was complete and timely). Fail-open-to-paying is the safe direction;
   the reverse is 임금체불. L-D6's payout logic withholds only on a complete, in-window, evidenced chain.
8. **Rounding default is none.** Rounding down against the worker is 임금체불 exposure. The knob is
   data, effective-dated, symmetric by construction (`CHECK (increment > grace)`), default off.
9. **Reversal into a closed period is forbidden** — it posts into the current open period with the
   original linked. SAP negative posting is explicitly skipped (`ponytail:` ceiling: compute
   net-of-reversals at read time).

---

## 4. Known unresolved blockers carried into the lanes (from brief §9)

V1 최저임금 산입범위 부칙 · V2 산재 업종별 요율 (ingest data.go.kr 15068737, never hand-type) ·
V3 징수법 §13② · V4 근기법 시행령 별표1 · V5 §60⑥ 출근 간주 기간 · V6 주휴수당 part-timer amount ·
V7 미사용수당 기준임금 · V8 지방소득세 basis · V9 채용절차법 §3 적용범위 · V10 파기 "지체 없이" SLA ·
V11 §61 "서면" electronic acceptability · V12 공정채용법 status · V13 2026 간이세액표 rows ·
V14 <5인 확대 roadmap.

Each is seeded as `NeedsVerification { reason, where_to_get }` in L-D0 and resolves to `Err`.
Not one of them may be defaulted, estimated, or guessed. Three additional parameters are NOT in the
brief and must be fetched live by their owning lane before coding: 법인세법 시행규칙 별표 내용연수
(L-D13), 산안법 §93 / 건설기계관리법 §13 inspection cycles (L-D13, L-D20), 소득세법 §12 비과세 한도
won-amounts (L-D14), 근기법 §24③ 50일 / 4대보험 자격상실신고 기한 (L-D15), 근기법 §42 서류 보존기한
(L-D21).
