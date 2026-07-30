# `openapi.yaml` staleness audit

Status: AUDIT — read-only, 2026-07-30

Target: `backend/openapi/openapi.yaml`, 35,935 lines, served verbatim via
`include_str!` at `backend/app/src/lib.rs:214`, route registered at
`backend/app/src/lib.rs:2889`, handler at `backend/app/src/lib.rs:3486`.

---

## Verdict: neither generate nor demote — fix and close the gate

The question assumed the file is stale. On paths it is not: **zero documented
paths are unserved**, and the 10 served-but-undocumented paths are all
deliberate non-API surfaces. On my 17-schema sample it is 82% accurate. The
defects are real but they concentrate in three fixable classes, and — the
finding that decides this — **six of the correct typed schemas already exist in
the file, defined and unreferenced.** The gap is wiring, not authorship.

**Demote is the worst option.** The file is served live at
`/openapi/openapi.yaml` (`backend/app/src/lib.rs:2889`). Demoting it to
documentation does not stop clients consuming it; it only stops anyone
maintaining it. A live artifact nobody owns is strictly worse than a live
artifact somebody hand-maintains.

**Generation costs ~3,900 annotations** (quantified in §5) and its main benefit
— schema/handler agreement — is not what my sample failures need. All three
failures are *omissions* (a 2xx body documented as an opaque object), and the
cheapest fix for two of them is a one-line `$ref` to a schema already in the
file. Generation would also destroy the ~824 hand-authored constraints
(`pattern`, `minimum`, `minLength`) unless each is re-encoded as a
`#[schema(...)]` attribute — and those constraints are the part an integrator
most needs, e.g. the If-Match ETag pattern at `openapi.yaml:12214`.

**The actual defect is the gate, not the hand-maintenance.**
`openapi_yaml_covers_configured_route_inventory`
(`backend/app/tests/openapi_drift.rs:351`) asserts YAML ⊇ inventory and never
the reverse, and **64 of 515 served paths (12.4%) sit outside the inventory
entirely**. Extending the existing gate costs a fraction of a contracts crate
and catches the same class of drift.

Recommended order: (1) the 10 security misstatements, (2) the gate holes,
(3) the 32 opaque response bodies, (4) identity, (5) parameter names.

---

## Findings table

| # | Category | Count | Example | Consequence |
|---|---|---|---|---|
| 1 | Documented paths that are not served | **0** | — | The premise was wrong. Path drift is zero. |
| 2 | Served paths not documented | **10** | `/metrics`, `/__wide_event/known` (`app/src/lib.rs`) | None — all deliberate non-API surfaces. |
| 3 | **Operations documented as unauthenticated that require a bearer token** | **10** | `authorizeBulk` `openapi.yaml:13199` vs `authorize_admin` `platform/authz-rest/src/lib.rs:390` | **Security-relevant.** An integrator omits the token and gets 401; a gateway config generated from this file leaves them ungated. |
| 4 | Served paths outside the drift inventory | **64 of 515 (12.4%)** | all 9 `leave/rest` routes; all 12 `app/src/objects.rs` routes | Nothing asserts these are documented. 54 are documented by luck, not enforcement. |
| 5 | Router files whose `.route()` calls the gate never parses | **16 files / 93 calls** | `backend/crates/logistics/rest/src/lib.rs` (9) | A new route in these files can ship undocumented and green. |
| 6 | Successful (2xx) bodies documented as opaque `{type: object, additionalProperties: true}` | **32 of 569 operations** | `reportEmployeeExitCase` 201 `openapi.yaml:2139` vs 23-field `EmployeeExitCaseResponse` `app/src/hr.rs:693` | Client generators emit `Map<String,Object>`. No compile-time contract at all. |
| 7 | Path-parameter name mismatches | **66** | served `{plan_id}` vs documented `{planId}` | Invisible to `normalize_path_parameters` (`openapi_drift.rs:833`), which erases names. |
| 8 | Orphan schemas (defined, unreachable from any path) | **16 of 956 (1.7%)** | `TraversalGraph` `openapi.yaml:32086` | 6 are the *correct* schema for an operation documented as opaque. Wiring gap. |
| 9 | Case-collided tags | **4 names / 2 domains** | `Policy` (12) + `policy` (16); `Evidence` (4) + `evidence` (8) | Generators emit two client classes per domain. |
| 10 | Stale product identity | **1** | `info.title: Maintenance FSM Backend API` `openapi.yaml:3` | Cosmetic but it is the first thing an integrator reads. |
| 11 | Missing `servers:` block | **1** | absent entirely | Clients have no base URL; every generated SDK needs manual configuration. |
| 12 | Undocumented error status | **1** | `413` appears 0 times, cap is 2 MiB (`app/src/lib.rs:209`) | A >2 MiB body returns axum's **plain-text** rejection, not `ErrorBody`. |
| 13 | Competing vocabularies for one concept | **2 pairs** | `/api/v1/object-types` (`app/src/objects.rs:39`) vs `/api/v1/ontology/object-types` (`ontology/rest/src/lib.rs:198`) | Only **1** `deprecated:` marker exists in 35,935 lines. An integrator can build the ontology vertical against the wrong surface. |

