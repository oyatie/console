# OpenAPI fragment integration plan (wave-2/3 consolidation)

Worktree: `maintenance-worktrees/pr488-design-mirror-sync`, branch
`wave23-consolidation-20260724`. Target: `backend/openapi/openapi.yaml`
(source-of-truth `include_str!`).

## Why this doc exists

A prior mechanical splice of every `openapi-fragment.yaml` corrupted the spec
and was reverted (`9bb877c6`), leaving `openapi.yaml` byte-identical to the
spine (`origin/codex/operational-object-runtime-progress`, md5
`f1d7b8e6…`). The fragments are **not** uniformly splice-able: two of them
carry non-`schemas` component sections, and one mixes edit-directives among
real schemas. This plan classifies each fragment before any spec edit, so the
integration is per-fragment and deliberate.

## Gate being closed

`cargo test -p mnt-app --test openapi_drift` baseline on this branch:
**11 passed, 2 failed**.

| Test | Baseline | Cause |
|------|----------|-------|
| `openapi_yaml_covers_configured_route_inventory` | **FAIL** | 21 census paths absent from the spec (below). This is the gate this work closes. |
| `openapi_documents_evidence_register_snapshot_and_evidentiary_contract` | **FAIL** | **Pre-existing spine bug**, not consolidation drift: seven `.find()` calls search for the two-character literal `\n` (written `"…:\\n"` in Rust source), which can never match YAML, so the test body had never executed. **Fixed** — see Outcome. |

### The 21 missing census paths

Computed by replaying the test's own resolution (`CONFIGURED_ROUTE_SURFACES` →
each crate's `*_ROUTE_PATHS` → path-parameter normalisation) against the spec's
path keys.

| Surface | Missing paths |
|---------|---------------|
| `support` (4) | `/api/v1/support/tickets/{id}/link`, `/api/v1/support/tickets/{id}/acceptance`, `/api/v1/field/sites`, `/api/v1/field/sites/{id}` |
| `workorder` (4) | `/api/v1/work-orders/{work_order_id}/settlement`, `/api/v1/settlements/{settlement_id}/submit`, `/api/v1/settlements/{settlement_id}/review`, `/api/v1/settlements/{settlement_id}/void` |
| `notices` (1) | `/api/v1/notices/{id}/receipts` |
| `payroll` (12) | `/api/v1/payroll/runs/{id}/` × `close-preflight`, `close-attendance`, `calculate`, `exceptions`, `exceptions/{exception_id}/resolve`, `submit`, `decision`, `withdraw`, `schedule-disbursement`, `disbursement/attest`, `issue-payslips`, `payslip-delivery` |

### Mounted-but-uncensused surfaces

Three merged lanes mount routers that are **absent from
`CONFIGURED_ROUTE_SURFACES`**, so the census⊆spec gate does not currently see
them. Their routes are equally undocumented, and the console modules that call
them have no generated types. They are integrated here and joined to the census
in the same commit as their spec paths (as `lib.rs` already instructs for
`orgchange`):

| Crate | Mounted at | Routes | Census entry |
|-------|-----------|--------|--------------|
| `mnt_notifications_rest` | `lib.rs:3138` | 4 undocumented of 9 | already partially documented under tag `me`; no census entry |
| `mnt_recruiting_rest` (+ app `recruiting_hire`) | `lib.rs:3000`, `:3006` | 18 + 1 hire | none |
| `mnt_orgchange_rest` | `lib.rs:3207` | 10 | none (explicit `NOTE(CAP-ORG-CONSOLE)` placeholder at `lib.rs:338`) |

### Not integrated: evaluation (dark)

`mnt-evaluation-rest` is a `mnt-app` dependency (`Cargo.toml:168`) but its
router is **not mounted** in `build_router`. Its 15 routes have no runtime
existence, so documenting them would publish a contract the server does not
serve. Its fragment states it must be merged "in the SAME change that mounts
`mnt_evaluation_rest::router`" — that mount is not part of this task.

> **Reported defect, not fixed here:** the *frontend* evaluation screen is
> live — `nav.ts:116,184` and `registry.ts:36,78` mount `EvaluationScreenBody`.
> A user reaching that screen calls `/api/v1/evaluation/*` against a router
> that is not mounted (404). Either the backend router must be mounted or the
> screen must be un-navved; both are outside this integration.

## Per-fragment classification

Legend — **P** real `paths`, **S** real `components.schemas`,
**R** real `components.responses`, **Q** real `components.parameters`,
**T** top-level `tags`, **D** EDIT-DIRECTIVES (structured instructions to
modify an *existing* spec object; must be applied by hand, never spliced).

