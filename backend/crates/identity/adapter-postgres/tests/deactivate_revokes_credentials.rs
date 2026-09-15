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
}
