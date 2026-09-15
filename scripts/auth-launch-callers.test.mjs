// Auth caller configuration evidence only: VM executes actual dev adapters;
// Compose renders configuration; workflow checks inspect executable step text.
// None proves database authentication, same-cluster binding, or production HA.
import assert from 'node:assert/strict';
import test from 'node:test';
import vm from 'node:vm';
import { createRequire } from 'node:module';
import { readFileSync, mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import path from 'node:path';
const yaml = createRequire(import.meta.url)('js-yaml');
const root = new URL('../', import.meta.url);
const read = (p) => readFileSync(new URL(p, root), 'utf8');
const document = (p) => yaml.load(read(p));
const dev = read('scripts/dev-up.mjs');

// Same actual-declaration boundary used by account-custody-orchestration tests.
function declaration(source, name) {
  const start = source.search(new RegExp(`^(?:async )?function ${name}\\(`, 'm'));
  assert.notEqual(start, -1, `required actual adapter ${name}`);
  const end = source.indexOf('\n}', start);
  assert.notEqual(end, -1);
  return source.slice(start, end + 2);
}
const authPassword = 'auth:@/?#%+ value';
function fixture(ambient = {}) {
  const logs = [], calls = [];
  const context = {
    process: { env: { ...ambient } }, URL, encodeURIComponent, path,
    POSTGRES_ADMIN_USER: 'operator', POSTGRES_ADMIN_PASSWORD: 'admin-distinct',
    APP_POSTGRES_PASSWORD: 'owner:@/?#%+ value', RT_POSTGRES_PASSWORD: 'runtime:@/?#%+ value',
    AUTH_POSTGRES_PASSWORD: authPassword,
    LEAVE_COMMAND_POSTGRES_PASSWORD: 'leave-distinct', ONTOLOGY_COMMAND_POSTGRES_PASSWORD: 'ontology-distinct',
    PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD: 'force-distinct', POSTGRES_DB: 'selected',
    PORTS: { postgres: 5544, backend: 8080, otel: 4317, s3: 8333, moxWebapi: 1080, office: 8090 },
    SECRETS_DIR: '/unused/secrets', REPO_ROOT: '/unused', PID_FILE: '/unused/pid',
    DEPS_SERVICES: ['postgres'], OFFICE_ENABLED: false, OFFICE_JWT_SECRET: 'synthetic-office',
    ensureDevKeys: () => ({ privateKeyPem: 'app-private', publicKeyPem: 'app-public' }),
    log: (message) => logs.push(message), detectCompose: () => ({ selected: true }),
    prepareLocalCustody: () => {}, runtimeBin: () => 'unexecuted-container-runtime',
    waitForContainersHealthy: async () => {}, ensureBucket: async () => {},
    readPidState: () => null, existsSync: () => false,
    runCompose: (...args) => { calls.push(args); return { status: 0 }; },
  };
  vm.createContext(context);
  const names = ['databaseUrl', 'runtimeDatabaseUrl', 'commandDatabaseUrl', 'buildAppEnv',
    'bringUpDeps', 'reconcileDatabaseTopology', 'finalizeDatabaseCustody', 'cmdDown'];
  vm.runInContext(names.map((name) => declaration(dev, name)).join('\n'), context, { timeout: 1000 });
  return { context, calls, logs };
}
function assertUrl(raw, role, password, host, port) {
  const url = new URL(raw);
  assert.ok(['postgres:', 'postgresql:'].includes(url.protocol));
  assert.equal(decodeURIComponent(url.username), role);
  assert.ok(decodeURIComponent(url.password) === password, 'URL must carry exact original credential');
  assert.equal(url.hostname, host); assert.equal(url.port, String(port));
  assert.equal(url.pathname, '/selected'); assert.equal(url.search, ''); assert.equal(url.hash, '');
}
function noSecretLogs(logs) {
  assert.ok(!JSON.stringify(logs).includes(authPassword), 'raw auth password must not be logged');
  assert.ok(!JSON.stringify(logs).includes(encodeURIComponent(authPassword)), 'encoded credential must not be logged');
}

test('actual dev credential definitions reuse canonical override and distinct local fallback', () => {
  const source = dev.slice(dev.indexOf('const POSTGRES_ADMIN_PASSWORD ='), dev.indexOf('\nfunction log('));
  for (const configured of [false, true]) {
    const context = { process: { env: configured ? { CONSOLE_AUTH_POSTGRES_PASSWORD: authPassword } : {} } };
    vm.createContext(context);
    vm.runInContext(`${source}\nglobalThis.auth = typeof AUTH_POSTGRES_PASSWORD === 'undefined' ? undefined : AUTH_POSTGRES_PASSWORD;\nglobalThis.others = [POSTGRES_ADMIN_PASSWORD, APP_POSTGRES_PASSWORD, RT_POSTGRES_PASSWORD, LEAVE_COMMAND_POSTGRES_PASSWORD, ONTOLOGY_COMMAND_POSTGRES_PASSWORD, PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD];`, context, { timeout: 1000 });
    assert.equal(typeof context.auth, 'string', 'canonical dev auth credential must be defined');
    assert.ok(context.auth.length > 0);
    assert.ok(!context.others.includes(context.auth), 'auth credential is separate from every other role');
    if (configured) assert.ok(context.auth === authPassword, 'canonical override must survive byte-exactly');
  }
});

test('actual dev URL helpers preserve reserved credentials for owner runtime and narrow roles', () => {
  const { context } = fixture();
  assertUrl(context.databaseUrl(), 'console_app', context.APP_POSTGRES_PASSWORD, '127.0.0.1', 5544);
  assertUrl(context.runtimeDatabaseUrl(), 'console_rt', context.RT_POSTGRES_PASSWORD, '127.0.0.1', 5544);
  for (const role of ['console_auth_rt', 'console_leave_cmd', 'console_ontology_cmd', 'console_platform_force_cmd']) {
    assertUrl(context.commandDatabaseUrl(role, authPassword), role, authPassword, '127.0.0.1', 5544);
  }
});

test('actual shared dev URL helper preserves a database name with reserved characters', () => {
  const { context } = fixture();
  context.POSTGRES_DB = 'selected-name?#% value';
  context.APP_POSTGRES_PASSWORD = 'owner-safe'; context.RT_POSTGRES_PASSWORD = 'runtime-safe';
  for (const raw of [context.databaseUrl(), context.runtimeDatabaseUrl(), context.commandDatabaseUrl('console_auth_rt', 'auth-safe')]) {
    const url = new URL(raw);
    assert.equal(decodeURIComponent(url.pathname.slice(1)), context.POSTGRES_DB);
    assert.equal(url.search, ''); assert.equal(url.hash, '');
  }
});

for (const role of ['api', 'worker', 'migrate']) {
  test(`actual dev ${role} receives only its required auth transport and no inherited password`, () => {
    const ambient = { AUTH_DATABASE_URL: 'postgres://console_auth_rt:wrong@wrong-db:6543/wrong',
      CONSOLE_AUTH_POSTGRES_PASSWORD: authPassword, PATH: '/safe/bin', UNRELATED_APP_SETTING: 'preserve' };
    const { context, logs } = fixture(ambient);
    const before = JSON.stringify(context.process.env);
    const env = context.buildAppEnv(role);
    assert.equal(Object.hasOwn(env, 'CONSOLE_AUTH_POSTGRES_PASSWORD'), false, 'raw auth password is not a child variable');
    if (role === 'api') assertUrl(env.AUTH_DATABASE_URL, 'console_auth_rt', authPassword, '127.0.0.1', 5544);
    else assert.equal(Object.hasOwn(env, 'AUTH_DATABASE_URL'), false, 'non-API child must not inherit auth bearer transport');
    assert.equal(env.PATH, '/safe/bin'); assert.equal(env.UNRELATED_APP_SETTING, 'preserve');
    assert.equal(JSON.stringify(context.process.env), before, 'parent environment remains unchanged');
    noSecretLogs(logs);
  });
}

for (const name of ['bringUpDeps', 'reconcileDatabaseTopology', 'finalizeDatabaseCustody', 'cmdDown']) {
  test(`actual dev ${name} passes same auth credential and container target to Compose model`, async () => {
    const { context, calls, logs } = fixture({ AUTH_DATABASE_URL: 'postgres://wrong@wrong/wrong' });
    await context[name]({ selected: true });
    assert.ok(calls.length > 0, 'actual adapter must reach its Compose boundary');
    for (const [, , options] of calls) {
      assert.ok(options.env.CONSOLE_AUTH_POSTGRES_PASSWORD === authPassword, 'topology credential must match host API credential');
      // This is Compose interpolation input, not finalizer/container env.
      assertUrl(options.env.AUTH_DATABASE_URL, 'console_auth_rt', authPassword, 'postgres', 5432);
    }
    noSecretLogs(logs);
  });
}

function composeEnvironment(directory) {
  const values = {
    ACCOUNT_CUSTODY_TARGET_ENV_FILE: path.join(directory, 'target.env'),
    ACCOUNT_CUSTODY_PASSWORD_FILE: path.join(directory, 'password'),
    ACCOUNT_CUSTODY_CA_FILE: path.join(directory, 'ca.crt'), ACCOUNT_CUSTODY_PG_TLS_DIR: directory,
    CONSOLE_POSTGRES_ADMIN_PASSWORD: 'admin-distinct', CONSOLE_APP_POSTGRES_PASSWORD: 'owner-distinct',
    CONSOLE_RT_POSTGRES_PASSWORD: 'runtime-distinct', CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD: 'leave-distinct',
    CONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD: 'ontology-distinct', CONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD: 'force-distinct',
    CONSOLE_AUTH_POSTGRES_PASSWORD: authPassword, CONSOLE_POSTGRES_DB: 'selected',
    AUTH_DATABASE_URL: `postgres://console_auth_rt:${encodeURIComponent(authPassword)}@postgres:5432/selected`,
  };
  const ambient = Object.fromEntries(Object.entries(process.env).filter(([key]) =>
    !key.startsWith('CONSOLE_') && !key.startsWith('ACCOUNT_CUSTODY_') && key !== 'AUTH_DATABASE_URL'));
  return { ...ambient, ...values };
}
function renderCompose(env) {
  return spawnSync('docker', ['compose', '-f', 'ops/compose.yml', 'config', '--format', 'json'],
    { cwd: root, env, encoding: 'utf8', timeout: 15000, maxBuffer: 4 * 1024 * 1024 });
}

test('actual Compose render binds explicit AUTH URL only to API and raw credential only to topology', (t) => {
  const declared = document('ops/compose.yml').services;
  assert.match(declared.app.environment.AUTH_DATABASE_URL ?? '', /^\$\{AUTH_DATABASE_URL:\?/,
    'Compose requires a pre-encoded complete AUTH URL; raw password interpolation is not URL encoding');
  const directory = mkdtempSync(path.join(tmpdir(), 'console-auth-compose-'));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  for (const file of ['target.env', 'password', 'ca.crt']) writeFileSync(path.join(directory, file), file === 'target.env' ? '' : 'synthetic', { mode: 0o600 });
  const env = composeEnvironment(directory);
  const rendered = renderCompose(env);
  assert.equal(rendered.status, 0, 'actual Compose render must succeed; missing Compose is a prerequisite, not a skip');
  const services = JSON.parse(rendered.stdout).services;
  assertUrl(services.app.environment.AUTH_DATABASE_URL, 'console_auth_rt', authPassword, 'postgres', 5432);
  assert.ok(services['postgres-topology'].environment.CONSOLE_AUTH_POSTGRES_PASSWORD === authPassword);
  for (const [name, service] of Object.entries(services)) {
    if (name !== 'app') assert.equal(Object.hasOwn(service.environment ?? {}, 'AUTH_DATABASE_URL'), false, 'AUTH URL is API-only');
    if (name !== 'postgres-topology') assert.equal(Object.hasOwn(service.environment ?? {}, 'CONSOLE_AUTH_POSTGRES_PASSWORD'), false, 'raw auth credential is topology-only');
  }
  for (const key of ['AUTH_DATABASE_URL', 'CONSOLE_AUTH_POSTGRES_PASSWORD']) {
    for (const absent of [true, false]) {
      const changed = { ...env }; if (absent) delete changed[key]; else changed[key] = '';
      const rejected = renderCompose(changed);
      assert.notEqual(rejected.status, 0, 'missing or empty required auth input must fail rendering');
      assert.ok(rejected.stderr.includes(key), 'refusal identifies missing configuration key');
      assert.ok(!rejected.stderr.includes(authPassword), 'rendering refusal must not echo auth secret');
    }
  }
});

function executable(run = '') {
  return run.split('\n').filter((line) => !/^\s*#/.test(line)).join('\n');
}
function assertWorkflowAuth(provision, boot, release) {
  const setup = executable(provision.run), serve = executable(boot.run);
  assert.match(setup, /AUTH_PASSWORD="\$\(openssl rand -hex 32\)"/);
  assert.equal((setup.match(/(?:^|[;\n])\s*(?:export\s+)?AUTH_PASSWORD=/g) ?? []).length, 1, 'one auth credential assignment; later reassignment is not reuse');
  assert.match(setup, /CONSOLE_AUTH_POSTGRES_PASSWORD="\$AUTH_PASSWORD"/);
  const mask = setup.indexOf('::add-mask::$AUTH_PASSWORD');
  assert.ok(mask >= 0 && mask < setup.indexOf('CONSOLE_AUTH_POSTGRES_PASSWORD='), 'mask before credential-bearing command');
  if (release) {
    assert.match(setup, /PROBE_AUTH_DATABASE_URL=postgres(?:ql)?:\/\/console_auth_rt:\$\{AUTH_PASSWORD\}@127\.0\.0\.1:5432\/console_release_probe/);
    assert.match(setup, /echo "CONSOLE_AUTH_POSTGRES_PASSWORD=\$\{AUTH_PASSWORD\}"/);
    assert.ok(setup.includes('probe-topology.env'), 'same credential survives topology replay carrier');
    assert.match(serve, /-e AUTH_DATABASE_URL="\$PROBE_AUTH_DATABASE_URL"/);
    assert.doesNotMatch(serve, /(?:^|[;\n])\s*(?:export\s+)?AUTH_PASSWORD=/);
  } else {
    assert.match(setup, /AUTH_DATABASE_URL="postgres(?:ql)?:\/\/console_auth_rt:\$\{AUTH_PASSWORD\}@localhost:5432\/console_ci"/);
    assert.match(serve, /AUTH_DATABASE_URL="\$AUTH_DATABASE_URL"\s*\\/);
  }
}

test('CI actual boot step provisions and delivers the same distinct auth credential (source guard)', () => {
  const step = document('.github/workflows/ci.yml').jobs.backend.steps.find((s) => s.id === 'boot-smoke');
  assert.ok(step?.run); assertWorkflowAuth(step, step, false);
});
test('image release actual provisioning and boot preserve one auth credential (source guard)', () => {
  const steps = document('.github/workflows/image-release.yml').jobs['release-probe'].steps;
  const provision = steps.find((s) => s.name === 'Provision and verify the probe database topology');
  const boot = steps.find((s) => s.name?.startsWith('Boot the release image'));
  assert.ok(provision?.run && boot?.run); assertWorkflowAuth(provision, boot, true);
});

test('workflow guard positive controls reject omitted delivery wrong-role reuse and regenerated auth', () => {
  const provision = { run: 'AUTH_PASSWORD="$(openssl rand -hex 32)"\necho "::add-mask::$AUTH_PASSWORD"\ndocker run -e CONSOLE_AUTH_POSTGRES_PASSWORD="$AUTH_PASSWORD"\necho "PROBE_AUTH_DATABASE_URL=postgres://console_auth_rt:${AUTH_PASSWORD}@127.0.0.1:5432/console_release_probe" >> "$GITHUB_ENV"\necho "CONSOLE_AUTH_POSTGRES_PASSWORD=${AUTH_PASSWORD}" > "$RUNNER_TEMP/probe-topology.env"' };
  const boot = { run: 'docker run -e AUTH_DATABASE_URL="$PROBE_AUTH_DATABASE_URL"' };
  assertWorkflowAuth(provision, boot, true);
  for (const mutate of [
    (p, b) => { b.run = b.run.replace('AUTH_DATABASE_URL=', 'UNRELATED='); },
    (p) => { p.run = p.run.replace('console_auth_rt:${AUTH_PASSWORD}', 'console_rt:${RT_PASSWORD}'); },
    (p) => { p.run += '\nAUTH_PASSWORD="$(openssl rand -hex 32)"'; },
    (p) => { p.run = p.run.replace('docker run', 'AUTH_PASSWORD="$RT_PASSWORD"\ndocker run'); },
    (p, b) => { b.run = 'AUTH_PASSWORD="$RT_PASSWORD"\n' + b.run; },
    (p) => { p.run = p.run.replace('CONSOLE_AUTH_POSTGRES_PASSWORD="$AUTH_PASSWORD"', 'CONSOLE_AUTH_POSTGRES_PASSWORD="$RT_PASSWORD"'); },
    (p) => { p.run = p.run.replace('echo "::add-mask::$AUTH_PASSWORD"', '# omitted mask'); },
  ]) {
    const p = { ...provision }, b = { ...boot }; mutate(p, b);
    assert.throws(() => assertWorkflowAuth(p, b, true), { name: 'AssertionError' });
  }
});
