//! Concrete first context-result projection probes. These exercise actual
//! identity/read and ordinary topology writers. They do not claim to cover every
//! payroll/history/live/event/egress alias in SEC-PROJECTION-01.
use crate::{context_owner_fixture::ContextFixture, operator_custody_producer::Result};
use console_identity_adapter_postgres::account13 as identity;
use console_platform_auth::account_session::configured_account_verifier;
use console_platform_provisioning::{PlatformProvisioner, account13 as provisioning};
use console_platform_request_context::account as context;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = false)]
async fn current_active_company_never_uses_other_switchable_company_and_old_context_expires(
    pool: PgPool,
) -> Result<()> {
    let f = ContextFixture::create(&pool, 2, 3).await?;
    let verifier = &f.verifier;
    let serving = &f.serving;
    let token = &f.accounts.submitter.account_access_token;
    let a = f.companies[1];
    let b = f.companies[2];
    let selected_a =
        context::switch_company_context(&f.auth_pool, &verifier, token, a, &serving).await?;
    let auth_a =
        context::resolve_company_context(&f.auth_pool, &verifier, token, &selected_a, &serving)
            .await?;
    let own = identity::read_company_context(&f.owner_pool, &auth_a, a).await?;
    assert_eq!(own.context_ref.org_id, a);
    assert!(own.label.is_some());
    assert!(
        identity::read_company_context(&f.owner_pool, &auth_a, b)
            .await
            .is_err(),
        "active A must not implicitly switch to allowed B"
    );
    let before: i64 =
        sqlx::query_scalar("SELECT context_generation FROM account_security WHERE account_id=$1")
            .bind(f.accounts.submitter.account_id)
            .fetch_one(&pool)
            .await?;
    let selected_b =
        context::switch_company_context(&f.auth_pool, &verifier, token, b, &serving).await?;
    let after: i64 =
        sqlx::query_scalar("SELECT context_generation FROM account_security WHERE account_id=$1")
            .bind(f.accounts.submitter.account_id)
            .fetch_one(&pool)
            .await?;
    assert!(after > before);
    assert!(
        identity::read_company_context(&f.owner_pool, &auth_a, a)
            .await
            .is_err()
    );
    assert!(
        context::resolve_company_context(&f.auth_pool, &verifier, token, &selected_a, &serving)
            .await
            .is_err()
    );
    let auth_b =
        context::resolve_company_context(&f.auth_pool, &verifier, token, &selected_b, &serving)
            .await?;
    assert_eq!(
        identity::read_company_context(&f.owner_pool, &auth_b, b)
            .await?
            .context_ref
            .org_id,
        b
    );
    // Another real Account is enrolled but carries no discovery grant in B.
    assert!(
        context::switch_company_context(
            &f.auth_pool,
            &verifier,
            &f.accounts.reviewer.account_access_token,
            b,
            &serving
        )
        .await
        .is_err()
    );
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn hidden_company_label_changes_do_not_change_allowed_context_projection(
    pool: PgPool,
) -> Result<()> {
    let f = ContextFixture::create(&pool, 2, 3).await?;
    let before = f.scan().await?;
    let provisioner = PlatformProvisioner::new(time::Duration::minutes(5));
    // Same Company field denied but identity permitted, then separate Group's
    // undiscoverable Company. These are real persisted private-world changes.
    for (org, secret) in [
        (f.companies[0], "TEST_ONLY hidden-field changed"),
        (f.companies[5], "TEST_ONLY other-group secret changed"),
    ] {
        let control = provisioner
            .read_account_company_control(&f.owner_pool, &f.operator, org)
            .await?;
        provisioner
            .update_account_company(
                &f.owner_pool,
                &f.operator,
                provisioning::CompanyUpdate {
                    command_id: Uuid::new_v4(),
                    org_id: org,
                    expected: control,
                    name: secret.into(),
                },
            )
            .await?;
        // Separately authorized operator read is the positive hidden-world
        // witness; equality of public output alone could otherwise be vacuous.
        let actual = provisioner
            .read_account_company_control(&f.owner_pool, &f.operator, org)
            .await?;
        assert_eq!(actual.name, secret);
        assert_eq!(f.scan().await?, before);
    }
    let visible = f.companies[1];
    let control = provisioner
        .read_account_company_control(&f.owner_pool, &f.operator, visible)
        .await?;
    provisioner
        .update_account_company(
            &f.owner_pool,
            &f.operator,
            provisioning::CompanyUpdate {
                command_id: Uuid::new_v4(),
                org_id: visible,
                expected: control,
                name: "TEST_ONLY visible updated".into(),
            },
        )
        .await?;
    let changed = f.scan().await?;
    assert_ne!(changed, before);
    assert_eq!(
        changed.get(&format!("org_id:{visible}")),
        Some(&Some("TEST_ONLY visible updated".into()))
    );
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn context_result_revocation_refuses_existing_opaque_context_but_fresh_unrevoked_read_succeeds(
    pool: PgPool,
) -> Result<()> {
    let f = ContextFixture::create(&pool, 2, 3).await?;
    let verifier = &f.verifier;
    let serving = &f.serving;
    let token = &f.accounts.submitter.account_access_token;
    let org = f.companies[1];
    let selection =
        context::switch_company_context(&f.auth_pool, &verifier, token, org, &serving).await?;
    let auth =
        context::resolve_company_context(&f.auth_pool, &verifier, token, &selection, &serving)
            .await?;
    assert!(
        identity::read_company_context(&f.owner_pool, &auth, org)
            .await?
            .label
            .is_some()
    );
    let expected = identity::read_company_policy_control(&f.owner_pool, &f.operator, org).await?;
    identity::revoke_company_grant(
        &f.owner_pool,
        &f.operator,
        identity::RevokeCompanyGrant {
            command_id: Uuid::new_v4(),
            org_id: org,
            grant_id: f.grants[1].grant_id,
            expected,
            reason: "TEST_ONLY disclosure revoked".into(),
        },
    )
    .await?;
    assert!(
        identity::read_company_context(&f.owner_pool, &auth, org)
            .await
            .is_err()
    );
    let other = f.companies[2];
    let selection =
        context::switch_company_context(&f.auth_pool, &verifier, token, other, &serving).await?;
    let fresh =
        context::resolve_company_context(&f.auth_pool, &verifier, token, &selection, &serving)
            .await?;
    assert!(
        identity::read_company_context(&f.owner_pool, &fresh, other)
            .await?
            .label
            .is_some()
    );
    Ok(())
}
