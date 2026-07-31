#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! EXPERIMENT X1 — a scratch probe, NOT a deliverable and NOT a regression
//! witness. It exists to answer one question by execution:
//!
//!   does a relationship declared ONLY as a link type produce an `ont_links`
//!   edge, ever?
//!
//! Read `docs/ideas/experiment-x1-x2.md` for the claim, the citations and the
//! verdict. Everything here is prefixed `x1probe_` so no stable key, type or
//! table name can be mistaken for something the product ships.
//!
//! The two halves differ in EXACTLY ONE thing. Both declare the same link type
//! with `to_object_type_id` correctly resolved to the target type; both carry a
//! `reference` property holding the referent's instance id. Only the property's
//! `config` differs — one carries `link = {stable_key, to_type}`, the other is
//! `{}`. Any difference in the edge count is therefore attributable to that
//! field and nothing else.
//!
//! Ordered CONTROL FIRST on purpose: a count query that always returned 0 would
//! "confirm" the claim while measuring nothing, so the working path is asserted
//! to return 1 before the bare path is allowed to return 0. And every read runs
//! as the genuine `console_rt` role, asserted NOSUPERUSER inline — a BYPASSRLS
//! connection would make any of this mean nothing.

use console_kernel_core::{OrgId, TraceContext, UserId};
use console_ontology_adapter_postgres::instances::{CreateInstance, PgInstanceStore};
use console_ontology_adapter_postgres::{
    CreateObjectTypeDraft, LinkTypeInput, PgOntologyError, PgOntologyStore, PropertyDefInput,
};
use console_ontology_domain::{BackingKind, InstanceId, LinkCardinality, ObjectTypeId};
use console_platform_db::with_org_conn;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use time::macros::datetime;
use uuid::Uuid;

const TARGET: &str = "x1probe_target";
/// The relationship rides the property's `config.link` — the path that works.
const LINKED: &str = "x1probe_source_linked";
/// The relationship is declared ONLY as a link type — the trap under test.
const BARE: &str = "x1probe_source_bare";
const EDGE: &str = "x1probe_edge";

const T0: OffsetDateTime = datetime!(2026-07-10 12:00 UTC);

async fn runtime_role_pool(owner_pool: &PgPool) -> PgPool {
    role_pool(owner_pool, |conn| {
        Box::pin(async move {
            sqlx::query("SET ROLE console_rt")
                .execute(conn)
                .await
                .map(|_| ())
        })
    })
    .await
}

async fn command_role_pool(owner_pool: &PgPool) -> PgPool {
    role_pool(owner_pool, |conn| {
        Box::pin(async move {
            sqlx::query("SET ROLE console_ontology_cmd")
                .execute(conn)
                .await
                .map(|_| ())
        })
    })
    .await
}

/// The role statement is a literal in each caller: a role name is an identifier,
/// which cannot be parameterised, and the SQL-injection lint rejects a
/// `format!`-built statement.
async fn role_pool<F>(owner_pool: &PgPool, set_role: F) -> PgPool
where
    F: for<'c> Fn(
            &'c mut sqlx::PgConnection,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<(), sqlx::Error>> + Send + 'c>,
        > + Send
        + Sync
        + 'static,
{
    let options = owner_pool.connect_options().as_ref().clone();
    PgPoolOptions::new()
        .max_connections(4)
        .after_connect(move |conn, _meta| set_role(conn))
        .connect_with(options)
        .await
        .unwrap()
}

async fn seed_org_and_user(owner_pool: &PgPool, org: Uuid, tag: &str) -> UserId {
    let slug = format!("org-{}", &org.simple().to_string()[..12]);
    sqlx::query(
        "INSERT INTO organizations (id, slug, name) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING",
    )
    .bind(org)
    .bind(slug)
    .bind(format!("Org {tag}"))
    // rls-arming: ok test fixture seeds RLS tables as owner during setup, before the console_rt role switch
    .execute(owner_pool)
    .await
    .unwrap();
    let user_id = UserId::new();
    sqlx::query("INSERT INTO users (id, display_name, roles, org_id) VALUES ($1, $2, $3, $4)")
        .bind(*user_id.as_uuid())
        .bind(format!("Admin {tag}"))
        .bind(["SUPER_ADMIN"].as_slice())
        .bind(org)
        // rls-arming: ok test fixture seeds RLS tables as owner during setup, before the console_rt role switch
        .execute(owner_pool)
        .await
        .unwrap();
    user_id
}

fn text_prop(key: &str) -> PropertyDefInput {
    PropertyDefInput {
        key: key.to_owned(),
        title: key.to_owned(),
        field_type: "text".to_owned(),
        config: serde_json::json!({}),
        backing_column: None,
        required: true,
        in_property_policy: false,
    }
}

