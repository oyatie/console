import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { CONSOLE_NAV_SOURCE, CONSOLE_REGISTRY_SOURCE, extractConsoleRouteFacts, extractConsoleRouteFactsFromTexts, extractConsoleWorkspaceFacts, lockedPackageNames, workspaceMemberViolations } from './route-inventory.mjs';

const repoRoot = fileURLToPath(new URL('../../', import.meta.url));
const authorityRegistry = JSON.parse(readFileSync(new URL('../../docs/program/console-capability-registry.json', import.meta.url)));
const ROUTE_CLAIM_FIELDS = ['source_mounted', 'production_exposed', 'registry_body_present', 'nav_declared'];

// The 2026-07-28 clean-slate pivot deleted the frontend, so the immutable
// candidate holds neither console route source. The former positive/negative
// tests read those two files out of the candidate and mutated them; both are
// permanently dead, because the extractor parses TypeScript shapes
// (MOUNTED_SCREEN_KEYS / EXPOSED_SCREEN_KEYS / SCREEN_REGISTRY) that the Leptos
// rebuild will never emit at those paths. They are deleted rather than skipped.
// What replaces them is the contract that still binds in the no-frontend state:
// the registry may not claim route presentation that no source can corroborate.
// Asserted against HEAD, not the registry's bound candidate. The candidate is
// unreachable wherever it matters most: this repository allows squash merges
// only, so C is orphaned the moment a pull request lands and `git ls-tree <C>`
// on `main` is "fatal: not a tree object" — which made this contract
// unverifiable on the only branch that ships. HEAD carries identical `web/**`
// content because T touches nothing but the three authority documents, so it
// asserts the same invariant and always resolves. A git failure here is a
// thrown error, never a skip.
const candidateTracks = (relativePath) => execFileSync(
  'git',
  ['-C', repoRoot, 'ls-tree', 'HEAD', '--', relativePath],
  { encoding: 'utf8' },
).trim() !== '';

test('the tree under test holds no console route source', () => {
  assert.deepEqual(
    [CONSOLE_NAV_SOURCE, CONSOLE_REGISTRY_SOURCE].filter(candidateTracks),
    [],
    'candidate tracks a console route source again: restore the source-inventory contract instead of asserting the empty state',
  );
});

test('with no route source the registry may not claim any route presentation', () => {
  const declared = authorityRegistry.capabilities.flatMap((capability) => capability.route_presentation.route_keys);
  assert.deepEqual(declared, [], 'registry declares route keys but the candidate has no console route source');
  assert.deepEqual(
    authorityRegistry.capabilities
      .filter((capability) => ROUTE_CLAIM_FIELDS.some((field) => capability.route_presentation[field] === true))
      .map((capability) => capability.id),
    [],
    'capabilities claim mounted/exposed/registry-body/nav route presentation that no candidate source can corroborate',
  );
  assert.deepEqual(authorityRegistry.source_inventory?.unmodeled_keys ?? [], [], 'unmodeled route keys cannot exist without a route source');
});

