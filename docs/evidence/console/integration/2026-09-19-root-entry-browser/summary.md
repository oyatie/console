# Private public-entry browser smoke

Observed **36 checks passed, 0 failed** using Playwright1.63.0 and Chromium153.0.8010.12 against the actual Console binary on http://127.0.0.1:49986. This is public entry smoke only; no database or Account/workspace journey was qualified. Source and binary bytes remained stable throughout. The owned server PID69397 stopped afterward with exit code0.

The anonymous root and ordinary-cookie root rendered the existing native Public page. Actual CSS/JS responses were200 and matched repository source SHA256. Login and create-account links navigated to /account and /account/register, both correctly503 without configured Auth. The only browser console errors were those two expected503resource reports at these destinations; public entry had no console errors, JavaScript exceptions or failed network requests. All captured requests used GET/HEAD, stayed on the loopback origin, and fetched no /api or /pkg resources. Public pages contained no forms or Leptos islands. These request observations do not establish database nonmutation under a configured environment.

Keyboard checks observed a visible initial skip link, Enter moving focus to main, and the registration action reached by Tab with a visible outline. Desktop1440x1000, mobile320x900 and a **200percent approximation** had no horizontal document overflow. Approximation used720x500CSS viewport with deviceScaleFactor2; this is not actual browser or OS zoom. Screenshots preserve all three layouts and focused/unavailable states. Source-independent visual/task critique is included in visual-critique.json; a second review from integration_security is requested separately. Neither substitutes for representative-user or full accessibility evidence.

Binary SHA256: 7ab368b41b3f5340f211ce2bde52db8541dcb3abbf846991646fbf7d0652cbe6
Head at observation: 17443500fb44a4026208cbe5c685dbfd2f55b348 plus source hashes in report.json. The recorded pre-run git status is empty and both recorded heads are17443500fb44a4026208cbe5c685dbfd2f55b348. The source was committed and clean at the browser run; an earlier dirty-tree inspection does not describe this run. This verifier changed no repository files. Hashes observed together bind this run; build provenance remains in the parent's build receipt.

Prerequisite failures remain preserved in predecessor directories. The originally supplied Playwright directory was incomplete; its npm cache also failed offline. Authorized temporary repair installed exactPlaywright1.63.0 with lifecycle scripts disabled and retained lockfile integrity. Loopback listening required sandbox escalation. One earlier harness attempt incorrectly hashed absent fullChrome while actually launching headless; the final successor explicitly launches and hashes the installed headless executable. Neither prerequisite attempt is counted as a successful page run.

Executed command:
`PLAYWRIGHT_BROWSERS_PATH=/private/tmp/console-preparation-browsers node /private/tmp/console-root-entry-browser-smoke-20260919-round4/smoke.cjs`

No release, production exposure, TLS, screen-reader, IME, real200percent zoom, authenticated recovery or complete accessibility acceptance is claimed.

This metadata-only successor preserves every raw runtime report, screenshot and executed script byte from round4. It corrects the stale worktree narrative and adds visual critique; no new browser execution is claimed.
