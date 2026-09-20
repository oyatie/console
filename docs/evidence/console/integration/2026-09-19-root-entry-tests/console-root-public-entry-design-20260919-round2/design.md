# Root public entry: owner-reuse successor design

This supersedes the initial absent-header-only proposal. Root remains sole writer. This is a design candidate, not admitted source, accepted evidence, or a completed root journey. Review must complete before semantic fixture changes or source implementation.

## Intended behavior and canonical owner

GET / gives an unauthenticated visitor the existing native Account public page with login/create-account actions even when the browser carries unrelated cookies. A current native Account credential reaches the existing Account page using current session validation and disclosure. Expired/revoked native access reaches the public page without silently refreshing or consuming proof. Invalid, ambiguous and unavailable native requests retain exact owner refusal/unavailable outcomes. Existing legacy Bearer root consumers keep their authorized shipping Home only when no native Account credential is supplied. /work and protected shipping routes remain unchanged.

Do not classify authentication from an empty business list, failed principal resolution, presence of a Company row or a client identifier. No raw cookie parser enters App.

## Minimal responsible-layer extension

1. In auth-rest account_browser.rs, factor the existing parse_cookies function into its **unchanged first Authorization-present rejection** followed by a private parse_cookie_fields containing the existing size accounting, HeaderValue UTF8 handling, field-line and semicolon grammar, four native cookie names, duplicate checks and CookieValue result. Existing API/document parse_cookies callers continue their exact rejection ordering and error mapping. No historical command, cookie syntax, session validation, transport status or strict limit changes.
2. Expose a narrow safe projection through account_entry.rs and existing auth-rest reexports: native Account credential presence derived from that shared private cookie-only parser. It returns Result<bool, NativeEntryError>; true iff any of session/refresh/enrollment/login slots is not Absent (including Malformed); it carries no credential values or authority. Duplicate/non-UTF8/oversized cookie parse errors remain exact sanitized NativeEntryError, never false. Native malformed values must route to the existing owner and refuse, not be reclassified as anonymous. Unknown cookie names, console_refresh, and console_session remain ignored by the same owner grammar; no new accepted native name exists.
3. Split only GET / from ui_shell into ui_root. Keep /work -> ui_shell. ui_root obtains the safe projection. On error use existing Page::Refused/Unavailable and mapped status. If native credential presence is true, invoke existing native_account_page with root presentation mode, passing all original headers and method intact. If false and Authorization is present, invoke unchanged ui_shell. If false and Authorization absent, render existing native_account::document(Page::Public, OK), with no Auth database dependency or business data reads.
4. Extend existing native_account_page presentation choice narrowly to distinguish root from /account and /account/register. A small private mode enum (PublicRoot, Account, Registration) replaces its existing registration bool; native_account_entry receives registration=true only for Registration. Map NativeAccountEntry::SignIn to Page::Public only for PublicRoot. Active/context/can_logout, Refused, Unavailable, status and GET-method checks remain exact existing mappings for every mode. This is presentation choice, not a screen generator or new credential writer.

The pure cookie-presence projection happens before any legacy delegation; therefore Bearer + native cookie follows the native owner and refuses AmbiguousCredentials. Bearer + unrelated well-formed cookies preserves the existing legacy route. This avoids the initial proposal's blank home for ordinary cookie states and the rejected blanket Cookie+Bearer compatibility change.

## Truth table

| Supplied request | Result |
|---|---|
| No credentials, or only unrelated/legacy non-native cookies | Page::Public200; no new authority/session rows/cookies |
| Valid legacy Authorization, no native credential, including ordinary unrelated cookies | Existing ui_shell and authorized projections unchanged |
| Invalid/malformed/duplicate Authorization without native cookie | Existing legacy rejection/deny behavior, never inferred anonymous |
| Native session/refresh/enrollment/login cookie, no Authorization | Existing native_account_entry classification; SignIn presentation becomes Public at root |
| Any native credential plus Authorization | Existing native parser rejects ambiguity; no legacy fallback |
| Malformed native cookie value | Existing native owner refuses; no Public fallback |
| Duplicate native cookie, invalid UTF8, excessive cookie bytes | Existing parser-derived refusal/status; no authority or DB fallback |
| Valid native session but context unavailable | Existing Active+Unavailable context mapping503, not anonymous |
| Expired/revoked native access with refresh present | Existing SignIn outcome rendered Public; refresh unconsumed, no session repair |
| Cross-site subresource carrying native cookies | Existing document admission refusal; no active account projection |

