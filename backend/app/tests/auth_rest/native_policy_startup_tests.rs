// Add ordinary child module inside company_setup. Production installer/classifiers,
// genuine runtime LOGIN, actual AppState startup and existing native fixtures only.
use super::*;
use console_app::{AppConfig, AppError, AppState, DatabaseDependency, build_router};
use console_identity_adapter_postgres::PgOrgStore;
use console_identity_application::company_policy::{
    CompanyPolicyScope, CompanyPolicyStore,
    business::NativeCompanyBusinessCommandV1,
    read_company_identity,
    workflow::{
        NativePolicyCommandRef, NativePolicyOutcome, NativePolicyWorkflowError,
        accept_native_policy_command, native_policy_form, submit_native_policy_command,
    },
};
use console_kernel_core::{OrgId, TraceContext};
use console_platform_auth::{
    JwtIssuer, JwtSettings, JwtVerifier, account::AccountEnrollmentCredentials,
};
use console_platform_authz::company_policy::CompanyPolicy;
use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
use futures::FutureExt;
use std::panic::AssertUnwindSafe;

const POLICY_CLASSIFIER: &str =
    include_str!("../../../../ops/postgres-native-company-policy-custody-state.sql");
const APP_POLICY_CLASSIFIER: &str =
    include_str!("../../src/native_company_policy_custody_state.sql");
const POLICY_INSTALLER: &str =
    include_str!("../../../../ops/postgres-finalize-native-company-policy.sql");
const CLASSIFIER_SESSION: &str = include_str!("../../src/account_custody_session.sql");

async fn classified(pool: &PgPool, source: &'static str) -> String {
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(tx.as_mut())
        .await
        .unwrap();
    sqlx::raw_sql(CLASSIFIER_SESSION)
        .execute(tx.as_mut())
        .await
        .unwrap();
    let value = sqlx::query_scalar(sqlx::AssertSqlSafe(source))
        .fetch_one(tx.as_mut())
        .await
        .expect("actual generated classifier must execute; SQL error is not absence");
    tx.commit().await.unwrap();
    value
}
async fn policy_classified(pool: &PgPool) -> String {
    assert_eq!(
        POLICY_CLASSIFIER, APP_POLICY_CLASSIFIER,
        "generated serving classifier copies drifted"
    );
    classified(pool, POLICY_CLASSIFIER).await
}
async fn install_policy(pool: &PgPool) {
    assert_eq!(
        hex::encode(Sha256::digest(POLICY_INSTALLER.as_bytes())),
        "3342fbb085b2d9644ec440edf2671c8de05c6153c547cd3a1b8b8c76be1f112e",
        "actual independently reviewed installer bytes changed"
    );
    assert_eq!(
        hex::encode(Sha256::digest(POLICY_CLASSIFIER.as_bytes())),
        "a0f4aef40030f33c3965589da45fe88d462693186c4af78622441fab039c7714",
        "actual independently reviewed classifier bytes changed"
    );
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL statement_timeout='120s'")
        .execute(tx.as_mut()).await.unwrap();
    // Sole production operator entry. No copied function bodies/test installer.
    sqlx::raw_sql(POLICY_INSTALLER)
        .execute(tx.as_mut())
        .await
        .expect("actual native policy custody finalization prerequisite");
    tx.commit().await.unwrap();
    let runtime = login_test_pool(pool, TestDatabaseLogin::Business).await;
    assert_eq!(
        policy_classified(&runtime).await,
        "native_company_policy.finalized"
    );
    runtime.close().await;
}
// New browser prerequisite only. Original Account/Company helpers are untouched.
pub(super) async fn prepare_policy_ready_database(pool: &PgPool) {
    prepare_ready_database(pool).await;
    install_policy(pool).await;
}
async fn ready_status(state: &AppState) -> StatusCode {
    build_router(state.clone())
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
        .status()
}
async fn close_states(states: &[AppState], outcome: Result<(), Box<dyn std::any::Any + Send>>) {
    for state in states {
        state.shutdown_realtime().await;
    }
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
}
async fn startup_refused(config: AppConfig, expected: &str) {
    match AppState::from_config(config).await {
        Ok(state) => {
            state.shutdown_realtime().await;
            panic!("invalid custody admitted startup");
        }
        Err(AppError::Config(code)) => assert_eq!(code, expected),
        Err(_) => panic!("unrelated startup failure does not prove custody refusal"),
    }
}
async fn configured_fixture(pool: &PgPool, policy: bool) -> (Fixture, SigningKey, AppState) {
    if policy {
        prepare_policy_ready_database(pool).await;
    } else {
        prepare_ready_database(pool).await;
    }
    let artifacts = Artifacts::new();
    let key = SigningKey::random(&mut OsRng);
    let state = AppState::from_config(account_browser_config(pool, artifacts.root.clone(), &key))
        .await
        .unwrap();
    let app = Fixture {
        service: build_router(state.clone()),
        _artifacts: artifacts,
        pool: pool.clone(),
    };
    (app, key, state)
}
async fn create_owned_company(pool: &PgPool, app: Fixture) -> (Fixture, Cookies, Committed) {
    let (app, account, cookies, startup, _) = designated_fixture(pool, app).await;
    let command = Uuid::new_v4();
    let input = enrollment(command, account.account);
    let csrf = proof(&app, &cookies).await;
    let response = submit(&app, &cookies, &csrf, &input).await;
    let created = committed(
        &response,
        StatusCode::CREATED,
        command,
        account.account,
        false,
    );
    durable(pool, &created, &input, account.account).await;
    startup.close().await;
    (app, cookies, created)
}
fn bindings(config: &AppConfig) -> (JwtVerifier, JwtIssuer, time::Duration) {
    let auth = config.auth_rest.as_ref().unwrap();
    let settings = JwtSettings {
        issuer: auth.jwt_issuer.clone(),
        audience: auth.jwt_audience.clone(),
        access_token_ttl: time::Duration::minutes(15),
    };
    let verifier =
        JwtVerifier::from_es256_public_pem(settings.clone(), auth.jwt_public_key_pem.as_bytes())
            .unwrap();
    let issuer = JwtIssuer::from_es256_pem(
        settings,
        auth.jwt_private_key_pem.as_bytes(),
        auth.jwt_public_key_pem.as_bytes(),
    )
    .unwrap();
    (verifier, issuer, auth.refresh_family_absolute_ttl)
}
fn read_credentials(cookies: &Cookies) -> AccountEnrollmentCredentials {
    AccountEnrollmentCredentials::for_read(cookies.0.get(ACCESS).unwrap()).unwrap()
}

