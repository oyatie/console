# Trial receipt — D-APR / console-7vm

| Field | Value |
|-------|-------|
| Status | **GREEN** |
| Recorded | `2026-08-05T11:56:58Z` |
| Worktree | `chore/w0-trial-receipts` @ `origin/main` |
| Package | `console-governance-domain` |
| Allowlist | `backend/crates/governance/**` |

## Command

```bash
SQLX_OFFLINE=true cargo test --locked --manifest-path backend/Cargo.toml -p console-governance-domain --lib -- --test-threads=1
```

## Result

PASS 7/7 including missing_required_gate_fails_closed, authority_deny_is_hard_stop

## Fanout note

Run in parallel with sibling Wave 0 trials (shared `CARGO_TARGET_DIR` contention possible; wall ~7s for all three).

## Next admission step

First preflight non-mutation / distinct-human increment; keep REST allowlist tight.

## Non-claims

- Does **not** clear PRODUCT HOLDs
- Does **not** implement domain increment product code
- Does **not** claim ROADMAP item complete

### Application package (parallel wave 2)

`cargo test -p console-governance-application --lib` — **PASS** (0 tests listed)
