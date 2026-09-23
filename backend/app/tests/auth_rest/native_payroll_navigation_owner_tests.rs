// Real native adapter finalization; delegates policy to real Cedar before faults.
mod native_payroll_navigation_owner_tests {
    use super::*;
    use console_payroll_application::read::payroll_navigation;
    use std::task::Poll;

    #[sqlx::test(migrations = false)]
    async fn navigation_final_authority_and_cancellation_release_no_rows_or_audit(pool: PgPool) {
        let f = FixtureRead::new(&pool).await;
        let result = AssertUnwindSafe(async {
            f.install().await;
            f.grant(&pool).await;
            let before = all_rows(&pool).await;
            let mut blocker = pool.begin().await.unwrap();
            sqlx::query("LOCK TABLE public.payroll_draft_runs IN ACCESS EXCLUSIVE MODE")
                .execute(blocker.as_mut())
                .await
                .unwrap();
            let result = payroll_navigation(&mut f.reader()).await;
            blocker.rollback().await.unwrap();
            assert_eq!(result.unwrap(), f.company());
            assert!(
                before == all_rows(&pool).await,
                "healthy application navigation read rows or persisted effects"
            );
            for fault in [
                Fault::InitialIdentityDeny,
                Fault::FinalIdentityDeny,
                Fault::InitialIdentityFailure,
                Fault::FinalIdentityFailure,
                Fault::FinalPayrollDeny,
            ] {
                let policy = Arc::new(FaultPolicy {
                    real: f.policy.clone(),
                    company: f.company(),
                    actor: f.created.administrator,
                    fault,
                    trace: Mutex::new(DecisionTrace::default()),
                });
                let mut reader = PgNativePayrollRunsReadPort::new(
                    f.runtime.clone(),
                    f.verifier.clone(),
                    f.ttl,
                    policy.clone(),
                    read_credentials(&f.cookies),
                    f.company(),
                );
                let before = all_rows(&pool).await;
                assert!(matches!(
                    reader.finish_navigation().await,
                    Err(PayrollRunsReadError::Unavailable)
                ));
                // The invalid out-of-order call consumes this reader, without effects.
                assert!(matches!(
                    reader.authorize().await,
                    Err(PayrollRunsReadError::Unavailable)
                ));
                assert!(before == all_rows(&pool).await);
                let mut reader = PgNativePayrollRunsReadPort::new(
                    f.runtime.clone(),
                    f.verifier.clone(),
                    f.ttl,
                    policy.clone(),
                    read_credentials(&f.cookies),
                    f.company(),
                );
                let initial = reader.authorize().await;
                assert_eq!(policy.trace.lock().unwrap().events, ["payroll", "identity"]);
                if fault == Fault::InitialIdentityFailure {
                    assert!(matches!(initial, Err(PayrollRunsReadError::Unavailable)));
                } else {
                    initial.unwrap();
                    match (fault, reader.finish_navigation().await) {
                        (Fault::InitialIdentityDeny | Fault::FinalIdentityDeny, Ok(company)) => {
                            assert_eq!(company, f.company())
                        }
                        (Fault::FinalIdentityFailure, Err(PayrollRunsReadError::Unavailable)) => {}
                        (
                            Fault::FinalPayrollDeny,
                            Err(PayrollRunsReadError::Authorization(error)),
                        ) => assert_eq!(error.kind, console_kernel_core::ErrorKind::NotFound),
                        (_, result) => panic!("navigation final authority changed: {result:?}"),
                    }
                }
                let events = policy.trace.lock().unwrap().events.clone();
                assert_eq!(
                    events,
                    match fault {
                        Fault::InitialIdentityFailure => vec!["payroll", "identity"],
                        Fault::FinalPayrollDeny => vec!["payroll", "identity", "payroll"],
                        _ => vec!["payroll", "identity", "payroll", "identity"],
                    }
                );
                assert_eq!(policy.trace.lock().unwrap().faults, 1);
                assert!(matches!(
                    reader.finish_navigation().await,
                    Err(PayrollRunsReadError::Unavailable)
                ));
                assert!(matches!(
                    reader
                        .read_page(ListPayrollRuns {
                            limit: None,
                            offset: None
                        })
                        .await,
                    Err(PayrollRunsReadError::Unavailable)
                ));
                drop(reader);
                released_company_lock(&pool, f.created.company).await;
                assert!(
                    before == all_rows(&pool).await,
                    "navigation created audit/business effects"
                );
            }
            for poll_once in [false, true] {
                let before = all_rows(&pool).await;
                let policy = Arc::new(FaultPolicy {
                    real: f.policy.clone(),
                    company: f.company(),
                    actor: f.created.administrator,
                    fault: Fault::FinalIdentityDeny,
                    trace: Mutex::new(DecisionTrace::default()),
                });
                let mut reader = PgNativePayrollRunsReadPort::new(
                    f.runtime.clone(),
                    f.verifier.clone(),
                    f.ttl,
                    policy.clone(),
                    read_credentials(&f.cookies),
                    f.company(),
                );
                reader.authorize().await.unwrap();
                assert!(!company_lock(&pool, f.created.company).await);
                let mut future = reader.finish_navigation();
                if poll_once {
                    let pending = std::future::poll_fn(|cx| {
                        Poll::Ready(future.as_mut().poll(cx).is_pending())
                    })
                    .await;
                    assert!(
                        pending,
                        "did not observe pre-COMMIT pending final source acquisition"
                    );
                    assert_eq!(policy.trace.lock().unwrap().events, ["payroll", "identity"]);
                }
                // Drop only the future; reader stays alive. No COMMIT was dispatched.
                drop(future);
                released_company_lock(&pool, f.created.company).await;
                assert!(matches!(
                    reader.finish_navigation().await,
                    Err(PayrollRunsReadError::Unavailable)
                ));
                assert!(matches!(
                    reader.authorize().await,
                    Err(PayrollRunsReadError::Unavailable)
                ));
                assert!(before == all_rows(&pool).await);
            }
            successful_empty_read(&pool, &f).await;
        })
        .catch_unwind()
        .await;
        f.close(result).await;
    }
    #[sqlx::test(migrations = false)]
    async fn navigation_assignment_expiry_after_authorize_withholds_link_and_audit(pool: PgPool) {
        let f = FixtureRead::new(&pool).await;
        let result = AssertUnwindSafe(async {
            f.install().await;
            let now: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                .fetch_one(&pool)
                .await
                .unwrap();
            // A real, whole-minute grant, never an edited assignment/receipt.
            // Leave enough time for the witnessed initial read on a loaded runner.
            let deadline = (now.unix_timestamp() / 60 + 1) * 60;
            let deadline = if deadline - now.unix_timestamp() >= 5 {
                deadline
            } else {
                deadline + 60
            };
            let expires = time::OffsetDateTime::from_unix_timestamp(deadline).unwrap();
            f.grant_until(expires).await;
            successful_empty_read(&pool, &f).await;
            // Wait outside the reader transaction: console_rt has real 30s idle
            // and 45s transaction deadlines, which must remain enabled.
            tokio::time::timeout(std::time::Duration::from_secs(66), async {
                loop {
                    let now: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                        .fetch_one(&pool)
                        .await
                        .unwrap();
                    if expires - now <= time::Duration::seconds(10) {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            })
            .await
            .expect("database clock did not reach bounded admission window");
            let before = all_rows(&pool).await;
            let mut reader = f.reader();
            reader.authorize().await.unwrap();
            let observed: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                .fetch_one(&pool)
                .await
                .unwrap();
            assert!(
                expires - observed > time::Duration::seconds(2)
                    && expires - observed <= time::Duration::seconds(10),
                "positive retained-authorization prerequisite missed bounded expiry window"
            );
            assert!(!company_lock(&pool, f.created.company).await);
            tokio::time::timeout(std::time::Duration::from_secs(66), async {
                loop {
                    let observed: time::OffsetDateTime =
                        sqlx::query_scalar("SELECT clock_timestamp()")
                            .fetch_one(&pool)
                            .await
                            .unwrap();
                    if observed >= expires {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            })
            .await
            .expect("database clock did not cross actual granted interval");
            match reader.finish_navigation().await {
                Err(PayrollRunsReadError::Authorization(error)) => {
                    assert_eq!(error.kind, console_kernel_core::ErrorKind::NotFound)
                }
                _ => panic!("expired final Payroll authority released or misclassified navigation"),
            }
            drop(reader);
            released_company_lock(&pool, f.created.company).await;
            assert!(
                before == all_rows(&pool).await,
                "expired retained navigation left an audit or other effect"
            );
            denied_without_writes(&pool, &f).await;
        })
        .catch_unwind()
        .await;
        f.close(result).await;
    }
}
