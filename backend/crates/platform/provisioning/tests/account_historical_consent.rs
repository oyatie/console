use console_platform_auth::account::{AccountHistoricalConsent, AccountOperationError};
use console_platform_provisioning::{AccountTermsArtifacts, AccountTermsItem};

#[test]
fn historical_consent_requires_exact_registered_manifest_and_complete_item_set()
-> Result<(), AccountOperationError> {
    let service = ("test.account.service".to_owned(), [2; 32]);
    let privacy = ("test.account.privacy".to_owned(), [3; 32]);
    let artifacts = AccountTermsArtifacts::new(
        [1; 32],
        vec![
            AccountTermsItem {
                terms_kind: service.0.clone(),
                content_sha256: service.1,
            },
            AccountTermsItem {
                terms_kind: privacy.0.clone(),
                content_sha256: privacy.1,
            },
        ],
    )?;
    // Pure typed-boundary test. This does not fabricate database consent,
    // verify signatures, select release authority, or simulate an owner helper.
    for items in [
        vec![service.clone(), privacy.clone()],
        vec![privacy.clone(), service.clone()],
    ] {
        assert!(
            artifacts
                .require_historical_consent(&AccountHistoricalConsent {
                    manifest_sha256: [1; 32],
                    items
                })
                .is_ok()
        );
    }
    let candidates = [
        vec![],
        vec![service.clone()],
        vec![
            service.clone(),
            privacy.clone(),
            ("test.account.extra".into(), [4; 32]),
        ],
        vec![service.clone(), privacy.clone(), privacy.clone()],
        vec![
            service.clone(),
            ("test.account.substituted".into(), privacy.1),
        ],
        vec![service.clone(), (privacy.0.clone(), [4; 32])],
    ];
    let mut checked = 0;
    for items in candidates {
        assert!(matches!(
            artifacts.require_historical_consent(&AccountHistoricalConsent {
                manifest_sha256: [1; 32],
                items
            }),
            Err(AccountOperationError::TermsAcceptanceRequired)
        ));
        checked += 1;
    }
    assert_eq!(checked, 6);
    assert!(matches!(
        artifacts.require_historical_consent(&AccountHistoricalConsent {
            manifest_sha256: [9; 32],
            items: vec![service, privacy],
        }),
        Err(AccountOperationError::AuthorityUnavailable)
    ));
    Ok(())
}
