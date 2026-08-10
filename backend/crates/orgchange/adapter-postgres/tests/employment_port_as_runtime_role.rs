#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `EmploymentPort` proven against a REAL PostgreSQL as the genuine runtime role
//! `console_rt` — never the BYPASSRLS superuser the `#[sqlx::test]` pool
//! connects as, which would mask a broken `org_isolation` policy.
//! `a_foreign_tenant_is_invisible_and_unwritable_to_the_runtime_role` is what
//! makes that claim observable: it is the only test that crosses a tenant
//! boundary, it asserts through the owner pool that the foreign rows genuinely
//! EXIST first, and it dies when `org_isolation` is loosened to `USING (true)`.
//!
//! WHY `#[sqlx::test]` IS NOT OPTIONAL HERE. Migration 0196 refuses a superuser
//! applier unless `CURRENT_DATABASE()` matches `^_sqlx_test_[A-Za-z0-9_]{52}$`
//! with the `console.sqlx_test_bootstrap` marker set, so the schema itself
//! admits exactly one applier and a hand-rolled `sqlx::migrate!` harness is not
//! a design choice that was available.
//!
//! WHY `execute` IS CALLED FROM `spawn_blocking`. `CanonicalPort::execute` is
//! SYNCHRONOUS — `canonical-domain` declares it so and this lane may not edit
//! that crate — so the adapter bridges to `sqlx` with `Handle::block_on`, which
//! panics inside an async context. A `spawn_blocking` thread is not one.
//!
//! IMMUTABILITY IS THE TRIGGER, AND ONLY THE TRIGGER.
//! `ops/postgres-reconcile-topology.sh` runs `ALTER DEFAULT PRIVILEGES FOR ROLE
//! console_app … GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO console_rt`
//! BEFORE migrations, so in the deployed database `console_rt` DOES hold UPDATE
//! on `employment_revisions` — 0214's own header says so.
//! `#[sqlx::test]` applies migrations as the cluster superuser, for whom no such
//! per-role default privilege exists, so an UPDATE here would otherwise be
//! refused by a `42501` that does not exist where the code ships.
//! `a_revision_is_append_only_and_the_trigger_is_what_refuses_the_update`
//! therefore GRANTs UPDATE first, asserts the grant is really held, and only
//! then asserts the `P0001` the trigger raises.
//!
//! THE LEGACY HEAD IS PART OF THE PORT, NOT NEXT TO IT. `employees` is the
//! fourth table the contract assigns to `Employment`, and
//! `a_promotion_carries_the_new_state_onto_the_legacy_employees_head` is the
//! test that makes the relocation of `backend/app/src/hr.rs`'s three statements
//! meaningful rather than cosmetic: the same
//! `employment::apply_employment_change` the REST lifecycle handler now calls is
//! the statement the port itself issues.

use console_kernel_core::{OrgId, UserId};
use console_ontology_canonical_domain::{
    CanonicalPort, CommandId, CommandReceipt, DispatchTarget, EmploymentPort, ObjectKey,
    ReceiptOwner,
};
use console_orgchange_adapter_postgres::employment::{
    EmploymentAttributes, EmploymentCommand, EmploymentError, EmploymentQuery, PgEmploymentPort,
};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

const ORG: Uuid = Uuid::from_u128(0xe3b0_0000_0000_0000_0000_0000_0000_0001);
const FOREIGN_ORG: Uuid = Uuid::from_u128(0xe3b0_0000_0000_0000_0000_0000_0000_0002);

/// The port must satisfy the NAMED trait, not merely `CanonicalPort`. The
/// blanket impl in `canonical-domain` makes `EmploymentPort` an alias for
/// `CanonicalPort<Object = Employment>`, so this bound stops holding the moment
/// the adapter is retargeted at a different object.
fn assert_implements_employment_port<P: EmploymentPort>() {}

/// `console_platform_test_support::runtime_role_pool`, inlined: adding that
/// crate as a dev-dependency rewrites this package's entry in
/// `backend/Cargo.lock` beyond the two names this lane structurally needs.
async fn runtime_role_pool(owner_pool: &PgPool) -> PgPool {
    let options = owner_pool.connect_options().as_ref().clone();
    PgPoolOptions::new()
        .max_connections(4)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET ROLE console_rt").execute(conn).await?;
                Ok(())
            })
        })
        .connect_with(options)
        .await
        .expect("connect console_rt-role test pool")
}

