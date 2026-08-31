// Semantic-contract gate over the six PRODUCT objects (links + action contracts).
//
// The hole this closes: #994 published Head schemas under Company / OrgUnit /
// JobPosition / Person / Employment / PayRun, but those schemas are still
// property bags. PRODUCT's admitted direction is a typed semantic manifest —
// link types, action input/result schemas, required permissions, declared
// edits, and concurrency — that generates OpenAPI. Generic
// CreateObjectTypeDraft children and OntologyActionRequest.params erase types.
//
// Chesterton: declare the runtime ports, not a Palantir/OSDK clone.
//   * links are Head FKs / parent_id, not ont_links fiction and not RLS org_id
//   * actions are the thirteen DispatchTarget strings, not invent payroll.calculate
//   * permissions are Feature::RoleManage (`role_manage`) — the ontology REST gate
//   * four_eyes matches requires_natural_person_four_eyes (company.*/hr.*/payroll.*)
//   * Person stays a closed four-field projection
//   * PayRun.payable stays const false
//   * links/actions are schema-level keywords, never instance properties
//
// Totality: js-yaml load + own-property reads of components.schemas. A walker
// that visits nothing reports nothing, so OBJECT_FLOOR / ACTION_FLOOR / LINK_FLOOR
// are the examined-zero locks.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const OBJECT_FLOOR = 6;
export const ACTION_FLOOR = 13;
export const LINK_FLOOR = 5;

export const RESULT_REF = "#/components/schemas/OntologyActionExecuteOutcome";
export const PERMISSION_ROLE_MANAGE = "role_manage";
export const CONCURRENCY_COMMAND_ID = "tenant_global_idempotency";
export const CONCURRENCY_EXPECTED_REVISION = "optional_cas";

export const PERSON_FORBIDDEN_FIELDS = Object.freeze([
  "phone",
  "salary",
  "bank_account",
  "rrn",
  "base_pay",
]);

/** Runtime Head FKs. Company / Person / PayRun have no outgoing Head link. */
export const CANONICAL_LINKS = Object.freeze([
  Object.freeze({
    key: "org_unit_parent",
    from: "OrgUnit",
    to: "OrgUnit",
    field: "parent_id",
    cardinality: "many-to-one",
    option: true,
  }),
  Object.freeze({
    key: "job_position_org_unit",
    from: "JobPosition",
    to: "OrgUnit",
    field: "org_unit_id",
    cardinality: "many-to-one",
    option: false,
  }),
  Object.freeze({
    key: "employment_person",
    from: "Employment",
    to: "Person",
    field: "person_id",
    cardinality: "many-to-one",
    option: true,
  }),
  Object.freeze({
    key: "employment_org_unit",
    from: "Employment",
    to: "OrgUnit",
    field: "org_unit_id",
    cardinality: "many-to-one",
    option: true,
  }),
  Object.freeze({
    key: "employment_job_position",
    from: "Employment",
    to: "JobPosition",
    field: "job_position_id",
    cardinality: "many-to-one",
    option: true,
  }),
]);

/**
 * Thirteen DispatchTarget strings from canonical-domain, plus the owning
 * object, four-eyes bar, and owned-table edits the port actually writes.
 */
