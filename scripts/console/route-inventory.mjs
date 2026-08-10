import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';

export const CONSOLE_NAV_SOURCE = 'web/src/console/shell/nav.ts';
export const CONSOLE_REGISTRY_SOURCE = 'web/src/console/screens/registry.ts';

// The single legitimate "no facts" value. `route_source_present: false` is the
// only thing that lets a consumer distinguish "the console has no frontend, so
// it presents no routes" from "extraction produced nothing". Consumers must
// treat a missing/false flag as absent source and refuse to corroborate any
// route claim against it.
export const ABSENT_CONSOLE_ROUTE_FACTS = Object.freeze({ route_source_present: false, facts: Object.freeze({}) });

function literals(text, declaration) {
  const match = text.match(new RegExp(`(?:export\\s+)?const\\s+${declaration}[\\s\\S]*?=\\s*\\[([\\s\\S]*?)\\](?:\\s+as const)?`));
  if (!match) throw new Error(`missing ${declaration}`);
  return [...match[1].matchAll(/"([A-Za-z][A-Za-z0-9]*)"/g)].map((entry) => entry[1]);
}
export function extractConsoleRouteFactsFromTexts(navText, registryText) {
  const mounted = literals(navText, 'MOUNTED_SCREEN_KEYS');
  const exposed = literals(navText, 'EXPOSED_SCREEN_KEYS');
  const nav = [...navText.matchAll(/screen:\s*"([A-Za-z][A-Za-z0-9]*)"/g)].map((entry) => entry[1]);
  const bodyBlock = registryText.match(/SCREEN_REGISTRY[^=]*=\s*\{([\s\S]*?)\n\};/);
  if (!bodyBlock) throw new Error('missing SCREEN_REGISTRY');
  const bodies = [...bodyBlock[1].matchAll(/^\s*([A-Za-z][A-Za-z0-9]*):/gm)].map((entry) => entry[1]);
  const all = new Set([...mounted, ...exposed, ...nav, ...bodies]);
  const facts = Object.fromEntries([...all].sort().map((key) => [key, { source_mounted: mounted.includes(key), production_exposed: exposed.includes(key), registry_body_present: bodies.includes(key), nav_declared: nav.includes(key) }]));
  return { route_source_present: true, facts, mounted, exposed, nav, bodies };
}

/**
 * Absence of BOTH route sources is the only tolerated failure: the 2026-07-28
 * clean-slate pivot deleted the frontend, and a console with no frontend
 * presents no routes. Every other failure — a renamed constant, a malformed
 * SCREEN_REGISTRY, an unreadable file, a half-landed frontend — propagates.
 * Swallowing those would report "no routes" for a console that has them.
 */
export function extractConsoleRouteFacts(repoRoot) {
  const navPath = path.join(repoRoot, CONSOLE_NAV_SOURCE);
  const registryPath = path.join(repoRoot, CONSOLE_REGISTRY_SOURCE);
  const navPresent = existsSync(navPath);
  const registryPresent = existsSync(registryPath);
  if (navPresent !== registryPresent) throw new Error(`console route source is partially present: ${navPresent ? CONSOLE_REGISTRY_SOURCE : CONSOLE_NAV_SOURCE} is missing`);
  if (!navPresent) return ABSENT_CONSOLE_ROUTE_FACTS;
  return extractConsoleRouteFactsFromTexts(readFileSync(navPath, 'utf8'), readFileSync(registryPath, 'utf8'));
}

// ---------------------------------------------------------------------------
// ADR-0030 §8 planning-only: the Rust/Leptos side of the inventory.
//
// The two tombstone paths above pin the deleted React stack, but ADR-0030 §1
// makes the console "a Leptos application composed of workspace crates", so
// the §8-forbidden artifact is a Rust workspace member — invisible to any
// web/** path assertion. The subject that can see it is the build graph
// itself: a console crate must be a workspace member to build, and Cargo is
// the only authority on what the member set is (the real manifest discovers
// members through `crates/<domain>/*` globs, so a crate becomes a member with
// zero manifest edits; manifest-text parsing would never see it).
//
// Deliberately NOT a grep over Rust source text: a source-text gate catches
// one spelling in one position and fails open (console-9ze). Three
// build-graph tiers instead, each dodging a different evasion:
//   1. member names ending in `-ui` — ADR-0030 §6 charters
//      `console-<domain>-ui` as the surface-crate name, so the chartered name
//      may not exist while the §7 gate is closed (catches a hand-rolled,
//      leptos-free UI crate under the chartered name);
//   2. any workspace member declaring a Leptos-family dependency — cargo
//      metadata reports the TRUE package name, so
//      `view = { package = "leptos" }` renames cannot hide it;
//   3. any Leptos-family package in backend/Cargo.lock — a member cannot
//      build without its graph in the lockfile (CI's `cargo metadata
//      --locked` step keeps it honest), so leptos smuggled in as a path
//      dependency of an existing member is still visible here.
// Unreadable inventory (missing manifest/lockfile, cargo failure, zero
// members, zero locked packages) throws — the gate fails closed, never
// reports a clean state it did not observe.
//
// The import lives down here so the tombstone constants above keep their
// lines: ADR-0030 cites route-inventory.mjs:4-5 verbatim. ESM hoists it.
import { execFileSync } from 'node:child_process';

export const BACKEND_WORKSPACE_MANIFEST = 'backend/Cargo.toml';
export const BACKEND_WORKSPACE_LOCKFILE = 'backend/Cargo.lock';

// Matches the framework's package namespace (leptos, leptos_axum,
// leptos_meta, leptos-use, ...) on a separator boundary, so `leptose`-style
// strangers do not match. Wrappers that carry other names (e.g. `leptonic`)
// cannot dodge: they depend on `leptos` itself, which tier 3 sees
// transitively through the lockfile.
export const LEPTOS_PACKAGE_FAMILY = /^leptos(?:[_-]|$)/;

