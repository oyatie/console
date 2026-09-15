// LC07 local orchestration contracts. These do not certify PostgreSQL effects,
// Kubernetes scheduling, enforcing CNI, published images or signing/provisioning.
import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import vm from 'node:vm';
const require = createRequire(import.meta.url);
const yaml = require('js-yaml');
const read = (p) => readFileSync(new URL(`../${p}`, import.meta.url), 'utf8');
const document = (p) => yaml.load(read(p));

function declaration(source, name) {
  const start = source.search(new RegExp(`^(?:async )?function ${name}\\(`, 'm'));
  assert.notEqual(start, -1, `required actual orchestration function ${name}`);
  const end = source.indexOf('\n}', start);
  assert.notEqual(end, -1);
  return source.slice(start, end + 2);
}

test('Compose gates both serving roles on finalization and confines admin transport', () => {
  const { services } = document('ops/compose.yml');
  const finalizer = services['account-finalize'];
  assert.ok(finalizer, 'actual one-shot finalizer service');
  assert.equal(finalizer.depends_on.migrate.condition, 'service_completed_successfully');
  assert.equal(finalizer.restart, 'no');
  for (const name of ['app', 'worker']) {
    assert.equal(services[name].depends_on['account-finalize']?.condition, 'service_completed_successfully');
    assert.doesNotMatch(JSON.stringify(services[name].environment), /POSTGRES_ADMIN|OPERATOR.*(?:PASSWORD|SECRET)|console_cluster_admin/);
  }
  assert.match(JSON.stringify(finalizer), /account-custody|account-finaliz/);
  assertComposeBoundary(document('ops/compose.yml'));
  assertOperatorImage(read('ops/account-custody.Dockerfile'));
});

for (const command of ['cmdUp', 'cmdBootstrap']) {
  for (const fail of [false, true]) {
   for (const devAuth of command === 'cmdBootstrap' ? [false, true] : [false]) {
    test(`${command} devAuth=${devAuth}: ${fail ? 'finalizer failure prevents seed and launch' : 'migration then finalization precedes seed and launch'}`, async () => {
      const trace = [];
      const stop = new Error('observed launch');
      const denied = new Error('injected finalizer failure');
      const context = {
        PORTS: { backend: 0 }, STATE_DIR: '/unused', REPO_ROOT: '/unused',
        process: { env: devAuth ? { CONSOLE_DEV_AUTH_E2E: '1' } : {}, platform: 'linux' }, path: { join: () => '/unused', relative: () => 'unused' },
        assertPortFree: async () => {}, bringUpDeps: async () => ({ selected: true }),
        reconcileDatabaseTopology: () => trace.push('topology'),
        buildAppBinary: () => ({ target: 'actual-selected-binary', outputPath: '/unused' }),
        runMigrations: () => trace.push('migrate'),
        finalizeDatabaseCustody: () => { trace.push('finalize'); if (fail) throw denied; },
        runSeed: () => trace.push('seed'), buildAppEnv: () => ({}), log: () => {},
        mkdirSync: () => {}, openSync: () => 1,
        spawn: () => { trace.push('spawn'); throw stop; },
      };
      vm.createContext(context);
      vm.runInContext(`${declaration(read('scripts/dev-up.mjs'), command)}\nglobalThis.run = ${command};`, context);
      await assert.rejects(context.run(), (error) => error === (fail ? denied : stop));
      assert.equal(trace.filter((v) => v === 'finalize').length, 1);
      assert.ok(trace.indexOf('migrate') < trace.indexOf('finalize'));
      if (fail) assert.deepEqual(trace, ['topology', 'migrate', 'finalize']);
      else {
        assert.ok(trace.indexOf('finalize') < trace.indexOf('spawn'));
        if (trace.includes('seed')) assert.ok(trace.indexOf('finalize') < trace.indexOf('seed'));
        assert.equal(trace.includes('seed'), command === 'cmdUp' || devAuth);
      }
    });
   }
  }
}

test('dev-up actual finalizer adapter propagates nonzero compose result', () => {
  const calls = [];
  const context = { log: () => {}, runCompose: (...args) => { calls.push(args); return { status: 19 }; },
    process: { env: {} }, REPO_ROOT: '/unused', PORTS: { postgres: 1234 }, POSTGRES_DB: 'selected',
    POSTGRES_ADMIN_USER: 'operator', POSTGRES_ADMIN_PASSWORD: 'synthetic',
    APP_POSTGRES_PASSWORD: 'synthetic-app', RT_POSTGRES_PASSWORD: 'synthetic-rt',
    AUTH_POSTGRES_PASSWORD: 'synthetic-auth',
    LEAVE_COMMAND_POSTGRES_PASSWORD: 'synthetic-leave', ONTOLOGY_COMMAND_POSTGRES_PASSWORD: 'synthetic-ontology',
    PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD: 'synthetic-force' };
  vm.createContext(context);
  vm.runInContext(`${declaration(read('scripts/dev-up.mjs'), 'commandDatabaseUrl')}\n${declaration(read('scripts/dev-up.mjs'), 'finalizeDatabaseCustody')}\nglobalThis.run = finalizeDatabaseCustody;`, context);
  assert.throws(() => context.run({ selected: true }), /finaliz|custody/i);
  assert.equal(calls.length, 1, 'must execute adapter, not just throw');
  assert.ok(calls[0][1].includes('account-finalize'));
  assertComposeInvocation(calls[0]);
});

