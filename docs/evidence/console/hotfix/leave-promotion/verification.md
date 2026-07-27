# Stage-2 verification — hf-leave-promotion (§61 연차 사용 촉진)

Independent adversarial pass. The verifier did not write the build-stage code and
checked the code, not `report.md`. Verified at build-stage tip `b171d35d`; two
verification commits follow it.

---

## 1. Did the change fix the root cause?

**Yes.** The defect was not "the refusal notice says the wrong thing" — that was the
symptom. The root cause was that `validate_round` (domain/src/lib.rs, deleted) accepted
`round ∈ {1,2}` with **no reference to any date**, so the store delivered a §61 notice on
any day the operator clicked, and `PromotionKind::Refusal => Ok(2)` normalised any input
without checking a round-2 통보 existed.

The fix moves the decision to the one place every caller routes through —
`PgLeaveStore::statutory_push` calls `validate_push` before `inbox.emit`, so no notice is
delivered unless the window holds. Re-derived by reading the call graph, not the report:
`statutory_push` is the sole writer of `leave_promotions`, and both REST handlers
(`push_promotion`, `push_refusal`) funnel into it. There is no second path.

Confirmed the guard sits **before** delivery, not after: `validate_push` at
adapter-postgres/src/lib.rs is evaluated before `NewInboxDoc::new` and before
`self.inbox.emit(...)`.

## 2. Red-proof — run independently by the verifier

The build stage committed `redproof.sh`. Rather than re-run its script, the verifier
applied its own mutation to a *different* file (`domain/src/lib.rs`, restoring the exact
deleted `validate_round` body inside `validate_push`) and watched the database-backed test
go red, then restored from a `cp` backup.

```
# mutation: validate_push ignores the context, accepts round ∈ {1,2}
running 2 tests
test statutory_push_enforces_the_section_61_windows_as_runtime_role ... FAILED
  panicked at leave_rls_surfaces_as_runtime_role.rs:2303:
  2026-06-30 is one day before the 1차 촉구 window opens:
    StatutoryPushView { kind: Promotion, round: 1, inbox_doc_id: 2cbf3b9e-…, … }
test statutory_push_target_binding_is_enforced_as_runtime_role ... ok
test result: FAILED. 1 passed; 1 failed

# same mutation, pure-logic suite
tests::push_validation_routes_each_kind_to_its_statutory_rule ... FAILED
test result: FAILED. 19 passed; 1 failed

# restored (cp backup, never `git checkout`)
git diff --stat backend/crates/leave/domain/src/lib.rs  →  (empty)
```

The failure message is the proof that matters: with the fix removed the store **returned a
delivered `inbox_doc_id`** for a notice served outside its statutory window. The test is
not a tautology.

## 3. The blocker the build stage reported is closed

The build stage reported `partial` because the runtime-role test had never executed (Docker
disk full; `mnt_buck_admin` credential unavailable). The verifier resolved this without
touching the shared dev stack: a disposable Postgres carrying the Buck harness identity,
provisioned exactly as `tools/buck/test_needs_postgres.sh` does it.

```
docker run -d --rm --name mnt-verify-pg-leavepromo -p 127.0.0.1::5432 \
  --env-file <(POSTGRES_USER=mnt_buck_admin …) postgres:18.4@sha256:65f70a15…
docker exec … bash /topology.sh        → seven application roles reconciled and verified
DATABASE_URL=postgres://mnt_buck_admin:…@127.0.0.1:<port>/mnt_buck_test_leavepromo\
  ?options%5Bmnt.sqlx_test_bootstrap%5D=buck-sqlx-superuser-v1
```

`dev-up down/up` was never run; the shared `mnt-dev-*` stack was not touched. Migration
0196's superuser bootstrap gate is satisfied by the container's own `mnt_buck_admin`, so no
credential had to be obtained or guessed.

## 4. Defects found by verification, and fixed

### 4.1 The 2차 통보 claimed more than it designated — `05511aa1`

The same class of defect the lane exists to remove, still present in the fixed code. With
`leave_remaining = 13.5` and two designated dates, the round-2 notice body read:

