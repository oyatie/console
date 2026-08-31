/**
 * Generated from backend/openapi/openapi.yaml by scripts/generate-openapi-ts-sdk.mjs.
 * Do not edit.
 */

/**
 * Typed params for `company.revise` (`CompanyQuery`). The tenant is `org_id` on the command envelope, not this bag. Preflight requires non-empty `legal_name`.
 */
export type CompanyReviseInput = {
  attributes: {
    legal_name: string;
    reg_no?: string | null;
    [key: string]: unknown;
  };
};

/**
 * Typed params for `organization.create_org_unit` (`OrgUnitQuery::Create`). Write preflight requires non-empty `name` and `kind` ∈ {site, department, team}.
 */
export type OrganizationCreateOrgUnitInput = {
  source?: OrgUnitSourceBinding | null;
  attributes: {
    name: string;
    kind: "site" | "department" | "team";
    parent_id?: Uuid | null;
    [key: string]: unknown;
  };
};

/**
 * Typed params for `organization.revise_org_unit` (`OrgUnitQuery::Revise`).
 */
export type OrganizationReviseOrgUnitInput = {
  org_unit_id: Uuid;
  source?: OrgUnitSourceBinding | null;
  attributes: {
    name: string;
    kind: "site" | "department" | "team";
    parent_id?: Uuid | null;
    [key: string]: unknown;
  };
};

/**
 * Typed params for `organization.create_job_position` (`JobPositionQuery::Create`). The named OrgUnit must already exist. Write preflight requires non-empty `title`.
 */
export type OrganizationCreateJobPositionInput = {
  org_unit_id: Uuid;
  attributes: {
    title: string;
    [key: string]: unknown;
  };
};

/**
 * Typed params for `organization.revise_job_position` (`JobPositionQuery::Revise`). Null `org_unit_id` leaves the position where it is; a UUID moves it.
 */
export type OrganizationReviseJobPositionInput = {
  job_position_id: Uuid;
  org_unit_id?: Uuid | null;
  attributes: {
    title: string;
    [key: string]: unknown;
  };
};

/**
 * Typed params for `people.create_person` (`PersonQuery::Create`). When `employee_id` is set (trusted uniquely resolved), the new person's id IS that employee id. Head projection stays four fields; this bag is not a license to publish phone/salary/bank_account/rrn on GET.
 */
export type PeopleCreatePersonInput = {
  employee_id?: Uuid | null;
  attributes: {
    legal_name?: string | null;
    display_name?: string | null;
    [key: string]: unknown;
  };
};

/**
 * Typed params for `people.revise_person` (`PersonQuery::Revise`). Optional `employee_id` binds a further employee record to the same natural person.
 */
export type PeopleRevisePersonInput = {
  person_id: Uuid;
  employee_id?: Uuid | null;
  attributes: {
    legal_name?: string | null;
    display_name?: string | null;
    [key: string]: unknown;
  };
};

/**
 * Typed params for `hr.appoint` (`EmploymentQuery::Appoint`). Opens a head at `valid_from`. Four-eyes is natural-person. The legacy employees row is not rewritten by this command.
 */
export type HrAppointInput = {
  employee_id: Uuid;
  valid_from: Timestamp;
  attributes: EmploymentAttributesInput;
};

/**
 * Typed params for `hr.promote` (`EmploymentQuery::Promote`). Appends a revision and carries the new state onto the legacy head. Four-eyes is natural-person.
 */
export type HrPromoteInput = {
  employment_id: Uuid;
  valid_from: Timestamp;
  attributes: EmploymentAttributesInput;
};

/**
 * Typed params for `hr.transfer` (`EmploymentQuery::Transfer`). Same shape as promote; the receipt records a different dispatch target. Four-eyes is natural-person.
 */
export type HrTransferInput = {
  employment_id: Uuid;
  valid_from: Timestamp;
  attributes: EmploymentAttributesInput;
};

/**
 * Typed params for `payroll.create_run` (`PayRunQuery::CreateRun`). Stages a draft under a caller-chosen source label. `payable` is not an input; stored calculations stay false. Payment execution remains HOLD.
 */
