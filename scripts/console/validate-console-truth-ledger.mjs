#!/usr/bin/env node

import { createHash } from 'node:crypto';
import { execFileSync, spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseImmutableJson } from './immutable-json.mjs';
import { ABSENT_CONSOLE_ROUTE_FACTS, CONSOLE_NAV_SOURCE, CONSOLE_REGISTRY_SOURCE, extractConsoleRouteFactsFromTexts } from './route-inventory.mjs';
import { CONSOLE_CANDIDATE_SIGNING_AUTHORITY, sshSignatureMatchesAuthority, verifyCommitWithCandidateSshPolicy } from './ssh-signature-policy.mjs';
import { verifyConsoleAuthorityTrain } from './verify-console-authority-train.mjs';
import { AUTHORITY_DIFF_ARGS, LEDGER_DIRECTORY, isLedgerEntryPath } from './authority-ledger-path.mjs';
import { RELEASE_PLEASE_TRAIN_CLASS } from './release-please-bot-candidate.mjs';

const SHA = /^[0-9a-f]{40}$/;
const SHA256 = /^[0-9a-f]{64}$/;
const TAG_REF = /^refs\/tags\/[A-Za-z0-9][A-Za-z0-9._/-]*$/;
const BUCK_TARGET = /^\/\/([A-Za-z0-9_./-]+):([A-Za-z0-9_.-]+)$/;
const STATES = new Set(['DECLARED', 'PLANNED', 'IMPLEMENTED', 'VERIFIED', 'EXPOSED', 'HOLD']);
const VERDICTS = new Set(['MEET', 'EXCEED', 'HOLD']);
const EDGE_TYPES = new Set(['requires', 'blocks', 'integrates_with', 'validates']);
const validatedRegistries = new WeakMap();
const immutableReceiptAttestations = new WeakSet();
function stable(value) { if (Array.isArray(value)) return value.map(stable); if (value && typeof value === 'object') return Object.fromEntries(Object.entries(value).sort(([a],[b]) => a.localeCompare(b, 'en')).map(([k,v]) => [k,stable(v)])); return value; }
function ledgerDigest(value) { return createHash('sha256').update(JSON.stringify(stable(value))).digest('hex'); }
const RESOURCE_KEYS = ['writer', 'postgres', 'browser', 'ios', 'graph', 'cas'];
const ROUTE_CLAIM_FIELDS = ['source_mounted', 'production_exposed', 'registry_body_present', 'nav_declared'];
const CONTINUATION_STATE_FIELDS = ['design_contract', 'backend', 'frontend', 'e2e', 'runtime', 'independent_review', 'production_exposure'];
const HISTORICAL_CONTINUATION_FIELDS = ['historical_worktree', 'historical_branch', 'historical_lane_assignments', 'historical_state', 'historical_reset_state'];
const CONTINUATION_SNAPSHOT_SHA256 = '65edb195ff699e4afc6d67b7f953deddaf6ea0ec83f251c27e05273c7214fbe4';
const AUTHORITY_CONTROL_PATHS = new Set([
  'docs/program/console-capability-registry.json',
  'docs/program/console-jurisdiction-register.json',
  'docs/program/console-program-ledger.md',
]);

function fail(message) { throw new Error(message); }
function array(value) { return Array.isArray(value) ? value : []; }
function object(value, label) { if (!value || typeof value !== 'object' || Array.isArray(value)) fail(`${label} must be an object`); return value; }
function nonempty(value, label) { if (typeof value !== 'string' || value.trim() === '') fail(`${label} must be a non-empty string`); return value; }
function sha(value, label) { if (!SHA.test(value ?? '')) fail(`${label} must be a full lowercase Git SHA`); return value; }
function tagRef(value, label) {
  nonempty(value, label);
  if (!TAG_REF.test(value) || value.includes('..') || value.includes('//') || value.includes('@{') || value.endsWith('/') || value.endsWith('.lock')) fail(`${label} must be a canonical refs/tags/ name`);
  return value;
}
function uniqueStrings(values, label) { const seen = new Set(); for (const value of values) { nonempty(value, label); if (seen.has(value)) fail(`duplicate ${label}: ${value}`); seen.add(value); } return seen; }

function canonicalReceiptPath(capabilityId, candidateSha) { return `docs/evidence/console/reviews/${capabilityId}/${candidateSha}.json`; }
function git(root, args, encoding = 'utf8') { return execFileSync('git', ['-C', root, ...args], { encoding, maxBuffer: 16 * 1024 * 1024, stdio: ['ignore', 'pipe', 'pipe'] }); }
function gitSucceeds(root, args) { try { git(root, args); return true; } catch { return false; } }
function repositoryPath(value) { return typeof value === 'string' && value !== '' && !value.includes('..') && !value.startsWith('/') && !value.includes('\\'); }
// One file per new ledger entry, so two lanes never write the same bytes. Status `A` is
// accepted for this prefix and NOWHERE else: the two registers and the legacy ledger .md stay
// modify-only, and an added file anywhere outside this directory is still refused. The
// predicate and the diff flags below are shared with the other two gates on purpose — see
// authority-ledger-path.mjs.
function isAuthorityControlPath(value) { return AUTHORITY_CONTROL_PATHS.has(value) || isLedgerEntryPath(value); }
function verifySignedCommit(repoRoot, candidateSha, sha, label, authority = CONSOLE_CANDIDATE_SIGNING_AUTHORITY) {
  if (!gitSucceeds(repoRoot, ['cat-file', '-e', `${sha}^{commit}`])) fail(`${label} SHA is unresolvable`);
  try { verifyCommitWithCandidateSshPolicy(repoRoot, candidateSha, sha, authority); } catch (error) { fail(`${label} commit signature is not valid: ${error instanceof Error ? error.message : String(error)}`); }
}
function assertAuthorityOnlyDiff(repoRoot, candidateSha, integrationTipSha) {
  const fields = git(repoRoot, [...AUTHORITY_DIFF_ARGS, candidateSha, integrationTipSha]).split('\0');
  const changed = new Set();
  for (let index = 0; index < fields.length - 1;) {
    const header = fields[index++];
    const match = header.match(/^:([0-7]{6}) ([0-7]{6}) [0-9a-f]{40} [0-9a-f]{40} ([A-Z])(?:\d+)?$/);
    if (!match) fail('integration tip diff entry is malformed');
    const [, oldMode, newMode, status] = match;
    const paths = status === 'R' || status === 'C' ? [fields[index++], fields[index++]] : [fields[index++]];
    if (paths.some((entry) => !isAuthorityControlPath(entry))) fail(`integration tip changes product path after candidate: ${paths.find((entry) => !isAuthorityControlPath(entry))}`);
    // Unreachable while the shared flags say `--no-renames`, and kept for that reason: an `R`/`C`
    // entry carries TWO paths, so a reader that lost this branch would silently shift the whole
    // field stream by one and read the wrong path for every later entry.
    if (status === 'R' || status === 'C') fail(`integration tip contains forbidden ${status === 'R' ? 'rename' : 'copy'}`);
    // Status `A` is accepted only under the ledger directory, which is why the check reads the
    // path and not just the status. A new register, a new legacy .md, or a new file anywhere
    // else is still an unsupported status here.
    if (status !== 'M' && !(status === 'A' && isLedgerEntryPath(paths[0]))) fail(`integration tip contains unsupported diff status: ${status}`);
    if (newMode !== '100644' || oldMode !== (status === 'A' ? '000000' : '100644')) fail('integration tip may only modify regular mode-100644 authority documents or add regular mode-100644 ledger entries');
    if (paths.length !== 1 || changed.has(paths[0])) fail('integration tip authority document diff is malformed');
    changed.add(paths[0]);
  }
  // Allow-list, not a checklist — see verify-console-authority-train.mjs for why "all three" is gone.
  if (changed.size === 0) fail('integration tip must modify at least one authority document');
  assertNoUnresolvedMerge(repoRoot, integrationTipSha);
}

