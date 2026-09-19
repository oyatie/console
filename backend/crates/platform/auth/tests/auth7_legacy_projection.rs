//! Auth7 SQL projection/audit capability tests against the actual operator install.
//! This does not certify WebAuthn proof, route authorization, or owner use-case
//! association: those remain the existing real-owner/HTTP acceptance tests.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use console_kernel_core::{AuditAction, AuditEvent, BranchId, OrgId, TraceContext, UserId};
use console_platform_test_support::{
    TestDatabaseLogin, login_test_pool, prepare_account_test_database,
};
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Row, Transaction};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

const AUDIT_SIGNATURE: &str = "public.auth_legacy_audit_append_v1(uuid,uuid,text,text,text,uuid,jsonb,jsonb,character,character,timestamp with time zone,uuid,text,text,text,text,text[],boolean,text)";

async fn setup(pool: &PgPool) -> (PgPool, PgPool, Uuid) {
    prepare_account_test_database(pool).await;
    for signature in [
        "public.auth_legacy_self_passkeys_v1(uuid,uuid)",
        "public.auth_legacy_self_passkey_state_v1(uuid,uuid,uuid)",
        "public.auth_legacy_self_passkey_count_v1(uuid,uuid)",
        "public.auth_legacy_user_has_passkey_v1(uuid,uuid)",
        AUDIT_SIGNATURE,
    ] {
        let exists: bool = sqlx::query_scalar("SELECT pg_catalog.to_regprocedure($1) IS NOT NULL")
            .bind(signature)
            .fetch_one(pool)
            .await
            .unwrap();
        assert!(
            exists,
            "AUTH7_PREREQUISITE: actual production finalizer has not installed {signature}; acceptance assertions not reached"
        );
    }
    let subject = seed_user(pool).await;
    (
        login_test_pool(pool, TestDatabaseLogin::Business).await,
        login_test_pool(pool, TestDatabaseLogin::Auth).await,
        subject,
    )
}

async fn seed_user(pool: &PgPool) -> Uuid {
    let subject = sqlx::query_scalar("INSERT INTO public.users (display_name,roles,org_id) VALUES ('Auth7 projection fixture',ARRAY['MEMBER']::text[],$1) RETURNING id")
        .bind(OrgId::knl().as_uuid()).fetch_one(pool).await.unwrap();
    let bridged: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.accounts a JOIN public.users u ON (a.id,a.created_at)=(u.id,u.created_at) WHERE u.id=$1)")
        .bind(subject).fetch_one(pool).await.unwrap();
    assert!(
        bridged,
        "AUTH7_PREREQUISITE: production legacy root bridge must create the real identity tuple; fixture never repairs it"
    );
    subject
}

async fn company_tx(pool: &PgPool) -> Transaction<'static, Postgres> {
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org',$1,true)")
        .bind(OrgId::knl().to_string())
        .execute(tx.as_mut())
        .await
        .unwrap();
    sqlx::raw_sql("SET LOCAL statement_timeout='5s'; SET LOCAL lock_timeout='1s';")
        .execute(tx.as_mut())
        .await
        .unwrap();
    tx
}

async fn insert_key(
    tx: &mut Transaction<'_, Postgres>,
    subject: Uuid,
    id: Uuid,
    used: Option<OffsetDateTime>,
) {
    sqlx::query("INSERT INTO public.auth_webauthn_credentials (id,user_id,credential_id,passkey_json,created_at,last_used_at,org_id) VALUES ($1,$2,$3,$4,$5,$6,$7)")
        .bind(id).bind(subject).bind(id.to_string())
        .bind(json!({"fixture_only":"private credential state must not project"}))
        .bind(instant()).bind(used).bind(OrgId::knl().as_uuid())
        .execute(tx.as_mut()).await.unwrap();
}

