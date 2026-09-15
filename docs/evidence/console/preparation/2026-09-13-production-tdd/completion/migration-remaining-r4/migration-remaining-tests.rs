//! Child of frozen migration-transition-acceptance.rs: exact shared legacy225
//! producer and mapping/readback helpers. No parallel business or auth pipeline.
use super::{
    apply_current_and_require_account_catalog, mapping_partition, seed_legacy, snapshot_mapping,
    source_keys,
};
use console_app::{account_migration as migration, serving_admission as admission};
use console_kernel_core::OrgId;
use console_platform_auth::RefreshTokenStore;
use console_platform_test_support::seed_org_and_super_admin;
use serde_json::Value;
use sqlx::PgPool;
use std::collections::BTreeSet;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[sqlx::test(migrations = false)]
async fn migration_related_refresh_revocation_invalidates_source_even_when_user_xmin_is_unchanged(
    pool: PgPool,
) -> TestResult {
    let f = seed_legacy(&pool).await;
    let org: Uuid = sqlx::query_scalar("SELECT org_id FROM users WHERE id=$1")
        .bind(f.subjects[0])
        .fetch_one(&pool)
        .await?;
    let family = RefreshTokenStore
        .issue_family(
            &pool,
            f.subjects[0],
            OrgId::from_uuid(org),
            OffsetDateTime::now_utc(),
            Duration::hours(2),
        )
        .await?;
    apply_current_and_require_account_catalog(&pool, &f).await;
    let manifest = migration::load_installed_manifest(&f.migrate_config).await?;
    let source = migration::capture_source_snapshot(&f.migrate_config, &manifest).await?;
    let users_before = source_keys(&pool, &f.subjects).await?;
    let before: (String, Option<OffsetDateTime>) =
        sqlx::query_as("SELECT xmin::text,revoked_at FROM auth_refresh_token_families WHERE id=$1")
            .bind(family.family_id)
            .fetch_one(&pool)
            .await?;
    assert!(before.1.is_none());
    // Ordinary security revocation changes the actual dependent credential
    // source, without touching users. It cannot be hidden behind users.xmin.
    RefreshTokenStore
        .revoke_family_for_logout(&pool, family.token.as_str(), OffsetDateTime::now_utc())
        .await?;
    let after: (String, Option<OffsetDateTime>) =
        sqlx::query_as("SELECT xmin::text,revoked_at FROM auth_refresh_token_families WHERE id=$1")
            .bind(family.family_id)
            .fetch_one(&pool)
            .await?;
    assert_ne!(after.0, before.0);
    assert!(after.1.is_some());
    assert_eq!(source_keys(&pool, &f.subjects).await?, users_before);
    let mappings = snapshot_mapping(&pool).await?;
    let request = migration::BatchRequest {
        manifest_digest: manifest.digest().into(),
        source_cursor: source.initial_cursor().clone(),
        source_version: source.version().into(),
        batch_id: Uuid::new_v4(),
        limits: migration::BatchLimits {
            max_rows: 500,
            max_bytes: 4 * 1024 * 1024,
            statement_timeout_ms: 2000,
        },
    };
    assert!(matches!(
        migration::migrate_batch(&f.migrate_config, &request).await,
        Err(migration::MigrationError::SourceChanged)
    ));
    assert_eq!(snapshot_mapping(&pool).await?, mappings);
    let preserved: Option<OffsetDateTime> =
        sqlx::query_scalar("SELECT revoked_at FROM auth_refresh_token_families WHERE id=$1")
            .bind(family.family_id)
            .fetch_one(&pool)
            .await?;
    assert_eq!(preserved, after.1);
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn migration_concurrent_distinct_batch_ids_cannot_duplicate_source_partition(
    pool: PgPool,
) -> TestResult {
    let f = seed_legacy(&pool).await;
    apply_current_and_require_account_catalog(&pool, &f).await;
    let manifest = migration::load_installed_manifest(&f.migrate_config).await?;
    let source = migration::capture_source_snapshot(&f.migrate_config, &manifest).await?;
    let expected = source_keys(&pool, &f.subjects).await?;
    assert_eq!(expected.len(), 2);
    let left = migration::BatchRequest {
        manifest_digest: manifest.digest().into(),
        source_cursor: source.initial_cursor().clone(),
        source_version: source.version().into(),
        batch_id: Uuid::new_v4(),
        limits: migration::BatchLimits {
            max_rows: 500,
            max_bytes: 4 * 1024 * 1024,
            statement_timeout_ms: 2000,
        },
    };
    let mut right = left.clone();
    right.batch_id = Uuid::new_v4();
    assert_ne!(left.batch_id, right.batch_id);
    let (a, b) = tokio::join!(
        migration::migrate_batch(&f.migrate_config, &left),
        migration::migrate_batch(&f.migrate_config, &right)
    );
    let mut seen = BTreeSet::new();
    let mut successes = Vec::new();
    for (request, outcome) in [(&left, a), (&right, b)] {
        match outcome {
            Ok(receipt) => {
                assert_eq!(receipt.batch_id, request.batch_id);
                for key in &receipt.source_keys {
                    assert!(expected.contains(key));
                    assert!(
                        seen.insert(key.clone()),
                        "different batch IDs re-consumed the same exact source"
                    );
                }
                successes.push((request, receipt));
            }
            Err(
                migration::MigrationError::WriterBusy
                | migration::MigrationError::CursorAlreadyConsumed,
            ) => {}
            Err(_) => panic!("only precise single-writer/cursor contention counts as a safe loser"),
        }
    }
    assert!(!successes.is_empty());
    assert_eq!(seen, expected);
    let (mapped, quarantined) = mapping_partition(&pool, manifest.digest()).await?;
    assert_eq!(
        mapped.union(&quarantined).cloned().collect::<BTreeSet<_>>(),
        expected
    );
    let before = snapshot_mapping(&pool).await?;
    for (request, receipt) in successes {
        assert_eq!(
            migration::migrate_batch(&f.migrate_config, request).await?,
            receipt
        );
    }
    assert_eq!(snapshot_mapping(&pool).await?, before);
    Ok(())
}

async fn wait_blocked(pool: &PgPool, blocker: i32) -> TestResult<i32> {
    for _ in 0..400 {
        let pids:Vec<i32>=sqlx::query_scalar("SELECT pid FROM pg_stat_activity WHERE datname=current_database() AND $1=ANY(pg_blocking_pids(pid)) ORDER BY pid")
            .bind(blocker).fetch_all(pool).await?;
        if pids.len() == 1 {
            return Ok(pids[0]);
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    Err("actual startup/migration lock witness absent".into())
}
#[sqlx::test(migrations = false)]
async fn migration_startup_waits_for_actual_migration_ledger_then_rejects_changed_catalog(
    pool: PgPool,
) -> TestResult {
    let f = seed_legacy(&pool).await;
    apply_current_and_require_account_catalog(&pool, &f).await;
    let installed = admission::load_installed_release(&f.migrate_config).await?;
    let healthy = admission::admit_process(&f.migrate_config, &installed).await?;
    assert!(healthy.command_service().is_some());
    let pause = admission::pause_and_drain_effects(&f.migrate_config).await?;
    let mut migration_tx = pool.begin().await?;
    let blocker: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *migration_tx)
        .await?;
    sqlx::query("LOCK TABLE _sqlx_migrations IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *migration_tx)
        .await?;
    let startup = admission::admit_process(&f.migrate_config, &installed);
    tokio::pin!(startup);
    tokio::select! {
        r=&mut startup=>{r?;panic!("startup admitted without current migration ledger lock")},
        p=wait_blocked(&pool,blocker)=>{assert_ne!(p?,blocker);}
    }
    // Real pending migration DDL commits behind the held authoritative ledger.
    // No manifest fields or returned admission bindings are edited by the test.
    sqlx::query(
        "ALTER TABLE payroll_input_units RENAME TO fixture_startup_incomplete_payroll_input_units",
    )
    .execute(&mut *migration_tx)
    .await?;
    migration_tx.commit().await?;
    let outcome = tokio::time::timeout(std::time::Duration::from_secs(10), &mut startup).await;
    sqlx::query(
        "ALTER TABLE fixture_startup_incomplete_payroll_input_units RENAME TO payroll_input_units",
    )
    .execute(&pool)
    .await?;
    assert!(matches!(
        outcome?,
        Err(admission::AdmissionError::CatalogMismatch { .. })
    ));
    let current = admission::admit_process(&f.migrate_config, &installed).await?;
    assert!(current.command_service().is_some());
    admission::resume_effects(&f.migrate_config, &pause, &current).await?;
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn migration_actual_process_loss_after_committed_batch_replays_original_receipt(
    pool: PgPool,
) -> TestResult {
    let mut f = seed_legacy(&pool).await;
    let org: Uuid = sqlx::query_scalar("SELECT org_id FROM users WHERE id=$1")
        .bind(f.subjects[0])
        .fetch_one(&pool)
        .await?;
    for n in 0..498 {
        f.subjects.push(
            *seed_org_and_super_admin(&pool, org, &format!("actual-postcommit-loss-{n}"))
                .await
                .as_uuid(),
        );
    }
    assert_eq!(f.subjects.len(), 500);
    apply_current_and_require_account_catalog(&pool, &f).await;
    let manifest = migration::load_installed_manifest(&f.migrate_config).await?;
    let source = migration::capture_source_snapshot(&f.migrate_config, &manifest).await?;
    let expected = source_keys(&pool, &f.subjects).await?;
    let request = migration::BatchRequest {
        manifest_digest: manifest.digest().into(),
        source_cursor: source.initial_cursor().clone(),
        source_version: source.version().into(),
        batch_id: Uuid::new_v4(),
        limits: migration::BatchLimits {
            max_rows: 500,
            max_bytes: 4 * 1024 * 1024,
            statement_timeout_ms: 2000,
        },
    };
    let mut child = crate::migration_batch_process::BatchProcess::launch_blocked_output(
        &f.migrate_config,
        &request,
    )
    .await?;
    tokio::time::timeout(std::time::Duration::from_secs(15), async {
        loop {
            let committed: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM account_migration_batch_receipts WHERE batch_id=$1)",
            )
            .bind(request.batch_id)
            .fetch_one(&pool)
            .await?;
            if committed {
                break;
            }
            assert!(
                child.still_running()?,
                "batch process exited before durable receipt witness"
            );
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    })
    .await??;
    let original = migration::read_batch_receipt(&f.migrate_config, request.batch_id)
        .await?
        .ok_or("actual committed batch receipt missing")?;
    assert_eq!(original.batch_id, request.batch_id);
    assert_eq!(original.source_keys.len(), 500);
    // Full ordinary CLI receipt cannot fit in the deliberately unread socket.
    // The process is alive after commit, then is REALLY terminated by its owner.
    assert!(serde_json::to_vec(&original)?.len() > child.stdout_capacity * 2);
    assert!(child.still_running()?);
    let before = snapshot_mapping(&pool).await?;
    child.kill_after_committed_receipt().await?;
    let replay =
        crate::migration_batch_process::BatchProcess::run_and_read(&f.migrate_config, &request)
            .await?;
    assert_eq!(replay, original);
    assert_eq!(snapshot_mapping(&pool).await?, before);
    let (mapped, quarantined) = mapping_partition(&pool, manifest.digest()).await?;
    assert_eq!(
        mapped.union(&quarantined).cloned().collect::<BTreeSet<_>>(),
        expected
    );
    Ok(())
}

#[path = "legacy-transport-producer.rs"]
mod legacy_transport;
#[path = "migrated-primary-producer.rs"]
mod migrated_primary;

async fn legacy_security_snapshot(pool: &PgPool, subjects: &[Uuid]) -> TestResult<Value> {
    // Only preserved legacy rows: no opaque success object or expected grants.
    let keys: Vec<Value> = sqlx::query_scalar(
        "SELECT to_jsonb(k) FROM auth_webauthn_credentials k WHERE user_id=ANY($1) ORDER BY id",
    )
    .bind(subjects)
    .fetch_all(pool)
    .await?;
    let families: Vec<Value> = sqlx::query_scalar(
        "SELECT to_jsonb(f) FROM auth_refresh_token_families f WHERE user_id=ANY($1) ORDER BY id",
    )
    .bind(subjects)
    .fetch_all(pool)
    .await?;
    let tokens: Vec<Value> = sqlx::query_scalar(
        "SELECT to_jsonb(t) FROM auth_refresh_tokens t WHERE user_id=ANY($1) ORDER BY id",
    )
    .bind(subjects)
    .fetch_all(pool)
    .await?;
    let bootstraps: Vec<Value> = sqlx::query_scalar(
        "SELECT to_jsonb(b) FROM auth_bootstrap_credentials b WHERE user_id=ANY($1) ORDER BY id",
    )
    .bind(subjects)
    .fetch_all(pool)
    .await?;
    Ok(serde_json::json!({"keys":keys,"families":families,"tokens":tokens,"bootstraps":bootstraps}))
}
#[sqlx::test(migrations = false)]
async fn migration_real_legacy_transports_drain_before_fence_and_stay_denied_after_fresh_primary_activation(
    pool: PgPool,
) -> TestResult {
    let f = seed_legacy(&pool).await;
    let mut transport = legacy_transport::build(&pool, &f.subjects).await?;
    assert_eq!(transport.groups.len(), 2);
    apply_current_and_require_account_catalog(&pool, &f).await;
    let manifest = migration::load_installed_manifest(&f.migrate_config).await?;
    // Before permanent cutover, pause rejects concurrently submitted real old
    // credentials and drain removes the actual already accepted WS connection.
    let pausing = migration::pause_legacy_admission(&f.migrate_config, &manifest).await?;
    let (denials, drained) = tokio::join!(
        transport.concurrent_denials(),
        migration::drain_legacy_work(&f.migrate_config, &pausing)
    );
    denials?;
    let drained = drained?;
    assert!(drained.unresolved_effect_refs().is_empty());
    transport.require_drained_live().await?;
    let prior: i64 =
        sqlx::query_scalar("SELECT count(*) FROM account_security WHERE account_id=ANY($1)")
            .bind(&f.subjects)
            .fetch_one(&pool)
            .await?;
    assert_eq!(
        prior, 0,
        "Account security cutover rows must not precede actual legacy drain"
    );
    let fence = migration::commit_account_cutover(&f.migrate_config, &drained).await?;
    let permanent: (Uuid, i64) = sqlx::query_as(
        "SELECT id,generation FROM legacy_authentication_fences WHERE id=$1 AND permanent",
    )
    .bind(fence.id())
    .fetch_one(&pool)
    .await?;
    assert_eq!(permanent, (fence.id(), fence.generation()));
    let state: (String, i64) = sqlx::query_as(
        "SELECT security_state,security_generation FROM account_security WHERE account_id=$1",
    )
    .bind(f.subjects[0])
    .fetch_one(&pool)
    .await?;
    assert_eq!(
        state.0, "PENDING_ENROLLMENT",
        "valid retained key awaits fresh Account primary activation"
    );
    let revoked_state: String =
        sqlx::query_scalar("SELECT security_state FROM account_security WHERE account_id=$1")
            .bind(f.subjects[1])
            .fetch_one(&pool)
            .await?;
    assert_eq!(
        revoked_state, "RECOVERY_REQUIRED",
        "pre-deleted key cannot be restored into primary eligibility"
    );
    let before = legacy_security_snapshot(&pool, &f.subjects).await?;
    transport.concurrent_denials().await?;
    assert_eq!(legacy_security_snapshot(&pool, &f.subjects).await?, before);
    // Real configured deployment/terms publisher and retained-key activation,
    // followed by a SECOND real fresh primary login. No SQL state toggle.
    let activated = migrated_primary::activate_and_login(&pool, &mut transport).await?;
    assert_eq!(activated.account_id(), f.subjects[0]);
    let active: (String, i64) = sqlx::query_as(
        "SELECT security_state,security_generation FROM account_security WHERE account_id=$1",
    )
    .bind(f.subjects[0])
    .fetch_one(&pool)
    .await?;
    assert_eq!(active.0, "ACTIVE");
    assert!(active.1 > state.1);
    assert_eq!(activated.security_generation(), active.1);
    let after_primary = legacy_security_snapshot(&pool, &f.subjects).await?;
    transport.concurrent_denials().await?;
    assert_eq!(
        legacy_security_snapshot(&pool, &f.subjects).await?,
        after_primary,
        "old credential refusal cannot mint/rotate/reset even after ACTIVE"
    );
    let revoked: i64 =
        sqlx::query_scalar("SELECT count(*) FROM auth_webauthn_credentials WHERE credential_id=$1")
            .bind(&transport.revoked_key)
            .fetch_one(&pool)
            .await?;
    assert_eq!(revoked, 0);
    assert_eq!(
        sqlx::query_scalar::<_, String>(
            "SELECT security_state FROM account_security WHERE account_id=$1"
        )
        .bind(f.subjects[1])
        .fetch_one(&pool)
        .await?,
        "RECOVERY_REQUIRED"
    );
    assert_eq!(
        sqlx::query_as::<_, (Uuid, i64)>(
            "SELECT id,generation FROM legacy_authentication_fences WHERE id=$1 AND permanent"
        )
        .bind(fence.id())
        .fetch_one(&pool)
        .await?,
        permanent
    );
    Ok(())
}