// The authority documents conflict on nearly every merge, and the correct resolution is a
// UNION — both entries kept — because nothing here verifies what the ledger SAYS, only that
// it changed. A union resolution done by hand leaves the marker lines behind, and nine of
// them reached main undetected before this check existed: `|||||||` with no `<<<<<<<` and no
// `>>>>>>>`, the signature of stripping two markers out of three.
//
// `=======` is deliberately NOT a marker here. It is also a Markdown setext heading rule, so
// matching it would fail the ledger on ordinary prose. The three asymmetric markers are
// unambiguous and each of them alone proves the resolution was left unfinished.
const MERGE_MARKERS = ['<<<<<<<', '|||||||', '>>>>>>>'];
// Every ledger entry file at the tip, not only the ones this train touched: a marker that
// arrived on an earlier train must stay refused, exactly as it does for the three fixed paths.
function ledgerEntryPaths(repoRoot, integrationTipSha) {
  return git(repoRoot, ['ls-tree', '-r', '--name-only', '-z', integrationTipSha, '--', LEDGER_DIRECTORY]).split('\0').filter(Boolean);
}
function assertNoUnresolvedMerge(repoRoot, integrationTipSha) {
  for (const entry of [...AUTHORITY_CONTROL_PATHS, ...ledgerEntryPaths(repoRoot, integrationTipSha)]) {
    const lines = git(repoRoot, ['show', `${integrationTipSha}:${entry}`]).split('\n');
    for (const [index, line] of lines.entries()) {
      const marker = MERGE_MARKERS.find((candidate) => line.startsWith(candidate));
      if (marker) fail(`${entry}:${index + 1} carries an unresolved merge marker (${marker}). Resolve the conflict as a union of both entries and delete the marker lines.`);
    }
  }
}

/**
 * Attests an immutable product candidate C and a later authority tip T.
 * Product facts must be read through this resolver; authority files remain at T.
 */
export function createConsoleCandidateSourceResolver(repoRoot, candidateSha, integrationTipSha, { candidateSigningAuthority = CONSOLE_CANDIDATE_SIGNING_AUTHORITY } = {}) {
  if (typeof repoRoot !== 'string' || !path.isAbsolute(repoRoot)) fail('candidate attestation requires canonical repository root');
  sha(candidateSha, 'candidate sha'); sha(integrationTipSha, 'integration tip SHA');
  verifySignedCommit(repoRoot, candidateSha, candidateSha, 'candidate', candidateSigningAuthority);
  verifySignedCommit(repoRoot, candidateSha, integrationTipSha, 'integration tip', candidateSigningAuthority);
  const parents = git(repoRoot, ['rev-list', '--parents', '-n', '1', integrationTipSha]).trim().split(/\s+/);
  if (parents.length !== 2 || parents[1] !== candidateSha) fail('integration tip must be the direct single-parent child of candidate');
  assertAuthorityOnlyDiff(repoRoot, candidateSha, integrationTipSha);
  const readText = (relativePath) => {
    if (!repositoryPath(relativePath)) fail('candidate source path is not repository-relative');
    try { return git(repoRoot, ['show', `${candidateSha}:${relativePath}`]); } catch { fail(`candidate source is missing: ${relativePath}`); }
  };
  const resolveSource = (relativePath) => {
    if (!repositoryPath(relativePath)) return false;
    const entry = git(repoRoot, ['ls-tree', candidateSha, '--', relativePath]).trim();
    return /^100644 blob [0-9a-f]{40}\t/.test(entry) || /^100755 blob [0-9a-f]{40}\t/.test(entry) ? { tracked_regular: true } : false;
  };
  return Object.freeze({ candidateSha, integrationTipSha, readText, resolveSource });
}
function nonRootCellRoots(configText) {
  const roots = []; let section = null;
  for (const line of configText.split('\n')) {
    const heading = line.match(/^\s*\[([^\]]+)\]/);
    if (heading) { section = heading[1]; continue; }
    if (section !== 'cells') continue;
    const entry = line.match(/^\s*[A-Za-z0-9_-]+\s*=\s*(\S+)\s*$/);
    if (entry && entry[1] !== '.') roots.push(entry[1].replace(/\/+$/, ''));
  }
  return roots;
}
/**
 * Resolves `//pkg:name` by reading the candidate's own `pkg/BUCK` blob for a
 * literal `name = "…"` declaration. It deliberately never invokes Buck2. This
 * validator is executed against candidate content inside the
 * `pull_request_target` authority job, where `buck2 targets` would evaluate
 * candidate-authored BUCK/`.bzl` — arbitrary code execution on an elevated
 * runner. Reading a blob is also the only form that behaves identically with and
 * without dotslash on PATH, which is why the assertion was dying there.
 *
 * ponytail: a literal declaration scan, not a Starlark evaluator. Measured
 * against `buck2 targets` over the whole repository: 0 false positives, and the
 * only 482 false negatives are reindeer-generated `//third-party/rust` targets,
 * which read as absent and therefore fail CLOSED. No first-party delivery unit
 * lives there. Upgrade path if one ever must: resolve targets from a job that
 * holds no candidate content — never by running Buck2 over the candidate.
 */
export function createConsoleBuckTargetResolver(candidateSource) {
  const cells = nonRootCellRoots(candidateSource.readText('.buckconfig'));
  return (target) => {
    const match = BUCK_TARGET.exec(typeof target === 'string' ? target : '');
    if (!match) return false;
    const [, pkg, name] = match;
    if (cells.some((cell) => pkg === cell || pkg.startsWith(`${cell}/`))) return false;
    const buckFile = `${pkg}/BUCK`;
    // resolveSource is consulted first: it answers false for an absent or
    // traversing path, where readText aborts the run. Either way "cannot read"
    // stays RED and never becomes "verified".
    if (!candidateSource.resolveSource(buckFile)) return false;
    const declaration = `name="${name}"`;
    return candidateSource.readText(buckFile).split('\n').some((line) => { const compact = line.replace(/\s/g, ''); return compact === declaration || compact === `${declaration},`; });
  };
}
/**
 * The 2026-07-28 clean-slate pivot deleted the whole frontend, so the console
 * route sources may be absent from the candidate. A console with no frontend
 * presents no routes, so the fact set is legitimately empty — but it is flagged
 * `route_source_present: false`, and validateConsoleTruthLedger then refuses
 * every positive route claim, because a claim that nothing can corroborate is a
 * contradiction, not a pass. A half-present source is a hard failure.
 */
