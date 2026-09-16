//! Append child module to existing account_migration.rs. Uses its immutable225
//! migration setup and real legacy WebAuthn/refresh/audit producer. All mapping,
//! pause/drain, admission and recovery calls below belong to actual app owners.
use super::{apply_current_and_require_account_catalog, material_snapshot, seed_legacy};
use console_app::account_migration as migration;
use console_app::serving_admission as admission;
use console_kernel_core::OrgId;
use console_platform_auth::RefreshTokenStore;
use console_platform_provisioning::BootstrapCredentialStore;
use console_platform_test_support::seed_org_and_super_admin;
use serde_json::{Value, json};
use sqlx::PgPool;
use std::collections::BTreeSet;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn source_rows(pool: &PgPool, ids: &[Uuid]) -> Result<Vec<(Uuid, String, Value)>> {
    // xmin is the existing old source row version, read in the same statement as
    // actual source bytes. The migration owner must bind the manifest's exact
    // PostgreSQL source-version codec; this is not a fabricated mapping receipt.
    Ok(
        sqlx::query_as(
            "SELECT id,xmin::text,to_jsonb(u) FROM users u WHERE id=ANY($1) ORDER BY id",
        )
        .bind(ids)
        .fetch_all(pool)
        .await?,
    )
}
async fn source_keys(pool: &PgPool, ids: &[Uuid]) -> Result<BTreeSet<(Uuid, String)>> {
    Ok(source_rows(pool, ids)
        .await?
        .into_iter()
        .map(|(id, version, _)| (id, version))
        .collect())
}
async fn mapping_partition(
    pool: &PgPool,
    manifest: &str,
) -> Result<(BTreeSet<(Uuid, String)>, BTreeSet<(Uuid, String)>)> {
    let mapped=sqlx::query_as("SELECT source_id,source_version FROM account_migration_mappings WHERE manifest_sha256=decode($1,'hex') ORDER BY source_id,source_version")
        .bind(manifest).fetch_all(pool).await?;
    let quarantined=sqlx::query_as("SELECT source_id,source_version FROM account_migration_quarantine WHERE manifest_sha256=decode($1,'hex') ORDER BY source_id,source_version")
        .bind(manifest).fetch_all(pool).await?;
    let mapped_count = mapped.len();
    let quarantine_count = quarantined.len();
    let mapped: BTreeSet<_> = mapped.into_iter().collect();
    let quarantined: BTreeSet<_> = quarantined.into_iter().collect();
    assert_eq!(
        mapped.len(),
        mapped_count,
        "duplicate mapping source/version"
    );
    assert_eq!(
        quarantined.len(),
        quarantine_count,
        "duplicate quarantine source/version"
    );
    assert!(mapped.is_disjoint(&quarantined));
    Ok((mapped, quarantined))
}
async fn snapshot_mapping(pool: &PgPool) -> Result<Value> {
    let receipts: Vec<Value> = sqlx::query_scalar(
        "SELECT to_jsonb(r) FROM account_migration_batch_receipts r ORDER BY batch_id",
    )
    .fetch_all(pool)
    .await?;
    let maps: Vec<Value> = sqlx::query_scalar(
        "SELECT to_jsonb(m) FROM account_migration_mappings m ORDER BY source_id,source_version",
    )
    .fetch_all(pool)
    .await?;
    let quarantine: Vec<Value> = sqlx::query_scalar(
        "SELECT to_jsonb(q) FROM account_migration_quarantine q ORDER BY source_id,source_version",
    )
    .fetch_all(pool)
    .await?;
    Ok(json!({"receipts":receipts,"maps":maps,"quarantine":quarantine}))
}

