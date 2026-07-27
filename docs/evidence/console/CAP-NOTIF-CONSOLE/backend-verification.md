# CAP-NOTIF-CONSOLE — Stage-3 backend verification (fresh-eyes adversarial)

Verified 2026-07-24 in worktree `console-notif-backend-20260724` (HEAD after
`d0add023`). Verifier did not write the stage-2 code; every claim below was
re-proven against the actual code and live runs, not the stage-2 report.

## Verdict

**PASS at crate layer, with one fix applied during verification** (rustfmt on
the lane's own E2E test file). The app-level E2E remains blocked by other
lanes' broken crates — re-confirmed, not assumed. No stubs, no placeholder
markers, no skipped tests, no fabricated values found in lane-owned code.

## What was re-run (fresh, this stage)

| Check | Command / scope | Result |
|---|---|---|
| Format | `cargo fmt --check` over all lane-touched crates | RED on `backend/app/tests/notif_routing_api.rs` (lane-owned) → fixed (`d0add023`); re-check `LANE_FMT_CLEAN` on all 11 lane-touched files |
| Lint | `cargo clippy --all-targets -D warnings -p mnt-notifications-{domain,application,adapter-postgres,rest} -p mnt-platform-realtime -p mnt-platform-auth` | GREEN (exit 0) |
| Domain tests | `cargo test -p mnt-notifications-domain` | 5/5 GREEN |
| Adapter tests | `cargo test -p mnt-notifications-adapter-postgres` vs dev Postgres 55432 (sqlx::test scratch DBs via `mnt_cluster_admin`; store runs on a genuine `SET ROLE mnt_rt` pool) | 8/8 GREEN in 4.03s |
| REST tests | `cargo test -p mnt-notifications-rest` (real router, ES256 JWT, mnt_rt pool via `mnt_platform_test_support::runtime_role_pool`) | 2/2 GREEN |
| App E2E | `cargo check -p mnt-app --tests` | STILL BLOCKED: `mnt-production-rest` 4 errors (`E0063 plan_digest`, E0308, E0382, lifetime) + `mnt-facilities-rest` 3 errors (`E0106`, `E0277 DueCaseBody: Serialize`, `E0599 Hmac::new_from_slice`) — other lanes' mid-flight crates, untouched per hot-check discipline |
| Placeholder scan | grep TODO/FIXME/unimplemented!/todo!/.skip/#[ignore] over `backend/crates/notifications/**`, `backend/app/tests/notif_routing_api.rs`, evidence dir | zero hits |

Note on the stage-2 report's test claim: reproducing it required the
`mnt_cluster_admin` dev URL — `mnt_app@55432` lacks CREATEDB, so
`sqlx::test` cannot create scratch DBs under it. With the admin URL every
claimed suite reproduces exactly (8/8, 5/5, 2/2).

## Adversarial checks against the code (not the report)

- **FORCE RLS + org policy on the new table**: `0196_notification_policies_and_object_agg.sql`
  has `ENABLE`+`FORCE ROW LEVEL SECURITY` and the `org_isolation` USING/WITH CHECK
  policy byte-identical in shape to `0099_create_notifications.sql`; grants
  SELECT/INSERT/UPDATE/DELETE to `mnt_rt` (DELETE is required — unmute is a real
  removal). DB CHECK enforces scope-shape exclusivity; the unique expression index
  `(org_id, user_id, action, scope, COALESCE(category,''), COALESCE(link::text,''))`
  is the upsert conflict target, matched verbatim by the `ON CONFLICT` clause.
- **Tests genuinely run as mnt_rt**: both the adapter suite and the REST suite build
  a second pool with `after_connect → SET ROLE mnt_rt` and hand THAT pool to the
  store; the owner pool is used only for seeding/readback. Cross-tenant assertions
  execute under a second org's GUC and assert zero rows/zero count (count-leak-free),
  and cross-user mark/toggle/delete assert `NotFound`.
- **Deny-by-default**: every route resolves the principal from the JWT before any
  data path; recipient is never taken from request input; anon → 401; cross-user and
  cross-tenant ids → 404 with the same envelope as truly-absent (E2E asserts the
  tuple equality). No admin/bypass path exists on this surface.
- **Audit per mutation with readback**: emit / read / unread / read_all / resolve /
  policy_set / policy_clear all route through `with_audit(s)`; adapter tests read
  `audit_events` back by action (`notification.unread` = 1, `policy_set` = 3
  incl. idempotent re-set, `policy_clear` = 1 only for the successful delete —
  the cross-user delete rolls back and leaves no audit row).
- **Fail-closed gates**: forged/undecodable by-object cursor → empty 200 page, never
  an error channel (test-proven); flat-list foreign cursor makes the keyset subquery
  NULL → empty page; `PgNotificationError::kind()` defaults unknown DB errors to
  Internal; `from_store` never leaks sqlx detail (logs server-side, generic body).
- **Idempotency replay returns the stored outcome**: dedup emit race is handled by
  `ON CONFLICT … DO NOTHING` + a rollback sentinel + read-back of the winner
  (notifier fires exactly once — test-proven); policy PUT re-upsert returns the same
  row id (test-proven). The single-chokepoint mute predicate (`MUTED_PREDICATE_SQL`)
  is the one SQL string reused by list/counts/summary/emit/latest — routing
  semantics cannot fork across paths.
- **Rejection classes**: no N+1 (by-object is one SQL statement with two LATERALs);
  duplicate/garbled query params fail axum `Query` parsing closed (400), same as the
  pre-existing family routes; error envelope is the canonical `{error:{code,message}}`;
  no terminal-state race surface (read/unread is a deliberate toggle per the design
  FSM; resolved never implies read; emit race covered above).
- **Design-contract fidelity**: the five chartered routes in
  `design-contract.md` §"mnt-notifications-rest" match the implementation and the
  OpenAPI fragment 1:1 (paths, operationIds, DTO fields, error codes). FSM 1a/1b
  behaviors (forensic `read_at`, mute = attention-only, direct-apply audited
  personal setting) are each covered by a passing test. "open-linked-object → read"
  is a frontend ack path (client calls mark_read), not a chartered API.
- **`unreachable!` in `emit_notification`**: pre-existing at stage-1 base
  (`c7c386fd`), logically dead (the closure maps `None` → `Dedup` sentinel);
  not a lane regression.

## Cross-lane facts re-confirmed (unchanged from stage 2)

- The three spine-repair commits (`23afd72b` auth args, `6c10dc2f` 0170→0181
  renumber, `a06b9dad` logistics hex) are each minimal and correctly flagged in
  `integration-manifest.json` for integrator arbitration.
- `mnt-logistics-adapter-postgres` still fails `clippy -D warnings` on two
  PRE-EXISTING lints (`expect_used` at src/lib.rs:287, `too_many_arguments` at
  :296) that were unreachable before the hex fix unblocked compilation. They are
  the logistics lane's code; widening `a06b9dad` to fix them would grow the
  out-of-root diff, so they are left for that lane/integrator.
- Pre-existing fmt violations in `backend/app/tests/{cedar_freshness_mint,logistics_pilot_story}.rs`
  and `backend/crates/platform/auth/tests/jwt_es256.rs` come from other lanes'
  base commits — not touched.

## Known bounds (accepted, not defects)

- `NotificationLink` has no max-length bound (DB CHECK is `jsonb_typeof='object'`
  only, inherited from 0099); a pathologically large link from an internal producer
  would fail the new btree index insert with a 500 at emit time. Producers are
  internal code, not a trust boundary; flagged for the eventual link-shape CHECK.
- `muted:false` on the realtime wire is truthful modulo the emit→fan-out race
  window (a mute created in between); the next read-path annotation corrects it.

## Open items

1. Run `backend/app/tests/notif_routing_api.rs` once `mnt-production-rest` and
   `mnt-facilities-rest` compile (their lanes' fixes) — it is committed, rustfmt-clean,
   and cannot be compile-checked until `mnt-app` builds.
2. Integrator actions unchanged from stage 2: renumber 0196; arbitrate/drop the three
   spine-repair commits; apply the OpenAPI fragment with ONE tag for the family and
   regenerate `clients/{ts,kotlin,swift}` (3 drift gates); merge `backend/Cargo.lock`.
3. Logistics lane: clear the two pre-existing clippy lints exposed by the hex fix.
