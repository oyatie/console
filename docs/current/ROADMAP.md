# Console roadmap authority

Status: active roadmap authority. Product scope comes from [`PRODUCT.md`](PRODUCT.md); delivery and issue state come from [`DELIVERY.md`](DELIVERY.md).

## Ordered work

1. **Documentation custody and active authority**
   - Establish README plus PRODUCT, ROADMAP, and DELIVERY as the only active authority concerns.
   - Generate the full first-party tracked-document manifest with class, owner, status, replacement, retention, and blob SHA. Archive tags remain structurally unpopulated: `archive_tag` is null for all 453 records, and `scripts/check-doc-links.mjs` and `scripts/console/generate-documentation-manifest.mjs` both require null until a signed-archive validation contract exists. Vendored trees (`third-party/`, 9 tracked files) remain outside that project-owned universe; former tracked agent-delivery trees such as `.grok/` have been removed and cannot regain authority merely by reappearing locally. `buck-out/`, `node_modules/` and `target/` are also excluded but contain no tracked Markdown.
   - Fail CI on new unclassified documents. The index reached `first-party-manifest` coverage, which is complete over the checker-defined universe rather than over every tracked Markdown file: `check-doc-links` enforces exactly one record per document inside that universe and rejects records outside it. Agent/runtime trees such as `.grok/` remain outside that universe; the exclusion is not permission to restore them or place active authority there. `authority-slice` survives only as a legacy alternative the checker still accepts.
2. **Executable-contract decoupling**
   - Replace machine checks that depend on historical prose or draft ideas with source-derived or machine-readable contracts while retaining behavioral regressions.
3. **Delivery substrate convergence**
   - Done: `Required / CI` (`.github/workflows/ci.yml`) and `Required / Security` (`.github/workflows/security.yml`) are out of shadow mode, report success on recent runs, and are merge-blocking required contexts on `main` alongside the independent `authenticate-console-authority` check.
   - Keep `main` as the canonical integration and source-release branch. Environment-named branches are non-authority mirrors and their promotion remains HOLD until a current authority records the branch model and every required context has an executable protected producer.
   - Partition the exact 214-target PostgreSQL reachability inventory (229 mapped entries, 214 in-workflow targets). These are Cargo invocations, not test cases -- one entry runs one test binary, and `governance_rls_as_runtime_role` alone carries 12 tests. An exact test count must come from discovered/executed output, never from this number across isolated disposable databases while retaining a strict compatibility aggregate and proving no omission or duplication. The partitioning, the five disposable-database facet jobs, and the fail-closed aggregate are already implemented.
   - Prove Cargo test membership, feature-bearing reachability, JavaScript reachability, credential safety, and zero required Buck-only coverage before any Buck deletion.
   - Fix the remaining migration-parser gaps. The migration-safety gaps (`ALTER TABLE ONLY`, schema-qualified audited tables) are fixed. The personal-data-classification gaps remain open — a multi-action `ALTER TABLE` is still judged from its first action, and concatenation-split keywords are still unreachable — and migrations 0212-0221 were admitted since 2026-08-04 under the compensating catalog-based completeness assertion in `backend/crates/platform/db/tests/personal_data_classification.rs`, not under a parser fix.
4. **Architecture foundations**
   - Complete branchless capability and temporal-grant contracts, contracts-crate/OpenAPI composition, true preflight, and distinct-human approval rules.
5. **Organization and HR**
   - Done: explicit owning ports and single-writer boundaries are mechanically proven for Company, OrgUnit, JobPosition, Person, Employment, and PayRun, releasing projection fan-out only.
   - Next: build the Company/OrgUnit reference and canonical HR assignment writer without reopening or widening the released boundary.
6. **Payroll**
   - Project the existing payroll writer without a second write path; preserve deterministic rounding, golden cases, immutable receipts, and payslip drafts.
7. **Leptos acceptance surface**
   - Only after its holds clear, land the contracts-only SSR shell and real organization, HR, and payroll surfaces using deny-by-omission.

## Explicit HOLDs

- Bulk documentation moves, deletion, or graveyard copies are **HOLD** until custody and recoverability are proven with the full manifest and signed archive references.
- Leptos and other frontend work are **HOLD** until the PRODUCT frontend conditions are satisfied.
- Live or production promotion, DNS, TLS, secrets, exposure, payment, credential-reset, and compliance claims are **HOLD** absent separate authority. A source release follows repository release authority and evidence; it does not authorize live promotion or exposure.
- Destruction, termination, resize, or reprovisioning of the grandfathered OCI Ampere A1 instance (4 OCPU / 24 GB) is permanently **HOLD** because the reserved capacity cannot be recreated.
- Workstation erase is **HOLD** until the reviewed Console candidate is on `main` and the PRODUCT custody gate has an itemized, read-back-verified preservation or explicit discard/reissue disposition for every local-only P0 item.
- Ambiguous roadmap prose, historical plans, and unpublished or partial work do not clear a HOLD and do not dispatch implementation.

## Exit rule

A roadmap item advances only when its exact candidate, independent review, acceptance evidence, post-merge containment, and remaining HOLDs are recorded under [`DELIVERY.md`](DELIVERY.md). Partial completion remains open work.
