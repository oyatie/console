# Console — full handoff, 2026-08-02

> **Superseded for restart purposes on 2026-08-03.** Preserve this as historical
> evidence, but begin a fresh session with
> [`2026-08-03-disk-wipe-consolidation.md`](2026-08-03-disk-wipe-consolidation.md).
> Its branch, PR, worktree, and backlog status replaces the time-sensitive state
> below; the infrastructure and compliance cautions here remain historical input.

**Audience:** an agent starting fresh on this repository with no prior context.
**Base:** `origin/main` = `f2646890b`.
**Shelf life:** the state in §2 and the backlog in §5 go stale within days. The constraints in §1
and the lessons in §6–§7 do not.

Read §1 and §2 before touching anything. Everything else is reference.

---

## 1. Non-negotiable constraints

These are owner decisions and program rules. Violating one is worse than not doing the task.

### 1.1 Infrastructure

**The OCI Ampere A1 must never be destroyed, terminated, resized, or reprovisioned.**
Owner, verbatim:

> "you are not allowed to decommission the instance. as it is valuable. 4 vCPU and 24GB RAM is no
> longer available and if we decommission it will be 2vCPU 12GB RAM free as OCI updated their terms."

OCI cut the Always Free A1 allotment. This instance is grandfathered at 4 OCPU / 24 GB; a rebuild
returns 2 vCPU / 12 GB. Destroy-and-recreate is a one-way 50% capacity loss, not a migration step.
Talos and Kubernetes upgrade **in place** (`talosctl upgrade`, `talosctl upgrade-k8s`).

Identity, so you recognise it under any of its names:

| | |
|---|---|
| OCI instance | `control`, compartment `prod`, `VM.Standard.A1.Flex` 4 OCPU / 24 GB — **verified live 2026-08-02** via `oci compute instance list` |
| Kubernetes node | **UNVERIFIED.** `deploy/OPS-RUNBOOK.md:42` says `console-fsm-node`; `deploy/opentofu/…/compute/main.tf:100` says `mnt-fsm-node`. The two have never been reconciled against the live cluster — see §5.6 |
| Reserved public IP | `140.245.68.253` (private `10.0.0.227`) |
| OS | Talos v1.13.4, kernel `6.18.34-talos (arm64)`, containerd 2.2.4, k8s v1.36.1 |

> #### ⚠️ `kubectl` here does **not** talk to the OCI cluster
>
> The only configured context is `admin@oya-talos` — the **laptop** QEMU cluster (3 nodes,
> `oya-talos-controlplane-1` / `-worker-1` / `-worker-2`, Talos v1.13.7, `10.5.0.0/24`), which is
> **shared with another repo and must not be mutated**. `kubectl get nodes` succeeds and shows a
> healthy cluster that is not production. The OCI kubeconfig lives in OCI Vault as
> `mnt-talos-kubeconfig`; fetch it deliberately, and check `kubectl config current-context` before
> believing any cluster observation.

**The trap:** `oci compute instance get` reports `Oracle-Linux-9.7-aarch64` and always will. OCI
reports the image an instance was *launched* from, forever, and this boot volume was `dd`-flashed
with Talos afterwards. `image-id` is not the running OS. Check the system, not the metadata: Talos
ships **no sshd**, so `22/ssh closed` + `50000/apid` + `50001/trustd` open is already conclusive.
This mistake was made in-session and reported to the owner as "the runbook is wrong." It was not.

**The Talos cluster on this laptop (`oya-talos`, 5 QEMU VMs) is shared with another repo.**
Read-only inspection only. No cluster mutation without asking.

### 1.2 Korea compliance

**Korea's six controls are `HOLD`. Only a qualified Korea legal/compliance authority may unhold
them.** The governing rule, verbatim: *"Missing, stale, conflicting, or unqualified authority is
HOLD; agents may not invent certainty."* `professionally_validated` stays `false`.

`allowed_sources` are exactly: official legislation portal, official regulator, court or authorized
interpretation, qualified legal opinion. **A GitHub repository is not an allowed source.**
`jclab-joseph/it-legal` may tell you *where to look*; the citation itself must be fetched from
국가법령정보센터 with a pinned `efYd` and quoted.

`LAW_OC` is **never committed**. The API echoes it into every `법령상세링크`, so raw API responses
must never be committed either. It lives in `~/.env`, not the repo `.env`.

고시 and other 행정규칙 are `--admrul`, **not** `target=law`. A null result means you queried the
wrong target, not that the instrument does not exist — this error has twice been written up as a
"not established" finding. Fetch article text; never recite it from memory.

`docs/release/PR-473-PRODUCTION-CARDINALITY.evidence.json` is a **blank template**. Never write
`verified: true` over it.

### 1.3 Git and process

- **`git stash` and `git reset --hard` are banned.** Never `git add -A`.
- **Never build in the main checkout** (`/Users/jasonlee/Developer/console`). Use a worktree.
- One writer per lane. Two lanes editing the same file is how a day gets lost.
- **Never click GitHub's "Update branch."** See §4.3 — it silently breaks the merge gate and the
  error message blames your signing key.
- Do not weaken a gate, widen production exposure, or fabricate a compliance claim. If a gate is
  wrong, say so and prove it; do not route around it.

---

## 2. Where things stand

### 2.1 Landed today (2026-08-02)

Seven PRs merged: **#556, #557, #554, #555, #558, #559, #560**. Together they ended the merge-conflict
problem that had been costing roughly half of all merges — see §3.

### 2.2 Open

| PR | What | State |
|---|---|---|
| **#561** | `fix(opentofu)` — guard the irreplaceable A1 | Open, `authenticate-console-authority` green, rest of CI running |
| #552 | release-please 0.3.1 | Open, BEHIND, routine |