async fn fence(tx: &mut Transaction<'_, Postgres>, subject: Uuid) {
    // Fixture state in this exclusively owned marked SQLx database, not a new
    // native enrollment implementation or a substitute authority function.
    sqlx::query("SELECT id FROM public.organizations WHERE id=$1 FOR KEY SHARE")
        .bind(OrgId::knl().as_uuid())
        .fetch_one(tx.as_mut())
        .await
        .unwrap();
    sqlx::query("SELECT id FROM public.users WHERE id=$1 FOR UPDATE")
        .bind(subject)
        .fetch_one(tx.as_mut())
        .await
        .unwrap();
    sqlx::query("SELECT id FROM public.accounts WHERE id=$1 FOR UPDATE")
        .bind(subject)
        .fetch_one(tx.as_mut())
        .await
        .unwrap();
    sqlx::query("INSERT INTO public.account_security (account_id,security_state,security_generation,revision,updated_at,context_generation) VALUES ($1,'ACTIVE',1,1,$2,1)")
        .bind(subject).bind(instant()).execute(tx.as_mut()).await.unwrap();
}

fn instant() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_800_000_000).unwrap()
}

async fn state(pool: &PgPool) -> Value {
    sqlx::query_scalar(r#"SELECT jsonb_build_object(
      'users',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) FROM public.users x),
      'accounts',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) FROM public.accounts x),
      'security',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.account_id),'[]'::jsonb) FROM public.account_security x),
      'keys',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) FROM public.auth_webauthn_credentials x),
      'ceremonies',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) FROM public.auth_webauthn_ceremonies x),
      'bindings',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.ceremony_id),'[]'::jsonb) FROM public.auth_webauthn_ceremony_bindings x),
      'families',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) FROM public.auth_refresh_token_families x),
      'tokens',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) FROM public.auth_refresh_tokens x),
      'bootstrap',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) FROM public.auth_bootstrap_credentials x),
      'handoffs',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) FROM public.auth_device_login_handoffs x),
      'audits',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) FROM public.audit_events x))"#)
        .fetch_one(pool).await.unwrap()
}

async fn count(tx: &mut Transaction<'_, Postgres>, subject: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT public.auth_legacy_self_passkey_count_v1($1,$2)")
        .bind(OrgId::knl().as_uuid())
        .bind(subject)
        .fetch_one(tx.as_mut())
        .await
}

async fn read_all(
    pool: &PgPool,
    subject: Uuid,
    key: Uuid,
) -> Vec<Result<Vec<sqlx::postgres::PgRow>, sqlx::Error>> {
    let mut results = Vec::new();
    for query in [
        "SELECT * FROM public.auth_legacy_self_passkeys_v1($1,$2) WHERE $3::uuid IS NOT NULL",
        "SELECT * FROM public.auth_legacy_self_passkey_state_v1($1,$2,$3)",
        "SELECT public.auth_legacy_self_passkey_count_v1($1,$2) WHERE $3::uuid IS NOT NULL",
        "SELECT public.auth_legacy_user_has_passkey_v1($1,$2) WHERE $3::uuid IS NOT NULL",
    ] {
        let mut tx = company_tx(pool).await;
        results.push(
            sqlx::query(sqlx::AssertSqlSafe(query))
                .bind(OrgId::knl().as_uuid())
                .bind(subject)
                .bind(key)
                .fetch_all(tx.as_mut())
                .await,
        );
        tx.rollback().await.unwrap();
    }
    results
}

fn assert_denied(error: &sqlx::Error) {
    let db = error
        .as_database_error()
        .expect("actual PostgreSQL owner refusal");
    let code = db.code();
    assert!(
        matches!(
            (code.as_deref(), db.message()),
            (
                Some("P0001"),
                "auth_legacy.fenced" | "auth_legacy.invalid_fence"
            ) | (Some("P0002"), "auth_legacy.subject_not_found")
                | (Some("42501"), "auth_legacy.company_context_mismatch")
                | (Some("22004"), "auth_legacy.null_identity")
                | (Some("22023"), "auth_legacy_audit.invalid_event")
        ),
        "exact admitted owner code/tag required; missing schema, decoder, FK, and connection failures cannot satisfy refusal"
    );
}

