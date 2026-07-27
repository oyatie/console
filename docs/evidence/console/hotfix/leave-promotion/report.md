# Hotfix — 연차사용촉진 (근로기준법 제61조): the fabricated statutory procedure

**Lane** `hf-leave-promotion` · **Worktree** `hf-leave-promotion-20260725` · **Date** 2026-07-25

**Status: COMPLETE.** Statutory logic implemented and red-proofed (§4.2); the runtime-role
test runs as `mnt_rt` and is green (§4.1); `fmt` and `clippy -D warnings` clean. 58 of 59
lane tests green — the single red is pre-existing on the base branch and belongs to
migration `0183` (§4.3). Wire/migration/web collision roots are specified as manifests, not
edited (§5).

The business-logic depth audit found exactly one *wrongly-fabricated* rule in the whole
codebase, as opposed to a shallow or missing one, and it is here. This is the record of
what was fabricated, what the statute actually says (verified live), and what was done.

---

## 1. The fabrication, located precisely

Three defects in the §61 statutory-push path. They compound: the first two let a notice be
served at any time, and the third told the worker that doing so had extinguished their
claim.

### 1.1 `validate_round` — the statutory windows were never enforced

`backend/crates/leave/domain/src/lib.rs:575-589` (at HEAD before this lane):

```rust
pub fn validate_round(kind: PromotionKind, round: i16) -> Result<i16, KernelError> {
    match kind {
        PromotionKind::Promotion => {
            if round == 1 || round == 2 { Ok(round) }
            else { Err(KernelError::validation("연차 촉진 round must be 1 or 2 (§61)")) }
        }
        PromotionKind::Refusal => Ok(2),
    }
}
```

**Claims:** the doc comment above it and on `PromotionKind` (`lib.rs:520-523`) describe a
two-round statutory procedure — "round 1 = 사용 촉구, round 2 = 시기 지정" — and the
function cites `§61`.

**Does:** checks that an integer is 1 or 2. Nothing else. §61 makes each round valid only
inside a window counted back from the end of the 연차 사용기간; none of those windows
existed anywhere in the codebase. A 1차 촉구 could be served in December, a 2차 통보 five
minutes later, both citing 근로기준법 제61조 on a locked legal notice in the worker's
개인 수신함.

The `Refusal => Ok(2)` arm is worse than permissive: it *normalises any input* and its
comment asserts the refusal "always follows a completed round-2 promotion" while checking
nothing. A 노무수령거부 could be served to an employee who had never received a 촉구.

The pre-existing integration test proved this rather than caught it —
`statutory_push_delivers_receipt_doc_and_is_idempotent` served round 1, round 2 and the
refusal back-to-back at `OffsetDateTime::now_utc()` and asserted success.

### 1.2 The refusal notice asserted a legal outcome that had not occurred

`backend/crates/leave/adapter-postgres/src/lib.rs:1577` (at HEAD before this lane):

```rust
"본 통지로써 해당 연차에 대한 사용자의 금전 보상 의무가 소멸함을 안내드립니다.",
```

> "By this notice, the employer's monetary compensation obligation for this leave is
> extinguished."

False as a general statement, and the worst class of output the audit looked for: legally
wrong, addressed to the affected worker, on a document carrying the statute's name behind
a passkey receipt gate. §61 relieves the employer **only** where every step was taken in
서면 inside its window and the worker still did not use the leave; and 대법원 2019다279283
holds that where the worker works the designated day and the employer accepts the labour,
the obligation survives regardless of the paperwork. The system observed none of those
conditions and issued the notice anyway.

Same paragraph set: the round-2 body said "사용 시기를 지정하여 통보하오니 지정된 시기에
연차를 사용하시기 바랍니다" — *designating no dates*. §61①2 requires the 2차 통보 to
designate the 사용 시기. A round-2 notice with no dates is not a §61①2 통보, however
authoritative it reads.

### 1.3 The unused-day count on the notice came from the client, defaulting to zero

`backend/crates/leave/rest/src/lib.rs:619-620` and `:637-638` (at HEAD before this lane):

```rust
#[serde(default)]
unused_days: f64,
```

