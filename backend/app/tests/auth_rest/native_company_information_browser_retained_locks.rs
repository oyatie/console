//! Actual Manager source locks retained through its first real ordinary Cedar.
//! SELECT locks only: no business writer or blocked final-reread claim.
use super::super::manager_policy::ObservedRead;
use super::{all_rows, credentials, expiry};
use console_app::AppConfig;
use console_identity_application::company_policy::{
    CompanyPolicyDecision, CompanyPolicyDecisionPort, CompanyPolicyError, CompanyPolicyRequest,
    CurrentCompanyAuthority, CurrentNativeBootstrapAuthority, CurrentPayrollReadAuthority,
    CurrentPeopleDirectoryAuthority, NativeBootstrapRequestV1, NativePeopleDirectoryRequestV1,
    workflow::{NativePolicyCommandRef, NativePolicyFormView, native_policy_current},
};
use console_platform_auth::account::{
    AccountEnrollmentCredentials, ensure_account_session_fresh_in_tx,
};
use futures::FutureExt;
use sqlx::{
    Connection, PgConnection, PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use uuid::Uuid;

#[path = "native_company_information_browser_lock_rows.rs"]
mod rows;
use rows::{Probe, probes};

async fn reader_xid(observer: &mut PgConnection, pid: i32) -> String {
    sqlx::query_scalar("SELECT backend_xid::text FROM pg_catalog.pg_stat_activity WHERE pid=$1 AND datname=current_database() AND usename='console_rt' AND xact_start IS NOT NULL AND backend_xid IS NOT NULL AND state='idle in transaction'")
        .bind(pid).fetch_one(observer).await.expect("STOP: exact actual Manager reader transaction absent")
}

async fn observe(
    options: PgConnectOptions,
    reader: Option<i32>,
    probes: &[Probe],
    observer_pids: &Arc<Mutex<Vec<i32>>>,
) {
    let mut observer =
        tokio::time::timeout(Duration::from_secs(3), PgConnection::connect_with(&options))
            .await
            .unwrap()
            .unwrap();
    let outcome = std::panic::AssertUnwindSafe(async {
        let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()").fetch_one(&mut observer).await.unwrap();
        observer_pids.lock().unwrap().push(pid);
        let marked: bool = sqlx::query_scalar("SELECT session_user=current_user AND current_user='console_buck_admin' AND starts_with(current_database(),'_sqlx_test_') AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user)").fetch_one(&mut observer).await.unwrap();
        assert!(marked,"STOP: root-marked disposable lock observer required");
        let xid = if let Some(reader) = reader { Some(reader_xid(&mut observer,reader).await) } else { None };
        for probe in probes {
            sqlx::raw_sql("BEGIN ISOLATION LEVEL READ COMMITTED; SET LOCAL statement_timeout='1s'; SET LOCAL lock_timeout='500ms'; SET LOCAL transaction_timeout='5s'").execute(&mut observer).await.unwrap();
            let mut query = sqlx::query(sqlx::AssertSqlSafe(probe.sql));
            for id in &probe.ids { query=query.bind(*id); }
            let result = query.fetch_all(&mut observer).await;
            sqlx::raw_sql("ROLLBACK").execute(&mut observer).await.unwrap();
            if reader.is_some() {
                let error = result.unwrap_err();
                assert_eq!(error.as_database_error().unwrap().code().as_deref(),Some("55P03"),"{}: actual retained source lock absent",probe.label);
            } else {
                assert_eq!(result.unwrap().len(),1,"{}: exact existing source row calibration",probe.label);
            }
            if let Some(reader) = reader { assert_eq!(reader_xid(&mut observer,reader).await,xid.as_ref().unwrap().as_str()); }
        }
    }).catch_unwind();
    let outcome = tokio::time::timeout(Duration::from_secs(5), outcome).await;
    let rollback = tokio::time::timeout(
        Duration::from_secs(1),
        sqlx::raw_sql("ROLLBACK").execute(&mut observer),
    )
    .await;
    let closed = tokio::time::timeout(Duration::from_secs(1), observer.close()).await;
    assert!(
        matches!(rollback, Ok(Ok(_))) && matches!(closed, Ok(Ok(()))),
        "STOP: owned lock-observer transaction/socket cleanup"
    );
    if let Err(panic) = outcome.expect("STOP: finite observer envelope exceeded") {
        std::panic::resume_unwind(panic);
    }
}

struct HeldAtFirstDecision {
    real: ObservedRead,
    calls: AtomicUsize,
    options: PgConnectOptions,
    reader: i32,
    probes: Vec<Probe>,
    observer_pids: Arc<Mutex<Vec<i32>>>,
}
impl CompanyPolicyDecisionPort for HeldAtFirstDecision {
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
        if self.calls.fetch_add(1, Ordering::SeqCst) == 0 {
            let (options, reader, probes, pids) = (
                self.options.clone(),
                self.reader,
                self.probes.clone(),
                self.observer_pids.clone(),
            );
            // An owned thread avoids nested Tokio block_on. Its isolated network
            // work has explicit timeouts and joins before this decision returns.
            std::thread::spawn(move || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(observe(options, Some(reader), &probes, &pids))
            })
            .join()
            .expect("STOP: owned observer thread panicked");
        }
        result
    }
}

