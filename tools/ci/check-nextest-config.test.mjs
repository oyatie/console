import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { checkNextestConfig, extractOverrideFilter } from "./check-nextest-config.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");

test("repo nextest.toml satisfies DN-0005 P3 control", () => {
  const text = readFileSync(resolve(root, ".config/nextest.toml"), "utf8");
  assert.deepEqual(checkNextestConfig(text), []);
});

test("missing serial group fails", () => {
  assert.ok(checkNextestConfig("[profile.ci]\n").length > 0);
});

test("missing one filter fails", () => {
  const text = `
[test-groups.cluster-global]
max-threads = 1
[[profile.default.overrides]]
filter = 'test(/leave_migration_expand_contract/)'
test-group = 'cluster-global'
# pin 0.9.138
`;
  const fails = checkNextestConfig(text);
  assert.ok(fails.some((f) => f.includes("key_revision")));
});

test("the pre-fix test() form is rejected, not merely tolerated", () => {
  // This is the exact shape that shipped: every required name present as a
  // substring, so the old gate was green, while cargo-nextest grouped 1 of 4
  // apalis_adapter tests and 0 of 9 leave_migration_expand_contract tests.
  const good = readFileSync(resolve(root, ".config/nextest.toml"), "utf8");
  const broken = good.replace(/binary\((\w+)\)/g, "test(/$1/)");
  const fails = checkNextestConfig(broken);
  assert.ok(fails.length > 0, "the under-grouping form must fail");
  assert.ok(
    fails.some((f) => /use binary\(apalis_adapter\)/.test(f)),
    `expected a binary() remedy, got ${JSON.stringify(fails)}`,
  );
});

test("a serial suite dropped from the filter fails", () => {
  const good = readFileSync(resolve(root, ".config/nextest.toml"), "utf8");
  const dropped = good.replace("binary(apalis_adapter)\n  + ", "");
  assert.ok(
    checkNextestConfig(dropped).some((f) => f.includes("binary(apalis_adapter)")),
  );
});

test("the filter scope excludes prose, so comments may quote the broken form", () => {
  // The committed file explains the defect using a literal test(...) example.
  // Scanning the whole file instead of the filter block would fail on its own
  // documentation.
  const good = readFileSync(resolve(root, ".config/nextest.toml"), "utf8");
  assert.match(good, /test\(\/apalis_adapter\/\)/, "fixture assumes the comment exists");
  assert.deepEqual(checkNextestConfig(good), []);
});

test("extractOverrideFilter returns null when no filter block exists", () => {
  assert.equal(extractOverrideFilter("[profile.ci]\n"), null);
  assert.equal(extractOverrideFilter(null), null);
});

