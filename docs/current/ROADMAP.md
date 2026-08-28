# Console roadmap authority

Status: active roadmap authority. Product scope comes from [`PRODUCT.md`](PRODUCT.md); delivery and issue state come from [`DELIVERY.md`](DELIVERY.md).

## Ordered work

1. **Documentation custody and active authority**
   - Establish README plus PRODUCT, ROADMAP, and DELIVERY as the only active authority concerns.
   - Generate the full first-party tracked-document manifest with class, owner, status, replacement, retention, and blob SHA. Exactly one of 454 records is a signed, validator-read-back archive probe and `archive_tag` remains null for the other 453; `scripts/check-doc-links.mjs` and `scripts/console/generate-documentation-manifest.mjs` fail closed on every non-null reference. GitHub has no tag-target ruleset and its single origin is not independent off-device custody, so this probe does not authorize `coverage: complete`, bulk moves, deletion, or graveyard copies. Vendored trees (`third-party/`, 9 tracked files) remain outside that project-owned universe; former tracked agent-delivery trees such as `.grok/` have been removed and cannot regain authority merely by reappearing locally. `buck-out/`, `node_modules/` and `target/` are also excluded but contain no tracked Markdown.
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
   - Group / multi-corporate is in scope as the existing platform overlay (migration 0060), not as `ObjectKey::Group`. Next: Company/OrgUnit reference and canonical HR assignment writer without reopening or widening the released boundary. Consolidated-read proofs over authorized member subsets are later overlay work, not a seventh object.
6. **Payroll**
   - Project the existing payroll writer without a second write path; preserve deterministic rounding, golden cases, immutable receipts, and payslip drafts.
7. **Leptos acceptance surface**
   - `Layer::Ui` is accepted (ADR-0041); first full-depth vertical is payroll execution.
   - Owner decision (2026-08-28) **starts** the mounted contracts-backed SSR shell in parallel, including while item 2 remains incomplete. That start does not wait for executable-contract decoupling to finish.
   - Shipping screens and production exposure stay **HOLD** until persona-based real-backend E2E evidence (ADR-0025 §4). Import/export is not the data-entry base except 자료실; the comms rail is out of this slice.
8. **Palantir AIP / Intelligence**
   - Palantir AIP is the target intelligence layer. It is built in the separate Intelligence repository until that repository names a SHA-bound stable base; cloning it into Console is a later lane. Not current Console tenant-app work.

## Explicit HOLDs

- Bulk documentation moves, deletion, or graveyard copies are **HOLD** until custody and recoverability are proven with the full manifest and signed archive references.
- Leptos **shipping** remains **HOLD** until persona-based real-backend E2E evidence. Owner decision (2026-08-28) starts the mounted SSR shell; that start does not authorize shipping screens or live exposure.
- Live or production promotion, DNS, TLS, secrets, exposure, payment, credential-reset, and compliance claims are **HOLD** absent separate authority. A source release follows repository release authority and evidence; it does not authorize live promotion or exposure.
- Destruction, termination, resize, or reprovisioning of the grandfathered OCI Ampere A1 instance (4 OCPU / 24 GB) is permanently **HOLD** because the reserved capacity cannot be recreated.
- Workstation full-disk erase **already occurred** (post-wipe, owner fact 2026-08-28). Remaining local P0 work is re-issue or off-device confirmation, not a future erase. Do not erase again as a program action.
- Cloning Intelligence into Console is **HOLD** until Intelligence names a SHA-bound stable base. Intelligence is not a tenant application and not autonomous merge authority.
- Ambiguous roadmap prose, historical plans, and unpublished or partial work do not clear a HOLD and do not dispatch implementation.

## Exit rule

A roadmap item advances only when its exact candidate, independent review, acceptance evidence, post-merge containment, and remaining HOLDs are recorded under [`DELIVERY.md`](DELIVERY.md). Partial completion remains open work.
