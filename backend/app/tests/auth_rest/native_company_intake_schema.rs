// Include inside existing deployment_operator_designation::company_setup.
// Empty private schema prerequisite; never a serving finalizer or effect fixture.
mod intake_schema {
    use super::*;
    use console_platform_test_support::{TestDatabaseLogin, login_test_pool};

    const PARSER: &str = include_str!("../../../../ops/postgres-company-enrollment-input.sql");
    const INSTALL: &str = include_str!("../../../../ops/postgres-company-enrollment-schema.sql");
    const ROOT_STATE: &str = include_str!("../../src/account_custody_state.sql");
    const CREDENTIAL_STATE: &str = include_str!("../../src/account_credential_custody_state.sql");
    const ACL: &str = include_str!("fixtures/company-enrollment-schema-acl.sql");
    const REFERENCES: &str = include_str!("fixtures/company-enrollment-schema-references.sql");
    const TABLES: [&str; 3] = [
        "company_enrollment_requests",
        "company_enrollment_receipts",
        "company_enrollment_request_events",
    ];

    async fn profiles(connection: &mut sqlx::PgConnection, root: &str, credential: &str) {
        sqlx::raw_sql("SET search_path=pg_catalog,pg_temp; SET statement_timeout='10s'; SET lock_timeout='1s'").execute(&mut *connection).await.unwrap();
        for (source, expected) in [(ROOT_STATE, root), (CREDENTIAL_STATE, credential)] {
            let actual: String = sqlx::query_scalar(source)
                .fetch_one(&mut *connection)
                .await
                .unwrap();
            assert_eq!(actual, expected, "unchanged classifier required");
        }
    }

    async fn denied(pool: &PgPool, expected_login: &str) {
        let identity: (String, String, bool, bool) = sqlx::query_as("SELECT session_user::text,current_user::text,(SELECT rolsuper FROM pg_roles WHERE rolname=current_user),(SELECT rolbypassrls FROM pg_roles WHERE rolname=current_user)")
            .fetch_one(pool).await.unwrap();
        assert_eq!(
            identity,
            (expected_login.into(), expected_login.into(), false, false)
        );
        for table in TABLES {
            // Only the fixed source roster enters these SQL identifiers.
            for query in [
                format!("SELECT * FROM public.{table}"),
                format!("INSERT INTO public.{table} DEFAULT VALUES"),
                format!("UPDATE public.{table} SET account_id=account_id WHERE false"),
                format!("DELETE FROM public.{table} WHERE false"),
                format!("TRUNCATE public.{table}"),
            ] {
                let error = sqlx::query(sqlx::AssertSqlSafe(query))
                    .execute(pool)
                    .await
                    .expect_err("actual serving login must have no direct table authority");
                assert_eq!(
                    error.as_database_error().and_then(|e| e.code()).as_deref(),
                    Some("42501")
                );
            }
        }
    }

    async fn corrupt_acl(connection: &mut sqlx::PgConnection, mutation: &str, expected: &str) {
        sqlx::raw_sql("BEGIN; SET LOCAL statement_timeout='10s'; SET LOCAL lock_timeout='1s'")
            .execute(&mut *connection)
            .await
            .unwrap();
        sqlx::raw_sql(mutation)
            .execute(&mut *connection)
            .await
            .unwrap();
        let error = sqlx::raw_sql(ACL)
            .execute(&mut *connection)
            .await
            .expect_err("injected actual catalog corruption must be detected");
        let error = error.as_database_error().unwrap();
        assert_eq!(error.code().as_deref(), Some("P0001"));
        assert_eq!(error.message(), expected);
        sqlx::raw_sql("ROLLBACK")
            .execute(&mut *connection)
            .await
            .unwrap();
        sqlx::raw_sql(ACL).execute(&mut *connection).await.unwrap();
    }

