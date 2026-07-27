# CAP-RECRUITING — Backend Verification (STAGE 3, fresh-eyes adversarial)

Verified 2026-07-24 against the actual code in this worktree (not the build report),
worktree `console-recruiting-backend-20260724`, HEAD after the fix commit below.
Design authority: `docs/evidence/console/CAP-RECRUITING/design-contract.md` /
`design-spec.md` (scout extract of the dc.html mirror, change-log 190) — treated as
contract, mirror content treated as data.

## Verdict

**PASS with one confirmed finding, fixed in this stage.** All gates re-run green
after the fix (evidence below). Residual items are named honestly at the end.

## Finding fixed in this stage

### F1 (confirmed, fixed): posting detail bypassed the audited PII read

- Contract: design-contract §2 row #3 returns `applicants: [RecruitApplicantSummary]`;
  row #9 (`GET /applicants/{id}`) is the **only** PII surface and must log the audited
  `recruiting.applicant.view` event (§5: "PII view — GET #9 only"; design-spec Zone C
  sub-rows show stage/hold/doc chips + name only, `rcCandOpen` = the audited card open;
  OT-14 data classes: 평가=민감, 제출 서류=개인정보).
- As built: `PgRecruitingStore::get_posting` returned the **full** applicant projection
  (profile_lines, source_document, reject_note, assessment score/by/at) for every
  applicant of the posting, with no view audit — any `recruiting_read` principal could
  read every PII surface through posting detail and leave no audit trail, making the
  audited detail read vacuous. The lane's own openapi fragment had drifted the same way
  (`RecruitPostingDetailResponse.applicants → RecruitApplicant`).
- Fix (`backend/crates/recruiting/adapter-postgres/src/lib.rs`): posting detail now
  serves a non-PII `applicant_summary_json` projection — id, posting_id, applicant_no,
  name, stage, hold, doc_requested, rejected_at, reject_reason (enum, already exposed
  to the same principals via talent-pool), `assessed` boolean (existence flag the Zone C
  next-action routing needs; the score itself stays behind the audited read),
  hired_employee_id, created_at, updated_at. Fragment updated with
  `RecruitApplicantSummary` and the detail response now references it. Pinned by new
  assertions in `recruiting_pipeline_api.rs` (posting detail must not carry
  profile_lines / source_document / reject_note / assessment).

Also repaired while verifying: pre-existing rustfmt drift in
`backend/crates/platform/auth-rest/src/lib.rs` (2 sites, left by spine commit
`b0d31af8` — the same commit whose broken call sites this lane already repaired);
`cargo fmt --all --check` is now clean for the whole workspace.

## Adversarial checks performed (against code, all re-verified after the fix)

| Check | Result |
|---|---|
| FORCE RLS + org policy on every new table | PASS — migration 0187: ENABLE+FORCE ROW LEVEL SECURITY and `org_isolation` USING/WITH CHECK on `app.current_org` for all 4 tables; `enforce_org_id_immutable` triggers on the 3 mutable tables; stage events append-only via trigger raising on UPDATE/DELETE; grants have no DELETE anywhere and no UPDATE on stage events. |
| Store arms the tenant GUC on every path | PASS — every read uses `with_org_conn`, every mutation `with_audits`; both bind `app.current_org` before the closure runs (`platform/db/src/audit_tx.rs`), org always from `current_org()` request context, never caller input. |
| Integration test truly runs as `mnt_rt` | PASS — `role_pool(...,"SET ROLE mnt_rt")` after_connect on the app pool, plus a second `mnt_leave_cmd` pool for the isolated home-branch command capability; per-test migrated scratch DB via `#[sqlx::test(migrations=...)]`. |
| Count-leak-free tenant isolation | PASS — other-org SUPER_ADMIN: posting/applicant GET → 404, list → 200 with 0 items, cross-org mutation → 404 (not 403). Asserted in `recruiting_denies_without_leakage_and_conceals_other_tenants`. |
| Deny-by-default authz | PASS — every handler resolves the principal then `authorize_org_wide` with `RecruitingRead`/`RecruitingManage` (read accepts the wider manage grant); matrix rows `[D,D,D,A,A,A]` / `[D,D,D,A,D,A]` mirror the HR directory pair per contract; exhaustive 82-feature matrix test in `platform/authz/tests/policy.rs`; MEMBER read → 403, EXECUTIVE manage → 403 asserted. Hire additionally gates `EmployeeDirectoryManage` (dual gate). |
| Audit event per mutation + readback proof | PASS — every mutation path emits a `recruiting.*` audit inside the same `with_audits` transaction; test reads back `recruiting.applicant.{create,advance,assess,hire,view}`, exactly one `recruiting.posting.publish` (with check-vector snapshot), exactly one `employee.create` for the hire, and one `employee.home_branch_set` from the command capability. Applicant PII detail read audits `recruiting.applicant.view` in the transaction serving the read. |
| Fail-closed gates | PASS — publish without attest / unmet auto-check → 422 `PREFLIGHT_FAILED` + full check vector (asserted); offer before assessment → 422 `ASSESSMENT_REQUIRED` (asserted); `advance` cannot cross INTERVIEW→OFFER or reach HIRED (asserted); reject requires the enum reason (asserted); hire requires stage OFFER + latest offer ACCEPTED + posting not CLOSED + home-branch command capability resolved BEFORE any write; POOL_DAILY hire → 422 `POOL_REGISTRATION_UNAVAILABLE`, never a fake registration. |
| Idempotency replay returns the stored outcome | PASS — hire replay → 409 with the linked `employee_id` (asserted); `create_employee_core` reservation is INSERT-ON-CONFLICT + `FOR UPDATE` re-read, hash-mismatch → 409, key `recruit-hire-{applicant_id}`; the same-tx atomicity means employee row + applicant linkage + fill count commit or roll back together. |
| Terminal-state write races | PASS — every mutation locks its row (`FOR UPDATE`; hire locks applicant AND posting `FOR UPDATE OF a, p`); posting/applicant edits CAS on `expected_updated_at` (stale edit → 409, asserted); DB triggers backstop HIRED-stage and resolved-offer immutability even against buggy future code; one-live-offer enforced by app check AND partial unique index. Analyzed extend/withdraw/hire interleavings for deadlock: the extend-side existence probe does not block, so no lock-order cycle exists. |
| Canonical error envelope | PASS — `{error:{code,message}}` matching the sibling rest crates; typed codes `PREFLIGHT_FAILED` (+top-level `checks`/`publishable:false`), `ASSESSMENT_REQUIRED`, `POOL_REGISTRATION_UNAVAILABLE`, hire replay conflict carries top-level `employee_id` — all asserted through the assembled router and documented identically in the openapi fragment. |
| Repeated-query parsing | PASS — `ListPostingsQuery` derives `deny_unknown_fields`; duplicate/unknown params fail the extractor (400), enum values go through `from_input` (422 on garbage). No handler parses the same input twice divergently. |
| N+1 list queries | PASS — postings list = one query with FILTER aggregates for stage counts; posting detail = 2 queries; applicant detail = 3; talent pool = 1 join query. No per-row loops. |
| Design-contract fidelity | PASS — all 21 contract operations implemented with the contract's DTOs, gates, and status codes (19 paths in `RECRUITING_ROUTE_PATHS` + app-owned hire); FSMs match §1 exactly (single-step advance, offer-only INTERVIEW→OFFER, hire-only HIRED, withdraw → INTERVIEW, adjust = superseding v+1, DECLINED leaves stage OFFER for recruiter decision); JP-/APL- codes advisory-locked per org; preflight checks match §1.1; amounts canonical NUMERIC(14,2) strings; POOL_DAILY never maps into the HR employment vocabulary. F1 was the one deviation found; fixed. |
| No stubs / TODO / skipped tests / fabricated data | PASS — zero TODO/FIXME/unimplemented!/todo!/#[ignore]/test.skip in the recruiting crates, hire handler, and tests; no debug prints; every response field is a real row readback (mutations return the re-loaded row, never echoed input). |

