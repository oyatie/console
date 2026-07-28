import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { extractConsoleRouteFactsFromTexts } from './route-inventory.mjs';

const repoRoot = fileURLToPath(new URL('../../', import.meta.url));
const authorityRegistry = JSON.parse(readFileSync(new URL('../../docs/program/console-capability-registry.json', import.meta.url)));
const candidateFile = (relativePath) => execFileSync(
  'git',
  ['-C', repoRoot, 'show', `${authorityRegistry.candidate.sha}:${relativePath}`],
  { encoding: 'utf8' },
);

function candidateRouteContract() {
  const expected = new Map();
  for (const capability of authorityRegistry.capabilities) {
    for (const key of capability.route_presentation.route_keys) {
      assert.equal(expected.has(key), false, `authority registry duplicates route key ${key}`);
      expected.set(key, Object.fromEntries(
        ['source_mounted', 'production_exposed', 'registry_body_present', 'nav_declared']
          .map((field) => [field, capability.route_presentation[field]]),
      ));
    }
  }
  for (const entry of authorityRegistry.source_inventory.unmodeled_keys) {
    assert.equal(expected.has(entry.key), false, `authority registry overlaps modeled and unmodeled key ${entry.key}`);
    assert.equal(entry.status, 'HOLD_UNMAPPED', `unmodeled key ${entry.key} must remain HOLD_UNMAPPED`);
    expected.set(entry.key, null);
  }
  return expected;
}

function assertCandidateRouteContract(result) {
  const expected = candidateRouteContract();
  assert.deepEqual([...Object.keys(result.facts)].sort(), [...expected.keys()].sort(), 'candidate source inventory must contain neither missing nor extra route keys');
  for (const [key, expectedFacts] of expected) {
    if (expectedFacts) assert.deepEqual(result.facts[key], expectedFacts, `candidate source classification drifted for ${key}`);
  }
  assert.deepEqual(result.exposed, [], 'candidate source must not expose an unverified route');
}

// The 2026-07-28 clean-slate pivot deleted the frontend, so the candidate holds
// no console route sources. `null` means "no frontend in this candidate", which
// is a valid state, not a failure: a console with no frontend presents no routes.
// The Leptos rebuild re-registers its route source here.
function candidateTexts() {
  try {
    return {
      nav: candidateFile('web/src/console/shell/nav.ts'),
      registry: candidateFile('web/src/console/screens/registry.ts'),
    };
  } catch {
    return null;
  }
}

test('route inventory exactly matches the immutable candidate authority contract', () => {
  const texts = candidateTexts();
  if (!texts) {
    // No route sources => the registry must declare no routes, or the two disagree.
    const declared = authorityRegistry.capabilities.flatMap((capability) => capability.route_presentation.route_keys);
    assert.deepEqual(declared, [], 'registry declares route keys but the candidate has no console route source');
    return;
  }
  assertCandidateRouteContract(extractConsoleRouteFactsFromTexts(texts.nav, texts.registry));
});

test('route inventory contract rejects missing, extra, and misclassified candidate routes', (t) => {
  const texts = candidateTexts();
  if (!texts) {
    // This is a negative test over real candidate route sources; it mutates them
    // to prove the contract rejects bad route sets. With the frontend deleted
    // there is nothing to mutate. Skipped loudly rather than passed silently, so
    // the Leptos rebuild restores this coverage instead of quietly losing it.
    t.skip('no console route source in candidate (2026-07-28 clean-slate pivot)');
    return;
  }
  const { nav, registry } = texts;

  const missing = extractConsoleRouteFactsFromTexts(
    nav.replace('  "consulting",\n', ''),
    registry.replace(/^  consulting:.*\n/m, ''),
  );
  assert.throws(() => assertCandidateRouteContract(missing), /missing nor extra route keys/);

  const extra = extractConsoleRouteFactsFromTexts(
    nav.replace('] as const;\n\nexport type MountedScreenKey', '  "unowned",\n] as const;\n\nexport type MountedScreenKey'),
    registry,
  );
  assert.throws(() => assertCandidateRouteContract(extra), /missing nor extra route keys/);

  const misclassified = extractConsoleRouteFactsFromTexts(
    nav.replace('  "overview",\n', ''),
    registry,
  );
  assert.throws(() => assertCandidateRouteContract(misclassified), /classification drifted for overview/);
});
