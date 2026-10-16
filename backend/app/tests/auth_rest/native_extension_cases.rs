// Frozen final9→extended227 native history bridge. Both historical stages
// use production dependency injection, NOT current228 from_config admission.
// Historical population uses the shared real registration-owner producer;
// current HTTP login/refresh run only after actual228/current-finalizer admission.
// SoftPasskey accommodations remain those of the existing fixture.

const NATIVE_EXTENSION_TABLES: [&str; 15] = [
    "account_context_candidates",
    "account_security",
    "account_security_events",
    "account_terms_acceptances",
    "account_terms_head",
    "account_terms_release_receipts",
    "accounts",
    "auth_bootstrap_credentials",
    "auth_device_login_handoffs",
    "auth_refresh_token_families",
    "auth_refresh_tokens",
    "auth_webauthn_ceremonies",
    "auth_webauthn_ceremony_bindings",
    "auth_webauthn_credentials",
    "company_actors",
];
const NATIVE_EXTENSION_HELPERS: [&str; 3] = [
    "account_login_consent_v1",
    "account_session_refresh_reuse_v1",
    "auth_account_refresh_reuse_revoke_v1",
];

async fn native_extension_rows(pool: &PgPool) -> BTreeMap<String, String> {
    let names: Vec<String> = sqlx::query_scalar(
        "SELECT c.relname::text FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relname=ANY($1) AND c.relkind='r' ORDER BY c.relname",
    ).bind(NATIVE_EXTENSION_TABLES.as_slice()).fetch_all(pool).await.unwrap();
    assert_eq!(
        names.iter().map(String::as_str).collect::<Vec<_>>(),
        NATIVE_EXTENSION_TABLES
    );
    let mut result = BTreeMap::new();
    for table in NATIVE_EXTENSION_TABLES
        .into_iter()
        .chain(["_sqlx_migrations"])
    {
        // Identifier comes only from the fixed local roster. Preserve PostgreSQL
        // JSON text, including numeric precision; do not deserialize row values.
        let sql = format!(
            "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text COLLATE \"C\"),'[]'::jsonb)::text FROM public.{table} t"
        );
        let rows: String = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .fetch_one(pool)
            .await
            .unwrap();
        assert!(serde_json::from_str::<Value>(&rows).unwrap().is_array());
        assert!(result.insert(table.to_owned(), rows).is_none());
    }
    assert!(
        !result["_sqlx_migrations"].eq("[]"),
        "actual SQLx ledger must be populated"
    );
    result
}

fn native_extension_rows_equal(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
) -> bool {
    let expected: BTreeSet<_> = NATIVE_EXTENSION_TABLES
        .into_iter()
        .chain(["_sqlx_migrations"])
        .collect();
    let complete = |rows: &BTreeMap<String, String>| {
        rows.keys().map(String::as_str).collect::<BTreeSet<_>>() == expected
            && rows.values().all(|value| {
                serde_json::from_str::<Value>(value).is_ok_and(|value| value.is_array())
            })
            && rows.get("_sqlx_migrations").is_some_and(|value| {
                serde_json::from_str::<Value>(value)
                    .is_ok_and(|value| value.as_array().is_some_and(|rows| !rows.is_empty()))
            })
    };
    complete(before) && complete(after) && before == after
}

// Full native metadata is retained as database-rendered JSON text. The only
// allowed projection removes the three explicitly named additive routines.
async fn native_extension_metadata(pool: &PgPool) -> (String, String, String, Vec<String>) {
    const QUERY: &str = include_str!("fixtures/account-native-extension-metadata.sql");
    assert_eq!(
        hex::encode(Sha256::digest(QUERY.as_bytes())),
        "1a3546869d2e2b15f6136c05f911d4379928721eba0171e8742d646262f089fb"
    );
    let query = QUERY.trim().trim_end_matches(';');
    let sql = format!(
        r#"WITH captured AS ({query}) SELECT snapshot::text,snapshot_sha256,
        jsonb_set(snapshot,'{{routines}}',COALESCE((SELECT jsonb_agg(r.value ORDER BY r.ordinal)
          FROM jsonb_array_elements(snapshot->'routines') WITH ORDINALITY r(value,ordinal)
          WHERE r.value->'metadata'->>'name' <> ALL($1)),'[]'::jsonb))::text,
        ARRAY(SELECT r.value->'metadata'->>'name' FROM jsonb_array_elements(snapshot->'routines') r(value)
          WHERE r.value->'metadata'->>'name'=ANY($1) ORDER BY r.value->'metadata'->>'name')
        FROM captured"#
    );
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql("SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL statement_timeout='10s'")
        .execute(tx.as_mut())
        .await
        .unwrap();
    let result: (String, String, String, Vec<String>) = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(NATIVE_EXTENSION_HELPERS.as_slice())
        .fetch_one(tx.as_mut())
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        hex::encode(Sha256::digest(result.0.as_bytes())),
        result.1,
        "database metadata hash must bind complete text"
    );
    result
}