| Fragment | Content | Notes |
|----------|---------|-------|
| `CAP-FIELD-CONSOLE` | P, S, **D** | Directives live in the header comment: `SupportTicketSummary` gains five properties (`site_id`, `site_name`, `customer_id`, `customer_name`, `work_order_id`); `GET /api/v1/support/tickets` gains a `site_id` query parameter. Applied by hand. |
| `CAP-MAINTENANCE-CONSOLE` | P, S, **D** | **The corrupting fragment.** Top-level `existing_operation_deltas:` (4 entries, incl. nested `schema_changes:`) and `existing_schema_deltas:` (4 schemas) are directives, *not* spec objects — a mechanical splice writes them into the document root. Applied by hand. |
| `CAP-BOARD-CONSOLE` | P, S, **D** | Directives in header: add `patch` under the *existing* `/api/v1/notices/{id}`; replace `CreateNoticeDraftRequest`; add `'422'` to `publishNotice`. |
| `CAP-PAYROLL-CONSOLE` | P, S, **D** | Header directive: `PayrollRunSummary`/`PayrollRunDetail` gain fields listed at fragment tail. |
| `CAP-NOTIF-CONSOLE` | P, S, **D** | Header directives: `NotificationSummary` gains `muted`; `NotificationCountsSummary` gains `muted_unread` and re-specifies existing counters. |
| `CAP-ORG-CONSOLE` | P, S | Clean. Reuses shared responses by `$ref`. |
| `CAP-RECRUITING` | P, S, **R**, **Q** | **The second corrupting fragment.** `components.responses` (4× `Recruiting*`) and `components.parameters` (3× `Recruit*Id`) are real objects but land in the wrong section under a schemas-only splice. Deduplicated — see below. |
| `CAP-EVALUATION-CONSOLE` | **T**, P, S | Top-level `tags:` list. Not integrated (dark router). |
| `CAP-EQUIPMENT-3R-PILOT` | P, S (+ JSON manifest) | **Already integrated** on the spine — all 12 path keys present. No action. |
| `CAP-LOGISTICS-PILOT` | `openapi.json` + applied-notes | **Already integrated** by a prior integrator; notes record deliberate deviations. No action. |

### Recruiting component deduplication

The fragment's four `Recruiting*` responses are near-duplicates of shared
responses that already exist, differing only in envelope schema name. The spec
already standardises on `ErrorBody`, and `RecruitingErrorResponse` is the same
`{ error: { code, message } }` envelope. Decision: **reuse the existing shared
responses** (`Forbidden`, `NotFound`, `Conflict`, `ValidationError`) and drop
`RecruitingForbidden`/`RecruitingNotFound`/`RecruitingConflict`/`RecruitingValidation`
and `RecruitingErrorResponse`. The fragment's three path parameters are inlined
at their single-use sites, matching how the spec inlines `{id}` elsewhere,
rather than adding three more `components.parameters` entries.

## Tag assignment

