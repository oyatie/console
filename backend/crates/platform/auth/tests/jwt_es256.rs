#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use console_kernel_core::{AccessScope, AccessScopeLevel, BranchId, OrgId, ScopeNodeId, UserId};
use console_platform_auth::{
    AccessClaims, AccessTokenInput, JwtIssuer, JwtSettings, TenantAccessContext,
};
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use p256::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use time::{Duration, OffsetDateTime};

fn es256_material() -> (JwtIssuer, String, String) {
    let signing_key = SigningKey::random(&mut OsRng);
    let private_pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();

    let issuer = JwtIssuer::from_es256_pem(
        JwtSettings {
            issuer: "console-platform-auth".to_owned(),
            audience: "console-api".to_owned(),
            access_token_ttl: Duration::minutes(15),
        },
        private_pem.as_bytes(),
        public_pem.as_bytes(),
    )
    .unwrap();

    (issuer, private_pem.to_string(), public_pem)
}

fn es256_issuer() -> JwtIssuer {
    es256_material().0
}

#[test]
fn es256_access_token_round_trips_with_expected_claims() {
    let issuer = es256_issuer();

    let user_id = UserId::new();
    let branch_id = BranchId::new();
    let now = OffsetDateTime::now_utc();

    let token = issuer
        .issue_access_token(AccessTokenInput {
            subject: user_id,
            org_id: OrgId::knl(),
            roles: vec!["MECHANIC".to_owned()],
            branches: vec![branch_id],
            platform: false,
            view_as: false,
            read_only: false,
            display_name: None,
            feature_grants: Vec::new(),
            authz_subject_version: 0,
            authz_policy_version: 0,
            session_generation: 0,
            issued_at: now,
        })
        .unwrap();

    let claims = issuer.verify_access_token(&token).unwrap();

    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.iss, "console-platform-auth");
    assert_eq!(claims.aud, "console-api");
    assert_eq!(claims.roles, vec!["MECHANIC"]);
    assert_eq!(claims.branches, vec![branch_id.to_string()]);
    assert_eq!(claims.iat, now.unix_timestamp());
    assert_eq!(claims.nbf, now.unix_timestamp());
    assert_eq!(claims.exp, (now + Duration::minutes(15)).unix_timestamp());
    assert_eq!(claims.alg, "ES256");
    // No display name supplied -> the optional `name` claim is absent.
    assert_eq!(claims.name, None);
    assert_eq!(
        claims.access_scope().unwrap(),
        AccessScope::legacy_org(OrgId::knl())
    );
    assert!(claims.group_roles.is_empty());
    assert!(claims.feature_grants.is_empty());
}

#[test]
fn es256_access_token_carries_feature_grant_ui_hints() {
    let issuer = es256_issuer();

    let token = issuer
        .issue_access_token(AccessTokenInput {
            subject: UserId::new(),
            org_id: OrgId::knl(),
            roles: vec!["MEMBER".to_owned()],
            branches: vec![],
            platform: false,
            view_as: false,
            read_only: false,
            display_name: None,
            feature_grants: vec!["mail_use".to_owned(), "role_manage".to_owned()],
            authz_subject_version: 0,
            authz_policy_version: 0,
            session_generation: 0,
            issued_at: OffsetDateTime::now_utc(),
        })
        .unwrap();

    let claims = issuer.verify_access_token(&token).unwrap();
    assert_eq!(claims.feature_grants, vec!["mail_use", "role_manage"]);
}

#[test]
fn es256_rejects_actor_home_org_on_non_delegated_tokens() {
    let (issuer, private_pem, _) = es256_material();
    let token = issuer
        .issue_access_token(AccessTokenInput {
            subject: UserId::new(),
            org_id: OrgId::knl(),
            roles: vec!["ADMIN".to_owned()],
            branches: Vec::new(),
            platform: false,
            view_as: false,
            read_only: false,
            display_name: None,
            feature_grants: Vec::new(),
            authz_subject_version: 0,
            authz_policy_version: 0,
            session_generation: 0,
            issued_at: OffsetDateTime::now_utc(),
        })
        .unwrap();
    let mut claims = issuer.verify_access_token(&token).unwrap();
    claims.actor_home_org = Some(OrgId::new().to_string());
    let forged = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::ES256),
        &claims,
        &jsonwebtoken::EncodingKey::from_ec_pem(private_pem.as_bytes()).unwrap(),
    )
    .unwrap();

    let err = issuer.verify_access_token(&forged).unwrap_err();
    assert!(
        err.to_string()
            .contains("actor_home_org requires group-admin tenant context")
    );
}

#[test]
fn es256_access_token_carries_optional_display_name_claim() {
    let issuer = es256_issuer();

    let token = issuer
        .issue_access_token(AccessTokenInput {
            subject: UserId::new(),
            org_id: OrgId::knl(),
            roles: vec!["ADMIN".to_owned()],
            branches: vec![],
            platform: false,
            view_as: false,
            read_only: false,
            display_name: Some("홍길동".to_owned()),
            feature_grants: Vec::new(),
            authz_subject_version: 0,
            authz_policy_version: 0,
            session_generation: 0,
            issued_at: OffsetDateTime::now_utc(),
        })
        .unwrap();

    // The display name round-trips in the `name` claim (display only; the
    // verifier never authorizes off it). The round-trip through encode/verify
    // proves the claim is serialized into and parsed back out of the JWT, which
    // is exactly what the web client decodes for the topbar identity.
    let claims = issuer.verify_access_token(&token).unwrap();
    assert_eq!(claims.name.as_deref(), Some("홍길동"));
}

#[test]
fn es256_access_token_can_carry_group_roles_without_widening_scope() {
    let issuer = es256_issuer();
    let org_id = OrgId::knl();

    let token = issuer
        .issue_access_token_with_group_roles(
            AccessTokenInput {
                subject: UserId::new(),
                org_id,
                roles: vec!["MEMBER".to_owned()],
                branches: vec![],
                platform: false,
                view_as: false,
                read_only: false,
                display_name: None,
                feature_grants: Vec::new(),
                authz_subject_version: 0,
                authz_policy_version: 0,
                session_generation: 0,
                issued_at: OffsetDateTime::now_utc(),
            },
            vec!["GROUP_ADMIN".to_owned()],
        )
        .unwrap();

    let claims = issuer.verify_access_token(&token).unwrap();
    assert_eq!(claims.group_roles, vec!["GROUP_ADMIN"]);
    assert_eq!(
        claims.access_scope().unwrap(),
        AccessScope::legacy_org(org_id),
        "group-role claims are UI hints; backend endpoints re-resolve live grants",
    );
}