/// Seeded as the migration owner, before any role switch.
async fn seed_org_and_super_admin(owner_pool: &PgPool, org: Uuid, tag: &str) -> UserId {
    sqlx::query(
        "INSERT INTO organizations (id, slug, name) VALUES ($1, $2, $3) \
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(org)
    .bind(format!("org-{tag}"))
    .bind(format!("Org {tag}"))
    .execute(owner_pool)
    .await
    .unwrap();
    let user_id = UserId::new();
    sqlx::query("INSERT INTO users (id, display_name, roles, org_id) VALUES ($1, $2, $3, $4)")
        .bind(*user_id.as_uuid())
        .bind(format!("Admin {tag}"))
        .bind(["SUPER_ADMIN"].as_slice())
        .bind(org)
        .execute(owner_pool)
        .await
        .unwrap();
    user_id
}

async fn seed_employee(owner_pool: &PgPool, org: Uuid, source_key: &str) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO employees \
         (org_id, company, name, source_filename, source_sheet, source_row, source_key) \
         VALUES ($1, 'ACME', $2, 'seed.xlsx', 'Sheet1', 1, $2) RETURNING id",
    )
    .bind(org)
    .bind(source_key)
    .fetch_one(owner_pool)
    .await
    .unwrap()
}

/// The tenant, its actor, and the port built on a `console_rt` pool.
async fn fixture(owner_pool: &PgPool) -> (OrgId, UserId, PgEmploymentPort) {
    let actor = seed_org_and_super_admin(owner_pool, ORG, "employment").await;
    let runtime_pool = runtime_role_pool(owner_pool).await;
    let port = PgEmploymentPort::new(runtime_pool, tokio::runtime::Handle::current());
    (OrgId::from_uuid(ORG), actor, port)
}

/// Drive the SYNCHRONOUS `execute` off the runtime's worker thread.
async fn execute(
    port: &PgEmploymentPort,
    command: EmploymentCommand,
) -> Result<CommandReceipt, EmploymentError> {
    let port = port.clone();
    tokio::task::spawn_blocking(move || port.execute(&command))
        .await
        .unwrap()
}

fn attributes(org_unit: &str, position: &str, status: &str) -> EmploymentAttributes {
    EmploymentAttributes {
        company: "ACME".to_owned(),
        org_unit: Some(org_unit.to_owned()),
        position: Some(position.to_owned()),
        employment_status: status.to_owned(),
    }
}

fn command(org: OrgId, actor: UserId, query: EmploymentQuery) -> EmploymentCommand {
    EmploymentCommand {
        org_id: org,
        command_id: CommandId::from_uuid(Uuid::new_v4()),
        actor_id: actor,
        query,
    }
}

fn at(seconds: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_800_000_000 + seconds).unwrap()
}

