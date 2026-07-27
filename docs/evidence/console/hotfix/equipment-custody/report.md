# Hotfix — equipment 3R handover evidence custody

**Lane** `hf-equipment-custody` · branch `claude/hf-equipment-custody-20260725`
· base `4cabe239` (wave-4 spine, PR #488) · 2026-07-25.

**Independently verified** by a fresh-eyes stage-2 pass —
[`stage2-verification.md`](stage2-verification.md). That pass reproduced the red,
proved every assertion load-bearing with four revert/mutation runs, and closed
two coverage gaps this report did not have: nothing asserted that a *permitted*
handover writes the custody row, and the
`GET /api/v1/equipment-3r/rental-cases/{id}` read path was untested (returning
`evidenceObjectId: null` passed). Both are now asserted in
`repair_lifecycle_completes_with_audits_history_and_no_finance_posting`.

## 1. Reproduction

```
$ cargo test -p mnt-app --test equipment_3r_api        # at 4cabe239
running 4 tests
test capabilities_deny_without_leakage_across_branch_grant_and_org ... ok
test concurrent_approvals_on_one_unit_have_exactly_one_winner ... ok
test repair_lifecycle_completes_with_audits_history_and_no_finance_posting ... FAILED
test resale_disposition_sells_unit_and_blocks_further_quotes ... FAILED

thread '...repair_lifecycle...' panicked at app/tests/equipment_3r_api.rs:166:5:
assertion `left == right` failed: handover:
  {"error":{"code":"internal","message":"internal server error"}}
  left: 500
 right: 200

test result: FAILED. 2 passed; 2 failed
```

Both failures are the same request:
`POST /api/v1/equipment-3r/rental-cases/{id}/handover` returning a
canonical-envelope 500.

These story tests target the Buck2 disposable-Postgres harness. They were run
against a disposable `postgres:18.4` reconciled with
`ops/postgres-reconcile-topology.sh` — the same bootstrap
`tools/buck/test_needs_postgres.sh` performs (`mnt_buck_admin` superuser +
`options[mnt.sqlx_test_bootstrap]`) — with every assertion still crossing the
assembled router on a `SET ROLE mnt_rt` pool.

## 2. Root cause

**Migration `0184_create_docs_equipment_handover_custody.sql` is on the spine
and applied; the crate code that reads and writes its column is not.** The
migration half of `codex/equipment-evidence-custody-20260724` was integrated and
the Rust half was left behind.

0184 (`backend/crates/platform/db/migrations/0184_...sql:20-33`):

- adds `equipment_3r_rental_cases.handover_evidence_object_id UUID`, FK to
  `docs_evidence_objects(id, org_id)`;
- adds a CHECK that a `HANDED_OVER|RETURNED|CLOSED` case must carry one;
- **`DROP COLUMN handover_evidence_reference`**.

`PgEquipment3rStore::handover_case` still issued
`UPDATE equipment_3r_rental_cases SET ... handover_evidence_reference=$3 ...`,
and `case_detail_tx` still selected it. Postgres raises `42703
undefined_column`, mapped to `PgEquipment3rError::Db` → 500.

The test encoded the correct requirement and the code was wrong. **No assertion
was weakened.** The two lifecycle assertions (handover → 200, status →
`HANDED_OVER`) are unchanged from HEAD.

## 3. Disposition of the live codex branch

`codex/equipment-evidence-custody-20260724` is one commit (`8cd1092f`) whose
parent `2b8f54b5` **is an ancestor of this branch**, so a plain
`git merge --no-ff` replays exactly that delta onto the wave-4 tip — no rebase,
no cherry-pick (D-2). One import-list conflict in
`backend/crates/docs/adapter-postgres/src/lib.rs` was resolved as a union
(HEAD's `EvidenceObjectCursor` kept).

**Its intent is right and is adopted in full:** handover must not accept a
caller-asserted `evidence://` string. It takes a typed Docs/Evidence object id
and binds an immutable custody row proving tenant, branch, admissibility,
non-disposal and an original verified-WORM copy, in the same audited transaction
as the FSM transition.

**Two parts of its implementation could not be taken.** Both are places where
that branch's own verification never ran.

### 3.1 Its crate wiring fails a ship-blocking CI gate

It adds `mnt-docs-adapter-postgres` + `mnt-docs-application` to
`mnt-equipment-adapter-postgres` and calls
`PgDocsStore::bind_equipment_handover_evidence_tx`. Merged verbatim:

```
$ cargo run -p mnt-gate-layer-boundary
mnt-gate-layer-boundary: FAILED — 1 violation(s):
  [ILLEGAL_LAYER_EDGE] mnt-equipment-adapter-postgres:
    mnt-equipment-adapter-postgres (adapter) → mnt-docs-adapter-postgres (adapter) is forbidden
```

`Layer::Adapter.allowed_deps()` is `[Application, Domain, Kernel, Platform]`
(`backend/ci/gates/layer-boundary/src/lib.rs:97-102`). Adapter → adapter has no
exemption, and the composition root cannot substitute: the bind must join a
transaction the equipment adapter owns.

**Replacement.** The equipment adapter issues the custody insert itself as one
`INSERT ... SELECT` (`bind_handover_custody` in
`backend/crates/equipment/adapter-postgres/src/lib.rs`). Selecting zero rows is a
`not_found`, so every rejection class — foreign tenant (RLS), unknown object,
non-`ADMISSIBLE`, disposed, unverified original — is concealed behind one 404 and
cannot be used to probe another org's evidence estate.

The eligibility predicate is not newly invented: the enforcing authority remains
the Docs-owned trigger `docs_equipment_handover_custody_eligible()` installed by
0184 (`0184_...sql:59-95`), which independently rechecks the case branch and the
same five evidence facts on insert. The `INSERT ... SELECT` resolves the
`ORIGINAL` copy the custody FK requires — at most one exists per object
(`docs_evidence_copies_one_original_per_object`) — and shapes the refusal. The
codex version carried the same predicate twice for the same reason; this only
moves the second copy to a layer-legal home. The four rejection classes in the
story test pin the two together: if the query ever admitted a row the trigger
refuses, the trigger raises and the case fails to transition instead of
returning 404. A doc comment above the function records this.

Consequence: **no `Cargo.toml` dependency edge changes**, so the
`gen_first_party.py` / BUCK-face consolidation that the codex branch's own note
demanded is not needed.

### 3.2 Its test fixture cannot produce eligible evidence

Two defects, both fatal, both meaning the eligible path of its new test was
never green (its one "valid" object is used only on a 403 path that returns
before custody resolution):

1. `INSERT INTO docs_evidence_objects (... created_by, updated_by) VALUES (..., $8, $8)`
   where `$8` is `disposed_by` — `NULL` unless the variant is disposed.
   `created_by` has been `NOT NULL` since `0151`. Every seed aborts with `23502`.
2. It sets `worm_status='VERIFIED', verified_at=now()` directly. Migration
   **`0195_docs_gaps.sql:35-69`** installs
   `docs_evidence_copies_bind_storage_attestation`, a `BEFORE INSERT` trigger
   that **overwrites** any caller-supplied `worm_status`/`verified_at` and
   promotes a copy only when a tenant-matching `evidence_media` row proves the
   WORM replica by object key and SHA-256. A fixture cannot assert verification —
   that is the whole point of the migration.

**Replacement.** The eligible variant seeds the real server-written attestation
(`seed_verified_worm_media`), mirroring the established fixture in
`backend/crates/docs/rest/tests/evidence_rest_rls_surfaces_as_runtime_role.rs:242-358`
— work order → `evidence_media` with `worm_replica_status='VERIFIED'` and
`checksum_sha256 = encode(decode($digest,'hex'),'base64')` — and the copy
references it through `source_evidence_media_id`. The ineligible variant omits
the attestation. The helper then asserts the promotion outcome, so the fixture
cannot silently drift back to forging a flag:

```rust
assert_eq!(promoted, if verified_worm { "VERIFIED" } else { "PENDING" },
    "the storage-attestation trigger, not the fixture, decides WORM verification");
```

The codex evidence note
`docs/evidence/console/CAP-EQUIPMENT-3R-PILOT/evidence-custody-boundary.md` was
dropped rather than merged: it describes the docs-crate ownership split and the
BUCK consolidation step this lane does not ship, so keeping it would commit a
false statement. Its accurate content is this section.

## 4. Verification

Run from `backend/` against a disposable topology-reconciled `postgres:18.4`;
app assertions cross the assembled router on a `SET ROLE mnt_rt` pool.

| Command | Result |
|---|---|
| `cargo test -p mnt-app --test equipment_3r_api` | **5 passed; 0 failed** (was 2 passed / 2 failed) |
| `cargo test -p mnt-equipment-{domain,application,rest,adapter-postgres}` | 7 passed; 0 failed |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -p mnt-equipment-{domain,application,adapter-postgres,rest} -p mnt-docs-adapter-postgres --all-targets -- -D warnings` | clean |
| `cargo run -p mnt-gate-layer-boundary` | PASSED — 166 crates, 0 violations |
| `cargo run -p mnt-gate-audit-coverage` | PASSED |
| `cargo run -p mnt-gate-rls-arming` | PASSED |
| `cargo run -p mnt-gate-tenant-isolation` | PASSED |
| `cargo run -p mnt-gate-dev-auth-absence` | PASSED |
| `cargo run -p mnt-gate-migration-safety` | FAILED — pre-existing, §5.3 |
| `cargo test -p mnt-app --test openapi_drift` | 12 passed / 1 FAILED — pre-existing, §5.4 |

The five story assertions now proven as `mnt_rt`:

- repair lifecycle: handover → 200 / `HANDED_OVER`, audits + history + no
  finance posting;
- resale disposition: handover → 200, unit sold, further quotes blocked;
- **custody concealment**: foreign-org, unverified-WORM, disposed and
  non-admissible evidence each → **404**, and the case **stays `DISPATCHED`**;
- **branch authorization**: a dispatch-capable actor in another branch → **403**
  with `docs_equipment_handover_custody` provably empty for that case
  (authorization runs before custody resolution);
- the pre-existing deny-without-leakage and concurrent-approval stories,
  unchanged.

## 5. Open items handed to the integrator

Manifests: `docs/evidence/console/hotfix/equipment-custody/manifests/`.

### 5.1 `backend/openapi/openapi.yaml` — LANDED (was blocking, openapi-integrator)

Closed by openapi-integrator on `wave23-consolidation-20260724`: `cc69ebf8`
(merge of this lane) then `b6915340` (spec + `clients/{ts,kotlin,swift}` regen),
in that order — the §5.5 ordering held. Verified from this worktree: `:14453`
now requires `evidenceObjectId`, and this branch's tip `c08a12db` is contained
in the spine. `openapi_drift` 13/0, both drift gates 0/0, web 2792/2792.

One thing the fragment got right by accident of scoping, worth recording
because the next person will grep: **`evidenceReference` occurs three times in
the spec, not two.** The third is `verifyLogisticsPod` at `openapi.yaml:14322`,
a different lane, where it is still legitimately an `^evidence://` string. A
name-based sweep of the retired field would silently break logistics. The
fragment named the two `equipment-3r` sites explicitly and left it alone.

Original statement of the item:

`handoverEquipment3rCase` still requires `evidenceReference` (`^evidence://`) at
`openapi.yaml:14453`, and `Equipment3rCaseDetailView` still returns it at
`:32564`. The server now takes and returns `evidenceObjectId` and rejects
unknown fields (`deny_unknown_fields`). Fragment:
`manifests/openapi-fragment.yaml` — two edits, existing `equipment-3r` tag, no
new operation or schema. Then regenerate `clients/{ts,kotlin,swift}` and re-run
`openapi_drift` + `check:api-drift:portable` + `:swift`. **Landing order is
load-bearing — see §5.5.**

### 5.2 `web/src/console/equipment/**` — needs an owner, not this lane's root

`equipmentApi.ts:83,170` and `EquipmentCaseDetail.tsx:171,334,446` post and
render `evidenceReference` as free text. **Already broken at HEAD** — the column
it targets was dropped by 0184, so it 500s today; after this lane it 422s. No
live exposure: `EXPOSED_SCREEN_KEYS` is `[]` and the equipment screen is DARK.
The fix is not a rename — the field stops being text a user types and becomes a
chosen Docs/Evidence object with an admissible, verified-WORM original.

### 5.3 `mnt-gate-migration-safety` red on the spine

`[NonContiguousMigrationVersion] missing migration version 0201 before 0202`.
This lane adds and edits no migration (`git status` shows nothing under
`backend/crates/platform/db/migrations/`). `0201` is the charter's recorded
RESERVED slot (§0), so the gate is red on the spine independently of this work.
Integrator / slot-ledger decision.

### 5.4 A second red test on the spine: `openapi_drift` — REPORTED, NOW FIXED

`openapi_documents_evidence_register_snapshot_and_evidentiary_contract` failed
with *"OpenAPI YAML must define the EV object list endpoint"*. It was a test
bug, not a missing endpoint — `backend/app/tests/openapi_drift.rs:446` searched
for `"  /api/v1/evidence/objects:\\n"`, which in Rust source is a literal
backslash followed by `n` and can never match YAML. Both `openapi_drift.rs` and
`openapi.yaml` are unmodified by this lane, so the failure was byte-identical at
HEAD.

Handed to openapi-integrator, who fixed it in `5f7181bc` on
`wave23-consolidation-20260724` — now 13 pass / 0 fail. **Two corrections to
what this lane reported**, recorded because the original claim was too small:

- it is **seven** `.find()` calls (446, 484, 487, 502, 505, 521, 524), not one;
  fixing only 446 moves the panic to 484;
- with the searches repaired the body executed for the first time and a
  fourteenth assertion failed on its own account — it substring-matches
  `as_of: { type: integer, format: int64 }` against `EvidenceObjectPage`, whose
  flow map legitimately also carries a `description`. The integrator loosened
  the assertion rather than delete a useful spec description to satisfy a
  substring match. That test asserts YAML *formatting* in several places and
  will break again on any reformat.

### 5.5 Landing order for §5.1 — agreed, then executed as agreed

The spec edit must **not** be applied to the spine ahead of the crate diff. The
crate half lives only on this branch; `wave23-consolidation-20260724` still has
`evidence_reference: String` at `equipment/rest/src/lib.rs:310`. A spec that
advertised `evidenceObjectId` against that server would make every console
handover 422, and **no gate would catch it** — `openapi_drift` compares path
inventories, not request bodies. Migration 0184 sitting on the spine does not
help: the migration is not the wire contract, the serde struct is.

Agreed resolution: the integrator merges this branch
(`git merge claude/hf-equipment-custody-20260725`, no push involved) and then
commits the spec edit plus the `clients/{ts,kotlin,swift}` regen on top, before
that branch moves. Both halves then reach the spine in one merge. File overlap
between `5f7181bc` and `963fa0a9` is empty, so the merge is clean.

This lane deliberately did **not** take the openapi + clients edit itself: those
roots are the integrator's, this lane may not push, and a second independent
regen of `clients/**` meeting theirs in a merge would be a hand-resolved
generated-code conflict.

Toolchain note for whoever runs it: there is no JRE on this machine
(`java -version` fails), but `scripts/generate-kotlin-client.mjs:249-289` falls
back to the openapitools Docker image when no JDK is on PATH, and Docker is
healthy (`docker info` → 29.5.2). This worktree has no `node_modules`.
