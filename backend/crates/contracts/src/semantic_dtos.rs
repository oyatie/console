//! Runtime Head / Input shapes that generate OpenAPI property bags.
//!
//! These structs are the compose-crate inventory of the types the runtime
//! already serializes: adapter Heads (`CompanyHead`, `OrgUnitHead`,
//! `JobPositionView`, `PersonHead`, `EmploymentHead`) and the thirteen
//! DispatchTarget Inputs (plus two nested write bags). `dto_schema_bags`
//! emits the JSON Schema property bags that used to live as literals in
//! `semantic_manifest.json`. Dual-written JSON blobs are not this contract.
//!
//! Marker types stand in for `uuid::Uuid` / `time` so this crate stays
//! dependency-free. Field *names* are the load-bearing inventory; CI drifts
//! them against the adapter Head structs.

#![allow(dead_code)]

use super::{GENERATED_SCHEMA_COUNT, Json, SemanticError};

/// UUID marker. Wire schema is `#/components/schemas/Uuid`.
pub struct Uuid;
/// RFC3339 timestamp marker. Wire schema is `#/components/schemas/Timestamp`.
pub struct Timestamp;
/// ISO calendar date marker. Wire schema is `string` / `format: date`.
pub struct IsoDate;
/// Open JSON object (`additionalProperties: true`), including nested write bags.
pub struct JsonObject;

// ---------------------------------------------------------------------------
// Runtime Head DTOs (field names must match the adapter Head structs)
// ---------------------------------------------------------------------------

/// `CompanyHead` in `console-ontology-canonical-adapter-postgres`.
pub struct Company {
    pub org_id: Uuid,
    pub legal_name: Option<String>,
    pub reg_no: Option<String>,
    pub version: i64,
}

/// `OrgUnitHead` in `console-ontology-canonical-adapter-postgres`.
pub struct OrgUnit {
    pub id: Uuid,
    pub name: Option<String>,
    pub parent_id: Option<Uuid>,
    pub version: i64,
}

/// `JobPositionView` in `console-ontology-canonical-adapter-postgres`.
pub struct JobPosition {
    pub job_position_id: Uuid,
    pub org_unit_id: Uuid,
    pub version: i64,
    pub attributes: JsonObject,
}

/// `PersonHead` in `console-ontology-canonical-adapter-postgres`. Closed four-field projection.
pub struct Person {
    pub id: Uuid,
    pub display_name: Option<String>,
    pub legal_name: Option<String>,
    pub version: i64,
}

/// `EmploymentHead` in `console-ontology-canonical-adapter-postgres`.
pub struct Employment {
    pub id: Uuid,
    pub person_id: Option<Uuid>,
    pub org_unit_id: Option<Uuid>,
    pub job_position_id: Option<Uuid>,
    pub appointed_on: Timestamp,
}

/// Canonical PayRun Head. `payable` stays `const: false`. REST list/detail remain
/// `PayrollRunSummary` / `PayrollRunDetail` (a superset); this is the ontology object.
pub struct PayRun {
    pub id: Uuid,
    pub period_start: IsoDate,
    pub period_end: IsoDate,
    pub source_label: String,
    pub status: String,
    pub payable: bool,
}

// ---------------------------------------------------------------------------
// Runtime Input DTOs (field names must match generated execute codecs)
// ---------------------------------------------------------------------------

pub struct CompanyReviseInput {
    pub attributes: JsonObject,
}

pub struct OrganizationCreateOrgUnitInput {
    pub source: Option<OrgUnitSourceBinding>,
    pub attributes: JsonObject,
}

pub struct OrganizationReviseOrgUnitInput {
    pub org_unit_id: Uuid,
    pub source: Option<OrgUnitSourceBinding>,
    pub attributes: JsonObject,
}

pub struct OrganizationCreateJobPositionInput {
    pub org_unit_id: Uuid,
    pub attributes: JsonObject,
}

pub struct OrganizationReviseJobPositionInput {
    pub job_position_id: Uuid,
    pub org_unit_id: Option<Uuid>,
    pub attributes: JsonObject,
}

