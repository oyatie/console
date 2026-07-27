#!/usr/bin/env node
/**
 * L-F1 §6.3-22 real-browser proof — the window host in the LIVE React tree.
 *
 * jsdom is the harness that hid this bug for 15 module bodies, and jsdom also
 * cannot lay anything out. So this script asserts the two things only a real
 * engine can answer:
 *   1. `/console` mounts exactly ONE `[data-window-host]`, and it wraps
 *      `[data-cshell-root]` — not a host nested inside a screen section.
 *   2. Inserting that wrapper did not break the flex chain: the shell still
 *      fills the viewport, the screen body keeps real height, and the document
 *      has no horizontal scroll (C-42).
 *   3. Padding on that host produces DESIGN §4.7-2's "진짜 split" — the shell
 *      reflows sideways by exactly the pinned panel's width instead of being
 *      overlaid. A flex chain that swallowed the padding would leave the mount
 *      cosmetic, and jsdom cannot tell the two apart.
 *
 * No backend. The boot silent-refresh and every `/api/**` read are fulfilled by
 * route handlers, so this runs against a plain `vite dev` origin. The dev server
 * starts in `web/`; the script path is repo-relative, so run it from the REPO
 * ROOT (there is no `web/docs`):
 *
 *   (cd web && VITE_CONSOLE_DEV_PREVIEW=1 npx vite --host 127.0.0.1 --port 5199) &
 *   node docs/evidence/console/wave4/L-F1/browser-window-host.mjs
 */
import { mkdirSync } from "node:fs";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";

import { chromium } from "playwright";

const BASE = process.env.L_F1_BASE_URL ?? "http://127.0.0.1:5199";
const OUT_DIR = dirname(fileURLToPath(import.meta.url));
const VIEWPORT = { width: 1440, height: 900 };

const b64url = (value) => Buffer.from(value, "utf8").toString("base64url");

/** Client-decoded claims only (`decodeAccessClaims`); the browser never verifies the signature. */
const ACCESS_TOKEN = [
  b64url(JSON.stringify({ alg: "none", typ: "JWT" })),
  b64url(
    JSON.stringify({
      sub: "user-lf1",
      name: "전성진",
      email: "lf1@example.test",
      org: "org-lf1",
      roles: ["SUPER_ADMIN"],
      group_roles: [],
      feature_grants: [],
      branches: [],
      exp: Math.floor(Date.now() / 1000) + 3600,
    }),
  ),
  "unsigned",
].join(".");

/** Minimal truthful shapes for the shell's own reads; nothing here is asserted on. */
function stubBody(pathname) {
  if (pathname === "/api/v1/me/action-inbox") return '{"items":[]}';
  if (pathname === "/api/v1/me/notifications/summary") {
    return '{"total":0,"unread":0,"categories":[]}';
  }
  if (pathname === "/api/v1/users/me") return "{}";
  return "[]";
}

const failures = [];
function check(name, condition, detail) {
  const suffix = detail === undefined ? "" : ` — ${detail}`;
  if (condition) {
    console.log(`  ok   ${name}${suffix}`);
  } else {
    failures.push(`${name}${suffix}`);
    console.log(`  FAIL ${name}${suffix}`);
  }
}

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: VIEWPORT });

await page.route("**/api/v1/auth/token/refresh", (route) =>
  route.fulfill({
    status: 200,
    contentType: "application/json",
    body: JSON.stringify({
      access_token: ACCESS_TOKEN,
      requires_passkey_setup: false,
    }),
  }),
);
// Predicate, not a glob: a `**/api/**` glob also matches Vite's own
// `/src/api/*.ts` module URLs and breaks the dev server's module graph.
await page.route(
  (url) => url.pathname.startsWith("/api/"),
  (route) => {
    const { pathname } = new URL(route.request().url());
    // Fail the projection so the shell falls back to the JWT grants; a `{}`
    // projection would read as deny-by-omission and hide most of the nav.
    if (pathname === "/api/v1/me/authz") {
      return route.fulfill({ status: 503, contentType: "application/json", body: "{}" });
    }
    return route.fulfill({
      status: 200,
      contentType: "application/json",
      body: stubBody(pathname),
    });
  },
);

const pageErrors = [];
page.on("pageerror", (error) => pageErrors.push(String(error)));

console.log(`\n/console window host — ${BASE}`);
await page.goto(`${BASE}/console`, { waitUntil: "domcontentloaded" });
await page.waitForSelector("[data-cshell-root]", { timeout: 30_000 });
await page.waitForLoadState("networkidle");