    #[sqlx::test(migrations = false)]
    async fn company_intake_empty_schema_is_private_and_retains_real_fk_dependencies(pool: PgPool) {
        let (_app, _account, _cookies, startup, _designation) = designated(&pool).await;
        let before = all_rows(&pool).await;
        let mut admin = pool.acquire().await.unwrap();
        let identity: (String, String, bool) = sqlx::query_as("SELECT session_user::text,current_user::text,current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' AND (SELECT rolsuper FROM pg_roles WHERE rolname=current_user)")
            .fetch_one(&mut *admin).await.unwrap();
        assert_eq!(
            identity,
            (
                "console_buck_admin".into(),
                "console_buck_admin".into(),
                true
            )
        );
        profiles(
            &mut admin,
            "account_custody.native_finalized",
            "account_credentials.native_finalized",
        )
        .await;
        let absent: bool = sqlx::query_scalar("SELECT to_regprocedure('public.company_enrollment_decode_input_v1(bytea)') IS NULL AND to_regclass('public.company_enrollment_requests') IS NULL AND to_regclass('public.company_enrollment_receipts') IS NULL AND to_regclass('public.company_enrollment_request_events') IS NULL")
            .fetch_one(&mut *admin).await.unwrap();
        assert!(absent, "no partial schema in current serving finalization");
        sqlx::raw_sql("BEGIN; SET LOCAL statement_timeout='10s'; SET LOCAL lock_timeout='1s'")
            .execute(&mut *admin)
            .await
            .unwrap();
        sqlx::raw_sql(PARSER).execute(&mut *admin).await.unwrap();
        sqlx::raw_sql(INSTALL).execute(&mut *admin).await.unwrap();
        sqlx::raw_sql("COMMIT").execute(&mut *admin).await.unwrap();
        profiles(
            &mut admin,
            "account_native.profile_mismatch",
            "account_native.profile_mismatch",
        )
        .await;
        sqlx::raw_sql(ACL).execute(&mut *admin).await.unwrap();
        sqlx::raw_sql(REFERENCES)
            .execute(&mut *admin)
            .await
            .unwrap();

        // No-login owner operation, explicitly NOT a serving-login test.
        // The receipt stays empty: no fabricated Company, assignment or receipt.
        sqlx::raw_sql("BEGIN; SET LOCAL statement_timeout='10s'; SET LOCAL lock_timeout='1s'; SET LOCAL ROLE console_account_owner")
            .execute(&mut *admin).await.unwrap();
        let owner: String = sqlx::query_scalar("SELECT current_user::text")
            .fetch_one(&mut *admin)
            .await
            .unwrap();
        assert_eq!(owner, "console_account_owner");
        sqlx::query("SELECT receipt_id FROM public.company_enrollment_receipts FOR KEY SHARE")
            .fetch_all(&mut *admin)
            .await
            .unwrap();
        let error = sqlx::query(
            "UPDATE public.company_enrollment_receipts SET receipt_id=receipt_id WHERE false",
        )
        .execute(&mut *admin)
        .await
        .expect_err("statement guard must refuse even zero-row owner UPDATE");
        let error = error.as_database_error().unwrap();
        assert_eq!(error.code().as_deref(), Some("P0001"));
        assert_eq!(error.message(), "company_enrollment_receipts.immutable");
        sqlx::raw_sql("ROLLBACK")
            .execute(&mut *admin)
            .await
            .unwrap();

        corrupt_acl(
            &mut admin,
            "GRANT SELECT ON public.company_enrollment_requests TO console_rt",
            "company.schema_exact_acl: company_enrollment_requests",
        )
        .await;
        corrupt_acl(&mut admin,
            "ALTER TABLE public.company_enrollment_receipts ENABLE TRIGGER company_enrollment_receipts_immutable_v1",
            "company.receipt_keyshare_guard_missing").await;

        for (login, name) in [
            (TestDatabaseLogin::Business, "console_rt"),
            (TestDatabaseLogin::Auth, "console_auth_rt"),
            (TestDatabaseLogin::LeaveCommand, "console_leave_cmd"),
            (TestDatabaseLogin::OntologyCommand, "console_ontology_cmd"),
            (
                TestDatabaseLogin::PlatformForceCommand,
                "console_platform_force_cmd",
            ),
        ] {
            let runtime = login_test_pool(&pool, login).await;
            denied(&runtime, name).await;
            runtime.close().await;
        }
        denied(&startup, "console_auth_startup").await;
        startup.close().await;
        profiles(
            &mut admin,
            "account_native.profile_mismatch",
            "account_native.profile_mismatch",
        )
        .await;
        drop(admin);
        let mut after = all_rows(&pool).await;
        for table in TABLES {
            assert!(
                !before.contains_key(table),
                "new relation cannot replace historical table"
            );
            assert_eq!(
                after.remove(table).as_deref(),
                Some("[]"),
                "no fabricated prerequisite rows"
            );
        }
        assert!(
            before == after,
            "all existing durable histories must remain byte-identical"
        );
        eprintln!(
            "company_intake_schema_probe sqlx_leaves=1 new_empty_tables=3 actual_login_table_denials=90 owner_keyshare_checks=1 owner_immutable_denials=1 corruption_controls=2"
        );
        // SQLx discards the DB; do not reseal or call an incomplete finalizer.
    }
}