export type PayrollCreateRunInput = {
  run_id: Uuid;
  period_start: string;
  period_end: string;
  connector?: string | null;
  job?: string | null;
};

/**
 * Typed params for `payroll.submit_run` (`PayRunQuery::SubmitRun`). `CALCULATED → SUBMITTED`, refused while any exception is open. `payable` stays false.
 */
export type PayrollSubmitRunInput = {
  run_id: Uuid;
};

/**
 * Typed params for `payroll.decide_run` (`PayRunQuery::DecideRun`). `SUBMITTED → APPROVED|REJECTED`, refused when the decider is the submitter. Distinct from path-bound `DecidePayrollRunRequest` (run_id is in this bag). `payable` stays false. Payment execution remains HOLD.
 */
export type PayrollDecideRunInput = {
  run_id: Uuid;
  decision: "APPROVE" | "REJECT";
  reason?: string | null;
};

/**
 * Write bag for `hr.appoint` / `hr.promote` / `hr.transfer` (`EmploymentAttributes`). Not the Employment Head. `company` is a non-blank string; `employment_status` is the 0066 CHECK set. UUID refs must not be nil.
 */
export type EmploymentAttributesInput = {
  company: string;
  org_unit_id?: Uuid | null;
  job_position_id?: Uuid | null;
  employment_status: "ACTIVE" | "EXITED" | "UNKNOWN";
};

/**
 * Optional legacy source binding on OrgUnit create/revise. `kind` and `id` are TEXT; the closed set of kinds is not yet enumerable and legacy identifiers are not all UUIDs.
 */
export type OrgUnitSourceBinding = {
  kind: string;
  id: string;
};

/**
 * Canonical Company head for one tenant. The tenant IS the company (`org_id`). GET /api/v1/companies/{id} returns this current head (path `id` is `org_id`). GET /api/v1/companies returns zero or one current head for the armed tenant (the same row `list` already returns). as_of / from / to are omitted: `company_revisions` has no valid-time columns, and using `created_at` as `valid_from` would be a second time model. `legal_name` and `reg_no` are read from the latest `company_revisions.attributes`; they are never copied from provisioning-owned `organizations.name`. Write preflight requires a non-empty string `legal_name`; an omitted key is JSON null on the Head. `founded_on` and `registration_number` are company_conformance fixture keys, not this port. `org_id` is the RLS cell, not a Palantir link. Temporal slices other than revision `version` remain HOLD.
 */
export type Company = {
  org_id: Uuid;
  legal_name: string | null;
  reg_no: string | null;
  version: number;
};

export const CompanyDefinition = {
  name: "Company",
  links: [],
  actions: [
    {
      action_key: "company.revise",
      object_key: "company",
      input: {
        $ref: "#/components/schemas/CompanyReviseInput",
      },
      result: {
        $ref: "#/components/schemas/OntologyActionExecuteOutcome",
      },
      permissions: [
        "role_manage",
      ],
      four_eyes: "natural_person",
      edits: [
        "company_revisions",
      ],
      concurrency: {
        command_id: "tenant_global_idempotency",
        expected_revision: "optional_cas",
      },
    },
  ],
  concurrency: {
    get_token: "version",
    write_field: "expected_revision",
    write_in: "body",
  },
} as const;

/**
 * Canonical OrgUnit head. GET /api/v1/org-units/{id} returns this current head. GET /api/v1/org-units returns current heads (the same rows `list` already returns). as_of / from / to are omitted: `org_units` / `org_unit_revisions` have no valid-time columns, and using `created_at` as `valid_from` would be a second time model. `name` and `parent_id` are parsed from the latest `org_unit_revisions.attributes`; neither is a column on `org_units`. Sites, regions, and branches stay the operational auth spine and are not OrgUnits. OrgUnit kinds Site/Department/Team are write-preflight on revision attributes (`kind` ∈ {site, department, team}) and are not a Head column. Temporal slices other than revision `version` remain HOLD.
 */
export type OrgUnit = {
  id: Uuid;
  name: string | null;
  parent_id: Uuid | null;
  version: number;
};