const observed = await page.evaluate(() => {
  const hosts = Array.from(document.querySelectorAll("[data-window-host]"));
  const shellRoot = document.querySelector("[data-cshell-root]");
  const screenSection = document.querySelector("[data-cshell-screen]");
  return {
    hostCount: hosts.length,
    hostWrapsShell: Boolean(hosts[0] && shellRoot && hosts[0].contains(shellRoot)),
    hostInsideScreenSection: hosts.some((element) =>
      Boolean(element.closest("[data-cshell-screen]")),
    ),
    shellHeight: shellRoot?.getBoundingClientRect().height ?? 0,
    screenHeight: screenSection?.getBoundingClientRect().height ?? 0,
    viewportHeight: window.innerHeight,
    scrollWidth: document.documentElement.scrollWidth,
    clientWidth: document.documentElement.clientWidth,
    activeScreen: screenSection?.getAttribute("data-cshell-screen"),
  };
});

console.log(`  screen=${observed.activeScreen ?? "(none)"}`);
check("exactly one window host", observed.hostCount === 1, `count=${String(observed.hostCount)}`);
check("host wraps the shell root", observed.hostWrapsShell);
check("no host nested inside a screen section", !observed.hostInsideScreenSection);
// The host wrapper is a plain block by default; without the shell's flex
// `hostStyle` the console collapses to content height and every inner scroll
// region grows the page instead. jsdom cannot see this.
check(
  "shell still fills the viewport",
  Math.abs(observed.shellHeight - observed.viewportHeight) <= 2,
  `shell=${String(observed.shellHeight)} viewport=${String(observed.viewportHeight)}`,
);
check(
  "screen body keeps real height",
  observed.screenHeight > 100,
  `screen=${String(observed.screenHeight)}`,
);
check(
  "no document-level horizontal scroll",
  observed.scrollWidth <= observed.clientWidth,
  `scrollWidth=${String(observed.scrollWidth)} clientWidth=${String(observed.clientWidth)}`,
);
check("no uncaught page error", pageErrors.length === 0, pageErrors[0]);

// The pin is a real split, not an overlay (DESIGN §4.7-2: "본문이 옆으로 재배치
// … 오버레이 아닌 진짜 split"). The manager expresses that as `paddingRight` on
// the host, so apply the same padding the pinned panel would and require the
// shell to actually give the space back. Applied and reverted here rather than
// driven through a card, because opening one needs backend object data.
// `transition: padding 0.18s` means the split animates; measure after it lands
// (measuring synchronously reads the pre-transition width and reports a false
// "overlay" verdict).
const SPLIT_PROBE_PX = 360;
const SPLIT_SETTLE_MS = 400;
const beforeWidth = await page.evaluate(
  () => document.querySelector("[data-cshell-root]")?.getBoundingClientRect().width ?? null,
);
await page.evaluate((probe) => {
  const host = document.querySelector("[data-window-host]");
  if (host instanceof HTMLElement) host.style.paddingRight = `${String(probe)}px`;
}, SPLIT_PROBE_PX);
await page.waitForTimeout(SPLIT_SETTLE_MS);
const afterProbe = await page.evaluate(() => ({
  width: document.querySelector("[data-cshell-root]")?.getBoundingClientRect().width ?? null,
  overflowed:
    document.documentElement.scrollWidth > document.documentElement.clientWidth,
}));
await page.evaluate(() => {
  const host = document.querySelector("[data-window-host]");
  if (host instanceof HTMLElement) host.style.paddingRight = "";
});
await page.waitForTimeout(SPLIT_SETTLE_MS);
const restoredWidth = await page.evaluate(
  () => document.querySelector("[data-cshell-root]")?.getBoundingClientRect().width ?? null,
);
const split =
  beforeWidth === null || afterProbe.width === null || restoredWidth === null
    ? null
    : {
        before: beforeWidth,
        after: afterProbe.width,
        restored: restoredWidth,
        overflowed: afterProbe.overflowed,
      };

check("host padding reflows the shell (real split, not overlay)",
  split !== null && Math.abs(split.before - split.after - SPLIT_PROBE_PX) <= 1,
  split === null ? "host or shell root missing" : `${String(split.before)} -> ${String(split.after)}`);
check("the split does not push the document into horizontal scroll",
  split !== null && !split.overflowed);
check("removing the padding restores the full width",
  split !== null && Math.abs(split.before - split.restored) <= 1,
  split === null ? undefined : `restored=${String(split.restored)}`);

mkdirSync(OUT_DIR, { recursive: true });
await page.screenshot({ path: `${OUT_DIR}/console-window-host.png` });

await browser.close();

if (failures.length > 0) {
  console.error(`\n${String(failures.length)} check(s) failed`);
  process.exit(1);
}
console.log("\nall checks passed");