---

## 1. How much describes surfaces that no longer exist? Essentially none.

**Method.** Extracted the 505 path keys from the `paths:` block
(`openapi.yaml:5`–`17714`). Separately extracted every `.route(...)` call from
all 53 non-test router source files under `backend/`, resolving path constants
crate-locally (a global resolution is wrong here: `OBJECT_TYPES_PATH` is
defined twice with different values — `/api/v1/object-types` in
`app/src/objects.rs:39` and `/api/v1/ontology/object-types` in
`ontology/rest/src/lib.rs:198`). 515 served paths, zero unresolved arguments.
Compared both directions with parameter names normalised.

**Documented but not served: 0.**

**Served but not documented: 10**, all deliberate:

| Path | Source |
|---|---|
| `/` | `crates/platform/request-context/src/lib.rs` |
| `/__test/client-ip` | `app/src/lib.rs` |
| `/__wide_event/known`, `/__wide_event/widgets/{widget_id}` | `app/src/lib.rs` |
| `/metrics` | `app/src/lib.rs` |
| `/openapi/openapi.yaml` | `app/src/lib.rs:2889` |
| `/realtime/slow`, `/timed/slow` | `app/src/lib.rs` |
| `/api/v1/dev-auth/session` | `platform/auth-rest/src/lib.rs` — deliberately gate-ignored (`openapi_drift.rs:182`) |
| `/api/v1/mail/mox/webhook` | `comms/rest/src/lib.rs` — deliberately gate-ignored (`openapi_drift.rs:162`) |

**The established premise was wrong, and this is the audit's most useful
correction.** The deleted `web/`, `android/`, `ios/` and `clients/` directories
were *clients*; deleting them removed no backend route. And the domains put
out of scope by the pivot — ERP, field ops, dispatch, comms, compliance,
ingest, evidence/WORM — are **all still served by this backend**.
**216 of 505 documented paths (42%)** belong to those out-of-scope domains,
led by `financial` (19), `work-orders` (14), `equipment-3r` (12),
`inventory` (12), `equipment` (11), `messenger` (11), `mail` (11).

So the file is not lying about what exists. It is accurately documenting a
backend that is 42% larger than the declared product scope. That is a
**scope decision, not a drift defect** — and it means "delete the stale parts
of the YAML" is not an available action. Deleting those 216 paths requires
deleting the routes first.

### The gate hole (loud, as requested)

Two distinct holes, both in `backend/app/tests/openapi_drift.rs`:

**Hole A — the coverage assertion is one-directional.**
`openapi_yaml_covers_configured_route_inventory` (`:351`) iterates
`CONFIGURED_ROUTE_SURFACES` and asserts each path is present in the YAML. There
is no assertion in the other direction, so a documented path for a route that
was deleted would never fail. Today that costs nothing (0 such paths), but it
is the direction staleness actually accumulates.

**Hole B — 12.4% of served routes are outside the inventory.** Resolving all
41 surface constants referenced at `app/src/lib.rs:224`–`392` yields 451
inventory paths against 515 served. The 64-path gap by source file:

