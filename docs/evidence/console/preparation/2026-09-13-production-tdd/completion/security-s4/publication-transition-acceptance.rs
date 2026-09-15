//! Boundary composition for BW31 publication revision arithmetic. The checked
//! transition function is the actual production function called by publish_terms
//! under the head lock; no local substitute model or9e18-publication setup.
use console_platform_auth::terms_publication::{RevisionError, checked_publication_revision};

#[test]
fn publication_revision_genesis_successor_and_signed_bigint_boundaries() {
    assert_eq!(checked_publication_revision(None, 0).unwrap(), 1);
    for previous in [1, 7, i64::MAX - 1] {
        assert_eq!(
            checked_publication_revision(Some(previous), previous).unwrap(),
            previous + 1
        );
    }
    assert!(matches!(
        checked_publication_revision(Some(i64::MAX), i64::MAX),
        Err(RevisionError::Overflow)
    ));
    assert!(matches!(
        checked_publication_revision(Some(0), 0),
        Err(RevisionError::InvalidHead)
    ));
    assert!(matches!(
        checked_publication_revision(Some(-1), -1),
        Err(RevisionError::InvalidHead)
    ));
}

#[test]
fn publication_revision_requires_exact_predecessor_and_never_repairs_a_head() {
    assert!(matches!(
        checked_publication_revision(None, 7),
        Err(RevisionError::Conflict)
    ));
    for expected in [0, 6, 8, i64::MAX] {
        assert!(matches!(
            checked_publication_revision(Some(7), expected),
            Err(RevisionError::Conflict)
        ));
    }
}
