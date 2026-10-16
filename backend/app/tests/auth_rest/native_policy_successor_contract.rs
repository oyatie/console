// Evidence-only proposal. Mount as child of native_policy_startup_tests only
// after replacing REVIEW_REQUIRED pins/literals through independent review.
// No test is ignored; missing reviewed prerequisites are NOT semantic RED.
mod native_policy_successor_contract {
    use super::*;
    use console_identity_application::company_policy::{
        AccountId,
        business::PolicyAssignmentExpectationV1,
        workflow::{
            NativePolicyEffect, NativePolicyStatus, NativePolicyTerminalView,
            execute_native_policy_command, native_policy_command_status,
        },
    };
    use sqlx::{Postgres, Transaction};

    const FINALIZER: &str =
        include_str!("../../../../ops/postgres-finalize-native-company-policy-v2.sql");
    const CLASSIFIER: &str =
        include_str!("../../../../ops/postgres-native-company-policy-v2-custody-state.sql");
    const APP_CLASSIFIER: &str =
        include_str!("../../src/native_company_policy_v2_custody_state.sql");
    const CAPTURE: &str =
        include_str!("../../../../ops/postgres-capture-native-company-policy-v2-custody.sql");
    const OLD_CAPTURE: &str =
        include_str!("../../../../ops/postgres-capture-native-company-policy-custody.sql");
    const FINALIZER_SHA: &str = "ec945607e209b93843116ae2b2a20772797dce38ff7884fb96081f09651f7d8e";
    const CLASSIFIER_SHA: &str = "e507d75f446ad8d3e0befe9321d94731a1a2cc9b3e0e9c0306c1459a098a53cc";
    const CAPTURE_SHA: &str = "7534375fbae287ccf5e5015815e788ef0d7a379eed623af9a72f5db0d89614b4";
    const SUCCESSOR_PLAIN: &str =
        "e94251d48fdee392d7632709b19d80cc04d53ea6742e0f2d6538d68391ab8bf2";
    const SUCCESSOR_OBSERVER: &str =
        "f132054a56641dcb0cc5875d6485846df2091e631d0d902a18356d2202343fd9";
    const OBSERVER: &str = include_str!("../../../../ops/postgres-install-durability-observer.sql");
    // Proposed exact public states/error; bind actual reviewed contracts before run.
    const ABSENT: &str = "native_company_policy_v2.absent";
    const FINALIZED: &str = "native_company_policy_v2.finalized";
    const MISMATCH: &str = "native_company_policy_v2.profile_mismatch";

    fn reviewed() {
        for (source, expected) in [
            (FINALIZER, FINALIZER_SHA),
            (CLASSIFIER, CLASSIFIER_SHA),
            (CAPTURE, CAPTURE_SHA),
            (
                OLD_CAPTURE,
                "d5d2c45c691383ddde9ac213d9fe2674db0750f3ceabdc733eaaf2a570d937e3",
            ),
        ] {
            assert_eq!(
                expected.len(),
                64,
                "missing independently reviewed input pin"
            );
            assert_eq!(hex::encode(Sha256::digest(source.as_bytes())), expected);
        }
        assert_eq!(CLASSIFIER, APP_CLASSIFIER);
        assert_eq!(
            SUCCESSOR_PLAIN.len(),
            64,
            "missing independently reviewed observed profile"
        );
    }
    async fn metadata(pool: &PgPool, source: &'static str) -> (Value, String, Option<bool>) {
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(tx.as_mut())
            .await
            .unwrap();
        sqlx::raw_sql(CLASSIFIER_SESSION)
            .execute(tx.as_mut())
            .await
            .unwrap();
        let result = sqlx::query_as(sqlx::AssertSqlSafe(source))
            .fetch_one(tx.as_mut())
            .await
            .unwrap();
        tx.rollback().await.unwrap();
        result
    }
    async fn install(tx: &mut Transaction<'_, Postgres>) -> Result<(), sqlx::Error> {
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='120s'")
            .execute(tx.as_mut()).await?;
        sqlx::raw_sql(FINALIZER).execute(tx.as_mut()).await?;
        sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE")
            .execute(tx.as_mut())
            .await?;
        Ok(())
    }
    fn upgrade_delta(before: &BTreeMap<String, String>, after: &BTreeMap<String, String>) {
        assert!(before.keys().eq(after.keys()));
        for (table, old) in before {
            let added =
                added_rows(old, &after[table]).expect("upgrade rewrote acknowledged history");
            if table == "ont_builtin_catalog_allowlist" {
                assert_eq!(added.len(), 1);
                assert_eq!(added[0]["catalog_version"], "native-people-directory-v1");
                assert_eq!(
                    added[0]["manifest_digest"],
                    "\\x591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e"
                );
            } else {
                assert!(added.is_empty(), "unexpected upgrade effect in {table}");
            }
        }
    }
    fn effect_assignment(
        terminal: &NativePolicyTerminalView,
        revoke: bool,
    ) -> PolicyAssignmentExpectationV1 {
        match (&terminal.outcome, revoke) {
            (
                NativePolicyOutcome::Committed(NativePolicyEffect::Granted { assignment, .. }),
                false,
            )
            | (
                NativePolicyOutcome::Committed(NativePolicyEffect::Revoked { assignment, .. }),
                true,
            ) => assignment.expectation,
            _ => panic!("requested Payroll operation did not commit exact effect"),
        }
    }

