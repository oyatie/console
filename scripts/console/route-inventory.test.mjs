import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { CONSOLE_NAV_SOURCE, CONSOLE_REGISTRY_SOURCE, extractConsoleRouteFacts, extractConsoleRouteFactsFromTexts } from './route-inventory.mjs';

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