| Routes outside inventory | File | Has a surface constant? |
|---|---|---|
| 12 | `backend/app/src/lib.rs` | partly (base routes only) |
| 12 | `backend/app/src/objects.rs` | **no** |
| 9 | `backend/crates/leave/rest/src/lib.rs` | **no** |
| 9 | `backend/crates/logistics/rest/src/lib.rs` | **no** |
| 5 | `backend/app/src/lifecycle.rs` | **no** |
| 4 | `backend/app/src/office.rs` | **no** |
| 3 | `backend/crates/inbox/rest/src/lib.rs` | **no** |
| 3 | `backend/crates/todos/rest/src/lib.rs` | **no** |
| 1 each | `console_telemetry.rs`, `action_inbox.rs`, `workbench.rs`, `workflow_object_context.rs`, `request-context`, `auth-rest`, `comms/rest` | mixed |

Separately, `configured_route_inventory_covers_router_route_calls` (`:304`)
parses only the 38 files listed in `CONFIGURED_ROUTE_SOURCES`. **16 router
files containing 93 `.route()` calls are never parsed.** And 5 of the 41
surfaces have a path constant but no source parsing at all — `audit`,
`consulting`, `workorder-mobile`, `facilities`, `production` — so a route added
in those crates without also editing the constant is invisible.

**Hole C — parameter names are erased before comparison.**
`normalize_path_parameters` (`:833`) rewrites `{anything}` to `{}`. This hides
**66 parameter-name mismatches** where the router uses `snake_case` and the YAML
uses `camelCase`:

```
served /api/v1/financial/purchase-requests/{purchase_request_id}/submit
docd   /api/v1/financial/purchase-requests/{purchaseRequestId}/submit
```

Wire-level harmless (clients substitute positionally), but every generated SDK
method signature and every path-template metric label diverges from the router.
The 66 are all `snake_case` → `camelCase`; concentrated in `financial` (13),
`messenger` (7), `daily-work-plans` (4).

---

## 2. Are the schemas accurate? 3 of 17 sampled failed (18%).

### Sampling method — stated so it can be extended

This is a **sample of 17 schemas out of 956 defined** (1.8%). Not a
verification of the file.

1. Enumerated every `post`/`put`/`patch` operation per domain in the `paths:`
   block.
2. For each priority domain (ontology, HR, payroll, attendance, leave,
   governance) took the request schema of the **first** write operation in that
   domain's path block, plus its primary 2xx response schema.
3. Resolved the Rust type by locating the handler registered at that path in
   the owning crate's `router()`, then following its `Json<T>` extractor and
   return type to the struct definition.
4. Compared field names, optionality (`Option` / `#[serde(default)]` vs
   `required:`), enum variants, and strictness
   (`#[serde(deny_unknown_fields)]` vs `additionalProperties: false`).
5. Added `ErrorBody` and two schemas named as already-suspected.

Weighted toward the first vertical as instructed. **Not sampled:** the other
939 schemas, and in particular every read-heavy list/page schema outside the
priority domains (inventory, financial, sales, recruiting, evaluation), where I
would expect the error rate to differ.

### The two already-suspected items: both confirmed

**`CreateObjectTypeDraft` — confirmed, and worse than reported.**
`openapi.yaml:31391`–`31433` documents four child arrays as
`items: {type: object, additionalProperties: true}`. The real types are in
`backend/crates/ontology/adapter-postgres/src/lib.rs`:

| YAML field | Real type | Fields | Required in Rust but absent from YAML |
|---|---|---|---|
| `properties` | `PropertyDefInput` `:115` | 7 | `key`, `title`, `field_type` |
| `links` | `LinkTypeInput` `:132` | 6 | `stable_key`, `title`, `cardinality` |
| `actions` | `ActionTypeInput` `:146` | 7+ | `stable_key`, `title` |
| `analytics` | `AnalyticInput` | — | — |

So **~26 fields carry no contract**, and a client that omits `cardinality` on a
link — which the YAML gives it no reason to send — gets a deserialisation
failure. The YAML's own `required:` list (`stable_key`, `title`,
`backing_kind`) is correct for the top level.