export function extractConsoleRouteFactsFromCandidate(candidateSource) {
  const navPresent = Boolean(candidateSource.resolveSource(CONSOLE_NAV_SOURCE));
  const registryPresent = Boolean(candidateSource.resolveSource(CONSOLE_REGISTRY_SOURCE));
  if (navPresent !== registryPresent) fail(`candidate console route source is partially present: ${navPresent ? CONSOLE_REGISTRY_SOURCE : CONSOLE_NAV_SOURCE} is missing`);
  if (!navPresent) return ABSENT_CONSOLE_ROUTE_FACTS;
  return extractConsoleRouteFactsFromTexts(
    candidateSource.readText(CONSOLE_NAV_SOURCE),
    candidateSource.readText(CONSOLE_REGISTRY_SOURCE),
  );
}
function canonicalJsonDigest(value) { return createHash('sha256').update(JSON.stringify(stable(value))).digest('hex'); }
// Receipt digests are excluded from the registry digest they bind. Otherwise a
// receipt would need to hash its own hash fields, creating a rebinding loop.
export function promotionAuthorityDigests(registry, jurisdiction) {
  const registryAuthority = structuredClone(registry);
  for (const capability of array(registryAuthority.capabilities)) {
    if (capability?.benchmark?.independent_outcome_review) delete capability.benchmark.independent_outcome_review;
  }
  return Object.freeze({ registry: canonicalJsonDigest(registryAuthority), jurisdiction: canonicalJsonDigest(jurisdiction) });
}
function gpgSignatureMatches(status, signing) {
  if (signing?.format !== 'gpg' || !/^[A-F0-9]{40,64}$/.test(signing.fingerprint ?? '')) return false;
  const lines = String(status).split(/\r?\n/).filter((line) => line.startsWith('[GNUPG:] VALIDSIG '));
  return lines.length === 1 && new RegExp(`^\\[GNUPG:\\] VALIDSIG ${signing.fingerprint}(?:\\s|$)`).test(lines[0]);
}
function verifyImmutableReviewReceipt(repoRoot, reviewer, cap, candidate, outcomeIds, review, authorityDigests) {
  if (typeof repoRoot !== 'string' || !path.isAbsolute(repoRoot)) fail(`${cap.id} non-HOLD review requires canonical repository root`);
  const receiptPath = canonicalReceiptPath(cap.id, candidate.sha);
  if (review.receipt_path !== receiptPath) fail(`${cap.id} review receipt path is not canonical`);
  if (!gitSucceeds(repoRoot, ['cat-file', '-e', `${review.review_commit}^{commit}`])) fail(`${cap.id} review commit is missing`);
  const parents = git(repoRoot, ['rev-list', '--parents', '-n', '1', review.review_commit]).trim().split(/\s+/);
  if (parents.length !== 2 || !gitSucceeds(repoRoot, ['merge-base', '--is-ancestor', candidate.sha, review.review_commit])) fail(`${cap.id} review commit ancestry is invalid`);
  const changed = git(repoRoot, ['diff-tree', '--no-commit-id', '--name-only', '-r', review.review_commit]).trim().split('\n').filter(Boolean);
  if (changed.length !== 1 || changed[0] !== receiptPath) fail(`${cap.id} review commit may only change its canonical receipt`);
  const tree = git(repoRoot, ['ls-tree', review.review_commit, '--', receiptPath]).trim().match(/^(100644|100755) blob ([0-9a-f]{40})\t/);
  if (!tree) fail(`${cap.id} receipt must be a regular Git blob`);
  const raw = git(repoRoot, ['show', `${review.review_commit}:${receiptPath}`]);
  const parsed = parseImmutableJson(raw, `${cap.id} immutable review receipt`).value;
  const rawDigest = createHash('sha256').update(raw).digest('hex');
  const canonicalDigest = canonicalJsonDigest(parsed);
  if (review.receipt_sha256 !== rawDigest || review.receipt_canonical_sha256 !== canonicalDigest) fail(`${cap.id} receipt digest does not bind immutable bytes`);
  const expectedOutcomes = [...new Set(review.outcome_ids)].sort();
  const receiptOutcomes = Array.isArray(parsed.outcome_ids) ? [...new Set(parsed.outcome_ids)].sort() : [];
  if (parsed.candidate_sha !== candidate.sha || parsed.capability_id !== cap.id || JSON.stringify(receiptOutcomes) !== JSON.stringify(expectedOutcomes) || !expectedOutcomes.every((id) => outcomeIds.has(id)) || parsed.evidence_digest !== review.evidence_digest || parsed.verdict !== review.status || parsed.reviewer_id !== reviewer.id || parsed.registry_canonical_sha256 !== authorityDigests.registry || parsed.jurisdiction_canonical_sha256 !== authorityDigests.jurisdiction || review.registry_canonical_sha256 !== authorityDigests.registry || review.jurisdiction_canonical_sha256 !== authorityDigests.jurisdiction) fail(`${cap.id} receipt payload is not bound to candidate and authority digests`);
  const [authorName, authorEmail, committerName, committerEmail] = git(repoRoot, ['show', '-s', '--format=%an%x00%ae%x00%cn%x00%ce', review.review_commit]).trim().split('\0');
  if (authorName !== reviewer.author_name || authorEmail !== reviewer.author_email || committerName !== reviewer.committer_name || committerEmail !== reviewer.committer_email) fail(`${cap.id} review commit identity is not trusted`);
  let status;
  if (reviewer.signing?.format === 'ssh') {
    try { status = verifyCommitWithCandidateSshPolicy(repoRoot, candidate.sha, review.review_commit, reviewer.signing); } catch { fail(`${cap.id} review signature is not trusted`); }
    if (!sshSignatureMatchesAuthority(status, reviewer.signing)) fail(`${cap.id} review signature is not trusted`);
  } else {
    const signature = spawnSync('git', ['-C', repoRoot, 'verify-commit', '--raw', review.review_commit], { encoding: 'utf8' });
    status = `${signature.stdout ?? ''}${signature.stderr ?? ''}`;
    if (signature.status !== 0 || !gpgSignatureMatches(status, reviewer.signing)) fail(`${cap.id} review signature is not trusted`);
  }
  const attestation = Object.freeze({}); immutableReceiptAttestations.add(attestation); return attestation;
}