test('route fact extraction fails closed on anything other than a wholly absent source', () => {
  const root = mkdtempSync(path.join(tmpdir(), 'console-route-source-'));
  const write = (relativePath, text) => {
    mkdirSync(path.join(root, path.dirname(relativePath)), { recursive: true });
    writeFileSync(path.join(root, relativePath), text);
  };
  const nav = 'export const MOUNTED_SCREEN_KEYS = ["overview"] as const;\nexport const EXPOSED_SCREEN_KEYS = [] as const;\nconst items = [{ screen: "overview" }];\n';
  const registry = 'export const SCREEN_REGISTRY = {\n  overview: () => null,\n};\n';
  try {
    assert.deepEqual(extractConsoleRouteFacts(root), { route_source_present: false, facts: {} });

    write(CONSOLE_NAV_SOURCE, nav);
    assert.throws(() => extractConsoleRouteFacts(root), /partially present/, 'a half-landed frontend must not report "no routes"');

    write(CONSOLE_REGISTRY_SOURCE, registry);
    const facts = extractConsoleRouteFacts(root);
    assert.equal(facts.route_source_present, true);
    assert.deepEqual(facts.facts, { overview: { source_mounted: true, production_exposed: false, registry_body_present: true, nav_declared: true } });

    write(CONSOLE_NAV_SOURCE, nav.replace('MOUNTED_SCREEN_KEYS', 'RENAMED_SCREEN_KEYS'));
    assert.throws(() => extractConsoleRouteFacts(root), /missing MOUNTED_SCREEN_KEYS/, 'a renamed constant must not be absorbed as "no routes"');

    write(CONSOLE_NAV_SOURCE, nav);
    write(CONSOLE_REGISTRY_SOURCE, registry.replace('SCREEN_REGISTRY', 'RENAMED_REGISTRY'));
    assert.throws(() => extractConsoleRouteFacts(root), /missing SCREEN_REGISTRY/, 'a malformed registry must not be absorbed as "no routes"');
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('extracted facts classify mounted, exposed, nav-declared and registry-bodied keys apart', () => {
  const facts = extractConsoleRouteFactsFromTexts(
    'export const MOUNTED_SCREEN_KEYS = ["alpha", "beta"] as const;\nexport const EXPOSED_SCREEN_KEYS = ["alpha"] as const;\nconst nav = [{ screen: "alpha" }];\n',
    'export const SCREEN_REGISTRY = {\n  alpha: () => null,\n  gamma: () => null,\n};\n',
  );
  assert.deepEqual(facts.facts, {
    alpha: { source_mounted: true, production_exposed: true, registry_body_present: true, nav_declared: true },
    beta: { source_mounted: true, production_exposed: false, registry_body_present: false, nav_declared: false },
    gamma: { source_mounted: false, production_exposed: false, registry_body_present: true, nav_declared: false },
  });
});

// ---------------------------------------------------------------------------
// ADR-0030 §8 planning-only, Rust side (console-9ze). The tombstone assertion
// above pins the deleted React paths; ADR-0030 §1 makes the console a Leptos
// WORKSPACE CRATE, so the forbidden artifact class is a Rust workspace member
// that no web/** assertion can observe. RED baseline recorded on
// 7b568df9db961fa8aa3f36917eaa13c6af2c3023: with a live `console-payroll-ui`
// member declaring leptos + leptos_axum planted under the `crates/payroll/*`
// member glob (zero Cargo.toml edits needed), this file passed 4/4. The three
// tests below are the sighted replacement.

test('the workspace holds no console implementation artifact (ADR-0030 §8 planning-only)', () => {
  const facts = extractConsoleWorkspaceFacts(repoRoot);
  assert.ok(facts.member_count > 0, 'workspace scan examined zero members; a gate that saw nothing must not pass');
  assert.ok(facts.locked_package_count > 0, 'lockfile scan examined zero packages; a gate that saw nothing must not pass');
  assert.deepEqual(
    facts.violations,
    [],
    'ADR-0030 §8 forbids console implementation artifacts until every §7 condition is measured green; this check may be retired only in the change that opens the gate',
  );
});

const writeWorkspaceFixture = (root, crates, { lockfilePackages } = {}) => {
  // Mirrors the real manifest's `crates/<domain>/*` discovery: the RED probe
  // proved a crate becomes a member with zero manifest edits, so the fixture
  // must exercise glob discovery, not an explicit member list.
  mkdirSync(path.join(root, 'backend'), { recursive: true });
  writeFileSync(path.join(root, 'backend', 'Cargo.toml'), '[workspace]\nresolver = "3"\nmembers = ["crates/*/*"]\n');
  for (const crate of crates) {
    const dir = path.join(root, 'backend', crate.path);
    mkdirSync(path.join(dir, 'src'), { recursive: true });
    writeFileSync(path.join(dir, 'src', 'lib.rs'), '');
    writeFileSync(path.join(dir, 'Cargo.toml'), `[package]\nname = "${crate.name}"\nversion = "0.0.0"\nedition = "2021"\n\n[dependencies]\n${crate.dependencies ?? ''}`);
  }
  if (lockfilePackages) {
    writeFileSync(
      path.join(root, 'backend', 'Cargo.lock'),
      `version = 4\n${lockfilePackages.map((name) => `\n[[package]]\nname = "${name}"\nversion = "0.0.0"\n`).join('')}`,
    );
  }
};

test('the planning-only scan sees the Rust artifact class the tombstone paths cannot', () => {
  const root = mkdtempSync(path.join(tmpdir(), 'console-planning-only-'));
  try {
    writeWorkspaceFixture(root, [
      // the §6-chartered artifact exactly as ADR-0030 describes it
      { path: 'crates/payroll/ui', name: 'console-payroll-ui', dependencies: 'leptos = "0.9.0-beta"\nleptos_axum = "0.9.0-beta"\n' },
      // rename dodge: crate name conforms to backend shapes, dependency aliased
      { path: 'crates/payroll/widgets', name: 'console-payroll-widgets', dependencies: 'view-framework = { package = "leptos", version = "0.9.0-beta" }\n' },
      // hand-rolled dodge: chartered -ui name with no leptos at all
      { path: 'crates/hr/ui', name: 'console-hr-ui' },
      // near-misses that must NOT be flagged: -gui name; lettre/leptonic deps
      // (leptonic's transitive leptos belongs to the lockfile tier, which the
      // fixture lockfile below exercises)
      { path: 'crates/payroll/gui', name: 'console-payroll-gui', dependencies: 'lettre = "0.11"\nleptonic = "0.5"\n' },
    ], { lockfilePackages: ['console-payroll-ui', 'leptos', 'leptos_axum', 'leptose', 'leptonic'] });
    // the old subject is blind here: neither tombstone path exists in this tree
    assert.equal(existsSync(path.join(root, CONSOLE_NAV_SOURCE)), false);
    assert.equal(existsSync(path.join(root, CONSOLE_REGISTRY_SOURCE)), false);
    const facts = extractConsoleWorkspaceFacts(root);
    const text = facts.violations.join('\n');
    assert.equal(facts.violations.length, 7, `expected the 3 name/dep members plus 2 locked packages to yield 7 violations, got:\n${text}`);
    assert.ok(facts.violations.every((violation) => violation.startsWith('ADR-0030 §8 planning-only violation')), text);
    assert.match(text, /'console-payroll-ui' \(backend\/crates\/payroll\/ui\/Cargo\.toml\) carries the §6-chartered console surface-crate name/, 'the chartered -ui name must be named');
    assert.match(text, /'console-payroll-ui' .* declares Leptos-family dependency 'leptos'/, 'the leptos dependency must be named');
    assert.match(text, /'console-payroll-ui' .* declares Leptos-family dependency 'leptos_axum'/, 'the leptos_axum dependency must be named');
    assert.match(text, /'console-payroll-widgets' .* declares Leptos-family dependency 'leptos' \(renamed locally to 'view-framework'\)/, 'a renamed dependency must be reported under its TRUE package name');
    assert.match(text, /'console-hr-ui' .* carries the §6-chartered console surface-crate name/, 'a leptos-free -ui crate must still be named');
    assert.match(text, /backend\/Cargo\.lock resolves Leptos-family package 'leptos'/, 'the locked graph tier must flag leptos');
    assert.match(text, /backend\/Cargo\.lock resolves Leptos-family package 'leptos_axum'/, 'the locked graph tier must flag leptos_axum');
    assert.doesNotMatch(text, /console-payroll-gui|'leptonic'|'leptose'|'lettre'/, 'near-miss names and dependencies must not be flagged');
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('the planning-only scan fails closed on anything it cannot read', () => {
  assert.throws(() => workspaceMemberViolations([]), /examined zero workspace members/, 'zero members must fail, never pass');
  assert.throws(() => workspaceMemberViolations([{ name: 'x' }]), /cannot read/, 'an unreadable member must fail, never be skipped');
  assert.throws(() => workspaceMemberViolations([{ name: 'x', manifest_path: '/x/Cargo.toml', dependencies: [{}] }]), /unreadable dependency/, 'an unreadable dependency must fail, never be skipped');
  assert.throws(() => lockedPackageNames(''), /unreadable must fail/, 'an empty lockfile must fail, never scan clean');
  assert.throws(() => lockedPackageNames('[[package]]\nversion = "1.0.0"\n'), /carries no name/, 'a nameless package block must fail, never be skipped');
  assert.throws(() => extractConsoleWorkspaceFacts(path.join(tmpdir(), 'console-planning-only-absent-root')), /backend\/Cargo\.toml is missing/, 'a missing manifest must fail, never pass');
  const root = mkdtempSync(path.join(tmpdir(), 'console-planning-only-closed-'));
  try {
    writeWorkspaceFixture(root, [{ path: 'crates/hr/domain', name: 'console-hr-domain' }]);
    assert.throws(() => extractConsoleWorkspaceFacts(root), /backend\/Cargo\.lock is missing/, 'a missing lockfile must fail, never pass');
    writeWorkspaceFixture(root, [{ path: 'crates/hr/domain', name: 'console-hr-domain' }], { lockfilePackages: ['console-hr-domain'] });
    assert.deepEqual(extractConsoleWorkspaceFacts(root).violations, [], 'a clean backend-shaped workspace must pass');
    writeFileSync(path.join(root, 'backend', 'Cargo.toml'), '[workspace]\nresolver = "3"\nmembers = ["crates/nonexistent/*"]\n');
    assert.throws(() => extractConsoleWorkspaceFacts(root), /cargo metadata failed/, 'an unresolvable member glob must fail, never pass');
    writeFileSync(path.join(root, 'backend', 'Cargo.toml'), '[workspace]\nresolver = "3"\nmembers = []\n');
    assert.throws(() => extractConsoleWorkspaceFacts(root), /examined zero workspace members/, 'an empty workspace must fail, never pass');
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
