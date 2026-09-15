//! Append inside request-context account owner tests. This exercises the actual
//! pure exact-binding check used after live authority loads; snapshots are plain
//! operands, NOT constructed ActiveContext/capabilities. DB provenance is tested
//! separately by the real context switch/revocation fixtures.
#[cfg(test)]
mod exact_authority_binding_acceptance {
    use super::{AuthorityBindingSnapshot, check_exact_authority_binding};
    use uuid::Uuid;
    fn operands() -> AuthorityBindingSnapshot {
        AuthorityBindingSnapshot {
            account_id: Uuid::from_u128(1),
            session_id: Uuid::from_u128(2),
            group_id: Uuid::from_u128(3),
            company_id: Some(Uuid::from_u128(4)),
            security_generation: 7,
            context_generation: 11,
            topology_revision: 13,
            policy_revision: 17,
            serving_generation: 19,
            recovery_generation: 23,
            membership_incarnation: Uuid::from_u128(29),
        }
    }
    #[test]
    fn exact_binding_accepts_equal_current_operands() {
        let current = operands();
        assert!(check_exact_authority_binding(&current, &current).is_ok());
    }
    #[test]
    fn exact_binding_rejects_each_stale_and_unexpected_future_operand() {
        let current = operands();
        macro_rules! numeric {($($field:ident),+)=>{$(for delta in [-1,1]{let mut presented=current.clone();presented.$field+=delta;assert!(check_exact_authority_binding(&presented,&current).is_err(),stringify!($field));})+};}
        numeric!(
            security_generation,
            context_generation,
            topology_revision,
            policy_revision,
            serving_generation,
            recovery_generation
        );
    }
    #[test]
    fn exact_binding_rejects_each_account_session_and_scope_identity() {
        let current = operands();
        for dimension in 0..6 {
            let mut presented = current.clone();
            match dimension {
                0 => presented.account_id = Uuid::from_u128(101),
                1 => presented.session_id = Uuid::from_u128(102),
                2 => presented.group_id = Uuid::from_u128(103),
                3 => presented.company_id = Some(Uuid::from_u128(104)),
                4 => presented.company_id = None,
                5 => presented.membership_incarnation = Uuid::from_u128(105),
                _ => unreachable!(),
            };
            assert!(check_exact_authority_binding(&presented, &current).is_err());
        }
    }
}