#[sqlx::test(migrations = false)]
async fn native_policy_absent_preserves_actual_account_and_company_startup(pool: PgPool) {
    prepare_http_database(&pool).await;
    let artifacts = Artifacts::new();
    let key = SigningKey::random(&mut OsRng);
    let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    assert_eq!(
        policy_classified(&runtime).await,
        "native_company_policy.absent"
    );
    let config = account_browser_config(&pool, artifacts.root.clone(), &key);
    let account = AppState::from_config(config.clone()).await.unwrap();
    let outcome = AssertUnwindSafe(async {
        assert_eq!(ready_status(&account).await, StatusCode::OK);
        assert_eq!(
            classified(
                &runtime,
                include_str!("../../src/account_custody_state.sql")
            )
            .await,
            "account_custody.native_finalized"
        );
    })
    .catch_unwind()
    .await;
    close_states(&[account], outcome).await;
    // Existing exact old Company finalizer, then real new application instance.
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql(include_str!(
        "../../../../ops/postgres-finalize-company-enrollment.sql"
    ))
    .execute(tx.as_mut())
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        policy_classified(&runtime).await,
        "native_company_policy.absent"
    );
    let company = AppState::from_config(config).await.unwrap();
    let outcome = AssertUnwindSafe(async {
        assert_eq!(
            classified(
                &runtime,
                include_str!("../../src/company_enrollment_custody_state.sql")
            )
            .await,
            "company_enrollment.finalized"
        );
        assert_eq!(ready_status(&company).await, StatusCode::OK);
    })
    .catch_unwind()
    .await;
    close_states(&[company], outcome).await;
    runtime.close().await;
}