export const CANONICAL_ACTIONS = Object.freeze([
  Object.freeze({
    action_key: "company.revise",
    object: "Company",
    object_key: "company",
    input: "CompanyReviseInput",
    four_eyes: "natural_person",
    edits: Object.freeze(["company_revisions"]),
  }),
  Object.freeze({
    action_key: "organization.create_org_unit",
    object: "OrgUnit",
    object_key: "org_unit",
    input: "OrganizationCreateOrgUnitInput",
    four_eyes: "account",
    edits: Object.freeze(["org_units", "org_unit_revisions", "org_unit_source_bindings"]),
  }),
  Object.freeze({
    action_key: "organization.revise_org_unit",
    object: "OrgUnit",
    object_key: "org_unit",
    input: "OrganizationReviseOrgUnitInput",
    four_eyes: "account",
    edits: Object.freeze(["org_unit_revisions", "org_unit_source_bindings"]),
  }),
  Object.freeze({
    action_key: "organization.create_job_position",
    object: "JobPosition",
    object_key: "job_position",
    input: "OrganizationCreateJobPositionInput",
    four_eyes: "account",
    edits: Object.freeze(["job_positions", "job_position_revisions"]),
  }),
  Object.freeze({
    action_key: "organization.revise_job_position",
    object: "JobPosition",
    object_key: "job_position",
    input: "OrganizationReviseJobPositionInput",
    four_eyes: "account",
    edits: Object.freeze(["job_positions", "job_position_revisions"]),
  }),
  Object.freeze({
    action_key: "people.create_person",
    object: "Person",
    object_key: "person",
    input: "PeopleCreatePersonInput",
    four_eyes: "account",
    edits: Object.freeze(["persons", "person_revisions", "employee_person_bindings"]),
  }),
  Object.freeze({
    action_key: "people.revise_person",
    object: "Person",
    object_key: "person",
    input: "PeopleRevisePersonInput",
    four_eyes: "account",
    edits: Object.freeze(["person_revisions", "employee_person_bindings"]),
  }),
  Object.freeze({
    action_key: "hr.appoint",
    object: "Employment",
    object_key: "employment",
    input: "HrAppointInput",
    four_eyes: "natural_person",
    edits: Object.freeze(["employment_heads", "employment_revisions", "employment_source_bindings"]),
  }),
  Object.freeze({
    action_key: "hr.promote",
    object: "Employment",
    object_key: "employment",
    input: "HrPromoteInput",
    four_eyes: "natural_person",
    edits: Object.freeze(["employment_revisions", "employees"]),
  }),
  Object.freeze({
    action_key: "hr.transfer",
    object: "Employment",
    object_key: "employment",
    input: "HrTransferInput",
    four_eyes: "natural_person",
    edits: Object.freeze(["employment_revisions", "employees"]),
  }),
  Object.freeze({
    action_key: "payroll.create_run",
    object: "PayRun",
    object_key: "pay_run",
    input: "PayrollCreateRunInput",
    four_eyes: "natural_person",
    edits: Object.freeze(["payroll_draft_runs"]),
  }),
  Object.freeze({
    action_key: "payroll.submit_run",
    object: "PayRun",
    object_key: "pay_run",
    input: "PayrollSubmitRunInput",
    four_eyes: "natural_person",
    edits: Object.freeze(["payroll_draft_runs"]),
  }),
  Object.freeze({
    action_key: "payroll.decide_run",
    object: "PayRun",
    object_key: "pay_run",
    input: "PayrollDecideRunInput",
    four_eyes: "natural_person",
    edits: Object.freeze(["payroll_draft_runs"]),
  }),
]);

export const CANONICAL_OBJECTS = Object.freeze([
  Object.freeze({ name: "Company", object_key: "company" }),
  Object.freeze({ name: "OrgUnit", object_key: "org_unit" }),
  Object.freeze({ name: "JobPosition", object_key: "job_position" }),
  Object.freeze({ name: "Person", object_key: "person" }),
  Object.freeze({ name: "Employment", object_key: "employment" }),
  Object.freeze({ name: "PayRun", object_key: "pay_run" }),
]);

function schemaRefName(schema) {
  if (!isPlainObject(schema)) return null;
  const ref = own(schema, "$ref");
  if (typeof ref !== "string") return null;
  const prefix = "#/components/schemas/";
  if (!ref.startsWith(prefix)) return null;
  return ref.slice(prefix.length);
}

function propertyMap(schema) {
  const properties = own(schema, "properties");
  return isPlainObject(properties) ? properties : null;
}

function isFreeFormObject(schema) {
  if (!isPlainObject(schema)) return false;
  if (own(schema, "additionalProperties") !== true) return false;
  const properties = propertyMap(schema);
  return !properties || Object.keys(properties).length === 0;
}

function inputIsTyped(inputSchema) {
  if (!isPlainObject(inputSchema)) return false;
  if (own(inputSchema, "type") !== "object") return false;
  if (isFreeFormObject(inputSchema)) return false;
  const properties = propertyMap(inputSchema);
  return Boolean(properties && Object.keys(properties).length > 0);
}

function findAction(list, actionKey) {
  if (!Array.isArray(list)) return null;
  return list.find((item) => isPlainObject(item) && own(item, "action_key") === actionKey) ?? null;
}

function findLink(list, key) {
  if (!Array.isArray(list)) return null;
  return list.find((item) => isPlainObject(item) && own(item, "key") === key) ?? null;
}

