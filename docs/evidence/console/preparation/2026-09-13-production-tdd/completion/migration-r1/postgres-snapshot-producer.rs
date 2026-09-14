//! Real PostgreSQL18.6 pg_dump/pg_restore in the already pinned test image.
//! Only disposable SQLx databases at the current test server are accepted.
use crate::native_fixture::TestResult;
use sqlx::PgPool;
use std::{path::{Path,PathBuf},process::Stdio};
use tokio::process::Command;
use url::Url;
use uuid::Uuid;
const IMAGE:&str="postgres:18.6@sha256:4ef4dbc939d61acea57712655ddb4b4ab27419c913f94cca0cd57cb3ea3c2280";
pub struct Snapshot { pub directory:tempfile::TempDir,pub archive:PathBuf,pub catalog:serde_json::Value }
pub struct Restored { pub name:String,pub pool:PgPool,pub connection:Url,cleanup:DatabaseGuard }
// Best-effort unwind cleanup supplements mandatory awaited cleanup on the normal
// path. All names are generated locally; no user-selected database/container is removed.
struct ContainerGuard(String);
fn bounded_cleanup(command:&mut std::process::Command)->bool {
    let Ok(mut child)=command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn() else{return false};
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
    loop { match child.try_wait(){Ok(Some(status))=>return status.success(),Ok(None)=>{},Err(_)=>return false}
        if std::time::Instant::now()>=deadline{let _=child.kill();let _=child.wait();return false;}
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}
impl Drop for ContainerGuard { fn drop(&mut self){
    let _=bounded_cleanup(std::process::Command::new("docker").args(["rm","-f",&self.0]));
}}
struct DatabaseGuard {name:String,admin:Url,armed:bool}
impl Drop for DatabaseGuard {fn drop(&mut self){
    if !self.armed{return;}
    assert!(self.name.starts_with("_sqlx_test_restore_")&&self.name.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'));
    let container=ContainerGuard(format!("console-migration-cleanup-{}",Uuid::new_v4().simple()));
    let mut command=std::process::Command::new("docker");
    command.args(["run","--rm","--name",&container.0,"--network=host"])
        .args(["--env","PGHOST","--env","PGPORT","--env","PGUSER","--env","PGPASSWORD","--env","PGDATABASE"])
        .args([IMAGE,"psql","-X","--no-password","-v","ON_ERROR_STOP=1","-c",&format!("DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)",self.name)])
        .env("PGHOST","127.0.0.1").env("PGPORT",self.admin.port().unwrap_or(5432).to_string())
        .env("PGUSER",self.admin.username()).env("PGPASSWORD",self.admin.password().unwrap())
        .env("PGDATABASE",self.admin.path().trim_start_matches('/'));
    if !bounded_cleanup(&mut command){eprintln!("FAIL: owned migration fixture database cleanup failed");}
}}

fn owner_url(pool:&PgPool)->TestResult<Url> {
    let options=pool.connect_options();let db=options.get_database().ok_or("actual test database missing")?;
    let suffix=db.strip_prefix("_sqlx_test_").ok_or("refuse non-SQLx database")?;
    assert_eq!(suffix.len(),52);assert!(suffix.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'));
    let mut url=Url::parse(&std::env::var("CONSOLE_APALIS_OWNER_DATABASE_URL")?)?;
    assert_eq!(url.username(),"console_app");assert!(url.password().is_some_and(|p|!p.is_empty()));
    assert_eq!(url.host_str(),Some(options.get_host()));assert_eq!(url.port().unwrap_or(5432),options.get_port());
    assert!(matches!(url.host_str(),Some("127.0.0.1"|"localhost")));
    assert!(url.query().is_none()&&url.fragment().is_none());url.set_path(db);Ok(url)
}

async fn pg_tool(url:&Url,tool:&str,input:Option<&Path>,output:Option<&Path>)->TestResult {
    assert!(matches!(tool,"pg_dump"|"pg_restore"));
    let container=ContainerGuard(format!("console-migration-snapshot-{}",Uuid::new_v4().simple()));
    let mut command=Command::new("docker");
    command.arg("run").arg("--rm").args(["--name",&container.0]).arg("--network=host").arg("-i")
        .args(["--env","PGHOST","--env","PGPORT","--env","PGUSER","--env","PGPASSWORD","--env","PGDATABASE"])
        .arg(IMAGE).arg(tool);
    command.env("PGHOST","127.0.0.1").env("PGPORT",url.port().unwrap_or(5432).to_string())
        .env("PGUSER",url.username()).env("PGPASSWORD",url.password().ok_or("owner password absent")?)
        .env("PGDATABASE",url.path().trim_start_matches('/'));
    if tool=="pg_dump" {command.args(["--format=custom","--no-password"]);}
    else {command.args(["--exit-on-error","--no-password","--dbname",url.path().trim_start_matches('/')]);}
    if let Some(path)=input {command.stdin(Stdio::from(std::fs::File::open(path)?));}else{command.stdin(Stdio::null());}
    if let Some(path)=output {
        use std::os::unix::fs::OpenOptionsExt;
        command.stdout(Stdio::from(std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path)?));
    } else {command.stdout(Stdio::null());}
    // Secrets never enter command arguments or logs. Nonzero exit is an explicit
    // fixture failure; stderr from restore can contain row data, so do not print.
    command.stderr(Stdio::null()).kill_on_drop(true);
    let mut child=command.spawn()?;
    let status=tokio::time::timeout(std::time::Duration::from_secs(90),child.wait()).await??;
    assert!(status.success(),"pinned PostgreSQL snapshot tool failed; fixture prerequisite, not admission refusal");
    Ok(())
}

pub async fn snapshot(pool:&PgPool)->TestResult<Snapshot> {
    let directory=tempfile::tempdir()?;let archive=directory.path().join("database.dump");
    let url=owner_url(pool)?;
    let catalog=crate::migration_admission_tests::actual_catalog(pool).await?;
    pg_tool(&url,"pg_dump",None,Some(&archive)).await?;
    assert!(std::fs::metadata(&archive)?.len()>0);
    Ok(Snapshot{directory,archive,catalog})
}

pub async fn restore(admin:&PgPool,image:&Snapshot)->TestResult<Restored> {
    let name=format!("_sqlx_test_restore_{}_{}",Uuid::new_v4().simple(),&Uuid::new_v4().simple().to_string()[..11]);
    let suffix=name.strip_prefix("_sqlx_test_").unwrap();assert_eq!(suffix.len(),52);
    let admin_url=owner_url(admin)?;
    // Arm before CREATE so a lost acknowledgement still leads to scoped cleanup.
    let cleanup=DatabaseGuard{name:name.clone(),admin:admin_url.clone(),armed:true};
    // Generated, validated identifier; only this newly allocated database is owned.
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!("CREATE DATABASE \"{name}\" OWNER console_app TEMPLATE template0"))).execute(admin).await?;
    let mut url=admin_url;url.set_path(&name);
    pg_tool(&url,"pg_restore",Some(&image.archive),None).await?;
    let pool=sqlx::postgres::PgPoolOptions::new().max_connections(4).connect(url.as_str()).await?;
    let restored=crate::migration_admission_tests::actual_catalog(&pool).await?;
    assert_eq!(restored,image.catalog,"restored catalog/ACL/role witness must match before stale-timeline assertion");
    Ok(Restored{name,pool,connection:url,cleanup})
}

impl Restored {
    pub async fn close(mut self,admin:&PgPool)->TestResult {
        self.pool.close().await;
        let suffix=self.name.strip_prefix("_sqlx_test_restore_").ok_or("refuse non-owned database cleanup")?;
        assert!(suffix.bytes().all(|b|b.is_ascii_hexdigit()||b==b'_'));
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!("DROP DATABASE \"{}\" WITH (FORCE)",self.name))).execute(admin).await?;self.cleanup.armed=false;Ok(())
    }
}