// Gate-only config controls. They do not run nextest, Account fixtures or SQL.
const ACCOUNT_ROLE_BINARY_IDS = [
  "console-app::audit_api",
  "console-app::console_route_telemetry",
  "console-app::evaluation_cycle_api",
  "console-app::health_readiness",
  "console-app::workflow_four_eyes_publish",
  "console-comms-rest::readiness",
  "console-consulting-rest::audit_atomicity",
  "console-financial-rest::purchase_request_list",
  "console-identity-adapter-postgres::deactivate_revokes_credentials",
  "console-identity-adapter-postgres::subject_authz_versions_freshness_rls",
  "console-identity-adapter-postgres::user_lifecycle_noop_replay",
  "console-identity-rest::org_setup",
  "console-inbox-rest::api",
  "console-leave-rest::leave_http_personas",
  "console-messenger-rest::api",
  "console-notices-rest::api",
  "console-notifications-rest::api",
  "console-ontology-rest::object_policy_attach_as_runtime_role",
  "console-ontology-rest::object_type_cas_as_runtime_role",
  "console-ontology-rest::object_type_lifecycle_over_http",
  "console-payroll-rest::api",
  "console-payroll-rest::payslip_draft_api",
  "console-payroll-rest::run_lifecycle_api",
  "console-platform-auth::auth7_legacy_projection",
  "console-platform-auth::refresh_tokens",
  "console-platform-auth::webauthn_ceremony",
  "console-platform-auth::webauthn_ceremony_replay",
  "console-platform-auth::webauthn_registration_deactivated_user",
  "console-platform-db::rls_isolation",
  "console-platform-provisioning::bootstrap_passkey",
  "console-platform-provisioning::bootstrap_passkey_replay",
  "console-platform-provisioning::onboard_mints_group_of_one",
  "console-platform-provisioning::rls_auth_chain_as_runtime_role",
  "console-platform-provisioning::roster_import",
  "console-platform-provisioning::self_enroll_handoff_as_runtime_role",
  "console-platform-rest::onboard_seeds_config_objects",
  "console-platform-rest::ops_dashboard",
  "console-platform-rest::platform_groups",
  "console-platform-rest::remove_tenant",
  "console-platform-rest::view_as",
  "console-production-rest::production_lifecycle_http",
  "console-registry-rest::equipment_admin",
  "console-support-rest::authz",
  "console-support-rest::intake",
  "console-workorder-rest::mobile_device_registration",
  "console-workorder-rest::mobile_evidence",
  "console-workorder-rest::mobile_sync",
];
const ACCOUNT_ROLE_SELECTORS = ACCOUNT_ROLE_BINARY_IDS.map((id) => `binary_id(=${id})`);
const LEGACY_SERIAL_SELECTORS = [
  "leave_migration_expand_contract", "key_revision_migration_upgrade",
  "attendance_console_migration_contract", "apalis_adapter",
  "apalis_schema_contract", "group_of_one_expand_contract",
].map((name) => `binary(${name})`);
const repoConfig = () => readFileSync(resolve(root, ".config/nextest.toml"), "utf8");
const gateFixture = (selectors = [...LEGACY_SERIAL_SELECTORS, ...ACCOUNT_ROLE_SELECTORS]) => `
# GATE FIXTURE ONLY; pin cargo-nextest 0.9.138
[test-groups.cluster-global]
max-threads = 1
[[profile.default.overrides]]
filter = '''
  ${selectors.join("\n  + ")}
'''
test-group = 'cluster-global'
[profile.ci]
test-threads = 8
slow-timeout = { period = "120s", terminate-after = 3 }
`;

test("repo Account initializer binaries have exact selectors in the existing serial override", () => {
  const text = repoConfig();
  assert.deepEqual(checkNextestConfig(text), []);
  const filter = extractOverrideFilter(text);
  assert.notEqual(filter, null);
  for (const selector of ACCOUNT_ROLE_SELECTORS) {
    assert.ok(filter.includes(selector), `repo serial override omits ${selector}`);
  }
});

test("gate rejects Account omissions in the exact pre-fix repo configuration", () => {
  let text = repoConfig();
  // Future repaired config can recreate this same pre-fix omission control;
  // only the reviewed ordinary-caller canonical one-line additions are removed.
  for (const selector of ACCOUNT_ROLE_SELECTORS) {
    text = text.replaceAll(`  + ${selector}\n`, "");
  }
  const failures = checkNextestConfig(text);
  for (const id of ACCOUNT_ROLE_BINARY_IDS) {
    assert.ok(failures.some((failure) => failure.includes(id)), `gate missed omitted ${id}`);
  }
});

test("gate-only complete finite positive selector union is accepted", () => {
  assert.deepEqual(checkNextestConfig(gateFixture()), []);
});