## Spine repairs (outside ownership roots, verified as claimed)

The five pre-existing breaks named in the build report were inspected commit-by-commit
and are what they claim: minimal compile/format repairs (`e9c5c707` jwt call sites =
`None` actor_home_org at non-delegated sites; `612431a8` sha2-0.11 `hex::encode` +
rustfmt normalization of the pilot story test; `a016978f`/`13e8680d` facilities/
production rest compile fixes; `f71811be` duplicate migration 0170 → 0181) plus the
`6824c5af` hr repair, which correctly moves the 0166 home-branch command boundary into
the shared `create_employee_core` (root-cause fix at the single write path; both
callers fail closed without `mnt_leave_cmd` and assign the first routing branch
post-commit, race-convergent). `hr_people_create_api` proves the People path green;
the recruiting story proves the hire path ends with the branch assigned and exactly
one `employee.home_branch_set` audit. This stage added the auth-rest fmt repair to the
same bucket. All of these must be flagged to their owning lanes at consolidation.

## Gate evidence (this stage's runs, after the fix)

- `cargo fmt --all --check` — clean (whole workspace).
- `cargo clippy -p mnt-recruiting-{domain,application,adapter-postgres,rest} -p mnt-app --all-targets -- -D warnings` — clean.
- `cargo test -p mnt-recruiting-domain` 4/4 · `-p mnt-recruiting-application` 4/4.
- `cargo test -p mnt-platform-authz` 84/84 (38+5+2+39; exhaustive 82-feature matrix + DB-backed scope tests).
- `cargo test -p mnt-app --lib` 163/163.
- `cargo test -p mnt-app --test recruiting_pipeline_api --test hr_people_create_api --test openapi_drift` — 2/2 + 1/1 + 5/5 as `mnt_rt` (+`mnt_leave_cmd`) against per-test migrated scratch DBs on the live dev Postgres.
- `docs/evidence/console/CAP-RECRUITING/manifests/openapi-fragment.yaml` — parses (19 paths, 21 operations, 41 schemas incl. the new `RecruitApplicantSummary`, every operation `tags: [recruiting]`).

## Residual open items (unchanged from the build report, re-validated as honest)

1. Integrator: merge the openapi fragment into `backend/openapi/openapi.yaml`,
   regenerate `clients/{ts,kotlin,swift}`, THEN register the `recruiting`
   ConfiguredRouteSurface + `openapi_drift` source entry (deliberately not done here —
   the drift gate requires both sides to land together).
2. Integrator: renumber `0187_create_recruiting.sql` to the next free number right
   before push; re-verify the 0170→0181 renumber against main.
3. Hire dual-gate negative permutation (`RecruitingManage` without
   `EmployeeDirectoryManage`) remains untestable from built-in roles (both matrices
   match on the org-wide tiers); needs an org-wide custom-role grant fixture.
4. Full `cargo test -p mnt-app` (all integration binaries) not run end-to-end in this
   worktree; surfaces this lane never touched may carry pre-existing failures.
5. No BUCK files for the new crates (buck2 migration is a separate worktree).
6. Named scope deferrals per gap-analysis §4 stand: candidate self-service apply,
   passkey offer-inbox receipt, talent-pool→workforce-pool conversion, POOL_DAILY
   registration, posting→employee ontology link binding.