test('Argo includes ordered separate operator job without app secret or API token', () => {
  const migration = document('deploy/apps/console/base/migrate-job.yaml');
  const finalizer = document('deploy/apps/console/base/account-finalize-job.yaml');
  const kustomization = document('deploy/apps/console/base/kustomization.yaml');
  assert.ok(kustomization.resources.includes('account-finalize-job.yaml'));
  assert.equal(migration.metadata.annotations['argocd.argoproj.io/hook'], 'PreSync');
  assert.equal(finalizer.metadata.annotations['argocd.argoproj.io/hook'], 'PreSync');
  assert.equal(String(migration.metadata.annotations['argocd.argoproj.io/sync-wave']), '-20');
  assert.equal(String(finalizer.metadata.annotations['argocd.argoproj.io/sync-wave']), '-10');
  assert.equal(finalizer.spec.template.spec.automountServiceAccountToken, false);
  assert.ok(finalizer.spec.activeDeadlineSeconds > 0);
  assert.ok(finalizer.spec.backoffLimit <= 2);
  assert.equal(finalizer.spec.template.spec.containers.length, 1);
  const operator = finalizer.spec.template.spec.containers[0];
  assert.notEqual(operator.image, migration.spec.template.spec.containers[0].image);
  assert.doesNotMatch(JSON.stringify(operator), /console-db-app|console-db-rt/);
  assert.ok(operator.env?.some((e) => e.valueFrom?.secretKeyRef) || operator.volumeMounts?.length,
    'actual secret/descriptor transport is required, not a prose comment');
  assertArgoBoundary(finalizer, migration, kustomization);
  const serving = ['deploy/apps/console/base/backend.yaml', 'deploy/apps/console/base/worker.yaml'].flatMap((path) => yaml.loadAll(read(path))).filter((doc) => doc?.spec?.template?.spec);
  assert.equal(serving.length, 2);
  assertKubernetesServingIsolation(finalizer, serving);
});