export const OrgUnitDefinition = {
  name: "OrgUnit",
  links: [
    {
      key: "org_unit_parent",
      from: "OrgUnit",
      to: "OrgUnit",
      field: "parent_id",
      cardinality: "many-to-one",
      option: true,
      operationId: "getOrgUnit",
    },
  ],
  actions: [
    {
      action_key: "organization.create_org_unit",
      object_key: "org_unit",
      input: {
        $ref: "#/components/schemas/OrganizationCreateOrgUnitInput",
      },
      result: {
        $ref: "#/components/schemas/OntologyActionExecuteOutcome",
      },
      permissions: [
        "role_manage",
      ],
      four_eyes: "account",
      edits: [
        "org_units",
        "org_unit_revisions",
        "org_unit_source_bindings",
      ],
      concurrency: {
        command_id: "tenant_global_idempotency",
        expected_revision: "optional_cas",
      },
    },
    {
      action_key: "organization.revise_org_unit",
      object_key: "org_unit",
      input: {
        $ref: "#/components/schemas/OrganizationReviseOrgUnitInput",
      },
      result: {
        $ref: "#/components/schemas/OntologyActionExecuteOutcome",
      },
      permissions: [
        "role_manage",
      ],
      four_eyes: "account",
      edits: [
        "org_unit_revisions",
        "org_unit_source_bindings",
      ],
      concurrency: {
        command_id: "tenant_global_idempotency",
        expected_revision: "optional_cas",
      },
    },
  ],
  concurrency: {
    get_token: "version",
    write_field: "expected_revision",
    write_in: "body",
  },
} as const;

/**
 * Canonical JobPosition head. Distinct from recruiting postings and from `employees.position` TEXT. `org_unit_id` is a foreign key to an existing OrgUnit. `attributes` is the latest revision bag; write preflight requires a non-empty string `title` (catalog JOB_POSITION_TITLE). Other attribute keys are not a published closed set. Temporal slices other than revision `version` remain HOLD.
 */
export type JobPosition = {
  job_position_id: Uuid;
  org_unit_id: Uuid;
  version: number;
  attributes: {
    [key: string]: unknown;
  };
};

export const JobPositionDefinition = {
  name: "JobPosition",
  links: [
    {
      key: "job_position_org_unit",
      from: "JobPosition",
      to: "OrgUnit",
      field: "org_unit_id",
      cardinality: "many-to-one",
      option: false,
      operationId: "getOrgUnit",
    },
  ],
  actions: [
    {
      action_key: "organization.create_job_position",
      object_key: "job_position",
      input: {
        $ref: "#/components/schemas/OrganizationCreateJobPositionInput",
      },
      result: {
        $ref: "#/components/schemas/OntologyActionExecuteOutcome",
      },
      permissions: [
        "role_manage",
      ],
      four_eyes: "account",
      edits: [
        "job_positions",
        "job_position_revisions",
      ],
      concurrency: {
        command_id: "tenant_global_idempotency",
        expected_revision: "optional_cas",
      },
    },
    {
      action_key: "organization.revise_job_position",
      object_key: "job_position",
      input: {
        $ref: "#/components/schemas/OrganizationReviseJobPositionInput",
      },
      result: {
        $ref: "#/components/schemas/OntologyActionExecuteOutcome",
      },
      permissions: [
        "role_manage",
      ],
      four_eyes: "account",
      edits: [
        "job_positions",
        "job_position_revisions",
      ],
      concurrency: {
        command_id: "tenant_global_idempotency",
        expected_revision: "optional_cas",
      },
    },
  ],
  concurrency: {
    get_token: "version",
    write_field: "expected_revision",
    write_in: "body",
  },
} as const;

/**
 * Canonical Person head. Closed four-field projection of the latest `person_revisions` row. GET /api/v1/persons/{id} returns this current head. GET /api/v1/persons returns current heads (the same closed four-field rows `list` already returns). as_of / from / to are omitted: `persons` / `person_revisions` have no valid-time columns, and using `created_at` as `valid_from` would be a second time model. `get`/`list` never copy the attributes object onto the wire, so stored `phone` / `salary` / `bank_account` / `rrn` cannot appear. Trusted directory create mints `person_id = employee_id`. No outgoing Head FK; Employment.person_id is the reverse. Temporal slices other than revision `version` remain HOLD.
 */
