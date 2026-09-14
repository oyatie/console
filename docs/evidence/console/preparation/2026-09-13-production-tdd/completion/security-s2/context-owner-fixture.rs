//! Real populated topology/grants through ordinary owners. No CompanyActor,
//! candidate, grant, verified Human or result rows are inserted by this fixture.
use crate::{
    account_enrollment_producer::{AccountBootstrapPhase, enroll_fixture_accounts},
    operator_custody_producer::{CustodyFixture, Result},
    ordinary_issuer_bootstrap::{FixtureIssuer, enroll_authority_and_companies},
    publish_then_root_producer::publish_genesis_then_enroll_operator,
};
use console_app::{AppConfig, AppState, build_router};
use console_identity_adapter_postgres::account13 as identity;
use console_platform_auth::{
    PasskeyService, WebauthnSettings,
    account_session::{VerifiedAccountSession, verify_account_session},
};
use console_platform_provisioning::{PlatformProvisioner, account13 as provisioning};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

pub struct ContextFixture {
    pub router: axum::Router,
    pub serving: console_platform_request_context::account::ServingAdmissionBinding,
    pub verifier: console_platform_auth::JwtVerifier,
    pub accounts: AccountBootstrapPhase,
    pub operator: VerifiedAccountSession,
    pub owner_pool: PgPool,
    pub auth_pool: PgPool,
    pub companies: Vec<Uuid>,
    pub groups: Vec<Uuid>,
    pub expected: BTreeMap<String, Option<String>>,
    pub undiscoverable: String,
    pub label_denied: String,
    pub grants: Vec<identity::CompanyGrantReceipt>,
    pub custody: CustodyFixture,
}
pub fn context_key(value: &Value) -> Result<String> {
    let object = value.as_object().ok_or("context_ref object")?;
    if object.len() != 1 {
        return Err("context_ref closed shape".into());
    }
    for name in ["org_id", "group_id"] {
        if let Some(id) = object.get(name) {
            return Ok(format!(
                "{name}:{}",
                Uuid::parse_str(id.as_str().ok_or("uuid")?)?
            ));
        }
    }
    Err("unknown context kind".into())
}
async fn configured_login(admin: &PgPool, key: &str, expected: &str) -> Result<PgPool> {
    let options = admin.connect_options();
    let mut url = url::Url::parse(&std::env::var(key)?)?;
    assert_eq!(url.username(), expected);
    assert_eq!(url.host_str(), Some(options.get_host()));
    assert_eq!(url.port().unwrap_or(5432), options.get_port());
    assert!(url.query().is_none() && url.fragment().is_none() && url.password().is_some());
    url.set_path(options.get_database().ok_or("marked database missing")?);
    let pool = PgPool::connect(url.as_str()).await?;
    let actual: (String, String) = sqlx::query_as("SELECT session_user::text,current_user::text")
        .fetch_one(&pool)
        .await?;
    assert_eq!(actual, (expected.into(), expected.into()));
    Ok(pool)
}
impl ContextFixture {
    pub async fn create(
        admin: &PgPool,
        group_count: usize,
        companies_per_group: usize,
    ) -> Result<Self> {
        assert!(group_count > 0 && group_count <= 32);
        assert!(group_count * companies_per_group <= 256);
        super::prepare_http_database(admin).await;
        let custody = CustodyFixture::new(
            "navigation",
            admin.connect_options().get_database().ok_or("database")?,
        )?;
        let startup = configured_login(
            admin,
            "CONSOLE_STARTUP_AUTH_DATABASE_URL",
            "console_auth_startup",
        )
        .await?;
        let auth_pool = console_platform_test_support::login_test_pool(
            admin,
            console_platform_test_support::TestDatabaseLogin::Auth,
        )
        .await;
        let owner_pool = configured_login(
            admin,
            "CONSOLE_IDENTITY_COMMAND_DATABASE_URL",
            "console_identity_cmd",
        )
        .await?;
        let origin = url::Url::parse("https://auth.example.com")?;
        let service = PasskeyService::new(WebauthnSettings {
            rp_id: "example.com".into(),
            rp_origin: origin.clone(),
            rp_name: "TEST_ONLY".into(),
            extra_allowed_origins: vec![],
            ceremony_ttl: time::Duration::minutes(5),
        })?;
        let root_secret = hex::encode(rand::random::<[u8; 32]>());
        let verifier = console_platform_auth::JwtVerifier::from_es256_public_pem(
            console_platform_auth::JwtSettings {
                issuer: std::env::var("CONSOLE_JWT_ISSUER")?,
                audience: std::env::var("CONSOLE_JWT_AUDIENCE")?,
                access_token_ttl: time::Duration::minutes(15),
            },
            std::env::var("CONSOLE_JWT_PUBLIC_KEY_PEM")?.as_bytes(),
        )?;
        let operator = publish_genesis_then_enroll_operator(
            &custody,
            &startup,
            &service,
            &verifier,
            &origin,
            &root_secret,
        )
        .await?;
        let accounts = enroll_fixture_accounts(&auth_pool, &service, &verifier, &origin).await?;
        // Company/provider synthetic signature is a different key from the
        // Account JWT and publisher keys. Real owners validate it after ordinary
        // provider registration by the verified current deployment operator.
        use p256::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
        let key = p256::ecdsa::SigningKey::random(&mut p256::elliptic_curve::rand_core::OsRng);
        let issuer = FixtureIssuer::from_config(
            "https://TEST_ONLY.invalid/identity".into(),
            "test.identity".into(),
            key.verifying_key()
                .to_public_key_pem(LineEnding::LF)?
                .into_bytes(),
            key.to_pkcs8_pem(LineEnding::LF)?.as_bytes(),
        )?;
        let mut companies = Vec::new();
        let mut groups = Vec::new();
        let mut expected = BTreeMap::new();
        let mut grants = Vec::new();
        for group_index in 0..group_count {
            let names = (0..companies_per_group)
                .map(|n| format!("visible-{group_index:02}-{n:02}"))
                .collect::<Vec<_>>();
            // No grants can be prebound to not-yet-created Company IDs. Empty
            // grant choices here; below resolve actual current schema per Company.
            let created = enroll_authority_and_companies(
                &owner_pool,
                &operator,
                &accounts,
                &issuer,
                &names,
                &[],
            )
            .await?;
            groups.push(created.group.group_id);
            let group_label = format!("visible-group-{group_index:02}");
            let provisioner = PlatformProvisioner::new(time::Duration::minutes(5));
            let control = provisioner
                .read_account_group_control(&owner_pool, &operator, created.group.group_id)
                .await?;
            provisioner
                .update_account_group(
                    &owner_pool,
                    &operator,
                    provisioning::GroupUpdate {
                        command_id: Uuid::new_v4(),
                        group_id: created.group.group_id,
                        expected: control,
                        name: group_label.clone(),
                    },
                )
                .await?;

            let group_schema =
                identity::read_group_policy_schema(&owner_pool, &operator, created.group.group_id)
                    .await?;
            let group_expected =
                identity::read_group_policy_control(&owner_pool, &operator, created.group.group_id)
                    .await?;
            identity::apply_group_grant_plan(
                &owner_pool,
                &operator,
                identity::ApplyGroupGrantPlan {
                    command_id: Uuid::new_v4(),
                    group_id: created.group.group_id,
                    expected: group_expected,
                    plan: identity::GroupGrantPlan {
                        account_id: accounts.submitter.account_id,
                        scope: identity::PolicyScope::GroupControl,
                        actions: vec![group_schema.action("context.discover")?],
                        field_projection: group_schema.projection("context.identity_and_label")?,
                        valid_from: None,
                        valid_to: None,
                        reason: "TEST_ONLY populated discovery".into(),
                    },
                },
            )
            .await?;
            expected.insert(
                format!("group_id:{}", created.group.group_id),
                Some(group_label),
            );
            for (index, company) in created.companies.into_iter().enumerate() {
                let hidden = group_index + 1 == group_count && index + 1 == companies_per_group;
                let label_denied = group_index == 0 && index == 0;
                companies.push(company.org_id);
                // Create actual denied candidate too: the explicit discovery
                // source remains but its current policy forbids disclosure.
                let schema =
                    identity::read_company_policy_schema(&owner_pool, &operator, company.org_id)
                        .await?;
                let current =
                    identity::read_company_policy_control(&owner_pool, &operator, company.org_id)
                        .await?;
                let grant = identity::apply_company_grant_plan(
                    &owner_pool,
                    &operator,
                    identity::ApplyCompanyGrantPlan {
                        command_id: Uuid::new_v4(),
                        org_id: company.org_id,
                        expected: current,
                        plan: identity::CompanyGrantPlan {
                            account_id: accounts.submitter.account_id,
                            scope: identity::PolicyScope::Company,
                            actions: vec![schema.action("context.discover")?],
                            field_projection: schema.projection(if label_denied {
                                "context.identity_only"
                            } else {
                                "context.identity_and_label"
                            })?,
                            valid_from: None,
                            valid_to: None,
                            reason: "TEST_ONLY populated discovery".into(),
                        },
                    },
                )
                .await?;
                if hidden {
                    let current = identity::read_company_policy_control(
                        &owner_pool,
                        &operator,
                        company.org_id,
                    )
                    .await?;
                    identity::revoke_company_grant(
                        &owner_pool,
                        &operator,
                        identity::RevokeCompanyGrant {
                            command_id: Uuid::new_v4(),
                            org_id: company.org_id,
                            grant_id: grant.grant_id,
                            expected: current,
                            reason: "TEST_ONLY undiscoverable".into(),
                        },
                    )
                    .await?;
                } else {
                    expected.insert(
                        format!("org_id:{}", company.org_id),
                        if label_denied {
                            None
                        } else {
                            Some(names[index].clone())
                        },
                    );
                }
                grants.push(grant);
            }
        }
        let undiscoverable = format!("org_id:{}", companies.last().ok_or("company fixture")?);
        let label_denied = format!("org_id:{}", companies[0]);
        // Exact live source readback; receipt alone cannot prove fixture rows.
        let actual:Vec<(Uuid,Uuid)>=sqlx::query_as("SELECT group_id,org_id FROM group_memberships WHERE org_id=ANY($1) ORDER BY group_id,org_id").bind(&companies).fetch_all(admin).await?;
        assert_eq!(actual.len(), companies.len());
        assert_eq!(
            actual.iter().map(|(_, c)| *c).collect::<BTreeSet<_>>(),
            companies.iter().copied().collect()
        );
        let candidates:Vec<Uuid>=sqlx::query_scalar("SELECT DISTINCT context_id FROM account_context_candidates WHERE account_id=$1 AND context_kind='COMPANY' AND context_id=ANY($2)").bind(accounts.submitter.account_id).bind(&companies).fetch_all(admin).await?;
        assert_eq!(
            candidates.into_iter().collect::<BTreeSet<_>>(),
            companies.iter().copied().collect()
        );

        let assignment_ids:Vec<Uuid>=sqlx::query_scalar("SELECT r.assignment_id FROM policy_assignment_heads h JOIN policy_assignment_revisions r ON (r.org_id,r.assignment_id,r.revision)=(h.org_id,h.assignment_id,h.current_revision) WHERE r.account_id=$1 AND r.org_id=ANY($2) ORDER BY r.org_id")
            .bind(accounts.submitter.account_id).bind(&companies).fetch_all(admin).await?;
        assert_eq!(assignment_ids.len(), grants.len());
        assert_eq!(
            assignment_ids.iter().copied().collect::<BTreeSet<_>>(),
            grants.iter().map(|g| g.grant_id).collect()
        );
        let mut pairs = super::account_transport_urls(admin);
        for name in [
            "CONSOLE_JWT_ISSUER",
            "CONSOLE_JWT_AUDIENCE",
            "CONSOLE_JWT_PRIVATE_KEY_PEM",
            "CONSOLE_JWT_PUBLIC_KEY_PEM",
        ] {
            pairs.push((name, std::env::var(name)?));
        }
        pairs.extend([
            ("CONSOLE_APP_ROLE", "api".into()),
            ("CONSOLE_WEBAUTHN_RP_ID", "example.com".into()),
            ("CONSOLE_WEBAUTHN_RP_ORIGIN", origin.to_string()),
            ("CONSOLE_WEBAUTHN_RP_NAME", "TEST_ONLY".into()),
            (
                "CONSOLE_ACCOUNT_TERMS_ARTIFACT_ROOT",
                custody
                    .directory
                    .path()
                    .to_str()
                    .ok_or("artifact path")?
                    .into(),
            ),
        ]);
        let state = AppState::from_config(AppConfig::from_pairs(pairs)?).await?;
        let serving = state.serving_admission_binding().clone();
        let router = build_router(state);
        // Reverify the actual token before entering HTTP; no fabricated browser
        // session row. This cookie carries exactly the owner-issued ACCOUNT_V1.
        console_platform_auth::account_session::verify_account_session(
            &auth_pool,
            &verifier,
            &accounts.submitter.account_access_token,
        )
        .await?;
        Ok(Self {
            router,
            serving,
            verifier,
            accounts,
            operator,
            owner_pool,
            auth_pool,
            companies,
            groups,
            expected,
            undiscoverable,
            label_denied,
            grants,
            custody,
        })
    }
    pub async fn page(
        &self,
        token: &str,
        path: &str,
    ) -> Result<(http::StatusCode, Value, http::HeaderMap)> {
        use axum::body::{Body, to_bytes};
        use tower::ServiceExt;
        let request = http::Request::builder()
            .method("GET")
            .uri(path)
            .header(
                http::header::COOKIE,
                format!("__Host-console_account_session={token}"),
            )
            .header(http::header::ORIGIN, "https://auth.example.com")
            .body(Body::empty())?;
        let response = self.router.clone().oneshot(request).await?;
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, 256 * 1024).await?;
        Ok((parts.status, serde_json::from_slice(&bytes)?, parts.headers))
    }
    pub async fn scan(&self) -> Result<BTreeMap<String, Option<String>>> {
        let mut path = "/api/v2/accounts/me/contexts?page_size=1".to_owned();
        let mut seen = BTreeMap::new();
        let mut cursors = BTreeSet::new();
        loop {
            let (status, page, headers) = self
                .page(&self.accounts.submitter.account_access_token, &path)
                .await?;
            assert_eq!(status, http::StatusCode::OK);
            assert!(
                headers[http::header::CACHE_CONTROL]
                    .to_str()?
                    .split(',')
                    .any(|v| v.trim() == "no-store")
            );
            let object = page.as_object().ok_or("page object")?;
            assert!(object.keys().all(|k| k == "contexts" || k == "next_cursor"));
            let items = page["contexts"].as_array().ok_or("contexts array")?;
            assert!(items.len() <= 1);
            for item in items {
                assert!(
                    item.as_object().ok_or("item")?.keys().all(|k| [
                        "context_ref",
                        "label",
                        "entry_ref"
                    ]
                    .contains(&k.as_str()))
                );
                assert_eq!(item["entry_ref"], json!({"kind":"WORKSPACE"}));
                let key = context_key(&item["context_ref"])?;
                assert_eq!(
                    item["kind"],
                    if key.starts_with("org_id:") {
                        json!("COMPANY")
                    } else {
                        json!("GROUP")
                    }
                );
                assert_ne!(key, self.undiscoverable);
                let label = item.get("label").map(|v| {
                    v.as_str()
                        .expect("label must be permitted string")
                        .to_owned()
                });
                assert!(
                    seen.insert(key, label).is_none(),
                    "duplicate authorized context"
                );
            }
            assert!(seen.len() <= self.groups.len() + self.companies.len());
            match page.get("next_cursor") {
                None => break,
                Some(v) => {
                    assert!(!items.is_empty());
                    let cursor = v.as_str().ok_or("opaque cursor")?;
                    assert!(cursors.insert(cursor.to_owned()));
                    path = format!("/api/v2/accounts/me/contexts?page_size=1&cursor={cursor}");
                }
            }
        }
        Ok(seen)
    }
}