#[test]
fn group_admin_tenant_context_token_is_bounded_and_distinct_from_super_admin() {
    let issuer = es256_issuer();
    let group_id = uuid::Uuid::new_v4();
    let target_org = OrgId::new();
    let actor_home_org = OrgId::knl();

    let token = issuer
        .issue_group_admin_tenant_context_access_token(
            AccessTokenInput {
                subject: UserId::new(),
                org_id: target_org,
                roles: vec!["ADMIN".to_owned()],
                branches: vec![],
                platform: false,
                view_as: false,
                read_only: false,
                display_name: None,
                feature_grants: Vec::new(),
                authz_subject_version: 0,
                authz_policy_version: 0,
                session_generation: 0,
                issued_at: OffsetDateTime::now_utc(),
            },
            group_id,
            actor_home_org,
            Duration::minutes(15),
        )
        .unwrap();

    let claims = issuer.verify_access_token(&token).unwrap();
    assert_eq!(claims.roles, vec!["ADMIN"]);
    assert!(!claims.roles.iter().any(|role| role == "SUPER_ADMIN"));
    assert_eq!(claims.group_roles, vec!["GROUP_ADMIN"]);
    assert_eq!(claims.tenant_context, Some(TenantAccessContext::GroupAdmin));
    assert_eq!(claims.group_context_id, Some(group_id.to_string()));
    assert_eq!(claims.actor_home_org, Some(actor_home_org.to_string()));
    assert_eq!(claims.org, target_org.to_string());
    assert_ne!(claims.actor_home_org.as_deref(), Some(claims.org.as_str()));
}

#[test]
fn group_admin_tenant_context_token_rejects_super_admin_role() {
    let issuer = es256_issuer();

    let err = issuer
        .issue_group_admin_tenant_context_access_token(
            AccessTokenInput {
                subject: UserId::new(),
                org_id: OrgId::knl(),
                roles: vec!["SUPER_ADMIN".to_owned()],
                branches: vec![],
                platform: false,
                view_as: false,
                read_only: false,
                display_name: None,
                feature_grants: Vec::new(),
                authz_subject_version: 0,
                authz_policy_version: 0,
                session_generation: 0,
                issued_at: OffsetDateTime::now_utc(),
            },
            uuid::Uuid::new_v4(),
            OrgId::knl(),
            Duration::minutes(15),
        )
        .unwrap_err();

    assert!(err.to_string().contains("cannot carry SUPER_ADMIN"));
}

#[test]
fn es256_access_token_round_trips_explicit_access_scope_claims() {
    let issuer = es256_issuer();

    let scope = AccessScope::new(
        AccessScopeLevel::Group,
        ScopeNodeId::from_uuid(uuid::Uuid::new_v4()),
    );
    let token = issuer
        .issue_scoped_access_token(
            AccessTokenInput {
                subject: UserId::new(),
                org_id: OrgId::knl(),
                roles: vec!["ADMIN".to_owned()],
                branches: Vec::new(),
                platform: false,
                view_as: false,
                read_only: false,
                display_name: None,
                feature_grants: Vec::new(),
                authz_subject_version: 0,
                authz_policy_version: 0,
                session_generation: 0,
                issued_at: OffsetDateTime::now_utc(),
            },
            scope,
            vec!["GROUP_ADMIN".to_owned()],
        )
        .unwrap();

    let claims = issuer.verify_access_token(&token).unwrap();
    assert_eq!(claims.scope_level, Some(AccessScopeLevel::Group));
    assert_eq!(claims.scope_node, Some(scope.node_id));
    assert_eq!(claims.access_scope().unwrap(), scope);
    assert_eq!(claims.group_roles, vec!["GROUP_ADMIN"]);
}

#[test]
fn es256_scoped_token_rejects_unknown_group_role_on_issue() {
    let issuer = es256_issuer();

    let err = issuer
        .issue_scoped_access_token(
            AccessTokenInput {
                subject: UserId::new(),
                org_id: OrgId::knl(),
                roles: vec!["ADMIN".to_owned()],
                branches: Vec::new(),
                platform: false,
                view_as: false,
                read_only: false,
                display_name: None,
                feature_grants: Vec::new(),
                authz_subject_version: 0,
                authz_policy_version: 0,
                session_generation: 0,
                issued_at: OffsetDateTime::now_utc(),
            },
            AccessScope::legacy_org(OrgId::knl()),
            vec!["group_admin".to_owned()],
        )
        .unwrap_err();

    assert!(err.to_string().contains("unknown group role code"));
}

#[test]
fn es256_scoped_token_rejects_unknown_group_role_on_verify() {
    let (issuer, private_pem, _) = es256_material();
    let now = OffsetDateTime::now_utc();
    let claims = AccessClaims {
        iss: "console-platform-auth".to_owned(),
        aud: "console-api".to_owned(),
        sub: UserId::new().to_string(),
        iat: now.unix_timestamp(),
        nbf: now.unix_timestamp(),
        exp: (now + Duration::minutes(15)).unix_timestamp(),
        jti: uuid::Uuid::new_v4().to_string(),
        org: OrgId::knl().to_string(),
        roles: vec!["ADMIN".to_owned()],
        branches: Vec::new(),
        platform: false,
        view_as: false,
        read_only: false,
        name: None,
        scope_level: Some(AccessScopeLevel::Group),
        scope_node: Some(ScopeNodeId::from_uuid(uuid::Uuid::new_v4())),
        group_roles: vec!["GROUP_OWNER".to_owned()],
        tenant_context: None,
        group_context_id: None,
        actor_home_org: None,
        feature_grants: Vec::new(),
        authz_subject_version: 0,
        authz_policy_version: 0,
        session_generation: 0,
        alg: "ES256".to_owned(),
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::ES256),
        &claims,
        &jsonwebtoken::EncodingKey::from_ec_pem(private_pem.as_bytes()).unwrap(),
    )
    .unwrap();

    let err = issuer.verify_access_token(&token).unwrap_err();
    assert!(err.to_string().contains("unknown group role code"));
}

