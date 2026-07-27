# CAP-FIELD-CONSOLE — gap analysis & owning-crate decisions

> Gap = design intent (mirror) vs backend/frontend reality (this worktree,
> HEAD migrations = 0180, `EXPOSED_SCREEN_KEYS = ["sales"]`). Each gap names the
> owning crate and whether THIS lane closes it (support-crate closure only) or
> files a manifest entry.

## 1. Closed by this lane (support crate, migration 0194)

| # | Gap | Closure |
|---|---|---|
| G1 | Tickets have no site/customer/work-order binding — the field console pivots on sites but `support_tickets` can't answer "which site" | 0194 columns + `POST /tickets/{id}/link` + `site_id` list filter (design-contract §2.2-C, §3, §6) |
| G2 | No per-site overview read model (list layer: site × customer × open issues × SLA × active visits × last arrival) | `GET /api/v1/field/sites` aggregation (§2.2-A). Decision: lives in support/rest — support owns intake/SLA and the field console is its primary consumer; read-only SQL joins to registry/workorder/compliance tables are documented coupling (writes/DDL stay with owners) |
| G3 | No site object-detail read model (object + history layers) | `GET /api/v1/field/sites/{id}` (§2.2-B) |
| G4 | No customer-acceptance record — story requires acceptance as audited closure evidence feeding SLA/billing | `support_ticket_acceptances` table + `POST /tickets/{id}/acceptance` driving RESOLVED→CLOSED / reopen (§2.2-D). Reuses existing FSM edges — no parallel state machine |
| G5 | SLA state not derivable per site | deterministic derivation rule (§5), computed in the overview/detail queries |

## 2. Manifest — workorder crate (maintenance lane owns; DO NOT touch here)

| # | Need | Why |
|---|---|---|
| W1 | `GET /api/v1/work-orders` (or v1 list) accepts `site_id` filter | field detail composes visit list per site; support's read-join covers the rollup but the console's WO drill-through needs the workorder API to answer per-site queries natively |
| W2 | "Create work order from support ticket" prefill path: accept optional `source_ticket_id` (echoed in response / status history) so dispatching a visit from an intake keeps provenance | story step 2; support stores the link (G1) but WO-side provenance belongs to workorder |
| W3 | JL- work-log projection: a per-site, date-keyed read of submitted reports (`report_submitted_at/by`, diagnosis, action_taken, result_type, evidence refs) | design §2: JL- = 일자×현장×작성자, cross-linked 근태/정비; field history layer wants it first-class rather than reconstructed client-side |
| W4 | Manual check-in/out endpoint for a dispatched visit (device-attested; geofence-verified) writing `site_attendance_events` | prototype `attCheckIn` is manual+deterministic (device × geofence, fail-closed); current backend records ARRIVAL/DEPARTURE only via GPS ping ingest (compliance). Owning decision workorder×compliance — flagged for the maintenance lane to arbitrate |

## 3. Manifest — registry crate (this lane may NOT add REST there; small & optional)

| # | Need | Why |
|---|---|---|
| R1 | (optional) `GET /api/v1/sites` include `customer_name` + geo in list DTO if not already present | avoids N+1 on the field screen; support overview (G2) already denormalizes, so this is only for the registry-native site picker |

## 4. Manifest — shared collision roots (consolidation integrator)

Machine-readable copy: `integration-manifest.json` (same dir).

| Root | Entry needed |
|---|---|
| `web/src/console/shell/nav.ts` | `"field"` appended to `MOUNTED_SCREEN_KEYS` (nav item for screen `field` ALREADY exists at lines 230–236, gate `OPERATIONAL_ROLES × work_order_read_all`; ko label `console.shell.nav.field` = "고객·현장" already in ko.ts line 983). NO `EXPOSED_SCREEN_KEYS` change — module stays DARK per ADR-0025 |
| `web/src/console/screens/registry.ts` | `field: FieldScreenBody` (import from `../field`) |
| `web/src/i18n/ko.ts` | none — nav label already present; module strings live in lane-owned `web/src/i18n/field.ts` |
| `backend/openapi/**` + `clients/**` | new paths/schemas from design-contract §2.2/§3 with per-domain `tags: [field]` (kotlin per-tag client requirement) + regenerated ts/kotlin/swift clients (3 CI drift gates) |
| `backend/crates/platform/db/migrations` | `0194_…` provisional — renumber to next free at integration |
| app composition root (`build_router` / app boot) | mount unchanged — new routes live inside the existing support router |

## 5. Design-intent gaps deliberately NOT built in this slice (registered, honest)

| Gap | Disposition |
|---|---|
| SLA policy as configurable setting object (§4-26/HO-01: threshold·window·escalation, no-code, revision staging) | `SlaPolicy` struct already override-capable; the governed-object editor is a future charter — register, don't fake |
| Site rows as ontology-engine instances (HANDOFF §18 MOD_SCREENS→engine query) | console-wide epic, not per-lane |
| Map round-trip (`지도에서 보기` with mapOv/mapSel), ops-map authoring | map screen not mounted in web console yet; link chips degrade to absent (not dead controls) |
| Series (SR-) rollups per site, 거래처 CL- dedicated module | CL- upstream link = registry customer detail for now |
| Billing reflection (acceptance/SLA breach → 정산/전표) | contract: acceptance + resolved_at/due_at are the durable evidence; finance/laborcost lanes consume — cross-lane event contract, filed here not fabricated |
| Public customer acceptance channel (customer clicks accept) | acceptance is staff-recorded with channel enum (§2.2-D); an unauthenticated customer ack link = future charter (needs tokenized egress gate) |
| Prototype `production.ts` i18n `subtitle` string pattern | grammar violation (§4-12) — field module must not copy it; noted for the exemplar's own cleanup |

## 6. Risks / decisions log

1. **Cross-table reads from support adapter** (work_orders, site_attendance_events,
   registry_*): accepted for the aggregation read model; all under org RLS as
   `mnt_rt`; adapter tests must run as the runtime role (mnt-gate-rls-arming
   lesson) and cover the site-scope confinement.
2. **Acceptance drives existing FSM edges** instead of adding states — keeps the
   ticket FSM matrix tests valid; declined acceptance = the already-legal reopen
   edge.
3. **Untriaged CUSTOMER tickets**: linking a site sets `branch_id` (= triage);
   this preserves the invariant that branch-scoped staff never see branch-less
   rows, while giving cross-branch staff a one-step site triage.
4. **`work_order_id` requires `site_id`** (CHECK) so a visit can never point at a
   ticket with no site — the story's chain stays referentially sound.
5. **Migration number collision**: 0194 is provisional; take the next free number
   right before integration push (learned via PR #223).
