# Trial receipt — D-ONT / console-g1n

| Field | Value |
|-------|-------|
| Status | **GREEN** |
| Recorded | `2026-08-05T11:56:58Z` |
| Worktree | `chore/w0-trial-receipts` @ `origin/main` |
| Package | `console-ontology-domain` |
| Allowlist | `backend/crates/ontology/**` |

## Command

```bash
SQLX_OFFLINE=true cargo test --locked --manifest-path backend/Cargo.toml -p console-ontology-domain --lib -- --test-threads=1
```

## Result

PASS 8/8 in ~7s (parallel fanout with POL/APR)

## Fanout note

Run in parallel with sibling Wave 0 trials (shared `CARGO_TARGET_DIR` contention possible; wall ~7s for all three).

## Next admission step

First increment under domain-increment workflow after #577 CI wall-clock lands preferred; pure domain is unblocked now.

## Non-claims

- Does **not** clear PRODUCT HOLDs
- Does **not** implement domain increment product code
- Does **not** claim ROADMAP item complete

### Application package (parallel wave 2)

`cargo test -p console-ontology-application --lib` — **PASS** (6 tests listed)
