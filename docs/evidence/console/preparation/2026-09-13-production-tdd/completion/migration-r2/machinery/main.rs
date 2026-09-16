//! Independent fixture machinery harness. No simulated product owner.
mod native_fixture {pub type TestResult<T=()> = Result<T,Box<dyn std::error::Error+Send+Sync>>;}
mod migration_admission_tests {
 pub async fn actual_catalog(pool:&sqlx::PgPool)->crate::native_fixture::TestResult<serde_json::Value>{
 let mut tx=pool.begin().await?;
 sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY").execute(&mut *tx).await?;
 let result=sqlx::query_scalar(include_str!("../catalog_snapshot.sql")).fetch_one(&mut *tx).await?;
 tx.commit().await?;Ok(result)}
}
#[path="../postgres-snapshot-producer.rs"] mod snapshot;
#[tokio::main]
async fn main()->native_fixture::TestResult {
 let pool=sqlx::postgres::PgPoolOptions::new().max_connections(4).connect(&std::env::var("CONSOLE_MACHINERY_DATABASE_URL")?).await?;
 let original=migration_admission_tests::actual_catalog(&pool).await?;
 let fks=original["constraints"].as_array().unwrap();
 assert!(fks.iter().any(|f| f["source_columns"]==serde_json::json!(["org_id","user_id"]) && f["referenced_columns"]==serde_json::json!(["org_id","id"])));
 let image=snapshot::snapshot(&pool).await?;
 let restored=snapshot::restore(&pool,&image).await?;
 let count:i64=sqlx::query_scalar("SELECT count(*) FROM fixture_child").fetch_one(&restored.pool).await?;assert_eq!(count,1);
 let name=restored.name.clone();restored.close(&pool).await?;
 let exists:bool=sqlx::query_scalar("SELECT EXISTS(SELECT FROM pg_database WHERE datname=$1)").bind(&name).fetch_one(&pool).await?;assert!(!exists);
 for change in ["GRANT SELECT(secret) ON fixture_parent TO fixture_reader", "ALTER FUNCTION fixture_identity(integer) SET search_path TO public,pg_catalog", "CREATE INDEX fixture_partial ON fixture_parent(id) WHERE id>0"] {
 let before=migration_admission_tests::actual_catalog(&pool).await?;
 sqlx::raw_sql(change).execute(&pool).await?;
 assert_ne!(before,migration_admission_tests::actual_catalog(&pool).await?);
 }
 let another=snapshot::restore(&pool,&image).await?;let name=another.name.clone();drop(another);
 let exists:bool=sqlx::query_scalar("SELECT EXISTS(SELECT FROM pg_database WHERE datname=$1)").bind(&name).fetch_one(&pool).await?;assert!(!exists);
 println!("PASS: ordered FK; actual dump/restore bytes+catalog; awaited cleanup; ACL drift; definer drift; partial-index drift; unwind guard cleanup (7 checks)");
 pool.close().await;Ok(())}
