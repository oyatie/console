# Console roadmap authority

Status: active roadmap authority. Product scope comes from [`PRODUCT.md`](PRODUCT.md); delivery and issue state come from [`DELIVERY.md`](DELIVERY.md).

## Ordered work

1. **Documentation custody and active authority**
   - Establish README plus PRODUCT, ROADMAP, and DELIVERY as the only active authority concerns.
   - Generate the full first-party tracked-document manifest with class, owner, status, replacement, retention, blob SHA, and archive tag; vendored and generated trees remain outside that project-owned universe.
   - Fail CI on new unclassified documents. The current `authority-slice` index is intentionally not a complete-coverage claim.
2. **Executable-contract decoupling**
   - Replace machine checks that depend on historical prose or draft ideas with source-derived or machine-readable contracts while retaining behavioral regressions.
3. **Delivery substrate convergence**
   - Prove `Required / CI` and `Required / Security` in shadow mode, then migrate branch protection to those two contexts plus the independent protected-target authority check.
   - Partition the exact 183-test PostgreSQL reachability inventory across isolated disposable databases while retaining a strict compatibility aggregate and proving no omission or duplication.
   - Prove Cargo test membership, feature-bearing reachability, JavaScript reachability, credential safety, and zero required Buck-only coverage before any Buck deletion.
   - Fix migration-parser gaps before admitting new migrations.
4. **Architecture foundations**
   - Complete branchless capability and temporal-grant contracts, contracts-crate/OpenAPI composition, true preflight, and distinct-human approval rules.
5. **Organization and HR**
   - Accept explicit owning ports and single-writer boundaries, then build the Company/OrgUnit reference and canonical HR assignment writer.
6. **Payroll**
   - Project the existing payroll writer without a second write path; preserve deterministic rounding, golden cases, immutable receipts, and payslip drafts.
7. **Leptos acceptance surface**
   - Only after its holds clear, land the contracts-only SSR shell and real organization, HR, and payroll surfaces using deny-by-omission.

## Explicit HOLDs

- Bulk documentation moves, deletion, or graveyard copies are **HOLD** until custody and recoverability are proven with the full manifest and signed archive references.
- JobPosition and projection fan-out are **HOLD** until the owning-port conditions in PRODUCT are satisfied.
- Leptos and other frontend work are **HOLD** until the PRODUCT frontend conditions are satisfied.
- Live or production promotion, DNS, TLS, secrets, exposure, payment, credential-reset, and compliance claims are **HOLD** absent separate authority. A source release follows repository release authority and evidence; it does not authorize live promotion or exposure.
- Destruction, termination, resize, or reprovisioning of the grandfathered OCI Ampere A1 instance (4 OCPU / 24 GB) is permanently **HOLD** because the reserved capacity cannot be recreated.
- Workstation erase is **HOLD** until the reviewed Console candidate is on `main` and the PRODUCT custody gate has an itemized, read-back-verified preservation or explicit discard/reissue disposition for every local-only P0 item.
- Ambiguous roadmap prose, historical plans, and unpublished or partial work do not clear a HOLD and do not dispatch implementation.

## Exit rule

A roadmap item advances only when its exact candidate, independent review, acceptance evidence, post-merge containment, and remaining HOLDs are recorded under [`DELIVERY.md`](DELIVERY.md). Partial completion remains open work.
