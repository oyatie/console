#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Offboarding closure: `deactivate_user` must revoke EVERY credential + session,
//! not merely flip `is_active`.
//!
//! A deactivated user who keeps an enrolled passkey or a live refresh-token
//! family is still a hole: the passkey authenticates and the family rotates until
//! natural expiry. This test runs the REAL `PgOrgStore::deactivate_user` as the
//! genuine non-owner `console_rt` role (FORCE RLS applies, exactly like prod) and
//! proves that after deactivation:
//!   * the user's WebAuthn credential rows are GONE (passkeys can't authenticate),
//!   * every refresh-token family + token is revoked (refresh fails closed,
//!     verified by a real `RefreshTokenStore::rotate` that now returns
//!     `FamilyRevoked`), and
//!   * each sub-action is audited.

use console_identity_adapter_postgres::PgOrgStore;
use console_identity_application::DeactivateUserCommand;
use console_kernel_core::{OrgId, TraceContext, UserId};
use console_platform_auth::{RefreshTokenStore, RefreshTokenUseError};
use console_platform_request_context::CURRENT_ORG;
use console_platform_test_support::{
    TestDatabaseLogin, login_test_pool, prepare_account_test_database,
};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// A pool whose every connection runs `SET ROLE console_rt`, so statements execute as
/// the production runtime role (NOSUPERUSER, NOBYPASSRLS) under FORCE RLS.
async fn runtime_role_pool(owner_pool: &PgPool) -> PgPool {
    let options = owner_pool.connect_options().as_ref().clone();
    PgPoolOptions::new()
        .max_connections(4)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET ROLE console_rt").execute(conn).await?;
                Ok(())
            })
        })
        .connect_with(options)
        .await
        .unwrap()
}

/// Seed an organization + one user as the OWNER with `row_security` off.
async fn seed_org_and_user(owner_pool: &PgPool, org: Uuid) -> Uuid {
    let mut tx = owner_pool.begin().await.unwrap();
    sqlx::query("SET LOCAL row_security = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO organizations (id, slug, name) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING",
    )
    .bind(org)
    .bind("org-knl")
    .bind("Org KNL")
    .execute(&mut *tx)
    .await
    .unwrap();
    let user_id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (display_name, roles, org_id, is_active) VALUES ($1, $2, $3, true) RETURNING id",
    )
    .bind("Offboard User")
    .bind(vec!["MECHANIC".to_string()])
    .bind(org)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    user_id
}