// This verifies planned inventory structure only. Promotion requires a separate
// authenticated, stage-specific verifier; descriptive evidence cannot grant it.
function validateReleaseInventory(registry, resolveSource) {
  const label = 'release inventory';
  const require = (condition, detail) => { if (!condition) fail(`${label}: ${detail}`); };
  const text = (value, detail) => nonempty(value, `${label} ${detail}`);
  const record = (value, detail) => object(value, `${label} ${detail}`);
  const list = (value, detail, empty = false) => {
    require(Array.isArray(value) && (empty || value.length > 0), `${detail} must be an array${empty ? '' : ' with entries'}`);
    return value;
  };
  const strings = (value, detail, empty = false) => uniqueStrings(list(value, detail, empty), `${label} ${detail}`);
  const uniqueRecords = (value, key, detail) => {
    const rows = list(value, detail);
    strings(rows.map((row) => record(row, detail)[key]), `${detail} ${key}`);
    return new Map(rows.map((row) => [row[key], row]));
  };
  require(Object.hasOwn(registry, 'release_inventory'), 'section is required');
  const release = record(registry.release_inventory, 'section');
  require(release.version === 1 && release.intent_date === '2026-09-19', 'unsupported version or intent date');
  require(JSON.stringify(release.authority) === JSON.stringify(['docs/current/PRODUCT.md','docs/current/ROADMAP.md','docs/current/DELIVERY.md']), 'current authority paths required');
  const population = record(release.population, 'population');
  require(population.groups === 1 && population.companies === 6 && population.people === 2000 && population.jurisdiction === 'KR', 'population differs from release contract');
  const stages = ['planned','implemented','integration_accepted','production_qualified','released'];
  const policy = record(release.state_policy, 'state policy');
  require(JSON.stringify(policy.order) === JSON.stringify(stages), 'state order is invalid');
  text(policy.rule, 'state rule'); text(policy.dispatch, 'dispatch rule');
  strings(release.cross_product_acceptance, 'cross-product acceptance');
  strings(release.blockers, 'blockers');
  const families = uniqueRecords(release.families, 'id', 'families');
  for (const family of families.values()) { text(family.label, 'family label'); text(family.coverage, 'family coverage'); }
  const exclusions = uniqueRecords(release.exclusions, 'id', 'exclusions');
  for (const excluded of exclusions.values()) {
    require(!families.has(excluded.id), 'family is also excluded'); text(excluded.reason, 'exclusion reason');
  }
  const leaves = uniqueRecords(release.leaves, 'id', 'leaves');
  for (const leaf of leaves.values()) {
    require(families.has(leaf.id.split('.')[0]) && leaf.id.includes('.'), `${leaf.id} has unknown family`);
    require(leaf.required === true, `${leaf.id} must be required`);
    require(Number.isInteger(leaf.milestone) && leaf.milestone >= 0 && leaf.milestone <= 10, `${leaf.id} milestone invalid`);
    text(leaf.owner, `${leaf.id} owner`); text(leaf.journey, `${leaf.id} journey`);
    strings(leaf.blockers, `${leaf.id} blockers`); record(leaf.evidence, `${leaf.id} evidence`);
    for (const dependency of strings(leaf.dependencies, `${leaf.id} dependencies`, true)) {
      require(dependency !== leaf.id && leaves.has(dependency), `${leaf.id} has invalid dependency ${dependency}`);
    }
    const acceptance = record(leaf.acceptance, `${leaf.id} acceptance`);
    text(acceptance.outcome, `${leaf.id} acceptance outcome`);
    require(acceptance.status === 'not_bound' && acceptance.command === null && acceptance.fixtures === null && acceptance.boundary === null, `${leaf.id} has unverified acceptance binding`);
    const states = record(leaf.states, `${leaf.id} states`);
    require(Object.keys(states).length === stages.length && stages.every((stage) => Object.hasOwn(states, stage) && typeof states[stage] === 'boolean'), `${leaf.id} state keys must be the five booleans`);
    require(states.planned, `${leaf.id} must be planned`);
    require(stages.slice(1).every((stage) => !states[stage]), `${leaf.id} unbound promotion cannot establish a later state`);
  }
  const visited = new Set(), active = new Set();
  const visit = (id) => {
    require(!active.has(id), `dependency cycle at ${id}`);
    if (visited.has(id)) return;
    active.add(id);
    for (const dependency of leaves.get(id).dependencies) visit(dependency);
    active.delete(id); visited.add(id);
  };
  for (const id of leaves.keys()) visit(id);

  const freeze = record(release.reference_freeze, 'reference freeze');
  require(freeze.target_date === '2026-09-19' && freeze.status === 'partial_snapshot', 'reference freeze is unverified');
  text(freeze.coverage, 'reference coverage'); strings(freeze.blockers, 'reference blockers');
  const sources = uniqueRecords(freeze.sources, 'id', 'sources');
  const url = (value) => {
    text(value, 'public URL');
    let parsed; try { parsed = new URL(value); } catch { fail(`${label}: invalid public URL`); }
    require(parsed.protocol === 'https:' && !parsed.username && !parsed.password, 'public URL must use HTTPS without credentials');
  };
  const timestamp = (value) => {
    require(typeof value === 'string' && /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})$/.test(value) && Number.isFinite(Date.parse(value)), 'source retrieval timestamp invalid');
    const [year, month, day] = value.slice(0, 10).split('-').map(Number);
    const date = new Date(0); date.setUTCFullYear(year, month - 1, day);
    require(date.getUTCFullYear() === year && date.getUTCMonth() === month - 1 && date.getUTCDate() === day, 'source retrieval timestamp calendar date invalid');
  };
  for (const source of sources.values()) {
    require(repositoryPath(source.path) && path.posix.normalize(source.path) === source.path && !/[\x00-\x1f]/.test(source.path), 'source path must be canonical repository relative');
    require(Number.isSafeInteger(source.bytes) && source.bytes > 0, 'source byte size invalid');
    require(SHA256.test(source.sha256 ?? '') && SHA256.test(source.compressed_sha256 ?? ''), 'source hashes invalid');
    url(source.requested_url); text(source.limitations, 'source limitations');
    if (Object.hasOwn(source, 'retrieved_utc')) {
      timestamp(source.retrieved_utc); url(source.effective_url); require(source.status === 200, 'source response not successful');
    } else { timestamp(source.retrieved_file_mtime_utc); text(source.timestamp_basis, 'source timestamp basis'); }
    const resolved = resolveSource(source.path);
    require(resolved === true || resolved?.tracked_regular === true, `source is not a tracked regular file: ${source.path}`);
  }
  const binding = (value) => {
    const item = record(value, 'source binding'), source = sources.get(item.source_id);
    require(source && item.artifact_path === source.path, 'source binding identity/path mismatch');
    const digests = ['sha256','artifact_uncompressed_sha256'].filter((key) => Object.hasOwn(item, key));
    require(digests.length > 0 && digests.every((key) => item[key] === source.sha256), 'source binding digest mismatch');
    text(item.quote, 'source quotation'); text(item.quote_matching ?? item.matching, 'quote matching'); text(item.scope, 'source scope');
  };
  const matrices = record(release.support_matrices, 'support matrices');
  const connectorRows = list(record(matrices.connectors, 'connectors matrix').entries, 'connector entries');
  const connectorNames = new Set(), connectorIdentities = new Set();
  for (const row of connectorRows) {
    record(row, 'connector entry'); text(row.reference_path, 'connector reference path');
    const named = typeof row.status === 'string' && row.status.match(/^Overview explicitly names (\S(?:.*\S)?) under (\S(?:[^;]*\S)?); individual support\/modes\/lifecycle not verified$/);
    require(named || row.status === 'discovered documentation link; support/modes/lifecycle not verified', 'malformed connector reference status');
    const identity = named ? `name:${named[1]}` : `path:${row.reference_path}`;
    require(!connectorIdentities.has(identity), 'duplicate connector identity');
    connectorIdentities.add(identity);
    if (named) connectorNames.add(named[1]);
  }
  const runtimeNames = new Set(list(record(matrices.runtimes, 'runtimes matrix').entries, 'runtime entries').map((row) => record(row, 'runtime entry').runtime));
  const bindings = record(freeze.source_bindings, 'source bindings');
  for (const [leaf, entries] of Object.entries(bindings)) {
    const targetExists = leaves.has(leaf)
      || (leaf.startsWith('connector:') && connectorNames.has(leaf.slice('connector:'.length)))
      || (leaf.startsWith('runtime:') && runtimeNames.has(leaf.slice('runtime:'.length)));
    require(targetExists, 'source binding names unknown leaf or support identity');
    for (const item of list(entries, 'leaf source bindings')) binding(item);
  }
  for (const name of connectorNames) require(Object.hasOwn(bindings, `connector:${name}`), 'named connector source binding required');
  for (const field of ['runtime_qualifications','models_administration_qualifications']) {
    if (Object.hasOwn(freeze, field)) for (const item of list(freeze[field], field)) binding(item);
  }
  const consoleAcceptance = (row) => {
    require(['not established','not established; does not remove explicit Console requirements'].includes(row.console_acceptance) && row.command === null, 'support cell claims unverified Console acceptance');
  };
  for (const [name, key] of [['connectors','reference_path'],['runtimes','runtime'],['sdks','language']]) {
    const matrix = record(matrices[name], `${name} matrix`);
    require(matrix.status === 'incomplete', `${name} support matrix is unverified`);
    strings(matrix.required_dimensions, `${name} dimensions`);
    if (name === 'connectors') require(sources.has(matrix.catalog_source), 'connector catalog source invalid');
    const rows = name === 'connectors' ? connectorRows : uniqueRecords(matrix.entries, key, `${name} entries`).values();
    for (const row of rows) {
      consoleAcceptance(row);
      if (name === 'connectors') {
        require(row.reference_path.startsWith('/docs/foundry/'), 'connector reference path invalid');
        text(row.status, 'connector reference status');
        if (row.source_binding) binding(row.source_binding);
      } else {
        require(sources.has(row.source), `${name} source invalid`); text(row.reference_status, `${name} reference status`);
        if (name === 'sdks') text(row.reference_distribution, 'SDK distribution');
      }
    }
  }
  // Frozen reference inventory only; no runtime semantics or Console support is qualified.
  const fusion = matrices.runtimes.entries.find((row) => row.runtime === 'Fusion formula functions');
  require(fusion, 'Fusion formula function catalogue is required');
  const catalogueKeys = (value, keys, detail, catalogue = 'Fusion') => {
    record(value, `${catalogue} ${detail}`);
    const actual = Object.keys(value);
    require(actual.length === keys.length && keys.every((key) => actual.includes(key)), `${catalogue} ${detail} keys differ from frozen reference`);
  };
  catalogueKeys(fusion, ['runtime','reference_status','source','console_acceptance','command','function_catalog','semantics_status','unqualified_semantics'], 'runtime');
  require(fusion.source === 'fusion-function-library'
    && fusion.reference_status === 'Frozen 202-entry reference catalogue; lifecycle labels preserved; Console execution unverified'
    && fusion.console_acceptance === 'not established' && fusion.command === null
    && fusion.semantics_status === 'not_bound', 'Fusion runtime must retain unqualified reference status');
  catalogueKeys(fusion.unqualified_semantics, ['runtime_and_library_versions','type_and_coercion','null_and_error','optional_and_variadic_arguments','locale_timezone_precision','dependency_recalculation','effects_authority_and_recovery','concurrency_reconnect','lifecycle_successor_equivalence'], 'semantics');
  require(Object.values(fusion.unqualified_semantics).every((value) => value === null), 'Fusion semantics remain unqualified');
  require(Array.isArray(fusion.function_catalog) && fusion.function_catalog.length === 202, 'Fusion catalogue must contain 202 ordered entries');
  for (const row of fusion.function_catalog) {
    // Check own keys before JSON hashing: undefined/function values disappear in JSON.
    catalogueKeys(row, ['section','signature','reference_lifecycle','required_leaf','console_acceptance','source_binding'], 'catalogue entry');
    catalogueKeys(row.source_binding, ['source_id','artifact_path','sha256','quote','matching','scope'], 'catalogue binding');
  }
  require(canonicalJsonDigest(fusion.function_catalog) === '520cb709cd701a074912c8d7a2040300090dd88023d4273ae0b052189b0b98ee', 'Fusion catalogue differs from frozen reviewed records');
  for (const row of fusion.function_catalog) {
    require(leaves.has(row.required_leaf) && ['F09.formula-action-library','F09.formula-chart-library','F09.formula-core-library','F09.formula-timeseries-library','F09.formula-validation-library'].includes(row.required_leaf)
      && row.console_acceptance === 'not established', 'Fusion catalogue leaf or acceptance invalid');
    binding(row.source_binding);
    require(bindings[row.required_leaf]?.some((item) => canonicalJsonDigest(item) === canonicalJsonDigest(row.source_binding)), 'Fusion catalogue exact shared source binding is required');
  }
  for (const [runtime, source, count, leaf, digest] of [
    ['Quiver time-series cards', 'quiver-cards-index-time-series', 39, 'F08.time-series-transform-library', 'fe9892f6fdd0efe6808255cb8a7cad7d3530d9482538277689b498e695d55945'],
    ['Quiver chart cards', 'quiver-cards-index-charts', 21, 'F08.chart-library', '3b34ea79d416110e8ddb259566a80cf53073747d4a3325a4ee938df1a714bc5a'],
  ]) {
    const row = matrices.runtimes.entries.find((entry) => entry.runtime === runtime);
    require(row, `Quiver catalogue ${runtime} is required`);
    catalogueKeys(row, ['runtime','reference_status','source','console_acceptance','command','analytical_card_catalog','semantics_status','unqualified_semantics'], 'runtime', 'Quiver');
    require(row.source === source && row.reference_status === `Frozen ${count}-entry index catalogue; label/path tuples preserved; individual semantics and Console execution unverified`
      && row.console_acceptance === 'not established' && row.command === null && row.semantics_status === 'not_bound', 'Quiver runtime must retain unqualified reference status');
    catalogueKeys(row.unqualified_semantics, ['runtime_and_library_versions','input_types_and_nulls','numeric_and_time_semantics','limits_and_performance','refresh_and_reproducibility','effects_authority_and_disclosure','collaboration_and_recovery','lifecycle_and_successor_equivalence'], 'semantics', 'Quiver');
    require(Object.values(row.unqualified_semantics).every((value) => value === null), 'Quiver semantics remain unqualified');
    require(Array.isArray(row.analytical_card_catalog) && row.analytical_card_catalog.length === count, `Quiver catalogue must contain ${count} ordered entries`);
    for (const card of row.analytical_card_catalog) {
      catalogueKeys(card, ['category','operation','reference_path','required_leaf','scope','source_binding'], 'card', 'Quiver');
      catalogueKeys(card.source_binding, ['source_id','artifact_path','sha256','quote','matching','scope'], 'binding', 'Quiver');
    }
    // Keep distinct labels sharing a path: the pin binds original ordered tuples.
    require(canonicalJsonDigest(row.analytical_card_catalog) === digest, 'Quiver catalogue differs from frozen reviewed records');
    for (const card of row.analytical_card_catalog) {
      require(card.category === source && card.source_binding.source_id === source && card.required_leaf === leaf && leaves.has(leaf), 'Quiver card source or leaf invalid');
      binding(card.source_binding);
      require(bindings[leaf]?.some((item) => canonicalJsonDigest(item) === canonicalJsonDigest(card.source_binding)), 'Quiver catalogue exact shared source binding is required');
    }
  }
  const media = matrices.runtimes.entries.find((row) => row.runtime === 'Media operations and reference compute rates');
  require(media, 'Media operation catalogue is required');
  catalogueKeys(media, ['runtime','reference_status','source','console_acceptance','command','media_transform_catalog','reference_rate_unit','semantics_status','unqualified_semantics'], 'runtime', 'Media');
  require(media.source === 'media-sets-advanced-formats-media-usage-limits'
    && media.reference_status === 'Frozen 44-row public operation/rate catalogue: 43 required references and 1 explicit Intelligence exclusion; vendor usage units are not Console price, latency or throughput; executable semantics unverified'
    && media.console_acceptance === 'not established' && media.command === null
    && media.reference_rate_unit === 'Foundry compute-seconds per GB processed'
    && media.semantics_status === 'not_bound', 'Media runtime must retain unqualified reference status and units');
  catalogueKeys(media.unqualified_semantics, ['operation_semantics','format_codec_and_version_support','runtime_and_model_versions','limits_and_resource_accounting','authorization_and_source_restrictions','output_publication_and_partial_failure','cancellation_and_recovery','retention_and_deletion','lifecycle_and_successor_equivalence'], 'semantics', 'Media');
  require(Object.values(media.unqualified_semantics).every((value) => value === null), 'Media semantics remain unqualified');
  require(Array.isArray(media.media_transform_catalog) && media.media_transform_catalog.length === 44, 'Media catalogue must contain 44 ordered entries');
  for (const row of media.media_transform_catalog) {
    catalogueKeys(row, ['category','operation','reference_compute_seconds_per_gb','required_leaf','scope','source_binding'], 'card', 'Media');
    catalogueKeys(row.source_binding, ['source_id','artifact_path','sha256','quote','matching','scope'], 'binding', 'Media');
  }
  // Bind category/name/rate tuples, including repeated names and the retained exclusion.
  require(canonicalJsonDigest(media.media_transform_catalog) === '76e13ccb2918a75b6612415d891f68fd8695667f6e3539d68806281934b17504', 'Media catalogue differs from frozen reviewed records');
  let excludedMediaRows = 0;
  for (const row of media.media_transform_catalog) {
    require(row.source_binding.source_id === media.source, 'Media catalogue source invalid');
    binding(row.source_binding);
    if (row.required_leaf === null) {
      excludedMediaRows += 1;
      require(row.category === 'Documents' && row.operation === 'Extract text using VLM *'
        && row.reference_compute_seconds_per_gb === 275
        && row.scope === 'Excluded generative/model-specific operation; Intelligence roadmap authority required', 'Media exclusion must retain the reviewed VLM scope');
    } else {
      require(leaves.has(row.required_leaf) && ['F04.media-capacity-recovery','F04.image-transform-library','F04.audio-transform-library','F04.video-transform-library','F04.document-transform-library','F04.spreadsheet-extraction','F12.inference'].includes(row.required_leaf), 'Media catalogue leaf invalid');
      require(bindings[row.required_leaf]?.some((item) => canonicalJsonDigest(item) === canonicalJsonDigest(row.source_binding)), 'Media catalogue exact shared source binding is required');
    }
  }
  require(excludedMediaRows === 1, 'Media catalogue must retain exactly one explicit exclusion');
  if (Object.hasOwn(matrices, 'models_administration')) {
    for (const row of uniqueRecords(matrices.models_administration, 'dimension', 'models administration matrix').values()) {
      strings(row.values, 'model support values'); text(row.reference_support, 'model reference support'); binding(row.source_binding); consoleAcceptance(row);
    }
  }
}

