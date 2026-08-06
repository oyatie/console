# Setup tip — rustfmt + executed-tests baseline (ops.rustfmt-drift / ops.executed-tests-baseline)

When changing `backend/**` tests or code before push:

1. `(cd backend && cargo fmt --all -- --check)` — fix with `cargo fmt --all`.
2. `node scripts/check-executed-tests.mjs` — if it says attributes gained, run
   `node scripts/check-executed-tests.mjs --update` and commit
   `docs/program/executed-tests-baseline.json` in the **same** PR.
3. Then rebuild authority tip (C/T) if ledger/manifest already staged.

Wired into `.grok/workflows/domain-increment.rhai` Admit phase.