**`to_object_type_id` — confirmed absent, and the reason is structural.**
Zero occurrences in `openapi.yaml`. It is a real field on
`LinkTypeInput:139` and `LinkTypeSummary:282`. It is missing because the
*entire containing read type is missing*: `ObjectTypeDetail`
(`adapter-postgres/src/lib.rs:311`–`320`, 8 fields including four typed child
arrays) appears **0 times** in the YAML, and the operation that returns it —
`getObjectType`, `GET /api/v1/ontology/object-types/{key}` — documents its 200
body as `{type: object, additionalProperties: true}` at `openapi.yaml:12076`.

### Sample results

**FAIL (3):**

1. `CreateObjectTypeDraft` `openapi.yaml:31391` — above.
2. `reportEmployeeExitCase` 201 `openapi.yaml:2139` — `{}` against
   `EmployeeExitCaseResponse` (`app/src/hr.rs:693`–`730`): **23 fields**,
   including a nested `settlement_package` object and a `next_actions` array.
3. `getObjectType` 200 `openapi.yaml:12076` — `{}` against `ObjectTypeDetail`.

**PASS (14):** `ObjectTypeSummary` (`:32093`, exact 8/8 against
`adapter-postgres/src/lib.rs:223`); `transitionObjectTypeLifecycle` body
(`:12215`); `attachObjectTypePolicy` body (`:12284`, against
`AttachObjectPolicyRequest` `ontology/rest/src/lib.rs:463`);
`RaiseAttendanceExceptionRequest` (`:33447`, exact 6/6 against `RaiseBody`
`attendance/rest/src/lib.rs:650`, and `additionalProperties: false` correctly
mirrors `deny_unknown_fields`); `AssignAttendanceSubstituteRequest` (`:33473`,
exact 11/11 against `AssignBody` `:763`); `ClosePayrollAttendanceRequest`
(`:34549` against `CloseAttendanceBody` `payroll/rest/src/lifecycle.rs:180`);
`ResolvePayrollExceptionRequest` (`:34557` against `ResolveExceptionBody`
`:307`); `LeaveDecideRequest` (`:26323` against `DecideRequestV1`
`leave/rest/src/lib.rs:338`); `GovernanceDecideApprovalRequest` (`:31520`
against `DecideApprovalRequest` `governance/rest/src/lib.rs:108`);
`GovernanceConfigureTransitionRequest` (`:31539`, exact 6/6 against `:116`);
`createGovernanceApproval` body (`:13122`, exact 4/4 against
`CreateApprovalRequest` `:91`); `reportEmployeeExitCase` request (`:2116`,
exact 5/5 against `ReportEmployeeExitCaseRequest` `app/src/hr.rs:647`);
`AttendanceImportPreviewResponse` (`:23463`, exact 10/10 against
`app/src/hr.rs:3385`, with `mapping_profile` legitimately opaque because it
really is `serde_json::Value`); `ErrorBody` (`:25641`, matches
`{error:{code,message}}` per `docs/rest/src/lib.rs:809`–`821`).

Two sub-threshold notes: `LeaveDecideRequest` lacks
`additionalProperties: false` although `DecideRequestV1` carries
`#[serde(deny_unknown_fields)]` (`leave/rest/src/lib.rs:337`) — the doc is more
permissive than the server, so a client sending an extra field gets an
unexplained 422. And `GovernanceDecideApprovalRequest` documents
`enum: [approved, rejected]` while `ApprovalDecision`
(`governance/application/src/lib.rs:16`) also has `Pending` — doc stricter than
server, which is the safe direction.

### The exhaustive count behind the sample

The sample's failure mode is mechanically countable across the whole file:
**32 of 569 operations document a successful 2xx body as an opaque object.**
They cluster precisely in the first vertical:

