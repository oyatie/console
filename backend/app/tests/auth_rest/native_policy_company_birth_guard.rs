// Include inside company_setup. Actual HTTP/WebAuthn owner regression,
// not browser acceptance; no successful business fixtures are inserted directly.
mod native_policy_company_birth_guard {
    use super::*;
    use futures::FutureExt;
    use std::panic::AssertUnwindSafe;

    const GUARD: &str = "ontology_api.native_catalog_attribution_guard_v2()";
    const EXCEPTION: &str = "EXCEPTION WHEN no_data_found OR too_many_rows THEN";

    async fn metadata(pool: &PgPool) -> Value {
        sqlx::query_scalar("SELECT jsonb_build_object('definition',pg_get_functiondef(p.oid),'oid',p.oid,'owner',pg_get_userbyid(p.proowner),'acl',p.proacl::text,'config',p.proconfig,'security_definer',p.prosecdef,'language',p.prolang,'volatility',p.provolatile,'parallel',p.proparallel,'result',pg_get_function_result(p.oid)) FROM pg_proc p WHERE p.oid=to_regprocedure($1)")
            .bind(GUARD).fetch_one(pool).await.unwrap()
    }
    async fn profile(pool: &PgPool) -> String {
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql("SET LOCAL search_path=pg_catalog,pg_temp")
            .execute(tx.as_mut())
            .await
            .unwrap();
        let result = sqlx::query_scalar(include_str!(
            "../../../../ops/postgres-native-company-policy-custody-state.sql"
        ))
        .fetch_one(tx.as_mut())
        .await
        .unwrap();
        tx.rollback().await.unwrap();
        result
    }