pub struct PeopleCreatePersonInput {
    pub employee_id: Option<Uuid>,
    pub attributes: JsonObject,
}

pub struct PeopleRevisePersonInput {
    pub person_id: Uuid,
    pub employee_id: Option<Uuid>,
    pub attributes: JsonObject,
}

pub struct HrAppointInput {
    pub employee_id: Uuid,
    pub valid_from: Timestamp,
    pub attributes: EmploymentAttributesInput,
}

pub struct HrPromoteInput {
    pub employment_id: Uuid,
    pub valid_from: Timestamp,
    pub attributes: EmploymentAttributesInput,
}

pub struct HrTransferInput {
    pub employment_id: Uuid,
    pub valid_from: Timestamp,
    pub attributes: EmploymentAttributesInput,
}

pub struct PayrollCreateRunInput {
    pub run_id: Uuid,
    pub period_start: IsoDate,
    pub period_end: IsoDate,
    pub connector: Option<String>,
    pub job: Option<String>,
}

pub struct PayrollSubmitRunInput {
    pub run_id: Uuid,
}

pub struct PayrollDecideRunInput {
    pub run_id: Uuid,
    pub decision: String,
    pub reason: Option<String>,
}

/// `EmploymentAttributes` write bag (adapter), not the Employment Head.
pub struct EmploymentAttributesInput {
    pub company: String,
    pub org_unit_id: Option<Uuid>,
    pub job_position_id: Option<Uuid>,
    pub employment_status: String,
}

pub struct OrgUnitSourceBinding {
    pub kind: String,
    pub id: String,
}

// ---------------------------------------------------------------------------
// JSON Schema emission from the DTO field inventory
// ---------------------------------------------------------------------------

fn uuid_ref() -> Json {
    Json::obj(vec![("$ref", Json::str("#/components/schemas/Uuid"))])
}

fn timestamp_ref() -> Json {
    Json::obj(vec![("$ref", Json::str("#/components/schemas/Timestamp"))])
}

fn codec_ref(name: &str) -> Json {
    Json::obj(vec![(
        "$ref",
        Json::str(&format!("#/components/schemas/{name}")),
    )])
}

fn type_null() -> Json {
    Json::obj(vec![("type", Json::str("null"))])
}

fn one_of_null(inner: Json) -> Json {
    Json::obj(vec![("oneOf", Json::Array(vec![inner, type_null()]))])
}

fn string_null() -> Json {
    Json::obj(vec![(
        "type",
        Json::Array(vec![Json::str("string"), Json::str("null")]),
    )])
}

fn string_min(min_length: u32) -> Json {
    Json::obj(vec![
        ("type", Json::str("string")),
        ("minLength", Json::Number(min_length.to_string())),
    ])
}

fn string_enum(values: &[&str]) -> Json {
    Json::obj(vec![
        ("type", Json::str("string")),
        ("enum", Json::arr_str(values)),
    ])
}

fn iso_date() -> Json {
    Json::obj(vec![
        ("type", Json::str("string")),
        ("format", Json::str("date")),
    ])
}

fn version_int64() -> Json {
    Json::obj(vec![
        ("type", Json::str("integer")),
        ("format", Json::str("int64")),
        ("minimum", Json::Number("1".to_owned())),
    ])
}

fn properties(fields: Vec<(&str, Json)>) -> Json {
    Json::obj(fields)
}

fn open_object(required: &[&str], props: Vec<(&str, Json)>) -> Json {
    let mut fields = vec![("type", Json::str("object"))];
    if !required.is_empty() {
        fields.push(("required", Json::arr_str(required)));
    }
    fields.push(("properties", properties(props)));
    fields.push(("additionalProperties", Json::Bool(true)));
    Json::obj(fields)
}

fn input_object(description: &str, required: &[&str], props: Vec<(&str, Json)>) -> Json {
    Json::obj(vec![
        ("type", Json::str("object")),
        ("additionalProperties", Json::Bool(false)),
        ("description", Json::str(description)),
        ("required", Json::arr_str(required)),
        ("properties", properties(props)),
    ])
}

