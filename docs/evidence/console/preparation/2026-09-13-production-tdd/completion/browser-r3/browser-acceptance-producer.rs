//! Real fixture supervisor. Exact invocation uses --test-threads=1 as well as
//! the in-process permit. Counts11Rust fixture cases and11browser cases separately.
use crate::{browser_case_producer::{self,Kind,Prepared},native_fixture::TestResult};
use sqlx::PgPool;use serde_json::{json,Value};use sha2::{Digest,Sha256};
use std::{path::Path,sync::Mutex,os::unix::fs::PermissionsExt};
use crate::browser_runtime_helpers::{sha,private_write,cookies,discovered,exact_leaf,run_child,validate_history_delta};
static SINGLE_BROWSER_FIXTURE:Mutex<()>=Mutex::new(());
async fn run_case(pool:PgPool,project:&str,title:&str,kind:Kind)->TestResult{
 let _permit=SINGLE_BROWSER_FIXTURE.try_lock().map_err(|_|"browser fixtures must run with --test-threads=1")?;
 let mut prepared=match kind{Kind::Draft=>browser_case_producer::draft(pool).await?,Kind::Scoped=>browser_case_producer::review(pool,false).await?,
  Kind::ReadOnly=>browser_case_producer::review(pool,true).await?,Kind::Historical=>browser_case_producer::historical(pool).await?};
 let d=&prepared.deployment;let f=&d.companies[0];
 let candidate=std::env::var("CONSOLE_BROWSER_CANDIDATE_SHA")?;
 assert_eq!(candidate.len(),40);assert!(candidate.bytes().all(|x|x.is_ascii_digit()||(b'a'..=b'f').contains(&x)));
 // Actual in-process running app identity is compiled into that app component;
 // startup already verified its signed release/code/schema/codec custody.
 let build=d.runtime.services.running_build_identity();assert_eq!(build.source_sha,candidate);
 let selection=console_identity_adapter_postgres::account13::read_current_company_selection(&d.pool,&d.operator,f.org).await?;
 let token=prepared.browser.cookies.get("__Host-console_account_session").ok_or("actual HTTP session cookie missing")?;
 let actor=console_platform_request_context::account::resolve_company_context(&d.pool,&d.verifier,token,&selection,&d.serving).await?;
 let baseline_history=match &prepared.target{console_app::ui30::UiTarget::Draft{draft_id}=>Some(draft_history(&f.pool,&actor,*draft_id).await?),_=>None};
 let route=console_app::ui30::read_registered_target_route(&f.pool,&actor,&prepared.target).await?;
 assert!(route.path.starts_with("/_ui/"));assert!(!route.path.contains('\\'));
 let target=d.runtime.origin.join(&route.path)?;assert_eq!(target.origin(),d.runtime.origin.origin());
 assert!(matches!(target.host_str(),Some("127.0.0.1")|Some("localhost")|Some("::1")));
 prepared.data["path"]=json!(route.path);prepared.data["cookies"]=json!(cookies(&d.runtime.origin,&prepared.browser.received_set_cookie,&prepared.browser.cookies)?);
 let temp=tempfile::Builder::new().prefix("console-browser-owner-").tempdir()?;
 let directory=temp.keep(); // Preserve0600evidence on both failure and success.
 assert_eq!(directory.metadata()?.permissions().mode()&0o077,0);
 let fixture=directory.join("fixture.json");let receipt=directory.join("owner-receipts.json");
 let key=format!("{project}/{title}");
 let data=json!({"kind":"SYNTHETIC_OWNER_PRODUCED_BROWSER_FIXTURE","candidate_sha":candidate,"origin":d.runtime.origin,"cases":{key:prepared.data}});
 let bytes=serde_json::to_vec(&data)?;private_write(&fixture,&bytes)?;
 private_write(&receipt,&serde_json::to_vec(&json!({"fixture_sha256":sha(&bytes),"running_build":build,"registered_route":route,
  "owner_provenance":prepared.provenance,"org_id":f.org,"project":project,"title":title}))?)?;
 let node=std::env::var("CONSOLE_BROWSER_NODE")?;let cli=std::env::var("CONSOLE_BROWSER_PLAYWRIGHT_CLI")?;
 let config=std::env::var("CONSOLE_BROWSER_PLAYWRIGHT_CONFIG")?;
 for path in [&node,&cli,&config]{assert!(Path::new(path).is_absolute()&&Path::new(path).is_file());}
 // A literal title anchored on both sides cannot silently select zero/many tests.
 // Pinned Playwright1.63 grep includes project/file prefixes. Escape literal
 // leaf title, anchor suffix, then independently require exact report leaf title.
 let grep=format!("(?:^| ){}$",regex::escape(title));
 let modules=std::env::var("CONSOLE_BROWSER_NODE_MODULES")?;
 assert!(Path::new(&modules).is_absolute()&&Path::new(&modules).is_dir());
 let invoke=async |list:bool|->TestResult<std::process::Output>{
  let mut c=tokio::process::Command::new(&node);c.arg(&cli).arg("test").arg("--config").arg(&config).arg("--project").arg(project)
   .arg("--grep").arg(&grep).arg("--reporter=json").arg("--workers=1")
   .env("NODE_PATH",&modules).env("CONSOLE_BROWSER_FIXTURE",&fixture).env("CONSOLE_BROWSER_FIXTURE_SHA256",sha(&bytes)).env("CONSOLE_BROWSER_CANDIDATE_SHA",&candidate)
   .env("CONSOLE_BROWSER_OUTPUT",directory.join("results")).env("CONSOLE_BROWSER_OBSERVATION",directory.join("browser-observation.json"));
  if list{c.arg("--list");}
  run_child(&mut c,std::time::Duration::from_secs(180)).await
 };
 private_write(&directory.join("invocations.json"),&serde_json::to_vec(&json!({"node":node,"cli":cli,"config":config,"project":project,"grep":grep,
  "NODE_PATH":modules,"fixture_sha256":sha(&bytes),"candidate_sha":candidate,"workers":1,"discovery_flag":"--list","expected_discovery_count":1,"expected_execution_count":1}))?)?;
 let list=invoke(true).await?;private_write(&directory.join("discovery.json"),&list.stdout)?;private_write(&directory.join("discovery.stderr"),&list.stderr)?;
 assert!(list.status.success(),"Playwright discovery failed; not application RED");
 let discovery:Value=serde_json::from_slice(&list.stdout)?;assert_eq!(discovered(&discovery),1,"exactly one application case must be discovered");
 assert!(exact_leaf(&discovery,title,project));
 let executed=invoke(false).await?;private_write(&directory.join("execution.json"),&executed.stdout)?;private_write(&directory.join("execution.stderr"),&executed.stderr)?;
 let results:Value=serde_json::from_slice(&executed.stdout)?;assert_eq!(discovered(&results),1);assert!(exact_leaf(&results,title,project));
 assert_eq!(results["stats"]["skipped"],0);assert_eq!(results["stats"]["flaky"],0);assert_eq!(results["stats"]["expected"],1);
 assert!(executed.status.success(),"actual browser application assertion failed; inspect private execution receipt");
 let post=match &prepared.target{
  console_app::ui30::UiTarget::Draft{draft_id}=>{
   let current=console_ontology_adapter_postgres::action30::read_acknowledged_registered_input::<console_payroll_adapter_postgres::action30::WageCorrect>(&f.pool,&actor,*draft_id).await?;
   let expected=match title{
    "draft save preserves exact acknowledged input across reload"|"keyboard user can discover edit save and recover acknowledged input"=>1_234_567,
    "stale save preserves acknowledged inputs and offers recovery"|"hydrated stale autosave preserves local input and exposes current receipt recovery"=>1_250_000,
    "hydrated edit autosaves an ordinary acknowledged draft receipt"|"WASM load failure labels autosave unavailable and retains real manual save"|"hydrated offline input remains unacknowledged and reconnect saves exactly once"=>1_234_567,
    _=>1_200_000,
   };assert_eq!(current.input.amount_won,expected);
   let history=console_ontology_adapter_postgres::action30::read_draft_history_manifest(&f.pool,&actor,*draft_id).await?;
   assert!(history.receipt_count>=2);
   if project=="hydrated"||(project=="no-js"&&title=="draft save preserves exact acknowledged input across reload") {
    let path=directory.join("browser-observation.json");assert_eq!(path.metadata()?.permissions().mode()&0o777,0o600);
    let observation:Value=serde_json::from_slice(&std::fs::read(&path)?)?;
    assert_eq!(observation["final_amount"],expected);
    if observation["kind"]=="stale" {
     let refusal:console_ontology_application::action30::ResultObservation=serde_json::from_value(observation["refusal"].clone())?;
     assert!(matches!(refusal,console_ontology_application::action30::ResultObservation::DefinitiveNoEffect{..}),"stale save must be a definitive conflict, not unknown/success");
    }
    let mut normalized=observation.clone();let mut acknowledgements=Vec::new();
    for value in observation["acknowledgements"].as_array().ok_or("observed acknowledgement list missing")? {
     let ack:console_ontology_application::action30::DraftAck=serde_json::from_value(value["ack"].clone())?;
     assert_eq!(ack.draft_id,*draft_id);
     let console_ontology_application::action30::ReceiptRef::Company{command,receipt_id}=ack.receipt else{return Err("Company draft returned Group receipt".into());};
     assert_eq!(value["command_id"],json!(command.command_id));
     acknowledgements.push(json!({"command_id":command.command_id,"receipt_id":receipt_id,"revision_id":ack.revision_id}));
    }
    normalized["acknowledgements"]=json!(acknowledgements);
    let final_history=draft_history(&f.pool,&actor,*draft_id).await?;
    validate_history_delta(baseline_history.as_ref().ok_or("baseline draft history missing")?,&final_history,&normalized)?;
    private_write(&directory.join("hydrated-owner-history.json"),&serde_json::to_vec(&json!({"baseline":baseline_history,"after":final_history,"observed":normalized}))?)?;
   }
   json!({"acknowledged":current,"history_manifest":history})
  },
  _=>{let run=uuid::Uuid::parse_str(prepared.provenance["run_id"].as_str().ok_or("actual run custody missing")?)?;
   let current=f.snapshot(run).await?;assert_eq!(current,prepared.provenance["actual_readback"]);current}
 };
 private_write(&directory.join("post-browser-owner-readback.json"),&serde_json::to_vec(&post)?)?;
 // All Runtime, origin, authenticated sessions and storage remain live until
 // child completion. No completed fixture is handed off after service teardown.
 drop(prepared);Ok(())
}
macro_rules! browser_case{($name:ident,$project:literal,$title:literal,$kind:ident)=>{
 #[sqlx::test(migrations="../crates/platform/db/migrations")]async fn $name(pool:PgPool)->TestResult{run_case(pool,$project,$title,Kind::$kind).await}
};}
browser_case!(browser_draft_save,"no-js","draft save preserves exact acknowledged input across reload",Draft);
browser_case!(browser_invalid_input,"no-js","invalid submission preserves input and exposes linked errors",Draft);
browser_case!(browser_stale_save,"no-js","stale save preserves acknowledged inputs and offers recovery",Draft);
browser_case!(browser_scoped_review,"no-js","scoped review presents permitted evidence without hidden coverage",Scoped);
browser_case!(browser_historical_own,"no-js","historical own publication remains nonpayable with correction entry",Historical);
browser_case!(browser_read_only,"no-js","read-only projection has no state-changing form or concealed private fields",ReadOnly);
browser_case!(browser_keyboard,"no-js","keyboard user can discover edit save and recover acknowledged input",Draft);
browser_case!(browser_narrow,"no-js","narrow composer preserves essential context inputs and actions",Draft);
browser_case!(browser_js_scoped,"js-enabled-ssr","scoped review presents permitted evidence without hidden coverage",Scoped);
browser_case!(browser_js_historical,"js-enabled-ssr","historical own publication remains nonpayable with correction entry",Historical);
browser_case!(browser_js_read_only,"js-enabled-ssr","read-only projection has no state-changing form or concealed private fields",ReadOnly);