§61①1 requires the employer to state the worker's own 미사용 휴가 일수. The figure was an
unvalidated caller-supplied float defaulting to `0` when absent, rendered straight into
the notice as "귀하의 미사용 연차 0일에 대하여…". The authoritative figure
(`employees.leave_remaining`) was one query away and unused.

### 1.4 The presentation scale — found by the runtime-role test, not by review

The server-side read replacing the client float was written as `leave_remaining::text`.
`employees.leave_remaining` is **`NUMERIC(16,6)`** — widened from `NUMERIC(10,2)` by
migration `0166_leave_exact_charge_and_home_branch.sql:30` — so the notice served to a
worker read **`미사용 연차 유급휴가는 13.000000일입니다`**. Not false, but not a figure a
statutory notice may print. Every hand-written unit test passed, because each fed
`notice_body` a literal `"13.00"` fixture; only the database-backed test saw the column's
real scale. Fixed with `trim_scale(leave_remaining)::text` (PG13+, drops trailing zeros
without changing the value) → `13일`, `13.5일`. The test fixture now seeds a half-day
remainder so a future `round()` cannot pass in its place.

Deliberately **not** applied to the import/ledger reads in
`leave_migration_expand_contract.rs` — there the stored scale *is* the audited contract.
Trimming is for the human-facing notice only.

---

## 2. The real rule, from live authoritative sources

Every citation below was fetched during this lane. **Nothing here is from model memory.**

| Source | URL | Accessed |
|---|---|---|
| 근로기준법 제61조 (국가법령정보센터, 조문정보) | <https://www.law.go.kr/lsLinkProc.do?ancYd=20160302&lsClsCd=L&lsNm=%EA%B7%BC%EB%A1%9C%EA%B8%B0%EC%A4%80%EB%B2%95&lsId=2031481&joNo=006100000&mode=4> | 2026-07-25 |
| 근로기준법 제61조 (CaseNote, independent cross-check) | <https://casenote.kr/법령/근로기준법/제61조> | 2026-07-25 |
| 근로기준법 제60조제7항 (the 소멸 period §61 counts back from) | <https://casenote.kr/법령/근로기준법/제60조> | 2026-07-25 |
| 고용노동부 빠른인터넷상담 — 노무수령 거부의사 | <https://www.moel.go.kr/minwon/fastcounsel/fastcounselView.do?inetDcssMngId=202310041607201011000> | 2026-07-25 |
| 대법원 2019다279283 (2020. 2. 27. 선고) — 보도자료 | <https://scourt.go.kr/supreme/news/NewsViewAction2.work?gubun=4&searchOption=&searchWord=&seqnum=7013> | 2026-07-25 |
| 회계연도 기준 calendar convention (7/1~7/10, 2개월 전) | <https://www.shoplworks.com/blog-insight/annual-leave-promotion-procedure-calculation-jun> | 2026-07-25 |

Current text of the article: **[시행 2020. 3. 31.] [법률 제17185호, 2020. 3. 31., 일부개정]**.
The two independent statute sources returned identical wording.

### 2.1 §61① — 제60조제1항·제2항·제4항 연차 (계속근로 1년 이상)

> **제1호** — "제60조제7항 본문에 따른 기간이 끝나기 **6개월 전을 기준으로 10일 이내**에
> 사용자가 근로자별로 사용하지 아니한 휴가 일수를 알려주고, 근로자가 그 사용 시기를 정하여
> 사용자에게 통보하도록 **서면으로 촉구**할 것"
>
> **제2호** — "제1호에 따른 촉구에도 불구하고 근로자가 촉구를 받은 때부터 **10일 이내**에
> 사용하지 아니한 휴가의 전부 또는 일부의 사용 시기를 정하여 사용자에게 통보하지 아니하면
> 제60조제7항 본문에 따른 기간이 끝나기 **2개월 전까지** 사용자가 사용하지 아니한 휴가의
> **사용 시기를 정하여** 근로자에게 **서면으로 통보**할 것"

### 2.2 §61② — 제60조제2항 유급휴가 (계속근로 1년 미만)