fn head_object(description: &str, required: &[&str], props: Vec<(&str, Json)>) -> Json {
    Json::obj(vec![
        ("type", Json::str("object")),
        ("description", Json::str(description)),
        ("required", Json::arr_str(required)),
        ("properties", properties(props)),
    ])
}

fn company_attributes() -> Json {
    open_object(
        &["legal_name"],
        vec![("legal_name", string_min(1)), ("reg_no", string_null())],
    )
}

fn org_unit_attributes() -> Json {
    open_object(
        &["name", "kind"],
        vec![
            ("name", string_min(1)),
            ("kind", string_enum(&["site", "department", "team"])),
            ("parent_id", one_of_null(uuid_ref())),
        ],
    )
}

fn job_position_attributes() -> Json {
    open_object(&["title"], vec![("title", string_min(1))])
}

fn person_attributes() -> Json {
    open_object(
        &[],
        vec![
            ("legal_name", string_null()),
            ("display_name", string_null()),
        ],
    )
}

fn with_desc(mut schema: Json, description: &str) -> Json {
    let Json::Object(ref mut fields) = schema else {
        return schema;
    };
    fields.push(("description".to_owned(), Json::str(description)));
    schema
}

/// Property bags generated from the DTO structs above. Name order is stable.
pub(super) fn dto_schema_bags() -> Result<Vec<(String, Json)>, SemanticError> {
    let bags = vec![
        (
            "CompanyReviseInput",
            input_object(
                "Typed params for `company.revise` (`CompanyQuery`). The tenant is `org_id` on the command envelope, not this bag. Preflight requires non-empty `legal_name`.",
                &["attributes"],
                vec![("attributes", company_attributes())],
            ),
        ),
        (
            "OrganizationCreateOrgUnitInput",
            input_object(
                "Typed params for `organization.create_org_unit` (`OrgUnitQuery::Create`). Write preflight requires non-empty `name` and `kind` ∈ {site, department, team}.",
                &["attributes"],
                vec![
                    ("source", one_of_null(codec_ref("OrgUnitSourceBinding"))),
                    ("attributes", org_unit_attributes()),
                ],
            ),
        ),
        (
            "OrganizationReviseOrgUnitInput",
            input_object(
                "Typed params for `organization.revise_org_unit` (`OrgUnitQuery::Revise`).",
                &["org_unit_id", "attributes"],
                vec![
                    ("org_unit_id", uuid_ref()),
                    ("source", one_of_null(codec_ref("OrgUnitSourceBinding"))),
                    ("attributes", org_unit_attributes()),
                ],
            ),
        ),
        (
            "OrganizationCreateJobPositionInput",
            input_object(
                "Typed params for `organization.create_job_position` (`JobPositionQuery::Create`). The named OrgUnit must already exist. Write preflight requires non-empty `title`.",
                &["org_unit_id", "attributes"],
                vec![
                    ("org_unit_id", uuid_ref()),
                    ("attributes", job_position_attributes()),
                ],
            ),
        ),
        (
            "OrganizationReviseJobPositionInput",
            input_object(
                "Typed params for `organization.revise_job_position` (`JobPositionQuery::Revise`). Null `org_unit_id` leaves the position where it is; a UUID moves it.",
                &["job_position_id", "attributes"],
                vec![
                    ("job_position_id", uuid_ref()),
                    ("org_unit_id", one_of_null(uuid_ref())),
                    ("attributes", job_position_attributes()),
                ],
            ),
        ),
        (
            "PeopleCreatePersonInput",
            input_object(
                "Typed params for `people.create_person` (`PersonQuery::Create`). When `employee_id` is set (trusted uniquely resolved), the new person's id IS that employee id. Head projection stays four fields; this bag is not a license to publish phone/salary/bank_account/rrn on GET.",
                &["attributes"],
                vec![
                    ("employee_id", one_of_null(uuid_ref())),
                    ("attributes", person_attributes()),
                ],
            ),
        ),
        (
            "PeopleRevisePersonInput",
            input_object(
                "Typed params for `people.revise_person` (`PersonQuery::Revise`). Optional `employee_id` binds a further employee record to the same natural person.",
                &["person_id", "attributes"],
                vec![
                    ("person_id", uuid_ref()),
                    ("employee_id", one_of_null(uuid_ref())),
                    ("attributes", person_attributes()),
                ],
            ),
        ),
        (
            "HrAppointInput",
            input_object(
                "Typed params for `hr.appoint` (`EmploymentQuery::Appoint`). Opens a head at `valid_from`. Four-eyes is natural-person. The legacy employees row is not rewritten by this command.",
                &["employee_id", "valid_from", "attributes"],
                vec![
                    ("employee_id", uuid_ref()),
                    ("valid_from", timestamp_ref()),
                    ("attributes", codec_ref("EmploymentAttributesInput")),
                ],
            ),
        ),
        (
            "HrPromoteInput",
            input_object(
                "Typed params for `hr.promote` (`EmploymentQuery::Promote`). Appends a revision and carries the new state onto the legacy head. Four-eyes is natural-person.",
                &["employment_id", "valid_from", "attributes"],
                vec![
                    ("employment_id", uuid_ref()),
                    ("valid_from", timestamp_ref()),
                    ("attributes", codec_ref("EmploymentAttributesInput")),
                ],
            ),
        ),
        (
            "HrTransferInput",
            input_object(
                "Typed params for `hr.transfer` (`EmploymentQuery::Transfer`). Same shape as promote; the receipt records a different dispatch target. Four-eyes is natural-person.",
                &["employment_id", "valid_from", "attributes"],
                vec![
                    ("employment_id", uuid_ref()),
                    ("valid_from", timestamp_ref()),
                    ("attributes", codec_ref("EmploymentAttributesInput")),
                ],
            ),
        ),
        (
            "PayrollCreateRunInput",
            input_object(
                "Typed params for `payroll.create_run` (`PayRunQuery::CreateRun`). Stages a draft under a caller-chosen source label. `payable` is not an input; stored calculations stay false. Payment execution remains HOLD.",
                &["run_id", "period_start", "period_end"],
                vec![
                    ("run_id", uuid_ref()),
                    ("period_start", iso_date()),
                    ("period_end", iso_date()),
                    ("connector", string_null()),
                    ("job", string_null()),
                ],
            ),
        ),
        (
            "PayrollSubmitRunInput",
            input_object(
                "Typed params for `payroll.submit_run` (`PayRunQuery::SubmitRun`). `CALCULATED → SUBMITTED`, refused while any exception is open. `payable` stays false.",
                &["run_id"],
                vec![("run_id", uuid_ref())],
            ),
        ),
        (
            "PayrollDecideRunInput",
            input_object(
                "Typed params for `payroll.decide_run` (`PayRunQuery::DecideRun`). `SUBMITTED → APPROVED|REJECTED`, refused when the decider is the submitter. Distinct from path-bound `DecidePayrollRunRequest` (run_id is in this bag). `payable` stays false. Payment execution remains HOLD.",
                &["run_id", "decision"],
                vec![
                    ("run_id", uuid_ref()),
                    ("decision", string_enum(&["APPROVE", "REJECT"])),
                    (
                        "reason",
                        with_desc(
                            string_null(),
                            "Required for REJECT at the lifecycle writer (blank reason is 422).",
                        ),
                    ),
                ],
            ),
        ),
        (
            "EmploymentAttributesInput",
            input_object(
                "Write bag for `hr.appoint` / `hr.promote` / `hr.transfer` (`EmploymentAttributes`). Not the Employment Head. `company` is a non-blank string; `employment_status` is the 0066 CHECK set. UUID refs must not be nil.",
                &["company", "employment_status"],
                vec![
                    ("company", string_min(1)),
                    ("org_unit_id", one_of_null(uuid_ref())),
                    ("job_position_id", one_of_null(uuid_ref())),
                    (
                        "employment_status",
                        string_enum(&["ACTIVE", "EXITED", "UNKNOWN"]),
                    ),
                ],
            ),
        ),
        (
            "OrgUnitSourceBinding",
            input_object(
                "Optional legacy source binding on OrgUnit create/revise. `kind` and `id` are TEXT; the closed set of kinds is not yet enumerable and legacy identifiers are not all UUIDs.",
                &["kind", "id"],
                vec![("kind", string_min(1)), ("id", string_min(1))],
            ),
        ),
        (
            "Company",
            head_object(
                "Canonical Company head for one tenant. The tenant IS the company (`org_id`). `legal_name` and `reg_no` are read from the latest `company_revisions.attributes`; they are never copied from provisioning-owned `organizations.name`. Write preflight requires a non-empty string `legal_name`; an omitted key is JSON null on the Head. `founded_on` and `registration_number` are company_conformance fixture keys, not this port. `org_id` is the RLS cell, not a Palantir link. Temporal slices other than revision `version` remain HOLD.",
                &["org_id", "version", "legal_name", "reg_no"],
                vec![
                    ("org_id", uuid_ref()),
                    (
                        "legal_name",
                        with_desc(
                            string_null(),
                            "Title property. Required non-empty on write; omitted key is null on read.",
                        ),
                    ),
                    (
                        "reg_no",
                        with_desc(
                            string_null(),
                            "Optional `reg_no` on the latest revision. Not `registration_number`.",
                        ),
                    ),
                    ("version", version_int64()),
                ],
            ),
        ),
        (
            "OrgUnit",
            head_object(
                "Canonical OrgUnit head. `name` and `parent_id` are parsed from the latest `org_unit_revisions.attributes`; neither is a column on `org_units`. Sites, regions, and branches stay the operational auth spine and are not OrgUnits. OrgUnit kinds Site/Department/Team are write-preflight on revision attributes (`kind` ∈ {site, department, team}) and are not a Head column. Temporal slices other than revision `version` remain HOLD.",
                &["id", "version", "name", "parent_id"],
                vec![
                    ("id", uuid_ref()),
                    (
                        "name",
                        with_desc(
                            string_null(),
                            "Title property. Required non-empty on write; omitted key is null on read.",
                        ),
                    ),
                    (
                        "parent_id",
                        with_desc(
                            one_of_null(uuid_ref()),
                            "Optional parent OrgUnit. Root units send JSON null.",
                        ),
                    ),
                    ("version", version_int64()),
                ],
            ),
        ),
        (
            "JobPosition",
            head_object(
                "Canonical JobPosition head. Distinct from recruiting postings and from `employees.position` TEXT. `org_unit_id` is a foreign key to an existing OrgUnit. `attributes` is the latest revision bag; write preflight requires a non-empty string `title` (catalog JOB_POSITION_TITLE). Other attribute keys are not a published closed set. Temporal slices other than revision `version` remain HOLD.",
                &["job_position_id", "org_unit_id", "version", "attributes"],
                vec![
                    ("job_position_id", uuid_ref()),
                    ("org_unit_id", uuid_ref()),
                    ("version", version_int64()),
                    (
                        "attributes",
                        Json::obj(vec![
                            ("type", Json::str("object")),
                            ("additionalProperties", Json::Bool(true)),
                            (
                                "description",
                                Json::str(
                                    "Latest revision JSON object. Write requires non-empty `title`.",
                                ),
                            ),
                        ]),
                    ),
                ],
            ),
        ),
        (
            "Person",
            head_object(
                "Canonical Person head. Closed four-field projection of the latest `person_revisions` row. `get`/`list` never copy the attributes object onto the wire, so stored `phone` / `salary` / `bank_account` / `rrn` cannot appear. Trusted directory create mints `person_id = employee_id`. No outgoing Head FK; Employment.person_id is the reverse. Temporal slices other than revision `version` remain HOLD.",
                &["id", "version", "display_name", "legal_name"],
                vec![
                    ("id", uuid_ref()),
                    (
                        "display_name",
                        with_desc(
                            string_null(),
                            "Optional `display_name` attribute. Never invented from `legal_name`.",
                        ),
                    ),
                    ("legal_name", string_null()),
                    ("version", version_int64()),
                ],
            ),
        ),
        (
            "Employment",
            head_object(
                "Canonical Employment head. Current get/list still return the *open* head (`employment_heads.valid_to IS NULL`). GET /api/v1/employments/{id}?as_of= reconstructs the slice effective at an RFC3339 instant using the same half-open predicate as ontology instance GET (`valid_from <= as_of < coalesce(valid_to, ∞)`); absent as_of = current open head. Closed (EXITED) windows are omitted by current get/list and by as_of at or after valid_to. `org_unit_id` / `job_position_id` are canonical UUIDs from the effective revision, not free-text team or title labels. `person_id` is the unique source-binding → person-binding path; ambiguous or unbound identities are null, never invented from `employee_id`. `appointed_on` is the head's opening `valid_from`. Write attributes also carry `company` and `employment_status` in {ACTIVE, EXITED, UNKNOWN}; those write fields are not this Head. The Head never includes phone, salary, bank_account, rrn, or base_pay. from/to range queries, corrections vs new slice, and delta transmission remain HOLD. EmployeeDetail as_of remains HOLD.",
                &[
                    "id",
                    "appointed_on",
                    "person_id",
                    "org_unit_id",
                    "job_position_id",
                ],
                vec![
                    ("id", uuid_ref()),
                    ("person_id", one_of_null(uuid_ref())),
                    ("org_unit_id", one_of_null(uuid_ref())),
                    ("job_position_id", one_of_null(uuid_ref())),
                    ("appointed_on", timestamp_ref()),
                ],
            ),
        ),
        (
            "PayRun",
            head_object(
                "Canonical PayRun scaffold over existing `payroll_draft_runs`. Not SAP-complete. Draft calculate is admitted on POST /api/v1/payroll/runs/{id}/calculate (not a DispatchTarget). `payable` is always false. Payment execution, Korea compliance conclusions, and wage-statement legal sign-off remain HOLD. REST list/detail DTOs remain `PayrollRunSummary` / `PayrollRunDetail`. No outgoing Head FK; the run is Company-scoped via RLS `org_id`, which is the cell, not a Palantir link. Temporal slices other than draft status remain HOLD.",
                &[
                    "id",
                    "period_start",
                    "period_end",
                    "source_label",
                    "status",
                    "payable",
                ],
                vec![
                    ("id", uuid_ref()),
                    ("period_start", iso_date()),
                    ("period_end", iso_date()),
                    (
                        "source_label",
                        Json::obj(vec![("type", Json::str("string"))]),
                    ),
                    (
                        "status",
                        with_desc(
                            string_enum(&[
                                "STAGED",
                                "BLOCKED_LEGAL_GATE",
                                "READY_FOR_REVIEW",
                                "ATTENDANCE_CLOSED",
                                "CALCULATING",
                                "CALCULATED",
                                "SUBMITTED",
                                "REJECTED",
                                "APPROVED",
                                "DISBURSEMENT_SCHEDULED",
                                "PAID",
                                "ISSUED",
                                "VOID",
                            ]),
                            "`payroll_draft_runs.status`. Same closed set as `PayrollRunSummary`.",
                        ),
                    ),
                    (
                        "payable",
                        Json::obj(vec![
                            ("type", Json::str("boolean")),
                            ("const", Json::Bool(false)),
                            (
                                "description",
                                Json::str(
                                    "Always false until the 노무사/세무사 release gate flips stored calculation rows. Payment execution remains HOLD. Not a payable payroll object.",
                                ),
                            ),
                        ]),
                    ),
                ],
            ),
        ),
    ];
    if bags.len() != GENERATED_SCHEMA_COUNT {
        return Err(SemanticError(format!(
            "DTO inventory generated {} schemas, expected {GENERATED_SCHEMA_COUNT}",
            bags.len()
        )));
    }
    Ok(bags
        .into_iter()
        .map(|(name, schema)| (name.to_owned(), schema))
        .collect())
}