/// `required: false` because the probe also wants a null referent to be legal.
fn reference_prop(config: serde_json::Value) -> PropertyDefInput {
    PropertyDefInput {
        key: "target_ref".to_owned(),
        title: "Target".to_owned(),
        field_type: "reference".to_owned(),
        config,
        backing_column: None,
        required: false,
        in_property_policy: false,
    }
}

/// A source type declaring `EDGE` as a link type whose `to_object_type_id` is the
/// REAL target type id — resolved, not null — plus one `reference` property whose
/// config the caller chooses. That config is the whole independent variable.
fn source_draft(key: &str, target: ObjectTypeId, prop_config: serde_json::Value) -> CreateObjectTypeDraft {
    CreateObjectTypeDraft {
        stable_key: key.to_owned(),
        title: key.to_owned(),
        title_property_key: Some("name".to_owned()),
        backing_kind: BackingKind::Instance,
        backing_table: None,
        primary_key_property: None,
        properties: vec![text_prop("name"), reference_prop(prop_config)],
        links: vec![LinkTypeInput {
            stable_key: EDGE.to_owned(),
            title: "Probe edge".to_owned(),
            reverse_title: None,
            // The field under test: correctly resolved to the target type, the
            // way a canvas that drew an arrow between two boxes would set it.
            to_object_type_id: Some(target),
            cardinality: LinkCardinality::OneMany,
            traversable: true,
        }],
        actions: Vec::new(),
        analytics: Vec::new(),
    }
}

fn create(type_id: ObjectTypeId, name: &str, target: Option<InstanceId>) -> CreateInstance {
    CreateInstance {
        object_type_id: type_id,
        title: name.to_owned(),
        attributes: serde_json::json!({
            "name": name,
            "target_ref": target.map(|t| t.to_string()),
        }),
        valid_from: Some(T0),
        action_type_id: None,
        reason: Some("x1 probe".to_owned()),
    }
}

/// The target type declares only `name`, and `validate_attributes` refuses an
/// attribute the schema does not carry — so it cannot reuse [`create`].
fn create_target(type_id: ObjectTypeId, name: &str) -> CreateInstance {
    CreateInstance {
        object_type_id: type_id,
        title: name.to_owned(),
        attributes: serde_json::json!({ "name": name }),
        valid_from: Some(T0),
        action_type_id: None,
        reason: Some("x1 probe".to_owned()),
    }
}

/// Every `ont_links` row out of `from`, open or closed, as the tenant sees it.
async fn edge_count(rt: &PgPool, org: OrgId, from: InstanceId) -> i64 {
    with_org_conn::<_, i64, PgOntologyError>(rt, org, |tx| {
        Box::pin(async move {
            let row = sqlx::query("SELECT COUNT(*) AS n FROM ont_links WHERE from_instance_id = $1")
                .bind(*from.as_uuid())
                .fetch_one(tx.as_mut())
                .await?;
            Ok(row.try_get::<i64, _>("n")?)
        })
    })
    .await
    .unwrap()
}

/// The link type as it actually landed in the registry, read as `console_rt`.
/// Without this the bare half has an escape hatch — "the declaration never
/// persisted" — and the experiment would prove nothing about the writer.
async fn declared_target(rt: &PgPool, org: OrgId, source: ObjectTypeId) -> Option<Uuid> {
    with_org_conn::<_, Option<Uuid>, PgOntologyError>(rt, org, |tx| {
        Box::pin(async move {
            let row = sqlx::query(
                "SELECT to_object_type_id FROM ont_link_types \
                 WHERE object_type_id = $1 AND stable_key = $2",
            )
            .bind(*source.as_uuid())
            .bind(EDGE)
            .fetch_one(tx.as_mut())
            .await?;
            Ok(row.try_get::<Option<Uuid>, _>("to_object_type_id")?)
        })
    })
    .await
    .unwrap()
}