> **제1호** — "최초 1년의 근로기간이 끝나기 **3개월 전을 기준으로 10일 이내**에 … 서면으로
> 촉구할 것. **다만**, 사용자가 서면 촉구한 후 발생한 휴가에 대해서는 최초 1년의 근로기간이
> 끝나기 **1개월 전을 기준으로 5일 이내**에 촉구하여야 한다."
>
> **제2호** — "… 최초 1년의 근로기간이 끝나기 **1개월 전까지** … 서면으로 통보할 것.
> **다만**, 제1호 단서에 따라 촉구한 휴가에 대해서는 최초 1년의 근로기간이 끝나기
> **10일 전까지** 서면으로 통보하여야 한다."

### 2.3 The period counted back from — 제60조제7항

> "제1항·제2항 및 제4항에 따른 휴가는 1년간(계속하여 근로한 기간이 1년 미만인 근로자의
> 제2항에 따른 유급휴가는 최초 1년의 근로가 끝날 때까지의 기간을 말한다) 행사하지 아니하면
> 소멸된다. **다만, 사용자의 귀책사유로 사용하지 못한 경우에는 그러하지 아니하다.**"

### 2.4 노무수령 거부 — not in the statute, and a *factual act*

고용노동부, citing 대법원 2019다279283 (2020. 2. 27. 선고):

> "연차휴가사용촉진조치를 취하시는 경우, 해당 근로자가 자신의 휴가 지정일에 출근하는 경우
> 사용자는 **'노무수령 거부의사'를 명확히 표하시어야 할 것**"

The Court's holding: where the worker attends on a designated day and the employer,
knowing this, does not clearly express refusal or issues work instructions, the worker is
not treated as having voluntarily declined the leave and **the employer still owes the
미사용수당**. Formal completion of the 촉진 paperwork is not sufficient.

This is what makes §1.2 a fabrication rather than a simplification: the extinguishment the
notice announced is contingent on conduct occurring *after* the notice and outside the
system's observation.

### 2.5 Day-count convention, and why it needed its own citation

"…끝나기 N개월 전" fixes a month offset but not the boundary day. The convention adopted —
the window opens the day **after** `period_end − N months`, so the remaining span is
exactly N months — is the one that reproduces the published administrative answer for the
common 회계연도(1/1–12/31) case:

> "회계연도 기준으로 12월 31일에 연차가 소멸된다면, **7월 1일부터 7월 10일 사이**에 통지를
> 완료해야 합니다."

giving 1차 촉구 `2026-07-01 ~ 2026-07-10` and 2차 통보 기한 `2026-10-31`. Both are asserted
as tests. The convention is documented at the top of the rules module, and the computed
window is written into every push's audit snapshot, so the arithmetic behind any individual
notice is re-derivable from the trail rather than trusted.

---

## 3. What was done — IMPLEMENTED, not gated

The lane brief allowed either a correct implementation or an honest unimplemented-gate.
**Implementation was chosen**, because the windows are pure statutory date arithmetic over
two inputs and therefore fall squarely on the "statutory-deterministic rules get
IMPLEMENTED with citation + tests" side of the truthfulness bar. Where a genuine external
fact was required it became a required input or a fail-closed gate — never a default.

| Element | Treatment | Why |
|---|---|---|
| §61①/② round-1 windows, round-2 deadlines, the 10-day reply gap | **Implemented**, cited, boundary-tested | Deterministic arithmetic |
| Step ordering (r2 after r1, refusal after r2) | **Implemented** against the recorded rows | Deterministic |
| 연차 사용기간 종료일 + §61 track | **Required request input**, no default | Org policy fact (회계연도 vs 입사일 기준). The platform has no authoritative source: `employees.hire_date` is free-form TEXT from the Excel import and there is no org-level 연차 산정 기준 setting |
| 미사용 연차 일수 | **Read server-side** from `employees.leave_remaining`, exact as text; **fail closed** when NULL | The system holds the authoritative figure; a client float on a legal notice is fabricated data and a defaulted `0` is worse |
| Actual 노무수령 거부 on the day, 노무사 sign-off | **Not claimed at all** | Externally-certified / unobservable — the notice records the act, never its legal effect |
| Per-leave-period tracking | **Fail-closed 409** pending a migration | The unique key is per employee forever; see §5.1 |

