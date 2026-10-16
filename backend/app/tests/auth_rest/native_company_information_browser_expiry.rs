//! Controlled negative Auth issuance from actual UI A's unchanged live family.
//! Not a browser renewal/login claim: the real configured JwtIssuer owns signing.
use super::super::manager_policy::ObservedRead;
use super::{all_rows, credentials, histories};
use console_app::AppConfig;
use console_identity_adapter_postgres::PgOrgStore;
use console_identity_application::company_policy::{
    CompanyPolicyDecision, CompanyPolicyDecisionPort, CompanyPolicyError, CompanyPolicyRequest,
    CurrentCompanyAuthority, CurrentNativeBootstrapAuthority, CurrentPayrollReadAuthority,
    CurrentPeopleDirectoryAuthority, NativeBootstrapRequestV1, NativePeopleDirectoryRequestV1,
    workflow::{
        NativePolicyCommandRef, NativePolicyFormView, NativePolicyWorkflowError,
        native_policy_current,
    },
};
use console_platform_auth::{
    AccountAccessTokenInput, JwtIssuer, JwtSettings,
    account::{AccountEnrollmentCredentials, ensure_account_session_fresh_in_tx},
};
use sqlx::PgPool;
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use time::OffsetDateTime;
use uuid::Uuid;

async fn issue(
    pool: &PgPool,
    runtime: &PgPool,
    config: &AppConfig,
    original: &AccountEnrollmentCredentials,
    authority: &CurrentCompanyAuthority,
) -> (AccountEnrollmentCredentials, OffsetDateTime) {
    let (verifier, _, ttl) = credentials::bindings(config);
    let mut tx = runtime.begin().await.unwrap();
    sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
        .execute(tx.as_mut())
        .await
        .unwrap();
    let session = original
        .read_session_in_tx(&mut tx, &verifier, ttl)
        .await
        .unwrap();
    assert_eq!(session.account_id, *authority.account().as_uuid());
    ensure_account_session_fresh_in_tx(&mut tx, &session)
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    // Reuse the accepted integer-second calibration: 650-750ms remains inside
    // the actual SQL source's fixed1s lock timeout, not an invented expiry clock.
    let at = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let at: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                .fetch_one(pool)
                .await
                .unwrap();
            if (250_000_000..=350_000_000).contains(&at.nanosecond()) {
                break at;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("STOP: actual DB deadline calibration unavailable");
    let auth = config.auth_rest.as_ref().unwrap();
    let issuer = JwtIssuer::from_es256_pem(
        JwtSettings {
            issuer: auth.jwt_issuer.clone(),
            audience: auth.jwt_audience.clone(),
            access_token_ttl: time::Duration::seconds(1),
        },
        auth.jwt_private_key_pem.as_bytes(),
        auth.jwt_public_key_pem.as_bytes(),
    )
    .unwrap();
    let signed = issuer
        .issue_account_access_token(AccountAccessTokenInput {
            account_id: session.account_id,
            session_id: session.session_id,
            security_generation: session.security_generation,
            auth_time: session.auth_time,
            assurance: session.assurance,
            issued_at: at,
            family_expires_at: session.family_expires_at,
        })
        .unwrap();
    let deadline = OffsetDateTime::from_unix_timestamp(at.unix_timestamp() + 1).unwrap();
    assert!(
        deadline < session.expires_at && deadline < session.family_expires_at,
        "STOP: negative access must expire before unchanged original authority"
    );
    let short = AccountEnrollmentCredentials::for_read(signed.as_str()).unwrap();
    let mut tx = runtime.begin().await.unwrap();
    sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
        .execute(tx.as_mut())
        .await
        .unwrap();
    assert_eq!(
        short
            .session_ids_in_tx(&mut tx, &verifier, ttl)
            .await
            .unwrap(),
        (session.account_id, session.session_id)
    );
    let live = short
        .read_session_in_tx(&mut tx, &verifier, ttl)
        .await
        .unwrap();
    assert!(
        live.account_id == session.account_id
            && live.session_id == session.session_id
            && live.security_generation == session.security_generation
            && live.auth_time == session.auth_time
            && live.assurance == session.assurance
            && live.family_expires_at == session.family_expires_at
            && live.expires_at == deadline,
        "STOP: genuine short issuance changed original live family binding"
    );
    ensure_account_session_fresh_in_tx(&mut tx, &live)
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    (short, deadline)
}

async fn positive(
    store: &PgOrgStore,
    credential: &AccountEnrollmentCredentials,
    authority: &CurrentCompanyAuthority,
    selector: NativePolicyCommandRef,
    expected: &NativePolicyFormView,
) {
    let policy = ObservedRead::new(authority.clone(), None);
    let result = native_policy_current(store, &policy, credential, selector).await;
    assert!(
        result.is_ok(),
        "STOP: actual short-issued Manager owner must be healthy before expiry"
    );
    assert_eq!(result.unwrap(), *expected);
    policy.count(2);
}

// The lease owns this isolated database. Observe every genuine console_rt
// backend so a leaked SHARE guard cannot hide behind a healthy recovery read.
pub(super) async fn released(pool: &PgPool) {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let clean: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity WHERE datname=current_database() AND usename='console_rt' AND (xact_start IS NOT NULL OR wait_event_type='Lock')) AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_locks l JOIN pg_catalog.pg_stat_activity a ON a.pid=l.pid WHERE a.datname=current_database() AND a.usename='console_rt' AND l.locktype IN ('relation','transactionid','tuple'))")
                .fetch_one(pool).await.unwrap();
            if clean { break; }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await.expect("STOP: actual Manager expiry transaction/row-lock cleanup not proven");
}

