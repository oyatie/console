// Supplementary actual mounted owner-progress witness. Business population and
// requested command come from the genuine preceding UI; no transport bypass.
use super::*;
use sqlx::Connection as _;

pub(super) async fn observe(
    pool: &PgPool,
    address: SocketAddr,
    cookie: &HeaderValue,
    account: Uuid,
    company: Uuid,
    group: Uuid,
    command: Uuid,
    business_secrets: &[String],
) -> Value {
    let before = all_rows(pool).await;
    // Prerequisite connections/PIDs precede locking and spawned request work.
    let mut observer = tokio::time::timeout(
        Duration::from_secs(5),
        sqlx::PgConnection::connect_with(pool.connect_options().as_ref()),
    )
    .await
    .expect("owner witness observer connection deadline")
    .unwrap();
    let mut blocker = tokio::time::timeout(
        Duration::from_secs(5),
        sqlx::PgConnection::connect_with(pool.connect_options().as_ref()),
    )
    .await
    .expect("owner witness blocker connection deadline")
    .unwrap();
    let observer_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut observer)
        .await
        .unwrap();
    let blocker_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut blocker)
        .await
        .unwrap();
    let http = client();
    let path = format!("/groups/{group}/identity/requests/{command}");
    let mut secrets = credential_values(cookie);
    secrets.extend([account.to_string(), company.to_string()]);
    secrets.extend_from_slice(business_secrets);
    let mut request: Option<tokio::task::JoinHandle<Document>> = None;
    let mut joined = false;
    let mut waited: Option<i32> = None;
    let mut blocker_retained = false;
    let mut owner_quiet_without_force = false;
    let outcome = AssertUnwindSafe(async {
        assert_eq!(document(&http,address,"GET","/readyz",None).await.status,StatusCode::OK);
        let active = document(&http,address,"GET","/account",Some(cookie)).await;
        let active_html=std::str::from_utf8(&active.bytes).unwrap();
        assert_eq!(active.status,StatusCode::OK);
        assert!(active_html.contains("data-account-state=\"active\"")
            && active_html.contains(&format!("href=\"/companies/{company}\""))
            && active_html.contains(&format!("href=\"/groups/{group}/identity\"")));
        assert!(!active.headers.contains_key(header::SET_COOKIE));
        no_effects(pool,&before).await;
        // The requested actual form command has not been submitted. Native404
        // is current visibility, never an assertion that effects cannot exist.
        let unknown = document(&http,address,"GET",&path,Some(cookie)).await;
        assert_eq!(unknown.status,StatusCode::NOT_FOUND);
        nondisclosing(&unknown,&secrets,true);
        no_effects(pool,&before).await;
        let deadline=tokio::time::Instant::now()+Duration::from_secs(5);
        let interrupted=tokio::time::timeout_at(deadline,async {
            sqlx::raw_sql("BEGIN ISOLATION LEVEL READ COMMITTED; SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='5s'")
                .execute(&mut blocker).await.unwrap();
            // Existing installed owner lock; no business SQL, hash reproduction,
            // trigger disabling, function replacement or new privilege.
            sqlx::query("SELECT public.native_group_process_group_guard_v1($1::uuid,true)")
                .bind(group).execute(&mut blocker).await.unwrap();
            let http=client();let supplied=cookie.clone();let requested=path.clone();
            request=Some(tokio::spawn(async move {document(&http,address,"GET",&requested,Some(&supplied)).await}));
            loop {
                waited=sqlx::query_scalar(
                    "SELECT a.pid FROM pg_catalog.pg_stat_activity a \
                     JOIN pg_catalog.pg_locks w ON w.pid=a.pid AND w.locktype='advisory' AND NOT w.granted \
                     JOIN pg_catalog.pg_locks b ON b.pid=$1 AND b.locktype=w.locktype \
                      AND b.database IS NOT DISTINCT FROM w.database AND b.classid=w.classid \
                      AND b.objid=w.objid AND b.objsubid=w.objsubid AND b.granted AND b.mode='ExclusiveLock' \
                     WHERE a.datname=current_database() AND a.usename='console_rt' AND a.wait_event_type='Lock' \
                      AND $1=ANY(pg_catalog.pg_blocking_pids(a.pid)) \
                      AND strpos(a.query,'identity_native_group_process_incarnation_selector_v1')>0 \
                     ORDER BY a.pid LIMIT 1")
                    .bind(blocker_pid).fetch_optional(&mut observer).await.unwrap();
                if waited.is_some() || request.as_ref().unwrap().is_finished() {break;}
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            let response=request.as_mut().unwrap().await;
            joined=true;
            let response=response.expect("actual mounted owner request task");
            blocker_retained=sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_locks WHERE pid=$1 AND locktype='advisory' AND mode='ExclusiveLock' AND granted)")
                .bind(blocker_pid).fetch_one(&mut observer).await.unwrap();
            // Retained owner uses a1s SQL lock bound. Observe actual transaction
            // quiescence while the blocker remains held; external cancellation
            // below is failure cleanup and must never manufacture this result.
            if let Some(waiter)=waited {
                let quiet_deadline=tokio::time::Instant::now()+Duration::from_secs(2);
                loop {
                    let (locks,busy):(i64,i64)=sqlx::query_as(
                        "SELECT (SELECT count(*) FROM pg_catalog.pg_locks WHERE pid=$1 AND locktype IN ('advisory','relation','transactionid','tuple')), \
                         (SELECT count(*) FROM pg_catalog.pg_stat_activity WHERE pid=$1 AND (state<>'idle' OR wait_event_type='Lock'))")
                        .bind(waiter).fetch_one(&mut observer).await.unwrap();
                    if (locks,busy)==(0,0) {owner_quiet_without_force=true;break;}
                    if tokio::time::Instant::now()>=quiet_deadline {break;}
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            }
            eprintln!("GROUP_NAVIGATION_MOUNTED_OWNER_WAIT_WITNESS {}",json!({
                "actual_owner_selector_wait":waited.is_some(),"status":response.status.as_u16(),
                "blocker_retained_until_timeout":blocker_retained,
                "owner_quiesced_without_forced_cancel":owner_quiet_without_force}));
            response
        }).await.expect("mounted owner observation exceeded five-second bound");
        assert!(waited.is_some(),"mounted request never reached retained Group SELECTOR lock");
        assert_eq!(interrupted.status,StatusCode::REQUEST_TIMEOUT);
        assert!(blocker_retained,"owner timeout required blocker release");
        assert!(owner_quiet_without_force,"HTTP timeout left retained owner transaction busy beyond its bounded recovery");
        nondisclosing(&interrupted,&secrets,true);
        let html=std::str::from_utf8(&interrupted.bytes).unwrap();
        assert!(html.contains("data-group-process-outcome=\"uncertain\""));
        assert_eq!(html.matches(&format!("href=\"{path}\"")).count(),1,
            "mounted owner timeout must retain the original requested locator");
        no_effects(pool,&before).await;
    }).catch_unwind().await;

    // Cleanup always runs after observation/semantic failure, with the blocker
    // released only after the observed timeout/quiescence fact was recorded.
    let cleanup_deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let rollback = tokio::time::timeout_at(
        cleanup_deadline,
        sqlx::raw_sql("ROLLBACK").execute(&mut blocker),
    )
    .await;
    let mut forced_cancel = None;
    if !owner_quiet_without_force && let Some(waiter) = waited {
        forced_cancel = Some(
            tokio::time::timeout_at(
                cleanup_deadline,
                sqlx::query_scalar::<_, bool>("SELECT pg_catalog.pg_cancel_backend($1)")
                    .bind(waiter)
                    .fetch_one(&mut observer),
            )
            .await,
        );
    }
    let mut aborted_join = None;
    if !joined && let Some(task) = request.as_mut() {
        task.abort();
        aborted_join = Some(tokio::time::timeout_at(cleanup_deadline, task).await);
    }
    let blocker_close = tokio::time::timeout_at(cleanup_deadline, blocker.close()).await;
    let census=tokio::time::timeout_at(cleanup_deadline,AssertUnwindSafe(async {
        loop {
            let (backends,locks,busy):(i64,i64,i64)=sqlx::query_as(
                "SELECT (SELECT count(*) FROM pg_catalog.pg_stat_activity WHERE pid=$1), \
                 (SELECT count(*) FROM pg_catalog.pg_locks WHERE pid=$1 OR (pid=$2 AND locktype IN ('advisory','relation','transactionid','tuple'))), \
                 (SELECT count(*) FROM pg_catalog.pg_stat_activity WHERE pid=$2 AND (state<>'idle' OR wait_event_type='Lock'))")
                .bind(blocker_pid).bind(waited.unwrap_or(-1)).fetch_one(&mut observer).await.unwrap();
            if (backends,locks,busy)==(0,0,0) {break;}
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        no_effects(pool,&before).await;
    }).catch_unwind()).await;
    let observer_close = tokio::time::timeout_at(cleanup_deadline, observer.close()).await;
    let observer_absent = tokio::time::timeout_at(cleanup_deadline, async {
        loop {
            let present: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity WHERE pid IN ($1,$2))",
            )
            .bind(observer_pid)
            .bind(blocker_pid)
            .fetch_one(pool)
            .await
            .unwrap();
            if !present {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await;
    let readmission = if outcome.is_ok() {
        Some(
            AssertUnwindSafe(async {
                let status = document(&http, address, "GET", &path, Some(cookie)).await;
                assert_eq!(status.status, StatusCode::NOT_FOUND);
                nondisclosing(&status, &secrets, true);
                no_effects(pool, &before).await;
                let account = document(&http, address, "GET", "/account", Some(cookie)).await;
                let html = std::str::from_utf8(&account.bytes).unwrap();
                assert_eq!(account.status, StatusCode::OK);
                assert!(
                    html.contains("data-account-state=\"active\"")
                        && html.contains(&format!("href=\"/companies/{company}\""))
                        && html.contains(&format!("href=\"/groups/{group}/identity\""))
                );
                assert!(!account.headers.contains_key(header::SET_COOKIE));
                no_effects(pool, &before).await;
            })
            .catch_unwind()
            .await,
        )
    } else {
        None
    };
    rollback
        .expect("Group blocker rollback deadline")
        .expect("Group blocker rollback failed");
    blocker_close
        .expect("Group blocker close deadline")
        .expect("Group blocker close failed");
    observer_close
        .expect("Group observer close deadline")
        .expect("Group observer close failed");
    observer_absent.expect("Group blocker/observer backend absence deadline");
    if let Some(cancel) = forced_cancel {
        cancel
            .expect("failed-case waiter cancel deadline")
            .expect("failed-case waiter cancel failed");
    }
    if let Some(join) = aborted_join {
        match join.expect("failed-case request join deadline") {
            Ok(_) => {}
            Err(error) if error.is_cancelled() => {}
            Err(_) => panic!("owner request panicked during cleanup"),
        }
    }
    match census {
        Ok(Ok(())) => {}
        Ok(Err(panic)) => std::panic::resume_unwind(panic),
        Err(_) => panic!("Group owner/backend/rows cleanup exceeded five-second bound"),
    };
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
    if let Some(Err(panic)) = readmission {
        std::panic::resume_unwind(panic);
    }
    json!({"path":path,"http_timeout_ms":500,"actual_owner_selector_wait":true,
        "blocker_retained_until_timeout":true,"status":408,"exact_original_locator":true,
        "owner_quiesced_without_forced_cancel":true,"complete_rows_unchanged":true,
        "actual_lock_cleanup":true,"status_readmission":404,"account_readmission":200,"authorized_account":true})
}