export function validateConsoleTruthLedger(registry, jurisdiction, options = {}) {
  validatedRegistries.delete(registry);
  const { resolveSha = () => true, resolveRef = () => true, resolveBuckTarget = () => true, resolveSource = () => true, expectedCandidateSha, routeFacts, repoRoot } = options;
  object(registry, 'registry'); object(jurisdiction, 'jurisdiction register');
  if (registry.schema_version !== 'console-capability-registry-v2') fail('unsupported console capability registry schema');
  if (jurisdiction.schema_version !== 'console-jurisdiction-register-v2') fail('unsupported console jurisdiction register schema');
  const governingLifecycle = object(jurisdiction.governing_lifecycle, 'governing lifecycle');
  if (governingLifecycle.status !== 'HOLD_UNAVAILABLE'
    || governingLifecycle.path !== null
    || governingLifecycle.mutation_policy !== 'not_an_active_dependency'
    || !SHA256.test(governingLifecycle.last_known_sha256 ?? '')) {
    fail('governing lifecycle must remain a pathless HOLD_UNAVAILABLE provenance record');
  }
  nonempty(governingLifecycle.reason, 'governing lifecycle unavailable reason');
  const continuationReset = object(registry.continuation_reset, 'continuation reset');
  if (continuationReset.as_of !== '2026-08-03'
    || continuationReset.status !== 'HOLD'
    || continuationReset.worktree_dependency !== 'none'
    || continuationReset.branch_dependency !== 'none'
    || continuationReset.lane_dispatch !== 'disabled') {
    fail('continuation reset must keep every pre-wipe worktree, branch, and lane on HOLD');
  }
  const historicalFields = array(continuationReset.historical_fields);
  if (historicalFields.length !== HISTORICAL_CONTINUATION_FIELDS.length
    || HISTORICAL_CONTINUATION_FIELDS.some((field, index) => historicalFields[index] !== field)) {
    fail('continuation reset historical field declaration is invalid');
  }
  if (continuationReset.historical_snapshot_sha256 !== CONTINUATION_SNAPSHOT_SHA256) {
    fail('continuation reset historical snapshot digest is not the pinned reset snapshot');
  }
  nonempty(continuationReset.reason, 'continuation reset reason');
  nonempty(continuationReset.historical_snapshot_scope, 'continuation reset historical snapshot scope');
  // The candidate SHA arrives from OUTSIDE these documents and is the only source: callers bind
  // it from Git object identity, and the planner takes `--candidate`. The registers used to store
  // a copy and this function compared the two, which is a value checked against a copy of itself
  // — the file was written from the same Git fact.
  const candidate = { sha: sha(expectedCandidateSha, 'candidate sha') };
  if (!resolveSha(candidate.sha)) fail('candidate SHA is unresolvable');
  const provenanceDocuments = [
    ['registry', object(registry.provenance, 'registry provenance')],
    ['jurisdiction', object(jurisdiction.provenance, 'jurisdiction provenance')],
  ];
  for (const [document, provenance] of provenanceDocuments) {
    for (const key of ['authority_base_sha', 'historical_implementation_freeze_sha']) {
      const value = sha(provenance[key], `${document} ${key}`);
      if (!resolveSha(value)) fail(`${document} ${key} SHA is unresolvable`);
      const refKey = key.replace(/_sha$/, '_ref');
      const ref = tagRef(provenance[refKey], `${document} ${refKey}`);
      if (!resolveRef(ref, value)) fail(`${document} ${refKey} does not resolve to ${key}`);
    }
    if (provenance.authority_base_sha === candidate.sha) fail(`${document} authority base SHA must remain distinct from exact candidate`);
    if (provenance.historical_implementation_freeze_sha === candidate.sha) fail(`${document} historical implementation freeze must remain distinct from exact candidate`);
  }
  if (registry.provenance.historical_implementation_freeze_sha !== jurisdiction.provenance.historical_implementation_freeze_sha
    || registry.provenance.historical_implementation_freeze_ref !== jurisdiction.provenance.historical_implementation_freeze_ref) fail('registry and jurisdiction historical implementation freeze provenance must match');
  if (registry.design_reference?.sha256?.length !== 64) fail('missing Claude Design digest');
  if (typeof registry.build_reference?.buck2_release_pin !== 'string' || typeof registry.build_reference?.buck2_embedded_binary_version !== 'string') fail('missing separate Buck2 release pin and embedded binary version');
  const omni = object(registry.shared_omni_platform_gate, 'shared omni-platform gate');
  for (const area of ['identity_scope', 'object_action_workflow', 'search', 'audit_lineage', 'interoperability']) nonempty(omni.required_outcomes?.[area], `shared omni-platform gate ${area}`);
  if (!Array.isArray(registry.capabilities) || registry.capabilities.length === 0) fail('registry capabilities must be a non-empty array');
  const ids = new Set();
  const globalOutcomeAssertions = new Set();
  const privateRoots = [];
  const sharedRoots = new Set(array(registry.shared_collision_roots?.paths));
  for (const cap of registry.capabilities) {
    object(cap, 'capability'); nonempty(cap.id, 'capability id');
    if (ids.has(cap.id)) fail(`duplicate capability id: ${cap.id}`); ids.add(cap.id);
    for (const field of HISTORICAL_CONTINUATION_FIELDS) {
      if (!Object.hasOwn(cap, field)) fail(`${cap.id} is missing historical continuation field ${field}`);
    }
    if (cap.historical_worktree !== null) nonempty(cap.historical_worktree, `${cap.id} historical worktree`);
    if (cap.historical_branch !== null) nonempty(cap.historical_branch, `${cap.id} historical branch`);
    if (cap.historical_lane_assignments !== null) object(cap.historical_lane_assignments, `${cap.id} historical lane assignments`);
    for (const field of ['historical_state', 'historical_reset_state']) {
      const historicalState = object(cap[field], `${cap.id} ${field}`);
      for (const stateField of CONTINUATION_STATE_FIELDS) nonempty(historicalState[stateField], `${cap.id} ${field} ${stateField}`);
    }
    if (cap.worktree !== null || cap.branch !== null || cap.lane_assignments !== null) {
      fail(`${cap.id} current continuation assignment must be null while reset is HOLD`);
    }
    const currentState = object(cap.state, `${cap.id} current continuation state`);
    if (Object.keys(currentState).length !== CONTINUATION_STATE_FIELDS.length
      || CONTINUATION_STATE_FIELDS.some((field) => currentState[field] !== 'HOLD')) {
      fail(`${cap.id} current continuation state must contain only HOLD values`);
    }
    const truth = object(cap.truth, `${cap.id} truth`);
    for (const key of ['declared', 'implementation', 'verification', 'exposure']) {
      if (!STATES.has(truth[key])) fail(`${cap.id} invalid truth state ${key}`);
    }
    if (truth.exposure === 'EXPOSED' && truth.verification !== 'VERIFIED') fail(`${cap.id} exposed claim requires verified evidence`);
    // `object(...)` carries the existence guarantee the deleted `evidence.candidate_sha !==
    // candidate.sha` equality was contributing, and the two lines after it carry the rest: that
    // equality refused an EMPTY payload too, because `undefined !== sha`. Existence alone would
    // be weaker than what was removed; existence plus the fields that make it evidence is equal.
    const evidence = object(cap.candidate_evidence, `${cap.id} candidate evidence`);
    if (!STATES.has(evidence.status)) fail(`${cap.id} candidate evidence status is invalid`);
    nonempty(evidence.reason, `${cap.id} candidate evidence reason`);
    object(evidence.contract, `${cap.id} candidate evidence contract`);
    // `source_sha` was in this list and is gone, and `candidate_sha` followed it out of the
    // document. Both were required non-empty, both held a copy of the candidate SHA, and neither
    // could disagree with the value it was copied from. A field that cannot fail is not a
    // control, it is a rebind cost. What the payload still owes is below: a real contract.
    for (const key of ['backend_binary_digest_or_build_sha', 'database', 'api', 'browser', 'trace_logs']) nonempty(evidence.contract[key], `${cap.id} candidate evidence contract ${key}`);
    const benchmark = object(cap.benchmark, `${cap.id} per-module benchmark`);
    for (const key of ['category', 'non_goals', 'evidence_binding']) nonempty(benchmark[key], `${cap.id} benchmark ${key}`);
    const sources = array(benchmark.comparator_sources);
    if (!sources.length) fail(`${cap.id} per-module benchmark has no comparator sources`);
    for (const source of sources) { object(source, `${cap.id} comparator source`); nonempty(source.source, `${cap.id} comparator source path`); nonempty(source.observation_as_of, `${cap.id} comparator observation date`); if (!/^\d{4}-\d{2}-\d{2}$/.test(source.observation_as_of) || (() => { const [y,m,d]=source.observation_as_of.split('-').map(Number); const date=new Date(Date.UTC(y,m-1,d)); return date.getUTCFullYear()!==y || date.getUTCMonth()!==m-1 || date.getUTCDate()!==d; })()) fail(`${cap.id} comparator observation date must be ISO`); const resolvedSource=resolveSource(source.source); if (!(resolvedSource === true || resolvedSource?.tracked_regular === true)) fail(`${cap.id} comparator source is not tracked regular file`); nonempty(source.observation, `${cap.id} comparator observation`); }
    if (array(benchmark.native_outcomes).length < 3 || array(benchmark.native_outcomes).length > 7) fail(`${cap.id} benchmark requires 3-7 measurable native outcomes`);
    if (array(benchmark.omni_outcomes).length < 1 || array(benchmark.omni_outcomes).length > 3) fail(`${cap.id} benchmark requires 1-3 additive omni outcomes`);
    const outcomeIds = new Set(); const outcomeShapes = new Set();
    for (const outcome of [...benchmark.native_outcomes, ...benchmark.omni_outcomes]) { object(outcome, `${cap.id} benchmark outcome`); for (const key of ['id','persona_scenario','action_workflow','measurable_assertion','required_receipts','status']) nonempty(outcome[key], `${cap.id} benchmark outcome ${key}`); if (outcome.status !== 'HOLD') fail(`${cap.id} benchmark outcome must remain HOLD`); if (outcomeIds.has(outcome.id)) fail(`${cap.id} duplicate benchmark outcome id`); outcomeIds.add(outcome.id); const shape=outcome.measurable_assertion; if (outcomeShapes.has(shape) || globalOutcomeAssertions.has(shape)) fail(`${cap.id} duplicate benchmark outcome assertion`); outcomeShapes.add(shape); globalOutcomeAssertions.add(shape); }
    if (!['SOURCE_BOUNDED_STARTING_DOSSIER','HOLD_INSUFFICIENT_CATEGORY_DOSSIER'].includes(benchmark.dossier_status)) fail(`${cap.id} benchmark dossier status is invalid`);
    if (benchmark.dossier_status === 'HOLD_INSUFFICIENT_CATEGORY_DOSSIER') nonempty(benchmark.missing_dossier_reason, `${cap.id} missing dossier reason`);
    if (!VERDICTS.has(benchmark.verdict)) fail(`${cap.id} benchmark verdict is invalid`);
    nonempty(benchmark.independent_outcome_review?.status, `${cap.id} independent outcome review status`);
    if (benchmark.verdict !== 'HOLD' && evidence.status !== 'VERIFIED') fail(`${cap.id} non-HOLD benchmark requires verified candidate evidence`);
    // The independent review was OPTIONAL, and that made every control below it optional too.
    //
    // `verdict: MEET` + `candidate_evidence.status: VERIFIED` + `independent_outcome_review.status:
    // HOLD` validated clean. Both words are written by the same hand that owns the capability, so a
    // passing verdict was self-assertable. The receipt machinery underneath is rigorous — signed
    // commit, canonical registry+jurisdiction digests, and `review.reviewer_id === cap.owner`
    // refused — but ALL of it hangs off the `status !== 'HOLD'` branch below, so declaring no
    // review at all skipped it. A prohibition on reviewing your own work is not a control if
    // "no reviewer" is an accepted answer.
    //
    // Inert on this candidate: all 27 capabilities are HOLD on verdict, review and evidence. That
    // is precisely why it is cheap to add now — the first capability to claim a passing verdict is
    // the one that would otherwise have spent the gap.
    if (benchmark.verdict !== 'HOLD' && benchmark.independent_outcome_review.status === 'HOLD') fail(`${cap.id} non-HOLD benchmark requires a non-HOLD independent outcome review`);
    if (benchmark.independent_outcome_review.status !== 'HOLD') { const review=benchmark.independent_outcome_review; const reviewer=array(registry.review_authority?.reviewers).find((entry) => entry.id === review.reviewer_id); const authorityDigests=promotionAuthorityDigests(registry, jurisdiction); if (!reviewer || review.reviewer_id === cap.owner || review.capability_id !== cap.id || review.candidate_sha !== candidate.sha || !Array.isArray(review.outcome_ids) || !review.outcome_ids.length || new Set(review.outcome_ids).size !== review.outcome_ids.length || !review.outcome_ids.every((id) => outcomeIds.has(id)) || !/^[0-9a-f]{64}$/.test(review.evidence_digest ?? '') || !SHA.test(review.review_commit ?? '') || !/^[0-9a-f]{64}$/.test(review.receipt_sha256 ?? '') || !/^[0-9a-f]{64}$/.test(review.receipt_canonical_sha256 ?? '') || !/^[0-9a-f]{64}$/.test(review.registry_canonical_sha256 ?? '') || !/^[0-9a-f]{64}$/.test(review.jurisdiction_canonical_sha256 ?? '')) fail(`${cap.id} non-HOLD review receipt schema is invalid`); const attestation=verifyImmutableReviewReceipt(repoRoot, reviewer, cap, candidate, outcomeIds, review, authorityDigests); if (!immutableReceiptAttestations.has(attestation)) fail(`${cap.id} internal receipt attestation was not minted`); }
    const delivery = object(cap.delivery_unit, `${cap.id} delivery unit`);
    nonempty(delivery.id, `${cap.id} delivery unit id`);
    if (!['NOT_APPLICABLE','REQUIRED','REQUIRED_UNRESOLVED'].includes(delivery.rust_status)) fail(`${cap.id} delivery unit has invalid Rust status`);
    const buckTargets = array(delivery.buck2_targets);
    const verificationBuckTargets = array(cap.tests?.buck2_targets);
    if (delivery.rust_status === 'REQUIRED' && !buckTargets.length) fail(`${cap.id} Rust-required delivery unit has empty Buck targets`);
    if (delivery.rust_status === 'REQUIRED_UNRESOLVED' && (truth.implementation !== 'HOLD' || evidence.status !== 'HOLD')) fail(`${cap.id} unresolved Rust delivery must remain HOLD`);
    if (verificationBuckTargets.length && (buckTargets.length !== verificationBuckTargets.length || buckTargets.some((target, index) => target !== verificationBuckTargets[index]))) fail(`${cap.id} delivery Buck targets must match declared verification targets`);
    for (const target of buckTargets) { if (typeof target !== 'string' || !BUCK_TARGET.test(target) || !resolveBuckTarget(target)) fail(`${cap.id} has invalid/nonexistent Buck target`); }
    const dependencies = array(cap.dependency_edges);
    for (const edge of dependencies) {
      object(edge, `${cap.id} dependency edge`); nonempty(edge.target, `${cap.id} dependency target`);
      if (edge.target === cap.id || !EDGE_TYPES.has(edge.type)) fail(`${cap.id} has dangling/invalid dependency`);
    }
    const route = object(cap.route_presentation, `${cap.id} route/presentation state`);
    if (!Array.isArray(route.route_keys)) fail(`${cap.id} route keys must be an array`);
    for (const key of ROUTE_CLAIM_FIELDS) if (typeof route[key] !== 'boolean') fail(`${cap.id} route/presentation ${key} must be boolean`);
    // Checked independently of the per-key loop below: with no route source in
    // the candidate there are zero keys to iterate, so the loop would corroborate
    // nothing while every claim stays `true`. A claim no source can corroborate
    // is a contradiction, not a pass.
    if (routeFacts && routeFacts.route_source_present !== true) for (const key of ROUTE_CLAIM_FIELDS) if (route[key] === true) fail(`${cap.id} route/presentation claims ${key} but the candidate has no console route source to corroborate it`);
    nonempty(route.evidence_receipt_status, `${cap.id} route evidence receipt status`); nonempty(route.source, `${cap.id} route/presentation source`);
    if (route.production_exposed && !route.source_mounted) fail(`${cap.id} exposed route must be mounted`);
    if (truth.exposure === 'EXPOSED' && !route.production_exposed) fail(`${cap.id} exposed truth contradicts route presentation`);
    if (route.production_exposed && truth.exposure !== 'EXPOSED') fail(`${cap.id} route exposure contradicts truth state`);
    if (routeFacts) for (const key of route.route_keys) {
      // Own-property only — facts maps from JSON inherit Object.prototype; a route_key
      // named constructor/toString must not resolve to a Function.
      const fact = routeFacts.facts && typeof routeFacts.facts === 'object' && Object.hasOwn(routeFacts.facts, key)
        ? routeFacts.facts[key]
        : undefined;
      if (!fact || ROUTE_CLAIM_FIELDS.some((field)=>fact[field]!==route[field])) fail(`${cap.id} route source fact mismatch for ${key}`);
    }
    const ownership = object(cap.ownership, `${cap.id} ownership`);
    for (const key of ['frontend_roots', 'backend_roots', 'api_schema_roots']) for (const root of array(ownership[key])) nonempty(root, `${cap.id} ownership root`);
    for (const root of array(ownership.private_roots)) { nonempty(root, `${cap.id} private ownership root`); if (sharedRoots.has(root)) fail(`${cap.id} private root is declared shared`); privateRoots.push([cap.id, root]); }
    for (const root of array(ownership.serial_roots)) nonempty(root, `${cap.id} serial ownership root`);
    object(cap.resource_requirements, `${cap.id} resources`);
    for (const key of RESOURCE_KEYS) if (!Number.isInteger(cap.resource_requirements[key]) || cap.resource_requirements[key] < 0) fail(`${cap.id} invalid resource ${key}`);
    if (!array(cap.jurisdiction_bindings).length) fail(`${cap.id} missing jurisdiction bindings`);
  }
  const continuationSnapshot = registry.capabilities.map((cap) => ({
    id: cap.id,
    historical_worktree: cap.historical_worktree,
    historical_branch: cap.historical_branch,
    historical_lane_assignments: cap.historical_lane_assignments,
    historical_state: cap.historical_state,
    historical_reset_state: cap.historical_reset_state,
  }));
  if (ledgerDigest(continuationSnapshot) !== CONTINUATION_SNAPSHOT_SHA256) {
    fail('historical continuation fields differ from the pinned reset snapshot');
  }
  for (const cap of registry.capabilities) for (const edge of array(cap.dependency_edges)) if (!ids.has(edge.target)) fail(`${cap.id} has dangling dependency target ${edge.target}`);
  for (let i = 0; i < privateRoots.length; i++) for (let j = i + 1; j < privateRoots.length; j++) {
    const [aId, a] = privateRoots[i], [bId, b] = privateRoots[j];
    if (aId !== bId && (a === b || a.startsWith(`${b.replace(/\/\*\*$/, '')}/`) || b.startsWith(`${a.replace(/\/\*\*$/, '')}/`))) fail(`overlapping private roots: ${aId}:${a} and ${bId}:${b}`);
  }
  // NOTHING stored in this document ties it to the candidate any more, and nothing could: every
  // value it might hold would be written from the same git fact the caller already supplies.
  // ONE thing binds it, and it is worth naming exactly: the C..T train. T is signed, is C's
  // direct single-parent child, and may modify nothing outside the authority allow-list, so the
  // register validated here is the tree exactly one commit after C.
  //
  // The bijection below is NOT a second binding, and an earlier draft of this comment claimed it
  // was. It compares this register against the capability registry, and both are read out of the
  // same T — two documents that are stale together satisfy it exactly as well as two that are
  // current. What it catches is disagreement between them, which is a different property.
  // The accepted residue is a WHOLESALE revert of the register while every row is HOLD; it is
  // stated in docs/program/ledger/2026-08-01-candidate-sha-leaves-the-registers.md.
  const targets = array(jurisdiction.target_jurisdiction_set); const jurisdictionRows = array(jurisdiction.jurisdictions);
  if (targets.length !== 1 || targets[0] !== 'KR' || jurisdictionRows.length !== 1 || jurisdictionRows[0]?.id !== 'JUR-KR-001' || jurisdictionRows[0]?.country_code !== 'KR') fail('jurisdiction target must be exactly KR / JUR-KR-001');
  const controls = new Map(); for (const control of array(jurisdiction.controls)) { if (controls.has(control.id)) fail(`duplicate control id: ${control.id}`); controls.set(control.id, control); }
  if (!controls.size) fail('jurisdiction register has no controls');
  for (const control of controls.values()) {
    if (control.release_disposition !== 'HOLD') fail(`jurisdiction control ${control.id} must remain HOLD without qualified authority`);
    nonempty(control.freshness?.status, `${control.id} freshness status`);
    nonempty(control.unhold_authority, `${control.id} explicit unhold authority`);
    if (!array(control.capability_traceability).length) fail(`${control.id} missing capability traceability`); const traceTuples = new Set(); for (const trace of control.capability_traceability) { const tuple=`${trace.capability_id}`; if (traceTuples.has(tuple)) fail(`${control.id} duplicate trace tuple`); traceTuples.add(tuple); } const controlEvidence = object(control.candidate_evidence, `${control.id} control candidate evidence`); if (!STATES.has(controlEvidence.status)) fail(`${control.id} control evidence status is invalid`); nonempty(controlEvidence.reason, `${control.id} control evidence reason`);
  }
  const bindingTuples = new Set(); for (const cap of registry.capabilities) for (const binding of cap.jurisdiction_bindings) { const tuple=`${binding.control_id}|${cap.id}`; if (bindingTuples.has(tuple)) fail(`${cap.id} duplicate jurisdiction binding`); bindingTuples.add(tuple);
    if (binding.jurisdiction_id !== 'JUR-KR-001' || !controls.has(binding.control_id)) fail(`${cap.id} has missing jurisdiction control ${binding.control_id}`);
    if (!array(controls.get(binding.control_id).capability_traceability).some((trace) => trace.capability_id === cap.id)) fail(`${cap.id} jurisdiction trace is not bidirectional`);
  }
  // Both sides of the bijection dropped a term that was the SAME CONSTANT on both sides. The
  // expected side never read a per-row leaf even before — it interpolated `candidate.sha`
  // directly — so the equality it tests is unchanged, only shorter.
  const expectedBindings = new Set(registry.capabilities.flatMap((cap) => array(cap.jurisdiction_bindings).map((binding) => `${binding.control_id}|${cap.id}`))); const actualTraces = new Set([...controls.values()].flatMap((control) => array(control.capability_traceability).map((trace) => `${control.id}|${trace.capability_id}`))); if (expectedBindings.size !== actualTraces.size || [...expectedBindings].some((tuple) => !actualTraces.has(tuple))) fail('Korea control trace is not an exact capability binding bijection');
    if (routeFacts) { const owners = registry.capabilities.flatMap((cap) => cap.route_presentation.route_keys.map((key) => `cap:${cap.id}:${key}`)).concat(array(registry.source_inventory?.unmodeled_keys).map((entry) => `unmodeled:${entry.key}`)); const keys=owners.map((entry) => entry.split(':').at(-1)); const actual=new Set(Object.keys(routeFacts.facts ?? {})); if (new Set(keys).size !== keys.length || keys.length !== actual.size || [...actual].some((key)=>!keys.includes(key))) fail('source route inventory is not a complete bijection'); }
  validateReleaseInventory(registry, resolveSource);
  validatedRegistries.set(registry, ledgerDigest(registry));
  return { capability_count: registry.capabilities.length, candidate_sha: candidate.sha, verdict: 'STRUCTURALLY_VALID_HOLD_PRESERVED' };
}
export function isValidatedConsoleTruthLedger(registry) { return validatedRegistries.get(registry) === ledgerDigest(registry); }