// ADR-0030 §6: the chartered per-domain surface crate is console-<domain>-ui.
export const CONSOLE_UI_MEMBER_NAME = /(?:^|-)ui$/;

const ADR = 'ADR-0030 §8 planning-only violation';

/**
 * Pure tier 1+2 classifier over `cargo metadata` packages. Throws instead of
 * returning clean when handed nothing or anything it cannot read: a scan that
 * examined zero members has observed nothing and must not pass.
 */
export function workspaceMemberViolations(packages, { repoRoot } = {}) {
  if (!Array.isArray(packages) || packages.length === 0) {
    throw new Error('planning-only workspace scan examined zero workspace members; refusing to report a clean state it did not observe');
  }
  const violations = [];
  for (const pkg of packages) {
    if (typeof pkg?.name !== 'string' || pkg.name === '' || typeof pkg?.manifest_path !== 'string' || !Array.isArray(pkg?.dependencies)) {
      throw new Error('planning-only workspace scan met a member missing name/manifest_path/dependencies; refusing to classify what it cannot read');
    }
    const where = repoRoot ? path.relative(repoRoot, pkg.manifest_path) : pkg.manifest_path;
    if (CONSOLE_UI_MEMBER_NAME.test(pkg.name)) {
      violations.push(`${ADR}: workspace member '${pkg.name}' (${where}) carries the §6-chartered console surface-crate name '-ui' while the §7 gate is closed`);
    }
    for (const dep of pkg.dependencies) {
      if (typeof dep?.name !== 'string' || dep.name === '') {
        throw new Error(`planning-only workspace scan met an unreadable dependency of '${pkg.name}'; refusing to classify what it cannot read`);
      }
      if (LEPTOS_PACKAGE_FAMILY.test(dep.name)) {
        const alias = typeof dep.rename === 'string' && dep.rename ? ` (renamed locally to '${dep.rename}')` : '';
        violations.push(`${ADR}: workspace member '${pkg.name}' (${where}) declares Leptos-family dependency '${dep.name}'${alias}; §1 makes Leptos the console stack, so this is a console implementation artifact`);
      }
    }
  }
  return violations;
}

/**
 * Pure lockfile reader: every `[[package]]` block must yield a name, and a
 * lockfile with no blocks is unreadable, not clean.
 */
export function lockedPackageNames(lockfileText) {
  if (typeof lockfileText !== 'string' || !lockfileText.includes('[[package]]')) {
    throw new Error(`${BACKEND_WORKSPACE_LOCKFILE} holds no [[package]] blocks; the locked dependency graph is unreadable and unreadable must fail`);
  }
  return lockfileText.split('[[package]]').slice(1).map((block, index) => {
    const name = block.match(/^\s*name\s*=\s*"([^"]+)"/m);
    if (!name) throw new Error(`${BACKEND_WORKSPACE_LOCKFILE} [[package]] block ${index + 1} carries no name; refusing to scan a graph it cannot read`);
    return name[1];
  });
}

/**
 * The Rust-side inventory: enumerate workspace members through Cargo's own
 * authority (`cargo metadata --no-deps`, glob-aware, rename-transparent) and
 * the locked graph, and classify against the ADR-0030 §8 planning-only
 * invariant. Every unreadable input throws. This check may be retired only in
 * the change that opens the §7 gate (ADR-0030 §8).
 */
export function extractConsoleWorkspaceFacts(repoRoot) {
  const manifestPath = path.join(repoRoot, BACKEND_WORKSPACE_MANIFEST);
  const lockfilePath = path.join(repoRoot, BACKEND_WORKSPACE_LOCKFILE);
  if (!existsSync(manifestPath)) throw new Error(`${BACKEND_WORKSPACE_MANIFEST} is missing under ${repoRoot}; the planning-only gate cannot see its subject and unreadable must fail`);
  if (!existsSync(lockfilePath)) throw new Error(`${BACKEND_WORKSPACE_LOCKFILE} is missing under ${repoRoot}; the planning-only gate cannot see the locked graph and unreadable must fail`);
  let stdout;
  try {
    stdout = execFileSync(
      'cargo',
      ['metadata', '--no-deps', '--format-version', '1', '--offline', '--manifest-path', manifestPath],
      { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, stdio: ['ignore', 'pipe', 'pipe'] },
    );
  } catch (error) {
    const stderr = typeof error?.stderr === 'string' ? error.stderr.trim() : '';
    throw new Error(`cargo metadata failed; the planning-only gate cannot enumerate workspace members and unreadable must fail: ${stderr || error.message}`);
  }
  let metadata;
  try {
    metadata = JSON.parse(stdout);
  } catch {
    throw new Error('cargo metadata emitted unparseable JSON; the planning-only gate cannot enumerate workspace members and unreadable must fail');
  }
  const memberViolations = workspaceMemberViolations(metadata.packages, { repoRoot });
  const lockedNames = lockedPackageNames(readFileSync(lockfilePath, 'utf8'));
  const lockedViolations = lockedNames
    .filter((name) => LEPTOS_PACKAGE_FAMILY.test(name))
    .map((name) => `${ADR}: ${BACKEND_WORKSPACE_LOCKFILE} resolves Leptos-family package '${name}' — a console implementation artifact is in the build graph (possibly transitive; \`cargo tree -i ${name}\` in backend/ names the introducer)`);
  return {
    workspace_scanned: true,
    member_count: metadata.packages.length,
    locked_package_count: lockedNames.length,
    violations: [...memberViolations, ...lockedViolations],
  };
}