Per-domain `tags:` on **every** operation is a hard rule: a missing tag
collapses the generated Kotlin client into one `DefaultApi.kt` that OOMs
`kotlinc` (PR #261). Where a fragment's charter tag would *split* an existing
resource family across two generated client classes, the existing family tag
wins.

| Operations | Tag | Rationale |
|-----------|-----|-----------|
| `/api/v1/support/tickets/{id}/link`, `…/acceptance` | `support` | Five existing `/api/v1/support/*` operations carry `support`; `field` would fork `SupportApi`. |
| `/api/v1/field/sites`, `/api/v1/field/sites/{id}` | `field` | Genuinely new resource family. |
| settlement × 4 | `maintenance` | Fragment's stated default; cohesive new resource. Both `maintenance` and `work-orders` satisfy the gate. |
| `/api/v1/notices/*` new + amended | `notices` | Fragment's own recommendation; the five shipped notices operations carry `notices`. |
| `/api/v1/payroll/*` | `payroll` | Matches the three existing payroll operations. |
| `/api/v1/me/notification*` new | `me` | The five existing notification operations carry `me`; `notif` would fork the family. |
| `/api/v1/recruiting/*`, `…/hire` | `recruiting` | New family. |
| `/api/v1/org-changes/*`, `/api/v1/org-entities` | `org` | New family. |

## Method

1. Ground truth is the **Rust code**, never the fragment. For every route the
   merged backends register, method / path / request body / response shape are
   confirmed against the handler signatures and their `serde` types. Where
   fragment and code disagree, the code wins and the discrepancy is recorded in
   the integration commit message.
2. One commit per fragment/domain. After each, the spec is YAML-parsed and
   `npm run gen:api:ts` is run. Only validated states are committed — the spec
   is never left broken between commits.
3. Clients (`ts`, `kotlin`, `swift`) are regenerated once at the end and
   committed together; the Kotlin output is checked for per-domain `*Api.kt`
   files and the absence of a monolithic `DefaultApi.kt`.

## Outcome

`openapi.yaml` went from 434 paths / 490 operations / 824 schemas to
**488 / 551 / 946**. `cargo test -p mnt-app --test openapi_drift` went from **11 pass / 2 fail** to
**13 pass / 0 fail**. The target gate
`openapi_yaml_covers_configured_route_inventory` is GREEN.

`openapi_documents_evidence_register_snapshot_and_evidentiary_contract` is
also fixed, after the hf-equipment-custody lane correctly pushed back on my
first reading of it. I had reported two causes; only one was real. The path
and the `EvidenceObjectPage` schema both DO exist (openapi.yaml:12874 and
:31113) — the sole defect was the literal `\\n` in seven `.find()` calls, which
meant the test body had never run at all. With that corrected the body
executed for the first time and 13 of its 14 assertions passed; the last one
substring-matched `as_of: { type: integer, format: int64 }` against a flow map
that legitimately also carries a `description`, so the assertion (not the
spec) was loosened to stop matching punctuation.

All three clients regenerated; `check:api-drift:portable` and
`check:api-drift:swift` both exit 0. Kotlin emitted 79 per-domain `*Api.kt`
files including new `FieldApi` / `MaintenanceApi` / `OrgApi` / `RecruitingApi`,
with no `DefaultApi.kt`.

### Known gap: swift-openapi-generator drops nullable-$ref properties

The Swift generator emits `Schema "null" is not supported … skipping` for every
property whose schema is a `oneOf`/`anyOf` union of a `$ref` and `type: 'null'`
— 42 properties spec-wide, 14 of them added by this integration. Neither the
union keyword nor member ordering changes it; the generator has no support for
the construct, so those fields are silently absent from the Swift client.
Fixing it means changing how the whole spec spells a nullable object reference,
which is a spec-wide decision, not an integration one.

### Not applied here: the equipment-3r handover contract change

The hf-equipment-custody lane (branch `claude/hf-equipment-custody-20260725`)
asked for `POST /api/v1/equipment-3r/rental-cases/{case_id}/handover` to take
`evidenceObjectId` instead of `evidenceReference`. **Deliberately not applied
on this branch.** The crate half of that change is not here —
`backend/crates/equipment/rest/src/lib.rs:310` still reads
`evidence_reference: String`, and the DTO is `deny_unknown_fields`. Editing
only the spec would publish a request body this branch's server rejects with a
422, which is the exact failure mode this integration exists to prevent, and no
gate would catch it: the drift test compares path inventories, not bodies.

The spec, the three clients and the crate have to move in one commit. That
belongs to whoever carries the crate change — either the lane lands its crate
diff here first, or the lane owns the openapi + clients edit in its own branch.
Migration `0184` being on the spine does not change this; the migration is not
the wire contract.

### Proposed new gate: request bodies are unguarded

`openapi_drift` compares **path inventories** — every route the census
declares must appear as a path key in the spec. Nothing compares **request or
response body schemas** against the handler's serde types.

That hole is not theoretical; this integration walked into it twice:

- The equipment-3r handover body advertised `evidenceReference` as an
  `^evidence://` string long after migration 0184 deleted the column and the
  crate moved to `evidenceObjectId: Uuid`. Every gate stayed green while the
  documented body was one the server rejects outright under
  `deny_unknown_fields`.
- `PayrollRunStatus` omitted seven statuses the FSM writes, inlined in two
  schemas. Also green.

Both were found by a human reading code, not by CI. A path-inventory gate can
never catch either, because the path was always present and correctly spelled.

Proposed: assert body schemas against handler serde types. The cheap version
that would have caught both — for each documented operation, resolve its
requestBody schema's `required` + property names and diff them against the
`#[derive(Deserialize)]` struct the handler binds, failing on any name present
on one side only. `deny_unknown_fields` makes the request direction strictly
decidable: an undocumented field is a guaranteed 422, not a maybe. Enum
variants are the natural second step, since that is the shape the payroll bug
took.

### Reported, not fixed

- **RESOLVED.** The two `nav.test.ts` failures were real and are now fixed:
  the payroll nav item gated on `employee_directory_read` with a role list, so
  a built-in branch-scoped ADMIN saw a screen `authorize_org_wide` then denied.
  Now grant-only on `payroll_run_read`. The test was correct throughout and is
  unchanged. I had reported these as pre-existing and not mine; the first half
  was true, the second was not — nobody else was going to fix them.
- `orgchange` is the only REST surface on the platform that serialises
  camelCase. The console was corrected to match the server, but the server is
  the outlier.