async fn absent(pool: &PgPool, pids: Vec<i32>) {
    tokio::time::timeout(Duration::from_secs(5),async {
        loop {
            let clean:bool=sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity WHERE pid=ANY($1)) AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_locks WHERE pid=ANY($1))")
                .bind(&pids).fetch_one(pool).await.unwrap();
            if clean { break; }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await.expect("STOP: owned reader/observer backend or locks remain");
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
    expiry::released(pool).await;
    // Preserve the actual restricted login/TLS/options. max1 fixes connection
    // identity; min0 permits no background/preconnected replacement.
    let owned = tokio::time::timeout(
        Duration::from_secs(5),
        PgPoolOptions::new()
            .max_connections(1)
            .min_connections(0)
            .acquire_timeout(Duration::from_secs(3))
            .connect_with(runtime.connect_options().as_ref().clone()),
    )
    .await
    .unwrap()
    .unwrap();
    let pids = Arc::new(Mutex::new(Vec::new()));
    let outcome=tokio::time::timeout(Duration::from_secs(20),std::panic::AssertUnwindSafe(async {
        let identity:(String,String,i32,bool,bool)=sqlx::query_as("SELECT session_user::text,current_user::text,pg_backend_pid(),rolsuper,rolbypassrls FROM pg_catalog.pg_roles WHERE rolname=current_user")
            .fetch_one(&owned).await.unwrap();
        pids.lock().unwrap().push(identity.2);
        assert!(identity.0=="console_rt" && identity.1=="console_rt" && !identity.3 && !identity.4,"STOP: genuine restricted retained-lock reader");
        let (verifier,_,ttl)=credentials::bindings(config);
        let mut tx=owned.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED").execute(tx.as_mut()).await.unwrap();
        let session=original.read_session_in_tx(&mut tx,&verifier,ttl).await.unwrap();
        assert_eq!(session.account_id,*authority.account().as_uuid());
        ensure_account_session_fresh_in_tx(&mut tx,&session).await.unwrap();
        tx.rollback().await.unwrap();
        let probes=probes(pool,*selector.company().as_uuid(),group,session.account_id,session.session_id,authority).await;
        expiry::released(pool).await;
        observe(pool.connect_options().as_ref().clone(),None,&probes,&pids).await;
        assert!(*before==all_rows(pool).await,"no-lock calibration changed public census");
        let policy=HeldAtFirstDecision { real:ObservedRead::new(authority.clone(),None),calls:AtomicUsize::new(0),options:pool.connect_options().as_ref().clone(),reader:identity.2,probes,observer_pids:pids.clone() };
        let result=tokio::time::timeout(Duration::from_secs(12),native_policy_current(&credentials::store(owned.clone(),config),&policy,original,selector)).await.unwrap();
        assert!(result.is_ok(),"actual retained-lock Manager read refused healthy source");
        assert_eq!(result.unwrap(),*expected);
        policy.real.count(2);assert_eq!(policy.calls.load(Ordering::SeqCst),2);
        expiry::released(pool).await;
        assert!(*before==all_rows(pool).await,"retained-lock Manager read changed public census");
    }).catch_unwind()).await;
    let closed = tokio::time::timeout(Duration::from_secs(5), owned.close()).await;
    let cleanup = std::panic::AssertUnwindSafe(async {
        let owned_pids = pids.lock().unwrap().clone();
        absent(pool, owned_pids).await;
        expiry::released(pool).await;
        assert!(
            *before == all_rows(pool).await,
            "retained-lock owned shutdown changed public census"
        );
    })
    .catch_unwind()
    .await;
    assert!(
        closed.is_ok(),
        "STOP: owned retained-lock runtime pool closure"
    );
    if let Err(panic) = cleanup {
        std::panic::resume_unwind(panic);
    }
    if let Err(panic) = outcome.expect("STOP: bounded actual retained-lock owner envelope") {
        std::panic::resume_unwind(panic);
    }
    let recovery = ObservedRead::new(authority.clone(), None);
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        native_policy_current(
            &credentials::store(runtime.clone(), config),
            &recovery,
            original,
            selector,
        ),
    )
    .await
    .unwrap();
    assert_eq!(result.unwrap(), *expected);
    recovery.count(2);
    expiry::released(pool).await;
    assert!(
        *before == all_rows(pool).await,
        "original-runtime retained-lock recovery changed public census"
    );
}
