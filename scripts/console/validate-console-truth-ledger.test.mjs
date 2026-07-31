import assert from 'node:assert/strict';
import { chmodSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
import test from 'node:test';

import { createConsoleBuckTargetResolver, createConsoleCandidateSourceResolver, promotionAuthorityDigests, validateConsoleTruthLedger } from './validate-console-truth-ledger.mjs';
import { verifyCommitWithCandidateSshPolicy } from './ssh-signature-policy.mjs';
import { ABSENT_CONSOLE_ROUTE_FACTS, extractConsoleRouteFactsFromTexts } from './route-inventory.mjs';

const registry = JSON.parse(readFileSync(new URL('../../docs/program/console-capability-registry.json', import.meta.url)));
const jurisdiction = JSON.parse(readFileSync(new URL('../../docs/program/console-jurisdiction-register.json', import.meta.url)));
const repoRoot = fileURLToPath(new URL('../../', import.meta.url));
const ROUTE_CLAIM_FIELDS = ['source_mounted', 'production_exposed', 'registry_body_present', 'nav_declared'];

function withoutRouteClaims(value) {
  const cleared = structuredClone(value);
  for (const capability of cleared.capabilities) {
    capability.route_presentation.route_keys = [];
    for (const field of ROUTE_CLAIM_FIELDS) capability.route_presentation[field] = false;
  }
  if (cleared.source_inventory) cleared.source_inventory.unmodeled_keys = [];
  return cleared;
}

test('current candidate truth ledger is structurally complete but remains candidate-bound HOLD where evidence is absent', () => {
  assert.doesNotThrow(() => validateConsoleTruthLedger(registry, jurisdiction, { expectedCandidateSha: registry.candidate.sha }));
  assert.equal(registry.schema_version, 'console-capability-registry-v2');
  assert.match(registry.candidate.sha, /^[0-9a-f]{40}$/);
  assert.ok(registry.capabilities.some((capability) => capability.id === 'CAP-ASSET-MASTER-ACTION'));
  assert.ok(registry.capabilities.every((capability) => capability.benchmark?.verdict === 'HOLD'));
  assert.ok(registry.capabilities.every((capability) => capability.benchmark?.native_outcomes?.length >= 3));
  assert.ok(registry.capabilities.every((capability) => capability.candidate_evidence?.candidate_sha === registry.candidate.sha));
  assert.ok(jurisdiction.controls.every((control) => control.release_disposition === 'HOLD'));
});

test('validator fails closed for duplicate IDs, unbound evidence, missing module benchmark, and unsafe exposure', () => {
  const duplicate = structuredClone(registry);
  duplicate.capabilities.push(structuredClone(duplicate.capabilities[0]));
  assert.throws(() => validateConsoleTruthLedger(duplicate, jurisdiction, { expectedCandidateSha: registry.candidate.sha }), /duplicate capability id/);

  const unbound = structuredClone(registry);
  unbound.capabilities[0].candidate_evidence.candidate_sha = 'a'.repeat(40);
  assert.throws(() => validateConsoleTruthLedger(unbound, jurisdiction, { expectedCandidateSha: registry.candidate.sha }), /candidate-bound/);

  const noBenchmark = structuredClone(registry);
  delete noBenchmark.capabilities[0].benchmark;
  assert.throws(() => validateConsoleTruthLedger(noBenchmark, jurisdiction, { expectedCandidateSha: registry.candidate.sha }), /per-module benchmark/);

  const noOutcomes = structuredClone(registry);
  noOutcomes.capabilities[0].benchmark.native_outcomes = [];
  assert.throws(() => validateConsoleTruthLedger(noOutcomes, jurisdiction, { expectedCandidateSha: registry.candidate.sha }), /3-7 measurable native outcomes/);

  const unsafeExposure = structuredClone(registry);
  unsafeExposure.capabilities[0].route_presentation = { route_keys: [], source_mounted: false, production_exposed: true, registry_body_present: false, nav_declared: true, evidence_receipt_status: 'HOLD', source: 'test' };
  assert.throws(() => validateConsoleTruthLedger(unsafeExposure, jurisdiction, { expectedCandidateSha: registry.candidate.sha }), /exposed route must be mounted/);
});

test('validator rejects generic outcomes, duplicate controls, legal promotion, and stale external candidate binding', () => {
  const generic = structuredClone(registry);
  generic.capabilities[1].benchmark.native_outcomes = structuredClone(generic.capabilities[0].benchmark.native_outcomes);
  assert.throws(() => validateConsoleTruthLedger(generic, jurisdiction, { expectedCandidateSha: registry.candidate.sha }), /duplicate benchmark outcome/);
  const duplicateControl = structuredClone(jurisdiction);
  duplicateControl.controls.push(structuredClone(duplicateControl.controls[0]));
  assert.throws(() => validateConsoleTruthLedger(registry, duplicateControl, { expectedCandidateSha: registry.candidate.sha }), /duplicate control id/);
  const promoted = structuredClone(jurisdiction);
  promoted.controls[0].release_disposition = 'MEET';
  assert.throws(() => validateConsoleTruthLedger(registry, promoted, { expectedCandidateSha: registry.candidate.sha }), /must remain HOLD/);
  assert.throws(() => validateConsoleTruthLedger(registry, jurisdiction, { expectedCandidateSha: 'a'.repeat(40) }), /expected candidate/);
});

test('raw ledger JSON rejects duplicate keys before JSON.parse can hide them', async () => {
  const { parseImmutableJson } = await import('./immutable-json.mjs');
  for (const raw of ['{"schema_version":"a","schema_version":"b"}', '{"controls":[{"id":"x","id":"y"}]}']) assert.throws(() => parseImmutableJson(raw, 'truth ledger'), /duplicate JSON key/);
});

test('source route facts reject a ledger claim that disagrees with mounted/exposed source', () => {
  // Fixture rather than candidate source: the 2026-07-28 pivot deleted the
  // frontend, so reading the candidate would exercise nothing. The rule under
  // test is the per-key comparison, which must reject a claim that contradicts
  // whatever route source exists.
  const routeFacts = extractConsoleRouteFactsFromTexts(
    'export const MOUNTED_SCREEN_KEYS = ["overview"] as const;\nexport const EXPOSED_SCREEN_KEYS = [] as const;\nconst nav = [{ screen: "overview" }];\n',
    'export const SCREEN_REGISTRY = {\n  overview: () => null,\n};\n',
  );
  const truthful = withoutRouteClaims(registry);
  Object.assign(truthful.capabilities[0].route_presentation, { route_keys: ['overview'], source_mounted: true, registry_body_present: true, nav_declared: true });
  assert.doesNotThrow(() => validateConsoleTruthLedger(truthful, jurisdiction, { expectedCandidateSha: registry.candidate.sha, routeFacts }));
  const bad = structuredClone(truthful);
  bad.capabilities[0].route_presentation.source_mounted = false;
  assert.throws(() => validateConsoleTruthLedger(bad, jurisdiction, { expectedCandidateSha: registry.candidate.sha, routeFacts }), /route source fact mismatch/);
  const unowned = structuredClone(truthful);
  unowned.capabilities[0].route_presentation.route_keys = [];
  unowned.capabilities[0].route_presentation.source_mounted = false;
  unowned.capabilities[0].route_presentation.registry_body_present = false;
  unowned.capabilities[0].route_presentation.nav_declared = false;
  assert.throws(() => validateConsoleTruthLedger(unowned, jurisdiction, { expectedCandidateSha: registry.candidate.sha, routeFacts }), /complete bijection/);
});

test('an absent route source refutes every positive route claim instead of iterating zero keys', () => {
  // With no route source the per-key loop and the bijection both see zero keys,
  // so they corroborate nothing. The boolean claims must be judged on their own.
  const truthful = withoutRouteClaims(registry);
  assert.doesNotThrow(() => validateConsoleTruthLedger(truthful, jurisdiction, { expectedCandidateSha: registry.candidate.sha, routeFacts: ABSENT_CONSOLE_ROUTE_FACTS }));
  for (const field of ROUTE_CLAIM_FIELDS) {
    const claimed = withoutRouteClaims(registry);
    claimed.capabilities[0].route_presentation[field] = true;
    if (field === 'production_exposed') { claimed.capabilities[0].route_presentation.source_mounted = true; claimed.capabilities[0].truth.exposure = 'EXPOSED'; claimed.capabilities[0].truth.verification = 'VERIFIED'; }
    assert.throws(
      () => validateConsoleTruthLedger(claimed, jurisdiction, { expectedCandidateSha: registry.candidate.sha, routeFacts: ABSENT_CONSOLE_ROUTE_FACTS }),
      /no console route source to corroborate it/,
      `${field} claim survived an absent route source`,
    );
  }
  // A fact object that forgets to declare its provenance is treated as absent,
  // never as corroboration.
  const claimed = withoutRouteClaims(registry);
  claimed.capabilities[0].route_presentation.nav_declared = true;
  assert.throws(() => validateConsoleTruthLedger(claimed, jurisdiction, { expectedCandidateSha: registry.candidate.sha, routeFacts: { facts: {} } }), /no console route source to corroborate it/);
});


test('forged comparator source/date and attacker review cannot pass', () => {
  const forged = structuredClone(registry); forged.capabilities[0].benchmark.comparator_sources[0].observation_as_of = '2026-99-99';
  assert.throws(() => validateConsoleTruthLedger(forged, jurisdiction, { expectedCandidateSha: registry.candidate.sha, resolveSource: () => true }), /ISO/);
  const source = structuredClone(registry);
  assert.throws(() => validateConsoleTruthLedger(source, jurisdiction, { expectedCandidateSha: registry.candidate.sha, resolveSource: () => false }), /tracked regular file/);
  const review = structuredClone(registry); review.capabilities[0].benchmark.independent_outcome_review = { status: 'MEET', candidate_sha: registry.candidate.sha, receipt_path: 'forged.json', reviewer_id: review.capabilities[0].owner };
  assert.throws(() => validateConsoleTruthLedger(review, jurisdiction, { expectedCandidateSha: registry.candidate.sha }), /receipt schema/);
});

test('required Buck target fails closed when resolver rejects it', () => {
  const bad = structuredClone(registry); const equipment = bad.capabilities.find((capability) => capability.id === 'CAP-EQUIPMENT-3R-PILOT');
  assert.throws(() => validateConsoleTruthLedger(bad, jurisdiction, { expectedCandidateSha: registry.candidate.sha, resolveBuckTarget: () => false }), /invalid\/nonexistent Buck target/);
});

test('Buck target existence is read from the candidate BUCK blob, never evaluated by Buck2', () => {
  const files = {
    '.buckconfig': '[cells]\n  root = .\n  prelude = prelude\n  toolchains = toolchains\n\n[project]\n  ignore = .git\n',
    'backend/crates/x/BUCK': 'rust_test(\n    name = "x-unit",\n    srcs = ["lib.rs"],\n)\n#    name = "ghost",\n    labels = ["name = \\"phantom\\""],\n',
    'toolchains/BUCK': 'toolchain(\n    name = "rust",\n)\n',
  };
  const resolve = createConsoleBuckTargetResolver({
    readText: (file) => { if (!(file in files)) throw new Error(`candidate source is missing: ${file}`); return files[file]; },
    resolveSource: (file) => (file.includes('..') ? false : file in files && { tracked_regular: true }),
  });
  assert.equal(resolve('//backend/crates/x:x-unit'), true);
  assert.equal(resolve('//backend/crates/x:ghost'), false, 'a commented-out declaration is not a target');
  assert.equal(resolve('//backend/crates/x:phantom'), false, 'a declaration lookalike inside a string is not a target');
  assert.equal(resolve('//backend/crates/x:x-uni'), false, 'a prefix of a real name is not a target');
  assert.equal(resolve('//backend/crates/gone:x-unit'), false, 'a package with no BUCK file fails closed');
  assert.equal(resolve('//toolchains:rust'), false, 'a nested cell is not a root-cell package');
  assert.equal(resolve('//../../etc:passwd'), false, 'a traversing label fails closed');
});

test('the real registry resolves against the real tree, and a nonexistent Buck target goes RED', () => {
  // Exercises the shipped resolver, not a stub: the same one main() hands to the
  // validator, so registry/tree drift fails here and not only in CI.
  const resolveBuckTarget = createConsoleBuckTargetResolver({
    readText: (file) => readFileSync(path.join(repoRoot, file), 'utf8'),
    resolveSource: (file) => (file.includes('..') || file.startsWith('/') ? false : existsSync(path.join(repoRoot, file)) && { tracked_regular: true }),
  });
  const declared = registry.capabilities.flatMap((capability) => capability.delivery_unit?.buck2_targets ?? []);
  assert.ok(declared.length > 0, 'the registry must declare at least one Buck target for this to assert anything');
  for (const target of declared) assert.equal(resolveBuckTarget(target), true, `declared Buck target does not exist in the tree: ${target}`);
  assert.doesNotThrow(() => validateConsoleTruthLedger(registry, jurisdiction, { expectedCandidateSha: registry.candidate.sha, resolveBuckTarget }));

  for (const mutate of [
    (capability) => { capability.delivery_unit.buck2_targets[0] = '//backend/crates/equipment/domain:no-such-target'; },
    (capability) => { capability.delivery_unit.buck2_targets[0] = '//backend/crates/nonexistent/pkg:console-equipment-domain-unit'; },
  ]) {
    const bad = structuredClone(registry);
    const capability = bad.capabilities.find((entry) => entry.id === 'CAP-EQUIPMENT-3R-PILOT');
    mutate(capability);
    capability.tests.buck2_targets = structuredClone(capability.delivery_unit.buck2_targets);
    assert.throws(() => validateConsoleTruthLedger(bad, jurisdiction, { expectedCandidateSha: registry.candidate.sha, resolveBuckTarget }), /invalid\/nonexistent Buck target/);
  }
});

test('delivery-unit Buck authority cannot diverge from declared verification targets', () => {
  const bad = structuredClone(registry);
  const equipment = bad.capabilities.find((capability) => capability.id === 'CAP-EQUIPMENT-3R-PILOT');
  equipment.delivery_unit.buck2_targets = [
    '//backend/crates/equipment/domain:console-equipment-domain-unit',
    '//backend/crates/equipment/rest:console-equipment-rest',
    '//backend/app:console-app-itest-equipment_3r_api',
  ];
  assert.throws(
    () => validateConsoleTruthLedger(bad, jurisdiction, { expectedCandidateSha: registry.candidate.sha }),
    /delivery Buck targets must match declared verification targets/,
  );
});

test('attestation rejects TOCTOU mutation after candidate-bound validation', async () => {
  const { isValidatedConsoleTruthLedger } = await import('./validate-console-truth-ledger.mjs');
  // Route facts are deliberately omitted: this attests digest staleness, not
  // route binding, and the candidate holds no route source to bind against.
  // Route-fact binding is covered by the two route tests above.
  const value = structuredClone(registry);
  validateConsoleTruthLedger(value, jurisdiction, { expectedCandidateSha: registry.candidate.sha });
  assert.equal(isValidatedConsoleTruthLedger(value), true);
  value.capabilities[0].truth.exposure = 'EXPOSED';
  assert.equal(isValidatedConsoleTruthLedger(value), false);
});

test('moving candidate branch names are irrelevant and jurisdiction target bypasses reject', () => {
  const branch = structuredClone(registry);
  branch.candidate.branch = 'attacker/moving-pr-branch';
  assert.doesNotThrow(() => validateConsoleTruthLedger(branch, jurisdiction, { expectedCandidateSha: registry.candidate.sha, resolveBranch: () => 'a'.repeat(40) }));
  const empty = structuredClone(jurisdiction); empty.target_jurisdiction_set = [];
  assert.throws(() => validateConsoleTruthLedger(registry, empty, { expectedCandidateSha: registry.candidate.sha }), /jurisdiction target/);
});

test('non-HOLD promotion requires canonical trusted immutable receipt fields', () => {
  const promoted = structuredClone(registry); const cap = promoted.capabilities[0];
  cap.benchmark.verdict = 'MEET'; cap.candidate_evidence.status = 'VERIFIED';
  cap.benchmark.independent_outcome_review = { status: 'MEET', reviewer_id: 'attacker', candidate_sha: registry.candidate.sha, capability_id: cap.id, outcome_ids: [cap.benchmark.native_outcomes[0].id], evidence_digest: 'a'.repeat(64), review_commit: 'b'.repeat(40), receipt_path: 'docs/evidence/fake.json' };
  assert.throws(() => validateConsoleTruthLedger(promoted, jurisdiction, { expectedCandidateSha: registry.candidate.sha, resolveReceipt: () => true }), /receipt schema/);
  const trusted = structuredClone(registry); const c = trusted.capabilities[0]; trusted.candidate_evidence = undefined; c.benchmark.verdict='MEET'; c.candidate_evidence.status='VERIFIED'; c.benchmark.independent_outcome_review={status:'MEET',reviewer_id:'jasonlee-ssh-reviewer',candidate_sha:registry.candidate.sha,capability_id:c.id,outcome_ids:[c.benchmark.native_outcomes[0].id],evidence_digest:'a'.repeat(64),review_commit:'b'.repeat(40),receipt_sha256:'c'.repeat(64),receipt_canonical_sha256:'d'.repeat(64),registry_canonical_sha256:'e'.repeat(64),jurisdiction_canonical_sha256:'f'.repeat(64),receipt_path:`docs/evidence/console/reviews/${c.id}/${registry.candidate.sha}.json`};
  assert.throws(() => validateConsoleTruthLedger(trusted, jurisdiction, { expectedCandidateSha: registry.candidate.sha,  }), /canonical repository root/);
});

test('Korea trace bijection rejects missing trace', () => {
  const bad=structuredClone(jurisdiction); bad.controls[0].capability_traceability.pop();
  assert.throws(() => validateConsoleTruthLedger(registry,bad,{expectedCandidateSha:registry.candidate.sha}),/bidirectional|bijection/);
});


test('Korea exactness rejects duplicate bindings and duplicate targets', () => {
  const duplicateTarget = structuredClone(jurisdiction); duplicateTarget.target_jurisdiction_set = ['KR', 'KR'];
  assert.throws(() => validateConsoleTruthLedger(registry, duplicateTarget, { expectedCandidateSha: registry.candidate.sha }), /jurisdiction target/);
  const duplicateBinding = structuredClone(registry); const cap = duplicateBinding.capabilities[0]; cap.jurisdiction_bindings.push(structuredClone(cap.jurisdiction_bindings[0]));
  assert.throws(() => validateConsoleTruthLedger(duplicateBinding, jurisdiction, { expectedCandidateSha: registry.candidate.sha }), /duplicate jurisdiction binding/);
});


test('a real SSH-signed canonical Git receipt is the only non-HOLD admission path', () => {
  const root = mkdtempSync(path.join(tmpdir(), 'console-receipt-'));
  const run = (args) => execFileSync('git', ['-C', root, ...args], { encoding: 'utf8' }).trim();
  try {
    run(['init']); run(['config', 'user.name', 'Jason Lee']); run(['config', 'user.email', 'jason19931225@gmail.com']);
    const signingKey = path.join(root, 'review_key'); execFileSync('ssh-keygen', ['-q', '-t', 'ed25519', '-N', '', '-f', signingKey]);
    run(['config', 'gpg.format', 'ssh']); run(['config', 'user.signingkey', signingKey]);
    const allowed = path.join(root, 'allowed_signers'); writeFileSync(allowed, 'jason19931225@gmail.com ' + readFileSync(`${signingKey}.pub`, 'utf8'));
    run(['config', 'gpg.ssh.allowedSignersFile', allowed]);
    const fingerprint = execFileSync('ssh-keygen', ['-lf', `${signingKey}.pub`, '-E', 'sha256'], { encoding: 'utf8' }).trim().split(/\s+/)[1];
    const candidateSigningAuthority = { format: 'ssh', principal: 'jason19931225@gmail.com', fingerprint };
    const publicKey = readFileSync(`${signingKey}.pub`, 'utf8').trim().split(/\s+/).slice(0, 2).join(' ');
    mkdirSync(path.join(root, 'scripts/console'), { recursive: true }); mkdirSync(path.join(root, '.github/trust'), { recursive: true }); mkdirSync(path.join(root, 'docs/program'), { recursive: true }); writeFileSync(path.join(root, 'scripts/console/control.mjs'), 'export const control = true;\n'); writeFileSync(path.join(root, 'candidate.txt'), 'candidate\n'); writeFileSync(path.join(root, '.github/trust/console.allowed_signers'), `jason19931225@gmail.com ${publicKey}\n`); for (const file of ['console-capability-registry.json', 'console-jurisdiction-register.json', 'console-program-ledger.md']) writeFileSync(path.join(root, 'docs/program', file), 'candidate\n'); run(['add', '.']); run(['commit', '-S', '-m', 'candidate']);
    const candidateSha = run(['rev-parse', 'HEAD']);
    for (const file of ['console-capability-registry.json', 'console-jurisdiction-register.json', 'console-program-ledger.md']) writeFileSync(path.join(root, 'docs/program', file), 'authority\n'); run(['add', '.']); run(['commit', '-S', '-m', 'authority control']);
    const authorityTip = run(['rev-parse', 'HEAD']);
    run(['config', '--unset', 'gpg.ssh.allowedSignersFile']);
    assert.equal(createConsoleCandidateSourceResolver(root, candidateSha, authorityTip, { candidateSigningAuthority }).readText('candidate.txt'), 'candidate\n');
    // A union merge resolution that keeps both entries but leaves the marker line behind is
    // otherwise invisible: the tip is correctly signed, is the candidate's only child, and
    // modifies exactly the three authority documents. Nine such lines reached main. Each of
    // the three asymmetric markers must fail on its own — a resolution that stripped two of
    // three is exactly what happened, so testing only `<<<<<<<` would have passed on the
    // real defect.
    for (const marker of ['<<<<<<< ours', '||||||| 18a21d7cd', '>>>>>>> theirs']) {
      run(['checkout', '-B', `marker-${marker.slice(0, 3)}`, candidateSha]);
      for (const file of ['console-capability-registry.json', 'console-jurisdiction-register.json', 'console-program-ledger.md']) writeFileSync(path.join(root, 'docs/program', file), 'authority\n');
      writeFileSync(path.join(root, 'docs/program/console-program-ledger.md'), `authority\n${marker}\nboth entries kept\n`);
      run(['add', '.']); run(['commit', '-S', '-m', 'authority control with unresolved marker']);
      assert.throws(
        () => createConsoleCandidateSourceResolver(root, candidateSha, run(['rev-parse', 'HEAD']), { candidateSigningAuthority }),
        /unresolved merge marker/,
        `a tip whose ledger line starts with ${marker.split(' ')[0]} must be refused`,
      );
    }
    // `=======` is a Markdown setext heading rule as well as a conflict marker. Failing on it
    // would fail the ledger on ordinary prose, so it is not a marker here — and that
    // exemption has to stay proven, not just commented.
    run(['checkout', '-B', 'setext-authority', candidateSha]);
    for (const file of ['console-capability-registry.json', 'console-jurisdiction-register.json', 'console-program-ledger.md']) writeFileSync(path.join(root, 'docs/program', file), 'authority\n');
    writeFileSync(path.join(root, 'docs/program/console-program-ledger.md'), 'A heading\n=======\n');
    run(['add', '.']); run(['commit', '-S', '-m', 'authority control with setext heading']);
    assert.doesNotThrow(() => createConsoleCandidateSourceResolver(root, candidateSha, run(['rev-parse', 'HEAD']), { candidateSigningAuthority }));
    run(['checkout', authorityTip]);
    const hostileHome = mkdtempSync(path.join(tmpdir(), 'console-hostile-home-'));
    const hostileVerifier = path.join(hostileHome, 'forged-ssh-keygen'); const hostileMarker = path.join(hostileHome, 'invoked');
    writeFileSync(hostileVerifier, `#!/bin/sh\ntouch '${hostileMarker}'\nprintf '%s\\n' 'Good "git" signature for ${candidateSigningAuthority.principal} with ED25519 key ${fingerprint}'\n`); chmodSync(hostileVerifier, 0o700);
    writeFileSync(path.join(hostileHome, '.gitconfig'), `[gpg "ssh"]\n\tprogram = ${hostileVerifier}\n`);
    const originalHome = process.env.HOME; process.env.HOME = hostileHome;
    try { assert.doesNotThrow(() => verifyCommitWithCandidateSshPolicy(root, candidateSha, authorityTip, candidateSigningAuthority)); assert.equal(existsSync(hostileMarker), false); } finally { process.env.HOME = originalHome; rmSync(hostileHome, { recursive: true, force: true }); }
    run(['checkout', '-b', 'renamed-authority', authorityTip]);
    run(['mv', 'scripts/console/control.mjs', 'scripts/console/renamed-control.mjs']); run(['commit', '--no-gpg-sign', '-m', 'rename authority control']);
    assert.throws(() => createConsoleCandidateSourceResolver(root, candidateSha, run(['rev-parse', 'HEAD']), { candidateSigningAuthority }), /integration tip commit signature|direct single-parent child/);
    run(['checkout', '-B', 'symlink-authority', authorityTip]);
    rmSync(path.join(root, 'scripts/console/control.mjs')); symlinkSync('target.mjs', path.join(root, 'scripts/console/control.mjs')); run(['add', '-A']); run(['commit', '--no-gpg-sign', '-m', 'symlink authority control']);
    assert.throws(() => createConsoleCandidateSourceResolver(root, candidateSha, run(['rev-parse', 'HEAD']), { candidateSigningAuthority }), /integration tip commit signature|direct single-parent child/);
    run(['checkout', '-B', 'product-drift', authorityTip]);
    writeFileSync(path.join(root, 'product.txt'), 'forbidden\n'); run(['add', '.']); run(['commit', '--no-gpg-sign', '-m', 'product drift']);
    const productTip = run(['rev-parse', 'HEAD']);
    assert.throws(() => createConsoleCandidateSourceResolver(root, candidateSha, productTip, { candidateSigningAuthority }), /integration tip commit signature|direct single-parent child/);
    assert.throws(() => createConsoleCandidateSourceResolver(root, productTip, productTip, { candidateSigningAuthority }), /candidate commit signature/);
    const orphanTip = run(['commit-tree', run(['write-tree']), '-m', 'unrelated authority']);
    assert.throws(() => createConsoleCandidateSourceResolver(root, candidateSha, orphanTip, { candidateSigningAuthority }), /integration tip commit signature|direct single-parent child/);
    const promoted = structuredClone(registry); const fixtureJurisdiction = structuredClone(jurisdiction); const cap = promoted.capabilities[0];
    promoted.candidate.sha = candidateSha; promoted.provenance.authority_base_sha = 'a'.repeat(40); promoted.provenance.historical_implementation_freeze_sha = 'b'.repeat(40);
    for (const capability of promoted.capabilities) { capability.candidate_evidence.candidate_sha = candidateSha; for (const binding of capability.jurisdiction_bindings) binding.candidate_sha = candidateSha; }
    for (const control of fixtureJurisdiction.controls) { control.candidate_evidence.candidate_sha = candidateSha; for (const trace of control.capability_traceability) trace.candidate_sha = candidateSha; }
    cap.candidate_evidence.candidate_sha = candidateSha; cap.candidate_evidence.status = 'VERIFIED'; cap.benchmark.verdict = 'MEET';
    const receiptPath = `docs/evidence/console/reviews/${cap.id}/${candidateSha}.json`; mkdirSync(path.dirname(path.join(root, receiptPath)), { recursive: true });
    Object.assign(promoted.review_authority.reviewers[0], { author_name: 'Jason Lee', author_email: 'jason19931225@gmail.com', committer_name: 'Jason Lee', committer_email: 'jason19931225@gmail.com', signing: { format: 'ssh', principal: 'jason19931225@gmail.com', fingerprint } });
    const payload = { candidate_sha: candidateSha, capability_id: cap.id, outcome_ids: [cap.benchmark.native_outcomes[0].id], evidence_digest: 'a'.repeat(64), verdict: 'MEET', reviewer_id: 'jasonlee-ssh-reviewer' };
    const authorityDigests = promotionAuthorityDigests(promoted, fixtureJurisdiction);
    payload.registry_canonical_sha256 = authorityDigests.registry;
    payload.jurisdiction_canonical_sha256 = authorityDigests.jurisdiction;
    const raw = `${JSON.stringify(payload)}\n`; writeFileSync(path.join(root, receiptPath), raw); run(['add', receiptPath]); run(['commit', '-S', '-m', 'review receipt']);
    const reviewCommit = run(['rev-parse', 'HEAD']);
    const [authorName, authorEmail, committerName, committerEmail] = run(['show', '-s', '--format=%an%x00%ae%x00%cn%x00%ce', reviewCommit]).split('\0');
    assert.deepEqual({ authorName, authorEmail, committerName, committerEmail }, { authorName: 'Jason Lee', authorEmail: 'jason19931225@gmail.com', committerName: 'Jason Lee', committerEmail: 'jason19931225@gmail.com' });
    const canonicalDigest = createHash('sha256').update(JSON.stringify(Object.fromEntries(Object.entries(payload).sort(([a,], [b,]) => a.localeCompare(b, 'en'))))).digest('hex');
    cap.benchmark.independent_outcome_review = { status: 'MEET', reviewer_id: payload.reviewer_id, candidate_sha: candidateSha, capability_id: cap.id, outcome_ids: payload.outcome_ids, evidence_digest: payload.evidence_digest, review_commit: reviewCommit, receipt_path: receiptPath, receipt_sha256: createHash('sha256').update(raw).digest('hex'), receipt_canonical_sha256: canonicalDigest, registry_canonical_sha256: payload.registry_canonical_sha256, jurisdiction_canonical_sha256: payload.jurisdiction_canonical_sha256 };
    assert.doesNotThrow(() => validateConsoleTruthLedger(promoted, fixtureJurisdiction, { expectedCandidateSha: candidateSha, repoRoot: root }));
    const forged = structuredClone(promoted); forged.capabilities[0].benchmark.independent_outcome_review.receipt_path = 'package.json';
    assert.throws(() => validateConsoleTruthLedger(forged, fixtureJurisdiction, { expectedCandidateSha: candidateSha, repoRoot: root }), /not canonical/);
    const wrongDigest = structuredClone(promoted); wrongDigest.capabilities[0].benchmark.independent_outcome_review.receipt_sha256 = '0'.repeat(64);
    assert.throws(() => validateConsoleTruthLedger(wrongDigest, fixtureJurisdiction, { expectedCandidateSha: candidateSha, repoRoot: root }), /digest/);
    const wrongAuthorityDigest = structuredClone(promoted); wrongAuthorityDigest.capabilities[0].benchmark.independent_outcome_review.registry_canonical_sha256 = '0'.repeat(64);
    assert.throws(() => validateConsoleTruthLedger(wrongAuthorityDigest, fixtureJurisdiction, { expectedCandidateSha: candidateSha, repoRoot: root }), /candidate and authority digests/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('candidate resolver requires the signed authority tip to be the direct three-document child of C', () => {
  const root = mkdtempSync(path.join(tmpdir(), 'console-authority-tip-'));
  const run = (args) => execFileSync('git', ['-C', root, ...args], { encoding: 'utf8' }).trim();
  try {
    run(['init']); run(['config', 'user.name', 'Jason Lee']); run(['config', 'user.email', 'jason19931225@gmail.com']);
    const signingKey = path.join(root, 'key'); execFileSync('ssh-keygen', ['-q', '-t', 'ed25519', '-N', '', '-f', signingKey]);
    run(['config', 'gpg.format', 'ssh']); run(['config', 'user.signingkey', signingKey]);
    const publicKey = readFileSync(`${signingKey}.pub`, 'utf8').trim().split(/\s+/).slice(0, 2).join(' ');
    const fingerprint = execFileSync('ssh-keygen', ['-lf', `${signingKey}.pub`, '-E', 'sha256'], { encoding: 'utf8' }).trim().split(/\s+/)[1];
    const authority = { format: 'ssh', principal: 'jason19931225@gmail.com', fingerprint };
    mkdirSync(path.join(root, '.github/trust'), { recursive: true });
    mkdirSync(path.join(root, 'docs/program'), { recursive: true });
    writeFileSync(path.join(root, '.github/trust/console.allowed_signers'), `jason19931225@gmail.com ${publicKey}\n`);
    for (const file of ['console-capability-registry.json', 'console-jurisdiction-register.json', 'console-program-ledger.md']) writeFileSync(path.join(root, 'docs/program', file), 'C\n');
    writeFileSync(path.join(root, 'candidate.txt'), 'C\n'); run(['add', '.']); run(['commit', '-S', '-m', 'C']);
    const candidate = run(['rev-parse', 'HEAD']);
    for (const file of ['console-capability-registry.json', 'console-jurisdiction-register.json', 'console-program-ledger.md']) writeFileSync(path.join(root, 'docs/program', file), 'T\n');
    run(['add', '.']); run(['commit', '-S', '-m', 'T']); const tip = run(['rev-parse', 'HEAD']);
    assert.doesNotThrow(() => createConsoleCandidateSourceResolver(root, candidate, tip, { candidateSigningAuthority: authority }));
    run(['commit', '--allow-empty', '--no-gpg-sign', '-m', 'unsigned extra']);
    assert.throws(() => createConsoleCandidateSourceResolver(root, candidate, run(['rev-parse', 'HEAD']), { candidateSigningAuthority: authority }), /integration tip commit signature|direct single-parent child/);
    run(['checkout', '-B', 'indirect', tip]); writeFileSync(path.join(root, 'docs/program/console-program-ledger.md'), 'extra\n'); run(['add', '.']); run(['commit', '-S', '-m', 'indirect']);
    assert.throws(() => createConsoleCandidateSourceResolver(root, candidate, run(['rev-parse', 'HEAD']), { candidateSigningAuthority: authority }), /direct single-parent child/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('Korea jurisdiction rows require exactly one canonical JUR-KR-001 row', () => {
  const empty = structuredClone(jurisdiction); empty.jurisdictions = [];
  assert.throws(() => validateConsoleTruthLedger(registry, empty, { expectedCandidateSha: registry.candidate.sha }), /jurisdiction target/);
  const duplicate = structuredClone(jurisdiction); duplicate.jurisdictions.push(structuredClone(duplicate.jurisdictions[0]));
  assert.throws(() => validateConsoleTruthLedger(registry, duplicate, { expectedCandidateSha: registry.candidate.sha }), /jurisdiction target/);
});
