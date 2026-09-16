//! Real reviewed loopback PostgreSQL wire relay; cuts after backend COMMIT success
//! and before delivery. Bootstrap/binders/observers never use the cut connection.
use sqlx::{PgPool,postgres::{PgPoolOptions,PgSslMode}};
use crate::native_fixture::TestResult;
pub struct CommitAckRelay {pub pool:PgPool, child:tokio::process::Child, directory:tempfile::TempDir}
impl CommitAckRelay {
 pub async fn start(direct:&PgPool)->TestResult<Self> {
  let options=direct.connect_options();
  assert!(matches!(options.get_host(),"127.0.0.1"|"localhost"),"synthetic loopback only");
  let directory=tempfile::Builder::new().prefix("source-commit-cut-").tempdir()?;
  let script=directory.path().join("commit_relay.py");
  std::fs::write(&script,include_bytes!("dependencies/commit_relay.py"))?;
  let port_file=directory.path().join("port");let events=directory.path().join("events.jsonl");
  let mut child=tokio::process::Command::new("python3").arg(script)
   .arg("--upstream-port").arg(options.get_port().to_string())
   .args(["--cut","after-commit-response"]).arg("--events").arg(&events).arg("--port-file").arg(&port_file)
   .kill_on_drop(true).spawn()?;
  tokio::time::timeout(std::time::Duration::from_secs(10),async {
   loop {if port_file.exists(){break Ok::<_,Box<dyn std::error::Error+Send+Sync>>(());}
    if child.try_wait()?.is_some(){break Err("relay exited before port allocation".into());}
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
   }
  }).await??;
  let port=std::fs::read_to_string(port_file)?.trim().parse::<u16>()?;
  let pool=PgPoolOptions::new().max_connections(1).connect_with((*options).clone().host("127.0.0.1").port(port).ssl_mode(PgSslMode::Disable)).await?;
  Ok(Self{pool,child,directory})
 }
 pub fn assert_cut(&self)->TestResult {
  let events=std::fs::read_to_string(self.directory.path().join("events.jsonl"))?;
  let rows=events.lines().map(serde_json::from_str::<serde_json::Value>).collect::<Result<Vec<_>,_>>()?;
  let committed=rows.iter().filter(|v|v["event"]=="commit_backend_success_witness").collect::<Vec<_>>();
  let cut=rows.iter().filter(|v|v["event"]=="cut_before_commit_response_delivery").collect::<Vec<_>>();
  assert_eq!(committed.len(),1);assert_eq!(cut.len(),1);
  assert_eq!(committed[0]["connection"],cut[0]["connection"]);
  assert!(!rows.iter().any(|v|v["event"]=="relay_fixture_error"));Ok(())
 }
 pub async fn stop(mut self)->TestResult {self.pool.close().await;self.child.kill().await?;let _=self.child.wait().await?;Ok(())}
}
