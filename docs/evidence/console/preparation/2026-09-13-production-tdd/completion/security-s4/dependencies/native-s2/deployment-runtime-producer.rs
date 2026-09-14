//! Fresh database and ordinary deployment process configuration. No seed/result files.
//! New deployment::start_services is production startup composition, not test authority.
use console_app::deployment;
use console_platform_auth::{JwtVerifier, PasskeyService};
use sqlx::{PgPool, postgres::PgPoolOptions};
use url::Url;
use crate::{native_fixture::TestResult,operator_custody_producer::CustodyFixture};

pub struct Runtime {
    pub migrator:PgPool, pub migrate_config:console_app::AppConfig,
    pub startup:PgPool,pub auth:PgPool, pub runtime:PgPool, pub readback:PgPool,
    pub verifier:std::sync::Arc<JwtVerifier>, pub passkeys:PasskeyService,
    pub origin:Url, pub client:reqwest::Client,
    pub services:deployment::RunningServices,
    pub config:deployment::DeploymentConfig,
    pub custody:CustodyFixture, pub storage:tempfile::TempDir,
}
fn role_url(variable:&str,database:&str,role:&str,owner:&PgPool)->TestResult<Url> {
    let mut url=Url::parse(&std::env::var(variable)?)?;
    assert_eq!(url.username(),role); assert!(url.password().is_some());
    assert!(url.query().is_none()&&url.fragment().is_none());
    assert_eq!(url.host_str(),Some(owner.connect_options().get_host()));
    assert_eq!(url.port_or_known_default().unwrap_or(5432),owner.connect_options().get_port());
    url.set_path(database); Ok(url)
}
async fn role_pool(variable:&str,database:&str,role:&str,owner:&PgPool)->TestResult<PgPool> {
    let url=role_url(variable,database,role,owner)?;
    let pool=PgPoolOptions::new().max_connections(12).connect(url.as_str()).await?;
    let actual:(String,String,String)=sqlx::query_as("SELECT current_database(), session_user, current_user").fetch_one(&pool).await?;
    assert_eq!(actual,(database.to_string(),role.to_string(),role.to_string()));
    Ok(pool)
}
pub async fn start_runtime(owner_pool:&PgPool,label:&str)->TestResult<Runtime> {
    let config=deployment::load_config(std::path::Path::new(&std::env::var("CONSOLE_NATIVE_DEPLOYMENT_CONFIG")?))?;
    start_runtime_with_config(owner_pool,label,config).await
}
// Untrusted ordinary startup configuration; actual deployment startup still
// verifies installed release custody and admission. It is never a ready receipt.
pub async fn start_runtime_with_config(owner_pool:&PgPool,label:&str,mut config:deployment::DeploymentConfig)->TestResult<Runtime> {
    let database:String=sqlx::query_scalar("SELECT current_database()").fetch_one(owner_pool).await?;
    assert!(database.starts_with("_sqlx_test_"));
    // This file contains only deployment keys, signed compiled-build custody,
    // service/workload trust, limits and local storage configuration. It may not
    // contain Account/Company/Person IDs, refs, contexts or successful fixture rows.
    assert_eq!(config.environment,deployment::Environment::TestOnly);
    config.database=database.clone();
    config.listen="127.0.0.1:0".parse()?;
    let storage=tempfile::Builder::new().prefix("console-native-artifacts-").tempdir()?;
    config.storage_root=storage.path().to_path_buf();
    let custody=CustodyFixture::new(label,&database)?;
    config.terms_publication_custody=custody.config_path.clone();
    let migration_url=role_url("CONSOLE_NATIVE_MIGRATION_DATABASE_URL",&database,"console_app",owner_pool)?;
    let mut migration_config=config.app.clone();
    migration_config.role=console_app::AppRole::Migrate;
    migration_config.database_url=Some(migration_url.to_string());
    console_app::run_migrations(&migration_config).await?;
    let migrator=role_pool("CONSOLE_NATIVE_MIGRATION_DATABASE_URL",&database,"console_app",owner_pool).await?;
    let startup=role_pool("CONSOLE_NATIVE_STARTUP_DATABASE_URL",&database,"console_auth_startup",owner_pool).await?;
    let auth=role_pool("CONSOLE_NATIVE_AUTH_DATABASE_URL",&database,"console_auth_rt",owner_pool).await?;
    let runtime=role_pool("CONSOLE_NATIVE_RUNTIME_DATABASE_URL",&database,"console_rt",owner_pool).await?;
    let readback=role_pool("CONSOLE_NATIVE_READBACK_DATABASE_URL",&database,"console_native_readback",owner_pool).await?;
    let services=deployment::start_services(config.clone(),runtime.clone(),auth.clone()).await?;
    // Startup owns router mounting, persisted signing configuration, worker trust,
    // serving admission and lifecycle. Does NOT populate business rows.
    let origin=services.local_origin().clone();
    let verifier=std::sync::Arc::new(JwtVerifier::from_es256_public_pem(config.jwt_settings.clone(),&config.jwt_public_pem)?);
    let passkeys=PasskeyService::new(config.webauthn_settings_for_origin(&origin)?)?;
    let client=reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).build()?;
    Ok(Runtime{migrator,migrate_config:migration_config,startup,auth,runtime,readback,verifier,passkeys,origin,client,services,config,custody,storage})
}