| Domain | Opaque 2xx bodies | Lines |
|---|---|---|
| policy / Cedar | 10 | `12742`, `12767`, `12796`, `12828`, `12864`, `12900`, `12931`, `12973`, `13009`, `13041` |
| ontology | 7 | `11999`, `12076`, `12377`, `12414`, `12447`, `12490`, `12528` |
| HR | 5 | `2097`, `2139`, `2188`, `2235`, plus `4255` group-admin |
| governance | 5 | `12614`, `12645`, `12678`, `12709`, `13150` |
| console rollout | 3 | `7723`, `7764`, `7808` |
| other | 2 | `171` (`listAuditLog`), `4289` |

**Six of these already have a correct typed schema in the file.** The orphan
scan found 16 schemas unreachable from any path, and among them:

- `TraversalGraph` / `TraversalNode` / `TraversalEdge`
  (`openapi.yaml:32086` / `32069` / `32078`) are exactly what
  `traverse_instance` returns (`ontology/rest/src/lib.rs:1059`:
  `Result<Json<TraversalGraph>, _>`), yet `traverseOntologyInstance`'s 200 is
  opaque at `openapi.yaml:12490`.
- `InstanceState` (`:32063`), `RevisionSummary` (`:32048`), `MessengerPresence`
  (`:31739`) likewise.
- `listObjectTypes` returns `Vec<ObjectTypeSummary>`
  (`ontology/rest/src/lib.rs:268`) and `ObjectTypeSummary` **is** defined and
  correct at `:32093` — but `openapi.yaml:11996`–`11999` documents the array
  items as opaque.

That is a one-line `$ref` per operation. It is the single cheapest accuracy
win available in this file, and it is strong evidence against generation: the
authorship already happened.

Also in the orphan set: `DecideLeaveRequest` (`:31977`) is dead while
`LeaveDecideRequest` (`:26323`) is live — duplicate vocabulary for one concept,
with no deprecation marker on either.

---

## 3. Other stale identity

| Item | Status |
|---|---|
| `info.title: Maintenance FSM Backend API` `openapi.yaml:3` | **Stale.** The only true product-name staleness in the file. |
| `info.version: 0.1.0` `openapi.yaml:4` | Not stale — matches `backend/app/Cargo.toml`. |
| `servers:` | **Absent entirely.** No stale URL to fix, but no base URL for any generated client either. |
| `contact:`, `license:`, `externalDocs:`, top-level `tags:` | All absent. |
| `x-` extensions | **Zero.** Nothing referring to a deleted tool or client. |
| `mnt-` prefixed resources | **Zero.** |
| `maintenance` as product name | 1 occurrence (`:3`). The other 47 are legitimate equipment-domain vocabulary — `maintenance_type` (`:802`, `:20383`), `MaintenanceCause` (`:819`), and a `maintenance` tag (`:15525` et al.). |
| Tags | 57 distinct, 429 assignments, **no descriptions**. Case-collided: `Policy` (12) / `policy` (16), `Evidence` (4) / `evidence` (8). |
| ios / android references (`:32`, `:47`, `:25122`, `:25138`, `:19982`) | **Not stale.** These document the live passkey app-link endpoints registered at `app/src/lib.rs:2894`–`2895` and handled at `:3499`. The `DevicePlatform` enum at `:19979` is likewise live. Whether these should exist now that `android/` and `ios/` are deleted is a product question, not a documentation defect — the handler serves a valid empty document when unconfigured (`app/src/lib.rs:3497`). |

---

## 4. Security-relevant misstatements — the most serious finding

There is **no top-level `security:`** in this file. Under OpenAPI 3.1 that
means any operation without its own `security:` is **specified as
unauthenticated**. 23 of 569 operations have none. Twelve are genuinely public
and correctly so, and one (`officeCallback`) is a separate case discussed below.
**Ten require a bearer token, and two of those require admin:**

