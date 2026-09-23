// Additive proposal: real application/store/SQL boundary. No People form dependency.
mod native_people_codec2_install_probe {
    use super::*;
    use console_identity_application::company_policy::{
        AccountId,
        people_business::NativePeoplePolicyCommandV1,
        workflow::{
            NativePolicyCommand, NativePolicyEffect, NativePolicyStatus, NativePolicyTerminalView,
            execute_native_policy_command,
        },
    };

    // Extend only this probe's prerequisite. The inherited v1 installer and
    // configured_fixture(true) remain unchanged for predecessor/upgrade tests.
    async fn configured_successor_fixture(pool: &PgPool) -> (Fixture, SigningKey, AppState) {
        const FINALIZER: &str =
            include_str!("../../../../ops/postgres-finalize-native-company-policy-v2.sql");
        const CLASSIFIER: &str =
            include_str!("../../../../ops/postgres-native-company-policy-v2-custody-state.sql");
        assert_eq!(
            hex::encode(Sha256::digest(FINALIZER.as_bytes())),
            "ec945607e209b93843116ae2b2a20772797dce38ff7884fb96081f09651f7d8e",
            "actual reviewed successor finalizer changed"
        );
        assert_eq!(
            hex::encode(Sha256::digest(CLASSIFIER.as_bytes())),
            "e507d75f446ad8d3e0befe9321d94731a1a2cc9b3e0e9c0306c1459a098a53cc",
            "actual reviewed successor classifier changed"
        );
        assert_eq!(
            CLASSIFIER,
            include_str!("../../src/native_company_policy_v2_custody_state.sql"),
            "successor serving classifier copies drifted"
        );
        prepare_policy_ready_database(pool).await;
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='120s'")
            .execute(tx.as_mut()).await.unwrap();
        sqlx::raw_sql(FINALIZER)
            .execute(tx.as_mut())
            .await
            .expect("actual reviewed successor finalizer prerequisite");
        sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE")
            .execute(tx.as_mut())
            .await
            .unwrap();
        tx.commit().await.unwrap();
        let runtime = login_test_pool(pool, TestDatabaseLogin::Business).await;
        let checked = AssertUnwindSafe(async {
            assert_eq!(
                classified(&runtime, CLASSIFIER).await,
                "native_company_policy_v2.finalized"
            );
        })
        .catch_unwind()
        .await;
        runtime.close().await;
        if let Err(panic) = checked {
            std::panic::resume_unwind(panic);
        }
        // AppState must verify successor custody on a fresh startup before any
        // actual Account/Company/Payroll/People workflow runs.
        let artifacts = Artifacts::new();
        let key = SigningKey::random(&mut OsRng);
        let state =
            AppState::from_config(account_browser_config(pool, artifacts.root.clone(), &key))
                .await
                .unwrap();
        let app = Fixture {
            service: build_router(state.clone()),
            _artifacts: artifacts,
            pool: pool.clone(),
        };
        (app, key, state)
    }

    fn delta(
        before: &BTreeMap<String, String>,
        after: &BTreeMap<String, String>,
        expected: &[(&str, usize)],
        mutable: &[&str],
    ) {
        assert!(before.keys().eq(after.keys()), "table census changed");
        let expected: BTreeMap<_, _> = expected.iter().copied().collect();
        for table in expected.keys() {
            assert!(before.contains_key(*table));
        }
        for (table, old) in before {
            if mutable.contains(&table.as_str()) {
                continue;
            }
            let added = added_rows(old, &after[table])
                .unwrap_or_else(|| panic!("acknowledged history changed in {table}"));
            assert_eq!(
                added.len(),
                expected.get(table.as_str()).copied().unwrap_or(0),
                "unexpected effects in {table}"
            );
        }
    }
    fn heads(raw: &str) -> BTreeMap<String, Value> {
        serde_json::from_str::<Vec<Value>>(raw)
            .unwrap()
            .into_iter()
            .map(|v| (v["org_id"].as_str().unwrap().to_owned(), v))
            .collect()
    }

