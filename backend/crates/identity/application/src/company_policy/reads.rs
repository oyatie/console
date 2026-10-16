//! Authorized views are released only after the retained read scope finishes.
use super::{
    AccountId, ActionRef, CompanyPolicyDecision, CompanyPolicyDecisionPort, CompanyPolicyError,
    CompanyPolicyRequest, CurrentCompanyAuthority, InitialCompanyAction, PropertyRef, company_id,
    sorted_unique,
};
use console_kernel_core::OrgId;
use std::future::Future;

pub trait CompanyPolicyScope {
    fn authority(&self) -> Option<&CurrentCompanyAuthority>;
    /// Recheck current credentials and commit the retained transaction.
    /// Dropping a scope must roll back; it never releases a successful view.
    fn finish(self) -> impl Future<Output = Result<(), CompanyPolicyError>> + Send;
}

pub trait CompanyPolicyStore {
    type Credentials: Sync;
    type Scope<'a>: CompanyPolicyScope + Send
    where
        Self: 'a;

    fn lock_current<'a>(
        &'a self,
        credentials: &'a Self::Credentials,
        company: OrgId,
    ) -> impl Future<Output = Result<Self::Scope<'a>, CompanyPolicyError>> + Send;

    /// Finish Account-only enumeration before acquiring any Company scope.
    fn enumerate_company_candidates(
        &self,
        credentials: &Self::Credentials,
    ) -> impl Future<Output = Result<CompanyContextCandidates, CompanyPolicyError>> + Send;

    fn check_context_generation(
        &self,
        credentials: &Self::Credentials,
        expected_generation: u64,
    ) -> impl Future<Output = Result<(), CompanyPolicyError>> + Send;
}

pub struct CompanyContextCandidates {
    generation: u64,
    companies: Vec<OrgId>,
}

