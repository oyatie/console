//! Manager-current owner probe only. Full grant bytes are an unaccepted locator fixture.
use super::*;
use super::{manager_credentials as credentials, manager_policy::ObservedRead};
use console_identity_application::company_policy::{
    AccountId, CompanyPolicyDecision, CompanyPolicyDecisionPort, CompanyPolicyRequest,
    CompanyPolicyScope, CompanyPolicyStore, InitialCompanyAction,
    business::NativeCompanyBusinessCommandV1,
    company_information::{
        CompanyInformationAssignmentRefV1, MANIFEST, NativeCompanyInformationCommandV1,
        NativeCompanyInformationGrantV1,
    },
    workflow::{
        NativePolicyCommand, NativePolicyCommandRef, NativePolicyFormView,
        NativePolicyWorkflowError, native_policy_current,
    },
};
use console_kernel_core::OrgId;
use console_platform_authz::company_policy::CompanyPolicy;

#[path = "native_company_information_browser_source.rs"]
mod source;

#[path = "native_company_information_browser_histories.rs"]
mod histories;

#[path = "native_company_information_browser_expiry.rs"]
mod expiry;

#[path = "native_company_information_browser_retained_locks.rs"]
mod retained_locks;

#[path = "native_company_information_browser_cancellation.rs"]
mod cancellation;

#[path = "native_company_information_browser_commit_loss.rs"]
mod commit_loss;

