//! One actual Manager read COMMIT acknowledgement loss through the retained relay.
//! Plaintext exists only on an owned disposable loopback fixture connection.
use super::super::manager_policy::ObservedRead;
use super::{all_rows, credentials};
use crate::legacy_platform_list_reads::{WireEvidence, evidence, proven_ack_loss, relay};
use console_app::AppConfig;
use console_identity_application::company_policy::{
    CurrentCompanyAuthority,
    workflow::{
        NativePolicyCommandRef, NativePolicyFormView, NativePolicyWorkflowError,
        native_policy_current,
    },
};
use console_platform_auth::account::AccountEnrollmentCredentials;
use console_platform_test_support::{TestDatabaseLogin, login_test_database_url};
use futures::FutureExt;
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{net::TcpListener, sync::oneshot};

async fn absent(pool: &PgPool, pid: i32) {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let clean: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity WHERE pid=$1) AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_locks WHERE pid=$1) AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity WHERE datname=current_database() AND $1=ANY(pg_catalog.pg_blocking_pids(pid)))")
                .bind(pid).fetch_one(pool).await.unwrap();
            if clean { break; }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await.expect("STOP: owned Manager wire backend/locks/dependent waiters remain");
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
    let options: PgConnectOptions = login_test_database_url(pool, TestDatabaseLogin::Business)
        .parse()
        .unwrap_or_else(|_| panic!("STOP: invalid isolated Business test transport"));
    assert!(
        options.get_socket().is_none()
            && matches!(options.get_host(), "127.0.0.1" | "::1" | "localhost"),
        "STOP: finite wire fixture requires disposable loopback TCP"
    );
    let upstream = (options.get_host().to_owned(), options.get_port());
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let local = listener.local_addr().unwrap();
    let armed = Arc::new(AtomicBool::new(false));
    let transcript = Arc::new(Mutex::new(WireEvidence::default()));
    let (stop, stopped) = oneshot::channel();
    let mut task = tokio::spawn(relay(
        listener,
        upstream,
        armed.clone(),
        transcript.clone(),
        stopped,
    ));
    // Native pool setup only. Authentication is observed inside the caught scope;
    // min0/max1 prevents a hidden second connection or preconnected replacement.
    let proxy = PgPoolOptions::new()
        .max_connections(1)
        .min_connections(0)
        .acquire_timeout(Duration::from_secs(3))
        .connect_lazy_with(
            options
                .host("127.0.0.1")
                .port(local.port())
                .ssl_mode(PgSslMode::Disable),
        );
    let mut actual_pid = None;
    let outcome=std::panic::AssertUnwindSafe(async {
        let identity:(String,String,i32,bool,bool)=tokio::time::timeout(Duration::from_secs(5),
            sqlx::query_as("SELECT session_user::text,current_user::text,pg_backend_pid(),rolsuper,rolbypassrls FROM pg_catalog.pg_roles WHERE rolname=current_user")
                .fetch_one(&proxy)).await.unwrap().unwrap();
        actual_pid=Some(identity.2);
        assert!(identity.0=="console_rt" && identity.1=="console_rt" && !identity.3 && !identity.4,
                "STOP: real restricted Manager LOGIN through transparent fixture relay");
        let store=credentials::store(proxy.clone(),config);
        let healthy=ObservedRead::new(authority.clone(),None);
        let result=tokio::time::timeout(Duration::from_secs(12),
            native_policy_current(&store,&healthy,original,selector)).await.unwrap();
        assert!(result.is_ok(),"STOP: actual Manager owner transparent-COMMIT prerequisite");
        assert_eq!(result.unwrap(),*expected);
        healthy.count(2);
        assert!(*before==all_rows(pool).await,"transparent Manager COMMIT changed public census");
        let first=evidence(&transcript);
        assert!(first.connections==1 && first.backend_pid==Some(identity.2)
                && first.unarmed_commits==1 && first.armed_commits==0
                && first.forwarded_commits==1 && first.positive_commit_ack==1 && !first.backend_error,
                "STOP: actual transparent single-COMMIT positive control");
        let negative=ObservedRead::new(authority.clone(),None);
        armed.store(true,Ordering::SeqCst);
        let result=tokio::time::timeout(Duration::from_secs(12),
            native_policy_current(&store,&negative,original,selector)).await;
        assert!(matches!(result,Ok(Err(NativePolicyWorkflowError::Unavailable))),
                "unknown actual Manager COMMIT completion released a view or wrong failure");
        negative.count(2);
        assert!(proven_ack_loss(&evidence(&transcript)),
                "STOP: exact backend COMMIT+idle success must precede downstream acknowledgement loss");
        absent(pool,identity.2).await;
        assert!(*before==all_rows(pool).await,"lost Manager COMMIT changed complete public census");
    }).catch_unwind().await;
    // Always attempt all owned shutdown before reporting any owner assertion.
    let _ = stop.send(());
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let (closed, joined) = tokio::join!(
        tokio::time::timeout_at(deadline, proxy.close()),
        tokio::time::timeout_at(deadline, &mut task),
    );
    let aborted_join = if joined.is_err() {
        task.abort();
        Some(tokio::time::timeout(Duration::from_secs(1), &mut task).await)
    } else {
        None
    };
    let pid = actual_pid.or_else(|| evidence(&transcript).backend_pid);
    let cleaned = std::panic::AssertUnwindSafe(async {
        if let Some(pid) = pid {
            absent(pool, pid).await;
        }
    })
    .catch_unwind()
    .await;
    assert!(
        closed.is_ok()
            && cleaned.is_ok()
            && (joined.is_ok() || matches!(aborted_join, Some(Ok(_)))),
        "STOP: owned Manager relay pool/listener/task/backend cleanup not proven"
    );
    assert!(
        *before == all_rows(pool).await,
        "wire fixture shutdown changed public census"
    );
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
    assert!(
        matches!(joined, Ok(Ok(Ok(())))),
        "STOP: actual finite relay did not finish cleanly"
    );
    assert!(
        proven_ack_loss(&evidence(&transcript)),
        "wire cleanup concealed a retry or transcript fault"
    );
    // Separate explicit recovery on the original genuine runtime pool/credential.
    let policy = ObservedRead::new(authority.clone(), None);
    let result = tokio::time::timeout(
        Duration::from_secs(12),
        native_policy_current(
            &credentials::store(runtime.clone(), config),
            &policy,
            original,
            selector,
        ),
    )
    .await
    .unwrap();
    assert!(
        result.is_ok(),
        "STOP: original-browser independent Manager recovery"
    );
    assert_eq!(result.unwrap(), *expected);
    policy.count(2);
    assert!(
        *before == all_rows(pool).await,
        "independent Manager wire-loss recovery changed census"
    );
    assert!(
        proven_ack_loss(&evidence(&transcript)),
        "independent recovery used uncertain relay again"
    );
}