#[sqlx::test(migrations = false)]
async fn native_policy_partial_presence_refuses_startup_without_old_fallback(pool: PgPool) {
    let (app, key, state) = configured_fixture(&pool, false).await;
    let config = account_browser_config(&pool, app._artifacts.root.clone(), &key);
    let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let outcome=AssertUnwindSafe(async {
        assert_eq!(ready_status(&state).await,StatusCode::OK);
        assert_eq!(policy_classified(&runtime).await,"native_company_policy.absent");
        let mutations=[
            ("CREATE TABLE public.native_company_policy_inputs_v1(probe integer)",
             "SELECT to_regclass('public.native_company_policy_inputs_v1') IS NOT NULL",
             Some("DROP TABLE public.native_company_policy_inputs_v1"),true),
            ("CREATE FUNCTION public.native_company_policy_probe_v1() RETURNS void LANGUAGE plpgsql AS 'BEGIN NULL; END;'",
             "SELECT to_regprocedure('public.native_company_policy_probe_v1()') IS NOT NULL",
             Some("DROP FUNCTION public.native_company_policy_probe_v1()"),false),
            ("ALTER TABLE public.company_authority_heads ADD COLUMN current_policy_receipt_id uuid",
             "SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_attribute WHERE attrelid='public.company_authority_heads'::regclass AND attname='current_policy_receipt_id' AND NOT attisdropped)",
             None,false),
        ];
        for (mutation,witness,restore,old_positive) in mutations {
            sqlx::raw_sql(sqlx::AssertSqlSafe(mutation)).execute(&pool).await.unwrap();
            let checked=AssertUnwindSafe(async {
                assert!(sqlx::query_scalar::<_,bool>(sqlx::AssertSqlSafe(witness)).fetch_one(&pool).await.unwrap());
                if old_positive {assert_eq!(classified(&runtime,include_str!("../../src/company_enrollment_custody_state.sql")).await,"company_enrollment.finalized");}
                assert_eq!(policy_classified(&runtime).await,"native_company_policy.profile_mismatch");
                startup_refused(config.clone(),"native_company_policy.profile_mismatch").await;
                assert_eq!(ready_status(&state).await,StatusCode::SERVICE_UNAVAILABLE);
            }).catch_unwind().await;
            if let Some(restore) = restore {
                sqlx::raw_sql(sqlx::AssertSqlSafe(restore)).execute(&pool).await.unwrap();
            }
            if let Err(panic)=checked {std::panic::resume_unwind(panic);}
            if restore.is_some() {
                assert_eq!(policy_classified(&runtime).await,"native_company_policy.absent");
                assert_eq!(ready_status(&state).await,StatusCode::OK);
            }
            // Last pointer-column fault stays only in this disposable DB. DROP
            // COLUMN retains attisdropped metadata; it is not exact restoration.
        }
    }).catch_unwind().await;
    close_states(&[state], outcome).await;
    runtime.close().await;
}

#[cfg(feature = "test-browser")]
#[sqlx::test(migrations = false)]
async fn native_policy_verified_startup_drives_browser_current_reader_after_epoch_advance(
    pool: PgPool,
) {
    // Existing reviewed driver; this case requires explicit browser prerequisites.
    // The one proposed fixture branch installs actual policy custody only when true.
    company_browser_journey(pool.clone(), true).await;
    let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    assert_eq!(
        policy_classified(&runtime).await,
        "native_company_policy.finalized"
    );
    let operations:Vec<i16>=sqlx::query_scalar("SELECT DISTINCT operation FROM public.native_company_policy_receipts_v1 WHERE outcome='COMMITTED' ORDER BY operation")
        .fetch_all(&pool).await.unwrap();
    assert_eq!(
        operations,
        vec![1, 2, 3],
        "actual browser must install,grant,revoke"
    );
    let heads:(i64,bool)=sqlx::query_as("SELECT count(*),COALESCE(bool_and(h.epoch>1 AND h.epoch=r.epoch_after AND r.outcome='COMMITTED'),false) FROM public.company_authority_heads h JOIN public.native_company_policy_receipts_v1 r ON r.org_id=h.org_id AND r.receipt_id=h.current_policy_receipt_id")
        .fetch_one(&pool).await.unwrap();
    assert!(
        heads.0 >= 1 && heads.1,
        "browser history must retain actual successful head"
    );
    runtime.close().await;
}

