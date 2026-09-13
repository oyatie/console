//! Applied-catalog candidates for AS1/BW31/design30. These tests inspect real
//! migrations and restricted LOGIN connections; they create no schema or roles.
//! Catalog failures admit only catalog work, not unreached Account behavior.

use super::{insert_account_fence, prepare_http_database, seed_branch};
use console_kernel_core::{OrgId, UserId};
use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool};
use std::collections::BTreeMap;

const ACCOUNT_RELATIONS: &[&str] = &[
    "accounts",
    "account_security",
    "account_security_events",
    "account_terms_acceptances",
    "account_terms_head",
    "account_terms_release_receipts",
    "auth_webauthn_credentials",
    "auth_webauthn_ceremonies",
    "auth_refresh_token_families",
    "auth_refresh_tokens",
    "auth_bootstrap_credentials",
    "auth_webauthn_ceremony_bindings",
    "auth_device_login_handoffs",
];

#[derive(FromRow)]
struct ForeignKey {
    source_table: String,
    target_table: String,
    source_columns: Vec<String>,
    target_columns: Vec<String>,
    validated: bool,
    delete_action: String,
    update_action: String,
}

impl ForeignKey {
    fn binds(&self, source: &str, target: &str, pairs: &[(&str, &str)]) -> bool {
        let actual: BTreeMap<_, _> = self
            .source_columns
            .iter()
            .map(String::as_str)
            .zip(self.target_columns.iter().map(String::as_str))
            .collect();
        self.source_table == source
            && self.target_table == target
            && self.source_columns.len() == pairs.len()
            && self.target_columns.len() == pairs.len()
            && actual == pairs.iter().copied().collect()
            && self.validated
            && matches!(self.delete_action.as_str(), "a" | "r")
            && matches!(self.update_action.as_str(), "a" | "r")
    }
}

