# Root entry round3: exact error precedence and HEAD behavior

This immutable successor incorporates /private/tmp/console-root-public-entry-design-20260919-round2 (manifest SHA256 b51847ae66168edc06b50d9b57000879e411ca2ffdd3ecbc0f5b627013c76cbb), except for the two explicit corrections below. Root remains sole writer. Exact implementation/new-fixture code is not yet approved. No admitted implementation or executed tests are claimed.

## Cookie projection errors delegate to the existing owner

Correct round2 step3: if safe native-cookie-presence projection returns an error, root calls **existing native_account_page(PublicRoot)** with all original headers and method, instead of mapping the projection error directly to a document/status. Any true native-credential projection does the same. Only a successful false projection can choose legacy Authorization delegation or static Public.

Reason: parse_cookies preserves Authorization rejection before cookie size/parsing. Root's cookie-only discriminator executes earlier solely for route selection, so rendering its TooLarge error directly would change configured Authorization+oversized-cookie from current owner400 to new413. Delegating projection errors restores the full existing owner order: method/configuration/document metadata/Authorization ambiguity/cookie parsing/session validation. It also preserves configured dependency unavailability and metadata errors ahead of cookie parsing where that is the current owner behavior. No second parser or fallback is added. Projection errors must never choose Legacy/Public.

Mandatory combined boundary cases: through actual root, Authorization+oversized Cookie and Authorization+duplicate native Cookie must equal the exact existing native Account owner status/body class under the same configured fixture (ordinary valid metadata yields400). Missing configuration and invalid metadata combinations preserve their actual existing owner precedence; do not hardcode400 ahead of it. No native/secret projection or state mutation may occur in any refusal.

## Exact HEAD semantics

- Static Public with successful false credential projection and no Authorization: GET200; HEAD200 with Axum's normal response-body stripping. Same public document/privacy headers, no authority reads or mutable effects.
- Native credential or cookie-projection-error branch: HEAD405 through the existing native_account_page method guard; do not validate, refresh, or mutate a native session to answer HEAD.
- Legacy delegated branch: existing ui_shell GET/HEAD behavior unchanged; no newly imposed native document CSP or method rule.
- /work and every other existing route unchanged.

## Review provenance and remaining gate

integration_tests reviewed round2 fixture strategy, confirmed original no-cookie root probe alone is insufficient, and required these exact HEAD semantics. integration_security independently identified the projection-error precedence bug and agreed this owner-delegating correction resolves it without new parsing. Formal fixture receipt must bind this addendum before formal security receipt. Required source/fixture hashes, current executableRED, four focused review rounds and lane admission remain root responsibilities before implementation. The original Account public-root test remains byte-identical; four old health-readiness presentation expectations require the explicitly described reviewed reconciliation.
