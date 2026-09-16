//! Concrete legacy225 crypto/router producer; only legacy fixture source rows
//! use disposable admin setup. Every tested request crosses the real auth router
//! on an actual restricted LOGIN. No Account/security/grant result is fabricated.
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, Response, StatusCode, header},
};
use console_kernel_core::OrgId;
use console_platform_auth::{
    JwtSettings, JwtVerifier, PasskeyRegistrationStart, PasskeyService, WebauthnSettings,
};
use console_platform_auth_rest::{AuthRestConfig, AuthRestState};
use console_platform_provisioning::BootstrapCredentialStore;
use console_platform_realtime::{PgRealtimeHub, RealtimeHubConfig, RealtimeRestState};
use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
use futures_util::StreamExt;
use p256::{
    ecdsa::SigningKey,
    elliptic_curve::rand_core::OsRng,
    pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding},
};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::sync::Arc;
use time::{Duration, OffsetDateTime};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream,
    tungstenite::{Message, client::IntoClientRequest},
};
use tower::ServiceExt;
use url::Url;
use uuid::Uuid;
use webauthn_authenticator_rs::{
    prelude::{RequestChallengeResponse, WebauthnAuthenticator},
    softpasskey::SoftPasskey,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub const ORIGIN: &str = "https://auth.example.com";
pub struct Key {
    pub account_id: Uuid,
    pub credential_id: String,
    pub authenticator: WebauthnAuthenticator<SoftPasskey>,
}
#[derive(Clone)]
pub struct Attempt {
    pub label: &'static str,
    pub method: &'static str,
    pub path: &'static str,
    pub access: Option<String>,
    pub cookie: Option<String>,
    pub body: Value,
}
pub struct Transport {
    pub router: Router,
    pub verifier: JwtVerifier,
    pub service: PasskeyService,
    pub key: Key,
    pub attempts: Vec<Attempt>,
    pub secrets: Vec<String>,
    pub revoked_key: String,
    pub revoked_authenticator: Key,
    pub groups: Vec<Uuid>,
    pub hub: Arc<PgRealtimeHub>,
    pub live: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    server: tokio::task::JoinHandle<()>,
    pub ws_url: String,
}
impl Drop for Transport {
    fn drop(&mut self) {
        self.server.abort();
    }
}
pub async fn request(router: &Router, a: &Attempt) -> Result<Response<Body>> {
    let mut request = Request::builder()
        .method(a.method)
        .uri(a.path)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(access) = &a.access {
        request = request.header(header::AUTHORIZATION, format!("Bearer {access}"));
    }
    if let Some(cookie) = &a.cookie {
        request = request
            .header("x-auth-transport", "cookie")
            .header(header::COOKIE, format!("console_refresh={cookie}"));
    }
    Ok(router
        .clone()
        .oneshot(request.body(if a.method == "GET" {
            Body::empty()
        } else {
            Body::from(serde_json::to_vec(&a.body)?)
        })?)
        .await?)
}
async fn json_response(response: Response<Body>, expected: StatusCode) -> Result<Value> {
    assert_eq!(response.status(), expected);
    Ok(serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024).await?,
    )?)
}
fn attempt(label: &'static str, path: &'static str, body: Value) -> Attempt {
    Attempt {
        label,
        method: "POST",
        path,
        access: None,
        cookie: None,
        body,
    }
}
async fn post(router: &Router, path: &'static str, body: Value) -> Result<Value> {
    json_response(
        request(router, &attempt("producer", path, body)).await?,
        StatusCode::OK,
    )
    .await
}
async fn register(pool: &PgPool, service: &PasskeyService, account_id: Uuid) -> Result<Key> {
    let org: Uuid = sqlx::query_scalar("SELECT org_id FROM users WHERE id=$1")
        .bind(account_id)
        .fetch_one(pool)
        .await?;
    let started = service
        .start_registration(
            pool,
            OrgId::from_uuid(org),
            PasskeyRegistrationStart {
                user_id: account_id,
                username: format!("migration-key-{account_id}"),
                display_name: "Migration retained key".into(),
            },
        )
        .await?;
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let credential = authenticator.do_registration(Url::parse(ORIGIN)?, started.challenge)?;
    let stored = service
        .finish_registration(pool, OrgId::from_uuid(org), started.ceremony_id, credential)
        .await?;
    assert_eq!(stored.user_id, account_id);
    Ok(Key {
        account_id,
        credential_id: stored.credential_id,
        authenticator,
    })
}
pub async fn assertion(router: &Router, key: &mut Key) -> Result<Value> {
    let started = post(router, "/api/v1/auth/passkey/login/start", json!({})).await?;
    let mut challenge = started["challenge"].clone();
    let allow = challenge["publicKey"]["allowCredentials"]
        .as_array_mut()
        .ok_or("missing discoverable allow list")?;
    assert!(allow.is_empty());
    allow
        .push(json!({"type":"public-key","id":serde_json::from_str::<Value>(&key.credential_id)?}));
    let challenge: RequestChallengeResponse = serde_json::from_value(challenge)?;
    let credential = key
        .authenticator
        .do_authentication(Url::parse(ORIGIN)?, challenge)?;
    Ok(json!({"ceremony_id":started["ceremony_id"],"credential":credential}))
}
fn cookie(response: &Response<Body>) -> Result<String> {
    let matches: Vec<_> = response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .filter(|v| v.starts_with("console_refresh="))
        .collect();
    assert_eq!(matches.len(), 1);
    let value = matches[0]
        .split(';')
        .next()
        .unwrap()
        .strip_prefix("console_refresh=")
        .unwrap();
    assert!(!value.is_empty());
    Ok(value.to_owned())
}
async fn websocket(
    url: &str,
    access: &str,
) -> Result<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>> {
    let mut request = url.into_client_request()?;
    request
        .headers_mut()
        .insert(header::AUTHORIZATION, format!("Bearer {access}").parse()?);
    let (socket, response) = tokio_tungstenite::connect_async(request).await?;
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    Ok(socket)
}
pub async fn build(pool: &PgPool, subjects: &[Uuid]) -> Result<Transport> {
    assert_eq!(subjects.len(), 2);
    let service = PasskeyService::new(WebauthnSettings {
        rp_id: "example.com".into(),
        rp_origin: Url::parse(ORIGIN)?,
        rp_name: "Console".into(),
        extra_allowed_origins: vec![],
        ceremony_ttl: Duration::minutes(5),
    })?;
    let mut key = register(pool, &service, subjects[0]).await?;
    let mut revoked = register(pool, &service, subjects[1]).await?;
    // Concrete legacy fixture inputs, matching existing group-admin fixture.
    // These create no Account grants and are never copied into expected authority.
    let mut groups = Vec::new();
    for (n, subject) in subjects.iter().enumerate() {
        let org: Uuid = sqlx::query_scalar("SELECT org_id FROM users WHERE id=$1")
            .bind(subject)
            .fetch_one(pool)
            .await?;
        // Immutable migration0225 already creates Group + unique membership
        // during the real organization insert. Reuse that authoritative identity.
        let memberships:Vec<Uuid>=sqlx::query_scalar(
            "SELECT g.id FROM organizations o JOIN groups g ON g.id=o.group_id JOIN group_memberships m ON m.org_id=o.id AND m.group_id=g.id WHERE o.id=$1")
            .bind(org).fetch_all(pool).await?;
        assert_eq!(
            memberships.len(),
            1,
            "actual225 organization must have one matching Group/membership"
        );
        let group = memberships[0];
        sqlx::query("INSERT INTO group_role_grants(group_id,user_id,group_role) VALUES($1,$2,'GROUP_ADMIN')").bind(group).bind(subjects[0]).execute(pool).await?;
        sqlx::query("INSERT INTO policy_versions(org_id,version,updated_at) VALUES($1,$2,now()) ON CONFLICT(org_id) DO UPDATE SET version=EXCLUDED.version").bind(org).bind(7_i64+n as i64).execute(pool).await?;
        groups.push(group);
    }
    assert_ne!(
        groups[0], groups[1],
        "the two legacy source organizations have independent Group-of-one scopes"
    );
    let signing = SigningKey::random(&mut OsRng);
    let private = signing.to_pkcs8_pem(LineEnding::LF)?.to_string();
    let public = signing.verifying_key().to_public_key_pem(LineEnding::LF)?;
    let verifier = JwtVerifier::from_es256_public_pem(
        JwtSettings {
            issuer: "console-platform-auth".into(),
            audience: "console-api".into(),
            access_token_ttl: Duration::minutes(15),
        },
        public.as_bytes(),
    )?;
    // Before225 legacy auth is on the ordinary Business LOGIN. Actual cutover
    // must fence this cached old router even after new auth custody is separated.
    let runtime = login_test_pool(pool, TestDatabaseLogin::Business).await;
    let auth = AuthRestState::new(
        runtime.clone(),
        AuthRestConfig {
            rp_id: "example.com".into(),
            rp_origin: ORIGIN.into(),
            rp_name: "Console".into(),
            ceremony_ttl: Duration::minutes(5),
            jwt_issuer: "console-platform-auth".into(),
            jwt_audience: "console-api".into(),
            jwt_private_key_pem: private,
            jwt_public_key_pem: public,
            refresh_token_ttl: Duration::hours(2),
            refresh_family_absolute_ttl: Duration::hours(24),
            cookie_secure: true,
        },
    )?;
    let hub = Arc::new(PgRealtimeHub::new(runtime, RealtimeHubConfig::default()));
    let router = console_platform_auth_rest::router(auth).merge(console_platform_realtime::router(
        RealtimeRestState::new(hub.clone(), Some(verifier.clone())),
    ));
    let login = post(
        &router,
        "/api/v1/auth/passkey/login/finish",
        assertion(&router, &mut key).await?,
    )
    .await?;
    let access = login["access_token"]
        .as_str()
        .ok_or("actual legacy access missing")?
        .to_owned();
    let body_refresh = login["refresh_token"]
        .as_str()
        .ok_or("actual legacy body refresh missing")?
        .to_owned();
    let cookie_login = attempt(
        "cookie login",
        "/api/v1/auth/passkey/login/finish",
        assertion(&router, &mut key).await?,
    );
    // cookie transport header independent of cookie input, before a cookie exists.
    let cookie_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(cookie_login.path)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-auth-transport", "cookie")
                .body(Body::from(serde_json::to_vec(&cookie_login.body)?))?,
        )
        .await?;
    assert_eq!(cookie_response.status(), StatusCode::OK);
    let cookie_refresh = cookie(&cookie_response)?;
    let body = json_response(cookie_response, StatusCode::OK).await?;
    assert!(body["refresh_token"].is_null());
    let read = Attempt {
        label: "old JWT",
        method: "GET",
        path: "/api/v1/auth/passkeys",
        access: Some(access.clone()),
        cookie: None,
        body: Value::Null,
    };
    let keys = json_response(request(&router, &read).await?, StatusCode::OK).await?;
    assert!(keys.as_array().is_some_and(|a| a.len() >= 2));
    let group_read = Attempt {
        label: "old two-Group JWT",
        path: "/api/v1/group-admin/groups",
        ..read.clone()
    };
    let group_body = json_response(request(&router, &group_read).await?, StatusCode::OK).await?;
    let group_text = serde_json::to_string(&group_body)?;
    for group in &groups {
        assert!(group_text.contains(&group.to_string()));
    }
    let mut attempts = vec![
        read,
        group_read,
        attempt(
            "body refresh",
            "/api/v1/auth/token/refresh",
            json!({"refresh_token":body_refresh}),
        ),
        Attempt {
            cookie: Some(cookie_refresh.clone()),
            ..attempt("cookie refresh", "/api/v1/auth/token/refresh", json!({}))
        },
        attempt(
            "pending passkey assertion",
            "/api/v1/auth/passkey/login/finish",
            assertion(&router, &mut key).await?,
        ),
    ];
    for (n, subject) in subjects.iter().enumerate() {
        let org: Uuid = sqlx::query_scalar("SELECT org_id FROM users WHERE id=$1")
            .bind(subject)
            .fetch_one(pool)
            .await?;
        let context = Attempt {
            access: Some(access.clone()),
            ..attempt(
                "old Group context",
                "/api/v1/group-admin/tenant-context",
                json!({"org_id":org}),
            )
        };
        let minted = json_response(request(&router, &context).await?, StatusCode::OK).await?;
        let token = minted["access_token"]
            .as_str()
            .ok_or("actual Group-context token missing")?;
        let claims = verifier.verify_access_token(token)?;
        assert_eq!(claims.authz_policy_version, 7_u64 + n as u64);
        attempts.push(context);
    }
    // A second real key is cryptographically usable before ordinary legacy reset.
    post(
        &router,
        "/api/v1/auth/passkey/login/finish",
        assertion(&router, &mut revoked).await?,
    )
    .await?;
    let org: Uuid = sqlx::query_scalar("SELECT org_id FROM users WHERE id=$1")
        .bind(subjects[1])
        .fetch_one(pool)
        .await?;
    let reset = BootstrapCredentialStore
        .reset_credentials_for_user(
            pool,
            subjects[1],
            OrgId::from_uuid(org),
            OffsetDateTime::now_utc(),
            Duration::hours(2),
        )
        .await?;
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM auth_webauthn_credentials WHERE credential_id=$1")
            .bind(&revoked.credential_id)
            .fetch_one(pool)
            .await?;
    assert_eq!(count, 0);
    let revoked_assertion = assertion(&router, &mut revoked).await?;
    let denied = request(
        &router,
        &attempt(
            "pre-revoked positive denial",
            "/api/v1/auth/passkey/login/finish",
            revoked_assertion,
        ),
    )
    .await?;
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
    attempts.push(attempt(
        "pre-revoked passkey",
        "/api/v1/auth/passkey/login/finish",
        assertion(&router, &mut revoked).await?,
    ));
    attempts.push(attempt(
        "employer OTP",
        "/api/v1/auth/otp/redeem",
        json!({"otp":reset.token.as_str()}),
    ));
    // Actual approved but unconsumed device handoff remains a live credential.
    let device = post(&router, "/api/v1/auth/device-login/start", json!({})).await?;
    let url = Url::parse(
        device["approve_url"]
            .as_str()
            .ok_or("missing device approval URL")?,
    )?;
    let approve = url::form_urlencoded::parse(
        url.fragment()
            .ok_or("missing approval fragment")?
            .as_bytes(),
    )
    .find(|(k, _)| k == "desktop_approve")
    .ok_or("missing approval token")?
    .1
    .into_owned();
    let mut approval = assertion(&router, &mut key).await?;
    approval["approve_token"] = json!(approve);
    let approved = request(
        &router,
        &attempt(
            "device approve",
            "/api/v1/auth/device-login/approve",
            approval,
        ),
    )
    .await?;
    assert_eq!(approved.status(), StatusCode::NO_CONTENT);
    let poll = device["poll_token"]
        .as_str()
        .ok_or("missing actual device poll token")?
        .to_owned();
    attempts.push(attempt(
        "approved device poll",
        "/api/v1/auth/device-login/poll",
        json!({"poll_token":poll}),
    ));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let ws_url = format!("ws://{}/api/v1/ws", listener.local_addr()?);
    let served = router.clone();
    let server = tokio::spawn(async move {
        axum::serve(listener, served)
            .await
            .expect("owned actual router server");
    });
    let live = websocket(&ws_url, &access).await?;
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        while hub.connection_count().await != 1 {
            tokio::task::yield_now().await;
        }
    })
    .await?;
    Ok(Transport {
        router,
        verifier,
        service,
        key,
        attempts,
        secrets: vec![
            access,
            body_refresh,
            cookie_refresh,
            reset.token.as_str().to_owned(),
            approve,
            poll,
        ],
        revoked_key: revoked.credential_id.clone(),
        revoked_authenticator: revoked,
        groups,
        hub,
        live,
        server,
        ws_url,
    })
}
pub async fn denied(response: Response<Body>, secrets: &[String]) -> Result {
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "storage failure/route absence is not credential refusal"
    );
    for (name, value) in response.headers() {
        let value = value.to_str()?;
        assert!(
            !secrets
                .iter()
                .any(|secret| !secret.is_empty() && value.contains(secret))
        );
        if name == header::SET_COOKIE {
            assert!(value.split(';').next().is_some_and(|v| v.ends_with('=')));
        }
    }
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await?;
    let text = std::str::from_utf8(&bytes)?;
    assert!(
        !secrets
            .iter()
            .any(|secret| !secret.is_empty() && text.contains(secret))
    );
    let body: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(body["error"]["code"], "unauthorized");
    assert!(body.get("access_token").is_none() && body.get("refresh_token").is_none());
    Ok(())
}
impl Transport {
    pub async fn concurrent_denials(&self) -> Result {
        let responses =
            futures_util::future::join_all(self.attempts.iter().map(|a| request(&self.router, a)))
                .await;
        for response in responses {
            denied(response?, &self.secrets).await?;
        }
        Ok(())
    }
    pub async fn require_drained_live(&mut self) -> Result {
        let frame = tokio::time::timeout(std::time::Duration::from_secs(2), self.live.next())
            .await?
            .ok_or("bare EOF is not explicit security drain")??;
        match frame {
            Message::Close(Some(close)) => {
                assert_eq!(u16::from(close.code), 1008);
                assert_eq!(close.reason, "authentication cutover");
            }
            _ => panic!("actual accepted legacy stream was not explicitly security-drained"),
        };
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            while self.hub.connection_count().await != 0 {
                tokio::task::yield_now().await;
            }
        })
        .await?;
        let mut request = self.ws_url.as_str().into_client_request()?;
        request.headers_mut().insert(
            header::AUTHORIZATION,
            format!("Bearer {}", self.secrets[0]).parse()?,
        );
        match tokio_tungstenite::connect_async(request).await {
            Err(tokio_tungstenite::tungstenite::Error::Http(response)) => {
                assert_eq!(response.status(), StatusCode::UNAUTHORIZED)
            }
            _ => panic!("old JWT must not reopen an upgraded connection"),
        };
        Ok(())
    }
}