#[sqlx::test(migrations = false)]
async fn projections_preserve_empty_present_unused_and_nonself_without_private_state(pool: PgPool) {
    let (business, auth, subject) = setup(&pool).await;
    let key = Uuid::new_v4();
    let empty = read_all(&business, subject, key).await;
    assert!(empty[0].as_ref().unwrap().is_empty());
    assert!(empty[1].as_ref().unwrap().is_empty());
    assert_eq!(empty[2].as_ref().unwrap()[0].get::<i64, _>(0), 0);
    assert!(!empty[3].as_ref().unwrap()[0].get::<bool, _>(0));
    let mut seed = pool.begin().await.unwrap();
    insert_key(&mut seed, subject, key, None).await;
    seed.commit().await.unwrap();
    let before = state(&pool).await;
    let present = read_all(&business, subject, key).await;
    let row = &present[0].as_ref().unwrap()[0];
    assert_eq!(row.len(), 3, "only id/created_at/last_used_at may project");
    assert_eq!(row.get::<Uuid, _>("id"), key);
    assert_eq!(row.get::<OffsetDateTime, _>("created_at"), instant());
    assert!(
        row.get::<Option<OffsetDateTime>, _>("last_used_at")
            .is_none()
    );
    let own_state = &present[1].as_ref().unwrap()[0];
    assert_eq!(own_state.len(), 1);
    assert!(
        own_state
            .get::<Option<OffsetDateTime>, _>("last_used_at")
            .is_none()
    );
    assert_eq!(present[2].as_ref().unwrap()[0].get::<i64, _>(0), 1);
    assert!(present[3].as_ref().unwrap()[0].get::<bool, _>(0));
    assert!(
        before == state(&pool).await,
        "pure projections must preserve all state/audit rows"
    );
    let other = seed_user(&pool).await;
    assert!(
        read_all(&business, other, key).await[1]
            .as_ref()
            .unwrap()
            .is_empty()
    );
    for result in read_all(&business, Uuid::new_v4(), key).await {
        assert_denied(&result.unwrap_err());
    }
    let mut wrong_company = company_tx(&business).await;
    sqlx::query("SELECT set_config('app.current_org',$1,true)")
        .bind(Uuid::new_v4().to_string())
        .execute(wrong_company.as_mut())
        .await
        .unwrap();
    let refused = count(&mut wrong_company, subject).await;
    wrong_company.rollback().await.unwrap();
    assert_denied(&refused.unwrap_err());
    let mut tx = company_tx(&auth).await;
    assert_eq!(count(&mut tx, subject).await.unwrap(), 1);
    let ids: Vec<Uuid> =
        sqlx::query_scalar("SELECT id FROM public.auth_legacy_self_passkeys_v1($1,$2)")
            .bind(OrgId::knl().as_uuid())
            .bind(subject)
            .fetch_all(tx.as_mut())
            .await
            .unwrap();
    assert_eq!(ids, vec![key]);
    tx.rollback().await.unwrap();
}

#[sqlx::test(migrations = false)]
async fn projections_take_no_company_user_or_account_row_locks(pool: PgPool) {
    let (business, _, subject) = setup(&pool).await;
    let key = Uuid::new_v4();
    let mut seed = pool.begin().await.unwrap();
    insert_key(&mut seed, subject, key, Some(instant())).await;
    seed.commit().await.unwrap();
    for signature in [
        "public.auth_legacy_self_passkeys_v1(uuid,uuid)",
        "public.auth_legacy_self_passkey_state_v1(uuid,uuid,uuid)",
        "public.auth_legacy_self_passkey_count_v1(uuid,uuid)",
        "public.auth_legacy_user_has_passkey_v1(uuid,uuid)",
    ] {
        let stable: bool = sqlx::query_scalar("SELECT provolatile='s' FROM pg_catalog.pg_proc WHERE oid=pg_catalog.to_regprocedure($1)")
            .bind(signature).fetch_one(&pool).await.unwrap();
        assert!(
            stable,
            "read functions must use the caller statement snapshot"
        );
    }
    let before = state(&pool).await;
    let mut holder = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM public.organizations WHERE id=$1 FOR UPDATE")
        .bind(OrgId::knl().as_uuid())
        .fetch_one(holder.as_mut())
        .await
        .unwrap();
    sqlx::query("SELECT id FROM public.users WHERE id=$1 FOR UPDATE")
        .bind(subject)
        .fetch_one(holder.as_mut())
        .await
        .unwrap();
    sqlx::query("SELECT id FROM public.accounts WHERE id=$1 FOR UPDATE")
        .bind(subject)
        .fetch_one(holder.as_mut())
        .await
        .unwrap();
    let results = read_all(&business, subject, key).await;
    holder.rollback().await.unwrap();
    for result in results {
        assert!(
            result.is_ok(),
            "read projection waited for an unrelated identity mutation row lock"
        );
    }
    assert!(before == state(&pool).await);
}

