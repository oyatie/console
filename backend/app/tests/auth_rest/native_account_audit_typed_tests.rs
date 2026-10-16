// Include inside auth_rest.rs::account_browser. Uses real finalized native
// registration fixture; no Account/User/business seed rows. This is writer
// representation coverage, not Company authorization or browser acceptance.
mod native_account_audit_typed {
    use super::*;
    use console_kernel_core::{
        AccountId, AuditAction, AuditClassification, AuditEvent, AuditRequestContext, TraceContext,
    };
    use console_platform_db::insert_audit_event;
    use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
    use sqlx::{Postgres, Row, Transaction};

    async fn state(pool: &PgPool) -> Value {
        let mut tx = pool.begin().await.unwrap();
        let rows = super::audit_account_transition::rows(&mut tx).await;
        tx.rollback().await.unwrap();
        rows
    }

    fn assert_only_append(before: &Value, after: &Value, event_id: Uuid) {
        let mut without_appended = after.clone();
        let rows = without_appended["audit_events"]["rows"]
            .as_array_mut()
            .expect("actual complete audit rows");
        let original_len = rows.len();
        rows.retain(|row| row["id"] != json!(event_id));
        assert_eq!(
            original_len - rows.len(),
            1,
            "exactly one typed audit append"
        );
        assert!(
            before == &without_appended,
            "typed append altered existing rows or another owner"
        );
    }

    async fn assert_exact_row(tx: &mut Transaction<'_, Postgres>, event: &AuditEvent<AccountId>) {
        let row = sqlx::query(
            "SELECT id,actor,action,target_type,target_id,branch_id,before_snap,after_snap,trace_id,span_id,occurred_at,org_id,ip,user_agent,auth_method,device,classification_badges,anomaly,reason,before_snap IS NULL AS before_is_sql_null,after_snap IS NULL AS after_is_sql_null FROM public.audit_events WHERE id=$1",
        )
        .bind(*event.id.as_uuid())
        .fetch_one(tx.as_mut())
        .await
        .unwrap();
        assert_eq!(row.try_get::<Uuid, _>("id").unwrap(), *event.id.as_uuid());
        assert_eq!(
            row.try_get::<Option<Uuid>, _>("actor").unwrap(),
            event.actor.map(|id| *id.as_uuid())
        );
        assert_eq!(
            row.try_get::<String, _>("action").unwrap(),
            event.action.as_str()
        );
        assert_eq!(
            row.try_get::<String, _>("target_type").unwrap(),
            event.target_type
        );
        assert_eq!(
            row.try_get::<String, _>("target_id").unwrap(),
            event.target_id
        );
        assert_eq!(
            row.try_get::<Option<Uuid>, _>("branch_id").unwrap(),
            event.branch_id.map(|id| *id.as_uuid())
        );
        assert_eq!(
            row.try_get::<Option<Value>, _>("before_snap").unwrap(),
            event.before
        );
        assert_eq!(
            row.try_get::<Option<Value>, _>("after_snap").unwrap(),
            event.after
        );
        assert_eq!(
            row.try_get::<bool, _>("before_is_sql_null").unwrap(),
            event.before.is_none()
        );
        assert_eq!(
            row.try_get::<bool, _>("after_is_sql_null").unwrap(),
            event.after.is_none()
        );
        assert_eq!(
            row.try_get::<String, _>("trace_id").unwrap(),
            event.trace.trace_id()
        );
        assert_eq!(
            row.try_get::<String, _>("span_id").unwrap(),
            event.trace.span_id()
        );
        assert_eq!(
            row.try_get::<OffsetDateTime, _>("occurred_at").unwrap(),
            event.occurred_at
        );
        assert_eq!(
            row.try_get::<Option<Uuid>, _>("org_id").unwrap(),
            event.org_id.map(|id| *id.as_uuid())
        );
        assert_eq!(
            row.try_get::<Option<String>, _>("ip").unwrap(),
            event.request_context.ip
        );
        assert_eq!(
            row.try_get::<Option<String>, _>("user_agent").unwrap(),
            event.request_context.user_agent
        );
        assert_eq!(
            row.try_get::<Option<String>, _>("auth_method").unwrap(),
            event.request_context.auth_method
        );
        assert_eq!(
            row.try_get::<Option<String>, _>("device").unwrap(),
            event.request_context.device
        );
        assert_eq!(
            row.try_get::<Option<Vec<String>>, _>("classification_badges")
                .unwrap(),
            event.classification.badges
        );
        assert_eq!(
            row.try_get::<Option<bool>, _>("anomaly").unwrap(),
            event.classification.anomaly
        );
        assert_eq!(
            row.try_get::<Option<String>, _>("reason").unwrap(),
            event.classification.reason
        );
    }

