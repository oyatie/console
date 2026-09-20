//! Reuse the actual acknowledged-COMMIT loss relay through the separate ops owner.
use super::super::super::legacy_platform_list_credentials::{
    WireEvidence, evidence, proven_ack_loss, relay,
};
use super::*;
use console_platform_test_support::login_test_database_url;
use futures::FutureExt;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use tokio::{net::TcpListener, sync::oneshot};

async fn ops_on(f: &Fixture, pool: &PgPool) -> Result<Vec<TenantHealth>, ProvisioningError> {
    PlatformProvisioner::new(Duration::hours(1))
        .list_tenant_health(
            pool,
            &f.verifier,
            &f.access,
            Duration::days(30),
            &PlatformPolicy::compile_current().unwrap(),
        )
        .await
}

#[sqlx::test(migrations = false)]
async fn direct_ops_lost_commit_ack_returns_unavailable_with_one_durable_audit_and_explicit_recovery(
    pool: PgPool,
) {
    let (f, expected) = populated_fixture(&pool).await;
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
        let positive=tokio::time::timeout(std::time::Duration::from_secs(12),ops_on(&f,&proxy)).await.unwrap().unwrap();
        let end=db_now(&pool).await;let after=all_rows(&pool).await;
        assert!(health_matches(&direct_projection(&positive),&expected));
        assert!(health_read_delta(&before,&after,f.actor,positive.len(),start,end),"unarmed real wire positive control");
        let first=evidence(&transcript);
        assert!(first.connections==1&&first.backend_pid==Some(identity.2)&&first.unarmed_commits==1&&first.armed_commits==0&&first.forwarded_commits==1&&first.positive_commit_ack==1&&!first.backend_error,"actual transparent COMMIT positive control");
        let before=all_rows(&pool).await;let start=db_now(&pool).await;
        armed.store(true,Ordering::SeqCst);
        let result=tokio::time::timeout(std::time::Duration::from_secs(12),ops_on(&f,&proxy)).await;
        let end=db_now(&pool).await;
        assert!(matches!(result,Ok(Err(ProvisioningError::PlatformHealthUnavailable))),"commit acknowledgement loss returns no collected rows and finite unavailable");
        assert!(proven_ack_loss(&evidence(&transcript)),"only consumed exact COMMIT and idle success before downstream loss qualifies");
        assert!(clean_pids(&mut observer,&[identity.2]).await,"wire loss closes owned transaction/backend without dependent waiters");
        let committed=all_rows(&pool).await;
        assert!(health_read_delta(&before,&committed,f.actor,expected["tenants"].as_array().unwrap().len(),start,end),"lost acknowledgement has exactly one durable audit, preserved history and no other effects");
        // Explicit caller recovery is a separate new owner invocation on the genuine
        // normal Business connection, never an automatic retry of uncertain COMMIT.
        owner_positive(&pool,&f,&f.access,&expected).await;
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
