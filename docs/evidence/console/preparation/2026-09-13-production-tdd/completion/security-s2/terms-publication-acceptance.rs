//! Append as auth_rest::terms_publication_acceptance; setup reuses the existing
//! real migration/database-login fixture. Missing publisher is a compile
//! prerequisite, never a passing or behavioral-red test.
use super::prepare_http_database;
use crate::operator_custody_producer::{ApprovedPublication, CustodyFixture, Result};
use console_platform_auth::terms_publication as publisher;
use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

async fn snapshot(pool: &PgPool) -> Result<Value> {
    let head: Vec<Value> =
        sqlx::query_scalar("SELECT to_jsonb(h) FROM public.account_terms_head h ORDER BY id")
            .fetch_all(pool)
            .await?;
    let receipts: Vec<Value> = sqlx::query_scalar(
        "SELECT to_jsonb(r) FROM public.account_terms_release_receipts r ORDER BY revision",
    )
    .fetch_all(pool)
    .await?;
    Ok(json!({"head":head,"receipts":receipts}))
}
async fn publish(
    pool: &PgPool,
    f: &CustodyFixture,
    a: &ApprovedPublication,
) -> Result<publisher::PublicationReceipt> {
    let operator = f.authenticate(publisher::Environment::TestOnly).await?;
    // Production owner obtains a dedicated publisher LOGIN internally from
    // deployment config; an admin PgPool is never publication authority.
    let result = publisher::publish_terms(
        &operator,
        publisher::OperatorApprovalRef {
            kind: publisher::ApprovalKind::OperatorReleaseApproval,
            approval_sha256: a.approval_digest.clone(),
        },
    )
    .await?;
    let actual:(Uuid,i64,String,Vec<u8>)=sqlx::query_as("SELECT r.id,r.revision,encode(r.manifest_sha256,'hex'),r.approval_bytes FROM public.account_terms_head h JOIN public.account_terms_release_receipts r ON (r.id,r.revision,r.manifest_sha256)=(h.release_receipt_ref,h.revision,h.manifest_sha256) WHERE h.id=1")
        .fetch_one(pool).await?;
    assert_eq!(
        actual,
        (
            result.id,
            result.revision,
            a.manifest_digest.clone(),
            a.approval_bytes.clone()
        )
    );
    Ok(result)
}
async fn prepare(pool: &PgPool) -> Result<CustodyFixture> {
    prepare_http_database(pool).await;
    let f = CustodyFixture::new(
        "publication",
        pool.connect_options().get_database().ok_or("database")?,
    )?;
    // Owner configuration binds this marked SQLx database through the existing
    // privileged deployment fixture. No role grants or fake head are installed.

    Ok(f)
}