async fn native_extension_apply_operator(pool: &PgPool, predecessor: bool) {
    const OLD_ROOT: &str = include_str!("fixtures/account-native-final9-root-9de5c767.sql");
    const OLD_CREDENTIALS: &str =
        include_str!("fixtures/account-native-final9-credentials-9de5c767.sql");
    assert_eq!(
        hex::encode(Sha256::digest(OLD_ROOT.as_bytes())),
        "a4bbbfe64d2582d6ebbf4b896c33fff71786f5a62e6bdcde3a67bffa05f69dd8"
    );
    assert_eq!(
        hex::encode(Sha256::digest(OLD_CREDENTIALS.as_bytes())),
        "a2ca6b1c3c6c765fada7941ca23a6e35f77bda1a9a6ca2baf5e11706ab18d1d0"
    );
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql("SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL statement_timeout='60s'; SET LOCAL lock_timeout='5s'")
        .execute(tx.as_mut()).await.unwrap();
    let identity: (String,String,bool) = sqlx::query_as("SELECT session_user::text,current_user::text,current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' AND (SELECT rolsuper FROM pg_roles WHERE rolname=current_user)")
        .fetch_one(tx.as_mut()).await.unwrap();
    assert_eq!(
        identity,
        (
            "console_buck_admin".to_owned(),
            "console_buck_admin".to_owned(),
            true
        )
    );
    if predecessor {
        sqlx::raw_sql(OLD_ROOT).execute(tx.as_mut()).await.unwrap();
        sqlx::raw_sql(OLD_CREDENTIALS)
            .execute(tx.as_mut())
            .await
            .unwrap();
    } else {
        // Byte-extracted from the immutable seven-asset extended227 bundle,
        // never today's changing finalizers or a synthesized historical schema.
        for (sql, digest) in [
            (
                include_str!("fixtures/account-native-extended227-root-17fba595.sql"),
                "4d10f8a670fabf9207a72c31baf9cd68f2a1a769fecc0b67222252e543636c99",
            ),
            (
                include_str!("fixtures/account-native-extended227-credentials-17fba595.sql"),
                "ce2939c1e41d1322fb9fd8af95a0725e2dd7d18c73eba50e472ce9d33c543111",
            ),
            (
                include_str!("fixtures/account-native-extended227-verify-17fba595.sql"),
                "1eefdb13cac90c8f147d5477627fef0d830f3aa16f951e9de78d125ce39f2eda",
            ),
        ] {
            assert_eq!(hex::encode(Sha256::digest(sql.as_bytes())), digest);
            sqlx::raw_sql(sql).execute(tx.as_mut()).await.unwrap();
        }
    }
    tx.commit().await.unwrap();
}