#[sqlx::test(migrations = false)]
async fn committed_fence_refuses_self_reads_but_clears_only_directory_legacy_flag(pool: PgPool) {
    let (business, _, subject) = setup(&pool).await;
    for with_key in [false, true] {
        let actor = if with_key {
            seed_user(&pool).await
        } else {
            subject
        };
        let key = Uuid::new_v4();
        let mut seed = pool.begin().await.unwrap();
        if with_key {
            insert_key(&mut seed, actor, key, None).await;
        }
        fence(&mut seed, actor).await;
        seed.commit().await.unwrap();
        let before = state(&pool).await;
        let mut results = read_all(&business, actor, key).await;
        let directory = results.pop().unwrap().unwrap();
        assert!(!directory[0].get::<bool, _>(0));
        for result in results {
            assert_denied(&result.unwrap_err());
        }
        assert!(
            before == state(&pool).await,
            "fenced reads cannot erase or audit retained credentials"
        );
    }
}

#[sqlx::test(migrations = false)]
async fn one_statement_snapshot_survives_mid_read_fence_then_next_statement_refuses(pool: PgPool) {
    let (business, _, subject) = setup(&pool).await;
    let mut seed = pool.begin().await.unwrap();
    insert_key(&mut seed, subject, Uuid::new_v4(), None).await;
    seed.commit().await.unwrap();
    let mut holder = pool.begin().await.unwrap();
    sqlx::query("LOCK TABLE public.auth_webauthn_credentials IN ACCESS EXCLUSIVE MODE")
        .execute(holder.as_mut())
        .await
        .unwrap();
    let mut reader = company_tx(&business).await;
    sqlx::query("SET LOCAL lock_timeout='4s'")
        .execute(reader.as_mut())
        .await
        .unwrap();
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(reader.as_mut())
        .await
        .unwrap();
    let pending = tokio::spawn(async move {
        let result = count(&mut reader, subject).await;
        reader.rollback().await.unwrap();
        result
    });
    // Synchronize on a real blocked read, not an elapsed sleep or a replacement
    // fence function. The wrapper's caller statement is already in progress.
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        let blocked: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_locks WHERE pid=$1 AND relation='public.auth_webauthn_credentials'::regclass AND mode='AccessShareLock' AND NOT granted)")
            .bind(pid).fetch_one(&pool).await.unwrap();
        if blocked {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "read did not reach controlled production relation gate"
        );
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    insert_key(&mut holder, subject, Uuid::new_v4(), None).await;
    fence(&mut holder, subject).await;
    holder.commit().await.unwrap();
    assert_eq!(
        pending.await.unwrap().unwrap(),
        1,
        "all projection SELECTs must retain the calling statement's old snapshot"
    );
    let mut next = company_tx(&business).await;
    assert_denied(&count(&mut next, subject).await.unwrap_err());
    next.rollback().await.unwrap();
}

#[sqlx::test(migrations = false)]
async fn empty_credential_reads_refuse_null_or_missing_fence_and_recover_exactly(pool: PgPool) {
    let (business, _, subject) = setup(&pool).await;
    for result in read_all(&business, subject, Uuid::new_v4()).await {
        result.expect("canonical empty-credential positive before fault injection");
    }
    let original: Value = sqlx::query_scalar("SELECT to_jsonb(p) FROM pg_catalog.pg_proc p WHERE oid='public.account_legacy_fenced_v1(uuid)'::regprocedure")
        .fetch_one(&pool).await.unwrap();
    let before = state(&pool).await;
    for missing in [false, true] {
        if missing {
            sqlx::query("ALTER FUNCTION public.account_legacy_fenced_v1(uuid) RENAME TO auth7_fixture_missing_fence")
                .execute(&pool).await.unwrap();
        } else {
            sqlx::query("UPDATE pg_catalog.pg_proc SET prosrc='BEGIN RETURN NULL; END;' WHERE oid='public.account_legacy_fenced_v1(uuid)'::regprocedure")
                .execute(&pool).await.unwrap();
        }
        // New restricted connection avoids reusing a cached function OID/plan.
        let fresh = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let results = read_all(&fresh, subject, Uuid::new_v4()).await;
        fresh.close().await;
        if missing {
            sqlx::query("ALTER FUNCTION public.auth7_fixture_missing_fence(uuid) RENAME TO account_legacy_fenced_v1")
                .execute(&pool).await.unwrap();
        } else {
            sqlx::query("UPDATE pg_catalog.pg_proc SET prosrc=$1 WHERE oid='public.account_legacy_fenced_v1(uuid)'::regprocedure")
                .bind(original["prosrc"].as_str().unwrap()).execute(&pool).await.unwrap();
        }
        for result in results {
            let error = result.unwrap_err();
            let code = error.as_database_error().and_then(|e| e.code());
            if missing {
                assert_eq!(code.as_deref(), Some("42883"));
            } else {
                assert_denied(&error);
            }
        }
        let restored: Value = sqlx::query_scalar("SELECT to_jsonb(p) FROM pg_catalog.pg_proc p WHERE oid='public.account_legacy_fenced_v1(uuid)'::regprocedure")
            .fetch_one(&pool).await.unwrap();
        assert!(
            original == restored,
            "restore exact original catalog/ACL; never rerun installer or substitute a success function"
        );
        assert!(before == state(&pool).await);
        let mut tx = company_tx(&business).await;
        assert_eq!(count(&mut tx, subject).await.unwrap(), 0);
        tx.rollback().await.unwrap();
    }
}

