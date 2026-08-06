# Continuous efficiency assessment — 2026-08-06

## Measured inefficiencies (this train)

| Inefficiency | Evidence | Cost | Process fix |
|--------------|----------|------|-------------|
| **Tip-serial bottleneck** | PRs #589–#592 all touch `documentation-manifest` + ledger tip | Only one can merge at a time; others BEHIND + full CI re-run | Batch pure-domain test increments **or** stack; never fan-out N tip writers |
| **Multi-PR CI wall tax** | ~4 open PRs × ~25–45m wall each | Hours of runner time for ~100 LOC of pure tests | Prefer **one PR per wave** for pure `#[test]` hardenings across disjoint crates when tip would serialize anyway |
| **Baseline file collision** | `executed-tests-baseline.json` shared by ONT/POL/APR | Restack thrash | Admit requires baseline update in same PR; when stacking, re-run `--update` once at tip |
| **Late admit gates** | rustfmt + baseline only after hosted red | 1–2 full CI cycles wasted per PR | domain-increment Admit (+ #591) |
| **Passive wait risk** | Dual-track workflows exist but multi-PR babysit still long | Agent idle if only watching | program-tick forces product/process leg; this assessment is the process track |

## Parallel tracks that *are* free (when tip queue non-empty)

| Track | Condition | Example |
|-------|-----------|---------|
| **Read-only** | Always | A-AUD #571, ROADMAP evidence inventory |
| **Process allowlist** | Disjoint from open tip PRs' `.grok` if none open | Admit/workflow edits — **if** no other process PR open |
| **Product crate pure tests** | Only if **not** adding ledger tip this PR (impossible today — authority requires tip) | Practically: **stack** onto one branch |
| **Local dual-clock** | Always | Implement next stack commit while prior PR CI runs |

## Improvement loop (operating rule)

Every `program-tick` / autonomous wake:

1. **Measure** open PRs: tip writers vs path-disjoint; CI wall still running; fails by failure-class.
2. **Act fleet**: merge CLEAN head of tip-serial queue only; restack others.
3. **Act product/process**: if waiting_ci, do **not** open another tip-writing PR unless batching into the same branch.
4. **Retro**: if a class repeated ≥2, process-upgrade (already cataloged).
5. **Record** one line in `.grok/memory/reflections/{date}.md` when a new inefficiency class appears.

## Decision for Wave 0 remaining pure tests

**Default:** one stack branch `feat/w0-domain-test-hardening` with ONT+POL+APR commits **or** merge train 589→590→591→592 with no *new* tip PRs until drain.

Opening a fifth tip-writing PR while four are open is **ops.tip-serial-contention**.