fn employment_of(receipt: &CommandReceipt) -> Uuid {
    receipt.result()["employment_id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

const COUNT_EMPLOYEES: &str = "SELECT count(*)::bigint FROM employees";
const COUNT_HEADS: &str = "SELECT count(*)::bigint FROM employment_heads";
const COUNT_REVISIONS: &str = "SELECT count(*)::bigint FROM employment_revisions";
const COUNT_BINDINGS: &str = "SELECT count(*)::bigint FROM employment_source_bindings";
const COUNT_RECEIPTS: &str = "SELECT count(*)::bigint FROM ont_action_command_receipts";

/// The four tables the contract assigns to `Employment`, each with the count
/// over it. `sqlx` 0.9 accepts only `&'static str` SQL, so the table name cannot
/// be interpolated and every statement here is a literal.
const OWNED_TABLES: [(&str, &str); 4] = [
    ("employees", COUNT_EMPLOYEES),
    ("employment_heads", COUNT_HEADS),
    ("employment_revisions", COUNT_REVISIONS),
    ("employment_source_bindings", COUNT_BINDINGS),
];

async fn count_rows(owner_pool: &PgPool, sql: &'static str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(owner_pool).await.unwrap()
}

/// The SQLSTATE and the message PostgreSQL actually returned, so a test quotes
/// the real error rather than a paraphrase of it.
fn database_error(error: &EmploymentError) -> (String, String) {
    let EmploymentError::Database(sqlx_error) = error else {
        panic!("expected a database error, got {error:?}");
    };
    let database = sqlx_error
        .as_database_error()
        .unwrap_or_else(|| panic!("expected a database error, got {sqlx_error:?}"));
    (
        database
            .code()
            .map(|code| code.into_owned())
            .unwrap_or_default(),
        database.message().to_owned(),
    )
}

/// The whole revision row as JSONB, so "unchanged" means the whole row and not
/// the two columns a test remembered to name.
async fn revision_snapshot(owner_pool: &PgPool, employment: Uuid, version: i64) -> String {
    sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT to_jsonb(r) FROM employment_revisions r \
         WHERE org_id = $1 AND employment_id = $2 AND version = $3",
    )
    .bind(ORG)
    .bind(employment)
    .bind(version)
    .fetch_one(owner_pool)
    .await
    .unwrap()
    .to_string()
}

/// The legacy head's four canonical columns.
async fn legacy_head(
    owner_pool: &PgPool,
    employee: Uuid,
) -> (String, Option<String>, Option<String>, String) {
    let row = sqlx::query(
        "SELECT company, org_unit, position, employment_status FROM employees \
         WHERE org_id = $1 AND id = $2",
    )
    .bind(ORG)
    .bind(employee)
    .fetch_one(owner_pool)
    .await
    .unwrap();
    (
        row.get("company"),
        row.get("org_unit"),
        row.get("position"),
        row.get("employment_status"),
    )
}

/// One appointed employment: the seeded legacy row, the head it opened.
async fn appointed(
    owner_pool: &PgPool,
    org: OrgId,
    actor: UserId,
    port: &PgEmploymentPort,
    tag: &str,
) -> (Uuid, Uuid) {
    let employee = seed_employee(owner_pool, ORG, tag).await;
    let receipt = execute(
        port,
        command(
            org,
            actor,
            EmploymentQuery::Appoint {
                employee_id: employee,
                valid_from: at(0),
                attributes: attributes("영업본부", "사원", "ACTIVE"),
            },
        ),
    )
    .await
    .unwrap();
    (employee, employment_of(&receipt))
}

/// The stated invariant is that `valid_to` is the ONE mutable column on a head.
///
/// Nothing enforced it: `GRANT ... UPDATE, DELETE ON employment_heads TO
/// console_rt` lets an org-armed session rewrite `valid_from` or `created_at`,
/// and such an update preserves `org_id` so the RLS policy passes untouched. A
/// stated invariant the grant does not restrict is a claim about a control, not
/// a control. Both directions are asserted, because a trigger that refuses
/// everything would satisfy the first half and break the column the design
/// exists to allow.
#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn only_valid_to_may_move_on_an_employment_head(owner_pool: PgPool) {
    let (org, _actor, _port) = fixture(&owner_pool).await;

    let head: Uuid = sqlx::query_scalar(
        "INSERT INTO employment_heads (org_id, valid_from) VALUES ($1, now()) RETURNING id",
    )
    .bind(org.as_uuid())
    .fetch_one(&owner_pool)
    .await
    .expect("seed a head");

    // PERMITTED: the one column the design names.
    sqlx::query("UPDATE employment_heads SET valid_to = now() WHERE id = $1")
        .bind(head)
        .execute(&owner_pool)
        .await
        .expect("valid_to is the legitimate mutation and must still work");

    // REFUSED: anything else, named in the error rather than a bare refusal.
    let refused = sqlx::query("UPDATE employment_heads SET valid_from = now() WHERE id = $1")
        .bind(head)
        .execute(&owner_pool)
        .await
        .expect_err("valid_from must not move on a head");
    let text = refused.to_string();
    assert!(
        text.contains("only valid_to may change") && text.contains("valid_from"),
        "the refusal must name the column that moved: {text}"
    );
}

#[test]
fn the_contract_identity_is_copied_verbatim_and_the_port_is_the_named_one() {
    assert_implements_employment_port::<PgEmploymentPort>();
    assert_eq!(ObjectKey::Employment.as_str(), "employment");
    assert_eq!(
        ObjectKey::Employment.owned_tables(),
        [
            "employees",
            "employment_heads",
            "employment_revisions",
            "employment_source_bindings"
        ]
    );
    assert_eq!(
        ObjectKey::Employment.owner_crate(),
        "console-orgchange-adapter-postgres",
        "this crate is the owner, which is why the port lives here"
    );
    for (target, wire) in [
        (DispatchTarget::HrAppoint, "hr.appoint"),
        (DispatchTarget::HrPromote, "hr.promote"),
        (DispatchTarget::HrTransfer, "hr.transfer"),
    ] {
        assert_eq!(target.as_str(), wire);
        assert_eq!(target.object(), ObjectKey::Employment);
    }
}