fn templates(subject: Uuid) -> Vec<AuditEvent> {
    let key = Uuid::new_v4();
    let family = Uuid::new_v4();
    let bootstrap = Uuid::new_v4();
    let handoff = Uuid::new_v4();
    let expires = instant() + Duration::hours(1) + Duration::nanoseconds(123_456_789);
    let encoded = json!(expires);
    assert_eq!(
        encoded.as_array().map(Vec::len),
        Some(9),
        "AUTH7_PREREQUISITE: legacy time serde feature shape changed; review exact allowlist before execution"
    );
    assert_eq!(
        encoded[5],
        json!(123_456_789),
        "snapshot nanoseconds must remain intact"
    );
    let mut events = Vec::new();
    for (action, target, id, after) in [
        (
            "auth.passkey.register",
            "auth_webauthn_credential",
            key,
            json!({"credential_id":key.to_string(),"user_id":subject}),
        ),
        (
            "auth.refresh.issue",
            "auth_refresh_token_family",
            family,
            json!({"family_id":family,"token_id":Uuid::new_v4(),"user_id":subject,"expires_at":expires}),
        ),
        (
            "auth.refresh",
            "auth_refresh_token_family",
            family,
            json!({"family_id":family,"used_token_id":Uuid::new_v4(),"replacement_token_id":Uuid::new_v4(),"expires_at":expires}),
        ),
        (
            "auth.refresh.absolute_ttl_revoked",
            "auth_refresh_token_family",
            family,
            json!({"family_id":family,"revoked_reason":"absolute_ttl_exceeded","family_created_at":instant()}),
        ),
        (
            "auth.refresh.reuse_detected",
            "auth_refresh_token_family",
            family,
            json!({"family_id":family,"revoked_reason":"reuse_detected","reused_token_id":Uuid::new_v4()}),
        ),
        (
            "auth.logout",
            "auth_refresh_token_family",
            family,
            json!({"family_id":family,"revoked_reason":"logout"}),
        ),
        (
            "auth.otp.redeem",
            "auth_bootstrap_credential",
            bootstrap,
            json!({"user_id":subject,"requires_passkey_setup":true}),
        ),
        (
            "auth.otp.consume",
            "auth_bootstrap_credential",
            bootstrap,
            json!({"user_id":subject}),
        ),
        (
            "auth.passkey.enroll_handoff_issued",
            "auth_bootstrap_credential",
            bootstrap,
            json!({"user_id":subject,"expires_at":expires,"purpose":"passkey_enrollment_handoff"}),
        ),
        (
            "auth.login",
            "users",
            subject,
            json!({"passkey_id":key,"refresh_family_id":family}),
        ),
        (
            "auth.otp.signin",
            "users",
            subject,
            json!({"refresh_family_id":family,"requires_passkey_setup":false}),
        ),
        (
            "auth.device_login.approve",
            "users",
            subject,
            json!({"handoff_id":handoff,"passkey_id":key}),
        ),
        (
            "auth.device_login.approve_session",
            "users",
            subject,
            json!({"handoff_id":handoff,"passkey_id":null}),
        ),
        (
            "auth.device_login.consume",
            "users",
            subject,
            json!({"handoff_id":handoff,"passkey_id":null,"refresh_family_id":family}),
        ),
    ] {
        events.push(
            AuditEvent::new(
                Some(UserId::from_uuid(subject)),
                AuditAction::new(action).unwrap(),
                target,
                id.to_string(),
                TraceContext::generate(),
                instant(),
            )
            .with_org(OrgId::knl())
            .with_snapshots(None, Some(after)),
        );
    }
    for (action, after) in [
        ("auth.otp.redeem_failed", json!({"outcome":"rejected"})),
        (
            "auth.device_login.start",
            json!({"handoff_id":handoff,"expires_at":expires}),
        ),
    ] {
        events.push(
            AuditEvent::new(
                None,
                AuditAction::new(action).unwrap(),
                "auth_bootstrap_credential",
                "redeem",
                TraceContext::generate(),
                instant(),
            )
            .with_snapshots(None, Some(after)),
        );
    }
    assert_eq!(events.len(), 16);
    events
}

