// Evidence-only additive child inside native_policy_successor_contract.
// Reuses its exact-reviewed constants, fixture, metadata and whole-finalizer
// helpers. That parent proposal still has unresolved pins; do not execute yet.
mod unknown_namespace {
    use super::*;

    #[sqlx::test(migrations = false)]
    async fn unknown_operator_owned_policy_names_refuse_startup_and_finalization_on_v1_and_v2(
        pool: PgPool,
    ) {
        reviewed();
        let (app, key, predecessor_state) = configured_fixture(&pool, true).await;
        let config = account_browser_config(&pool, app._artifacts.root.clone(), &key);
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let mut successor_state = None;
        let outcome=AssertUnwindSafe(async {
            let faults=[
                ("CREATE FUNCTION public.native_company_policy_probe_v2() RETURNS smallint LANGUAGE sql SECURITY INVOKER AS 'SELECT 1::smallint'",
                 "SELECT p.proowner=(SELECT oid FROM pg_roles WHERE rolname=session_user) AND NOT p.prosecdef FROM pg_proc p WHERE p.oid=to_regprocedure('public.native_company_policy_probe_v2()')",
                 "DROP FUNCTION public.native_company_policy_probe_v2()"),
                ("CREATE FUNCTION public.native_company_people_probe_v1() RETURNS smallint LANGUAGE sql SECURITY INVOKER AS 'SELECT 1::smallint'",
                 "SELECT p.proowner=(SELECT oid FROM pg_roles WHERE rolname=session_user) AND NOT p.prosecdef FROM pg_proc p WHERE p.oid=to_regprocedure('public.native_company_people_probe_v1()')",
                 "DROP FUNCTION public.native_company_people_probe_v1()"),
                ("CREATE FUNCTION ontology_api.install_native_company_people_probe_v1() RETURNS smallint LANGUAGE sql SECURITY INVOKER AS 'SELECT 1::smallint'",
                 "SELECT p.proowner=(SELECT oid FROM pg_roles WHERE rolname=session_user) AND NOT p.prosecdef FROM pg_proc p WHERE p.oid=to_regprocedure('ontology_api.install_native_company_people_probe_v1()')",
                 "DROP FUNCTION ontology_api.install_native_company_people_probe_v1()"),
            ];
            let mut executed=0;
            for upgraded in [false,true] {
                if upgraded {
                    let mut tx=pool.begin().await.unwrap();install(&mut tx).await.unwrap();tx.commit().await.unwrap();
                    let fresh=AppState::from_config(config.clone()).await.unwrap();
                    successor_state=Some(fresh);
                }
                let state=if upgraded {successor_state.as_ref().unwrap()} else {&predecessor_state};
                let current=if upgraded {FINALIZED} else {ABSENT};
                assert_eq!(classified(&runtime,CLASSIFIER).await,current);
                assert_eq!(ready_status(state).await,StatusCode::OK);
                let before=all_rows(&pool).await;
                let snapshot=metadata(&runtime,CAPTURE).await;
                let legacy_snapshot=metadata(&runtime,OLD_CAPTURE).await;
                if !upgraded {assert_eq!(policy_classified(&runtime).await,"native_company_policy.finalized");}
                for (mutation,witness,restore) in faults {
                    sqlx::raw_sql(sqlx::AssertSqlSafe(mutation)).execute(&pool).await.unwrap();
                    let checked=AssertUnwindSafe(async {
                        assert!(sqlx::query_scalar::<_,bool>(sqlx::AssertSqlSafe(witness)).fetch_one(&pool).await.unwrap(),"fault must be operator-owned SECURITY INVOKER, not a captured owner/definer");
                        let changed=metadata(&runtime,CAPTURE).await;
                        assert_ne!(changed.0,snapshot.0,"unknown-name metadata omitted");
                        assert_ne!(changed.1,snapshot.1,"unknown-name fingerprint ignored");
                        if !upgraded {
                            // Positive control: the historical classifier remains
                            // valid, so selecting it would hide this new defect.
                            assert_eq!(metadata(&runtime,OLD_CAPTURE).await,legacy_snapshot);
                            assert_eq!(policy_classified(&runtime).await,"native_company_policy.finalized");
                        }
                        assert_eq!(classified(&runtime,CLASSIFIER).await,MISMATCH);
                        startup_refused(config.clone(),MISMATCH).await;
                        assert_eq!(ready_status(state).await,StatusCode::SERVICE_UNAVAILABLE);
                        let mut tx=pool.begin().await.unwrap();
                        let error=install(&mut tx).await.unwrap_err();
                        let database=error.as_database_error().unwrap();
                        assert_eq!(database.code().as_deref(),Some("P0001"));
                        assert_eq!(database.message(),MISMATCH,"partial/unknown presence must refuse before applying source");
                        tx.rollback().await.unwrap();
                        assert_eq!(metadata(&runtime,CAPTURE).await,changed,"refused finalizer repaired or altered candidate metadata");
                        assert!(before==all_rows(&pool).await,"fault/finalizer refusal changed business rows");
                    }).catch_unwind().await;
                    sqlx::raw_sql(sqlx::AssertSqlSafe(restore)).execute(&pool).await.unwrap();
                    assert_eq!(metadata(&runtime,CAPTURE).await,snapshot);
                    assert!(before==all_rows(&pool).await);
                    assert_eq!(classified(&runtime,CLASSIFIER).await,current);
                    assert_eq!(ready_status(state).await,StatusCode::OK);
                    if let Err(panic)=checked {std::panic::resume_unwind(panic);}
                    executed+=1;
                }
            }
            assert_eq!(executed,6,"both custody states require all three namespace controls");
        }).catch_unwind().await;
        if let Some(state) = successor_state {
            state.shutdown_realtime().await;
        }
        runtime.close().await;
        close_states(&[predecessor_state], outcome).await;
    }
}