async fn role_identity(rt: &PgPool, org: OrgId) -> (String, bool) {
    with_org_conn::<_, (String, bool), PgOntologyError>(rt, org, |tx| {
        Box::pin(async move {
            let row = sqlx::query(
                "SELECT current_user::text AS who, \
                 (SELECT rolsuper FROM pg_roles WHERE rolname = current_user) AS super",
            )
            .fetch_one(tx.as_mut())
            .await?;
            Ok((row.try_get::<String, _>("who")?, row.try_get::<bool, _>("super")?))
        })
    })
    .await
    .unwrap()
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn x1_a_link_type_alone_writes_no_edge(owner_pool: PgPool) {
    let rt = runtime_role_pool(&owner_pool).await;
    let org = OrgId::knl();
    let actor = seed_org_and_user(&owner_pool, *org.as_uuid(), "x1probe").await;

    // ---- CONTROL 0: the reader is not a superuser -------------------------
    let (who, is_super) = role_identity(&rt, org).await;
    println!("X1 CONTROL 0  current_user={who} rolsuper={is_super}");
    assert_eq!(who, "console_rt", "reads must run as the runtime role");
    assert!(!is_super, "a superuser bypasses RLS and proves nothing");

    let cmd = command_role_pool(&owner_pool).await;
    let (target_type, linked_type, bare_type) = console_platform_request_context::scope_org(
        org,
        async {
            let store = PgOntologyStore::new(owner_pool.clone()).with_command_pool(cmd);
            let target = store
                .create_object_type(
                    actor,
                    CreateObjectTypeDraft {
                        stable_key: TARGET.to_owned(),
                        title: "Target".to_owned(),
                        title_property_key: Some("name".to_owned()),
                        backing_kind: BackingKind::Instance,
                        backing_table: None,
                        primary_key_property: None,
                        properties: vec![text_prop("name")],
                        links: Vec::new(),
                        actions: Vec::new(),
                        analytics: Vec::new(),
                    },
                    TraceContext::generate(),
                    T0,
                )
                .await
                .expect("create target type")
                .id;

            // The working path: the property carries the binding as DATA.
            let linked = store
                .create_object_type(
                    actor,
                    source_draft(
                        LINKED,
                        target,
                        serde_json::json!({"link": {"stable_key": EDGE, "to_type": TARGET}}),
                    ),
                    TraceContext::generate(),
                    T0,
                )
                .await
                .expect("create linked source type")
                .id;

            // The trap: identical, minus `config.link`. If this draft were
            // REFUSED the trap would be disarmed — record that it is accepted.
            let bare = store
                .create_object_type(
                    actor,
                    source_draft(BARE, target, serde_json::json!({})),
                    TraceContext::generate(),
                    T0,
                )
                .await
                .expect("a link-type-only draft is ACCEPTED today: no validate_draft guard")
                .id;
            (target, linked, bare)
        },
    )
    .await;
    println!("X1 both drafts accepted: linked={linked_type:?} bare={bare_type:?}");

    // Both link types persisted with the target RESOLVED, so neither half can be
    // dismissed as a declaration that never landed.
    let linked_target = declared_target(&rt, org, linked_type).await;
    let bare_target = declared_target(&rt, org, bare_type).await;
    println!(
        "X1 ont_link_types.to_object_type_id  linked={linked_target:?} bare={bare_target:?} \
         target_type={:?}",
        target_type.as_uuid()
    );
    assert_eq!(
        bare_target,
        Some(*target_type.as_uuid()),
        "the bare half's link type must carry a RESOLVED target, or it tests nothing"
    );
    assert_eq!(linked_target, bare_target, "both halves declare the same edge");

    console_platform_request_context::scope_org(org, async {
        let store = PgInstanceStore::new(rt.clone());
        let referent = store
            .create_instance(actor, create_target(target_type, "referent"), TraceContext::generate(), T0)
            .await
            .expect("create referent")
            .instance
            .id;

        // ---- CONTROL 1 (RED-proving): the count query CAN see an edge ------
        // Written by the path that works. If this returned 0, every assertion
        // below would be measuring a broken query, not the system.
        let linked_instance = store
            .create_instance(
                actor,
                create(linked_type, "via-property-config-link", Some(referent)),
                TraceContext::generate(),
                T0,
            )
            .await
            .expect("create linked instance")
            .instance
            .id;
        let via_property = edge_count(&rt, org, linked_instance).await;
        println!("X1 CONTROL 1  edges via property config.link = {via_property}");
        assert_eq!(
            via_property, 1,
            "CONTROL FAILED: the counting query cannot see an edge at all"
        );

        // ---- THE MEASUREMENT: the same relationship, link type only --------
        let bare_instance = store
            .create_instance(
                actor,
                create(bare_type, "via-link-type-only", Some(referent)),
                TraceContext::generate(),
                T0,
            )
            .await
            .expect("a link-type-only instance write SUCCEEDS: no error is raised")
            .instance
            .id;
        let via_link_type = edge_count(&rt, org, bare_instance).await;
        println!("X1 MEASURED   edges via link type alone   = {via_link_type}");
        assert_eq!(
            via_link_type, 0,
            "X1 REFUTED: a bare link type DID write an edge"
        );

        // The write did not merely fail to make an edge — it reported success,
        // and the row is there. That is the silent-empty shape of the trap.
        let head = store.get_current(bare_instance).await.expect("the instance exists");
        println!(
            "X1 the row exists and reports success: title={:?} target_ref={}",
            head.instance.title, head.revision.attributes["target_ref"]
        );
        assert_eq!(
            head.revision.attributes["target_ref"], serde_json::json!(referent.to_string()),
            "the attribute holds the referent, so the reference is not lost — only the EDGE is"
        );
    })
    .await;
}