async fn append(tx: &mut Transaction<'_, Postgres>, event: &AuditEvent) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT public.auth_legacy_audit_append_v1($1,$2,$3,$4,$5,$6,$7,$8,$9::char(32),$10::char(16),$11,$12,$13,$14,$15,$16,$17,$18,$19)")
        .bind(event.id.as_uuid()).bind(event.actor.map(|v| *v.as_uuid()))
        .bind(event.action.as_str()).bind(&event.target_type).bind(&event.target_id)
        .bind(event.branch_id.map(|v| *v.as_uuid())).bind(&event.before).bind(&event.after)
        .bind(event.trace.trace_id()).bind(event.trace.span_id()).bind(event.occurred_at)
        .bind(event.org_id.map(|v| *v.as_uuid())).bind(&event.request_context.ip)
        .bind(&event.request_context.user_agent).bind(&event.request_context.auth_method)
        .bind(&event.request_context.device).bind(&event.classification.badges)
        .bind(event.classification.anomaly).bind(&event.classification.reason)
        .execute(tx.as_mut()).await.map(|_| ())
}

async fn assert_row(pool: &PgPool, event: &AuditEvent) {
    let row = sqlx::query("SELECT id,actor,action,target_type,target_id,branch_id,before_snap,after_snap,trace_id,span_id,occurred_at,org_id,ip,user_agent,auth_method,device,classification_badges,anomaly,reason FROM public.audit_events WHERE id=$1")
        .bind(event.id.as_uuid()).fetch_one(pool).await.unwrap();
    assert_eq!(row.len(), 19);
    assert_eq!(row.get::<Uuid, _>("id"), *event.id.as_uuid());
    assert_eq!(
        row.get::<Option<Uuid>, _>("actor"),
        event.actor.map(|v| *v.as_uuid())
    );
    assert_eq!(row.get::<String, _>("action"), event.action.as_str());
    assert_eq!(row.get::<String, _>("target_type"), event.target_type);
    assert_eq!(row.get::<String, _>("target_id"), event.target_id);
    assert_eq!(
        row.get::<Option<Uuid>, _>("branch_id"),
        event.branch_id.map(|v| *v.as_uuid())
    );
    assert!(row.get::<Option<Value>, _>("before_snap") == event.before);
    assert!(
        row.get::<Option<Value>, _>("after_snap") == event.after,
        "exact JSON values including nanoseconds"
    );
    assert_eq!(row.get::<String, _>("trace_id"), event.trace.trace_id());
    assert_eq!(row.get::<String, _>("span_id"), event.trace.span_id());
    assert_eq!(
        row.get::<OffsetDateTime, _>("occurred_at"),
        event.occurred_at
    );
    assert_eq!(
        row.get::<Option<Uuid>, _>("org_id"),
        event.org_id.map(|v| *v.as_uuid())
    );
    for column in ["ip", "user_agent", "auth_method", "device", "reason"] {
        assert!(row.get::<Option<String>, _>(column).is_none());
    }
    assert!(
        row.get::<Option<Vec<String>>, _>("classification_badges")
            .is_none()
    );
    assert!(row.get::<Option<bool>, _>("anomaly").is_none());
}

