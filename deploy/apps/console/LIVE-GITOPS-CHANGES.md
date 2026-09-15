# Live GitOps changes

> **POST-PIVOT UNVERIFIED / HOLD:** The topology and sync claims below are a
> historical change ledger, not proof that Argo CD currently reconciles this
> repository or that merging mutates a live environment. The repository
> currently authorizes zero production mutations. Start with the
> [disk-wipe consolidation handoff](../../../docs/handoffs/2026-08-03-disk-wipe-consolidation.md).

ArgoCD syncs `deploy/apps/console/overlays/prod` from `dev` with `targetRevision: dev`.
A change to any live input therefore takes effect **the instant it merges** — there is no
separate deploy step to catch it, and no environment between the merge and production.

`scripts/check-command-database-wiring.test.mjs` enforces two different things about those
paths. It refuses the DARK governed-command-database topology **by name**, and it refuses
any change at all that is not declared **here**. The second is the backstop for a topology
nobody has named yet.

## The rule

Changing any of these paths requires an entry below naming each changed path:

- `deploy/argocd/apps/console.yaml`
- `deploy/apps/console/base`
- `deploy/apps/console/overlays/prod`
- `deploy/apps/secrets-management/wiring`

The gate reads the **diff** of this file against `origin/dev`, not its contents. A path
named in an earlier entry does not buy silence for a later change — each change declares
itself. That is the whole cost: one entry, in the same commit as the change.

This exists because the check used to be an unconditional byte-identity assertion against
`origin/main`, which no branch carrying a change could ever satisfy. A 90-day retention
policy was withdrawn rather than landed for that reason on 2026-07-31. A control with no
exception route does not hold the line; it gets deleted by whoever needs the next change
badly enough.

---

## 2026-08-30 — retarget live Argo to `dev`

**Changed:** `deploy/argocd/apps/console.yaml`

Phase C: Application `targetRevision` moves from `main` to `dev` so GitOps tracks the
integration branch. Creating environment-named mirrors is not go-live. DNS/TLS/secrets
and Ampere A1 remain HOLD. Live Argo still follows whatever git ref it currently
syncs until this Application spec is present on that ref.

---

## 2026-07-31 — a finite backup retention window

**Changed:** `deploy/apps/console/base/database.yaml`

The `console-backups` ObjectStore declared no `retentionPolicy`, so barman-cloud never
pruned base backups or WALs and point-in-time recovery reached back to the first backup
forever. Now `35d`.

That window is the erasure horizon: for as long as the archive covers a person's lifetime
in the database, deleting their row does not make them unreconstructable. Korean law sets
no duration for it — the only backup provision in Korean privacy law, 개인정보의 안전성
확보조치 기준 제11조, requires a backup-and-recovery *plan* above a subject-count threshold
and states no period. The statutory retention floors (근로기준법 제42조, 국세기본법
제85조의3제2항) attach to records the live database holds, not to the archive. ADR-0037
carries the full table and the citations.

So the number comes from the operational question instead — how long corruption can go
undiscovered — and payroll's monthly cycle sets it. One cycle plus slack.

**Safe to set now, expensive later:** verified on 2026-07-31 that no CNPG cluster in the
tenancy declares a backup, the `barmancloud` ObjectStore CRD is not installed, and the
`console` namespace does not exist. There are no backups for this policy to prune. Setting
a retention policy after a production archive exists deletes history.

The number is the owner's and counsel's to change. ADR-0015 constrains recovery *speed*
(RPO ≤ 5 min, RTO ≤ 1 h) and says nothing about window *length*; neither is affected by
this change.

## 2026-09-14 — declare Account custody preparation hooks

**Status: source declaration only; POST-PIVOT UNVERIFIED / HOLD remains in force.** This entry records four changed GitOps inputs. It does not establish a live Argo CD reconciliation, approve a deployment or merge, certify Account readiness, or authorize production mutations.

**Changed:**

