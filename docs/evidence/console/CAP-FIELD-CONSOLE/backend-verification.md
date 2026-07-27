# CAP-FIELD-CONSOLE — Stage 3 backend verification (fresh-eyes adversarial)

Date: 2026-07-24. Verifier: independent session (did not write the stage-2 code).
Scope: commits `6f7e8907..0345cfd5` (migration 0194, support domain/application/
adapter/rest, `backend/app/tests/field_visit_api.rs`, openapi fragment,
integration manifest) verified against actual code and live runs — not against
the build report.

## Verdict

Stage-2 work is real and largely sound: FORCE RLS + org policy + org-immutable
trigger + append-only `mnt_rt` grants on the new table are present in 0194;
the integration suite genuinely runs the assembled router as `mnt_rt`
(`SET ROLE` on every pooled connection); idempotency replay returns the stored
row without re-audit or re-notification; every mutation lands audit events with
readback assertions proving `accepted_by` never enters snapshots; scope
predicates fail closed (`Branches([]) → FALSE`); list counts derive from the
same scoped aggregation as the rows; no stubs, TODOs, skipped tests, or
fabricated calculations anywhere in the lane's files.

Three defects were found and FIXED in this pass (below). All gates re-run green
after the fixes.

## Findings fixed in this pass

### F1 — field read routes authorized `Feature::Login` instead of the design gate (CONFIRMED, high)

`GET /api/v1/field/sites` and `GET /api/v1/field/sites/{id}` authorized
`Feature::Login`, which the authz matrix grants to ALL roles — including
`MEMBER`, the open-signup default that is deliberately default-DENY everywhere
else. The design authority's nav gate for the `field` screen is
`OPERATIONAL_ROLES x work_order_read_all`; nav hiding alone is not server
enforcement. A Login-tier MEMBER could curl the overview and read customer
names, site addresses, contact phones, and geo coordinates for their branch.
The stage-1 design-contract itself had baked in the weaker gate (its §A/§B said
`authorize(Login, …)` while its own `fieldCapabilities` table derived `canRead`
from `work_order_read_all`) — the builder followed the contract; the contract
was wrong against the design authority.

Fix: both field read routes now authorize `Feature::WorkOrderReadAll`
(`[D,A,A,A,A,A]` — MEMBER denied, all operational roles allowed). Out-of-scope
detail stays 404 (scope resolves before the feature check); in-scope-but-
role-denied is 403. Test now asserts MEMBER gets 403 on the list AND on an
in-scope site detail. design-contract.md §A/§B/§fieldCapabilities and the
openapi fragment 403 descriptions were re-aligned.

### F2 — `link_ticket` work-order lookup was an existence oracle (CONFIRMED, low)

The site lookup in `link_ticket` is scope-confined (out-of-scope → 404), but
the work-order lookup was a bare `WHERE id = $1`: an out-of-scope work order
returned 409 "work order is not dispatched to the ticket's linked site" instead
of 404, disclosing that the UUID exists in the org (branch-level leak; org RLS
still applied). Fix: the lookup now carries the same `push_site_scope`
branch predicate — out-of-scope work orders are 404, the wrong-site guardrail
409 remains for in-scope work orders. Test seeds a branch-B work order and
asserts the 404.

### F3 — Idempotency-Key bounds counted bytes, not characters (CONFIRMED, low)

`record_acceptance` validated `key.len()` (bytes) while the 0194 CHECK is
`char_length` (chars): a multibyte key of <16 chars passed the app check and
died as a 23514 → 500 instead of 422. Fix: `key.chars().count()` bounds,
matching the CHECK exactly.

## Verified properties (against code + live runs, not the report)