impl CompanyContextCandidates {
    /// Bounded hints only. The store must prove their authenticated provenance.
    pub fn new(generation: u64, companies: Vec<OrgId>) -> Result<Self, CompanyPolicyError> {
        if !(1..=257).contains(&generation) || companies.len() > 256 || !sorted_unique(&companies) {
            return Err(CompanyPolicyError::MaterialUnavailable);
        }
        for company in &companies {
            company_id(*company)?;
        }
        Ok(Self {
            generation,
            companies,
        })
    }
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    pub fn companies(&self) -> &[OrgId] {
        &self.companies
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompanyContextView {
    pub org_id: OrgId,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompanyIdentityView {
    pub org_id: OrgId,
    pub name: String,
    pub slug: String,
    pub show_policy_navigation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompanyPolicyView {
    pub initial_ceiling: CompanyInitialCeilingView,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompanyInitialCeilingView {
    pub org_id: OrgId,
    pub account_id: AccountId,
    pub action_keys: Vec<&'static str>,
    pub action_refs: Vec<ActionRef>,
    pub delegable_action_keys: Vec<&'static str>,
    pub delegable_action_refs: Vec<ActionRef>,
    // Keys are a display roster; sorted refs do not establish key-to-UUID pairs.
    pub company_property_keys: Vec<&'static str>,
    pub company_property_refs: Vec<PropertyRef>,
    pub delegable_company_property_keys: Vec<&'static str>,
    pub delegable_company_property_refs: Vec<PropertyRef>,
    pub future_registrations: bool,
    pub group_control: bool,
}

async fn read_current<S: CompanyPolicyStore, T>(
    store: &S,
    credentials: &S::Credentials,
    company: OrgId,
    project: impl FnOnce(&CurrentCompanyAuthority) -> Result<T, CompanyPolicyError>,
) -> Result<T, CompanyPolicyError> {
    company_id(company)?;
    let scope = store.lock_current(credentials, company).await?;
    let result = match scope.authority() {
        Some(authority) if authority.company() == company => project(authority),
        Some(_) => return Err(CompanyPolicyError::MaterialUnavailable),
        None => Err(CompanyPolicyError::NotFound),
    };
    // Healthy denial/absence must still pass final authentication. Faults have
    // no authorized projection and drop the scope without committing it.
    if result.is_ok() || matches!(result, Err(CompanyPolicyError::NotFound)) {
        scope.finish().await?;
    }
    result
}

fn decide<P: CompanyPolicyDecisionPort + ?Sized>(
    policy: &P,
    authority: &CurrentCompanyAuthority,
    kind: InitialCompanyAction,
    navigation_only: bool,
) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
    let clause = authority
        .clauses()
        .iter()
        .find(|c| !c.delegable() && c.action_kind() == kind)
        .ok_or(CompanyPolicyError::MaterialUnavailable)?;
    let object = match kind {
        InitialCompanyAction::Discover | InitialCompanyAction::ReadIdentity => {
            *authority.company().as_uuid()
        }
        _ => authority.assignment_id(),
    };
    let request = CompanyPolicyRequest::new(
        authority.company(),
        clause.action().object_type_id(),
        object,
        clause.action().clone(),
        if navigation_only {
            vec![]
        } else {
            clause.fields().to_vec()
        },
    )?;
    policy.decide(authority, &request)
}

/// Project the identity pair only after its own field-level policy decision.
pub fn project_company_identity<P: CompanyPolicyDecisionPort + ?Sized>(
    policy: &P,
    authority: &CurrentCompanyAuthority,
) -> Result<CompanyContextView, CompanyPolicyError> {
    if decide(policy, authority, InitialCompanyAction::ReadIdentity, false)?
        == CompanyPolicyDecision::Deny
    {
        return Err(CompanyPolicyError::NotFound);
    }
    Ok(CompanyContextView {
        org_id: authority.company(),
        name: authority.name().to_owned(),
        slug: authority.slug().to_owned(),
    })
}

pub async fn read_company_identity<S: CompanyPolicyStore, P: CompanyPolicyDecisionPort + ?Sized>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    company: OrgId,
) -> Result<CompanyIdentityView, CompanyPolicyError> {
    read_current(store, credentials, company, |a| {
        let identity = project_company_identity(policy, a)?;
        let show_policy_navigation = decide(policy, a, InitialCompanyAction::ReadPolicy, true)?
            == CompanyPolicyDecision::Allow;
        Ok(CompanyIdentityView {
            org_id: identity.org_id,
            name: identity.name,
            slug: identity.slug,
            show_policy_navigation,
        })
    })
    .await
}

pub async fn read_company_policy<S: CompanyPolicyStore, P: CompanyPolicyDecisionPort + ?Sized>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    company: OrgId,
) -> Result<CompanyPolicyView, CompanyPolicyError> {
    read_current(store, credentials, company, |a| {
        if decide(policy, a, InitialCompanyAction::ReadPolicy, false)?
            == CompanyPolicyDecision::Deny
        {
            return Err(CompanyPolicyError::NotFound);
        }
        // The checked initial graph has five use clauses and two ceilings.
        let uses = &a.clauses()[..5];
        let ceilings = &a.clauses()[5..];
        Ok(CompanyPolicyView {
            initial_ceiling: CompanyInitialCeilingView {
                org_id: a.company(),
                account_id: a.account(),
                action_keys: uses.iter().map(|c| c.action_kind().as_str()).collect(),
                action_refs: uses.iter().map(|c| c.action().clone()).collect(),
                delegable_action_keys: ceilings.iter().map(|c| c.action_kind().as_str()).collect(),
                delegable_action_refs: ceilings.iter().map(|c| c.action().clone()).collect(),
                company_property_keys: vec!["company.name", "company.slug"],
                company_property_refs: uses[0].fields().to_vec(),
                delegable_company_property_keys: vec!["company.name", "company.slug"],
                delegable_company_property_refs: ceilings[0].fields().to_vec(),
                future_registrations: false,
                group_control: false,
            },
        })
    })
    .await
}

pub async fn discover_company_context<
    S: CompanyPolicyStore,
    P: CompanyPolicyDecisionPort + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    company: OrgId,
) -> Result<CompanyContextView, CompanyPolicyError> {
    read_current(store, credentials, company, |a| {
        if decide(policy, a, InitialCompanyAction::Discover, false)? == CompanyPolicyDecision::Deny
        {
            return Err(CompanyPolicyError::NotFound);
        }
        Ok(CompanyContextView {
            org_id: a.company(),
            name: a.name().to_owned(),
            slug: a.slug().to_owned(),
        })
    })
    .await
}

pub async fn discover_company_contexts<
    S: CompanyPolicyStore,
    P: CompanyPolicyDecisionPort + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
) -> Result<Vec<CompanyContextView>, CompanyPolicyError> {
    for attempt in 0..2 {
        let candidates = store.enumerate_company_candidates(credentials).await?;
        let mut views = Vec::with_capacity(candidates.companies().len());
        for company in candidates.companies() {
            match discover_company_context(store, policy, credentials, *company).await {
                Ok(view) => views.push(view),
                Err(CompanyPolicyError::NotFound) => {}
                Err(error) => return Err(error),
            }
        }
        match store
            .check_context_generation(credentials, candidates.generation())
            .await
        {
            Ok(()) => return Ok(views),
            Err(CompanyPolicyError::Conflict) if attempt == 0 => continue,
            Err(CompanyPolicyError::Conflict) => {
                return Err(CompanyPolicyError::MaterialUnavailable);
            }
            Err(error) => return Err(error),
        }
    }
    Err(CompanyPolicyError::MaterialUnavailable)
}