function push(findings, location, message) {
  findings.push({ location, message });
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   objects: number,
 *   actions: number,
 *   links: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateSemanticContract({ repoRoot }) {
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const findings = [];
  const components = own(document, "components");
  const schemas = own(components, "schemas");
  if (!isPlainObject(schemas)) {
    push(findings, "#/components/schemas", "published document has no components.schemas mapping");
    return { objects: 0, actions: 0, links: 0, findings };
  }

  let objects = 0;
  let actions = 0;
  let links = 0;
  const seenActionKeys = new Set();

  for (const spec of CANONICAL_OBJECTS) {
    const location = `#/components/schemas/${spec.name}`;
    if (!hasOwnKey(schemas, spec.name)) {
      push(
        findings,
        location,
        "canonical object schema is absent; CreateObjectTypeDraft / OntologyActionRequest.params are not this contract",
      );
      continue;
    }
    objects += 1;
    const schema = own(schemas, spec.name);
    if (!isPlainObject(schema) || own(schema, "type") !== "object") {
      push(findings, location, "canonical object schema must be type: object");
      continue;
    }

    const properties = propertyMap(schema) ?? {};
    if (hasOwnKey(properties, "links") || hasOwnKey(properties, "actions")) {
      push(
        findings,
        `${location}/properties`,
        "links/actions must be schema-level ontology keywords, not instance properties (Head GET fiction)",
      );
    }

    if (spec.name === "Person") {
      for (const field of PERSON_FORBIDDEN_FIELDS) {
        if (hasOwnKey(properties, field)) {
          push(
            findings,
            `${location}/properties/${field}`,
            "Person Head is a closed four-field projection; phone/salary/bank_account/rrn/base_pay stay off the wire",
          );
        }
      }
    }

    if (spec.name === "PayRun") {
      const payable = own(properties, "payable");
      if (own(payable, "const") !== false) {
        push(
          findings,
          `${location}/properties/payable`,
          "PayRun.payable must be const: false — honest scaffold, payment HOLD",
        );
      }
    }

    if (!hasOwnKey(schema, "links")) {
      push(
        findings,
        `${location}/links`,
        "schema-level links keyword is absent; Head FKs are not an ontology",
      );
    } else {
      const declared = own(schema, "links");
      if (!Array.isArray(declared)) {
        push(findings, `${location}/links`, "links must be an array of link-type declarations");
      }
    }

    if (!hasOwnKey(schema, "actions")) {
      push(
        findings,
        `${location}/actions`,
        "schema-level actions keyword is absent; DispatchTarget contracts are not published",
      );
    } else {
      const declared = own(schema, "actions");
      if (!Array.isArray(declared)) {
        push(findings, `${location}/actions`, "actions must be an array of action contracts");
      }
    }

    const declaredLinks = own(schema, "links");
    const declaredActions = own(schema, "actions");
    const expectedLinks = CANONICAL_LINKS.filter((link) => link.from === spec.name);
    const expectedActions = CANONICAL_ACTIONS.filter((action) => action.object === spec.name);

    if (Array.isArray(declaredLinks)) {
      for (const link of expectedLinks) {
        const found = findLink(declaredLinks, link.key);
        const linkLoc = `${location}/links/${link.key}`;
        if (!found) {
          push(findings, linkLoc, `runtime Head FK ${link.field} is not declared as a link type`);
          continue;
        }
        links += 1;
        for (const [field, value] of [
          ["from", link.from],
          ["to", link.to],
          ["field", link.field],
          ["cardinality", link.cardinality],
        ]) {
          if (own(found, field) !== value) {
            push(findings, `${linkLoc}/${field}`, `must be ${value} (runtime Head), got ${JSON.stringify(own(found, field))}`);
          }
        }
        if (own(found, "option") !== link.option) {
          push(
            findings,
            `${linkLoc}/option`,
            `must be ${link.option} to match Option on the Head (not OAS 3.0 nullable)`,
          );
        }
      }
    }

    if (Array.isArray(declaredActions)) {
      for (const action of expectedActions) {
        const found = findAction(declaredActions, action.action_key);
        const actionLoc = `${location}/actions/${action.action_key}`;
        if (!found) {
          push(
            findings,
            actionLoc,
            `dispatch target ${action.action_key} is not declared; OntologyActionRequest.params is not this contract`,
          );
          continue;
        }
        actions += 1;
        seenActionKeys.add(action.action_key);

        if (own(found, "object_key") !== action.object_key) {
          push(
            findings,
            `${actionLoc}/object_key`,
            `must be ${action.object_key} (ObjectKey::as_str)`,
          );
        }
        if (own(found, "four_eyes") !== action.four_eyes) {
          push(
            findings,
            `${actionLoc}/four_eyes`,
            `must be ${action.four_eyes} (requires_natural_person_four_eyes)`,
          );
        }

        const permissions = own(found, "permissions");
        if (!Array.isArray(permissions) || !permissions.includes(PERMISSION_ROLE_MANAGE)) {
          push(
            findings,
            `${actionLoc}/permissions`,
            "must include role_manage (ontology REST Feature::RoleManage)",
          );
        }

        const inputName = schemaRefName(own(found, "input"));
        if (inputName !== action.input) {
          push(
            findings,
            `${actionLoc}/input`,
            `must $ref #/components/schemas/${action.input}, not OntologyActionRequest.params`,
          );
        } else {
          const inputSchema = own(schemas, inputName);
          const inputLoc = `#/components/schemas/${inputName}`;
          if (!hasOwnKey(schemas, inputName)) {
            push(findings, inputLoc, "typed action input schema is absent");
          } else if (!inputIsTyped(inputSchema)) {
            push(
              findings,
              inputLoc,
              "action input must be a typed object (not additionalProperties: true with no properties)",
            );
          } else if (inputName === "OntologyActionRequest" || hasOwnKey(propertyMap(inputSchema) ?? {}, "params")) {
            const params = own(propertyMap(inputSchema), "params");
            if (isFreeFormObject(params)) {
              push(findings, `${inputLoc}/properties/params`, "free-form params erase types");
            }
          }
        }

        const resultName = schemaRefName(own(found, "result"));
        if (own(found, "result") && schemaRefName(own(found, "result")) !== "OntologyActionExecuteOutcome") {
          push(
            findings,
            `${actionLoc}/result`,
            `must $ref ${RESULT_REF} (runtime execute outcome, not a Palantir ActionResponse)`,
          );
        } else if (resultName !== "OntologyActionExecuteOutcome") {
          push(findings, `${actionLoc}/result`, `must $ref ${RESULT_REF}`);
        }

        const edits = own(found, "edits");
        if (!Array.isArray(edits)) {
          push(findings, `${actionLoc}/edits`, "must declare the tables this port writes");
        } else {
          for (const table of action.edits) {
            if (!edits.includes(table)) {
              push(findings, `${actionLoc}/edits`, `missing owned-table ${table}`);
            }
          }
        }

        const concurrency = own(found, "concurrency");
        if (!isPlainObject(concurrency)) {
          push(
            findings,
            `${actionLoc}/concurrency`,
            "must declare command_id idempotency and expected_revision CAS",
          );
        } else {
          if (own(concurrency, "command_id") !== CONCURRENCY_COMMAND_ID) {
            push(
              findings,
              `${actionLoc}/concurrency/command_id`,
              `must be ${CONCURRENCY_COMMAND_ID}`,
            );
          }
          if (own(concurrency, "expected_revision") !== CONCURRENCY_EXPECTED_REVISION) {
            push(
              findings,
              `${actionLoc}/concurrency/expected_revision`,
              `must be ${CONCURRENCY_EXPECTED_REVISION}`,
            );
          }
        }
      }
    }
  }

  if (seenActionKeys.size > 0 && seenActionKeys.size !== ACTION_FLOOR) {
    const missing = CANONICAL_ACTIONS.map((action) => action.action_key).filter(
      (key) => !seenActionKeys.has(key),
    );
    if (missing.length > 0) {
      push(
        findings,
        "#/components/schemas",
        `dispatch roster is ${seenActionKeys.size}/${ACTION_FLOOR}; missing ${missing.join(", ")}`,
      );
    }
  }

  return { objects, actions, links, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateSemanticContract({ repoRoot });
  } catch (error) {
    console.error(`backend/openapi/openapi.yaml cannot be parsed: ${error.message}`);
    process.exit(1);
  }
  const { objects, actions, links, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowFloor =
    objects < OBJECT_FLOOR || actions < ACTION_FLOOR || links < LINK_FLOOR;
  if (belowFloor) {
    console.error(
      `saw ${objects}/${OBJECT_FLOOR} objects, ${actions}/${ACTION_FLOOR} actions, `
        + `${links}/${LINK_FLOOR} links — below the floor; Heads are not an ontology`,
    );
  }
  if (findings.length > 0 || belowFloor) {
    console.error(
      `openapi semantic-contract gate FAILED: ${findings.length} finding(s), `
        + `${objects}/${OBJECT_FLOOR} objects, ${actions}/${ACTION_FLOOR} actions, `
        + `${links}/${LINK_FLOOR} links`,
    );
    process.exit(1);
  }
  console.log(
    `openapi semantic-contract gate passed `
      + `(${objects}/${OBJECT_FLOOR} objects, ${actions}/${ACTION_FLOOR} actions, `
      + `${links}/${LINK_FLOOR} links, 0 findings)`,
  );
}
