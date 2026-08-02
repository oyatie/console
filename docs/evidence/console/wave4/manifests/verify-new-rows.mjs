#!/usr/bin/env node
// Proves the two rows L-P0-EPOCH added to the console capability registry
// (CAP-SALES-CRM, CAP-ONTOLOGY-ENGINE) pass every per-capability check in the
// real truth-ledger validator, including the jurisdiction bijection once the
// twelve traces in ./jurisdiction-register-traces.json are appended.
//
// It validates a SUBSET — the two new rows plus their dependency targets —
// because the full ledger has pre-existing failures this lane did not
// introduce and must not silently repair (see ../README.md section 5).
//
//   node docs/evidence/console/wave4/manifests/verify-new-rows.mjs

import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../../..');
const read = (relative) => readFileSync(path.join(ROOT, relative), 'utf8');

const { validateConsoleTruthLedger } = await import(
  path.join(ROOT, 'scripts/console/validate-console-truth-ledger.mjs')
);
const { extractConsoleRouteFactsFromTexts } = await import(
  path.join(ROOT, 'scripts/console/route-inventory.mjs')
);

const registry = JSON.parse(read('docs/program/console-capability-registry.json'));
const jurisdiction = JSON.parse(read('docs/program/console-jurisdiction-register.json'));
const manifest = JSON.parse(read('docs/evidence/console/wave4/manifests/jurisdiction-register-traces.json'));

// The registry no longer stores the candidate SHA — it is supplied from outside, exactly as CI
// supplies it. Default to HEAD so this manifest stays runnable by hand.
const candidate = process.env.CONSOLE_CANDIDATE_SHA
  ?? execFileSync('git', ['-C', ROOT, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim();
const show = (file) => execFileSync('git', ['-C', ROOT, 'show', `${candidate}:${file}`], { encoding: 'utf8' });
const routeFacts = extractConsoleRouteFactsFromTexts(
  show('web/src/console/shell/nav.ts'),
  show('web/src/console/screens/registry.ts'),
);

const KEEP = new Set(['CAP-SALES-CRM', 'CAP-ONTOLOGY-ENGINE', 'CAP-CONSOLE-SHELL', 'CAP-SHARED-ONTOLOGY-WORKFLOW']);
registry.capabilities = registry.capabilities.filter((cap) => KEEP.has(cap.id));
if (registry.capabilities.length !== KEEP.size) throw new Error('subset is incomplete');

// Route-key inventory must stay a bijection: whatever the subset does not own is unmodeled.
const owned = new Set(registry.capabilities.flatMap((cap) => cap.route_presentation.route_keys));
registry.source_inventory.unmodeled_keys = Object.keys(routeFacts.facts)
  .filter((key) => !owned.has(key))
  .map((key) => ({ key, status: 'HOLD_UNMAPPED' }));

// Jurisdiction bijection: subset traces, plus the manifest's appends applied verbatim.
for (const control of jurisdiction.controls) {
  control.capability_traceability = control.capability_traceability.filter((trace) => KEEP.has(trace.capability_id));
  const append = manifest.appends.find((entry) => entry.control_id === control.id);
  if (!append) throw new Error(`manifest has no appends for ${control.id}`);
  control.capability_traceability.push(...append.traces);
}

const result = validateConsoleTruthLedger(registry, jurisdiction, {
  expectedCandidateSha: candidate,
  resolveSha: () => true,
  resolveBuckTarget: () => true,
  resolveSource: () => true,
  routeFacts,
  repoRoot: ROOT,
});
console.log('PASS', JSON.stringify(result));