export type Person = {
  id: Uuid;
  display_name: string | null;
  legal_name: string | null;
  version: number;
};

export const PersonDefinition = {
  name: "Person",
  links: [],
  actions: [
    {
      action_key: "people.create_person",
      object_key: "person",
      input: {
        $ref: "#/components/schemas/PeopleCreatePersonInput",
      },
      result: {
        $ref: "#/components/schemas/OntologyActionExecuteOutcome",
      },
      permissions: [
        "role_manage",
      ],
      four_eyes: "account",
      edits: [
        "persons",
        "person_revisions",
        "employee_person_bindings",
      ],
      concurrency: {
        command_id: "tenant_global_idempotency",
        expected_revision: "optional_cas",
      },
    },
    {
      action_key: "people.revise_person",
      object_key: "person",
      input: {
        $ref: "#/components/schemas/PeopleRevisePersonInput",
      },
      result: {
        $ref: "#/components/schemas/OntologyActionExecuteOutcome",
      },
      permissions: [
        "role_manage",
      ],
      four_eyes: "account",
      edits: [
        "person_revisions",
        "employee_person_bindings",
      ],
      concurrency: {
        command_id: "tenant_global_idempotency",
        expected_revision: "optional_cas",
      },
    },
  ],
  concurrency: {
    get_token: "version",
    write_field: "expected_revision",
    write_in: "body",
  },
} as const;

/**
 * Canonical Employment head. Current get/list still return the *open* head (`employment_heads.valid_to IS NULL`). GET /api/v1/employments/{id}?as_of= reconstructs the slice effective at an RFC3339 instant using the same half-open predicate as ontology instance GET (`valid_from <= as_of < coalesce(valid_to, ∞)`); absent as_of = current open head. GET /api/v1/employments?from=&to= lists heads whose `[valid_from, valid_to)` overlaps `[from, to)` with that same algebra (absent both = current open heads). Closed (EXITED) windows are omitted by current get/list and by as_of at or after valid_to. `org_unit_id` / `job_position_id` are canonical UUIDs from the effective revision, not free-text team or title labels. `person_id` is the unique source-binding → person-binding path; ambiguous or unbound identities are null, never invented from `employee_id`. `appointed_on` is the head's opening `valid_from`. `version` is `employment_revisions.version` on that already-joined effective row (MAX(valid_from), not MAX(version)). Write attributes also carry `company` and `employment_status` in {ACTIVE, EXITED, UNKNOWN}; those write fields are not this Head. The Head never includes phone, salary, bank_account, rrn, or base_pay. Corrections vs new slice and delta transmission remain HOLD. EmployeeDetail as_of remains HOLD.
 */
export type Employment = {
  id: Uuid;
  person_id: Uuid | null;
  org_unit_id: Uuid | null;
  job_position_id: Uuid | null;
  appointed_on: Timestamp;
  version: number;
};