for (const id of ACCOUNT_ROLE_BINARY_IDS) {
  const selector = `binary_id(=${id})`;
  test(`gate-only removing exact serial membership for ${id} is refused`, () => {
    const text = gateFixture([...LEGACY_SERIAL_SELECTORS, ...ACCOUNT_ROLE_SELECTORS.filter((item) => item !== selector)]);
    assert.ok(checkNextestConfig(text).some((failure) => failure.includes(id)));
  });
  for (const [kind, misleading] of [
    ["unanchored prefix", `binary_id(${id})`],
    ["anchored different suffix", `binary_id(=${id}_shadow)`],
    ["quoted matcher", `binary_id(="${id}")`],
  ]) {
    test(`gate-only ${kind} cannot supply exact membership for ${id}`, () => {
      assert.ok(checkNextestConfig(gateFixture().replace(selector, misleading)).length > 0);
    });
  }
}

test("gate-only comment mentions do not supply serial Account membership", () => {
  const text = gateFixture(LEGACY_SERIAL_SELECTORS)
    + ACCOUNT_ROLE_SELECTORS.map((selector) => `# ${selector}`).join("\n") + "\n";
  assert.ok(checkNextestConfig(text).length > 0);
});

test("gate-only Account selectors in another group cannot borrow an unrelated serial override", () => {
  const text = gateFixture().replace("test-group = 'cluster-global'", "test-group = 'ordinary'") + `
[test-groups.ordinary]
max-threads = 8
[[profile.default.overrides]]
filter = 'test(unrelated_control)'
test-group = 'cluster-global'
`;
  assert.ok(checkNextestConfig(text).length > 0);
});

test("gate-only negated membership cannot serialize the required Account binary", () => {
  const selector = ACCOUNT_ROLE_SELECTORS[0];
  assert.ok(checkNextestConfig(gateFixture().replace(selector, `!${selector}`)).length > 0);
});

test("gate-only conditional membership cannot serialize only some Account tests", () => {
  const selector = ACCOUNT_ROLE_SELECTORS[0];
  assert.ok(checkNextestConfig(gateFixture().replace(selector, `${selector} & test(one_test)`)).length > 0);
});

test("gate-only legacy hazards remain required alongside the Account callers", () => {
  for (const selector of LEGACY_SERIAL_SELECTORS) {
    const text = gateFixture([...LEGACY_SERIAL_SELECTORS.filter((item) => item !== selector), ...ACCOUNT_ROLE_SELECTORS]);
    assert.ok(checkNextestConfig(text).some((failure) => failure.includes(selector)));
  }
});

test("gate-only unrelated parallel override and larger thread allocation remain available", () => {
  const text = gateFixture() + `
[test-groups.ordinary]
max-threads = 8
[[profile.default.overrides]]
filter = 'test(unrelated_control)'
test-group = 'ordinary'
threads-required = 2
`;
  assert.deepEqual(checkNextestConfig(text), []);
  assert.match(text, /test-threads = 8/);
  assert.match(text, /threads-required = 2/);
  assert.doesNotMatch(text, /test-threads\s*=\s*1(?:\s|$)/);
});


test("gate-only already-isolated auth_rest is not required in the ordinary serial union", () => {
  assert.equal(ACCOUNT_ROLE_BINARY_IDS.length, 47);
  assert.equal(ACCOUNT_ROLE_BINARY_IDS.includes("console-app::auth_rest"), false);
  const text = gateFixture();
  assert.doesNotMatch(text, /binary_id\(=console-app::auth_rest\)/);
  assert.deepEqual(checkNextestConfig(text), []);
});

// Real-repository omission controls for the three reviewed Root+Auth7 callers.
// A synthetic complete gateFixture must not hide their absence from the map.
for (const id of [
  "console-app::workflow_four_eyes_publish",
  "console-identity-adapter-postgres::subject_authz_versions_freshness_rls",
  "console-identity-adapter-postgres::user_lifecycle_noop_replay",
]) {
  test(`repo gate refuses missing serialization for new Account initializer ${id}`, () => {
    const text = repoConfig().replaceAll(`  + binary_id(=${id})\n`, "");
    assert.ok(
      checkNextestConfig(text).includes(`filter must group Account fixture by binary_id(=${id})`),
      `owner gate omitted the real initializer caller ${id}`,
    );
  });
}