#[sqlx::test(migrations = false)]
async fn native_final9_http_enrollment_upgrade_preserves_history_and_allows_login_refresh(
    pool: PgPool,
) {
    let owner_url = historical227::prepare_historical227_database(&pool).await;
    native_extension_apply_operator(&pool, true).await;
    let initial = native_extension_metadata(&pool).await;
    assert_eq!(
        initial.1,
        "a453c9f30950a3f3ff9f545d6fb40bfc35ba0f79b97b94902be19cce2736d481"
    );
    assert!(initial.3.is_empty());
    seed_terms(&pool).await;
    let artifacts = Artifacts::new();
    let key = SigningKey::random(&mut OsRng);
    let config = account_browser_config(&pool, artifacts.root.clone(), &key);
    let business =
        console_platform_test_support::login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let auth = console_platform_test_support::login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    // The new executable must refuse old-profile serving. Test DI permits real
    // HTTP registration start; the shared production-owner producer creates
    // historical data without requiring successor-only live-session helpers.
    let injected = AppState::new(
        config.clone(),
        console_app::DatabaseDependency::Postgres(business.clone()),
    )
    .unwrap()
    .with_auth_database(auth.clone());
    let predecessor = Fixture {
        service: build_router(injected.clone()),
        _artifacts: artifacts,
        pool: pool.clone(),
    };
    let (mut attempt, original_cookies) = historical_enrolled(&predecessor, &key).await;
    assert_committed(&pool, &attempt).await;

    // Existing data-only user provisioning, then real restricted bootstrap/HTTP
    // authentication, preserve a legacy sibling alongside the native Account.
    let branch = seed_branch(
        &pool,
        "Native extension legacy region",
        "Native extension legacy branch",
    )
    .await;
    let legacy = seed_user_with_branch(
        &pool,
        "Native extension legacy employee",
        "010-8660-0001",
        "MECHANIC",
        branch,
    )
    .await;
    // Company provisioning issues bootstrap through Business; Auth remains the
    // separate restricted transport used by HTTP redemption and enrollment.
    let legacy_access = admin_session_via_otp(&predecessor, &business, legacy).await;
    let mut legacy_authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let _legacy_key = enroll_passkey(&predecessor, &mut legacy_authenticator, &legacy_access).await;
    let populations: (i64,i64,i64,i64) = sqlx::query_as("SELECT (SELECT count(*) FROM auth_webauthn_credentials WHERE user_id=$1 AND org_id IS NULL),(SELECT count(*) FROM auth_refresh_token_families WHERE user_id=$1 AND protocol='ACCOUNT_V1'),(SELECT count(*) FROM auth_webauthn_credentials WHERE user_id=$2 AND org_id IS NOT NULL),(SELECT count(*) FROM auth_refresh_token_families WHERE user_id=$2 AND protocol='LEGACY_COMPANY')")
        .bind(attempt.account).bind(*legacy.as_uuid()).fetch_one(&pool).await.unwrap();
    assert_eq!(populations, (1, 1, 1, 1));
    let contexts: (i64,i64) = sqlx::query_as("SELECT (SELECT count(*) FROM company_actors),(SELECT count(*) FROM account_context_candidates)")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(
        contexts,
        (0, 0),
        "no fabricated actor/context entitlement fixtures"
    );
    let old_metadata = native_extension_metadata(&pool).await;
    assert_eq!(
        old_metadata.0, initial.0,
        "real HTTP population must preserve exact final9 metadata"
    );
    let before_rows = native_extension_rows(&pool).await;
    let before_account = snapshot(&pool, attempt.account).await;

    let startup = AppState::from_config(config.clone()).await;
    if let Ok(state) = &startup {
        state.shutdown_realtime().await;
    }
    assert!(
        matches!(startup,Err(console_app::AppError::Config(ref code)) if code=="account_custody.native_upgrade_required"),
        "new normal startup must refuse the exact populated final9 profile"
    );
    assert!(
        native_extension_rows_equal(&before_rows, &native_extension_rows(&pool).await),
        "startup refusal changed retained rows or real SQLx ledger"
    );
    injected.shutdown_realtime().await;
    let Fixture {
        service: old_service,
        _artifacts,
        pool: _,
    } = predecessor;
    drop(old_service);
    drop(injected);

    native_extension_apply_operator(&pool, false).await;
    let upgraded = native_extension_metadata(&pool).await;
    // Independent declared-source reconciliation of the actual disposable capture;
    // do not let a changed generator and its own classifier agree on wrong helpers.
    assert_eq!(
        upgraded.1,
        "dc8e947c67171c1e10c268d4a1d54f18c1b5b39947d13b20a604d7e6f8de6766"
    );
    assert_eq!(
        upgraded.3.iter().map(String::as_str).collect::<Vec<_>>(),
        NATIVE_EXTENSION_HELPERS
    );
    assert_eq!(
        upgraded.2, old_metadata.0,
        "upgrade changed old routine or other protected metadata"
    );
    assert!(
        native_extension_rows_equal(&before_rows, &native_extension_rows(&pool).await),
        "upgrade lost or rewrote old rows/SQLx ledger"
    );
    native_extension_apply_operator(&pool, false).await;
    assert_eq!(
        native_extension_metadata(&pool).await,
        upgraded,
        "operator replay changed metadata"
    );
    assert!(
        native_extension_rows_equal(&before_rows, &native_extension_rows(&pool).await),
        "operator replay changed rows/SQLx ledger"
    );

    // New-binary startup remains refused on exact extended227. Preserve every
    // historical transition/replay assertion before the actual current upgrade.
    let startup = AppState::from_config(config.clone()).await;
    if let Ok(state) = &startup {
        state.shutdown_realtime().await;
    }
    assert!(
        matches!(startup,Err(console_app::AppError::Config(ref code)) if code=="account_custody.native_upgrade_required"),
        "current startup must refuse the exact populated extended227 profile"
    );
    assert!(
        native_extension_rows_equal(&before_rows, &native_extension_rows(&pool).await),
        "extended227 startup refusal changed retained rows or real SQLx ledger"
    );
    assert_eq!(
        native_extension_metadata(&pool).await,
        upgraded,
        "extended227 startup refusal changed protected metadata"
    );
    // Preserve the HTTP positives below at their supported binary/schema
    // boundary. No historical runtime shim or fabricated migration ledger.
    business.close().await;
    auth.close().await;
    let migration_config = AppConfig::from_pairs([
        ("CONSOLE_APP_ROLE", AppRole::Migrate.to_string()),
        ("DATABASE_URL", owner_url),
    ])
    .unwrap();
    static CURRENT: sqlx::migrate::Migrator = sqlx::migrate!("../crates/platform/db/migrations");
    assert_eq!(CURRENT.iter().last().map(|m| m.version), Some(228));
    let migration228 = CURRENT.iter().find(|m| m.version == 228).unwrap();
    run_migrations(&migration_config)
        .await
        .expect("actual current migration after complete frozen227 assertions");
    finalize_serving_account_custody(&pool).await;
    let current_rows = native_extension_rows(&pool).await;
    for table in NATIVE_EXTENSION_TABLES {
        assert!(
            before_rows[table] == current_rows[table],
            "current upgrade changed retained historical native rows"
        );
    }
    let prefix: String = sqlx::query_scalar(
        "SELECT jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text COLLATE \"C\")::text FROM public._sqlx_migrations t WHERE version<=227",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        prefix == before_rows["_sqlx_migrations"],
        "current upgrade rewrote historical227 ledger"
    );
    let ledger: Vec<(i64, bool, Vec<u8>)> = sqlx::query_as(
        "SELECT version,success,checksum FROM public._sqlx_migrations ORDER BY version",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(
        ledger.len() == 228
            && ledger
                .iter()
                .enumerate()
                .all(|(i, (version, success, _))| *version == i as i64 + 1 && *success)
            && ledger[227].2.as_slice() == migration228.checksum.as_ref(),
        "actual complete228 migration ledger/checksum required"
    );
    let state = AppState::from_config(config)
        .await
        .expect("current production admission before retained HTTP positives");
    let service = build_router(state.clone());
    let me = request(
        &service,
        "GET",
        "/api/v2/accounts/me",
        &original_cookies,
        None,
        &[],
    )
    .await;
    projection(&me.json(StatusCode::OK), attempt.account, &pool).await;
    me.private();
    let (mut cookies, input) = login_attempt(&service, &mut attempt).await;
    let signed_in = request(
        &service,
        "POST",
        "/api/v2/auth/passkey/login/finish",
        &cookies,
        Some(input),
        &[],
    )
    .await;
    session(
        &signed_in,
        StatusCode::OK,
        attempt.account,
        &mut cookies,
        &pool,
    )
    .await;
    let after_login = snapshot(&pool, attempt.account).await;
    for field in ["security", "terms", "events"] {
        assert!(
            before_account[field] == after_login[field],
            "login changed retained immutable enrollment evidence"
        );
    }
    assert_eq!(after_login["families"].as_array().unwrap().len(), 2);
    assert!(
        after_login["families"]
            .as_array()
            .unwrap()
            .contains(&before_account["families"][0]),
        "fresh login changed old primary family"
    );
    let proof = proof(&service, &cookies).await;
    let old_refresh = cookies.0[REFRESH].clone();
    let refreshed = request(
        &service,
        "POST",
        "/api/v2/auth/token/refresh",
        &cookies,
        Some(json!({})),
        &[("X-Console-CSRF", &proof)],
    )
    .await;
    session(
        &refreshed,
        StatusCode::OK,
        attempt.account,
        &mut cookies,
        &pool,
    )
    .await;
    assert!(
        cookies.0[REFRESH] != old_refresh,
        "refresh did not rotate its token"
    );
    let after_refresh = snapshot(&pool, attempt.account).await;
    assert!(
        after_refresh["families"] == after_login["families"],
        "refresh replaced or extended a primary family"
    );
    assert_eq!(after_refresh["tokens"].as_array().unwrap().len(), 3);
    assert_no_company_identity(&pool, attempt.account).await;
    state.shutdown_realtime().await;
    business.close().await;
    auth.close().await;
}

// The previous latest-startup positive is deliberately retained here at the
// successor boundary. This test must not be admitted as migration228 coverage
// until the real numbered migration and current operator finalizers exist.
#[sqlx::test(migrations = false)]
async fn native_extended227_upgrade_to228_preserves_history_and_admits_startup(pool: PgPool) {
    static CURRENT: sqlx::migrate::Migrator = sqlx::migrate!("../crates/platform/db/migrations");
    assert_eq!(
        CURRENT.iter().last().map(|m| m.version),
        Some(228),
        "NEXT1 actual migration228 prerequisite; later assertions not reached"
    );
    let migration228 = CURRENT.iter().find(|m| m.version == 228).unwrap();
    let owner_url = historical227::prepare_historical227_database(&pool).await;
    native_extension_apply_operator(&pool, false).await;
    let before_metadata = native_extension_metadata(&pool).await;
    assert_eq!(
        before_metadata.1,
        "dc8e947c67171c1e10c268d4a1d54f18c1b5b39947d13b20a604d7e6f8de6766"
    );
    seed_terms(&pool).await;
    let artifacts = Artifacts::new();
    let key = SigningKey::random(&mut OsRng);
    let config = account_browser_config(&pool, artifacts.root.clone(), &key);
    let business =
        console_platform_test_support::login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let auth = console_platform_test_support::login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let injected = AppState::new(
        config.clone(),
        console_app::DatabaseDependency::Postgres(business.clone()),
    )
    .unwrap()
    .with_auth_database(auth.clone());
    let predecessor = Fixture {
        service: build_router(injected.clone()),
        _artifacts: artifacts,
        pool: pool.clone(),
    };
    let (attempt, original_cookies) = historical_enrolled(&predecessor, &key).await;
    assert_committed(&pool, &attempt).await;
    let before_rows = native_extension_rows(&pool).await;
    let startup = AppState::from_config(config.clone()).await;
    if let Ok(state) = &startup {
        state.shutdown_realtime().await;
    }
    assert!(
        matches!(startup,Err(console_app::AppError::Config(ref code)) if code=="account_custody.native_upgrade_required"),
        "current startup must refuse populated227 before real228 transition"
    );
    assert!(
        native_extension_rows_equal(&before_rows, &native_extension_rows(&pool).await),
        "pre-transition startup refusal changed rows or ledger"
    );
    assert_eq!(
        native_extension_metadata(&pool).await,
        before_metadata,
        "pre-transition startup refusal changed protected metadata"
    );
    injected.shutdown_realtime().await;
    let Fixture {
        service: old_service,
        _artifacts,
        pool: _,
    } = predecessor;
    drop(old_service);
    drop(injected);
    business.close().await;
    auth.close().await;
    let migration_config = AppConfig::from_pairs([
        ("CONSOLE_APP_ROLE", AppRole::Migrate.to_string()),
        ("DATABASE_URL", owner_url),
    ])
    .unwrap();
    // Use the actual production migration entry; never execute fixture DDL,
    // insert a migration row, bypass ownership, or emulate an operator grant.
    run_migrations(&migration_config)
        .await
        .expect("actual227→228 production migration");
    finalize_serving_account_custody(&pool).await;
    let after_rows = native_extension_rows(&pool).await;
    for table in NATIVE_EXTENSION_TABLES {
        assert!(
            before_rows[table] == after_rows[table],
            "228 transition changed retained protected rows"
        );
    }
    let prefix: String = sqlx::query_scalar(
        "SELECT jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text COLLATE \"C\")::text FROM public._sqlx_migrations t WHERE version<=227")
        .fetch_one(&pool).await.unwrap();
    assert!(
        prefix == before_rows["_sqlx_migrations"],
        "228 transition rewrote historical SQLx ledger"
    );
    let ledger: Vec<(i64, bool, Vec<u8>)> = sqlx::query_as(
        "SELECT version,success,checksum FROM public._sqlx_migrations ORDER BY version",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(ledger.len(), 228);
    assert!(
        ledger
            .iter()
            .enumerate()
            .all(|(i, (version, success, _))| *version == i as i64 + 1 && *success)
    );
    assert!(
        ledger[227].2.as_slice() == migration228.checksum.as_ref(),
        "actual228 SQLx checksum differs from embedded source"
    );
    // Current finalizer replay must preserve the entire real228 ledger and all
    // retained old rows before serving admission is allowed.
    finalize_serving_account_custody(&pool).await;
    assert!(
        native_extension_rows_equal(&after_rows, &native_extension_rows(&pool).await),
        "current operator replay changed retained rows or real228 ledger"
    );
    let state = AppState::from_config(config)
        .await
        .expect("normal production startup after exact227→228 upgrade");
    let service = build_router(state.clone());
    let me = request(
        &service,
        "GET",
        "/api/v2/accounts/me",
        &original_cookies,
        None,
        &[],
    )
    .await;
    projection(&me.json(StatusCode::OK), attempt.account, &pool).await;
    me.private();
    assert!(
        native_extension_rows_equal(&after_rows, &native_extension_rows(&pool).await),
        "production startup or retained session read changed protected rows or ledger"
    );
    assert_no_company_identity(&pool, attempt.account).await;
    state.shutdown_realtime().await;
}

#[test]
fn native_extension_row_oracle_refuses_missing_table_null_and_precise_mutations() {
    let mut before: BTreeMap<String, String> = NATIVE_EXTENSION_TABLES
        .into_iter()
        .chain(["_sqlx_migrations"])
        .map(|name| (name.to_owned(), "[]".to_owned()))
        .collect();
    before.insert(
        "_sqlx_migrations".to_owned(),
        "[{\"version\":227}]".to_owned(),
    );
    assert!(native_extension_rows_equal(&before, &before));
    let mut empty_ledger = before.clone();
    empty_ledger.insert("_sqlx_migrations".to_owned(), "[]".to_owned());
    assert!(!native_extension_rows_equal(&empty_ledger, &empty_ledger));
    let mut missing = before.clone();
    missing.remove("account_terms_acceptances");
    assert!(!native_extension_rows_equal(&before, &missing));
    let mut null = before.clone();
    null.insert("account_security".to_owned(), "null".to_owned());
    assert!(!native_extension_rows_equal(&before, &null));
    for (old, new) in [
        ("true", "1"),
        ("false", "0"),
        ("1.00000000000000001", "1.00000000000000002"),
        ("9007199254740993.00000001", "9007199254740993.00000002"),
    ] {
        let mut original = before.clone();
        original.insert(
            "account_security_events".to_owned(),
            format!("[{{\"payload\":{old}}}]"),
        );
        let mut changed = original.clone();
        changed.insert(
            "account_security_events".to_owned(),
            format!("[{{\"payload\":{new}}}]"),
        );
        assert!(native_extension_rows_equal(&original, &original));
        assert!(!native_extension_rows_equal(&original, &changed));
    }
}