    async fn install(
        pool: &PgPool,
        store: &PgOrgStore,
        policy: &CompanyPolicy,
        mutation: &AccountEnrollmentCredentials,
        actor: AccountId,
        input: NativePolicyCommand,
        epoch: u64,
        predecessor: Option<Uuid>,
    ) -> NativePolicyTerminalView {
        let selector = NativePolicyCommandRef::from_command(&input);
        let before = all_rows(pool).await;
        let trace = TraceContext::generate();
        let result = accept_native_policy_command(store, policy, mutation, &input, &trace).await;
        let acceptance = match result {
            Ok(value) => value,
            Err(NativePolicyWorkflowError::InvalidInput) if input.codec_version() == 2 => {
                assert!(
                    before == all_rows(pool).await,
                    "failed codec2 accept wrote durable effects"
                );
                // Only this exact owner error, after the real Payroll control,
                // qualifies as the intended RED. Missing schema/functions do not.
                panic!(
                    "PEOPLE_CODEC2_OWNER_RED: existing owner rejects valid codec2 install as InvalidInput"
                );
            }
            Err(error) => panic!("prerequisite or unrelated owner failure: {error:?}"),
        };
        assert!(acceptance.inserted);
        let pending = match acceptance.status {
            NativePolicyStatus::AcceptedPending(value) => value,
            _ => panic!("install acceptance was not pending"),
        };
        assert_eq!(pending.input, input);
        let accepted = all_rows(pool).await;
        delta(
            &before,
            &accepted,
            &[("native_company_policy_inputs_v1", 1), ("audit_events", 1)],
            &[],
        );
        let codec = input.codec_version();
        let encoded = input.encode(actor);
        let stored: (i16, Vec<u8>, Vec<u8>) = sqlx::query_as(
            "SELECT codec_version,input_bytes,input_digest FROM public.native_company_policy_inputs_v1 WHERE actor_account_id=$1 AND command_id=$2 AND org_id=$3")
            .bind(*actor.as_uuid()).bind(selector.command_id()).bind(*selector.company().as_uuid())
            .fetch_one(pool).await.unwrap();
        assert_eq!(stored.0, codec);
        assert!(stored.1 == encoded, "stored command bytes differ");
        assert!(
            stored.2 == Sha256::digest(&encoded).as_slice(),
            "wrong command digest"
        );
        let execution = execute_native_policy_command(store, policy, mutation, selector, &trace)
            .await
            .expect("real execute owner must commit the accepted install");
        assert!(execution.inserted);
        let terminal = execution.terminal;
        assert_eq!(terminal.accepted, pending);
        assert_eq!(
            (terminal.epoch_before, terminal.epoch_after),
            (epoch, epoch + 1)
        );
        match &terminal.outcome {
            NativePolicyOutcome::Committed(NativePolicyEffect::Installed { object_type_id }) => {
                assert!(!object_type_id.is_nil())
            }
            _ => panic!("install did not commit exact Installed effect"),
        }
        let properties = if codec == 1 { 18 } else { 6 };
        let actions = if codec == 1 { 1 } else { 2 };
        let after = all_rows(pool).await;
        delta(
            &accepted,
            &after,
            &[
                ("native_company_policy_receipts_v1", 1),
                ("audit_events", 2),
                ("ont_object_type_key_revisions", 1),
                ("ont_object_types", 1),
                ("ont_property_defs", properties),
                ("ont_action_types", actions),
                ("ont_builtin_catalog_installs", 1),
                ("native_company_catalog_installs", 1),
                ("native_company_object_refs", 1),
                ("native_company_action_refs", actions),
                ("native_company_property_refs", properties),
            ],
            &["company_authority_heads"],
        );
        // All role/assignment/clause tables are covered by the complete census
        // with expected zero additions. Installation cannot grant access.
        let mut after_heads = heads(&after["company_authority_heads"]);
        let head = after_heads
            .get_mut(&selector.company().as_uuid().to_string())
            .unwrap();
        assert_eq!(head["epoch"], json!(epoch + 1));
        assert_eq!(
            head["current_policy_receipt_id"],
            json!(terminal.receipt_id)
        );
        head["epoch"] = json!(epoch);
        head["current_policy_receipt_id"] = json!(predecessor);
        assert_eq!(after_heads, heads(&accepted["company_authority_heads"]));
        let binding: bool = sqlx::query_scalar(
            "SELECT r.codec_version=$4 AND r.predecessor_receipt_id IS NOT DISTINCT FROM $5 AND r.epoch_before=$6 AND r.epoch_after=$6+1 AND r.committed_epoch=r.epoch_after AND r.input_digest=i.input_digest AND r.intake_receipt_id=i.intake_receipt_id AND r.effect_xid<>i.acceptance_xid AND r.catalog_version=CASE $4 WHEN 1 THEN 'native-payroll-collection-read-v1' ELSE 'native-people-directory-v1' END FROM public.native_company_policy_receipts_v1 r JOIN public.native_company_policy_inputs_v1 i USING(actor_account_id,command_id,org_id) WHERE r.actor_account_id=$1 AND r.command_id=$2 AND r.org_id=$3")
            .bind(*actor.as_uuid()).bind(selector.command_id()).bind(*selector.company().as_uuid())
            .bind(codec).bind(predecessor).bind(epoch as i64).fetch_one(pool).await.unwrap();
        assert!(binding, "receipt/input/history binding failed");
        let replay = execute_native_policy_command(store, policy, mutation, selector, &trace)
            .await
            .expect("committed install must reopen through real execute owner");
        assert!(!replay.inserted);
        assert_eq!(replay.terminal, terminal);
        assert!(
            after == all_rows(pool).await,
            "replay changed durable state"
        );
        terminal
    }