- **RLS**: `support_ticket_acceptances` ENABLE + FORCE RLS, `org_isolation`
  USING/WITH CHECK on `app.current_org`, org-immutable BEFORE UPDATE trigger,
  `GRANT SELECT, INSERT` / `REVOKE UPDATE, DELETE` for `mnt_rt` (append-only).
  All cross-crate read targets (`registry_sites`, `registry_customers`,
  `work_orders`, `site_attendance_events`) confirmed FORCE RLS (0030/0042).
- **Runtime role**: all three integration tests dispatch through
  `build_router` over a pool whose every connection runs `SET ROLE mnt_rt`.
- **Tenant isolation**: sibling-org admin sees `total: 0` (count concealment)
  and 404 on detail with no name leak — proven as `mnt_rt`, not superuser.
- **Deny-by-omission**: out-of-branch sites absent from rows AND totals;
  detail/cursor/link lookups scope-confined to 404; empty branch scope → SQL
  `FALSE`; untriaged tickets actionable only by cross-branch principals.
- **Idempotency**: same key + same sha256(unit-separated fields) fingerprint →
  201 with the stored acceptance, no second row, no second audit event, no
  notification fan-out; same key + different request → 409; key uniqueness is
  per-org (`UNIQUE (org_id, idempotency_key)`), and the replay lookup is
  org-confined by RLS. Concurrent duplicate resolves via `FOR UPDATE` ticket
  lock → deterministic 409 (never a double insert).
- **Terminal-state races**: acceptance re-checks status under `FOR UPDATE`;
  FSM edges come from the domain `transition_to` (no new states); link is
  rejected on CLOSED under the same lock.
- **Audit**: link (before/after link snapshots), acceptance (+ transition
  event, decline + comment event) — counted and content-checked by readback;
  `accepted_by` PII proven absent from snapshots.
- **Error envelope**: every handler-emitted error is `{ error: { code,
  message } }` with kind→status mapping; DB errors logged server-side and
  returned generic (schema names never leaked).
- **Stat-bar honesty**: overview counts/SLA and the `sla` filter derive from
  one shared SQL aggregation (`FIELD_SLA_CASE` single evaluation site); COUNT
  shares scope+filters but never the cursor, so totals are page-stable.
- **No N+1**: list = 2 queries (count + page) with correlated subqueries in
  SQL; detail = 6 fixed queries in one org-armed transaction, each capped at 50.

## Gates (re-run after fixes)

- `cargo fmt --check` — clean on all four support crates and
  `field_visit_api.rs` (pre-existing drift in `cedar_freshness_mint.rs` /
  `logistics_pilot_story.rs` belongs to other lanes).
- `cargo clippy --tests -D warnings` on mnt-support-{domain,application,
  adapter-postgres,rest} — clean.
- `cargo test` support crates — domain 9, rest 7 unit + 6 sqlx, adapter 4 unit
  + 14 sqlx (incl. the RLS-as-runtime-role suites) — all green, exit 0.
- `cargo test -p mnt-app --test field_visit_api` — 3/3 green as `mnt_rt`
  against dev Postgres 127.0.0.1:55432 (per-test scratch DBs via `#[sqlx::test]`).

## Known limits / not fixed here (out of ownership or accepted)

- Replay returns **201** (sibling-pilot parity) — flip to 200-on-replay is a
  one-line REST change if integrators prefer.
- Axum `Query`/`Json` extractor rejections (malformed/duplicated query params)
  return axum's default plain-text 400, not the canonical envelope — this is
  spine-wide behavior shared by every module, not lane-fixable without a
  platform-level rejection mapper.
- `cargo clippy -p mnt-app --test field_visit_api` cannot complete because the
  UNCOMMITTED local spine unblocks (logistics adapter, production/rest — other
  lanes' files, documented in the integration manifest) trip
  `-D clippy::expect-used`; the owning lanes' real fixes supersede those
  patches. The test target compiles warning-free under `cargo test`.
- The spine's duplicate migration 0170 pair and non-compiling HEAD remain the
  integrator/owning lanes' items exactly as manifested by stage 2 (verified
  still true this session).
