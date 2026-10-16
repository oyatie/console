// Include inside native_people_directory_owner_tests after exact successor fixture review.
// Exercises accepted records produced through the actual owner, never direct SQL fixtures.
mod native_directory_row_lock_immutability_tests {
    use super::*;

    async fn metadata(pool: &PgPool) -> (Value, String, Option<bool>) {
        let source =
            include_str!("../../../../ops/postgres-capture-native-people-directory-custody.sql");
        assert_eq!(
            hex::encode(Sha256::digest(source.as_bytes())),
            "bc8a1f87f57cd676ca1a3deae12263b1cca4a290c71a12e9a1749d189e090ca7"
        );
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(tx.as_mut())
            .await
            .unwrap();
        sqlx::raw_sql(CLASSIFIER_SESSION)
            .execute(tx.as_mut())
            .await
            .unwrap();
        let value = sqlx::query_as(sqlx::AssertSqlSafe(source))
            .fetch_one(tx.as_mut())
            .await
            .unwrap();
        tx.rollback().await.unwrap();
        value
    }

    #[sqlx::test(migrations = false)]
    async fn accepted_native_input_remains_immutable_with_owner_row_lock_privilege(pool: PgPool) {
        let f = DirectoryFixture::new(&pool).await;
        let outcome = AssertUnwindSafe(async {
            f.configure(true, true).await;
            let (input, credentials) = f.input("K-ROW-LOCK").await;
            let before = all_rows(&pool).await;
            let prepared = directory_prepare(&f.store, &f.decision, &credentials, &input, &TraceContext::generate()).await.unwrap();
            assert!(prepared.inserted);
            let DirectoryStatus::Pending(accepted) = prepared.status else { panic!("real owner must accept pending input") };
            let prepared_rows = all_rows(&pool).await;
            exact_delta(&before, &prepared_rows, &[("native_people_inputs_v1", 1), ("audit_events", 1)]);
            let pending = directory_status(&f.store, &f.decision, &read_credentials(&f.cookies), input.locator(), &TraceContext::generate(), &mut TestAdmission::default()).await.unwrap();
            assert_eq!(pending.status, DirectoryStatus::Pending(accepted));
            let proof = pending.proof.unwrap();
            let execute_credentials = AccountEnrollmentCredentials::for_mutation(&f.cookies.0[ACCESS], proof.as_str()).unwrap();
            let committed = directory_execute(&f.store, &f.decision, &execute_credentials, input.locator(), &TraceContext::generate()).await.unwrap();
            assert!(committed.inserted);
            assert_eq!(committed.terminal.outcome(), DirectoryTerminalOutcomeV1::Committed);
            let rows = all_rows(&pool).await;
            exact_delta(&prepared_rows, &rows, &[("employees", 1), ("persons", 1), ("person_revisions", 1), ("employee_person_bindings", 1), ("ont_action_command_receipts", 1), ("native_people_terminals_v1", 1), ("audit_events", 1)]);
            let original = metadata(&pool).await;
            assert_eq!(original.2, Some(true));
            let command = input.locator().command_id();
            let replacement = Uuid::new_v4();
            assert_ne!(replacement, command);
            for statement in [
                "UPDATE public.native_people_inputs_v1 SET command_id=command_id WHERE org_id=$1 AND command_id=$2 AND $3::uuid<>command_id",
                "UPDATE public.native_people_inputs_v1 SET command_id=$3 WHERE org_id=$1 AND command_id=$2",
                "UPDATE public.native_people_inputs_v1 SET command_id=$3 WHERE org_id=$1 AND command_id=$2 AND false",
            ] {
                let mut tx = pool.begin().await.unwrap();
                sqlx::raw_sql("SET LOCAL ROLE console_account_owner").execute(tx.as_mut()).await.unwrap();
                let role: (String, bool, bool) = sqlx::query_as("SELECT current_user::text,rolsuper,rolbypassrls FROM pg_catalog.pg_roles WHERE rolname=current_user").fetch_one(tx.as_mut()).await.unwrap();
                assert_eq!(role, ("console_account_owner".into(), false, false));
                let _: String = sqlx::query_scalar("SELECT pg_catalog.set_config('app.current_org',$1,true)").bind(f.created.company.to_string()).fetch_one(tx.as_mut()).await.unwrap();
                let visible: i64 = sqlx::query_scalar("SELECT count(*) FROM public.native_people_inputs_v1 WHERE org_id=$1 AND command_id=$2").bind(f.created.company).bind(command).fetch_one(tx.as_mut()).await.unwrap();
                assert_eq!(visible, 1, "affected-row attempts must target a real RLS-visible accepted input");
                let error = sqlx::query(sqlx::AssertSqlSafe(statement)).bind(f.created.company).bind(command).bind(replacement).execute(tx.as_mut()).await.unwrap_err();
                let error = error.as_database_error().unwrap();
                assert_eq!(error.code().as_deref(), Some("P0001"));
                assert_eq!(error.message(), "people.directory.history_immutable");
                tx.rollback().await.unwrap();
                assert_eq!(all_rows(&pool).await, rows);
                assert_eq!(metadata(&pool).await, original);
            }
            for (as_owner, statement) in [
                (true, "DELETE FROM public.native_people_inputs_v1"),
                (true, "TRUNCATE public.native_people_inputs_v1"),
                (false, "UPDATE public.native_people_inputs_v1 SET command_id=command_id"),
                (false, "DELETE FROM public.native_people_inputs_v1"),
                (false, "TRUNCATE public.native_people_inputs_v1"),
            ] {
                let mut tx = if as_owner { pool.begin().await.unwrap() } else { f.runtime.begin().await.unwrap() };
                if as_owner { sqlx::raw_sql("SET LOCAL ROLE console_account_owner").execute(tx.as_mut()).await.unwrap(); }
                let role: (String, String, bool, bool) = sqlx::query_as("SELECT session_user::text,current_user::text,rolsuper,rolbypassrls FROM pg_catalog.pg_roles WHERE rolname=current_user").fetch_one(tx.as_mut()).await.unwrap();
                assert_eq!(role.1, if as_owner { "console_account_owner" } else { "console_rt" });
                assert_eq!((role.2, role.3), (false, false));
                if !as_owner { assert_eq!(role.0, "console_rt"); }
                let _: String = sqlx::query_scalar("SELECT pg_catalog.set_config('app.current_org',$1,true)").bind(f.created.company.to_string()).fetch_one(tx.as_mut()).await.unwrap();
                let error = sqlx::raw_sql(sqlx::AssertSqlSafe(statement)).execute(tx.as_mut()).await.unwrap_err();
                let error = error.as_database_error().unwrap();
                assert_eq!(error.code().as_deref(), Some("42501"));
                assert_eq!(error.message(), "permission denied for table native_people_inputs_v1");
                tx.rollback().await.unwrap();
                assert_eq!(all_rows(&pool).await, rows);
                assert_eq!(metadata(&pool).await, original);
            }
            let replay = directory_execute(&f.store, &f.decision, &execute_credentials, input.locator(), &TraceContext::generate()).await.unwrap();
            assert!(!replay.inserted);
            assert_eq!(replay.terminal, committed.terminal);
            let reopened = directory_status(&f.store, &f.decision, &read_credentials(&f.cookies), input.locator(), &TraceContext::generate(), &mut TestAdmission::default()).await.unwrap();
            assert!(reopened.proof.is_none());
            assert_eq!(reopened.status, DirectoryStatus::Terminal(committed.terminal));
            assert_eq!(all_rows(&pool).await, rows);
            assert_eq!(metadata(&pool).await, original);
        }).catch_unwind().await;
        f.close(outcome).await;
    }
}