    #[sqlx::test(migrations = false)]
    async fn native_policy_successor_preserves_company_birth_key_guard_and_recovery(pool: PgPool) {
        super::native_policy_startup_tests::prepare_policy_ready_database(&pool).await;
        let artifacts = Artifacts::new();
        let key = SigningKey::random(&mut OsRng);
        let state = state_with_key(&pool, artifacts.root.clone(), &key).await;
        let app = Fixture {
            service: build_router(state.clone()),
            _artifacts: artifacts,
            pool: pool.clone(),
        };
        let outcome=AssertUnwindSafe(async {
        let (app,account,cookies,startup,_)=designated_fixture(&pool,app).await;
        let csrf=proof(&app,&cookies).await;
        let command=Uuid::new_v4();
        let input=enrollment(command,account.account);
        let before=all_rows(&pool).await;
        let original_metadata=metadata(&pool).await;
        let original=original_metadata["definition"].as_str().unwrap().to_owned();
        assert_eq!(original.matches(EXCEPTION).count(),1,"guard exception shape drift");
        // Observe the actual preserved guard's exact denial, then rethrow it
        // unchanged. An unrelated503 or the mutation alone cannot satisfy this.
        let observed=original.replacen(EXCEPTION,r#"EXCEPTION WHEN insufficient_privilege THEN
 IF TG_TABLE_NAME='ont_object_type_key_revisions' AND TG_OP='INSERT'
  AND current_user='console_ontology_writer'
  AND NEW.stable_key='company_birth_forbidden_test'
  AND SQLSTATE='42501' AND SQLERRM='identity_native.catalog_invalid_birth' THEN
  PERFORM nextval('public.policy_birth_guard_denied'::regclass);
 END IF;
 RAISE;
WHEN no_data_found OR too_many_rows THEN"#,1);
        let fault=format!(r#"
CREATE SEQUENCE public.policy_birth_key_changed;
CREATE SEQUENCE public.policy_birth_guard_denied;
REVOKE ALL ON SEQUENCE public.policy_birth_key_changed,public.policy_birth_guard_denied FROM PUBLIC;
GRANT USAGE ON SEQUENCE public.policy_birth_key_changed,public.policy_birth_guard_denied TO console_ontology_writer;
CREATE FUNCTION public.policy_birth_key_fault() RETURNS trigger
LANGUAGE plpgsql SECURITY INVOKER SET search_path=pg_catalog,pg_temp SET row_security=on AS $fault$
DECLARE b record;
BEGIN
 IF NEW.stable_key='company_workspace' THEN
  IF current_user IS DISTINCT FROM 'console_ontology_writer' THEN RAISE EXCEPTION 'wrong_fault_owner'; END IF;
  SELECT * INTO STRICT b FROM public.company_enrollment_catalog_binding_v1(NEW.org_id);
  IF b.command_id IS DISTINCT FROM '{command}'::uuid
   OR b.account_id IS DISTINCT FROM '{actor}'::uuid OR b.request_state IS DISTINCT FROM 'PENDING'
   OR b.effect_xid IS DISTINCT FROM pg_current_xact_id() OR b.effect_backend_pid IS DISTINCT FROM pg_backend_pid() THEN
   RAISE EXCEPTION 'wrong_fault_binding';
  END IF;
  PERFORM nextval('public.policy_birth_key_changed'::regclass);
  NEW.stable_key:='company_birth_forbidden_test';
 END IF;
 RETURN NEW;
END
$fault$;
REVOKE ALL ON FUNCTION public.policy_birth_key_fault() FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.policy_birth_key_fault() TO console_ontology_writer;
CREATE TRIGGER aaaa_policy_birth_key_fault BEFORE INSERT ON public.ont_object_type_key_revisions
 FOR EACH ROW EXECUTE FUNCTION public.policy_birth_key_fault();
ALTER TABLE public.ont_object_type_key_revisions ENABLE ALWAYS TRIGGER aaaa_policy_birth_key_fault;
"#,actor=account.account);
        let fault_outcome=AssertUnwindSafe(async {
            // Atomic metadata setup prevents partially installed injection.
            let mut tx=pool.begin().await.unwrap();
            sqlx::raw_sql(sqlx::AssertSqlSafe(observed.as_str())).execute(tx.as_mut()).await.unwrap();
            sqlx::raw_sql(sqlx::AssertSqlSafe(fault.as_str())).execute(tx.as_mut()).await.unwrap();
            tx.commit().await.unwrap();
            assert_eq!(metadata(&pool).await["definition"],json!(observed));
            assert_eq!(profile(&pool).await,"native_company_policy.profile_mismatch");
            let response=submit(&app,&cookies,&csrf,&input).await;
            let witnesses:(bool,bool)=sqlx::query_as("SELECT (SELECT is_called AND last_value=1 FROM public.policy_birth_key_changed),(SELECT is_called AND last_value=1 FROM public.policy_birth_guard_denied)")
                .fetch_one(&pool).await.unwrap();
            assert_eq!(witnesses,(true,true),"actual bound key mutation and exact guard SQLSTATE42501 denial required");
            response.error(StatusCode::SERVICE_UNAVAILABLE,"company_enrollment_unavailable");
        }).catch_unwind().await;
        // Always restore exact function and remove every fault object before
        // resuming panic. CREATE OR REPLACE preserves identity/owner/ACL.
        let mut restore=pool.begin().await.unwrap();
        sqlx::raw_sql(sqlx::AssertSqlSafe(original.as_str())).execute(restore.as_mut()).await.unwrap();
        sqlx::raw_sql("DROP TRIGGER IF EXISTS aaaa_policy_birth_key_fault ON public.ont_object_type_key_revisions; DROP FUNCTION IF EXISTS public.policy_birth_key_fault(); DROP SEQUENCE IF EXISTS public.policy_birth_key_changed,public.policy_birth_guard_denied")
            .execute(restore.as_mut()).await.unwrap();
        restore.commit().await.unwrap();
        assert_eq!(metadata(&pool).await,original_metadata,"exact guard metadata restoration");
        assert_eq!(profile(&pool).await,"native_company_policy.finalized");
        if let Err(panic)=fault_outcome {std::panic::resume_unwind(panic);}
        let failed = all_rows(&pool).await;
        let mut before_effects = before.clone();
        let mut after_effects = failed.clone();
        let old_requests = before_effects
            .remove("company_enrollment_requests")
            .expect("admitted request owner must exist before submission");
        let requests = after_effects
            .remove("company_enrollment_requests")
            .expect("durable input request required after composition failure");
        let old_events = before_effects
            .remove("company_enrollment_request_events")
            .expect("admitted request event owner must exist before submission");
        let events = after_effects
            .remove("company_enrollment_request_events")
            .expect("durable preparation event required after composition failure");
        assert!(
            before_effects == after_effects,
            "failed composition retained partial Company/Group/catalog/grant/context/receipt/audit effect"
        );
        let added = added_rows(&old_requests, &requests).expect("request history preserved");
        assert_eq!(added.len(), 1, "exactly one durable input request");
        assert!(
            added[0]["account_id"] == json!(account.account)
                && added[0]["command_id"] == json!(command)
                && added[0]["state"] == "PENDING"
        );
        // Prepare commits exactly one request/event pair before the effect
        // transaction. Neither prior history nor any other effect is excluded.
        let added_events =
            added_rows(&old_events, &events).expect("request event history preserved");
        assert_eq!(
            added_events.len(),
            1,
            "exactly one durable preparation event"
        );
        let tokens: Vec<Value> = serde_json::from_str(&before["auth_refresh_tokens"]).unwrap();
        let token_hash = format!(
            "\\x{}",
            hex::encode(Sha256::digest(cookies.0[REFRESH].as_bytes()))
        );
        let tokens: Vec<_> = tokens
            .iter()
            .filter(|token| token["token_hash"] == token_hash)
            .collect();
        assert_eq!(
            tokens.len(),
            1,
            "submitted cookie must identify one original token"
        );
        let token = tokens[0];
        assert!(
            token["user_id"] == json!(account.account)
                && token["used_at"].is_null()
                && token["revoked_at"].is_null()
        );
        let families: Vec<Value> =
            serde_json::from_str(&before["auth_refresh_token_families"]).unwrap();
        let families: Vec<_> = families
            .iter()
            .filter(|family| family["id"] == token["family_id"])
            .collect();
        assert_eq!(
            families.len(),
            1,
            "original token must identify one native session"
        );
        let family = families[0];
        assert!(
            family["user_id"] == json!(account.account)
                && family["protocol"] == "ACCOUNT_V1"
                && family["revoked_at"].is_null()
        );
        assert!(
            added[0]["created_at"].is_string(),
            "request must retain authoritative creation time"
        );
        assert_eq!(
            added_events[0],
            json!({
                "account_id": account.account,
                "command_id": command,
                "event_revision": 1,
                "from_state": null,
                "to_state": "PENDING",
                "occurred_at": added[0]["created_at"],
                "actor_account_id": account.account,
                "session_id": family["id"],
                "reason_code": "PREPARED"
            }),
            "only the exact request-correlated preparation event may survive"
        );
        let pending = request(
            &app,
            "GET",
            &format!("/api/v2/companies/enrollments/{command}"),
            &cookies,
            None,
            &[],
        )
        .await;
        pending.private();
        let pending = pending.json(StatusCode::OK);
        exact_keys(
            &pending,
            &["outcome", "original_command_id", "input", "result_path"],
        );
        assert!(
            pending["outcome"] == "PENDING"
                && pending["original_command_id"] == json!(command)
                && pending["input"] == input
                && pending["result_path"] == format!("/account/companies/requests/{command}")
        );
        let reopened = document(
            &app,
            &format!("/account/companies/requests/{command}"),
            &cookies,
        )
        .await;
        let reopened = native_entry_html(&reopened, StatusCode::OK);
        assert!(
            reopened.contains("연결된 업무 회사")
                && reopened.contains("다시 시도")
                && !reopened.contains("생성 완료"),
            "durable pending input must reopen without invented success"
        );
        assert!(
            failed == all_rows(&pool).await,
            "pending read modified command or effects"
        );

        let created=committed(&submit(&app,&cookies,&csrf,&input).await,StatusCode::CREATED,command,account.account,false);
        durable(&pool,&created,&input,account.account).await;
        // Actual successful owner retry must produce exactly the original two
        // birth keys with revision/validator/time and Account-command provenance.
        let correct:(i64,i64)=sqlx::query_as("SELECT count(*),count(*) FILTER (WHERE k.stable_key IN ('company_workspace','company_policy_assignment') AND k.revision=1 AND k.validator_id IS NOT NULL AND k.validator_id<>'00000000-0000-0000-0000-000000000000'::uuid AND k.created_at=b.started_at AND k.updated_at=b.started_at AND o.attribution_protocol='NATIVE_ACCOUNT' AND o.origin_account_id=$2 AND o.origin_command_id=$3 AND o.origin_receipt_id=$4 AND o.created_by_account_id=$2 AND o.created_at=b.started_at AND o.updated_at=b.started_at) FROM public.ont_object_type_key_revisions k JOIN public.company_enrollment_effect_bindings b ON b.org_id=k.org_id LEFT JOIN public.ont_object_types o ON o.org_id=k.org_id AND o.stable_key=k.stable_key WHERE k.org_id=$1")
            .bind(created.company).bind(account.account).bind(command).bind(created.receipt).fetch_one(&pool).await.unwrap();
        assert_eq!(correct,(2,2));
        let after=all_rows(&pool).await;
        let replay=committed(&submit(&app,&cookies,&csrf,&input).await,StatusCode::OK,command,account.account,true);
        assert!(replay.receipt==created.receipt && replay.company==created.company && replay.group==created.group);
        assert!(after==all_rows(&pool).await,"replay changed durable business history");
        assert_eq!(profile(&pool).await,"native_company_policy.finalized");
        startup.close().await;
        }).catch_unwind().await;
        state.shutdown_realtime().await;
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }
}
