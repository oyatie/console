//! Product-independent fixture machinery; real-owner source imports this module.
use serde_json::{json,Value};use sha2::{Digest,Sha256};
use std::{path::Path,io::Write,os::unix::fs::{OpenOptionsExt,PermissionsExt}};
pub type TestResult<T=()> = Result<T,Box<dyn std::error::Error+Send+Sync>>;
pub fn sha(bytes:&[u8])->String{hex::encode(Sha256::digest(bytes))}
pub fn private_write(path:&Path,bytes:&[u8])->TestResult{
 let mut f=std::fs::OpenOptions::new().create_new(true).write(true).mode(0o600).open(path)?;
 f.write_all(bytes)?;f.sync_all()?;assert_eq!(f.metadata()?.permissions().mode()&0o777,0o600);Ok(())
}
pub fn cookies(origin:&url::Url,headers:&[(String,i64)],actual:&std::collections::BTreeMap<String,String>)->TestResult<Vec<Value>>{
 let mut parsed=std::collections::BTreeMap::new();
 for (header,received_at) in headers{
  let cookie=cookie::Cookie::parse(header.clone())?;
  if cookie.value().is_empty(){parsed.remove(cookie.name());continue;}
  assert!(cookie.name().starts_with("__Host-"));assert_eq!(cookie.path(),Some("/"));
  assert_eq!(cookie.domain(),None);assert_eq!(cookie.secure(),Some(true));
  let same_site=match cookie.same_site().ok_or("issuer cookie has no SameSite")?{
   cookie::SameSite::Strict=>"Strict",cookie::SameSite::Lax=>"Lax",cookie::SameSite::None=>"None"};
  let mut value=json!({"name":cookie.name(),"value":cookie.value(),"url":origin.origin().ascii_serialization(),
   "httpOnly":cookie.http_only().unwrap_or(false),"secure":true,"sameSite":same_site});
  if let Some(age)=cookie.max_age(){
   if age.whole_seconds()<=0{parsed.remove(cookie.name());continue;}
   value["expires"]=json!(received_at.checked_add(age.whole_seconds()).ok_or("cookie expiry overflow")?);
  }else if let Some(expiry)=cookie.expires_datetime(){
   if expiry.unix_timestamp()<=*received_at{parsed.remove(cookie.name());continue;}
   value["expires"]=json!(expiry.unix_timestamp());
  }
  parsed.insert(cookie.name().to_string(),value);
 }
 for key in ["__Host-console_account_session","__Host-console_account_refresh"]{
  assert_eq!(parsed.get(key).and_then(|x|x["value"].as_str()),actual.get(key).map(String::as_str));
  assert!(parsed.contains_key(key));
 }
 Ok(parsed.into_values().collect())
}
pub fn discovered(v:&Value)->usize{
 let own=v.get("specs").and_then(Value::as_array).map_or(0,|specs|specs.iter().map(|s|s["tests"].as_array().map_or(0,Vec::len)).sum::<usize>());
 own+v.get("suites").and_then(Value::as_array).map_or(0,|suites|suites.iter().map(discovered).sum::<usize>())
}

pub fn exact_leaf(v:&Value,title:&str,project:&str)->bool{
  let own=v.get("specs").and_then(Value::as_array).is_some_and(|specs|specs.iter().any(|s|s["title"]==title&&s["tests"].as_array().is_some_and(|ts|ts.iter().any(|t|t["projectName"]==project))));
  own||v.get("suites").and_then(Value::as_array).is_some_and(|ss|ss.iter().any(|s|exact_leaf(s,title,project)))
 }

/// Keep the hosting Tokio runtime responsive while a browser child uses it.
pub async fn run_child(command: &mut tokio::process::Command, deadline: std::time::Duration) -> TestResult<std::process::Output> {
 use tokio::io::AsyncReadExt;
 async fn bounded(reader: impl tokio::io::AsyncRead+Unpin)->TestResult<Vec<u8>> {
  const LIMIT:u64=8*1024*1024;
  let mut data=Vec::new();reader.take(LIMIT+1).read_to_end(&mut data).await?;
  if data.len() as u64>LIMIT{return Err("browser child output exceeds8MiB per stream".into());}Ok(data)
 }
 command.kill_on_drop(true).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
 let mut child=command.spawn()?;
 let stdout=child.stdout.take().ok_or("child stdout pipe missing")?;
 let stderr=child.stderr.take().ok_or("child stderr pipe missing")?;
 let result=tokio::time::timeout(deadline,async {
  let (stdout,stderr,status)=tokio::try_join!(bounded(stdout),bounded(stderr),async {Ok::<_,Box<dyn std::error::Error+Send+Sync>>(child.wait().await?)} )?;
  Ok::<_,Box<dyn std::error::Error+Send+Sync>>(std::process::Output{stdout,stderr,status})
 }).await;
 match result {Ok(Ok(output))=>Ok(output),other=>{
  let _=child.kill().await;let _=child.wait().await;
  match other{Ok(Err(error))=>Err(error),Err(error)=>Err(error.into()),_=>unreachable!()}
 }}
}
