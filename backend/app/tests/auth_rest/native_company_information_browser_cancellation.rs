//! Drop the actual retained Manager read after a real ordinary Cedar decision.
//! No claim of a paused final SQL query; original UI credentials remain unchanged.
use super::super::manager_policy::ObservedRead;
use super::{all_rows, credentials, expiry};
use console_app::AppConfig;
use console_identity_application::company_policy::{
    CompanyPolicyDecision, CompanyPolicyDecisionPort, CompanyPolicyError, CompanyPolicyRequest,
    CurrentCompanyAuthority, CurrentNativeBootstrapAuthority, CurrentPayrollReadAuthority,
    CurrentPeopleDirectoryAuthority, NativeBootstrapRequestV1, NativePeopleDirectoryRequestV1,
    workflow::{NativePolicyCommandRef, NativePolicyFormView, native_policy_current},
};
use console_platform_auth::account::AccountEnrollmentCredentials;
use futures::FutureExt;
use sqlx::PgPool;
use std::{
    collections::BTreeMap,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::sync::oneshot;

struct CancelAfterDecision {
    real: ObservedRead,
    at: usize,
    calls: AtomicUsize,
    cancel: Mutex<Option<oneshot::Sender<()>>>,
}

impl CompanyPolicyDecisionPort for CancelAfterDecision {
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
            self.cancel
                .lock()
                .unwrap()
                .take()
                .unwrap()
                .send(())
                .unwrap();
        }
        result
    }
}

async fn positive(
    pool: &PgPool,
    runtime: &PgPool,
    config: &AppConfig,
    original: &AccountEnrollmentCredentials,
    authority: &CurrentCompanyAuthority,
    selector: NativePolicyCommandRef,
    expected: &NativePolicyFormView,
    before: &BTreeMap<String, String>,
) {
    let policy = ObservedRead::new(authority.clone(), None);
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        native_policy_current(
            &credentials::store(runtime.clone(), config),
            &policy,
            original,
            selector,
        ),
    )
    .await
    .expect("STOP: bounded adjacent actual Manager cancellation positive");
    assert!(result.is_ok(), "STOP: actual Manager cancellation positive");
    assert_eq!(result.unwrap(), *expected);
    policy.count(2);
    expiry::released(pool).await;
    assert!(
        *before == all_rows(pool).await,
        "adjacent actual Manager cancellation positive changed public census"
    );
}

pub(super) async fn verify(
    pool: &PgPool,
    runtime: &PgPool,
    config: &AppConfig,
    original: &AccountEnrollmentCredentials,
    authority: &CurrentCompanyAuthority,
    selector: NativePolicyCommandRef,
    expected: &NativePolicyFormView,
    before: &BTreeMap<String, String>,
) {
    expiry::released(pool).await;
    for at in [1, 2] {
        positive(
            pool, runtime, config, original, authority, selector, expected, before,
        )
        .await;
        let store = credentials::store(runtime.clone(), config);
        let (cancel, cancelled) = oneshot::channel();
        let policy = CancelAfterDecision {
            real: ObservedRead::new(authority.clone(), None),
            at,
            calls: AtomicUsize::new(0),
            cancel: Mutex::new(Some(cancel)),
        };
        let outcome = std::panic::AssertUnwindSafe(async {
            let result = tokio::time::timeout(Duration::from_secs(5), async {
                let future = native_policy_current(&store, &policy, original, selector);
                tokio::pin!(future);
                tokio::select! {
                    biased;
                    signal = cancelled => {
                        signal.expect("STOP: actual Cedar cancellation signal was not sent");
                        None
                    },
                    result = &mut future => Some(result),
                }
                // The pinned actual owner future drops at the end of this scope,
                // including on timeout/panic, before cleanup starts below.
            })
            .await
            .expect("STOP: actual post-Cedar Manager cancellation timed out");
            assert!(
                result.is_none(),
                "post-Cedar cancelled actual owner released a result"
            );
            policy.real.count(at);
            assert_eq!(policy.calls.load(Ordering::SeqCst), at);
            assert!(policy.cancel.lock().unwrap().is_none());
        })
        .catch_unwind()
        .await;
        // Preserve the existing leased-database all-console_rt guard/transaction
        // oracle. Attempt cleanup before surfacing an owner/count assertion.
        let cleanup = std::panic::AssertUnwindSafe(async {
            expiry::released(pool).await;
            assert!(
                *before == all_rows(pool).await,
                "post-Cedar Manager cancellation changed complete public census"
            );
        })
        .catch_unwind()
        .await;
        if let Err(panic) = cleanup {
            std::panic::resume_unwind(panic);
        }
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
        positive(
            pool, runtime, config, original, authority, selector, expected, before,
        )
        .await;
    }
}