> 미사용 연차 **13.5일의 사용 시기**를 아래와 같이 지정하여 통보합니다.
> 지정 사용 시기: 2026-12-23, 2026-12-24

The prose claims the notice designates all 13.5 days; the payload it sits in lists two. A
legal notice contradicting its own payload. Nothing enforced or reconciled the two, and the
runtime-role test drove exactly this shape and passed.

Fixed to `미사용 연차 13.5일 **중 아래 날짜에 대하여** 사용 시기를 지정하여 통보합니다` — true
for a full or a partial designation. Both the new phrasing and the absence of the old one
are pinned in `statutory_notice_tests::a_round_two_notice_states_the_designated_dates_it_claims_to_designate`.

Same commit normalises the notice citation to the form the statute and this crate's own
error messages already use: `근로기준법 제61조제1항제1호`, not `제61조제1항 제1호`.

### 4.2 The transport boundary had no test — `41b5da85`

The lane changed both §61 request bodies (removed `unused_days`; added required `track` and
`leave_period_end`; added `designated_dates`) and rewired `push_refusal`, but every proof
sat **below** REST. `crates/leave/rest/tests/leave_http_personas.rs` — a real-router
harness with signed JWTs and a genuine `mnt_rt` pool — already existed and covered none of
it.

Added `statutory_push_requires_its_statutory_inputs_over_http`:

| case | asserted |
|---|---|
| the body the console sends today (`unused_days`, no `track`/`leave_period_end`) | 422 — fail closed, proving `manifests/web.json`'s blocking claim |
| `track: "annual_leave"` | 422 `{"error":{"code":"validation"}}` |
| a 사용기간 whose 1차 촉구 window has not opened | 422, message contains `1차 촉구는 …` so the operator sees the window |
| an in-window round 1 | 200, `round: 1`, `ap_submission: pending_engine_definition`; `inbox_docs` row is an **unconfirmed** `legal_notice` / `연차촉진` addressed to the target, `payload.unused_days == "13"` — the roster figure, not the `13.0` the caller supplied |

The happy path derives its `leave_period_end` from today via `first_round_window` instead of
a literal date: the handler stamps `occurred_at` from the clock, so a hard-coded period end
would pass this week and fail next.

## 5. Enterprise bar — checked, not assumed

