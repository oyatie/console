# Session reader oracle: retained JSON contract reconciliation

Read-only test candidate; root remains repository writer. No production code changes. Independent review/approval required before semantic test change. Original39failure audit separately preserved.

## Cause and authority

Three existing fault tests fail at JSON code comparison: current retained middleware service_unavailable versus imported unavailable. The helper comment incorrectly describes canonical middleware as plaintext. Current PRODUCT.md:46 and retained #991/ad62b515a6def8903edd50223a6c14bd29ad9e13 authority require JSON error envelopes; 01e06988 integrated SessionVerificationUnavailable into that retained mapper. Runtime/request-context, auth-rest and realtime retain distinct exact code/message contracts. No production contract should change to appease old oracle text.

## Exact candidate

Only backend/app/tests/auth_rest/account_fence_transport.rs changes, supplied as exact candidate plus unified patch. Existing helper gets explicit 503 code+message parameters. Canonical middleware routes /api/v1/users/me and /api/platform/orgs require service_unavailable / session verification is unavailable. Auth /api/v1/auth/passkeys requires service_unavailable / session verification unavailable. Realtime retains unavailable / session verification unavailable. Unreviewed path in the fault loop panics. No accept-either code/message or shape remains: all current reader errors must parse as JSON ErrorBody, eliminating the obsolete plaintext fallback. Existing JSON code/message string shape assertions remain for non503 responses; no new arbitrary 401message contract is invented.

Status, Set-Cookie absence, header/body token secrecy, UTF8, no access/refresh-token fields, same-route token-tier precedence, actual Auth outage/missing/null fault creation and restoration, identity/session row equality, audit equality, protected SSR omission, both websocket credential transports, actual recovery and unfenced positive controls remain unchanged. No security assertion, deadline, fixture/production schema, mutation oracle or original fault is removed. The former plaintext fallback allowed older shapes; requiring current JSON tightens shape acceptance while repairing owner-specific503 expectations.

## Required execution

Parent will reproduce current exact RED before applying reviewed candidate. Original fullrun dirty base is not substitute. Run these exact unchanged test names under functioning current prerequisites:

- account_fence_transport::session_reader_contract::auth_only_outage_preserves_pure_rejections_and_refuses_readers
- account_fence_transport::session_reader_contract::missing_projection_refuses_all_reader_boundaries_and_recovers
- account_fence_transport::session_reader_contract::null_projection_refuses_all_reader_boundaries_and_recovers

Then run the existing session_reader_contract module to check current JSON requirement also holds at the unchanged401/403 reader cases. Preserve exact executed/pass/fail counts and candidate/environment custody. No test executed in this packet. Real owner/status failure after correction remains RED and must be investigated rather than expanding accepted strings.

## Risk and rollback

Pre-mortem: a broad fallback could accept wrong transport owner or plaintext; a generic503message could hide wrong mapping; weakening fault/restoration assertions could skip the session fence. This candidate instead uses exact per-owner pairs and leaves all fault, secrecy, nonmutation and recovery code byte-identical. Blast radius: one shared integration-test helper and one four-route expectation block. Rollback restores this test-only diff; no persistent effects. Stop on changed source hashes, unknown route, unexpected nonJSON current owner, or reviewer-discovered assertion loss. Exact fixture review receipt plus currentRED/admission must precede application.