function main() {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
  const candidateSha = process.env.CONSOLE_CANDIDATE_SHA;
  const authorityTipSha = process.env.CONSOLE_AUTHORITY_TIP_SHA;
  const syntheticMergeSha = process.env.CONSOLE_SYNTHETIC_MERGE_SHA;
  if (!SHA.test(candidateSha ?? '')) fail('CONSOLE_CANDIDATE_SHA must be a full lowercase Git SHA');
  if (!SHA.test(authorityTipSha ?? '')) fail('CONSOLE_AUTHORITY_TIP_SHA must be a full lowercase Git SHA');
  if (!SHA.test(syntheticMergeSha ?? '')) fail('CONSOLE_SYNTHETIC_MERGE_SHA must be a full lowercase Git SHA');
  const train = verifyConsoleAuthorityTrain(root, candidateSha, authorityTipSha, syntheticMergeSha);
  // Release-please bot tips are docs-only (manifest + CHANGELOG). They have no signed product
  // candidate C and must not enter createConsoleCandidateSourceResolver's SSH/authority path.
  if (train.trainClass === RELEASE_PLEASE_TRAIN_CLASS) {
    console.log(JSON.stringify({
      verdict: 'RELEASE_PLEASE_BOT_CANDIDATE_ADMITTED',
      train_class: RELEASE_PLEASE_TRAIN_CLASS,
      candidate_sha: train.candidateSha,
      authority_tip_sha: train.authorityTipSha,
      synthetic_merge_sha: train.syntheticMergeSha,
    }, null, 2));
    return;
  }
  const registry = parseImmutableJson(git(root, ['show', `${authorityTipSha}:docs/program/console-capability-registry.json`]), 'console capability registry').value;
  const jurisdiction = parseImmutableJson(git(root, ['show', `${authorityTipSha}:docs/program/console-jurisdiction-register.json`]), 'console jurisdiction register').value;
  const candidateSource = createConsoleCandidateSourceResolver(root, candidateSha, authorityTipSha);
  const resolveBuckTarget = createConsoleBuckTargetResolver(candidateSource);
  const resolveSha = (value) => { try { execFileSync('git', ['cat-file', '-e', `${value}^{commit}`], { cwd: root, stdio: 'ignore' }); return true; } catch { return false; } };
  const resolveRef = (ref, expectedSha) => { try { return execFileSync('git', ['rev-parse', '--verify', `${ref}^{commit}`], { cwd: root, encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }).trim() === expectedSha; } catch { return false; } };
  const routeFacts = extractConsoleRouteFactsFromCandidate(candidateSource);
  console.log(JSON.stringify(validateConsoleTruthLedger(registry, jurisdiction, { expectedCandidateSha: candidateSha, resolveSha, resolveRef, resolveSource: candidateSource.resolveSource, resolveBuckTarget, routeFacts, repoRoot: root }), null, 2));
}
if (process.argv[1] === fileURLToPath(import.meta.url)) main();