| Bar | Finding |
|---|---|
| RLS as `mnt_rt` | No new tables. Both new reads (`find_promotion`'s `leave_promotions ⋈ inbox_docs` join, `trim_scale(leave_remaining)`) run inside `with_org_conn`, so `app.current_org` is armed; `inbox_docs` RLS is org-scoped (0119), so the join does not silently return empty for a non-recipient admin. Both statutory tests assert as `mnt_rt`. |
| Deny-by-default | Every new branch fails closed: no round 1 → 409; no round 2 → 409; no roster figure → 409; a different 연차 사용기간 → 409; outside a window → 422; round 1 carrying dates → 422; round 2 with none → 422. Verified by reading each arm and by the runtime-role assertions. |
| Audit | `leave_promotion.push` audit event now carries `unused_days` and a `statutory_basis` snapshot (track, period end, served-on, computed window or deadline, designated dates), so the arithmetic is re-derivable from the trail alone. |
| Canonical envelope | Unchanged `RestError::from_kernel`; `Validation → 422 "validation"`, `Conflict → 409 "conflict"`. Now asserted over HTTP. |
| Idempotency | The replay path returns the recorded row without re-delivering; the period-tagged `source_id` makes a *different* period a 409 rather than a false idempotent hit. Both proven in the runtime-role test. |
| No fabricated values | `unused_days` is `trim_scale(leave_remaining)::text` from the roster; a NULL is a 409, never a printed `0`. The refusal reads its dates back out of the recorded 2차 통보 payload. |
| Statutory citations | Verified present and live-sourced (law.go.kr, casenote.kr, moel.go.kr citing 대법원 2019다279283, scourt.go.kr, shoplworks) with accessed dates in the module header. The three windows were re-derived by hand against the quoted text and agree, including the month-end clamping and the 회계연도 published answer (7/1–7/10, 기한 10/31). |
| Stubs / TODO / skip | `grep -rn "TODO\|FIXME\|unimplemented!\|todo!\|placeholder\|XXX"` over `backend/crates/leave/` and the evidence dir: no hits. No `#[ignore]`. |
| Roots | 16 files across the lane's commits, all under `backend/crates/leave/**` and `docs/evidence/console/hotfix/leave-promotion/**`. `git diff --name-only 4cabe239..HEAD -- backend/crates/platform/db/migrations/` → 0. No shared collision root touched; three manifests written instead, plus this pass's fourth. |

## 6. Gate set, re-run at the verification tip

```
cargo fmt --check -p mnt-leave-domain -p mnt-leave-application \
                  -p mnt-leave-adapter-postgres -p mnt-leave-rest        → clean
cargo clippy      (same four crates, --all-targets)                      → 0 warnings
cargo test -p mnt-leave-domain                                           → 20 passed
cargo test -p mnt-leave-application                                      → 0 tests
cargo test -p mnt-leave-rest --test leave_http_personas                  → 3 passed (1 new)
cargo test -p mnt-leave-adapter-postgres --lib                           → 7 passed
cargo test -p mnt-leave-adapter-postgres --test leave_rls_surfaces_as_runtime_role
                                                                         → 12 passed, 1 failed
```

The one red is `leave_command_preprovision_and_privilege_matrix_are_fail_closed`, and it is
**not this lane's**. Verified independently of the build stage's claim: the lane changed no
migration (`git diff --name-only 4cabe239..HEAD -- …/migrations/` → 0 files) and did not
touch that assertion (`git diff 4cabe239..HEAD -- <test file> | grep -c "exactly six"` → 0).
The assertion reads only database grant state, which comes from migrations alone, so it is
red on the base branch too.

Root cause re-derived: `0183_leave_api_create_employee.sql` creates the `SECURITY DEFINER`
helper `leave_api.assert_employee_directory_manager` (line 61), sets its owner (line 125),
and — unlike every sibling in the same file — issues **no `REVOKE`**, leaving PostgreSQL's
default `EXECUTE TO PUBLIC`. `grep -rn assert_employee_directory_manager …/migrations/*.sql`
returns exactly three lines, none of them a GRANT or REVOKE. Filed as
`manifests/privilege-revoke.json`.

## 7. Open items after verification

1. **Round 2 does not verify the worker failed to reply.** §61①2 conditions the employer's
   designation on the worker not having 통보-ed within 10 days. There is no modeled reply
   channel, so only elapsed time is checked, and the notice recites `1차 촉구에 대한 회신이
   없어` — the employer's own assertion, matching the MOEL standard form, but unverified by
   the system. Left as-is deliberately; the honest resolution is an operator attestation in
   the UI, not a fabricated data check.
2. **The reply window is anchored on the served date, not the receipt stamp.** Correct under
   도달주의, but this system's own 0119 migration defines receipt as the confirmation stamp
   (`열람 = 법적 수령`). Anchoring on `confirmed_at` would deadlock 촉진 whenever a worker
   never opens the notice, so served-on is the right default — but the two models should be
   reconciled explicitly rather than by accident.
3. **The refusal has no upper date bound relative to the designated days.** It may be served
   any time up to `period_end`, so a notice refusing labour on days already past is
   accepted. Deliberately not fixed: §61 imposes no timing on the refusal, and inventing one
   would repeat the lane's original sin. Worth a UI-level warning.
4. **Nothing bounds `designated_dates.len()` against the remaining balance.** An operator can
   designate 20 days for an employee with 5 left; the notice then states both figures
   truthfully and contradicts itself only by arithmetic. Cheap guard, deferred as it is an
   input-validation nicety, not a statutory rule.
5. **Legacy `leave_promotions` rows are permanently blocked.** Rows written before this lane
   carry the old `source_id` shape, so `ensure_same_period` 409s every further push for that
   employee until the migration lands. Fail-closed and correct — those notices were legally
   void anyway — but it is a real consequence, and the build-stage manifest calls the
   migration "non-blocking" without saying so.
6. Items 1–6 of `report.md` §5 stand as written and were re-checked.
