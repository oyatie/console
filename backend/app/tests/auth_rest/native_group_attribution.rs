// Include inside the existing auth_rest::account_browser module.
// This proves the existing Group persistence owner's native Account attribution.
// It does NOT authenticate a native Group command, consume deployment designation,
// mint Company authority, establish idempotency or certify activation/recovery.
mod native_group_attribution {
    use super::*;
    use console_platform_provisioning::{PlatformProvisioner, ProvisioningError};
    use console_platform_test_support::login_test_pool;

    const UNCHANGED: &[&str] = &[
        "accounts",
        "account_security",
        "account_security_events",
        "account_terms_acceptances",
        "auth_webauthn_credentials",
        "auth_refresh_token_families",
        "auth_refresh_tokens",
        "auth_bootstrap_credentials",
        "users",
        "organizations",
        "group_memberships",
        "group_role_grants",
        "company_actors",
        "account_context_candidates",
        "deployment_operator_receipts",
        "deployment_operator_head",
    ];

    async fn inventory(pool: &PgPool, tables: &[&str]) -> Vec<String> {
        let mut rows = Vec::new();
        for table in tables {
            // Fixed test-owned identifiers only. Never print captured row bytes.
            let query = format!(
                "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text COLLATE \"C\"),'[]'::jsonb)::text FROM public.{table} t"
            );
            rows.push(
                sqlx::query_scalar(sqlx::AssertSqlSafe(query))
                    .fetch_one(pool)
                    .await
                    .unwrap(),
            );
        }
        rows
    }

    #[sqlx::test(migrations = false)]
    async fn native_account_group_persistence_retains_actor_without_legacy_user(pool: PgPool) {
        let app = fixture(&pool).await;
        let (account, cookies) = enrolled(&app).await;
        let me = request(&app, "GET", "/api/v2/accounts/me", &cookies, None, &[]).await;
        projection(&me.json(StatusCode::OK), account.account);
        let legacy_rows: i64 = sqlx::query_scalar("SELECT count(*) FROM public.users WHERE id=$1")
            .bind(account.account)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(legacy_rows, 0, "real Account must have no legacy user");
        let before = inventory(&pool, UNCHANGED).await;
        let old_groups = inventory(&pool, &["groups"]).await.remove(0);
        let old_audit = inventory(&pool, &["audit_events"]).await.remove(0);
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let login: (String, String) =
            sqlx::query_as("SELECT session_user::text,current_user::text")
                .fetch_one(&runtime)
                .await
                .unwrap();
        assert_eq!(login, ("console_rt".into(), "console_rt".into()));
        let now: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&pool)
            .await
            .unwrap();
        let slug = format!("native-{}", &Uuid::new_v4().simple().to_string()[..20]);
        let result = PlatformProvisioner::new(Duration::minutes(5))
            .create_group(
                &runtime,
                Some(UserId::from_uuid(account.account)),
                &slug,
                "Synthetic native Group",
                now,
            )
            .await;
        assert!(
            result.is_ok(),
            "NATIVE_GROUP_ACCOUNT_ATTRIBUTION: existing Group persistence owner must accept a real Account without users row"
        );
        let group = result.unwrap();
        assert_eq!(group.slug, slug);
        assert_eq!(group.name, "Synthetic native Group");
        assert_eq!(group.member_count, 0);
        assert!(group.members.is_empty());
        let actual: (Uuid, String, String, String) =
            sqlx::query_as("SELECT id,slug,name,status FROM public.groups WHERE id=$1")
                .bind(group.id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(
            actual,
            (
                group.id,
                slug,
                "Synthetic native Group".into(),
                "ACTIVE".into()
            )
        );
        let audit: Vec<(Uuid, Option<Uuid>, String, String, OffsetDateTime)> = sqlx::query_as(
            "SELECT actor,org_id,action,target_type,occurred_at FROM public.audit_events WHERE target_id=$1"
        ).bind(group.id.to_string()).fetch_all(&pool).await.unwrap();
        assert_eq!(
            audit,
            vec![(
                account.account,
                None,
                "platform.group.create".into(),
                "groups".into(),
                now
            )]
        );
        let retained_groups: String = sqlx::query_scalar(
            "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text COLLATE \"C\"),'[]'::jsonb)::text FROM public.groups t WHERE id<>$1"
        ).bind(group.id).fetch_one(&pool).await.unwrap();
        let retained_audit: String = sqlx::query_scalar(
            "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text COLLATE \"C\"),'[]'::jsonb)::text FROM public.audit_events t WHERE target_id<>$1"
        ).bind(group.id.to_string()).fetch_one(&pool).await.unwrap();
        assert!(
            retained_groups == old_groups,
            "all preexisting Group rows preserved"
        );
        assert!(
            retained_audit == old_audit,
            "all preexisting audit bytes preserved"
        );
        assert!(
            inventory(&pool, UNCHANGED).await == before,
            "no identity, Company, permission or credential side effects"
        );
        runtime.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn unknown_account_group_persistence_rolls_back_topology_and_audit(pool: PgPool) {
        let _app = fixture(&pool).await;
        let missing = Uuid::new_v4();
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM public.accounts WHERE id=$1")
            .bind(missing)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
        let unchanged = inventory(&pool, UNCHANGED).await;
        let before = inventory(&pool, &["groups", "audit_events"]).await;
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let now: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&pool)
            .await
            .unwrap();
        let slug = format!("missing-{}", &Uuid::new_v4().simple().to_string()[..20]);
        let error = PlatformProvisioner::new(Duration::minutes(5))
            .create_group(
                &runtime,
                Some(UserId::from_uuid(missing)),
                &slug,
                "Synthetic absent Account",
                now,
            )
            .await
            .expect_err("missing attribution root must refuse");
        match error {
            ProvisioningError::Db(console_platform_db::DbError::Sqlx(error)) => {
                let database = error
                    .as_database_error()
                    .expect("native PostgreSQL FK error");
                assert_eq!(database.code().as_deref(), Some("23503"));
                let name = database.constraint().expect("named attribution FK");
                let target: String = sqlx::query_scalar(
                    "SELECT confrelid::regclass::text FROM pg_catalog.pg_constraint WHERE conrelid='public.audit_events'::regclass AND conname=$1 AND contype='f'"
                ).bind(name).fetch_one(&pool).await.unwrap();
                assert_eq!(
                    target, "accounts",
                    "refusal must come from the enrolled Account FK"
                );
            }
            _ => panic!("expected audit owner Account FK refusal, not setup or unrelated failure"),
        }
        assert!(
            inventory(&pool, &["groups", "audit_events"]).await == before,
            "failed actor append rolls back actual Group insertion and audit"
        );
        assert!(
            inventory(&pool, UNCHANGED).await == unchanged,
            "no surrounding authority or identity effects"
        );
        runtime.close().await;
    }
}