#[sqlx::test(migrations = false)]
async fn audit_all_sixteen_templates_preserve_every_current_column_and_time_value(pool: PgPool) {
    let (_, auth, subject) = setup(&pool).await;
    let events = templates(subject);
    let mut tx = company_tx(&auth).await;
    for event in &events {
        append(&mut tx, event).await.unwrap();
    }
    tx.commit().await.unwrap();
    for event in &events {
        assert_row(&pool, event).await;
    }
    // Exercise the other permitted Option serialization in the two session templates.
    for action in [
        "auth.device_login.approve_session",
        "auth.device_login.consume",
    ] {
        let mut event = templates(subject)
            .into_iter()
            .find(|e| e.action.as_str() == action)
            .unwrap();
        event.after.as_mut().unwrap()["passkey_id"] = json!(Uuid::new_v4());
        let mut tx = company_tx(&auth).await;
        append(&mut tx, &event).await.unwrap();
        tx.commit().await.unwrap();
        assert_row(&pool, &event).await;
    }
}

async fn reject_unchanged(owner: &PgPool, auth: &PgPool, event: &AuditEvent) {
    let before = state(owner).await;
    let mut tx = company_tx(auth).await;
    let result = append(&mut tx, event).await;
    tx.rollback().await.unwrap();
    assert_denied(&result.unwrap_err());
    assert!(
        before == state(owner).await,
        "denied audit capability cannot change credential or audit state"
    );
}

#[sqlx::test(migrations = false)]
async fn audit_rejects_extra_fields_types_targets_identity_anonymous_and_metadata_channels(
    pool: PgPool,
) {
    let (_, auth, subject) = setup(&pool).await;
    let events = templates(subject);
    // Every template must reject an unlisted key; a nominal positive must pass
    // first so a missing helper or blanket-deny implementation cannot satisfy it.
    for event in &events {
        let mut positive = company_tx(&auth).await;
        append(&mut positive, event).await.unwrap();
        positive.rollback().await.unwrap();
        let mut bad = event.clone();
        bad.after.as_mut().unwrap()["unlisted_secret"] =
            json!("must not be an audit message channel");
        reject_unchanged(&pool, &auth, &bad).await;
        let mut bad = event.clone();
        bad.target_type = "user".into();
        reject_unchanged(&pool, &auth, &bad).await;
        let mut bad = event.clone();
        bad.after = Some(json!("wrong top-level type"));
        reject_unchanged(&pool, &auth, &bad).await;
    }
    let nominal = events[0].clone();
    let mut bads = Vec::new();
    for action in [
        "auth.unlisted",
        "auth.bootstrap.issue",
        "auth.signup",
        "auth.passkey.admin_reset",
        "auth.otp.issue",
        "auth.coldstart.seed",
        "auth.passkey.revoke",
        "auth.passkey.revoke_all",
        "auth.refresh.revoke_all",
        "dev_auth.session.mint",
    ] {
        let mut bad = nominal.clone();
        bad.action = AuditAction::new(action).unwrap();
        bads.push(bad);
    }
    let mut bad = nominal.clone();
    bad.actor = None;
    bads.push(bad);
    let mut bad = nominal.clone();
    bad.actor = Some(UserId::new());
    bads.push(bad);
    let mut bad = nominal.clone();
    bad.org_id = None;
    bads.push(bad);
    let mut bad = nominal.clone();
    bad.org_id = Some(OrgId::from_uuid(Uuid::new_v4()));
    bads.push(bad);
    let mut bad = nominal.clone();
    bad.branch_id = Some(BranchId::new());
    bads.push(bad);
    let mut bad = nominal.clone();
    bad.before = Some(json!({}));
    bads.push(bad);
    let mut bad = nominal.clone();
    bad.after = None;
    bads.push(bad);
    let mut bad = nominal.clone();
    bad.after.as_mut().unwrap()["user_id"] = json!(Uuid::new_v4());
    bads.push(bad);
    let mut bad = nominal.clone();
    bad.after.as_mut().unwrap()["credential_id"] = json!({"private":"state"});
    bads.push(bad);
    let mut bad = nominal.clone();
    bad.request_context.ip = Some("private message".into());
    bads.push(bad);
    let mut bad = nominal.clone();
    bad.request_context.user_agent = Some("private message".into());
    bads.push(bad);
    let mut bad = nominal.clone();
    bad.request_context.auth_method = Some("private message".into());
    bads.push(bad);
    let mut bad = nominal.clone();
    bad.request_context.device = Some("private message".into());
    bads.push(bad);
    let mut bad = nominal.clone();
    bad.classification.badges = Some(vec!["private message".into()]);
    bads.push(bad);
    let mut bad = nominal.clone();
    bad.classification.anomaly = Some(false);
    bads.push(bad);
    let mut bad = nominal.clone();
    bad.classification.reason = Some("private message".into());
    bads.push(bad);
    let mut bad = events[14].clone();
    bad.actor = nominal.actor;
    bads.push(bad);
    let mut bad = events[14].clone();
    bad.org_id = nominal.org_id;
    bads.push(bad);
    let mut bad = events[15].clone();
    bad.target_id = subject.to_string();
    bads.push(bad);
    let mut bad = events[15].clone();
    bad.after.as_mut().unwrap()["expires_at"] = json!("2027-01-15T08:00:00Z");
    bads.push(bad);
    for bad in &bads {
        reject_unchanged(&pool, &auth, bad).await;
    }
    let mut seed = pool.begin().await.unwrap();
    fence(&mut seed, subject).await;
    seed.commit().await.unwrap();
    reject_unchanged(&pool, &auth, &nominal).await;
}

