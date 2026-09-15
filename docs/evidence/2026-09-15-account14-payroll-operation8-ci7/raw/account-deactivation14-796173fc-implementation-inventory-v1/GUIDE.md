# Admitted private Account deactivation guard candidate

Author and private candidate writer: `/root/inventory_repair`. Integration,
Cargo, PostgreSQL, Docker, generated/authority integration writer: `/root` only.
Immutable base `796173fc484279dbb9cca103caa57e026719ffaf`; exact four preimages
are in `preimages.json`. Later root-only disjoint payroll work does not change
these bindings. Independent implementation reviewer: `/root/capture_review`.

Admission: root ran `python3 tools/lanes/fanout.py admit --spec
/private/tmp/account-deactivation14-lane.json` with exit0. Receipt:
`/private/tmp/account-deactivation14-796173fc-admission/result.json`.
Root's exact14 probe executed3PASS/11FAIL:9 semantic failures and2 prerequisite
missing-guard failures, classified separately. This author did not run it.
Reviewed test SHA256:
`e06b77f8841cf3e7f5bc595756c1b8933c136e9cbfae1a536fac2862375dc471`.
Design: GUIDE `19f29899b2c7bf3d6ac448c0daec73f0cdfba6f648458a86b044333c73ddb773`
plus supplement `fee7092299e1a758c3e26d4334efa56e0671dc9f2a57f0a81632a9dc7293f35e`.

The patch changes exactly the identity adapter, shared custody query, existing
generator, and generated operator SQL. It adds no migrations, dependencies,
endpoints or tests. All six existing custody routine bodies and both existing
install blocks remain byte-identical. The migration ledger remains byte-identical.

The Company guard executes inside `with_audits` before any Company mutation.
It first refuses unsupported isolation and invalid Company context, then locks
the scoped users row and accounts root in that order. Company RLS stays active.
A separate post-lock SPI statement calls the existing strict fence projection;
any Account security row preserves custody. Missing root, unavailable authority,
NULL result and SQL failures propagate and roll back the Company transaction.
Only the exact guard-specific P0002/message maps to the existing Domain(NotFound),
preserving the missing/cross-tenant owner contract without an absence fallback.

Both transition and inactive replay retain the locks until transaction end.
Unfenced legacy paths retain the original sweep and both security audit events.
Fenced paths do neither; Company archive/transition events, truthful operation
revocation counts0, user context and Company session_generation remain.

The profile pins the new routine body, signature, options, language, owner,
volatility, exact search_path, exact EXECUTE ACL and absence of overloads.
Users rights are exactly non-grantable SELECT(id,org_id), UPDATE(id), with the
users owner as grantor. Raw PUBLIC/Account table grants, effective broad table
rights including PG18 MAINTAIN, other effective column rights, and grant options
refuse. Existing unrelated Company grants remain unchanged.

A completely absent guard and users extension is an upgrade input. Any present
corruption refuses before mutation. The full existing profile is certified under
users-first table locks before the installer uses root presence to select its
path. Certified roots also undergo the existing missing-root corruption check.
The existing-root path skips both old install blocks and root backfill, installs
only the new guard/column grants, then certifies the full output. Complete replay
returns before mutations. An installation failure rolls back its SQL statement.

Mechanical review: run `python3 verify.py` from this package. It reads integration
migration references, executes the existing generator with a private path adapter,
checks ledger and generated SQL bytes, parses/formats Rust through stdin, and
applies/checks/inverts the exact patch in a fresh private copy. `checks.json`
records exact invocations and counts. No Cargo, Rust tests or database commands.
No SQL runtime/parser acceptance is inferred from generation.

Root integration sequence after independent approval: recheck four preimages,
apply `candidate.patch`, run the ordinary generator `--check`, then rerun the same
admitted exact14 probe and retained root/profile/legacy lifecycle controls. Root
records discovered/executed counts and first-failure boundaries. Do not weaken
tests or treat an environment/fixture failure as semantic RED or GREEN.

Pre-mortem: stale post-wait classification, missing replay locks, widened users
authority, root backfill on upgrade, false credential audit claims, or swallowed
authority failure. Blast radius: four bounded Company/Account implementation
files. Detection: immutable review plus exact14 and retained profile/root/legacy
tests. Rollback: discard this private patch before integration; failed installer
rolls back atomically. After activation, recover with a compatible guarded binary
and profile, never unconditional fenced credential deletion. Stop on source drift,
scope expansion, weak assertions, unsupported proof, failed independent review,
or an unexpected runtime first failure.

Lenses: Cartesian doubt, Essentialism, Red Team, Operability / Day-2,
Blast-radius / cell-based, Zero-trust / defense-in-depth. Remaining HOLDs:
independent implementation approval, root compiler/SQL runtime and14GREEN,
retained root/profile/lifecycle gates, broader cancellation/unrelated-subject
progress/hidden-authority cases, exact-head closure, native enrollment/Auth7 and
production authority. Arbitrary privileged post-startup routine replacement is
outside the demonstrated runtime trust boundary; no per-operation full catalog
fingerprint is introduced.
