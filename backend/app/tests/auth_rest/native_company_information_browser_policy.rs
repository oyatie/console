//! Injection at the existing owning decision boundary; real Cedar remains the oracle.
use console_identity_application::company_policy::{
    CompanyPolicyDecision, CompanyPolicyDecisionPort, CompanyPolicyError, CompanyPolicyRequest,
    CurrentCompanyAuthority, CurrentNativeBootstrapAuthority, CurrentPayrollReadAuthority,
    CurrentPeopleDirectoryAuthority, InitialCompanyAction, NativeBootstrapRequestV1,
    NativePeopleDirectoryRequestV1,
};
use console_platform_authz::company_policy::CompanyPolicy;
use std::sync::Mutex;

pub(super) struct ObservedRead {
    real: CompanyPolicy,
    expected: CurrentCompanyAuthority,
    request: CompanyPolicyRequest,
    calls: Mutex<usize>,
    refusal: Option<(usize, bool)>,
}
impl ObservedRead {
    pub(super) fn new(expected: CurrentCompanyAuthority, refusal: Option<(usize, bool)>) -> Self {
        let clause = expected
            .clauses()
            .iter()
            .find(|c| !c.delegable() && c.action_kind() == InitialCompanyAction::ReadPolicy)
            .unwrap();
        assert_eq!(clause.fields().len(), 8);
        let request = CompanyPolicyRequest::new(
            expected.company(),
            clause.action().object_type_id(),
            expected.assignment_id(),
            clause.action().clone(),
            clause.fields().to_vec(),
        )
        .unwrap();
        Self {
            real: CompanyPolicy::new().unwrap(),
            expected,
            request,
            calls: Mutex::new(0),
            refusal,
        }
    }
    pub(super) fn count(&self, expected: usize) {
        assert_eq!(
            *self.calls.lock().unwrap(),
            expected,
            "current read omitted/duplicated initial or final real Cedar decision"
        );
    }
}
impl CompanyPolicyDecisionPort for ObservedRead {
    fn decide_native_people_directory(
        &self,
        _: &CurrentPeopleDirectoryAuthority,
        _: &NativePeopleDirectoryRequestV1,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        panic!("manager current entered People owner");
    }
    fn decide_native_bootstrap(
        &self,
        _: &CurrentNativeBootstrapAuthority,
        _: &NativeBootstrapRequestV1,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        panic!("manager current substituted bootstrap/operator authority");
    }
    fn decide_native_payroll_collection(
        &self,
        _: &CurrentPayrollReadAuthority,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        panic!("manager current entered Payroll owner");
    }
    fn decide(
        &self,
        authority: &CurrentCompanyAuthority,
        request: &CompanyPolicyRequest,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        assert_eq!(
            request, &self.request,
            "manager read changed real assignment resource/fields"
        );
        assert_eq!(authority.account(), self.expected.account());
        assert_eq!(authority.company(), self.expected.company());
        assert_eq!(authority.epoch(), self.expected.epoch());
        assert_eq!(
            authority.context_generation(),
            self.expected.context_generation()
        );
        assert_eq!(
            authority.current_policy_receipt_id(),
            self.expected.current_policy_receipt_id()
        );
        assert_eq!(authority.assignment_id(), self.expected.assignment_id());
        assert_eq!(
            authority.assignment_revision(),
            self.expected.assignment_revision()
        );
        assert_eq!(authority.role_id(), self.expected.role_id());
        assert_eq!(authority.role_revision(), self.expected.role_revision());
        assert_eq!(authority.clauses(), self.expected.clauses());
        assert_eq!(authority.name(), self.expected.name());
        assert_eq!(authority.slug(), self.expected.slug());
        let result = self.real.decide(authority, request)?;
        assert_eq!(
            result,
            CompanyPolicyDecision::Allow,
            "actual Cedar positive source stopped permitting"
        );
        let mut calls = self.calls.lock().unwrap();
        *calls += 1;
        match self.refusal {
            Some((at, fault)) if *calls == at => {
                if fault {
                    Err(CompanyPolicyError::EvaluatorUnavailable)
                } else {
                    Ok(CompanyPolicyDecision::Deny)
                }
            }
            _ => Ok(result),
        }
    }
}
