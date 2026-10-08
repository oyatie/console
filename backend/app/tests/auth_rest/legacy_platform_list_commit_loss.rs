//! Actual local-test PostgreSQL wire acknowledgement loss, not a production switch.
use super::*;
use console_platform_test_support::login_test_database_url;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::oneshot,
};

use crate::native_pg_test_wire::{read_frame, read_startup, write_frame};
#[derive(Clone, Default)]
pub(in super::super::super::super::super) struct WireEvidence {
    pub(in super::super::super::super::super) connections: usize,
    startup_valid: bool,
    pub(in super::super::super::super::super) backend_pid: Option<i32>,
    pub(in super::super::super::super::super) unarmed_commits: usize,
    pub(in super::super::super::super::super) armed_commits: usize,
    pub(in super::super::super::super::super) forwarded_commits: usize,
    pub(in super::super::super::super::super) positive_commit_ack: usize,
    suppressed_commit_ack: usize,
    suppressed_idle: usize,
    pub(in super::super::super::super::super) backend_error: bool,
    wrong_commit_ack: bool,
    lost_after_success: bool,
    relay_finished: bool,
    relay_failed: bool,
}
pub(in super::super::super::super::super) fn proven_ack_loss(e: &WireEvidence) -> bool {
    e.connections == 1
        && e.startup_valid
        && e.backend_pid.is_some_and(|pid| pid > 0)
        && e.unarmed_commits == 1
        && e.armed_commits == 1
        && e.forwarded_commits == 2
        && e.positive_commit_ack == 1
        && e.suppressed_commit_ack == 1
        && e.suppressed_idle == 1
        && !e.backend_error
        && !e.wrong_commit_ack
        && e.lost_after_success
        && e.relay_finished
        && !e.relay_failed
}
pub(in super::super::super::super::super) fn evidence(
    e: &Arc<Mutex<WireEvidence>>,
) -> WireEvidence {
    e.lock().unwrap().clone()
}
fn invalid_wire() -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "finite fixture wire protocol failure",
    )
}