    #[sqlx::test(migrations = false)]
    async fn finalized_native_account_common_writer_commit_and_rollback(pool: PgPool) {
        // Existing fixture owns migrations, actual finalized custody, TEST_ONLY
        // terms publication and real native registration/primary assertion.
        let app = fixture(&pool).await;
        let (attempt, _cookies) = enrolled(&app).await;
        assert_committed(&pool, &attempt).await;
        let count: (i64, i64) = sqlx::query_as(
            "SELECT (SELECT count(*) FROM public.accounts WHERE id=$1),(SELECT count(*) FROM public.users WHERE id=$1)",
        ).bind(attempt.account).fetch_one(&pool).await.unwrap();
        assert_eq!(
            count,
            (1, 0),
            "native Account exists without legacy User surrogate"
        );
        let mut admin = pool.begin().await.unwrap();
        super::audit_account_transition::assert_final(&mut admin).await;
        admin.rollback().await.unwrap();

        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let identity: (String, String, bool, bool) = sqlx::query_as(
            "SELECT session_user::text,current_user::text,rolsuper,rolbypassrls FROM pg_catalog.pg_roles WHERE rolname=current_user",
        ).fetch_one(&runtime).await.unwrap();
        assert_eq!(
            identity,
            ("console_rt".into(), "console_rt".into(), false, false)
        );
        let account = AccountId::from_uuid(attempt.account).unwrap();
        let baseline = state(&pool).await;
        let now: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&runtime)
            .await
            .unwrap();
        let event: AuditEvent<AccountId> = AuditEvent::new_account(
            account,
            AuditAction::new("audit.typed_account_probe").unwrap(),
            "TEST_ONLY_TYPED_AUDIT",
            "commit",
            TraceContext::new("0123456789abcdef0123456789abcdef", "0123456789abcdef").unwrap(),
            now,
        )
        .with_snapshots(None, Some(Value::Null))
        .with_request_context(AuditRequestContext {
            ip: Some("192.0.2.21".into()),
            user_agent: Some("TEST_ONLY typed Account audit".into()),
            auth_method: Some("passkey".into()),
            device: Some("fixture".into()),
        })
        .with_classification(AuditClassification {
            badges: Some(vec!["TEST_ONLY".into(), "확인".into()]),
            anomaly: Some(false),
            reason: Some("typed actor representation".into()),
        });
        let mut tx = runtime.begin().await.unwrap();
        insert_audit_event(&mut tx, &event).await.unwrap();
        // Read with test observer pool only after commit; serving permissions
        // need not grant raw global audit SELECT. Never elevate the writer.
        tx.commit().await.unwrap();
        let mut observer = pool.begin().await.unwrap();
        assert_exact_row(&mut observer, &event).await;
        observer.rollback().await.unwrap();
        let committed = state(&pool).await;
        assert_only_append(&baseline, &committed, *event.id.as_uuid());

        let rollback_event = AuditEvent::new_account(
            account,
            AuditAction::new("audit.typed_account_probe").unwrap(),
            "TEST_ONLY_TYPED_AUDIT",
            "rollback",
            TraceContext::generate(),
            now,
        )
        .with_snapshots(Some(json!({"nested":[null,1,"확인"]})), None);
        let mut tx = runtime.begin().await.unwrap();
        insert_audit_event(&mut tx, &rollback_event).await.unwrap();
        tx.rollback().await.unwrap();
        assert!(
            committed == state(&pool).await,
            "rollback leaked typed event or changed owner history"
        );
        let absent: i64 =
            sqlx::query_scalar("SELECT count(*) FROM public.audit_events WHERE id=$1")
                .bind(*rollback_event.id.as_uuid())
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(absent, 0);
        runtime.close().await;
    }
}
