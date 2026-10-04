// V4 diagnostic and actual startup RED; no product SQL, generated successor
// classifier, accepted corrected hash, business population or new installer.
const NAVIGATION_IDENTITY: &str =
    "public.identity_native_group_process_navigation_candidates_v1(uuid, uuid, bytea)";

struct NavigationCorrection {
    original: String,
    corrected: String,
    original_body: String,
    corrected_body: String,
}

fn navigation_correction() -> NavigationCorrection {
    group_pins();
    let header = "CREATE FUNCTION public.identity_native_group_process_navigation_candidates_v1(\n";
    assert_eq!(GROUP_OWNER.matches(header).count(), 1);
    let remainder = GROUP_OWNER.split_once(header).unwrap().1;
    let terminator = "\n$body$;\n";
    let end = remainder
        .find(terminator)
        .expect("pinned navigation terminator");
    let original = format!("{header}{}", &remainder[..end + terminator.len()]);
    assert!(original.starts_with(&format!(
        "{header} p_actor uuid,p_family uuid,p_original bytea) RETURNS jsonb\nLANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE\n"
    )));
    assert_eq!(original.matches("AS $body$").count(), 1);
    assert_eq!(original.matches(terminator).count(), 1);
    assert!(!original.contains("v_navigation_head_revision"));
    let mut corrected = original.replacen("CREATE FUNCTION", "CREATE OR REPLACE FUNCTION", 1);
    let changes = [
        (
            "head_revision bigint",
            "v_navigation_head_revision bigint",
            1,
        ),
        ("head_revision:=", "v_navigation_head_revision:=", 5),
        (
            "<>head_revision::numeric",
            "<>v_navigation_head_revision::numeric",
            2,
        ),
        ("IF head_revision>0", "IF v_navigation_head_revision>0", 2),
        (
            "h.head_revision<=head_revision ORDER BY h.head_revision",
            "h.head_revision<=v_navigation_head_revision ORDER BY h.head_revision",
            1,
        ),
    ];
    for (before, after, count) in changes {
        assert_eq!(
            corrected.matches(before).count(),
            count,
            "exact local rename frame"
        );
        corrected = corrected.replace(before, after);
    }
    assert_eq!(corrected.matches("v_navigation_head_revision").count(), 11);
    let inverse = corrected
        .replace("v_navigation_head_revision", "head_revision")
        .replacen("CREATE OR REPLACE FUNCTION", "CREATE FUNCTION", 1);
    assert_eq!(
        inverse, original,
        "inverse must recover every original source byte"
    );
    let body = |source: &str| {
        source
            .split_once("AS $body$")
            .unwrap()
            .1
            .strip_suffix("$body$;\n")
            .unwrap()
            .to_owned()
    };
    NavigationCorrection {
        original_body: body(&original),
        corrected_body: body(&corrected),
        original,
        corrected,
    }
}

fn navigation_catalog_delta(before: &Value, after: &Value, correction: &NavigationCorrection) {
    let old = routine_map(before);
    let new = routine_map(after);
    assert_eq!(
        old.len(),
        new.len(),
        "correction must preserve every routine identity"
    );
    let old_navigation = old
        .get(NAVIGATION_IDENTITY)
        .expect("actual pinned navigation routine");
    let new_navigation = new.get(NAVIGATION_IDENTITY).expect("same navigation ABI");
    assert_eq!(old_navigation["raw"]["prosrc"], correction.original_body);
    assert_eq!(new_navigation["raw"]["prosrc"], correction.corrected_body);
    assert_eq!(
        old_navigation["source_sha256"],
        digest(&correction.original_body)
    );
    assert_eq!(
        new_navigation["source_sha256"],
        digest(&correction.corrected_body)
    );
    let original_definition = old_navigation["definition"].as_str().unwrap();
    assert_eq!(
        original_definition
            .matches(correction.original_body.as_str())
            .count(),
        1
    );
    assert_eq!(
        new_navigation["definition"],
        original_definition.replace(
            correction.original_body.as_str(),
            &correction.corrected_body
        ),
        "actual definition may change only the eleven local sites"
    );
    assert_ne!(
        before, after,
        "catalog comparison must detect the correction"
    );
    let mut restored_catalog = after.clone();
    let routines = restored_catalog["routines"].as_array_mut().unwrap();
    let matching: Vec<_> = routines
        .iter_mut()
        .filter(|routine| {
            routine["schema"] == "public"
                && routine["name"] == "identity_native_group_process_navigation_candidates_v1"
                && routine["type_arguments"] == "uuid, uuid, bytea"
        })
        .collect();
    assert_eq!(matching.len(), 1);
    let navigation = matching.into_iter().next().unwrap();
    navigation["raw"]["prosrc"] = old_navigation["raw"]["prosrc"].clone();
    navigation["source_sha256"] = old_navigation["source_sha256"].clone();
    navigation["definition"] = old_navigation["definition"].clone();
    assert_eq!(
        restored_catalog, *before,
        "only navigation prosrc/source_sha256/definition may differ in the complete actual catalog"
    );
}

