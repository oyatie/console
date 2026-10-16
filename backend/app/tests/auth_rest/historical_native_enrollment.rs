// Parent account_browser historical-data producer only. Ordinary enrolled()
// remains the actual HTTP registration acceptance owner for current profiles.
async fn historical_enrolled(app: &Fixture, key: &SigningKey) -> (Attempt, Cookies) {
    use console_platform_auth::{
        AccountAccessVerification, AccountAssurance, JwtIssuer, JwtSettings, JwtVerifier,
        PasskeyService, WebauthnSettings,
    };
    use console_platform_provisioning::{
        AccountRegistrationFinishInput, AccountTermsArtifacts, AccountTermsItem,
        finish_account_registration_in_tx,
    };
    use console_platform_test_support::login_test_pool;

    let old: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_proc WHERE pronamespace='public'::regnamespace AND proname IN ('account_session_shared_material_v1','auth_account_session_shared_material_v1'))")
        .fetch_one(&app.pool).await.unwrap();
    assert!(
        old,
        "historical producer must never bypass ordinary current HTTP acceptance"
    );
    // Obtain actual content from the configured trusted HTTP artifact service,
    // then independently hash every byte before constructing typed artifacts.
    let manifest_response = request(
        app,
        "GET",
        &format!("/api/v2/auth/terms/manifests/{MANIFEST_DIGEST}"),
        &Cookies::default(),
        None,
        &[],
    )
    .await;
    assert!(
        manifest_response.status == StatusCode::OK
            && manifest_response.bytes == MANIFEST_TEXT.as_bytes()
            && hex::encode(Sha256::digest(&manifest_response.bytes)) == MANIFEST_DIGEST,
        "actual historical manifest bytes required"
    );
    let manifest_value: Value = serde_json::from_slice(&manifest_response.bytes).unwrap();
    let mut items = Vec::new();
    for item in manifest_value["items"].as_array().unwrap() {
        assert!(item["required"] == json!(true));
        let digest = item["content_sha256"].as_str().unwrap();
        let content = request(
            app,
            "GET",
            &format!("/api/v2/auth/terms/content/{digest}"),
            &Cookies::default(),
            None,
            &[],
        )
        .await;
        assert!(
            content.status == StatusCode::OK
                && hex::encode(Sha256::digest(&content.bytes)) == digest,
            "actual historical content bytes required"
        );
        items.push(AccountTermsItem {
            terms_kind: item["terms_kind"].as_str().unwrap().to_owned(),
            content_sha256: hex::decode(digest).unwrap().try_into().unwrap(),
        });
    }
    let digest: [u8; 32] = hex::decode(MANIFEST_DIGEST).unwrap().try_into().unwrap();
    let terms = AccountTermsArtifacts::new(digest, items).unwrap();
    let kinds: Vec<_> = terms.required_kinds().collect();
    // Real HTTP start, issued ceremony and real signed authenticator proof.
    let attempt = start(app).await;
    let config = account_browser_config(&app.pool, app._artifacts.root.clone(), key)
        .auth_rest
        .unwrap();
    let passkeys = PasskeyService::new(WebauthnSettings {
        rp_id: config.rp_id.clone(),
        rp_origin: Url::parse(&config.rp_origin).unwrap(),
        rp_name: config.rp_name.clone(),
        extra_allowed_origins: Vec::new(),
        ceremony_ttl: config.ceremony_ttl,
    })
    .unwrap();
    let settings = JwtSettings {
        issuer: config.jwt_issuer.clone(),
        audience: config.jwt_audience.clone(),
        access_token_ttl: Duration::minutes(15),
    };
    let issuer = JwtIssuer::from_es256_pem(
        settings.clone(),
        config.jwt_private_key_pem.as_bytes(),
        config.jwt_public_key_pem.as_bytes(),
    )
    .unwrap();
    let verifier =
        JwtVerifier::from_es256_public_pem(settings, config.jwt_public_key_pem.as_bytes()).unwrap();
    let auth = login_test_pool(&app.pool, TestDatabaseLogin::Auth).await;
    let mut tx = auth.begin().await.unwrap();
    let identity: (String, String) = sqlx::query_as("SELECT session_user::text,current_user::text")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert!(identity == ("console_auth_rt".into(), "console_auth_rt".into()));
    let issued = finish_account_registration_in_tx(
        &mut tx,
        &passkeys,
        &issuer,
        AccountRegistrationFinishInput {
            terms: &terms,
            accepted_terms: digest,
            accepted_kinds: &kinds,
            ceremony_id: attempt.ceremony,
            browser_nonce: &attempt.cookies.0[ENROLLMENT],
            origin: &config.rp_origin,
            credential: serde_json::from_value(attempt.finish["credential"].clone()).unwrap(),
            refresh_ttl: config.refresh_token_ttl,
            absolute_family_ttl: config.refresh_family_absolute_ttl,
        },
    )
    .await
    .unwrap_or_else(|_| panic!("real historical registration owner refused actual proof"));
    let now: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    let verification = verifier
        .verify_account_access_token(issued.access.as_str(), now)
        .unwrap_or_else(|_| panic!("actual issuer signature/claims must verify"));
    let AccountAccessVerification::Current(claims) = verification else {
        panic!("newly issued historical token already expired");
    };
    assert!(
        claims.sub == attempt.account
            && claims.sid == issued.family.family_id
            && claims.auth_time == issued.family.auth_time.unix_timestamp()
            && claims.security_generation == 1
            && claims.assurance == AccountAssurance::PasskeyPrimary
            && claims.exp <= issued.family.family_expires_at.unix_timestamp()
    );
    // Independent reads in the same real Auth transaction before commit. This
    // verifies fixture output, not a replacement production live-session resolver.
    let credential_id = attempt.finish["credential"]["rawId"]
        .as_str()
        .expect("historical credential rawId must be a string")
        .to_owned();
    assert!(attempt.finish["credential"]["rawId"].is_string());
    let binding:bool = sqlx::query_scalar(r#"SELECT
      EXISTS(SELECT 1 FROM public.account_security_lock_exclusive_v1($1) s WHERE s.security_state='ACTIVE' AND s.security_generation=$3 AND s.revision>0 AND s.context_generation=1)
      AND EXISTS(SELECT 1 FROM public.auth_refresh_token_families f WHERE f.id=$2 AND f.user_id=$1 AND f.protocol='ACCOUNT_V1' AND f.account_security_generation=$3 AND f.auth_time=$4 AND f.created_at=$4 AND f.assurance='PASSKEY_PRIMARY' AND f.org_id IS NULL AND f.revoked_at IS NULL)
      AND (SELECT count(*)=1 FROM public.auth_webauthn_credentials k WHERE k.user_id=$1 AND k.credential_id=$5 AND k.org_id IS NULL)
      AND EXISTS(SELECT 1 FROM public.auth_webauthn_ceremonies c WHERE c.id=$6 AND c.user_id=$1 AND c.account_browser_flow='ACCOUNT_REGISTRATION' AND c.consumed_at IS NOT NULL)
      AND EXISTS(SELECT 1 FROM public.auth_refresh_tokens t WHERE t.family_id=$2 AND t.user_id=$1 AND t.token_hash=$7 AND t.expires_at=$8 AND t.org_id IS NULL AND t.used_at IS NULL AND t.revoked_at IS NULL)"#)
        .bind(attempt.account).bind(issued.family.family_id).bind(claims.security_generation).bind(issued.family.auth_time)
        .bind(credential_id).bind(attempt.ceremony).bind(Sha256::digest(issued.family.token.as_str().as_bytes()).to_vec()).bind(issued.family.token_expires_at)
        .fetch_one(&mut *tx).await.unwrap();
    assert!(
        binding,
        "historical Account/family/credential/ceremony/token binding mismatch"
    );
    tx.commit().await.unwrap();
    assert_committed(&app.pool, &attempt).await;
    auth.close().await;
    // Test-only carrier of real committed owner output, never fabricated HTTP
    // cookies or a claim that predecessor serving works with the new binary.
    let mut cookies = Cookies::default();
    cookies
        .0
        .insert(ACCESS.into(), issued.access.as_str().to_owned());
    cookies
        .0
        .insert(REFRESH.into(), issued.family.token.as_str().to_owned());
    (attempt, cookies)
}