async fn upload<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    mut client: R,
    mut server: W,
    armed: Arc<AtomicBool>,
    e: Arc<Mutex<WireEvidence>>,
) -> std::io::Result<()> {
    loop {
        let (tag, body) = read_frame(&mut client).await?;
        let commit = tag == b'Q' && body == b"COMMIT\0";
        if commit {
            let mut e = e.lock().unwrap();
            if armed.load(Ordering::SeqCst) {
                e.armed_commits += 1;
            } else {
                e.unarmed_commits += 1;
            }
        }
        write_frame(&mut server, tag, &body).await?;
        if commit {
            e.lock().unwrap().forwarded_commits += 1;
        }
    }
}
async fn download<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    mut server: R,
    mut client: W,
    e: Arc<Mutex<WireEvidence>>,
) -> std::io::Result<()> {
    loop {
        let (tag, body) = read_frame(&mut server).await?;
        if tag == b'K' && body.len() == 8 {
            e.lock().unwrap().backend_pid = Some(i32::from_be_bytes(
                body[..4].try_into().map_err(|_| invalid_wire())?,
            ));
        }
        let armed = e.lock().unwrap().armed_commits > 0;
        if tag == b'E' {
            e.lock().unwrap().backend_error = true;
        }
        if tag == b'C' && body == b"COMMIT\0" {
            if armed {
                e.lock().unwrap().suppressed_commit_ack += 1;
                continue;
            }
            e.lock().unwrap().positive_commit_ack += 1;
        } else if armed && tag == b'C' {
            e.lock().unwrap().wrong_commit_ack = true;
        }
        if armed && tag == b'Z' {
            let mut state = e.lock().unwrap();
            if body == b"I"
                && state.suppressed_commit_ack == 1
                && !state.backend_error
                && !state.wrong_commit_ack
            {
                state.suppressed_idle += 1;
                state.lost_after_success = true;
                // Both backend success frames were consumed. Drop this downstream write
                // half now; caller sees EOF before receiving either acknowledgement.
                return Ok(());
            }
        }
        write_frame(&mut client, tag, &body).await?;
    }
}
async fn relay_connection(
    mut client: TcpStream,
    upstream: (String, u16),
    armed: Arc<AtomicBool>,
    e: Arc<Mutex<WireEvidence>>,
) -> std::io::Result<()> {
    let mut server = TcpStream::connect(upstream).await?;
    let startup = read_startup(&mut client).await?;
    e.lock().unwrap().startup_valid = true;
    server.write_u32((startup.len() + 4) as u32).await?;
    server.write_all(&startup).await?;
    server.flush().await?;
    drop(startup);
    let (client_read, client_write) = client.into_split();
    let (server_read, server_write) = server.into_split();
    // Each half has a persistent future. No partially-read frame is cancelled
    // merely because the opposite direction becomes readable.
    let up = upload(client_read, server_write, armed, e.clone());
    let down = download(server_read, client_write, e.clone());
    tokio::pin!(up, down);
    let result = tokio::select! {result=&mut up=>result,result=&mut down=>result};
    // Dropping the other future also drops both remaining socket halves.
    let mut state = e.lock().unwrap();
    state.relay_finished = true;
    state.relay_failed = result.is_err();
    result
}
pub(in super::super::super::super::super) async fn relay(
    listener: TcpListener,
    upstream: (String, u16),
    armed: Arc<AtomicBool>,
    e: Arc<Mutex<WireEvidence>>,
    mut stop: oneshot::Receiver<()>,
) -> std::io::Result<()> {
    let (first, _) =
        tokio::select! {result=listener.accept()=>result?,_=&mut stop=>return Err(invalid_wire())};
    e.lock().unwrap().connections += 1;
    let connection = relay_connection(first, upstream, armed, e.clone());
    tokio::pin!(connection);
    let mut finished = false;
    loop {
        tokio::select! {
            result=&mut connection,if !finished=>{result?;finished=true;},
            result=listener.accept()=>{let (extra,_)=result?;e.lock().unwrap().connections+=1;drop(extra);},
            _=&mut stop=>return if finished{Ok(())}else{Err(invalid_wire())},
        }
    }
}
async fn list_on(
    f: &Fixture,
    pool: &PgPool,
) -> Result<Vec<OrganizationSummary>, ProvisioningError> {
    PlatformProvisioner::new(Duration::hours(1))
        .list_tenants(
            pool,
            &f.verifier,
            &f.access,
            Duration::days(30),
            &PlatformPolicy::compile_current().unwrap(),
        )
        .await
}