    #[sqlx::test(migrations = false)]
    async fn populated_payroll_predecessor_upgrade_preserves_history_and_compatible_readmission(
        pool: PgPool,
    ) {
        reviewed();
        // Frozen existing v1 installer/classifier pins are intentionally retained.
        let (app, key, state) = configured_fixture(&pool, true).await;
        let mut runtime_cleanup = None;
        let mut startup_cleanup = None;
        let mut fresh_cleanup = None;
        let outcome = AssertUnwindSafe(async {
            let (app, operator, cookies, startup, _) = designated_fixture(&pool, app).await;
            startup_cleanup = Some(startup);
            let config = account_browser_config(&pool, app._artifacts.root.clone(), &key);
            assert_eq!(
                ready_status(&state).await,
                StatusCode::OK,
                "reader must support predecessor before upgrade"
            );
            let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
            runtime_cleanup = Some(runtime.clone());
            assert_eq!(classified(&runtime, CLASSIFIER).await, ABSENT);
            assert_eq!(
                policy_classified(&runtime).await,
                "native_company_policy.finalized"
            );
            let (verifier, issuer, ttl) = bindings(&config);
            let store =
                PgOrgStore::new(runtime.clone()).with_native_account_policy(verifier, issuer, ttl);
            let policy = CompanyPolicy::new().unwrap();
            let actor = AccountId::from_uuid(operator.account).unwrap();
            let proof = proof(&app, &cookies).await;
            let mutation =
                AccountEnrollmentCredentials::for_mutation(&cookies.0[ACCESS], &proof).unwrap();
            let read = read_credentials(&cookies);
            let mut history = Vec::new();
            let mut pending = None;
            // A separate second Company preserves the pending grant's epoch.
            for index in 0..2 {
                let command = Uuid::new_v4();
                let input = enrollment(command, operator.account);
                let created = committed(
                    &submit(&app, &cookies, &proof, &input).await,
                    StatusCode::CREATED,
                    command,
                    operator.account,
                    false,
                );
                durable(&pool, &created, &input, operator.account).await;
                let company = OrgId::from_uuid(created.company);
                let command =
                    NativeCompanyBusinessCommandV1::install(Uuid::new_v4(), company, 1).unwrap();
                let done = submit_native_policy_command(
                    &store,
                    &policy,
                    &mutation,
                    &command,
                    &TraceContext::generate(),
                )
                .await
                .unwrap();
                assert!(done.inserted);
                assert!(matches!(
                    done.terminal.outcome,
                    NativePolicyOutcome::Committed(NativePolicyEffect::Installed { .. })
                ));
                history.push((
                    NativePolicyCommandRef::from_command(&command),
                    done.terminal,
                ));
                let now: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
                let until = time::OffsetDateTime::from_unix_timestamp(
                    (now.unix_timestamp() / 60 + 1440) * 60,
                )
                .unwrap();
                let grant = NativeCompanyBusinessCommandV1::grant(
                    Uuid::new_v4(),
                    company,
                    2,
                    actor,
                    None,
                    until,
                )
                .unwrap();
                if index == 1 {
                    let accepted = accept_native_policy_command(
                        &store,
                        &policy,
                        &mutation,
                        &grant,
                        &TraceContext::generate(),
                    )
                    .await
                    .unwrap();
                    assert!(accepted.inserted);
                    assert!(matches!(
                        accepted.status,
                        NativePolicyStatus::AcceptedPending(_)
                    ));
                    pending = Some((grant, accepted.status));
                } else {
                    let done = submit_native_policy_command(
                        &store,
                        &policy,
                        &mutation,
                        &grant,
                        &TraceContext::generate(),
                    )
                    .await
                    .unwrap();
                    let assignment = effect_assignment(&done.terminal, false);
                    history.push((NativePolicyCommandRef::from_command(&grant), done.terminal));
                    let revoke = NativeCompanyBusinessCommandV1::revoke(
                        Uuid::new_v4(),
                        company,
                        3,
                        assignment,
                    )
                    .unwrap();
                    let done = submit_native_policy_command(
                        &store,
                        &policy,
                        &mutation,
                        &revoke,
                        &TraceContext::generate(),
                    )
                    .await
                    .unwrap();
                    let assignment = effect_assignment(&done.terminal, true);
                    history.push((NativePolicyCommandRef::from_command(&revoke), done.terminal));
                    let grant = NativeCompanyBusinessCommandV1::grant(
                        Uuid::new_v4(),
                        company,
                        4,
                        actor,
                        Some(assignment),
                        until,
                    )
                    .unwrap();
                    let done = submit_native_policy_command(
                        &store,
                        &policy,
                        &mutation,
                        &grant,
                        &TraceContext::generate(),
                    )
                    .await
                    .unwrap();
                    effect_assignment(&done.terminal, false);
                    history.push((NativePolicyCommandRef::from_command(&grant), done.terminal));
                }
            }
            assert_eq!(history.len(), 5);
            for (selector, terminal) in &history {
                assert_eq!(
                    native_policy_command_status(&store, &policy, &read, *selector)
                        .await
                        .unwrap(),
                    NativePolicyStatus::Terminal(terminal.clone())
                );
            }
            let before = all_rows(&pool).await;
            let old_metadata = metadata(&runtime, OLD_CAPTURE).await;
            assert_eq!(
                old_metadata.1,
                "3755792f52a4f78236a70e509f4f1546588049daa53f7545d8a101c74684d843"
            );
            assert_eq!(old_metadata.2, Some(true));
            // Real full finalizer succeeds, then a known post-DDL failure rolls
            // the entire transaction back; no copied installer fragments.
            let mut tx = pool.begin().await.unwrap();
            install(&mut tx).await.unwrap();
            let error = sqlx::query("SELECT 1/0")
                .execute(tx.as_mut())
                .await
                .unwrap_err();
            assert_eq!(
                error.as_database_error().unwrap().code().as_deref(),
                Some("22012")
            );
            tx.rollback().await.unwrap();
            assert!(before == all_rows(&pool).await);
            assert_eq!(metadata(&runtime, OLD_CAPTURE).await, old_metadata);
            assert_eq!(classified(&runtime, CLASSIFIER).await, ABSENT);
            assert_eq!(ready_status(&state).await, StatusCode::OK);
            let mut tx = pool.begin().await.unwrap();
            install(&mut tx).await.unwrap();
            tx.commit().await.unwrap();
            let upgraded = all_rows(&pool).await;
            upgrade_delta(&before, &upgraded);
            let successor = metadata(&runtime, CAPTURE).await;
            assert_eq!(successor.1, SUCCESSOR_PLAIN);
            assert_eq!(successor.2, Some(true));
            assert_eq!(classified(&runtime, CLASSIFIER).await, FINALIZED);
            assert_eq!(
                policy_classified(&runtime).await,
                "native_company_policy.profile_mismatch",
                "v1-only classifier must refuse successor"
            );
            let mut tx = pool.begin().await.unwrap();
            install(&mut tx).await.unwrap();
            tx.commit().await.unwrap();
            assert!(
                upgraded == all_rows(&pool).await,
                "idempotent install changed rows"
            );
            assert_eq!(metadata(&runtime, CAPTURE).await, successor);
            let fresh = AppState::from_config(config).await.unwrap();
            fresh_cleanup = Some(fresh.clone());
            assert_eq!(ready_status(&fresh).await, StatusCode::OK);
            for (selector, terminal) in &history {
                assert_eq!(
                    native_policy_command_status(&store, &policy, &read, *selector)
                        .await
                        .unwrap(),
                    NativePolicyStatus::Terminal(terminal.clone())
                );
                let replay = execute_native_policy_command(
                    &store,
                    &policy,
                    &mutation,
                    *selector,
                    &TraceContext::generate(),
                )
                .await
                .unwrap();
                assert!(!replay.inserted);
                assert_eq!(&replay.terminal, terminal);
                assert_eq!(
                    read_company_identity(&store, &policy, &read, selector.company())
                        .await
                        .unwrap()
                        .org_id,
                    selector.company()
                );
            }
            let (command, status) = pending.unwrap();
            let selector = NativePolicyCommandRef::from_command(&command);
            assert_eq!(
                native_policy_command_status(&store, &policy, &read, selector)
                    .await
                    .unwrap(),
                status
            );
            assert!(
                upgraded == all_rows(&pool).await,
                "readmission/reopening changed acknowledged rows"
            );
            let done = execute_native_policy_command(
                &store,
                &policy,
                &mutation,
                selector,
                &TraceContext::generate(),
            )
            .await
            .unwrap();
            assert!(done.inserted);
            effect_assignment(&done.terminal, false);
            assert_eq!(done.terminal.accepted.input, (&command).into());
            let NativePolicyStatus::AcceptedPending(accepted) = status else {
                panic!("pre-upgrade request was not durably pending");
            };
            assert_eq!(done.terminal.accepted, accepted);
            assert_eq!((done.terminal.epoch_before,done.terminal.epoch_after),(2,3));
            let after=all_rows(&pool).await;
            assert!(upgraded.keys().eq(after.keys()),"table census changed");
            let effects=BTreeMap::from([
                ("native_company_policy_receipts_v1",1), ("audit_events",1),
                ("policy_assignment_revisions",1), ("policy_roles",1),
                ("policy_role_revisions",1), ("policy_capability_clauses",1),
                ("policy_capability_clause_fields",18), ("user_role_assignments",1),
            ]);
            for table in effects.keys() {assert!(upgraded.contains_key(*table));}
            for (table,old) in &upgraded {
                if table=="company_authority_heads" {continue;}
                let added=added_rows(old,&after[table]).expect("pending execution rewrote prior rows");
                assert_eq!(added.len(),effects.get(table.as_str()).copied().unwrap_or(0),"unexpected pending effect in {table}");
            }
            let heads=|raw:&str|->BTreeMap<String,Value>{
                serde_json::from_str::<Vec<Value>>(raw).unwrap().into_iter()
                    .map(|v|(v["org_id"].as_str().unwrap().to_owned(),v)).collect()
            };
            let before_heads=heads(&upgraded["company_authority_heads"]);
            let mut after_heads=heads(&after["company_authority_heads"]);
            let company=selector.company().as_uuid().to_string();
            let head=after_heads.get_mut(&company).unwrap();
            assert_eq!(head["epoch"],json!(3));
            assert_eq!(head["current_policy_receipt_id"],json!(done.terminal.receipt_id));
            assert_eq!(before_heads[&company]["epoch"],json!(2));
            head["epoch"]=before_heads[&company]["epoch"].clone();
            head["current_policy_receipt_id"]=before_heads[&company]["current_policy_receipt_id"].clone();
            assert_eq!(after_heads,before_heads,"pending command changed unrelated head fields");
            let receipt_bound:bool=sqlx::query_scalar(
                "SELECT r.receipt_id=$4 AND r.codec_version=1 AND r.outcome='COMMITTED' AND r.result_code='granted' AND r.epoch_before=2 AND r.epoch_after=3 AND r.committed_epoch=3 AND r.intake_receipt_id=$5 AND r.intake_receipt_id=i.intake_receipt_id AND r.input_digest=i.input_digest AND i.input_bytes=$6 AND r.effect_xid<>i.acceptance_xid FROM public.native_company_policy_receipts_v1 r JOIN public.native_company_policy_inputs_v1 i USING(actor_account_id,command_id,org_id) WHERE r.actor_account_id=$1 AND r.command_id=$2 AND r.org_id=$3")
                .bind(*actor.as_uuid()).bind(command.command_id()).bind(*command.company().as_uuid())
                .bind(done.terminal.receipt_id).bind(accepted.intake_receipt_id).bind(command.encode(actor))
                .fetch_one(&pool).await.unwrap();
            assert!(receipt_bound,"reported pending completion has no exact stored terminal");
            assert_eq!(native_policy_command_status(&store,&policy,&read,selector).await.unwrap(),
                NativePolicyStatus::Terminal(done.terminal.clone()));
            let replay=execute_native_policy_command(&store,&policy,&mutation,selector,&TraceContext::generate()).await.unwrap();
            assert!(!replay.inserted);assert_eq!(replay.terminal,done.terminal);
            assert!(after==all_rows(&pool).await,"completed pending status/replay wrote durable effects");
        })
        .catch_unwind()
        .await;
        if let Some(fresh) = fresh_cleanup {
            fresh.shutdown_realtime().await;
        }
        if let Some(runtime) = runtime_cleanup {
            runtime.close().await;
        }
        if let Some(startup) = startup_cleanup {
            startup.close().await;
        }
        close_states(&[state], outcome).await;
    }

