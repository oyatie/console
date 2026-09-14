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

/// Validate observed client ACKs against complete ordinary owner history. Pure
/// checker tests are synthetic machinery controls, never application receipts.
pub fn validate_history_delta(before:&Value,after:&Value,observation:&Value)->TestResult {
 fn rows<'a>(v:&'a Value,key:&str)->TestResult<&'a Vec<Value>>{v[key].as_array().ok_or_else(||format!("missing {key} array").into())}
 fn unique(rows:&[Value],key:&str)->TestResult{let mut seen=std::collections::BTreeSet::new();for row in rows{let id=row[key].as_str().ok_or("missing identity")?;if !seen.insert(id){return Err("duplicate history identity".into());}}Ok(())}
 let old=rows(before,"receipts")?;let new=rows(after,"receipts")?;
 for key in ["receipt_id","command_id"]{unique(old,key)?;unique(new,key)?;}
 let old_revisions=rows(before,"revisions")?;let revisions=rows(after,"revisions")?;unique(old_revisions,"revision_id")?;unique(revisions,"revision_id")?;
 if new.len()!=old.len()+1||revisions.len()!=old_revisions.len()+1{return Err("expected exactly one additional save receipt/revision".into());}
 for row in old{if !new.contains(row){return Err("prior receipt omitted or rewritten".into());}}
 for row in old_revisions{if !revisions.contains(row){return Err("prior revision omitted or rewritten".into());}}
 let _=rows(before,"domain_effects")?;let _=rows(after,"domain_effects")?;
 if after["domain_effects"]!=before["domain_effects"]{return Err("draft save changed business effects".into());}
 let receipt=new.iter().find(|x|!old.contains(x)).ok_or("new receipt missing")?;
 let added_revision=revisions.iter().find(|r|!old_revisions.contains(r)).ok_or("new revision missing")?;
 if receipt["revision_id"]!=after["head_revision_id"]||added_revision["revision_id"]!=receipt["revision_id"]{return Err("new receipt is not retained head revision".into());}
 let acks=rows(observation,"acknowledgements")?;let manual=rows(observation,"manual_commands")?;
 if acks.len()+manual.len()!=1{return Err("exactly one client save observation required".into());}
 for ack in acks{for key in ["command_id","receipt_id","revision_id"]{if ack[key]!=receipt[key]{return Err("client ACK does not match persisted owner receipt".into());}}}
 for command in manual{if *command!=receipt["command_id"]{return Err("manual command does not match ordinary receipt".into());}}
 if let Some(refused)=observation.get("refused_command"){if new.iter().any(|r|r["command_id"]==*refused){return Err("stale conflicting command received a save receipt".into());}}
 Ok(())
}
