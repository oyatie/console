//! M0 and dormant-coupling acceptance source. The observation SQL is independent
//! from the actual admission owner; no test inserts enrollment or grants.
use console_app::{account_migration as migration,serving_admission as admission};
use console_ontology_adapter_postgres::action30 as ontology;
use console_ontology_application::action30::*;
use console_payroll_adapter_postgres::action30 as payroll;
use crate::native_fixture::{build_identity_deployment,TestResult};
use serde_json::{json,Value};
use sqlx::PgPool;
use sha2::{Digest,Sha256};

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

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn m0_enrollment_binds_full_actual_catalog_and_all_actor_references(pool:PgPool)->TestResult {
    let identity=build_identity_deployment(pool,"m0_catalog",1).await?;
    let observed=actual_catalog(&identity.readback).await?;
    let installed=admission::load_installed_release(&identity.runtime.migrate_config).await?;
    let actual=admission::capture_enrollment_observations(&identity.runtime.migrate_config).await?;
    // Returned evidence is actual owner observation, not a caller-constructed
    // authority token. The owner's canonical catalog must equal independent SQL.
    assert_eq!(actual.catalog,observed);
    assert_eq!(actual.source_sha,installed.source_sha);
    assert_eq!(actual.migration_checksums,installed.migration_checksums);
    assert_eq!(actual.codecs,installed.codecs);assert_eq!(actual.projections,installed.projections);
    assert_eq!(actual.action_aliases,installed.action_aliases);
    assert_eq!(actual.credential_holders,installed.credential_holders);
    assert_eq!(actual.manifest_sha256,installed.manifest_sha256);
    let reviewed=admission::read_actor_migration_classification(&identity.runtime.migrate_config,&installed).await?;
    // CSV is custody-pinned from approved design30, not a regex inference.
    let mut reader=csv::Reader::from_reader(include_bytes!("actor-migration.csv").as_slice());
    let expected:Vec<_>=reader.records().collect::<Result<_,_>>()?;assert_eq!(expected.len(),46);
    for entry in expected {
        let key=(entry[0].to_owned(),entry[1].to_owned());
        let classification=reviewed.entries.iter().find(|x|(x.table.clone(),x.column.clone())==key).ok_or("approved actor source omitted")?;
        assert_eq!(classification.target.as_str(),&entry[2]);
        if !classification.is_fenced() {
            let (relation,columns,target)=match &entry[2] {
                "ACCOUNT" => ("accounts",json!([&entry[1]]),json!(["id"])),
                "COMPANY_ACTOR" => ("company_actors",json!(["org_id",&entry[1]]),json!(["org_id","account_id"])),
                other => panic!("unreviewed actor target {other}"),
            };
            let actual_fk=observed["constraints"].as_array().unwrap().iter().find(|fk|
                fk["nspname"]=="public" && fk["relname"]==entry[0] && fk["contype"]=="f" &&
                fk["source_columns"]==columns && fk["referenced_schema"]=="public" && fk["referenced_relation"]==relation && fk["referenced_columns"]==target)
                .ok_or("declared mapped actor lacks exact actual ordered FK")?;
            assert!(actual_fk["convalidated"].as_bool().unwrap());
            assert!(matches!(actual_fk["confdeltype"].as_str(),Some("a"|"r")));
            assert!(matches!(actual_fk["confupdtype"].as_str(),Some("a"|"r")));
        } else {
            assert!(actual.fenced_writer_aliases.contains(&classification.writer_alias));
        }
        assert!(observed["columns"].as_array().unwrap().iter().any(|c|c["nspname"]=="public"&&c["relname"]==entry[0]&&c["attname"]==entry[1]));
    }
    // Enumerate discovered extra references from real catalog ordered key tuples.
    // They must have explicit reviewed disposition and actual fenced owner proof;
    // the original46 are a minimum, never an allowlist hiding new callers.
    for fk in observed["constraints"].as_array().unwrap().iter().filter(|c|c["referenced_relation"]=="users") {
        let columns=fk["source_columns"].as_array().ok_or("FK column tuple absent")?;
        let target=fk["referenced_columns"].as_array().ok_or("FK target tuple absent")?;
        assert_eq!(columns.len(),target.len());
        for (source,target) in columns.iter().zip(target) {
            if target!="id"{continue;}
            let classification=reviewed.entries.iter().find(|x|json!(x.table)==fk["relname"]&&json!(x.column)==*source).ok_or("unclassified extra legacy actor/subject FK")?;
            assert!(classification.is_mapped_or_explicitly_fenced());
            if classification.is_fenced(){assert!(actual.fenced_writer_aliases.contains(&classification.writer_alias));}
        }
    }
    assert_eq!(reviewed.entries.len(),reviewed.entries.iter().map(|x|(&x.table,&x.column)).collect::<std::collections::BTreeSet<_>>().len());
    assert_eq!(actual.catalog_sha256,format!("{:x}",Sha256::digest(serde_json::to_vec(&observed)?)));
    Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn m0_actual_column_acl_drift_blocks_enrollment(pool:PgPool)->TestResult {
    let identity=build_identity_deployment(pool,"m0_acl_drift",1).await?;
    let installed=admission::load_installed_release(&identity.runtime.migrate_config).await?;
    let before=actual_catalog(&identity.readback).await?;
    // This deliberately mutates one ACL in a disposable SQLx database. It is
    // fault injection, not fixture permission setup or a production grant.
    let allowed:bool=sqlx::query_scalar("SELECT has_column_privilege('console_rt','public.auth_webauthn_credentials','passkey_json','SELECT')").fetch_one(&identity.readback).await?;
    assert!(!allowed,"mutation must introduce a real previously absent privilege");
    sqlx::query("GRANT SELECT(passkey_json) ON public.auth_webauthn_credentials TO console_rt").execute(&identity.runtime.migrator).await?;
    let after=actual_catalog(&identity.readback).await?;assert_ne!(before,after);
    let result=admission::admit_process(&identity.runtime.migrate_config,&installed).await;
    assert!(matches!(result,Err(admission::AdmissionError::CatalogMismatch{..})));
    Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn m0_actual_security_definer_search_path_drift_blocks_enrollment(pool:PgPool)->TestResult {
    let identity=build_identity_deployment(pool,"m0_search_path",1).await?;
    let installed=admission::load_installed_release(&identity.runtime.migrate_config).await?;
    let before=actual_catalog(&identity.readback).await?;
    let function:Option<(String,String)>=sqlx::query_as("SELECT n.nspname||'.'||quote_ident(p.proname),pg_get_function_identity_arguments(p.oid) FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname='public' AND p.prosecdef AND NOT coalesce(p.proconfig,ARRAY[]::text[]) @> ARRAY['search_path=public, pg_catalog'] ORDER BY p.proname,pg_get_function_identity_arguments(p.oid) LIMIT 1").fetch_optional(&identity.readback).await?;
    let (name,args)=function.ok_or("real SECURITY DEFINER fixture prerequisite absent")?;
    // Identifiers/signature come from PostgreSQL's own formatted catalog output,
    // never tenant input. Restrict selected schema and named regular function.
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!("ALTER FUNCTION {name}({args}) SET search_path TO public,pg_catalog"))).execute(&identity.runtime.migrator).await?;
    let after=actual_catalog(&identity.readback).await?;assert_ne!(before,after,"mutation must change actual function configuration");
    assert!(matches!(admission::admit_process(&identity.runtime.migrate_config,&installed).await,Err(admission::AdmissionError::CatalogMismatch{..})));
    Ok(())
}