async fn draft_history(pool:&PgPool,actor:&console_platform_request_context::account::AuthenticatedCompanyContext,id:uuid::Uuid)->TestResult<Value>{
 use console_ontology_adapter_postgres::action30 as owner;
 let manifest=owner::read_draft_history_manifest(pool,actor,id).await?;
 assert!(manifest.page_budget>0&&manifest.page_budget<=1024);
 let(mut revisions,mut receipts,mut cursor,mut seen)=(Vec::new(),Vec::new(),None,std::collections::BTreeSet::new());
 loop{let page=owner::read_draft_history_page(pool,actor,&manifest,cursor.as_deref()).await?;assert_eq!(page.manifest,manifest);
  revisions.extend(page.revisions);receipts.extend(page.receipts);
  assert!(revisions.len()<=manifest.revision_count&&receipts.len()<=manifest.receipt_count);
  match page.next_cursor{None=>break,Some(next)=>{assert!(seen.insert(next.clone())&&seen.len()<=manifest.page_budget);cursor=Some(next);}}
 }
 assert_eq!(revisions.len(),manifest.revision_count);assert_eq!(receipts.len(),manifest.receipt_count);
 Ok(json!({"revisions":revisions,"receipts":receipts,"head_revision_id":manifest.head_revision_id,"domain_effects":manifest.domain_effects}))
}
browser_case!(browser_hydrated_autosave,"hydrated","hydrated edit autosaves an ordinary acknowledged draft receipt",Draft);
browser_case!(browser_hydrated_stale,"hydrated","hydrated stale autosave preserves local input and exposes current receipt recovery",Draft);
browser_case!(browser_wasm_fallback,"hydrated","WASM load failure labels autosave unavailable and retains real manual save",Draft);
browser_case!(browser_hydrated_reconnect,"hydrated","hydrated offline input remains unacknowledged and reconnect saves exactly once",Draft);