async fn foreign_keys(pool: &PgPool) -> Vec<ForeignKey> {
    sqlx::query_as(
        "SELECT src.relname::text AS source_table, dst.relname::text AS target_table,
                ARRAY(SELECT a.attname::text FROM unnest(c.conkey) WITH ORDINALITY k(num, ord)
                      JOIN pg_catalog.pg_attribute a ON a.attrelid=c.conrelid AND a.attnum=k.num ORDER BY k.ord) AS source_columns,
                ARRAY(SELECT a.attname::text FROM unnest(c.confkey) WITH ORDINALITY k(num, ord)
                      JOIN pg_catalog.pg_attribute a ON a.attrelid=c.confrelid AND a.attnum=k.num ORDER BY k.ord) AS target_columns,
                c.convalidated AS validated, c.confdeltype::text AS delete_action, c.confupdtype::text AS update_action
         FROM pg_catalog.pg_constraint c
         JOIN pg_catalog.pg_class src ON src.oid=c.conrelid
         JOIN pg_catalog.pg_namespace ns ON ns.oid=src.relnamespace
         JOIN pg_catalog.pg_class dst ON dst.oid=c.confrelid
         JOIN pg_catalog.pg_namespace nd ON nd.oid=dst.relnamespace
         WHERE c.contype='f' AND ns.nspname='public' AND nd.nspname='public'",
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

async fn require_account_catalog(pool: &PgPool) {
    let present: Vec<String> = sqlx::query_scalar(
        "SELECT c.relname::text FROM pg_catalog.pg_class c
         JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
         WHERE n.nspname='public' AND c.relkind IN ('r','p') AND c.relname=ANY($1)",
    )
    .bind(ACCOUNT_RELATIONS)
    .fetch_all(pool)
    .await
    .unwrap();
    let missing: Vec<_> = ACCOUNT_RELATIONS
        .iter()
        .filter(|name| !present.iter().any(|found| found == **name))
        .collect();
    assert!(missing.is_empty(), "ACCOUNT_CATALOG_ABSENT: {missing:?}");
}

#[sqlx::test(migrations = false)]
async fn all_46_enrolled_actor_references_use_exact_account_or_company_actor_keys(pool: PgPool) {
    prepare_http_database(&pool).await;
    // This immutable design inventory is an independent expected map, not
    // generated from the catalog being tested. A semantic change needs review.
    let inventory = include_str!("actor-migration.csv");
    assert_eq!(
        Sha256::digest(inventory.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        "434cfef1ead44079323100e491e3b657a6b31b4820921e3220824ddc93f7f392"
    );
    assert_eq!(
        inventory.lines().next(),
        Some("table,column,target,source,constraint")
    );
    let expected: Vec<_> = inventory
        .lines()
        .skip(1)
        .map(|line| line.splitn(5, ',').collect::<Vec<_>>())
        .collect();
    assert_eq!(expected.len(), 46);
    let keys = foreign_keys(&pool).await;
    let mut violations = Vec::new();
    for row in expected {
        assert_eq!(row.len(), 5);
        let (table, column, target) = (row[0], row[1], row[2]);
        let (target_table, pairs) = match target {
            "ACCOUNT" => ("accounts", vec![(column, "id")]),
            "COMPANY_ACTOR" => (
                "company_actors",
                vec![("org_id", "org_id"), (column, "account_id")],
            ),
            _ => panic!("unreviewed actor target"),
        };
        if !keys
            .iter()
            .any(|key| key.binds(table, target_table, &pairs))
        {
            violations.push(format!(
                "{table}.{column}: missing validated non-destructive {target_table} key"
            ));
        }
        if keys.iter().any(|key| {
            key.source_table == table
                && key.source_columns.iter().any(|source| source == column)
                && key.target_table == "users"
        }) {
            violations.push(format!(
                "{table}.{column}: legacy users FK still couples attribution to employer identity"
            ));
        }
    }
    assert!(
        violations.is_empty(),
        "ACTOR_CATALOG_MISMATCH:\n{}",
        violations.join("\n")
    );
}

#[sqlx::test(migrations = false)]
async fn account_custody_has_no_company_ownership_or_destructive_foreign_keys(pool: PgPool) {
    prepare_http_database(&pool).await;
    require_account_catalog(&pool).await;
    let keys = foreign_keys(&pool).await;
    let columns: Vec<(String, String, bool)> = sqlx::query_as(
        "SELECT c.relname::text,a.attname::text,a.attnotnull
         FROM pg_catalog.pg_attribute a JOIN pg_catalog.pg_class c ON c.oid=a.attrelid
         JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
         WHERE n.nspname='public' AND c.relname=ANY($1) AND a.attnum>0 AND NOT a.attisdropped",
    )
    .bind(ACCOUNT_RELATIONS)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(!columns.iter().any(|(table, column, _)| table == "accounts"
        && matches!(
            column.as_str(),
            "org_id" | "employee_id" | "employment_id" | "roles"
        )));
    for (table, column, required) in &columns {
        assert!(
            !(column == "org_id" && *required),
            "{table}: Company provenance must not be mandatory custody ownership"
        );
    }
    for key in &keys {
        if ACCOUNT_RELATIONS.contains(&key.source_table.as_str()) {
            assert!(
                !matches!(
                    key.target_table.as_str(),
                    "users" | "employees" | "employments"
                ),
                "{} still depends on employer identity {}",
                key.source_table,
                key.target_table
            );
            assert!(
                matches!(key.delete_action.as_str(), "a" | "r"),
                "{} -> {} may erase or detach retained custody",
                key.source_table,
                key.target_table
            );
            assert!(
                matches!(key.update_action.as_str(), "a" | "r"),
                "{} -> {} permits identity rewrite",
                key.source_table,
                key.target_table
            );
        }
    }
    for table in [
        "auth_webauthn_credentials",
        "auth_webauthn_ceremonies",
        "auth_refresh_token_families",
        "auth_refresh_tokens",
        "auth_bootstrap_credentials",
    ] {
        assert!(
            keys.iter()
                .any(|key| key.binds(table, "accounts", &[("user_id", "id")])),
            "{table}: missing validated non-destructive Account FK"
        );
    }
    assert!(
        keys.iter().any(|key| key.binds(
            "auth_refresh_tokens",
            "auth_refresh_token_families",
            &[("family_id", "id"), ("user_id", "user_id")]
        )),
        "token/family cross-Account substitution is not prevented"
    );
}

#[sqlx::test(migrations = false)]
async fn terms_head_acceptance_and_event_references_bind_complete_immutable_tuples(pool: PgPool) {
    prepare_http_database(&pool).await;
    require_account_catalog(&pool).await;
    let keys = foreign_keys(&pool).await;
    for (source, target, pairs) in [
        (
            "account_terms_head",
            "account_terms_release_receipts",
            vec![
                ("release_receipt_ref", "id"),
                ("revision", "revision"),
                ("manifest_sha256", "manifest_sha256"),
            ],
        ),
        (
            "account_terms_acceptances",
            "account_terms_release_receipts",
            vec![
                ("terms_release_receipt_id", "id"),
                ("terms_release_revision", "revision"),
                ("terms_manifest_sha256", "manifest_sha256"),
            ],
        ),
        (
            "account_terms_acceptances",
            "account_security_events",
            vec![("account_id", "account_id"), ("security_event_id", "id")],
        ),
    ] {
        assert!(
            keys.iter().any(|key| key.binds(source, target, &pairs)),
            "{source} -> {target}: exact validated tuple FK absent"
        );
    }
}

#[sqlx::test(migrations = false)]
async fn handoff_target_storage_accepts_an_account_without_company_or_employer_user(pool: PgPool) {
    prepare_http_database(&pool).await;
    require_account_catalog(&pool).await;
    let account = UserId::new();
    // Privileged fixture establishes an Account root. This does not authorize
    // login for PENDING_ENROLLMENT or simulate a verified handoff approval.
    insert_account_fence(&pool, account, "PENDING_ENROLLMENT").await;
    let employer_users: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE id=$1")
        .bind(account.as_uuid())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(employer_users, 0);
    let handoff: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO auth_device_login_handoffs
         (poll_token_hash,approve_token_hash,issued_at,expires_at,target_user_id,target_org_id)
         VALUES ($1,$2,now(),now()+interval '60 seconds',$3,NULL) RETURNING id",
    ).bind(vec![0x51_u8; 32]).bind(vec![0x52_u8; 32]).bind(account.as_uuid())
        .fetch_one(&pool).await.expect("nullable target_org_id must remain usable when target Account is present; no hidden CHECK coupling");
    let stored: (uuid::Uuid, Option<uuid::Uuid>, Option<uuid::Uuid>) = sqlx::query_as(
        "SELECT target_user_id,target_org_id,approved_user_id FROM auth_device_login_handoffs WHERE id=$1",
    ).bind(handoff).fetch_one(&pool).await.unwrap();
    assert_eq!(stored, (*account.as_uuid(), None, None));
    let keys = foreign_keys(&pool).await;
    assert!(
        keys.iter().any(|key| key.binds(
            "auth_device_login_handoffs",
            "accounts",
            &[("target_user_id", "id")]
        )),
        "target must bind the Account, not merely be an unchecked UUID"
    );
}

#[sqlx::test(migrations = false)]
async fn company_login_cannot_read_or_mutate_account_custody_even_with_company_context(
    pool: PgPool,
) {
    prepare_http_database(&pool).await;
    require_account_catalog(&pool).await;
    let branch = seed_branch(&pool, "storage-positive", "storage-positive").await;
    let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let mut tx = runtime.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org',$1,true)")
        .bind(OrgId::knl().to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let visible: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM branches WHERE id=$1)")
        .bind(branch.as_uuid())
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert!(
        visible,
        "positive control must read real permitted Company data"
    );
    tx.rollback().await.unwrap();
    for relation in ACCOUNT_RELATIONS {
        for operation in [
            format!("SELECT * FROM public.{relation} LIMIT 0"),
            format!("DELETE FROM public.{relation} WHERE false"),
        ] {
            let mut tx = runtime.begin().await.unwrap();
            sqlx::query("SELECT set_config('app.current_org',$1,true)")
                .bind(OrgId::knl().to_string())
                .execute(&mut *tx)
                .await
                .unwrap();
            // Only the compile-time ACCOUNT_RELATIONS allowlist enters this
            // identifier position; no fixture, request, or database value does.
            let error = sqlx::query(sqlx::AssertSqlSafe(operation))
                .execute(&mut *tx)
                .await
                .expect_err("Company LOGIN must have no direct custody privilege");
            assert_eq!(
                error
                    .as_database_error()
                    .and_then(|error| error.code())
                    .as_deref(),
                Some("42501"),
                "{relation}: refusal must be privilege denial, not a missing relation or empty RLS result"
            );
            tx.rollback().await.unwrap();
        }
    }
    runtime.close().await;
}

#[sqlx::test(migrations = false)]
async fn account_custody_owners_and_effective_column_grants_are_separate_from_business(
    pool: PgPool,
) {
    prepare_http_database(&pool).await;
    require_account_catalog(&pool).await;
    let owners: Vec<(String, String, bool)> = sqlx::query_as(
        "SELECT c.relname::text,r.rolname::text,r.rolcanlogin FROM pg_catalog.pg_class c
         JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace JOIN pg_catalog.pg_roles r ON r.oid=c.relowner
         WHERE n.nspname='public' AND c.relname=ANY($1)",
    ).bind(ACCOUNT_RELATIONS).fetch_all(&pool).await.unwrap();
    assert_eq!(owners.len(), ACCOUNT_RELATIONS.len());
    assert!(
        owners
            .iter()
            .all(|(_, role, login)| !login
                && !matches!(role.as_str(), "console_rt" | "console_auth_rt")),
        "custody must have distinct NOLOGIN owners: {owners:?}"
    );
    let grants: Vec<(String, bool, bool, bool)> = sqlx::query_as(
        "SELECT c.relname::text,
                pg_catalog.has_table_privilege(b.oid,c.oid,'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER'),
                pg_catalog.has_any_column_privilege(b.oid,c.oid,'SELECT,INSERT,UPDATE,REFERENCES'),
                pg_catalog.pg_has_role(b.oid,c.relowner,'MEMBER') OR pg_catalog.pg_has_role(b.oid,c.relowner,'SET') OR pg_catalog.pg_has_role(b.oid,c.relowner,'USAGE')
         FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
         CROSS JOIN pg_catalog.pg_roles b WHERE n.nspname='public' AND b.rolname='console_rt' AND c.relname=ANY($1)",
    ).bind(ACCOUNT_RELATIONS).fetch_all(&pool).await.unwrap();
    assert_eq!(grants.len(), ACCOUNT_RELATIONS.len());
    assert!(
        grants
            .iter()
            .all(|(_, table, column, owner)| !table && !column && !owner),
        "effective inherited/PUBLIC/column privilege or owner escalation: {grants:?}"
    );
}
