async fn manager_controls(
    tx: &mut Transaction<'_, Postgres>,
    accepted: &Capture,
    accepted_catalog: &(String, Value),
    accepted_rows: &BTreeMap<String, String>,
    accepted_ledger: &Value,
) -> Value {
    let mut results = Vec::new();
    for (name, fault, namespace_changes) in [
        (
            "helper_runtime_acl",
            "GRANT EXECUTE ON FUNCTION public.identity_company_information_group_lock_v1(uuid,uuid) TO console_rt",
            false,
        ),
        (
            "public_acl",
            "GRANT EXECUTE ON FUNCTION public.identity_company_information_manager_current_v1(uuid,uuid,uuid,uuid,uuid) TO PUBLIC",
            false,
        ),
        (
            "grant_option",
            "GRANT EXECUTE ON FUNCTION public.identity_company_information_selected_lock_v1(uuid,uuid) TO console_account_owner WITH GRANT OPTION",
            false,
        ),
        (
            "manager_owner",
            "ALTER FUNCTION public.identity_company_information_manager_current_v1(uuid,uuid,uuid,uuid,uuid) OWNER TO console_app",
            true,
        ),
        (
            "helper_setting",
            "ALTER FUNCTION public.identity_company_information_group_lock_v1(uuid,uuid) SET row_security=off",
            false,
        ),
        (
            "search_path",
            "ALTER FUNCTION public.identity_company_information_manager_current_v1(uuid,uuid,uuid,uuid,uuid) SET search_path=public,pg_catalog",
            false,
        ),
        (
            "parallel",
            "ALTER FUNCTION public.identity_company_information_selected_lock_v1(uuid,uuid) PARALLEL SAFE",
            false,
        ),
        (
            "security_invoker",
            "ALTER FUNCTION public.identity_company_information_root_material_v1(uuid,uuid) SECURITY INVOKER",
            true,
        ),
        (
            "helper_body_only",
            "CREATE OR REPLACE FUNCTION public.identity_company_information_group_lock_v1(p_company uuid,p_group uuid) RETURNS TABLE(group_row jsonb,group_head_row jsonb) LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE SET search_path=pg_catalog,pg_temp SET row_security=on SET TimeZone='UTC' SET bytea_output='hex' SET DateStyle='ISO, YMD' SET IntervalStyle='postgres' AS 'BEGIN RETURN; END'",
            false,
        ),
        (
            "language_body",
            "CREATE OR REPLACE FUNCTION public.identity_company_information_group_lock_v1(p_company uuid,p_group uuid) RETURNS TABLE(group_row jsonb,group_head_row jsonb) LANGUAGE sql VOLATILE SECURITY DEFINER PARALLEL UNSAFE SET search_path=pg_catalog,pg_temp SET row_security=on SET TimeZone='UTC' SET bytea_output='hex' SET DateStyle='ISO, YMD' SET IntervalStyle='postgres' AS 'SELECT NULL::jsonb,NULL::jsonb'",
            false,
        ),
        (
            "defaults",
            "CREATE OR REPLACE FUNCTION public.identity_company_information_selected_lock_v1(p_company uuid,p_group uuid DEFAULT NULL) RETURNS TABLE(company_row jsonb,membership_row jsonb,membership_revision_row jsonb) LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE SET search_path=pg_catalog,pg_temp SET row_security=on SET TimeZone='UTC' SET bytea_output='hex' SET DateStyle='ISO, YMD' SET IntervalStyle='postgres' AS 'BEGIN RETURN; END'",
            false,
        ),
        (
            "overload",
            "CREATE FUNCTION public.identity_company_information_manager_current_v1(text) RETURNS text LANGUAGE sql IMMUTABLE AS 'SELECT $1'; ALTER FUNCTION public.identity_company_information_manager_current_v1(text) OWNER TO console_app; REVOKE ALL ON FUNCTION public.identity_company_information_manager_current_v1(text) FROM PUBLIC",
            true,
        ),
        (
            "foreign_schema",
            "CREATE SCHEMA manager_capture_control; CREATE FUNCTION manager_capture_control.identity_company_information_extra_v1() RETURNS integer LANGUAGE sql IMMUTABLE AS 'SELECT 1'; ALTER FUNCTION manager_capture_control.identity_company_information_extra_v1() OWNER TO console_app; REVOKE ALL ON FUNCTION manager_capture_control.identity_company_information_extra_v1() FROM PUBLIC",
            true,
        ),
        (
            "routine_kind",
            "CREATE PROCEDURE public.identity_company_information_extra_v1() LANGUAGE plpgsql AS 'BEGIN RETURN; END'; ALTER PROCEDURE public.identity_company_information_extra_v1() OWNER TO console_app; REVOKE ALL ON PROCEDURE public.identity_company_information_extra_v1() FROM PUBLIC",
            true,
        ),
        (
            "partial_namespace",
            "DROP FUNCTION public.identity_company_information_manager_current_v1(uuid,uuid,uuid,uuid,uuid)",
            true,
        ),
    ] {
        let mut attempt = tx.begin().await.unwrap();
        let outcome = AssertUnwindSafe(async {
            execute(&mut attempt, fault).await;
            let observed = policy_capture(attempt.as_mut(), QUERY).await;
            assert_eq!(
                observed.rights,
                name != "public_acl",
                "startup privilege visibility control: {name}"
            );
            assert!(
                observed != *accepted,
                "capture omitted metadata control: {name}"
            );
            let namespace = manager_namespace(attempt.as_mut()).await;
            assert_eq!(
                namespace != expected_namespace(),
                namespace_changes,
                "namespace control: {name}"
            );
            assert_eq!(
                observed.snapshot[NAMESPACE_KEY], namespace,
                "capture namespace is incomplete: {name}"
            );
            let corrupted_catalog = catalog(attempt.as_mut()).await;
            assert!(
                corrupted_catalog != *accepted_catalog,
                "raw security catalog omitted control: {name}"
            );
            assert!(
                rows(attempt.as_mut()).await == *accepted_rows,
                "metadata control changed durable rows: {name}"
            );
            assert_eq!(applied_ledger(attempt.as_mut()).await, *accepted_ledger);
            json!({"name":name,"capture_changed":true,"namespace_changed":namespace_changes,
                "observed_capture":observed.record(),"observed_namespace":namespace,
                "observed_catalog_text_sha256":digest(&corrupted_catalog.0)})
        })
        .catch_unwind()
        .await;
        attempt
            .rollback()
            .await
            .expect("metadata control savepoint rollback required");
        assert!(
            policy_capture(tx.as_mut(), QUERY).await == *accepted,
            "control rollback changed raw capture: {name}"
        );
        assert!(
            catalog(tx.as_mut()).await == *accepted_catalog,
            "control rollback changed catalog: {name}"
        );
        assert_eq!(manager_namespace(tx.as_mut()).await, expected_namespace());
        assert!(
            rows(tx.as_mut()).await == *accepted_rows,
            "control rollback changed rows: {name}"
        );
        assert_eq!(applied_ledger(tx.as_mut()).await, *accepted_ledger);
        manager_exact_routines(&accepted_catalog.1);
        match outcome {
            Ok(value) => results.push(value),
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }
    assert_eq!(results.len(), 15);
    json!(results)
}
