//! S3 actual database contention and hostile runtime DML. No owner implementation.
use crate::{native_fixture::{fixture,TestResult},binder_sequence::{bind_and_seal,explicit_new_intent,ProducerStop},native_owner_producer::complete_native};
use console_ontology_application::action30::*;
use console_payroll_adapter_postgres::action30 as payroll;
use sqlx::PgPool;


async fn wait_for_commands(admin:&PgPool,blocker:i32,commands:[i32;2])->TestResult {
 assert_ne!(commands[0],commands[1]);
 let deadline=tokio::time::Instant::now()+std::time::Duration::from_secs(5);
 loop {
  // Follow actual upstream guard waits too. Two direct run-row waiters are not
  // required if the approved period/owner guard serializes these commands first.
  let reached:Vec<i32>=sqlx::query_scalar("WITH RECURSIVE waits(root,pid,path) AS (SELECT pid,pid,ARRAY[pid] FROM pg_stat_activity WHERE datname=current_database() AND pid=ANY($1) AND state='active' UNION ALL SELECT w.root,b.pid,w.path||b.pid FROM waits w CROSS JOIN LATERAL unnest(pg_blocking_pids(w.pid)) b(pid) WHERE NOT b.pid=ANY(w.path)) SELECT DISTINCT root FROM waits WHERE pid=$2 ORDER BY root")
   .bind(commands.as_slice()).bind(blocker).fetch_all(admin).await?;
  if reached.len()==2{return Ok(());}
  if tokio::time::Instant::now()>=deadline{return Err("both exact native command backends did not reach held run lock".into());}
  tokio::time::sleep(std::time::Duration::from_millis(5)).await;
 }
}
#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn native_calculate_two_distinct_commands_rendezvous_on_actual_run_lock(pool:PgPool)->TestResult {
 let admin=pool.clone();let f=fixture(pool,"native_actual_run_rendezvous").await?;
 let (run,_)=f.closed().await?;
 let expected=payroll::read_run_control(&f.pool,&f.submitter,run.run_id).await?;
 let selection=UntrustedActionTargetSelection::existing_run(run.run_id);
 let first=explicit_new_intent::<payroll::RunCalculate>(&f.pool,&f.submitter,&selection).await?;
 let one=bind_and_seal(&f.pool,&f.submitter,&first,&payroll::RunCalculate{expected:expected.clone()}).await?;
 let next=explicit_new_intent::<payroll::RunCalculate>(&f.pool,&f.submitter,&selection).await?;
 let two=bind_and_seal(&f.pool,&f.submitter,&next,&payroll::RunCalculate{expected}).await?;
 assert_ne!(one.command_id,two.command_id);
 let before=f.snapshot(run.run_id).await?;
 let mut barrier=admin.begin().await?;
 let pid:i32=sqlx::query_scalar("SELECT pg_backend_pid()").fetch_one(&mut *barrier).await?;
 let locked:uuid::Uuid=sqlx::query_scalar("SELECT id FROM public.payroll_draft_runs WHERE org_id=$1 AND id=$2 FOR UPDATE")
  .bind(f.org).bind(run.run_id).fetch_one(&mut *barrier).await?;assert_eq!(locked,run.run_id);
 // Actual same-role dedicated single-connection pools bind each observed PID
 // to its command; no runtime worker can substitute as the second waiter.
 let pa=sqlx::postgres::PgPoolOptions::new().max_connections(1).connect_with((*f.pool.connect_options()).clone()).await?;
 let pb=sqlx::postgres::PgPoolOptions::new().max_connections(1).connect_with((*f.pool.connect_options()).clone()).await?;
 let pida:i32=sqlx::query_scalar("SELECT pg_backend_pid()").fetch_one(&pa).await?;
 let pidb:i32=sqlx::query_scalar("SELECT pg_backend_pid()").fetch_one(&pb).await?;
 let a=payroll::payroll_calculate_run(&pa,&f.submitter,&one);
 let b=payroll::payroll_calculate_run(&pb,&f.submitter,&two);
 tokio::pin!(a);tokio::pin!(b);
 tokio::select! {
  x=&mut a=>{x?;panic!("first native command crossed held actual run lock")},
  x=&mut b=>{x?;panic!("second native command crossed held actual run lock")},
  witness=wait_for_commands(&admin,pid,[pida,pidb])=>{witness?;}
 }
 assert_eq!(f.snapshot(run.run_id).await?,before);
 barrier.rollback().await?;
 let (aa,bb)=tokio::join!(&mut a,&mut b);
 let aa=complete_native(&f.pool,&f.submitter,&f.worker,&one,aa?).await;
 let bb=complete_native(&f.pool,&f.submitter,&f.worker,&two,bb?).await;
 assert_eq!(usize::from(aa.is_ok())+usize::from(bb.is_ok()),1);
 let loser=if let Err(e)=aa{e}else{bb.err().unwrap()};
 assert!(matches!(loser,ProducerStop::Observation(ResultObservation::DefinitiveNoEffect{..})));
 let after=f.snapshot(run.run_id).await?;
 assert_eq!(after["batches"].as_array().unwrap().len(),1);
 assert_eq!(after["money"].as_array().unwrap().len(),1);Ok(())
}
#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn native_runtime_cannot_update_or_delete_actual_immutable_custody(pool:PgPool)->TestResult {
 let f=fixture(pool,"native_hostile_runtime_dml").await?;let (run,_)=f.calculated().await?;
 let before=f.snapshot(run.run_id).await?;
 // Audit: SQL text comes only from this closed literal table/operation roster.
 // Company/run identifiers stay bound. No request or catalog string enters SQL.
 for (table,key) in [("payroll_input_revisions","inputs"),("payroll_input_units","units"),
  ("payroll_input_source_refs","sources"),("payroll_calculation_batches","batches"),
  ("payroll_calculation_units","outcomes"),("payroll_line_calculations","money")] {
  assert!(!before[key].as_array().unwrap().is_empty(),"hostile probe must address actual committed rows");
  for operation in ["UPDATE","DELETE"] {
   let sql=if operation=="UPDATE" {format!("UPDATE public.{table} SET org_id=org_id WHERE org_id=$1 AND run_id=$2")}
    else{format!("DELETE FROM public.{table} WHERE org_id=$1 AND run_id=$2")};
   let mut tx=f.runtime.runtime.begin().await?;
   // Current Company GUC cannot supply a private command credential or DML grant.
   sqlx::query("SELECT set_config('app.current_org',$1,true)").bind(f.org.to_string()).execute(&mut *tx).await?;
   let err=sqlx::query(sqlx::AssertSqlSafe(sql)).bind(f.org).bind(run.run_id).execute(&mut *tx).await.expect_err("runtime mutated immutable native evidence");
   let code=err.as_database_error().and_then(|e|e.code()).ok_or("not a classified database refusal")?;
   assert_eq!(code,"42501","runtime role must lack direct UPDATE/DELETE authority");
   tx.rollback().await?;
  }
 }
 assert_eq!(f.snapshot(run.run_id).await?,before);Ok(())
}