| Operation | YAML | Actual requirement |
|---|---|---|
| `listEvidenceObjects` | `:13053` | `authorize(.., Feature::EvidenceAttach)` — `crates/docs/rest/src/lib.rs:242`, helper at `:645` |
| `getEvidenceObject` | `:13073` | same — `crates/docs/rest/src/lib.rs:256` |
| `verifyEvidenceObject` | `:13081` | `crates/docs/rest/src/lib.rs:447` |
| `holdEvidenceObject` | `:13102` | `crates/docs/rest/src/lib.rs:536` |
| `createGovernanceApproval` | `:13115` | `authorize_governance` — `crates/governance/rest/src/lib.rs:187` |
| `commitInstanceLifecycle` | `:13158` | `authorize_ontology` — `crates/ontology/rest/src/lib.rs:1980` |
| `listInstanceActing` | `:13175` | `authorize_ontology` — `crates/ontology/rest/src/lib.rs:2097` |
| `resolveInstanceByCode` | `:13189` | `authorize_ontology` — `crates/ontology/rest/src/lib.rs:2133` |
| `authorizeBulk` | `:13199` | **`authorize_admin`** — `crates/platform/authz-rest/src/lib.rs:390` |
| `listPolicyDecisions` | `:13211` | **`authorize_admin`** — `crates/platform/authz-rest/src/lib.rs:433` |

These ten are contiguous (`:13053`–`:13211`), which reads like one editing
session that omitted the `security:` key rather than ten independent slips.

Verified correctly-public (12): `healthz` `:10`, `readyz` `:19`,
`appleAppSiteAssociation` `:30`, `androidAssetLinks` `:46`,
`submitSupportIntake` `:1419` (unauthenticated by design — the crate doc
comment says so at `crates/support/rest/src/lib.rs:6`, route at `:163`),
`startDeviceLogin` `:3887`, `pollDeviceLogin` `:3905`, `approveDeviceLogin`
`:3931` (authorises on a body-carried `approve_token`,
`platform/auth-rest/src/lib.rs:1837`, rate-limited at `:1828`),
`storefrontListListings` `:8493` (`sales/rest/src/lib.rs:362`, no auth
extractor), `storefrontGetListing` `:8542`, `storefrontGetListingMedia`
`:8565`, `submitInquiry` `:8595` (`sales/rest/src/lib.rs:431`, public +
rate-limited).

**`officeCallback` `:13491` is a thirteenth case and belongs in neither
column.** It is not bearer-authenticated, so it is not one of the ten above —
but it is not public either. `handle_callback` (`app/src/office.rs:947`)
requires **two** independent signatures: our own HS256 callback token in the
`?ct=` query parameter, exp-checked with no skew grace (`:957`), *and* an
ONLYOFFICE-signed payload taken from the body `token` or the
`Authorization: Bearer` header (`:961`–`:965`). The file documents neither. The
bespoke scheme is not expressible as the existing `bearerAuth` (which means a
JWT from our own issuer), so this one needs a third `securityScheme` rather than
a one-line fix — but whoever configures DocumentServer currently gets no signal
from the spec that `?ct=` is mandatory at all.

**Security schemes are otherwise accurate.** `basicAuth` (`:17717`) is declared
and used on exactly one operation — `ingestProductionSource` (`:14742`) — and
its description at `:14743` correctly states the machine-only service-principal
model. That is not a stale scheme.

**Error contract — mostly accurate, two gaps:**

- `ErrorBody` (`:25641`) matches the real shape. `RestError::into_response`
  (`crates/docs/rest/src/lib.rs:809`–`821`) emits
  `{"error": {"code": …, "message": …}}`, and the 13 `components/responses`
  wrappers (`:17859`–`17944`) all `$ref` it.
- **Inconsistent:** `createGovernanceApproval`'s 403 / 409 / 422
  (`:13152`–`:13154`) are bare `description:` strings with no content schema,
  unlike every neighbouring operation. A client generator produces no error
  type for them.
- **Missing:** `413` appears **zero times** in the file, but there is a hard
  2 MiB body cap (`app/src/lib.rs:209`, `MAX_REQUEST_BODY_BYTES`). Exceeding it
  yields axum's default rejection — **`text/plain`, not `ErrorBody`** — so a
  client that parses every error as `ErrorBody` will fail to parse this one.
  The same applies to malformed-JSON rejections: no custom rejection handler is
  installed, so those are plain text too while `:17872` promises `ErrorBody`
  for 400.

---

