//! Genuine restricted LOGIN plus four transport-only final-source/Auth faults.
use super::super::manager_policy::ObservedRead;
use super::{all_rows, credentials};
use crate::native_pg_test_wire::{invalid_wire, read_frame, read_startup, write_frame};
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
    io::Result,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::oneshot,
};
#[path = "native_company_information_final_controls.rs"]
mod controls;
#[path = "native_company_information_final_wire.rs"]
mod wire;
use wire::{Fault, Grammar};

fn proven(state: &Arc<Mutex<Grammar>>, connections: &Arc<Mutex<usize>>, pid: i32) -> bool {
    let state = state.lock().unwrap();
    state.proven() && state.backend_pid == Some(pid) && *connections.lock().unwrap() == 1
}

async fn upload<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    mut client: R,
    mut server: W,
    state: Arc<Mutex<Grammar>>,
) -> Result<()> {
    loop {
        let (tag, body) = read_frame(&mut client).await?;
        let output = { state.lock().unwrap().upload(tag, &body)? };
        write_frame(&mut server, tag, &output).await?;
        if tag == b'X' {
            return Ok(());
        }
    }
}
async fn download<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    mut server: R,
    mut client: W,
    state: Arc<Mutex<Grammar>>,
) -> Result<()> {
    loop {
        let (tag, body) = read_frame(&mut server).await?;
        let output = { state.lock().unwrap().download(tag, &body)? };
        if let Some(output) = output {
            write_frame(&mut client, tag, &output).await?;
        }
    }
}
async fn connection(
    mut client: TcpStream,
    upstream: (String, u16),
    state: Arc<Mutex<Grammar>>,
) -> Result<()> {
    let mut server = TcpStream::connect(upstream).await?;
    let startup = read_startup(&mut client).await?;
    server.write_u32((startup.len() + 4) as u32).await?;
    server.write_all(&startup).await?;
    server.flush().await?;
    drop(startup);
    let (cr, cw) = client.into_split();
    let (sr, sw) = server.into_split();
    let up = upload(cr, sw, state.clone());
    let down = download(sr, cw, state);
    tokio::pin!(up, down);
    tokio::select! {r=&mut up=>r,r=&mut down=>r}
}
async fn relay(
    listener: TcpListener,
    upstream: (String, u16),
    state: Arc<Mutex<Grammar>>,
    connections: Arc<Mutex<usize>>,
    mut stopped: oneshot::Receiver<()>,
) -> Result<()> {
    let (first, _) =
        tokio::select! {r=listener.accept()=>r?,_=&mut stopped=>return Err(invalid_wire())};
    *connections.lock().unwrap() += 1;
    let connection = connection(first, upstream, state.clone());
    tokio::pin!(connection);
    let mut finished = false;
    loop {
        tokio::select! {
            r=&mut connection,if !finished=>{r?;finished=true;},
            r=listener.accept()=>{let(extra,_)=r?;*connections.lock().unwrap()+=1;drop(extra);return Err(invalid_wire());},
            _=&mut stopped=>return if state.lock().unwrap().proven()&&*connections.lock().unwrap()==1{Ok(())}else{Err(invalid_wire())},
        }
    }
}
async fn absent(pool: &PgPool, pid: i32) {
    tokio::time::timeout(Duration::from_secs(5),async {loop {
        let clean:bool=sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity WHERE pid=$1) AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_locks WHERE pid=$1) AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity WHERE datname=current_database() AND $1=ANY(pg_catalog.pg_blocking_pids(pid)))")
            .bind(pid).fetch_one(pool).await.unwrap();
        if clean{break;}tokio::time::sleep(Duration::from_millis(5)).await;
    }}).await.expect("STOP: owned finalization backend/locks/dependent waiters remain");
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
    let (verifier, _, ttl) = credentials::bindings(config);
    let mut tx = runtime.begin().await.unwrap();
    let (actor, family) = original
        .session_ids_in_tx(&mut tx, &verifier, ttl)
        .await
        .unwrap();
    let live = original
        .read_session_in_tx(&mut tx, &verifier, ttl)
        .await
        .unwrap();
    assert!(
        live.account_id == actor && live.session_id == family,
        "STOP: original signed credential/live family namespace"
    );
    tx.rollback().await.unwrap();
    let ids = [
        actor,
        family,
        *selector.company().as_uuid(),
        selector.command_id(),
    ];
    for fault in [
        Fault::Material,
        Fault::SourceZero,
        Fault::Group,
        Fault::AuthZero,
    ] {
        one(
            pool, runtime, config, original, authority, selector, expected, before, ids, fault,
        )
        .await;
    }
}
#[allow(clippy::too_many_arguments)]
async fn one(
    pool: &PgPool,
    runtime: &PgPool,
    config: &AppConfig,
    original: &AccountEnrollmentCredentials,
    authority: &CurrentCompanyAuthority,
    selector: NativePolicyCommandRef,
    expected: &NativePolicyFormView,
    before: &BTreeMap<String, String>,
    ids: [uuid::Uuid; 4],
    fault: Fault,
) {
    let options: PgConnectOptions = login_test_database_url(pool, TestDatabaseLogin::Business)
        .parse()
        .unwrap_or_else(|_| panic!("STOP: invalid owned Business transport"));
    assert!(
        options.get_socket().is_none()
            && matches!(options.get_host(), "127.0.0.1" | "::1" | "localhost"),
        "STOP: finite wire fixture requires disposable loopback TCP"
    );
    let upstream = (options.get_host().to_owned(), options.get_port());
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let local = listener.local_addr().unwrap();
    let state = Arc::new(Mutex::new(Grammar::new(
        ids,
        expected.group_id,
        authority.epoch(),
        authority.current_policy_receipt_id(),
    )));
    let connections = Arc::new(Mutex::new(0));
    let (stop, stopped) = oneshot::channel();
    let mut task = tokio::spawn(relay(
        listener,
        upstream,
        state.clone(),
        connections.clone(),
        stopped,
    ));
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
        let identity:(String,String,i32,bool,bool)=tokio::time::timeout(Duration::from_secs(5),sqlx::query_as("SELECT session_user::text,current_user::text,pg_backend_pid(),rolsuper,rolbypassrls FROM pg_catalog.pg_roles WHERE rolname=current_user").fetch_one(&proxy)).await.unwrap().unwrap();
        actual_pid=Some(identity.2);
        assert!(identity.0=="console_rt"&&identity.1=="console_rt"&&!identity.3&&!identity.4,"STOP: actual restricted LOGIN through transparent relay");
        let store=credentials::store(proxy.clone(),config);let policy=ObservedRead::new(authority.clone(),None);
        let positive=tokio::time::timeout(Duration::from_secs(12),native_policy_current(&store,&policy,original,selector)).await.unwrap();
        assert!(positive.is_ok(),"STOP: actual first/final-source transparent owner prerequisite");
        assert!(positive.unwrap()==*expected,"STOP: real Manager transparent view mismatch");policy.count(2);
        assert!(proven(&state,&connections,identity.2),"STOP: two actual correlated source/Auth calls, same Group/XID/PID and confirmed COMMIT prerequisite");
        assert!(*before==all_rows(pool).await,"transparent finalization changed full public census");
        state.lock().unwrap().arm(fault).expect("STOP: finite prepared cache arming");
        let policy=ObservedRead::new(authority.clone(),None);
        let result=tokio::time::timeout(Duration::from_secs(12),native_policy_current(&store,&policy,original,selector)).await;
        assert!(match fault {Fault::AuthZero=>matches!(result,Ok(Err(NativePolicyWorkflowError::AuthenticationInvalid))),_=>matches!(result,Ok(Err(NativePolicyWorkflowError::Conflict)))},"final actual source/Auth refusal released a view or wrong failure");
        policy.count(1);
        // Flush the real queued rollback on this same max1 connection, without a new owner attempt.
        let _:i32=tokio::time::timeout(Duration::from_secs(5),sqlx::query_scalar("SELECT 1").fetch_one(&proxy)).await.unwrap().unwrap();
        assert!(proven(&state,&connections,identity.2),"STOP: missing/extra final-source/Auth call, transformation, retained Group, actual40001 or silent retry");
        assert!(*before==all_rows(pool).await,"finalization transport fault changed full public census");
    }).catch_unwind().await;
    // Preserve the existing always-run independent shutdown/census discipline.
    let _ = stop.send(());
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let (closed, joined) = tokio::join!(
        tokio::time::timeout_at(deadline, proxy.close()),
        tokio::time::timeout_at(deadline, &mut task)
    );
    let aborted_join = if joined.is_err() {
        task.abort();
        Some(tokio::time::timeout(Duration::from_secs(1), &mut task).await)
    } else {
        None
    };
    let pid = actual_pid.or_else(|| state.lock().unwrap().backend_pid);
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
        "STOP: owned finalization pool/listener/task/PID cleanup not proven"
    );
    assert!(
        *before == all_rows(pool).await,
        "finalization shutdown changed full public census"
    );
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
    assert!(
        matches!(joined, Ok(Ok(Ok(())))),
        "STOP: finite finalization relay did not finish cleanly"
    );
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
        result.is_ok() && result.unwrap() == *expected,
        "STOP: independent original-browser runtime recovery"
    );
    policy.count(2);
    assert!(
        *before == all_rows(pool).await,
        "independent finalization recovery changed full public census"
    );
}
