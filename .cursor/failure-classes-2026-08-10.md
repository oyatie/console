# Cursor session failure classes (2026-08-09/10)

Measured on the #618/#619/#620 land attempt. Promote into hooks/rules after ≥1 repeat (oyatie: after 2; we promote now because each class already burned a full round).

| ID | What happened | Root cause | Control (now) |
|----|---------------|------------|---------------|
| `transport.mm-role-default` | First PR train used `mm-role` → Sol BLOCK vs Opus APPROVE, zero merges | Treated Grok CLI transport as process | Hook deny + BASE_LOCK; native Task critics only |
| `process.known-blockers-skipped` | Push, then fix whatever the latest critic said | No mandatory inventory of threads+prior findings before first push | BASE_LOCK + implementer: inventory → one commit |
| `process.bot-thread-treadmill` | Every push attracted new Codex P1/P2; each became a new fix round | Unproven bot opinions treated as merge bars | Standing lenses: after one fix pass, only `blocker` or `major+provenByExecution` reopen the lane |
| `process.tip-land-without-restack` | #620 merge left #619 CONFLICTING; agent stopped (force-with-lease gated) | Serial tip PRs without restack in the brief | Hook allows `git rebase origin/main` + `--force-with-lease`; briefs must name restack |
| `process.hook-failclosed-unready` | `failClosed` + non-executable hooks locked all shells (exit 126) | Shipped hard gate before smoke | Hooks invoke via `bash …`; no failClosed until probe-ratchet.py green |
| `process.stop-nag-foreign-dirt` | Stop hook demanded receipt because #618 migrations were dirty | Subject ≠ owned root | stop-receipt-gate only watches `.cursor/**` + `scripts/cursor/**` |
| `ops.gh-auth-stale` | Merge agents dispatched while `gh` token invalid / rate-limited | No preflight on forge credentials | `scripts/cursor/preflight-forge.sh` before any `gh pr merge` / thread resolve fan-out |
| `process.critic-tiebreak-missing` | Sol proven BLOCK vs Opus APPROVE → paralysis | No tie-break rule | provenByExecution wins; APPROVE cannot stand over unfixed proven majors |
| `process.subagent-quota-death` | Rebase agent died mid-flight on usage limit | No parent fallback | Parent owns merge; on subagent error, parent continues or re-dispatches once — do not wait idle |
| `process.lane-setup-hook-deadlock` | mbl lane: brief mandates `git worktree add`, git-lock hook denies it; env escape unreachable from subagents; enforcement also inconsistent (`-C` form slipped past, bare form denied) | Hook denylist written for sprawl never got a lane-provisioning allowlist; regex missed `git -C … worktree` | Hook allowlists `git [-C <hub>] worktree add <hub>/.worktrees/<name> -b lane/*\|admission/* origin/main` (+ `scripts/cursor/provision-lane-worktree.sh`); sibling `../console-lane-*` denied |
| `process.worktree-outside-workspace` | Agents editing `../console-lane-*` / admit siblings constantly hit Cursor "allow edit" / External-File Protection while hub `/…/console` is open | Git worktrees created as sibling directories outside the opened workspace root | New lanes only under `<hub>/.worktrees/` (gitignored); BASE_LOCK + ritual + hook path constraint; open hub folder (multi-root `.code-workspace` unreliable) |
| `process.squash-title-not-conventional` | Release 0.3.4 changelog (#621) lists only #618; #619/#620 invisible because squash titles weren't conventional-commit format; #622 as opened ("Wave 1: …") would repeat this | PR titles composed for humans, not for release-please's conventional-commit parser | Train PR titles MUST be `type(scope): …` before squash-merge (put it on the coordinator's brief); #622 retitled `fix(p4): …` 2026-08-10 |
| `process.release-candidate-unsigned` | #621 preflight: `verify-console-authority-train.mjs` rejects release-please bot candidate (not SSH-signed by trusted key); every release PR structurally un-mergeable | Authority train gate assumes human/agent-signed candidates; bot commits can never carry the trusted SSH signature | Bead `9ry` (P1): pick ONE mechanism — narrow verifier rule for bot+docs-only, re-sign flow, or signed-candidate release strategy |
| `process.receipt-location-drift` | ann/cm3/soe receipts landed in hub `.cursor/receipts/` while critics look in the lane; lanes branched from main also cannot self-validate (validator untracked) | Briefs said "receipt" without naming the lane worktree; validator not on origin/main yet | Ritual + BASE_LOCK: receipt MUST be `<lane-worktree>/.cursor/receipts/<id>.json`; validate via hub absolute path until `chore-cursor-ratchet` lands |
| `process.doc-vs-hook-drift` | BASE_LOCK still forbade all `worktree add` after the hook allowlisted lane provisioning; also still named banned `composer-2.5-fast` | Hook fixed under friction protocol; rules not updated in the same turn | Same turn as any hook change: update BASE_LOCK / agents / ritual; cost policy = never `*-fast` |
| `process.admit-tip-unpublished` | Admit worktree rebuilt to `f53067b95` while PR #622 remote tip stayed `9c7ea19f6` — green local, dead remote | Coordinator treated "commit locally" as landed; no publish check | `scripts/cursor/check-admit-sync.sh` on admission/*; orchestrator must push or re-dispatch within one turn of a clean local tip |
| `process.lane-unsigned-product` | q06 lane tip unsigned (gpg unavailable); critic BLOCKed; disposed only because admission re-signed | Signing assumed ambient | Implementer: commit -S smoke before product; elevate on fail — do not write product unsigned |
| `process.ci-fmt-masks-gates` | #622 Backend job failed at `cargo fmt` first → clippy/tests/writer-ownership gate never ran at tip; fmt-only green would false-clear lexer holes | Job short-circuits on first step | Admit/local: fmt + gate pins before push; QA must note when Required green ≠ gate executed |
| `process.openapi-enum-peripheral-drift` | cm3: response schemas gained `time_change_consult` but list filter enums + "all four" copy stayed stale → critic BLOCK | Peripherals updated incompletely | Critic peripherals lens + implementer OpenAPI inventory when enums change |
| `process.git-pkill-lock-race` | Subagents "unstick" hung `git`/`gh` with `pkill -f 'git -C …/console'`, `killall -9 git`, `pgrep … \| kill -9`, or `rm …/index.lock` while sibling lanes still hold the hub lock → stampede, force-with-lease blocks, `bd` reads hang | Shared `.git` + no ownership check on kill/unlock; `CURSOR_ALLOW_GIT_DANGEROUS` was used as a bypass | Hook denies git-targeted pkill/killall/pgrep-kill and unlock-by-rm **before** `CURSOR_ALLOW_GIT_DANGEROUS`; wait/timeout; stale locks only via `scripts/cursor/safe-stale-git-lock.sh` (PID dead + age); ops: never pkill git across worktrees |

## Anti-pattern that looked like "not catching issues"

Catching issues across **multiple rounds** often meant:
1. Round 1 critic found class A (good),
2. Implementer fixed only A, not open review threads B/C already visible,
3. Round 2 bot filed D (new opinion) + restated B,
4. Process treated D as mandatory → treadmill.

So the defect is usually **incomplete inventory + wrong convergence rule**, not "the critic missed something."