## 5. What generation would have to reproduce: ~3,900 hand-authored facts

Counted by construct across all 35,935 lines. "Free" means `utoipa` derives it
from the Rust type with no annotation; "hand" means it survives only if
someone writes a `#[schema(...)]` attribute or a doc comment.

| Construct | Count | Free or hand |
|---|---|---|
| `description:` | 1,503 | **hand** (relocate into `///` doc comments) |
| `summary:` | 565 | **hand** |
| `operationId:` | 569 | **hand** — all camelCase; Rust fns are snake_case, so every one needs an explicit `operation_id` |
| tag assignments | 429 | **hand** |
| `minimum:` | 243 | **hand** |
| `minLength:` | 180 | **hand** |
| `maxLength:` | 134 | **hand** |
| `default:` | 83 | **hand** |
| `maximum:` | 78 | **hand** |
| `title:` | 43 | **hand** |
| `minItems:` / `maxItems:` | 18 / 15 | **hand** |
| `pattern:` | 28 | **hand** |
| `example:` / `examples:` | 7 / 4 | **hand** |
| `discriminator:` | 8 | **hand** |
| `deprecated:` | 1 | **hand** |
| `uniqueItems:` / `readOnly:` | 1 / 1 | **hand** |
| **hand-authored total** | **≈ 3,910** | |
| `required:` | 1,498 | free (from `Option` / `#[serde(default)]`) |
| `format:` | 735 | free (`Uuid` → uuid, `i64` → int64) |
| `enum:` | 299 | free (from Rust enums) |
| `nullable:` | 76 | free (from `Option`) |
| `oneOf:` / `allOf:` / `anyOf:` | 50 / 18 / 4 | free **only where** the Rust enum is serde-tagged the same way; otherwise hand |

**Estimate, with its basis.** ~2,070 of the hand items are prose
(`description` + `summary`) and are mechanical to move — tedious, low-risk,
roughly 1–2 engineer-weeks of careful copying across ~90 crates, and easy to
verify by diffing the emitted YAML against the current one.

The ~824 constraint annotations (`minimum`, `maximum`, `minLength`,
`maxLength`, `pattern`, `minItems`, `maxItems`, `default`, `title`,
`uniqueItems`, `readOnly`) are the real risk. They **fail silently**: skip one
and generation still succeeds, emitting a looser schema, and no test notices.
They are also the highest-value content in the file — e.g. the If-Match ETag
pattern `^"ont-object-type-key:[0-9a-f]{32}:r[1-9][0-9]*"$` (`:12214`), the
`from_minutes` 0–1439 / `to_minutes` 1–1440 window (`:33477`), the
`source_sha256` `^[a-f0-9]{64}$` (`:23463`). Losing these converts a precise
contract into a vague one while making the file *look* healthier.

**What is genuinely free to migrate:** the extension surface is empty.
**Zero `x-` extensions**, **one** `deprecated: true` (`:26371`, on a legacy
`days` projection), **eleven** examples. So the usual "generation loses all our
custom metadata" objection does not apply here at all — that part costs
nothing.

Net: generation is a **project, not a cheap swap** — ~3,900 annotations plus a
contracts crate plus the emit-and-diff harness — and it buys drift-freedom the
existing gate could largely buy by being extended in three places.

---

## What a client integrator would currently get wrong

Ordered by how much damage trusting the file causes. This is the section that
matters, because the file is live at `/openapi/openapi.yaml`.

1. **They would build ten authenticated endpoints as public.** Including
   `POST /api/v1/policy/authorize-bulk` and `GET /api/v1/policy/decisions`,
   which need **admin**. Every call 401s. Worse, an API gateway or a security
   scanner configured from this file treats them as intentionally open and
   raises no finding. `openapi.yaml:13053`–`13211`.
2. **They cannot type any response from the ontology, governance, policy or HR
   read surfaces.** 32 operations return `Map<String,Object>` in every
   generated SDK — including the entire Cedar policy authoring flow
   (`:12742`–`:13041`, 10 operations) and `GET /api/v1/ontology/object-types`
   and `/{key}`. For the first vertical this is the difference between a typed
   client and hand-written JSON poking.
