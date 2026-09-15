#[path="../../browser-runtime-helpers.rs"]
mod helpers;
#[cfg(test)]mod tests{
 use super::helpers::*;use serde_json::json;use std::collections::BTreeMap;
 fn input()->(url::Url,Vec<(String,i64)>,BTreeMap<String,String>){
  let names=["__Host-console_account_session","__Host-console_account_refresh"];
  (url::Url::parse("https://127.0.0.1:8443").unwrap(),names.iter().map(|n|(format!("{n}=synthetic-{n}; Secure; HttpOnly; Path=/; SameSite=Lax; Max-Age=60; Expires=Wed, 21 Oct 2015 07:28:00 GMT"),1_700_000_000)).collect(),
   names.iter().map(|n|(n.to_string(),format!("synthetic-{n}"))).collect())
 }
 #[test]fn cookie_preserves_actual_value_flags_and_max_age_precedence(){let(o,h,m)=input();let c=cookies(&o,&h,&m).unwrap();assert_eq!(c.len(),2);for v in c{assert_eq!(v["expires"],1_700_000_060);assert_eq!(v["httpOnly"],true);assert_eq!(v["secure"],true);assert_eq!(v["sameSite"],"Lax");}}
 #[test]fn cookie_latest_set_cookie_wins_and_deleted_nonce_stays_absent(){let(o,mut h,mut m)=input();h.push(("__Host-nonce=old; Path=/; Secure; SameSite=Strict".into(),1));h.push(("__Host-nonce=; Path=/; Secure; Max-Age=0; SameSite=Strict".into(),2));h.push(("__Host-console_account_session=new; Path=/; Secure; HttpOnly; SameSite=Strict".into(),2));m.insert("__Host-console_account_session".into(),"new".into());let c=cookies(&o,&h,&m).unwrap();assert_eq!(c.len(),2);assert_eq!(c.iter().find(|v|v["name"]=="__Host-console_account_session").unwrap()["value"],"new");}
 #[test]fn cookie_host_domain_violation_and_value_mismatch_refuse(){let(o,h,m)=input();let mut bad=h.clone();bad[0].0.push_str("; Domain=localhost");assert!(std::panic::catch_unwind(||cookies(&o,&bad,&m)).is_err());let mut wrong=m.clone();wrong.insert("__Host-console_account_session".into(),"different".into());assert!(std::panic::catch_unwind(||cookies(&o,&h,&wrong)).is_err());}
 #[test]fn cookie_expires_only_past_is_deleted_but_positive_max_age_wins(){let(o,mut h,m)=input();h.push(("__Host-nonce=old; Path=/; Secure; SameSite=Strict".into(),1));h.push(("__Host-nonce=expired; Path=/; Secure; SameSite=Strict; Expires=Wed, 21 Oct 2015 07:28:00 GMT".into(),1_700_000_000));let c=cookies(&o,&h,&m).unwrap();assert_eq!(c.len(),2);assert!(!c.iter().any(|v|v["name"]=="__Host-nonce"));for v in c{assert_eq!(v["expires"],1_700_000_060);}}
 #[test]fn report_counts_leaf_project_and_refuses_wrong_selection(){let v=json!({"suites":[{"specs":[{"title":"title.(1)","tests":[{"projectName":"no-js"}]}]}]});assert_eq!(discovered(&v),1);assert!(exact_leaf(&v,"title.(1)","no-js"));assert!(!exact_leaf(&v,"titleX1","no-js"));assert!(!exact_leaf(&v,"title.(1)","other"));assert_eq!(discovered(&json!({"suites":[]})),0);let two=json!({"suites":[v.clone(),v]});assert_eq!(discovered(&two),2);}
 #[test]fn literal_leaf_regex_escapes_metacharacters_and_respects_boundary(){let title="title.(1)";let re=regex::Regex::new(&format!("(?:^| ){}$",regex::escape(title))).unwrap();assert!(re.is_match(" no-js acceptance.spec.cjs title.(1)"));assert!(!re.is_match(" no-js acceptance.spec.cjs titleX1"));assert!(!re.is_match(" no-js acceptance.spec.cjs nottitle.(1)"));}
 #[test]fn evidence_file_is_private_and_cannot_overwrite(){use std::os::unix::fs::PermissionsExt;let d=tempfile::tempdir().unwrap();let p=d.path().join("proof.json");private_write(&p,b"synthetic").unwrap();assert_eq!(p.metadata().unwrap().permissions().mode()&0o777,0o600);assert!(private_write(&p,b"overwrite").is_err());assert_eq!(std::fs::read(p).unwrap(),b"synthetic");assert_eq!(sha(b"abc"),"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");}
 #[test]fn actual_approved_actor_csv_parser_has_exact_46_rows(){let mut reader=csv::Reader::from_reader(include_bytes!("actor-migration.csv").as_slice());let expected:Vec<_>=reader.records().collect::<Result<_,_>>().unwrap();assert_eq!(expected.len(),46);for entry in expected{assert_eq!(entry.len(),5);assert!(!entry[0].is_empty()&&!entry[1].is_empty());assert!(matches!(&entry[2],"ACCOUNT"|"COMPANY_ACTOR"));}}
}

#[cfg(test)]mod async_tests {
 use super::helpers::run_child;
 #[tokio::test(flavor="current_thread")] async fn excessive_child_output_is_rejected_and_reaped(){let mut command=tokio::process::Command::new("/usr/bin/yes");assert!(run_child(&mut command,std::time::Duration::from_secs(5)).await.unwrap_err().to_string().contains("output exceeds"));}
 #[tokio::test(flavor="current_thread")] async fn child_wait_allows_host_runtime_progress_and_timeout_is_failure() {
  let progress=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
  let p=progress.clone();
  let host=tokio::spawn(async move { tokio::time::sleep(std::time::Duration::from_millis(5)).await; p.store(true,std::sync::atomic::Ordering::SeqCst); });
  let mut child=tokio::process::Command::new("/bin/sleep");child.arg("0.05");
  assert!(run_child(&mut child,std::time::Duration::from_secs(5)).await.unwrap().status.success());
  assert!(progress.load(std::sync::atomic::Ordering::SeqCst));host.await.unwrap();
  let mut slow=tokio::process::Command::new("/bin/sleep");slow.arg("5");
  assert!(run_child(&mut slow,std::time::Duration::from_millis(10)).await.is_err());
 }
}
