// Organization V12 stage C: actual startup/readiness over a committed generated
// closed perimeter. Reuses the unchanged reviewed stage B predecessor/protocol.
// This is catalog serving admission, not a seeded or UI-created business journey.
const SERVING_APP_CLOSED_STATE: &str =
    include_str!("../../src/native_org_unit_closed_perimeter_v1_custody_state.sql");
const SERVING_APP_BOUNDED_CLOSED_STATE: &str =
    include_str!("../../src/native_org_unit_closed_perimeter_v1_bounded_custody_state.sql");

fn serving_pins() {
    pins();
    assert_eq!(CLOSED_STATE, SERVING_APP_CLOSED_STATE);
    assert_eq!(
        digest(SERVING_APP_CLOSED_STATE),
        "670564ce4a107746d8f50d316014b4aca763af0335d9c38a01fcfa23ec9e46dc"
    );
    assert_eq!(BOUNDED_CLOSED_STATE, SERVING_APP_BOUNDED_CLOSED_STATE);
    assert_eq!(
        digest(SERVING_APP_BOUNDED_CLOSED_STATE),
        "66ace47fbfeacbd8df3330e6caf9ae2bf28d44191bf9a347bd3a0a986f8f2ee8"
    );
    const BEFORE: &str = "), snapshots AS (\n";
    const AFTER: &str = "), snapshots AS MATERIALIZED (\n";
    assert_eq!(SERVING_APP_CLOSED_STATE.matches(BEFORE).count(), 2);
    assert_eq!(SERVING_APP_CLOSED_STATE.matches(AFTER).count(), 0);
    assert_eq!(SERVING_APP_BOUNDED_CLOSED_STATE.matches(AFTER).count(), 2);
    assert_eq!(SERVING_APP_BOUNDED_CLOSED_STATE.matches(BEFORE).count(), 0);
    assert_eq!(
        SERVING_APP_BOUNDED_CLOSED_STATE.replace(AFTER, BEFORE),
        SERVING_APP_CLOSED_STATE
    );
}

async fn serving_snapshot(pool: &PgPool, variant: usize) -> Baseline {
    let mut connection = direct(pool).await;
    let frozen_target = target(&mut connection).await;
    let mut tx = sqlx::Connection::begin(&mut connection).await.unwrap();
    assert_eq!(
        capture(tx.as_mut(), CAPTURE).await.sha256,
        CLOSED73[variant]
    );
    assert_eq!(
        raw_wider_capture(tx.as_mut()).await["snapshot_sha256"],
        CLOSED76[variant]
    );
    assert_eq!(
        state(tx.as_mut()).await,
        "native_org_unit.closed_perimeter_compatible"
    );
    relation_and_schema_closure(tx.as_mut(), true).await;
    let result = Baseline {
        target: frozen_target,
        rows: rows(tx.as_mut()).await,
        catalog: catalog(tx.as_mut()).await,
        ledger: applied_ledger(tx.as_mut()).await,
        denied: added_three_denied_rights(tx.as_mut()).await,
    };
    tx.rollback().await.unwrap();
    connection.close().await.unwrap();
    result
}

async fn serving_restored(pool: &PgPool, variant: usize, before: &Baseline) {
    let after = serving_snapshot(pool, variant).await;
    assert_eq!(after.target, before.target);
    assert!(after.rows == before.rows, "serving changed durable rows");
    assert!(
        after.catalog == before.catalog,
        "catalog was not exactly restored"
    );
    assert!(
        after.ledger == before.ledger,
        "serving changed the migration ledger"
    );
    assert!(
        after.denied == before.denied,
        "startup rights were not restored"
    );
}

async fn serving_fresh_refused(config: AppConfig) {
    match AppState::from_config(config).await {
        Ok(state) => {
            state.shutdown_realtime().await;
            panic!("committed closed-profile corruption admitted fresh startup");
        }
        Err(AppError::Config(code)) => assert!(
            matches!(
                code.as_str(),
                "native_org_unit.profile_mismatch" | "company_provenance.profile_mismatch"
            ),
            "unrelated configuration failure does not prove closed custody refusal"
        ),
        Err(_) => panic!("unrelated startup failure does not prove custody refusal"),
    }
}

