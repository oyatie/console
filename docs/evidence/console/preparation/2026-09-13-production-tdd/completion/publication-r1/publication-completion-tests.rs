//! Child of auth_rest; ordinary signed TEST_ONLY custody, never fake verified rows.
use crate::expanded_custody_producer::{ExpandedCustodyFixture,Result,digest};
use console_platform_auth::terms_publication as publisher;
use console_platform_test_support::{TestDatabaseLogin,login_test_pool};
use serde_json::{json,Value};
use sqlx::PgPool;
use time::OffsetDateTime;

async fn fixture(pool:&PgPool)->Result<ExpandedCustodyFixture>{
    super::prepare_http_database(pool).await;
    ExpandedCustodyFixture::new("publication-completion",pool.connect_options().get_database().ok_or("db")?,&["test.account.service".into()])
}
async fn state(pool:&PgPool)->Result<Value>{
    Ok(sqlx::query_scalar("SELECT jsonb_build_object('head',(SELECT jsonb_agg(to_jsonb(h) ORDER BY id) FROM account_terms_head h),'receipts',(SELECT jsonb_agg(to_jsonb(r) ORDER BY revision) FROM account_terms_release_receipts r))").fetch_one(pool).await?)
}
async fn publish(f:&ExpandedCustodyFixture,expected:i64,manifest:&Value)->Result<publisher::PublicationReceipt>{
    let approval=f.approve_manifest(expected,&serde_json::to_vec(manifest)?)?;
    let operator=f.authenticate(publisher::Environment::TestOnly).await?;
    Ok(publisher::publish_terms(&operator,publisher::OperatorApprovalRef{kind:publisher::ApprovalKind::OperatorReleaseApproval,approval_sha256:approval.approval_digest}).await?)
}
fn invalid(error:&(dyn std::error::Error+Send+Sync+'static))->bool{
    error.downcast_ref::<publisher::PublicationError>().is_some_and(|e|matches!(e,publisher::PublicationError::InvalidArtifact))
}
fn put_content(f:&ExpandedCustodyFixture,bytes:&[u8])->Result<String>{
    use std::{io::Write,os::unix::fs::OpenOptionsExt};
    let hash=digest(bytes);let path=f.directory.path().join("objects").join(&hash);
    let mut file=std::fs::OpenOptions::new().write(true).create_new(true).mode(0o400).open(path)?;
    file.write_all(bytes)?;file.sync_all()?;Ok(hash)
}

#[sqlx::test(migrations=false)]
async fn publication_title_and_locale_unicode_cardinality_has_both_boundaries(pool:PgPool)->Result{
    let f=fixture(&pool).await?;let mut revision=0;
    // JSON Schema min/maxLength count Unicode code points, not UTF-8 bytes.
    // Non-ASCII ASCII-equivalent values ensure byte/character confusion fails.
    for (title,locale) in [("x".into(),"ko".into()),("가".repeat(128),"가".repeat(32))]{
        let mut m=f.manifest_value()?;m["items"][0]["title"]=json!(title);m["items"][0]["locale"]=json!(locale);
        let receipt=publish(&f,revision,&m).await?;revision+=1;assert_eq!(receipt.revision,revision);
    }
    for (field,value) in [("title",String::new()),("title","가".repeat(129)),("locale","가".into()),("locale","가".repeat(33))]{
        let before=state(&pool).await?;let mut m=f.manifest_value()?;m["items"][0][field]=json!(value);
        let error=publish(&f,revision,&m).await.expect_err("out-of-bounds artifact accepted");assert!(invalid(error.as_ref()));
        assert_eq!(state(&pool).await?,before);
    }
    assert_eq!(publish(&f,revision,&f.manifest_value()?).await?.revision,revision+1);Ok(())
}

#[sqlx::test(migrations=false)]
async fn publication_content_limit_counts_utf8_bytes_and_preserves_exact_digest(pool:PgPool)->Result{
    let f=fixture(&pool).await?;
    let mut exact="가".repeat(21845).into_bytes();exact.push(b'x');assert_eq!(exact.len(),65536);
    let hash=put_content(&f,&exact)?;let mut m=f.manifest_value()?;m["items"][0]["content_sha256"]=json!(hash);
    let receipt=publish(&f,0,&m).await?;assert_eq!(receipt.revision,1);
    assert_eq!(std::fs::read(f.directory.path().join("objects").join(&hash))?,exact);
    let before=state(&pool).await?;
    for bytes in ["가".repeat(21846).into_bytes(),vec![0xff,0xfe]]{
        let hash=put_content(&f,&bytes)?;let mut m=f.manifest_value()?;m["items"][0]["content_sha256"]=json!(hash);
        let error=publish(&f,1,&m).await.expect_err("invalid UTF8 content accepted");assert!(invalid(error.as_ref()));
        assert_eq!(state(&pool).await?,before);
    }
    assert_eq!(publish(&f,1,&f.manifest_value()?).await?.revision,2);Ok(())
}

#[sqlx::test(migrations=false)]
async fn publication_real_operator_challenge_expires_without_changing_head(pool:PgPool)->Result{
    let f=fixture(&pool).await?;
    let custody=publisher::load_deployment_custody(&f.config_path,publisher::Environment::TestOnly).await?;
    let challenge=publisher::begin_operator_authentication(&custody,"test.operator").await?;
    let signature=f.sign_operator_challenge(challenge.signing_bytes())?;
    let expires=challenge.expires_at();
    let now:OffsetDateTime=sqlx::query_scalar("SELECT clock_timestamp()").fetch_one(&pool).await?;
    assert!(expires>now);assert!(expires-now<=time::Duration::minutes(5),"operator challenge lifetime exceeds bounded acceptance run");
    let before=state(&pool).await?;
    tokio::time::timeout(std::time::Duration::from_secs(305),async{
        loop{let now:OffsetDateTime=sqlx::query_scalar("SELECT clock_timestamp()").fetch_one(&pool).await?;
            if now>=expires{return Ok::<_,sqlx::Error>(());}
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }).await??;
    assert!(matches!(publisher::finish_operator_authentication(custody,challenge,&signature).await,Err(publisher::PublicationError::OperatorChallengeExpired)));
    assert_eq!(state(&pool).await?,before);
    assert_eq!(publish(&f,0,&f.manifest_value()?).await?.revision,1);Ok(())
}

#[sqlx::test(migrations=false)]
async fn publication_digest_is_local_address_and_never_remote_or_alias_resolution(pool:PgPool)->Result{
    let f=fixture(&pool).await?;
    // A loopback URL is valid inert content. Its exact digest is the local object
    // address; interpreting the content as a URL would fetch different bytes.
    let listener=std::net::TcpListener::bind("127.0.0.1:0")?;listener.set_nonblocking(true)?;
    let url=format!("http://{}/fixture-content",listener.local_addr()?);
    let hash=put_content(&f,url.as_bytes())?;let mut m=f.manifest_value()?;m["items"][0]["content_sha256"]=json!(hash);
    let receipt=publish(&f,0,&m).await?;assert_eq!(receipt.revision,1);
    assert_eq!(std::fs::read(f.directory.path().join("objects").join(hash))?,url.as_bytes());
    let before=state(&pool).await?;
    for field in ["content_sha256","content_url"]{
        let mut bad=f.manifest_value()?;bad["items"][0][field]=json!(url);
        let error=publish(&f,1,&bad).await.expect_err("remote address accepted");assert!(invalid(error.as_ref()));
        assert_eq!(state(&pool).await?,before);
    }
    // A friendly alias file does not substitute for a missing exact digest.
    let mut absent=f.manifest_value()?;let missing="ab".repeat(32);
    assert!(!f.directory.path().join("objects").join(&missing).exists());
    std::fs::write(f.directory.path().join("objects").join("current"),&f.content)?;
    absent["items"][0]["content_sha256"]=json!(missing);
    let error=publish(&f,1,&absent).await.expect_err("friendly alias substituted");
    assert!(error.downcast_ref::<publisher::PublicationError>().is_some_and(|e|matches!(e,publisher::PublicationError::CustodyUnavailable)));
    assert_eq!(state(&pool).await?,before);
    // This finite observation composes with static local-only custody/source
    // enrollment; it is not a proof against arbitrary delayed background work.
    for _ in 0..20{assert!(matches!(listener.accept(),Err(e) if e.kind()==std::io::ErrorKind::WouldBlock));tokio::time::sleep(std::time::Duration::from_millis(10)).await;}
    assert_eq!(publish(&f,1,&f.manifest_value()?).await?.revision,2);Ok(())
}

#[sqlx::test(migrations=false)]
async fn publication_runtime_roles_cannot_escalate_or_write_any_publication_column(pool:PgPool)->Result{
    let f=fixture(&pool).await?;let before=state(&pool).await?;
    for login in [TestDatabaseLogin::Auth,TestDatabaseLogin::Business,TestDatabaseLogin::OntologyCommand,TestDatabaseLogin::LeaveCommand,TestDatabaseLogin::PlatformForceCommand]{
        let rt=login_test_pool(&pool,login).await;
        let privilege:(bool,bool,bool)=sqlx::query_as("SELECT pg_has_role(session_user,'console_terms_publisher','USAGE'),pg_has_role(session_user,'console_terms_publisher','SET'),(SELECT rolsuper OR rolcreaterole OR rolbypassrls FROM pg_roles WHERE rolname=session_user)").fetch_one(&rt).await?;
        assert_eq!(privilege,(false,false,false));
        let writes:i64=sqlx::query_scalar("SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace CROSS JOIN (VALUES ('INSERT'),('UPDATE'),('DELETE'),('TRUNCATE'),('TRIGGER')) p(privilege) WHERE n.nspname='public' AND c.relname IN ('account_terms_head','account_terms_release_receipts') AND has_table_privilege(session_user,c.oid,p.privilege)").fetch_one(&rt).await?;
        assert_eq!(writes,0);
        let columns:i64=sqlx::query_scalar("SELECT count(*) FROM pg_attribute a JOIN pg_class c ON c.oid=a.attrelid JOIN pg_namespace n ON n.oid=c.relnamespace CROSS JOIN (VALUES ('INSERT'),('UPDATE'),('REFERENCES')) p(privilege) WHERE a.attnum>0 AND NOT a.attisdropped AND n.nspname='public' AND c.relname IN ('account_terms_head','account_terms_release_receipts') AND has_column_privilege(session_user,c.oid,a.attnum,p.privilege)").fetch_one(&rt).await?;
        assert_eq!(columns,0);
        // All actual overloads of the sole declared publication SQL owner must
        // refuse EXECUTE. Extra helper names are bound by full M0/source review.
        let entries:Vec<bool>=sqlx::query_scalar("SELECT has_function_privilege(session_user,p.oid,'EXECUTE') FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname='public' AND p.proname='account_publish_terms'").fetch_all(&rt).await?;
        assert!(!entries.is_empty());assert!(entries.iter().all(|v|!*v));rt.close().await;
    }
    assert_eq!(state(&pool).await?,before);assert_eq!(publish(&f,0,&f.manifest_value()?).await?.revision,1);Ok(())
}
