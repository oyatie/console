use vstd::prelude::*;
verus! {
pub struct SubjectFreshness {
    pub policy_version: u64,
    pub subject_version: u64,
    pub session_generation: u64,
    pub step_up_generation: Option<u64>,
}

pub struct SubjectFreshnessRequirement {
    pub min_policy_version: u64,
    pub min_subject_version: u64,
    pub min_session_generation: u64,
    pub required_step_up_generation: Option<u64>,
}

impl SubjectFreshness {
    pub const fn satisfies(self, required: SubjectFreshnessRequirement) -> (ok: bool)
        ensures
            ok ==> self.policy_version >= required.min_policy_version,
            ok ==> self.subject_version >= required.min_subject_version,
            ok ==> self.session_generation >= required.min_session_generation,
            ok && required.required_step_up_generation.is_some() ==> self.step_up_generation.is_some(),
            ok && required.required_step_up_generation.is_some() ==> match (self.step_up_generation, required.required_step_up_generation) { (Some(a), Some(r)) => a >= r, _ => false },
            self.policy_version >= required.min_policy_version
                && self.subject_version >= required.min_subject_version
                && self.session_generation >= required.min_session_generation
                && (required.required_step_up_generation.is_none()
                    || (self.step_up_generation.is_some() && match (self.step_up_generation, required.required_step_up_generation) { (Some(a), Some(r)) => a >= r, _ => false })) ==> ok,
    {
        self.policy_version >= required.min_policy_version
            && self.subject_version >= required.min_subject_version
            && self.session_generation >= required.min_session_generation
            && match required.required_step_up_generation {
                Some(required_step_up) => match self.step_up_generation {
                    Some(actual_step_up) => actual_step_up >= required_step_up,
                    None => false,
                },
                None => true,
            }
    }
}

fn equal_current_inputs_succeed(p: u64, s: u64, g: u64, step: u64) {
    let actual = SubjectFreshness { policy_version: p, subject_version: s, session_generation: g, step_up_generation: Some(step) };
    let required = SubjectFreshnessRequirement { min_policy_version: p, min_subject_version: s, min_session_generation: g, required_step_up_generation: Some(step) };
    let ok = actual.satisfies(required);
    assert(ok);
}
fn absent_required_step_up_refuses(p: u64, s: u64, g: u64, step: u64) {
    let actual = SubjectFreshness { policy_version: p, subject_version: s, session_generation: g, step_up_generation: None };
    let required = SubjectFreshnessRequirement { min_policy_version: p, min_subject_version: s, min_session_generation: g, required_step_up_generation: Some(step) };
    let ok = actual.satisfies(required);
    assert(!ok);
}
fn strengthening_session_requirement_preserves_refusal(actual: SubjectFreshness, required: SubjectFreshnessRequirement, stronger: u64)
{
    if stronger < required.min_session_generation { return; }
    let copy = SubjectFreshness { policy_version: actual.policy_version, subject_version: actual.subject_version, session_generation: actual.session_generation, step_up_generation: actual.step_up_generation };
    let raised = SubjectFreshnessRequirement { min_policy_version: required.min_policy_version, min_subject_version: required.min_subject_version, min_session_generation: stronger, required_step_up_generation: required.required_step_up_generation };
    let original_ok = actual.satisfies(required);
    let raised_ok = copy.satisfies(raised);
    assert(!original_ok ==> !raised_ok);
}
}