#[test]
fn es256_view_as_token_refuses_group_roles() {
    let issuer = es256_issuer();

    let err = issuer
        .issue_scoped_access_token(
            AccessTokenInput {
                subject: UserId::new(),
                org_id: OrgId::knl(),
                roles: vec!["ADMIN".to_owned()],
                branches: Vec::new(),
                platform: false,
                view_as: true,
                read_only: true,
                display_name: None,
                feature_grants: Vec::new(),
                authz_subject_version: 0,
                authz_policy_version: 0,
                session_generation: 0,
                issued_at: OffsetDateTime::now_utc(),
            },
            AccessScope::legacy_org(OrgId::knl()),
            vec!["GROUP_ADMIN".to_owned()],
        )
        .unwrap_err();

    assert!(
        err.to_string()
            .contains("view-as tokens cannot carry group roles")
    );
}

#[test]
fn access_scope_claims_must_be_a_complete_pair() {
    let claims = AccessClaims {
        iss: "console-platform-auth".to_owned(),
        aud: "console-api".to_owned(),
        sub: UserId::new().to_string(),
        iat: 1,
        nbf: 1,
        exp: 2,
        jti: uuid::Uuid::new_v4().to_string(),
        org: OrgId::knl().to_string(),
        roles: Vec::new(),
        branches: Vec::new(),
        platform: false,
        view_as: false,
        read_only: false,
        name: None,
        scope_level: Some(AccessScopeLevel::Org),
        scope_node: None,
        group_roles: Vec::new(),
        tenant_context: None,
        group_context_id: None,
        actor_home_org: None,
        feature_grants: Vec::new(),
        authz_subject_version: 0,
        authz_policy_version: 0,
        session_generation: 0,
        alg: "ES256".to_owned(),
    };

    let err = claims.access_scope().unwrap_err();
    assert!(
        err.to_string()
            .contains("scope claims must include both scope_level and scope_node")
    );
}

// Cedar/PBAC activation (ADR-0021): the access token carries a subject
// authorization freshness snapshot. SLICE-2 sources it; no decision consults it.
#[test]
fn es256_access_token_stamps_subject_authz_freshness() {
    let issuer = es256_issuer();
    let now = OffsetDateTime::now_utc();

    let token = issuer
        .issue_access_token(AccessTokenInput {
            subject: UserId::new(),
            org_id: OrgId::knl(),
            roles: vec!["SUPER_ADMIN".to_owned()],
            branches: Vec::new(),
            platform: false,
            view_as: false,
            read_only: false,
            display_name: None,
            feature_grants: Vec::new(),
            authz_subject_version: 7,
            authz_policy_version: 3,
            session_generation: 5,
            issued_at: now,
        })
        .unwrap();

    let claims = issuer.verify_access_token(&token).unwrap();
    assert_eq!(claims.authz_subject_version, 7);
    assert_eq!(claims.authz_policy_version, 3);
    assert_eq!(claims.session_generation, 5);
}

// A token minted before the freshness claims existed simply omits them on the
// wire. #[serde(default)] must accept it and default all three to 0, so old
// tokens keep their exact meaning on every live path (a 0-carrying token is only
// ever denied on the still-unreachable Cedar path).
#[test]
fn legacy_access_token_without_freshness_claims_defaults_to_zero() {
    let (issuer, private_pem, _) = es256_material();
    let now = OffsetDateTime::now_utc();

    let legacy = serde_json::json!({
        "iss": "console-platform-auth",
        "aud": "console-api",
        "sub": UserId::new().to_string(),
        "iat": now.unix_timestamp(),
        "nbf": now.unix_timestamp(),
        "exp": (now + Duration::minutes(15)).unix_timestamp(),
        "jti": uuid::Uuid::new_v4().to_string(),
        "org": OrgId::knl().to_string(),
        "roles": ["MECHANIC"],
        "branches": [],
        "alg": "ES256",
    });
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::ES256),
        &legacy,
        &jsonwebtoken::EncodingKey::from_ec_pem(private_pem.as_bytes()).unwrap(),
    )
    .unwrap();

    let claims = issuer.verify_access_token(&token).unwrap();
    assert_eq!(claims.authz_subject_version, 0);
    assert_eq!(claims.authz_policy_version, 0);
    assert_eq!(claims.session_generation, 0);
}
// AS1/BW31 cryptographic claims only: these tests establish no live session,
// Account state, custody privilege, browser identity, or Company authority.
mod account_crypto {
    use super::*;
    use console_platform_auth::{
        AccountAccessClaims, AccountAccessTokenInput, AccountAccessVerification, AccountAssurance,
        AccountCsrfTokenInput, AuthError, JwtVerifier, SignedAccountToken,
    };
    use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
    use serde::ser::SerializeMap;
    use serde::{Serialize, Serializer};
    use serde_json::{Value, json};
    use uuid::Uuid;

    const ISSUER: &str = "console-platform-auth";
    const ACCESS_AUD: &str = "console.account.v1";
    const CSRF_AUD: &str = "console.account.csrf.v1";
    const ACCOUNT: &str = "deadc0de-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
    const FAMILY: &str = "fadedade-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
    const SECRET_MARKER: &str = "TEST_ONLY_PRIVATE_CLAIM_MUST_NOT_APPEAR_IN_ERRORS";

    #[derive(Clone, Copy)]
    enum Protocol {
        Access,
        Csrf,
    }

    impl Protocol {
        const ALL: [Self; 2] = [Self::Access, Self::Csrf];

        fn audience(self) -> &'static str {
            match self {
                Self::Access => ACCESS_AUD,
                Self::Csrf => CSRF_AUD,
            }
        }

