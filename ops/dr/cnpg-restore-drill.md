# CNPG Restore Drill (production Barman → OCI path)

> **POST-PIVOT UNVERIFIED / HOLD:** This drill is retained historical/operator
> reference. It does not prove that the named cluster, backups, credentials, or
> recovery path exist, and it does not authorize access to or mutation of any
> cluster. The repository currently authorizes zero production mutations. Start
> with the
> [disk-wipe consolidation handoff](../../docs/handoffs/2026-08-03-disk-wipe-consolidation.md).

## Scope

This drill exercises the **production** disaster-recovery path: a CloudNativePG
`Cluster` recovered from the `console-backups` Barman Cloud `ObjectStore` in OCI
Object Storage. It is the Kubernetes/CNPG analogue of the Compose-stack drills in
`ops/backup/restore-drill.sh` and `ops/dr/pitr-drill.sh` — those test the local
Docker Compose Postgres + SeaweedFS stack, **not** the CNPG → Barman → OCI path
that production actually runs.

Run this against the live cluster (single-node Talos on OCI Ampere A1) declared
in `deploy/apps/console/base/database.yaml`.

| Production object | Value | Source of truth |
|---|---|---|
| Live `Cluster` | `console-db` (namespace `maintenance`) | `base/database.yaml` |
| Database / owner | `maintenance` / `console_app` | `base/database.yaml` bootstrap.initdb |
| `ObjectStore` | `console-backups` (namespace `maintenance`) | `base/database.yaml` |
| Barman plugin | `barman-cloud.cloudnative-pg.io` | Barman Cloud Plugin 0.13 |
| OCI creds secret | `oci-objectstore-creds` (keys `ACCESS_KEY_ID`, `ACCESS_SECRET_KEY`) | `deploy/SECRETS.md` |
| `serverName` for recovery | `console-db` (the source cluster's name = its folder in the bucket) | CNPG recovery API |

CNPG/plugin versions this drill targets: **CloudNativePG 1.29**, **Barman Cloud
Plugin 0.13**. The recovery uses the plugin-based `externalClusters` +
`bootstrap.recovery.source` schema, **not** the deprecated in-tree
`spec.bootstrap.recovery.backup` / inline `barmanObjectStore`.

## What the drill proves

1. The base backups and archived WAL in `s3://mnt-db-backups/console-db/` are
   readable with the production OCI credentials.
2. CNPG can bootstrap a brand-new `Cluster` from them (`bootstrap.recovery`).
3. The recovered database promotes and regular-table row multisets and column
   metadata match an independently retained reference for that database/system
   identifier and requested target. Actual replay position, failover safety and
   full schema/security equivalence are separate checks.
4. None of this touches the live `console-db` cluster — the recovery runs in a
   throwaway namespace with its own PVCs and is deleted at the end.

## Safety model

- A scratch namespace (`console-dr-<timestamp>`) is created, used, and deleted.
- The recovery `Cluster` is **read-only** against the object store: it does
  **not** declare `.spec.plugins`, so it never archives WAL and cannot write to,
  rotate, or expire the production backups. It only declares the
  `externalClusters[].plugin` recovery source.
- The OCI credentials secret and a recovery-scoped `ObjectStore` are copied into
  the scratch namespace (both kinds are namespaced; the recovery Cluster reads
  them from its own namespace).
- The live `console-db` Cluster, its PVCs, and the `console-backups` ObjectStore in
  `maintenance` are never modified.

## Cadence

Run after any change to `base/database.yaml`, the Barman plugin version, or the
OCI bucket/credentials; before first production data entry; and at least monthly
after launch. Record evidence under `ops/dr/drill-logs/`.

## Automated drill

```sh
ops/dr/cnpg-restore-drill.sh --expected-manifest /protected/recovery-reference.json \
  2>&1 | tee "ops/dr/drill-logs/$(date -u +%Y%m%dT%H%M%SZ)-cnpg-restore-drill.log"
```

The expected manifest is required. Before a drill, retain a qualified snapshot
independently of the restored target. On an authorized reference connection,
use the fixed SQL generator and capture the result with a failing pipeline:

```sh
set -euo pipefail
umask 077
python3 ops/dr/recovery-manifest.py sql |
  psql -XqAt -v ON_ERROR_STOP=1 --dbname="$REFERENCE_DATABASE" |
  python3 ops/dr/recovery-manifest.py capture --target-time "$TARGET_TIME" \
    > /protected/recovery-reference.json
```

Supply credentials through protected libpq configuration, not the connection
argument. The reference must represent the intended recovery point: coordinate
writes or use a separately qualified snapshot and retain its provenance. For
PITR, include known before/after-target changes in that qualification. Capturing
the restored target as its own reference is invalid. The script freezes the
reference before provisioning and records its digest. A latest-WAL target must
also have a known expected state; ongoing source writes cannot be compared to
an unrelated old reference.

SQL hashes each row, sorts the hashes with `C` collation, and Python streams the
multiset digest. Duplicate rows remain significant. Raw values are not logged.
SQL uses one read-only repeatable-read transaction with a 300-second transaction
limit, 120-second statement limit, 16 MiB work memory and 1 GiB temporary-file
limit; exceeding those limits fails the drill rather than relaxing comparison.
Foreign tables/partitions are refused before traversal. This verifies ordinary
tables, partition-parent content and column names/types/nullability. It does
not certify sequences, constraints, indexes, triggers, RLS, roles, routines,
partition topology, large objects, materialized views or non-PostgreSQL stores.
Those need their own recovery acceptance before exposure.

Useful flags (see `--help`):

- `--target-time "YYYY-MM-DD HH:MM:SS+00"` — PITR recovery target. Omit to
  recover to the latest archived WAL.
- `--namespace NAME` — override the generated scratch namespace.
- `--keep-scratch` — skip teardown for incident debugging.
- `--timeout-seconds N` — how long to wait for the recovery Cluster to become
  healthy (default 1200).

Required success markers in the log:

- `cnpg_recovery_cluster=healthy`
- `verify_recovered_state=match manifest_sha256=... tables=...`
- `verify_pitr_state=match` (only when `--target-time` is supplied; expected data state, not a WAL-position assertion)
- `cnpg_restore_drill_complete=ok`
- `scratch_teardown=complete namespace=<scratch-namespace>`

## Manual procedure (if the script cannot run)

1. **Create a scratch namespace** with the restricted Pod Security labels CNPG
   expects:

   ```sh
   ns="console-dr-$(date -u +%Y%m%dT%H%M%SZ)"
   kubectl create namespace "$ns"
   kubectl label namespace "$ns" \
     pod-security.kubernetes.io/enforce=restricted \
     pod-security.kubernetes.io/enforce-version=latest
   ```

2. **Copy the OCI credentials** into the scratch namespace (the recovery Cluster
   reads the secret from its own namespace):

   ```sh
   kubectl get secret oci-objectstore-creds -n console -o yaml \
     | sed "s/namespace: maintenance/namespace: ${ns}/" \
     | kubectl apply -n "$ns" -f -
   ```

   (The piped object still carries `resourceVersion`/`uid`; strip them or use
   `kubectl create secret ... --from-literal` if your cluster rejects the apply.)

3. **Create a recovery `ObjectStore`** in the scratch namespace pointing at the
   same bucket. `serverName` in the ObjectStore must stay empty (it is a
   compatibility-only field in plugin 0.13); the source server name is set on
   the recovery Cluster's `externalClusters` entry.

   ```yaml
   apiVersion: barmancloud.cnpg.io/v1
   kind: ObjectStore
   metadata:
     name: console-backups-recovery
     namespace: <scratch-namespace>
   spec:
     configuration:
       destinationPath: s3://mnt-db-backups/
       endpointURL: https://axdotp9iv3ua.compat.objectstorage.ap-chuncheon-1.oraclecloud.com
       s3Credentials:
         accessKeyId:    { name: oci-objectstore-creds, key: ACCESS_KEY_ID }
         secretAccessKey: { name: oci-objectstore-creds, key: ACCESS_SECRET_KEY }
       wal:    { compression: gzip, maxParallel: 1 }
       data:   { compression: gzip }
   ```

4. **Apply the recovery `Cluster`.** It references the recovery ObjectStore as a
   read-only `externalClusters` source and bootstraps from it. For PITR add the
   `recoveryTarget`; omit it to recover to the latest WAL.

   ```yaml
   apiVersion: postgresql.cnpg.io/v1
   kind: Cluster
   metadata:
     name: console-db-recovery
     namespace: <scratch-namespace>
   spec:
     instances: 1
     imageName: ghcr.io/cloudnative-pg/postgresql:18.4
     storage:
       size: 5Gi
     bootstrap:
       recovery:
         source: console-db-origin
         # PITR (optional). Format: 'YYYY-MM-DD HH:MM:SS+00'.
         recoveryTarget:
           targetTime: "<target-time>"
     externalClusters:
       - name: console-db-origin
         plugin:
           name: barman-cloud.cloudnative-pg.io
           parameters:
             barmanObjectName: console-backups-recovery
             serverName: console-db   # the live cluster's folder in the bucket
   ```

   Note: there is **no** `.spec.plugins` block — that would turn on WAL archiving
   and let the recovery cluster write to the production bucket. Leave it out so
   the drill is strictly read-only.

5. **Wait for recovery to finish and the cluster to be ready:**

   ```sh
   kubectl wait --for=condition=Ready cluster/console-db-recovery -n "$ns" --timeout=20m
   kubectl exec -n "$ns" console-db-recovery-1 -c postgres -- \
     psql -U postgres -d console -tAc 'SELECT pg_is_in_recovery();'   # expect: f
   ```

6. **Inspect schema statistics** against the recovered database (diagnostic only;
   estimated counts cannot satisfy the required manifest comparison above):

   ```sh
   kubectl exec -n "$ns" console-db-recovery-1 -c postgres -- \
     psql -U postgres -d console -tAc \
     "SELECT format('%I.%I', schemaname, relname) AS t, n_live_tup
        FROM pg_stat_user_tables ORDER BY 1;"
   ```

   For a PITR drill, confirm the target boundary: a row known to be committed
   **before** the target time is present, and one committed **after** is absent.

7. **Tear down:**

   ```sh
   kubectl delete namespace "$ns" --wait=true
   ```

   Deleting the namespace removes the recovery Cluster, its PVCs, the copied
   secret, and the recovery ObjectStore. The live `console-db` cluster and the
   `console-backups` ObjectStore in `maintenance` are untouched.

## Failure modes

- **Recovery job cannot list the object store**: the copied
  `oci-objectstore-creds` secret is wrong/absent in the scratch namespace, or
  `serverName` does not match the live cluster name (`console-db`). Check the
  recovery job and plugin sidecar logs:
  `kubectl logs -n "$ns" job/console-db-recovery-1-full-recovery` and the
  `plugin-barman-cloud` container.
- **Cluster never leaves recovery**: the requested `recoveryTarget.targetTime`
  is earlier than the oldest base backup, or WAL is missing for the window.
  Pick a target inside an archived window.
- **PodSecurity admission rejects the pods**: the scratch namespace is missing
  the `restricted` enforce label — re-apply step 1.
- **PVCs left behind after teardown**: namespace deletion was interrupted;
  `kubectl delete pvc -n "$ns" --all` then delete the namespace again.