    #[sqlx::test(migrations = false)]
    async fn partial_successor_never_falls_back_to_predecessor_startup(pool: PgPool) {
        reviewed();
        let (app, key, state) = configured_fixture(&pool, true).await;
        let config = account_browser_config(&pool, app._artifacts.root.clone(), &key);
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let before = all_rows(&pool).await;
        let original = metadata(&runtime, OLD_CAPTURE).await;
        let outcome=AssertUnwindSafe(async {
            assert_eq!(classified(&runtime,CLASSIFIER).await,ABSENT);
            assert_eq!(ready_status(&state).await,StatusCode::OK);
            // Intentional malformed metadata fixture, no business truth.
            sqlx::raw_sql("CREATE FUNCTION public.native_company_policy_codec_v2(p_input bytea) RETURNS smallint LANGUAGE sql IMMUTABLE AS 'SELECT 1::smallint'")
                .execute(&pool).await.unwrap();
            let checked=AssertUnwindSafe(async {
                assert!(sqlx::query_scalar::<_,bool>("SELECT to_regprocedure('public.native_company_policy_codec_v2(bytea)') IS NOT NULL").fetch_one(&pool).await.unwrap());
                assert_eq!(classified(&runtime,CLASSIFIER).await,MISMATCH);
                startup_refused(config.clone(),MISMATCH).await;
                assert_eq!(ready_status(&state).await,StatusCode::SERVICE_UNAVAILABLE);
                let mut tx=pool.begin().await.unwrap();
                let error=install(&mut tx).await.unwrap_err();
                let database=error.as_database_error().unwrap();
                assert_eq!(database.code().as_deref(),Some("P0001"));assert_eq!(database.message(),MISMATCH);
                tx.rollback().await.unwrap();
            }).catch_unwind().await;
            sqlx::raw_sql("DROP FUNCTION public.native_company_policy_codec_v2(bytea)").execute(&pool).await.unwrap();
            assert_eq!(metadata(&runtime,OLD_CAPTURE).await,original);
            assert!(before==all_rows(&pool).await);
            assert_eq!(classified(&runtime,CLASSIFIER).await,ABSENT);
            assert_eq!(ready_status(&state).await,StatusCode::OK);
            if let Err(panic)=checked {std::panic::resume_unwind(panic);}
        }).catch_unwind().await;
        runtime.close().await;
        close_states(&[state], outcome).await;
    }

