//! Bounded PUB prerequisite coverage: actual runtime LOGINs cannot directly
//! mutate terms head/receipts or acquire their owners. No publisher, approval,
//! terms, Account or Company authority rows are created by this source.
//! This does not prove callable publisher-helper access or operator custody.
use super::prepare_http_database;
use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
use sqlx::PgPool;

const RELATIONS: &[&str] = &["account_terms_head", "account_terms_release_receipts"];
const LOGINS: &[(TestDatabaseLogin, &str)] = &[
    (TestDatabaseLogin::Auth, "console_auth_rt"),
    (TestDatabaseLogin::Business, "console_rt"),
    (TestDatabaseLogin::OntologyCommand, "console_ontology_cmd"),
    (TestDatabaseLogin::LeaveCommand, "console_leave_cmd"),
    (
        TestDatabaseLogin::PlatformForceCommand,
        "console_platform_force_cmd",
    ),
];

async fn require_publication_catalog(pool: &PgPool) {
    let missing: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM unnest($1::text[]) AS required(name)
         WHERE NOT EXISTS(SELECT 1 FROM pg_catalog.pg_class c
             JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
             WHERE n.nspname='public' AND c.relname=name AND c.relkind IN ('r','p'))
         ORDER BY name",
    )
    .bind(RELATIONS)
    .fetch_all(pool)
    .await
    .unwrap();
    assert!(
        missing.is_empty(),
        "PUB_CATALOG_PREREQUISITE: mutation and owner-isolation assertions not reached; missing={missing:?}"
    );
    let roles: Vec<String> =
        sqlx::query_scalar("SELECT rolname::text FROM pg_catalog.pg_roles WHERE rolname=ANY($1)")
            .bind(LOGINS.iter().map(|(_, role)| *role).collect::<Vec<_>>())
            .fetch_all(pool)
            .await
            .unwrap();
    assert_eq!(
        roles.len(),
        LOGINS.len(),
        "PUB_LOGIN_PREREQUISITE: all actual serving/command roles must exist; denial assertions not reached"
    );
    let required_columns: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_catalog.pg_attribute a
         JOIN pg_catalog.pg_class c ON c.oid=a.attrelid
         JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
         WHERE n.nspname='public' AND c.relname=ANY($1)
         AND a.attname IN ('id','revision') AND a.attnum>0 AND NOT a.attisdropped",
    )
    .bind(RELATIONS)
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(
        required_columns, 4,
        "PUB_COLUMN_PREREQUISITE: real approved head/receipt columns required before no-op mutation probes"
    );
}

#[sqlx::test(migrations = false)]
async fn serving_and_command_logins_cannot_directly_publish_terms(pool: PgPool) {
    prepare_http_database(&pool).await;
    require_publication_catalog(&pool).await;
    for (login, role) in LOGINS {
        // Real LOGIN with session_user/current_user and all administrative bits
        // checked by the existing helper. SET ROLE is never the connection path.
        let runtime = login_test_pool(&pool, *login).await;
        for operation in [
            "INSERT INTO public.account_terms_head(id) SELECT 1 WHERE false",
            "UPDATE public.account_terms_head SET revision=1 WHERE false",
            "DELETE FROM public.account_terms_head WHERE false",
            "INSERT INTO public.account_terms_release_receipts(id) SELECT gen_random_uuid() WHERE false",
            "UPDATE public.account_terms_release_receipts SET revision=1 WHERE false",
            "DELETE FROM public.account_terms_release_receipts WHERE false",
        ] {
            // Constant-false DML still checks SQL privileges. Even an unexpected
            // statement-trigger side effect is rolled back; no fake terms row.
            let mut tx = runtime.begin().await.unwrap();
            let result = sqlx::query(operation).execute(&mut *tx).await;
            tx.rollback().await.unwrap();
            let error = result.expect_err("runtime SQL must not have publisher mutation authority");
            assert_eq!(
                error
                    .as_database_error()
                    .and_then(|error| error.code())
                    .as_deref(),
                Some("42501"),
                "{role}: refusal must be privilege denial for {operation}"
            );
        }
        runtime.close().await;
    }
}

#[sqlx::test(migrations = false)]
async fn terms_owner_and_effective_column_privileges_do_not_admit_serving_logins(pool: PgPool) {
    prepare_http_database(&pool).await;
    require_publication_catalog(&pool).await;
    let owners: Vec<(String, String, bool)> = sqlx::query_as(
        "SELECT c.relname::text,r.rolname::text,r.rolcanlogin FROM pg_catalog.pg_class c
         JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
         JOIN pg_catalog.pg_roles r ON r.oid=c.relowner
         WHERE n.nspname='public' AND c.relname=ANY($1)",
    )
    .bind(RELATIONS)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(owners.len(), RELATIONS.len());
    assert!(
        owners.iter().all(|(_, owner, can_login)| !can_login
            && !LOGINS.iter().any(|(_, role)| owner.as_str() == *role)),
        "terms custody requires NOLOGIN owners separate from serving identities: {owners:?}"
    );
    for (login, role) in LOGINS {
        let runtime = login_test_pool(&pool, *login).await;
        let grants: Vec<(String, bool, bool, bool)> = sqlx::query_as(
            "SELECT c.relname::text,
             pg_catalog.has_table_privilege(current_user,c.oid,'INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER'),
             pg_catalog.has_any_column_privilege(current_user,c.oid,'INSERT,UPDATE,REFERENCES'),
             pg_catalog.pg_has_role(current_user,c.relowner,'MEMBER') OR
             pg_catalog.pg_has_role(current_user,c.relowner,'SET') OR
             pg_catalog.pg_has_role(current_user,c.relowner,'USAGE')
             FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
             WHERE n.nspname='public' AND c.relname=ANY($1)",
        ).bind(RELATIONS).fetch_all(&runtime).await.unwrap();
        assert_eq!(grants.len(), RELATIONS.len());
        assert!(
            grants
                .iter()
                .all(|(_, table, column, owner)| !table && !column && !owner),
            "{role}: direct, inherited, PUBLIC, column or owner escalation admits publishing: {grants:?}"
        );
        runtime.close().await;
    }
}
