// Include inside existing auth_rest::account_browser; does not rerun migrations.
pub(super) async fn native_family_for_legacy_reader(pool: &PgPool) -> (Uuid, Uuid) {
    seed_terms(pool).await;
    let artifacts = Artifacts::new();
    let service = router(pool, artifacts.root.clone()).await;
    let app = Fixture {
        service,
        _artifacts: artifacts,
        pool: pool.clone(),
    };
    let (attempt, _cookies) = enrolled(&app).await;
    assert_committed(&app.pool, &attempt).await;
    let family:(Uuid,String,Option<Uuid>,i64)=sqlx::query_as(
        "SELECT id,protocol,org_id,account_security_generation FROM public.auth_refresh_token_families WHERE user_id=$1")
        .bind(attempt.account).fetch_one(pool).await.unwrap();
    assert!(family.1 == "ACCOUNT_V1" && family.2.is_none() && family.3 == 1);
    (attempt.account, family.0)
}