test('image smoke declares a gating finalizer between actual migration and application boot', () => {
  const workflow = document('.github/workflows/image-release.yml');
  const steps = workflow.jobs['release-probe'].steps;
  const executable = (step) => typeof step.run === 'string'
    ? step.run.split('\n').filter((line) => !/^\s*#/.test(line)).join('\n') : '';
  const migration = steps.findIndex((step) => /CONSOLE_APP_ROLE=migrate/.test(executable(step)));
  const boot = steps.findIndex((step) => /CONSOLE_APP_ROLE=api/.test(executable(step)));
  const finalizers = steps.map((step, index) => ({ step, index, code: executable(step) }))
    .filter(({ code }) => /postgres-finalize-account-custody|account-finalize/.test(code));
  assert.equal(finalizers.length, 1, 'one actual smoke finalization invocation, not a comment or unrelated job');
  const { step, index, code } = finalizers[0];
  assert.ok(migration >= 0 && migration < index && index < boot);
  assert.equal(step['continue-on-error'] ?? false, false);
  assert.equal(step.if, undefined, 'finalization must not be conditional within the smoke job');
  assert.doesNotMatch(code, /\|\|\s*true/);
  assert.match(code, /(?:^|\n)\s*(?:docker\s+run|bash\s+)/);
  assert.match(JSON.stringify(step), /console_release_probe|PROBE_.*(?:DATABASE|TARGET)/,
    'bind the actual serving DB; canonical_probe_release alone is insufficient');
  assertSmokeBoundary(step);
  assert.equal(step.env.PROBE_DATABASE, workflow.jobs['release-probe'].services.postgres.env.POSTGRES_DB, 'finalizer binds the actual serving probe database');
  // Manifest assertions are not execution of the published image or its database.
});

// These validators deliberately operate on parsed production declarations.
// Positive/hostile fixtures below exercise the same validators independently;
// they are checker controls, not fake deployments or wrapper execution proof.
const descriptorKeys = [
  'ACCOUNT_CUSTODY_EXPECTED_OPERATOR', 'ACCOUNT_CUSTODY_EXPECTED_SYSTEM_IDENTIFIER',
  'ACCOUNT_CUSTODY_EXPECTED_DATABASE', 'ACCOUNT_CUSTODY_EXPECTED_DATABASE_OID',
  'ACCOUNT_CUSTODY_EXPECTED_TLS_HOST',
];
const transportKeys = ['POSTGRES_HOST', 'POSTGRES_PORT', 'POSTGRES_DB', 'POSTGRES_ADMIN_USER'];
function environment(value = {}) {
  if (!Array.isArray(value)) return value;
  return Object.fromEntries(value.map((entry) => {
    const at = entry.indexOf('=');
    assert.ok(at > 0, 'unbound inherited environment is not an explicit operator contract');
    return [entry.slice(0, at), entry.slice(at + 1)];
  }));
}
function envFiles(service) {
  return (service.env_file ?? []).map((entry) => typeof entry === 'string' ? entry : entry.path);
}
function mounts(service) {
  return (service.volumes ?? []).map((entry) => {
    if (typeof entry === 'string') {
      // Protected Compose interpolation may itself contain a colon. Parse
      // the target from the right, not from the first interpolation colon.
      const match = entry.match(/^(.*):(\/[^:]+)(?::(ro|rw))?$/);
      assert.ok(match, `unresolved mount shape ${entry}`);
      return { source: match[1], target: match[2], read_only: match[3] === 'ro' };
    }
    return entry;
  });
}
function secretMounts(service) {
  return (service.secrets ?? []).map((entry) => typeof entry === 'string'
    ? { source: entry, target: `/run/secrets/${entry}`, read_only: true }
    : { source: entry.source, target: (entry.target ?? entry.source).startsWith('/') ? (entry.target ?? entry.source) : `/run/secrets/${entry.target ?? entry.source}`, read_only: true });
}
function assertComposeBoundary(compose) {
  const operator = compose.services['account-finalize'];
  assert.ok(operator, 'missing real finalizer service');
  assert.equal(operator.command, undefined, 'no caller command may replace fixed operator entrypoint');
  assert.equal(operator.entrypoint, undefined, 'dedicated image owns fixed entrypoint');
  assert.equal(operator.build?.dockerfile, 'ops/account-custody.Dockerfile');
  assert.equal(operator.build?.context, '..');
  const env = environment(operator.environment);
  const inventory = envFiles(operator);
  assert.equal(inventory.length, 1, 'one explicit independent operator inventory file');
  assert.match(inventory[0], /\$\{ACCOUNT_CUSTODY_TARGET_ENV_FILE:\?[^}]+\}/,
    'target inventory is required input, never derived from live connection');
  for (const key of [...descriptorKeys, ...transportKeys]) {
    assert.equal(env[key], undefined, `${key} must come from independently supplied inventory, not a DSN-derived override`);
  }
  assert.equal(env.PGSSLMODE ?? 'verify-full', 'verify-full');
  const attached = [...mounts(operator), ...secretMounts(operator)];
  for (const key of ['POSTGRES_ADMIN_PASSWORD_FILE', 'PGSSLROOTCERT']) {
    assert.equal(typeof env[key], 'string', `${key} must reference mounted file`);
    const matches = attached.filter((m) => m.target === env[key]);
    assert.equal(matches.length, 1, `exact mounted ${key}`);
    assert.equal(matches[0].read_only, true, `${key} must be read-only`);
  }
  assert.equal(env.POSTGRES_ADMIN_PASSWORD, undefined, 'no password value in operator environment');
  const protectedSources = attached.filter((m) => m.target === env.POSTGRES_ADMIN_PASSWORD_FILE).map((m) => m.source);
  for (const name of ['app', 'worker']) {
    const serving = compose.services[name];
    assert.ok(serving);
    assert.doesNotMatch(JSON.stringify(environment(serving.environment)), /POSTGRES_ADMIN|ACCOUNT_CUSTODY|console_cluster_admin|PGPASSWORD|PGPASSFILE/);
    assert.equal(envFiles(serving).length, 0,
      'uninspected serving env_file may import operator credentials; use explicit serving environment');
    for (const mount of [...mounts(serving), ...secretMounts(serving)]) {
      assert.ok(!protectedSources.includes(mount.source), `${name} receives operator secret source`);
      assert.doesNotMatch(JSON.stringify(mount), /account.custody|cluster.admin|postgres.admin|operator.secret|docker\.sock|postgres.socket/i);
    }
  }
}
function assertOperatorImage(source) {
  const lines = source.split('\n').map((line) => line.trim()).filter((line) => line && !line.startsWith('#'));
  const from = lines.filter((line) => line.startsWith('FROM '));
  assert.equal(from.length, 1, 'one pinned PostgreSQL client build stage');
  assert.match(from[0], /^FROM postgres:18\.6@sha256:[a-f0-9]{64}$/);
  const entryLines = lines.filter((line) => line.startsWith('ENTRYPOINT '));
  assert.equal(entryLines.length, 1);
  const entry = JSON.parse(entryLines[0].slice('ENTRYPOINT '.length));
  assert.equal(entry.length, 2);
  assert.equal(entry[0], 'bash');
  assert.match(entry[1], /^\/[a-zA-Z0-9_/-]+\/postgres-finalize-account-custody\.sh$/);
  const directory = entry[1].slice(0, entry[1].lastIndexOf('/'));
  assert.equal(lines.filter((line) => line.startsWith('COPY ')).length, 3, 'exact three candidate artifacts; no later replacement copy');
  assert.ok(!lines.some((line) => line.startsWith('RUN ')), 'fixed COPY packaging must not rewrite release-bound artifacts');
  for (const file of ['postgres-finalize-account-custody.sh', 'postgres-finalize-account-custody.sql', 'account-custody-migrations.sha384']) {
    assert.ok(lines.some((line) => {
      const words = line.split(/\s+/).filter((word) => !word.startsWith('--chmod='));
      return words.length === 3 && words[0] === 'COPY' && words[1] === `ops/${file}`
        && [directory + '/', `${directory}/${file}`].includes(words[2]);
    }), `exact candidate COPY for ${file} beside wrapper`);
  }
  assert.ok(!lines.some((line) => /^(?:ADD|CMD)\s/.test(line)), 'no downloaded artifacts or default caller command');
  assert.doesNotMatch(source, /\b(?:curl|wget|eval)\b|https?:\/\//);
}
function assertComposeInvocation(call) {
  const [selection, args] = call;
  assert.equal(selection.selected, true, 'adapter must use selected Compose stack');
  assert.ok(Array.isArray(args));
  assert.equal(args[0], 'run', 'must run one-shot service, not config/stop/up');
  assert.equal(args.at(-1), 'account-finalize');
  assert.ok(args.includes('--rm'));
  assert.ok(args.includes('--no-deps'), 'dev-up already ran real migrations; do not rerun prerequisites');
  assert.ok(args.slice(1, -1).every((arg) => ['--rm', '--no-deps', '-T'].includes(arg)), 'no shell/entrypoint/SQL override');
}
function assertArgoBoundary(finalizer, migration, kustomization) {
  const spec = finalizer.spec.template.spec;
  const operator = spec.containers[0];
  assert.match(operator.image, /account-custody|account-finaliz/);
  assert.equal(operator.command, undefined, 'operator image fixed entrypoint cannot be overridden');
  assert.equal(operator.args, undefined, 'operator accepts no caller SQL');
  assert.equal(operator.envFrom, undefined, 'explicit auditable operator environment keys');
  const env = new Map((operator.env ?? []).map((e) => [e.name, e]));
  assert.equal(env.size, (operator.env ?? []).length, 'duplicate environment keys');
  const descriptorRefs = [...descriptorKeys, ...transportKeys].map((key) => {
    const entry = env.get(key);
    assert.ok(entry?.valueFrom?.configMapKeyRef, `${key} comes from protected independent prerequisite inventory`);
    assert.equal(entry.value, undefined);
    assert.notEqual(entry.valueFrom.configMapKeyRef.optional, true, 'missing inventory fails closed');
    return entry.valueFrom.configMapKeyRef;
  });
  assert.equal(new Set(descriptorRefs.map((ref) => ref.name)).size, 1, 'one bound inventory');
  for (let i = 0; i < descriptorRefs.length; ++i) assert.equal(descriptorRefs[i].key, [...descriptorKeys, ...transportKeys][i]);
  assert.equal(spec.initContainers, undefined, 'must not derive expected inventory from target readback');
  assert.equal(env.get('POSTGRES_ADMIN_PASSWORD'), undefined);
  const volumes = spec.volumes ?? [];
  const mounted = operator.volumeMounts ?? [];
  for (const key of ['POSTGRES_ADMIN_PASSWORD_FILE', 'PGSSLROOTCERT']) {
    const file = env.get(key)?.value;
    assert.equal(typeof file, 'string');
    const mount = mounted.find((m) => file.startsWith(`${m.mountPath}/`) || file === m.mountPath);
    assert.ok(mount?.readOnly, `${key} must use read-only prerequisite volume`);
    const volume = volumes.find((v) => v.name === mount.name);
    assert.ok(volume?.secret, `${key} requires explicitly provisioned secret volume`);
    assert.notEqual(volume.secret.optional, true);
    assert.doesNotMatch(volume.secret.secretName, /console-db-app|console-db-rt/);
  }
  assert.equal(env.get('PGSSLMODE')?.value ?? 'verify-full', 'verify-full');
  assert.ok(!kustomization.resources.some((r) => /account.*(?:inventory|operator-secret|target-descriptor)/.test(r)),
    'normal Sync resource cannot supply a first PreSync prerequisite');
  // Local reference/payload checks are not image signing or remote inventory
  // provisioning. Published digest/signature and enforcing CNI remain exposure gates.
}

// A deliberately narrow shell subset: one docker run invocation, multiline
// continuations and double-quoted variable words. Single quotes, redirections,
// operators/subshells/extra commands
// are rejected rather than pretending to parse arbitrary executable shell.
function smokeWords(code) {
  const source = code.replace(/\\\n/g, ' ').trim();
  assert.doesNotMatch(source, /[;|&`'<>\n]|\$\(/, 'smoke finalizer step must be one auditable invocation');
  const words = source.match(/"[^"\n]*"|[^\s]+/g) ?? [];
  assert.ok(words.every((word) => !/["']/.test(word.slice(1, -1))), 'unsupported shell quoting');
  return words.map((word) => /^["']/.test(word) ? word.slice(1, -1) : word);
}
function assertSmokeBoundary(step) {
  const words = smokeWords(step.run);
  assert.deepEqual(words.slice(0, 2), ['docker', 'run']);
  assert.ok(words.includes('--rm'));
  const env = step.env ?? {};
  assert.ok(env.ACCOUNT_CUSTODY_OPERATOR_IMAGE, 'actual dedicated operator image binding');
  assert.match(env.ACCOUNT_CUSTODY_OPERATOR_IMAGE, /account-custody|account-finaliz/);
  assert.ok(env.ACCOUNT_CUSTODY_DESCRIPTOR_FILE, 'pre-existing independent smoke target inventory');
  assert.ok(env.ACCOUNT_CUSTODY_PASSWORD_FILE);
  assert.ok(env.ACCOUNT_CUSTODY_CA_FILE);
  assert.ok(env.PROBE_DATABASE, 'same DB name as actual image serving probe');
  assert.equal(words.at(-1), '${ACCOUNT_CUSTODY_OPERATOR_IMAGE}', 'fixed operator entrypoint, no echo/shell/command after image');
  const pairs = new Map();
  const mounts = [];
  for (let i = 2; i < words.length - 1; ++i) {
    if (['--rm', '--read-only'].includes(words[i])) continue;
    assert.ok(['--env-file', '--env', '--volume', '--network', '--tmpfs'].includes(words[i]), `unexpected docker option ${words[i]}`);
    const flag = words[i++]; const value = words[i]; assert.ok(value);
    if (flag === '--volume') mounts.push(value);
    else if (flag === '--env') { const at = value.indexOf('='); assert.ok(at > 0); assert.ok(!pairs.has(value.slice(0, at))); pairs.set(value.slice(0, at), value.slice(at + 1)); }
    else { assert.ok(!pairs.has(flag)); pairs.set(flag, value); }
  }
  assert.equal(pairs.get('--tmpfs'), '/tmp:rw,noexec,nosuid,size=16m', 'fixed bounded private temporary storage required by wrapper');
  assert.equal(pairs.get('--env-file'), '${ACCOUNT_CUSTODY_DESCRIPTOR_FILE}');
  assert.equal(pairs.get('POSTGRES_DB'), '${PROBE_DATABASE}');
  for (const key of descriptorKeys) assert.ok(!pairs.has(key), 'expected identity must come from independent descriptor, not current probe overrides');
  for (const [key, source] of [['POSTGRES_ADMIN_PASSWORD_FILE', 'ACCOUNT_CUSTODY_PASSWORD_FILE'], ['PGSSLROOTCERT', 'ACCOUNT_CUSTODY_CA_FILE']]) {
    const target = pairs.get(key); assert.match(target ?? '', /^\/run\/[a-zA-Z0-9_/-]+$/);
    assert.ok(mounts.includes(`\${${source}}:${target}:ro`), `readonly protected ${key} transport`);
  }
  assert.equal(pairs.get('PGSSLMODE') ?? 'verify-full', 'verify-full');
}

function nominalCompose() {
  return { services: {
    'account-finalize': {
      build: { context: '..', dockerfile: 'ops/account-custody.Dockerfile' },
      env_file: ['${ACCOUNT_CUSTODY_TARGET_ENV_FILE:?supply protected inventory}'],
      environment: { POSTGRES_ADMIN_PASSWORD_FILE: '/run/operator/password', PGSSLROOTCERT: '/run/operator/ca' },
      volumes: ['${ACCOUNT_CUSTODY_PASSWORD_FILE:?supply password}:/run/operator/password:ro', '${ACCOUNT_CUSTODY_CA_FILE:?supply CA}:/run/operator/ca:ro'],
    }, app: { environment: {} }, worker: { environment: {} },
  } };
}
function nominalArgo() {
  return { spec: { template: { spec: {
    containers: [{ image: 'console-account-custody:local-unpublished', env: [
      ...[...descriptorKeys, ...transportKeys].map((name) => ({ name, valueFrom: { configMapKeyRef: { name: 'protected-operator-inventory', key: name } } })),
      { name: 'POSTGRES_ADMIN_PASSWORD_FILE', value: '/run/operator/password' },
      { name: 'PGSSLROOTCERT', value: '/run/ca/root.crt' },
    ], volumeMounts: [{ name: 'operator', mountPath: '/run/operator', readOnly: true }, { name: 'ca', mountPath: '/run/ca', readOnly: true }] }],
    volumes: [{ name: 'operator', secret: { secretName: 'operator-only' } }, { name: 'ca', secret: { secretName: 'operator-ca' } }],
  } } } };
}
function nominalSmoke() {
  return { env: {
    ACCOUNT_CUSTODY_OPERATOR_IMAGE: 'console-account-custody:local-unpublished',
    ACCOUNT_CUSTODY_DESCRIPTOR_FILE: '/protected/probe-target.env',
    ACCOUNT_CUSTODY_PASSWORD_FILE: '/protected/operator-password',
    ACCOUNT_CUSTODY_CA_FILE: '/protected/ca.pem', PROBE_DATABASE: 'console_release_probe',
  }, run: 'docker run --rm --read-only --tmpfs /tmp:rw,noexec,nosuid,size=16m --network host --env-file "${ACCOUNT_CUSTODY_DESCRIPTOR_FILE}" --env "POSTGRES_DB=${PROBE_DATABASE}" --env POSTGRES_ADMIN_PASSWORD_FILE=/run/operator/password --env PGSSLROOTCERT=/run/operator/ca --volume "${ACCOUNT_CUSTODY_PASSWORD_FILE}:/run/operator/password:ro" --volume "${ACCOUNT_CUSTODY_CA_FILE}:/run/operator/ca:ro" "${ACCOUNT_CUSTODY_OPERATOR_IMAGE}"' };
}
const nominalDockerfile = `FROM postgres:18.6@sha256:${'a'.repeat(64)}
COPY ops/postgres-finalize-account-custody.sh /some/fixed/path/
COPY ops/postgres-finalize-account-custody.sql /some/fixed/path/
COPY ops/account-custody-migrations.sha384 /some/fixed/path/
ENTRYPOINT ["bash","/some/fixed/path/postgres-finalize-account-custody.sh"]`;

test('LC07 machinery: nominal controls accept unrelated fixed image path and unpublished local image', () => {
  assertComposeBoundary(nominalCompose());
  assertArgoBoundary(nominalArgo(), {}, { resources: [] });
  assertSmokeBoundary(nominalSmoke());
  assertOperatorImage(nominalDockerfile);
  assertComposeInvocation([{ selected: true }, ['run', '--rm', '--no-deps', 'account-finalize']]);
});

for (const [name, mutate] of [
  ['sleep-only finalizer', (x) => { x.services['account-finalize'].command = ['sleep', '1']; }],
  ['replacement entrypoint', (x) => { x.services['account-finalize'].entrypoint = ['echo', 'done']; }],
  ['missing independent inventory', (x) => { delete x.services['account-finalize'].env_file; }],
  ['DSN-derived expected OID override', (x) => { x.services['account-finalize'].environment.ACCOUNT_CUSTODY_EXPECTED_DATABASE_OID = '${CURRENT_DATABASE_OID}'; }],
  ['missing CA', (x) => { delete x.services['account-finalize'].environment.PGSSLROOTCERT; }],
  ['writable password mount', (x) => { x.services['account-finalize'].volumes[0] = x.services['account-finalize'].volumes[0].replace(':ro', ':rw'); }],
  ['password environment exposure', (x) => { x.services.app.environment.PGPASSWORD = 'synthetic'; }],
  ['serving env_file imports unknown secrets', (x) => { x.services.worker.env_file = ['/protected/operator.env']; }],
  ['admin volume renamed in application', (x) => { x.services.app.volumes = ['${ACCOUNT_CUSTODY_PASSWORD_FILE:?supply password}:/harmless/name:ro']; }],
  ['admin secret reference', (x) => { x.services.worker.secrets = ['cluster-admin']; }],
]) {
  test(`LC07 machinery: Compose rejects ${name}`, () => {
    const fixture = nominalCompose(); mutate(fixture);
    assert.throws(() => assertComposeBoundary(fixture), { name: 'AssertionError' });
  });
}
for (const [name, mutate] of [
  ['unrelated image', (x) => { x.spec.template.spec.containers[0].image = 'alpine:latest'; }],
  ['sleep command', (x) => { x.spec.template.spec.containers[0].command = ['sleep', '1']; }],
  ['unrelated secret instead of target descriptor', (x) => { x.spec.template.spec.containers[0].env[0].valueFrom = { secretKeyRef: { name: 'anything', key: 'anything' } }; }],
  ['literal target OID', (x) => { const e = x.spec.template.spec.containers[0].env.find(e => e.name === 'ACCOUNT_CUSTODY_EXPECTED_DATABASE_OID'); delete e.valueFrom; e.value = '1234'; }],
  ['optional target inventory', (x) => { x.spec.template.spec.containers[0].env[0].valueFrom.configMapKeyRef.optional = true; }],
  ['target-readback init container', (x) => { x.spec.template.spec.initContainers = [{ name: 'discover', image: 'postgres', command: ['psql'] }]; }],
  ['serving secret reused', (x) => { x.spec.template.spec.volumes[0].secret.secretName = 'console-db-app'; }],
  ['unverified TLS override', (x) => { x.spec.template.spec.containers[0].env.push({ name: 'PGSSLMODE', value: 'require' }); }],
]) {
  test(`LC07 machinery: Argo rejects ${name}`, () => {
    const fixture = nominalArgo(); mutate(fixture);
    assert.throws(() => assertArgoBoundary(fixture, {}, { resources: [] }), { name: 'AssertionError' });
  });
}
for (const [name, mutate] of [
  ['echo-only false finalization', (x) => { x.run = 'docker run alpine echo account-finalize console_release_probe'; }],
  ['trailing caller command', (x) => { x.run += ' echo done'; }],
  ['wrong database override', (x) => { x.run = x.run.replace('POSTGRES_DB=${PROBE_DATABASE}', 'POSTGRES_DB=canonical_probe_release'); }],
  ['missing inventory mount', (x) => { x.run = x.run.replace('--env-file "${ACCOUNT_CUSTODY_DESCRIPTOR_FILE}"', ''); }],
  ['ignored failure', (x) => { x.run += ' || true'; }],
  ['recomputed target descriptor', (x) => { x.run = x.run.replace('--rm', '--rm --env ACCOUNT_CUSTODY_EXPECTED_DATABASE_OID=1234'); }],
  ['entrypoint replacement', (x) => { x.run = x.run.replace('--rm', '--rm --entrypoint echo'); }],
  ['literal single-quoted image variable', (x) => { x.run = x.run.replace('"${ACCOUNT_CUSTODY_OPERATOR_IMAGE}"', "'${ACCOUNT_CUSTODY_OPERATOR_IMAGE}'"); }],
  ['unquoted output redirection', (x) => { x.run = x.run.replace('--network host', '--network host>/tmp/ignored'); }],
  ['unquoted input redirection', (x) => { x.run = x.run.replace('--network host', '--network host</tmp/ignored'); }],
]) {
  test(`LC07 machinery: smoke rejects ${name}`, () => {
    const fixture = nominalSmoke(); mutate(fixture);
    assert.throws(() => assertSmokeBoundary(fixture), { name: 'AssertionError' });
  });
}
test('LC07 machinery: packaging rejects omitted checksum, mutable base and mismatched wrapper path', () => {
  for (const bad of [nominalDockerfile.replace('COPY ops/account-custody-migrations.sha384 /some/fixed/path/\n', ''), nominalDockerfile.replace(/@sha256:[a-f0-9]{64}/, ''), nominalDockerfile.replace('"/some/fixed/path/postgres-finalize', '"/different/postgres-finalize'), nominalDockerfile + '\nCOPY forged.sql /some/fixed/path/postgres-finalize-account-custody.sql', nominalDockerfile + '\nRUN echo forged > /some/fixed/path/postgres-finalize-account-custody.sql']) {
    assert.throws(() => assertOperatorImage(bad), { name: 'AssertionError' });
  }
});
test('LC07 machinery: wrong Compose subcommand, selected stack and caller override fail', () => {
  for (const call of [[{ selected: true }, ['stop', 'account-finalize']], [{ other: true }, ['run', '--rm', '--no-deps', 'account-finalize']], [{ selected: true }, ['run', '--rm', '--no-deps', '--entrypoint', 'echo', 'account-finalize']]]) {
    assert.throws(() => assertComposeInvocation(call), { name: 'AssertionError' });
  }
});

for (const status of [0, null]) {
  test(`dev-up actual finalizer adapter ${status === 0 ? 'returns on successful run' : 'rejects spawn failure'}`, () => {
    const calls = [];
    const context = { log: () => {}, runCompose: (...args) => { calls.push(args); return { status }; },
      process: { env: {} }, REPO_ROOT: '/unused', PORTS: { postgres: 1234 }, POSTGRES_DB: 'selected',
      POSTGRES_ADMIN_USER: 'operator', POSTGRES_ADMIN_PASSWORD: 'synthetic',
      APP_POSTGRES_PASSWORD: 'synthetic-app', RT_POSTGRES_PASSWORD: 'synthetic-rt',
      AUTH_POSTGRES_PASSWORD: 'synthetic-auth',
      LEAVE_COMMAND_POSTGRES_PASSWORD: 'synthetic-leave', ONTOLOGY_COMMAND_POSTGRES_PASSWORD: 'synthetic-ontology',
      PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD: 'synthetic-force' };
    vm.createContext(context);
    vm.runInContext(`${declaration(read('scripts/dev-up.mjs'), 'commandDatabaseUrl')}\n${declaration(read('scripts/dev-up.mjs'), 'finalizeDatabaseCustody')}\nglobalThis.run = finalizeDatabaseCustody;`, context);
    if (status === 0) assert.doesNotThrow(() => context.run({ selected: true }));
    else assert.throws(() => context.run({ selected: true }), /finaliz|custody/i);
    assert.equal(calls.length, 1);
    assertComposeInvocation(calls[0]);
  });
}

function referencedSecrets(value, result = new Set()) {
  if (!value || typeof value !== 'object') return result;
  for (const ref of [value.secretName, value.secretKeyRef?.name, value.secretRef?.name, value.secret?.name]) if (ref) result.add(ref);
  for (const child of Object.values(value)) referencedSecrets(child, result);
  return result;
}
function assertKubernetesServingIsolation(finalizer, serving) {
  const spec = finalizer.spec.template.spec;
  const operator = spec.containers[0];
  const passwordPath = operator.env.find((e) => e.name === 'POSTGRES_ADMIN_PASSWORD_FILE').value;
  const mount = operator.volumeMounts.find((m) => passwordPath.startsWith(`${m.mountPath}/`));
  assert.ok(mount);
  const secret = spec.volumes.find((v) => v.name === mount.name).secret.secretName;
  for (const deployment of serving) {
    const pod = deployment.spec.template.spec;
    assert.ok(!referencedSecrets(pod).has(secret), 'serving pod receives operator secret through env, volume or projected mount');
    for (const container of [...(pod.containers ?? []), ...(pod.initContainers ?? [])]) {
      assert.doesNotMatch(JSON.stringify(container.env ?? []), /POSTGRES_ADMIN|ACCOUNT_CUSTODY|console_cluster_admin|PGPASSWORD|PGPASSFILE/);
    }
  }
}
test('LC07 machinery: Kubernetes serving secret alias via projected volume is rejected', () => {
  const safe = { spec: { template: { spec: { containers: [{ env: [] }] } } } };
  assertKubernetesServingIsolation(nominalArgo(), [safe]);
  const unsafe = structuredClone(safe);
  unsafe.spec.template.spec.volumes = [{ name: 'innocent', projected: { sources: [{ secret: { name: 'operator-only' } }] } }];
  assert.throws(() => assertKubernetesServingIsolation(nominalArgo(), [unsafe]), { name: 'AssertionError' });
});


for (const [name, replacement] of [
  ['missing private temp', ''],
  ['unbounded temp', '--tmpfs /tmp:rw,noexec,nosuid'],
  ['executable temp', '--tmpfs /tmp:rw,nosuid,size=16m'],
  ['wrong temp path', '--tmpfs /scratch:rw,noexec,nosuid,size=16m'],
]) {
  test(`LC07 machinery: smoke rejects ${name}`, () => {
    const fixture = nominalSmoke();
    fixture.run = fixture.run.replace('--tmpfs /tmp:rw,noexec,nosuid,size=16m', replacement);
    assert.throws(() => assertSmokeBoundary(fixture), { name: 'AssertionError' });
  });
}

for (const role of ['api', 'worker', 'migrate']) {
  test(`dev-up actual buildAppEnv ${role} confines ambient operator environment`, () => {
    const forbidden = [
      'CONSOLE_POSTGRES_ADMIN_USER', 'CONSOLE_POSTGRES_ADMIN_PASSWORD',
      'CONSOLE_POSTGRES_ADMIN_PASSWORD_FILE', 'POSTGRES_ADMIN_USER',
      'POSTGRES_ADMIN_PASSWORD', 'POSTGRES_ADMIN_PASSWORD_FILE',
      'ACCOUNT_CUSTODY_TARGET_ENV_FILE', 'ACCOUNT_CUSTODY_PASSWORD_FILE',
      'ACCOUNT_CUSTODY_CA_FILE', 'ACCOUNT_CUSTODY_EXPECTED_OPERATOR',
      'ACCOUNT_CUSTODY_EXPECTED_SYSTEM_IDENTIFIER', 'ACCOUNT_CUSTODY_EXPECTED_DATABASE',
      'ACCOUNT_CUSTODY_EXPECTED_DATABASE_OID', 'ACCOUNT_CUSTODY_EXPECTED_TLS_HOST',
      'PGPASSWORD', 'PGPASSFILE', 'PGSERVICE', 'PGSERVICEFILE', 'PGSYSCONFDIR',
      'PGUSER', 'PGDATABASE', 'PGHOST', 'PGHOSTADDR', 'PGPORT', 'PGOPTIONS',
      'PGSSLROOTCERT', 'PGSSLCERT', 'PGSSLKEY', 'PGSSLMODE', 'PGGSSENCMODE',
    ];
    const ambient = Object.fromEntries(forbidden.map((key) => [key, `operator-only-${key}`]));
    Object.assign(ambient, { PATH: '/safe/bin', RUST_LOG: 'warn', CONSOLE_EMAIL_STUB_MODE: 'test', UNRELATED_APP_SETTING: 'preserve-me' });
    const before = { ...ambient };
    const context = {
      process: { env: ambient },
      ensureDevKeys: () => ({ privateKeyPem: 'app-private', publicKeyPem: 'app-public' }),
      databaseUrl: () => 'postgres://migration-owner/selected',
      runtimeDatabaseUrl: () => 'postgres://runtime/selected',
      commandDatabaseUrl: (name) => `postgres://${name}/selected`,
      LEAVE_COMMAND_POSTGRES_PASSWORD: 'leave-app', ONTOLOGY_COMMAND_POSTGRES_PASSWORD: 'ontology-app',
      PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD: 'force-app', AUTH_POSTGRES_PASSWORD: 'auth-app',
      PORTS: { backend: 8080, otel: 4317, s3: 8333, moxWebapi: 1080, office: 8090 },
      OFFICE_ENABLED: false,
    };
    vm.createContext(context);
    vm.runInContext(`${declaration(read('scripts/dev-up.mjs'), 'buildAppEnv')}\nglobalThis.run = buildAppEnv;`, context);
    const env = context.run(role);
    for (const key of forbidden) assert.equal(Object.hasOwn(env, key), false, `${role} inherits ${key}`);
    assert.equal(env.PATH, '/safe/bin');
    assert.equal(env.RUST_LOG, 'warn');
    assert.equal(env.CONSOLE_EMAIL_STUB_MODE, 'test');
    assert.equal(env.UNRELATED_APP_SETTING, 'preserve-me');
    assert.equal(env.CONSOLE_APP_ROLE, role);
    assert.equal(env.DATABASE_URL, role === 'migrate' ? 'postgres://migration-owner/selected' : 'postgres://runtime/selected');
    assert.deepEqual(ambient, before, 'child sanitization must not mutate the operator parent environment');
  });
}