#[sqlx::test(migrations = false)]
async fn audit_execute_is_auth_only_and_auth_has_no_raw_audit_table_capability(pool: PgPool) {
    let (business, auth, subject) = setup(&pool).await;
    let public: i64=sqlx::query_scalar("SELECT count(*) FROM pg_catalog.pg_proc p CROSS JOIN LATERAL pg_catalog.aclexplode(COALESCE(p.proacl,pg_catalog.acldefault('f',p.proowner))) a WHERE p.oid=pg_catalog.to_regprocedure($1) AND a.grantee=0 AND a.privilege_type='EXECUTE'")
        .bind(AUDIT_SIGNATURE).fetch_one(&pool).await.unwrap();
    assert_eq!(public, 0, "PUBLIC has no executable audit capability");
    let event = templates(subject).remove(0);
    let mut allowed = company_tx(&auth).await;
    append(&mut allowed, &event).await.unwrap();
    allowed.rollback().await.unwrap();
    let mut denied = company_tx(&business).await;
    let result = append(&mut denied, &event).await;
    denied.rollback().await.unwrap();
    assert_eq!(
        result
            .unwrap_err()
            .as_database_error()
            .unwrap()
            .code()
            .as_deref(),
        Some("42501")
    );
    for query in [
        "SELECT id FROM public.audit_events LIMIT 1",
        "INSERT INTO public.audit_events(id,action,target_type,target_id,trace_id,span_id,occurred_at) VALUES (gen_random_uuid(),'auth.otp.redeem_failed','auth_bootstrap_credential','redeem',repeat('a',32),repeat('b',16),now()) RETURNING id",
    ] {
        let mut tx = company_tx(&auth).await;
        let result = sqlx::query(sqlx::AssertSqlSafe(query))
            .fetch_all(tx.as_mut())
            .await;
        tx.rollback().await.unwrap();
        assert_eq!(
            result
                .unwrap_err()
                .as_database_error()
                .unwrap()
                .code()
                .as_deref(),
            Some("42501")
        );
    }
}

#[sqlx::test(migrations = false)]
async fn credential_mutation_and_audit_append_roll_back_in_the_same_auth_transaction(pool: PgPool) {
    let (_, auth, subject) = setup(&pool).await;
    let event = templates(subject).remove(0);
    let key = Uuid::parse_str(&event.target_id).unwrap();
    let before = state(&pool).await;
    let mut tx = company_tx(&auth).await;
    insert_key(&mut tx, subject, key, None).await;
    append(&mut tx, &event).await.unwrap();
    assert!(
        before == state(&pool).await,
        "uncommitted key and event cannot become externally visible"
    );
    tx.rollback().await.unwrap();
    assert!(
        before == state(&pool).await,
        "both successful statements share rollback"
    );
    let mut tx = company_tx(&auth).await;
    insert_key(&mut tx, subject, key, None).await;
    let mut bad = event.clone();
    bad.after.as_mut().unwrap()["extra"] = json!(true);
    let refused = append(&mut tx, &bad).await;
    tx.rollback().await.unwrap();
    assert_denied(&refused.unwrap_err());
    assert!(
        before == state(&pool).await,
        "audit refusal must not strand preceding credential mutation"
    );
}