#[sqlx::test(migrations = false)]
async fn m1_real_cutover_preserves_active_revoked_and_unresolved_legacy_material(
    pool: PgPool,
) -> Result<()> {
    let mut f = seed_legacy(&pool).await;
    let org: Uuid = sqlx::query_scalar("SELECT org_id FROM users WHERE id=$1")
        .bind(f.subjects[0])
        .fetch_one(&pool)
        .await?;
    let unresolved = *seed_org_and_super_admin(&pool, org, "unresolved natural-person provenance")
        .await
        .as_uuid();
    f.subjects.push(unresolved);
    // Valid old user deliberately has no proven Human binding. A genuine old
    // employer OTP creates an independent null-actor audit, never Account grant.
    let otp = BootstrapCredentialStore
        .issue_for_zero_credential_user(
            &pool,
            unresolved,
            OrgId::from_uuid(org),
            OffsetDateTime::now_utc(),
            Duration::hours(4),
        )
        .await?;
    let active = RefreshTokenStore
        .issue_family(
            &pool,
            f.subjects[0],
            OrgId::from_uuid(org),
            OffsetDateTime::now_utc(),
            Duration::hours(2),
        )
        .await?;
    f.audit_ids=sqlx::query_scalar("SELECT id FROM audit_events WHERE actor=ANY($1) OR (actor IS NULL AND action='auth.bootstrap.issue') ORDER BY id")
        .bind(&f.subjects).fetch_all(&pool).await?;
    let null_actors: i64 =
        sqlx::query_scalar("SELECT count(*) FROM audit_events WHERE id=ANY($1) AND actor IS NULL")
            .bind(&f.audit_ids)
            .fetch_one(&pool)
            .await?;
    assert!(null_actors >= 1);
    f.snapshot = material_snapshot(&pool, &f.subjects, &f.audit_ids).await;
    // Current schema is installed dormant. This must NOT itself invent Human
    // proof or mark a user ACTIVE before a real drain and permanent fence.
    apply_current_and_require_account_catalog(&pool, &f).await;
    let manifest = migration::load_installed_manifest(&f.migrate_config).await?;
    let draining = migration::pause_legacy_admission(&f.migrate_config, &manifest).await?;
    let drained = migration::drain_legacy_work(&f.migrate_config, &draining).await?;
    assert!(drained.unresolved_effect_refs().is_empty());
    let fence = migration::commit_account_cutover(&f.migrate_config, &drained).await?;
    let actual: (Uuid, i64) = sqlx::query_as(
        "SELECT id,generation FROM legacy_authentication_fences WHERE id=$1 AND permanent",
    )
    .bind(fence.id())
    .fetch_one(&pool)
    .await?;
    assert_eq!(actual, (fence.id(), fence.generation()));
    assert_eq!(
        material_snapshot(&pool, &f.subjects, &f.audit_ids).await,
        f.snapshot
    );
    for token in [active.token.as_str(), f.revoked_token.as_str()] {
        assert!(
            RefreshTokenStore
                .rotate(
                    &pool,
                    token,
                    OffsetDateTime::now_utc(),
                    Duration::hours(2),
                    Duration::days(1)
                )
                .await
                .is_err(),
            "old refresh cannot survive real cutover"
        );
    }
    assert!(
        BootstrapCredentialStore
            .redeem_otp(&pool, otp.token.as_str(), OffsetDateTime::now_utc())
            .await
            .is_err()
    );
    let fabricated: i64 =
        sqlx::query_scalar("SELECT count(*) FROM account_human_bindings WHERE account_id=$1")
            .bind(unresolved)
            .fetch_one(&pool)
            .await?;
    assert_eq!(
        fabricated, 0,
        "unresolved provenance must not manufacture Human identity"
    );
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn m5_more_than_500_sources_form_exact_restartable_partition(pool: PgPool) -> Result<()> {
    let mut f = seed_legacy(&pool).await;
    let org: Uuid = sqlx::query_scalar("SELECT org_id FROM users WHERE id=$1")
        .bind(f.subjects[0])
        .fetch_one(&pool)
        .await?;
    // Explicit old-schema fixture input, identical helper used by retained225
    // tests. This writes no mapping, new Account, Human or verified authority.
    for n in 0..501 {
        f.subjects.push(
            *seed_org_and_super_admin(&pool, org, &format!("M5-{n}"))
                .await
                .as_uuid(),
        );
    }
    let expected = source_keys(&pool, &f.subjects).await?;
    assert_eq!(expected.len(), 503);
    apply_current_and_require_account_catalog(&pool, &f).await;
    let manifest = migration::load_installed_manifest(&f.migrate_config).await?;
    let source = migration::capture_source_snapshot(&f.migrate_config, &manifest).await?;
    // Instrument the actual production mapping transaction, not the returned
    // metadata. This disposable observation trigger only records its real
    // session setting/transaction; it cannot write Accounts or migration receipts.
    sqlx::raw_sql("CREATE TABLE public.test_only_migration_settings(txid bigint NOT NULL,setting text NOT NULL); CREATE FUNCTION public.test_only_observe_mapping_setting() RETURNS trigger LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $$ BEGIN INSERT INTO public.test_only_migration_settings VALUES(txid_current(),current_setting('statement_timeout')); RETURN NEW; END $$; REVOKE ALL ON FUNCTION public.test_only_observe_mapping_setting() FROM PUBLIC; CREATE TRIGGER test_only_mapping_setting AFTER INSERT ON public.account_migration_mappings FOR EACH ROW EXECUTE FUNCTION public.test_only_observe_mapping_setting();").execute(&pool).await?;
    let mut cursor = source.initial_cursor().clone();
    let mut seen = BTreeSet::new();
    loop {
        let batch = Uuid::new_v4();
        let request = migration::BatchRequest {
            manifest_digest: manifest.digest().to_owned(),
            source_cursor: cursor.clone(),
            source_version: source.version().to_owned(),
            batch_id: batch,
            limits: migration::BatchLimits {
                max_rows: 500,
                max_bytes: 4 * 1024 * 1024,
                statement_timeout_ms: 2000,
            },
        };
        let receipt = migration::migrate_batch(&f.migrate_config, &request).await?;
        assert!(receipt.source_keys.len() <= 500);
        assert!(receipt.source_bytes <= 4 * 1024 * 1024);
        assert_eq!(receipt.statement_timeout_ms, 2000);
        for key in &receipt.source_keys {
            assert!(seen.insert(key.clone()), "source consumed twice");
        }
        // Intentionally discard the first successful response at the caller's
        // boundary. The real committed batch receipt must make exact retry stable.
        let before = snapshot_mapping(&pool).await?;
        let retry = migration::migrate_batch(&f.migrate_config, &request).await?;
        assert_eq!(retry, receipt);
        assert_eq!(snapshot_mapping(&pool).await?, before);
        match receipt.next_cursor {
            Some(next) => {
                assert_ne!(next, cursor);
                cursor = next
            }
            None => break,
        }
        assert!(seen.len() <= expected.len());
    }
    let (mapped, quarantined) = mapping_partition(&pool, manifest.digest()).await?;
    assert_eq!(
        mapped.union(&quarantined).cloned().collect::<BTreeSet<_>>(),
        expected
    );
    assert_eq!(seen, expected);
    let settings: Vec<String> =
        sqlx::query_scalar("SELECT DISTINCT setting FROM public.test_only_migration_settings")
            .fetch_all(&pool)
            .await?;
    assert_eq!(
        settings,
        vec!["2s"],
        "actual mapping backend statement timeout"
    );
    let provenance:i64=sqlx::query_scalar("SELECT count(*) FROM account_migration_quarantine WHERE manifest_sha256=decode($1,'hex') AND source_id=ANY($2) AND reason_code='UNRESOLVED_IDENTITY_PROVENANCE' AND source_bytes IS NOT NULL AND octet_length(source_sha256)=32").bind(manifest.digest()).bind(&f.subjects).fetch_one(&pool).await?;
    assert!(
        provenance > 0,
        "retain exact unresolved source provenance, not count-only quarantine"
    );

    assert!(
        !quarantined.is_empty(),
        "valid old unresolved identity must be quarantined, not promoted by role"
    );
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn m5_byte_budget_is_independent_of_row_limit(pool: PgPool) -> Result<()> {
    let mut f = seed_legacy(&pool).await;
    let org: Uuid = sqlx::query_scalar("SELECT org_id FROM users WHERE id=$1")
        .bind(f.subjects[0])
        .fetch_one(&pool)
        .await?;
    // TEXT has no legacy name length CHECK. These are valid old rows, not a
    // constraint bypass or new production identity proof. 129*32768 exceeds4MiB
    // while the entire input has fewer than500 source rows.
    for n in 0..129 {
        let tag = format!("TEST_ONLY-{n}-{}", "x".repeat(32768));
        f.subjects
            .push(*seed_org_and_super_admin(&pool, org, &tag).await.as_uuid());
    }
    let raw = source_rows(&pool, &f.subjects).await?;
    let source_bytes: usize = raw
        .iter()
        .map(|(_, _, row)| serde_json::to_vec(row).unwrap().len())
        .sum();
    assert!(source_bytes > 4 * 1024 * 1024 && raw.len() < 500);
    let expected = source_keys(&pool, &f.subjects).await?;
    apply_current_and_require_account_catalog(&pool, &f).await;
    let manifest = migration::load_installed_manifest(&f.migrate_config).await?;
    let source = migration::capture_source_snapshot(&f.migrate_config, &manifest).await?;
    let receipt = migration::migrate_batch(
        &f.migrate_config,
        &migration::BatchRequest {
            manifest_digest: manifest.digest().into(),
            source_cursor: source.initial_cursor().clone(),
            source_version: source.version().into(),
            batch_id: Uuid::new_v4(),
            limits: migration::BatchLimits {
                max_rows: 500,
                max_bytes: 4 * 1024 * 1024,
                statement_timeout_ms: 2000,
            },
        },
    )
    .await?;
    assert!(receipt.source_keys.len() < expected.len());
    assert!(receipt.next_cursor.is_some());
    assert!(receipt.source_bytes <= 4 * 1024 * 1024);
    assert!(receipt.source_keys.len() < 500);
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn m5_source_version_change_cannot_silently_map_stale_input(pool: PgPool) -> Result<()> {
    let f = seed_legacy(&pool).await;
    apply_current_and_require_account_catalog(&pool, &f).await;
    let manifest = migration::load_installed_manifest(&f.migrate_config).await?;
    let source = migration::capture_source_snapshot(&f.migrate_config, &manifest).await?;
    let old = source_keys(&pool, &f.subjects).await?;
    // Real concurrent legacy writer transaction remains a valid old-schema
    // update. It is either fenced by the owner-installed guard or changes xmin.
    let mut writer = pool.begin().await?;
    let changed = sqlx::query(
        "UPDATE users SET display_name=display_name||' source-version-change' WHERE id=$1",
    )
    .bind(f.subjects[0])
    .execute(&mut *writer)
    .await;
    match changed {
        Ok(_) => writer.commit().await?,
        Err(e) => {
            writer.rollback().await?;
            assert_eq!(
                e.as_database_error().and_then(|e| e.code()).as_deref(),
                Some("55000")
            );
        }
    }
    let now = source_keys(&pool, &f.subjects).await?;
    let result = migration::migrate_batch(
        &f.migrate_config,
        &migration::BatchRequest {
            manifest_digest: manifest.digest().into(),
            source_cursor: source.initial_cursor().clone(),
            source_version: source.version().into(),
            batch_id: Uuid::new_v4(),
            limits: migration::BatchLimits {
                max_rows: 500,
                max_bytes: 4 * 1024 * 1024,
                statement_timeout_ms: 2000,
            },
        },
    )
    .await;
    if old != now {
        assert!(matches!(
            result,
            Err(migration::MigrationError::SourceChanged)
        ));
    } else {
        result?;
    }
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn m6_actual_startup_requires_exact_manifest_and_independent_current_generation(
    pool: PgPool,
) -> Result<()> {
    let f = seed_legacy(&pool).await;
    apply_current_and_require_account_catalog(&pool, &f).await;
    let installed = admission::load_installed_release(&f.migrate_config).await?;
    let healthy = admission::admit_process(&f.migrate_config, &installed).await?;
    assert!(healthy.command_service().is_some());
    // Input is an untrusted release descriptor read from disk. Copying it grants
    // no service. The actual startup owner re-reads DB/catalog/current timeline.
    for field in [
        "binary_sha256",
        "catalog_sha256",
        "codec_sha256",
        "projection_sha256",
        "policy_sha256",
        "alias_manifest_sha256",
    ] {
        let mut bytes = serde_json::to_value(installed.descriptor())?;
        bytes[field] = json!("ab".repeat(32));
        let bad = admission::ReleaseDescriptor::parse(&serde_json::to_vec(&bytes)?)?;
        let result = admission::admit_process(&f.migrate_config, &bad).await;
        assert!(matches!(
            result,
            Err(admission::AdmissionError::ReleaseMismatch)
        ));
    }
    let before = healthy.serving_binding().clone();
    let drain = admission::pause_and_drain_effects(&f.migrate_config).await?;
    let recovery = admission::advance_recovery_generation(&f.migrate_config, &drain).await?;
    assert!(recovery.generation() > before.recovery_generation());
    assert!(
        admission::revalidate_serving_binding(&f.migrate_config, &before)
            .await
            .is_err()
    );
    let fresh = admission::admit_process(&f.migrate_config, &installed).await?;
    assert_eq!(
        fresh.serving_binding().recovery_generation(),
        recovery.generation()
    );
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn m5_worker_backend_interruption_before_commit_has_no_partial_mapping(
    pool: PgPool,
) -> Result<()> {
    let f = seed_legacy(&pool).await;
    apply_current_and_require_account_catalog(&pool, &f).await;
    let manifest = migration::load_installed_manifest(&f.migrate_config).await?;
    let source = migration::capture_source_snapshot(&f.migrate_config, &manifest).await?;
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
    let before = snapshot_mapping(&pool).await?;
    let mut barrier = pool.begin().await?;
    let blocker: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *barrier)
        .await?;
    sqlx::query("LOCK TABLE account_migration_mappings IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *barrier)
        .await?;
    let operation = migration::migrate_batch(&f.migrate_config, &request);
    tokio::pin!(operation);
    let witness = async {
        for _ in 0..100 {
            let pids:Vec<i32>=sqlx::query_scalar("SELECT pid FROM pg_stat_activity WHERE datname=current_database() AND $1=ANY(pg_blocking_pids(pid))").bind(blocker).fetch_all(&pool).await?;
            if pids.len() == 1 {
                return Ok::<i32, Box<dyn std::error::Error + Send + Sync>>(pids[0]);
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        Err("no actual migration writer lock witness".into())
    };
    let pid = tokio::select! {r=&mut operation=>{r?;panic!("mapping bypassed held relation")},p=witness=>p?};
    // Terminate only the witnessed backend inside this isolated marked database.
    let killed:bool=sqlx::query_scalar("SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE pid=$1 AND datname=current_database() AND $2=ANY(pg_blocking_pids(pid))").bind(pid).bind(blocker).fetch_one(&pool).await?;
    assert!(killed);
    assert!(operation.await.is_err());
    barrier.rollback().await?;
    assert_eq!(snapshot_mapping(&pool).await?, before);
    let retry = migration::migrate_batch(&f.migrate_config, &request).await?;
    let again = migration::migrate_batch(&f.migrate_config, &request).await?;
    assert_eq!(retry, again);
    Ok(())
}
