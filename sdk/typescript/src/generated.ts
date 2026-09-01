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
  permissions: [
    "employee_directory_read",
  ],
} as const;

/**
 * Canonical OrgUnit head. GET /api/v1/org-units/{id} returns this current head. GET /api/v1/org-units returns current heads (the same rows `list` already returns). GET /api/v1/org-units/{id}/job-positions returns current JobPosition heads under this unit (the same rows `list_for_org_unit` already returns). as_of / from / to are omitted: `org_units` / `org_unit_revisions` have no valid-time columns, and using `created_at` as `valid_from` would be a second time model. `name` and `parent_id` are parsed from the latest `org_unit_revisions.attributes`; neither is a column on `org_units`. Sites, regions, and branches stay the operational auth spine and are not OrgUnits. OrgUnit kinds Site/Department/Team are write-preflight on revision attributes (`kind` ∈ {site, department, team}) and are not a Head column. Temporal slices other than revision `version` remain HOLD.
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
    {
      key: "org_unit_job_positions",
      from: "OrgUnit",
      to: "JobPosition",
      field: "org_unit_id",
      cardinality: "one-to-many",
      option: false,
      operationId: "listOrgUnitJobPositions",
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
  permissions: [
    "employee_directory_read",
  ],
} as const;

/**
 * Canonical JobPosition head. Distinct from recruiting postings and from `employees.position` TEXT. GET /api/v1/job-positions/{id} returns this current head. GET /api/v1/job-positions returns current heads (the same rows `list` already returns). as_of / from / to are omitted: `job_positions` / `job_position_revisions` have no valid-time columns, and using `created_at` as `valid_from` would be a second time model. `org_unit_id` is a foreign key to an existing OrgUnit. `attributes` is the latest revision bag; write preflight requires a non-empty string `title` (catalog JOB_POSITION_TITLE). Other attribute keys are not a published closed set. Temporal slices other than revision `version` remain HOLD.
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
  permissions: [
    "employee_directory_read",
  ],
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
  permissions: [
    "employee_directory_read",
  ],
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
      operationId: "getJobPosition",
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
  permissions: [
    "employee_directory_read",
  ],
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
  permissions: [],
} as const;

export type AbsenceExitDashboardResponse = {
  summary: AbsenceExitSummary;
  alerts: Array<EmployeeAbsenceAlertResponse>;
  exit_cases: Array<EmployeeExitCaseResponse>;
};

export type AbsenceExitSummary = {
  open_absence_alerts: number;
  exit_cases_pending_hr: number;
  settlement_needs_source: number;
  settlement_ready: number;
  approval_drafts: number;
  submitted: number;
};

export type AccountDrillEntry = {
  voucher_id: Uuid;
  voucher_no: string;
  status: VoucherStatus;
  line_id: Uuid;
  account_code: string;
  side: DebitCredit;
  amount_won: number;
  source_object_type?: string | null;
  source_object_id?: string | null;
  entry_at: Timestamp;
};

/**
 * Derived account-setup state for the console roster. ACTIVE only once the user has enrolled a passkey (can sign in); PENDING_SETUP when created / OTP-issued but not yet enrolled; ARCHIVED when soft-disabled.
 */
export type AccountStatus = "ACTIVE" | "PENDING_SETUP" | "ARCHIVED";

/**
 * Where the acquisition figure that anchors TCO came from.
 */
export type AcquisitionBasis = "EXPLICIT" | "VEHICLE_VALUE_FALLBACK" | "NONE";

export type ActingRule = {
  id: string;
  label: string;
  kind: "automation" | "policy";
};

/**
 * One unified actionable item. Field names mirror the overview prototype's items[] shape. Source-partial fields (site/who/due/submitted) are absent when the originating source does not carry them.
 */
export type ActionInboxItem = {
  id: string;
  kind: "approval" | "dispatch" | "work" | "support";
  urg: "now" | "today" | "wait";
  ref: string;
  title: string;
  site?: string;
  who?: string;
  due?: Timestamp;
  dueTone: "danger" | "warn" | "neutral";
  submitted?: Timestamp;
  links: Array<ActionInboxLink>;
  done: boolean;
};

/**
 * One server-ordered canonical source-object reference for an action-inbox item. Clients resolve only explicitly registered kind/id pairs; current browser-linkable kinds are approval_run, work_order, and support_ticket. Unknown kinds remain valid for forward compatibility and MUST stay inert until a client registry adds support. Clients MUST NOT accept a server URL or infer a destination from the enclosing item kind, item id, label, or a code prefix.
 */
export type ActionInboxLink = {
  kind: string;
  id: string;
  label?: string;
};

export type ActionInboxResponse = {
  items: Array<ActionInboxItem>;
  total: number;
  total_is_exact: boolean;
  next_cursor: string | null;
};

export type ActionTypeSummary = {
  id: string;
  stable_key: string;
  title: string;
  params_schema: {
    [key: string]: unknown;
  };
  edits: {
    [key: string]: unknown;
  };
  submission_criteria: {
    [key: string]: unknown;
  };
  side_effects: {
    [key: string]: unknown;
  };
  dispatch: "projected_usecase" | "instance_revision";
  dispatch_target?: string | null;
  control_points: {
    [key: string]: unknown;
  };
};

export type AddCommentRequest = {
  body: string;
  is_internal_note?: boolean;
};

export type AddEvaluationSubjectRequest = {
  cycle_id: Uuid;
  employee_id: Uuid;
  manager_user_id: Uuid;
};

/**
 * The amount_period is inherited from the offer being adjusted and cannot be changed.
 */
export type AdjustRecruitOfferRequest = {
  amount: string;
  reply_deadline?: string | null;
};

export type AdminCredentialResetRequest = {
  user_id: Uuid;
};

export type AdminCredentialResetResponse = {
  user_id: Uuid;
  otp: string;
  expires_at: Timestamp;
};

export type AdminIssueOtpRequest = {
  user_id: Uuid;
  branch_id: Uuid;
  ttl_seconds?: number;
};

export type AdminIssueOtpResponse = {
  user_id: Uuid;
  otp: string;
  expires_at: Timestamp;
};

export type AdminWorkflowRunListResponse = {
  items: Array<WorkflowRunListItem>;
  next_cursor?: Uuid;
};

export type AdmissibilityStatus = "ADMISSIBLE" | "REVIEW_NEEDED" | "BLOCKED" | "INADMISSIBLE";

export type AdvanceRecruitApplicantRequest = {
  expected_updated_at: Timestamp;
};

export type AnalyticSummary = {
  id: string;
  key: string;
  title: string;
  formula: {
    [key: string]: unknown;
  };
  result_type: {
    [key: string]: unknown;
  };
};

/**
 * A single Android Digital Asset Links statement authorizing the native app's signing keys to provide login credentials for the RP domain.
 */
export type AndroidAssetLinkStatement = {
  relation: Array<string>;
  target: {
    namespace: string;
    package_name: string;
    sha256_cert_fingerprints: Array<string>;
  };
};

export type AppendManualCostLedgerRequest = {
  branch_id: Uuid;
  work_order_id?: string | null;
  amount_won: number;
  memo: string;
  config: FinancialConfigSnapshot;
};

/**
 * Apple App Site Association document authorizing the native iOS app's passkeys for the RP domain.
 */
export type AppleAppSiteAssociation = {
  webcredentials: {
    apps: Array<string>;
  };
};

/**
 * A typed approval task over an ontology object. The source-specific payloads are mutually exclusive; `source` selects which payload is populated.
 */
export type ApprovalItem = {
  id: string;
  source: "WORK_ORDER" | "DAILY_PLAN" | "TARGET_CHANGE";
  source_id: Uuid;
  branch_id: Uuid;
  status: string;
  title: string;
  summary: string;
  requested_at: string | null;
  due_at: string | null;
  href: string;
  action_href: string;
  ontology: ApprovalOntologyContext;
  workflow: ApprovalWorkflowContext;
  policy: ApprovalPolicyContext;
  work_order?: WorkOrderListItem;
  daily_plan?: DailyPlanSummary;
  target_change?: TargetChangeRequestSummary;
};

export type ApprovalItemSource = {
  key: "workOrders" | "dailyPlans" | "targetChanges";
  label: string;
  status: "ok";
  count: number;
};

export type ApprovalItemsPage = {
  items: Array<ApprovalItem>;
  sources: Array<ApprovalItemSource>;
  limit: number;
  offset: number;
  total: number;
};

/**
 * Tenant/org/branch/object identity for ontology-aware workflow, analytics, audit, and future group/organization hierarchy views.
 */
export type ApprovalOntologyContext = {
  object_type: "WORK_ORDER" | "DAILY_PLAN" | "TARGET_CHANGE";
  object_id: Uuid;
  tenant_id: Uuid;
  branch_id: Uuid;
};

/**
 * Server policy result and requirements. Clients may display it, but must not treat it as authorization proof; source mutation endpoints always re-check PBAC/RBAC/ABAC server-side.
 */
export type ApprovalPolicyContext = {
  decision: "ALLOWED";
  enforcement: "server";
  required_features: Array<string>;
  scope_kind: "BRANCH";
  scope_id: Uuid;
};

export type ApprovalStepSummary = {
  id: Uuid;
  step_order: number;
  role: string;
  approver_id: string | null;
  approver_name: string | null;
  status: string;
  requested_at: string | null;
  approved_at: string | null;
  approved_by_id: string | null;
  approved_by_name: string | null;
  decision_comment: string | null;
};

export type ApprovalSummary = {
  id: Uuid;
  request_ref: Uuid;
  kind: string;
  requested_by: Uuid;
  approver_id: Uuid;
  decision: "approved" | "rejected";
  decided_at: Timestamp;
};

/**
 * Workflow/action identity for a typed approval task.
 */
export type ApprovalWorkflowContext = {
  workflow_key: string;
  action_key: string;
};

export type ApproveWorkOrderRequest = {
  comment: string;
};

export type ArrivalEvent = {
  id: string;
  work_order_id: Uuid;
  site_id: Uuid;
  work_order_no: string;
  site_name: string;
  customer_name: string;
  mechanic_name: string;
  latitude: number | null;
  longitude: number | null;
  kind: "ARRIVAL" | "DEPARTURE";
  occurred_at: Timestamp;
};

export type ArrivalEventPage = {
  items: Array<ArrivalEvent>;
  limit: number;
  offset: number;
  total: number;
};

export type AssessRecruitApplicantRequest = {
  score: RecruitAssessmentScore;
};

/**
 * Per-asset lifecycle / total-cost-of-ownership rollup. outsource_unlinked_won is read-only and never summed into tco_won.
 */
export type AssetLifecycleCostSummary = {
  equipment_id: Uuid;
  equipment_no: string;
  status: string;
  acquisition_cost_won?: number | null;
  acquisition_date?: string | null;
  acquisition_source: AcquisitionBasis;
  maintenance_total_won: number;
  manual_total_won: number;
  purchase_total_won: number;
  entry_count: number;
  outsource_unlinked_won?: number | null;
  residual_value_won: number;
  sale_price_won?: number | null;
  sold_at?: string | null;
  gross_margin_won?: number | null;
  tco_won: number;
  cost_per_month_won?: number | null;
  cost_per_hour_won?: number | null;
  timeline: Array<CostLedgerEntrySummary>;
};

export type AssignAttendanceSubstituteRequest = {
  site: string;
  branch_id?: Uuid | null;
  role: string;
  cover_date: string;
  from_minutes: number;
  to_minutes: number;
  covered_employee_id: Uuid;
  reason_kind: string;
  reason_detail?: string | null;
  worker_employee_id: Uuid;
  exception_id?: Uuid | null;
};

export type AssignSubstituteRequest = {
  source_equipment_id: string;
  substitute_equipment_id: string;
  assigned_to?: string | null;
  assignment_location: string;
};

export type AssignTicketRequest = {
  assignee_user_id: Uuid;
  branch_id?: Uuid;
};

export type AssignWorkOrderRequest = {
  assignments: Array<{
    mechanic_id: Uuid;
    role: AssignmentRole;
  }>;
  admin_approver_id?: Uuid;
  executive_approver_id?: Uuid;
};

export type AssignmentRole = "PRIMARY" | "SECONDARY";

export type AssignmentSummary = {
  id: Uuid;
  mechanic_id: Uuid;
  mechanic_name: string;
  role: AssignmentRole;
  assigned_at: Timestamp;
};

export type AttachInstanceAck = {
  attached: boolean;
};

/**
 * Attach an instance (kind, id) to a series.
 */
export type AttachInstanceRequest = {
  kind: string;
  id: string;
};

export type AttachmentStage = "REQUEST" | "BEFORE" | "DURING" | "AFTER" | "REPORT" | "OUTSOURCE_RESULT";

export type AttendanceCloseAmendment = {
  id: Uuid;
  reason: string;
  actor: Uuid;
  created_at: string;
};

export type AttendanceCloseAmendmentRequest = {
  reason: string;
  detail: string;
  ref?: string | null;
};

export type AttendanceCloseBoard = {
  month: string;
  items: Array<AttendanceMonthCloseItem>;
};

export type AttendanceCloseCheck = {
  key: string;
  ok: boolean;
  warn?: boolean;
  note?: string | null;
};

export type AttendanceClosePreflight = {
  month: string;
  branch_scope: string;
  checks: Array<AttendanceCloseCheck>;
  can_close: boolean;
};

export type AttendanceCloseRequest = {
  month: string;
  branch_scope?: Uuid | null;
  attest?: boolean | null;
};

export type AttendanceException = {
  id: Uuid;
  code: string;
  kind: "LATE" | "NO_SHOW" | "UNAPPROVED_OVERTIME" | "EARLY_LEAVE";
  status: "OPEN" | "RESOLVED";
  employee_id: Uuid;
  employee_name: string;
  team?: string | null;
  branch_id?: Uuid | null;
  work_date: string;
  occurred_at: string;
  detail: string;
  evidence: Array<AttendanceExceptionEvidence>;
  links: Array<AttendanceExceptionLink>;
  resolution?: AttendanceExceptionResolution | null;
  created_at: string;
};

export type AttendanceExceptionEvidence = {
  name: string;
  size?: string | null;
};

export type AttendanceExceptionLink = {
  kind: string;
  label: string;
  ref?: string | null;
};

export type AttendanceExceptionPage = {
  items: Array<AttendanceException>;
  total: number;
  limit: number;
  offset: number;
};

export type AttendanceExceptionResolution = {
  action: string;
  reason: string;
  linked_work_ref?: string | null;
  ot_hours?: number | null;
  actor: Uuid;
  resolved_at: string;
};

export type AttendanceImportApplyReport = {
  run_id: Uuid;
  inserted: number;
  skipped: number;
  error_rows: number;
};

export type AttendanceImportColumn = {
  source_header: string;
  normalized_header: string;
  target?: "employee_number" | "employee_name" | "branch_name" | "work_date" | "check_in_at" | "check_out_at" | "minutes_worked" | null;
  classification: "canonical" | "retained" | "restricted";
  preview_allowed: boolean;
};

export type AttendanceImportDryRunSummary = {
  run_id: Uuid;
  input_rows: number;
  candidate_rows: number;
  preserved_rows: number;
  ready_rows: number;
  error_rows: number;
  duplicate_rows: number;
  missing_employee_rows: number;
  ambiguous_employee_rows: number;
  row_errors: Array<AttendanceImportRowError>;
};

export type AttendanceImportPreviewResponse = {
  run_id: Uuid;
  entity_type: "attendance_direct";
  source_filename: string;
  source_sha256: string;
  input_rows: number;
  candidate_rows: number;
  preserved_rows: number;
  columns: Array<AttendanceImportColumn>;
  sample_rows: Array<AttendanceImportPreviewRow>;
  mapping_profile: {
    [key: string]: unknown;
  };
};

export type AttendanceImportPreviewRow = {
  source_sheet: string;
  source_row: number;
  row_status: "CANDIDATE" | "PRESERVED" | "ERROR";
  values: {
    [key: string]: unknown;
  };
  validation: {
    [key: string]: unknown;
  };
};

export type AttendanceImportRowError = {
  source_sheet: string;
  source_row: number;
  source_key: string;
  code: string;
  message: string;
};

export type AttendanceImportSummaryItem = {
  run_id: Uuid;
  status: "PREVIEWED" | "DRY_RUN" | "APPLIED" | "FAILED";
  source_filename: string;
  source_format: "xlsx" | "csv";
  source_sha256: string;
  input_rows: number;
  candidate_rows: number;
  preserved_rows: number;
  dry_run_summary: {
    [key: string]: unknown;
  };
  apply_summary: {
    [key: string]: unknown;
  };
  created_at: Timestamp;
  applied_at?: Timestamp | null;
};

export type AttendanceImportSummaryPage = {
  items: Array<AttendanceImportSummaryItem>;
  total: number;
  limit: number;
  offset: number;
};

export type AttendanceMonthClose = {
  id: Uuid;
  month: string;
  branch_scope: string;
  checks: Array<AttendanceCloseCheck>;
  attested_by: Uuid;
  attested_at: string;
  period_lock_id?: Uuid | null;
  closed_at: string;
  amendments: Array<AttendanceCloseAmendment>;
};

export type AttendanceMonthCloseItem = {
  branch_scope: string;
  closed: boolean;
  close?: AttendanceMonthClose | null;
  open_exceptions: number;
  pending_leave: number;
};

export type AttendanceRecordKind = "CLOCK_IN" | "OUT_FOR_WORK" | "BUSINESS_TRIP" | "RETURNED" | "CLOCK_OUT";

export type AttendanceSubstitution = {
  id: Uuid;
  site: string;
  branch_id?: Uuid | null;
  role: string;
  cover_date: string;
  from_minutes: number;
  to_minutes: number;
  covered_employee_id: Uuid;
  covered_name: string;
  reason_kind: string;
  reason_detail?: string | null;
  worker_employee_id?: Uuid | null;
  worker_name: string;
  worker_type: string;
  worker_rate?: string | null;
  status: "ASSIGNED" | "CANCELLED";
  exception_id?: Uuid | null;
  created_by: Uuid;
  created_at: string;
};

export type AttendanceSubstitutionCandidate = {
  employee_id: Uuid;
  employee_name: string;
  branch_id: Uuid;
};

export type AttendanceSubstitutionCandidatePage = {
  items: Array<AttendanceSubstitutionCandidate>;
  total: number;
  limit: number;
  offset: number;
};

export type AttendanceSubstitutionPage = {
  items: Array<AttendanceSubstitution>;
  total: number;
  limit: number;
  offset: number;
};

export type AttendanceSummaryItem = {
  user_id: Uuid;
  display_name: string;
  arrivals: number;
  departures: number;
  last_kind?: string | null;
  last_event_at?: Timestamp | null;
};

export type AttendanceSummaryPage = {
  items: Array<AttendanceSummaryItem>;
  total: number;
  limit: number;
  offset: number;
};

export type AttendanceWeek52AckRequest = {
  employee_id: Uuid;
  week_start: string;
};

export type AttendanceWeek52Board = {
  week_start: string;
  items: Array<AttendanceWeek52Row>;
};

export type AttendanceWeek52Row = {
  employee_id: Uuid;
  name: string;
  team?: string | null;
  week_start: string;
  current_hours: number;
  projected_hours: number;
  tone: "OK" | "WARN" | "DANGER";
  acked: boolean;
  acked_at?: string | null;
};

export type AttestPayrollDisbursementRequest = {
  status: "SCHEDULED" | "SUBMITTED_TO_BANK" | "PAID" | "FAILED";
  reason?: string;
};

export type AuditChainAttestation = {
  org_id: string;
  ok: boolean;
  first_bad_seq?: number | null;
  kind: "ok" | "seal_hash_mismatch" | "batch_hash_mismatch" | "broken_continuity" | "bad_signature" | "missing_seq" | "coverage_gap" | "corrupt_seal";
  unsealed_tail: boolean;
};

export type AuditRecord = {
  id: Uuid;
  actor: Uuid | null;
  action: string;
  target_type: string;
  target_id: string;
  branch_id: Uuid | null;
  before_snap: {
    [key: string]: unknown;
  } | null;
  after_snap: {
    [key: string]: unknown;
  } | null;
  ip: string | null;
  user_agent: string | null;
  auth_method: string | null;
  device: string | null;
  classification_badges: Array<string> | null;
  anomaly: boolean | null;
  reason: string | null;
  trace_id: string;
  span_id: string;
  occurred_at: Timestamp;
};

export type AuditStreamPage = {
  items: Array<AuditStreamRecord>;
  limit: number;
  offset: number;
  total: number;
  stream_key: "ceo_covert_audit";
  read_kind: AuditStreamReadKind;
  access_audit_id: string;
};

export type AuditStreamReadKind = "events" | "access_events";

export type AuditStreamRecord = {
  id: string;
  actor: string | null;
  action: string;
  target_type: string;
  target_id: string;
  sensitivity: string;
  before_snap: {
    [key: string]: unknown;
  } | null;
  after_snap: {
    [key: string]: unknown;
  } | null;
  trace_id: string;
  span_id: string;
  occurred_at: Timestamp;
  created_at: Timestamp;
};

export type BenefitCatalogCondition = {
  id?: Uuid;
  benefit_id?: Uuid;
  condition_kind: string;
  operator: string;
  condition_key: string;
  condition_value: {
    [key: string]: unknown;
  };
  display_label: string;
  cedar_policy_ref?: string | null;
  display_order: number;
};

/**
 * Tenant and actor are derived from the verified bearer token; neither can be supplied by callers.
 */
export type BenefitCatalogCreateRequest = {
  scope: BenefitCatalogScope;
  category: "legal" | "extra";
  name: string;
  coverageLabel: string;
  coveredCount?: number | null;
  costLabel: string;
  estimatedAnnualCostWon?: number | null;
  employerRateBps?: number | null;
  note?: string | null;
  legalBasis?: string | null;
  relatedDomain?: string | null;
  relatedObjectId?: Uuid | null;
  effectiveOn?: string | null;
  retiresOn?: string | null;
  displayOrder?: number;
  metadata?: {
    [key: string]: unknown;
  };
  tiers?: Array<BenefitCatalogTier>;
  conditions?: Array<BenefitCatalogCondition>;
};

export type BenefitCatalogItem = {
  id: Uuid;
  benefit_code: string;
  category: "legal" | "extra";
  name: string;
  scope: BenefitCatalogScope;
  coverage_label: string;
  covered_count?: number | null;
  cost_label: string;
  estimated_annual_cost_won?: number | null;
  employer_rate_bps?: number | null;
  note?: string | null;
  legal_basis?: string | null;
  related_domain?: string | null;
  related_object_id?: Uuid | null;
  effective_on?: string | null;
  retires_on?: string | null;
  tiers: Array<BenefitCatalogTier>;
  conditions: Array<BenefitCatalogCondition>;
  lifecycle: BenefitCatalogLifecycleBinding;
};

export type BenefitCatalogItemPage = {
  items: Array<BenefitCatalogItem>;
  limit: number;
  offset: number;
  total: number;
};

export type BenefitCatalogLifecycleBinding = {
  object_type: "benefit_catalog_item";
  object_id: Uuid;
  current_state?: string | null;
  legal_hold?: boolean | null;
  retention_until?: string | null;
};

export type BenefitCatalogReplaceConditionsRequest = {
  conditions: Array<BenefitCatalogCondition>;
};

export type BenefitCatalogReplaceTiersRequest = {
  tiers: Array<BenefitCatalogTier>;
};

export type BenefitCatalogScope = {
  scope_type: "ORG" | "BRANCH" | "SITE" | "TEAM" | "ROLE" | "EMPLOYEE_SEGMENT";
  scope_ref?: Uuid | null;
  branch_id?: Uuid | null;
  site_id?: Uuid | null;
};

export type BenefitCatalogTier = {
  id?: Uuid;
  benefit_id?: Uuid;
  tier_basis: string;
  tier_key: string;
  value_label: string;
  amount_won?: number | null;
  limit_period?: string | null;
  criteria: {
    [key: string]: unknown;
  };
  display_order: number;
};

/**
 * Mutable catalog fields only; tenant, actor, identifiers, and lifecycle state are server-owned.
 */
export type BenefitCatalogUpdateRequest = {
  category?: "legal" | "extra";
  name?: string;
  scope?: BenefitCatalogScope;
  coverageLabel?: string;
  coveredCount?: number | null;
  costLabel?: string;
  estimatedAnnualCostWon?: number | null;
  employerRateBps?: number | null;
  note?: string | null;
  legalBasis?: string | null;
  relatedDomain?: string | null;
  relatedObjectId?: Uuid | null;
  effectiveOn?: string | null;
  retiresOn?: string | null;
  displayOrder?: number;
  metadata?: {
    [key: string]: unknown;
  };
};

/**
 * The set of branches a principal may act within. `all` (SUPER_ADMIN / EXECUTIVE rollup) carries no `branches`; `branches` carries the explicit set.
 */
export type BranchScope = {
  kind: "all" | "branches";
  branches?: Array<Uuid>;
};

export type BranchSummary = {
  id: Uuid;
  region_id: Uuid;
  name: string;
  deactivated_at: string | null;
  created_at: Timestamp;
};

export type BulkApprovalCapability = {
  decidable: boolean;
  reason?: "CLAIMED_BY_ANOTHER_USER";
};

export type BulkApprovalInboxResponse = {
  items: Array<BulkApprovalTask>;
  has_more: boolean;
  next_cursor?: string;
};

export type BulkApprovalTask = {
  task_id: Uuid;
  run_id: Uuid;
  waiting_key: string;
  title: string;
  assignee_role_key?: string;
  status: string;
  claimed_by?: Uuid;
  due_at?: Timestamp;
  bulk_decision: BulkApprovalCapability;
};

export type BulkAuthorizeBody = {
  subject: PolicySimSubject;
  checks: Array<{
    action: string;
    resource: PolicySimResource;
    purpose?: string | null;
    field?: string | null;
  }>;
};

export type BulkDecisionResponse = {
  decisions: Array<SimulationOutcome>;
};

export type CalendarEventListResponse = {
  items: Array<CalendarEventResponse>;
};

export type CalendarEventResponse = {
  id: Uuid;
  scope_type: CollaborationScopeType;
  scope_ref?: string | null;
  title: string;
  description: string;
  starts_at: Timestamp;
  ends_at: Timestamp;
  all_day: boolean;
  status: CalendarEventStatus;
  object_type?: string | null;
  object_id?: Uuid | null;
  created_by?: Uuid | null;
  created_at: Timestamp;
  updated_at: Timestamp;
  policy: CollaborationScopePolicy;
};

export type CalendarEventStatus = "ACTIVE" | "CANCELLED";

export type CalibrateEvaluationSubjectRequest = {
  final_grade: EvaluationGrade;
  reason?: string | null;
};

export type CancelAttendanceSubstitutionRequest = {
  reason: string;
};

export type CancelOrgChangeRequest = {
  reason: string;
};

export type CatalogEntry = {
  id: Uuid;
  stable_key: string;
  title: string;
  effect: string;
  status: string;
  source: string;
  validation_status: string;
  updated_at: Timestamp;
};

export type ClaimWorkflowTaskRequest = {
  idempotency_key: string;
};

export type ClaimWorkflowTaskResponse = {
  task: ClaimedWorkflowTask;
};

export type ClaimedWorkflowTask = {
  task_id: Uuid;
  run_id: Uuid;
  status: string;
  claimed_by?: Uuid;
  claimed_at?: Timestamp;
};

export type CloneWorkflowDefinitionRequest = {
  workflow_key?: string;
  display_name?: string;
  step_up?: PasskeyStepUpAssertion;
};

export type ClosePayrollAttendanceRequest = {
  attest: boolean;
};

export type CloseRecruitPostingRequest = {
  expected_updated_at: Timestamp;
};

export type CollaborationScopePolicy = {
  enforcement: "server";
  scope_type: CollaborationScopeType;
  scope_ref?: string | null;
  visibility: "org_members" | "creator_only" | "department_target" | "team_target";
};

export type CollaborationScopeType = "TENANT" | "ORG" | "DEPARTMENT" | "TEAM" | "PERSONAL";

