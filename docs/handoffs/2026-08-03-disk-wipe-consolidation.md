# Disk-wipe consolidation handoff — 2026-08-03

This is the durable restart record for a machine with none of the old worktrees,
branches, caches, ignored files, or chat context. Read it only from the latest
`origin/main`; local refs named below are evidence identities, not continuation
dependencies.

## Manual blockers before disk erase

Repository merge completion does not preserve workstation secrets or external
business inputs. No off-device data volume or verified cloud-secret-manager
transfer was observable during this consolidation. **Do not erase the disk**
until the owner has completed and read-back verified the custody decisions in
the [ignored files and secrets](#ignored-files-and-secrets) section.

The highest-value blocker is `~/.ssh/id_ed25519`: the candidate and authority
verifiers pin its public fingerprint
`SHA256:5grGNUtX9Zgmy1SWne6wF9DR8W1ElUQaF/Z8SYRz8E8`. A newly issued key is not
an equivalent replacement and will not pass the current trust gate. Either copy
the exact private key to approved encrypted off-device custody and verify the
restored public fingerprint, or complete a separately reviewed trust-root
rotation while the old key still works. The 189 MB `~/.config/talos-mnt/**`
recovery directory likewise needs an actual encrypted off-device archive with a
read-back hash, or an itemized owner-approved discard/reissue decision; hashes
listed in this handoff cannot reconstruct its bytes.

## Fresh-session entrypoint

Restore GitHub authentication and the repository's signed-commit configuration
from approved external custody, then start from a new clone with no local branch
other than `main`:

```sh
gh auth status
gh repo clone jason931225/console console
cd console
git switch main
git pull --ff-only
gh pr list --state open
git config --local user.name "Jason Lee"
git config --local user.email "jason19931225@gmail.com"
git config --local gpg.format ssh
git config --local user.signingkey ~/.ssh/id_ed25519
git config --local gpg.ssh.allowedsignersfile .github/trust/console.allowed_signers
git config --local commit.gpgsign true
ssh-keygen -lf ~/.ssh/id_ed25519.pub
```

The final command must report the pinned fingerprint above before signed work
continues.

Then read, in order:

1. `AGENTS.md`
2. `HANDOFF.md`
3. this file
4. `docs/PIVOT-2026-07-28.md`
5. `docs/program/README.md`
6. `.omx/plans/reasoning-lens-contract-execution-handoff.json`

At candidate-writing time, PR #552 was merged and release `v0.3.1` pointed to
`435e251edfab12750850d5b1d411528b10a3ed8a`; PR #562 was still the sole open PR.
That is a historical observation, not a completion claim. Do not wipe until a
later closeout on `main` records #562's exact candidate/authority/merge identities,
green run URLs, branch-protection readback, and any generated release PR. In a
future session, inspect those exact identities; a newly opened unrelated PR or a
newer CI run does not retroactively make this consolidation incomplete.

## Canonical branch and scope

- `main` is the only integration target. No `dev` or `develop` branch existed
  locally or remotely during the audit.
- The remote-ref audit began with 252 branch heads. It removed 250 exact,
  individually enumerated heads: 167 were heads of merged PRs, 76 were
  unassociated pre-pivot leftovers, six belonged to closed-unmerged PRs whose
  only useful final net was already preserved here, and one was the rejected
  post-pivot Cosskorea lane. During review, only `main` and PR #562's
  `docs/session-handoff-2026-08-02` head remain. GitHub's
  `deleteBranchOnMerge` setting is enabled; after #562 and any release PR merge,
  verify that `main` is the sole branch rather than recreating deleted refs. The
  exact point-in-time deletion inventory is
  [`2026-08-03-remote-branch-deletion-manifest.tsv`](2026-08-03-remote-branch-deletion-manifest.tsv).
- The product boundary remains Ontology/Foundry/Policy → Company/OrgUnit/Employee
  → HR/Payroll. ERP, field operations, dispatch, communications, compliance as a
  product, ingest/evidence, office editing, AI judgment, and frontend work remain
  outside the active pivot unless a later owner decision changes it.
- Cargo is the chosen build system. Buck2 is still present because its remaining
  test/build reachability must be replaced with measured Cargo parity before a
  dedicated removal train deletes it.
- Local branch names, worktree status, OMX/OMC runtime state, and old handoffs do
  not establish program completion or authority.

## CI and merge boundary

- Release `v0.3.1` and its exact `main` commit passed the complete CI workflow,
  including the serialized PostgreSQL reachability lane, before the
  consolidation candidate was finalized.
- PR #562 published the truthful `API contract — text-only contract checks`
  context. Before merge, branch protection replaced the obsolete
  `API contract — app-served OpenAPI` requirement with that exact context and
  replaced `Company conformance — 12/12 required target` with
  `Company conformance — generic-engine regression`, then added the already-running
  `Domain crates — unit tests` and
  `dev-up.mjs smoke — compose deps + migrate + /readyz` contexts. Strict
  up-to-date checks, the existing GitHub Actions app binding, every other
  required context, stale-review dismissal, and review-conversation resolution
  remain enabled and were read back after the update.
- Every final candidate is reviewed at its exact SHA by two independent
  adversarial reviewers, with no unresolved findings and all required checks
  green before squash merge. The owner subsequently corrected the GitHub review
  rule for this solo-development repository: historical instructions requiring
  one non-author approval and approval by someone other than the last pusher are
  superseded. Current `main` protection requires zero GitHub approvals and sets
  `require_last_push_approval` to false. This correction does not waive the two
  independent exact-object reviews, required hosted checks, conversation
  resolution, or immediate pre-merge protection readback.
- Interim signed tip `900c1749a94f945deaa85cc5097a51287620dd95` was
  pushed to PR #562 as a remote disaster-recovery checkpoint and then made draft
  after hosted verification exposed defects. It is superseded by the later exact
  C/T pair recorded in the final authority ledger and must never be merged on the
  strength of its partial checks.
- Signed repair checkpoint `baadf03cf4692754fd0b964834e324d14e48f20e`
  was also pushed while the PR remained draft. It adds the reviewed Buck test-face
  reconciliation and a provenance-safe deploy-test harness. A clean fast verifier
  passed every non-authority stage at that checkpoint; its two exact-M admissions
  correctly rejected the head because a content checkpoint is not the required
  ledger-only direct-child authority tip. It is recoverable evidence, not merge
  authority; only the replacement C/T pair may proceed.
- Signed authority tip `3885ed49b4bf2906f75c1907cfcd7f03f9aac0c3`
  subsequently passed the protected-main simulator, 72 authenticated candidate
  checks, and the complete fast verifier. Exact-object review nevertheless revoked
  it: two documents listed as current authority still instructed a fresh session
  to freeze the generic `company_conformance` fixture and dispatch a
  JobPosition/projection pilot, contradicting the roadmap and catalog HOLD. Those
  instructions are now reconciled. Review also found the same obsolete product-
  target claim in the CI job label, an accepted ADR that named path filters the
  workflow intentionally forbids, and a pre-pivot readiness record presenting
  historical deployment and test claims as current. Those claims are narrowed or
  retired. A later exact-object pass also reproduced a dangerous retention-floor
  blind spot: the catalog completeness reader accepted non-public schemas,
  materialized views, and foreign tables that the derivation ignored. It found
  exactly five free-form JSONB columns missing the migration's own `undeclared`
  class and two touched API-contract mismatches. The replacement repairs align
  the readers, mutation-lock all five classifications, and reconcile those API
  contracts. No simulation, CI, or review result from the superseded tip carries
  forward to the final replacement pair.
- Hosted run `30834869062` against that revoked tip also failed the serialized
  PostgreSQL lane: `mobile_evidence_fixtures.rs` was absent from the generated
  workorder source map. The replacement exports both shared fixture modules,
  maps all four path-module users, and wires three previously dark dispatch/mobile
  binaries into the PostgreSQL lane. All four Buck targets build locally and
  their 15 PostgreSQL tests pass; hosted proof still belongs to the replacement
  exact tip, not to this local measurement.
- Signed tip `db7eda3ec2783ca93039b9d03cc2aaade613a927` then passed local
  verification, protected-main simulation, authenticated authority checks,
  independent exact-object review, and the hosted authority bootstrap. It was
  nevertheless revoked before review or merge when a deeper ignored-file audit
  found twelve OMX files still named by tracked records and disproved the
  handoff's broad self-contained-planning claim. Ten stale pre-pivot reference
  edges are now explicitly retired by hash; two workbook-derived profiles remain
  classified as restricted external inputs alongside their source. Preservation
  or discard and any read-back remain pending. No result on that tip carries
  merge authority into any later correction pair.
- Local signed tip `17f84f210a9dbf32987cdff36d4aeabaff4540ac`
  was never pushed. Independent exact-object review refused it because three
  durable statements implied the restricted workbook inputs had already reached
  external encrypted custody even though no usable destination or read-back
  exists. Those statements are now conditional on the still-pending custody-or-
  discard decision; no result on that local tip carries merge authority.
- Two signed archive tags make provenance commits outside the post-pivot main
  ancestry reproducible from a fresh clone:
  `archive/pre-pivot-implementation-freeze-2026-07-24` resolves to
  `78cb5197927a031ead30c6dc0426c23455d3cb16`, and
  `archive/pivot-authority-base-2026-07-28` resolves to
  `d138ed28b65fa6dbec01bd8022be5a4e1db57687`. The truth-ledger validator
  binds the advertised refs to their exact commits; a bare SHA is no longer
  accepted as durable custody.

## Non-authoritative external opinion

During consolidation the user relayed an external static audit as an opinion,
not repository authority. It praised the domain model and supply-chain controls
while criticizing unfinished pivot convergence, CI brittleness, repository
size, and operational-readiness claims. The auditor did not run the complete
PostgreSQL suite or deploy the system, and individual observations may be
incomplete, stale, or wrong. Treat every score, claim, and recommendation only
as a hypothesis. The audit authorizes no deletion, restructuring, product
scope, release, deployment, or production claim; only findings reproduced
against current authoritative sources may enter implementation or program
state.

## What was preserved

The following useful final nets were prepared in the signed consolidation
candidate intended for PR #562. They become durable only when the PR head is
pushed and read back at the exact candidate/authority tip, then merged. Source
identities let future archaeology distinguish reviewed work from coincidental
similarity; no source branch is a continuation dependency.

| Area | Preserved outcome | Source identity |
| --- | --- | --- |
| Earlier PR #562 | Full 2026-08-02 handoff, its correction ledger, a main-gate simulator, and MJS test-reachability helper | old head `20cef095ae9bea4d920e5210effc4910ff635370`, joined by signed merge `9ce8e2a151742d3e7bdaed70000ca56d6172173d` |
| Post-pivot truth | Root guidance and current program documents reconciled to the pivot; migration parser handles qualified, multi-action `ALTER TABLE` | `f276d323b179473badf593bf858abb5dd5b94885`, `bd31825f46bcd4e00118e2b2f45938480b37bc2e` |
| Reasoning contract | Frozen 16-lens vocabulary, root manifests, evidence schema, templates, validator, and tests | `ed8424f5153b760162545e0ea669a4be5950280a`, `963302685bae2c28d7c6107dc2bb0b4f902392cd` |
| Test-graph decision | Proposed One Graph ADR records the Cargo-native direction without prematurely deleting Buck2 | `fcca3fab3fcf1a784e1794c6edf2c61bdfc6abb9` |
| Ignored planning packet | The exact approved Ultragoal/ralplan context, plan, and architect/critic handoff were deliberately force-added | `dff758724f7b428584a602cee7b2429dfbd7c5c1` |
| Domain attempt | Only the six-finding `BLOCKED` disposition was kept; no domain/DNS/credential implementation was admitted | `bdd24a16a7373522245c7ffcffb85fdedc0ec752` |
| Branch authorization | Eighteen fabricated-branch helpers were replaced with capability authorization; the final correction also removed the three path-wide gate exemptions and repaired concrete tenant/branch boundaries in HR exit reporting, registry site creation/master import, and reporting rollups. The changed pure tests and PostgreSQL regressions are CI-reachable. | initial implementation `8d7871a96f0fdcfd387d1e05260237d4eeec9b2b`, gate `9d4a136903c4d5d7978a046d0db9d2293a944ce5`; final signed C named by its direct-child authority ledger |
| Personal-data classification | Schema-derived closed-vocabulary column classification, exact-name baseline, database assertion, CI reachability, and payroll classifications. The final correction preserves the official instrument's one-/two-year unit, keeps schema introspection owner-only, removes the out-of-pivot compliance product route, aligns retention and completeness over every non-temporary `r/p/m/f` relation outside `pg_catalog`, returns schema-qualified identities, and adds `undeclared` to the five audited free-form JSONB omissions. Disposable PostgreSQL proves non-public, materialized-view, foreign-table, Rule-C, floor, and owner-only cases. | `fccdf5288f916c5180045ef19d4daaa395998fbf` through `851b999f49222a4d04282088f508f552469d756d`; final signed C named by its direct-child authority ledger |
| Gate integrity | Request-body and undeclared-import false-green repairs, plus CI preflight enforcement of `openapi_drift` | `1ec3e69d7ec3b8348561d2d699326f0949258521` |
| Documentation gates | Stale citations were reconciled and local-link checking learned to ignore inline-code examples while still checking adjacent real links | `d90f127654e0b4b0fe423304bbf5086e3cc24228` |
| CI/tooling ratchets | Exact executed-test named sets, Cargo/feature reachability, credential argv hardening, local-CI parity, and removal of dead post-pivot tooling. The complete ten-job surface now locks 95 run steps, 29 action steps, job/workflow envelopes, and unfiltered required-context triggers. The final repair admits registry/reporting inline tests to the generated Buck face, updates both app inline-test cardinality locks, runs fresh command doubles through load-bearing Bash wrappers so macOS provenance checks cannot turn an expected deploy failure into a harness timeout, derives the MJS reachability root from its own module URL, and exports/maps the dispatch/mobile shared fixtures. Three formerly dark PostgreSQL binaries are now in CI; the exact dark baseline shrank from 13 to 10. | reduced from `09f147a7`, `26491630`, and `eb60e2e4`; final execution lock `3d5f0b0649b21c8e647b2fa31ec40bd2aeeb8fec`; repair checkpoints `aba19c8db6fa66ee4a6e642f72471e00dc65483f` and `baadf03cf4692754fd0b964834e324d14e48f20e`; final signed C named by its direct-child authority ledger |
| Security proofs | Five required security contexts have exact parsed job/proof contracts; Trivy installs checksum-pinned before checkout, Cargo audit/deny use isolated direct binaries, and exception-policy regressions execute. Hosted execution caught the direct `cargo-audit` invocation missing its required `audit` subcommand; the final correction fixes and mutation-locks that exact command. | `d6cfedfd1c8230ca6e5ea43053d1ccc7dbcc2026`; final signed C named by its direct-child authority ledger |
| Program boundary | The generic instance-backed `company_conformance` fixture remains useful engine regression evidence but is explicitly not the Company/HR projected product target. Replacement conformance and projected Company/Person/Employment/PayRun work remain HOLD until owning-port and single-writer contracts exist. The current lane protocol and engineering playbook now forbid product writer worktrees and named JobPosition/projection pilots until a later authority accepts those gates; neither document can dispatch the provisional catalog. Its blocking CI context is relabeled as a generic-engine regression, and the pre-pivot readiness record is retired as current evidence. | final signed C named by its direct-child authority ledger |
| Continuation state | The machine-readable capability registry no longer points a new session at this disposable worktree or provisional lanes. Current worktree, branch, and lane assignments are null; every current capability state is `HOLD`; the exact prior state remains separately hash-bound as history. | final signed C named by its direct-child authority ledger |
| Operations boundary | Production/readiness prose is explicitly historical or unverified, and destructive backup/restore/PITR/CNPG drill entrypoints call a common authority guard that exits 78 before substantive action. This records a HOLD; it does not establish that a deployment or recovery path works. | final signed C named by its direct-child authority ledger |
| External PR authority | Unsigned fork and Dependabot heads fail the same protected-main C/T authentication instead of reporting a successful skip. | `e448f48ea840af2258d5bb374a9185e93b9d838e` |
| Release | Release-please 0.3.1 manifest/changelog plus an independently reviewed authority train | PR #552, C `5e297ca0ab2134e793bb40d5999908606ef1e050`, T `bab0ff594b1b66c85aaaa8c92ffcbedc88c0c2c2` |

## What was deliberately discarded

Discarded means “do not reconstruct it from reflogs or backups after the wipe.”

| Candidate | Disposition |
| --- | --- |
| Audit-boundary lane (`1f426033`) | Rejected in review. It changed the public policy-audit snapshot from `{user: UserSummary}` to an unversioned flat object and replaced the actual `team` ABAC value with `team_recorded`, preventing reconstruction of team-driven authorization. The remaining kernel API was opt-in, bypassable, and unused, so it was not kept as speculative infrastructure. |
| Ontology field-policy lane (`20ce7536`, integrated/reviewed as `090b927a314699a80d11ac655bfab4bdc19a71bf`) | Rejected by two independent high-risk reviews and signed-reverted in `3a3619a5a8c8c3871f5bbf3d7446ec3fb02e1033`. The granted command-role SQL entrypoint could bypass four-eyes and spoof/null the actor; migration 0212 activated previously untrusted attachments as `read_field`; and rollback to pre-enforcement application code would expose protected fields with no detach/repair or compatibility floor. A future implementation must atomically consume the exact grant in the definer, quarantine or reject historical rows, and design forward repair plus rollback-safe deployment. |
| Cosskorea domain code (`f3a0d9d8`; 13 paths in the final reviewed package, of which eight code/deploy paths remain dirty in the original root checkout) | Rejected after five correction rounds. Six sustained defects include session-takeover risk, unbound WebAuthn evidence, unsafe consent provenance, unusable ordinary-user recovery, missing served-certificate proof, and broken rollback cleanup. The authoritative disposition is [`docs/program/ledger/2026-08-03-cosskorea-domain-swap-blocked.md`](../program/ledger/2026-08-03-cosskorea-domain-swap-blocked.md). It explicitly revokes the superseded 2026-08-02 handoff's “ready to execute” instructions; the blocker ledger is the only durable artifact. |
| Generic work graph/handover slice (`7390720`) | Outside the hard post-pivot boundary and overlapping current authz/ontology objects; retaining it would create an unowned second design. |
| Workflow notifications, workflow approval, the broad reporting feature lane, broad audit, and other dirty feature lanes | Outside the active pivot and not necessary for a stable restart. This does not discard the narrow reporting tenant-scope repair preserved above. |
| KubeSpan worker ADR/work | Live topology was not authoritatively established and the work was not needed for this consolidation. The irreplaceable OCI node must not be used as an experiment. |
| Full Buck2 deletion | Deferred, not forgotten. Deleting it before Cargo reaches every current test would create invisible tests; ADR-0039 preserves the measured replacement direction. |
| Stash `546f0` | Obsolete employee-import fragments and removed React work; current main contains the fuller backend implementation and the frontend was intentionally deleted by the pivot. |
| Old authority/merge repair branches and historical worktrees | Superseded by reviewed PR contents or by main; branch topology itself carries no product value after squash merge. |

## Ultragoal and planning continuity

Most ignored `.omx` state was runtime churn: approximately 3.6 GB and 138,000
files of sessions, temporary worktrees, caches, logs, and superseded goals. `.omc`
contained no durable unique plan. Exactly three active-pivot planning inputs were
retained byte-for-byte, together with one tracked execution status record:

The selection was time- and state-audited rather than guessed: the ignored tree
held 294 plan files and 5,395 context files, while all 292 excluded plans and
5,394 excluded contexts predated the July 28 pivot; `.omc` project memory had
empty user directives/notes and its latest checkpoint had no tasks, modes, or
jobs. Pre-pivot age alone was not treated as proof that every byte was runtime
churn. A later reference-edge audit found ten reconstructible historical OMX
artifacts and two restricted workbook profiles still named by tracked files.
Their explicit dispositions follow; none is an unstated fresh-clone dependency.

| Artifact | SHA-256 |
| --- | --- |
| `.omx/context/reasoning-lens-contract-20260803T101035Z.md` | `dea80c0a1fa5cc47c7ba12e4ee1c62480cd675e7b26bf33434ddf1fc94be61e4` |
| `.omx/plans/reasoning-lens-contract.md` | `76dc05561d7d6c07ee26afb68ea60841321eab7516f6d0546ba96d41825aad5c` |
| `.omx/plans/reasoning-lens-contract-handoff.json` | `db3ca7563223e271c6cf481a5766cfe6b96d2c73dbcf8dbaa5fa12092c010fc2` |
| `.omx/plans/reasoning-lens-contract-execution-handoff.json` | `49333378c2c756107798609be42c3266b7ef1bfbbe526a38618a0d3c11a76ce2` at the custody-truth-correction resealing stage |

The original JSON correctly says execution had not started at the instant the
architect/critic plan was frozen. Do not edit that historical claim. The sibling
`reasoning-lens-contract-execution-handoff.json` is the machine-readable receipt
for subsequent implementation and verification. Its state and checksum change
until the consolidation closeout is committed; the first three hashes above do
not. A fresh session should continue from current program state and the latest
tracked receipt; it should not rerun the old Ultragoal as if implementation were
pending.

### Retired ignored OMX evidence

The following ten local artifacts supported broad pre-pivot frontend/platform
work. They are stale or reconstructible, outside the July 28 active boundary,
and not suitable as current product evidence. Their tracked reference edges were
replaced with this receipt. The bytes are explicitly retired and may be discarded;
their hashes are retained so a later copy cannot silently regain authority.

| Retired local artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `.omx/context/backlog-ledger/issues-6-19-55-56-20260629T0906Z.json` | 135,932 | `7e9103febcad3b81a1bfae10d0cc3600ea27d1c10f8d62516c6b4b01d3c74f52` |
| `.omx/context/backlog-ledger/prs-61-86-gh-20260629T090935Z.json` | 16,821 | `20ae2420d8939c343bde953be691eb7a9fd2eed48561c7580582a929bd1ccabe` |
| `.omx/context/platform-maturity-g001/issues-detail-20260630T0012Z.json` | 170,861 | `2a0cd280b4315182c0c1c1e3291f112ffa81dd675e5fe15485225aebc705646b` |
| `.omx/context/platform-maturity-g001/issues-summary-20260630T0012Z.json` | 4,035 | `2b002abec663f001d8da6d4400b7b5719de22cc590c1ef7da61d0f61fb0886a9` |
| `.omx/context/platform-maturity-g001/live-baseline-20260630T0017Z.txt` | 5,685 | `fe80a0f6b71f4c0ddffea95462bdc0d6ea57710dfdd7ae315599dca80890a56c` |
| `.omx/context/platform-maturity-g001/red-route-audit-gate-20260630T0014Z.txt` | 37,576 | `6dc040afa7b23079e04b5d228e46e961db6fc75194ab6eafe6ff70a27462b7ac` |
| `.omx/context/platform-maturity-g001/green-route-audit-gate-20260630T0016Z.txt` | 42,628 | `fde937f84659a955a05d50ec0363d4389814075c32762fcbafc30f629f801d47` |
| `.omx/context/public-cx/g009-completion-evidence-20260629.md` | 2,947 | `2bc739746cc397e7dbc749e34c3aaeb67d3532e293b6602594e0b43d1c0baa83` |
| `.omx/plans/platform-maturity-e2e-completion-prd-20260629T215449Z.md` | 54,777 | `986f9c57933f06d08560812b5d31b64883d33391cbde6c401b131a4a4a5cac46` |
| `.omx/plans/platform-maturity-e2e-completion-test-spec-20260629T215449Z.md` | 11,607 | `a5e3ec0ca48bc149c1ce74449b2fc14c9cbf538258f0782e141a0690eada1226` |

### Restricted workbook custody

Two ignored profiles are derived from the real eight-sheet HR/payroll workbook
named below. Pattern scanning found masked sensitive previews and no
high-confidence private key, token, RRN, phone, or email value, but that is not a
proof that non-sensitive cells are publishable or fully de-identified. Do not
commit these profiles. Preserve all three inputs together in approved encrypted
off-device custody and verify a read-back hash, or explicitly approve discarding
the source and both profiles.

| Restricted local input | Bytes | SHA-256 / observation |
| --- | ---: | --- |
| `.omx/context/workbook-profile-untitled-spreadsheet.md` | 53,955 | `a11423948582bc5050ad9358b1d7f7d82784c4ad2fb05c6f86683ef6f318eabb` |
| `.omx/context/workbook-profile-untitled-spreadsheet.json` | 220,509 | `3fa610fd382c0428f92cc7fe0c556a481854ed6684de4a2814162c3c4652a281` |
| `~/Downloads/Untitled spreadsheet.xlsx` | 910,174 | Bytes were not readable to the audit process; no hash was claimed. |

## Ignored files and secrets

Git cannot safely preserve live secrets. Before the disk is erased, copy the
values of `DISCORD_BOT_TOKEN` and `DISCORD_WEBHOOK_URL` from the ignored root
`.env` into an approved external secret manager, then rotate them if feasible.
Channel identifiers and the `CLAWHIP_*`/`GAJAE_*` entries are workstation-local
configuration; preserve them externally only if those tools are still needed.
Never commit `.env` or paste its values into an issue, PR, handoff, or log.

The repository is not sufficient to reconstruct workstation identity or
infrastructure access. Preserve the exact configured SSH commit-signing private
key (`~/.ssh/id_ed25519`) as described in the manual blocker above; merely
issuing a new key is not compatible with the pinned verifier. Separately escrow
or deliberately reissue OCI SSH/API access (`~/.ssh/mnt_oci`, `~/.oci/config`,
and `~/.oci/mnt_api_key.pem`), GitHub CLI/keyring authentication, and any needed
`~/.kube/config` or `~/.talos/config`. Store them only in an approved encrypted
secret manager or off-device backup.

The entire `~/.config/talos-mnt/**` tree is in the custody boundary: it was
approximately 189 MB across 76 files and included cluster configs, generated
machine configuration, SSH/API material, a JWT private key, an OTP, database and
object-store material, OCIDs, logs, and 18 helper scripts. Secret contents were
not read into this handoff. Do not commit the directory or selectively assume
the three hashes below preserve it. Three scripts referenced by the historical
runbook were present at audit time:

| External file | SHA-256 |
| --- | --- |
| `~/.config/talos-mnt/talos-up.sh` | `08c80322e7e65e86359e0aedf5549d76c46259f8516dd0252f678b2f86b78aa8` |
| `~/.config/talos-mnt/reserve-relaunch.sh` | `a080c689c6b263359b4af03f3c0e1718f15abf1078c18f90d93dd34bc5ea2938` |
| `~/.config/talos-mnt/deploy.sh` | `9c21c735d4d7d850fee48d2019dcfdf9be8dec5926e32c5096d35d7e8715e112` |

Preserve that directory as one encrypted external bundle if OCI recovery remains
desired, then read the archive back and record its archive hash outside this
repository. A matching per-file hash proves custody of only those bytes, not that
the procedure is current or safe to run. The kubeconfig observed during this consolidation
points at the shared laptop cluster, not proven OCI production; preserving it
does not change that authority boundary. After restore, verify `gh auth status`
and a throwaway signed commit's displayed signer before relying on either.

Four tracked historical tools/profile records point to business inputs outside
Git. They are outside the active pivot and are not CI/package dependencies, so
they must not be imported into this PR. They still require an explicit owner
decision before wipe—encrypted external preservation or deliberate discard:

| External input | Audit observation |
| --- | --- |
| iCloud `TalkFile_장비Master List.xlsx` | 90,753 bytes; SHA-256 `caf83ec76dbdf35e85096a46ffb33372d22adeacc37c49c102a213a10875eba2` |
| `~/Desktop/COSS Group/` | Directory exists; contents were not readable to the audit process |
| `~/Downloads/Untitled spreadsheet.xlsx` | 910,174-byte file; bytes were not readable to the audit process; preserve or discard it together with the two profiles in [restricted workbook custody](#restricted-workbook-custody) |
| iCloud `TalkFile_조직도_그룹웨어_회사_20260626105223.csv` | 7,424 bytes; SHA-256 `d5f4dcf2b856e12015b2af62a59725c1f0a030e5ec4bcba5cfa785e136ad6687` |

An iCloud path alone is not proof that upload/synchronization completed, and
Desktop/Downloads paths are local unless separately backed up. Record the
owner's decision and, for preserved inputs, verify an off-device read-back by
hash before erase.

### Whole-disk blockers outside Console

The wipe boundary is larger than this repository. A bounded read-only audit
found no usable custody destination: Time Machine had no configured destination
or latest backup, `~/Library/CloudStorage` was empty, and `/Volumes` exposed only
system or read-only application mounts. Console merge authority does not
authorize modifying the repositories below, but their local-only state makes a
whole-disk erase unsafe:

- `/Users/jasonlee/Developer/oyatie` is at
  `c52bdb09ea337de103b05317de0c120f2b7a3e45` on
  `preserve/hermes-w1-dirty-20260630` with its upstream at 0/0, but has 13 staged
  files, 1,386 unstaged files, 374 untracked files, two stashes, 151 commits not
  reachable from fetched remote-tracking refs, and 175 registered/existing
  worktrees. A Git bundle alone would omit the dirty and untracked bytes.
- `/Users/jasonlee/Developer/TencentDB-Agent-Memory` is at
  `f3df79326dfd763f45199c441e2129d780467949` on `feat/server_team` with its
  upstream at 0/0, but 28 material subscription-gateway files under `deploy/`
  and `reports/` are untracked and not remote.
- `/Users/jasonlee/Developer/asterinas` has 17 commits not reachable from
  remote-tracking refs. Worktree `agent-a0954f296eb65a23d` has six unpublished
  commits at `ac0324984484b377dc0621213058e4ae2558c56d`, five modified source files,
  and six untracked DTB/DTS test fixtures. Four authored root `.omc/artifacts`
  also remain local: `arm64-container-verification-recipe.md`
  (`6e2cf029bd251903eefec0ecf88ac98087dfdba29b5497b007a976e017cfd80e`),
  `aarch64-local-host-capability.md`
  (`778af950b36b3bcd7c80111fc3840ec1ffccdb2b1bccd5e9b6db3431e17fef5a`),
  `pr-3270-security-review.md`
  (`b3c1f5e2f4ff21e934223794bd96800bb1dfa9c9c25913df3ed00f8c4646837b`),
  and `aarch64-maturity-parity-matrix.md`
  (`6be28a697631dc9ee5409d81f330c5713a4379bf70fe918a146e0ced667c2903`).

Preserve each repository's refs plus dirty/untracked bytes to encrypted writable
off-device storage and independently read them back, or make an explicit
repository-specific discard decision. Do not infer that an upstream at 0/0
contains the working tree.

The ignored `ops/.dev-secrets/jwt-private.pem` and `jwt-public.pem` are local
development keys generated by the dev bootstrap and should be regenerated, not
backed up. Also discard `node_modules`, Rust/Buck build outputs, `.tmp`,
`.local-dev`, Python caches, Tofu provider locks/state generated in scratch
areas, and the remaining OMX/OMC runtime directories except for the two
restricted workbook profiles above until their custody decision is complete.
The durable root
`.gitignore` now ignores newly generated `.omx/` runtime content; the four
already tracked planning/status files remain tracked. Also discard
`.superpowers` progress and temporary review packets under `/tmp`: they contain
interim Coss approvals superseded by the final `BLOCKED` record and are a
specific resurrection hazard.

The repository is self-contained after the merge for active-pivot product
source, reviewed history, and active planning authority. It is intentionally not
sufficient for workstation identity, secret recovery, the restricted workbook
evidence, or an OCI/Talos rebuild: those require the external custody items
above. The ten retired historical artifacts cannot regain authority merely
because a later machine happens to recover their bytes.

## Safety holds that survive the wipe

- Never destroy, terminate, resize, or reprovision the grandfathered OCI Ampere
  A1 instance (4 OCPU / 24 GB). Re-creation permanently loses capacity.
- The laptop's `kubectl` context is not proof of the OCI production cluster and
  is shared with another repository. Confirm context and authority before any
  cluster observation; mutation still requires explicit authorization.
- Korea's six compliance controls remain `HOLD` until qualified Korean
  legal/compliance authority validates them. GitHub material is not a legal
  source, and `LAW_OC`/raw API responses must never be committed.
- No live DNS, TLS, production, credential-reset, WebAuthn, release exposure,
  payment, or compliance-claim operation is authorized by this handoff.

## Next product work

Continue from the sequence in `docs/program/README.md`: prove remaining Cargo/CI
equivalence and advance the bounded ontology/company/HR/payroll program. Recheck
the pivot before accepting any resurrected branch. For every proposed recovery,
ask whether main already contains the outcome, whether it is inside the current
scope, whether its source of truth is singular, and whether its exact tests are
reachable from CI. Do not use the existing `company_conformance` fixture to
dispatch projected Company/HR work; its current HOLD is recorded in
`docs/program/CATALOG.md`. Default to leaving rejected branches dead.
