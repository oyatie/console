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
    pub const fn satisfies(self, required: SubjectFreshnessRequirement) -> bool {
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

#[cfg(test)] mod tests {
use super::*;
fn actual(p:u64,s:u64,g:u64,step:Option<u64>)->SubjectFreshness { SubjectFreshness {policy_version:p,subject_version:s,session_generation:g,step_up_generation:step} }
fn need(p:u64,s:u64,g:u64,step:Option<u64>)->SubjectFreshnessRequirement { SubjectFreshnessRequirement {min_policy_version:p,min_subject_version:s,min_session_generation:g,required_step_up_generation:step} }
#[test] fn equal_current_inputs_succeed() { for n in [0,1,u64::MAX] { assert!(actual(n,n,n,Some(n)).satisfies(need(n,n,n,Some(n)))); } }
#[test] fn stale_policy_refuses() { assert!(!actual(4,8,8,None).satisfies(need(5,1,1,None))); }
#[test] fn stale_subject_refuses() { assert!(!actual(8,4,8,None).satisfies(need(1,5,1,None))); }
#[test] fn stale_session_refuses() { assert!(!actual(8,8,4,None).satisfies(need(1,1,5,None))); }
#[test] fn step_up_absence_and_staleness() { assert!(actual(1,1,1,None).satisfies(need(1,1,1,None))); assert!(!actual(1,1,1,None).satisfies(need(1,1,1,Some(0)))); assert!(!actual(1,1,1,Some(0)).satisfies(need(1,1,1,Some(1)))); assert!(actual(1,1,1,Some(2)).satisfies(need(1,1,1,Some(1)))); }
#[test] fn maximum_values_do_not_wrap() { assert!(actual(u64::MAX,u64::MAX,u64::MAX,Some(u64::MAX)).satisfies(need(0,0,0,Some(0)))); assert!(!actual(0,0,0,Some(0)).satisfies(need(u64::MAX,u64::MAX,u64::MAX,Some(u64::MAX)))); }
}
