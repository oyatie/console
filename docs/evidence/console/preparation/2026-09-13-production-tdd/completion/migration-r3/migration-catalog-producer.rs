use crate::native_fixture::TestResult;
use sqlx::PgPool;
use serde_json::Value;

pub async fn actual_catalog(pool:&PgPool)->TestResult<Value> {
    let mut tx=pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY").execute(&mut *tx).await?;
    let catalog:Value=sqlx::query_scalar(include_str!("catalog_snapshot.sql")).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    assert!(!catalog["relations"].as_array().unwrap().is_empty());
    assert!(!catalog["columns"].as_array().unwrap().is_empty());
    for category in ["relations","columns","constraints","functions","policies","triggers","roles","memberships","effective_column_grants","default_grants","indexes","schemas"] {
        assert!(catalog[category].is_array(),"missing catalog category {category}");
    }
    Ok(catalog)
}