#[sqlx::test(migrations = false)]
async fn native_policy_current_reader_fault_never_uses_still_working_initial_reader(pool: PgPool) {
    let (app, key, state) = configured_fixture(&pool, true).await;
    let (app, cookies, created) = create_owned_company(&pool, app).await;
    let config = account_browser_config(&pool, app._artifacts.root.clone(), &key);
    let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let (verifier, _, ttl) = bindings(&config);
    let old = PgOrgStore::new(runtime.clone()).with_native_account_auth(verifier, ttl);
    let policy = CompanyPolicy::new().unwrap();
    let credentials = read_credentials(&cookies);
    let target = format!("/companies/{}", created.company);
    let before = all_rows(&pool).await;
    let outcome=AssertUnwindSafe(async {
        assert_eq!(document(&app,&target,&cookies).await.status,StatusCode::OK);
        assert!(read_company_identity(&old,&policy,&credentials,OrgId::from_uuid(created.company)).await.is_ok());
        assert_eq!(ready_status(&state).await,StatusCode::OK);
        // Actual missing-v2 fault only after both current and old positive controls.
        sqlx::raw_sql("ALTER FUNCTION public.identity_company_projection_v2(uuid,uuid,uuid) RENAME TO policy_test_hidden_projection_v2")
            .execute(&pool).await.unwrap();
        let checked=AssertUnwindSafe(async {
            assert!(sqlx::query_scalar::<_,bool>("SELECT to_regprocedure('public.identity_company_projection_v2(uuid,uuid,uuid)') IS NULL AND to_regprocedure('public.policy_test_hidden_projection_v2(uuid,uuid,uuid)') IS NOT NULL")
                .fetch_one(&pool).await.unwrap());
            assert!(read_company_identity(&old,&policy,&credentials,OrgId::from_uuid(created.company)).await.is_ok(),"old positive proves fallback would reveal identity");
            let denied=document(&app,&target,&cookies).await;
            assert_eq!(denied.status,StatusCode::SERVICE_UNAVAILABLE);denied.private();
            let text=String::from_utf8(denied.bytes.clone()).unwrap();
            assert!(!text.contains("연결된 업무 회사")&&!text.contains(&created.company.to_string()));
            assert_eq!(ready_status(&state).await,StatusCode::SERVICE_UNAVAILABLE);
        }).catch_unwind().await;
        sqlx::raw_sql("ALTER FUNCTION public.policy_test_hidden_projection_v2(uuid,uuid,uuid) RENAME TO identity_company_projection_v2")
            .execute(&pool).await.unwrap();
        if let Err(panic)=checked{std::panic::resume_unwind(panic);}
        assert_eq!(document(&app,&target,&cookies).await.status,StatusCode::OK);
        assert_eq!(ready_status(&state).await,StatusCode::OK);
        assert!(before==all_rows(&pool).await,"read/fault/recovery must not change any business table row");
    }).catch_unwind().await;
    close_states(&[state], outcome).await;
    runtime.close().await;
}

#[sqlx::test(migrations = false)]
async fn native_policy_successor_metadata_requires_restarted_reader_profile(pool: PgPool) {
    let (app, key, old) = configured_fixture(&pool, false).await;
    let config = account_browser_config(&pool, app._artifacts.root.clone(), &key);
    let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let outcome = AssertUnwindSafe(async {
        assert_eq!(ready_status(&old).await, StatusCode::OK);
        assert_eq!(
            policy_classified(&runtime).await,
            "native_company_policy.absent"
        );
        install_policy(&pool).await;
        assert_eq!(
            ready_status(&old).await,
            StatusCode::SERVICE_UNAVAILABLE,
            "old selected reader cannot remain ready against successor schema"
        );
        let fresh = AppState::from_config(config).await.unwrap();
        let checked = AssertUnwindSafe(async {
            assert_eq!(
                policy_classified(&runtime).await,
                "native_company_policy.finalized"
            );
            assert_eq!(ready_status(&fresh).await, StatusCode::OK);
        })
        .catch_unwind()
        .await;
        close_states(&[fresh], checked).await;
    })
    .catch_unwind()
    .await;
    close_states(&[old], outcome).await;
    runtime.close().await;
}