#[sqlx::test(migrations = false)]
async fn pub_authenticated_custody_genesis_and_same_digest_republication(
    pool: PgPool,
) -> Result<()> {
    let f = prepare(&pool).await?;
    assert_eq!(snapshot(&pool).await?["head"], json!([]));
    let mut last = None;
    for predecessor in 0..8 {
        let approval = f.approve(predecessor)?;
        let receipt = publish(&pool, &f, &approval).await?;
        assert_eq!(receipt.revision, predecessor + 1);
        assert_eq!(
            receipt.previous_revision,
            if predecessor == 0 {
                None
            } else {
                Some(predecessor)
            }
        );
        if let Some(id) = last {
            assert_ne!(receipt.id, id);
        }
        last = Some(receipt.id);
    }
    let state = snapshot(&pool).await?;
    assert_eq!(state["receipts"].as_array().unwrap().len(), 8);
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn pub_expected_seven_concurrent_publishers_append_exactly_one_receipt(
    pool: PgPool,
) -> Result<()> {
    let f = prepare(&pool).await?;
    for n in 0..7 {
        publish(&pool, &f, &f.approve(n)?).await?;
    }
    let left = f.approve(7)?;
    let right = f.approve(7)?;
    let a = f.authenticate(publisher::Environment::TestOnly).await?;
    let b = f.authenticate(publisher::Environment::TestOnly).await?;
    let reference = |a: &ApprovedPublication| publisher::OperatorApprovalRef {
        kind: publisher::ApprovalKind::OperatorReleaseApproval,
        approval_sha256: a.approval_digest.clone(),
    };
    let (x, y) = tokio::join!(
        publisher::publish_terms(&a, reference(&left)),
        publisher::publish_terms(&b, reference(&right))
    );
    let (winner, loser, approval) = match (x, y) {
        (Ok(x), Err(y)) => (x, y, &left),
        (Err(x), Ok(y)) => (y, x, &right),
        _ => panic!("exactly one authorized concurrent expected-seven publisher must succeed"),
    };
    assert!(matches!(
        loser,
        publisher::PublicationError::RevisionConflict {
            current_revision: 8
        }
    ));
    assert_eq!(winner.revision, 8);
    let state = snapshot(&pool).await?;
    assert_eq!(state["receipts"].as_array().unwrap().len(), 8);
    let bytes: Vec<u8> =
        sqlx::query_scalar("SELECT approval_bytes FROM account_terms_release_receipts WHERE id=$1")
            .bind(winner.id)
            .fetch_one(&pool)
            .await?;
    assert_eq!(bytes, approval.approval_bytes);
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn pub_untrusted_copy_and_wrong_signature_have_no_custody_authority(
    pool: PgPool,
) -> Result<()> {
    let f = prepare(&pool).await?;
    let approval = f.approve(0)?;
    let before = snapshot(&pool).await?;
    let other = CustodyFixture::new(
        "same application JSON cannot authorize",
        pool.connect_options().get_database().ok_or("database")?,
    )?;
    let copied = other
        .directory
        .path()
        .join("approvals")
        .join(&approval.approval_digest);
    std::fs::copy(&approval.custody_path, &copied)?;
    std::fs::copy(
        approval.custody_path.with_extension("sig"),
        copied.with_extension("sig"),
    )?;

    let operator = other.authenticate(publisher::Environment::TestOnly).await?;
    let result = publisher::publish_terms(
        &operator,
        publisher::OperatorApprovalRef {
            kind: publisher::ApprovalKind::OperatorReleaseApproval,
            approval_sha256: approval.approval_digest.clone(),
        },
    )
    .await;
    assert!(matches!(
        result,
        Err(publisher::PublicationError::UntrustedCustody)
    ));
    assert_eq!(snapshot(&pool).await?, before);
    // Exact same bytes are a real positive under the correct authenticated root.
    publish(&pool, &f, &approval).await?;
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn pub_production_rejects_test_roots_and_missing_qualified_content(
    pool: PgPool,
) -> Result<()> {
    let f = prepare(&pool).await?;
    let approval = f.approve(0)?;
    let before = snapshot(&pool).await?;
    assert!(
        f.authenticate(publisher::Environment::Production)
            .await
            .is_err()
    );
    let value: Value = serde_json::from_slice(&approval.approval_bytes)?;
    let authority = value["content_authority_refs"][0].as_str().unwrap();
    std::fs::remove_file(f.directory.path().join("authority").join(authority))?;
    let operator = f.authenticate(publisher::Environment::TestOnly).await?;
    let actual = publisher::publish_terms(
        &operator,
        publisher::OperatorApprovalRef {
            kind: publisher::ApprovalKind::OperatorReleaseApproval,
            approval_sha256: approval.approval_digest,
        },
    )
    .await;
    assert!(matches!(
        actual,
        Err(publisher::PublicationError::UntrustedCustody)
    ));
    assert_eq!(snapshot(&pool).await?, before);
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn pub_closed_manifest_validation_and_content_integrity(pool: PgPool) -> Result<()> {
    let f = prepare(&pool).await?;
    let before = snapshot(&pool).await?;
    let base = f.manifest_value()?;
    let mut variants = Vec::new();
    let mut v = base.clone();
    v["items"] = json!([]);
    variants.push(v);
    let mut v = base.clone();
    v["items"] = json!(vec![base["items"][0].clone(); 9]);
    variants.push(v);
    let mut v = base.clone();
    v["items"] = json!([base["items"][0], base["items"][0]]);
    variants.push(v);
    let mut v = base.clone();
    v["items"][0]["terms_kind"] = json!("unregistered.kind");
    variants.push(v);
    let mut v = base.clone();
    v["items"][0]["required"] = json!(false);
    variants.push(v);
    let mut v = base.clone();
    v["items"][0]["locale"] = json!("x");
    variants.push(v);
    let mut v = base.clone();
    v["items"][0]["title"] = json!("x".repeat(129));
    variants.push(v);
    let mut v = base.clone();
    v["items"][0]["private_extra"] = json!(true);
    variants.push(v);
    for value in variants {
        let approval = f.approve_manifest(0, &serde_json::to_vec(&value)?)?;
        let operator = f.authenticate(publisher::Environment::TestOnly).await?;
        let actual = publisher::publish_terms(
            &operator,
            publisher::OperatorApprovalRef {
                kind: publisher::ApprovalKind::OperatorReleaseApproval,
                approval_sha256: approval.approval_digest,
            },
        )
        .await;
        assert!(matches!(
            actual,
            Err(publisher::PublicationError::InvalidArtifact)
        ));
        assert_eq!(snapshot(&pool).await?, before);
    }
    let valid = f.approve(0)?;
    let content_path = f.directory.path().join("objects").join(&f.content_digest);
    f.corrupt_object(&content_path, b"different TEST_ONLY bytes")?;
    assert!(publish(&pool, &f, &valid).await.is_err());
    assert_eq!(snapshot(&pool).await?, before);
    f.corrupt_object(&content_path, &f.content)?;
    publish(&pool, &f, &valid).await?;
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn pub_real_database_failure_rolls_back_receipt_and_head(pool: PgPool) -> Result<()> {
    let f = prepare(&pool).await?;
    publish(&pool, &f, &f.approve(0)?).await?;
    let before = snapshot(&pool).await?;
    // Disposable fault only: a deferred constraint trigger fails at COMMIT, after
    // the real authenticated owner has attempted both durable mutations.
    sqlx::raw_sql("CREATE FUNCTION public.test_only_reject_terms_commit() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'TEST_ONLY commit failure' USING ERRCODE='23514'; END $$; CREATE CONSTRAINT TRIGGER test_only_terms_commit AFTER INSERT ON public.account_terms_release_receipts DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.test_only_reject_terms_commit();").execute(&pool).await?;
    let result = publish(&pool, &f, &f.approve(1)?).await;
    sqlx::raw_sql("DROP TRIGGER test_only_terms_commit ON public.account_terms_release_receipts; DROP FUNCTION public.test_only_reject_terms_commit();").execute(&pool).await?;
    assert!(result.is_err());
    assert_eq!(snapshot(&pool).await?, before);
    publish(&pool, &f, &f.approve(1)?).await?;
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn pub_security_definer_entrypoint_denies_real_runtime_logins(pool: PgPool) -> Result<()> {
    let f = prepare(&pool).await?;
    let approval = f.approve(0)?;
    let before = snapshot(&pool).await?;
    // Exact narrow publisher SQL function is a required production registration,
    // not guessed by scanning ACLs. Exercise it with a valid approval reference.
    for login in [
        TestDatabaseLogin::Auth,
        TestDatabaseLogin::Business,
        TestDatabaseLogin::OntologyCommand,
        TestDatabaseLogin::LeaveCommand,
        TestDatabaseLogin::PlatformForceCommand,
    ] {
        let runtime = login_test_pool(&pool, login).await;
        let mut tx = runtime.begin().await?;
        let actual = sqlx::query("SELECT public.account_publish_terms($1::jsonb)")
            .bind(json!({"kind":"OPERATOR_RELEASE_APPROVAL","approval_sha256":approval.approval_digest}))
            .execute(&mut *tx)
            .await;
        tx.rollback().await?;
        let error =
            actual.expect_err("runtime must not invoke publisher SECURITY DEFINER function");
        assert_eq!(
            error.as_database_error().and_then(|e| e.code()).as_deref(),
            Some("42501")
        );
        runtime.close().await;
    }
    assert_eq!(snapshot(&pool).await?, before);
    publish(&pool, &f, &approval).await?;
    Ok(())
}

async fn blocked_pid(pool: &PgPool, blocker: i32) -> Result<i32> {
    for _ in 0..200 {
        let pids:Vec<i32>=sqlx::query_scalar("SELECT pid FROM pg_stat_activity WHERE datname=current_database() AND $1=ANY(pg_blocking_pids(pid)) ORDER BY pid")
            .bind(blocker).fetch_all(pool).await?;
        if pids.len() == 1 {
            return Ok(pids[0]);
        }
        assert!(
            pids.len() <= 1,
            "unexpected concurrent blockers in isolated test"
        );
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    Err("no witnessed PostgreSQL blocker; elapsed time is not lock evidence".into())
}

#[sqlx::test(migrations = false)]
async fn pub_registration_holds_share_through_commit_and_publisher_only_waits_on_head(
    pool: PgPool,
) -> Result<()> {
    use console_platform_auth::{PasskeyService, WebauthnSettings, account_enrollment as accounts};
    use sha2::{Digest, Sha256};
    use webauthn_authenticator_rs::{prelude::WebauthnAuthenticator, softpasskey::SoftPasskey};
    let f = prepare(&pool).await?;
    let old = publish(&pool, &f, &f.approve(0)?).await?;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let origin = url::Url::parse("https://auth.example.com")?;
    let service = PasskeyService::new(WebauthnSettings {
        rp_id: "example.com".into(),
        rp_origin: origin.clone(),
        rp_name: "TEST_ONLY".into(),
        extra_allowed_origins: vec![],
        ceremony_ttl: time::Duration::minutes(5),
    })?;
    let terms = accounts::read_current_account_terms(&auth).await?;
    let nonce = rand::random::<[u8; 32]>();
    let start = accounts::start_account_enrollment(
        &auth,
        &service,
        accounts::EnrollmentStart {
            display_name: "TEST_ONLY publication race".into(),
            terms: terms.reference.clone(),
            browser_nonce_digest: hex::encode(Sha256::digest(nonce)),
        },
    )
    .await?;
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let credential = authenticator.do_registration(origin, start.challenge)?;
    // Block the last ordered ceremony lock. Real finish must first acquire the
    // Account guard and head SHARE, then wait on this exact row.
    let mut barrier = pool.begin().await?;
    let barrier_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *barrier)
        .await?;
    sqlx::query("SELECT id FROM auth_webauthn_ceremonies WHERE id=$1 FOR UPDATE")
        .bind(start.ceremony_id)
        .fetch_one(&mut *barrier)
        .await?;
    let finish = accounts::finish_account_enrollment(
        &auth,
        &service,
        accounts::EnrollmentFinish {
            enrollment_id: start.enrollment_id,
            ceremony_id: start.ceremony_id,
            browser_nonce: hex::encode(nonce),
            terms: terms.reference,
            credential,
        },
    );
    tokio::pin!(finish);
    let finish_pid = tokio::select! {r=&mut finish=>{r?;panic!("finish bypassed held ceremony row")},p=blocked_pid(&pool,barrier_pid)=>p?};
    let next = f.approve(1)?;
    let operator = f.authenticate(publisher::Environment::TestOnly).await?;
    let publishing = publisher::publish_terms(
        &operator,
        publisher::OperatorApprovalRef {
            kind: publisher::ApprovalKind::OperatorReleaseApproval,
            approval_sha256: next.approval_digest.clone(),
        },
    );
    tokio::pin!(publishing);
    let publisher_pid = tokio::select! {
    r=&mut publishing=>{r?;panic!("publisher bypassed finish head SHARE; FOR KEY SHARE mutation")},
    r=&mut finish=>{r?;panic!("finish escaped barrier")},
    p=blocked_pid(&pool,finish_pid)=>p?};
    let account_lock_count:i64=sqlx::query_scalar("SELECT count(*) FROM pg_locks WHERE pid=$1 AND relation='public.account_security'::regclass")
        .bind(publisher_pid).fetch_one(&pool).await?;
    assert_eq!(account_lock_count, 0, "publisher must not lock Accounts");
    let waiting: i64 =
        sqlx::query_scalar("SELECT count(*) FROM pg_locks WHERE pid=$1 AND NOT granted")
            .bind(publisher_pid)
            .fetch_one(&pool)
            .await?;
    assert!(waiting > 0);
    barrier.commit().await?;
    let (finished, published) = tokio::join!(finish, publishing);
    let finished = finished?;
    let published = published?;
    assert_eq!(published.revision, old.revision + 1);
    let accepted:Vec<(Uuid,i64,Uuid)>=sqlx::query_as("SELECT terms_release_receipt_id,terms_release_revision,security_event_id FROM account_terms_acceptances WHERE account_id=$1 ORDER BY terms_kind")
        .bind(finished.account_id).fetch_all(&pool).await?;
    assert!(!accepted.is_empty());
    for (receipt, revision, event) in accepted {
        assert_eq!((receipt, revision), (old.id, old.revision));
        let actual: (Uuid, String) =
            sqlx::query_as("SELECT account_id,kind FROM account_security_events WHERE id=$1")
                .bind(event)
                .fetch_one(&pool)
                .await?;
        assert_eq!(actual, (finished.account_id, "TERMS_ACCEPTED".into()));
    }
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn pub_symlink_escape_and_oversized_or_non_utf8_content_refuse_before_effect(
    pool: PgPool,
) -> Result<()> {
    use crate::operator_custody_producer::digest;
    let f = prepare(&pool).await?;
    let before = snapshot(&pool).await?;
    for bytes in [vec![b'x'; 65537], vec![0xff, 0xfe]] {
        let hash = digest(&bytes);
        let object = f.directory.path().join("objects").join(&hash);
        std::fs::write(&object, &bytes)?;
        let mut manifest = f.manifest_value()?;
        manifest["items"][0]["content_sha256"] = json!(hash);
        let approval = f.approve_manifest(0, &serde_json::to_vec(&manifest)?)?;
        assert!(publish(&pool, &f, &approval).await.is_err());
        assert_eq!(snapshot(&pool).await?, before);
    }
    let approval = f.approve(0)?;
    let object = f.directory.path().join("objects").join(&f.content_digest);
    let outside = tempfile::TempDir::new()?;
    let target = outside.path().join("content");
    std::fs::write(&target, &f.content)?;
    std::fs::remove_file(&object)?;
    std::os::unix::fs::symlink(&target, &object)?;
    assert!(publish(&pool, &f, &approval).await.is_err());
    assert_eq!(snapshot(&pool).await?, before);
    std::fs::remove_file(&object)?;
    std::fs::write(&object, &f.content)?;
    publish(&pool, &f, &approval).await?;
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn pub_actual_a_b_a_publications_do_not_revive_old_registration(pool: PgPool) -> Result<()> {
    use console_platform_auth::{PasskeyService, WebauthnSettings, account_enrollment as accounts};
    use sha2::{Digest, Sha256};
    use webauthn_authenticator_rs::{prelude::WebauthnAuthenticator, softpasskey::SoftPasskey};
    let f = prepare(&pool).await?;
    let first = publish(&pool, &f, &f.approve(0)?).await?;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let origin = url::Url::parse("https://auth.example.com")?;
    let service = PasskeyService::new(WebauthnSettings {
        rp_id: "example.com".into(),
        rp_origin: origin.clone(),
        rp_name: "TEST_ONLY".into(),
        extra_allowed_origins: vec![],
        ceremony_ttl: time::Duration::minutes(5),
    })?;
    let terms = accounts::read_current_account_terms(&auth).await?;
    let nonce = rand::random::<[u8; 32]>();
    let start = accounts::start_account_enrollment(
        &auth,
        &service,
        accounts::EnrollmentStart {
            display_name: "TEST_ONLY ABA".into(),
            terms: terms.reference.clone(),
            browser_nonce_digest: hex::encode(Sha256::digest(nonce)),
        },
    )
    .await?;
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let credential = authenticator.do_registration(origin, start.challenge)?;
    let mut b = f.manifest_value()?;
    b["items"][0]["title"] = json!("TEST_ONLY manifest B");
    publish(&pool, &f, &f.approve_manifest(1, &serde_json::to_vec(&b)?)?).await?;
    let last = publish(&pool, &f, &f.approve(2)?).await?;
    assert_eq!(first.manifest_sha256, last.manifest_sha256);
    assert_ne!(first.id, last.id);
    assert_eq!(last.revision, 3);
    let result = accounts::finish_account_enrollment(
        &auth,
        &service,
        accounts::EnrollmentFinish {
            enrollment_id: start.enrollment_id,
            ceremony_id: start.ceremony_id,
            browser_nonce: hex::encode(nonce),
            terms: terms.reference,
            credential,
        },
    )
    .await;
    assert!(matches!(
        result,
        Err(accounts::EnrollmentError::TermsChanged)
    ));
    let partial:(i64,i64)=sqlx::query_as("SELECT (SELECT count(*) FROM account_terms_acceptances WHERE account_id=$1),(SELECT count(*) FROM auth_refresh_token_families WHERE user_id=$1)").bind(start.account_id).fetch_one(&pool).await?;
    assert_eq!(partial, (0, 0));
    Ok(())
}