export type CompleteInspectionRoundRequest = {
  outcome: InspectionRoundOutcome;
  completed_at?: string | null;
  findings: string;
  note?: string | null;
};

export type CompleteOrgChangeSettlementItemRequest = {
  memo?: string;
};

export type ComplianceControl = {
  id: Uuid;
  framework_id: Uuid;
  control_key: string;
  title: string;
  objective: string;
  control_type: ControlType;
  cadence?: "CONTINUOUS" | "DAILY" | "WEEKLY" | "MONTHLY" | "QUARTERLY" | "ANNUAL" | "EVENT_DRIVEN" | null;
  status: ControlStatus;
  evidence_requirements: {
    [key: string]: unknown;
  };
  owner_user_id?: string | null;
  created_by: Uuid;
  updated_by: Uuid;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type ComplianceControlPage = {
  items: Array<ComplianceControl>;
  limit: number;
  offset: number;
  total: number;
};

export type ComplianceFramework = {
  id: Uuid;
  code: string;
  name: string;
  version_label: string;
  framework_kind: FrameworkKind;
  status: FrameworkStatus;
  owner_user_id?: string | null;
  effective_from?: string | null;
  effective_to?: string | null;
  metadata: {
    [key: string]: unknown;
  };
  created_by: Uuid;
  updated_by: Uuid;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type ComplianceFrameworkPage = {
  items: Array<ComplianceFramework>;
  limit: number;
  offset: number;
  total: number;
};

export type ComplianceObligation = {
  id: Uuid;
  code: string;
  title: string;
  description: string;
  obligation_type: ObligationType;
  scope: ComplianceScope;
  owner_user_id?: string | null;
  severity: ComplianceRiskLevel;
  status: ObligationStatus;
  effective_from?: string | null;
  effective_to?: string | null;
  review_cadence?: "MONTHLY" | "QUARTERLY" | "SEMI_ANNUAL" | "ANNUAL" | "EVENT_DRIVEN" | null;
  next_review_on?: string | null;
  metadata: {
    [key: string]: unknown;
  };
  created_by: Uuid;
  updated_by: Uuid;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type ComplianceObligationPage = {
  items: Array<ComplianceObligation>;
  limit: number;
  offset: number;
  total: number;
};

export type ComplianceRiskLevel = "INFO" | "LOW" | "MEDIUM" | "HIGH" | "CRITICAL";

/**
 * ORG carries no IDs; BRANCH requires matching branch_id and scope_ref; SITE requires branch_id and matching site_id/scope_ref. TEAM and ROLE are not accepted for writes yet.
 */
export type ComplianceScope = {
  kind: ComplianceScopeKind;
  scope_ref?: string | null;
  branch_id?: string | null;
  site_id?: string | null;
};

export type ComplianceScopeKind = "ORG" | "BRANCH" | "SITE" | "TEAM" | "ROLE";

export type ComputeRentalQuoteRequest = {
  branch_id: Uuid;
  acquisition_value_won: number;
  current_residual_value_won: number;
  cumulative_repair_cost_won: number;
  config: FinancialConfigSnapshot;
};

export type ComputedRentalQuote = {
  effective_residual_value: number;
  residual_was_floored: boolean;
  lines: Array<QuoteLine>;
  monthly_total: number;
};

export type ConditionValue = ConditionValueLiteral | ConditionValueSubjectAttr | ConditionValueBool;

export type ConditionValueBool = {
  kind: "bool";
  value: boolean;
};

export type ConditionValueLiteral = {
  kind: "literal";
  value: string;
};

export type ConditionValueSubjectAttr = {
  kind: "subject_attr";
  value: string;
};

/**
 * Configure (create or replace) the mailbox. A password field that is absent or null leaves the stored secret unchanged; a present value is re-sealed. A first-time configure requires both passwords.
 */
export type ConfigureMailAccountRequest = {
  display_name: string;
  email_address: string;
  from_name?: string | null;
  imap_host: string;
  imap_port: number;
  imap_security: MailSecurity;
  imap_username: string;
  imap_password?: string | null;
  smtp_host: string;
  smtp_port: number;
  smtp_security: MailSecurity;
  smtp_username: string;
  smtp_password?: string | null;
};

export type ConsoleRouteSurface = "console" | "legacy";

export type ConsoleRouteTelemetryAccepted = {
  accepted: boolean;
};

export type ConsoleRouteTelemetryEventKind = "route_selection" | "rum_error" | "rum_perf";

export type ConsoleRouteTelemetryRequest = {
  event_kind: ConsoleRouteTelemetryEventKind;
  route_surface: ConsoleRouteSurface;
  route_path: string;
  release_cycle: string;
  duration_ms?: number | null;
  error_name?: string | null;
};

export type ConsultingBenefitObservation = {
  id: Uuid;
  initiative_id: Uuid;
  kpi_definition_id: Uuid;
  evidence_id: Uuid;
  observed_at: string;
  note: string;
  created_at: string;
};

export type ConsultingDiagnostic = {
  id: Uuid;
  summary: string;
  document_id?: Uuid | null;
  created_at: string;
};

export type ConsultingDiagnosticCreateRequest = {
  summary: string;
  documentId?: Uuid | null;
};

export type ConsultingEngagement = {
  id: Uuid;
  customer_id: Uuid;
  customer_document_id?: Uuid | null;
  ontology_instance_id?: Uuid | null;
  title: string;
  status: "DRAFT" | "PROPOSED" | "APPROVED" | "IMPLEMENTED" | "MEASURED" | "SUSTAINED" | "CORRECTIVE";
  approval_id?: Uuid | null;
  workflow_execution_id?: Uuid | null;
  version: number;
  created_at: string;
  updated_at: string;
};

export type ConsultingEngagementCreateRequest = {
  customerId: Uuid;
  customerDocumentId?: Uuid | null;
  ontologyInstanceId?: Uuid | null;
  title: string;
  idempotencyKey: string;
};

export type ConsultingEngagementDetail = ConsultingEngagement & {
  diagnostics: Array<ConsultingDiagnostic>;
  findings: Array<ConsultingFinding>;
  initiatives: Array<ConsultingInitiative>;
  observations: Array<ConsultingBenefitObservation>;
};

export type ConsultingEngagementPage = {
  items: Array<ConsultingEngagement>;
  limit: number;
  offset: number;
  total: number;
};

export type ConsultingFinding = {
  id: Uuid;
  diagnostic_id: Uuid;
  statement: string;
  evidence_id: Uuid;
  document_id?: Uuid | null;
  created_at: string;
};

export type ConsultingFindingCreateRequest = {
  diagnosticId: Uuid;
  statement: string;
  evidenceId: Uuid;
  documentId?: Uuid | null;
};

export type ConsultingHistoryEntry = {
  id: Uuid;
  event_type: string;
  from_status?: string | null;
  to_status?: string | null;
  version: number;
  payload: {
    [key: string]: unknown;
  };
  occurred_at: string;
};

export type ConsultingInitiative = {
  id: Uuid;
  finding_id: Uuid;
  title: string;
  hypothesis: string;
  kpi_definition_id: Uuid;
  target_direction: "INCREASE" | "DECREASE";
  created_at: string;
};

export type ConsultingInitiativeCreateRequest = {
  findingId: Uuid;
  title: string;
  hypothesis: string;
  kpiDefinitionId: Uuid;
  targetDirection: "INCREASE" | "DECREASE";
};

export type ConsultingObservationCreateRequest = {
  initiativeId: Uuid;
  kpiDefinitionId: Uuid;
  evidenceId: Uuid;
  observedAt: string;
  note: string;
};

export type ConsultingTransitionRequest = {
  toStatus: "PROPOSED" | "APPROVED" | "IMPLEMENTED" | "MEASURED" | "SUSTAINED" | "CORRECTIVE";
  expectedVersion: number;
  approvalId?: Uuid | null;
  reason: string;
};

export type ConsumeInventoryItemRequest = {
  source: InventoryConsumptionSource;
  quantityConsumedMilli: number;
  occurredAt?: string | null;
  memo?: string | null;
  idempotencyKey: string;
};

export type ControlObligationCoverage = {
  id: Uuid;
  control_id: Uuid;
  obligation_id: Uuid;
  coverage_level: CoverageLevel;
  coverage_rationale?: string | null;
  status: CoverageStatus;
  created_by: Uuid;
  updated_by: Uuid;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type ControlStatus = "DRAFT" | "ACTIVE" | "RETIRED" | "ARCHIVED";

export type ControlType = "PREVENTIVE" | "DETECTIVE" | "CORRECTIVE" | "DIRECTIVE" | "COMPENSATING";

export type CopyVerification = {
  copy_id: string;
  copy_kind: EvidenceCopyKind;
  recorded_digest_sha256: string;
  storage_checksum_sha256?: string | null;
  recorded_size_bytes: number;
  storage_size_bytes?: number | null;
  status: FixityStatus;
};

export type CostLedgerEntrySummary = {
  id: Uuid;
  branch_id: Uuid;
  equipment_id: Uuid;
  work_order_id?: string | null;
  purchase_request_id?: string | null;
  source: CostLedgerSource;
  amount_won: number;
  memo: string;
  residual_before_won: number;
  residual_after_won: number;
  entry_at: Timestamp;
};

export type CostLedgerSource = "MANUAL_ADMIN" | "PURCHASE_EXECUTION";

export type CoverageLevel = "PRIMARY" | "PARTIAL" | "SUPPORTING" | "COMPENSATING";

export type CoverageStatus = "ACTIVE" | "RETIRED";

export type CreateBranchRequest = {
  region_id: Uuid;
  name: string;
};

export type CreateCalendarEventRequest = {
  scope_type: CollaborationScopeType;
  scope_ref?: string | null;
  title: string;
  description?: string;
  starts_at: Timestamp;
  ends_at: Timestamp;
  all_day?: boolean;
  object_type?: string | null;
  object_id?: Uuid | null;
};

export type CreateComplianceControlRequest = {
  framework_id: Uuid;
  control_key: string;
  title: string;
  objective: string;
  control_type: ControlType;
  cadence?: "CONTINUOUS" | "DAILY" | "WEEKLY" | "MONTHLY" | "QUARTERLY" | "ANNUAL" | "EVENT_DRIVEN" | null;
  evidence_requirements?: {
    [key: string]: unknown;
  };
  owner_user_id?: string | null;
};

export type CreateComplianceFrameworkRequest = {
  name: string;
  version_label: string;
  framework_kind: FrameworkKind;
  owner_user_id?: string | null;
  effective_from?: string | null;
  effective_to?: string | null;
  metadata?: {
    [key: string]: unknown;
  };
};

export type CreateComplianceObligationRequest = {
  title: string;
  description: string;
  obligation_type: ObligationType;
  scope: ComplianceScope;
  owner_user_id?: string | null;
  severity: ComplianceRiskLevel;
  effective_from?: string | null;
  effective_to?: string | null;
  review_cadence?: "MONTHLY" | "QUARTERLY" | "SEMI_ANNUAL" | "ANNUAL" | "EVENT_DRIVEN" | null;
  next_review_on?: string | null;
  metadata?: {
    [key: string]: unknown;
  };
  regulation_links?: Array<RegulationLinkRequest>;
};

export type CreateCustomerRequest = {
  name: string;
};

export type CreateDailyPlanRequest = {
  branch_id: Uuid;
  mechanic_id: Uuid;
  plan_date: Date;
  items: Array<{
    work_order_id: Uuid;
    description: string;
  }>;
};

export type CreateEmployeeAttendanceRecordRequest = {
  kind: AttendanceRecordKind;
  idempotency_key: string;
  note?: string | null;
};

export type CreateEmployeeLifecycleEventRequest = {
  event_type: "ONBOARD" | "OFFBOARD" | "TERMINATE" | "TRANSFER";
  to_status?: "ACTIVE" | "EXITED" | "UNKNOWN" | null;
  to_company?: string | null;
  to_org_unit?: string | null;
  to_position?: string | null;
  effective_date: string;
  comment: string;
  signoffs: EmployeeLifecycleSignoffs;
};

export type CreateEmployeeRequest = {
  employee_number: string;
  name: string;
  company: string;
  employment_type: "REGULAR" | "CONTRACT" | "PART_TIME" | "INTERN";
  phone: string;
  org_unit: string;
  position: string;
  site: string;
  home_branch_id: Uuid;
  base_pay: string;
  idempotency_key: string;
};

export type CreateEquipmentRequest = {
  equipment_no: string;
  customer_name: string;
  site_name: string;
  status: EquipmentStatus;
  specification: string;
  ton_text: string;
  management_no?: string | null;
  power_label?: string | null;
  manager_name?: string | null;
  placement_location?: string | null;
  placement_no?: string | null;
  operation_shift?: string | null;
  maker?: string | null;
  model?: string | null;
  vin?: string | null;
  year?: string | null;
  hours?: number | null;
  vehicle_registration_no?: string | null;
  insured?: boolean | null;
  insurer?: string | null;
  policy_holder?: string | null;
  insured_party?: string | null;
  asset_owner?: string | null;
  asset_registered_on?: string | null;
  rental_started_on?: string | null;
  rental_fee?: number | null;
  vehicle_value?: number | null;
  residual_value?: number | null;
  note?: string | null;
};

export type CreateEquipmentResponse = {
  id: Uuid;
};

export type CreateEvaluationCycleRequest = {
  name: string;
  kind: EvaluationCycleKind;
  period_label: string;
  due_date: Date;
};

export type CreateEvidenceBindingRequest = {
  control_id: Uuid;
  obligation_id?: string | null;
  evidence_target_type: EvidenceTargetType;
  evidence_target_id: string;
  source_audit_event_id?: string | null;
  confidence: EvidenceConfidence;
  collected_at?: string | null;
  collected_by?: string | null;
  valid_from?: string | null;
  valid_to?: string | null;
  hash_sha256?: string | null;
  metadata?: {
    [key: string]: unknown;
  };
};

export type CreateInspectionScheduleRequest = {
  branch_id: Uuid;
  equipment_id: Uuid;
  mechanic_id: Uuid;
  cycle: InspectionCycle;
  interval_days: number;
  due_date: Date;
  note?: string | null;
};

export type CreateInternalTicketRequest = {
  branch_id: Uuid;
  category: SupportTicketCategory;
  priority: SupportTicketPriority;
  title: string;
  body: string;
};

/**
 * Full editable field set for creating a sales listing.
 */
export type CreateListingRequest = {
  kind: ListingKind;
  condition: ListingCondition;
  model_name: string;
  capacity_milli?: number | null;
  model_year?: number | null;
  usage_hours?: number | null;
  price_won?: number | null;
  badge?: string | null;
  usage_label?: string | null;
  condition_label?: string | null;
  availability?: string | null;
  location?: string | null;
  description?: string | null;
  listing_type: ListingType;
  status: ListingStatus;
  sort_weight: number;
  equipment_id?: Uuid | null;
};

export type CreateListingResponse = {
  id: Uuid;
};

export type CreateMessengerThreadRequest = {
  branch_id: Uuid;
  kind: MessengerThreadKind;
  visibility?: MessengerThreadVisibility | null;
  title?: string | null;
  work_order_id?: string | null;
  member_ids: Array<Uuid>;
};

export type CreateNoticeDraftRequest = {
  title: string;
  body: string;
  category?: NoticeCategory;
  audience?: NoticeAudienceInput;
};

/**
 * Request to create a directed link between two known objects.
 */
export type CreateObjectLinkRequest = {
  src_kind: string;
  src_id: string;
  dst_kind: string;
  dst_id: string;
  link_type: string;
};

/**
 * Draft object-type schema (registry backbone) with its property/link/action/analytic children.
 */
export type CreateObjectTypeDraft = {
  stable_key: string;
  title: string;
  title_property_key?: string;
  backing_kind: "projected" | "instance";
  backing_table?: string;
  primary_key_property?: string;
  properties?: Array<{
    [key: string]: unknown;
  }>;
  links?: Array<{
    [key: string]: unknown;
  }>;
  actions?: Array<{
    [key: string]: unknown;
  }>;
  analytics?: Array<{
    [key: string]: unknown;
  }>;
};

export type CreateOrgChangeRequest = {
  kind: OrgChangeKind;
  target: OrgChangeTarget;
  effectiveDate: Date;
  reason: string;
  proposal: Array<OrgProposalOp>;
  supersedesId?: Uuid;
};

export type CreateOutsourceWorkRequest = {
  vendor_name: string;
  vendor_contact?: string;
  reason: string;
};

export type CreateOwnershipTransferRequest = {
  to_owner: string;
  reason: string;
};

export type CreatePeriodLockRequest = {
  domain: "payroll" | "accounting";
  periodStart: string;
  periodEnd: string;
  reason: string;
};

export type CreatePlatformGroupAccountRequest = {
  org_id: Uuid;
  display_name: string;
  phone?: string;
  tenant_roles?: Array<PlatformTenantRole>;
  group_role?: PlatformGroupRole;
};

export type CreatePlatformGroupAccountResponse = {
  account: PlatformGroupAccount;
  otp: string;
  otp_expires_at: Timestamp;
};

export type CreatePlatformGroupRequest = {
  slug: string;
  name: string;
};

export type CreatePlatformOrgRequest = {
  slug: string;
  name: string;
};

export type CreatePolicyRoleRequest = {
  role_key: string;
  display_name: string;
  description?: string | null;
  permissions: Array<PolicyPermissionResponse>;
  conditions?: Array<PolicyConditionResponse>;
};

export type CreatePollRequest = {
  target_scope_type: CollaborationScopeType;
  target_scope_ref?: string | null;
  title: string;
  question: string;
  status?: PollStatus;
  anonymity?: PollAnonymity;
  allow_multiple?: boolean;
  closes_at?: Timestamp | null;
  options: Array<string>;
  object_type?: string | null;
  object_id?: Uuid | null;
};

export type CreateProductionPlan = {
  branch_id: Uuid;
  customer_demand_id: Uuid;
  capacity_slot_id: Uuid;
  material_item_id: Uuid;
  quantity: number;
  due_at: string;
  ontology_type_id: Uuid;
  idempotency_key: string;
};

export type CreatePurchaseRequest = {
  branch_id: Uuid;
  equipment_id?: string | null;
  work_order_id?: string | null;
  statement_evidence_id?: string | null;
  purchase_type: PurchaseType;
  vendor_name: string;
  amount_won?: number | null;
  lines: Array<PurchaseRequestLineInput>;
  quote_attachment_ids: Array<Uuid>;
  memo: string;
  config: FinancialConfigSnapshot;
};

export type CreateRecruitApplicantRequest = {
  name: string;
  profile_lines?: Array<string>;
  source_document?: string | null;
};

export type CreateRecruitPostingRequest = {
  role_title: string;
  company: string;
  worksite: string;
  employment_type: RecruitEmploymentType;
  scope: RecruitPostingScope;
  headcount: number;
  deadline?: string | null;
  requirements?: Array<string>;
  position_ref?: string | null;
};

export type CreateRegionRequest = {
  name: string;
};

export type CreateRegulationImpactRequest = {
  title: string;
  jurisdiction: string;
  regulator?: string | null;
  citation: string;
  source_url?: string | null;
  impact_area: string;
  impact_summary: string;
  risk_level: ComplianceRiskLevel;
  effective_from?: string | null;
  effective_to?: string | null;
  review_due_on?: string | null;
  owner_user_id?: string | null;
  metadata?: {
    [key: string]: unknown;
  };
};

export type CreateRentalQuoteRequest = {
  branch_id: Uuid;
  equipment_id: Uuid;
  config: FinancialConfigSnapshot;
};

/**
 * Create a series and attach its first instance.
 */
export type CreateSeriesRequest = {
  label: string;
  kind: string;
  id: string;
};

export type CreateSettlementRequest = {
  lines: Array<SettlementLineRequest>;
  note?: string;
};

/**
 * Create a site under an existing customer. Optional location/contact fields follow the same WGS84 ranges and length bounds as UpdateSiteRequest; latitude and longitude must be supplied together.
 */
export type CreateSiteRequest = {
  customer_id: Uuid;
  name: string;
  address?: string | null;
  province?: string | null;
  city?: string | null;
  postal_code?: string | null;
  latitude?: number | null;
  longitude?: number | null;
  geofence_radius_m?: number | null;
  contact_name?: string | null;
  contact_phone?: string | null;
  contact_email?: string | null;
};

export type CreateTodoRequest = {
  text: string;
  scopes?: Array<TodoRef>;
  links?: Array<TodoRef>;
};

export type CreateTriggerBindingRequest = {
  definition_id: Uuid;
  trigger_type: "OBJECT_EVENT" | "IMPORT_EVENT" | "MAIL_EVENT" | "MESSENGER_EVENT" | "CALENDAR_EVENT" | "POLL_EVENT";
  event_key: string;
  subject_kind?: string;
  enabled?: boolean;
};

export type CreateUserRequest = {
  display_name: string;
  employee_id?: string | null;
  phone?: string | null;
  team?: Team;
  roles?: Array<string>;
  branch_ids?: Array<Uuid>;
};

export type CreateVoucherRequest = {
  branch_id: Uuid;
  memo?: string;
  lines: Array<VoucherLineInput>;
};

export type CreateWorkOrderRequest = {
  branch_id: Uuid;
  management_no: string;
  symptom: string;
  customer_request?: string;
  target_due_at?: Timestamp;
  maintenance_type?: MaintenanceType;
  maintenance_cause?: MaintenanceCause;
};

export type CreateWorkflowDefinitionRequest = {
  workflow_key: string;
  display_name: string;
  object_type: string;
  definition?: {
    [key: string]: unknown;
  };
  approval_line?: Array<{
    [key: string]: unknown;
  }>;
  payment_line?: Array<{
    [key: string]: unknown;
  }>;
  notification_rules?: Array<{
    [key: string]: unknown;
  }>;
  action_allowlist?: Array<WorkflowActionAllowlistEntry>;
  required_approval_line?: boolean;
  required_payment_line?: boolean;
};

export type CreateWorkflowScheduleRequest = {
  label: string;
  cron_expr: string;
  timezone?: string;
  definition_id: Uuid;
  enabled?: boolean;
};

export type CreatedCustomer = {
  id: Uuid;
  branch_id: Uuid;
  name: string;
};

export type CreatedSite = {
  id: Uuid;
  customer_id: Uuid;
  branch_id: Uuid;
  name: string;
  address: string | null;
  province: string | null;
  city: string | null;
  postal_code: string | null;
  latitude: number | null;
  longitude: number | null;
  geofence_radius_m: number | null;
  contact_name: string | null;
  contact_phone: string | null;
  contact_email: string | null;
};

export type CustodyEventView = {
  id: string;
  evidence_object_id: string;
  stage: CustodyStage;
  actor_user_id: string;
  from_custodian?: {
    [key: string]: unknown;
  } | null;
  to_custodian?: {
    [key: string]: unknown;
  } | null;
  location_label?: string | null;
  reason: string;
  source_ref?: EvidenceSourceRef | null;
  audit_event_id?: string | null;
  previous_event_id?: string | null;
  event_digest_sha256: string;
  occurred_at: string;
  created_at: string;
};

export type CustodyStage = "REGISTERED" | "HASH_RECORDED" | "TSA_SUBMITTED" | "TSA_VERIFIED" | "WORM_REPLICATED" | "CUSTODY_TRANSFERRED" | "UNDER_REVIEW" | "ADMISSIBILITY_EVALUATED" | "LEGAL_HOLD_APPLIED" | "LEGAL_HOLD_RELEASED" | "EXPORTED" | "ARCHIVED" | "DISPOSAL_REQUESTED" | "DISPOSED";

export type CustomerInquiryPage = {
  items: Array<CustomerInquiryView>;
  limit: number;
  offset: number;
  total: number;
};

export type CustomerInquiryView = {
  id: Uuid;
  name: string;
  phone: string;
  topic: InquiryTopic;
  location: string | null;
  message: string | null;
  listing_id: Uuid | null;
  status: InquiryStatus;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type CustomerIntakeRequest = {
  category: SupportTicketCategory;
  priority: SupportTicketPriority;
  title: string;
  body: string;
  requester_name: string;
  requester_contact: string;
};

export type CycleCount = {
  id: Uuid;
  ccCode: string;
  branchId: Uuid;
  stockLocation: InventoryStockLocationSummary;
  status: "DRAFT" | "SUBMITTED" | "APPROVED" | "REJECTED" | "CANCELLED";
  version: number;
  openedBy: Uuid;
  submittedBy?: Uuid | null;
  submittedAt?: string | null;
  decidedBy?: Uuid | null;
  decidedAt?: string | null;
  decisionMemo?: string | null;
  lineCount: number;
  varianceLineCount: number;
  createdAt: string;
  updatedAt: string;
};

export type CycleCountDetail = {
  count: CycleCount;
  lines: Array<CycleCountLine>;
  appliedMovementIds: Array<Uuid>;
};

export type CycleCountLine = {
  id: Uuid;
  itemId: Uuid;
  ivCode: string;
  displayName: string;
  unitCode: string;
  systemQuantityMilli: number;
  countedQuantityMilli: number;
  varianceMilli: number;
  reason?: "DAMAGE" | "LOSS" | "MISCOUNT" | "FOUND" | "OTHER" | null;
  note?: string | null;
  recordedBy: Uuid;
  recordedAt: string;
};

export type CycleCountPage = {
  items: Array<CycleCount>;
  limit: number;
  offset: number;
  total: number;
};

export type CycleCountVersionRequest = {
  expectedVersion: number;
};

export type DailyPlanItemSummary = {
  work_order_id?: Uuid | null;
  request_no?: string | null;
  equipment_no?: string | null;
  management_no?: string | null;
  customer_name?: string | null;
  site_name?: string | null;
  description: string;
  sort_order: number;
};

export type DailyPlanListPage = {
  items: Array<DailyPlanSummary>;
};

export type DailyPlanStatus = "DRAFT" | "REQUESTED" | "APPROVED" | "REJECTED" | "FINAL_CONFIRMED";

export type DailyPlanSummary = {
  id?: Uuid;
  branch_id?: Uuid;
  mechanic_id?: Uuid;
  plan_date?: Date;
  status?: DailyPlanStatus;
  items?: Array<DailyPlanItemSummary>;
};

export type Date = string;

export type DebitCredit = "DEBIT" | "CREDIT";

export type DecideCycleCountRequest = {
  expectedVersion: number;
  decision: "APPROVE" | "REJECT";
  memo?: string | null;
  idempotencyKey?: string | null;
};

export type DecideOwnershipTransferRequest = {
  decision: "approve" | "reject";
  comment: string;
};

export type DecidePayrollRunRequest = {
  decision: "APPROVE" | "REJECT";
  reason?: string;
};

export type DecideWorkflowTaskRequest = {
  decision: "approve" | "reject" | "return";
  comment?: string;
  idempotency_key: string;
};

export type DecideWorkflowTaskResponse = {
  task: DecidedWorkflowTask;
  run: FinalizedWorkflowRun;
  next_task?: WorkflowTaskSummary;
};

export type DecidedWorkflowTask = {
  task_id: Uuid;
  run_id: Uuid;
  status: string;
  decision_payload: {
    [key: string]: unknown;
  };
};

export type DecisionLogRow = {
  id: string;
  decided_at: string;
  subject_ref: string;
  action: string;
  resource_type: string;
  resource_id?: string | null;
  effect: "allow" | "deny";
  determining_policies: Array<string>;
  reason: string;
};

export type DecisionResponse = {
  outcome: SimulationOutcome;
};

export type DefinitionsByObjectKindResponse = {
  kind: string;
  definitions: Array<WorkflowDefinitionResponse>;
  bindings: Array<TriggerBindingResponse>;
};

export type DepreciationMethod = "STRAIGHT_LINE" | "DECLINING_BALANCE";

export type DeviceLoginApproveRequest = {
  approve_token: string;
  ceremony_id: Uuid;
  credential: {
    [key: string]: unknown;
  };
};

export type DeviceLoginApproveSessionRequest = {
  approve_token: string;
};

export type DeviceLoginPollRequest = {
  poll_token: string;
};

export type DeviceLoginPollResponse = {
  status: "pending" | "approved" | "expired";
  access_token?: string | null;
  refresh_token?: string | null;
  token_type?: "Bearer" | null;
  refresh_expires_at?: string | null;
  requires_passkey_setup?: boolean | null;
};

export type DeviceLoginStartResponse = {
  poll_token: string;
  approve_url: string;
  expires_at: Timestamp;
};

export type DevicePlatform = "ios" | "android";

export type DeviceRegistrationRequest = {
  platform: DevicePlatform;
  push_token?: string | null;
  app_version: string;
};

export type DeviceRegistrationResponse = {
  id: Uuid;
  user_id: Uuid;
  device_hash: string;
  platform: DevicePlatform;
  push_token?: string | null;
  app_version: string;
  last_registered_at: Timestamp;
};

export type DirectoryPage = {
  items: Array<DirectoryPerson>;
  limit: number;
  offset: number;
  total: number;
};

export type DirectoryPerson = {
  id: Uuid;
  display_name: string;
  employee_id: string | null;
  employee_name: string | null;
  employee_number: string | null;
  employee_company: string | null;
  employee_org_unit: string | null;
  employee_position: string | null;
  employee_identity_review_required: boolean | null;
  employee_identity_resolution_confidence: string | null;
  employee_link_status: "LINKED" | "UNLINKED";
  team: Team;
  roles: Array<string>;
  branch_ids: Array<Uuid>;
  is_active: boolean;
  has_passkey: boolean;
  account_status: AccountStatus;
  created_at: Timestamp;
};

export type DispatchCandidatePage = {
  items: Array<DispatchCandidateSummary>;
};

export type DispatchCandidateSummary = {
  mechanic_id: Uuid;
  score_milli: number;
  gps_ranked: boolean;
  distance_meters?: number;
  location_recorded_at?: Timestamp;
  workload: {
    [key: string]: unknown;
  };
  score_reason: string;
  response?: DispatchResponseKind;
  responded_at?: Timestamp;
};

export type DispatchQueueDispatch = {
  id: Uuid;
  status: DispatchStatus;
  accept_window_ends_at: Timestamp;
  target_count: number;
  accepted_count: number;
  declined_count: number;
  manual_call_required: boolean;
};

export type DispatchQueueItem = {
  work_order_id: Uuid;
  request_no: string;
  branch_id: Uuid;
  status: WorkOrderStatus;
  priority: PriorityLevel;
  symptom: string;
  equipment_id: Uuid;
  customer_id: Uuid;
  site_id: Uuid;
  target_due_at?: Timestamp;
  assigned_mechanic_id?: Uuid;
  dispatch?: DispatchQueueDispatch;
  updated_at: Timestamp;
};

export type DispatchQueuePage = {
  items: Array<DispatchQueueItem>;
  next_after?: string;
  stats: DispatchQueueStats;
};

export type DispatchQueueStats = {
  unassigned_count: number;
  sla_due_count: number;
};

export type DispatchQueueStatus = "RECEIVED" | "UNASSIGNED" | "ASSIGNED" | "IN_PROGRESS" | "PART_WAITING" | "DELAYED";

export type DispatchResponseKind = "ACCEPT" | "DECLINE";

export type DispatchStatus = "BROADCASTING" | "AUTO_ASSIGNED" | "MANAGER_FORCE_PENDING";

/**
 * One immutable version of an in-console office document. The storage key is internal and never returned.
 */
export type DocumentVersion = {
  id: string;
  documentRef: string;
  versionNo: number;
  contentHash: string;
  fileType: "docx" | "xlsx" | "pptx";
  byteSize: number;
  restoredFrom?: number;
  createdBy?: string;
  createdAt: string;
};

export type DraftRecord = {
  id: Uuid;
  draft_key: string;
  title: string;
  normalized_row: {
    [key: string]: unknown;
  };
  generated_policy_text: string;
  validation_status: string;
  validation_errors: {
    [key: string]: unknown;
  };
  review_status: string;
  reviewer_id: Uuid | null;
  created_by: Uuid;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type Employee = {
  id: Uuid;
  company: string;
  name: string;
  employee_number?: string | null;
  org_unit?: string | null;
  worksite_name?: string | null;
  worksite?: string | null;
  job?: string | null;
  position?: string | null;
  hire_date?: string | null;
  exit_date?: string | null;
  status?: string | null;
  leave_accrued?: string | null;
  leave_used?: string | null;
  leave_remaining?: string | null;
  home_branch_id?: string | null;
  home_branch_name?: string | null;
  home_branch_review_required: boolean;
  identity_resolution_strategy: "employee_number" | "legal_identifier_hash" | "birth_hire_fingerprint" | "source_row_fingerprint";
  identity_resolution_confidence: "high" | "medium" | "low";
  identity_review_required: boolean;
  identity_name_only_merge: boolean;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type EmployeeAbsenceAlertResponse = {
  id: Uuid;
  employee_id: Uuid;
  employee_name: string;
  employee_number?: string;
  company: string;
  org_unit?: string;
  worksite_name?: string;
  branch_id?: Uuid;
  branch_name?: string;
  work_date: string;
  source: string;
  status: string;
  severity: string;
  audience_roles: Array<string>;
  signal_payload: {
    [key: string]: unknown;
  };
  notification_title: string;
  notification_message: string;
  link_href: string;
  exit_case_id?: Uuid;
  detected_at: Timestamp;
};

export type EmployeeAttendanceRecord = {
  id: Uuid;
  employee_id: Uuid;
  employee_display_name: string;
  kind: AttendanceRecordKind;
  occurred_at: Timestamp;
  work_date: string;
  state_after: "CLOCKED_IN" | "OUT_FOR_WORK" | "BUSINESS_TRIP" | "OFF_DUTY";
  note?: string | null;
  payroll_material_ref_id: Uuid;
  payroll_link_status: "LINKED";
  duplicate: boolean;
};

export type EmployeeAttendanceRecordPage = {
  items: Array<EmployeeAttendanceRecord>;
  total: number;
  limit: number;
  offset: number;
};

export type EmployeeDetail = {
  employee: Employee;
  employment: EmployeeEmploymentDetail;
};

export type EmployeeEmploymentDetail = {
  employment_type: "REGULAR" | "CONTRACT" | "PART_TIME" | "INTERN";
  phone_e164: string;
  base_pay: string;
  currency: "KRW";
};

export type EmployeeExitCaseResponse = {
  id: Uuid;
  employee_id: Uuid;
  employee_name: string;
  employee_number?: string;
  company: string;
  org_unit?: string;
  worksite_name?: string;
  branch_id?: Uuid;
  branch_name?: string;
  absence_alert_id?: Uuid;
  status: string;
  effective_exit_date: string;
  site_manager_note: string;
  reported_by: Uuid;
  reported_at: Timestamp;
  hr_confirmed_by?: Uuid;
  hr_confirmed_at?: Timestamp;
  hq_confirmed_by?: Uuid;
  hq_confirmed_at?: Timestamp;
  approval_submitted_by?: Uuid;
  approval_submitted_at?: Timestamp;
  settlement_package?: EmployeeExitSettlementPackageResponse;
  next_actions: Array<ExitCaseNextAction>;
};

export type EmployeeExitSettlementPackageResponse = {
  id: Uuid;
  status: string;
  service_days?: number;
  average_wage_period_start?: string;
  average_wage_period_end?: string;
  average_wage_calendar_days?: number;
  average_wage_total_won?: number;
  average_daily_wage_milliwon?: number;
  severance_pay_won?: number;
  monthly_ordinary_wage_won?: number;
  ordinary_daily_wage_won?: number;
  statutory_daily_wage_milliwon?: number;
  missing_source_fields: Array<string>;
  statutory_basis: {
    [key: string]: unknown;
  };
  insurance_loss_payload: {
    [key: string]: unknown;
  };
  approval_payload: {
    [key: string]: unknown;
  };
  certification_status: string;
  generated_at: Timestamp;
  submitted_by?: Uuid;
  submitted_at?: Timestamp;
};

export type EmployeeHomeBranch = {
  employee_id: Uuid;
  branch_id: Uuid;
  branch_name: string;
  updated_at: Timestamp;
};

export type EmployeeImportColumn = {
  source_header: string;
  normalized_header: string;
  target?: "name" | "employee_number" | "org_unit" | "job" | "position" | "worksite_name" | "worksite_address" | "hire_date" | "exit_date" | "leave_accrued" | "leave_used" | "leave_remaining" | "company" | null;
  classification: "canonical" | "retained" | "restricted" | "location";
  preview_allowed: boolean;
};

export type EmployeeImportCompanySummary = {
  company: string;
  input_rows: number;
  inserted: number;
  updated: number;
  skipped: number;
};

export type EmployeeImportDryRunSummary = {
  run_id: Uuid;
  input_rows: number;
  candidate_rows: number;
  preserved_rows: number;
  insert_candidates: number;
  update_candidates: number;
  companies: Array<EmployeeImportCompanySummary>;
};

export type EmployeeImportPreviewResponse = {
  run_id: Uuid;
  entity_type: "employee_hr";
  source_filename: string;
  source_sha256: string;
  input_rows: number;
  candidate_rows: number;
  preserved_rows: number;
  columns: Array<EmployeeImportColumn>;
  sample_rows: Array<EmployeeImportPreviewRow>;
  mapping_profile: {
    [key: string]: unknown;
  };
};

export type EmployeeImportPreviewRow = {
  source_sheet: string;
  source_row: number;
  row_status: "CANDIDATE" | "PRESERVED" | "ERROR";
  values: {
    [key: string]: unknown;
  };
};

export type EmployeeImportReport = {
  input_rows: number;
  inserted: number;
  updated: number;
  skipped: number;
  companies: Array<EmployeeImportCompanySummary>;
};

export type EmployeeLifecycleEvent = {
  id: Uuid;
  employee_id: Uuid;
  event_type: "ONBOARD" | "OFFBOARD" | "TERMINATE" | "TRANSFER";
  from_status?: string | null;
  to_status: "ACTIVE" | "EXITED" | "UNKNOWN";
  from_company?: string | null;
  to_company?: string | null;
  from_org_unit?: string | null;
  to_org_unit?: string | null;
  from_position?: string | null;
  to_position?: string | null;
  effective_date: string;
  comment: string;
  signoffs: EmployeeLifecycleSignoffs;
  created_by: Uuid;
  created_at: Timestamp;
};

export type EmployeeLifecycleEventPage = {
  items: Array<EmployeeLifecycleEvent>;
};

export type EmployeeLifecycleSignoffs = {
  privacy_notice_ack: boolean;
  korean_labor_law_ack: boolean;
  payroll_cutoff_ack: boolean;
  retirement_settlement_ack: boolean;
};

export type EmployeePage = {
  items: Array<Employee>;
  total: number;
  limit: number;
  offset: number;
};

/**
 * Cross-device passkey-enrollment handoff request. Carries NO user/org — those come from the verified access token, so a caller can only mint a handoff for itself. `step_up` is REQUIRED only when the caller already has a passkey (adding a device); a mid-onboarding caller (zero passkeys) omits it.
 */
export type EnrollHandoffRequest = {
  step_up?: PasskeyStepUpAssertion;
};

/**
 * The minted single-use, short-lived passkey-enrollment code (returned once, only its hash is stored) plus the ready-to-encode enrollment URL the frontend renders as a QR. The phone opens `enroll_url`, redeems `otp` via the first-sign-in path, enrolls a platform passkey, then approves the paired desktop poll token when `desktop_approve` is present in the URL fragment.
 */
export type EnrollHandoffResponse = {
  otp: string;
  expires_at: Timestamp;
  enroll_url: string;
  poll_token: string;
};

export type Equipment3rCaseDetailView = Equipment3rCaseView & {
  approval?: {
    decision?: "APPROVED" | "DECLINED";
    reason?: string | null;
    decidedBy?: Uuid;
    decidedAt?: string;
  } | null;
  dispatch?: {
    carrierName?: string;
    vehicleReference?: string;
    dispatchedAt?: string;
  } | null;
  handover?: {
    recipientName?: string;
    evidenceObjectId?: Uuid;
    handedOverAt?: string;
  } | null;
  returnedAt?: string | null;
  assessment?: {
    conditionGrade?: "A" | "B" | "C" | "D";
    findings?: string;
    disposition?: "REPAIR" | "REFURBISH" | "RESALE" | "REDEPLOY";
    assessedBy?: Uuid;
    assessedAt?: string;
  } | null;
  dispositionId?: Uuid | null;
  inspections: Array<Equipment3rInspectionView>;
  createdBy: Uuid;
  createdAt: string;
  updatedAt: string;
};

export type Equipment3rCaseView = {
  id: Uuid;
  unitId: Uuid;
  status: "QUOTED" | "APPROVED" | "DECLINED" | "DISPATCHED" | "HANDED_OVER" | "RETURNED" | "CLOSED";
  customerName: string;
  siteReference: string;
  monthlyRateMinor: number;
  durationMonths: number;
  currencyCode: "KRW";
  branchId: Uuid;
  replayed?: boolean;
};

export type Equipment3rDispositionView = {
  id: Uuid;
  unitId: Uuid;
  caseId: Uuid;
  kind: "REPAIR" | "REFURBISH" | "RESALE" | "REDEPLOY";
  status: "OPEN" | "COMPLETED";
  costMinor?: number | null;
  saleAmountMinor?: number | null;
  buyerName?: string | null;
  completedBy: Uuid;
  completedAt: string;
  financeGlPosting: null;
};

export type Equipment3rHistoryEntry = {
  aggregateKind: "unit" | "case" | "disposition";
  aggregateId: Uuid;
  transition: string;
  actorId: Uuid;
  occurredAt: string;
};

export type Equipment3rInspectionView = {
  id: Uuid;
  caseId: Uuid;
  outcome: "PASS" | "MAINTENANCE_PERFORMED";
  findings: string;
  maintenanceNote?: string | null;
  inspectedBy: Uuid;
  inspectedAt: string;
};

export type Equipment3rUnitDetailView = Equipment3rUnitView & {
  activeCaseId?: Uuid | null;
  openDispositionId?: Uuid | null;
  createdAt: string;
  updatedAt: string;
};

export type Equipment3rUnitView = {
  id: Uuid;
  serialNo: string;
  modelName: string;
  capacityClass: string;
  availability: "AVAILABLE" | "RESERVED" | "ON_RENT" | "IN_ASSESSMENT" | "IN_REPAIR" | "IN_REFURBISHMENT" | "FOR_SALE" | "SOLD";
  acquisitionCostMinor: number;
  branchId: Uuid;
};

export type EquipmentAutocompletePage = {
  items: Array<EquipmentLookupResponse>;
  limit: number;
};

export type EquipmentByLocationPage = {
  items: Array<SiteLocationGroup>;
  total: number;
};

export type EquipmentGraphEdge = {
  from: string;
  to: string;
  kind: string;
  label: string;
};

export type EquipmentGraphNode = {
  id: string;
  node_type: string;
  label: string;
  subtitle?: string | null;
  href?: string | null;
  current: boolean;
};

export type EquipmentLifecycleEvent = {
  id: string;
  kind: string;
  label: string;
  description?: string | null;
  event_date?: Date | null;
  occurred_at?: Timestamp | null;
  href?: string | null;
};

export type EquipmentListItem = {
  equipment_id: Uuid;
  branch_id: Uuid;
  equipment_no: string;
  management_no?: string | null;
  status: EquipmentStatus;
  model?: string | null;
  maker?: string | null;
  specification: string;
  ton_text: string;
  customer_name: string;
  site_name: string;
  asset_owner?: string | null;
  vin?: string | null;
  updated_at: Timestamp;
};

export type EquipmentListPage = {
  items: Array<EquipmentListItem>;
  total: number;
  limit: number;
  offset: number;
};

export type EquipmentLookupResponse = {
  id: Uuid;
  branch_id: Uuid;
  equipment_no: string;
  management_no: string | null;
  model: string | null;
  status: string;
  specification: string;
  ton_text: string;
  maker: string | null;
  vin: string | null;
  vehicle_registration_no: string | null;
  customer: NamedEntity;
  site: NamedEntity;
};

export type EquipmentRelationshipGraph = {
  nodes: Array<EquipmentGraphNode>;
  edges: Array<EquipmentGraphEdge>;
};

export type EquipmentRollbackResult = {
  version: number;
};

export type EquipmentSortBy = "equipment_no" | "model" | "customer" | "updated_at";

export type EquipmentStatus = "rented" | "spare" | "disposed" | "replacement" | "sold";

export type EquipmentSummary = {
  id: Uuid;
  equipment_no: string;
  management_no: string | null;
  model: string | null;
  status: string;
  specification: string;
  ton_text: string;
};

export type EquipmentTimelineEquipment = {
  equipment_id: Uuid;
  branch_id: Uuid;
  equipment_no: string;
  management_no?: string | null;
  status: EquipmentStatus;
  model?: string | null;
  maker?: string | null;
  customer_id: Uuid;
  customer_name: string;
  site_id: Uuid;
  site_name: string;
};

export type EquipmentTimelineGraph = {
  equipment: EquipmentTimelineEquipment;
  lifecycle_events: Array<EquipmentLifecycleEvent>;
  graph: EquipmentRelationshipGraph;
  work_order_count: number;
  cost_ledger_total_won: number;
};

export type EquipmentVersion = {
  version: number;
  status: "CAPTURED" | "ROLLBACK";
  sourceVersion?: number;
  content: {
    [key: string]: unknown;
  };
  createdBy?: string;
  createdAt: string;
};

export type EquipmentVersionList = {
  items: Array<EquipmentVersion>;
};

export type ErrorBody = {
  error: {
    code: string;
    message: string;
    reasons?: Array<string>;
    current_key_write_revision?: number;
  };
};

export type EvaluationCycleDetail = {
  id: Uuid;
  name: string;
  kind: EvaluationCycleKind;
  period_label: string;
  due_date: Date;
  stage: EvaluationCycleStage;
  subjects_total: number;
  manager_submitted: number;
  self_submitted: number;
  calibrated: number;
  finalized: number;
  created_at: Timestamp;
  opened_at: Timestamp | null;
  calibration_started_at: Timestamp | null;
  finalized_at: Timestamp | null;
  archived_at: Timestamp | null;
  created_by: Uuid;
  progress_by_unit: Array<EvaluationUnitProgress>;
  subjects: Array<EvaluationSubjectSummary>;
};

export type EvaluationCycleKind = "REGULAR" | "PROBATION";

export type EvaluationCyclePage = {
  items: Array<EvaluationCycleSummary>;
  total: number;
};

export type EvaluationCycleStage = "DRAFT" | "OPEN" | "CALIBRATION" | "FINALIZED" | "ARCHIVED";

export type EvaluationCycleSummary = {
  id: Uuid;
  name: string;
  kind: EvaluationCycleKind;
  period_label: string;
  due_date: Date;
  stage: EvaluationCycleStage;
  subjects_total: number;
  manager_submitted: number;
  self_submitted: number;
  calibrated: number;
  finalized: number;
  created_at: Timestamp;
};

export type EvaluationCycleTransition = "open" | "start_calibration" | "finalize" | "archive";

export type EvaluationEvidenceKind = "ATTENDANCE" | "WORK_ORDER" | "APPROVAL" | "KPI" | "OTHER";

export type EvaluationEvidenceLink = {
  id: Uuid;
  object_kind: EvaluationEvidenceKind;
  object_ref: string;
  label: string;
  sort_order: number;
};

export type EvaluationEvidenceLinkInput = {
  object_kind: EvaluationEvidenceKind;
  object_ref: string;
  label: string;
};

export type EvaluationGoal = {
  id: Uuid;
  title: string;
  metric_kind: EvaluationMetricKind;
  target_label: string;
  weight_pct: number;
  sort_order: number;
};

export type EvaluationGoalInput = {
  title: string;
  metric_kind: EvaluationMetricKind;
  target_label: string;
  weight_pct: number;
};

export type EvaluationGrade = "S" | "A" | "B" | "C" | "D";

export type EvaluationLedgerEntry = {
  rv_code: string;
  cycle_id: Uuid;
  cycle_name: string;
  period_label: string;
  final_grade: EvaluationGrade;
  finalized_at: Timestamp;
  subject_id: Uuid;
};

export type EvaluationLedgerPage = {
  items: Array<EvaluationLedgerEntry>;
};

export type EvaluationMetricKind = "KPI" | "ATTENDANCE" | "TASK" | "CUSTOM";

export type EvaluationPreflightItem = {
  code: string;
  message: string;
  subject_id: Uuid | null;
};

export type EvaluationPreflightReport = {
  next_transition: EvaluationCycleTransition | null;
  blockers: Array<EvaluationPreflightItem>;
  advisories: Array<EvaluationPreflightItem>;
};

export type EvaluationReview = {
  id: Uuid;
  subject_id: Uuid;
  kind: EvaluationReviewKind;
  status: EvaluationReviewStatus;
  evaluator_user_id: Uuid;
  grade: EvaluationGrade | null;
  note: string | null;
  evidence_links: Array<EvaluationEvidenceLink>;
  submitted_at: Timestamp | null;
  updated_at: Timestamp;
};

export type EvaluationReviewKind = "SELF" | "MANAGER";

export type EvaluationReviewStatus = "DRAFT" | "SUBMITTED";

export type EvaluationSubjectDetail = {
  id: Uuid;
  cycle_id: Uuid;
  employee_id: Uuid;
  employee_name: string;
  org_unit: string | null;
  manager_user_id: Uuid;
  state: EvaluationSubjectState;
  final_grade: EvaluationGrade | null;
  rv_code: string | null;
  goals: Array<EvaluationGoal>;
  reviews: Array<EvaluationReview>;
  calibrated_grade: EvaluationGrade | null;
  calibration_reason: string | null;
  calibrated_by: Uuid | null;
  calibrated_at: Timestamp | null;
  finalized_at: Timestamp | null;
};

export type EvaluationSubjectState = "ENROLLED" | "IN_REVIEW" | "REVIEWED" | "CALIBRATED" | "FINALIZED";

export type EvaluationSubjectSummary = {
  id: Uuid;
  cycle_id: Uuid;
  employee_id: Uuid;
  employee_name: string;
  org_unit: string | null;
  manager_user_id: Uuid;
  state: EvaluationSubjectState;
  final_grade: EvaluationGrade | null;
  rv_code: string | null;
};

export type EvaluationTaskItem = {
  subject_id: Uuid;
  cycle_id: Uuid;
  cycle_name: string;
  due_date: Date;
  employee_id: Uuid;
  employee_name: string;
  kind: EvaluationReviewKind;
  review_status: EvaluationReviewStatus | null;
};

export type EvaluationTaskPage = {
  items: Array<EvaluationTaskItem>;
};

export type EvaluationUnitProgress = {
  org_unit: string | null;
  total: number;
  manager_submitted: number;
};

export type EvidenceBinding = {
  id: Uuid;
  control_id: Uuid;
  obligation_id?: string | null;
  evidence_target_type: EvidenceTargetType;
  evidence_target_id: string;
  source_audit_event_id?: string | null;
  status: EvidenceBindingStatus;
  confidence: EvidenceConfidence;
  collected_at?: string | null;
  collected_by?: string | null;
  valid_from?: string | null;
  valid_to?: string | null;
  hash_sha256?: string | null;
  metadata: {
    [key: string]: unknown;
  };
  created_by: Uuid;
  updated_by: Uuid;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type EvidenceBindingPage = {
  items: Array<EvidenceBinding>;
  limit: number;
  offset: number;
  total: number;
};

export type EvidenceBindingStatus = "PROPOSED" | "ACCEPTED" | "REJECTED" | "EXPIRED" | "RETRACTED";

export type EvidenceClassification = "General" | "Internal" | "Sensitive" | "Confidential" | "Secret";

export type EvidenceConfidence = "LOW" | "MEDIUM" | "HIGH" | "SYSTEM";

export type EvidenceConfirmResponse = {
  id: Uuid;
  work_order_id: Uuid;
  stage: AttachmentStage;
  worm_replica_status: WormReplicaStatus;
  retry_count: number;
  verified_at?: Timestamp;
};

/**
 * Server-derived classification; verified replicas of derivative copies never become evidence-equivalent originals.
 */
export type EvidenceCopyEvidentiaryStatus = "VERIFIED_ORIGINAL" | "ORIGINAL_UNVERIFIED" | "NON_EVIDENTIARY_DERIVATIVE";

export type EvidenceCopyKind = "ORIGINAL" | "DERIVATIVE";

export type EvidenceCopyView = {
  id: string;
  evidence_object_id: string;
  copy_kind: EvidenceCopyKind;
  evidentiary_status: EvidenceCopyEvidentiaryStatus;
  derivative_kind?: "REDACTED" | "THUMBNAIL" | "TRANSCODED" | "EXCERPT" | "EXPORT_MANIFEST" | "NORMALIZED_TEXT" | "OTHER" | null;
  parent_copy_id?: string | null;
  storage: EvidenceStorageRef;
  source_evidence_media_id?: string | null;
  digest_sha256: string;
  content_type: string;
  size_bytes: number;
  worm_status: WormStorageStatus;
  verified_at?: string | null;
  created_by: string;
  created_at: string;
};

export type EvidenceExportView = {
  id: string;
  evidence_object_id: string;
  manifest_digest_sha256: string;
  signature_algorithm: string;
  signature_ref?: string | null;
  export_reason: string;
  exported_by: string;
  exported_at: string;
};

export type EvidenceHoldRequest = {
  op: "apply";
  case_ref: string;
  basis: string;
  reason: string;
} | {
  op: "release";
  hold_id: string;
  reason: string;
  four_eyes_request_ref: string;
};

export type EvidenceObjectDetail = {
  object: EvidenceObjectView;
  copies: Array<EvidenceCopyView>;
  tsa_proofs: Array<TimestampAuthorityProofView>;
  custody_history: Array<CustodyEventView>;
  legal_holds: Array<LegalHoldRecordView>;
  exports: Array<EvidenceExportView>;
};

export type EvidenceObjectPage = {
  items: Array<EvidenceObjectView>;
  limit: number;
  offset: number;
  total: number;
  as_of: number;
  next_cursor: string | null;
};

export type EvidenceObjectView = {
  id: string;
  code: string;
  title: string;
  description?: string | null;
  source: EvidenceSourceRef;
  classification: EvidenceClassification;
  record_owner_user_id?: string | null;
  current_custody_stage: CustodyStage;
  legal_hold_state: LegalHoldState;
  admissibility_status: AdmissibilityStatus;
  admissibility_reasons: Array<string>;
  admissibility_inputs: {
    [key: string]: unknown;
  };
  created_by: string;
  updated_by: string;
  created_at: string;
  updated_at: string;
  disposed_at?: string | null;
};

export type EvidencePresignRequest = {
  work_order_id: Uuid;
  stage: AttachmentStage;
  content_type: string;
  size_bytes: number;
  checksum_sha256?: string;
};

export type EvidencePresignResponse = {
  id: Uuid;
  work_order_id: Uuid;
  stage: AttachmentStage;
  upload: PresignedUpload;
};

export type EvidenceSourceRef = {
  source_type: EvidenceSourceType;
  source_id: string;
  source_code?: string | null;
};

export type EvidenceSourceType = "record_archive" | "inbox_doc" | "mail_attachment" | "ingest_job" | "work_order_evidence_media" | "external_document";

export type EvidenceStagingPresignRequest = {
  work_order_id: Uuid;
  stage: AttachmentStage;
  content_type: string;
  size_bytes: number;
  checksum_sha256?: string;
};

export type EvidenceStagingPresignResponse = {
  id: Uuid;
  work_order_id: Uuid;
  stage: AttachmentStage;
  media_kind: MediaKind;
  processing_status: ProcessingStatus;
  upload: PresignedUpload;
};

export type EvidenceStatusResponse = {
  id: Uuid;
  work_order_id: Uuid;
  stage: AttachmentStage;
  processing_status: ProcessingStatus;
  content_type: string;
  thumbnail_url?: string;
  processing_error?: string;
  processed_at?: Timestamp;
};

export type EvidenceStorageRef = {
  provider: string;
  object_id: string;
  key_ref?: string | null;
  version_id?: string | null;
};

export type EvidenceSummary = {
  id: Uuid;
  stage: AttachmentStage;
  content_type: string;
  size_bytes: number;
  uploaded_by: Uuid;
  worm_replica_status: WormReplicaStatus;
  retry_count: number;
  verified_at: string | null;
  created_at: Timestamp;
};

export type EvidenceTargetType = "audit_event" | "evidence_media" | "workflow_run" | "workflow_task" | "object_link" | "governance_finding" | "external_document" | "future_ev_object";

export type EvidenceVerifyReport = {
  evidence_object_id: string;
  verified_at: string;
  outcome: VerifyOutcome;
  copies: Array<CopyVerification>;
};

export type ExecuteObjectActionRequest = {
  action_id: "equipment.update_profile";
  object_type: "equipment";
  object_id: Uuid;
  input: UpdateEquipmentRequest;
  idempotency_key?: string | null;
  step_up?: PasskeyStepUpAssertion;
};

export type ExitCaseNextAction = {
  key: string;
  label: string;
  href: string;
};

export type ExportSourceNote = {
  source_domain: string;
  reason: string;
};

export type ExtendRecruitOfferRequest = {
  amount: string;
  amount_period: RecruitAmountPeriod;
  reply_deadline: Date;
};

export type FacilitiesAcceptanceRequest = {
  decision: "ACCEPTED" | "REJECTED";
  reason?: string;
};

export type FacilitiesAssignRequest = {
  assigneeId: Uuid;
};

export type FacilitiesCase = {
  id: Uuid;
  branchId: Uuid;
  status: "DUE" | "TRIAGED" | "SCHEDULED" | "ASSIGNED" | "IN_PROGRESS" | "SUBMITTED" | "REWORK_REQUIRED" | "AWAITING_ACCEPTANCE" | "CLOSED";
  assigneeId?: string | null;
  responseDueAt: string;
  completionDueAt: string;
  acceptanceDueAt: string;
  energyDeltaKwh?: string | null;
  totalCostKrw: number;
};

export type FacilitiesDueCaseRequest = {
  obligationId: Uuid;
  idempotencyKey: string;
};

export type FacilitiesObservationRequest = {
  preKwh?: string;
  postKwh?: string;
  observedAt: string;
  costKrw?: number;
};

export type FacilitiesSubmitRequest = {
  safetyChecklistEvidenceId: Uuid;
  serviceReportEvidenceId: Uuid;
  photoEvidenceId?: Uuid;
};

export type FacilitiesTriageRequest = {
  scheduledFor: string;
};

export type FieldAttendanceEvent = {
  user_id: Uuid;
  user_name: string | null;
  work_order_id: Uuid;
  kind: string;
  occurred_at: Timestamp;
};

export type FieldSiteDetail = {
  site: FieldSiteSummary;
  sla: FieldSlaSummary;
  tickets: Array<SupportTicketSummary>;
  work_orders: Array<FieldWorkOrderRef>;
  attendance: Array<FieldAttendanceEvent>;
  acceptances: Array<SupportTicketAcceptance>;
};

export type FieldSitePage = {
  items: Array<FieldSiteRow>;
  next_cursor: string | null;
  total: number;
};

export type FieldSiteRow = {
  site_id: Uuid;
  site_name: string;
  branch_id: Uuid;
  customer_id: Uuid;
  customer_name: string;
  address: string | null;
  latitude: number | null;
  longitude: number | null;
  open_ticket_count: number;
  breached_ticket_count: number;
  next_due_at: string | null;
  active_work_order_count: number;
  last_arrival_at: string | null;
  sla: FieldSlaState;
};

export type FieldSiteSummary = {
  id: Uuid;
  name: string;
  branch_id: Uuid;
  customer_id: Uuid;
  customer_name: string;
  address: string | null;
  province: string | null;
  city: string | null;
  postal_code: string | null;
  latitude: number | null;
  longitude: number | null;
  geofence_radius_m: number | null;
  contact_name: string | null;
  contact_phone: string | null;
};

/**
 * Deterministic per-site SLA state over OPEN/IN_PROGRESS/ON_HOLD tickets — BREACHED when any due_at is past, else AT_RISK when any due_at falls within 24 hours, else OK.
 */
export type FieldSlaState = "OK" | "AT_RISK" | "BREACHED";

export type FieldSlaSummary = {
  state: FieldSlaState;
  open: number;
  breached: number;
  next_due_at: string | null;
  resolved_within_sla_90d: number;
  resolved_breached_90d: number;
};

export type FieldWorkOrderRef = {
  id: Uuid;
  request_no: string;
  status: string;
  priority: string;
  target_due_at: string | null;
  report_submitted_at: string | null;
  result_type: string;
  created_at: Timestamp;
};

export type FinalizeWorkflowTaskRequest = {
  mode: "author" | "delegate";
  reason?: string;
  idempotency_key: string;
};

export type FinalizeWorkflowTaskResponse = {
  task: FinalizedWorkflowTask;
  run: FinalizedWorkflowRun;
  archive_ref?: {
    [key: string]: unknown;
  };
};

export type FinalizedWorkflowRun = {
  id: Uuid;
  status: string;
};

export type FinalizedWorkflowTask = {
  id: Uuid;
  run_id: Uuid;
  status: string;
  completed_by?: Uuid;
  decision_payload: {
    [key: string]: unknown;
  };
};

export type FinancialConfigSnapshot = {
  depreciation_method: DepreciationMethod;
  useful_life_months: number;
  residual_rate_bps: number;
  declining_balance_rate_bps: number;
  management_fee_rate_bps: number;
  profit_rate_bps: number;
  floor_negative_quote_residual: boolean;
  executive_approval_threshold_won: number;
};

/**
 * Fresh passkey step-up evidence required before sensitive financial state transitions.
 */
export type FinancialStepUpRequest = {
  step_up: PasskeyStepUpAssertion;
};

export type FindingSeverity = "INFO" | "LOW" | "MEDIUM" | "HIGH" | "CRITICAL";

/**
 * Lifecycle of a governance finding. OPEN findings are triaged to one of REVIEWED, DISMISSED, or ESCALATED.
 */
export type FindingStatus = "OPEN" | "REVIEWED" | "DISMISSED" | "ESCALATED";

export type FixityStatus = "MATCH" | "MISMATCH" | "CHECKSUM_UNAVAILABLE" | "STORAGE_ERROR";

export type ForceAssignP1DispatchRequest = {
  mechanic_id: Uuid;
};

export type FrameworkKind = "LEGAL_BASELINE" | "INTERNAL_CONTROL" | "CUSTOMER_CONTROL" | "SECURITY_STANDARD" | "SAFETY_STANDARD" | "AUDIT_PROGRAM";

export type FrameworkStatus = "DRAFT" | "ACTIVE" | "RETIRED" | "ARCHIVED";

export type GateChainConfig = {
  authority: boolean;
  self_checklist: boolean;
  four_eyes: boolean;
  egress_dlp: boolean;
};

export type GateChainOutcome = {
  allow: boolean;
  gates: Array<GateOutcome>;
};

export type GateKind = "authority" | "self_checklist" | "four_eyes" | "egress_dlp";

export type GateOutcome = {
  gate: GateKind;
  status: GateStatus;
};

export type GateStatus = GateStatusNotRequired | GateStatusSatisfied | GateStatusPending | GateStatusDenied;

export type GateStatusDenied = {
  status: "denied";
  reason: string;
};

export type GateStatusNotRequired = {
  status: "not_required";
};

export type GateStatusPending = {
  status: "pending";
  reason: string;
};

export type GateStatusSatisfied = {
  status: "satisfied";
};

export type GovernanceConfigureTransitionRequest = {
  object_type_id: Uuid;
  from_state: LifecycleState;
  to_state: LifecycleState;
  requires_reason?: boolean;
  requires_four_eyes?: boolean;
  requires_checklist?: boolean;
};

export type GovernanceDecideApprovalRequest = {
  request_ref: Uuid;
  kind: string;
  requested_by: Uuid;
  decision: "approved" | "rejected";
};

/**
 * A governance finding raised by the integrity engine (#34). Findings are "검토 필요" (review needed) — e.g. a self-approval record (detector_id `anomaly.self_approval`) or a price anomaly (`anomaly.price_outlier`) — and are NOT accusations of wrongdoing.
 */
export type GovernanceFinding = {
  id: Uuid;
  org_id: Uuid;
  detector_id: string;
  entity_type: string;
  entity_id: string;
  source_audit_event_id?: string | null;
  subject_user_id?: string | null;
  score: number;
  severity: FindingSeverity;
  evidence: {
    [key: string]: unknown;
  };
  status: FindingStatus;
  detected_at: Timestamp;
  created_at: Timestamp;
  updated_at: Timestamp;
  reviewed_by?: string | null;
  reviewed_at?: Timestamp | null;
  review_memo?: string | null;
};

export type GovernanceLifecyclePreflightRequest = {
  object_type_id: Uuid;
  from_state: LifecycleState;
  to_state: LifecycleState;
  authority_allow?: boolean;
  checklist_all_acknowledged?: boolean;
  four_eyes_request_ref?: Uuid;
  egress_cleared?: boolean;
};

export type GovernanceOpenOverrideRequest = {
  target_type: string;
  target_id: Uuid;
  reason: string;
  before_snapshot: {
    [key: string]: unknown;
  };
};

export type GroupAdminGroupResponse = {
  id: Uuid;
  slug: string;
  name: string;
  status: string;
  members: Array<GroupAdminMemberOrgResponse>;
};

export type GroupAdminGroupsResponse = {
  groups: Array<GroupAdminGroupResponse>;
};

export type GroupAdminMemberOrgResponse = {
  id: Uuid;
  slug: string;
  name: string;
  status: string;
};

export type GroupAdminTenantContextStartResponse = {
  access_token: string;
  token_type: "Bearer";
  acting_org_id: Uuid;
  acting_org_name: string;
  acting_role: "GROUP_ADMIN_DELEGATED_ADMIN";
  expires_at: Timestamp;
};

export type HireRecruitApplicantRequest = {
  employee_number: string;
  phone: string;
  org_unit: string;
  position: string;
  site: string;
  home_branch_id: Uuid;
  base_pay?: string | null;
};

export type HireRecruitApplicantResponse = {
  employee_id: Uuid;
  applicant: RecruitApplicant;
  posting: RecruitPosting;
};

export type HoldRecruitApplicantRequest = {
  hold: boolean;
};

export type HrAnnualLeaveReadinessSummary = {
  obligations: number;
  usage_promotion_required: number;
  payout_review_required: number;
  needs_review: number;
  remaining_days: string;
};

export type HrAttendanceReadinessSummary = {
  durable_events: number;
  self_service_records: number;
  payroll_material_refs: number;
};

export type HrImportReadinessSummary = {
  runs: number;
  applied_runs: number;
  input_rows: number;
  candidate_rows: number;
  preserved_rows: number;
  ledger_rows: number;
  latest_import_at?: Timestamp;
};

export type HrOrgChartCompany = {
  company: string;
  total: number;
  active: number;
  units: Array<HrOrgChartUnit>;
};

export type HrOrgChartEmployee = {
  id: Uuid;
  name: string;
  employee_number?: string | null;
  status: string;
};

export type HrOrgChartPosition = {
  title: string;
  total: number;
  employees: Array<HrOrgChartEmployee>;
};

export type HrOrgChartResponse = {
  companies: Array<HrOrgChartCompany>;
};

export type HrOrgChartUnit = {
  name: string;
  total: number;
  positions: Array<HrOrgChartPosition>;
};

export type HrPayrollReadinessSummary = {
  draft_runs: number;
  blocked_runs: number;
  calculation_enabled_runs: number;
  active_close_runs: number;
  draft_lines: number;
  payroll_source_rows: number;
  attendance_source_rows: number;
  attendance_event_links: number;
  attendance_material_refs: number;
  gross_pay_source_lines: number;
  net_pay_source_lines: number;
  latest_status?: string;
  latest_source_label?: string;
  latest_period_start?: string;
  latest_period_end?: string;
  latest_updated_at?: Timestamp;
};

export type HrReadinessSummary = {
  imports: HrImportReadinessSummary;
  payroll: HrPayrollReadinessSummary;
  annual_leave: HrAnnualLeaveReadinessSummary;
  attendance: HrAttendanceReadinessSummary;
};

/**
 * §16 self-checklist evidence for an ingest-commit ("적재") apply. Missing or false ⇒ fail-closed deny, nothing written (85 판정).
 */
export type ImportApplyRequest = {
  checklist_all_acknowledged?: boolean | null;
};

/**
 * The fresh passkey assertion proving present possession of an authenticator. Its absence yields 428 (precondition required).
 */
export type InboxDocConfirmReceiptRequest = {
  step_up: PasskeyStepUpAssertion;
};

/**
 * A single inbox document. Extends InboxDocSummary with the body `payload`, present only when readable (a payslip, or an already-confirmed legal notice) and omitted while a legal notice is locked.
 */
export type InboxDocDetail = InboxDocSummary & {
  payload?: {
    [key: string]: unknown;
  };
};

export type InboxDocPage = {
  items: Array<InboxDocSummary>;
  next_cursor: string | null;
};

/**
 * A list-row / confirmation view of one inbox document. Never carries the body `payload` — that is only ever on InboxDocDetail, and only once readable.
 */
export type InboxDocSummary = {
  id: Uuid;
  recipient_user_id: Uuid;
  kind: "payslip" | "legal_notice";
  notice_type?: string;
  title: string;
  legal_basis?: string;
  source_kind?: string;
  source_id?: string;
  locked: boolean;
  confirmed_by?: string | null;
  confirmed_at?: string | null;
  created_at: Timestamp;
};

export type IncidentLocation = {
  latitude: number;
  longitude: number;
};

/**
 * Minimal acknowledgement of a submitted inquiry (no identifiers, no PII echo).
 */
export type InquiryAck = {
  status: string;
};

/**
 * Triage state of an inbound inquiry in the internal inbox.
 */
export type InquiryStatus = "NEW" | "CONTACTED" | "CLOSED";

/**
 * Subject of a customer inquiry.
 */
export type InquiryTopic = "RENTAL" | "USED_SALES" | "MAINTENANCE" | "OTHER";

export type InspectionCycle = "DAILY" | "WEEKLY" | "MONTHLY" | "QUARTERLY" | "YEARLY" | "CUSTOM";

export type InspectionRoundOutcome = "COMPLETED" | "FOLLOW_UP_REQUIRED";

export type InspectionRoundSummary = {
  id: Uuid;
  schedule_id: Uuid;
  branch_id: Uuid;
  equipment_id: Uuid;
  mechanic_id: Uuid;
  completed_by: Uuid;
  outcome: InspectionRoundOutcome;
  findings: string;
  note: string | null;
  completed_at: Timestamp;
};

export type InspectionSchedulePage = {
  items: Array<InspectionScheduleSummary>;
  limit: number;
  offset: number;
  total: number;
};

export type InspectionScheduleStatus = "SCHEDULED" | "COMPLETED" | "CANCELLED";

export type InspectionScheduleSummary = {
  id: Uuid;
  branch_id: Uuid;
  equipment_id: Uuid;
  mechanic_id: Uuid;
  mechanic_display_name: string | null;
  cycle: InspectionCycle;
  interval_days: number;
  due_date: Date;
  status: InspectionScheduleStatus;
  completed_at: string | null;
  note: string | null;
  site_name: string;
  management_no: string | null;
  model: string | null;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type InstanceHead = {
  id: string;
  object_type_id: string;
  title: string;
  current_revision_id?: string | null;
  lifecycle_state: InstanceLifecycleState;
};

export type InstanceLifecycleState = "draft" | "active" | "locked" | "archived" | "disposed";

export type InstanceState = {
  instance: InstanceHead;
  revision: RevisionSummary;
};

export type InventoryConsumptionEvent = {
  id: Uuid;
  item_id: Uuid;
  iv_code: string;
  branch_id: Uuid;
  stock_location_id: Uuid;
  source: InventoryConsumptionSource;
  quantity_before_milli: number;
  quantity_consumed_milli: number;
  quantity_after_milli: number;
  unit_cost_won?: number | null;
  cost_won?: number | null;
  consumed_by: Uuid;
  occurred_at: string;
  memo?: string | null;
  created_at: string;
};

export type InventoryConsumptionResult = {
  event: InventoryConsumptionEvent;
  item: InventoryItem;
};

export type InventoryConsumptionSource = {
  kind: "work_order";
  work_order_id: Uuid;
} | {
  kind: "p1_dispatch";
  dispatch_id: Uuid;
};

export type InventoryItem = {
  id: Uuid;
  branch_id: Uuid;
  site_id?: Uuid | null;
  stock_location: InventoryStockLocationSummary;
  iv_code: string;
  sku?: string | null;
  display_name: string;
  description?: string | null;
  unit_code: string;
  quantity_on_hand_milli: number;
  safety_stock_milli: number;
  unit_cost_won?: number | null;
  low_stock: boolean;
  status: string;
  href: string;
  created_by: Uuid;
  created_at: string;
  updated_at: string;
};

export type InventoryItemPage = {
  items: Array<InventoryItem>;
  limit: number;
  offset: number;
  total: number;
};

export type InventoryMovement = {
  id: Uuid;
  itemId: Uuid;
  ivCode: string;
  kind: "ISSUE" | "RECEIPT" | "ADJUSTMENT";
  quantityDeltaMilli: number;
  quantityBeforeMilli: number;
  quantityAfterMilli: number;
  source: InventoryMovementSource;
  actor: Uuid;
  occurredAt: string;
  memo?: string | null;
};

export type InventoryMovementSource = InventoryMovementSourceWorkOrder | InventoryMovementSourceP1Dispatch | InventoryMovementSourceCycleCount | InventoryMovementSourceExternalRef;

export type InventoryMovementSourceCycleCount = {
  kind: "cycle_count";
  cycleCountId: Uuid;
  ccCode: string;
};

export type InventoryMovementSourceExternalRef = {
  kind: "external_ref";
  sourceRef: string | null;
};

export type InventoryMovementSourceP1Dispatch = {
  kind: "p1_dispatch";
  dispatchId: Uuid;
  workOrderId: Uuid;
};

export type InventoryMovementSourceWorkOrder = {
  kind: "work_order";
  workOrderId: Uuid;
};

export type InventoryMrpLine = {
  itemId: Uuid;
  ivCode: string;
  displayName: string;
  unitCode: string;
  quantityOnHandMilli: number;
  safetyStockMilli: number;
  inboundExpectedMilli: number;
  reservedOutboundMilli: number;
  monthlyUsageMilli: number;
  coverMonthsCenti?: number | null;
  short: boolean;
  proposedOrderMilli: number;
};

export type InventoryReceiptResult = {
  movement: InventoryMovement;
  item: InventoryItem;
};

export type InventoryStockLocationSummary = {
  id: Uuid;
  label: string;
};

export type KpiMetric = "completed_count" | "average_response_speed" | "completion_duration_and_due_compliance" | "revisit_rate" | "delay_rate_and_reason_distribution" | "inspection_plan_completion_rate" | "p1_acceptance_rate";

export type KpiReport = {
  period: Period;
  requested_scope: KpiScope;
  rollups: Array<KpiRollup>;
  unavailable_metrics: Array<UnavailableMetric>;
};

export type KpiRollup = {
  scope: KpiRollupScope;
  scope_display_name?: string | null;
  approved_report_count: number;
  completed_count: number;
  weighted_completed_points: number;
  average_response_seconds: number | null;
  average_completion_seconds: number | null;
  target_due_compliance_bps: number | null;
  revisit_rate_bps: number;
  delay_rate_bps: number;
  delay_reason_distribution: {
    [key: string]: number;
  };
  inspection_schedule_due_count: number;
  inspection_schedule_completed_count: number;
  inspection_plan_completion_bps: number | null;
  p1_dispatch_count: number;
  p1_accepted_count: number;
  p1_acceptance_bps: number | null;
};

/**
 * `id` is present for region, branch, and technician rollups and absent for company.
 */
export type KpiRollupScope = {
  kind: "company" | "region" | "branch" | "technician";
  id?: Uuid;
};

/**
 * `id` is present for region, branch, and technician scopes and absent for company.
 */
export type KpiScope = {
  kind: "company" | "region" | "branch" | "technician";
  id?: Uuid;
};

/**
 * Exact signed fixed-scale ledger days; historical imports may legitimately be negative.
 */
export type LeaveBalanceAmount = string;

export type LeaveBalanceItem = {
  id: Uuid;
  company: string;
  name: string;
  employee_number?: string | null;
  org_unit?: string | null;
  position?: string | null;
  leave_accrued?: string | null;
  leave_used?: string | null;
  leave_remaining?: string | null;
};

export type LeaveBalancePage = {
  items: Array<LeaveBalanceItem>;
  total: number;
  limit: number;
  offset: number;
  summary: LeaveBalanceSummary;
};

export type LeaveBalanceSummary = {
  accrued: string;
  used: string;
  remaining: string;
};

export type LeaveChargeResolutionRequest = {
  expected_version: number;
  date_charges: Array<LeaveDateCharge>;
  calendar_revision_ref: LeaveSourceRevisionRef;
  policy_revision_ref: LeaveSourceRevisionRef;
  supporting_source_refs?: Array<LeaveSourceRevisionRef>;
};

export type LeaveChargeResolutionView = {
  request_id: Uuid;
  request_version: number;
  charge_units: LeaveUnits;
  charge_state: "resolved";
  charge_version: number;
  server_digest: string;
  resolution_origin: "automated" | "manual";
  resolved_by: string | null;
};

export type LeaveChargeReviewReason = "missing_calendar" | "ambiguous_calendar" | "calendar_source_unavailable" | "missing_policy" | "ambiguous_policy" | "policy_source_unavailable";

/**
 * A self-service leave-request filing. The subject employee and branch are NOT accepted here — they are resolved from the authenticated caller. There is deliberately no `reason` field: 근로기준법 §60 grants the worker the 시기지정권, so a 연차/반차 filing carries no 사유 and a body that still sends one is rejected (422).
 */
export type LeaveCreateRequest = {
  idempotency_key: string;
  leave_type: "annual" | "half_day";
  partial_day_period?: "am" | "pm" | null;
  start_date: string;
  end_date: string;
};

export type LeaveDateCharge = {
  date: string;
  obligation: {
    kind: "scheduled";
    minutes: number;
  } | {
    kind: "not_scheduled";
    basis: "rest_day" | "public_holiday" | "substitute_holiday" | "contractual_day_off" | "other";
  };
  charge_units: LeaveUnits;
};

export type LeaveDecideRequest = {
  decision: "approve" | "time_change";
  comment?: string;
};

export type LeaveDecideV2Request = {
  expected_version: number;
  decision: "approve" | "time_change";
  comment?: string;
};

/**
 * A §61 연차 사용 촉진 push to a target employee.
 */
export type LeavePromotionRequest = {
  branch_id: Uuid;
  target_user_id: Uuid;
  target_employee_id: Uuid;
  target_name: string;
  round: number;
  unused_days?: number;
};

export type LeaveProposeAlternateDatesRequest = {
  start_date: string;
  end_date: string;
};

/**
 * A 노무수령거부 notice served after a round-2 promotion.
 */
export type LeaveRefusalRequest = {
  branch_id: Uuid;
  target_user_id: Uuid;
  target_employee_id: Uuid;
  target_name: string;
  unused_days?: number;
};

export type LeaveRequestPage = {
  items: Array<LeaveRequestView>;
};

export type LeaveRequestV2Page = {
  items: Array<LeaveRequestV2View>;
  next_cursor: string | null;
};

/**
 * One leave request in the approval queue (결재함 leave variant).
 */
export type LeaveRequestV2View = {
  id: Uuid;
  branch_id: Uuid;
  requester_user_id: Uuid;
  subject_employee_id: Uuid;
  leave_type: "annual" | "half_day";
  days: number;
  charge_units: LeaveUnits | null;
  charge_state: "review_required" | "resolved" | "not_required" | "legacy_unverified";
  charge_review_reasons: Array<LeaveChargeReviewReason>;
  request_version: number;
  charge_version: number;
  charge_digest?: string;
  charge_resolved_by?: Uuid;
  charge_resolution_origin?: "automated" | "manual";
  partial_day_period?: "am" | "pm";
  start_date: string;
  end_date: string;
  reason?: string;
  status: "pending" | "approved" | "time_change_consult" | "returned" | "rejected";
  decided_by: string | null;
  decided_at: string | null;
  decision_comment?: string;
  time_change_grounds?: TimeChangeGroundsCode;
  time_change_evidence?: TimeChangeCoverageEvidence;
  alternate_start_date?: string;
  alternate_end_date?: string;
  alternate_partial_day_period?: "am" | "pm";
  alternate_proposed_at?: string;
  ap_run_id?: string;
  created_at: Timestamp;
};

/**
 * One leave request in the approval queue (결재함 leave variant).
 */
export type LeaveRequestView = {
  id: Uuid;
  branch_id: Uuid;
  requester_user_id: Uuid;
  subject_employee_id: Uuid;
  leave_type: "annual" | "half_day";
  days: number;
  start_date: string;
  end_date: string;
  reason: string;
  status: "pending" | "approved" | "time_change_consult" | "returned" | "rejected";
  decided_by: string | null;
  decided_at: string | null;
  decision_comment?: string;
  ap_run_id?: string;
  created_at: Timestamp;
};

/**
 * One employee's annual-leave balance row (직원별 연차 현황).
 */
export type LeaveRosterEntry = {
  employee_id: Uuid;
  name: string;
  team: string | null;
  grant: number;
  used: number;
  left: number;
  tone: "ok" | "promote" | "low";
};

export type LeaveRosterPage = {
  items: Array<LeaveRosterEntry>;
};

export type LeaveSourceRevisionRef = {
  kind: string;
  reference: string;
  revision: string;
};

/**
 * The result of a §61 push — the delivered notice + engine state.
 */
export type LeaveStatutoryPushView = {
  id: Uuid;
  kind: "promotion" | "refusal";
  round: number;
  target_user_id: Uuid;
  inbox_doc_id: Uuid;
  ap_run_id?: string;
  ap_submission: "submitted" | "pending_engine_definition";
};

/**
 * Exact fixed-scale leave days; six fractional digits and no floating-point semantics.
 */
export type LeaveUnits = string;

export type LegalHoldRecordView = {
  id: string;
  evidence_object_id: string;
  status: LegalHoldStatus;
  case_ref: string;
  basis: string;
  reason: string;
  applied_by: string;
  applied_at: string;
  released_by?: string | null;
  released_at?: string | null;
  release_reason?: string | null;
  audit_event_id?: string | null;
};

export type LegalHoldState = "CLEAR" | "ACTIVE";

export type LegalHoldStatus = "ACTIVE" | "RELEASED";

export type LifecycleOutcome = {
  instance: InstanceHead;
  config: GateChainConfig;
  gates: GateChainOutcome;
};

export type LifecyclePreflight = {
  configured: boolean;
  config: GateChainConfig;
  outcome: GateChainOutcome;
};

export type LifecycleRequest = {
  to_state: InstanceLifecycleState;
  reason?: string | null;
  checklist_all_acknowledged?: boolean | null;
  four_eyes_request_ref?: string | null;
};

/**
 * Instance lifecycle state. Dispose is terminal; there is no hard delete.
 */
export type LifecycleState = "DRAFT" | "ACTIVE" | "LOCKED" | "ARCHIVED" | "DISPOSED";

export type LifecycleTransitionConfig = {
  object_type_id: Uuid;
  from_state: LifecycleState;
  to_state: LifecycleState;
  requirements: TransitionRequirements;
};

export type LinkControlObligationRequest = {
  control_id: Uuid;
  obligation_id: Uuid;
  coverage_level: CoverageLevel;
  coverage_rationale?: string | null;
};

export type LinkObligationRegulationRequest = {
  obligation_id: Uuid;
  regulation_impact_id: Uuid;
  relationship: ObligationRegulationRelationship;
  rationale?: string | null;
};

/**
 * At least one of site_id / work_order_id must be present. An absent field leaves that link untouched; an explicit null clears it.
 */
export type LinkSupportTicketRequest = {
  site_id?: string | null;
  work_order_id?: string | null;
};

/**
 * A registered edge type (object-link relationship label).
 */
export type LinkTypeResponse = {
  link_type: string;
  description: string;
  status: "draft" | "active" | "archived";
};

export type LinkTypeSummary = {
  id: string;
  stable_key: string;
  title: string;
  reverse_title?: string | null;
  to_object_type_id?: string | null;
  cardinality: "one_one" | "one_many" | "many_many";
  traversable: boolean;
};

/**
 * Whether a listed unit is used (중고) or brand-new (신차).
 */
export type ListingCondition = "USED" | "NEW";

/**
 * Fuel / drive class of a listed forklift.
 */
export type ListingKind = "ELECTRIC" | "DIESEL" | "LPG" | "REACH";

/**
 * One photo attached to a sales listing.
 */
export type ListingMediaView = {
  id: string;
  url: string;
  content_type: string;
  alt_text: string | null;
  sort_order: number;
};

/**
 * Publication lifecycle of a listing.
 */
export type ListingStatus = "DRAFT" | "PUBLISHED" | "RESERVED" | "SOLD" | "WITHDRAWN";

/**
 * Whether a listing is offered for sale, rental, or both.
 */
export type ListingType = "SALE" | "RENTAL" | "BOTH";

export type LocationConsentLedgerEntry = {
  id: Uuid;
  consent_id: Uuid;
  user_id: Uuid;
  branch_id: Uuid;
  actor?: string | null;
  action: "consent.grant" | "consent.suspend" | "consent.resume" | "consent.withdraw";
  from_status: LocationConsentState;
  to_status: LocationConsentState;
  occurred_at: Timestamp;
  created_at: Timestamp;
};

export type LocationConsentLedgerPage = {
  items: Array<LocationConsentLedgerEntry>;
  limit: number;
  offset: number;
  total: number;
};

export type LocationConsentState = "NO_RECORD" | "GRANTED" | "SUSPENDED" | "WITHDRAWN";

export type LocationConsentStatus = {
  consent_id: Uuid;
  user_id: Uuid;
  branch_id: Uuid;
  state: LocationConsentState;
  may_collect: boolean;
  granted_at?: string | null;
  suspended_at?: string | null;
  resumed_at?: string | null;
  withdrawn_at?: string | null;
  updated_at?: string | null;
};

export type LocationConsentTransitionRequest = {
  branch_id?: Uuid;
};

export type LocationPingRequest = {
  branch_id?: Uuid;
  latitude: number;
  longitude: number;
  accuracy_m?: number | null;
  recorded_at: Timestamp;
  on_duty: boolean;
};

export type LogisticsAsnCreated = {
  id: Uuid;
  status: "EXPECTED";
  branchId: Uuid;
};

export type LogisticsAsnPutaway = {
  id: Uuid;
  status: "PUTAWAY";
};

export type LogisticsAsnReceipt = {
  id: Uuid;
  status: "PARTIAL_RECEIVED" | "RECEIVED";
  receivedQuantity?: number;
  replayed?: boolean;
};

export type LogisticsFulfillmentPacked = {
  id: Uuid;
  status: "PACKED";
  pickedQuantity: number;
};

export type LogisticsFulfillmentPicked = {
  id: Uuid;
  status: "PICKED" | "SHORT_PICK";
  pickedQuantity: number;
};

export type LogisticsFulfillmentReleased = {
  id: Uuid;
  status: "RELEASED";
  reservedQuantity: number;
};

export type LogisticsPodVerified = {
  id: Uuid;
  status: "DELIVERED";
  recipientConfirmedEvidenceReference: string;
  slaAssessment: "MET" | "BREACHED";
};

export type LogisticsShipmentDispatched = {
  id: Uuid;
  fulfillmentId: Uuid;
  status: "DISPATCHED";
};

export type LogisticsShipmentSettlement = {
  id: Uuid;
  status: "SETTLED";
  operationalCost: {
    currency: "KRW";
    amountMinor: number;
  };
  financeGlPosting: null;
};

/**
 * Wire form of the logistics pilot datetime fields (dueAt, confirmedAt, settledAt): the time-crate default serde tuple [year, ordinal-day, hour, minute, second, nanosecond, offsetHours, offsetMinutes, offsetSeconds]. The deployed rest crate deserializes these as plain OffsetDateTime without rfc3339 serde, so an RFC3339 string is rejected with 422. Encode in UTC (offsets 0). Divergence recorded in docs/evidence/console/CAP-LOGISTICS-PILOT/manifests/openapi-applied-notes.md.
 */
export type LogisticsTimeTuple = Array<number>;

export type LogoutRequest = RefreshTokenRequest;

/**
 * The write-only view of a configured mailbox. It NEVER contains a password; has_smtp_password / has_imap_password signal whether a credential is on file.
 */
export type MailAccountView = {
  id: Uuid;
  display_name: string;
  email_address: string;
  from_name?: string | null;
  imap_host: string;
  imap_port: number;
  imap_security: MailSecurity;
  imap_username: string;
  smtp_host: string;
  smtp_port: number;
  smtp_security: MailSecurity;
  smtp_username: string;
  has_smtp_password: boolean;
  has_imap_password: boolean;
  status: string;
};

export type MailAddress = {
  address: string;
  name?: string | null;
};

export type MailAttachment = {
  filename: string;
  content_type: string;
  content_base64: string;
};

export type MailAttachmentDownload = {
  url: string;
};

export type MailAttachmentView = {
  id: Uuid;
  filename: string;
  content_type: string;
  size_bytes: number;
  is_inline: boolean;
};

export type MailFolderView = {
  id: Uuid;
  role: string;
  name: string;
  unread_count: number;
  total_count: number;
};

export type MailMessageView = {
  id: Uuid;
  thread_id: Uuid;
  direction: string;
  message_id?: string | null;
  in_reply_to?: string | null;
  from_address: string;
  from_name?: string | null;
  to: Array<MailAddress>;
  cc: Array<MailAddress>;
  subject: string;
  snippet: string;
  body_text?: string | null;
  body_html?: string | null;
  seen: boolean;
  flagged: boolean;
  answered: boolean;
  has_attachments: boolean;
  received_at: string;
  attachments: Array<MailAttachmentView>;
};

/**
 * Transport security. SSL_TLS is implicit TLS (port 993/465); START_TLS upgrades a plaintext connection (port 143/587). There is no plaintext option.
 */
export type MailSecurity = "SSL_TLS" | "START_TLS";

export type MailTestConnectionResult = {
  ok: boolean;
  error_code?: string | null;
};

export type MailThreadDetail = {
  id: Uuid;
  subject: string;
  messages: Array<MailMessageView>;
};

export type MailThreadReadStateRequest = {
  seen: boolean;
};

export type MailThreadView = {
  id: Uuid;
  subject: string;
  last_message_at: string;
  message_count: number;
  unread_count: number;
  has_attachments: boolean;
  is_flagged: boolean;
};

/**
 * Typed maintenance cause (원인).
 */
export type MaintenanceCause = "BREAKDOWN" | "RETURN_PREP" | "SCHEDULED" | "INSPECTION_FINDING" | "OTHER";

/**
 * Typed maintenance classification (유형).
 */
export type MaintenanceType = "EMERGENCY" | "CORRECTIVE" | "PREVENTIVE" | "INSPECTION";

export type MarkMessengerThreadReadRequest = {
  last_read_message_id: Uuid;
};

export type MeAuthzCapability = {
  feature: string;
  permission: "request_only" | "limited" | "allow";
  branch_scope: BranchScope;
};

/**
 * NON-AUTHORITATIVE authorization projection for the caller. A rendering hint only — the backend matrix is the sole enforcer.
 */
export type MeAuthzResponse = {
  authority: "advisory_ui_only";
  source: "legacy_matrix" | "cedar";
  user_id: Uuid;
  org_id: Uuid;
  roles: Array<string>;
  branch_scope: BranchScope;
  capabilities: Array<MeAuthzCapability>;
};

export type MediaKind = "IMAGE" | "VIDEO";

export type MessengerAckSummary = {
  message_id: Uuid;
  thread_id: Uuid;
  acked: boolean;
  ack_count: number;
};

export type MessengerMemberListResponse = {
  items: Array<MessengerMemberSummary>;
};

export type MessengerMemberPresence = {
  user_id: Uuid;
  display_name: string | null;
  last_activity_at: string | null;
  status: MessengerPresenceStatus;
};

export type MessengerMemberPresenceListResponse = {
  items: Array<MessengerMemberPresence>;
};

export type MessengerMemberSummary = {
  id: Uuid;
  display_name: string;
  team: string | null;
};

export type MessengerMessageListResponse = {
  items: Array<MessengerMessageSummary>;
};

export type MessengerMessagePage = {
  items: Array<MessengerMessageSummary>;
  next_cursor: string | null;
};

export type MessengerMessageSummary = {
  id: Uuid;
  thread_id: Uuid;
  branch_id: Uuid;
  sender_id: Uuid;
  sender_name: string | null;
  body: string;
  attachment_evidence_ids: Array<Uuid>;
  read_count: number;
  read_target_count: number;
  ack_count: number;
  acked_by_me: boolean;
  quoted_message_id: string | null;
  quoted_body: string | null;
  quoted_sender_name: string | null;
  sent_at: Timestamp;
  created_at: Timestamp;
};

/**
 * Activity-derived presence — online within the freshness window of the member's last action, else away, else offline (or never seen).
 */
export type MessengerPresenceStatus = "online" | "away" | "offline";

export type MessengerReadReceiptSummary = {
  thread_id: Uuid;
  user_id: Uuid;
  last_read_message_id: Uuid;
  read_at: Timestamp;
  updated_at: Timestamp;
};

export type MessengerThreadKind = "work_order" | "team" | "dm" | "group";

export type MessengerThreadListResponse = {
  items: Array<MessengerThreadSummary>;
};

export type MessengerThreadMuteSummary = {
  thread_id: Uuid;
  muted: boolean;
};

export type MessengerThreadSummary = {
  id: Uuid;
  kind: MessengerThreadKind;
  visibility: MessengerThreadVisibility;
  muted: boolean;
  branch_id: Uuid;
  title: string | null;
  work_order_id: string | null;
  last_message_id: string | null;
  last_message_at: string | null;
  member_count: number;
  unread_count: number;
  created_at: Timestamp;
  updated_at: Timestamp;
};

/**
 * channel = a named, branch-scoped room any active branch member may join; direct = a fixed member set (DMs, work-order threads, groups).
 */
export type MessengerThreadVisibility = "channel" | "direct";

export type MobileApproveWorkOrderRequest = {
  comment: string;
  step_up: MobilePasskeyStepUpEnvelope;
};

export type MobilePasskeyStepUpBinding = {
  action_kind: MobileStepUpActionKind;
  object_id: Uuid;
  reason_key: "operations_passkey_approval_decision" | "operations_passkey_poll_vote";
  replay_attempt: number | null;
};

export type MobilePasskeyStepUpEnvelope = {
  binding: MobilePasskeyStepUpBinding;
  assertion: PasskeyStepUpAssertion;
};

export type MobilePasskeyStepUpStartRequest = {
  binding: MobilePasskeyStepUpBinding;
};

export type MobilePasskeyStepUpStartResponse = {
  ceremony_id: Uuid;
  challenge: {
    [key: string]: unknown;
  };
  expires_at: Timestamp;
  binding: MobilePasskeyStepUpBinding;
};

export type MobileStepUpActionKind = "APPROVAL_DECISION" | "POLL_VOTE";

export type MobileVotePollRequest = {
  selected_option_ids: Array<Uuid>;
  step_up: MobilePasskeyStepUpEnvelope;
};

/**
 * One pending P1 offer for the signed-in mechanic: a BROADCASTING dispatch that fanned out to the caller, still inside its accept window, with no response from the caller yet.
 */
export type MyDispatchOffer = {
  dispatch_id: Uuid;
  work_order_id: Uuid;
  branch_id: Uuid;
  request_no: string;
  accept_window_started_at: Timestamp;
  accept_window_ends_at: Timestamp;
};

export type MyDispatchOfferPage = {
  items: Array<MyDispatchOffer>;
};

export type MyLeaveV2Overview = {
  balance: SelfLeaveBalance;
  requests: LeaveRequestV2Page;
};

export type MyPayrollLine = {
  run_id: Uuid;
  period_start: string;
  period_end: string;
  run_status: "STAGED" | "BLOCKED_LEGAL_GATE" | "READY_FOR_REVIEW" | "ATTENDANCE_CLOSED" | "CALCULATING" | "CALCULATED" | "SUBMITTED" | "REJECTED" | "APPROVED" | "DISBURSEMENT_SCHEDULED" | "PAID" | "ISSUED" | "VOID";
  calculation_status: "BLOCKED_LEGAL_GATE" | "READY_FOR_REVIEW" | "APPROVED" | "ISSUED" | "VOID";
  work_days?: number | null;
  regular_hours?: number | null;
  overtime_hours?: number | null;
  night_hours?: number | null;
  holiday_hours?: number | null;
  leave_used?: number | null;
  leave_remaining?: number | null;
  gross_pay_source_present: boolean;
  net_pay_source_present: boolean;
};

export type MyPayrollLinePage = {
  items: Array<MyPayrollLine>;
  total: number;
  limit: number;
  offset: number;
};

export type MyWorkbenchResponse = {
  as_of: Timestamp;
  timezone: "Asia/Seoul";
  range: WorkbenchRange;
  scope: WorkbenchEffectiveScope;
  partial: boolean;
  action_inbox: WorkbenchActionSourceEnvelope;
  todos: WorkbenchTodoSourceEnvelope;
  calendar: WorkbenchCalendarSourceEnvelope;
};

export type NamedEntity = {
  id: Uuid;
  name: string;
};

export type NoticeAudienceInput = {
  scope: "org" | "branches";
  branch_ids?: Array<Uuid>;
};

export type NoticeCategory = "general" | "legal" | "hr_order" | "training";

export type NoticeMyReceipt = {
  acknowledged_at: string | null;
};

export type NoticeProgress = {
  total: number;
  acknowledged: number;
};

export type NoticeReceipt = {
  recipient_user_id: Uuid;
  display_name: string;
  acknowledged_at: string | null;
};

export type NoticeReceiptPage = {
  items: Array<NoticeReceipt>;
  total: number;
};

export type NoticeSummary = {
  id: Uuid;
  code: string | null;
  author_user_id: Uuid;
  title: string;
  body: string;
  status: "draft" | "published";
  published_at: string | null;
  created_at: Timestamp;
  category: NoticeCategory;
  audience_scope: "org" | "branches";
  audience_branches: Array<NamedEntity>;
  my_receipt: NoticeMyReceipt | null;
  progress: NoticeProgress | null;
};

export type NotificationCategoryCount = {
  category: string;
  unread: number;
};

export type NotificationCountsSummary = {
  total_unread: number;
  by_category: Array<NotificationCategoryCount>;
  muted_unread: number;
};

/**
 * Deep-link target — an object reference (kind + id) or a bare app screen.
 */
export type NotificationLink = {
  "type": "object";
  kind: string;
  id: string;
} | {
  "type": "screen";
  screen: string;
};

export type NotificationObjectGroup = {
  link: NotificationLink;
  total: number;
  unread: number;
  categories: Array<NotificationCategoryCount>;
  latest: NotificationSummary;
  muted: boolean;
};

export type NotificationObjectGroupPage = {
  items: Array<NotificationObjectGroup>;
  next_cursor: string | null;
};

export type NotificationPage = {
  items: Array<NotificationSummary>;
  next_cursor: string | null;
};

export type NotificationPolicyList = {
  items: Array<NotificationPolicySummary>;
};

export type NotificationPolicySummary = {
  id: Uuid;
  scope: "all" | "category" | "object";
  category?: string;
  link?: NotificationLink;
  action: string;
  created_at: Timestamp;
};

export type NotificationReadAllResponse = {
  marked: number;
};

export type NotificationSummary = {
  id: Uuid;
  recipient_user_id: Uuid;
  category: string;
  kind: string;
  text: string;
  link: NotificationLink;
  unread: boolean;
  created_at: Timestamp;
  read_at: string | null;
  resolved_at: string | null;
  muted: boolean;
};

export type ObjectActionCatalogResponse = {
  object_type: string;
  object_id: Uuid;
  actions: Array<ObjectActionDescriptor>;
};

export type ObjectActionDescriptor = {
  action_id: string;
  object_type: string;
  object_id: Uuid;
  label: string;
  description: string;
  submit_label: string;
  requires_passkey_step_up: boolean;
  risk_level: "sensitive_write";
  fields: Array<ObjectActionFieldDescriptor>;
};

export type ObjectActionExecutionResponse = {
  execution_id: Uuid;
  action_id: string;
  object_type: string;
  object_id: Uuid;
  status: "succeeded";
  audit_event_id: Uuid;
  target_href: string;
  message: string;
};

export type ObjectActionFieldDescriptor = {
  field_key: string;
  label: string;
  field_type: "text" | "select";
  required: boolean;
  current_value?: string | null;
  options: Array<ObjectActionFieldOption>;
};

export type ObjectActionFieldOption = {
  value: string;
  label: string;
};

/**
 * The bounded, caller-visible neighborhood of an object, walked over object_links. Unresolvable nodes are omitted entirely (never returned as stubs), and edges touching an omitted node are omitted too.
 */
export type ObjectGraphResponse = {
  nodes: Array<ObjectHead>;
  edges: Array<ObjectLinkResponse>;
  truncated: boolean;
};

/**
 * Compact, kind-agnostic head for any object. exists=false means the object is absent OR outside the caller's scope (indistinguishable, by design). Carries no route/URL — objectRegistry (frontend) is the sole kind->URL authority.
 */
export type ObjectHead = {
  kind: string;
  id: string;
  code?: string | null;
  title?: string | null;
  status?: string | null;
  exists: boolean;
};

export type ObjectLifecycle = {
  objectType: string;
  objectId: string;
  currentState: string;
  legalHold: boolean;
  retentionUntil?: string;
  createdAt: string;
  updatedAt: string;
  transitions: Array<ObjectLifecycleTransition>;
};

export type ObjectLifecycleTransition = {
  fromState: string;
  toState: string;
  actor?: string;
  reason: string;
  occurredAt: string;
};

export type ObjectLinkResponse = {
  id: Uuid;
  src_kind: string;
  src_id: string;
  dst_kind: string;
  dst_id: string;
  link_type: string;
  created_by?: Uuid | null;
  created_at: Timestamp;
};

/**
 * Links touching one object, split by direction.
 */
export type ObjectLinksListResponse = {
  outgoing: Array<ObjectLinkResponse>;
  incoming: Array<ObjectLinkResponse>;
};

export type ObjectTypeDetail = {
  object_type: ObjectTypeSummary;
  title_property_key?: string | null;
  backing_table?: string | null;
  primary_key_property?: string | null;
  properties: Array<PropertyDefSummary>;
  links: Array<LinkTypeSummary>;
  actions: Array<ActionTypeSummary>;
  analytics: Array<AnalyticSummary>;
};

/**
 * An object-type registry row plus the caller-visible active instance count.
 */
export type ObjectTypeResponse = {
  kind: string;
  code_prefix?: string | null;
  description: string;
  status: "draft" | "active" | "archived";
  active_count: number;
};

export type ObjectTypeSummary = {
  id: string;
  stable_key: string;
  title: string;
  backing_kind: "instance" | "projected";
  schema_version: number;
  lifecycle_state: "draft" | "review_pending" | "published" | "superseded" | "retired";
  key_write_revision: number;
  key_write_etag: string;
};

export type ObligationRegulationLink = {
  id: Uuid;
  obligation_id: Uuid;
  regulation_impact_id: Uuid;
  relationship: ObligationRegulationRelationship;
  rationale?: string | null;
  created_by: Uuid;
  created_at: Timestamp;
};

export type ObligationRegulationRelationship = "DERIVED_FROM" | "AMENDED_BY" | "SUPERSEDED_BY" | "INTERPRETS" | "EVIDENCES";

export type ObligationStatus = "DRAFT" | "ACTIVE" | "WAIVED" | "SUPERSEDED" | "ARCHIVED";

export type ObligationType = "LEGAL" | "REGULATORY" | "CONTRACTUAL" | "INTERNAL_POLICY" | "CONTROL_REQUIREMENT";

/**
 * Immutable replay receipt for an accepted instance_revision action command.
 */
export type OntologyActionCommandReceipt = {
  command_id: Uuid;
  payload_digest: string;
  instance: InstanceState;
  gates: GateChainOutcome;
};

export type OntologyActionExecuteOutcome = {
  dispatch: "instance_revision" | "projected_usecase";
  gates: GateChainOutcome;
  instance?: InstanceState;
  projected?: {
    [key: string]: unknown;
  };
  receipt?: OntologyActionCommandReceipt;
};

/**
 * An object-action preflight/execute request. Canonical DispatchTarget `params` `$ref` the typed input schemas; generic instance-revision actions keep a free-form object validated against the action's `params_schema` at runtime.
 */
export type OntologyActionRequest = {
  object_type_id: Uuid;
  instance_id?: Uuid;
  title?: string;
  params?: CompanyReviseInput | OrganizationCreateOrgUnitInput | OrganizationReviseOrgUnitInput | OrganizationCreateJobPositionInput | OrganizationReviseJobPositionInput | PeopleCreatePersonInput | PeopleRevisePersonInput | HrAppointInput | HrPromoteInput | HrTransferInput | PayrollCreateRunInput | PayrollSubmitRunInput | PayrollDecideRunInput | {
    [key: string]: unknown;
  };
  reason?: string;
  valid_from?: Timestamp;
  checklist_all_acknowledged?: boolean;
  four_eyes_request_ref?: Uuid;
  command_id?: Uuid;
  expected_revision?: number;
};

export type OntologyInstanceAggregateBucket = {
  key?: string | null;
  count: number;
};

export type OpenCycleCountRequest = {
  branchId: Uuid;
  stockLocationId: Uuid;
};

export type OpsEquipmentStatus = {
  rented: number;
  spare: number;
  scrapped: number;
  replacement: number;
  sold: number;
};

export type OpsFunnel = {
  received: number;
  assigned: number;
  in_progress: number;
  completed: number;
};

export type OpsMechanicLoad = {
  mechanic_id: string;
  display_name: string;
  active_assignments: number;
};

export type OpsSummary = {
  funnel: OpsFunnel;
  aging_hours: number;
  aging_work_orders: number;
  sla_breached: number;
  sla_at_risk: number;
  mechanic_load: Array<OpsMechanicLoad>;
  equipment_status: OpsEquipmentStatus;
  active_substitutions: number;
  pending_approvals: number;
  open_support_tickets: number;
};

export type OrgChangeApprovalRoleKey = "hr" | "finance" | "legal" | "executive";

export type OrgChangeApprovalStep = {
  id: Uuid;
  stepOrder: number;
  roleKey: OrgChangeApprovalRoleKey;
  decision: OrgChangeStepDecision;
  decidedBy?: Uuid;
  decidedAt?: Timestamp;
  memo?: string;
};

export type OrgChangeDecisionRequest = {
  decision: "APPROVED" | "REJECTED";
  memo?: string;
};

export type OrgChangeDetail = OrgChangeSummary & {
  proposal: Array<OrgProposalOp>;
  preflight?: OrgChangePreflightReport;
  approvalSteps: Array<OrgChangeApprovalStep>;
  settlementItems: Array<OrgChangeSettlementItem>;
  events: Array<OrgChangeEvent>;
};

export type OrgChangeEvent = {
  at: Timestamp;
  actor: Uuid;
  action: string;
  fromStatus?: string;
  toStatus?: string;
  reason?: string;
};

export type OrgChangeKind = "NEW" | "REORG" | "DISSOLVE";

export type OrgChangePage = {
  items: Array<OrgChangeSummary>;
  total: number;
};

export type OrgChangePreflightBlocker = {
  code: string;
  label: string;
  dependentKind: string;
  count: number;
};

export type OrgChangePreflightReport = {
  computedAt: Timestamp;
  stale: boolean;
  blockers: Array<OrgChangePreflightBlocker>;
  warnings: Array<OrgChangePreflightWarning>;
  headcount: number;
  dependentsTotal: number;
};

export type OrgChangePreflightWarning = {
  code: string;
  label: string;
  dependentKind?: string;
  count?: number;
};

export type OrgChangeSettlementItem = {
  id: Uuid;
  itemKey: OrgChangeSettlementKey;
  label: string;
  done: boolean;
  doneBy?: Uuid;
  doneAt?: Timestamp;
  memo?: string;
};

export type OrgChangeSettlementKey = "TRANSFER_EMPLOYEES" | "POSITIONS" | "COST_CENTERS" | "CLOSE_OPEN_DOCS" | "ASSETS" | "PAYROLL_SOCIAL_FINAL";

/**
 * PRECHECKED is legacy and is never entered by a request created today — preflight is read-only, so the lifecycle is DRAFT → IN_APPROVAL at submit. It stays in the enum because rows written before that change still carry it on disk and are still accepted by PATCH, preflight, submit and cancel. Do not poll for it.
 */
export type OrgChangeStatus = "DRAFT" | "PRECHECKED" | "IN_APPROVAL" | "APPROVED" | "APPLIED" | "SETTLING" | "ARCHIVED" | "REJECTED" | "CANCELLED";

export type OrgChangeStepDecision = "PENDING" | "APPROVED" | "REJECTED";

export type OrgChangeSummary = {
  id: Uuid;
  code: string;
  kind: OrgChangeKind;
  status: OrgChangeStatus;
  target: OrgChangeTarget;
  effectiveDate: Date;
  reason: string;
  headcount: number;
  siteCount: number;
  teamCount: number;
  draftedBy: Uuid;
  createdAt: Timestamp;
  updatedAt: Timestamp;
  supersedesId?: Uuid;
};

export type OrgChangeTarget = {
  kind: OrgChangeTargetKind;
  ref: string;
  label: string;
};

export type OrgChangeTargetKind = "ENTITY" | "REGION" | "BRANCH" | "SITE" | "ORG_UNIT";

export type OrgEntitySummary = {
  orgId: Uuid;
  slug: string;
  name: string;
  status: string;
};

/**
 * One typed sandbox-diff operation, replayed in order at apply time. Internally tagged by `op`; unknown properties are rejected.
 */
export type OrgProposalOp = OrgProposalOpCreateRegion | OrgProposalOpRenameRegion | OrgProposalOpDeactivateRegion | OrgProposalOpCreateBranch | OrgProposalOpRenameBranch | OrgProposalOpDeactivateBranch | OrgProposalOpCreateSite | OrgProposalOpUpdateSite | OrgProposalOpReassignOrgUnit;

export type OrgProposalOpCreateBranch = {
  op: "CREATE_BRANCH";
  regionId: Uuid;
  name: string;
};

export type OrgProposalOpCreateRegion = {
  op: "CREATE_REGION";
  name: string;
};

export type OrgProposalOpCreateSite = {
  op: "CREATE_SITE";
  customerId: Uuid;
  name: string;
};

export type OrgProposalOpDeactivateBranch = {
  op: "DEACTIVATE_BRANCH";
  branchId: Uuid;
};

export type OrgProposalOpDeactivateRegion = {
  op: "DEACTIVATE_REGION";
  regionId: Uuid;
};

/**
 * Reassign employees from one OrgUnit to another via canonical `hr.transfer` (one transfer per matched employee). `fromOrgUnit` and `toOrgUnit` are OrgUnit UUIDs (not free-text team labels) and must differ.
 */
export type OrgProposalOpReassignOrgUnit = {
  op: "REASSIGN_ORG_UNIT";
  fromOrgUnit: Uuid;
  toOrgUnit: Uuid;
  scope: OrgProposalReassignScope;
};

/**
 * Rename and/or move a branch across regions; at least one of `name` and `regionId` must be present.
 */
export type OrgProposalOpRenameBranch = {
  op: "RENAME_BRANCH";
  branchId: Uuid;
  name?: string;
  regionId?: Uuid;
};

export type OrgProposalOpRenameRegion = {
  op: "RENAME_REGION";
  regionId: Uuid;
  name: string;
};

export type OrgProposalOpUpdateSite = {
  op: "UPDATE_SITE";
  siteId: Uuid;
  name: string;
};

export type OrgProposalReassignScope = {
  company: string;
};

export type OtpRedeemRequest = {
  otp: string;
};

export type OtpRedeemResponse = {
  access_token: string;
  refresh_token?: string | null;
  token_type: string;
  refresh_expires_at: Timestamp;
  requires_passkey_setup: boolean;
};

export type OutsourceWorkSummary = {
  id?: Uuid;
  work_order_id?: Uuid;
  vendor_id?: Uuid;
  vendor_name?: string;
  status?: "REQUESTED" | "ASSIGNED" | "IN_PROGRESS" | "RESULT_SUBMITTED" | "COMPLETED" | "CANCELLED";
};

export type OverrideSummary = {
  id: Uuid;
  target_type: string;
  target_id: Uuid;
  actor: Uuid;
  reason: string;
  before_snapshot: {
    [key: string]: unknown;
  };
  created_at: Timestamp;
};

export type OwnAttendanceException = {
  id: Uuid;
  code: string;
  kind: "LATE" | "NO_SHOW" | "UNAPPROVED_OVERTIME" | "EARLY_LEAVE";
  status: "OPEN" | "RESOLVED";
  work_date: string;
  occurred_at: string;
  detail: string;
  evidence: Array<AttendanceExceptionEvidence>;
  resolution?: OwnAttendanceExceptionResolution | null;
  created_at: string;
};

export type OwnAttendanceExceptionPage = {
  items: Array<OwnAttendanceException>;
  total: number;
  limit: number;
  offset: number;
};

export type OwnAttendanceExceptionResolution = {
  action: string;
  reason: string;
  ot_hours?: string | null;
  resolved_at: string;
};

export type OwnAttendanceWeek52 = {
  week_start: string;
  current_hours: number;
  projected_hours: number;
  tone: "OK" | "WARN" | "DANGER";
  acknowledged_at?: string | null;
};

export type OwnAttendanceWeek52Response = {
  status: "available" | "not_available";
  projection?: OwnAttendanceWeek52;
};

export type OwnershipTransfer = {
  id: string;
  equipment_id: string;
  branch_id: string;
  from_owner: string;
  to_owner: string;
  reason: string;
  status: "PENDING" | "APPROVED" | "REJECTED";
  current_step: "sending_org_admin" | "receiving_org_admin" | "legal_signoff" | "accounting_signoff" | null;
  approval_line: Array<OwnershipTransferStep>;
  requested_by?: string | null;
  requested_at: string;
  decided_at?: string | null;
  completed_at?: string | null;
};

export type OwnershipTransferPage = {
  items: Array<OwnershipTransfer>;
};

export type OwnershipTransferStep = {
  step_key: "sending_org_admin" | "receiving_org_admin" | "legal_signoff" | "accounting_signoff";
  label: string;
  status: "WAITING" | "PENDING" | "APPROVED" | "REJECTED";
  decided_by?: string | null;
  decided_at?: string | null;
  comment?: string | null;
};

export type P1DispatchResponsePage = {
  items: Array<P1DispatchResponseSummary>;
};

export type P1DispatchResponseSummary = {
  dispatch_id: Uuid;
  user_id: Uuid;
  response: DispatchResponseKind;
  responded_at: Timestamp;
  score_milli?: number;
  gps_ranked: boolean;
  distance_meters?: number;
  score_reason?: string;
};

export type P1DispatchSummary = {
  id: Uuid;
  work_order_id: Uuid;
  branch_id: Uuid;
  status: DispatchStatus;
  incident_location?: IncidentLocation;
  accept_window_started_at: Timestamp;
  accept_window_ends_at: Timestamp;
  auto_assigned_mechanic_id?: Uuid;
  manager_force_pending_at?: Timestamp;
  manual_call_required: boolean;
  manual_call_required_at?: Timestamp;
  manual_call_cleared_at?: Timestamp;
  target_count: number;
  accepted_count: number;
  declined_count: number;
};

export type PasskeyLoginFinishRequest = {
  ceremony_id: Uuid;
  credential: {
    [key: string]: unknown;
  };
};

export type PasskeyLoginStartResponse = {
  ceremony_id: Uuid;
  challenge: {
    [key: string]: unknown;
  };
  expires_at: Timestamp;
};

export type PasskeyRegisterFinishRequest = {
  ceremony_id: Uuid;
  credential: {
    [key: string]: unknown;
  };
};

export type PasskeyRegisterFinishResponse = {
  passkey_id: Uuid;
  user_id: Uuid;
  credential_id: string;
};

/**
 * Optional overrides for the authenticated session user's passkey registration (username/display_name default to the user's stored profile when omitted), plus the step-up assertion required to ADD a passkey when the user already has one. A user with zero passkeys (initial enrollment) omits `step_up`; an already-enrolled user MUST supply a fresh `step_up` assertion of an existing passkey (user verification required), or register/start returns 401 — so a stolen session cannot silently add a credential.
 */
export type PasskeyRegisterStartRequest = {
  username?: string;
  display_name?: string;
  step_up?: PasskeyStepUpAssertion;
};

export type PasskeyRegisterStartResponse = {
  ceremony_id: Uuid;
  challenge: {
    [key: string]: unknown;
  };
  expires_at: Timestamp;
};

/**
 * A fresh assertion of an EXISTING passkey, proving the caller currently possesses an authenticator (not just a bearer token). The `ceremony_id` comes from a preceding `POST /api/v1/auth/passkey/login/start`; the `credential` is the resulting WebAuthn assertion. Verified with user verification (UV) required and rejected unless the asserted credential belongs to the authenticated caller.
 */
export type PasskeyStepUpAssertion = {
  ceremony_id: Uuid;
  credential: {
    [key: string]: unknown;
  };
};

/**
 * A passkey credential summary for the self-service management surface. It deliberately carries no secret material (no passkey blob, public key, or raw credential id) — only the opaque row id and the registration / last-use timestamps.
 */
export type PasskeySummary = {
  id: Uuid;
  created_at: Timestamp;
  last_used_at: string | null;
};

export type PayrollClosePreflight = {
  checks: Array<PayrollPreflightCheck>;
  can_close: boolean;
};

export type PayrollDisbursement = {
  id: Uuid;
  run_id: Uuid;
  scheduled_at: Timestamp;
  status: "SCHEDULED" | "SUBMITTED_TO_BANK" | "PAID" | "FAILED";
  attested_by: string | null;
  attested_at: string | null;
  reason: string | null;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type PayrollException = {
  id: Uuid;
  run_id: Uuid;
  line_id: string | null;
  employee_id: string | null;
  employee_display_name: string;
  kind: "OVERTIME_ALLOWANCE" | "RETRO_ADJUSTMENT" | "ABSENCE_DEDUCTION" | "PRORATION" | "ACCOUNT_VERIFICATION";
  severity: "info" | "warn" | "danger";
  amount_delta_won: number | null;
  summary_ko: string;
  detail: Record<string, unknown>;
  linked_refs: Array<PayrollLinkedRef>;
  status: "OPEN" | "CONFIRMED" | "HELD";
  resolved_by: string | null;
  resolved_at: string | null;
  resolved_reason: string | null;
  carried_from_run_id: string | null;
  created_at: Timestamp;
};

export type PayrollExceptionPage = {
  items: Array<PayrollException>;
  total: number;
  open: number;
  limit: number;
  offset: number;
};

export type PayrollLineSummary = {
  id: Uuid;
  employee_id?: string | null;
  employee_display_name: string;
  employee_company: string;
  work_days?: number | null;
  regular_hours?: number | null;
  overtime_hours?: number | null;
  night_hours?: number | null;
  holiday_hours?: number | null;
  leave_used?: number | null;
  leave_remaining?: number | null;
  gross_pay_source_present: boolean;
  net_pay_source_present: boolean;
  nts_tax_row_status: "REQUIRED_NOT_SUPPLIED" | "SUPPLIED_UNVERIFIED" | "VERIFIED_SOURCE_ROW";
  calculation_status: "BLOCKED_LEGAL_GATE" | "READY_FOR_REVIEW" | "APPROVED" | "ISSUED" | "VOID";
  blockers: Array<unknown>;
};

export type PayrollLinkedRef = {
  kind: string;
  code: string;
  id?: string | null;
};

export type PayrollPayslipDeliveryItem = {
  line_id: Uuid;
  employee_id: Uuid;
  inbox_doc_id: Uuid;
  issued_at: Timestamp;
  acknowledged_at: string | null;
};

export type PayrollPayslipDeliverySummary = {
  run_id: Uuid;
  issued: number;
  acknowledged: number;
  items: Array<PayrollPayslipDeliveryItem>;
  total: number;
  limit: number;
  offset: number;
};

/**
 * The payslip draft. The first nine properties are always present; the computed half is absent when a blocker stopped the draft before the arithmetic (no contract wage in force, or an hourly contract).
 *
 */
export type PayrollPayslipDraft = {
  employee_id: Uuid;
  employee_name: string;
  period: {
    start: string;
    end: string;
  };
  pay_date: string;
  contract: {
    id: Uuid;
    effective_from: string;
    wage_kind: "MONTHLY" | "HOURLY";
    amount_won: number;
    monthly_standard_hours: number;
    source_note: string;
  } | null;
  attendance: {
    period_start: string;
    period_end: string;
    worked_days: number;
    clock_in_events: number;
    clock_out_events: number;
  };
  issuable: boolean;
  blockers: Array<string>;
  statutory_citations: Array<PayrollStatutoryCitation>;
  earnings?: Array<{
    code: string;
    label_ko: string;
    amount_won: number;
    note_ko: string;
  }>;
  gross_won?: number;
  deductions?: Array<PayrollPayslipDraftDeduction>;
  not_computed?: Array<{
    code: string;
    label_ko: string;
    reason_ko: string;
    instrument: PayrollStatutoryInstrument;
  }>;
  minimum_wage_check?: {
    hourly_floor_won: number;
    monthly_209h_floor_won: number;
    monthly_standard_hours: number | null;
    effective_hourly_won: number | null;
    passes: boolean | null;
    instrument: PayrollStatutoryInstrument;
  };
  total_employee_insurance_won?: number | null;
  remainder_after_insurance_won?: number | null;
  net_pay_won?: null;
  net_pay_unavailable_reason_ko?: string;
  compliance_notice_ko?: string;
};

/**
 * One 공제 component, carrying its own basis, rate and instrument so the payslip is recomputable under 근로기준법 제42조. An amount this engine did not compute is null, never 0.
 *
 */
export type PayrollPayslipDraftDeduction = {
  code: "NationalPension" | "HealthInsurance" | "LongTermCare" | "EmploymentUnemployment" | "IndustrialAccident";
  label_ko: string;
  basis_kind: "MonthlyStandardIncome" | "MonthlyRemuneration" | "HealthInsurancePremium" | "IndustryTariff";
  basis_won: number | null;
  rate_num: number | null;
  rate_den: number | null;
  total_won: number | null;
  employee_won: number | null;
  employer_only: boolean;
  blocked_by: string | null;
  total_rounding: string;
  employee_rounding: string;
  instrument: PayrollStatutoryInstrument;
  share_instrument: PayrollStatutoryInstrument | null;
  provenance_ko: string;
};

export type PayrollPreflightCheck = {
  key: "attendance_material" | "period_lock" | "pending_leave";
  label_ko: string;
  ok: boolean;
  warn: boolean;
  note: string | null;
  blocking_refs: Array<Uuid>;
};

export type PayrollRunCalcSummary = {
  version: number;
  calculated_at: Timestamp;
  calculated_lines: number;
  blocked_lines: number;
  payable: boolean;
  kernel_rate_table: string;
  total_net_won: number | null;
};

export type PayrollRunDetail = {
  run: PayrollRunSummary;
  legal_basis: Record<string, unknown>;
  source_summary: Record<string, unknown>;
  lines: Array<PayrollLineSummary>;
  lines_total: number;
  lines_limit: number;
  lines_offset: number;
  exceptions_open: number;
  exceptions_total: number;
  calculation: PayrollRunCalcSummary | null;
  disbursement: PayrollDisbursement | null;
  payslip_delivery: PayrollPayslipDeliverySummary | null;
};

export type PayrollRunPage = {
  items: Array<PayrollRunSummary>;
  total: number;
  limit: number;
  offset: number;
};

export type PayrollRunSummary = {
  id: Uuid;
  period_start: string;
  period_end: string;
  source_label: string;
  status: "STAGED" | "BLOCKED_LEGAL_GATE" | "READY_FOR_REVIEW" | "ATTENDANCE_CLOSED" | "CALCULATING" | "CALCULATED" | "SUBMITTED" | "REJECTED" | "APPROVED" | "DISBURSEMENT_SCHEDULED" | "PAID" | "ISSUED" | "VOID";
  calculation_enabled: boolean;
  created_by?: string | null;
  approved_by?: string | null;
  approved_at?: string | null;
  close_receipt?: Record<string, unknown> | null;
  submitted_by?: string | null;
  submitted_at?: string | null;
  decided_by?: string | null;
  decided_at?: string | null;
  decision_reason?: string | null;
  approval_ref?: string | null;
  created_at: Timestamp;
  updated_at: Timestamp;
};

/**
 * One row of `payroll_statutory_rates` in force on the pay date, verbatim.
 */
export type PayrollStatutoryCitation = {
  code: string;
  effective_from: string;
  effective_to_exclusive: string | null;
  rate_num: number | null;
  rate_den: number | null;
  floor_won: number | null;
  cap_won: number | null;
  basis: string;
  bearer: string;
  instrument_ko: string;
  article_ko: string;
  promulgation_ko: string;
  enforced_on: string;
  source_url: string;
  retrieved_on: string;
  provenance_ko: string;
};

/**
 * The document that actually sets a number. The version anchor is 공포번호 (법령) or 발령번호 (고시) together with `enforced_on` — never a `flSeq` file handle, three of which serve byte-identical 별표 2 content.
 *
 */
export type PayrollStatutoryInstrument = {
  name_ko: string;
  article_ko: string;
  promulgation_ko: string;
  enforced_on: string;
  url: string;
  retrieved_on: string;
};

export type Period = {
  start: Timestamp;
  end: Timestamp;
};

export type PeriodLock = {
  id: string;
  domain: "payroll" | "accounting";
  periodStart: string;
  periodEnd: string;
  reason: string;
  lockedBy?: string;
  lockedAt: string;
  unlockedBy?: string;
  unlockedAt?: string;
  unlockReason?: string;
};

export type PeriodLockList = {
  items: Array<PeriodLock>;
};

export type PlatformAccountStatus = "ACTIVE" | "PENDING_SETUP" | "DEACTIVATED";

export type PlatformExitResponse = {
  ended: boolean;
};

export type PlatformGroup = {
  id: Uuid;
  slug: string;
  name: string;
  status: PlatformOrgStatus;
  member_count: number;
  members: Array<PlatformGroupMember>;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type PlatformGroupAccount = {
  user_id: Uuid;
  display_name: string;
  phone: string | null;
  tenant_roles: Array<PlatformTenantRole>;
  is_active: boolean;
  has_passkey: boolean;
  account_status: PlatformAccountStatus;
  org_id: Uuid;
  org_slug: string;
  org_name: string;
  group_roles: Array<PlatformGroupRole>;
  created_at: Timestamp;
};

export type PlatformGroupMember = {
  id: Uuid;
  slug: string;
  name: string;
  status: PlatformOrgStatus;
};

export type PlatformGroupRole = "GROUP_ADMIN" | "GROUP_VIEWER" | "GROUP_FINANCE";

export type PlatformOpsResponse = {
  tenants: Array<PlatformTenantHealth>;
};

export type PlatformOrg = {
  id: Uuid;
  slug: string;
  name: string;
  status: PlatformOrgStatus;
  group_id: string | null;
  group_slug: string | null;
  group_name: string | null;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type PlatformOrgOnboardingResponse = {
  org: PlatformOrg;
  admin_user_id: Uuid;
  otp: string;
  admin_otp_expires_at: Timestamp;
};

export type PlatformOrgStatus = "ACTIVE" | "SUSPENDED" | "ARCHIVED";

export type PlatformTenantContextStartRequest = {
  org_id: Uuid;
};

export type PlatformTenantContextStartResponse = {
  access_token: string;
  token_type: "Bearer";
  acting_org_id: Uuid;
  acting_org_name: string;
  acting_role: "SUPER_ADMIN";
  expires_at: Timestamp;
};

export type PlatformTenantHealth = {
  id: Uuid;
  slug: string;
  name: string;
  status: PlatformOrgStatus;
  group_id: string | null;
  group_slug: string | null;
  group_name: string | null;
  user_count: number;
  active_user_count: number;
  active_work_orders: number;
  open_work_orders: number;
  last_activity_at: string | null;
  route_adoption: Array<RouteAdoptionMetric>;
  zero_legacy_release_cycles: number;
};

export type PlatformTenantRole = "SUPER_ADMIN" | "ADMIN" | "MECHANIC" | "RECEPTIONIST" | "EXECUTIVE" | "MEMBER";

export type PlatformViewAsStartRequest = {
  org_id: Uuid;
  role: PlatformTenantRole;
};

export type PlatformViewAsStartResponse = {
  access_token: string;
  token_type: "Bearer";
  acting_org_id: Uuid;
  acting_org_name: string;
  acting_role: PlatformTenantRole;
  expires_at: Timestamp;
};

export type PolicyAssignmentPreviewResponse = {
  user_id: Uuid;
  preview_receipt_id: Uuid;
  preview_receipt_expires_at: Timestamp;
  effective: boolean;
  system_roles: Array<string>;
  current_system_roles: Array<string>;
  requested_system_roles: Array<string>;
  current_branch_ids: Array<Uuid>;
  requested_branch_ids: Array<Uuid>;
  current_role_ids: Array<Uuid>;
  requested_role_ids: Array<Uuid>;
  delta: PolicyRoleAssignmentDeltaResponse;
  custom_roles: Array<PolicyRoleImpactResponse>;
  feature_grants: Array<PolicyFeatureGrantPreviewResponse>;
  warnings: Array<string>;
};

/**
 * Append-only Policy Studio audit evidence. Console UI should render human-safe summaries and avoid exposing raw target identifiers by default; the identifiers remain in the API for export and traceability.
 */
export type PolicyAuditEventResponse = {
  id: Uuid;
  actor?: Uuid | null;
  action: string;
  target_type: "policy_role" | "policy_role_assignment";
  target_id: string;
  before_snapshot?: {
    [key: string]: unknown;
  } | null;
  after_snapshot?: {
    [key: string]: unknown;
  } | null;
  trace_id: string;
  span_id: string;
  occurred_at: Timestamp;
};

export type PolicyAuthorizeRequest = {
  request: PolicySimRequest;
  object_type_id?: Uuid;
  property_def_id?: Uuid;
};

/**
 * ABAC/PBAC condition metadata attached to a custom policy role. These constraints are persisted for audit and preview. Runtime evaluation supports data-backed branch narrowing and team matching; unsupported ABAC/PBAC attributes remain visible but fail closed until implemented.
 */
export type PolicyConditionResponse = {
  condition_key: string;
  attribute: "group" | "tenant" | "organization" | "org" | "department" | "team" | "position" | "employment_status" | "assignment" | "location" | "site" | "branch" | "device_posture" | "purpose" | "action" | "resource" | "sensitive_action";
  operator: "equals" | "not_equals" | "in";
  values: Array<string>;
};

export type PolicyCreateDraftRequest = {
  draft_key: string;
  title: string;
  author_note?: string;
  blocks: PolicyNoCodeBlocks;
};

export type PolicyDefaultPermissionResponse = {
  role_key: string;
  permission_level: "deny" | "request_only" | "limited" | "allow";
};

export type PolicyFeatureGrantPreviewResponse = {
  feature_key: string;
  permission_level: "request_only" | "limited" | "allow";
  source_type: "system_role" | "custom_role";
  source_key: string;
  source_label: string;
};

export type PolicyFeatureResponse = {
  feature_key: string;
  elevated: boolean;
  default_permissions: Array<PolicyDefaultPermissionResponse>;
};

/**
 * One authored policy - one effect, one action, one resource type, and a set of AND-ed conditions.
 */
export type PolicyNoCodeBlocks = {
  effect: "permit" | "forbid";
  action: string;
  resource_type: string;
  conditions?: Array<PolicyNoCodeCondition>;
};

export type PolicyNoCodeCondition = {
  attr: string;
  op: "eq" | "ne" | "contains";
  value: ConditionValue;
};

export type PolicyPermissionResponse = {
  feature_key: string;
  permission_level: "deny" | "request_only" | "limited" | "allow";
};

export type PolicyReviewRequest = {
  decision: "approve" | "reject";
  note?: string;
};

export type PolicyRoleAssignmentDeltaResponse = {
  added_role_ids: Array<Uuid>;
  removed_role_ids: Array<Uuid>;
  unchanged_role_ids: Array<Uuid>;
};

export type PolicyRoleAssignmentResponse = {
  user_id: Uuid;
  role_id: Uuid;
  role_key: string;
  display_name: string;
  status: "DRAFT" | "ACTIVE" | "RETIRED";
  assigned_by: string | null;
  created_at: Timestamp;
};

export type PolicyRoleCatalogResponse = {
  policy_version: PolicyVersionResponse;
  system_roles: Array<SystemPolicyRoleResponse>;
  custom_roles: Array<PolicyRoleResponse>;
};

/**
 * Per-custom-role impact row for assignment preview. Runtime fields are computed by the same fail-closed evaluator used for the top-level preview: only ACTIVE roles whose supported conditions (branch and team) match the target user's live attributes/scope and whose permissions are supported can expose runtime grants.
 */
export type PolicyRoleImpactResponse = {
  role_id: Uuid;
  role_key: string;
  display_name: string;
  status: "DRAFT" | "ACTIVE" | "RETIRED";
  runtime_effective: boolean;
  runtime_warnings: Array<"custom_role_status_not_active" | "custom_role_condition_unsupported_by_runtime_evaluator" | "custom_role_condition_invalid_branch_value" | "custom_role_condition_outside_target_branch_scope" | "custom_role_condition_outside_target_attributes" | "custom_role_no_runtime_allowed_permissions">;
  conditions: Array<PolicyConditionResponse>;
};

export type PolicyRoleResponse = {
  id: Uuid;
  role_key: string;
  display_name: string;
  description: string | null;
  status: "DRAFT" | "ACTIVE" | "RETIRED";
  is_system: boolean;
  permissions: Array<PolicyPermissionResponse>;
  conditions: Array<PolicyConditionResponse>;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type PolicyRoleStatusPreviewRequest = {
  status: "DRAFT" | "ACTIVE" | "RETIRED";
};

export type PolicyRoleStatusPreviewResponse = {
  role_id: Uuid;
  role_key: string;
  display_name: string;
  current_status: "DRAFT" | "ACTIVE" | "RETIRED";
  requested_status: "DRAFT" | "ACTIVE" | "RETIRED";
  permission_count: number;
  condition_count: number;
  planned_assignment_count: number;
  requires_passkey_step_up: boolean;
  effective_runtime_change: boolean;
  warnings: Array<"passkey_step_up_required" | "no_status_change" | "assigned_users_may_gain_or_lose_runtime_permissions" | "rollback_disables_assigned_custom_role_runtime_grants" | "retire_disables_assigned_custom_role_runtime_grants" | "publish_enables_assigned_custom_role_runtime_grants">;
};

export type PolicyRoleTemplateResponse = {
  template_key: string;
  role_key: string;
  display_name: string;
  category: string;
  description: string;
  permissions: Array<PolicyPermissionResponse>;
};

export type PolicySimRequest = {
  subject: PolicySimSubject;
  action: string;
  resource: PolicySimResource;
  purpose?: string;
  field?: string;
};

export type PolicySimResource = {
  org: Uuid;
  resource_type: string;
  resource_id?: string;
  owner?: string;
  branch?: string;
  legal_hold?: boolean;
};

export type PolicySimSubject = {
  org: Uuid;
  user_id: string;
  roles?: Array<string>;
  clearance_keys?: Array<string>;
};

export type PolicySimulateRequest = {
  request: PolicySimRequest;
  include_draft_id?: Uuid;
};

export type PolicyUpdateDraftRequest = {
  title?: string;
  author_note?: string;
  blocks: PolicyNoCodeBlocks;
};

/**
 * Monotonic tenant policy revision. Version 0 means no custom policy write has occurred yet; every role or custom-role assignment write bumps the stored version for future effective-policy cache invalidation.
 */
export type PolicyVersionResponse = {
  version: number;
  updated_at?: string | null;
};

export type PollAnonymity = "NAMED" | "ANONYMOUS";

export type PollListResponse = {
  items: Array<PollResponse>;
};

export type PollMyVote = {
  submitted: boolean;
  selected_option_ids?: Array<Uuid> | null;
};

export type PollOptionResponse = {
  id: Uuid;
  label: string;
  position: number;
  vote_count: number;
};

export type PollResponse = {
  id: Uuid;
  target_scope_type: CollaborationScopeType;
  target_scope_ref?: string | null;
  title: string;
  question: string;
  status: PollStatus;
  anonymity: PollAnonymity;
  allow_multiple: boolean;
  closes_at?: Timestamp | null;
  object_type?: string | null;
  object_id?: Uuid | null;
  options: Array<PollOptionResponse>;
  vote_count: number;
  my_vote: PollMyVote;
  created_by?: Uuid | null;
  created_at: Timestamp;
  updated_at: Timestamp;
  policy: CollaborationScopePolicy;
};

export type PollStatus = "DRAFT" | "OPEN" | "CLOSED" | "ARCHIVED";

export type PostFinalizationRejectionDocument = {
  id: Uuid;
  original_run_id: Uuid;
  reason: string;
  created_by: Uuid;
};

export type PostFinalizationRejectionRequest = {
  reason: string;
  idempotency_key: string;
};

export type PostFinalizationRejectionResponse = {
  compensation: PostFinalizationRejectionDocument;
  run: FinalizedWorkflowRun;
};

export type PreflightOutcome = {
  dispatch: "instance_revision" | "projected_usecase";
  dispatch_target: string | null;
  config: GateChainConfig;
  gates: GateChainOutcome;
  criteria_ok: boolean;
  criteria_error?: string;
  would_execute: boolean;
};

export type PrepareExpenditureRequest = {
  expenditure_no: string;
  step_up: PasskeyStepUpAssertion;
};

export type PresignedUpload = {
  method: "PUT";
  url: string;
  headers: Array<Array<unknown>>;
  expires_in_secs: number;
};

export type PreviewScheduleRequest = {
  cron_expr: string;
  timezone?: string;
};

export type PreviewScheduleResponse = {
  cron_expr: string;
  timezone: string;
  fire_times: Array<string>;
};

export type PriorityLevel = "P1" | "P2" | "P3" | "OUTSOURCE" | "UNSET";

/**
 * Required initial-login Korean privacy collection/use and service-terms acknowledgement. These required agreements are explicit booleans so the client cannot bundle them into one generic "agree all" flag. Optional marketing consent and GPS/location consent are not collected by this request.
 */
export type PrivacyConsentAcceptRequest = {
  policy_version: string;
  privacy_collection: boolean;
  terms_of_service: boolean;
};

/**
 * Current required first-login privacy/terms consent status.
 */
export type PrivacyConsentStatusResponse = {
  policy_version: string;
  accepted: boolean;
  accepted_at?: string | null;
};

export type ProcessingStatus = "PROCESSING" | "READY" | "FAILED";

export type ProductionCapacityIngress = {
  kind: "capacity";
  id: Uuid;
  site_id: Uuid;
  capacity_date: string;
  available_quantity: number;
  source_id: string;
  source_version: string;
};

export type ProductionCapacitySlot = {
  id: Uuid;
  branch_id: Uuid;
  site_id: Uuid;
  capacity_date: string;
  available_quantity: number;
  reserved_quantity: number;
  version: number;
  source_ref: string;
  evaluated_at: string;
};

export type ProductionDemandIngress = {
  kind: "demand";
  id: Uuid;
  inquiry_id: Uuid;
  product_code: string;
  quantity: number;
  due_at: string;
  source_id: string;
  source_version: string;
};

export type ProductionMaterialIngress = {
  kind: "material";
  material_item_id: Uuid;
  quantity_on_hand_milli: number;
  safety_stock_milli: number;
  source_id: string;
  source_version: string;
};

export type ProductionOperation = {
  id: Uuid;
  sequence: number;
  status: "PENDING" | "RELEASED" | "RECORDED";
  output_quantity: number;
  scrap_quantity: number;
  downtime_minutes: number;
  quality_evidence_ref?: string | null;
  quality_passed?: boolean | null;
  version: number;
};

export type ProductionPlan = {
  id: Uuid;
  branch_id: Uuid;
  customer_demand_id: Uuid;
  product_code: string;
  quantity: number;
  status: "DRAFT" | "RELEASED";
  version: number;
  first_operation_id: Uuid;
  created_at: string;
  due_at: string;
  plan_digest: string;
};

export type ProductionPlanDetail = ProductionPlan & {
  checks: {
    [key: string]: unknown;
  };
  events: Array<{
    id: Uuid;
    event_type: string;
    actor_id: Uuid;
    payload: {
      [key: string]: unknown;
    };
    occurred_at: string;
  }>;
  operation: ProductionOperation;
};

export type ProductionSourceIngress = ProductionDemandIngress | ProductionCapacityIngress | ProductionMaterialIngress;

export type ProductionSourceIngressReceipt = {
  kind: "demand" | "capacity" | "material";
  id: Uuid;
  source_version: string;
};

export type ProductionSourceSystemCredential = {
  id: Uuid;
  source_system: string;
  enabled: boolean;
  credential_generation: number;
  secret: string;
};

export type ProductionSourceSystemGenerationRequest = {
  expected_generation: number;
};

export type ProductionSourceSystemReceipt = {
  id: Uuid;
  enabled: boolean;
  credential_generation: number;
};

export type ProjectionAssumptions = {
  ewma_volatility: number;
  student_t_nu: number;
  drift: number;
  simulations: number;
  seed: number;
};

export type ProjectionRequest = {
  series: Array<number>;
  horizon: number;
  kind: "money" | "percent";
};

export type ProjectionResult = {
  point_estimate: number;
  ci95_low: number;
  ci95_high: number;
  cvar95: number;
  assumptions: ProjectionAssumptions;
};

export type PropertyDefSummary = {
  id: string;
  key: string;
  title: string;
  field_type: string;
  field_kind: string;
  config: {
    [key: string]: unknown;
  };
  backing_column?: string | null;
  required: boolean;
  in_property_policy: boolean;
};

export type PublishRecruitPostingRequest = {
  attest_exposure_scope: boolean;
  expected_updated_at: Timestamp;
};

export type PurchaseAttachmentDownloadResponse = {
  url: string;
};

export type PurchaseAttachmentPresignRequest = {
  branch_id: Uuid;
  file_name: string;
  content_type: string;
  size_bytes: number;
  checksum_sha256?: string | null;
  role?: string | null;
};

export type PurchaseAttachmentPresignResponse = {
  attachment_id: Uuid;
  upload: PresignedUpload;
  file_name: string;
  content_type: string;
  size_bytes: number;
  role: string;
  upload_state: string;
};

export type PurchaseAttachmentSummary = {
  id: Uuid;
  file_name: string;
  content_type: string;
  size_bytes: number;
  role: string;
  download_url: string;
  created_at: Timestamp;
};

export type PurchaseAttachmentUploadRecord = {
  id: Uuid;
  branch_id: Uuid;
  file_name: string;
  content_type: string;
  size_bytes: number;
  role: string;
  upload_state: string;
  created_at: Timestamp;
};

export type PurchaseFeaturePreferences = {
  feature_key: string;
  schema_version: number;
  preferences: {
    [key: string]: unknown;
  };
};

export type PurchasePolicySummary = {
  equipment_required: boolean;
  statement_evidence_required: boolean;
  price_anomaly: boolean;
  quote_update_required: boolean;
  submit_blocked: boolean;
  messages: Array<string>;
};

export type PurchaseRequestLineInput = {
  item: string;
  quantity: number;
  unit_supply_price_won: number;
  vat_won?: number | null;
};

export type PurchaseRequestLineSummary = {
  id: Uuid;
  line_no: number;
  item: string;
  quantity: number;
  unit_supply_price_won: number;
  vat_won: number;
  vat_overridden: boolean;
  line_total_won: number;
};

/**
 * Stable offset page for the branch-scoped purchase-request queue.
 */
export type PurchaseRequestPage = {
  items: Array<PurchaseRequestSummary>;
  limit: number;
  offset: number;
  total: number;
};

export type PurchaseRequestSummary = {
  id: Uuid;
  branch_id: Uuid;
  equipment_id: string | null;
  work_order_id: string | null;
  statement_evidence_id: string | null;
  purchase_type: PurchaseType;
  vendor_name: string;
  amount_won: number;
  status: PurchaseStatus;
  requester: PurchaseRequesterSummary;
  lines: Array<PurchaseRequestLineSummary>;
  quote_attachments: Array<PurchaseAttachmentSummary>;
  policy: PurchasePolicySummary;
  expenditure_no: string | null;
  rejection_memo: string | null;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type PurchaseRequesterSummary = {
  user_id: Uuid;
  display_name: string;
};

export type PurchaseStatus = "STATEMENT_ATTACHED" | "REQUEST_SUBMITTED" | "ADMIN_APPROVED" | "EXECUTIVE_PENDING" | "READY_TO_EXECUTE" | "EXECUTED" | "REJECTED";

export type PurchaseType = "REGULAR" | "ONE_OFF" | "OTHER" | "LEGACY_MANUAL";

export type QuoteLine = {
  code: string;
  label: string;
  amount: number;
};

export type RaiseAttendanceExceptionRequest = {
  kind: "LATE" | "NO_SHOW" | "UNAPPROVED_OVERTIME" | "EARLY_LEAVE";
  employee_id: Uuid;
  branch_id?: Uuid | null;
  work_date: string;
  detail: string;
  evidence?: Array<{
    name: string;
    size?: string | null;
  }>;
};

export type RecordInventoryReceiptRequest = {
  quantityReceivedMilli: number;
  sourceRef?: string | null;
  memo?: string | null;
  idempotencyKey: string;
};

export type RecordProductionOperation = {
  expected_version: number;
  idempotency_key: string;
  output_quantity: number;
  scrap_quantity: number;
  downtime_minutes: number;
  quality_evidence_ref: string;
  quality_passed: boolean;
  note: string;
};

export type RecordRecruitOfferReplyRequest = {
  decision: "ACCEPTED" | "DECLINED";
};

export type RecordSupportTicketAcceptanceRequest = {
  kind: SupportTicketAcceptanceKind;
  channel: SupportTicketAcceptanceChannel;
  accepted_by: string;
  note?: string | null;
};

export type RecruitAmountPeriod = "MONTHLY" | "DAILY";

export type RecruitApplicant = {
  id: Uuid;
  posting_id: Uuid;
  applicant_no: string;
  name: string;
  profile_lines: Array<string>;
  source_document: string | null;
  stage: RecruitApplicantStage;
  hold: boolean;
  doc_requested: boolean;
  rejected_at: string | null;
  reject_reason: RecruitRejectReason | null;
  reject_note: string | null;
  assessment: RecruitAssessment | null;
  hired_employee_id: Uuid | null;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type RecruitApplicantDetailResponse = {
  applicant: RecruitApplicant;
  offers: Array<RecruitOffer>;
  events: Array<RecruitStageEvent>;
};

export type RecruitApplicantStage = "APPLIED" | "SCREENING" | "INTERVIEW" | "OFFER" | "HIRED";

/**
 * Non-PII pipeline projection for posting detail. Profile lines, source document, reject note, and the assessment signature are served only by the audited applicant detail read.
 */
export type RecruitApplicantSummary = {
  id: Uuid;
  posting_id: Uuid;
  applicant_no: string;
  name: string;
  stage: RecruitApplicantStage;
  hold: boolean;
  doc_requested: boolean;
  rejected_at: string | null;
  reject_reason: RecruitRejectReason | null;
  assessed: boolean;
  hired_employee_id: Uuid | null;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type RecruitAssessment = {
  score: RecruitAssessmentScore;
  by: Uuid | null;
  at: string | null;
};

export type RecruitAssessmentScore = "SUITABLE" | "NEUTRAL" | "UNSUITABLE";

export type RecruitEmploymentType = "REGULAR" | "RESIDENT_SHIFT" | "PART_TIME" | "POOL_DAILY";

export type RecruitHireConflictResponse = ErrorBody & {
  employee_id?: Uuid;
};

export type RecruitOffer = {
  id: Uuid;
  applicant_id: Uuid;
  version: number;
  amount: string;
  amount_period: RecruitAmountPeriod;
  currency: "KRW";
  reply_deadline: Date;
  status: RecruitOfferStatus;
  withdraw_reason: string | null;
  extended_by: Uuid;
  extended_at: Timestamp;
  resolved_at: string | null;
};

export type RecruitOfferStatus = "EXTENDED" | "SUPERSEDED" | "WITHDRAWN" | "ACCEPTED" | "DECLINED";

export type RecruitPosting = {
  id: Uuid;
  posting_no: string;
  role_title: string;
  company: string;
  worksite: string;
  employment_type: RecruitEmploymentType;
  scope: RecruitPostingScope;
  headcount: number;
  hired_count: number;
  deadline: string | null;
  requirements: Array<string>;
  position_ref: string | null;
  status: RecruitPostingStatus;
  published_at: string | null;
  closed_at: string | null;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type RecruitPostingDetailResponse = {
  posting: RecruitPosting;
  applicants: Array<RecruitApplicantSummary>;
};

export type RecruitPostingListResponse = {
  items: Array<RecruitPostingSummary>;
};

export type RecruitPostingPreflightResponse = {
  checks: Array<RecruitPreflightCheck>;
  publishable: boolean;
};

export type RecruitPostingScope = "INTERNAL" | "EXTERNAL";

export type RecruitPostingStatus = "DRAFT" | "PUBLISHED" | "CLOSED";

export type RecruitPostingSummary = RecruitPosting & {
  stage_counts: RecruitStageCounts;
};

export type RecruitPreflightCheck = {
  key: "role_defined" | "quota_defined" | "no_duplicate_open" | "exposure_attested";
  ok: boolean;
  note: string;
};

export type RecruitPublishFailedResponse = ErrorBody & {
  checks?: Array<RecruitPreflightCheck>;
  publishable?: boolean;
};

export type RecruitRejectReason = "CAREER_SHORTFALL" | "ROLE_MISMATCH" | "COMP_MISMATCH" | "ACCEPTED_ELSEWHERE" | "OTHER";

/**
 * Non-rejected applicant counts per pipeline stage; HIRED is not counted.
 */
export type RecruitStageCounts = {
  applied: number;
  screening: number;
  interview: number;
  offer: number;
};

export type RecruitStageEvent = {
  id: Uuid;
  action: RecruitStageEventAction;
  from_stage: RecruitApplicantStage | null;
  to_stage: RecruitApplicantStage | null;
  reason: string | null;
  actor: Uuid;
  occurred_at: Timestamp;
};

export type RecruitStageEventAction = "APPLY" | "ADVANCE" | "ASSESS" | "HOLD" | "UNHOLD" | "REQUEST_DOCUMENTS" | "OFFER_EXTEND" | "OFFER_ADJUST" | "OFFER_WITHDRAW" | "OFFER_REPLY" | "REJECT" | "REINSTATE" | "HIRE";

export type RecruitTalentPoolEntry = {
  applicant_id: Uuid;
  applicant_no: string;
  name: string;
  role_title: string;
  reason: RecruitRejectReason | null;
  rejected_at: Timestamp;
};

export type RecruitTalentPoolListResponse = {
  items: Array<RecruitTalentPoolEntry>;
};

/**
 * Refresh/logout request body. `refresh_token` is OPTIONAL because the web transport carries the token in the HttpOnly `console_refresh` cookie (sent automatically by the browser) and the body is empty; mobile clients send the token here.
 */
export type RefreshTokenRequest = {
  refresh_token?: string | null;
};

export type RegionSummary = {
  id: Uuid;
  name: string;
  deactivated_at: string | null;
  created_at: Timestamp;
};

export type RegisterProductionSourceSystem = {
  branch_id: Uuid;
  source_system: string;
};

export type RegistryImportReport = {
  input_rows: number;
  equipment_count: number;
  added: number;
  updated: number;
  unchanged: number;
  orphaned: number;
  errors: Array<RegistryRowError>;
};

export type RegistryRowError = {
  sheet: string;
  row: number;
  message: string;
};

export type RegulationImpact = {
  id: Uuid;
  code: string;
  title: string;
  jurisdiction: string;
  regulator?: string | null;
  citation: string;
  source_url?: string | null;
  impact_area: string;
  impact_summary: string;
  risk_level: ComplianceRiskLevel;
  status: RegulationImpactStatus;
  effective_from?: string | null;
  effective_to?: string | null;
  review_due_on?: string | null;
  owner_user_id?: string | null;
  metadata: {
    [key: string]: unknown;
  };
  created_by: Uuid;
  updated_by: Uuid;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type RegulationImpactPage = {
  items: Array<RegulationImpact>;
  limit: number;
  offset: number;
  total: number;
};

export type RegulationImpactStatus = "DRAFT" | "ACTIVE" | "SUPERSEDED" | "ARCHIVED";

export type RegulationLinkRequest = {
  regulation_impact_id: Uuid;
  relationship: ObligationRegulationRelationship;
  rationale?: string | null;
};

export type RejectPurchaseRequest = {
  memo: string;
  step_up: PasskeyStepUpAssertion;
};

export type RejectRecruitApplicantRequest = {
  reason: RecruitRejectReason;
  note?: string | null;
};

export type RejectWorkOrderRequest = {
  memo: string;
};

export type ReleaseProductionPlan = {
  expected_version: number;
  approval_ref: Uuid;
  idempotency_key: string;
};

export type RentalQuoteSummary = {
  id: Uuid;
  branch_id: Uuid;
  equipment_id: Uuid;
  acquisition_value: number;
  current_residual_value: number;
  effective_residual_value: number;
  residual_was_floored: boolean;
  cumulative_repair_cost: number;
  monthly_total: number;
  lines: Array<QuoteLine>;
  created_at: Timestamp;
};

export type ReplaceEvaluationGoalsRequest = {
  goals: Array<EvaluationGoalInput>;
};

/**
 * Policy assignment/account-scope preview and custom-role assignment replacement. The preview endpoint accepts `role_ids` plus optional `system_roles` and `branch_ids` for account/person mutation previews. The mutating custom-role replacement endpoint also requires `preview_acknowledged: true`, the server-issued `preview_receipt_id` from a current impact preview for the same actor/user/role set, plus a fresh passkey `step_up` assertion server-side so the audit trail binds the sensitive policy write to the human actor. ACTIVE custom-role assignments become runtime-effective on the next request when their grants and conditions pass the fail-closed runtime evaluator.
 */
export type ReplacePolicyRoleAssignmentsRequest = {
  role_ids?: Array<Uuid>;
  system_roles?: Array<string>;
  branch_ids?: Array<Uuid>;
  preview_acknowledged?: boolean;
  preview_receipt_id?: Uuid;
  step_up?: PasskeyStepUpAssertion;
};

export type ResolveAttendanceExceptionRequest = {
  action: string;
  reason: string;
  linked_work_ref?: string | null;
  ot_hours?: number | null;
};

export type ResolvePayrollExceptionRequest = {
  action: "CONFIRM" | "HOLD";
  reason?: string;
};

export type ResolvedInstance = {
  id: string;
  "type": string;
  title: string;
};

export type RespondP1DispatchRequest = {
  response: DispatchResponseKind;
};

export type RestartPurchaseRequest = {
  statement_evidence_id?: string | null;
  amount_won?: number | null;
  lines: Array<PurchaseRequestLineInput>;
  quote_attachment_ids: Array<Uuid>;
  memo: string;
};

export type ReturnSubstituteRequest = {
  return_note?: string | null;
};

export type ReverseVoucherRequest = {
  memo?: string;
};

export type ReviewDailyPlanRequest = {
  decision: DailyPlanStatus;
  memo?: string;
};

export type ReviewSettlementRequest = {
  decision: "APPROVED" | "RETURNED";
  comment?: string;
};

export type ReviewTargetChangeRequest = {
  decision: TargetChangeDecision;
  memo?: string;
};

export type RevisionSummary = {
  id: string;
  instance_id: string;
  version: number;
  attributes: {
    [key: string]: unknown;
  };
  valid_from: string;
  valid_to?: string | null;
  action_type_id?: string | null;
  actor?: string | null;
  reason?: string | null;
  prev_hash: string;
  row_hash: string;
};

export type RollbackWorkflowDefinitionRequest = {
  target_version: number;
  step_up?: PasskeyStepUpAssertion;
};

export type RouteAdoptionMetric = {
  release_cycle: string;
  console_route_events: number;
  legacy_route_events: number;
  rum_error_events: number;
  rum_perf_p95_ms: number | null;
  last_event_at: string;
};

export type SalesListingPage = {
  items: Array<SalesListingView>;
  limit: number;
  offset: number;
  total: number;
};

/**
 * A sales listing as read by the storefront or the admin console.
 */
export type SalesListingView = {
  id: Uuid;
  equipment_id: Uuid | null;
  kind: ListingKind;
  condition: ListingCondition;
  model_name: string;
  capacity_milli: number | null;
  model_year: number | null;
  usage_hours: number | null;
  price_won: number | null;
  badge: string | null;
  usage_label: string | null;
  condition_label: string | null;
  availability: string | null;
  location: string | null;
  description: string | null;
  listing_type: ListingType;
  status: ListingStatus;
  sort_weight: number;
  created_at: Timestamp;
  updated_at: Timestamp;
  media: Array<ListingMediaView>;
};

export type SaveEvaluationReviewRequest = {
  grade?: EvaluationGrade | null;
  note?: string | null;
  evidence_links?: Array<EvaluationEvidenceLinkInput>;
};

export type SavePurchasePreferencesRequest = {
  schema_version: number;
  preferences: {
    [key: string]: unknown;
  };
};

export type SchedulePayrollDisbursementRequest = {
  scheduled_at: string;
};

export type ScheduleRunItem = {
  run_id: Uuid;
  status: string;
  definition_id: Uuid;
  definition_version: number;
  started_at: string;
  completed_at?: string | null;
  failed_at?: string | null;
};

export type ScheduleRunListResponse = {
  items: Array<ScheduleRunItem>;
};

/**
 * Global search hits, each an ObjectHead scoped identically to resolveObject (a hit the caller could not resolve never appears). Grouped by kind; exists is always true for a hit.
 */
export type SearchResponse = {
  results: Array<ObjectHead>;
};

export type SelfLeaveBalance = {
  employee_id: Uuid;
  name: string;
  accrued_units: LeaveBalanceAmount | null;
  used_units: LeaveBalanceAmount | null;
  remaining_units: LeaveBalanceAmount | null;
  filing_state: "ready" | "home_branch_required";
  home_branch_id: string | null;
};

/**
 * Compose a message. For reply/forward, in_reply_to is required and references carries the accumulated chain. The From is constrained to the configured account address (it cannot be set here).
 */
export type SendMailRequest = {
  to: Array<MailAddress>;
  cc?: Array<MailAddress>;
  bcc?: Array<MailAddress>;
  subject: string;
  body_text: string;
  attachments?: Array<MailAttachment>;
  in_reply_to?: string | null;
  references?: Array<string>;
};

export type SendMailResult = {
  message_id: Uuid;
  rfc_message_id: string;
};

export type SendMessengerMessageRequest = {
  body: string;
  attachment_evidence_ids?: Array<Uuid>;
  quoted_message_id?: string | null;
};

/**
 * The series an instance belongs to, or null.
 */
export type SeriesByInstanceResponse = {
  series: SeriesHead | null;
};

/**
 * A series head plus its instances resolved to ObjectHeads, ordered by attach time. Instances not resolvable for the caller are omitted (deny-by-omission).
 */
export type SeriesDetailResponse = {
  id: Uuid;
  code: string;
  label: string;
  created_at: Timestamp;
  instances: Array<ObjectHead>;
};

export type SeriesHead = {
  id: Uuid;
  code: string;
  label: string;
  created_at: Timestamp;
};

export type SetEmployeeHomeBranchRequest = {
  branch_id: Uuid;
  expected_updated_at: Timestamp;
};

export type SetLifecycleHoldRequest = {
  legalHold: boolean;
  retentionUntil?: string;
};

export type SetMessengerThreadMuteRequest = {
  muted: boolean;
};

export type SetTodoDoneRequest = {
  done: boolean;
};

export type SettlementLineKind = "LABOR" | "PART" | "OUTSOURCE" | "OTHER";

export type SettlementLineRequest = {
  kind: SettlementLineKind;
  label: string;
  amount_krw: number;
  source_ref?: string;
  sort_order?: number;
};

export type SettlementLineSummary = {
  id: Uuid;
  kind: SettlementLineKind;
  label: string;
  amount_krw: number;
  source_ref: string | null;
  sort_order: number;
};

/**
 * Cost-settlement lifecycle (정산 → 전표). VOID is terminal and frees the one-live-settlement slot.
 */
export type SettlementStatus = "DRAFT" | "SUBMITTED" | "APPROVED" | "VOID";

export type SettlementSummary = {
  id: Uuid;
  work_order_id: Uuid;
  branch_id: Uuid;
  status: SettlementStatus;
  total_amount_krw: number;
  voucher_ref: string | null;
  note: string | null;
  lines: Array<SettlementLineSummary>;
  created_by: Uuid;
  submitted_by: string | null;
  submitted_at: string | null;
  approved_by: string | null;
  approved_at: string | null;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type SignupRequest = {
  email: string;
};

export type SignupResponse = {
  accepted: boolean;
};

export type SimulateWorkflowDefinitionRequest = {
  definition?: {
    [key: string]: unknown;
  };
  approval_line?: Array<{
    [key: string]: unknown;
  }>;
  payment_line?: Array<{
    [key: string]: unknown;
  }>;
  notification_rules?: Array<{
    [key: string]: unknown;
  }>;
  action_allowlist?: Array<WorkflowActionAllowlistEntry>;
  sample_context?: {
    [key: string]: unknown;
  };
};

export type SimulationOutcome = {
  effect: "allow" | "deny";
  determining_policies: Array<string>;
  errors: Array<string>;
  reason: string;
};

export type SiteContact = {
  name: string | null;
  phone: string | null;
  email: string | null;
};

export type SiteLocationGroup = {
  site_id: Uuid;
  site_name: string;
  customer_id: Uuid;
  customer_name: string;
  branch_id: Uuid;
  address: string | null;
  postal_code: string | null;
  province: string | null;
  city: string | null;
  latitude: number | null;
  longitude: number | null;
  geofence_radius_m: number | null;
  contact_name: string | null;
  contact_phone: string | null;
  contact_email: string | null;
  equipment_count: number;
  rented_count: number;
  spare_count: number;
  substitution_active_count: number;
};

export type StartP1DispatchRequest = {
  incident_location?: IncidentLocation;
  include_region?: boolean;
};

export type StartWorkflowRunRequest = {
  definition_id: Uuid;
  definition_version?: number;
  object_type?: string;
  object_id?: Uuid;
  trigger_type: "MANUAL" | "SCHEDULE" | "OBJECT_EVENT" | "IMPORT_EVENT" | "MAIL_EVENT" | "MESSENGER_EVENT" | "CALENDAR_EVENT" | "POLL_EVENT" | "API";
  idempotency_key: string;
  correlation_id?: string;
  input_payload?: {
    [key: string]: unknown;
  };
  context_payload?: {
    [key: string]: unknown;
  };
};

export type StartWorkflowRunResponse = {
  run: WorkflowRunSummary;
  next_task?: WorkflowTaskSummary;
};

export type StatusHistorySummary = {
  id: Uuid;
  actor: string | null;
  action: string;
  from_status: string | null;
  to_status: WorkOrderStatus;
  occurred_at: Timestamp;
};

/**
 * Public lead intake payload. `name` and `phone` are required; `location`, `message`, and `listing_id` are optional. The server never echoes any field back.
 */
export type SubmitInquiryRequest = {
  name: string;
  phone: string;
  topic: InquiryTopic;
  location?: string | null;
  message?: string | null;
  listing_id?: Uuid | null;
};

export type SubmitReportRequest = {
  result_type: WorkResultType;
  diagnosis: string;
  action_taken: string;
};

export type SubmittableDefinitionListResponse = {
  items: Array<SubmittableDefinitionResponse>;
};

/**
 * An ACTIVE workflow definition the caller may start from the 기안 template gallery. Carries only the metadata definitions actually hold; active_version is the version a start binds to.
 */
export type SubmittableDefinitionResponse = {
  id: Uuid;
  workflow_key: string;
  display_name: string;
  object_type: string;
  active_version: number;
  required_approval_line: boolean;
  required_payment_line: boolean;
};

export type SubstituteAssignment = {
  id: string;
  branch_id: string;
  source_equipment_id: string;
  substitute_equipment_id: string;
  assigned_by: string;
  assigned_to?: string | null;
  assignment_location: string;
  assigned_at: string;
  returned_by?: string | null;
  returned_at?: string | null;
  return_note?: string | null;
};

export type SubstituteCandidate = {
  equipment_id: Uuid;
  branch_id: Uuid;
  equipment_no: string;
  management_no: string | null;
  model: string | null;
  status: EquipmentStatus;
  specification: string;
  ton_text: string;
  ton_milli: number | null;
  power_code: string;
  power_label: string | null;
  customer_name: string;
  site_name: string;
  placement_location: string | null;
  match_kind: SubstituteMatchKind;
  ton_delta_milli: number | null;
};

export type SubstituteCandidatePage = {
  items: Array<SubstituteCandidate>;
  total: number;
};

export type SubstituteMatchKind = "exact_ton" | "nearest_above" | "unknown_ton_exact_text";

export type SupportIntakeAck = {
  status: string;
};

export type SupportTicketAcceptance = {
  id: Uuid;
  ticket_id: Uuid;
  kind: SupportTicketAcceptanceKind;
  channel: SupportTicketAcceptanceChannel;
  accepted_by: string;
  note: string | null;
  recorded_by_user_id: Uuid;
  recorded_by_name: string | null;
  occurred_at: Timestamp;
};

export type SupportTicketAcceptanceChannel = "IN_PERSON" | "PHONE" | "EMAIL" | "MESSENGER";

export type SupportTicketAcceptanceKind = "CUSTOMER_ACCEPTED" | "CUSTOMER_DECLINED";

export type SupportTicketCategory = "SYSTEM_BUG" | "ACCESS_REQUEST" | "OPERATIONAL" | "EQUIPMENT_INQUIRY" | "COMPLAINT" | "OTHER";

export type SupportTicketComment = {
  id: Uuid;
  ticket_id: Uuid;
  author_user_id: Uuid;
  author_name: string | null;
  body: string;
  is_internal_note: boolean;
  created_at: Timestamp;
};

export type SupportTicketDetail = {
  ticket: SupportTicketSummary;
  comments: Array<SupportTicketComment>;
};

export type SupportTicketOrigin = "INTERNAL" | "CUSTOMER";

export type SupportTicketPage = {
  items: Array<SupportTicketSummary>;
  next_cursor: string | null;
  total: number;
};

export type SupportTicketPriority = "LOW" | "MEDIUM" | "HIGH" | "URGENT";

export type SupportTicketStatus = "OPEN" | "IN_PROGRESS" | "ON_HOLD" | "RESOLVED" | "CLOSED";

export type SupportTicketSummary = {
  id: Uuid;
  branch_id: Uuid;
  origin: SupportTicketOrigin;
  category: SupportTicketCategory;
  priority: SupportTicketPriority;
  status: SupportTicketStatus;
  title: string;
  requester_user_id: Uuid;
  requester_name: string | null;
  assignee_user_id: Uuid;
  assignee_name: string | null;
  due_at: string | null;
  created_at: Timestamp;
  updated_at: Timestamp;
  resolved_at: string | null;
  closed_at: string | null;
  site_id: string | null;
  site_name: string | null;
  customer_id: string | null;
  customer_name: string | null;
  work_order_id: string | null;
};

export type SyncBatchRequest = {
  sync_id: string;
  operations: Array<SyncOperationRequest>;
};

export type SyncBatchResponse = {
  sync_id: string;
  results: Array<SyncOperationResult>;
};

export type SyncError = {
  code: string;
  message: string;
};

export type SyncOperationKind = "WORK_ORDER_START" | "WORK_ORDER_REPORT";

export type SyncOperationRequest = {
  request_id: string;
  operation: SyncOperationKind;
  created_at: Timestamp;
  payload: SyncWorkOrderStartPayload | SyncWorkOrderReportPayload;
};

export type SyncOperationResult = {
  request_id: string;
  operation: SyncOperationKind;
  status: SyncOperationStatus;
  http_status: number;
  result?: WorkOrderSummary;
  error?: SyncError;
  replayed: boolean;
};

export type SyncOperationStatus = "APPLIED" | "FAILED";

export type SyncWorkOrderReportPayload = {
  work_order_id: Uuid;
  result_type: WorkResultType;
  diagnosis: string;
  action_taken: string;
};

export type SyncWorkOrderStartPayload = {
  work_order_id: Uuid;
};

export type SystemPolicyRoleResponse = {
  role_key: string;
  display_name: string;
  status: string;
  is_system: boolean;
  permissions: Array<PolicyPermissionResponse>;
};

export type TargetChangeDecision = "APPROVED" | "REJECTED";

export type TargetChangeRequest = {
  requested_target_due_at: Timestamp;
  reason: string;
};

export type TargetChangeRequestSummary = {
  id?: Uuid;
  work_order_id?: Uuid;
  branch_id?: Uuid;
  requested_target_due_at?: Timestamp;
  status?: "REQUESTED" | "APPROVED" | "REJECTED";
};

export type Team = "MAINTENANCE" | "PREVENTION" | "MANAGEMENT" | "RECEPTION";

export type TimeChangeCoverageEvidence = {
  headcount: number;
  already_out: number;
  minimum_on_duty: number;
  projected_available: number;
};

/**
 * Closed set of system-judged §60⑤ grounds. Free-text manager narrative is never a grounds code.
 */
export type TimeChangeGroundsCode = "branch_coverage_shortfall";

export type Timestamp = string;

/**
 * Nullable/absent until an RFC-3161 TSA lane exists (FUTURE — not faked).
 */
export type TimestampAuthorityProofView = {
  id: string;
  copy_id: string;
  status: TsaProofStatus;
  provider: string;
  policy_oid?: string | null;
  serial_number?: string | null;
  hash_algorithm: string;
  message_imprint_sha256?: string | null;
  generated_at?: string | null;
  accuracy_millis?: number | null;
  ordering?: boolean | null;
  tsa_cert_fingerprint_sha256?: string | null;
  token_digest_sha256?: string | null;
  token_storage?: EvidenceStorageRef | null;
  verified_at?: string | null;
  failure_reason?: string | null;
  created_by: string;
  created_at: string;
};

export type TodoPage = {
  items: Array<TodoSummary>;
};

/**
 * One scope chip or object link: a reference to a domain object by kind + id with an optional display-label snapshot. `kind` is an extensible free-form string (frontend object-registry kinds), not an enum.
 */
export type TodoRef = {
  kind: string;
  id: string;
  label?: string;
};

export type TodoSummary = {
  id: Uuid;
  owner_user_id: Uuid;
  text: string;
  scopes: Array<TodoRef>;
  links: Array<TodoRef>;
  done: boolean;
  created_at: Timestamp;
  updated_at: Timestamp;
  done_at: string | null;
};

export type TokenPairResponse = {
  access_token: string;
  refresh_token?: string | null;
  token_type: "Bearer";
  refresh_expires_at: Timestamp;
  requires_passkey_setup: boolean;
};

export type TransitionLifecycleRequest = {
  toState: string;
  reason: string;
};

export type TransitionRequirements = {
  requires_reason: boolean;
  requires_four_eyes: boolean;
  requires_checklist: boolean;
};

export type TransitionTicketRequest = {
  to_status: SupportTicketStatus;
};

export type TraversalEdge = {
  id: string;
  link_type_id: string;
  from_instance_id: string;
  to_instance_id: string;
};

export type TraversalGraph = {
  root: string;
  nodes: Array<TraversalNode>;
  edges: Array<TraversalEdge>;
};

export type TraversalNode = {
  instance_id: string;
  object_type_id: string;
  title: string;
  lifecycle_state: InstanceLifecycleState;
  depth: number;
};

/**
 * Triage an OPEN finding. A memo is required when status is DISMISSED or ESCALATED, optional for REVIEWED. Max 2000 characters.
 */
export type TriageFindingRequest = {
  status: "REVIEWED" | "DISMISSED" | "ESCALATED";
  memo?: string | null;
};

export type TriggerBindingListResponse = {
  items: Array<TriggerBindingResponse>;
  registered_event_keys: Array<string>;
};

export type TriggerBindingResponse = {
  id: Uuid;
  definition_id: Uuid;
  trigger_type: "OBJECT_EVENT" | "IMPORT_EVENT" | "MAIL_EVENT" | "MESSENGER_EVENT" | "CALENDAR_EVENT" | "POLL_EVENT";
  event_key: string;
  subject_kind: string | null;
  enabled: boolean;
  created_at: string;
  updated_at: string;
};

export type TriggerWorkflowRunRequest = {
  trigger_type?: "MANUAL" | "SCHEDULE" | "WEBHOOK" | "SYSTEM";
  idempotency_key?: string | null;
  four_eyes_request_ref?: string | null;
};

export type TsaProofStatus = "MISSING" | "PENDING" | "VERIFIED" | "FAILED" | "REVOKED" | "EXPIRED_CA";

export type UnavailableMetric = {
  metric: KpiMetric;
  source_domain: string;
  reason: string;
};

export type UnlockPeriodLockRequest = {
  reason: string;
};

export type UnreadNotificationCountResponse = {
  unread: number;
};

export type UpdateBranchRequest = {
  region_id?: Uuid;
  name?: string;
};

/**
 * Partial update. Absent keys are left unchanged; nullable keys set to null clear the column.
 */
export type UpdateEquipmentRequest = {
  customer_name?: string;
  site_name?: string;
  status?: EquipmentStatus;
  specification?: string;
  ton_text?: string;
  management_no?: string | null;
  power_label?: string | null;
  manager_name?: string | null;
  placement_location?: string | null;
  placement_no?: string | null;
  operation_shift?: string | null;
  maker?: string | null;
  model?: string | null;
  vin?: string | null;
  year?: string | null;
  hours?: number | null;
  vehicle_registration_no?: string | null;
  insured?: boolean | null;
  insurer?: string | null;
  policy_holder?: string | null;
  insured_party?: string | null;
  asset_owner?: string | null;
  asset_registered_on?: string | null;
  rental_started_on?: string | null;
  rental_fee?: number | null;
  vehicle_value?: number | null;
  residual_value?: number | null;
  acquisition_cost_won?: number | null;
  acquisition_date?: string | null;
  note?: string | null;
};

export type UpdateInquiryStatusRequest = {
  status: InquiryStatus;
};

/**
 * Partial update. Absent keys are left unchanged; nullable keys explicitly set to null clear the column. At least one field must be supplied.
 */
export type UpdateListingRequest = {
  kind?: ListingKind;
  condition?: ListingCondition;
  model_name?: string;
  capacity_milli?: number | null;
  model_year?: number | null;
  usage_hours?: number | null;
  price_won?: number | null;
  badge?: string | null;
  usage_label?: string | null;
  condition_label?: string | null;
  availability?: string | null;
  location?: string | null;
  description?: string | null;
  listing_type?: ListingType;
  status?: ListingStatus;
  sort_weight?: number;
  equipment_id?: Uuid | null;
};

export type UpdateNoticeDraftRequest = {
  title?: string;
  body?: string;
  category?: NoticeCategory;
  audience?: NoticeAudienceInput;
};

/**
 * Every field is optional; an omitted field is left unchanged.
 */
export type UpdateOrgChangeDraftRequest = {
  kind?: OrgChangeKind;
  effectiveDate?: Date;
  reason?: string;
  proposal?: Array<OrgProposalOp>;
};

export type UpdatePlatformGroupRequest = {
  slug?: string;
  name?: string;
  status?: PlatformOrgStatus;
};

export type UpdatePlatformOrgRequest = {
  status: PlatformOrgStatus;
};

export type UpdatePolicyRoleRequest = {
  display_name: string;
  description?: string | null;
  permissions: Array<PolicyPermissionResponse>;
  conditions?: Array<PolicyConditionResponse>;
  step_up: PasskeyStepUpAssertion;
};

export type UpdatePolicyRoleStatusRequest = {
  status: "DRAFT" | "ACTIVE" | "RETIRED";
  step_up: PasskeyStepUpAssertion;
};

export type UpdateRecruitPostingRequest = CreateRecruitPostingRequest & {
  expected_updated_at: Timestamp;
};

export type UpdateRegionRequest = {
  name: string;
};

export type UpdateSelfProfileRequest = {
  display_name?: string;
  phone?: string | null;
};

/**
 * Partial site update. Absent keys are left unchanged; nullable keys set to null clear the column. Latitude and longitude must be supplied together and within WGS84 ranges.
 */
export type UpdateSiteRequest = {
  address?: string | null;
  province?: string | null;
  city?: string | null;
  postal_code?: string | null;
  latitude?: number | null;
  longitude?: number | null;
  geofence_radius_m?: number | null;
  contact_name?: string | null;
  contact_phone?: string | null;
  contact_email?: string | null;
};

export type UpdateUserRequest = {
  display_name?: string;
  employee_id?: string | null;
  phone?: string | null;
  team?: Team;
  roles?: Array<string>;
  branch_ids?: Array<Uuid>;
  preview_acknowledged?: boolean;
  preview_receipt_id?: Uuid;
};

export type UpdateWorkOrderIntakeRequest = {
  symptom?: string;
  customer_request?: string;
  maintenance_type?: MaintenanceType;
  maintenance_cause?: MaintenanceCause;
};

/**
 * Partial update for a DRAFT workflow definition. Workflow key and object type are immutable.
 */
export type UpdateWorkflowDefinitionRequest = {
  display_name?: string;
  definition?: {
    [key: string]: unknown;
  };
  approval_line?: Array<{
    [key: string]: unknown;
  }>;
  payment_line?: Array<{
    [key: string]: unknown;
  }>;
  notification_rules?: Array<{
    [key: string]: unknown;
  }>;
  action_allowlist?: Array<WorkflowActionAllowlistEntry>;
  required_approval_line?: boolean;
  required_payment_line?: boolean;
};

export type UpdateWorkflowScheduleRequest = {
  label?: string;
  cron_expr?: string;
  timezone?: string;
  enabled?: boolean;
};

export type UpsertCycleCountLineRequest = {
  expectedVersion: number;
  itemId: Uuid;
  countedQuantityMilli: number;
  reason?: "DAMAGE" | "LOSS" | "MISCOUNT" | "FOUND" | "OTHER" | null;
  note?: string | null;
};

export type UpsertNotificationPolicyRequest = {
  scope: "all" | "category" | "object";
  category?: string;
  link?: NotificationLink;
};

export type UserPage = {
  items: Array<UserSummary>;
  limit: number;
  offset: number;
  total: number;
};

export type UserSummary = {
  id: Uuid;
  display_name: string;
  employee_id: string | null;
  employee_name: string | null;
  employee_number: string | null;
  employee_company: string | null;
  employee_org_unit: string | null;
  employee_position: string | null;
  employee_identity_review_required: boolean | null;
  employee_identity_resolution_confidence: string | null;
  employee_link_status: "LINKED" | "UNLINKED";
  phone: string | null;
  team: Team;
  roles: Array<string>;
  branch_ids: Array<Uuid>;
  is_active: boolean;
  has_passkey: boolean;
  account_status: AccountStatus;
  created_at: Timestamp;
};

export type Uuid = string;

export type VerifyOutcome = "VERIFIED" | "MISMATCH" | "INDETERMINATE";

export type VoidSettlementRequest = {
  reason: string;
};

export type VotePollRequest = {
  selected_option_ids: Array<Uuid>;
};

export type VoucherLineInput = {
  account_code: string;
  side: DebitCredit;
  amount_won: number;
  memo?: string;
};

export type VoucherLineSummary = {
  id: Uuid;
  line_no: number;
  account_code: string;
  side: DebitCredit;
  amount_won: number;
  memo: string;
};

export type VoucherStatus = "DRAFT" | "BALANCE_CHECKED" | "APPROVED" | "POSTED" | "REVERSED";

export type VoucherSummary = {
  id: Uuid;
  voucher_no: string;
  branch_id: Uuid;
  branch_name?: string | null;
  status: VoucherStatus;
  memo: string;
  source_object_type?: string | null;
  source_object_id?: string | null;
  reversal_of_voucher_id?: string | null;
  reversed_by_voucher_id?: string | null;
  debit_total_won: number;
  credit_total_won: number;
  lines: Array<VoucherLineSummary>;
  created_by: Uuid;
  created_by_name?: string | null;
  approved_by?: string | null;
  approved_by_name?: string | null;
  posted_at?: string | null;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type WithdrawRecruitOfferRequest = {
  reason: string;
};

export type WorkDiaryActionEntry = {
  site_name: string;
  management_no: string;
  diagnosis: string;
  action_taken: string;
};

export type WorkDiaryBody = {
  previous_results: string;
  today_plans: string;
  urgent_actions: Array<WorkDiaryActionEntry>;
  source_notes: Array<ExportSourceNote>;
};

export type WorkDiaryDraft = {
  id: Uuid;
  date: Date;
  status: WorkDiaryStatus;
  body: WorkDiaryBody;
  confirmed_by: string | null;
  confirmed_at: string | null;
};

export type WorkDiaryStatus = "DRAFT" | "CONFIRMED";

export type WorkDiaryUpdateRequest = {
  body: WorkDiaryBody;
};

export type WorkOrderDetail = WorkOrderListItem & {
  symptom: string;
  customer_request: string | null;
  delay_reason: string | null;
  delay_note: string | null;
  diagnosis: string | null;
  action_taken: string | null;
  report_submitted_by: string | null;
  report_submitted_at: string | null;
  kpi_excluded: boolean;
  evidence_verified: boolean;
  approval_line: Array<ApprovalStepSummary>;
  status_history: Array<StatusHistorySummary>;
  evidence: Array<EvidenceSummary>;
  settlement: null | SettlementSummary;
};

export type WorkOrderFacetBucket = {
  value: string;
  count: number;
  filters: {
    [key: string]: string;
  };
};

export type WorkOrderHistogramBucket = {
  bucket: string;
  count: number;
  filters: {
    [key: string]: string;
  };
};

export type WorkOrderLensAggregates = {
  total_count: number;
  p1_count: number;
  overdue_open_count: number;
  unassigned_count: number;
  preventive_on_time_rate: number | null;
  mttr_minutes: number | null;
};

export type WorkOrderLensFacets = {
  status: Array<WorkOrderFacetBucket>;
  priority: Array<WorkOrderFacetBucket>;
  maintenance_type: Array<WorkOrderFacetBucket>;
  maintenance_cause: Array<WorkOrderFacetBucket>;
};

export type WorkOrderLensHistograms = {
  target_due_date: Array<WorkOrderHistogramBucket>;
};

export type WorkOrderLensListograms = {
  customers: Array<WorkOrderNamedBucket>;
  sites: Array<WorkOrderNamedBucket>;
};

export type WorkOrderListItem = {
  id: Uuid;
  request_no: string;
  branch_id: Uuid;
  status: WorkOrderStatus;
  priority: PriorityLevel;
  result_type: WorkResultType;
  maintenance_type: null | MaintenanceType;
  maintenance_cause: null | MaintenanceCause;
  target_due_at: string | null;
  created_at: Timestamp;
  updated_at: Timestamp;
  equipment: EquipmentSummary;
  customer: NamedEntity;
  site: NamedEntity;
  site_contact: null | SiteContact;
  assignments: Array<AssignmentSummary>;
};

export type WorkOrderListPage = {
  items: Array<WorkOrderListItem>;
  limit: number;
  offset: number;
  total: number;
  lens?: WorkOrderObjectSetLens;
};

export type WorkOrderNamedBucket = {
  id: Uuid;
  name: string;
  count: number;
  filters: {
    [key: string]: string;
  };
};

export type WorkOrderObjectSetLens = {
  object_type: "work_order";
  aggregates: WorkOrderLensAggregates;
  facets: WorkOrderLensFacets;
  histograms: WorkOrderLensHistograms;
  listograms: WorkOrderLensListograms;
};

export type WorkOrderStatus = "RECEIVED" | "UNASSIGNED" | "ASSIGNED" | "IN_PROGRESS" | "REPORT_SUBMITTED" | "ADMIN_REVIEW" | "FINAL_COMPLETED" | "REJECTED" | "ON_HOLD" | "DELAYED" | "TEMPORARY_ACTION" | "PART_WAITING" | "EQUIPMENT_IN_USE" | "REVISIT_REQUIRED" | "ARCHIVED" | "CANCELLED";

export type WorkOrderSummary = {
  id: Uuid;
  request_no: string;
  branch_id: Uuid;
  equipment_id: Uuid;
  customer_id: Uuid;
  site_id: Uuid;
  status: WorkOrderStatus;
  priority: PriorityLevel;
  result_type: WorkResultType;
  maintenance_type: null | MaintenanceType;
  maintenance_cause: null | MaintenanceCause;
  evidence_verified: boolean;
};

export type WorkResultType = "COMPLETED" | "TEMPORARY_ACTION" | "INCOMPLETE" | "REVISIT_REQUIRED" | "UNKNOWN";

export type WorkbenchActionInboxItem = {
  id: string;
  urgency: WorkbenchUrgency;
  title: string;
  due_at?: Timestamp;
  source: WorkbenchSourceRef;
  target: WorkbenchTarget;
};

export type WorkbenchActionSourceEnvelope = WorkbenchActionSourceOk | WorkbenchDeniedSourceEnvelope | WorkbenchUnavailableSourceEnvelope;

/**
 * Exact, complete action-inbox snapshot admitted at the request ceiling. The server follows the public immutable cursor contract in pages of at most 200 items and fails this source closed rather than returning a priority-ranked prefix when the exact set exceeds its 1000-item budget, drifts, repeats a cursor, or repeats an item id.
 */
export type WorkbenchActionSourceOk = {
  status: "ok";
  as_of: Timestamp;
  items: Array<WorkbenchActionInboxItem>;
  total: number;
  truncated: boolean;
};

export type WorkbenchCalendarItem = {
  id: Uuid;
  title: string;
  starts_at: Timestamp;
  ends_at: Timestamp;
  target: WorkbenchTarget;
};

export type WorkbenchCalendarSourceEnvelope = WorkbenchCalendarSourceOk | WorkbenchDeniedSourceEnvelope | WorkbenchUnavailableSourceEnvelope;

export type WorkbenchCalendarSourceOk = {
  status: "ok";
  as_of: Timestamp;
  items: Array<WorkbenchCalendarItem>;
  total: number;
  truncated: boolean;
};

export type WorkbenchDeniedSourceEnvelope = {
  status: "denied";
  code: string;
};

export type WorkbenchEffectiveScope = WorkbenchScopeAll | WorkbenchScopeBranches;

export type WorkbenchRange = {
  from: Timestamp;
  to: Timestamp;
};

export type WorkbenchScopeAll = {
  kind: "all";
  selected_branch_id?: Uuid;
};

export type WorkbenchScopeBranches = {
  kind: "branches";
  branch_ids: Array<Uuid>;
  selected_branch_id?: Uuid;
};

export type WorkbenchSourceRef = {
  kind: string;
  id: Uuid;
};

/**
 * Server-issued bounded module target; never an arbitrary URL.
 */
export type WorkbenchTarget = {
  module: string;
  id: string;
};

export type WorkbenchTodoItem = {
  id: Uuid;
  text: string;
  done: boolean;
  source_order: number;
  target: WorkbenchTarget;
};

export type WorkbenchTodoSourceEnvelope = WorkbenchTodoSourceOk | WorkbenchDeniedSourceEnvelope | WorkbenchUnavailableSourceEnvelope;

export type WorkbenchTodoSourceOk = {
  status: "ok";
  as_of: Timestamp;
  items: Array<WorkbenchTodoItem>;
  total: number;
  truncated: boolean;
};

export type WorkbenchUnavailableSourceEnvelope = {
  status: "unavailable";
  code: string;
};

export type WorkbenchUrgency = "now" | "today" | "wait";

export type WorkflowActionAllowlistEntry = {
  connector_key: string;
  action_key: string;
};

export type WorkflowConnectorDescriptor = {
  connector_key: string;
  display_name: string;
  action_keys: Array<string>;
};

export type WorkflowDefinitionEventResponse = {
  id: Uuid;
  definition_id: Uuid;
  version: number | null;
  status: string;
  action: string;
  actor_display_name: string | null;
  summary: string;
  created_at: string;
};

export type WorkflowDefinitionHistoryResponse = {
  items: Array<WorkflowDefinitionEventResponse>;
};

export type WorkflowDefinitionListResponse = {
  items: Array<WorkflowDefinitionResponse>;
};

export type WorkflowDefinitionResponse = {
  id: Uuid;
  workflow_key: string;
  display_name: string;
  object_type: string;
  status: "DRAFT" | "ACTIVE" | "PAUSED" | "RETIRED";
  latest_version: number;
  active_version: number | null;
  definition: {
    [key: string]: unknown;
  };
  approval_line: Array<{
    [key: string]: unknown;
  }>;
  payment_line: Array<{
    [key: string]: unknown;
  }>;
  notification_rules: Array<{
    [key: string]: unknown;
  }>;
  action_allowlist: Array<WorkflowActionAllowlistEntry>;
  required_approval_line: boolean;
  required_payment_line: boolean;
  object_kinds: Array<string>;
  pending_version: number | null;
  pending_staged_by: Uuid | null;
  created_at: string;
  updated_at: string;
};

export type WorkflowObjectKind = "work_order" | "support_ticket";

export type WorkflowObjectSubject = {
  object_type: WorkflowObjectKind;
  object_id: Uuid;
};

export type WorkflowRunDetailResponse = {
  run: WorkflowRunDetailRun;
  waiting_tasks: Array<WorkflowTaskSummary>;
  timeline: Array<WorkflowRunTimelineStep>;
};

export type WorkflowRunDetailRun = {
  id: Uuid;
  status: string;
  definition_id: Uuid;
  definition_version: number;
  trigger_type: string;
  object_type?: string;
  object_id?: Uuid;
  initiated_by?: Uuid;
  error_payload?: {
    [key: string]: unknown;
  };
  started_at: Timestamp;
  updated_at: Timestamp;
  completed_at?: Timestamp;
  failed_at?: Timestamp;
};

export type WorkflowRunDetailTarget = {
  kind: "workflow_run_detail";
  run_id: Uuid;
};

export type WorkflowRunForObjectSummary = {
  run_id: Uuid;
  definition_id: Uuid;
  definition_version: number;
  status: string;
  trigger_type: string;
  object_type: WorkflowObjectKind;
  object_id: Uuid;
  started_at: Timestamp;
  updated_at: Timestamp;
  completed_at?: Timestamp;
  detail_target: WorkflowRunDetailTarget;
};

export type WorkflowRunListItem = {
  run_id: Uuid;
  status: string;
  definition_id: Uuid;
  definition_version: number;
  object_type?: string;
  object_id?: Uuid;
  initiated_by?: Uuid;
  started_at: Timestamp;
  updated_at: Timestamp;
};

export type WorkflowRunListResponse = {
  items: Array<WorkflowRunListItem>;
};

export type WorkflowRunLogResponse = {
  items: Array<WorkflowRunResponse>;
};

export type WorkflowRunResponse = {
  id: Uuid;
  code: string;
  definition_id: Uuid;
  definition_version: number;
  trigger_type: "MANUAL" | "SCHEDULE" | "WEBHOOK" | "SYSTEM";
  status: "STARTING" | "RUNNING" | "WAITING" | "SUCCEEDED" | "FAILED" | "CANCELLED";
  actor_display_name: string | null;
  summary: string;
  error_message: string | null;
  generated_objects: Array<string>;
  started_at: string;
  updated_at: string;
  completed_at: string | null;
  failed_at: string | null;
};

export type WorkflowRunSummary = {
  id: Uuid;
  status: string;
  definition_id: Uuid;
  definition_version: number;
  object_type?: string;
  object_id?: Uuid;
  initiated_by?: Uuid;
  started_at: Timestamp;
};

export type WorkflowRunTimelineStep = {
  node_key: string;
  node_type: string;
  status: string;
  attempt: number;
  started_at?: Timestamp;
  finished_at?: Timestamp;
  actor?: Uuid;
  outcome?: {
    [key: string]: unknown;
  };
  error?: {
    [key: string]: unknown;
  };
};

export type WorkflowRunsForObjectResponse = {
  subject: WorkflowObjectSubject;
  as_of: Timestamp;
  items: Array<WorkflowRunForObjectSummary>;
  next_before?: Uuid;
};

export type WorkflowScheduleListResponse = {
  items: Array<WorkflowScheduleResponse>;
};

export type WorkflowScheduleResponse = {
  id: Uuid;
  label: string;
  cron_expr: string;
  timezone: string;
  definition_id: Uuid;
  enabled: boolean;
  next_run_at?: string | null;
  last_run_at?: string | null;
  last_status?: "STARTED" | "SKIPPED" | "FAILED" | null;
  created_at: string;
  updated_at: string;
};

export type WorkflowSimulationFinding = {
  severity: "info" | "warning" | "blocker";
  code: string;
  message: string;
};

export type WorkflowSimulationResponse = {
  decision: "ready" | "blocked";
  findings: Array<WorkflowSimulationFinding>;
  simulated_path?: Array<string>;
};

/**
 * Fresh passkey step-up assertion for a sensitive Workflow Studio mutation.
 */
export type WorkflowStepUpRequest = {
  step_up: PasskeyStepUpAssertion;
  four_eyes_request_ref?: string | null;
};

export type WorkflowStudioCatalogResponse = {
  connectors: Array<WorkflowConnectorDescriptor>;
  templates: Array<WorkflowTemplateDescriptor>;
};

export type WorkflowTaskListResponse = {
  items: Array<WorkflowTaskSummary>;
};

export type WorkflowTaskSummary = {
  task_id: Uuid;
  run_id: Uuid;
  waiting_key: string;
  title: string;
  assignee_role_key?: string;
  required_policy?: string;
  object_type?: string;
  object_id?: Uuid;
  status: string;
  claimed_by?: Uuid;
  due_at?: Timestamp;
  form_payload: {
    [key: string]: unknown;
  };
};

export type WorkflowTemplateDescriptor = {
  template_key: string;
  display_name: string;
  object_type: string;
  required_approval_line: boolean;
  required_payment_line: boolean;
};

export type WorkspaceResponse = {
  layout: {
    [key: string]: unknown;
  };
};

export type WorkspaceUpsertRequest = {
  layout: {
    [key: string]: unknown;
  };
};

export type WormReplicaStatus = "PENDING" | "VERIFIED" | "FAILED";

export type WormStorageStatus = "PENDING" | "VERIFIED" | "FAILED";