pub(super) async fn probe(
    pool: &PgPool,
    config: &AppConfig,
    captured: &credentials::CapturedCookies,
    administrator: Uuid,
    recipient: Uuid,
    operator: Uuid,
    result: &Committed,
) {
    let runtime = credentials::runtime(pool).await;
    let outcome = std::panic::AssertUnwindSafe(async {
        let before = all_rows(pool).await;
        let a = credentials::credentials(&runtime, config, captured, administrator).await;
        let b = credentials::credentials(&runtime, config, captured, recipient).await;
        let o = credentials::credentials(&runtime, config, captured, operator).await;
        let store = credentials::store(runtime.clone(), config);
        let policy = CompanyPolicy::new().unwrap();
        let company = OrgId::from_uuid(result.company);
        let scope = store
            .lock_current(&a, company)
            .await
            .expect("existing A birth-current source prerequisite");
        let authority = scope
            .authority()
            .expect("actual distinct A manager birth source")
            .clone();
        assert_eq!(*authority.account().as_uuid(), administrator);
        assert_eq!(authority.company(), company);
        assert_eq!(authority.epoch(), 1);
        assert_eq!(authority.assignment_revision(), 1);
        assert_eq!(authority.role_revision(), 1);
        assert_eq!(authority.clauses().len(), 7);
        assert_eq!(
            authority
                .clauses()
                .iter()
                .map(|c| c.fields().len())
                .sum::<usize>(),
            16
        );
        assert_eq!(
            authority.name(),
            "브라우저로 연결한 독립 관리 회사 <연구 & 본사>"
        );
        assert_eq!(authority.slug(), format!("handoff-{}", operator.simple()));
        let read = authority
            .clauses()
            .iter()
            .find(|c| !c.delegable() && c.action_kind() == InitialCompanyAction::ReadPolicy)
            .unwrap();
        assert_eq!(read.fields().len(), 8);
        let request = CompanyPolicyRequest::new(
            company,
            read.action().object_type_id(),
            authority.assignment_id(),
            read.action().clone(),
            read.fields().to_vec(),
        )
        .unwrap();
        assert_eq!(
            policy.decide(&authority, &request).unwrap(),
            CompanyPolicyDecision::Allow,
            "real Cedar ReadPolicy positive prerequisite"
        );
        scope
            .finish()
            .await
            .expect("existing A birth-current scope prerequisite completion");
        let bootstrap =
            NativeCompanyBusinessCommandV1::install(Uuid::new_v4(), company, authority.epoch())
                .unwrap();
        let bootstrap_selector = NativePolicyCommandRef::from_command(&bootstrap);
        let original = native_policy_current(&store, &policy, &o, bootstrap_selector)
            .await
            .expect("actual O bootstrap Current positive prerequisite");
        assert_eq!(original.selector, bootstrap_selector);
        assert_eq!(original.group_id, result.group);
        assert_eq!(*original.acting_account_id.as_uuid(), operator);
        assert_eq!(*original.administrative_account_id.as_uuid(), administrator);
        let discover = authority
            .clauses()
            .iter()
            .find(|c| !c.delegable() && c.action_kind() == InitialCompanyAction::Discover)
            .unwrap();
        let identity = authority
            .clauses()
            .iter()
            .find(|c| !c.delegable() && c.action_kind() == InitialCompanyAction::ReadIdentity)
            .unwrap();
        assert_eq!(discover.fields(), identity.fields());
        assert_eq!(discover.action().manifest_digest(), &MANIFEST);
        assert_eq!(identity.action().manifest_digest(), &MANIFEST);
        let fields = discover
            .fields()
            .to_vec()
            .try_into()
            .expect("actual registered Company identity field pair");
        let now: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&runtime)
            .await
            .unwrap();
        assert_eq!(now.unix_timestamp_nanos() % 1000, 0);
        let parent = CompanyInformationAssignmentRefV1::new(
            authority.assignment_id(),
            authority.assignment_revision(),
        )
        .unwrap();
        let grant = NativeCompanyInformationGrantV1::new(
            AccountId::from_uuid(recipient).unwrap(),
            parent,
            now,
            now + time::Duration::hours(1),
            [discover.action().clone(), identity.action().clone()],
            fields,
        )
        .unwrap();
        let command = NativePolicyCommand::from(
            NativeCompanyInformationCommandV1::grant(
                Uuid::new_v4(),
                company,
                authority.epoch(),
                "회사 정보 접근 업무 문맥 확인".to_owned(),
                grant,
            )
            .unwrap(),
        );
        let encoded = command.encode(authority.account());
        let decoded = NativePolicyCommand::decode(4, &encoded)
            .expect("reviewed codec-four fixture roundtrip prerequisite");
        assert_eq!(decoded, (authority.account(), command.clone()));
        assert_eq!(decoded.1.encode(decoded.0), encoded);
        let selector = NativePolicyCommandRef::from_command(&command);
        assert_eq!(selector.codec_version(), 4);
        assert!(
            before == all_rows(pool).await,
            "prerequisite reads changed any public table"
        );
        source::prerequisite(
            pool,
            &runtime,
            config,
            [&a, &b, &o],
            &authority,
            selector,
            result.group,
            &before,
        )
        .await;
        let observed = ObservedRead::new(authority.clone(), None);
        let current = native_policy_current(&store, &observed, &a, selector).await;
        assert!(
            before == all_rows(pool).await,
            "manager Current changed any table including limiter/audit"
        );
        assert!(
            current.is_ok() || matches!(current, Err(NativePolicyWorkflowError::Unavailable)),
            "NON_ADMITTING_MANAGER_CURRENT_FAILURE"
        );
        assert!(
            current.is_ok(),
            "COMPANY_INFORMATION_MANAGER_CURRENT_UNAVAILABLE"
        );
        let expected = NativePolicyFormView {
            selector,
            group_id: result.group,
            company_epoch: authority.epoch(),
            acting_account_id: authority.account(),
            administrative_account_id: authority.account(),
            installed_object_type_id: Some(discover.action().object_type_id()),
            assignment: None,
        };
        assert_eq!(current.unwrap(), expected);
        observed.count(2);
        assert_eq!(
            native_policy_current(&store, &observed, &a, selector)
                .await
                .unwrap(),
            expected
        );
        observed.count(4);
        assert!(
            before == all_rows(pool).await,
            "manager reopening changed any table"
        );
        super::manager_controls::negatives(
            pool,
            &store,
            &authority,
            [&a, &b, &o],
            &command,
            &before,
        )
        .await;
        histories::verify(
            pool,
            &runtime,
            config,
            &a,
            &authority,
            selector,
            result.group,
            &expected,
            &before,
        )
        .await;
        expiry::verify(
            pool,
            &runtime,
            config,
            &a,
            &authority,
            selector,
            result.group,
            &expected,
            &before,
        )
        .await;
        retained_locks::verify(
            pool,
            &runtime,
            config,
            &a,
            &authority,
            selector,
            result.group,
            &expected,
            &before,
        )
        .await;
        cancellation::verify(
            pool, &runtime, config, &a, &authority, selector, &expected, &before,
        )
        .await;
        commit_loss::verify(
            pool, &runtime, config, &a, &authority, selector, &expected, &before,
        )
        .await;
    })
    .catch_unwind()
    .await;
    runtime.close().await;
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
}

pub(super) fn fixture_lease() -> std::fs::File {
    assert_eq!(
        std::env::var("CONSOLE_MANAGER_DEDICATED_CLUSTER").as_deref(),
        Ok("root-owned-disposable"),
        "STOP: root-owned dedicated cluster required"
    );
    let path = std::path::PathBuf::from(
        std::env::var_os("CONSOLE_MANAGER_MAINTENANCE_LEASE")
            .expect("STOP: immutable root probe must declare the owned cluster lease"),
    );
    assert!(
        path.is_absolute()
            && std::fs::symlink_metadata(&path)
                .unwrap()
                .file_type()
                .is_file()
    );
    let lease = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap();
    assert!(lease.metadata().unwrap().is_file());
    lease
        .try_lock()
        .expect("STOP: maintenance lease is occupied");
    lease
}