/// Insert a WebAuthn credential row for the user (owner pool, row_security off).
/// The `passkey_json` payload is opaque to deactivation — only the row's presence
/// matters for the revoke assertion.
async fn seed_credential(owner_pool: &PgPool, org: Uuid, user_id: Uuid, credential_id: &str) {
    let mut tx = owner_pool.begin().await.unwrap();
    sqlx::query("SET LOCAL row_security = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query(
        r#"
        INSERT INTO auth_webauthn_credentials
            (id, user_id, credential_id, passkey_json, created_at, org_id)
        VALUES ($1, $2, $3, $4, now(), $5)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(credential_id)
    .bind(serde_json::json!({ "stub": true }))
    .bind(org)
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

// Privileged fixture observation only: Business must not read credential rows.
// Preserve the original Company RLS + user scope as an explicit two-key predicate.
async fn count_credentials_as_owner(owner_pool: &PgPool, org: OrgId, user_id: Uuid) -> i64 {
    let mut tx = owner_pool.begin().await.unwrap();
    sqlx::query("SET LOCAL row_security = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM public.auth_webauthn_credentials WHERE org_id = $1 AND user_id = $2",
    )
    .bind(org.as_uuid())
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    count
}

async fn audit_count(owner_pool: &PgPool, action: &str, user_id: Uuid) -> i64 {
    let mut tx = owner_pool.begin().await.unwrap();
    sqlx::query("SET LOCAL row_security = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_events WHERE action = $1 AND target_id = $2",
    )
    .bind(action)
    .bind(user_id.to_string())
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    count
}

#[sqlx::test(migrations = false)]
async fn deactivate_revokes_passkeys_and_sessions_as_runtime_role(owner_pool: PgPool) {
    prepare_account_test_database(&owner_pool).await;
    let rt_pool = runtime_role_pool(&owner_pool).await;
    let auth_pool = login_test_pool(&owner_pool, TestDatabaseLogin::Auth).await;
    let knl = OrgId::knl();
    let user_id = seed_org_and_user(&owner_pool, *knl.as_uuid()).await;
    // The actor must be a real user (audit_events.actor FKs to users).
    let actor_id = seed_org_and_user(&owner_pool, *knl.as_uuid()).await;

    // The user has an enrolled passkey ...
    seed_credential(&owner_pool, *knl.as_uuid(), user_id, "cred-offboard-1").await;
    assert_eq!(
        count_credentials_as_owner(&owner_pool, knl, user_id).await,
        1
    );

    // ... and a live refresh-token family (their session), minted as console_rt.
    let now = OffsetDateTime::now_utc();
    let family = RefreshTokenStore
        .issue_family(&rt_pool, &auth_pool, user_id, knl, now, Duration::days(30))
        .await
        .expect("issue_family must pass RLS as console_rt");

    // Deactivate via the REAL adapter, with the org task-local armed exactly as
    // the request-context middleware arms it on the authenticated route.
    let store = PgOrgStore::new(rt_pool.clone());
    let summary = CURRENT_ORG
        .scope(
            knl,
            store.deactivate_user(DeactivateUserCommand {
                actor: UserId::from_uuid(actor_id),
                user_id: UserId::from_uuid(user_id),
                trace: TraceContext::generate(),
                occurred_at: now,
            }),
        )
        .await
        .expect("deactivate_user must succeed as console_rt");
    assert!(!summary.is_active, "the user is soft-deactivated");

    // 1) Every passkey is gone: a deactivated user can no longer authenticate.
    assert_eq!(
        count_credentials_as_owner(&owner_pool, knl, user_id).await,
        0,
        "deactivation must DELETE all of the user's passkeys"
    );

    // 2) The refresh family is dead: rotating the issued token now fails closed
    //    with FamilyRevoked (a real rotation, not a DB peek).
    let rotate = RefreshTokenStore
        .rotate(
            &rt_pool,
            &auth_pool,
            family.token.as_str(),
            now + Duration::minutes(1),
            Duration::days(30),
            Duration::days(30),
        )
        .await
        .expect_err("a deactivated user's refresh token must not rotate");
    assert_eq!(rotate, RefreshTokenUseError::FamilyRevoked);

    // 3) Each sub-action is audited (deactivate + passkey revoke + session revoke).
    assert_eq!(
        audit_count(&owner_pool, "user.deactivate", user_id).await,
        1
    );
    assert_eq!(
        audit_count(&owner_pool, "auth.passkey.revoke_all", user_id).await,
        1
    );
    assert_eq!(
        audit_count(&owner_pool, "auth.refresh.revoke_all", user_id).await,
        1
    );
}

// AS1 owner-boundary tests. Company deactivation is not Account suspension.
// Explicit security rows below are privileged fixture facts, not enrollment or
// native-session construction. Real legacy family history is retained custody;
// these tests never describe it as authenticated ACCOUNT_V1 authority.
mod account_custody_deactivation {
    use super::*;
    use console_identity_adapter_postgres::PgOrgError;
    use console_kernel_core::ErrorKind;
    use serde_json::Value;

    struct Subject {
        user: Uuid,
        actor: Uuid,
        current_refresh: String,
        now: OffsetDateTime,
    }

    async fn bounded_login(owner: &PgPool, login: TestDatabaseLogin) -> PgPool {
        // Start with the existing helper's verified real LOGIN configuration.
        // No SET ROLE, credential grants or auth-only API is exposed to Business.
        let verified = login_test_pool(owner, login).await;
        let options = verified.connect_options().as_ref().clone();
        verified.close().await;
        PgPoolOptions::new()
            .max_connections(1)
            .after_connect(|connection, _| {
                Box::pin(async move {
                    sqlx::raw_sql("SET lock_timeout = '250ms'; SET statement_timeout = '2s';")
                        .execute(connection)
                        .await?;
                    Ok(())
                })
            })
            .connect_with(options)
            .await
            .expect("connect bounded real LOGIN for owner-boundary proof")
    }

    async fn subject(owner: &PgPool, business: &PgPool, auth: &PgPool) -> Subject {
        let org = OrgId::knl();
        let user = seed_org_and_user(owner, *org.as_uuid()).await;
        let actor = seed_org_and_user(owner, *org.as_uuid()).await;
        seed_credential(
            owner,
            *org.as_uuid(),
            user,
            &format!("account-deactivation-key-{user}"),
        )
        .await;
        let now = OffsetDateTime::now_utc();
        let first = RefreshTokenStore
            .issue_family(business, auth, user, org, now, Duration::days(30))
            .await
            .expect("unfenced legacy issuance is a required setup positive");
        let rotated = RefreshTokenStore
            .rotate(
                business,
                auth,
                first.token.as_str(),
                now + Duration::minutes(1),
                Duration::days(30),
                Duration::days(30),
            )
            .await
            .expect("real rotation must establish retained token history");
        let historical = RefreshTokenStore
            .issue_family(business, auth, user, org, now, Duration::days(30))
            .await
            .expect("second real family is a required preservation fixture");
        RefreshTokenStore
            .revoke_family_for_logout(business, historical.token.as_str(), now)
            .await
            .expect("fixture must retain an already-revoked family too");
        Subject {
            user,
            actor,
            current_refresh: rotated.token.as_str().to_owned(),
            now,
        }
    }

    async fn fence(owner: &PgPool, subject: &Subject, state: &str) {
        // users INSERT already created the immutable Account root via the real
        // bridge. Do not invent a second root or derive state from Employment.
        let affected = sqlx::query(
            "INSERT INTO public.account_security \
             (account_id,security_state,security_generation,revision,updated_at,context_generation) \
             VALUES ($1,$2,41,7,$3,11)",
        )
        .bind(subject.user)
        .bind(state)
        .bind(subject.now)
        .execute(owner)
        .await
        .expect("explicit Account security fixture requires existing root")
        .rows_affected();
        assert_eq!(affected, 1);
    }

    async fn snapshot(owner: &PgPool, user: Uuid) -> Value {
        // Owner readback is an observation fixture. It grants no Business
        // credential access and compares opaque rows without parsing key bytes.
        sqlx::query_scalar(
            r#"
            SELECT jsonb_build_object(
                'root',(SELECT to_jsonb(a) FROM public.accounts a WHERE a.id=$1),
                'security',(SELECT jsonb_build_object(
                    'account_id',s.account_id,'security_state',s.security_state,
                    'security_generation',s.security_generation)
                    FROM public.account_security s WHERE s.account_id=$1),
                'keys',(SELECT COALESCE(jsonb_agg(to_jsonb(k) ORDER BY k.id),'[]'::jsonb)
                    FROM public.auth_webauthn_credentials k WHERE k.user_id=$1),
                'families',(SELECT COALESCE(jsonb_agg(to_jsonb(f) ORDER BY f.id),'[]'::jsonb)
                    FROM public.auth_refresh_token_families f WHERE f.user_id=$1),
                'tokens',(SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb)
                    FROM public.auth_refresh_tokens t WHERE t.user_id=$1),
                'security_events',(SELECT COALESCE(jsonb_agg(to_jsonb(e) ORDER BY e.id),'[]'::jsonb)
                    FROM public.account_security_events e WHERE e.account_id=$1),
                'company_user',(SELECT to_jsonb(u) FROM public.users u WHERE u.id=$1),
                'company_versions',(SELECT COALESCE(jsonb_agg(to_jsonb(v) ORDER BY v.org_id),'[]'::jsonb)
                    FROM public.subject_authz_versions v WHERE v.user_id=$1),
                'raw_security',(SELECT to_jsonb(s) FROM public.account_security s WHERE s.account_id=$1),
                'audit',(SELECT COALESCE(jsonb_agg(to_jsonb(e) ORDER BY e.id),'[]'::jsonb)
                    FROM public.audit_events e)
            )
            "#,
        )
        .bind(user)
        .fetch_one(owner)
        .await
        .expect("complete owner observation must be available")
    }

    fn retained(mut snapshot: Value) -> Value {
        // Company permission/context revisions may legitimately change. The
        // Account security state/generation and credential history may not.
        let object = snapshot.as_object_mut().unwrap();
        for field in ["company_user", "company_versions", "raw_security", "audit"] {
            assert!(object.remove(field).is_some());
        }
        snapshot
    }

    async fn expected_legacy_custody(owner: &PgPool, before: &Value, subject: &Subject) -> Value {
        // Convert the supplied event time BEFORE effects using PostgreSQL's
        // native precision/JSON encoding, never the observed mutation result.
        let occurred_at: Value = sqlx::query_scalar("SELECT to_jsonb($1::timestamptz)")
            .bind(subject.now + Duration::minutes(2))
            .fetch_one(owner)
            .await
            .expect("serialize the exact supplied legacy revocation time");
        let mut expected = retained(before.clone());
        expected["keys"] = serde_json::json!([]);
        for family in expected["families"].as_array_mut().unwrap() {
            if family["revoked_at"].is_null() {
                family["revoked_at"] = occurred_at.clone();
                family["revoked_reason"] = serde_json::json!("user_deactivated");
            }
        }
        for token in expected["tokens"].as_array_mut().unwrap() {
            if token["revoked_at"].is_null() {
                token["revoked_at"] = occurred_at.clone();
            }
        }
        expected
    }

    async fn deactivate(business: &PgPool, subject: &Subject) -> Result<bool, PgOrgError> {
        let store = PgOrgStore::new(business.clone());
        CURRENT_ORG
            .scope(
                OrgId::knl(),
                store.deactivate_user(DeactivateUserCommand {
                    actor: UserId::from_uuid(subject.actor),
                    user_id: UserId::from_uuid(subject.user),
                    trace: TraceContext::generate(),
                    occurred_at: subject.now + Duration::minutes(2),
                }),
            )
            .await
            .map(|summary| summary.is_active)
    }

    async fn assert_retained_deactivation(owner: &PgPool, state: &str) {
        let business = bounded_login(owner, TestDatabaseLogin::Business).await;
        let auth = bounded_login(owner, TestDatabaseLogin::Auth).await;
        let subject = subject(owner, &business, &auth).await;
        fence(owner, &subject, state).await;
        let before = snapshot(owner, subject.user).await;
        assert_eq!(before["company_user"]["is_active"], true);
        assert_eq!(before["security"]["security_state"], state);
        assert_eq!(before["security"]["security_generation"], 41);
        assert_eq!(before["keys"].as_array().unwrap().len(), 1);
        assert_eq!(before["families"].as_array().unwrap().len(), 2);
        assert_eq!(before["tokens"].as_array().unwrap().len(), 3);
        let security_audits_before = (
            audit_count(owner, "auth.passkey.revoke_all", subject.user).await,
            audit_count(owner, "auth.refresh.revoke_all", subject.user).await,
        );
        let active = deactivate(&business, &subject)
            .await
            .expect("Company relationship deactivation must remain usable for fenced Account");
        assert!(!active, "Company owner result must remove work eligibility");
        let after = snapshot(owner, subject.user).await;
        assert_eq!(after["company_user"]["is_active"], false);
        assert!(
            retained(before.clone()) == retained(after),
            "COMPANY_DEACTIVATION_CHANGED_ACCOUNT_CUSTODY: Account state/generation and key/family/token history must survive"
        );
        assert_eq!(
            (
                audit_count(owner, "auth.passkey.revoke_all", subject.user).await,
                audit_count(owner, "auth.refresh.revoke_all", subject.user).await,
            ),
            security_audits_before,
            "Company relationship action must not claim global security revocation"
        );
        assert_eq!(audit_count(owner, "user.deactivate", subject.user).await, 1);

        // Existing API reports a conflict for already-inactive replay. Its old
        // unconditional replay sweep must not gain Account credential authority.
        let replay_error = deactivate(&business, &subject)
            .await
            .expect_err("already-inactive replay must preserve the existing Conflict contract");
        assert_eq!(replay_error.kind(), ErrorKind::Conflict);
        let replay = snapshot(owner, subject.user).await;
        assert_eq!(replay["company_user"]["is_active"], false);
        assert!(
            retained(before) == retained(replay),
            "COMPANY_DEACTIVATION_REPLAY_CHANGED_ACCOUNT_CUSTODY"
        );
        assert_eq!(audit_count(owner, "user.deactivate", subject.user).await, 1);
        assert_eq!(
            (
                audit_count(owner, "auth.passkey.revoke_all", subject.user).await,
                audit_count(owner, "auth.refresh.revoke_all", subject.user).await,
            ),
            security_audits_before,
            "Company replay must not claim global security revocation"
        );
        business.close().await;
        auth.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn active_account_custody_survives_company_deactivation_and_replay(owner: PgPool) {
        prepare_account_test_database(&owner).await;
        assert_retained_deactivation(&owner, "ACTIVE").await;
    }

    #[sqlx::test(migrations = false)]
    async fn nonactive_account_security_is_not_reclassified_by_company_deactivation(owner: PgPool) {
        prepare_account_test_database(&owner).await;
        for state in [
            "PENDING_ENROLLMENT",
            "SECURITY_SUSPENDED",
            "RECOVERY_REQUIRED",
        ] {
            assert_retained_deactivation(&owner, state).await;
        }
    }

    async fn unavailable_authority_refuses(owner: &PgPool, fenced: bool) {
        let business = bounded_login(owner, TestDatabaseLogin::Business).await;
        let auth = bounded_login(owner, TestDatabaseLogin::Auth).await;
        let subject = subject(owner, &business, &auth).await;
        if fenced {
            fence(owner, &subject, "ACTIVE").await;
        }
        let available: bool = sqlx::query_scalar("SELECT public.account_legacy_fenced_v1($1)")
            .bind(subject.user)
            .fetch_one(&auth)
            .await
            .expect("real Auth authority positive control must execute");
        assert_eq!(available, fenced);
        let before = snapshot(owner, subject.user).await;
        let legacy_expected = if fenced {
            None
        } else {
            Some(expected_legacy_custody(owner, &before, &subject).await)
        };

        let mut blocked_authority = owner.begin().await.unwrap();
        sqlx::query("LOCK TABLE public.account_security IN ACCESS EXCLUSIVE MODE")
            .execute(&mut *blocked_authority)
            .await
            .unwrap();
        let unavailable =
            sqlx::query_scalar::<_, bool>("SELECT public.account_legacy_fenced_v1($1)")
                .bind(subject.user)
                .fetch_one(&auth)
                .await
                .expect_err("locked Account authority must be genuinely unavailable");
        assert_eq!(
            unavailable
                .as_database_error()
                .and_then(|error| error.code())
                .as_deref(),
            Some("55P03"),
            "fault must be the actual Account authority lock timeout"
        );
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(3),
            deactivate(&business, &subject),
        )
        .await;
        blocked_authority.rollback().await.unwrap();
        let after = snapshot(owner, subject.user).await;
        assert!(
            before == after,
            "UNAVAILABLE_ACCOUNT_AUTHORITY_MUTATED_COMPANY_OR_CUSTODY: current classification is mandatory before effects"
        );
        assert!(
            result
                .expect("canonical owner must return within bounded database timeout")
                .is_err(),
            "unavailable Account authority must refuse, never select legacy revocation"
        );

        // Restoration must permit the real Company action; always-deny cannot
        // pass this availability test. The native case still preserves custody.
        let active = deactivate(&business, &subject)
            .await
            .expect("restored authority must permit Company deactivation");
        assert!(!active);
        let restored = snapshot(owner, subject.user).await;
        assert_eq!(restored["company_user"]["is_active"], false);
        if fenced {
            assert!(retained(before) == retained(restored));
        } else {
            assert!(
                legacy_expected.unwrap() == retained(restored),
                "RESTORED_LEGACY_REVOCATION_CHANGED_RETAINED_HISTORY: only the exact permitted revocation fields may change"
            );
        }
        business.close().await;
        auth.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn unavailable_account_authority_preserves_fenced_subject_before_company_effect(
        owner: PgPool,
    ) {
        prepare_account_test_database(&owner).await;
        unavailable_authority_refuses(&owner, true).await;
    }

    #[sqlx::test(migrations = false)]
    async fn unavailable_account_authority_cannot_select_unfenced_legacy_revocation(owner: PgPool) {
        prepare_account_test_database(&owner).await;
        unavailable_authority_refuses(&owner, false).await;
    }

    #[sqlx::test(migrations = false)]
    async fn unfenced_legacy_subject_still_loses_keys_families_and_refresh_after_company_deactivation(
        owner: PgPool,
    ) {
        prepare_account_test_database(&owner).await;
        let business = bounded_login(&owner, TestDatabaseLogin::Business).await;
        let auth = bounded_login(&owner, TestDatabaseLogin::Auth).await;
        let subject = subject(&owner, &business, &auth).await;
        let before = snapshot(&owner, subject.user).await;
        assert!(before["security"].is_null());
        assert_eq!(before["keys"].as_array().unwrap().len(), 1);
        assert_eq!(before["families"].as_array().unwrap().len(), 2);
        assert_eq!(before["tokens"].as_array().unwrap().len(), 3);
        let expected = expected_legacy_custody(&owner, &before, &subject).await;
        assert!(!deactivate(&business, &subject).await.unwrap());
        let after = snapshot(&owner, subject.user).await;
        assert_eq!(after["company_user"]["is_active"], false);
        assert!(
            after["security"].is_null(),
            "legacy offboarding must not enroll an Account"
        );
        assert_eq!(after["keys"], serde_json::json!([]));
        assert!(
            expected == retained(after),
            "LEGACY_REVOCATION_CHANGED_RETAINED_HISTORY: preserve every family/token row and all other fields"
        );
        let error = RefreshTokenStore
            .rotate(
                &business,
                &auth,
                &subject.current_refresh,
                subject.now + Duration::minutes(3),
                Duration::days(30),
                Duration::days(30),
            )
            .await
            .expect_err("unfenced legacy offboarding must still prevent actual refresh");
        assert_eq!(error, RefreshTokenUseError::FamilyRevoked);
        assert!(
            expected == retained(snapshot(&owner, subject.user).await),
            "refused legacy refresh must preserve the exact revoked history"
        );
        assert_eq!(
            audit_count(&owner, "user.deactivate", subject.user).await,
            1
        );
        assert_eq!(
            audit_count(&owner, "auth.passkey.revoke_all", subject.user).await,
            1
        );
        assert_eq!(
            audit_count(&owner, "auth.refresh.revoke_all", subject.user).await,
            1
        );
        business.close().await;
        auth.close().await;
    }

    // Approved GUIDE19f29899 + V2fee70922. These privileged INSERTs model
    // the real immediate FK; they are not native enrollment/authentication.
    mod guard_races {
        use super::*;
        use console_platform_test_support::{
            account_custody_finalizer_sql, finalize_account_custody,
        };
        use tokio::task::JoinHandle;

        async fn race_pool(owner: &PgPool) -> PgPool {
            PgPoolOptions::new()
                .max_connections(1)
                .connect_with(owner.connect_options().as_ref().clone())
                .await
                .unwrap()
        }

        async fn race_pid(pool: &PgPool) -> i32 {
            let mut connection = pool.acquire().await.unwrap();
            sqlx::raw_sql("SET lock_timeout='12s'; SET statement_timeout='15s'")
                .execute(&mut *connection)
                .await
                .unwrap();
            sqlx::query_scalar("SELECT pg_backend_pid()")
                .fetch_one(&mut *connection)
                .await
                .unwrap()
        }

        // Return observations without asserting while independent tasks/blockers
        // are live. Release transactions, join tasks and close their pools first.
        async fn blocked_by(owner: &PgPool, waiting: i32, holding: i32) -> bool {
            tokio::time::timeout(std::time::Duration::from_secs(4), async {
                loop {
                    match sqlx::query_scalar::<_, bool>(
                        "SELECT $2=ANY(pg_catalog.pg_blocking_pids($1))",
                    )
                    .bind(waiting)
                    .bind(holding)
                    .fetch_one(owner)
                    .await
                    {
                        Ok(true) => return true,
                        Ok(false) => tokio::time::sleep(std::time::Duration::from_millis(10)).await,
                        Err(_) => return false,
                    }
                }
            })
            .await
            .unwrap_or(false)
        }

        async fn joined<T>(mut task: JoinHandle<T>) -> Result<T, String> {
            match tokio::time::timeout(std::time::Duration::from_secs(18), &mut task).await {
                Ok(result) => result.map_err(|error| error.to_string()),
                Err(_) => {
                    task.abort();
                    let _ = task.await;
                    Err("race task exceeded bounded database statement timeout".to_owned())
                }
            }
        }

        fn company_task(
            business: &PgPool,
            subject: &Subject,
        ) -> JoinHandle<Result<bool, PgOrgError>> {
            let business = business.clone();
            let subject = Subject {
                user: subject.user,
                actor: subject.actor,
                current_refresh: subject.current_refresh.clone(),
                now: subject.now,
            };
            tokio::spawn(async move { deactivate(&business, &subject).await })
        }

        async fn insert_security(
            connection: &mut sqlx::PgConnection,
            user: Uuid,
            now: OffsetDateTime,
        ) -> Result<Value, sqlx::Error> {
            sqlx::query_scalar(
                "INSERT INTO public.account_security AS s \
                 (account_id,security_state,security_generation,revision,updated_at,context_generation) \
                 VALUES ($1,'ACTIVE',41,7,$2,11) RETURNING to_jsonb(s)",
            )
            .bind(user)
            .bind(now)
            .fetch_one(connection)
            .await
        }

        fn native_task(pool: &PgPool, subject: &Subject) -> JoinHandle<Result<Value, sqlx::Error>> {
            let pool = pool.clone();
            let user = subject.user;
            let now = subject.now;
            tokio::spawn(async move {
                let mut tx = pool.begin().await?;
                let row = insert_security(&mut tx, user, now).await?;
                tx.commit().await?;
                Ok(row)
            })
        }

        async fn security_fixture(owner: &PgPool, subject: &Subject) -> Value {
            // Derive the exact full JSON row from explicit fixture inputs before
            // racing, including PostgreSQL timestamp precision. Roll it back.
            let mut tx = owner.begin().await.unwrap();
            let row = insert_security(&mut tx, subject.user, subject.now)
                .await
                .unwrap();
            tx.rollback().await.unwrap();
            row
        }

        fn add_security(snapshot: &mut Value, raw: &Value) {
            snapshot["security"] = serde_json::json!({
                "account_id": raw["account_id"],
                "security_state": raw["security_state"],
                "security_generation": raw["security_generation"],
            });
            if snapshot.get("raw_security").is_some() {
                snapshot["raw_security"] = raw.clone();
            }
        }

        async fn assert_audits(owner: &PgPool, user: Uuid, legacy: bool) {
            assert_eq!(audit_count(owner, "user.deactivate", user).await, 1);
            for action in ["auth.passkey.revoke_all", "auth.refresh.revoke_all"] {
                assert_eq!(audit_count(owner, action, user).await, i64::from(legacy));
            }
        }

        #[sqlx::test(migrations = false)]
        async fn immediate_security_fk_waits_for_account_root_update_lock(owner: PgPool) {
            prepare_account_test_database(&owner).await;
            let business = bounded_login(&owner, TestDatabaseLogin::Business).await;
            let auth = bounded_login(&owner, TestDatabaseLogin::Auth).await;
            let subject = subject(&owner, &business, &auth).await;
            let native = race_pool(&owner).await;
            let native_pid = race_pid(&native).await;
            let expected = security_fixture(&owner, &subject).await;
            let mut before = snapshot(&owner, subject.user).await;
            add_security(&mut before, &expected);
            let mut root = owner.begin().await.unwrap();
            let root_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
                .fetch_one(&mut *root)
                .await
                .unwrap();
            let _: Uuid =
                sqlx::query_scalar("SELECT id FROM public.accounts WHERE id=$1 FOR UPDATE")
                    .bind(subject.user)
                    .fetch_one(&mut *root)
                    .await
                    .unwrap();
            let task = native_task(&native, &subject);
            let observed = blocked_by(&owner, native_pid, root_pid).await;
            let release = root.rollback().await;
            let result = joined(task).await;
            native.close().await;
            business.close().await;
            auth.close().await;
            release.unwrap();
            assert!(
                observed,
                "NATIVE_FK_DID_NOT_WAIT_FOR_EXACT_ACCOUNT_ROOT_HOLDER"
            );
            assert_eq!(result.unwrap().unwrap(), expected);
            assert_eq!(snapshot(&owner, subject.user).await, before);
        }

        async fn enrollment_wins(owner: &PgPool, commit: bool) {
            let business = bounded_login(owner, TestDatabaseLogin::Business).await;
            let auth = bounded_login(owner, TestDatabaseLogin::Auth).await;
            let subject = subject(owner, &business, &auth).await;
            let before = snapshot(owner, subject.user).await;
            let mut expected = if commit {
                retained(before.clone())
            } else {
                expected_legacy_custody(owner, &before, &subject).await
            };
            let business_pid = race_pid(&business).await;
            let mut enrollment = owner.begin().await.unwrap();
            let enrollment_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
                .fetch_one(&mut *enrollment)
                .await
                .unwrap();
            let raw = insert_security(&mut enrollment, subject.user, subject.now)
                .await
                .unwrap();
            if commit {
                add_security(&mut expected, &raw);
            }
            let task = company_task(&business, &subject);
            let observed = blocked_by(owner, business_pid, enrollment_pid).await;
            let release = if commit {
                enrollment.commit().await
            } else {
                enrollment.rollback().await
            };
            let result = joined(task).await;
            business.close().await;
            auth.close().await;
            release.unwrap();
            assert!(
                observed,
                "COMPANY_DID_NOT_WAIT_FOR_EXACT_ENROLLMENT_TRANSACTION"
            );
            assert!(!result.unwrap().unwrap());
            let after = snapshot(owner, subject.user).await;
            assert_eq!(after["company_user"]["is_active"], false);
            assert_eq!(
                retained(after),
                expected,
                "POST_WAIT_CLASSIFICATION_OR_LEGACY_HISTORY_WRONG"
            );
            assert_audits(owner, subject.user, !commit).await;
        }

        #[sqlx::test(migrations = false)]
        async fn enrollment_commit_winner_uses_fresh_fenced_snapshot(owner: PgPool) {
            prepare_account_test_database(&owner).await;
            enrollment_wins(&owner, true).await;
        }

        #[sqlx::test(migrations = false)]
        async fn enrollment_rollback_winner_selects_exact_legacy_sweep(owner: PgPool) {
            prepare_account_test_database(&owner).await;
            enrollment_wins(&owner, false).await;
        }

        async fn credential_nowait(owner: &PgPool, user: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
            // A separate owner transaction observes the exact fixture key. While
            // the real sweep has DELETEd it but is waiting on the family row,
            // FOR UPDATE NOWAIT must report PostgreSQL lock_not_available.
            let mut probe = owner.begin().await?;
            sqlx::query("SET LOCAL statement_timeout='2s'")
                .execute(&mut *probe)
                .await?;
            let result = sqlx::query_scalar(
                "SELECT id FROM public.auth_webauthn_credentials \
                 WHERE user_id=$1 ORDER BY id FOR UPDATE NOWAIT",
            )
            .bind(user)
            .fetch_all(&mut *probe)
            .await;
            probe.rollback().await?;
            result
        }

        async fn company_wins(owner: &PgPool, kill_business: bool) {
            let business = bounded_login(owner, TestDatabaseLogin::Business).await;
            let auth = bounded_login(owner, TestDatabaseLogin::Auth).await;
            let subject = subject(owner, &business, &auth).await;
            let before = snapshot(owner, subject.user).await;
            let expected_key: Uuid =
                serde_json::from_value(before["keys"][0]["id"].clone()).unwrap();
            assert_eq!(
                credential_nowait(owner, subject.user).await.unwrap(),
                vec![expected_key],
                "credential NOWAIT positive must see the exact unlocked fixture key"
            );
            let raw = security_fixture(owner, &subject).await;
            let mut expected = if kill_business {
                before.clone()
            } else {
                expected_legacy_custody(owner, &before, &subject).await
            };
            add_security(&mut expected, &raw);
            let native = race_pool(owner).await;
            let native_pid = race_pid(&native).await;
            let business_pid = race_pid(&business).await;
            let mut families = owner.begin().await.unwrap();
            let families_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
                .fetch_one(&mut *families)
                .await
                .unwrap();
            let _: Uuid = sqlx::query_scalar(
                "SELECT id FROM public.auth_refresh_token_families \
                 WHERE user_id=$1 AND revoked_at IS NULL FOR UPDATE",
            )
            .bind(subject.user)
            .fetch_one(&mut *families)
            .await
            .unwrap();
            let company = company_task(&business, &subject);
            // The real sweep DELETEs keys before attempting this family UPDATE.
            let company_waited = blocked_by(owner, business_pid, families_pid).await;
            // Capture the result without asserting until blockers/tasks are
            // released and joined. A missing key or unrelated error cannot prove
            // the real credential DELETE is still held by this transaction.
            let credential_wait = credential_nowait(owner, subject.user).await;
            let enrollment = native_task(&native, &subject);
            let enrollment_waited = blocked_by(owner, native_pid, business_pid).await;
            let terminated = if kill_business {
                sqlx::query_scalar::<_, bool>("SELECT pg_catalog.pg_terminate_backend($1,5000)")
                    .bind(business_pid)
                    .fetch_one(owner)
                    .await
            } else {
                Ok(false)
            };
            let release = families.rollback().await;
            let company_result = joined(company).await;
            let native_result = joined(enrollment).await;
            native.close().await;
            business.close().await;
            auth.close().await;
            release.unwrap();
            assert!(
                company_waited,
                "COMPANY_DID_NOT_REACH_REAL_PARTIAL_CREDENTIAL_SWEEP"
            );
            assert!(
                enrollment_waited,
                "NATIVE_FK_DID_NOT_WAIT_FOR_EXACT_BUSINESS_TRANSACTION"
            );
            let credential_error = credential_wait
                .expect_err("in-flight credential DELETE must refuse the exact NOWAIT probe");
            assert_eq!(
                credential_error
                    .as_database_error()
                    .and_then(|error| error.code())
                    .as_deref(),
                Some("55P03"),
                "PARTIAL_CREDENTIAL_DELETE_REQUIRES_ACTUAL_NOWAIT_55P03"
            );
            assert_eq!(native_result.unwrap().unwrap(), raw);
            let company_result = company_result.unwrap();
            let after = snapshot(owner, subject.user).await;
            if kill_business {
                assert!(
                    terminated.unwrap(),
                    "exact Business backend termination must complete"
                );
                assert!(
                    company_result.is_err(),
                    "dead Business transaction must not report success"
                );
                let death_error = company_result.as_ref().unwrap_err();
                let death_code = match death_error {
                    PgOrgError::Db(console_platform_db::DbError::Sqlx(error)) => {
                        error.as_database_error().and_then(|error| error.code())
                    }
                    _ => None,
                };
                assert_eq!(
                    death_code.as_deref(),
                    Some("57P01"),
                    "BUSINESS_BACKEND_DEATH_REQUIRES_ACTUAL_ADMIN_SHUTDOWN_57P01"
                );
                assert_eq!(
                    credential_nowait(owner, subject.user).await.unwrap(),
                    vec![expected_key],
                    "backend death must restore the exact key and release its row lock"
                );
                assert_eq!(
                    after, expected,
                    "BACKEND_DEATH_MUST_ROLL_BACK_COMPANY_CREDENTIAL_AND_AUDIT_EFFECTS"
                );
            } else {
                assert!(!company_result.unwrap());
                assert!(
                    credential_nowait(owner, subject.user)
                        .await
                        .unwrap()
                        .is_empty(),
                    "committed legacy sweep must leave no credential row or retained lock"
                );
                assert_eq!(after["company_user"]["is_active"], false);
                assert_eq!(
                    retained(after),
                    expected,
                    "LATER_NATIVE_FIXTURE_MUST_NOT_CHANGE_COMPLETED_LEGACY_HISTORY"
                );
                assert_audits(owner, subject.user, true).await;
            }
        }

        #[sqlx::test(migrations = false)]
        async fn company_commit_holds_native_fk_until_legacy_sweep_commits(owner: PgPool) {
            prepare_account_test_database(&owner).await;
            company_wins(&owner, false).await;
        }

        #[sqlx::test(migrations = false)]
        async fn business_backend_death_rolls_back_partial_company_and_credential_effects(
            owner: PgPool,
        ) {
            prepare_account_test_database(&owner).await;
            company_wins(&owner, true).await;
        }

        #[sqlx::test(migrations = false)]
        async fn repeatable_read_and_serializable_refuse_without_effects(owner: PgPool) {
            prepare_account_test_database(&owner).await;
            for isolation in ["repeatable read", "serializable"] {
                let business = bounded_login(&owner, TestDatabaseLogin::Business).await;
                let auth = bounded_login(&owner, TestDatabaseLogin::Auth).await;
                let subject = subject(&owner, &business, &auth).await;
                let before = snapshot(&owner, subject.user).await;
                let expected = expected_legacy_custody(&owner, &before, &subject).await;
                sqlx::query("SELECT set_config('default_transaction_isolation',$1,false)")
                    .bind(isolation)
                    .execute(&business)
                    .await
                    .unwrap();
                let actual: String = sqlx::query_scalar("SHOW transaction_isolation")
                    .fetch_one(&business)
                    .await
                    .unwrap();
                assert_eq!(
                    actual, isolation,
                    "real session must use the unsupported snapshot mode"
                );
                let refused = deactivate(&business, &subject).await;
                sqlx::query("SET default_transaction_isolation='read committed'")
                    .execute(&business)
                    .await
                    .unwrap();
                assert!(
                    refused.is_err(),
                    "UNSUPPORTED_ISOLATION_MUST_REFUSE_LEGACY_DECISION: {isolation}"
                );
                assert_eq!(snapshot(&owner, subject.user).await, before);
                assert!(!deactivate(&business, &subject).await.unwrap());
                assert_eq!(retained(snapshot(&owner, subject.user).await), expected);
                assert_audits(&owner, subject.user, true).await;
                business.close().await;
                auth.close().await;
            }
        }

        async fn guard(
            pool: &PgPool,
            armed_company: Option<Uuid>,
            company: Option<Uuid>,
            user: Option<Uuid>,
        ) -> Result<bool, sqlx::Error> {
            let mut tx = pool.begin().await?;
            sqlx::query("SELECT set_config('app.current_org',$1,true)")
                .bind(
                    armed_company
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                )
                .execute(&mut *tx)
                .await?;
            let result =
                sqlx::query_scalar("SELECT public.account_company_deactivation_guard_v1($1,$2)")
                    .bind(company)
                    .bind(user)
                    .fetch_one(&mut *tx)
                    .await;
            tx.rollback().await?;
            result
        }

        fn database_refusal(result: Result<bool, sqlx::Error>, label: &str) {
            let error = result.expect_err(label);
            let database = error
                .as_database_error()
                .expect("refusal must come from PostgreSQL");
            assert_ne!(
                database.code().as_deref(),
                Some("42883"),
                "missing guard is not refusal"
            );
        }

        #[sqlx::test(migrations = false)]
        async fn guard_requires_company_subject_context_and_business_only_execution(owner: PgPool) {
            prepare_account_test_database(&owner).await;
            let business = bounded_login(&owner, TestDatabaseLogin::Business).await;
            let auth = bounded_login(&owner, TestDatabaseLogin::Auth).await;
            let subject = subject(&owner, &business, &auth).await;
            let knl = Some(*OrgId::knl().as_uuid());
            let foreign_org = Uuid::new_v4();
            sqlx::query("INSERT INTO public.organizations(id,slug,name) VALUES($1,$2,'Guard foreign company')")
                .bind(foreign_org)
                .bind(format!("g-{foreign_org}"))
                .execute(&owner)
                .await
                .unwrap();
            let foreign_user: Uuid = sqlx::query_scalar(
                "INSERT INTO public.users(org_id,display_name,roles,is_active) \
                 VALUES($1,'Guard foreign user',ARRAY['MECHANIC'],true) RETURNING id",
            )
            .bind(foreign_org)
            .fetch_one(&owner)
            .await
            .unwrap();
            let native_only = Uuid::new_v4();
            sqlx::query("INSERT INTO public.accounts(id,created_at) VALUES($1,$2)")
                .bind(native_only)
                .bind(subject.now)
                .execute(&owner)
                .await
                .unwrap();
            let observed = [subject.user, foreign_user, native_only];
            let mut before = Vec::new();
            for user in observed {
                before.push(snapshot(&owner, user).await);
            }
            assert!(
                guard(&business, knl, knl, Some(subject.user))
                    .await
                    .unwrap()
            );
            for (label, armed, company, user) in [
                ("cross-company subject", knl, knl, Some(foreign_user)),
                (
                    "unarmed foreign company",
                    knl,
                    Some(foreign_org),
                    Some(foreign_user),
                ),
                ("wrong GUC", Some(foreign_org), knl, Some(subject.user)),
                ("missing GUC", None, knl, Some(subject.user)),
                ("native-only root", knl, knl, Some(native_only)),
                ("missing identity", knl, knl, Some(Uuid::new_v4())),
                ("NULL company", knl, None, Some(subject.user)),
                ("NULL subject", knl, knl, None),
            ] {
                database_refusal(guard(&business, armed, company, user).await, label);
            }
            for login in [
                TestDatabaseLogin::Auth,
                TestDatabaseLogin::LeaveCommand,
                TestDatabaseLogin::OntologyCommand,
                TestDatabaseLogin::PlatformForceCommand,
            ] {
                let denied = bounded_login(&owner, login).await;
                let result = guard(&denied, knl, knl, Some(subject.user)).await;
                denied.close().await;
                let error = result.expect_err("unsupported real LOGIN must lack guard EXECUTE");
                assert_eq!(
                    error.as_database_error().and_then(|e| e.code()).as_deref(),
                    Some("42501")
                );
            }
            let denied =
                sqlx::query_scalar::<_, bool>("SELECT public.account_legacy_fenced_v1($1)")
                    .bind(subject.user)
                    .fetch_one(&business)
                    .await
                    .unwrap_err();
            assert_eq!(
                denied.as_database_error().and_then(|e| e.code()).as_deref(),
                Some("42501")
            );
            for (index, user) in observed.into_iter().enumerate() {
                assert_eq!(snapshot(&owner, user).await, before[index]);
            }
            fence(&owner, &subject, "ACTIVE").await;
            let fenced = snapshot(&owner, subject.user).await;
            assert!(
                !guard(&business, knl, knl, Some(subject.user))
                    .await
                    .unwrap()
            );
            assert_eq!(snapshot(&owner, subject.user).await, fenced);
            business.close().await;
            auth.close().await;
        }

        async fn catalog(owner: &PgPool) -> Value {
            let mut connection = owner.acquire().await.unwrap();
            catalog_connection(&mut connection).await
        }

        async fn catalog_connection(connection: &mut sqlx::PgConnection) -> Value {
            sqlx::query_scalar(r#"
                SELECT jsonb_build_object(
                  'routines',(SELECT jsonb_agg(jsonb_build_array(p.proname,
                    pg_get_function_identity_arguments(p.oid),pg_get_functiondef(p.oid),
                    (SELECT jsonb_agg(jsonb_build_array(a.grantor,a.grantee,a.privilege_type,a.is_grantable)
                      ORDER BY a.grantor,a.grantee,a.privilege_type,a.is_grantable)
                     FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a)) ORDER BY p.proname,p.oid)
                    FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
                    WHERE n.nspname='public' AND p.proname LIKE 'account\_%' ESCAPE '\'),
                  'relations',(SELECT jsonb_agg(jsonb_build_array(c.relname,c.relowner,c.relrowsecurity,c.relforcerowsecurity,
                    (SELECT jsonb_agg(jsonb_build_array(a.grantor,a.grantee,a.privilege_type,a.is_grantable)
                      ORDER BY a.grantor,a.grantee,a.privilege_type,a.is_grantable)
                     FROM aclexplode(COALESCE(c.relacl,acldefault('r',c.relowner))) a),
                    (SELECT jsonb_agg(jsonb_build_array(att.attname,
                      (SELECT jsonb_agg(jsonb_build_array(a.grantor,a.grantee,a.privilege_type,a.is_grantable)
                        ORDER BY a.grantor,a.grantee,a.privilege_type,a.is_grantable)
                       FROM aclexplode(att.attacl) a)) ORDER BY att.attnum)
                     FROM pg_attribute att WHERE att.attrelid=c.oid AND att.attnum>0 AND NOT att.attisdropped)
                    ) ORDER BY c.relname) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
                    WHERE n.nspname='public' AND c.relkind='r'
                      AND (c.relname='users' OR c.relname LIKE 'account\_%' ESCAPE '\'
                        OR c.relname='accounts' OR c.relname LIKE 'auth\_%' ESCAPE '\')),
                  'roots',(SELECT jsonb_agg(to_jsonb(a) ORDER BY a.id) FROM public.accounts a))
            "#).fetch_one(connection).await.unwrap()
        }

        #[sqlx::test(migrations = false)]
        async fn exact_guard_profile_upgrades_finalized_root_replays_and_refuses_drift(
            owner: PgPool,
        ) {
            prepare_account_test_database(&owner).await;
            let business = bounded_login(&owner, TestDatabaseLogin::Business).await;
            let auth = bounded_login(&owner, TestDatabaseLogin::Auth).await;
            let subject = subject(&owner, &business, &auth).await;
            let knl = Some(*OrgId::knl().as_uuid());
            assert!(
                guard(&business, knl, knl, Some(subject.user))
                    .await
                    .unwrap()
            );
            let profile: bool = sqlx::query_scalar(r#"
                SELECT p.proowner='console_account_owner'::regrole AND p.prosecdef
                  AND p.prokind='f'
                  AND p.prolang=(SELECT oid FROM pg_language WHERE lanname='plpgsql')
                  AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
                  AND (SELECT count(*)=1 FROM pg_proc candidate JOIN pg_namespace n ON n.oid=candidate.pronamespace
                    WHERE n.nspname='public' AND candidate.proname='account_company_deactivation_guard_v1')
                  AND p.provolatile='v' AND NOT p.proisstrict AND NOT p.proleakproof
                  AND NOT p.proretset AND p.proparallel='u' AND p.prorettype='boolean'::regtype
                  AND NOT COALESCE('row_security=off'=ANY(p.proconfig),false)
                  AND EXISTS(SELECT 1 FROM pg_roles r WHERE r.oid=p.proowner
                    AND NOT r.rolcanlogin AND NOT r.rolsuper AND NOT r.rolbypassrls
                    AND NOT r.rolcreaterole AND NOT r.rolcreatedb AND NOT r.rolreplication)
                  AND NOT EXISTS(SELECT 1 FROM pg_auth_members m WHERE m.roleid=p.proowner OR m.member=p.proowner)
                  AND NOT EXISTS(SELECT 1 FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a
                    WHERE a.grantee NOT IN ('console_account_owner'::regrole,'console_rt'::regrole)
                      OR a.privilege_type<>'EXECUTE' OR a.is_grantable)
                FROM pg_proc p WHERE p.oid='public.account_company_deactivation_guard_v1(uuid,uuid)'::regprocedure
            "#).fetch_one(&owner).await.unwrap();
            assert!(
                profile,
                "guard must retain the reviewed definer and non-public execution profile"
            );
            let columns: Value = sqlx::query_scalar(r#"
                SELECT jsonb_agg(jsonb_build_array(a.attname,x.privilege_type) ORDER BY a.attname,x.privilege_type)
                FROM pg_attribute a CROSS JOIN LATERAL aclexplode(a.attacl) x
                WHERE a.attrelid='public.users'::regclass AND x.grantee='console_account_owner'::regrole
            "#).fetch_one(&owner).await.unwrap();
            assert_eq!(
                columns,
                serde_json::json!([["id", "SELECT"], ["id", "UPDATE"], ["org_id", "SELECT"]])
            );
            let column_grants: Value = sqlx::query_scalar(r#"
                SELECT jsonb_agg(jsonb_build_array(a.attname,x.privilege_type,x.is_grantable)
                  ORDER BY a.attname,x.privilege_type,x.is_grantable)
                FROM pg_attribute a CROSS JOIN LATERAL aclexplode(a.attacl) x
                WHERE a.attrelid='public.users'::regclass AND x.grantee='console_account_owner'::regrole
            "#).fetch_one(&owner).await.unwrap();
            assert_eq!(
                column_grants,
                serde_json::json!([
                    ["id", "SELECT", false],
                    ["id", "UPDATE", false],
                    ["org_id", "SELECT", false]
                ]),
                "guard owner must have only the exact non-grantable users column rights"
            );
            let exact_users_privileges: bool = sqlx::query_scalar(r#"
                SELECT NOT has_table_privilege('console_account_owner','public.users',
                    'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER')
                  AND NOT EXISTS(SELECT 1 FROM pg_class c
                    CROSS JOIN LATERAL aclexplode(COALESCE(c.relacl,acldefault('r',c.relowner))) x
                    WHERE c.oid='public.users'::regclass
                      AND (x.grantee=0 OR x.grantee='console_account_owner'::regrole))
                  AND (SELECT bool_and(
                    has_column_privilege('console_account_owner','public.users',a.attname,'SELECT')
                      = (a.attname IN ('id','org_id'))
                    AND has_column_privilege('console_account_owner','public.users',a.attname,'UPDATE')
                      = (a.attname='id')
                    AND NOT has_column_privilege('console_account_owner','public.users',a.attname,'INSERT,REFERENCES')
                    AND NOT has_column_privilege('console_account_owner','public.users',a.attname,
                      'SELECT WITH GRANT OPTION,INSERT WITH GRANT OPTION,UPDATE WITH GRANT OPTION,REFERENCES WITH GRANT OPTION'))
                    FROM pg_attribute a WHERE a.attrelid='public.users'::regclass
                      AND a.attnum>0 AND NOT a.attisdropped)
            "#).fetch_one(&owner).await.unwrap();
            assert!(
                exact_users_privileges,
                "guard owner must not inherit broad users table/column or grant-option authority"
            );
            let rows = snapshot(&owner, subject.user).await;
            let installed = catalog(&owner).await;
            finalize_account_custody(&owner).await;
            assert_eq!(
                catalog(&owner).await,
                installed,
                "complete guard replay must be exact no-op"
            );
            assert_eq!(snapshot(&owner, subject.user).await, rows);
            // Recreate only the valid old root profile. Existing roots overlap
            // users, so re-entering historical root backfill must fail this case.
            sqlx::raw_sql(
                "DROP FUNCTION public.account_company_deactivation_guard_v1(uuid,uuid); \
                REVOKE SELECT(id,org_id), UPDATE(id) ON public.users FROM console_account_owner;",
            )
            .execute(&owner)
            .await
            .unwrap();
            finalize_account_custody(&owner).await;
            assert_eq!(
                catalog(&owner).await,
                installed,
                "upgrade must preserve exact root/auth custody and restore only guard profile"
            );
            assert_eq!(snapshot(&owner, subject.user).await, rows);
            assert!(
                guard(&business, knl, knl, Some(subject.user))
                    .await
                    .unwrap()
            );
            // Each exact ACL corruption is scoped to its own transaction.
            // A savepoint lets us observe refusal without discarding the drift
            // fixture; rolling back the outer transaction restores the profile.
            for drift_sql in [
                "GRANT SELECT ON public.users TO console_account_owner",
                "GRANT SELECT(id) ON public.users TO console_account_owner WITH GRANT OPTION",
            ] {
                let mut tx = owner.begin().await.unwrap();
                sqlx::query("SET LOCAL statement_timeout='15s'")
                    .execute(&mut *tx)
                    .await
                    .unwrap();
                sqlx::query(drift_sql).execute(&mut *tx).await.unwrap();
                let drifted = catalog_connection(&mut tx).await;
                sqlx::query("SAVEPOINT before_guard_installer")
                    .execute(&mut *tx)
                    .await
                    .unwrap();
                let refusal = sqlx::raw_sql(sqlx::AssertSqlSafe(account_custody_finalizer_sql()))
                    .execute(&mut *tx)
                    .await;
                sqlx::query("ROLLBACK TO SAVEPOINT before_guard_installer")
                    .execute(&mut *tx)
                    .await
                    .unwrap();
                let retained_drift = catalog_connection(&mut tx).await;
                tx.rollback().await.unwrap();
                let error = refusal.expect_err("installer must refuse widened guard users ACL");
                assert_eq!(
                    error.as_database_error().and_then(|e| e.code()).as_deref(),
                    Some("P0001"),
                    "USERS_GUARD_ACL_DRIFT_MUST_REFUSE: {drift_sql}"
                );
                assert_ne!(
                    drifted, installed,
                    "ACL drift fixture must change the catalog"
                );
                assert_eq!(
                    retained_drift, drifted,
                    "failed installer must not repair ACL drift"
                );
                assert_eq!(
                    catalog(&owner).await,
                    installed,
                    "outer rollback must restore exact catalog"
                );
                assert_eq!(snapshot(&owner, subject.user).await, rows);
            }
            sqlx::query(
                "ALTER FUNCTION public.account_company_deactivation_guard_v1(uuid,uuid) STABLE",
            )
            .execute(&owner)
            .await
            .unwrap();
            let drifted = catalog(&owner).await;
            let mut tx = owner.begin().await.unwrap();
            sqlx::query("SET LOCAL statement_timeout='15s'")
                .execute(&mut *tx)
                .await
                .unwrap();
            let refusal = sqlx::raw_sql(sqlx::AssertSqlSafe(account_custody_finalizer_sql()))
                .execute(&mut *tx)
                .await;
            tx.rollback().await.unwrap();
            let error = refusal.expect_err("installer must refuse existing guard volatility drift");
            assert_eq!(
                error.as_database_error().and_then(|e| e.code()).as_deref(),
                Some("P0001")
            );
            assert_eq!(
                catalog(&owner).await,
                drifted,
                "failed installer must not repair drift"
            );
            assert_eq!(snapshot(&owner, subject.user).await, rows);
            business.close().await;
            auth.close().await;
        }
    }
}