    #[sqlx::test(migrations = false)]
    async fn successor_finalizer_rejects_body_system_acl_and_unknown_overload_drift(pool: PgPool) {
        reviewed();
        prepare_policy_ready_database(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        install(&mut tx).await.unwrap();
        tx.commit().await.unwrap();
        let before = all_rows(&pool).await;
        let original = metadata(&pool, CAPTURE).await;
        assert_eq!(original.1, SUCCESSOR_PLAIN);
        assert_eq!(original.2, Some(true));
        for fault in [
            "CREATE OR REPLACE FUNCTION public.native_company_policy_codec_v2(p_input bytea) RETURNS smallint LANGUAGE sql IMMUTABLE AS 'SELECT 1::smallint'",
            "GRANT SELECT(xmin) ON public.audit_events TO PUBLIC",
            "CREATE FUNCTION public.native_company_policy_codec_v2(text) RETURNS smallint LANGUAGE sql IMMUTABLE AS 'SELECT 1::smallint'",
        ] {
            let mut tx = pool.begin().await.unwrap();
            sqlx::raw_sql(sqlx::AssertSqlSafe(fault))
                .execute(tx.as_mut())
                .await
                .unwrap();
            let checked = AssertUnwindSafe(async {
                sqlx::raw_sql(CLASSIFIER_SESSION)
                    .execute(tx.as_mut())
                    .await
                    .unwrap();
                let changed: (Value, String, Option<bool>) =
                    sqlx::query_as(sqlx::AssertSqlSafe(CAPTURE))
                        .fetch_one(tx.as_mut())
                        .await
                        .unwrap();
                assert_ne!(changed.0, original.0, "fault absent from capture");
                assert_ne!(changed.1, original.1);
                let status: String = sqlx::query_scalar(sqlx::AssertSqlSafe(CLASSIFIER))
                    .fetch_one(tx.as_mut())
                    .await
                    .unwrap();
                assert_eq!(status, MISMATCH);
                // Do not SET TRANSACTION after the witness query; finalizer's
                // actual statement executes within the default READ COMMITTED.
                sqlx::raw_sql("SET LOCAL statement_timeout='120s'")
                    .execute(tx.as_mut())
                    .await
                    .unwrap();
                let error = sqlx::raw_sql(FINALIZER)
                    .execute(tx.as_mut())
                    .await
                    .unwrap_err();
                let database = error.as_database_error().unwrap();
                assert_eq!(database.code().as_deref(), Some("P0001"));
                assert_eq!(database.message(), MISMATCH);
            })
            .catch_unwind()
            .await;
            tx.rollback().await.unwrap();
            assert!(before == all_rows(&pool).await);
            assert_eq!(metadata(&pool, CAPTURE).await, original);
            if let Err(panic) = checked {
                std::panic::resume_unwind(panic);
            }
        }
    }
    #[sqlx::test(migrations = false)]
    async fn exact_observer_predecessor_finalizer_is_atomic_and_idempotent(pool: PgPool) {
        reviewed();
        assert_eq!(
            SUCCESSOR_OBSERVER.len(),
            64,
            "missing independently reviewed observer profile"
        );
        assert_eq!(
            hex::encode(Sha256::digest(OBSERVER.as_bytes())),
            "ffe7038b43de0207d6ae3e3ce0487498edc4d061c21e4249abacc91abaacdbbc"
        );
        prepare_policy_ready_database(&pool).await;
        let before = all_rows(&pool).await;
        let original = metadata(&pool, OLD_CAPTURE).await;
        let mut tx = pool.begin().await.unwrap();
        let result=AssertUnwindSafe(async {
            sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='120s'").execute(tx.as_mut()).await.unwrap();
            sqlx::raw_sql(OBSERVER).execute(tx.as_mut()).await.unwrap();
            let predecessor:(Value,String,Option<bool>)=sqlx::query_as(sqlx::AssertSqlSafe(OLD_CAPTURE)).fetch_one(tx.as_mut()).await.unwrap();
            assert_eq!(predecessor.1,"9bde6410d51f8b665e5ca4400820e6d8a549a6c19b7b4d84feb26ef32c4bd604");
            assert_eq!(predecessor.2,Some(true));
            sqlx::raw_sql(FINALIZER).execute(tx.as_mut()).await.unwrap();
            sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE").execute(tx.as_mut()).await.unwrap();
            let first:(Value,String,Option<bool>)=sqlx::query_as(sqlx::AssertSqlSafe(CAPTURE)).fetch_one(tx.as_mut()).await.unwrap();
            assert_eq!(first.1,SUCCESSOR_OBSERVER);assert_eq!(first.2,Some(true));
            sqlx::raw_sql(FINALIZER).execute(tx.as_mut()).await.unwrap();
            let second:(Value,String,Option<bool>)=sqlx::query_as(sqlx::AssertSqlSafe(CAPTURE)).fetch_one(tx.as_mut()).await.unwrap();
            assert_eq!(first,second);
        }).catch_unwind().await;
        tx.rollback().await.unwrap();
        assert_eq!(metadata(&pool, OLD_CAPTURE).await, original);
        assert!(before == all_rows(&pool).await);
        assert!(sqlx::query_scalar::<_,bool>("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_durability_observer') AND to_regprocedure('public.console_durability_observation_v1(name,oid)') IS NULL").fetch_one(&pool).await.unwrap());
        if let Err(panic) = result {
            std::panic::resume_unwind(panic);
        }
    }
    include!("native_policy_successor_unknown_namespace.rs");
}