        fn required(self) -> &'static [&'static str] {
            match self {
                Self::Access => &[
                    "iss",
                    "aud",
                    "token_kind",
                    "sub",
                    "sid",
                    "jti",
                    "iat",
                    "nbf",
                    "exp",
                    "security_generation",
                    "auth_time",
                    "assurance",
                ],
                Self::Csrf => &[
                    "iss",
                    "aud",
                    "token_kind",
                    "sub",
                    "sid",
                    "iat",
                    "nbf",
                    "exp",
                    "security_generation",
                ],
            }
        }

        fn max_seconds(self) -> i64 {
            match self {
                Self::Access => 900,
                Self::Csrf => 300,
            }
        }
    }

    struct Fixture {
        issuer: JwtIssuer,
        verifier: JwtVerifier,
        encoding_key: EncodingKey,
        decoding_key: DecodingKey,
        now: OffsetDateTime,
    }

    impl Fixture {
        fn new(ttl: Duration) -> Self {
            Self::with_issuer(ttl, ISSUER)
        }

        fn with_issuer(ttl: Duration, issuer: &str) -> Self {
            // Reuse the registered binary's existing ephemeral key fixture.
            let (_, private_pem, public_pem) = es256_material();
            let settings = JwtSettings {
                issuer: issuer.to_owned(),
                audience: "console-api".to_owned(),
                access_token_ttl: ttl,
            };
            Self {
                issuer: JwtIssuer::from_es256_pem(
                    settings.clone(),
                    private_pem.as_bytes(),
                    public_pem.as_bytes(),
                )
                .unwrap(),
                verifier: JwtVerifier::from_es256_public_pem(settings, public_pem.as_bytes())
                    .unwrap(),
                encoding_key: EncodingKey::from_ec_pem(private_pem.as_bytes()).unwrap(),
                decoding_key: DecodingKey::from_ec_pem(public_pem.as_bytes()).unwrap(),
                now: OffsetDateTime::now_utc().replace_nanosecond(0).unwrap(),
            }
        }

        fn access_input(&self, family_expires_at: OffsetDateTime) -> AccountAccessTokenInput {
            AccountAccessTokenInput {
                account_id: Uuid::parse_str(ACCOUNT).unwrap(),
                session_id: Uuid::parse_str(FAMILY).unwrap(),
                security_generation: 7,
                auth_time: self.now - Duration::minutes(2),
                assurance: AccountAssurance::PasskeyPrimary,
                issued_at: self.now,
                family_expires_at,
            }
        }

        fn csrf_input(&self, family_expires_at: OffsetDateTime) -> AccountCsrfTokenInput {
            AccountCsrfTokenInput {
                account_id: Uuid::parse_str(ACCOUNT).unwrap(),
                session_id: Uuid::parse_str(FAMILY).unwrap(),
                security_generation: 7,
                issued_at: self.now,
                family_expires_at,
            }
        }

        fn legacy_token(&self) -> String {
            self.issuer
                .issue_access_token(AccessTokenInput {
                    subject: UserId::new(),
                    org_id: OrgId::knl(),
                    roles: vec!["MEMBER".to_owned()],
                    branches: Vec::new(),
                    platform: false,
                    view_as: false,
                    read_only: false,
                    display_name: None,
                    feature_grants: Vec::new(),
                    authz_subject_version: 0,
                    authz_policy_version: 0,
                    session_generation: 0,
                    issued_at: self.now,
                })
                .unwrap_or_else(|_| {
                    panic!("ACCOUNT_CRYPTO_CONTROL: ordinary legacy issuance failed")
                })
        }

        fn issued(&self, protocol: Protocol) -> SignedAccountToken {
            let expiry = self.now + Duration::hours(1);
            match protocol {
                Protocol::Access => self
                    .issuer
                    .issue_account_access_token(self.access_input(expiry)),
                Protocol::Csrf => self
                    .issuer
                    .issue_account_csrf_token(self.csrf_input(expiry)),
            }
            .unwrap_or_else(|_| panic!("ACCOUNT_CRYPTO_CONTROL: valid issuance failed"))
        }

        fn current_access(&self, token: &str) -> AccountAccessClaims {
            match self.verifier.verify_account_access_token(token, self.now) {
                Ok(AccountAccessVerification::Current(claims)) => claims,
                _ => panic!("ACCOUNT_CRYPTO_CONTROL: current access required"),
            }
        }

        fn verify_at(
            &self,
            protocol: Protocol,
            token: &str,
            now: OffsetDateTime,
        ) -> Result<(), AuthError> {
            match protocol {
                Protocol::Access => self
                    .verifier
                    .verify_account_access_token(token, now)
                    .map(|_| ()),
                Protocol::Csrf => self
                    .verifier
                    .verify_account_csrf_token(token, now)
                    .map(|_| ()),
            }
        }

        fn verify(&self, protocol: Protocol, token: &str) -> Result<(), AuthError> {
            self.verify_at(protocol, token, self.now)
        }

        fn sign<T: Serialize>(&self, claims: &T) -> String {
            encode(&Header::new(Algorithm::ES256), claims, &self.encoding_key)
                .unwrap_or_else(|_| panic!("ACCOUNT_CRYPTO_CONTROL: test signing failed"))
        }

        fn issued_value(&self, protocol: Protocol) -> Value {
            let token = self.issued(protocol);
            assert!(
                self.verify(protocol, token.as_str()).is_ok(),
                "ACCOUNT_CRYPTO_CONTROL: valid owner token refused"
            );
            // Independent signature/issuer/audience readback. The production
            // boundary separately receives the explicit server clock above.
            let mut validation = Validation::new(Algorithm::ES256);
            validation.set_issuer(&[ISSUER]);
            validation.set_audience(&[protocol.audience()]);
            validation.validate_exp = false;
            validation.validate_nbf = false;
            validation.leeway = 0;
            let decoded = decode::<Value>(token.as_str(), &self.decoding_key, &validation)
                .unwrap_or_else(|_| {
                    panic!("ACCOUNT_CRYPTO_CONTROL: independent ES256 readback failed")
                });
            assert!(decoded.header.alg == Algorithm::ES256);
            let resigned = self.sign(&decoded.claims);
            assert!(
                self.verify(protocol, &resigned).is_ok(),
                "ACCOUNT_CRYPTO_CONTROL: unchanged signer control refused"
            );
            decoded.claims
        }

        fn reject(&self, protocol: Protocol, claims: &Value, case: &str) {
            let token = self.sign(claims);
            assert!(
                matches!(
                    self.verify(protocol, &token),
                    Err(AuthError::Jwt(error)) if matches!(error.kind(), jsonwebtoken::errors::ErrorKind::InvalidToken)
                ),
                "ACCOUNT_CRYPTO: {case} did not return fixed InvalidToken"
            );
        }
    }

    // Serialize directly to the encoder with an actual repeated JSON member.
    // Parsing into Value before verification would erase this adversarial input.
    struct DuplicateClaim<'a> {
        claims: &'a Value,
        key: &'a str,
        repeated: Value,
    }

    impl Serialize for DuplicateClaim<'_> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let object = self.claims.as_object().unwrap();
            let mut map = serializer.serialize_map(Some(object.len() + 1))?;
            for (key, value) in object {
                map.serialize_entry(key, value)?;
            }
            map.serialize_entry(self.key, &self.repeated)?;
            map.end()
        }
    }

    #[test]
    fn access_round_trip_is_closed_company_free_and_bound_to_primary_metadata() {
        let fixture = Fixture::new(Duration::minutes(15));
        let token = fixture.issued(Protocol::Access);
        let claims = fixture.current_access(token.as_str());
        assert!(claims.sub == Uuid::parse_str(ACCOUNT).unwrap());
        assert!(claims.sid == Uuid::parse_str(FAMILY).unwrap());
        assert!(claims.security_generation == 7);
        assert!(claims.auth_time == (fixture.now - Duration::minutes(2)).unix_timestamp());
        assert!(matches!(claims.assurance, AccountAssurance::PasskeyPrimary));
        let value = fixture.issued_value(Protocol::Access);
        let mut keys: Vec<_> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        let mut required = Protocol::Access.required().to_vec();
        required.sort_unstable();
        assert!(keys == required, "ACCOUNT_CRYPTO: access claim set differs");
        assert!(value["aud"] == ACCESS_AUD && value["token_kind"] == "account_v1");
        assert!(value["security_generation"] == "7");
        assert!(
            value["iat"] == fixture.now.unix_timestamp()
                && value["nbf"] == fixture.now.unix_timestamp()
        );
        assert!(value["exp"] == (fixture.now + Duration::minutes(15)).unix_timestamp());
        let next = fixture.issued_value(Protocol::Access);
        assert!(
            value["jti"] != next["jti"],
            "ACCOUNT_CRYPTO: access issuance reused jti"
        );
    }

    #[test]
    fn csrf_round_trip_is_closed_and_has_no_access_or_assurance_projection() {
        let fixture = Fixture::new(Duration::minutes(15));
        let token = fixture.issued(Protocol::Csrf);
        let claims = fixture
            .verifier
            .verify_account_csrf_token(token.as_str(), fixture.now)
            .unwrap_or_else(|_| panic!("ACCOUNT_CRYPTO: proof round trip failed"));
        assert!(claims.sub == Uuid::parse_str(ACCOUNT).unwrap());
        assert!(claims.sid == Uuid::parse_str(FAMILY).unwrap());
        assert!(claims.security_generation == 7);
        let value = fixture.issued_value(Protocol::Csrf);
        let mut keys: Vec<_> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        let mut required = Protocol::Csrf.required().to_vec();
        required.sort_unstable();
        assert!(keys == required, "ACCOUNT_CRYPTO: proof claim set differs");
        assert!(value["aud"] == CSRF_AUD && value["token_kind"] == "account_csrf_v1");
        assert!(value["security_generation"] == "7");
        assert!(value["exp"] == (fixture.now + Duration::minutes(5)).unix_timestamp());
    }

    #[test]
    fn access_issuance_respects_configured_fifteen_minute_and_family_ceilings() {
        for (ttl_seconds, family_seconds, expected_seconds) in
            [(30, 3600, 30), (3600, 3600, 900), (900, 29, 29)]
        {
            let fixture = Fixture::new(Duration::seconds(ttl_seconds));
            let token = fixture
                .issuer
                .issue_account_access_token(
                    fixture.access_input(fixture.now + Duration::seconds(family_seconds)),
                )
                .unwrap_or_else(|_| panic!("ACCOUNT_CRYPTO: valid bounded access issuance failed"));
            let claims = fixture.current_access(token.as_str());
            assert!(
                claims.exp == (fixture.now + Duration::seconds(expected_seconds)).unix_timestamp()
            );
        }
    }

    #[test]
    fn csrf_issuance_respects_five_minute_and_family_ceilings() {
        for (family_seconds, expected_seconds) in [(3600, 300), (17, 17)] {
            let fixture = Fixture::new(Duration::minutes(15));
            let token = fixture
                .issuer
                .issue_account_csrf_token(
                    fixture.csrf_input(fixture.now + Duration::seconds(family_seconds)),
                )
                .unwrap_or_else(|_| panic!("ACCOUNT_CRYPTO: valid bounded proof issuance failed"));
            let claims = fixture
                .verifier
                .verify_account_csrf_token(token.as_str(), fixture.now)
                .unwrap_or_else(|_| panic!("ACCOUNT_CRYPTO: bounded proof refused"));
            assert!(
                claims.exp == (fixture.now + Duration::seconds(expected_seconds)).unix_timestamp()
            );
        }
    }

    #[test]
    fn issuance_rejects_zero_ttl_expired_ceiling_invalid_identity_and_generation() {
        for ttl in [Duration::ZERO, Duration::seconds(-1)] {
            let fixture = Fixture::new(ttl);
            assert!(
                fixture
                    .issuer
                    .issue_account_access_token(
                        fixture.access_input(fixture.now + Duration::minutes(1))
                    )
                    .is_err()
            );
        }
        let fixture = Fixture::new(Duration::minutes(15));
        for seconds in [0, -1] {
            let ceiling = fixture.now + Duration::seconds(seconds);
            assert!(
                fixture
                    .issuer
                    .issue_account_access_token(fixture.access_input(ceiling))
                    .is_err()
            );
            assert!(
                fixture
                    .issuer
                    .issue_account_csrf_token(fixture.csrf_input(ceiling))
                    .is_err()
            );
        }
        for invalid in 0..5 {
            let mut access = fixture.access_input(fixture.now + Duration::minutes(1));
            let mut csrf = fixture.csrf_input(fixture.now + Duration::minutes(1));
            match invalid {
                0 => {
                    access.account_id = Uuid::nil();
                    csrf.account_id = Uuid::nil();
                }
                1 => {
                    access.session_id = Uuid::nil();
                    csrf.session_id = Uuid::nil();
                }
                2 => {
                    access.security_generation = 0;
                    csrf.security_generation = 0;
                }
                3 => {
                    access.security_generation = -1;
                    csrf.security_generation = -1;
                }
                _ => {
                    access.issued_at = OffsetDateTime::UNIX_EPOCH - Duration::seconds(1);
                    csrf.issued_at = access.issued_at;
                }
            }
            assert!(
                fixture.issuer.issue_account_access_token(access).is_err(),
                "ACCOUNT_CRYPTO: invalid access issuance input accepted"
            );
            assert!(
                fixture.issuer.issue_account_csrf_token(csrf).is_err(),
                "ACCOUNT_CRYPTO: invalid proof issuance input accepted"
            );
        }
        let mut access = fixture.access_input(fixture.now + Duration::minutes(1));
        access.auth_time = fixture.now + Duration::seconds(1);
        assert!(
            fixture.issuer.issue_account_access_token(access).is_err(),
            "ACCOUNT_CRYPTO: future primary time accepted"
        );
    }

    #[test]
    fn empty_configured_issuer_never_enables_account_or_csrf_tokens() {
        let empty = Fixture::with_issuer(Duration::minutes(15), "");
        let ceiling = empty.now + Duration::minutes(1);
        assert!(
            empty
                .issuer
                .issue_account_access_token(empty.access_input(ceiling))
                .is_err()
        );
        assert!(
            empty
                .issuer
                .issue_account_csrf_token(empty.csrf_input(ceiling))
                .is_err()
        );
        let valid = Fixture::new(Duration::minutes(15));
        for protocol in Protocol::ALL {
            let mut claims = valid.issued_value(protocol);
            claims["iss"] = json!("");
            let token = empty.sign(&claims);
            assert!(
                matches!(
                    empty.verify(protocol, &token),
                    Err(AuthError::InvalidStoredData(_))
                ),
                "ACCOUNT_CRYPTO: empty configured issuer did not return safe configuration error"
            );
        }
    }

    #[test]
    fn lifetime_arithmetic_near_representable_time_limit_does_not_overflow() {
        let mut fixture = Fixture::new(Duration::seconds(i64::MAX));
        fixture.now = OffsetDateTime::from_unix_timestamp(253_402_300_790).unwrap();
        let ceiling = OffsetDateTime::from_unix_timestamp(253_402_300_798).unwrap();
        let access = fixture
            .issuer
            .issue_account_access_token(fixture.access_input(ceiling))
            .unwrap_or_else(|_| panic!("ACCOUNT_CRYPTO: representable bounded access rejected"));
        let csrf = fixture
            .issuer
            .issue_account_csrf_token(fixture.csrf_input(ceiling))
            .unwrap_or_else(|_| panic!("ACCOUNT_CRYPTO: representable bounded proof rejected"));
        let access = fixture.current_access(access.as_str());
        let csrf = fixture
            .verifier
            .verify_account_csrf_token(csrf.as_str(), fixture.now)
            .unwrap_or_else(|_| panic!("ACCOUNT_CRYPTO: bounded proof near time limit refused"));
        assert!(access.exp == ceiling.unix_timestamp() && csrf.exp == ceiling.unix_timestamp());
    }

    #[test]
    fn account_csrf_and_legacy_parsers_are_separate_in_both_directions() {
        let fixture = Fixture::new(Duration::minutes(15));
        let access = fixture.issued(Protocol::Access);
        let csrf = fixture.issued(Protocol::Csrf);
        assert!(fixture.verify(Protocol::Access, access.as_str()).is_ok());
        assert!(fixture.verify(Protocol::Csrf, csrf.as_str()).is_ok());
        assert!(fixture.verify(Protocol::Csrf, access.as_str()).is_err());
        assert!(fixture.verify(Protocol::Access, csrf.as_str()).is_err());
        assert!(
            fixture
                .verifier
                .verify_access_token(access.as_str())
                .is_err()
        );
        assert!(fixture.verifier.verify_access_token(csrf.as_str()).is_err());
        let legacy = fixture.legacy_token();
        assert!(
            fixture.verifier.verify_access_token(&legacy).is_ok(),
            "ACCOUNT_CRYPTO_CONTROL: ordinary legacy token refused"
        );
        assert!(fixture.verify(Protocol::Access, &legacy).is_err());
        assert!(fixture.verify(Protocol::Csrf, &legacy).is_err());
    }

    #[test]
    fn both_protocols_require_exact_issuer_audience_and_kind() {
        let fixture = Fixture::new(Duration::minutes(15));
        for protocol in Protocol::ALL {
            let valid = fixture.issued_value(protocol);
            for (key, bad) in [
                ("iss", json!("another-test-issuer")),
                ("aud", json!("console-api")),
                ("aud", json!([protocol.audience(), "console-api"])),
                ("aud", json!(null)),
                ("token_kind", json!("unregistered_kind")),
                ("token_kind", json!(null)),
                ("token_kind", json!(1)),
            ] {
                let mut claims = valid.clone();
                claims[key] = bad;
                fixture.reject(protocol, &claims, key);
            }
        }
    }

    #[test]
    fn both_protocols_reject_missing_extra_and_raw_duplicate_claims() {
        let fixture = Fixture::new(Duration::minutes(15));
        for protocol in Protocol::ALL {
            let valid = fixture.issued_value(protocol);
            for key in protocol.required() {
                let mut missing = valid.clone();
                missing.as_object_mut().unwrap().remove(*key);
                fixture.reject(protocol, &missing, "missing required claim");
                let duplicate = DuplicateClaim {
                    claims: &valid,
                    key,
                    repeated: valid[*key].clone(),
                };
                let raw = serde_json::to_string(&duplicate).unwrap();
                let member = format!("\"{key}\":");
                assert!(
                    raw.matches(&member).count() == 2,
                    "ACCOUNT_CRYPTO_CONTROL: duplicate input collapsed"
                );
                let token = fixture.sign(&duplicate);
                assert!(
                    fixture.verify(protocol, &token).is_err(),
                    "ACCOUNT_CRYPTO: identical duplicate claim accepted"
                );
            }
            for key in ["org", "roles", "platform", "name", "unexpected"] {
                let mut extra = valid.clone();
                extra[key] = json!(SECRET_MARKER);
                fixture.reject(protocol, &extra, "unexpected claim");
            }
        }
    }

    #[test]
    fn identity_generation_and_assurance_claims_have_closed_canonical_values() {
        let fixture = Fixture::new(Duration::minutes(15));
        for protocol in Protocol::ALL {
            let valid = fixture.issued_value(protocol);
            for key in ["sub", "sid"] {
                for bad in [
                    json!(Uuid::nil().to_string()),
                    json!(ACCOUNT.to_uppercase()),
                    json!("invalid"),
                    json!(null),
                    json!(7),
                ] {
                    let mut claims = valid.clone();
                    claims[key] = bad;
                    fixture.reject(protocol, &claims, "noncanonical identity");
                }
            }
            for bad in [
                json!("0"),
                json!("-1"),
                json!("01"),
                json!("+1"),
                json!(" 1"),
                json!("9223372036854775808"),
                json!(1),
                json!(1.5),
                json!(null),
            ] {
                let mut claims = valid.clone();
                claims["security_generation"] = bad;
                fixture.reject(protocol, &claims, "noncanonical generation");
            }
            let mut max = valid.clone();
            max["security_generation"] = json!(i64::MAX.to_string());
            assert!(
                fixture.verify(protocol, &fixture.sign(&max)).is_ok(),
                "ACCOUNT_CRYPTO: valid maximum bigint generation refused"
            );
        }
        let valid = fixture.issued_value(Protocol::Access);
        for (key, bad) in [
            ("jti", json!(Uuid::nil().to_string())),
            ("jti", json!(ACCOUNT.to_uppercase())),
            ("assurance", json!("EMPLOYER_ADMIN")),
            ("assurance", json!(true)),
        ] {
            let mut claims = valid.clone();
            claims[key] = bad;
            fixture.reject(Protocol::Access, &claims, "invalid access metadata");
        }
    }

    #[test]
    fn time_validation_has_no_grace_and_enforces_signed_lifetime_and_primary_age() {
        let fixture = Fixture::new(Duration::minutes(15));
        let now = fixture.now.unix_timestamp();
        for protocol in Protocol::ALL {
            let valid = fixture.issued_value(protocol);
            let mut boundary = valid.clone();
            boundary["exp"] = json!(now + 1);
            let token = fixture.sign(&boundary);
            assert!(fixture.verify(protocol, &token).is_ok());
            let exact_expiry = fixture.now + Duration::seconds(1);
            match protocol {
                Protocol::Access => assert!(
                    matches!(
                        fixture
                            .verifier
                            .verify_account_access_token(&token, exact_expiry),
                        Ok(AccountAccessVerification::Expired(_))
                    ),
                    "ACCOUNT_CRYPTO: access expiry classification did not retain verified claims"
                ),
                Protocol::Csrf => assert!(
                    matches!(
                        fixture.verifier.verify_account_csrf_token(&token, exact_expiry),
                        Err(AuthError::Jwt(error)) if matches!(error.kind(), jsonwebtoken::errors::ErrorKind::ExpiredSignature)
                    ),
                    "ACCOUNT_CRYPTO: proof expiry was not the fixed expiry error"
                ),
            }
            for (key, bad) in [
                ("iat", json!(now + 1)),
                ("nbf", json!(now + 1)),
                ("exp", json!(now)),
                ("iat", json!(-1)),
                ("nbf", json!(-1)),
                ("exp", json!(-1)),
                ("iat", json!(1.5)),
                ("nbf", json!("1")),
            ] {
                let mut claims = valid.clone();
                claims[key] = bad;
                fixture.reject(protocol, &claims, "invalid claim time");
            }
            let mut overlong = valid.clone();
            overlong["exp"] = json!(now + protocol.max_seconds() + 1);
            fixture.reject(
                protocol,
                &overlong,
                "signed lifetime exceeds protocol maximum",
            );
            let mut impossible_window = valid.clone();
            impossible_window["iat"] = json!(now - 120);
            impossible_window["nbf"] = json!(now - 10);
            impossible_window["exp"] = json!(now - 20);
            fixture.reject(
                protocol,
                &impossible_window,
                "not-before after expired window",
            );
        }
        let valid = fixture.issued_value(Protocol::Access);
        for bad in [json!(now + 1), json!(-1), json!("1"), json!(null)] {
            let mut claims = valid.clone();
            claims["auth_time"] = bad;
            fixture.reject(
                Protocol::Access,
                &claims,
                "invalid primary authentication time",
            );
        }
    }

    #[test]
    fn expired_access_preserves_verified_claims_without_skipping_other_validation() {
        let fixture = Fixture::new(Duration::minutes(15));
        let now = fixture.now.unix_timestamp();
        let mut expired = fixture.issued_value(Protocol::Access);
        expired["iat"] = json!(now - 120);
        expired["nbf"] = json!(now - 120);
        expired["auth_time"] = json!(now - 180);
        expired["exp"] = json!(now - 1);
        let token = fixture.sign(&expired);
        let outcome = fixture
            .verifier
            .verify_account_access_token(&token, fixture.now)
            .unwrap_or_else(|_| panic!("ACCOUNT_CRYPTO: otherwise valid expired access refused"));
        let shown = format!("{outcome:?}");
        assert!(
            !shown.contains(ACCOUNT) && !shown.contains(FAMILY) && !shown.contains(ISSUER),
            "ACCOUNT_CRYPTO: expiry outcome Debug exposed claims"
        );
        let claims = match outcome {
            AccountAccessVerification::Expired(claims) => claims,
            AccountAccessVerification::Current(_) => {
                panic!("ACCOUNT_CRYPTO: expired access classified current")
            }
        };
        assert!(
            claims.sub == Uuid::parse_str(ACCOUNT).unwrap()
                && claims.sid == Uuid::parse_str(FAMILY).unwrap()
        );
        assert!(claims.security_generation == 7 && claims.exp == now - 1);
        // No live generation/state/family exists here. These retained values are
        // the later session owner's inputs, never permission for fallback.
        for (key, bad) in [
            ("iss", json!("wrong-issuer")),
            ("aud", json!(CSRF_AUD)),
            ("token_kind", json!("account_csrf_v1")),
            ("security_generation", json!("0")),
            ("sub", json!(Uuid::nil().to_string())),
            ("iat", json!(now + 1)),
            ("nbf", json!(now + 1)),
            ("auth_time", json!(now - 100)),
        ] {
            let mut changed = expired.clone();
            changed[key] = bad;
            fixture.reject(Protocol::Access, &changed, "invalid expired access");
        }
        let mut impossible_window = expired.clone();
        impossible_window["nbf"] = json!(now - 10);
        impossible_window["exp"] = json!(now - 20);
        fixture.reject(
            Protocol::Access,
            &impossible_window,
            "not-before after expiry",
        );
        let other = Fixture::new(Duration::minutes(15));
        assert!(
            matches!(fixture.verifier.verify_account_access_token(&other.sign(&expired), fixture.now),
            Err(AuthError::Jwt(error)) if matches!(error.kind(), jsonwebtoken::errors::ErrorKind::InvalidToken)),
            "ACCOUNT_CRYPTO: expired access bypassed signature validation"
        );
    }

    #[test]
    fn wrong_keys_hmac_payload_tampering_and_malformed_tokens_are_refused() {
        let fixture = Fixture::new(Duration::minutes(15));
        let other = Fixture::new(Duration::minutes(15));
        for protocol in Protocol::ALL {
            let valid = fixture.issued_value(protocol);
            let wrong_key = other.sign(&valid);
            assert!(
                fixture.verify(protocol, &wrong_key).is_err(),
                "ACCOUNT_CRYPTO: wrong ES256 key accepted"
            );
            let hmac = encode(
                &Header::new(Algorithm::HS256),
                &valid,
                &EncodingKey::from_secret(b"TEST_ONLY_HMAC_KEY"),
            )
            .unwrap_or_else(|_| panic!("ACCOUNT_CRYPTO_CONTROL: HMAC fixture encoding failed"));
            assert!(
                fixture.verify(protocol, &hmac).is_err(),
                "ACCOUNT_CRYPTO: HS256 accepted"
            );
            let original = fixture.sign(&valid);
            let mut changed = valid.clone();
            changed["security_generation"] = json!("8");
            let changed = fixture.sign(&changed);
            let original_parts: Vec<_> = original.split('.').collect();
            let changed_parts: Vec<_> = changed.split('.').collect();
            let tampered = format!(
                "{}.{}.{}",
                original_parts[0], changed_parts[1], original_parts[2]
            );
            assert!(
                fixture.verify(protocol, &tampered).is_err(),
                "ACCOUNT_CRYPTO: changed payload accepted with old signature"
            );
            for malformed in ["", "not-a-jwt", "a.b.c", "a.b.c.d"] {
                assert!(
                    fixture.verify(protocol, malformed).is_err(),
                    "ACCOUNT_CRYPTO: malformed token accepted"
                );
            }
        }
    }

    #[test]
    fn signed_token_debug_and_invalid_claim_errors_do_not_echo_secrets() {
        let fixture = Fixture::new(Duration::minutes(15));
        for protocol in Protocol::ALL {
            let token = fixture.issued(protocol);
            let debug = format!("{token:?} {token:#?}");
            assert!(
                !debug.contains(token.as_str()),
                "ACCOUNT_CRYPTO: token Debug exposed full credential"
            );
            for part in token.as_str().split('.') {
                assert!(
                    !debug.contains(part),
                    "ACCOUNT_CRYPTO: token Debug exposed encoded credential segment"
                );
            }
            let shown_claims = match protocol {
                Protocol::Access => {
                    let outcome = fixture
                        .verifier
                        .verify_account_access_token(token.as_str(), fixture.now)
                        .unwrap_or_else(|_| {
                            panic!("ACCOUNT_CRYPTO_CONTROL: debug access verification failed")
                        });
                    let outcome_debug = format!("{outcome:?}");
                    let claims = match outcome {
                        AccountAccessVerification::Current(claims) => claims,
                        AccountAccessVerification::Expired(_) => {
                            panic!("ACCOUNT_CRYPTO_CONTROL: debug access expired")
                        }
                    };
                    format!("{outcome_debug} {claims:?}")
                }
                Protocol::Csrf => {
                    let claims = fixture
                        .verifier
                        .verify_account_csrf_token(token.as_str(), fixture.now)
                        .unwrap_or_else(|_| {
                            panic!("ACCOUNT_CRYPTO_CONTROL: debug proof verification failed")
                        });
                    format!("{claims:?}")
                }
            };
            assert!(
                !shown_claims.contains(ACCOUNT)
                    && !shown_claims.contains(FAMILY)
                    && !shown_claims.contains(ISSUER),
                "ACCOUNT_CRYPTO: claim/outcome Debug exposed claim values"
            );
            // Both a malformed known value and a hostile unknown member name
            // must lose their input-bearing parser diagnostics at this boundary.
            for (field, value) in [
                ("assurance", json!(SECRET_MARKER)),
                (SECRET_MARKER, json!("test-only-value")),
            ] {
                let mut claims = fixture.issued_value(protocol);
                claims[field] = value;
                let malformed = fixture.sign(&claims);
                let error = match fixture.verify(protocol, &malformed) {
                    Ok(()) => panic!("ACCOUNT_CRYPTO: malformed private claim accepted"),
                    Err(error) => error,
                };
                assert!(
                    matches!(&error, AuthError::Jwt(error) if matches!(error.kind(), jsonwebtoken::errors::ErrorKind::InvalidToken)),
                    "ACCOUNT_CRYPTO: hostile claim did not return fixed InvalidToken"
                );
                let shown = format!("{error} {error:?}");
                assert!(
                    !shown.contains(SECRET_MARKER),
                    "ACCOUNT_CRYPTO: error echoed hostile claim name or value"
                );
                assert!(
                    !shown.contains(&malformed),
                    "ACCOUNT_CRYPTO: error echoed token"
                );
                for part in malformed.split('.') {
                    assert!(
                        !shown.contains(part),
                        "ACCOUNT_CRYPTO: error echoed credential segment"
                    );
                }
            }
        }
    }
}