- `deploy/apps/console/base/account-finalize-job.yaml` adds the bounded `console-account-finalize` PreSync Job at wave **-10**. It invokes the dedicated Account custody operator image after migrations; operator failure prevents completion of this declared PreSync sequence. The job has a 180-second deadline and two retries, no service-account token, a non-root UID, dropped capabilities, a read-only root filesystem, and a bounded memory-backed temporary directory. Its separate operator password and CA are read-only secret mounts; a separate target ConfigMap supplies the expected operator, system identifier, database name/OID and TLS hostname. The operator wrapper requires verified TLS and checks the selected target and reviewed migration inventory before executing its atomic finalizer. These are manifest/wrapper properties, not proof that a particular cluster supplies valid prerequisites or completes finalization.
- `deploy/apps/console/base/kustomization.yaml` includes the migration-operator network policies and Account finalizer Job in the base resource set. It introduces no additional serving workload or grant by this declaration.
- `deploy/apps/console/base/migrate-job.yaml` assigns the existing migration PreSync Job wave **-20**, preserving its existing migration image, owner transport and deadline. The intended sequence is network policies **-30**, migrations **-20**, Account finalization **-10**, then ordinary workload synchronization.
- `deploy/apps/console/base/migration-networkpolicy.yaml` adds two PreSync policies at wave **-30**: DB ingress from the two named migration/finalizer pod labels, and their egress to same-namespace `console-db` pods on TCP 5432 plus selected cluster DNS pods on UDP/TCP 53. This file contains no public HTTPS allowance for those operators. Its policies remain after hook completion and are recreated before a subsequent hook execution. Effective isolation still depends on a policy-enforcing CNI and the union of all applicable policies.

**Unresolved before any exposure:** `console-account-custody:local-unpublished` is an explicitly unpublished image reference, and the current production overlay does not replace it with an admitted immutable custody image. This declaration does not resolve that HOLD. The namespace, existing healthy database, protected target inventory, operator secret, verified TLS material, and required DNS/network infrastructure must already be provisioned independently before PreSync; resources first created in the ordinary Sync phase cannot satisfy that prerequisite. Target binding is not authorization to repair unexpected custody state or broaden database privileges.

The checked-in policy union contains default-deny ingress and PostgreSQL allowances for app/worker and migration/finalizer pods, but no DB-peer or CNPG-controller ingress allowance; the production overlay adds none. These manifests therefore do not establish replication or controller connectivity for a multi-instance target. An independently provisioned policy and effective connectivity readback are required before these hooks run; the presence of an existing database is not evidence that those paths are preserved. Likewise, hook ordering and policy object creation do not prove dataplane enforcement or uninterrupted restriction while a `BeforeHookCreation` policy is replaced. No live network or Argo readback was performed for this declaration.

Existing LC07 evidence records source/orchestration and bounded local operator checks; it is not Kubernetes execution, live image promotion, complete Account/auth activation, or production authorization. The header HOLD and all existing release/production gates remain unchanged.

## 2026-09-15 — require an explicit payroll durability policy

**Status: source declaration only; POST-PIVOT UNVERIFIED / HOLD remains in force.**

**Changed:** `deploy/apps/console/base/backend.yaml` and `deploy/apps/console/base/worker.yaml` require the `CONSOLE_DATABASE_DURABILITY` key from `console-config`. Both serving roles pass that policy to their payroll owner. There is no implicit local-completion default. The checked-in ConfigMap and production overlay do not supply an admitted descriptor; these manifests therefore cannot establish a deployable production configuration.

Before exposure, the operator must supply a reviewed policy and its matching observer installation, peer/transport admission, and recovery evidence. The implemented private-network descriptor does not establish production TLS peer authentication or certify HA, failover, or backup recovery. This declaration changes no live resource, supplies no production credential, and authorizes no deployment or Argo reconciliation. Production, payment and legal HOLDs remain unchanged.