async fn navigation_apply(connection: &mut PgConnection, correction: &NavigationCorrection) {
    bounds(connection).await;
    sqlx::raw_sql(sqlx::AssertSqlSafe(correction.corrected.clone()))
        .execute(connection)
        .await
        .unwrap();
}

#[sqlx::test(migrations = false)]
async fn navigation_corrected_full83_plain_observer_capture_and_rollback(pool: PgPool) {
    let sources = group_pins();
    let oracle = group_oracle();
    let correction = navigation_correction();
    let original = predecessor(&pool, 0).await;
    let closed = group_closed_fixture(&pool, &original, 0).await;
    let mut packets = Vec::new();
    for variant in 0..2 {
        let mut admin = direct(&pool).await;
        let mut tx = begin_protocol(&mut admin, &closed.target).await;
        let outcome = AssertUnwindSafe(async {
            // The observer is wholly inside this variant's rollback boundary;
            // predecessor() is not rerun against a committed non-fresh cluster.
            if variant == 1 {
                execute(&mut tx, OBSERVER).await;
            }
            assert_eq!(capture(tx.as_mut(), CAPTURE).await.sha256, CLOSED73[variant]);
            assert_eq!(raw_wider_capture(tx.as_mut()).await["snapshot_sha256"], CLOSED76[variant]);
            let variant_before = Baseline {
                target: closed.target.clone(),
                rows: rows(tx.as_mut()).await,
                catalog: catalog(tx.as_mut()).await,
                ledger: applied_ledger(tx.as_mut()).await,
                denied: added_three_denied_rights(tx.as_mut()).await,
            };
            assert!(variant_before.rows == closed.rows);
            assert!(variant_before.ledger == closed.ledger);
            let installation = group_install_replay(tx.as_mut(), &variant_before, variant, &oracle).await;
            let installed = group_capture(tx.as_mut(), true).await;
            assert_eq!(installed.sha256, INSTALLED83[variant]);
            let installed_catalog = catalog(tx.as_mut()).await;
            let installed_rows = rows(tx.as_mut()).await;
            let locks = group_locks(tx.as_mut(), &oracle).await;
            sqlx::raw_sql("SAVEPOINT navigation_diagnostic")
                .execute(tx.as_mut()).await.unwrap();
            navigation_apply(tx.as_mut(), &correction).await;
            let corrected = group_capture(tx.as_mut(), true).await;
            assert_ne!(corrected.sha256, installed.sha256);
            let corrected_catalog = catalog(tx.as_mut()).await;
            navigation_catalog_delta(&installed_catalog.1, &corrected_catalog.1, &correction);
            assert_eq!(group_classification(tx.as_mut()).await, ("native_group_process.profile_mismatch".into(), None));
            assert!(group_capture(tx.as_mut(), true).await == corrected);
            assert!(catalog(tx.as_mut()).await == corrected_catalog);
            assert!(rows(tx.as_mut()).await == installed_rows);
            assert!(applied_ledger(tx.as_mut()).await == closed.ledger);
            assert_eq!(group_locks(tx.as_mut(), &oracle).await, locks);
            // Actual savepoint rollback, never executing historical CREATE over
            // the installed routine or inventing a successor finalizer/profile.
            sqlx::raw_sql("ROLLBACK TO SAVEPOINT navigation_diagnostic; RELEASE SAVEPOINT navigation_diagnostic")
                .execute(tx.as_mut()).await.unwrap();
            assert!(group_capture(tx.as_mut(), true).await == installed);
            assert!(catalog(tx.as_mut()).await == installed_catalog);
            assert!(rows(tx.as_mut()).await == installed_rows);
            assert!(applied_ledger(tx.as_mut()).await == closed.ledger);
            assert_eq!(group_classification(tx.as_mut()).await,
                ("native_group_process.finalized".into(), Some(if variant == 0 { "plain" } else { "observer" }.into())));
            navigation_apply(tx.as_mut(), &correction).await;
            assert!(group_capture(tx.as_mut(), true).await == corrected);
            assert!(catalog(tx.as_mut()).await == corrected_catalog);
            assert!(rows(tx.as_mut()).await == installed_rows);
            assert!(applied_ledger(tx.as_mut()).await == closed.ledger);
            assert_eq!(group_locks(tx.as_mut(), &oracle).await, locks);
            let failure = sqlx::query("SELECT 1/0").execute(tx.as_mut()).await
                .expect_err("diagnostic must exercise actual PostgreSQL precommit failure");
            let database = failure.as_database_error().expect("actual PostgreSQL injected error");
            assert_eq!(database.code().as_deref(), Some("22012"));
            json!({"schema":"console.group_navigation_correction_diagnostic.v1",
                "variant":if variant == 0 { "plain" } else { "observer" },"sources":sources,
                "original_definition_sha256":digest(&correction.original),
                "corrective_definition_sha256":digest(&correction.corrected),"local_rename_sites":11,
                "installation":installation,"full83_installed_v1":installed.record(),
                "full83_corrected":corrected.record(),"raw_catalog_installed_text":installed_catalog.0,
                "raw_catalog_corrected_text":corrected_catalog.0,
                "raw_catalog_installed_sha256":digest(&installed_catalog.0),
                "raw_catalog_corrected_sha256":digest(&corrected_catalog.0),
                "only_navigation_body_digest_definition_changed":true,"rights_both_true":true,
                "savepoint_original_exactly_restored":true,"recorrection_byte_identical":true,
                "diagnostic_injected_sqlstate":"22012","ledger_records":231,"rows_unchanged":true,
                "corrected_hashes_are_observations_not_accepted_authority":true,
                "business_empty":false,"group_browser_accepted":false,"production_qualified":false})
        }).catch_unwind().await;
        tx.rollback()
            .await
            .expect("whole variant, observer, Group and correction must roll back");
        admin.close().await.unwrap();
        serving_restored(&pool, 0, &closed).await;
        let mut packet = match outcome {
            Ok(packet) => packet,
            Err(panic) => std::panic::resume_unwind(panic),
        };
        packet["complete_catalog_rows_ledger_target_denials_rollback_verified"] = json!(true);
        packets.push(packet);
    }
    assert_eq!(packets.len(), 2);
    assert_ne!(
        packets[0]["full83_corrected"]["snapshot_sha256"],
        packets[1]["full83_corrected"]["snapshot_sha256"]
    );
    let mut output = std::io::stderr().lock();
    for packet in packets {
        writeln!(
            &mut output,
            "GROUP_NAVIGATION_CORRECTION_DIAGNOSTIC {packet}"
        )
        .unwrap();
    }
    output.flush().unwrap();
}

