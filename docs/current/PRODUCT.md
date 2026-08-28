# Console product authority

Status: active product authority.

## Product boundary

Console builds one governed company object system in this order:

1. Ontology / Foundry-style object engine and deterministic policy.
2. Company, OrgUnit, JobPosition, Person, and Employment.
3. HR appointment, promotion, and transfer through one canonical assignment writer.
4. Payroll projected from existing payroll truth as PayRun.
5. A Leptos SSR frontend. ADR-0030 substrate gates are green and `Layer::Ui` is accepted (ADR-0041); first full-depth vertical is payroll execution. GET `/_ui` serves deny-by-omission SSR (`console-payroll-ui` nested by `console-app`; #952). Unauthenticated and MEMBER get the empty shell; `PayrollRunRead` gets OpenAPI `PayrollRunSummary` required fields as `data-run-*` attributes, no won (#959). Remaining frontend work: islands/WASM hydration, persona E2E. Shipping screens, production exposure, and persona E2E remain **HOLD**.

**Group / multi-corporate is in scope.** Group is an abstract entity, not a tenant and not a legal person. Multiple corporates (separate legal entities; `organizations`) sit under a Group. Authorized reads may be the group, any authorized combination of member corporates, or one corporate. This overlay already exists (migration 0060 `groups` / `group_memberships` with `UNIQUE (org_id)`, grants `GROUP_ADMIN` / `GROUP_VIEWER` / `GROUP_FINANCE`, fail-closed `group_member_org_ids`, `console-platform-group::consolidated_read` as N× `with_org_conn`). Group never arms `app.current_org`. The canonical `ObjectKey` roster stays six; Group is not a seventh projected object. Writes stay one `org_id`. Empty grant → empty view.

Console adopts Foundry / Palantir **system patterns** (ontology on data, lineage of writes, applications on objects, write-back through owning actions) on this Postgres + object-type substrate. It does not use Palantir SaaS and does not import or export Palantir files.

**Palantir AIP is the target** for the intelligence layer. That layer is built in the separate Intelligence repository until Intelligence has a named, SHA-bound stable base; cloning it into Console is then authorized. Until that clone lane, AI judgment remains out of Console's tenant product. Intelligence is not a tenant application and is not autonomous merge authority. Do not call Palantir APIs.

ERP and finance modules, communications, compliance products, ingest/evidence products, office editing, payment execution, and unrelated verticals remain out of scope. Existing code and documents in those areas are historical, quarry, evidence, or maintenance-only; their presence does not authorize expansion.

## Product invariants

- Commands are deterministic and revision-aware, with replay-safe receipts and auditable mutations.
- Tenant isolation, deny-by-omission, and nondisclosure apply at every read and write boundary.
- Effective-dated truth uses half-open intervals; history is closed and appended, never overwritten.
- Projected objects have exactly one domain writer. Ontology and adapters do not create alternate write paths.
- Requester and approver are distinct natural persons for `company.*`, `hr.*`, and `payroll.*` approvals, even when their capacities differ. All other kinds — including `organization.*` and `people.*` — hold only the account-level `approver_id <> requested_by` bar. `requires_natural_person_four_eyes` (`backend/crates/governance/domain`) is prefix-scoped and a test asserts the exclusion. Extending the bar to the remaining kinds is **unscheduled, not blocked**: migration 0076 shows only that `users.employee_id` is nullable, and a NULL resolution can fail closed exactly as the enforced kinds already do. Whether unbound accounts actually exist is an open question no census has answered, so the compatibility risk is a hypothesis on HOLD rather than a reason to leave six of thirteen dispatch targets outside the invariant.
- Preflight uses the same authorization, policy, state, revision, and input validation as execute and performs no mutation.
- Legal sources are versioned evidence, not transferable compliance conclusions. Production exposure and compliance claims require separate authority.

## Architecture

The existing Rust backend is reused as verified substrate rather than rewritten wholesale. The product exposes governed company objects and authorized actions:

`Company → OrgUnit → JobPosition → Person/Employment → HR action → PayRun`

Group is a platform overlay over those Company tenants, not a node on this chain and not an `ObjectKey`. REST and any future server functions are *intended as* sibling adapters over the same application-layer use cases. This is an aspiration, not an enforced boundary: `Layer::allowed_deps` lets `Layer::Rest` reach Adapter, Platform, Domain and Kernel directly, and 6 of the 34 non-platform REST crates (analytics-quant, consulting, facilities, logistics, orgchange, production) declare no application dependency at all. Counting every package whose name ends `-rest` the ratio is 9 of 37: `console-platform-auth-rest`, `console-platform-authz-rest` and `console-platform-rest` also declare none, and are excluded above only because the layer gate classifies `crates/platform/*` as Platform before it reaches the `-rest` suffix. Converging them is unscheduled work. The frontend reads real contracts, omits unauthorized data server-side, and contains no client-side business authority. Cargo is the target build system; existing Buck paths remain repository reality until a dedicated, evidence-backed convergence change removes them without losing test coverage. `-ui` members are skipped by first-party Buck generation until Leptos is vendored.

## Holds

- Frontend **shipping** remains **HOLD**. ADR-0030 substrate gates are green and `Layer::Ui` is accepted (ADR-0041); those facts do not authorize shipping screens or live exposure. GET `/_ui` is mounted (deny-by-omission SSR, Leptos 0.9.0-beta, #952). Contracts-backed authorized reads (#959) do not put business authority in the client and are not persona E2E. Remaining: islands/WASM hydration, persona-based real-backend E2E (ADR-0025 §4). React tombstone paths stay absent. Deny-by-omission is server composition. Import/export is not the data-entry base except 자료실 (later, not this slice). The comms rail is not in this slice. 0.8.x is the rollback if 0.9.0-beta blocks production.
- Company, Person, Employment, and PayRun projection fan-out is **RELEASED** (2026-08-19, by repository-owner decision). The stated condition — an explicit owning port and a proven single-writer boundary for each — is met, and stays checked rather than asserted once: `node tools/ci/hold-release-conditions.mjs` runs in the preflight sweep and reports the owning crate, owned tables, proving port suite and PostgreSQL-job wiring for all six canonical objects; the writer-ownership gate holds all twenty owned tables with an empty `KNOWN_SECOND_WRITERS` ratchet that `stale_exemptions()` can only shrink; and the boundary is mutation-proven — injecting one second writer of `employment_heads` or of `payroll_disbursements` into a non-owner crate turns three tests red, and reverting returns the suite to green. This releases projection fan-out only. It confers no authority over any other hold in this list, and it does not assert per-row attribution on the shared receipt store, which is a separate boundary tracked apart from projection.
- Live production, DNS, TLS, secret, exposure, payment, credential-reset, and compliance-claim actions are **HOLD** without separate authority and evidence.
- Korea compliance conclusions remain **HOLD** pending qualified authority.
- The grandfathered OCI Ampere A1 instance (4 OCPU / 24 GB) must **never** be destroyed, terminated, resized, or reprovisioned; re-creation permanently loses the reserved capacity.
- Full-disk erase of the owner workstation **already occurred** (post-wipe, owner fact 2026-08-28). This is not a future HOLD. Do not erase again as a program action. Do not invent a pre-wipe custody ledger for bytes that are gone. Remaining local P0 categories (signing identity, account recovery and passkeys/2FA, OCI/Talos access, local secret files, restricted business inputs, unpublished repository/worktree bytes) are re-issue or off-device confirmation on this machine, not a reason to wait. Destroy/reprovision of the Ampere A1 instance remains HOLD (that is not the laptop).
- Cloning Intelligence into Console is **HOLD** until Intelligence names a SHA-bound stable base. Until that clone lane, AI judgment remains out of Console's tenant product.
- Historical documents, branches, chats, handoffs, and transient runtime state are context or evidence only. They cannot clear these holds; the current handoff may inventory custody evidence without becoming product authority.