#[test]
fn preflight_is_pure_and_blocks_what_the_database_would_only_refuse_at_23514() {
    let blank = EmploymentQuery::Appoint {
        employee_id: Uuid::new_v4(),
        valid_from: at(0),
        attributes: EmploymentAttributes {
            company: "   ".to_owned(),
            org_unit: None,
            position: None,
            employment_status: "PROBATION".to_owned(),
        },
    };
    let preflight = <PgEmploymentPort as CanonicalPort>::preflight(&blank);
    assert!(!preflight.is_ok());
    assert_eq!(
        preflight.blockers(),
        [
            "company must not be blank".to_owned(),
            "employment_status must be one of [\"ACTIVE\", \"EXITED\", \"UNKNOWN\"]".to_owned(),
        ]
    );

    let nil_head = <PgEmploymentPort as CanonicalPort>::preflight(&EmploymentQuery::Promote {
        employment_id: Uuid::nil(),
        valid_from: at(0),
        attributes: attributes("영업본부", "사원", "ACTIVE"),
    });
    assert!(!nil_head.is_ok());
    assert_eq!(
        nil_head.blockers(),
        ["employment_id must not be nil".to_owned()]
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn an_appointment_opens_a_head_binds_the_legacy_row_and_appends_revision_one(
    owner_pool: PgPool,
) {
    let (org, actor, port) = fixture(&owner_pool).await;
    let employee = seed_employee(&owner_pool, ORG, "appoint-1").await;
    let query = EmploymentQuery::Appoint {
        employee_id: employee,
        valid_from: at(0),
        attributes: attributes("영업본부", "사원", "ACTIVE"),
    };
    assert!(<PgEmploymentPort as CanonicalPort>::preflight(&query).is_ok());

    let receipt = execute(&port, command(org, actor, query)).await.unwrap();

    assert_eq!(receipt.org_id(), org);
    assert_eq!(receipt.actor_id(), actor);
    assert_eq!(
        receipt.owner(),
        ReceiptOwner::Canonical(ObjectKey::Employment),
        "the receipt must be owned by the canonical Employment object"
    );
    assert_eq!(receipt.target(), DispatchTarget::HrAppoint);
    assert_eq!(receipt.result()["version"].as_i64(), Some(1));

    let employment_id = employment_of(&receipt);
    let row = sqlx::query(
        "SELECT h.valid_from, h.valid_to, r.version, r.attributes, r.valid_from AS revision_from, \
                r.command_id, r.payload_digest, b.employee_id \
         FROM employment_heads h \
         JOIN employment_revisions r ON r.org_id = h.org_id AND r.employment_id = h.id \
         JOIN employment_source_bindings b ON b.org_id = h.org_id AND b.employment_id = h.id \
         WHERE h.org_id = $1 AND h.id = $2",
    )
    .bind(ORG)
    .bind(employment_id)
    .fetch_one(&owner_pool)
    .await
    .unwrap();

    assert_eq!(row.get::<i64, _>("version"), 1);
    assert_eq!(row.get::<OffsetDateTime, _>("valid_from"), at(0));
    assert_eq!(row.get::<Option<OffsetDateTime>, _>("valid_to"), None);
    assert_eq!(row.get::<OffsetDateTime, _>("revision_from"), at(0));
    assert_eq!(row.get::<Uuid, _>("employee_id"), employee);
    assert_eq!(
        row.get::<serde_json::Value, _>("attributes"),
        serde_json::json!({
            "company": "ACME",
            "employment_status": "ACTIVE",
            "org_unit": "영업본부",
            "position": "사원",
        })
    );
    assert_eq!(
        row.get::<Uuid, _>("command_id"),
        *receipt.command_id().as_uuid()
    );
    assert_eq!(
        row.get::<Vec<u8>, _>("payload_digest"),
        receipt.payload_digest().to_vec(),
        "the stored digest is the 32 bytes the receipt carries"
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_promotion_carries_the_new_state_onto_the_legacy_employees_head(owner_pool: PgPool) {
    let (org, actor, port) = fixture(&owner_pool).await;
    let (employee, employment_id) = appointed(&owner_pool, org, actor, &port, "promote-1").await;

    // The appointment BINDS the legacy row and does not rewrite it, so the
    // assertion below is not vacuous: these are the seeded values.
    assert_eq!(
        legacy_head(&owner_pool, employee).await,
        ("ACME".to_owned(), None, None, "ACTIVE".to_owned())
    );

    let promoted = execute(
        &port,
        command(
            org,
            actor,
            EmploymentQuery::Promote {
                employment_id,
                valid_from: at(86_400),
                attributes: attributes("영업본부", "팀장", "ACTIVE"),
            },
        ),
    )
    .await
    .unwrap();
    assert_eq!(promoted.target(), DispatchTarget::HrPromote);
    assert_eq!(promoted.result()["version"].as_i64(), Some(2));

    assert_eq!(
        legacy_head(&owner_pool, employee).await,
        (
            "ACME".to_owned(),
            Some("영업본부".to_owned()),
            Some("팀장".to_owned()),
            "ACTIVE".to_owned()
        ),
        "the promotion must reach `employees`, the legacy compatibility head"
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_transfer_appends_a_third_revision_and_the_intervals_stay_half_open(owner_pool: PgPool) {
    let (org, actor, port) = fixture(&owner_pool).await;
    let (employee, employment_id) = appointed(&owner_pool, org, actor, &port, "transfer-1").await;

    execute(
        &port,
        command(
            org,
            actor,
            EmploymentQuery::Promote {
                employment_id,
                valid_from: at(86_400),
                attributes: attributes("영업본부", "팀장", "ACTIVE"),
            },
        ),
    )
    .await
    .unwrap();

    let transferred = execute(
        &port,
        command(
            org,
            actor,
            EmploymentQuery::Transfer {
                employment_id,
                valid_from: at(172_800),
                attributes: attributes("기술본부", "팀장", "ACTIVE"),
            },
        ),
    )
    .await
    .unwrap();
    assert_eq!(
        transferred.target(),
        DispatchTarget::HrTransfer,
        "the target is what distinguishes a transfer from a promotion in the receipt"
    );
    assert_eq!(transferred.result()["version"].as_i64(), Some(3));

    assert_eq!(
        legacy_head(&owner_pool, employee).await.1,
        Some("기술본부".to_owned())
    );

    // 0214 stores no per-revision `valid_to`: an interval ends where the next
    // one begins. Derive them and assert the window is a partition, not a set
    // that may overlap.
    let windows: Vec<(i64, OffsetDateTime, Option<OffsetDateTime>)> = sqlx::query(
        "SELECT version, valid_from, lead(valid_from) OVER (ORDER BY valid_from) AS derived_to \
         FROM employment_revisions WHERE org_id = $1 AND employment_id = $2 ORDER BY version",
    )
    .bind(ORG)
    .bind(employment_id)
    .fetch_all(&owner_pool)
    .await
    .unwrap()
    .into_iter()
    .map(|row| {
        (
            row.get::<i64, _>("version"),
            row.get::<OffsetDateTime, _>("valid_from"),
            row.get::<Option<OffsetDateTime>, _>("derived_to"),
        )
    })
    .collect();

    assert_eq!(
        windows,
        vec![
            (1, at(0), Some(at(86_400))),
            (2, at(86_400), Some(at(172_800))),
            (3, at(172_800), None),
        ]
    );

    // A second revision at an instant already taken is refused structurally.
    let clash = execute(
        &port,
        command(
            org,
            actor,
            EmploymentQuery::Transfer {
                employment_id,
                valid_from: at(172_800),
                attributes: attributes("총무본부", "팀장", "ACTIVE"),
            },
        ),
    )
    .await
    .unwrap_err();
    let (code, message) = database_error(&clash);
    assert_eq!(code, "23505", "got {message}");
    assert!(
        message.contains("employment_revisions_org_id_employment_id_valid_from_key"),
        "non-overlap is structural — UNIQUE (org_id, employment_id, valid_from) — got {message}"
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_revision_is_append_only_and_the_trigger_is_what_refuses_the_update(owner_pool: PgPool) {
    let (org, actor, port) = fixture(&owner_pool).await;
    let (_employee, employment_id) = appointed(&owner_pool, org, actor, &port, "immutable-1").await;
    let before = revision_snapshot(&owner_pool, employment_id, 1).await;

    // Reproduce the DEPLOYED ACL before asserting anything about it. See the
    // module doc: production applies migrations as `console_app`, after
    // `ALTER DEFAULT PRIVILEGES … GRANT … UPDATE … TO console_rt`, so the
    // runtime role holds UPDATE there.
    sqlx::query("GRANT UPDATE ON employment_revisions TO console_rt")
        .execute(&owner_pool)
        .await
        .unwrap();
    let runtime_holds_update: bool = sqlx::query_scalar(
        "SELECT has_table_privilege('console_rt', 'employment_revisions', 'UPDATE')",
    )
    .fetch_one(&owner_pool)
    .await
    .unwrap();
    assert!(
        runtime_holds_update,
        "the production ACL must be in place, or the assertion below observes a privilege error \
         that does not exist where this code ships"
    );

    let runtime_pool = runtime_role_pool(&owner_pool).await;
    let mut tx = runtime_pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(ORG.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let as_runtime_role = sqlx::query(
        "UPDATE employment_revisions SET attributes = '{\"company\":\"forged\"}'::jsonb \
         WHERE org_id = $1 AND employment_id = $2",
    )
    .bind(ORG)
    .bind(employment_id)
    .execute(&mut *tx)
    .await
    .unwrap_err();
    let runtime_error = as_runtime_role.as_database_error().unwrap();
    assert_eq!(
        runtime_error.code().unwrap(),
        "P0001",
        "the trigger is the enforcement, not the grant; got {}",
        runtime_error.message()
    );
    assert_eq!(
        runtime_error.message(),
        "canonical employment table employment_revisions: UPDATE is refused, the row is immutable"
    );
    drop(tx);

    // The owner is refused by the same trigger; ownership buys no exemption.
    // DELETE too — the revisions are the history the derived intervals are read
    // out of, so a removed row silently rewrites its predecessor's window.
    let as_owner =
        sqlx::query("UPDATE employment_revisions SET version = version + 100 WHERE org_id = $1")
            .bind(ORG)
            .execute(&owner_pool)
            .await
            .unwrap_err();
    assert_eq!(
        as_owner.as_database_error().unwrap().message(),
        "canonical employment table employment_revisions: UPDATE is refused, the row is immutable"
    );
    let deleted = sqlx::query("DELETE FROM employment_revisions WHERE org_id = $1")
        .bind(ORG)
        .execute(&owner_pool)
        .await
        .unwrap_err();
    assert_eq!(
        deleted.as_database_error().unwrap().message(),
        "canonical employment table employment_revisions: DELETE is refused, the row is immutable"
    );

    let after = revision_snapshot(&owner_pool, employment_id, 1).await;
    assert_eq!(before, after, "a refused write rewrote the revision row");
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_foreign_tenant_is_invisible_and_unwritable_to_the_runtime_role(owner_pool: PgPool) {
    let (_org, _actor, _port) = fixture(&owner_pool).await;
    let foreign_actor = seed_org_and_super_admin(&owner_pool, FOREIGN_ORG, "foreign").await;
    let foreign_employee = seed_employee(&owner_pool, FOREIGN_ORG, "foreign-1").await;

    // A head, a revision and a binding that genuinely exist — under the OTHER
    // tenant. Seeded through the BYPASSRLS owner pool, which is the only way to
    // put rows on the far side of the boundary being tested.
    let foreign_employment: Uuid = sqlx::query_scalar(
        "INSERT INTO employment_heads (org_id, valid_from) VALUES ($1, now()) RETURNING id",
    )
    .bind(FOREIGN_ORG)
    .fetch_one(&owner_pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO employment_revisions \
         (org_id, employment_id, version, command_id, actor_id, payload_digest, valid_from, \
          attributes, receipt) \
         VALUES ($1, $2, 1, gen_random_uuid(), $3, $4, now(), '{}'::jsonb, '{}'::jsonb)",
    )
    .bind(FOREIGN_ORG)
    .bind(foreign_employment)
    .bind(*foreign_actor.as_uuid())
    .bind([0_u8; 32].as_slice())
    .execute(&owner_pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO employment_source_bindings \
         (org_id, employee_id, employment_id, actor_id, payload_digest) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(FOREIGN_ORG)
    .bind(foreign_employee)
    .bind(foreign_employment)
    .bind(*foreign_actor.as_uuid())
    .bind([0_u8; 32].as_slice())
    .execute(&owner_pool)
    .await
    .unwrap();

    // The rows are really there — otherwise "sees zero" below would be the
    // vacuous kind of zero, an empty table rather than a policy doing work.
    for (table, count) in OWNED_TABLES {
        assert_eq!(
            count_rows(&owner_pool, count).await,
            1,
            "{table} must hold exactly the foreign tenant's row before the boundary is tested"
        );
    }

    // A `console_rt` session armed for ORG, which owns none of them.
    let runtime_pool = runtime_role_pool(&owner_pool).await;
    let mut tx = runtime_pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org', $1, true)")
        .bind(ORG.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    for (table, count) in OWNED_TABLES {
        let visible: i64 = sqlx::query_scalar(count).fetch_one(&mut *tx).await.unwrap();
        assert_eq!(
            visible, 0,
            "org_isolation must hide {table} rows belonging to another tenant"
        );
    }

    // And the write half: WITH CHECK refuses a row planted in the other tenant.
    let refused =
        sqlx::query("INSERT INTO employment_heads (org_id, valid_from) VALUES ($1, now())")
            .bind(FOREIGN_ORG)
            .execute(&mut *tx)
            .await
            .unwrap_err();
    let error = refused.as_database_error().unwrap();
    assert_eq!(error.code().unwrap(), "42501", "got {}", error.message());
    assert_eq!(
        error.message(),
        "new row violates row-level security policy for table \"employment_heads\""
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_second_binding_for_the_same_employee_is_refused(owner_pool: PgPool) {
    let (org, actor, port) = fixture(&owner_pool).await;
    let (employee, employment_id) = appointed(&owner_pool, org, actor, &port, "rebind-1").await;

    let refused = execute(
        &port,
        command(
            org,
            actor,
            EmploymentQuery::Appoint {
                employee_id: employee,
                valid_from: at(1),
                attributes: attributes("영업본부", "사원", "ACTIVE"),
            },
        ),
    )
    .await
    .unwrap_err();
    let (code, message) = database_error(&refused);
    assert_eq!(code, "23505", "got {message}");
    assert!(
        message.contains("employment_source_bindings_pkey"),
        "the primary key (org_id, employee_id) must be what refuses it; got {message}"
    );

    // The refused command rolled back whole: one binding, no orphan head.
    let bound: Vec<Uuid> = sqlx::query_scalar(
        "SELECT employment_id FROM employment_source_bindings \
         WHERE org_id = $1 AND employee_id = $2",
    )
    .bind(ORG)
    .bind(employee)
    .fetch_all(&owner_pool)
    .await
    .unwrap();
    assert_eq!(bound, vec![employment_id]);
    assert_eq!(
        count_rows(&owner_pool, COUNT_HEADS).await,
        1,
        "the refused command must persist no head"
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn an_actor_from_another_org_is_refused(owner_pool: PgPool) {
    let (org, _actor, port) = fixture(&owner_pool).await;
    let foreign_actor = seed_org_and_super_admin(&owner_pool, FOREIGN_ORG, "foreign").await;
    let employee = seed_employee(&owner_pool, ORG, "foreign-actor-1").await;

    let refused = execute(
        &port,
        command(
            org,
            foreign_actor,
            EmploymentQuery::Appoint {
                employee_id: employee,
                valid_from: at(0),
                attributes: attributes("영업본부", "사원", "ACTIVE"),
            },
        ),
    )
    .await
    .unwrap_err();
    let (code, message) = database_error(&refused);
    assert_eq!(code, "23503", "got {message}");
    assert!(
        message.contains("employment_revisions"),
        "the (actor_id, org_id) foreign key on employment_revisions must refuse it; got {message}"
    );
    assert_eq!(
        count_rows(&owner_pool, COUNT_HEADS).await,
        0,
        "a refused command must persist no head"
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_repeat_of_the_same_command_replays_the_stored_receipt(owner_pool: PgPool) {
    let (org, actor, port) = fixture(&owner_pool).await;
    let employee = seed_employee(&owner_pool, ORG, "replay-1").await;
    let command_id = CommandId::from_uuid(Uuid::new_v4());
    let query = EmploymentQuery::Appoint {
        employee_id: employee,
        valid_from: at(0),
        attributes: attributes("영업본부", "사원", "ACTIVE"),
    };

    let first = execute(
        &port,
        EmploymentCommand {
            org_id: org,
            command_id,
            actor_id: actor,
            query: query.clone(),
        },
    )
    .await
    .unwrap();

    let replayed = execute(
        &port,
        EmploymentCommand {
            org_id: org,
            command_id,
            actor_id: actor,
            query,
        },
    )
    .await
    .unwrap();

    assert_eq!(
        replayed, first,
        "a repeat of the same command id must replay the stored receipt verbatim"
    );
    assert_eq!(count_rows(&owner_pool, COUNT_HEADS).await, 1);
    assert_eq!(
        count_rows(&owner_pool, COUNT_REVISIONS).await,
        1,
        "a replayed command must append no revision"
    );
    assert_eq!(count_rows(&owner_pool, COUNT_BINDINGS).await, 1);
    assert_eq!(count_rows(&owner_pool, COUNT_RECEIPTS).await, 1);
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_repeat_with_a_different_payload_is_refused(owner_pool: PgPool) {
    let (org, actor, port) = fixture(&owner_pool).await;
    let employee = seed_employee(&owner_pool, ORG, "conflict-1").await;
    let command_id = CommandId::from_uuid(Uuid::new_v4());

    execute(
        &port,
        EmploymentCommand {
            org_id: org,
            command_id,
            actor_id: actor,
            query: EmploymentQuery::Appoint {
                employee_id: employee,
                valid_from: at(0),
                attributes: attributes("영업본부", "사원", "ACTIVE"),
            },
        },
    )
    .await
    .unwrap();

    let refused = execute(
        &port,
        EmploymentCommand {
            org_id: org,
            command_id,
            actor_id: actor,
            query: EmploymentQuery::Appoint {
                employee_id: employee,
                valid_from: at(0),
                attributes: attributes("영업본부", "임원", "ACTIVE"),
            },
        },
    )
    .await
    .unwrap_err();

    assert!(
        matches!(refused, EmploymentError::DigestConflict(id) if id == *command_id.as_uuid()),
        "got {refused:?}"
    );
    assert_eq!(
        count_rows(&owner_pool, COUNT_REVISIONS).await,
        1,
        "a refused repeat must append no revision"
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_blocked_preflight_never_reaches_the_database(owner_pool: PgPool) {
    let (org, actor, port) = fixture(&owner_pool).await;

    let blocked = execute(
        &port,
        command(
            org,
            actor,
            EmploymentQuery::Promote {
                employment_id: Uuid::nil(),
                valid_from: at(0),
                attributes: attributes("영업본부", "사원", "ACTIVE"),
            },
        ),
    )
    .await
    .unwrap_err();
    let EmploymentError::Blocked(blockers) = &blocked else {
        panic!("expected a blocked preflight, got {blocked:?}");
    };
    assert_eq!(blockers, &["employment_id must not be nil".to_owned()]);
    assert_eq!(count_rows(&owner_pool, COUNT_REVISIONS).await, 0);
    assert_eq!(count_rows(&owner_pool, COUNT_RECEIPTS).await, 0);
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn a_stored_receipt_naming_no_dispatch_target_is_refused(owner_pool: PgPool) {
    let (org, actor, port) = fixture(&owner_pool).await;
    let employee = seed_employee(&owner_pool, ORG, "unreadable-1").await;
    let command_id = CommandId::from_uuid(Uuid::new_v4());
    let query = EmploymentQuery::Appoint {
        employee_id: employee,
        valid_from: at(0),
        attributes: attributes("영업본부", "사원", "ACTIVE"),
    };
    let accepted = execute(
        &port,
        EmploymentCommand {
            org_id: org,
            command_id,
            actor_id: actor,
            query: query.clone(),
        },
    )
    .await
    .unwrap();

    // Put back a row carrying the SAME digest — so the replay gets past the
    // digest comparison — but a receipt naming no dispatch target, which is the
    // shape an `ontology.action` row has. 0177's trigger refuses UPDATE and
    // DELETE per row and TRUNCATE is statement-level, so this is the only way a
    // test can stand a hostile row where a good one was.
    sqlx::query("TRUNCATE ont_action_command_receipts")
        .execute(&owner_pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO ont_action_command_receipts \
         (org_id, command_id, actor_id, payload_digest, receipt, created_at) \
         VALUES ($1, $2, $3, $4, $5, now())",
    )
    .bind(ORG)
    .bind(*command_id.as_uuid())
    .bind(*actor.as_uuid())
    .bind(accepted.payload_digest().as_slice())
    .bind(
        serde_json::json!({ "employment_id": employment_of(&accepted).to_string(), "version": 1 }),
    )
    .execute(&owner_pool)
    .await
    .unwrap();

    let refused = execute(
        &port,
        EmploymentCommand {
            org_id: org,
            command_id,
            actor_id: actor,
            query,
        },
    )
    .await
    .unwrap_err();
    assert!(
        matches!(refused, EmploymentError::UnreadableReceipt(id, _) if id == *command_id.as_uuid()),
        "a receipt the roster cannot read must be refused, never replayed; got {refused:?}"
    );
}