#[sqlx::test(migrations = false)]
async fn navigation_corrected_actual_api_worker_startup_acceptance(pool: PgPool) {
    let sources = group_pins();
    let oracle = group_oracle();
    let correction = navigation_correction();
    let original = predecessor(&pool, 0).await;
    let closed = group_closed_fixture(&pool, &original, 0).await;
    let artifacts = Artifacts::new();
    let key = SigningKey::random(&mut OsRng);
    let config = account_browser_config(&pool, artifacts.root.clone(), &key);
    let mut states = Vec::<AppState>::new();
    let mut admin = direct(&pool).await;
    let outcome = AssertUnwindSafe(async {
        let mut install = begin_protocol(&mut admin, &closed.target).await;
        group_install_replay(install.as_mut(), &closed, 0, &oracle).await;
        let installed = group_capture(install.as_mut(), true).await;
        assert_eq!(installed.sha256, INSTALLED83[0]);
        let installed_catalog = catalog(install.as_mut()).await;
        let installed_rows = rows(install.as_mut()).await;
        install.commit().await.unwrap();
        for role in [console_app::AppRole::Api, console_app::AppRole::Worker] {
            let mut held_config = config.clone();
            held_config.role = role;
            let held = AppState::from_config(held_config).await
                .expect("frozen committed v1 positive must start API and Worker");
            states.push(held);
            assert_eq!(ready_status(states.last().unwrap()).await, StatusCode::OK);
        }
        assert_eq!(target(&mut admin).await, closed.target);
        let mut correction_tx = begin_protocol(&mut admin, &closed.target).await;
        assert!(group_capture(correction_tx.as_mut(), true).await == installed);
        assert!(catalog(correction_tx.as_mut()).await == installed_catalog);
        assert!(rows(correction_tx.as_mut()).await == installed_rows);
        assert!(applied_ledger(correction_tx.as_mut()).await == closed.ledger);
        group_run(correction_tx.as_mut()).await;
        let locks = group_locks(correction_tx.as_mut(), &oracle).await;
        navigation_apply(correction_tx.as_mut(), &correction).await;
        let corrected = group_capture(correction_tx.as_mut(), true).await;
        assert_ne!(corrected.sha256, installed.sha256);
        let corrected_catalog = catalog(correction_tx.as_mut()).await;
        navigation_catalog_delta(&installed_catalog.1, &corrected_catalog.1, &correction);
        assert!(rows(correction_tx.as_mut()).await == installed_rows);
        assert!(applied_ledger(correction_tx.as_mut()).await == closed.ledger);
        assert_eq!(group_locks(correction_tx.as_mut(), &oracle).await, locks);
        correction_tx.commit().await.unwrap();
        for held in &states {
            assert_eq!(ready_status(held).await, StatusCode::SERVICE_UNAVAILABLE,
                "held v1 reader must fence the committed corrective successor");
        }
        let mut fresh_outcomes = Vec::new();
        for (name, role) in [("api", console_app::AppRole::Api), ("worker", console_app::AppRole::Worker)] {
            let mut fresh_config = config.clone();
            fresh_config.role = role;
            let record = match AppState::from_config(fresh_config).await {
                Ok(fresh) => {
                    states.push(fresh);
                    let status = ready_status(states.last().unwrap()).await;
                    json!({"role":name,"accepted":true,"ready_status":status.as_u16()})
                }
                Err(AppError::Config(code)) => json!({"role":name,"accepted":false,"error_kind":"config","code":code}),
                Err(_) => json!({"role":name,"accepted":false,"error_kind":"unrelated_startup_failure"}),
            };
            fresh_outcomes.push(record);
        }
        // Both actual startup attempts precede assertions; missing includes,
        // schemas/pools/artifacts never qualify as the intended custody RED.
        assert_eq!(target(&mut admin).await, closed.target);
        let mut readback = sqlx::Connection::begin(&mut admin).await.unwrap();
        assert!(group_capture(readback.as_mut(), true).await == corrected);
        assert!(catalog(readback.as_mut()).await == corrected_catalog);
        assert!(rows(readback.as_mut()).await == installed_rows);
        assert!(applied_ledger(readback.as_mut()).await == closed.ledger);
        readback.rollback().await.unwrap();
        let mut output = std::io::stderr().lock();
        writeln!(&mut output, "GROUP_NAVIGATION_CORRECTED_STARTUP {}", json!({
            "sources":sources,"variant":"plain","local_rename_sites":11,
            "original_definition_sha256":digest(&correction.original),
            "corrective_definition_sha256":digest(&correction.corrected),
            "installed_v1_full83_sha256":installed.sha256,"corrected_full83_sha256":corrected.sha256,
            "held_api_worker_before":200,"held_api_worker_after":503,"fresh_outcomes":fresh_outcomes,
            "postcommit_complete_catalog_rows_ledger_unchanged":true,
            "business_empty":false,"group_browser_accepted":false,"production_qualified":false
        })).unwrap();
        output.flush().unwrap();
        drop(output);
        assert_eq!(fresh_outcomes.len(), 2);
        for result in &fresh_outcomes {
            assert!((result["accepted"] == true && result["ready_status"] == 200) || (result["error_kind"] == "config"
                && result["code"] == "native_group_process.profile_mismatch"),
                "unrelated failure is not the admitted corrected-startup RED: {result}");
        }
        assert!(fresh_outcomes.iter().all(|result| result["accepted"] == true && result["ready_status"] == 200),
            "GROUP_NAVIGATION_CORRECTED_FRESH_STARTUP: corrected metadata must compose API and Worker ready200; observed {fresh_outcomes:?}");
    }).catch_unwind().await;
    // Complete owned resource cleanup before propagating the intentional RED.
    close_states(&states, Ok(())).await;
    states.clear();
    admin.close().await.unwrap();
    drop(artifacts);
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
}
