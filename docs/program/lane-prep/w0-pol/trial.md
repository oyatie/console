# Trial receipt — D-POL / console-93w

| Field | Value |
|-------|-------|
| Status | **GREEN** |
| Recorded | `2026-08-05T11:56:58Z` |
| Worktree | `chore/w0-trial-receipts` @ `origin/main` |
| Package | `console-policy-domain` |
| Allowlist | `backend/crates/policy/**` |

## Command

```bash
SQLX_OFFLINE=true cargo test --locked --manifest-path backend/Cargo.toml -p console-policy-domain --lib -- --test-threads=1
```

## Result

PASS 2/2 including draft_statuses_are_never_runtime_enforced (fail-closed draft)

## Fanout note

Run in parallel with sibling Wave 0 trials (shared `CARGO_TARGET_DIR` contention possible; wall ~7s for all three).

## Next admission step

First fail-closed policy increment; dual security review on any authz surface.

## Non-claims

- Does **not** clear PRODUCT HOLDs
- Does **not** implement domain increment product code
- Does **not** claim ROADMAP item complete

### Application package (parallel wave 2)

`cargo test -p console-policy-application --lib` — **PASS** (1 tests listed)