async fn serving_native_org_still_closed(state: &AppState) {
    // No business fixture or credentials are manufactured. Exact route closure
    // is an omission control only; authorized Company/People/Payroll positives
    // remain separate unchanged owner/browser acceptance requirements.
    let company = Uuid::new_v4();
    for path in [
        format!("/companies/{company}/organization"),
        format!("/companies/{company}/organization/new"),
    ] {
        for cookie in [None, Some("console_account_access=invalid-token")] {
            let mut request = Request::builder().uri(&path);
            if let Some(value) = cookie {
                request = request.header(axum::http::header::COOKIE, value);
            }
            let response = build_router(state.clone())
                .oneshot(request.body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert!(
                matches!(
                    response.status(),
                    StatusCode::NOT_FOUND | StatusCode::SERVICE_UNAVAILABLE
                ),
                "closed serving profile must not mount native Site work"
            );
            let bytes = to_bytes(response.into_body(), 256 * 1024).await.unwrap();
            let html = std::str::from_utf8(&bytes).unwrap();
            assert!(!html.contains("name=\"csrf_proof\""));
            assert!(!html.contains("data-native-organization"));
        }
    }
}

#[derive(Debug)]
enum ServingCorruption {
    ReservedTable,
    StartupTableSelect,
    NullStartupOid { renamed: String },
}

async fn serving_inject(connection: &mut PgConnection, corruption: &ServingCorruption) {
    let mut tx = sqlx::Connection::begin(&mut *connection).await.unwrap();
    sqlx::raw_sql("SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='30s'")
        .execute(tx.as_mut())
        .await
        .unwrap();
    match corruption {
        ServingCorruption::ReservedTable => {
            sqlx::raw_sql(
                "CREATE TABLE public.native_org_unit_unreferenced (); \
                 ALTER TABLE public.native_org_unit_unreferenced OWNER TO console_app",
            )
            .execute(tx.as_mut())
            .await
            .unwrap();
        }
        ServingCorruption::StartupTableSelect => {
            sqlx::raw_sql("GRANT SELECT ON public.org_units TO console_auth_startup")
                .execute(tx.as_mut())
                .await
                .unwrap();
        }
        ServingCorruption::NullStartupOid { renamed } => {
            assert!(
                renamed.starts_with("console_org_serving_null_")
                    && renamed.bytes().all(|byte| byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || byte == b'_')
            );
            let sql = format!("ALTER ROLE console_auth_startup RENAME TO {renamed}");
            sqlx::raw_sql(sqlx::AssertSqlSafe(sql))
                .execute(tx.as_mut())
                .await
                .unwrap();
        }
    }
    tx.commit().await.unwrap();
}

async fn serving_remove(connection: &mut PgConnection, corruption: &ServingCorruption) {
    let mut tx = sqlx::Connection::begin(&mut *connection).await.unwrap();
    sqlx::raw_sql("SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='30s'")
        .execute(tx.as_mut())
        .await
        .unwrap();
    match corruption {
        ServingCorruption::ReservedTable => {
            // Only the uniquely named object created above; never CASCADE.
            sqlx::raw_sql("DROP TABLE public.native_org_unit_unreferenced")
                .execute(tx.as_mut())
                .await
                .unwrap();
        }
        ServingCorruption::StartupTableSelect => {
            sqlx::raw_sql("REVOKE SELECT ON public.org_units FROM console_auth_startup")
                .execute(tx.as_mut())
                .await
                .unwrap();
        }
        ServingCorruption::NullStartupOid { renamed } => {
            let sql = format!("ALTER ROLE {renamed} RENAME TO console_auth_startup");
            sqlx::raw_sql(sqlx::AssertSqlSafe(sql))
                .execute(tx.as_mut())
                .await
                .unwrap();
        }
    }
    tx.commit().await.unwrap();
}

async fn serving_replay_refused(connection: &mut PgConnection, expected: &Target) {
    let mut tx = begin_protocol(connection, expected).await;
    bounds(tx.as_mut()).await;
    let failure = sqlx::raw_sql(FINALIZER)
        .execute(tx.as_mut())
        .await
        .expect_err("corrupted committed closed perimeter must not be repaired on replay");
    let database = failure
        .as_database_error()
        .expect("actual database refusal");
    assert_eq!(database.code().as_deref(), Some("P0001"));
    assert_eq!(database.message(), "native_org_unit.profile_mismatch");
    tx.rollback().await.unwrap();
}

async fn serving_current_rows_unchanged(
    connection: &mut PgConnection,
    before: &Baseline,
    corruption: Option<&ServingCorruption>,
) {
    let mut expected = before.rows.clone();
    if matches!(corruption, Some(ServingCorruption::ReservedTable)) {
        let owned_empty_relation =
            serde_json::to_string(&["public", "native_org_unit_unreferenced"]).unwrap();
        assert!(
            expected
                .insert(owned_empty_relation, "[]".to_owned())
                .is_none()
        );
    }
    assert!(
        rows(connection).await == expected,
        "readiness/corruption changed business rows"
    );
    assert!(
        applied_ledger(connection).await == before.ledger,
        "readiness/corruption changed historical ledger"
    );
}

async fn closed_serving_history(pool: &PgPool, variant: usize) {
    serving_pins();
    let predecessor = predecessor(pool, variant).await;
    let artifacts = Artifacts::new();
    let key = SigningKey::random(&mut OsRng);
    let config = account_browser_config(pool, artifacts.root.clone(), &key);
    let mut states = Vec::<AppState>::new();
    let mut admin = direct(pool).await;
    let mut injected = None::<ServingCorruption>;
    let outcome = AssertUnwindSafe(async {
        // Actual configured console_rt/auth/command pools. No terms database
        // publication, Account, Person or Company rows are inserted by this test.
        let held = AppState::from_config(config.clone())
            .await
            .expect("genuine reviewed predecessor must start before closed installation");
        states.push(held.clone());
        assert_eq!(ready_status(&held).await, StatusCode::OK);
        serving_native_org_still_closed(&held).await;
        serving_current_rows_unchanged(&mut admin, &predecessor, None).await;

        let mut install = begin_protocol(&mut admin, &predecessor.target).await;
        run_finalizer(install.as_mut()).await;
        retained_locks(install.as_mut()).await;
        // The real reviewed finalizer commits only here. Startup never installs
        // custody SQL, reseals a fingerprint, or accepts a mocked classifier.
        install.commit().await.unwrap();
        let closed = serving_snapshot(pool, variant).await;
        assert!(closed.rows == predecessor.rows);
        assert!(closed.ledger == predecessor.ledger);

        // Intended RED: current verify only reads the frozen older provenance
        // capture and rejects this genuinely installed closed catalog.
        assert_eq!(
            ready_status(&held).await,
            StatusCode::OK,
            "ORG_CLOSED_HELD_READINESS: known closed must preserve the predecessor Bridge profile"
        );
        let fresh = AppState::from_config(config.clone())
            .await
            .expect("ORG_CLOSED_FRESH_STARTUP: known closed must compose the same Bridge profile");
        states.push(fresh.clone());
        assert_eq!(ready_status(&fresh).await, StatusCode::OK);
        for state in [&held, &fresh] {
            serving_native_org_still_closed(state).await;
        }
        serving_restored(pool, variant, &closed).await;

        for corruption in [
            ServingCorruption::ReservedTable,
            ServingCorruption::StartupTableSelect,
            ServingCorruption::NullStartupOid {
                renamed: format!("console_org_serving_null_{}", Uuid::new_v4().simple()),
            },
        ] {
            serving_inject(&mut admin, &corruption).await;
            injected = Some(corruption);
            let corruption = injected.as_ref().unwrap();
            if let ServingCorruption::NullStartupOid { renamed } = corruption {
                let actual: (Option<String>, Option<bool>, String) = sqlx::query_as(
                    "SELECT (SELECT oid::text FROM pg_catalog.pg_roles WHERE rolname='console_auth_startup'), \
                      pg_catalog.has_table_privilege((SELECT oid FROM pg_catalog.pg_roles WHERE rolname='console_auth_startup'), \
                        'public.org_units'::regclass::oid,'SELECT'), \
                      (SELECT oid::text FROM pg_catalog.pg_roles WHERE rolname=$1)",
                )
                .bind(renamed)
                .fetch_one(&mut admin)
                .await
                .unwrap();
                assert_eq!(actual.0, None, "actual missing role OID required");
                assert_eq!(actual.1, None, "actual effective-privilege SQL NULL required");
                assert_eq!(actual.2, closed.denied["role"][0]["oid"].as_str().unwrap());
            }
            let mut verify = sqlx::Connection::begin(&mut admin).await.unwrap();
            match corruption {
                ServingCorruption::ReservedTable => {
                    let actual: i64 = sqlx::query_scalar(
                        "SELECT count(*) FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace \
                         WHERE c.relname='native_org_unit' OR starts_with(c.relname,'native_org_unit_')"
                    ).fetch_one(verify.as_mut()).await.unwrap();
                    assert_eq!(actual, 1, "real all-schema reserved relation corruption required");
                    assert_eq!(capture(verify.as_mut(), CAPTURE).await.sha256, CLOSED73[variant],
                        "this control must prove namespace denial independently of frozen hashes");
                    assert_eq!(raw_wider_capture(verify.as_mut()).await["snapshot_sha256"], CLOSED76[variant]);
                }
                ServingCorruption::StartupTableSelect => {
                    let actual: Option<bool> = sqlx::query_scalar(
                        "SELECT pg_catalog.has_table_privilege('console_auth_startup','public.org_units','SELECT')"
                    ).fetch_one(verify.as_mut()).await.unwrap();
                    assert_eq!(actual, Some(true), "real effective startup privilege corruption required");
                }
                ServingCorruption::NullStartupOid { .. } => {}
            }
            assert_eq!(state(verify.as_mut()).await, "native_org_unit.profile_mismatch");
            verify.rollback().await.unwrap();
            for held_state in [&held, &fresh] {
                assert_eq!(ready_status(held_state).await, StatusCode::SERVICE_UNAVAILABLE);
            }
            serving_fresh_refused(config.clone()).await;
            serving_replay_refused(&mut admin, &closed.target).await;
            serving_current_rows_unchanged(&mut admin, &closed, Some(corruption)).await;
            serving_remove(&mut admin, corruption).await;
            injected = None;
            serving_restored(pool, variant, &closed).await;
            for held_state in [&held, &fresh] {
                assert_eq!(ready_status(held_state).await, StatusCode::OK);
                serving_native_org_still_closed(held_state).await;
            }
            let recovered = AppState::from_config(config.clone())
                .await
                .expect("exact restoration must recover fresh startup without broadening custody");
            states.push(recovered.clone());
            assert_eq!(ready_status(&recovered).await, StatusCode::OK);
            recovered.shutdown_realtime().await;
            states.pop();
            drop(recovered);
            serving_restored(pool, variant, &closed).await;
        }
    })
    .catch_unwind()
    .await;
    // Restore our exact cluster role/object/ACL even if any assertion fails.
    // Root grants one dedicated disposable cluster per exact leaf; this is not
    // safe to execute against a shared or production cluster.
    if let Some(corruption) = injected.as_ref() {
        serving_remove(&mut admin, corruption).await;
    }
    admin.close().await.unwrap();
    close_states(&states, outcome).await;
    drop(artifacts);
}

#[sqlx::test(migrations = false)]
async fn closed_serving_maps_closed_only_to_bridge_and_fences_held_fresh_on_corruption(
    pool: PgPool,
) {
    closed_serving_history(&pool, 0).await;
}

#[sqlx::test(migrations = false)]
async fn closed_serving_observer_maps_closed_only_to_bridge_and_fences_held_fresh_on_corruption(
    pool: PgPool,
) {
    closed_serving_history(&pool, 1).await;
}
