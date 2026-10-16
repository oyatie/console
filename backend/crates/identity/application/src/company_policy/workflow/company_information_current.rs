//! Proof-free manager context; the retained owner confirms current authority
//! and the same transaction before the provisional view can leave this use case.
use super::{NativePolicyCommandRef, NativePolicyFormView, NativePolicyWorkflowError as Error};
use crate::company_policy::{
    CompanyPolicyDecision, CompanyPolicyDecisionPort, CompanyPolicyRequest,
    CurrentCompanyAuthority, InitialCompanyAction, business::NativeBusinessOperationV1,
};
use std::future::Future;

pub trait CompanyInformationManagerCurrentScope: Send {
    fn selector(&self) -> NativePolicyCommandRef;
    fn authority(&self) -> Option<&CurrentCompanyAuthority>;
    fn view(&self) -> Option<&NativePolicyFormView>;
    fn finish_not_found(self) -> impl Future<Output = Result<(), Error>> + Send;
    fn finish<P: CompanyPolicyDecisionPort + ?Sized>(
        self,
        policy: &P,
    ) -> impl Future<Output = Result<(), Error>> + Send;
}

pub trait CompanyInformationManagerCurrentStore: Sync {
    type Credentials: Sync;
    type Scope<'a>: CompanyInformationManagerCurrentScope + Send
    where
        Self: 'a;
    fn lock_company_information_current<'a>(
        &'a self,
        credentials: &'a Self::Credentials,
        selector: NativePolicyCommandRef,
    ) -> impl Future<Output = Result<Self::Scope<'a>, Error>> + Send;
}

/// Ordinary ReadPolicy on the actual root assignment and its registered fields.
/// Both retained observations use this helper; bootstrap powers never enter it.
pub fn authorize_company_information_current<P: CompanyPolicyDecisionPort + ?Sized>(
    policy: &P,
    authority: &CurrentCompanyAuthority,
) -> Result<CompanyPolicyDecision, Error> {
    let mut clauses = authority
        .clauses()
        .iter()
        .filter(|c| !c.delegable() && c.action_kind() == InitialCompanyAction::ReadPolicy);
    let clause = clauses.next().ok_or(Error::Unavailable)?;
    if clauses.next().is_some() || clause.fields().len() != 8 {
        return Err(Error::Unavailable);
    }
    let request = CompanyPolicyRequest::new(
        authority.company(),
        clause.action().object_type_id(),
        authority.assignment_id(),
        clause.action().clone(),
        clause.fields().to_vec(),
    )
    .map_err(|_| Error::Unavailable)?;
    policy
        .decide(authority, &request)
        .map_err(|_| Error::Unavailable)
}

pub async fn company_information_manager_current<
    S: CompanyInformationManagerCurrentStore,
    P: CompanyPolicyDecisionPort + ?Sized,
>(
    store: &S,
    policy: &P,
    credentials: &S::Credentials,
    selector: NativePolicyCommandRef,
) -> Result<NativePolicyFormView, Error> {
    if selector.codec_version() != 4 || selector.operation() != NativeBusinessOperationV1::Grant {
        return Err(Error::Unavailable);
    }
    let scope = store
        .lock_company_information_current(credentials, selector)
        .await?;
    selector.resolve(scope.selector())?;
    let Some(authority) = scope.authority() else {
        if scope.view().is_some() {
            return Err(Error::Unavailable);
        }
        scope.finish_not_found().await?;
        return Err(Error::NotFound);
    };
    if authority.company() != selector.company() {
        return Err(Error::Unavailable);
    }
    if authorize_company_information_current(policy, authority)? == CompanyPolicyDecision::Deny {
        scope.finish_not_found().await?;
        return Err(Error::NotFound);
    }
    let view = scope.view().ok_or(Error::Unavailable)?.clone();
    if view.selector != selector
        || view.company_epoch != authority.epoch()
        || view.acting_account_id != authority.account()
        || view.administrative_account_id != authority.account()
        || view.group_id.is_nil()
        || view.installed_object_type_id.is_none_or(|id| id.is_nil())
        || view.assignment.is_some()
    {
        return Err(Error::Unavailable);
    }
    scope.finish(policy).await?;
    Ok(view)
}