#[sqlx::test(migrations = false)]
async fn direct_list_lost_commit_ack_returns_unavailable_with_one_durable_audit_and_explicit_recovery(
    pool: PgPool,
) {
    let f = fixture(&pool).await;
    let options: PgConnectOptions = login_test_database_url(&pool, TestDatabaseLogin::Business)
        .parse()
        .unwrap_or_else(|_| panic!("invalid isolated Business test transport"));
    assert!(
        options.get_socket().is_none()
            && matches!(options.get_host(), "127.0.0.1" | "::1" | "localhost"),
        "wire fixture requires a disposable loopback TCP database transport"
    );
    let upstream = (options.get_host().to_owned(), options.get_port());
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let local = listener.local_addr().unwrap();
    let armed = Arc::new(AtomicBool::new(false));
    let transcript = Arc::new(Mutex::new(WireEvidence::default()));
    let (stop_tx, stop_rx) = oneshot::channel();
    // No production TLS setting is readjusted. Only this isolated plaintext local
    // fixture connection lets a test relay distinguish PostgreSQL frame boundaries.
    let task = tokio::spawn(relay(
        listener,
        upstream,
        armed.clone(),
        transcript.clone(),
        stop_rx,
    ));
    let proxy = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        PgPoolOptions::new()
            .max_connections(1)
            .min_connections(0)
            .acquire_timeout(std::time::Duration::from_secs(8))
            .connect_with(
                options
                    .host("127.0.0.1")
                    .port(local.port())
                    .ssl_mode(PgSslMode::Disable),
            ),
    )
    .await;
    let proxy = match proxy {
        Ok(Ok(pool)) => pool,
        _ => {
            let _ = stop_tx.send(());
            let _ = finish(task).await;
            panic!("disposable local plaintext Business relay prerequisite unavailable");
        }
    };
    let outcome=std::panic::AssertUnwindSafe(async{
        let identity:(String,String,i32,bool,bool)=sqlx::query_as("SELECT session_user::text,current_user::text,pg_backend_pid(),rolsuper,rolbypassrls FROM pg_catalog.pg_roles WHERE rolname=current_user").fetch_one(&proxy).await.unwrap();
        assert!(identity.0=="console_rt"&&identity.1=="console_rt"&&!identity.3&&!identity.4,"actual restricted authentication through relay");
        let mut observer=pool.acquire().await.unwrap();
        let before=all_rows(&pool).await;let start=db_now(&pool).await;
        let positive=tokio::time::timeout(std::time::Duration::from_secs(12),list_on(&f,&proxy)).await.unwrap().unwrap();
        let end=db_now(&pool).await;let after=all_rows(&pool).await;
        assert!(metadata_matches(&metadata(&positive),&f.expected));
        assert!(exact_read_delta(&before,&after,f.actor,positive.len(),start,end),"unarmed real wire positive control");
        let first=evidence(&transcript);
        assert!(first.connections==1&&first.backend_pid==Some(identity.2)&&first.unarmed_commits==1&&first.armed_commits==0&&first.forwarded_commits==1&&first.positive_commit_ack==1&&!first.backend_error,"actual transparent COMMIT positive control");
        let before=all_rows(&pool).await;let start=db_now(&pool).await;
        armed.store(true,Ordering::SeqCst);
        let result=tokio::time::timeout(std::time::Duration::from_secs(12),list_on(&f,&proxy)).await;
        let end=db_now(&pool).await;
        assert!(matches!(result,Ok(Err(ProvisioningError::PlatformListUnavailable))),"commit acknowledgement loss returns no collected rows and finite unavailable");
        assert!(proven_ack_loss(&evidence(&transcript)),"only consumed exact COMMIT and idle success before downstream loss qualifies");
        assert!(clean_pids(&mut observer,&[identity.2]).await,"wire loss closes owned transaction/backend without dependent waiters");
        let committed=all_rows(&pool).await;
        assert!(exact_read_delta(&before,&committed,f.actor,f.expected.as_array().unwrap().len(),start,end),"lost acknowledgement has exactly one durable audit, preserved history and no other effects");
        // Explicit caller recovery is a separate new owner invocation on the genuine
        // normal Business connection, never an automatic retry of uncertain COMMIT.
        direct_positive(&pool,&f,&f.access,Duration::days(30)).await;
        assert!(proven_ack_loss(&evidence(&transcript)),"no extra relay attempt before or during explicit independent recovery");
    }).catch_unwind().await;
    let _ = stop_tx.send(());
    proxy.close().await;
    let relay_result = finish(task).await;
    f.state.shutdown_realtime().await;
    f.business.close().await;
    if let Err(p) = outcome {
        std::panic::resume_unwind(p);
    }
    assert!(
        matches!(relay_result, Ok(Ok(()))),
        "owned relay and listener complete without detached tasks"
    );
}

