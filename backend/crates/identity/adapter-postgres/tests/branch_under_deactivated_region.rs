#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Region-active guard for branch creation / re-parenting (console-lx6).
//!
//! A branch created under, or moved into, a DEACTIVATED region is hidden from
//! every org-tree read (`list_regions` filters `deactivated_at IS NULL`) while
//! still carrying live users/equipment — it vanishes with no 409. The adapter
//! must refuse both entry points with a `Conflict` while the target region is
//! deactivated, inside the same tenant-armed transaction, so the check cannot
//! race a concurrent `deactivate_region`.

use console_identity_adapter_postgres::{PgOrgError, PgOrgStore};
use console_identity_application::{
    CreateBranchCommand, DeactivateRegionCommand, UpdateBranchCommand,
};
use console_kernel_core::{
    BranchId, BranchScope, ErrorKind, OrgId, RegionId, TraceContext, UserId,
};
use console_platform_request_context::CURRENT_ORG;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use time::OffsetDateTime;
use uuid::Uuid;

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
        .unwrap()
}

async fn seed_org(owner_pool: &PgPool, org: Uuid, tag: &str) {
    let mut tx = owner_pool.begin().await.unwrap();
    sqlx::query("SET LOCAL row_security = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO organizations (id, slug, name) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING",
    )
    .bind(org)
    .bind(format!("org-{}", tag.to_lowercase()))
    .bind(format!("Org {tag}"))
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

async fn seed_region(owner_pool: &PgPool, org: Uuid, name: &str) -> RegionId {
    let mut tx = owner_pool.begin().await.unwrap();
    sqlx::query("SET LOCAL row_security = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    let id: Uuid =
        sqlx::query_scalar("INSERT INTO regions (name, org_id) VALUES ($1, $2) RETURNING id")
            .bind(name)
            .bind(org)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    tx.commit().await.unwrap();
    RegionId::from_uuid(id)
}

async fn seed_branch(owner_pool: &PgPool, org: Uuid, region: RegionId, name: &str) -> BranchId {
    let mut tx = owner_pool.begin().await.unwrap();
    sqlx::query("SET LOCAL row_security = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO branches (region_id, name, org_id) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(*region.as_uuid())
    .bind(name)
    .bind(org)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    BranchId::from_uuid(id)
}

async fn seed_active_user(owner_pool: &PgPool, org: Uuid, branch: Option<BranchId>) -> UserId {
    let mut tx = owner_pool.begin().await.unwrap();
    sqlx::query("SET LOCAL row_security = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (display_name, roles, org_id, is_active) VALUES ($1, $2, $3, true) RETURNING id",
    )
    .bind(format!("User {}", Uuid::new_v4()))
    .bind(vec!["MECHANIC".to_string()])
    .bind(org)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    if let Some(branch) = branch {
        sqlx::query("INSERT INTO user_branches (user_id, branch_id, org_id) VALUES ($1, $2, $3)")
            .bind(user_id)
            .bind(*branch.as_uuid())
            .bind(org)
            .execute(&mut *tx)
            .await
            .unwrap();
    }
    tx.commit().await.unwrap();
    UserId::from_uuid(user_id)
}

/// Deactivate an EMPTY region as the runtime role (the referential guard
/// refuses a populated one), so the region is now a deactivated parent target.
async fn deactivate_empty_region(store: &PgOrgStore, org: OrgId, actor: UserId, region: RegionId) {
    CURRENT_ORG
        .scope(
            org,
            store.deactivate_region(DeactivateRegionCommand {
                actor,
                region_id: region,
                trace: TraceContext::generate(),
                occurred_at: OffsetDateTime::now_utc(),
            }),
        )
        .await
        .expect("an empty region must deactivate");
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn create_branch_under_deactivated_region_is_conflict(owner_pool: PgPool) {
    let rt_pool = runtime_role_pool(&owner_pool).await;
    let org = OrgId::knl();
    let org_uuid = *org.as_uuid();
    seed_org(&owner_pool, org_uuid, "A").await;
    let region = seed_region(&owner_pool, org_uuid, "권역-빈것").await;
    let actor = seed_active_user(&owner_pool, org_uuid, None).await;

    let store = PgOrgStore::new(rt_pool.clone());
    deactivate_empty_region(&store, org, actor, region).await;

    let result = CURRENT_ORG
        .scope(
            org,
            store.create_branch(CreateBranchCommand {
                actor,
                region_id: region,
                name: "숨겨질지점".to_owned(),
                trace: TraceContext::generate(),
                occurred_at: OffsetDateTime::now_utc(),
            }),
        )
        .await;
    match result {
        Err(PgOrgError::Domain(err)) => assert_eq!(
            err.kind,
            ErrorKind::Conflict,
            "creating a branch under a deactivated region must be refused (409)"
        ),
        other => panic!("expected region-active conflict, got {other:?}"),
    }
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn reparenting_branch_under_deactivated_region_is_conflict(owner_pool: PgPool) {
    let rt_pool = runtime_role_pool(&owner_pool).await;
    let org = OrgId::knl();
    let org_uuid = *org.as_uuid();
    seed_org(&owner_pool, org_uuid, "A").await;
    let live_region = seed_region(&owner_pool, org_uuid, "권역-활성").await;
    let dead_region = seed_region(&owner_pool, org_uuid, "권역-비활성").await;
    let branch = seed_branch(&owner_pool, org_uuid, live_region, "강남지점").await;
    let actor = seed_active_user(&owner_pool, org_uuid, None).await;

    let store = PgOrgStore::new(rt_pool.clone());
    deactivate_empty_region(&store, org, actor, dead_region).await;

    let result = CURRENT_ORG
        .scope(
            org,
            store.update_branch(UpdateBranchCommand {
                actor,
                branch_scope: BranchScope::All,
                branch_id: branch,
                region_id: Some(dead_region),
                name: None,
                trace: TraceContext::generate(),
                occurred_at: OffsetDateTime::now_utc(),
            }),
        )
        .await;
    match result {
        Err(PgOrgError::Domain(err)) => assert_eq!(
            err.kind,
            ErrorKind::Conflict,
            "re-parenting a branch under a deactivated region must be refused (409)"
        ),
        other => panic!("expected region-active conflict, got {other:?}"),
    }
}