**#561 in one line:** `talos_image_ocid` defaulted to `""` and the node's `count` was
`var.talos_image_ocid != "" ? 1 : 0`, so a missing tfvars produced `Plan: 9 to add` with **zero**
mentions of the cluster's only instance — an appliable plan that silently omitted it. The default is
removed (absent input now errors), `prevent_destroy = true` is on the node, and the second A1
(`oci_core_instance.flasher`) is deleted. Both guards were proven by execution, including importing
the node into a throwaway local state to watch `prevent_destroy` refuse, then deleting the state.
Nothing was applied.

### 2.3 Worktrees in play

`git worktree list` shows many. The ones with unlanded content:

- `console-lanes/lane-1` … `lane-5` — parallel implementation lanes.
- `.claude/worktrees/wf_a4436a32-1c4-4` — **payroll**, migration `0210` (landed as #558).
- `.claude/worktrees/wf_41b8be20-737-4` — **PII classification**, migration `0211`, 5 commits,
  verified, was blocked behind #558 which has now landed. **This is ready to push.**
- `fix/cosskorea-domain-swap` — **on GitHub**, the preserved domain expand (§5.1).

**The main checkout is not clean, and that is where the domain work came from.**
`/Users/jasonlee/Developer/console` is on branch `docs/ecosystem-plan-session` with **8 modified,
uncommitted files** — the domain lane's expand, now also preserved on
`fix/cosskorea-domain-swap`. Leave that working tree alone; see §5.1.

### 2.4 Infrastructure state

- **Domain:** the owner has chosen **cosskorea.com**. The zone already has 3 **proxied** A-records
  and `ssl: strict`; `always_use_https` was set on and `min_tls_version` raised to 1.2 this session.
- **knllogistic.com** is to be taken down as a *website* — "we can bring it up later if needed."
  **Its DNS must not be deleted**; see §5.1, the mail plane still depends on it.
- `CF_ZONE_DNS_API_TOKEN` in `~/.env` was verified by executing cert-manager's actual DNS-01
  sequence (create then delete an `_acme-challenge` TXT). It covers both zones.
- **No app workload is deployed.** The frontend was deleted in the 2026-07-28 pivot. `/healthz` is
  nginx answering. There is currently **no way to learn that the cluster died.**

---

## 3. How this repo merges: the authority train

You cannot land anything without understanding this. Read the scripts, not this summary, before you
rely on details: `scripts/console/verify-console-authority-train.mjs`,
`verify-console-pr-authority-bootstrap.mjs`, `authority-ledger-path.mjs`.

**Every PR is a signed two-commit train `C → T`:**

- **C** — your actual change. Signed.
- **T** — the *authority tip*: a direct single-parent child of C, signed, whose diff against C
  touches **only** authority documents. In practice today that means **one added `.md` file under
  `docs/program/ledger/`**.
- **M** — the synthetic merge GitHub computes. Must have T as parent 2 and a tree identical to T.

Build T **last**, after C is final. Rewriting C invalidates T.

**The gate is self-validating.** `console-authority-bootstrap.yml` runs on `pull_request_target` and
checks out `ref: main`, so your PR is judged by **main's** scripts, not your branch's. Any change to
the gate itself therefore needs expand/contract sequencing across two PRs.

**Prove it locally before pushing:**

```sh
node scratchpad/simulate-main-gate.mjs <gate-worktree> <T-sha> [base-sha]
```

It imports main's real verifier and calls its exported pure seams. It reproduces everything except
the `refs/pull/N/merge` fetch. A green run here has matched CI every time it was used.

**What made this tractable.** Until today, three shared files —
`docs/program/console-capability-registry.json`, `console-jurisdiction-register.json`,
`console-program-ledger.md` — appeared in 59, 56 and 56 of the last 120 merges. Every PR rewrote the
same bytes, so 48% of merges conflicted. The cause was a **denormalised candidate SHA**: 30 lines in
one register and 8 in the other, all the same value, all derivable from git parentage. #556/#557
deleted the field and the three rebind scripts; the ledger became a **directory** of one file per
entry. Result, measured: when #554 merged, #555 came back `MERGEABLE/BEHIND` instead of
`CONFLICTING`, and its rebase was one clean `git rebase origin/main`.

The general rule, now a standing one: **a registry every change must append to should be a directory
of files, not a file.** Ask what outside information a shared field carries. If the answer is
"none, it is derivable," it is a conflict generator.

---

## 4. Failure modes this repo keeps hitting

Each of these cost real time. They are listed because they recur, not because they are interesting.

### 4.1 A count is not a set — three instances

A baseline pinned as a **number** passes when one member is swapped for another.

- The personal-data baseline pinned the *net count* of unclassified columns per table, so a migration
  that added one unclassified column and classified another moved nothing. Six rounds of hardening
  the SQL parser could not fix what pinning **the set of column names** fixed in one.
- `check-executed-tests.mjs` ratchets on `dark.length > baseline`, so darkening one test binary while
  wiring another nets zero — and its baseline file already **named** all eleven entries while the
  script read only the integer. The names were decoration.

**If a baseline can be written as a set, pin the set.** Report both directions (entered / left) and
require the file to be edited in the same change when a member leaves.

### 4.2 Ask what the measurement's denominator excludes

The same gate derived a crate's lib test binary from `#[cfg(test)]` appearing in `src/lib.rs`
*itself* — but nine crates keep it only in `src/<module>.rs`. Those binaries were never in
`defined`, could never be `dark`, and were deletable in silence. **A gate reports on its denominator.
Ask what is outside it before trusting the number.** Treat "gate did not run", "gate ran nothing" and
"gate passed" as three different states.

### 4.3 Never click "Update branch"

GitHub's *Update branch* button (and *Sync fork*, and merge-queue update-by-merge) creates:

```
Merge branch 'main' into <your-branch>
parents = <your signed tip>  <main tip>       committer: GitHub <noreply@github.com>   %G? = N
```

and makes it the PR head. That breaks the train **twice**: unsigned, and two parents. The error says:

```
console authority train: authority tip T signature is not valid:
git verify-commit rejected the exact trusted SSH signature
```

which reads like a key problem. Your key is fine; the head is not yours. **The only lawful fix is
rebase + force-push + re-prove:**

```sh
git rebase origin/main
node scratchpad/simulate-main-gate.mjs <gate-worktree> <T> origin/main
git push --force-with-lease=<branch>:$(git rev-parse origin/<branch>) origin <newT>:<branch>
```

Two traps in that last line, both paid for: **compute** the lease SHA with `git rev-parse` rather
than typing it from earlier output, and **never pipe a push to `tail`** — the pipe swallows the exit
code, so a `||` fallback never fires and a failure reads as success.

### 4.4 Docker was never down

`docker info` exited 1, two lanes concluded no container runtime was available, and one of them took
over `console_buck_admin` — a `rolsuper` on the shared local cluster — resetting its password to a
literal that now lives in a transcript. `docker context ls` showed the answer immediately: the
selected context was a stopped Docker Desktop while **colima was running the whole time**.
`DOCKER_CONTEXT=colima tools/lanes/pgtest.sh …` worked unmodified.

**Before working around a missing capability — especially with anything that mutates shared state —
spend one command establishing that it is really missing.** The workaround is always more invasive
than the check.

### 4.5 A test that embeds its inputs at compile time goes green without running them

`#[sqlx::test(migrations = "./migrations")]` bakes the migration set in at **compile** time, so
adding a `.sql` does not dirty the test crate. A planted migration that should have failed returned
exit 0 in **0.14s**. The give-away is the duration, not the verdict. `touch` the test file before
every run, and **report durations** so a cached green is visible as one.

### 4.6 Prose asserting unimplemented intent

Four instances found in one day. `deploy/talos/oci-guest/controlplane.patch.yaml:34-35` still claims
*"keep only KubeSpan-less local discovery"* while **no `cluster.discovery` block exists anywhere in
the repo** — Talos defaults apply and the node registers with public `discovery.talos.dev`.

Docs here go stale in one direction: they describe holds already changed and problems already fixed.
**Cite executable code, not prose about it.** And note the inverse is the same defect: prose
describing implementation that has been *removed*.

### 4.7 Migrations are numbered at merge, not at authoring

A contiguity gate governs migration numbers. Reserving ranges for parallel lanes **breaks** it.
Payroll and PII were both authored as `0210`; the fix was to renumber PII to `0211` at merge time and
prove it: PII alone → `NonContiguousMigrationVersion`; the union → PASSED. **Grep for the gate that
already governs a thing before inventing coordination process around it.**

### 4.8 Source-text gates fail open

Two lanes, same failure: a grep-shaped gate catches one spelling in one position. The last PII escape
used the repo's own house idiom from 25 existing migrations plus one comma. **Use a type boundary, or
make the build system refuse the edge.** Do not harden the parser again — that is an explicit
standing decision.

---

## 5. The backlog

Ordered by consequence within each group. Items marked **specified** can be executed as written.

### 5.1 Domain swap to cosskorea.com — **specified, ready to execute**

Owner decision: *"i want you to use cosskorea.com we own it"* and *"take down knllogistic then. we can
bring it up later if needed."*

> #### ⚠️ The lane's work is NOT lost — it is uncommitted in the main checkout, and at risk
>
> `origin/main` contains zero cosskorea references, but the **working tree of
> `/Users/jasonlee/Developer/console` does**. Eight files are modified and uncommitted on branch
> `docs/ecosystem-plan-session`:
>
> ```
> backend/crates/comms/mailbox/src/lib.rs        deploy/apps/console/base/middleware.yaml
> backend/crates/platform/auth-rest/src/lib.rs   deploy/apps/console/overlays/prod/kustomization.yaml
> deploy/apps/console/base/configmap.yaml        deploy/infra/cert-manager/cluster-issuer.yaml
> deploy/apps/console/base/ingress.yaml          scripts/deploy.sh
> ```
>
> This is the exact checkout the repo's own rule says never to work in, and it **has already been
> wiped once** this program by a `git checkout … -- .` run as a supposed no-op. Uncommitted content
> destroyed that way is unrecoverable.
>
> **It is already preserved on GitHub.** Branch **`fix/cosskorea-domain-swap`**, commit `f3a0d9d8e`,
> pushed as a copy — the working tree was left exactly as it was. **Do not merge that branch as-is:**
> it is the 7-rule expand, and the owner's decision makes it a 3-rule swap. Start from it, reduce it.
> Nothing on it is proven to compile; see the owed cargo commands below.

**What the lane actually built — and why it now needs reducing, not extending.** It is the
**expand**: 7 ingress rules, 3 cosskorea + 4 knllogistic, both domains served from one SAN cert. It
also did an **anchor-alias refactor** worth keeping — base placeholders became
`console.primary.example.com` / `primary.example.com` / `www.primary.example.com` and a matching
`secondary.*` set, instead of the old flat `console.example.com` / `apex.example.com`. That refactor
survives at 3 rules and should be kept; drop the `secondary.*` half.

The owner's later decision — *"take down knllogistic then"* — turns this from an expand into a
**swap**, which also **deletes the lane's own HIGH finding**. For the record, since it explains the
configmap comments you will read: with `CONSOLE_WEBAUTHN_RP_ID = cosskorea.com` while
`console.knllogistic.com` still served, the browser rejects `navigator.credentials.get()` with
`SecurityError` *before any network request*, while the page still returns 200 — invisible to every
smoke check. `extra_allowed_origins` is hardcoded `Vec::new()` at `backend/app/src/lib.rs:656` and
`backend/crates/platform/auth-rest/src/lib.rs:234`, and only relaxes the **server-side** check. With
knllogistic off the air there is no dead passkey path, so the finding disappears rather than
needing a fix.

On `origin/main` (the base you will branch from) `deploy/apps/console/base/ingress.yaml` has
**4 rules** with flat `example.com` placeholders, and
`deploy/apps/console/overlays/prod/kustomization.yaml` patches the real hosts in by index. The line
numbers in the table below are **`origin/main`'s**, not the working tree's.

**Target: 3 rules** — `console.cosskorea.com`, apex `cosskorea.com`, `www.cosskorea.com`. The legacy
`fsm.` host goes away entirely.

Exact edits:

| file | change |
|---|---|
| `deploy/apps/console/base/ingress.yaml` | drop the 4th rule (`fsm.example.com`, lines 102–132); `tls[0].hosts` → 3 entries; fix the comments at :11–14, :18, :47, :75 |
| `deploy/apps/console/overlays/prod/kustomization.yaml` | `:27` RP_ID → `cosskorea.com`; `:30` RP_ORIGIN → `https://console.cosskorea.com`; `:44` tls hosts → the 3; `:47/:50/:53` rule hosts → the 3; **delete** the `:54–56` rule-3 patch, the `:39–41` middleware annotation patch, and the whole `:57–64` Middleware patch |
| `deploy/apps/console/base/middleware.yaml` | **delete the file** — with knllogistic off the air there is no source host |
| `deploy/apps/console/base/kustomization.yaml:14` | remove `- middleware.yaml` |
| `deploy/apps/console/base/configmap.yaml:21,22` | base RP id/origin → cosskorea (and the comments at :16–18) |
| `deploy/OPS-RUNBOOK.md:449`, `ops/launch/multi-tenant-cutover-runbook.md:130-131`, `scripts/deploy.sh:76` | the post-deploy checks still curl **only** knllogistic hosts, so a cosskorea failure would pass verification unnoticed |
| `deploy/README.md:138`, `docs/PLATFORM-ROADMAP.md:21`, `deploy/infra/cert-manager/cluster-issuer.yaml:3,11`, `deploy/OPS-RUNBOOK.md:1,46` | prose that names the wrong zone |

**The middleware annotation and the Middleware object must be deleted together.** A Traefik router
annotated with `console-fsm-console-redirect@kubernetescrd` when that Middleware does not exist is a
broken router, not a no-op.

> #### ⚠️ Do **not** delete knllogistic's DNS — the mail plane still lives there
>
> `CONSOLE_EMAIL_FROM` is `no-reply@knllogistic.com` (`configmap.yaml:62`) and **cannot move** until
> an OCI email domain, DKIM selector, SPF and DMARC exist for cosskorea. A sending domain needs no
> website, but it does need its records. Taking the *web* down must not take the *mail* down.
>
> **Keep in Cloudflare:** the `knl1._domainkey` CNAME → `…dkim.yny1.oracleemaildelivery.com`, the SPF
> TXT (`v=spf1 include:rp.oracleemaildelivery.com ~all`), and the `_dmarc` TXT.
> **Safe to remove:** the four A-records (apex, `www`, `console`, `fsm`). They only point at the
> origin; removing them is what "off the air" means, and they are trivially restorable.
>
> Leave these alone in code: `configmap.yaml:53,62` · `mox.yaml:46,54,56` ·
> `scripts/check-production-hardening.test.mjs:141` · `scripts/mox-e2e.mjs:30` ·
> `deploy/OPS-RUNBOOK.md:235` · `deploy/SECRETS.md:82` ·
> `docs/runbooks/non-oci-talos-mail-imessage-relay.md:12`. The `backend/crates/comms/mailbox`
> occurrences are unit-test fixtures using an arbitrary domain — harmless either way.

**Say this out loud in the PR:** changing `CONSOLE_WEBAUTHN_RP_ID` invalidates **every passkey
already enrolled under `knllogistic.com`**. WebAuthn credentials are bound to the RP id and cannot be
migrated. Pre-launch this is acceptable; it must not arrive as a surprise. `docs/ios-swift-client-drift-handoff.md`
(:147, :149, :151, :231) documents an iOS `webcredentials:knllogistic.com` entitlement that will need
the same change.

**Out of scope, deliberately:** the `console.knllogistic.com/...` Kubernetes *annotation keys* in
`components/admission-audit/` and `components/imessage-relay/`. They use the domain as a namespace
prefix and are functionally inert. Renaming them is a separate debranding pass — the owner also asked
for `mnt-*` resources to be debranded, and these belong with that.

**Verification:**

1. `kubectl kustomize deploy/apps/console/overlays/prod` — assert exactly **3** rules, all cosskorea,
   and that `rules[].host` and `tls[0].hosts` are the **same set in both directions**. A host served
   but absent from the SAN is a cert error; a SAN with no rule is a 404.
2. `git grep -n knllogistic deploy/apps/` must return only mail-plane hits.
3. The four cargo commands an earlier lane **reported as not run** — unverified is not green:
   `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test -p console-comms-mailbox`, `cargo test -p console-platform-auth-rest`.
4. Mail, before and after: `CONSOLE_EMAIL_FROM` still renders `no-reply@knllogistic.com`; the DKIM
   CNAME, SPF TXT and `_dmarc` TXT still resolve — **check by DNS query, not by looking at the
   dashboard**. Then send one OTP end to end. This failure is silent until a user cannot sign in.

### 5.2 External origin probe + TLS expiry — **specified, highest value per unit of effort**

The repo names this gap in its own words twice: `docs/ENTERPRISE-READINESS.md:53` *"Gap: no monitoring
stack deployed; no alert routing/test-fired runbooks"*, and `docs/GO-LIVE-CHECKLIST.md:157-158` as an
unchecked box.

Use a **hosted** free-tier checker (healthchecks.io, Better Stack, UptimeRobot — any is adequate for
two checks; **this is an account decision the owner has not yet made**). It must be outside the
failure domain; that is the entire point of the component. This is the one place ADR-0024's
"self-host first" posture must be **explicitly waived in writing**, or someone will later "fix" the
inconsistency and silently break it.

**Probe the origin, not the hostname.** Every cosskorea A-record is Cloudflare-**proxied** (which is
why cert-manager uses DNS-01). A check against `console.cosskorea.com` terminates at Cloudflare's
edge and goes **green against a dead cluster**. Hit `140.245.68.253:443` with an SNI/Host override.
The idiom already exists at `deploy/apps/vip-ingress/README.md:152`:

```sh
curl -vk --resolve "$HOST:443:$VIP" "https://$HOST/healthz"
```

No firewall change is needed — 80/443 are already world-open in
`deploy/opentofu/contexts/oci-guest/primitives/network/main.tf`.

Second check: **certificate expiry.** With a DNS-01 issuer, a renewal that stops working is invisible
behind Cloudflare's edge cert until the origin cert dies.

Be precise about what this proves: **node-and-TLS liveness, not application health.** Deliverable is a
`deploy/OPS-RUNBOOK.md` entry recording the check URL, what it asserts, and who receives the pages.

### 5.3 Correct the false discovery comment — **specified, 2 minutes**

`deploy/talos/oci-guest/controlplane.patch.yaml:34-35` claims KubeSpan-less local discovery is
configured. It is not (§4.6). **Fix the comment now** to describe what the file actually does.

**Defer the config change.** `cluster.discovery.enabled: false` is correct and removes a live external
dependency, but applying any machine config to the single irreplaceable control plane is a non-zero
act for an aesthetic gain. Bundle it with the next planned `talosctl` apply — a Talos patch upgrade is
already available (client v1.13.7 against server v1.13.4). `talosctl validate --mode cloud` first.

Verification: `git grep -n "cluster.discovery" deploy/talos/` must still return **nothing**, proving
the comment now matches.

### 5.4 The oyatie handoff — **specified, findings only, no asks**

Write `docs/handoffs/2026-08-02-oyatie-infra-findings.md`. Console needs nothing from that team; this
is courtesy. Every claim carries the command that measured it, and every OCID is pasted from an API
response rather than transcribed.

- **Three dead resources:** `oya-cloud-kms-adapter-oci` and `oya-ops-workspace-shell` (both 0 images,
  0 layers, 0 bytes since 2026-05-16); `oyatie-audit-cold-backup` (0 objects).
- **Both E2.1.Micro instances carry `free-tier-retained=true`**; the A1 does not.
  `oyatie-watchdog-kr-01` additionally shows an Always Free idle-reclamation warning. Console is not
  asking to use them, so oyatie can decide freely.
- **`deploy/OPS-RUNBOOK.md` names a bastion OCID that does not exist.** Live is `oyatie-ops-bastion`
  (`…cmvxme2j6t4vomybcjuyum4kykalsx4oshtnqw2edoja`); the runbook has
  `…5kxkkxtbrrd5xnxx4yn4mlj4ir3rwicy6y4im7sqyhua`. Console will fix its own runbook.
- **`bitween-default-vault` in the `cloud` compartment holds console's `mnt-*` secrets** — including
  `mnt-talos-secrets` and `mnt-talos-kubeconfig`, the only recovery path for a node that cannot be
  rebuilt. `bitween` is the **tenancy name**, so this is simply the tenancy default vault. **Do not
  delete it on the strength of its name.**

An optional appendix: if oyatie wants the micros doing something, the two highest-value uses are an
**external watcher** (prober + dead-man's-switch + alert webhook + public status page — one Go binary,
~50 MB RSS) and an **off-cluster backup restore verifier**. Both must be *offered*, not requested.
See §5.7 for why the console-side versions of these were killed.

### 5.5 Ready to push

- **PII classification**, worktree `.claude/worktrees/wf_41b8be20-737-4`, 5 commits, verified,
  migration **0211**. It was blocked behind #558, which has landed. Push it.
  **Keep whatever happens:** `employees.raw_row` and `data_import_rows.raw_row` stay classified
  `unique-id/rrn, sensitive/health`. Open counsel question: are our data subjects 이용자, which
  decides whether 고시 제2026-9호 §7② or §7③ binds. That question is HOLD, not a blocker on the code.

### 5.6 Open findings with no lane

| # | Finding | Note |
|---|---|---|
| 4 | 전자결재 emits no notification on 상신 / 승인 / 반려 / 회수 | **Reopened** — a lane self-marked this complete while still running. A self-report is not a finding of fact. |
| 5 | `import_master_list` is a live authenticated route the owner says was demo-only | Remove it |
| 6 | 24 WIRE items from the audit triage | Includes `repository_filter`, the designed fix for the six cross-branch defects, with **zero callers** |
| 7 | `ip` / `user_agent` in the append-only audit log | Assess: PIPA processing record implications |
| 8 | `ResourceBranch` newtype | So a fabricated branch cannot compile. Six fabricated-branch helpers exist. **`authorize_org_wide` is not the fix — it would cause an outage.** A branch-less capability check is. |
| 9 | Duplicated finalize-policy rules in `workflow_studio.rs` | Retire |
| 11 | Restart the audit type boundary | Previous attempt went BLOCK → FIX_FIRST → BLOCK, ending with 99 compile errors across 8 crates it never opened. **Reset to the smallest thing a reviewer has already proven works, and name what to DELETE.** See §7.3. |
| 13 | Rotate `console_buck_admin` | `rolsuper`, password now in a transcript (§4.4) |
| 14 | `check-executed-tests.mjs` measures one population of three | 14 of 29 `.test.mjs` suites execute nowhere; 4 have an npm script defined and never invoked. Resolver already written: `scratchpad/mjs-test-reachability.mjs`. Extend the gate rather than wiring 13 by hand |
| 15 | `check:doc-citations` covers two files | `docs/specs/payroll.md` is checked by nobody. Trap: the verifier scores any `file.rs:123` citation UNVERIFIABLE against a budget of 0 |
| — | Buck2 safety-net round 3 | Bin-target tests invisible; `#[cfg(test)]` matched as one literal string |
| — | **No CI job runs `tofu validate` or `tofu fmt`** | Found while doing #561. Neither would have caught that bug, so adding one is arguably ceremony — but the gap is real |
| — | **The OCI node has two names in the repo and neither is verified** | `deploy/OPS-RUNBOOK.md:42` = `console-fsm-node`; `deploy/opentofu/…/compute/main.tf:100` = `mnt-fsm-node`. The live OCI *instance* display name is `control` (verified). Settle the Kubernetes name against the live cluster and make the two files agree. Cheap, and it is exactly the kind of contradiction that becomes a false finding later |
| — | `ci.yml` is the next conflict choke point | 34 of the last 120 merges. Same shape as the authority fix; §3 is the template. **Owner has not been asked yet** |

### 5.7 Blocked on the owner

- **A hosted uptime-checker account** for §5.2.
- **OCI email domain + DKIM/SPF/DMARC for cosskorea** — the precondition for moving
  `CONSOLE_EMAIL_FROM` off knllogistic.
- **cert-manager secret swap to `CF_ZONE_DNS_API_TOKEN`** — one `kubectl` command, user-side.
- **오픈뱅킹 (KFTC)** — `developers.kftc.or.kr`. A callback URL must be registered, and the
  account-storage shape decided, before payroll needs 급여이체. Task #16.

### 5.8 Explicitly decided against — do not re-propose

Each of these was researched and killed this session. The reasons are recorded so the work is not
repeated.

| Proposal | Why not |
|---|---|
| Self-host Talos discovery on a micro | A single-node cluster has nobody to discover. Adds a bootstrap dependency serving zero peers |
| Enable KubeSpan for remote workers | KubeSpan *requires* discovery, re-adding the dependency — and there are no eligible workers: the micros are 1 GB against Talos's 2 GB floor |
| Dead-man's-switch on `oyatie-watchdog-kr-01` | Same tenancy, same failure domain, and it is the box already carrying the reclamation flag. A monitor inside what it monitors is not a monitor |
| An ADR for "single control plane + KubeSpan workers" | Decides a hypothetical with no participants. The real content is a *constraint record*: the A1 is irreplaceable → in-place upgrades only → no HA on this substrate → **single control plane is forced, not chosen** |
| A second control plane on the laptop | etcd quorum is `(N/2)+1`: 1 CP tolerates 0 failures, **2 CPs also tolerate 0** while doubling what can fail. Three is the first number that buys anything, and it would put 2 of 3 members on a laptop that sleeps |
| Importing the A1 into OpenTofu | For an irreplaceable resource, IaC ownership is mostly downside — the payoff is recreate-from-code, the one forbidden operation. #561's README records the safe order if it is ever wanted |
| Self-hosted GitHub Actions runner on a micro | 1 OCPU / 1 GB is not a slower runner, it is one that never finishes. A single `rustc` codegen unit on a large crate exceeds 1 GB |
| Loki / Prometheus / Grafana on the micros | The repo's own design specifies them as k8s PVCs, and the micros cannot join the cluster. Also: there are no logs — no app workload runs. `oci-logging` is already an approved, phase-out-registered seam |
| Wiring `deploy/apps/observability/` into Argo | Annotated `dark-stage: "true"`; would OOM the one node that must also run the app |
| OCI ONS for alerting | An **untracked** vendor seam — `backend/ci/gates/vendor-lockin/` tracks only `oci-logging`, `oci-object-storage`, `oci-vault`, so ONS would create exactly the lock-in the doctrine forbids, *silently*. A plain HTTPS webhook is portable and free |
| Pre-assigning an ADR number | `docs/ideas/revision-log.md:243` records four judges each computing "next free" and **all four claiming 0027**. Numbers are assigned centrally |

**Buck2 verdict — remove it.** The justification for a second build system is remote execution and a
shared cache, and this repo has neither: without RE, warm state cannot cross a CI run boundary,
because the action cache lives in daemon memory rather than portably in `buck-out`. What is left is
registration cost — 6 registration points per new PostgreSQL test versus 1 under Cargo.

Measured on `f2646890b`, with the commands, because earlier figures circulated that did not survive
re-measurement:

| | measured | command |
|---|---|---|
| BUCK files | 177 (176 first-party) | `find . -name BUCK \| wc -l` |
| lines in `backend/**/BUCK` | 22,703 | `cat $(find backend -name BUCK) \| wc -l` |
| `rust_test` targets under `backend/` | 328 | `grep -rho 'rust_test(' --include=BUCK backend/ \| wc -l` |
| third-party `visibility = []` | **1,045 of 1,104** | `grep -rc 'visibility = \[\]' third-party/` |
| first-party `visibility = ["PUBLIC"]` | 678 | `grep -rho 'visibility = \["PUBLIC"\]' backend/ \| wc -l` |
| `within_view` uses anywhere | **0** | `grep -rn within_view --include=BUCK --include='*.bzl' .` |

Two nuances, so they are not misremembered in either direction. **Third-party visibility is real and
enforced** — 1,045 of 1,104 third-party targets are `visibility = []`, and that is a genuine
constraint operating today. **First-party is generated wide open**, 678 `PUBLIC` declarations, so no
first-party architectural edge is refused by anything. And `within_view` — the one primitive that
would fail at *load* time, and therefore the only one the required `buck2 uquery` preflight can even
see — has zero uses. `uquery` is unconfigured, so it is **blind to `visibility` and fatal to
`within_view`**: measured one variable at a time, a bad `visibility` gave exit 0 and a `within_view`
violation gave **exit 3** with the offending chain printed.

**The trap if anyone revives this:** `within_view = []` means **unconstrained**, not "nothing
allowed". The default is `["PUBLIC"]`, so an empty list reads as *unset* — the exact inverse of
`visibility = []`. A generator emitting an empty allowlist for the layer with no permitted
dependencies produces a constraint that looks maximally strict and enforces nothing. Assert non-empty.

---

## 6. Lessons from outside this repo

The owner asked twice whether these were being retained. They are recorded here so the answer stays
yes across sessions.

### 6.1 `blog.gaebal-gajae.dev` — evidence discipline

Distilled after one session in which nine lanes reported themselves complete and nine came back
FIX_FIRST or BLOCK.

**Evidence binds to a head. Restate the head in every claim.**
> "변경 뒤 head가 달라지면 이전 검증 결과는 새 상태를 설명하지 못합니다."

A lane proved main's gate accepted its commits; main then moved mid-verification, and every
`ACCEPTED` line described commits that no longer existed. Any report that a gate passed must carry
the SHA it ran against, and that SHA must still be `origin/main` at merge.

**A label is not a finding of fact.**
> "성공, 완료, 활성 같은 표시는 유용하지만 그 자체로 사실 판정은 아닙니다."

Never merge on an implementer's self-report; merge on a reviewer's verdict. `check-executed-tests`
read green while 18 executing tests were deleted — a green gate is a label pointing at something
nobody re-read. (This is why backlog item #4 was reopened.)

**Test the invocation shape, and assert the fast path did NOT run.**
> "원치 않는 빠른 경로가 실행되지 않았는지를 단언하세요."

A guard test that calls the store directly leaves the *handler* lines — the ones that decide whether
the HTTP surface is closed — compile-checked only. Two open doors were found this way by accident.

**A plausible cause is not proof.** Four diagnoses were refuted in a single session. Before a brief
asserts a mechanism, **grep for the gate that already governs it.**

### 6.2 `jclab-joseph/it-legal` — obligation-anchored drafting

The owner supplied this as a **writing standard for our own compliance and design documents**, drawn
from how that repository is drafted. It is not an endorsement of its content as a source (§1.2).

| step | what it does |
|---|---|
| **Anchor to the obligation** | every section starts from a cited statute, then derives the technical requirement |
| **Cite inline** | §21③ sits in the text, so a reader verifies without leaving |
| **Make it enumerable** | tables for sets, matrices for cross-products |
| **The schema IS the spec** | not "we need a retention table" — the actual DDL |
| **Pattern, not product** | "append-only log", never "use Postgres" |
| **Macro → micro** | overall structure, then field-level |

Why each bites here:

- **Anchor to the obligation** is why the payroll engine survived five citation rounds: every rate
  binds to the instrument that sets it, so a wrong number is traceable to a wrong *citation* rather
  than a wrong opinion. Documents that cite afterwards cannot be audited backwards.
- **Cite inline** is already the local habit (`file:line`, and `법령ID · MST · efYd · 조문` for
  statute). A citation in a footnote or a separate register is a citation nobody checks — the same
  disease as the denormalised candidate SHA.
- **Make it enumerable.** The Korea control work only became tractable once it was a cross-product
  (class × 시행일 × implementing control). Prose hides gaps; a matrix makes the empty cell visible.
- **The schema IS the spec** is the step most often failed here. Residual lists get written in prose
  where the DDL or the type would state the same thing *and be checkable*. If the artefact is a
  table, ship the `CREATE TABLE`. If it is a boundary, ship the type.
- **Pattern, not product.** ADR-0038 does this correctly. Naming PostgreSQL in a requirement makes
  the requirement untestable against the obligation, which never mentioned PostgreSQL.
- **Macro → micro** so a reader can stop early and still have something true.

**Applied rule:** a gap analysis is a **matrix**, one row per obligation, and the "what we do today"
column is a `file:line` or a DDL fragment — never an adjective.

### 6.3 `bun.com/blog/bun-in-rust` — a 535,496-line port in 11 days

The relevant part is *method*, not the rewrite. Claims below are theirs, with their numbers.

**Adversarial review with diff-only reviewers.** They split model instances into an implementer and
**two independent reviewers**, who received *only the diff* and were told to **assume the code is
wrong**. Bugs this caught before merge included an async `uv_close` leaving freed memory, negative
timestamps producing invalid nanoseconds, and eager evaluation in `unwrap_or` causing panics.

> This repo already implements the pattern — it is the `Prove` phase of the `slice` workflow: *"two
> adversarial reviewers — diff only, no author reasoning."* Treat that phase as load-bearing, not
> ceremony. The one local addition worth keeping: **the diff-only reviewer may overrule the brief.**

**Port mechanically; refactor later.** They deliberately produced "a rewrite that looks like we
transpiled our Zig code to Rust," deferring idiomatic cleanup until after shipping. This repo learned
the same lesson the expensive way: improving *while* porting is what turned one lane into 99 compile
errors across 8 crates it had never opened. **Behaviour is the oracle.**

**All-at-once beat incremental** *for them*, to avoid temporary bridging code. Note the precondition
that made it safe, because it is the transferable part, not the strategy.

**The oracle was a language-independent test suite.** "Bun's own test suite is written in TypeScript,
which means it doesn't depend on the runtime's programming language" — 60,624 tests, 1,386,826
`expect()` calls, and **0 tests skipped or deleted** during the port. That is the precondition. This
repo's equivalent question is uncomfortable and worth asking: we have a gate whose baseline is an
integer while 14 of 29 `.test.mjs` suites execute nowhere (§5.6 #14). **Zero deletions is only
meaningful if you can prove what ran.**

**Split for compile time, and pay for it honestly.** They split a monolith into ~100 crates
specifically to parallelise compilation — and it *introduced cyclical dependency problems requiring a
separate refactoring phase*. Directly relevant to the Cargo-workspace direction here: the win is
real, the cost is real, and it lands as a distinct piece of work rather than free.

**Rust traps they hit, all applicable here:**
- `debug_assert!` does not execute side effects in release builds.
- Release builds keep bounds checks (unlike Zig `ReleaseFast`) — a performance assumption to verify,
  not inherit.
- Slice casting panics on odd-length slices where the old code silently ignored trailing bytes.

**Fuzz what parses.** They added 24/7 coverage-guided fuzzing of every parser post-merge, ~100 billion
executions, ~15 PRs of findings. This repo has parsers in the gate layer — and §4.8 records six
consecutive escapes from one of them. A parser that must be *correct* to keep a control closed is a
fuzzing target, or better, should not be a parser at all.

**Cost, for calibration:** 6,502 commits, peak 64 instances across 4 worktrees, ~$165k at API
pricing, against an estimated 3 engineers × 1 year by hand. Parallelism at that scale is bounded by
**review capacity**, not generation.

---

## 7. Working agreements

### 7.1 Direction of the product

Narrow and deep, not thin and wide. Owner, verbatim: *"thin verticals cannot be production. having
narrow and deep is better than thin and wide"* and *"once we have mature verticals that can lay the
foundations of business organizations we can worry about a new vertical."* HR + payroll is the first
vertical; Korean market; pre-launch.

Every design answers **"how does SAP do this, and what can we learn from it?"** — cited, or
explicitly marked UNVERIFIED.

### 7.2 No unnecessary bureaucracy

Owner, verbatim: *"don't add or maintain unnecessary bureaucracy in our pipeline."* When ceremony
hurts, **delete it** — do not build tooling to manage it. The test for any shared artefact: *what
outside information does this carry?* If the answer is "none, it is derivable," it is a conflict
generator, not a control (§3).

Corollary, from the owner's own reframing of the linter question:

> "The hyperscaler pattern isn't 'a gate that reads crates and judges them.' At Google the directory
> is the declaration and visibility is the enforcement. **Placement is mechanical because the build
> system refuses the edge, not because a linter inspects names.**"

Before writing a gate that judges structure, ask whether the build system can be made to **refuse**
it. A refused edge cannot be reviewed away and needs no enumeration of the spellings it forbids —
which is the failure mode of every source-text gate here (§4.8).

### 7.3 Running a lane

- **Retries reuse the same smallest contract.** When a lane returns FIX_FIRST, the remediation brief
  fixes the named findings and adds **no scope**. Folding fresh discoveries into a remediation prompt
  is how a two-finding fix becomes an eight-finding one.
- **When a lane gets worse each round, delete diff — do not add findings.** Signals to watch: a lane
  asking for a constraint waiver ("it cannot be avoided" — it could), a package-scoped `cargo check`
  reported as a workspace check, and a new ratchet whose own test is dark.
- **Ask what can still be killed, not whether it is fixed.** Passing tests plus a longer diff widen
  the attack surface. One "fix" that changed `Result` to `Option` deleted the only 403 on an
  endpoint. State the before/after verdict per principal class; anything looser is a failure
  regardless of what it fixed.
- **Never self-approve in the same context.** Authoring and review are separate passes.

### 7.4 Tools that already exist — do not rebuild them

| tool | what it does |
|---|---|
| `scripts/console/simulate-main-gate.mjs` | Runs **main's** authority gate against a candidate tip locally — the single most useful tool here. Usage: `node scripts/console/simulate-main-gate.mjs <main-worktree> <T-sha> [base-sha]`. It *imports* main's real verifier rather than reimplementing it; the only step it cannot reproduce is the `refs/pull/N/merge` fetch. Sanity check: pointed at `origin/main`'s own tip it must **REFUSE** (that tip is an unsigned squash) |
| a clean `origin/main` worktree | Needed as the first argument above. `git worktree add /tmp/wt-gate origin/main` |
| *(a Cloudflare helper existed and was deliberately not committed)* | It embedded a personal email as a default. If you need one: the useful subcommand was a **DNS-01 permission probe** that creates then deletes an `_acme-challenge` TXT — the only check that actually proves `Zone:DNS:Edit`, as opposed to reading the token's stated scopes. Auth differs by credential type: a scoped token goes in `Authorization: Bearer`, the Global API Key in `X-Auth-Email` + `X-Auth-Key`. Tokens live in `~/.env` |
| `scripts/dev/mjs-test-reachability.mjs` | Resolves which `.test.mjs` suites are actually reachable from an npm script — the input to backlog #14 |
| `tools/lanes/pgtest.sh` | Disposable PostgreSQL for lane tests. Prefix with `DOCKER_CONTEXT=colima` (§4.4) |
| `scripts/console/lane-env.sh` | The **only** place `RUSTC_WRAPPER=sccache` is exported. Now also sets `CARGO_INCREMENTAL=0` — without it sccache saw *zero* compilations, not merely a low hit rate |

---

## 8. If you read only one page

1. Never destroy the OCI A1 (§1.1). Never unhold a Korea control (§1.2). Never `git stash`,
   `reset --hard`, or click "Update branch" (§1.3, §4.3).
2. Every PR is a signed `C → T` train with T touching only a new file under `docs/program/ledger/`.
   Build T last. Prove it with `simulate-main-gate.mjs` before pushing (§3).
3. Pin sets, not counts. Ask what your denominator excludes. Report durations. Verify a capability is
   missing before working around it (§4).
4. The next three pieces of work, all fully specified: the **cosskorea domain swap** (§5.1), the
   **external origin probe** (§5.2), and the **false discovery comment** (§5.3). Push the **PII
   branch** — it is verified and no longer blocked (§5.5).
5. Cite executable code, not prose about it. A label is not a finding of fact.