#[test]
fn commit_loss_evidence_rejects_missing_duplicate_and_precommit_failure() {
    let valid = WireEvidence {
        connections: 1,
        startup_valid: true,
        backend_pid: Some(123),
        unarmed_commits: 1,
        armed_commits: 1,
        forwarded_commits: 2,
        positive_commit_ack: 1,
        suppressed_commit_ack: 1,
        suppressed_idle: 1,
        backend_error: false,
        wrong_commit_ack: false,
        lost_after_success: true,
        relay_finished: true,
        relay_failed: false,
    };
    assert!(proven_ack_loss(&valid));
    for fault in [
        "extra_connection",
        "no_startup",
        "no_pid",
        "no_positive",
        "unforwarded",
        "no_commit",
        "duplicate_commit",
        "no_positive_ack",
        "no_commit_ack",
        "duplicate_ack",
        "no_idle",
        "backend_error",
        "wrong_ack",
        "precommit_cut",
        "unfinished",
        "relay_error",
    ] {
        let mut bad = valid.clone();
        match fault {
            "extra_connection" => bad.connections = 2,
            "no_startup" => bad.startup_valid = false,
            "no_pid" => bad.backend_pid = None,
            "no_positive" => bad.unarmed_commits = 0,
            "unforwarded" => bad.forwarded_commits = 1,
            "no_commit" => bad.armed_commits = 0,
            "duplicate_commit" => bad.armed_commits = 2,
            "no_positive_ack" => bad.positive_commit_ack = 0,
            "no_commit_ack" => bad.suppressed_commit_ack = 0,
            "duplicate_ack" => bad.suppressed_commit_ack = 2,
            "no_idle" => bad.suppressed_idle = 0,
            "backend_error" => bad.backend_error = true,
            "wrong_ack" => bad.wrong_commit_ack = true,
            "precommit_cut" => bad.lost_after_success = false,
            "unfinished" => bad.relay_finished = false,
            "relay_error" => bad.relay_failed = true,
            _ => unreachable!(),
        }
        assert!(
            !proven_ack_loss(&bad),
            "invalid acknowledgement-loss evidence: {fault}"
        );
    }
}

#[tokio::test]
async fn relay_frame_decoder_rejects_invalid_lengths_and_requires_complete_payload() {
    for wire in [
        vec![b'Q', 0, 0, 0, 3],
        vec![b'Q', 0, 32, 0, 0],
        vec![b'C', 0, 0, 0, 11, b'C', b'O'],
    ] {
        let result = read_frame(&mut wire.as_slice()).await;
        assert!(result.is_err(), "bounded complete wire frame required");
    }
    let mut wire = Vec::new();
    write_frame(&mut wire, b'C', b"COMMIT\0").await.unwrap();
    let (tag, body) = read_frame(&mut wire.as_slice()).await.unwrap();
    assert!(tag == b'C' && body == b"COMMIT\0");
}

#[tokio::test]
async fn relay_suppression_requires_actual_commit_completion_and_idle_frames() {
    for fault in [
        "valid",
        "rollback",
        "missing_completion",
        "still_transaction",
        "failed_transaction",
        "backend_error",
        "duplicate_completion",
    ] {
        let (mut backend_writer, backend_reader) = tokio::io::duplex(4096);
        let (downstream_writer, mut downstream_reader) = tokio::io::duplex(4096);
        let transcript = Arc::new(Mutex::new(WireEvidence {
            connections: 1,
            startup_valid: true,
            backend_pid: Some(123),
            unarmed_commits: 1,
            armed_commits: 1,
            forwarded_commits: 2,
            positive_commit_ack: 1,
            ..WireEvidence::default()
        }));
        if fault == "backend_error" {
            write_frame(&mut backend_writer, b'E', b"fixture error")
                .await
                .unwrap();
        }
        if fault != "missing_completion" {
            write_frame(
                &mut backend_writer,
                b'C',
                if fault == "rollback" {
                    b"ROLLBACK\0"
                } else {
                    b"COMMIT\0"
                },
            )
            .await
            .unwrap();
        }
        if fault == "duplicate_completion" {
            write_frame(&mut backend_writer, b'C', b"COMMIT\0")
                .await
                .unwrap();
        }
        write_frame(
            &mut backend_writer,
            b'Z',
            match fault {
                "still_transaction" => b"T",
                "failed_transaction" => b"E",
                _ => b"I",
            },
        )
        .await
        .unwrap();
        backend_writer.shutdown().await.unwrap();
        let result = download(backend_reader, downstream_writer, transcript.clone()).await;
        let mut returned = Vec::new();
        downstream_reader.read_to_end(&mut returned).await.unwrap();
        let mut observed = evidence(&transcript);
        observed.relay_finished = true;
        observed.relay_failed = result.is_err();
        if fault == "valid" {
            assert!(result.is_ok() && returned.is_empty() && proven_ack_loss(&observed));
        } else {
            assert!(
                result.is_err() && !proven_ack_loss(&observed),
                "incorrect backend history must not qualify as committed loss: {fault}"
            );
        }
    }
}