#[sqlx::test(migrations = false)]
async fn native_policy_unverified_new_is_unready_and_does_not_enable_successor_routes(
    pool: PgPool,
) {
    let (app, key, verified) = configured_fixture(&pool, true).await;
    let (mut app, cookies, created) = create_owned_company(&pool, app).await;
    let config = account_browser_config(&pool, app._artifacts.root.clone(), &key);
    let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
    let injected = AppState::new(config, DatabaseDependency::Postgres(runtime.clone()))
        .unwrap()
        .with_auth_database(auth.clone());
    let old_router = build_router(injected.clone());
    let target = format!("/companies/{}/policy/payroll-read/install", created.company);
    let before = all_rows(&pool).await;
    let outcome = AssertUnwindSafe(async {
        assert_eq!(
            policy_classified(&runtime).await,
            "native_company_policy.finalized"
        );
        assert_eq!(
            ready_status(&injected).await,
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(ready_status(&verified).await, StatusCode::OK);
        // Authenticated positive uses the identical actual target, not a typo404.
        let positive = document(&app, &target, &cookies).await;
        assert_eq!(positive.status, StatusCode::OK);
        positive.private();
        let original_service = std::mem::replace(&mut app.service, old_router.clone());
        let rejected = document(&app, &target, &cookies).await;
        app.service = original_service;
        assert!(matches!(
            rejected.status,
            StatusCode::NOT_FOUND | StatusCode::SERVICE_UNAVAILABLE
        ));
        let html = String::from_utf8(rejected.bytes.clone()).unwrap();
        assert!(!html.contains("name=\"csrf_proof\""));
        assert!(
            before == all_rows(&pool).await,
            "GET form/custody checks cannot create business state"
        );
    })
    .catch_unwind()
    .await;
    close_states(&[injected, verified], outcome).await;
    auth.close().await;
    runtime.close().await;
}

#[sqlx::test(migrations = false)]
async fn native_policy_old_adapter_rejects_command_new_atomic_builder_is_real_control(
    pool: PgPool,
) {
    let (app, key, state) = configured_fixture(&pool, true).await;
    let (app, cookies, created) = create_owned_company(&pool, app).await;
    let config = account_browser_config(&pool, app._artifacts.root.clone(), &key);
    let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    let (verifier, issuer, ttl) = bindings(&config);
    let old = PgOrgStore::new(runtime.clone()).with_native_account_auth(verifier.clone(), ttl);
    let current =
        PgOrgStore::new(runtime.clone()).with_native_account_policy(verifier, issuer, ttl);
    let policy = CompanyPolicy::new().unwrap();
    let command = NativeCompanyBusinessCommandV1::install(
        Uuid::new_v4(),
        OrgId::from_uuid(created.company),
        1,
    )
    .unwrap();
    let outcome=AssertUnwindSafe(async {
        let form=native_policy_form(&current,&policy,&read_credentials(&cookies),NativePolicyCommandRef::from_command(&command)).await.unwrap();
        let credentials=AccountEnrollmentCredentials::for_mutation(cookies.0.get(ACCESS).unwrap(),form.proof.as_str()).unwrap();
        let before=all_rows(&pool).await;
        assert!(matches!(accept_native_policy_command(&old,&policy,&credentials,&command,&TraceContext::generate()).await,Err(NativePolicyWorkflowError::Unavailable)));
        assert!(before==all_rows(&pool).await,"old builder must not accept or write anything");
        let executed=submit_native_policy_command(&current,&policy,&credentials,&command,&TraceContext::generate()).await.unwrap();
        assert!(executed.inserted&&matches!(executed.terminal.outcome,NativePolicyOutcome::Committed(_)));
        let actual:(i64,i64,bool)=sqlx::query_as("SELECT (SELECT count(*) FROM public.native_company_policy_inputs_v1 WHERE actor_account_id=$1 AND command_id=$2),(SELECT count(*) FROM public.native_company_policy_receipts_v1 WHERE actor_account_id=$1 AND command_id=$2 AND receipt_id=$3 AND outcome='COMMITTED'),EXISTS(SELECT 1 FROM public.company_authority_heads WHERE org_id=$4 AND epoch=2 AND current_policy_receipt_id=$3)")
            .bind(created.administrator).bind(command.command_id()).bind(executed.terminal.receipt_id).bind(created.company)
            .fetch_one(&pool).await.unwrap();
        assert_eq!(actual,(1,1,true));
        let read = read_credentials(&cookies);
        let scope=current.lock_current(&read,OrgId::from_uuid(created.company)).await.unwrap();
        let a=scope.authority().expect("actual current Company authority");
        assert_eq!(a.epoch(),2);assert_eq!(a.current_policy_receipt_id(),Some(executed.terminal.receipt_id));
        scope.finish().await.unwrap();
    }).catch_unwind().await;
    close_states(&[state], outcome).await;
    runtime.close().await;
}