    #[sqlx::test(migrations = false)]
    async fn valid_people_codec2_install_commits_after_real_payroll_control(pool: PgPool) {
        // Exact v1 prerequisite, reviewed v2 finalizer, then verified App startup.
        let (app, key, state) = configured_successor_fixture(&pool).await;
        let (app, operator, cookies, startup, _) = designated_fixture(&pool, app).await;
        let (verifier, issuer, ttl) = bindings(&account_browser_config(
            &pool,
            app._artifacts.root.clone(),
            &key,
        ));
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let login: (String, String, bool, bool) = sqlx::query_as(
            "SELECT session_user::text,current_user::text,rolsuper,rolbypassrls FROM pg_roles WHERE rolname=current_user")
            .fetch_one(&runtime).await.unwrap();
        assert_eq!(
            login,
            ("console_rt".into(), "console_rt".into(), false, false)
        );
        let store =
            PgOrgStore::new(runtime.clone()).with_native_account_policy(verifier, issuer, ttl);
        let policy = CompanyPolicy::new().unwrap();
        let outcome = AssertUnwindSafe(async {
            let (admin, _) = enrolled(&app).await;
            assert_ne!(admin.account, operator.account);
            let command = Uuid::new_v4();
            let enrollment_input = enrollment(command, admin.account);
            let enrollment_proof = proof(&app, &cookies).await;
            let created = committed(&submit(&app, &cookies, &enrollment_proof, &enrollment_input).await,
                StatusCode::CREATED, command, admin.account, false);
            durable(&pool, &created, &enrollment_input, operator.account).await;
            let actor = AccountId::from_uuid(operator.account).unwrap();
            let company = OrgId::from_uuid(created.company);
            // Real browser HTTP owner issues a session-bound proof. Neither
            // contract nor verifier binds this proof to action/catalog/command.
            let csrf = proof(&app, &cookies).await;
            let mutation = AccountEnrollmentCredentials::for_mutation(&cookies.0[ACCESS], &csrf).unwrap();
            let payroll = NativePolicyCommand::Payroll(
                NativeCompanyBusinessCommandV1::install(Uuid::new_v4(), company, 1).unwrap());
            let payroll = install(&pool, &store, &policy, &mutation, actor, payroll, 1, None).await;
            eprintln!("PEOPLE_CODEC2_PAYROLL_CONTROL_OK: real Payroll install committed and replayed with same HTTP proof");
            let people = NativePolicyCommand::People(
                NativePeoplePolicyCommandV1::install(Uuid::new_v4(), company, 2).unwrap());
            let people = install(&pool, &store, &policy, &mutation, actor, people, 2, Some(payroll.receipt_id)).await;
            assert_ne!(people.receipt_id, payroll.receipt_id);
        }).catch_unwind().await;
        runtime.close().await;
        startup.close().await;
        state.shutdown_realtime().await;
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }
}
