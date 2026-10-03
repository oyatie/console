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
        "4e7fc41b1d2ed6c2155d44d43347c18815ed9e70611eb9285f21e1c590bf996a",
        "actual independently reviewed installer bytes changed"
    );
    assert_eq!(
        hex::encode(Sha256::digest(POLICY_CLASSIFIER.as_bytes())),
        "072794defc065f8730eafeab66a111bdd2439c49b8c1d7c3052ca83a7edf1479",
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

mod native_org_bridge_bootstrap_red {
    use super::*;

    #[sqlx::test(migrations = false)]
    async fn predecessor_profile_fences_legacy_tenant_read_before_org_ddl(pool: PgPool) {
        let (app, _, state) = native_people_directory_finalizer_tests::native_people_directory_row_lock_tests::configured_row_lock_native_directory_fixture(&pool).await;
        let outcome = AssertUnwindSafe(async {
            assert_eq!(ready_status(&state).await, StatusCode::OK);
            let (app, _, created) = create_owned_company(&pool, app).await;
            let native_origins: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM public.company_enrollment_receipts WHERE org_id=$1",
            )
            .bind(created.company)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(native_origins, 1, "real native Company prerequisite");

            let branch = seed_branch(&pool, "Bridge legacy region", "Bridge legacy branch").await;
            let actor = seed_user_with_branch(
                &pool,
                "Bridge legacy reader",
                "010-8900-0119",
                "SUPER_ADMIN",
                branch,
            )
            .await;
            let legacy_origin: (Uuid, i64) = sqlx::query_as(
                "SELECT u.org_id, (SELECT count(*) FROM public.company_enrollment_receipts r WHERE r.org_id=u.org_id) FROM public.users u WHERE u.id=$1",
            )
            .bind(actor.as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(legacy_origin, (*OrgId::knl().as_uuid(), 0));
            let legacy_topology: bool = sqlx::query_scalar(
                "SELECT o.slug='knl' AND o.origin_account_id IS NULL \
                        AND o.origin_command_id IS NULL AND o.origin_receipt_id IS NULL \
                        AND g.origin_account_id IS NULL AND g.origin_command_id IS NULL \
                        AND g.origin_receipt_id IS NULL \
                        AND EXISTS(SELECT 1 FROM public.group_memberships m \
                                   WHERE m.group_id=o.group_id AND m.org_id=o.id) \
                 FROM public.organizations o JOIN public.groups g ON g.id=o.group_id \
                 WHERE o.id=$1",
            )
            .bind(*OrgId::knl().as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
            assert!(legacy_topology, "legacy Group and Company prerequisite");
            let token = admin_session_via_otp(&app.service, &pool, actor).await;
            let auth_read = get_legacy_raw(&app.service, "/api/v1/auth/passkeys", &token).await;
            assert_eq!(auth_read.status(), StatusCode::OK, "real legacy session prerequisite");

            let before = all_rows(&pool).await;
            let response = get_legacy_raw(&app.service, "/api/v1/users/me", &token).await;
            assert_eq!(
                response.status(),
                StatusCode::SERVICE_UNAVAILABLE,
                "ORG_BRIDGE_BOOTSTRAP: legacy bearer tenant read escaped before positive provenance DDL",
            );
            let bytes = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
            let problem: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(problem["error"]["code"], "legacy_provenance_unavailable");
            assert_eq!(all_rows(&pool).await, before, "fenced read changed durable state");
            drop(app);
        })
        .catch_unwind()
        .await;
        close_states(&[state], outcome).await;
    }
}

mod native_org_bridge_transport_red {
    use super::*;

    struct LoopbackServer(tokio::task::JoinHandle<std::io::Result<()>>);

    impl Drop for LoopbackServer {
        fn drop(&mut self) {
            self.0.abort();
        }
    }

    #[sqlx::test(migrations = false)]
    async fn predecessor_profile_fences_legacy_ssr_and_websocket_before_org_ddl(pool: PgPool) {
        let (app, _, state) = native_people_directory_finalizer_tests::native_people_directory_row_lock_tests::configured_row_lock_native_directory_fixture(&pool).await;
        let outcome = AssertUnwindSafe(async {
            assert_eq!(ready_status(&state).await, StatusCode::OK);
            let branch =
                seed_branch(&pool, "Bridge transport region", "Bridge transport branch").await;
            let actor = seed_user_with_branch(
                &pool,
                "Bridge transport reader",
                "010-8900-0120",
                "SUPER_ADMIN",
                branch,
            )
            .await;
            let token = admin_session_via_otp(&app.service, &pool, actor).await;
            assert_eq!(
                get_legacy_raw(&app.service, "/api/v1/users/me", "invalid-token")
                    .await
                    .status(),
                StatusCode::UNAUTHORIZED,
                "invalid credentials must retain the authentication boundary",
            );
            let invalid_ssr = get_legacy_raw(&app.service, "/organization", "invalid-token").await;
            assert_eq!(invalid_ssr.status(), StatusCode::OK);
            assert_eq!(
                String::from_utf8(
                    to_bytes(invalid_ssr.into_body(), 256 * 1024)
                        .await
                        .unwrap()
                        .to_vec()
                )
                .unwrap(),
                console_payroll_ui::render_shell(),
                "invalid credentials must retain the omitted Organization shell",
            );
            assert_eq!(
                get_legacy_raw(&app.service, "/api/v1/auth/passkeys", &token)
                    .await
                    .status(),
                StatusCode::OK,
                "self-auth must remain usable during bridge bootstrap",
            );
            let before = all_rows(&pool).await;
            let ssr = get_legacy_raw(&app.service, "/organization", &token).await;

            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let router = app.service.clone();
            let mut server =
                LoopbackServer(tokio::spawn(
                    async move { axum::serve(listener, router).await },
                ));
            let client = reqwest::Client::builder()
                .no_proxy()
                .timeout(std::time::Duration::from_secs(40))
                .build()
                .unwrap();
            let mut handshakes = Vec::new();
            for (credential, protocol) in [
                (None, false),
                (Some("invalid-token"), false),
                (Some("invalid-token"), true),
                (Some(token.as_str()), false),
                (Some(token.as_str()), true),
            ] {
                let mut request = client
                    .get(format!("http://{address}/api/v1/ws"))
                    .header(axum::http::header::CONNECTION, "Upgrade")
                    .header(axum::http::header::UPGRADE, "websocket")
                    .header(axum::http::header::SEC_WEBSOCKET_VERSION, "13")
                    .header(
                        axum::http::header::SEC_WEBSOCKET_KEY,
                        "dGhlIHNhbXBsZSBub25jZQ==",
                    );
                request = match credential {
                    None => request,
                    Some(value) if protocol => request.header(
                        axum::http::header::SEC_WEBSOCKET_PROTOCOL,
                        format!("bearer, {value}"),
                    ),
                    Some(value) => {
                        request.header(axum::http::header::AUTHORIZATION, format!("Bearer {value}"))
                    }
                };
                let response = request.send().await.expect("real loopback handshake");
                let status = response.status();
                let body = if status == StatusCode::SWITCHING_PROTOCOLS {
                    Vec::new()
                } else {
                    response.bytes().await.unwrap().to_vec()
                };
                handshakes.push((status, body));
            }
            server.0.abort();
            tokio::time::timeout(std::time::Duration::from_secs(5), &mut server.0)
                .await
                .expect("owned loopback server must stop")
                .ok();

            for (status, body) in handshakes.iter().take(3) {
                assert_eq!(*status, StatusCode::UNAUTHORIZED);
                let problem: Value = serde_json::from_slice(body).unwrap();
                assert_eq!(problem["error"]["code"], "unauthorized");
            }
            assert_eq!(
                [ssr.status(), handshakes[3].0, handshakes[4].0],
                [StatusCode::SERVICE_UNAVAILABLE; 3],
                "SSR and both real WebSocket credential forms must all be fenced",
            );
            let ssr_body = to_bytes(ssr.into_body(), 256 * 1024).await.unwrap();
            assert!(
                String::from_utf8_lossy(&ssr_body).contains("legacy_provenance_unavailable"),
                "SSR must show an accountable bridge interruption",
            );
            for (status, body) in handshakes.into_iter().skip(3) {
                assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
                let problem: Value = serde_json::from_slice(&body).unwrap();
                assert_eq!(problem["error"]["code"], "legacy_provenance_unavailable");
            }
            assert_eq!(
                all_rows(&pool).await,
                before,
                "fenced transport changed durable state"
            );
        })
        .catch_unwind()
        .await;
        close_states(&[state], outcome).await;
    }
}

mod native_org_provenance_contract_red {
    use super::*;

    // The same full ABI/ACL oracle checks both declared routine signatures.
    async fn routine_contract(runtime: &PgPool, signature: &str) -> Value {
        sqlx::query_scalar(
                "SELECT jsonb_build_object( \
                    'schema',n.nspname,'name',p.proname,'arguments',pg_get_function_identity_arguments(p.oid), \
                    'result',pg_get_function_result(p.oid),'owner',pg_get_userbyid(p.proowner),'language',l.lanname, \
                    'kind',p.prokind,'security_definer',p.prosecdef,'strict',p.proisstrict,'returns_set',p.proretset, \
                    'leakproof',p.proleakproof,'volatility',p.provolatile,'parallel',p.proparallel, \
                    'config',p.proconfig,'argnames',p.proargnames, \
                    'plain_arguments',p.provariadic=0 AND p.pronargdefaults=0 AND p.proargdefaults IS NULL AND p.proargmodes IS NULL, \
                    'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor), \
                        CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable) \
                        ORDER BY pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type) \
                        FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a),'[]'::jsonb)) \
                 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace JOIN pg_language l ON l.oid=p.prolang \
                 WHERE p.oid=to_regprocedure($1)",
        )
        .bind(signature)
        .fetch_one(runtime)
        .await
        .unwrap()
    }

    #[sqlx::test(migrations = false)]
    async fn positive_company_provenance_requires_exact_source_contract(pool: PgPool) {
        let (app, key, state) =
            native_people_directory_finalizer_tests::native_people_directory_row_lock_tests::configured_row_lock_native_directory_fixture(&pool).await;
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let auth = login_test_pool(&pool, TestDatabaseLogin::Auth).await;
        let outcome = AssertUnwindSafe(async {
            assert_eq!(ready_status(&state).await, StatusCode::OK);
            let role: (String, bool, bool) = sqlx::query_as(
                "SELECT current_user::text, rolsuper, rolbypassrls FROM pg_roles WHERE rolname=current_user",
            )
            .fetch_one(&runtime)
            .await
            .unwrap();
            assert_eq!(role, ("console_rt".into(), false, false));
            assert_eq!(
                classified(
                    &runtime,
                    include_str!("../../../../ops/postgres-native-people-directory-row-lock-custody-state.sql"),
                )
                .await,
                "native_people_directory.finalized",
            );

            // Genuine enrollment proves its receipt, topology, root authority,
            // actor and catalog prerequisites through the existing owner oracle.
            let (app, cookies, created) = create_owned_company(&pool, app).await;
            let legacy = *OrgId::knl().as_uuid();
            assert_ne!(created.company, legacy);
            let legacy_source: bool = sqlx::query_scalar(
                "SELECT o.origin_account_id IS NULL AND o.origin_command_id IS NULL \
                    AND o.origin_receipt_id IS NULL AND g.origin_account_id IS NULL \
                    AND g.origin_command_id IS NULL AND g.origin_receipt_id IS NULL \
                    AND h.revision>0 AND h.incarnation<>'00000000-0000-0000-0000-000000000000'::uuid \
                    AND r.state='ACTIVE' AND r.to_time IS NULL AND r.provenance_kind='LEGACY_BACKFILL' \
                    AND r.native_account_id IS NULL AND r.legacy_actor_user_id IS NULL \
                    AND r.force_actor_user_id IS NULL AND r.command_id IS NULL AND r.command_receipt IS NULL \
                    AND NOT EXISTS(SELECT 1 FROM public.company_enrollment_receipts x WHERE x.org_id=o.id OR x.group_id=g.id) \
                    AND NOT EXISTS(SELECT 1 FROM public.company_enrollment_effect_bindings x WHERE x.org_id=o.id OR x.group_id=g.id) \
                    AND NOT EXISTS(SELECT 1 FROM public.company_authority_heads x JOIN public.organizations n ON n.id=x.org_id WHERE x.org_id=o.id OR n.group_id=g.id) \
                    AND NOT EXISTS(SELECT 1 FROM public.company_actors x JOIN public.organizations n ON n.id=x.org_id WHERE x.org_id=o.id OR n.group_id=g.id) \
                    AND NOT EXISTS(SELECT 1 FROM public.native_company_catalog_installs x JOIN public.organizations n ON n.id=x.org_id WHERE x.org_id=o.id OR n.group_id=g.id) \
                 FROM public.organizations o JOIN public.groups g ON g.id=o.group_id \
                 JOIN public.group_authority_heads h ON h.group_id=g.id \
                 JOIN public.group_memberships m ON m.org_id=o.id AND m.group_id=g.id \
                 JOIN public.group_membership_revisions r ON (r.group_id,r.org_id,r.membership_id,r.revision,r.incarnation) \
                    =(m.group_id,m.org_id,m.membership_id,m.current_revision,m.incarnation) WHERE o.id=$1",
            )
            .bind(legacy)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert!(legacy_source, "positive legacy source prerequisite");
            let missing = Uuid::new_v4();
            let missing_source: bool = sqlx::query_scalar(
                "SELECT NOT EXISTS(SELECT 1 FROM public.organizations WHERE id=$1)",
            )
            .bind(missing)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert!(missing_source);
            // Exercise real policy work before provenance installation; origin is
            // historical evidence, independent of the current policy head.
            let config = account_browser_config(&pool, app._artifacts.root.clone(), &key);
            let (verifier, issuer, ttl) = bindings(&config);
            let store = PgOrgStore::new(runtime.clone()).with_native_account_policy(verifier, issuer, ttl);
            let policy = CompanyPolicy::new().unwrap();
            let install = NativeCompanyBusinessCommandV1::install(
                Uuid::new_v4(), OrgId::from_uuid(created.company), 1,
            ).unwrap();
            let form = native_policy_form(&store, &policy, &read_credentials(&cookies),
                NativePolicyCommandRef::from_command(&install)).await.unwrap();
            let credentials = AccountEnrollmentCredentials::for_mutation(
                &cookies.0[ACCESS], form.proof.as_str(),
            ).unwrap();
            let installed = submit_native_policy_command(&store, &policy, &credentials,
                &install, &TraceContext::generate()).await.unwrap();
            assert!(matches!(installed.terminal.outcome,
                NativePolicyOutcome::Committed(
                    console_identity_application::company_policy::workflow::NativePolicyEffect::Installed { .. }
                )));
            let epoch: i64 = sqlx::query_scalar(
                "SELECT epoch FROM public.company_authority_heads WHERE org_id=$1",
            ).bind(created.company).fetch_one(&pool).await.unwrap();
            assert_eq!(epoch, 2, "genuine policy install did not advance the Company head");
            let before = all_rows(&pool).await;

            // Install only the exact committed UNINSTALLED resource in this
            // disposable database. This is not a production finalizer/profile.
            const SOURCE: &str = include_str!("../../../../ops/postgres-company-provenance-v1-owner.sql");
            assert_eq!(hex::encode(Sha256::digest(SOURCE.as_bytes())),
                "e813293ace00c46358a0c1c62093e0911aae4da1b93c4dd1577dcb351c5398c2");
            let mut setup = pool.begin().await.unwrap();
            sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='30s'")
                .execute(setup.as_mut()).await.unwrap();
            let setup_identity: bool = sqlx::query_scalar(
                "SELECT session_user=current_user AND current_user='console_buck_admin' \
                   AND starts_with(current_database(),'_sqlx_test_') \
                   AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' \
                   AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user)",
            ).fetch_one(setup.as_mut()).await.unwrap();
            assert!(setup_identity, "declared source installer is not the marked disposable owner");
            sqlx::raw_sql(SOURCE).execute(setup.as_mut()).await.expect("declared source installation prerequisite");
            setup.commit().await.unwrap();
            assert!(before == all_rows(&pool).await, "source DDL changed durable business rows");

            // Preserve the original exact public source-presence oracle.
            let source: Option<i64> = sqlx::query_scalar(
                "SELECT to_regprocedure('public.account_company_provenance_v1(uuid)')::oid::bigint",
            )
            .fetch_one(&runtime)
            .await
            .unwrap();
            assert!(
                source.is_some(),
                "ORG_PROVENANCE_CONTRACT: exact public.account_company_provenance_v1(uuid) source is absent",
            );
            let contract = routine_contract(&runtime, "public.account_company_provenance_v1(uuid)").await;
            assert_eq!(
                contract,
                json!({
                    "schema":"public", "name":"account_company_provenance_v1", "arguments":"p_company uuid",
                    "result":"text", "owner":"console_account_owner", "language":"plpgsql", "kind":"f",
                    "security_definer":true, "strict":false, "returns_set":false, "leakproof":false,
                    "volatility":"v", "parallel":"u", "config":["search_path=pg_catalog, pg_temp","row_security=on"],
                    "argnames":["p_company"], "plain_arguments":true,
                    "acl":[["console_account_owner","console_account_owner","EXECUTE",false],
                           ["console_account_owner","console_rt","EXECUTE",false]],
                }),
            );
            assert_eq!(
                routine_contract(&runtime, "public.account_company_provenance_lock_v1(uuid,uuid)").await,
                json!({
                    "schema":"public", "name":"account_company_provenance_lock_v1",
                    "arguments":"p_company uuid, p_group uuid", "result":"boolean",
                    "owner":"console_app", "language":"plpgsql", "kind":"f",
                    "security_definer":true, "strict":false, "returns_set":false,
                    "leakproof":false, "volatility":"v", "parallel":"u",
                    "config":["search_path=pg_catalog, pg_temp","row_security=on"],
                    "argnames":["p_company","p_group"], "plain_arguments":true,
                    "acl":[["console_app","console_account_owner","EXECUTE",false],
                           ["console_app","console_app","EXECUTE",false]],
                }),
            );
            let exact_namespace: Vec<String> = sqlx::query_scalar(
                "SELECT p.oid::regprocedure::text FROM pg_catalog.pg_proc p \
                 JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace \
                 WHERE n.nspname='public' AND p.proname IN \
                    ('account_company_provenance_v1','account_company_provenance_lock_v1') \
                 ORDER BY p.oid::regprocedure::text",
            ).fetch_all(&runtime).await.unwrap();
            assert_eq!(exact_namespace, vec![
                "account_company_provenance_lock_v1(uuid,uuid)",
                "account_company_provenance_v1(uuid)",
            ]);
            for denied in [&runtime, &auth] {
                let error = sqlx::query("SELECT public.account_company_provenance_lock_v1($1,$2)")
                    .bind(created.company).bind(created.group).execute(denied).await.unwrap_err();
                match error {
                    sqlx::Error::Database(error) => {
                        assert_eq!(error.code().as_deref(), Some("42501"));
                        assert!(error.message().contains("account_company_provenance_lock_v1"));
                    }
                    error => panic!("helper denial was an unrelated error: {error}"),
                }
            }
            let unchanged_privileges: bool = sqlx::query_scalar(
                "SELECT NOT has_any_column_privilege('console_account_owner','public.organizations','UPDATE') \
                    AND NOT has_any_column_privilege('console_account_owner','public.groups','UPDATE') \
                    AND NOT has_any_column_privilege('console_account_owner','public.group_memberships','UPDATE') \
                    AND NOT has_any_column_privilege('console_account_owner','public.group_membership_revisions','UPDATE') \
                    AND NOT has_any_column_privilege('console_rt','public.company_enrollment_receipts','SELECT') \
                    AND NOT has_any_column_privilege('console_rt','public.company_enrollment_effect_bindings','SELECT') \
                    AND NOT has_any_column_privilege('console_rt','public.company_authority_heads','SELECT') \
                    AND NOT has_any_column_privilege('console_rt','public.company_actors','SELECT') \
                    AND NOT has_any_column_privilege('console_rt','public.native_company_catalog_installs','SELECT') \
                    AND NOT has_function_privilege('console_auth_rt','public.account_company_provenance_v1(uuid)','EXECUTE') \
                    AND NOT has_function_privilege('console_auth_startup','public.account_company_provenance_v1(uuid)','EXECUTE')",
            )
            .fetch_one(&runtime)
            .await
            .unwrap();
            assert!(unchanged_privileges, "classifier widened raw or Auth privileges");

            let cases = [
                ("native", Some(created.company), "NATIVE"),
                ("legacy", Some(legacy), "LEGACY"),
                ("null", None, "UNKNOWN"),
                ("nil", Some(Uuid::nil()), "UNKNOWN"),
                ("missing", Some(missing), "UNKNOWN"),
            ];
            let mut executed = 0;
            for (label, company, expected) in cases {
                let mut tx = runtime.begin().await.unwrap();
                sqlx::query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
                    .execute(tx.as_mut())
                    .await
                    .unwrap();
                sqlx::query("SELECT set_config('app.current_org',$1,true)")
                    .bind(legacy.to_string())
                    .execute(tx.as_mut())
                    .await
                    .unwrap();
                let result: Option<String> = sqlx::query_scalar(
                    "SELECT public.account_company_provenance_v1($1::uuid)",
                )
                .bind(company)
                .fetch_one(tx.as_mut())
                .await
                .unwrap();
                assert_eq!(result.as_deref(), Some(expected), "{label}");
                let context: String =
                    sqlx::query_scalar("SELECT current_setting('app.current_org')")
                        .fetch_one(tx.as_mut())
                        .await
                        .unwrap();
                assert_eq!(context, legacy.to_string(), "{label}: source leaked RLS context");
                tx.commit().await.unwrap();
                executed += 1;
            }
            assert_eq!(executed, 5);
            for isolation in ["REPEATABLE READ", "SERIALIZABLE"] {
                let mut tx = runtime.begin().await.unwrap();
                sqlx::raw_sql(sqlx::AssertSqlSafe(format!("SET TRANSACTION ISOLATION LEVEL {isolation}")))
                    .execute(tx.as_mut()).await.unwrap();
                sqlx::query("SELECT set_config('app.current_org',$1,true)")
                    .bind(legacy.to_string()).execute(tx.as_mut()).await.unwrap();
                let value: String = sqlx::query_scalar("SELECT public.account_company_provenance_v1($1)")
                    .bind(created.company).fetch_one(tx.as_mut()).await.unwrap();
                assert_eq!(value, "UNKNOWN", "unsupported isolation did not fail closed");
                let context: String = sqlx::query_scalar("SELECT current_setting('app.current_org')")
                    .fetch_one(tx.as_mut()).await.unwrap();
                assert_eq!(context, legacy.to_string());
                tx.commit().await.unwrap();
            }
            for (company, expected) in [(created.company,"NATIVE"),(legacy,"LEGACY")] {
                let mut tx = runtime.begin().await.unwrap();
                sqlx::query("SELECT set_config('app.current_org','',true)")
                    .execute(tx.as_mut()).await.unwrap();
                let value: String = sqlx::query_scalar("SELECT public.account_company_provenance_v1($1)")
                    .bind(company).fetch_one(tx.as_mut()).await.unwrap();
                assert_eq!(value, expected);
                let context: String = sqlx::query_scalar("SELECT current_setting('app.current_org')")
                    .fetch_one(tx.as_mut()).await.unwrap();
                assert_eq!(context, "");
                tx.commit().await.unwrap();
            }
            // Deterministic unknown-ID corpus is a bounded generated-input check;
            // it does not claim coverage-guided fuzzing or the full browser suite.
            let mut corpus = runtime.begin().await.unwrap();
            sqlx::query("SELECT set_config('app.current_org',$1,true)")
                .bind(legacy.to_string()).execute(corpus.as_mut()).await.unwrap();
            for index in 0..64 {
                let digest = Sha256::digest(format!("company-provenance-unknown-id-v1/{index}").as_bytes());
                let unknown = Uuid::from_slice(&digest[..16]).unwrap();
                let absent: bool = sqlx::query_scalar(
                    "SELECT NOT EXISTS(SELECT 1 FROM public.organizations WHERE id=$1)",
                ).bind(unknown).fetch_one(&pool).await.unwrap();
                assert!(absent, "generated case is not an unknown Company");
                let value: String = sqlx::query_scalar("SELECT public.account_company_provenance_v1($1)")
                    .bind(unknown).fetch_one(corpus.as_mut()).await.unwrap();
                assert_eq!(value, "UNKNOWN", "generated case {index}");
            }
            let context: String = sqlx::query_scalar("SELECT current_setting('app.current_org')")
                .fetch_one(corpus.as_mut()).await.unwrap();
            assert_eq!(context, legacy.to_string());
            corpus.commit().await.unwrap();

            let lock_queries = [
                ("Company", "SELECT 1 FROM public.organizations WHERE id=$1 FOR UPDATE NOWAIT", created.company),
                ("Group", "SELECT 1 FROM public.groups WHERE id=$1 FOR UPDATE NOWAIT", created.group),
                ("Group head", "SELECT 1 FROM public.group_authority_heads WHERE group_id=$1 FOR UPDATE NOWAIT", created.group),
                ("membership", "SELECT 1 FROM public.group_memberships WHERE org_id=$1 FOR UPDATE NOWAIT", created.company),
                ("history", "SELECT 1 FROM public.group_membership_revisions h JOIN public.group_memberships m ON (h.group_id,h.org_id,h.membership_id,h.revision,h.incarnation)=(m.group_id,m.org_id,m.membership_id,m.current_revision,m.incarnation) WHERE m.org_id=$1 FOR UPDATE OF h NOWAIT", created.company),
            ];
            let mut holder = runtime.begin().await.unwrap();
            let held: String = sqlx::query_scalar("SELECT public.account_company_provenance_v1($1)")
                .bind(created.company).fetch_one(holder.as_mut()).await.unwrap();
            assert_eq!(held, "NATIVE");
            for (label, query, id) in lock_queries {
                let mut contender = pool.begin().await.unwrap();
                let error = sqlx::query(sqlx::AssertSqlSafe(query)).bind(id)
                    .fetch_one(contender.as_mut()).await.unwrap_err();
                match error {
                    sqlx::Error::Database(error) => assert_eq!(error.code().as_deref(),Some("55P03"),"{label}"),
                    error => panic!("{label}: unrelated lock-probe failure: {error}"),
                }
                contender.rollback().await.unwrap();
            }
            holder.commit().await.unwrap();
            for (label, query, id) in lock_queries {
                let mut contender = pool.begin().await.unwrap();
                sqlx::query(sqlx::AssertSqlSafe(query)).bind(id)
                    .fetch_one(contender.as_mut()).await.unwrap_or_else(|error| panic!("{label}: committed classifier retained lock: {error}"));
                contender.rollback().await.unwrap();
            }
            assert!(before == all_rows(&pool).await, "classification changed durable rows");
            drop(app);
        })
        .catch_unwind()
        .await;
        runtime.close().await;
        auth.close().await;
        close_states(&[state], outcome).await;
    }
}

// Organization V12 source-transition prerequisite. This exercises actual
// owners and mounted HTTP documents, not the browser acceptance journey.
mod native_org_bridge_readiness_red {
    use super::*;
    use console_identity_application::company_policy::{
        AccountId,
        people_business::{DirectoryActionV1, NativePeoplePolicyCommandV1},
        workflow::{NativePolicyCommand, NativePolicyEffect},
    };

    fn company_document(page: &Response, company: Uuid, name: &str) {
        assert_eq!(page.status, StatusCode::OK);
        page.private();
        let html = std::str::from_utf8(&page.bytes).unwrap();
        assert!(html.contains(&format!("data-company-id=\"{company}\"")));
        assert!(
            html.contains(&format!("<h1>{name}</h1>")),
            "owner-projected Company name absent"
        );
    }

    async fn install_declared_source(pool: &PgPool) {
        const SOURCE: &str =
            include_str!("../../../../ops/postgres-company-provenance-v1-owner.sql");
        assert_eq!(
            hex::encode(Sha256::digest(SOURCE.as_bytes())),
            "e813293ace00c46358a0c1c62093e0911aae4da1b93c4dd1577dcb351c5398c2"
        );
        let mut setup = pool.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='30s'")
            .execute(setup.as_mut()).await.unwrap();
        let marked: bool = sqlx::query_scalar(
            "SELECT session_user=current_user AND current_user='console_buck_admin' \
               AND starts_with(current_database(),'_sqlx_test_') \
               AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' \
               AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user)",
        )
        .fetch_one(setup.as_mut())
        .await
        .unwrap();
        assert!(
            marked,
            "source installation requires marked disposable admin"
        );
        sqlx::raw_sql(SOURCE).execute(setup.as_mut()).await.unwrap();
        setup.commit().await.unwrap();
    }

    async fn provenance(runtime: &PgPool, company: Uuid) -> String {
        let mut tx = runtime.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='5s'")
            .execute(tx.as_mut()).await.unwrap();
        let result = sqlx::query_scalar("SELECT public.account_company_provenance_v1($1)")
            .bind(company)
            .fetch_one(tx.as_mut())
            .await
            .unwrap();
        tx.commit().await.unwrap();
        result
    }

    #[sqlx::test(migrations = false)]
    async fn declared_provenance_keeps_held_fresh_readiness_and_native_company_policy_work(
        pool: PgPool,
    ) {
        let (app, key, state) =
            native_people_directory_finalizer_tests::native_people_directory_row_lock_tests::configured_row_lock_native_directory_fixture(&pool).await;
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let mut states = vec![state.clone()];
        let outcome = AssertUnwindSafe(async {
            assert_eq!(ready_status(&state).await, StatusCode::OK);
            let (mut app, cookies, created) = create_owned_company(&pool, app).await;
            let config = account_browser_config(&pool, app._artifacts.root.clone(), &key);
            let company_path = format!("/companies/{}", created.company);
            let company_name: String = sqlx::query_scalar("SELECT name FROM public.organizations WHERE id=$1")
                .bind(created.company).fetch_one(&pool).await.unwrap();
            let before_document = document(&app, &company_path, &cookies).await;
            company_document(&before_document, created.company, &company_name);
            let before = all_rows(&pool).await;
            install_declared_source(&pool).await;
            assert!(before == all_rows(&pool).await, "source DDL changed durable rows");
            assert_eq!(provenance(&runtime, *OrgId::knl().as_uuid()).await, "LEGACY");
            assert_eq!(provenance(&runtime, created.company).await, "NATIVE");
            assert_eq!(provenance(&runtime, Uuid::nil()).await, "UNKNOWN");
            assert_eq!(
                classified(&runtime, include_str!("../../../../ops/postgres-native-people-directory-row-lock-custody-state.sql")).await,
                "native_people_directory.profile_mismatch",
                "historical classifier must not be resealed for added routines",
            );
            // Intended RED follows genuine source/fixture prerequisites.
            assert_eq!(ready_status(&state).await, StatusCode::OK,
                "ORG_BRIDGE_HELD_READINESS: exact declared source must preserve existing native work");
            let fresh = AppState::from_config(config.clone()).await
                .expect("exact source-installed bridge must compose fresh state");
            states.push(fresh.clone());
            assert_eq!(ready_status(&fresh).await, StatusCode::OK);
            for selected in [&state, &fresh] {
                app.service = build_router(selected.clone());
                let page = document(&app, &company_path, &cookies).await;
                company_document(&page, created.company, &company_name);
                for path in [format!("/companies/{}/organization", created.company),
                             format!("/companies/{}/organization/new", created.company)] {
                    let closed = document(&app, &path, &cookies).await;
                    assert!(matches!(closed.status, StatusCode::NOT_FOUND | StatusCode::SERVICE_UNAVAILABLE),
                        "classifier-installed bridge cannot activate native OrgUnit work");
                    assert!(!String::from_utf8(closed.bytes).unwrap().contains("name=\"csrf_proof\""));
                }
            }
            assert!(before == all_rows(&pool).await, "readiness/documents changed durable rows");
            let (verifier, issuer, ttl) = bindings(&config);
            let store = PgOrgStore::new(runtime.clone()).with_native_account_policy(verifier, issuer, ttl);
            let policy = CompanyPolicy::new().unwrap();
            let company = OrgId::from_uuid(created.company);
            let commands = [
                (NativePolicyCommand::Payroll(NativeCompanyBusinessCommandV1::install(Uuid::new_v4(), company, 1).unwrap()),
                 "native-payroll-collection-read-v1", 1_i16, 1_i64,
                 "07781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd"),
                (NativePolicyCommand::People(NativePeoplePolicyCommandV1::install(Uuid::new_v4(), company, 2).unwrap()),
                 "native-people-directory-v1", 2_i16, 2_i64,
                 "591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e"),
            ];
            for (command, catalog, codec, epoch, manifest) in commands {
                let locator = NativePolicyCommandRef::from_command(&command);
                let command_id = locator.command_id();
                let form = native_policy_form(&store, &policy, &read_credentials(&cookies),
                    locator).await.unwrap();
                let credentials = AccountEnrollmentCredentials::for_mutation(
                    &cookies.0[ACCESS], form.proof.as_str()).unwrap();
                let result = submit_native_policy_command(&store, &policy, &credentials,
                    &command, &TraceContext::generate()).await.unwrap();
                let object_type_id = match result.terminal.outcome {
                    NativePolicyOutcome::Committed(NativePolicyEffect::Installed { object_type_id }) => object_type_id,
                    _ => panic!("real policy install did not commit its object type"),
                };
                let witnesses: i64 = sqlx::query_scalar(
                    "SELECT count(*) FROM public.native_company_catalog_installs n \
                     JOIN public.native_company_policy_receipts_v1 r ON (r.org_id,r.receipt_id)=(n.org_id,n.policy_receipt_id) \
                     JOIN public.native_company_policy_inputs_v1 i ON \
                       (i.actor_account_id,i.command_id,i.org_id,i.operation,i.codec_version,i.intake_receipt_id,i.input_digest)= \
                       (r.actor_account_id,r.command_id,r.org_id,r.operation,r.codec_version,r.intake_receipt_id,r.input_digest) \
                     JOIN public.ont_builtin_catalog_installs b ON \
                       (b.org_id,b.catalog_version,b.manifest_digest)=(n.org_id,n.catalog_version,n.manifest_digest) \
                     JOIN public.native_company_object_refs o ON \
                       (o.org_id,o.catalog_version,o.manifest_digest,o.object_type_id)= \
                       (n.org_id,n.catalog_version,n.manifest_digest,r.installed_object_type_id) \
                     WHERE n.org_id=$1 AND r.actor_account_id=$2 AND r.command_id=$3 \
                       AND r.receipt_id=$4 AND r.installed_object_type_id=$5 AND r.operation=1 \
                       AND r.outcome='COMMITTED' AND r.codec_version=$6 AND r.epoch_before=$7 \
                       AND r.epoch_after=$7+1 AND n.catalog_version=$8 AND r.catalog_version=$8 \
                       AND n.manifest_digest=decode($9,'hex') AND r.manifest_digest=n.manifest_digest \
                       AND i.input_digest=sha256(i.input_bytes)",
                ).bind(created.company).bind(created.administrator).bind(command_id)
                    .bind(result.terminal.receipt_id).bind(object_type_id).bind(codec).bind(epoch)
                    .bind(catalog).bind(manifest).fetch_one(&pool).await.unwrap();
                assert_eq!(witnesses, 1, "missing exact catalog/input/receipt/object witness: {catalog}");
            }
            let epoch: i64 = sqlx::query_scalar("SELECT epoch FROM public.company_authority_heads WHERE org_id=$1")
                .bind(created.company).fetch_one(&pool).await.unwrap();
            assert_eq!(epoch, 3, "both real catalog owner effects are required");
            let now: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                .fetch_one(&runtime).await.unwrap();
            let until = time::OffsetDateTime::from_unix_timestamp((now.unix_timestamp()/60+1440)*60).unwrap();
            let recipient = AccountId::from_uuid(created.administrator).unwrap();
            let grant = NativePolicyCommand::People(NativePeoplePolicyCommandV1::grant(
                Uuid::new_v4(), company, 3, DirectoryActionV1::Read, recipient, None, until).unwrap());
            let form = native_policy_form(&store, &policy, &read_credentials(&cookies),
                NativePolicyCommandRef::from_command(&grant)).await.unwrap();
            let credentials = AccountEnrollmentCredentials::for_mutation(&cookies.0[ACCESS], form.proof.as_str()).unwrap();
            let granted = submit_native_policy_command(&store, &policy, &credentials, &grant,
                &TraceContext::generate()).await.unwrap();
            assert!(matches!(granted.terminal.outcome,
                NativePolicyOutcome::Committed(NativePolicyEffect::Granted { recipient: actual, .. }) if actual==recipient));
            for selected in [&state, &fresh] {
                app.service = build_router(selected.clone());
                company_document(&document(&app, &company_path, &cookies).await, created.company, &company_name);
                let directory = document(&app, &format!("/companies/{}/people", created.company), &cookies).await;
                assert_eq!(directory.status, StatusCode::OK);
                directory.private();
                let html = std::str::from_utf8(&directory.bytes).unwrap();
                assert!(html.contains("native-people-workspace") && html.contains("id=\"people-directory-heading\"")
                    && html.contains("표시할 사람이 없습니다") && html.contains(&company_name));
                assert!(!html.contains("data-people-record="));
                let policy_form = document(&app, &format!("/companies/{}/policy/people-directory/read/grant", created.company), &cookies).await;
                assert_eq!(policy_form.status, StatusCode::OK);
                policy_form.private();
                let html = std::str::from_utf8(&policy_form.bytes).unwrap();
                assert!(html.contains("name=\"csrf_proof\"") && html.contains("name=\"command_id\""));
            }
            let after_work = all_rows(&pool).await;
            for table in ["org_units", "org_unit_revisions", "org_unit_source_bindings",
                          "employment_heads", "employment_revisions", "payroll_draft_runs"] {
                assert!(before.contains_key(table), "required fixture relation absent: {table}");
                assert!(before.get(table) == after_work.get(table), "unrelated business fact changed: {table}");
            }
            assert_eq!(provenance(&runtime, created.company).await, "NATIVE");
            for selected in [&state, &fresh] {
                assert_eq!(ready_status(selected).await, StatusCode::OK);
            }
            // Committed ACL corruption must fence live and fresh serving. Restore
            // the exact prior routine catalog without recreating either function.
            let original_helper: String = sqlx::query_scalar(
                "SELECT to_jsonb(p)::text FROM pg_catalog.pg_proc p WHERE p.oid= \
                   'public.account_company_provenance_lock_v1(uuid,uuid)'::regprocedure",
            ).fetch_one(&pool).await.unwrap();
            sqlx::query("GRANT EXECUTE ON FUNCTION public.account_company_provenance_lock_v1(uuid,uuid) TO console_rt")
                .execute(&pool).await.unwrap();
            for selected in [&state, &fresh] {
                assert_eq!(ready_status(selected).await, StatusCode::SERVICE_UNAVAILABLE,
                    "unknown provenance helper ACL admitted live readiness");
            }
            match AppState::from_config(config.clone()).await {
                Err(AppError::Config(code)) => assert_eq!(code, "company_provenance.profile_mismatch"),
                Err(_) => panic!("unrelated startup error does not prove custody refusal"),
                Ok(untrusted) => {
                    states.push(untrusted);
                    panic!("unknown provenance helper ACL admitted fresh startup");
                }
            }
            sqlx::query("REVOKE EXECUTE ON FUNCTION public.account_company_provenance_lock_v1(uuid,uuid) FROM console_rt")
                .execute(&pool).await.unwrap();
            let restored_helper: String = sqlx::query_scalar(
                "SELECT to_jsonb(p)::text FROM pg_catalog.pg_proc p WHERE p.oid= \
                   'public.account_company_provenance_lock_v1(uuid,uuid)'::regprocedure",
            ).fetch_one(&pool).await.unwrap();
            assert_eq!(restored_helper, original_helper, "helper catalog restoration was inexact");
            assert!(after_work == all_rows(&pool).await, "corruption/restore changed durable business rows");
            for selected in [&state, &fresh] {
                assert_eq!(ready_status(selected).await, StatusCode::OK,
                    "exact restoration did not recover held serving");
            }
            let restored = AppState::from_config(config).await.unwrap();
            states.push(restored.clone());
            assert_eq!(ready_status(&restored).await, StatusCode::OK);
            drop(app);
        }).catch_unwind().await;
        runtime.close().await;
        close_states(&states, outcome).await;
    }
}