struct SlowDecision {
    real: ObservedRead,
    at: usize,
    calls: AtomicUsize,
    deadline: OffsetDateTime,
}
impl CompanyPolicyDecisionPort for SlowDecision {
    fn decide_native_people_directory(
        &self,
        a: &CurrentPeopleDirectoryAuthority,
        r: &NativePeopleDirectoryRequestV1,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        self.real.decide_native_people_directory(a, r)
    }
    fn decide_native_bootstrap(
        &self,
        a: &CurrentNativeBootstrapAuthority,
        r: &NativeBootstrapRequestV1,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        self.real.decide_native_bootstrap(a, r)
    }
    fn decide_native_payroll_collection(
        &self,
        a: &CurrentPayrollReadAuthority,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        self.real.decide_native_payroll_collection(a)
    }
    fn decide(
        &self,
        authority: &CurrentCompanyAuthority,
        request: &CompanyPolicyRequest,
    ) -> Result<CompanyPolicyDecision, CompanyPolicyError> {
        let result = self.real.decide(authority, request);
        if self.calls.fetch_add(1, Ordering::SeqCst) + 1 == self.at {
            assert!(
                authority.observed_at() < self.deadline,
                "STOP: actual ordinary Cedar source reached only after access expiry"
            );
            std::thread::sleep(Duration::from_secs(1));
        }
        result
    }
}

pub(super) async fn verify(
    pool: &PgPool,
    runtime: &PgPool,
    config: &AppConfig,
    original: &AccountEnrollmentCredentials,
    authority: &CurrentCompanyAuthority,
    selector: NativePolicyCommandRef,
    group: Uuid,
    expected: &NativePolicyFormView,
    before: &BTreeMap<String, String>,
) {
    let store = credentials::store(runtime.clone(), config);
    released(pool).await;
    for on_group in [true, false] {
        let (short, deadline) = issue(pool, runtime, config, original, authority).await;
        positive(&store, &short, authority, selector, expected).await;
        histories::expire(
            pool, runtime, config, &short, original, authority, selector, group, expected, before,
            deadline, on_group,
        )
        .await;
    }
    for (at, deny) in [(1, false), (2, false), (1, true), (2, true)] {
        let (short, deadline) = issue(pool, runtime, config, original, authority).await;
        positive(&store, &short, authority, selector, expected).await;
        let policy = SlowDecision {
            real: ObservedRead::new(authority.clone(), deny.then_some((at, false))),
            at,
            calls: AtomicUsize::new(0),
            deadline,
        };
        let started: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(pool)
            .await
            .unwrap();
        assert!(
            started < deadline,
            "STOP: delayed-decision invocation began after access expiry"
        );
        let result = tokio::time::timeout(
            Duration::from_secs(5),
            native_policy_current(&store, &policy, &short, selector),
        )
        .await
        .unwrap();
        let after: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(pool)
            .await
            .unwrap();
        assert!(
            after >= deadline,
            "STOP: actual DB time did not cross access expiry"
        );
        assert!(
            matches!(
                result,
                Err(NativePolicyWorkflowError::AuthenticationInvalid)
            ),
            "post-ordinary-decision access expiry bypassed final original Auth/freshness"
        );
        policy.real.count(at);
        released(pool).await;
        assert!(
            *before == all_rows(pool).await,
            "short-access refusal changed complete public census"
        );
        positive(&store, original, authority, selector, expected).await;
        released(pool).await;
        assert!(
            *before == all_rows(pool).await,
            "original browser recovery changed census"
        );
    }
}
