// Included inside audit_account_transition. Frozen real audit228 predecessor;
// current operator composition remains owned by the existing parent.
mod business_session_transition {
    use super::*;
    use console_platform_auth::account::live_account_session_in_tx;
    use console_platform_auth::{JwtSettings, JwtVerifier};
    use sqlx::Acquire;

    const NAMES: &[&str] = &[
        "account_session_shared_material_v1",
        "auth_account_session_shared_material_v1",
    ];
    const FROZEN: &[(&str, &str)] = &[
        (
            include_str!("fixtures/account-audit228-root-20aafabb.sql"),
            "bf7a330f5010482099e48cfe5eb9b021d3e04cfe5965b9d16e5e5983564ef7a1",
        ),
        (
            include_str!("fixtures/account-audit228-credentials-20aafabb.sql"),
            "a14bebd6b6f0b02c4864bf771dc55a1280a9b098c006dca197a8334e7c4d4ea7",
        ),
        (
            include_str!("fixtures/account-audit228-verify-20aafabb.sql"),
            "a18c9b0c3161a116b64622c56fc02e05aabf0f312f6451f83e1766d4069b21c6",
        ),
    ];

    async fn audited_state(connection: &mut PgConnection) {
        operator(connection).await;
        for (source, digest, expected) in [
            (
                include_str!("fixtures/account-audit228-root-state-20aafabb.sql"),
                "487099095e922f103228c73a36eb61f477d71f0c4262a31777fd8b0bd2f96d4a",
                "account_custody.native_finalized",
            ),
            (
                include_str!("fixtures/account-audit228-credential-state-20aafabb.sql"),
                "fee02682fcb32f5d8c6f4a772c3ae4623617e8ffbcdd77eb39003a37fcaea875",
                "account_credentials.native_finalized",
            ),
        ] {
            assert!(
                hex::encode(Sha256::digest(source)) == digest,
                "frozen audit228 classifier identity"
            );
            let observed: String = sqlx::query_scalar(sqlx::AssertSqlSafe(source))
                .fetch_one(&mut *connection)
                .await
                .unwrap();
            assert!(
                observed == expected,
                "exact real audit228 predecessor required"
            );
        }
        exact_actor_fk(connection).await;
        let absent: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_proc WHERE pronamespace='public'::regnamespace AND proname=ANY($1))")
            .bind(NAMES).fetch_one(&mut *connection).await.unwrap();
        assert!(
            absent,
            "old profile must lack every overload of both new names"
        );
    }

    async fn audited228(pool: &PgPool) -> (Fixture, AppConfig, SigningKey, Attempt, Cookies) {
        prior228(pool).await;
        let mut tx = pool.begin().await.unwrap();
        operator(&mut tx).await;
        for (source, digest) in FROZEN {
            assert!(
                hex::encode(Sha256::digest(source)) == *digest,
                "frozen actual audit228 operator source"
            );
            sqlx::raw_sql(*source).execute(&mut *tx).await.unwrap();
        }
        audited_state(&mut tx).await;
        tx.commit().await.unwrap();
        seed_terms(pool).await;
        let artifacts = Artifacts::new();
        let key = SigningKey::random(&mut OsRng);
        let config = account_browser_config(pool, artifacts.root.clone(), &key);
        let business = login_test_pool(pool, TestDatabaseLogin::Business).await;
        let auth = login_test_pool(pool, TestDatabaseLogin::Auth).await;
        // Existing historical fixture DI only. No fabricated Account/grant/session.
        let injected = AppState::new(
            config.clone(),
            console_app::DatabaseDependency::Postgres(business.clone()),
        )
        .unwrap()
        .with_auth_database(auth.clone());
        let app = Fixture {
            service: build_router(injected.clone()),
            _artifacts: artifacts,
            pool: pool.clone(),
        };
        let (attempt, cookies) = historical_enrolled(&app, &key).await;
        assert_committed(pool, &attempt).await;
        designate_real_account(pool, attempt.account).await;
        let initialized: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM public.users WHERE id=$1) AND EXISTS(SELECT 1 FROM public.deployment_operator_head h JOIN public.deployment_operator_receipts r ON r.receipt_id=h.receipt_id WHERE h.account_id=$1 AND r.account_id=$1 AND r.kind='DESIGNATE' AND h.revision=1)")
            .bind(attempt.account).fetch_one(pool).await.unwrap();
        assert!(
            initialized,
            "real initialized designation receipt, no legacy User identity"
        );
        injected.shutdown_realtime().await;
        auth.close().await;
        business.close().await;
        (app, config, key, attempt, cookies)
    }

    // Vetted additions are actual pg_proc rows collected by an independent exact
    // signature/owner/config/ACL query below. All previous rows remain unprojected.
    fn strip_exact_additions(before: &Value, after: &Value, additions: &[Value]) -> Value {
        adversarial::assert_metadata_shape(before);
        adversarial::assert_metadata_shape(after);
        let prior = before["functions"].as_array().unwrap();
        let final_rows = after["functions"].as_array().unwrap();
        assert!(
            additions.len() == 2 && final_rows.len() == prior.len() + 2,
            "exact two-function addition required"
        );
        let mut seen = BTreeSet::new();
        let mut ids = BTreeSet::new();
        for addition in additions {
            let name = addition["proname"].as_str().expect("vetted function name");
            let id = addition["oid"].as_str().expect("vetted catalog OID");
            let numeric = id.parse::<u32>().expect("catalog OID");
            assert!(
                numeric > 0
                    && numeric.to_string() == id
                    && NAMES.contains(&name)
                    && seen.insert(name)
                    && ids.insert(id),
                "exact distinct new function identities"
            );
            assert!(
                !prior
                    .iter()
                    .any(|r| r["oid"] == addition["oid"] || r["proname"] == addition["proname"]),
                "predecessor contained a new helper identity"
            );
            assert!(
                final_rows.iter().filter(|r| *r == addition).count() == 1,
                "new helper row omitted, altered or duplicated"
            );
        }
        let mut stripped = after.clone();
        stripped["functions"]
            .as_array_mut()
            .unwrap()
            .retain(|row| !additions.contains(row));
        assert!(
            stripped["functions"] == before["functions"],
            "pre-existing function changed or disappeared"
        );
        stripped
    }

    async fn checked_additions(connection: &mut PgConnection) -> Vec<Value> {
        let records: Vec<(Value, bool)> = sqlx::query_as(r#"
          SELECT to_jsonb(p),
            p.proargtypes='2950 2950'::oidvector AND p.pronargs=2 AND p.pronargdefaults=0
            AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND NOT p.proleakproof
            AND p.provolatile='v' AND p.proparallel='u' AND p.proretset
            AND p.prorettype='record'::regtype AND p.prolang=(SELECT oid FROM pg_language WHERE lanname='plpgsql')
            AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp','row_security=on']::text[]
            AND p.proowner=(CASE WHEN p.proname='account_session_shared_material_v1' THEN 'console_account_owner' ELSE 'console_credential_owner' END)::regrole
            AND p.proargnames=CASE WHEN p.proname='account_session_shared_material_v1'
              THEN ARRAY['p_account','p_family','security_state','security_generation','revision','context_generation','user_id','protocol','account_security_generation','auth_time','assurance','created_at','revoked_at','org_id']::text[]
              ELSE ARRAY['p_account','p_family','user_id','protocol','account_security_generation','auth_time','assurance','created_at','revoked_at','org_id']::text[] END
            AND p.proallargtypes=CASE WHEN p.proname='account_session_shared_material_v1'
              THEN ARRAY[2950,2950,25,20,20,20,2950,25,20,1184,25,1184,1184,2950]::oid[]
              ELSE ARRAY[2950,2950,2950,25,20,1184,25,1184,1184,2950]::oid[] END
            AND p.proargmodes=CASE WHEN p.proname='account_session_shared_material_v1'
              THEN ARRAY['i','i','t','t','t','t','t','t','t','t','t','t','t','t']::"char"[]
              ELSE ARRAY['i','i','t','t','t','t','t','t','t','t']::"char"[] END
            AND (SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable) ORDER BY CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END)
                 FROM aclexplode(p.proacl) a)
              = CASE WHEN p.proname='account_session_shared_material_v1'
                THEN '[["console_account_owner","console_account_owner","EXECUTE",false],["console_account_owner","console_auth_rt","EXECUTE",false],["console_account_owner","console_rt","EXECUTE",false]]'::jsonb
                ELSE '[["console_credential_owner","console_account_owner","EXECUTE",false],["console_credential_owner","console_credential_owner","EXECUTE",false]]'::jsonb END
          FROM pg_proc p WHERE p.pronamespace='public'::regnamespace AND p.proname=ANY($1) ORDER BY p.oid
        "#).bind(NAMES).fetch_all(&mut *connection).await.unwrap();
        assert!(
            records.len() == 2 && records.iter().all(|r| r.1),
            "exact declared session helper additions required"
        );
        records.into_iter().map(|r| r.0).collect()
    }

    pub(super) async fn predecessor_equivalent(
        connection: &mut PgConnection,
        before: &Value,
        after: &Value,
    ) -> Value {
        let without_eligibility =
            eligibility_transition::without_checked_addition(connection, before, after).await;
        strip_exact_additions(
            before,
            &without_eligibility,
            &checked_additions(connection).await,
        )
    }

    #[test]
    fn addition_oracle_rejects_extra_missing_mutated_and_omitted_prior_functions() {
        // Synthetic structural checker controls only; not database fixtures.
        let mut before = adversarial::metadata_control(false);
        before["functions"][0]["proname"] = json!("existing_owner");
        let additions = vec![
            json!({"oid":"700","proname":NAMES[0]}),
            json!({"oid":"701","proname":NAMES[1]}),
        ];
        let mut after = before.clone();
        after["functions"]
            .as_array_mut()
            .unwrap()
            .extend(additions.clone());
        assert!(strip_exact_additions(&before, &after, &additions) == before);
        let mut exercised = 0;
        for fault in 0..7 {
            let mut left = before.clone();
            let mut right = after.clone();
            let mut allowed = additions.clone();
            match fault {
                0 => {
                    right["functions"].as_array_mut().unwrap().pop();
                }
                1 => right["functions"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"oid":"702","proname":"unexpected"})),
                2 => right["functions"][0]["changed"] = json!(true),
                3 => {
                    right["functions"].as_array_mut().unwrap().remove(0);
                }
                4 => allowed[1] = allowed[0].clone(),
                5 => {
                    left["functions"][0] = additions[0].clone();
                }
                _ => right["functions"][1]["changed"] = json!(true),
            }
            assert!(
                std::panic::catch_unwind(|| strip_exact_additions(&left, &right, &allowed))
                    .is_err(),
                "addition oracle accepted fault {fault}"
            );
            exercised += 1;
        }
        assert_eq!(exercised, 7);
    }

    // Observe actual backend exit after dropping every serving handle. This
    // never terminates sessions: an undrained consumer makes maintenance fail.
    async fn assert_session_consumers_drained(pool: &PgPool) {
        let isolated: bool = sqlx::query_scalar(
            "SELECT current_database() ~ '^_sqlx_test_' AND session_user='console_buck_admin' AND current_user=session_user AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=session_user)",
        )
        .fetch_one(pool)
        .await
        .unwrap();
        assert!(
            isolated,
            "rollback requires the disposable operator-owned test database"
        );
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            loop {
                let drained: bool = sqlx::query_scalar(
                    "SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity a WHERE a.datname=current_database() AND a.backend_type='client backend' AND a.usename IS DISTINCT FROM 'console_buck_admin')",
                )
                .fetch_one(pool)
                .await
                .unwrap();
                if drained {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            }
        })
        .await
        .expect("all non-operator database consumers must actually exit before maintenance");
    }

    async fn upgrade(pool: &PgPool, observer: bool) {
        let (app, config, key, attempt, cookies) = audited228(pool).await;
        let mut tx = pool.begin().await.unwrap();
        audited_state(&mut tx).await;
        let untouched_rows = rows(&mut tx).await;
        let untouched_meta = metadata(&mut tx).await;
        tx.rollback().await.unwrap();
        let startup = AppState::from_config(config.clone()).await;
        if let Ok(state) = &startup {
            state.shutdown_realtime().await;
        }
        assert!(
            matches!(startup,Err(console_app::AppError::Config(ref code)) if code=="account_custody.native_upgrade_required"),
            "new serving must refuse real current audit228"
        );
        let mut tx = pool.begin().await.unwrap();
        operator(&mut tx).await;
        assert_preserved(&untouched_rows, &rows(&mut tx).await);
        assert!(
            untouched_meta == metadata(&mut tx).await,
            "failed startup modified the predecessor"
        );
        if observer {
            let source =
                include_str!("fixtures/account-native-deployment228-observer-6c884418.sql");
            assert!(hex::encode(Sha256::digest(source)) == OBSERVER_SHA256);
            sqlx::raw_sql(source).execute(&mut *tx).await.unwrap();
        }
        audited_state(&mut tx).await;
        assert!(state(&mut tx).await == "account_custody.native_upgrade_required");
        let before = metadata(&mut tx).await;
        let before_rows = rows(&mut tx).await;
        compose(&mut tx).await.unwrap();
        assert_final(&mut tx).await;
        let after = metadata(&mut tx).await;
        assert!(
            predecessor_equivalent(&mut tx, &before, &after).await == before,
            "audited228 transition changed metadata beyond two exact helpers"
        );
        assert_preserved(&before_rows, &rows(&mut tx).await);
        compose(&mut tx).await.unwrap();
        assert_final(&mut tx).await;
        assert!(
            after == metadata(&mut tx).await,
            "finalized replay changed metadata"
        );
        assert_preserved(&before_rows, &rows(&mut tx).await);
        if observer {
            // Cluster-scoped observer installation is never leaked across tests.
            tx.rollback().await.unwrap();
            let mut check = pool.begin().await.unwrap();
            audited_state(&mut check).await;
            assert!(untouched_meta == metadata(&mut check).await);
            assert_preserved(&untouched_rows, &rows(&mut check).await);
            check.rollback().await.unwrap();
        } else {
            tx.commit().await.unwrap();
            let rollback_config = config.clone();
            let actual = AppState::from_config(config.clone()).await.unwrap();
            let service = build_router(actual.clone());
            let me = request(&service, "GET", "/api/v2/accounts/me", &cookies, None, &[]).await;
            projection(&me.json(StatusCode::OK), attempt.account, pool).await;
            me.private();
            let jwt = config.jwt.unwrap();
            let verifier = JwtVerifier::from_es256_public_pem(
                JwtSettings {
                    issuer: jwt.issuer,
                    audience: jwt.audience,
                    access_token_ttl: Duration::minutes(15),
                },
                jwt.public_key_pem.as_bytes(),
            )
            .unwrap();
            let runtime = login_test_pool(pool, TestDatabaseLogin::Business).await;
            let mut read = runtime.begin().await.unwrap();
            let live = live_account_session_in_tx(
                &mut read,
                &verifier,
                &cookies.0[ACCESS],
                config.auth_rest.unwrap().refresh_family_absolute_ttl,
            )
            .await
            .unwrap_or_else(|_| {
                panic!("original real session must resolve through Business after upgrade")
            });
            assert!(
                live.account_id == attempt.account
                    && signed_claims(&cookies.0[ACCESS], &key).unwrap()["sid"]
                        == json!(live.session_id)
            );
            read.commit().await.unwrap();
            runtime.close().await;
            actual.shutdown_realtime().await;
            let mut check = pool.acquire().await.unwrap();
            assert_preserved(&before_rows, &rows(&mut check).await);
            assert!(after == metadata(&mut check).await);
            drop(check);
            drop(service);
            drop(actual);
            assert_session_consumers_drained(pool).await;

            // The committed successor already served the genuine original
            // session. Reuse its actual no-op operator replay to acquire the
            // existing maintenance locks and certify privilege/profile before
            // the exact helper-only inverse. No new rollback framework.
            let mut rollback = pool.begin().await.unwrap();
            compose(&mut rollback).await.unwrap();
            assert_final(&mut rollback).await;
            checked_additions(&mut rollback).await;
            eligibility_transition::checked_addition(&mut rollback).await;
            assert!(after == metadata(&mut rollback).await);
            assert_preserved(&before_rows, &rows(&mut rollback).await);
            eligibility_transition::rollback_platform_source(&mut rollback, &before).await;
            sqlx::raw_sql("DROP FUNCTION public.account_company_setup_eligibility_v1(uuid) RESTRICT; DROP FUNCTION public.account_session_shared_material_v1(uuid,uuid) RESTRICT; DROP FUNCTION public.auth_account_session_shared_material_v1(uuid,uuid) RESTRICT;")
                .execute(&mut *rollback).await.unwrap();
            audited_state(&mut rollback).await;
            assert!(
                before == metadata(&mut rollback).await,
                "helper inverse must restore exact predecessor metadata"
            );
            assert_preserved(&before_rows, &rows(&mut rollback).await);
            rollback.commit().await.unwrap();

            // Fresh transaction proves the inverse committed; startup is the
            // actual current binary admission path, not a simulated status.
            let mut committed = pool.begin().await.unwrap();
            audited_state(&mut committed).await;
            assert!(before == metadata(&mut committed).await);
            assert_preserved(&before_rows, &rows(&mut committed).await);
            committed.rollback().await.unwrap();
            let refused = AppState::from_config(rollback_config.clone()).await;
            if let Ok(unexpected) = &refused {
                unexpected.shutdown_realtime().await;
            }
            assert!(
                matches!(&refused, Err(console_app::AppError::Config(code)) if code == "account_custody.native_upgrade_required"),
                "current serving must refuse the committed exact predecessor"
            );
            drop(refused);
            assert_session_consumers_drained(pool).await;

            let mut reapply = pool.begin().await.unwrap();
            audited_state(&mut reapply).await;
            assert!(
                before == metadata(&mut reapply).await,
                "refused startup must not repair predecessor metadata"
            );
            assert_preserved(&before_rows, &rows(&mut reapply).await);
            compose(&mut reapply).await.unwrap();
            assert_final(&mut reapply).await;
            let reapplied_metadata = metadata(&mut reapply).await;
            // Dropped functions legitimately receive new pg_proc OIDs. Vet
            // exactly the two re-created additions; every old row stays exact.
            assert!(
                predecessor_equivalent(&mut reapply, &before, &reapplied_metadata).await == before
            );
            assert_preserved(&before_rows, &rows(&mut reapply).await);
            compose(&mut reapply).await.unwrap();
            assert_final(&mut reapply).await;
            assert!(
                reapplied_metadata == metadata(&mut reapply).await,
                "reapplied successor replay changed metadata"
            );
            assert_preserved(&before_rows, &rows(&mut reapply).await);
            reapply.commit().await.unwrap();

            let restored = AppState::from_config(rollback_config.clone())
                .await
                .unwrap();
            let restored_service = build_router(restored.clone());
            let restored_me = request(
                &restored_service,
                "GET",
                "/api/v2/accounts/me",
                &cookies,
                None,
                &[],
            )
            .await;
            projection(&restored_me.json(StatusCode::OK), attempt.account, pool).await;
            restored_me.private();
            let restored_runtime = login_test_pool(pool, TestDatabaseLogin::Business).await;
            let mut restored_read = restored_runtime.begin().await.unwrap();
            let restored_live = live_account_session_in_tx(
                &mut restored_read,
                &verifier,
                &cookies.0[ACCESS],
                rollback_config.auth_rest.as_ref().unwrap().refresh_family_absolute_ttl,
            )
            .await
            .unwrap_or_else(|_| panic!("original genuine session must remain usable after committed rollback and reapply"));
            assert!(
                restored_live.account_id == attempt.account
                    && signed_claims(&cookies.0[ACCESS], &key).unwrap()["sid"]
                        == json!(restored_live.session_id)
            );
            restored_read.commit().await.unwrap();
            restored_runtime.close().await;
            restored.shutdown_realtime().await;
            drop(restored_service);
            drop(restored);
            assert_session_consumers_drained(pool).await;
            let mut final_check = pool.acquire().await.unwrap();
            assert_preserved(&before_rows, &rows(&mut final_check).await);
            assert!(reapplied_metadata == metadata(&mut final_check).await);
        }
        drop(app);
    }

    #[sqlx::test(migrations = false)]
    async fn populated_audit228_session_upgrade_preserves_history_and_replays(pool: PgPool) {
        upgrade(&pool, false).await;
    }

    #[sqlx::test(migrations = false)]
    async fn initialized_observer_audit228_session_upgrade_preserves_receipts_and_replays(
        pool: PgPool,
    ) {
        upgrade(&pool, true).await;
    }

    #[sqlx::test(migrations = false)]
    async fn malformed_session_helpers_refuse_without_repair_and_partial_install_rolls_back(
        pool: PgPool,
    ) {
        let (app, _, _, _, _) = audited228(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        audited_state(&mut tx).await;
        let original = metadata(&mut tx).await;
        let original_rows = rows(&mut tx).await;
        for name in NAMES {
            for signature in ["uuid,uuid", "text,text"] {
                let mut fault = tx.begin().await.unwrap();
                let ddl = format!(
                    "CREATE FUNCTION public.{name}({signature}) RETURNS boolean LANGUAGE sql SECURITY INVOKER AS 'SELECT false'; ALTER FUNCTION public.{name}({signature}) OWNER TO console_app; REVOKE ALL ON FUNCTION public.{name}({signature}) FROM PUBLIC"
                );
                sqlx::raw_sql(sqlx::AssertSqlSafe(ddl))
                    .execute(&mut *fault)
                    .await
                    .unwrap();
                let identity = format!("public.{name}({signature})");
                let witnessed: bool = sqlx::query_scalar("SELECT proowner='console_app'::regrole AND NOT prosecdef AND prorettype='boolean'::regtype FROM pg_proc WHERE oid=to_regprocedure($1)")
                    .bind(identity).fetch_one(&mut *fault).await.unwrap();
                assert!(witnessed, "actual malformed helper required");
                assert!(state(&mut fault).await == "account_native.profile_mismatch");
                let fault_meta = metadata(&mut fault).await;
                let mut attempt = fault.begin().await.unwrap();
                let result = compose(&mut attempt).await;
                attempt.rollback().await.unwrap();
                let Err(error) = result else {
                    panic!("operator repaired malformed helper");
                };
                assert!(
                    error
                        .as_database_error()
                        .is_some_and(|e| e.code().as_deref() == Some("P0001")
                            && e.message() == "account_native.profile_mismatch"),
                    "exact profile refusal required"
                );
                assert!(fault_meta == metadata(&mut fault).await);
                assert_preserved(&original_rows, &rows(&mut fault).await);
                fault.rollback().await.unwrap();
                assert!(original == metadata(&mut tx).await);
                audited_state(&mut tx).await;
            }
        }
        let mut fault = tx.begin().await.unwrap();
        sqlx::raw_sql(r#"CREATE FUNCTION public.session_test_after_inner_create() RETURNS event_trigger LANGUAGE plpgsql AS $$
          BEGIN IF to_regprocedure('public.auth_account_session_shared_material_v1(uuid,uuid)') IS NOT NULL THEN
            RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='SESSION_TEST_AFTER_ACTUAL_INNER_CREATE';
          END IF; END $$;
          CREATE EVENT TRIGGER session_test_after_inner_create ON ddl_command_end WHEN TAG IN ('CREATE FUNCTION') EXECUTE FUNCTION public.session_test_after_inner_create()"#)
            .execute(&mut *fault).await.unwrap();
        let fault_meta = metadata(&mut fault).await;
        let mut attempt = fault.begin().await.unwrap();
        let result = compose(&mut attempt).await;
        attempt.rollback().await.unwrap();
        let Err(error) = result else {
            panic!("actual inner-install fault not reached");
        };
        assert!(
            error
                .as_database_error()
                .is_some_and(|e| e.code().as_deref() == Some("P0001")
                    && e.message() == "SESSION_TEST_AFTER_ACTUAL_INNER_CREATE"),
            "failure must witness actual inner function creation"
        );
        assert!(
            fault_meta == metadata(&mut fault).await,
            "partial install escaped rollback"
        );
        assert_preserved(&original_rows, &rows(&mut fault).await);
        audited_state(&mut fault).await;
        fault.rollback().await.unwrap();
        assert!(original == metadata(&mut tx).await);
        compose(&mut tx).await.unwrap();
        assert_final(&mut tx).await;
        let after = metadata(&mut tx).await;
        assert!(predecessor_equivalent(&mut tx, &original, &after).await == original);
        assert_preserved(&original_rows, &rows(&mut tx).await);
        tx.rollback().await.unwrap();
        drop(app);
    }
    include!("audit_company_eligibility_transition.rs");
}
