#!/usr/bin/env node
/**
 * DN-0005 P3: fail-closed presence of nextest serial-group config.
 * Does not run nextest — only asserts the durable process control exists.
 */
import { readFileSync, existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const path = resolve(root, ".config/nextest.toml");

const REQUIRED_FILTERS = [
  "leave_migration_expand_contract",
  "key_revision_migration_upgrade",
  "attendance_console_migration_contract",
  "apalis_adapter",
  "apalis_schema_contract",
  "group_of_one_expand_contract",
];

// Current ordinary callers of the shared Account role initializer.
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

/**
 * The cluster-global override's filter value, or null.
 * Scoped deliberately: comments elsewhere in the file may legitimately quote the
 * broken test(...) form while explaining why it is broken.
 *
 * @param {string} tomlText
 * @returns {string|null}
 */
export function extractOverrideFilter(tomlText) {
  const text = String(tomlText ?? "");
  // TOML permits multi-line literal, single-quoted, and basic strings. Accept
  // all three: a filter is no less wrong for being written on one line.
  const multi = /^[ \t]*filter[ \t]*=[ \t]*'''([\s\S]*?)'''/m.exec(text);
  if (multi) return multi[1];
  const single = /^[ \t]*filter[ \t]*=[ \t]*'([^'\n]*)'/m.exec(text);
  if (single) return single[1];
  const basic = /^[ \t]*filter[ \t]*=[ \t]*"([^"\n]*)"/m.exec(text);
  return basic ? basic[1] : null;
}

export function checkNextestConfig(tomlText) {
  const text = String(tomlText ?? "");
  const failures = [];
  const group = text.split(/^\[test-groups\.cluster-global\][ \t]*(?:#[^\n]*)?$/m)[1]
    ?.split(/^\[[^\n]*\]/m)[0];
  if (group === undefined) {
    failures.push("missing [test-groups.cluster-global]");
  }
  if (!/^[ \t]*max-threads[ \t]*=[ \t]*1[ \t]*(?:#[^\r\n]*)?$/m.test(group ?? "")) {
    failures.push("cluster-global must set max-threads = 1");
  }
  const serialOverride = text
    .split(/^\[\[profile\.default\.overrides\]\][ \t]*(?:#[^\n]*)?$/m)
    .slice(1)
    .map((block) => block.split(/^\[[^\n]*\]/m)[0])
    .find((block) => /^[ \t]*test-group[ \t]*=[ \t]*(?:'cluster-global'|"cluster-global")[ \t]*(?:#[^\r\n]*)?$/m.test(block));
  if (serialOverride === undefined) {
    failures.push("override must assign test-group = cluster-global");
  }
  // The names below are test BINARY names, so they must be matched with
  // `binary(...)`. `test(...)` matches TEST names, and the difference is not
  // cosmetic: measured 2026-08-18 with `cargo nextest show-config test-groups`,
  // the old test-name form put 1 of 4 tests of apalis_adapter in the serial
  // group and 0 of 9 for leave_migration_expand_contract, while the control read
  // green throughout because every name was present as a substring. Assert the
  // form that actually groups, not the spelling.
  const filterBlock = extractOverrideFilter(serialOverride);
  if (filterBlock === null) {
    failures.push("cluster-global override must declare a filter block");
  } else {
    const atoms = filterBlock.split("+").map((atom) => atom.trim());
    // This fixed safety override supports positive exact unions, not predicates.
    if (atoms.some((atom) => !/^(?:binary\([A-Za-z0-9_.-]+\)|binary_id\(=[A-Za-z0-9_.-]+(?:::[A-Za-z0-9_.-]+)?\))$/.test(atom))) {
      failures.push("cluster-global filter must be a finite positive binary union");
    }
    for (const id of ACCOUNT_ROLE_BINARY_IDS) {
      if (!atoms.includes(`binary_id(=${id})`)) {
        failures.push(`filter must group Account fixture by binary_id(=${id})`);
      }
    }
    for (const name of REQUIRED_FILTERS) {
      if (!atoms.includes(`binary(${name})`)) {
        failures.push(`filter must group serial suite by binary(${name})`);
      }
      if (new RegExp(String.raw`test\(/?${name}`).test(filterBlock)) {
        failures.push(
          `filter uses test(${name}); that matches test names, not the binary, `
          + `and silently under-groups it -- use binary(${name})`,
        );
      }
    }
  }
  if (!text.includes("0.9.138")) {
    failures.push("pin cargo-nextest 0.9.138 must be documented in nextest.toml comments");
  }
  return failures;
}

const isMain =
  process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1]);

if (isMain) {
  if (!existsSync(path)) {
    console.error(`missing ${path}`);
    process.exit(1);
  }
  const failures = checkNextestConfig(readFileSync(path, "utf8"));
  if (failures.length) {
    console.error(failures.join("\n"));
    process.exit(1);
  }
  console.log("nextest-config OK (cluster-global serial group + pin 0.9.138 documented)");
}