Static Public is entirely public, has no identity projection, and therefore requires no current session or document metadata admission. Metadata admission remains unchanged when any native credential is supplied. Review this distinction explicitly. GET/HEAD behavior must be documented: native owner keeps GET-only checks, while static Public has no credential-consuming operation.

## Exact existing RED and new-boundary proof requirements

Existing actual Account root test stays byte-identical:
`CONSOLE_BUCK_NEEDS_POSTGRES_TEST_EXACT=account_browser::native_entry_public_root_exposes_discoverable_routes tools/buck/test_needs_postgres.sh //tools/buck:app-auth-rest-pg`

Original dirty-base full-suite failure proves historical missing Cache-Control only. Root must execute this on current source before lane admission. It also exactly asserts root action links, Korean content, privacy/frame envelope, no business navigation and no owner mutations.

Because this successor changes more than the original no-cookie path, the original single root test is insufficient by itself. Fixture reviewer must provide bounded additional real-router probes reusing existing native_entry_get/enrolled/signed_fixture/row snapshots, with exact expected branches: unrelated cookie Public; each four native cookie classes; active native Account; expired/revoked no silent refresh; malformed and duplicate native; mixed native+Bearer no fallback; cross-site subresource denied; context/database unavailable not anonymous; legacy Bearer alone and with unrelated cookie retains same authorized rows/CSP/island. Also verify parser factoring preserves existing parse_cookies error precedence under Authorization+malformed/oversize/native duplicates. No implementation-mirroring broad unit suite needed; retain existing strict input tests and add only missing observable boundary coverage.

## Existing conflicting fixture contracts needing explicit semantic review

health_readiness.rs has four live old-root assumptions. None may be silently loosened, skipped, removed, or made vacuously pass:

- ui_shell_serves_empty_ssr_html (:107): requires noDB root exactly render_shell and returns after first successful route. Updated review must explicitly require / public native document and independently verify every protected route remains exact empty shell; must not hide root503 by proceeding to next route.
- browser_navigation_reaches_no_authorized_screen_adr_0042 (:528): console_refresh (old non-native name) at / currently equals empty shell. Preserve positive Bearer run/island assertions and negative cookie business-disclosure assertions; root public presentation is the only intended expected-body difference. This does not authorize console_refresh or adoption of ignored proposed ADR0042 cookie transport.
- proposed_ssr_session_cookie_does_not_authorize_html_get_yet (:595): same narrow presentation reconciliation for ignored console_session; retain no run/island and all bearer/API cookie denial proof. Native __Host-console_account_* names are a separate accepted owner.
- ui_shell_omits_runs_unless_payroll_run_read (:661): only unauthenticated root expected body changes to exact existing native Public. MEMBER bearer remains exact empty shell, SUPER_ADMIN same authorized row and island, no salary/payslip leakage, no WASM on Public.

Other protected deny-by-omission/persona tests must remain byte-identical. Existing Account root test remains byte-identical. Any approved fixture modification must be source-hashed, independently reviewed, and the corresponding intended RED observed before implementation; do not weaken a security assertion to accommodate this UI repair.

## Visual/runtime scope

Reuse native Page::Public; no new layout, stylesheet, dependency or frontend framework. Public page contains normal /account and /account/register actions, Korean typography, landmark/viewport, and existing exact privacy/CSP document envelope. Protected shipping CSP and inline ISLAND_BOOTSTRAP/hydration stay unchanged. Native Public currently loads its existing external native-account.js module even without a form; review/browser verification must ensure it produces no incidental owner writes and no unnecessary Leptos islands. Actual browser screenshots/keyboard/mobile checks remain separate from HTTP fixture proof.

## Risk, rollback, and review sequence

Pre-mortem: classification fallback could turn native corruption into Public; changing parser ordering could bypass ambiguity; blanket Cookie rejection could break legacy; routing /work differently could alter protected UX; relaxing tests could conceal salary leakage; using strict native CSP on shipping could break hydration. Detect with the exact boundary cases above and mechanical diff review. Source blast radius is auth-rest parser extraction plus safe projection/export and App root presentation wiring; no migrations, SQL, generated contract, lockfile, historical record or new business transition. Rollback is the small admitted source diff; no persistent effects to restore.

Stop if current named RED is unexecutable/green, source hashes drift without rebind, reviewers cannot approve exact fixture reconciliation, or parser equivalence/current-authentication proof fails. Four focused independent design review rounds are required by parent: fixture/oracle, identity/security, frontend/interaction, and final mechanical design. Fixture review precedes security receipt. This packet is source-private and has no admission/release claim.