export const EmploymentDefinition = {
  name: "Employment",
  links: [
    {
      key: "employment_person",
      from: "Employment",
      to: "Person",
      field: "person_id",
      cardinality: "many-to-one",
      option: true,
      operationId: "getPerson",
    },
    {
      key: "employment_org_unit",
      from: "Employment",
      to: "OrgUnit",
      field: "org_unit_id",
      cardinality: "many-to-one",
      option: true,
      operationId: "getOrgUnit",
    },
    {
      key: "employment_job_position",
      from: "Employment",
      to: "JobPosition",
      field: "job_position_id",
      cardinality: "many-to-one",
      option: true,
    },
  ],
  actions: [
    {
      action_key: "hr.appoint",
      object_key: "employment",
      input: {
        $ref: "#/components/schemas/HrAppointInput",
      },
      result: {
        $ref: "#/components/schemas/OntologyActionExecuteOutcome",
      },
      permissions: [
        "role_manage",
      ],
      four_eyes: "natural_person",
      edits: [
        "employment_heads",
        "employment_revisions",
        "employment_source_bindings",
      ],
      concurrency: {
        command_id: "tenant_global_idempotency",
        expected_revision: "optional_cas",
      },
    },
    {
      action_key: "hr.promote",
      object_key: "employment",
      input: {
        $ref: "#/components/schemas/HrPromoteInput",
      },
      result: {
        $ref: "#/components/schemas/OntologyActionExecuteOutcome",
      },
      permissions: [
        "role_manage",
      ],
      four_eyes: "natural_person",
      edits: [
        "employment_revisions",
        "employees",
      ],
      concurrency: {
        command_id: "tenant_global_idempotency",
        expected_revision: "optional_cas",
      },
    },
    {
      action_key: "hr.transfer",
      object_key: "employment",
      input: {
        $ref: "#/components/schemas/HrTransferInput",
      },
      result: {
        $ref: "#/components/schemas/OntologyActionExecuteOutcome",
      },
      permissions: [
        "role_manage",
      ],
      four_eyes: "natural_person",
      edits: [
        "employment_revisions",
        "employees",
      ],
      concurrency: {
        command_id: "tenant_global_idempotency",
        expected_revision: "optional_cas",
      },
    },
  ],
  concurrency: {
    get_token: "version",
    write_field: "expected_revision",
    write_in: "body",
  },
} as const;

/**
 * Canonical PayRun scaffold over existing `payroll_draft_runs`. Not SAP-complete. Draft calculate is admitted on POST /api/v1/payroll/runs/{id}/calculate (not a DispatchTarget). `payable` is always false. Payment execution, Korea compliance conclusions, and wage-statement legal sign-off remain HOLD. REST list/detail DTOs remain `PayrollRunSummary` / `PayrollRunDetail`. No outgoing Head FK; the run is Company-scoped via RLS `org_id`, which is the cell, not a Palantir link. Temporal slices other than draft status remain HOLD.
 */
export type PayRun = {
  id: Uuid;
  period_start: string;
  period_end: string;
  source_label: string;
  status: "STAGED" | "BLOCKED_LEGAL_GATE" | "READY_FOR_REVIEW" | "ATTENDANCE_CLOSED" | "CALCULATING" | "CALCULATED" | "SUBMITTED" | "REJECTED" | "APPROVED" | "DISBURSEMENT_SCHEDULED" | "PAID" | "ISSUED" | "VOID";
  payable: false;
};

export const PayRunDefinition = {
  name: "PayRun",
  links: [],
  actions: [
    {
      action_key: "payroll.create_run",
      object_key: "pay_run",
      input: {
        $ref: "#/components/schemas/PayrollCreateRunInput",
      },
      result: {
        $ref: "#/components/schemas/OntologyActionExecuteOutcome",
      },
      permissions: [
        "role_manage",
      ],
      four_eyes: "natural_person",
      edits: [
        "payroll_draft_runs",
      ],
      concurrency: {
        command_id: "tenant_global_idempotency",
        expected_revision: "optional_cas",
      },
    },
    {
      action_key: "payroll.submit_run",
      object_key: "pay_run",
      input: {
        $ref: "#/components/schemas/PayrollSubmitRunInput",
      },
      result: {
        $ref: "#/components/schemas/OntologyActionExecuteOutcome",
      },
      permissions: [
        "role_manage",
      ],
      four_eyes: "natural_person",
      edits: [
        "payroll_draft_runs",
      ],
      concurrency: {
        command_id: "tenant_global_idempotency",
        expected_revision: "optional_cas",
      },
    },
    {
      action_key: "payroll.decide_run",
      object_key: "pay_run",
      input: {
        $ref: "#/components/schemas/PayrollDecideRunInput",
      },
      result: {
        $ref: "#/components/schemas/OntologyActionExecuteOutcome",
      },
      permissions: [
        "role_manage",
      ],
      four_eyes: "natural_person",
      edits: [
        "payroll_draft_runs",
      ],
      concurrency: {
        command_id: "tenant_global_idempotency",
        expected_revision: "optional_cas",
      },
    },
  ],
  concurrency: {
    get_token: null,
    write_field: "expected_revision",
    write_in: "body",
  },
} as const;

export type Timestamp = string;

export type Uuid = string;