// Additive fixture only. The separate startup lane must first map this exact
// classifier to the finite profile. AppState's profile is private: preserve the
// application-owned actual_profile oracle and existing startup/readiness checks.
async fn fixture_classified(pool: &PgPool, source: &'static str) -> String {
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(tx.as_mut())
        .await
        .unwrap();
    sqlx::raw_sql(include_str!("../../src/account_custody_session.sql"))
        .execute(tx.as_mut())
        .await
        .unwrap();
    let state = sqlx::query_scalar(sqlx::AssertSqlSafe(source))
        .fetch_one(tx.as_mut())
        .await
        .expect("STOP: actual finite Manager classifier prerequisite failed");
    tx.commit()
        .await
        .expect("STOP: Manager classifier completion was not confirmed");
    state
}

pub(super) async fn prepare_successor_fixture(pool: &PgPool) {
    const FINALIZER: &str = include_str!(
        "../../../../ops/postgres-finalize-company-information-manager-current-policy-v1.sql"
    );
    const STATE: &str = include_str!(
        "../../../../ops/postgres-company-information-manager-current-policy-v1-custody-state.sql"
    );
    const APP_STATE: &str =
        include_str!("../../src/company_information_manager_current_policy_v1_custody_state.sql");
    for (source, expected) in [
        (
            FINALIZER,
            "4916eaf30459b7d28f17b4fbb4793f2159b82e5f9bf5b8102ec6b0abde9fc30d",
        ),
        (
            STATE,
            "2ad10750d99ded7ff02acd5ddb10214eefa4063d7d0a14c2ca83e1db9e263853",
        ),
        (
            APP_STATE,
            "2ad10750d99ded7ff02acd5ddb10214eefa4063d7d0a14c2ca83e1db9e263853",
        ),
    ] {
        assert_eq!(
            hex::encode(Sha256::digest(source.as_bytes())),
            expected,
            "STOP: committed finite Manager source prerequisite drift"
        );
    }
    assert_eq!(
        STATE, APP_STATE,
        "STOP: exact ops/app Manager classifier copies differ"
    );
    let before = all_rows(pool).await;
    let runtime = credentials::runtime(pool).await;
    let outcome = std::panic::AssertUnwindSafe(async {
        for classifier in [STATE, APP_STATE] {
            assert_eq!(fixture_classified(&runtime, classifier).await,
                       "company_information_manager_current_policy_v1.install_required",
                       "STOP: exact PolicyV1 Manager predecessor prerequisite failed");
        }
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL jit=off; SET LOCAL statement_timeout='60s'; SET LOCAL lock_timeout='1s'; SET LOCAL idle_in_transaction_session_timeout='30s'; SET LOCAL transaction_timeout='120s'")
            .execute(tx.as_mut()).await.unwrap();
        let marked: bool = sqlx::query_scalar("SELECT session_user=current_user AND current_user='console_buck_admin' AND starts_with(current_database(),'_sqlx_test_') AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user)")
            .fetch_one(tx.as_mut()).await.unwrap();
        assert!(marked, "STOP: genuine marked disposable operator required");
        let owners: Vec<String> = sqlx::query_scalar("SELECT oid::text FROM pg_catalog.pg_authid WHERE rolname='console_account_owner' FOR UPDATE")
            .fetch_all(tx.as_mut()).await.unwrap();
        assert_eq!(owners.len(), 1, "STOP: actual retained account-owner row lock required");
        assert!(owners[0].parse::<u32>().unwrap() > 0);
        // One existing production finalizer; no copied DDL or business population.
        sqlx::raw_sql(FINALIZER).execute(tx.as_mut()).await
            .expect("STOP: committed Manager production-finalizer prerequisite failed");
        sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE").execute(tx.as_mut()).await.unwrap();
        tx.commit().await.expect("STOP: Manager fixture COMMIT was not confirmed");
        for classifier in [STATE, APP_STATE] {
            assert_eq!(fixture_classified(&runtime, classifier).await,
                       "company_information_manager_current_policy_v1.finalized",
                       "STOP: installed Manager classifier prerequisite failed");
        }
        assert!(before == all_rows(pool).await,
                "STOP: finite Manager installation changed complete public-table census");
    }).catch_unwind().await;
    runtime.close().await;
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
}