#[sqlx::test(migrations="../crates/platform/db/migrations")]
async fn m2_m4_incomplete_business_catalog_cannot_readmit_while_account_still_works(pool:PgPool)->TestResult {
    let mut d=crate::native_fixture::build_native_deployment(pool,"m2_m4_dormant",1).await?;
    let installed=admission::load_installed_release(&d.runtime.migrate_config).await?;
    let before:Value=sqlx::query_scalar("SELECT jsonb_build_object('inputs',(SELECT count(*) FROM payroll_input_revisions),'runs',(SELECT count(*) FROM payroll_draft_runs),'consumptions',(SELECT count(*) FROM gov_approval_consumptions))").fetch_one(&d.readback).await?;
    // Real deployment drain precedes the incompatible catalog transition. Account
    // identity has its own admitted owner; no invented dormant flag or seed row.
    let pause=admission::pause_and_drain_effects(&d.runtime.migrate_config).await?;
    sqlx::query("ALTER TABLE payroll_input_units RENAME TO fixture_incomplete_payroll_input_units").execute(&d.runtime.migrator).await?;
    let rejected=admission::admit_process(&d.runtime.migrate_config,&installed).await;
    let identity_read=console_platform_auth::account_session::read_own_account(&d.pool,&d.verifier,&d.accounts.submitter.account_access_token).await;
    // Restore the exact owned schema fault before assertions or returning errors.
    sqlx::query("ALTER TABLE fixture_incomplete_payroll_input_units RENAME TO payroll_input_units").execute(&d.runtime.migrator).await?;
    assert!(matches!(rejected,Err(admission::AdmissionError::CatalogMismatch{..})));
    assert_eq!(identity_read?.account_id,d.accounts.submitter.account_id);
    let after:Value=sqlx::query_scalar("SELECT jsonb_build_object('inputs',(SELECT count(*) FROM payroll_input_revisions),'runs',(SELECT count(*) FROM payroll_draft_runs),'consumptions',(SELECT count(*) FROM gov_approval_consumptions))").fetch_one(&d.readback).await?;
    assert_eq!(before,after);
    // Normal observed release admission, not fixture data population, restores
    // the serving capability. A stale pre-drain Company context is never reused.
    let admitted=admission::admit_process(&d.runtime.migrate_config,&installed).await?;
    assert!(admitted.command_service().is_some());
    admission::resume_effects(&d.runtime.migrate_config,&pause,&admitted).await?;
    let f=&mut d.companies[0];
    let selection=console_identity_adapter_postgres::account13::read_current_company_selection(&d.pool,&d.operator,f.org).await?;
    f.submitter=console_platform_request_context::account::resolve_company_context(&d.pool,&d.verifier,&d.accounts.submitter.account_access_token,&selection,admitted.serving_binding()).await?;
    let (run,_)=f.staged(false).await?;assert!(!run.payable);
    Ok(())
}