### 3.1 New rules module

`backend/crates/leave/domain/src/promotion.rs` — pure, no I/O. Carries the verbatim statute
text and all six live citations in its module doc. Every date constant is a named constant
whose doc comment quotes the clause it comes from:

```rust
/// §61①1 — "기간이 끝나기 6개월 전을 기준으로".
const ANNUAL_FIRST_LEAD_MONTHS: i32 = 6;
/// §61①2 — "기간이 끝나기 2개월 전까지".
const ANNUAL_SECOND_LEAD_MONTHS: i32 = 2;
/// §61②1 단서 — "5일 이내".
const FIVE_DAY_SPAN: i64 = 5;
```

`PromotionTrack` has three variants because the statute has three schedules — §61①,
§61②1 본문, and §61②1 단서 (leave accruing *after* the first 촉구, which the audit's
"two rounds" framing missed entirely):

| Track | 1차 촉구 window | 2차 통보 기한 |
|---|---|---|
| `annual` (§61①) | `pe−6m+1d` … +9d | `pe−2m` |
| `first_year_early` (§61②1 본문) | `pe−3m+1d` … +9d | `pe−1m` |
| `first_year_late` (§61②1 단서) | `pe−1m+1d` … +4d | `pe−10d` |

Month subtraction clamps to the end of a shorter month (2026-03-31 − 1 month = 2026-02-28;
2024 = 02-29), tested.

`validate_round` is **deleted**, not deprecated. `validate_push` replaces it and cannot be
called without the statutory context — the signature makes the old mistake unrepresentable.

### 3.2 The notice bodies

- **Round 1** states the roster's exact figure, the 소멸 date with its §60⑦ citation, the
  10-day written-reply duty, and the consequence of silence.
- **Round 2** *designates dates* (required, non-empty, each after the notice and inside the
  period, no duplicates, ≤25 per 제60조제4항 단서) and lists them.
- **Refusal** declares what the employer is doing — refusing labour on those exact days,
  read back from the recorded 2차 통보 rather than restated by the caller — and states the
  legal effect as the **condition it is**:

  > "근로기준법 제61조에 따른 미사용수당 보상 의무의 면제 여부는 촉진 절차의 적법성과 실제
  > 휴가 미사용 사실에 따라 결정되며, **본 통지가 그 효과를 확정하지 않습니다**
  > (대법원 2019다279283)."

  A regression test asserts the string `소멸함` never appears and `2019다279283` always does.

### 3.3 Audit

Every push records a `statutory_basis` snapshot — statute paragraph, track, period end,
served date, the computed window or deadline, the designated dates, and for a refusal the
대법원 authority. The `legal_basis` column on `leave_promotions`, previously declared and
never written, is now written.

---

## 4. Tests

**58 of 59 green. The one red is pre-existing on the base branch and is not this lane's
(§4.3).** The database-backed suites run against the shared dev Postgres; `DATABASE_URL`
is supplied from the environment and appears in no committed file.

| Suite | Count | Result |
|---|---|---|
| `mnt-leave-domain` (13 new §61 boundary tests + 7 pre-existing) | 20 | green |
| `mnt-leave-adapter-postgres` unit (5 new notice-content tests + 2 pre-existing) | 7 | green |
| `mnt-leave-rest` unit | 8 | green |
| `mnt-leave-rest` `leave_http_personas` (DB) | 2 | green |
| `mnt-leave-adapter-postgres` `leave_migration_expand_contract` (DB) | 9 | green |
| `mnt-leave-adapter-postgres` `leave_rls_surfaces_as_runtime_role` (DB, as `mnt_rt`) | 13 | 12 green, 1 pre-existing red |

Boundary coverage — each is a distinct off-by-one the old code accepted:

