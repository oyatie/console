//! Current authority is checked at actual buffered emission, not merely when a
//! subscription or cursor was created. Opaque queue comes from ordinary owner.
use crate::{
    native_fixture::{TestResult, build_native_deployment},
    security_grant_producer::fresh_context,
};
use console_payroll_adapter_postgres::action30 as payroll;
use console_platform_realtime::projection as live;
use console_platform_request_context::account as context;
use sqlx::PgPool;
#[sqlx::test(migrations = false)]
async fn live_buffered_actual_projection_refuses_after_switch_and_fresh_b_emits(
    pool: PgPool,
) -> TestResult {
    let d = build_native_deployment(pool, "live-switch-current-emission", 2).await?;
    let a = &d.companies[0];
    let b = &d.companies[1];
    let (arun, _) = a.calculated().await?;
    let (brun, _) = b.calculated().await?;
    let auth = fresh_context(&d, 0, &d.accounts.submitter.account_access_token).await?;
    let resource =
        payroll::read_native_line_policy_resource(&d.pool, &auth, arun.run_id, a.facts.employee_id)
            .await?;
    let schema = payroll::read_native_projection_schema(&d.pool, &auth, arun.run_id).await?;
    let purpose = schema.resolve_registered_purpose("current")?;
    let mut positive =
        live::open_projection_subscription(&d.pool, &auth, &resource.run.object, purpose.clone())
            .await?;
    let emitted = live::drain_authorized_frames(&d.pool, &auth, &mut positive).await?;
    assert!(!emitted.is_empty());
    assert!(
        emitted
            .iter()
            .any(|f| String::from_utf8_lossy(&f.encode().unwrap()).contains("3000000"))
    );
    let mut buffered =
        live::open_projection_subscription(&d.pool, &auth, &resource.run.object, purpose.clone())
            .await?;
    assert!(buffered.pending_len() > 0, "actual queued initial snapshot");
    let selected = context::switch_company_context(
        &d.pool,
        &d.verifier,
        &d.accounts.submitter.account_access_token,
        b.org,
        &d.serving,
    )
    .await?;
    let bctx = context::resolve_company_context(
        &d.pool,
        &d.verifier,
        &d.accounts.submitter.account_access_token,
        &selected,
        &d.serving,
    )
    .await?;
    assert!(matches!(
        live::drain_authorized_frames(&d.pool, &auth, &mut buffered).await,
        Err(live::ProjectionStreamError::StaleContext)
    ));
    assert_eq!(
        buffered.pending_len(),
        0,
        "stale buffered plaintext must be discarded"
    );
    assert!(matches!(
        live::drain_authorized_frames(&d.pool, &bctx, &mut buffered).await,
        Err(live::ProjectionStreamError::BindingMismatch)
            | Err(live::ProjectionStreamError::Closed)
    ));
    let bresource =
        payroll::read_native_line_policy_resource(&d.pool, &bctx, brun.run_id, b.facts.employee_id)
            .await?;
    let bschema = payroll::read_native_projection_schema(&d.pool, &bctx, brun.run_id).await?;
    let mut fresh = live::open_projection_subscription(
        &d.pool,
        &bctx,
        &bresource.run.object,
        bschema.resolve_registered_purpose("current")?,
    )
    .await?;
    assert!(
        !live::drain_authorized_frames(&d.pool, &bctx, &mut fresh)
            .await?
            .is_empty()
    );
    Ok(())
}
#[sqlx::test(migrations = false)]
async fn context_cursor_actual_switch_invalidates_prior_scan_and_new_scan_succeeds(
    pool: PgPool,
) -> TestResult {
    let f = crate::context_owner_fixture::ContextFixture::create(&pool, 2, 3).await?;
    let token = &f.accounts.submitter.account_access_token;
    let (_, first, _) = f
        .page(token, "/api/v2/accounts/me/contexts?page_size=1")
        .await?;
    let cursor = first["next_cursor"]
        .as_str()
        .ok_or("actual nonempty cursor")?;
    context::switch_company_context(&f.auth_pool, &f.verifier, token, f.companies[2], &f.serving)
        .await?;
    let (status, body, _) = f
        .page(
            token,
            &format!("/api/v2/accounts/me/contexts?page_size=1&cursor={cursor}"),
        )
        .await?;
    assert_eq!(status, http::StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "stale_cursor");
    assert!(!f.scan().await?.is_empty());
    Ok(())
}
