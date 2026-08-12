# Handoff — shared laptop NativeLink CAS + Buck2-everywhere for `jason931225/console`

**From:** oyatie laptop CAS lab agent (`~/oyatie-cas/`) + console lane  
**To:** console orchestrator  
**Date:** 2026-08-11  
**Goal:** `console` shares `instance_name=main` CAS with `oyatie`, and **Cargo ceases to be the primary build/test/CI driver** — Buck2 everywhere.

---

## Founder decisions (binding)

1. Shared CAS substrate for console + oyatie (multi-repo, digest-keyed).
2. **Prefer Buck2 remote/CAS** over cargo-centric caching.
3. **Stronger:** switch **cargo everywhere to Buck2** (workflows, scripts, gates, domain-unit, PG harness, local paths). Cargo remains lock/reindeer input only until later admissions.

---

## Substrate (lab)

| Item | Value |
|------|--------|
| Lab root | `~/oyatie-cas/` (never commit keys) |
| Instance | `main` (shared) |
| Writer | `cw.oyatie.dev` / local `127.0.0.1:50051` (mTLS writer) |
| Reader | `cr.oyatie.dev` / local `127.0.0.1:50052` (mTLS reader, AC read_only) |
| Tunnel origins | `50151`/`50152` HTTPS for cloudflared; GHA uses **Access TCP** sidecar |
| Caps | CAS 500 GB / AC 50 GB slow tier |
| REAPI canary | `~/oyatie-cas/canary/reapi-access-canary.sh` → lab verdict **GREEN_REAPI** |
| Secrets bootstrap | `~/oyatie-cas/gha/bootstrap-cas-secrets.sh` |

Auth: CF Access Service Token (belt) + NativeLink client certs (suspenders). Forks get nothing.

---

## Cargo → Buck2 inventory (executable surfaces)

### CI (`.github/workflows/ci.yml`) — primary blast radius

| Job / area | Today | Buck2 target / path |
|------------|--------|---------------------|
| `preflight` | `cargo metadata --locked` | Keep temporarily (lock proof); later reindeer-only check |
| `domain-unit` | Many `cargo test --lib` / file suites | Per-crate `//backend/...:*-unit` (Wave D) |
| `postgres-reachability-*` | `tools/ci/cargo_needs_postgres.sh` + map | Map already has `buck_inner` (215 entries) — Wave E |
| `backend` fmt/clippy | `cargo fmt` / `cargo clippy` | Buck/rustfmt story TBD; do not block gate flip |
| `backend` gates (11×) | `cargo run -p console-gate-*` | `tools/buck2 run //backend/ci/gates/<name>:console-gate-<name>` (Wave C) |
| `backend` mutation / authz / app unit / openapi | **Already Buck2** | Expand; attach CAS overlay on trusted |
| `security.yml` | `cargo-audit` / `cargo-deny` | Keep as supply-chain tools (not build driver) until Buck-native policy exists |

### Tools / scripts

- `tools/ci/cargo_needs_postgres.sh`, `postgres-cargo-map.json`, `check-postgres-cargo-map.mjs`
- `tools/lanes/pgtest.sh`, `fanout.py` (cargo lock commentary)
- `scripts/check-ci-preflight.mjs` + `.test.mjs` (**hard-codes cargo proof strings** — must move with each wave)
- `.cursor/hooks/cargo-scope-enforcer.sh`
- `backend/bacon.toml`

### Docs / decisions

- ADR-0039 + **DN-0005** cargo-primary under CAS absence → superseded in planning by **DN-0006**
- Program ledger / ecosystem drafts still say “no `[buck2_re_client]`”

### Intentionally not “Cargo hits CAS”

Only Buck2 REAPI clients use this store. Demoting cargo jobs is the migration; do not invent a cargo→CAS bridge.

---

## Console repo wiring (Wave A — landed on this lane)

Opt-in overlays (not root `.buckconfig`):

- `infra/ci/buckconfig/warm-cache-lab-{rw,ro}.buckconfig` — local `:50051`/`:50052`
- `infra/ci/buckconfig/warm-cache-gha-{rw,ro}.buckconfig` — Access TCP `:55051`/`:55052`

Helpers:

- `scripts/cas/materialize-buckconfig-local.sh` → mode-0600 `.buckconfig.local` (gitignored)
- `scripts/cas/start-access-tcp.sh` → delegates to lab REAPI canary

Example local warm build:

```bash
scripts/cas/materialize-buckconfig-local.sh --role writer
tools/buck2 --config-file infra/ci/buckconfig/warm-cache-lab-rw.buckconfig \
  build //backend/ci/gates/layer-boundary:console-gate-layer-boundary
```

---

## Acceptance checks

| # | Check | Status 2026-08-11 |
|---|--------|-------------------|
| 1 | Local mTLS reader/writer `openssl s_client` `:50052`/`:50051` | **OK** (Verify return code 0) |
| 2 | Access TCP + REAPI (lab canary) | **GREEN_REAPI** (lab); host `cloudflared access tcp` alone was flaky — prefer dockerized canary |
| 3 | Cold Buck2 **without** overlay | **OK** (`toolchains//:rust` BUILD SUCCEEDED in lane worktree) |
| 4 | Trusted GHA upload + second-job read | **BLOCKED** — console repo secrets not installed yet |
| 5 | Fork PR has no CAS secrets | **Policy** — enforce when workflows land |

---

## Blockers for orchestrator

1. **Install console GitHub secrets** (founder):  
   `~/oyatie-cas/gha/bootstrap-cas-secrets.sh console`  
   (CF_ACCESS_{READ,WRITE}_* + OYA_CAS_TLS_*). BAN fork PR exposure.
2. **Serialize CI writer** for Waves C–E (`.github/workflows/ci.yml` + `scripts/check-ci-preflight.mjs` + postgres map/harness) — one lane at a time.
3. **Hermeticity** — `system_cxx_toolchain` limits cross-OS/arch AC hits; CAS blob sharing still correct.
4. **Warm license** — keep fail-closed; no silent always-warm on Required CI.
5. **DN-0005 / ADR-0039 text** — DN-0006 records the reversal; ADR acceptance/update is a separate authority act.

---

## Next PRs (ordered)

1. **This lane (Wave A):** overlays + scripts + DN-0006 + this handoff — merge when reviewed.
2. **Wave B:** secrets bootstrap + composite action `Start CAS Access TCP` (trusted only) + optional non-required canary workflow.
3. **Wave C:** backend gates → `buck2 run` (smallest CI flip; Buck targets exist).
4. **Wave D:** domain-unit → Buck + CAS on trusted.
5. **Wave E:** postgres harness → `buck_inner` driver; retire cargo argv as primary.
6. **Waves F–G:** rust-cache demotion, docs/hooks/bacon, leftover cargo.

---

## Contacts / SSOT

- Lab README: `~/oyatie-cas/README.md`
- DN-0006: `docs/decisions/notes/DN-0006-buck2-primary-shared-cas.md`
- Do not spawn lasting satellite architecture plans outside founder SSOT.