3. **They cannot construct a valid `POST /api/v1/ontology/object-types` body.**
   The four child arrays are contentless (`:31414`–`:31433`), so nothing tells
   them that a link needs `stable_key`, `title` and `cardinality`, or that a
   property needs `key`, `title` and `field_type`. First request fails; the
   only recovery is reading Rust.
4. **They may build against the wrong object-type surface.** Two live,
   documented, differently-shaped endpoints answer to "object types":
   `/api/v1/object-types` returning `ObjectTypeResponse` (`:19009`:
   `kind`, `code_prefix`, `description`, `status`, `active_count`; route at
   `app/src/objects.rs:201`, handler returning `Json<Vec<ObjectTypeResponse>>`
   at `:1560`–`:1564`) and `/api/v1/ontology/object-types` returning
   `ObjectTypeSummary` (`:32093`: `id`, `stable_key`, `title`, `backing_kind`,
   …). Nothing in the file marks either as legacy — there is exactly one
   `deprecated:` marker in 35,935 lines and it is on an unrelated field.
5. **Their error handling breaks on two paths the file does not mention.** A
   body over 2 MiB and a malformed JSON body both return `text/plain`, not the
   `ErrorBody` the file promises for 400 (`:17872`). `413` is undocumented
   entirely.
6. **Two client classes per domain, and wrong parameter names.** The
   `Policy`/`policy` and `Evidence`/`evidence` tag collisions split each domain
   across two generated classes. Separately, 66 path parameters are documented
   in `camelCase` where the router uses `snake_case` — harmless on the wire,
   but every generated method signature disagrees with the server.
7. **No base URL.** No `servers:` block, so every generated SDK needs manual
   host configuration before its first call.
8. **They see the wrong product name** on line 3.

---

## What I did not check

Stated so the audit's limits are legible.

- **939 of 956 schemas.** I sampled 17 (1.8%), weighted to the first vertical
  as instructed. The 18% failure rate is the rate *in that sample*, not in the
  file. I would expect a different rate in the read-heavy list/page schemas of
  inventory, financial, sales, recruiting and evaluation — untouched here.
- **Response schemas generally.** My sample was request-weighted (12 requests,
  5 responses) because requests are where an integrator fails first. The
  32-opaque-body count is exhaustive, but I verified only 3 of those 32 against
  their Rust type.
- **HTTP method accuracy.** I compared *paths*, not path×method. A documented
  `PUT` on a path that only serves `GET` would not appear in my numbers.
- **Query parameters and headers.** Only checked incidentally, within the 17
  sampled operations. The 20 `components/parameters` entries
  (`:17724`–`:17858`) were read but not verified against extractors.
- **Status-code completeness.** I checked `413` (absent) and read the 13
  `components/responses`. I did not verify, per operation, that every status
  the handler can emit is documented.
- **`oneOf`/`allOf`/`discriminator` correctness.** Counted (50/18/8), not
  validated against the corresponding serde representations.
- **Nothing was executed.** No cargo, no buck2, no server. `openapi_drift.rs`
  was read, not run, so my claims about it are claims about its source. The
  route extraction is static parsing of `.route(...)` calls with crate-local
  constant resolution — it resolved all 515 with zero unresolved arguments, but
  it would miss a route registered through a macro or a runtime-built path.
- **`console-lanes/lane-1`, `lane-3`, `lane-4`, `lane-5`** were not entered, as
  instructed. All figures describe the working tree at `docs/ecosystem-plan-session`.

### Reproducing the counts

Every number above comes from static extraction over the working tree:
path keys from the `paths:` block; served paths from `.route(...)` calls across
all 53 non-test router files with crate-local `const … &str` resolution;
inventory paths by resolving the 41 constants referenced at
`app/src/lib.rs:224`–`392`; schema names and orphan reachability by transitive
`$ref` closure from the `paths:` section; construct counts by anchored line
match. The one non-obvious requirement is crate-local constant resolution —
resolving globally silently mis-maps `OBJECT_TYPES_PATH` and fabricates a
documented-but-unserved path that does not exist.