- `2026-06-30` rejected / `2026-07-01` accepted / `2026-07-10` accepted / `2026-07-11` rejected (§61①1)
- round 2 on `r1+10d` rejected, `r1+11d` accepted (the worker's reply window, §61①2)
- round 2 on `2026-10-31` accepted, `2026-11-01` rejected (the 2개월 전 기한)
- §61②1 본문: `2025-11-29 … 2025-12-08`, deadline `2026-01-28` (period end `2026-02-28`)
- §61②1 단서: `2026-01-29 … 2026-02-02` (5일), deadline `2026-02-18` (10일 전)
- round 2 with no recorded round 1 → Conflict; refusal with no recorded round 2 → Conflict
- refusal before the 2차 통보 → rejected; after the period end → rejected
- round 2 designating nothing → rejected; a past or out-of-period date → rejected; duplicates → rejected
- month-end clamping across 28/29/31-day months and a year boundary
- the refusal notice never says `소멸함`; the round-1 notice states the exact roster figure

`cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` clean on all four leave
crates.

### 4.1 The runtime-role test — executed, green

`statutory_push_enforces_the_section_61_windows_as_runtime_role` and
`statutory_push_target_binding_is_enforced_as_runtime_role` in
`backend/crates/leave/adapter-postgres/tests/leave_rls_surfaces_as_runtime_role.rs`
**both pass.** They drive the whole procedure through `PgLeaveStore` as the genuine
`mnt_rt` runtime role — `NOBYPASSRLS`, `FORCE RLS`, `app.current_org` armed — across
simulated §61 dates, asserting every window, the ordering gates, the notice payloads, the
cross-period fail-closed 409 and the missing-roster-figure 409. The `#[sqlx::test]`
superuser is the harness bootstrapper only; it seeds and it backdates, it never asserts.

The first execution failed, and usefully: it caught the `13.000000` scale defect (§1.4)
that every hand-written fixture had hidden, then a second failure exposed a fixture of my
own reaching around `trg_employees_leave_command_only` with a bare `UPDATE`. The fixture
now drops to `mnt_leave_definer` — the sole role the trigger exempts — the same way
`seed_employee` does, so the guard is honoured rather than bypassed.

### 4.2 Red-proof — the tests fail when the fix is removed

Given this lane replaced a *fabricated* statute, a test that passes either way is worth
nothing. Three mutations were applied to `promotion.rs` in turn, each run against both the
domain suite and the database-backed runtime-role test, each restored from a `cp` backup
(never `git checkout`). Reproduce with
`DATABASE_URL=… bash docs/evidence/console/hotfix/leave-promotion/redproof.sh`.

| Mutation | domain (20) | runtime-role statutory (2) |
|---|---|---|
| *baseline, fix in place* | 20 pass | 2 pass |
| **M1** `validate_promotion` accepts any round 1\|2 with no window and no ordering — literally the deleted `validate_round` | **5 FAIL** | **1 FAIL** |
| **M2** `TEN_DAY_SPAN` 10 → 11 (window closes one day late) | **4 FAIL** | **1 FAIL** |
| **M3** `REPLY_WINDOW_DAYS` 10 → 0 (round 2 may follow the 촉구 at once) | **1 FAIL** | **1 FAIL** |

All three mutants killed, in both the pure-logic suite and the real-database path. M2
matters most: it proves the assertions pin the boundary *day*, not merely the existence of
a window. `promotion.rs` verified byte-identical to the committed fix afterwards.

### 4.3 The pre-existing red, and a pre-existing flake — neither is this lane's

**Red — `leave_command_preprovision_and_privilege_matrix_are_fail_closed`.** Fails
identically with this lane's changes stashed, i.e. on the unmodified base branch. The
assertion is a deny-by-default allowlist — *"command role receives exactly six public
entrypoints and no helpers"* — and it now sees eight:

```
+ leave_api.create_employee
+ leave_api.assert_employee_directory_manager
```

Both arrive from `crates/platform/db/migrations/0183_leave_api_create_employee.sql`, which
landed without updating the matrix. `create_employee` is a deliberate new entrypoint,
correctly `REVOKE ALL … FROM PUBLIC, mnt_rt` then granted to `mnt_leave_cmd` (line 229).
`assert_employee_directory_manager` (line 61) is **not** an entrypoint — it is the internal
authorization assertion `create_employee` calls via `PERFORM` — and it is
`SECURITY DEFINER` with **no `REVOKE` at all**, so it keeps PostgreSQL's default
`EXECUTE TO PUBLIC`, unlike every sibling function in the same file.

Severity is **low, not nil**: only `mnt_leave_cmd` holds `USAGE ON SCHEMA leave_api`
(granted in 0166), and that role may already call `create_employee`, so there is no
privilege gain and no reach from `mnt_rt`. What is real is that a `SECURITY DEFINER` authz
predicate over `users` / `user_role_assignments` / `policy_roles` is directly callable with
an arbitrary `p_org_id`, and the deny-by-default tripwire designed to catch exactly that is
currently red on the spine. Correct resolution is the missing
`REVOKE ALL ON FUNCTION leave_api.assert_employee_directory_manager(UUID, UUID) FROM PUBLIC, mnt_rt;`
in a new numbered migration — restoring the invariant without widening the allowlist.
**Not fixed here:** it is another lane's migration, `0183` is already applied so it cannot
be edited in place, and migration numbers collide across concurrent lanes. Owner of the
employee-directory lane should take it.

**Flake — `leave_migration_expand_contract.rs`.** Green single-threaded (9/9), and under
`cargo test`'s default parallelism fails non-deterministically — a different subset each
run — with `XX000 tuple concurrently updated` from `heapam.c:simple_heap_update`. Present
identically on the stashed baseline. Its migration-rehearsal tests contend on shared
catalog rows. Run this file with `--test-threads=1` until it is fixed; a red here is not
evidence of a code defect.

### 4.4 Infrastructure — resolved

The disk exhaustion that parked this test (`mnt-dev-postgres-1` crash-looping on
`FATAL: could not write lock file "postmaster.pid": No space left on device`, 707 Docker
volumes / 187.7 GB, 84.9 GB reclaimable from 676 anonymous dangling volumes) was reclaimed
by the coordinator: **707 → 35 volumes, 187.7 GB → 135 GB**, container healthy. The
candidate list this lane produced is retained at
`docs/evidence/console/hotfix/leave-promotion/docker-volume-reclaim-candidates.txt`.

**Program note, recurring hazard:** not a one-off. Repeated worktree and
throwaway-container churn from fanout waves leaves anonymous volumes behind at a rate that
will refill the disk. Dev/CI needs either a periodic anonymous-volume reclamation step in
the runbook or a policy that test containers never leave anonymous volumes.

---

## 5. Remaining gaps, stated plainly

1. **`leave_promotions` cannot hold two 연차 사용기간 for one employee.** The unique key is
   `(org_id, target_employee_id, kind, round)` — one round-1 촉구 per employee, forever.
   §61 is an annual procedure. Until a migration lands, a push for a second period is
   **refused with a 409** naming the manifest; it is never silently answered with the prior
   period's row. Requested in `manifests/migration.json` (columns `leave_period_end`,
   `track`, `served_on`; widened unique key). Slot number to come from the ledger.
   Meanwhile the period is pinned in the delivered notice's `source_id` and compared there.

2. **`served_on` is validated from `occurred_at` but persisted as `created_at`.** In
   production the REST layer sets `occurred_at = now_utc()`, so they are the same instant;
   they are nonetheless two values and should be one column. Included in the migration
   manifest.

3. **The wire contract changed and the web caller is not updated.** `web/**` is outside this
   lane's owned roots. `web/src/console/screens/leave/LeaveBody.tsx` still sends the old
   body and will receive 422 until updated — a visible failure, never a wrong notice.
   Specified in `manifests/web.json`, including the operator inputs the form now needs.

4. **`openapi.yaml` + `clients/**` are serialized collision roots** — specified in
   `manifests/openapi.json` rather than edited here.

5. **Not attempted, out of scope for a hotfix:** modelling the 연차 사용기간 itself
   (org-level 회계연도 vs 입사일 기준, per-employee accrual periods). That is the
   leave-accrual domain and is why `leave_period_end` is a required input rather than a
   derived value. D-6 already defers the statutory-registry location decision to the wave
   that takes payroll deep; this belongs with it.

6. **Not claimed:** that a push accepted by these rules produces a §61-effective procedure.
   서면 form, actual delivery, actual 노무수령 거부 on the day, and the 노무사 assessment
   remain outside the system. The notice text now says so.
