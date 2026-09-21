use super::*;

#[sqlx::test(migrations = false)]
async fn native_catalog_each_audit_kind_is_required_and_same_command_recovers(pool: PgPool) {
    let (app, account, cookies, startup, _) = designated(&pool).await;
    for action in [
        "ontology.object_type.builtin_install",
        "ontology.object_policy.attach",
    ] {
        let command = Uuid::new_v4();
        let input = enrollment(command, account.account);
        let before = all_rows(&pool).await;
        // Invoker-only fault: omit exactly the first genuine row of this kind.
        // USAGE is scoped to the actual Account audit owner, not serving roles.
        let fault = format!(
            r#"
          CREATE SEQUENCE public.company_catalog_omitted_audit;
          REVOKE ALL ON SEQUENCE public.company_catalog_omitted_audit FROM PUBLIC;
          GRANT USAGE ON SEQUENCE public.company_catalog_omitted_audit TO console_account_owner;
          CREATE FUNCTION public.company_catalog_omit_audit() RETURNS trigger
          LANGUAGE plpgsql SECURITY INVOKER SET search_path=pg_catalog,pg_temp AS $body$
          BEGIN
            IF NEW.action='{action}' AND NEW.after_snap->'enrollment'->>'command_id'='{command}' THEN
              IF current_user<>'console_account_owner' THEN RAISE EXCEPTION 'unexpected actual native audit owner'; END IF;
              IF nextval('public.company_catalog_omitted_audit'::regclass)=1 THEN RETURN NULL; END IF;
            END IF;
            RETURN NEW;
          END $body$;
          REVOKE ALL ON FUNCTION public.company_catalog_omit_audit() FROM PUBLIC;
          CREATE TRIGGER zz_company_catalog_omit_audit BEFORE INSERT ON public.audit_events
          FOR EACH ROW EXECUTE FUNCTION public.company_catalog_omit_audit();
        "#
        );
        sqlx::raw_sql(sqlx::AssertSqlSafe(fault))
            .execute(&pool)
            .await
            .unwrap();
        let response = submit(&app, &cookies, &input).await;
        let seen: (bool, i64) =
            sqlx::query_as("SELECT is_called,last_value FROM public.company_catalog_omitted_audit")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(
            seen.0 && (1..=2).contains(&seen.1),
            "intended actual audit omission was not reached"
        );
        response.error(
            StatusCode::SERVICE_UNAVAILABLE,
            "company_enrollment_unavailable",
        );
        let failed = all_rows(&pool).await;
        assert_only_pending_company_intake(&pool, &before, &failed, account.account, command).await;
        sqlx::raw_sql("DROP TRIGGER zz_company_catalog_omit_audit ON public.audit_events; DROP FUNCTION public.company_catalog_omit_audit(); DROP SEQUENCE public.company_catalog_omitted_audit;")
            .execute(&pool).await.unwrap();
        let result = committed(
            &submit(&app, &cookies, &input).await,
            StatusCode::CREATED,
            command,
            account.account,
            false,
        );
        durable(&pool, &result, &input, account.account).await;
        let complete = all_rows(&pool).await;
        assert!(native_catalog_attribution_matches(
            &complete,
            &result,
            account.account
        ));
        let replay = committed(
            &submit(&app, &cookies, &input).await,
            StatusCode::OK,
            command,
            account.account,
            true,
        );
        assert_eq!(result.receipt, replay.receipt);
        assert_eq!(complete, all_rows(&pool).await);
    }
    assert_native_topology_metadata(&pool).await;
    startup.close().await;
}
