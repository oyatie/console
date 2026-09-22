//! Checked current Company material and selectors, never authentication proof.
//! The persistence scope retains current authority through its final check.
pub use console_identity_domain::AccountId;
use console_kernel_core::OrgId;
use serde::{Deserialize, Deserializer, de};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

const INITIAL_MANIFEST: &str = "0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935";

pub mod business;
mod native_business;
pub mod workflow;
pub use native_business::{
    CurrentNativeBootstrapAuthority, CurrentPayrollReadAuthority, NativeBootstrapProjectionRow,
    NativeBootstrapRequestV1, NativePayrollReadProjectionRow, NativePolicySourceBinding,
};
mod reads;
pub use reads::{
    CompanyContextCandidates, CompanyContextView, CompanyIdentityView, CompanyInitialCeilingView,
    CompanyPolicyScope, CompanyPolicyStore, CompanyPolicyView, discover_company_context,
    discover_company_contexts, read_company_identity, read_company_policy,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompanyPolicyError {
    AuthenticationInvalid,
    NotFound,
    Conflict,
    MaterialUnavailable,
    EvaluatorUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompanyPolicyDecision {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitialCompanyAction {
    Discover,
    ReadIdentity,
    ReadPolicy,
    AssignIdentity,
    RevokeIdentity,
}

impl InitialCompanyAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Discover => "context.discover",
            Self::ReadIdentity => "company.identity.read",
            Self::ReadPolicy => "company.policy.read",
            Self::AssignIdentity => "company.policy.assign",
            Self::RevokeIdentity => "company.policy.revoke",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRef {
    org_id: OrgId,
    action_type_id: Uuid,
    object_type_id: Uuid,
    registration_revision: u64,
    manifest_digest: [u8; 32],
}

impl ActionRef {
    pub fn new(
        org_id: OrgId,
        action_type_id: Uuid,
        object_type_id: Uuid,
        registration_revision: u64,
        manifest_digest: [u8; 32],
    ) -> Result<Self, CompanyPolicyError> {
        company_id(org_id)?;
        nonnil(action_type_id)?;
        nonnil(object_type_id)?;
        revision(registration_revision)?;
        Ok(Self {
            org_id,
            action_type_id,
            object_type_id,
            registration_revision,
            manifest_digest,
        })
    }
    pub const fn org_id(&self) -> OrgId {
        self.org_id
    }
    pub const fn action_type_id(&self) -> Uuid {
        self.action_type_id
    }
    pub const fn object_type_id(&self) -> Uuid {
        self.object_type_id
    }
    pub const fn registration_revision(&self) -> u64 {
        self.registration_revision
    }
    pub const fn manifest_digest(&self) -> &[u8; 32] {
        &self.manifest_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PropertyRef {
    org_id: OrgId,
    object_type_id: Uuid,
    property_id: Uuid,
    schema_revision: u64,
}

impl PropertyRef {
    pub fn new(
        org_id: OrgId,
        object_type_id: Uuid,
        property_id: Uuid,
        schema_revision: u64,
    ) -> Result<Self, CompanyPolicyError> {
        company_id(org_id)?;
        nonnil(object_type_id)?;
        nonnil(property_id)?;
        revision(schema_revision)?;
        Ok(Self {
            org_id,
            object_type_id,
            property_id,
            schema_revision,
        })
    }
    pub const fn org_id(&self) -> OrgId {
        self.org_id
    }
    pub const fn object_type_id(&self) -> Uuid {
        self.object_type_id
    }
    pub const fn property_id(&self) -> Uuid {
        self.property_id
    }
    pub const fn schema_revision(&self) -> u64 {
        self.schema_revision
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompanyPolicyRequest {
    requested_company: OrgId,
    object_type_id: Uuid,
    resource_object_id: Uuid,
    action: ActionRef,
    requested_properties: Vec<PropertyRef>,
}

impl CompanyPolicyRequest {
    /// Selectors are syntactically checked here; only the decision port grants access.
    pub fn new(
        requested_company: OrgId,
        object_type_id: Uuid,
        resource_object_id: Uuid,
        action: ActionRef,
        requested_properties: Vec<PropertyRef>,
    ) -> Result<Self, CompanyPolicyError> {
        company_id(requested_company)?;
        nonnil(object_type_id)?;
        nonnil(resource_object_id)?;
        if requested_properties.len() > 8 || !sorted_unique(&requested_properties) {
            return Err(CompanyPolicyError::MaterialUnavailable);
        }
        Ok(Self {
            requested_company,
            object_type_id,
            resource_object_id,
            action,
            requested_properties,
        })
    }
    pub const fn requested_company(&self) -> OrgId {
        self.requested_company
    }
    pub const fn object_type_id(&self) -> Uuid {
        self.object_type_id
    }
    pub const fn object_id(&self) -> Uuid {
        self.resource_object_id
    }
    pub fn action(&self) -> &ActionRef {
        &self.action
    }
    pub fn requested_properties(&self) -> &[PropertyRef] {
        &self.requested_properties
    }
}

pub struct CompanyProjectionRow {
    pub company_epoch: i64,
    pub context_generation: i64,
    pub assignment_id: Uuid,
    pub assignment_revision: i64,
    pub role_id: Uuid,
    pub role_revision: i64,
    pub registered_clauses: String,
    pub company_name: String,
    pub company_slug: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentCompanyClause {
    action: ActionRef,
    action_kind: InitialCompanyAction,
    fields: Vec<PropertyRef>,
    valid_from: OffsetDateTime,
    valid_until: Option<OffsetDateTime>,
    delegable: bool,
}

impl CurrentCompanyClause {
    pub fn action(&self) -> &ActionRef {
        &self.action
    }
    pub const fn action_kind(&self) -> InitialCompanyAction {
        self.action_kind
    }
    pub fn fields(&self) -> &[PropertyRef] {
        &self.fields
    }
    pub const fn valid_from(&self) -> OffsetDateTime {
        self.valid_from
    }
    pub const fn valid_until(&self) -> Option<OffsetDateTime> {
        self.valid_until
    }
    pub const fn delegable(&self) -> bool {
        self.delegable
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentCompanyAuthority {
    account: AccountId,
    company: OrgId,
    company_epoch: u64,
    current_policy_receipt_id: Option<Uuid>,
    context_generation: u64,
    assignment_id: Uuid,
    assignment_revision: u64,
    role_id: Uuid,
    role_revision: u64,
    observed_at: OffsetDateTime,
    clauses: Vec<CurrentCompanyClause>,
    company_name: String,
    company_slug: String,
}

impl CurrentCompanyAuthority {
    /// Convert only the initial 7-clause/16-field graph emitted by the SQL owner.
    /// Account, Company and observation time must come from the retained store scope.
    pub fn from_initial_projection(
        account: AccountId,
        company: OrgId,
        observed_at: OffsetDateTime,
        row: CompanyProjectionRow,
    ) -> Result<Self, CompanyPolicyError> {
        if row.company_epoch != 1 {
            return Err(CompanyPolicyError::MaterialUnavailable);
        }
        Self::from_current_projection(account, company, observed_at, row, None)
    }

    /// Preserve the current epoch and receipt while parsing the unchanged birth
    /// assignment. The retained store must verify the receipt's actual custody.
    pub fn from_current_projection(
        account: AccountId,
        company: OrgId,
        observed_at: OffsetDateTime,
        row: CompanyProjectionRow,
        current_policy_receipt_id: Option<Uuid>,
    ) -> Result<Self, CompanyPolicyError> {
        if row.company_epoch < 1
            || (row.company_epoch == 1) != current_policy_receipt_id.is_none()
            || current_policy_receipt_id.is_some_and(|id| id.is_nil())
        {
            return Err(CompanyPolicyError::MaterialUnavailable);
        }
        Self::parse_birth_assignment(
            account,
            company,
            observed_at,
            row,
            current_policy_receipt_id,
        )
    }

    fn parse_birth_assignment(
        account: AccountId,
        company: OrgId,
        observed_at: OffsetDateTime,
        row: CompanyProjectionRow,
        current_policy_receipt_id: Option<Uuid>,
    ) -> Result<Self, CompanyPolicyError> {
        use CompanyPolicyError::MaterialUnavailable;
        company_id(company)?;
        nonnil(row.assignment_id)?;
        nonnil(row.role_id)?;
        if row.assignment_revision != 1
            || row.role_revision != 1
            || row.context_generation < 1
            || row.registered_clauses.len() > 32768
        {
            return Err(MaterialUnavailable);
        }
        super::company::validate_name_slug(&row.company_name, &row.company_slug)
            .map_err(|_| MaterialUnavailable)?;
        let wire: Vec<Object<WireClause>> =
            serde_json::from_str(&row.registered_clauses).map_err(|_| MaterialUnavailable)?;
        if wire.len() != 7 {
            return Err(MaterialUnavailable);
        }
        let kinds = [
            InitialCompanyAction::Discover,
            InitialCompanyAction::ReadIdentity,
            InitialCompanyAction::ReadPolicy,
            InitialCompanyAction::AssignIdentity,
            InitialCompanyAction::RevokeIdentity,
            InitialCompanyAction::Discover,
            InitialCompanyAction::ReadIdentity,
        ];
        let mut clauses = Vec::with_capacity(7);
        for (index, (Object(c), kind)) in wire.into_iter().zip(kinds).enumerate() {
            let Object(a) = c.action;
            if c.kind != "COMPANY_CAPABILITY_CLAUSE_V1"
                || c.resource.0.kind != "COMPANY"
                || canonical_uuid(&c.resource.0.org_id)? != *company.as_uuid()
                || canonical_uuid(&a.org_id)? != *company.as_uuid()
                || a.manifest_digest != INITIAL_MANIFEST
                || c.valid_until.is_some()
                || c.delegable != (index >= 5)
                || c.fields.len() != [2, 2, 8, 0, 0, 2, 2][index]
            {
                return Err(MaterialUnavailable);
            }
            let mut digest = [0; 32];
            for (i, byte) in digest.iter_mut().enumerate() {
                *byte = u8::from_str_radix(&a.manifest_digest[i * 2..i * 2 + 2], 16)
                    .map_err(|_| MaterialUnavailable)?;
            }
            let action = ActionRef::new(
                company,
                canonical_uuid(&a.action_type_id)?,
                canonical_uuid(&a.object_type_id)?,
                canonical_revision(&a.registration_revision)?,
                digest,
            )?;
            if action.registration_revision != 1 {
                return Err(MaterialUnavailable);
            }
            let mut fields = Vec::with_capacity(c.fields.len());
            for Object(f) in c.fields {
                let field = PropertyRef::new(
                    OrgId::from_uuid(canonical_uuid(&f.org_id)?),
                    canonical_uuid(&f.object_type_id)?,
                    canonical_uuid(&f.property_id)?,
                    canonical_revision(&f.schema_revision)?,
                )?;
                if field.org_id != company
                    || field.object_type_id != action.object_type_id
                    || field.schema_revision != 1
                {
                    return Err(MaterialUnavailable);
                }
                fields.push(field);
            }
            let valid_from = canonical_time(&c.valid_from)?;
            if !sorted_unique(&fields) || valid_from > observed_at {
                return Err(MaterialUnavailable);
            }
            clauses.push(CurrentCompanyClause {
                action,
                action_kind: kind,
                fields,
                valid_from,
                valid_until: None,
                delegable: c.delegable,
            });
        }
        let company_type = clauses[0].action.object_type_id;
        let assignment_type = clauses[2].action.object_type_id;
        if company_type == assignment_type
            || clauses[1].action.object_type_id != company_type
            || clauses[3].action.object_type_id != assignment_type
            || clauses[4].action.object_type_id != assignment_type
            || clauses[0].fields != clauses[1].fields
            || clauses
                .iter()
                .any(|c| c.valid_from != clauses[0].valid_from)
        {
            return Err(MaterialUnavailable);
        }
        for i in 0..5 {
            if clauses[..i]
                .iter()
                .any(|c| c.action.action_type_id == clauses[i].action.action_type_id)
            {
                return Err(MaterialUnavailable);
            }
        }
        for (a, b) in [(0, 5), (1, 6)] {
            if clauses[a].action != clauses[b].action || clauses[a].fields != clauses[b].fields {
                return Err(MaterialUnavailable);
            }
        }
        Ok(Self {
            account,
            company,
            company_epoch: u64::try_from(row.company_epoch).map_err(|_| MaterialUnavailable)?,
            current_policy_receipt_id,
            context_generation: u64::try_from(row.context_generation)
                .map_err(|_| MaterialUnavailable)?,
            assignment_id: row.assignment_id,
            assignment_revision: 1,
            role_id: row.role_id,
            role_revision: 1,
            observed_at,
            clauses,
            company_name: row.company_name,
            company_slug: row.company_slug,
        })
    }
    pub const fn account(&self) -> AccountId {
        self.account
    }
    pub const fn company(&self) -> OrgId {
        self.company
    }
    pub const fn epoch(&self) -> u64 {
        self.company_epoch
    }
    pub const fn current_policy_receipt_id(&self) -> Option<Uuid> {
        self.current_policy_receipt_id
    }
    pub const fn context_generation(&self) -> u64 {
        self.context_generation
    }
    pub const fn assignment_id(&self) -> Uuid {
        self.assignment_id
    }
    pub const fn assignment_revision(&self) -> u64 {
        self.assignment_revision
    }
    pub const fn role_id(&self) -> Uuid {
        self.role_id
    }
    pub const fn role_revision(&self) -> u64 {
        self.role_revision
    }
    pub const fn observed_at(&self) -> OffsetDateTime {
        self.observed_at
    }
    pub fn clauses(&self) -> &[CurrentCompanyClause] {
        &self.clauses
    }
    pub fn name(&self) -> &str {
        &self.company_name
    }
    pub fn slug(&self) -> &str {
        &self.company_slug
    }
}

pub trait CompanyPolicyDecisionPort: Send + Sync {
    fn decide_native_bootstrap(
        &self,
        authority: &CurrentNativeBootstrapAuthority,
        request: &NativeBootstrapRequestV1,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError>;
    fn decide_native_payroll_collection(
        &self,
        authority: &CurrentPayrollReadAuthority,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError>;

    fn decide(
        &self,
        authority: &CurrentCompanyAuthority,
        request: &CompanyPolicyRequest,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError>;
}

fn nonnil(value: Uuid) -> Result<(), CompanyPolicyError> {
    if value.is_nil() {
        Err(CompanyPolicyError::MaterialUnavailable)
    } else {
        Ok(())
    }
}
fn company_id(value: OrgId) -> Result<(), CompanyPolicyError> {
    nonnil(*value.as_uuid())?;
    if value == OrgId::platform() {
        Err(CompanyPolicyError::MaterialUnavailable)
    } else {
        Ok(())
    }
}
fn revision(value: u64) -> Result<(), CompanyPolicyError> {
    if value == 0 || value > i64::MAX as u64 {
        Err(CompanyPolicyError::MaterialUnavailable)
    } else {
        Ok(())
    }
}
fn canonical_uuid(raw: &str) -> Result<Uuid, CompanyPolicyError> {
    let value = Uuid::parse_str(raw).map_err(|_| CompanyPolicyError::MaterialUnavailable)?;
    nonnil(value)?;
    if value.to_string() != raw {
        return Err(CompanyPolicyError::MaterialUnavailable);
    }
    Ok(value)
}
fn canonical_revision(raw: &str) -> Result<u64, CompanyPolicyError> {
    let value = raw
        .parse()
        .map_err(|_| CompanyPolicyError::MaterialUnavailable)?;
    revision(value)?;
    if value.to_string() != raw {
        return Err(CompanyPolicyError::MaterialUnavailable);
    }
    Ok(value)
}
fn canonical_time(raw: &str) -> Result<OffsetDateTime, CompanyPolicyError> {
    let value = OffsetDateTime::parse(raw, &Rfc3339)
        .map_err(|_| CompanyPolicyError::MaterialUnavailable)?;
    let canonical = format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:06}Z",
        value.year(),
        u8::from(value.month()),
        value.day(),
        value.hour(),
        value.minute(),
        value.second(),
        value.microsecond()
    );
    if raw != canonical || value.offset() != time::UtcOffset::UTC || value.nanosecond() % 1000 != 0
    {
        return Err(CompanyPolicyError::MaterialUnavailable);
    }
    Ok(value)
}
fn sorted_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

// Serde structs also accept sequences by default. Force maps at every nested
// object boundary, preserving duplicate-member evidence for the closed structs.
struct Object<T>(T);
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Object<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Map<T>(std::marker::PhantomData<T>);
        impl<'de, T: Deserialize<'de>> de::Visitor<'de> for Map<T> {
            type Value = Object<T>;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a closed object")
            }
            fn visit_map<M: de::MapAccess<'de>>(self, map: M) -> Result<Self::Value, M::Error> {
                T::deserialize(de::value::MapAccessDeserializer::new(map)).map(Object)
            }
        }
        d.deserialize_map(Map(std::marker::PhantomData))
    }
}
fn required_nullable<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Option::<String>::deserialize(d)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireClause {
    kind: String,
    action: Object<WireAction>,
    resource: Object<WireResource>,
    fields: Vec<Object<WireProperty>>,
    valid_from: String,
    #[serde(deserialize_with = "required_nullable")]
    valid_until: Option<String>,
    delegable: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireAction {
    org_id: String,
    action_type_id: String,
    object_type_id: String,
    registration_revision: String,
    manifest_digest: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireResource {
    kind: String,
    org_id: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireProperty {
    org_id: String,
    object_type_id: String,
    property_id: String,
    schema_revision: String,
}
