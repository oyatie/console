//! Proposed cfg(test) child of platform/auth/src/jwt.rs.
//! Pure issuer shape tests only: synthetic RefreshTokenIssue is never live authority.
use super::*;
use crate::{RefreshTokenIssue, refresh::RefreshToken};
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use p256::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use time::OffsetDateTime;

struct Fixture {
    issuer: JwtIssuer,
    now: OffsetDateTime,
    family: RefreshTokenIssue,
}
impl Fixture {
    fn new() -> Self {
        let key = SigningKey::random(&mut OsRng);
        let issuer = JwtIssuer::from_es256_pem(
            JwtSettings {
                issuer: "console-platform-auth".into(),
                audience: "console-api".into(),
                access_token_ttl: Duration::minutes(15),
            },
            key.to_pkcs8_pem(LineEnding::LF).unwrap().as_bytes(),
            key.verifying_key()
                .to_public_key_pem(LineEnding::LF)
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        let now = OffsetDateTime::now_utc();
        let family = RefreshTokenIssue {
            token: RefreshToken("pure-shape-fixture-not-authority".into()),
            family_id: Uuid::new_v4(),
            token_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            org_id: OrgId::platform(),
            expires_at: now + Duration::hours(1),
        };
        Self {
            issuer,
            now,
            family,
        }
    }
    fn input(&self) -> AccessTokenInput {
        AccessTokenInput {
            subject: UserId::from_uuid(self.family.user_id),
            org_id: OrgId::platform(),
            roles: vec!["SUPER_ADMIN".into()],
            branches: vec![],
            platform: true,
            view_as: false,
            read_only: false,
            display_name: Some("Platform operator".into()),
            feature_grants: vec![],
            authz_subject_version: 3,
            authz_policy_version: 5,
            session_generation: 7,
            issued_at: self.now,
        }
    }
}

#[test]
fn platform_family_issuer_preserves_existing_codec_and_old_apis() {
    let f = Fixture::new();
    let token = f
        .issuer
        .issue_platform_access_token_for_family(f.input(), vec!["GROUP_FINANCE".into()], &f.family)
        .unwrap();
    let c = f.issuer.verify_access_token(&token).unwrap();
    let b = c.legacy_session.as_ref().unwrap();
    assert!(
        b.version == 1
            && b.family_id == f.family.family_id
            && b.home_org == *OrgId::platform().as_uuid()
            && b.kind == LegacySessionKind::Direct
    );
    assert!(
        c.sub == f.family.user_id.to_string()
            && c.org == OrgId::platform().to_string()
            && c.platform
            && !c.view_as
            && !c.read_only
            && c.tenant_context.is_none()
    );
    assert!(
        c.group_roles == vec!["GROUP_FINANCE"]
            && c.authz_subject_version == 3
            && c.authz_policy_version == 5
            && c.session_generation == 7
            && c.alg == "ES256"
    );
    assert!(
        c.iat == f.now.unix_timestamp()
            && c.nbf == c.iat
            && c.exp == (f.now + Duration::minutes(15)).unix_timestamp()
    );
    for old in [
        f.issuer.issue_access_token(f.input()).unwrap(),
        f.issuer
            .issue_access_token_with_group_roles(f.input(), vec!["GROUP_FINANCE".into()])
            .unwrap(),
    ] {
        let claims = f.issuer.verify_access_token(&old).unwrap();
        assert!(
            claims.legacy_session.is_none(),
            "old API must not fabricate a family"
        );
        assert!(
            serde_json::to_value(claims)
                .unwrap()
                .get("legacy_session")
                .is_none()
        );
    }
}

#[test]
fn platform_family_issuer_refuses_wrong_correlation_scope_and_expiry() {
    type Fault = (
        &'static str,
        fn(&mut AccessTokenInput, &mut RefreshTokenIssue, &mut Vec<String>),
    );
    let faults: [Fault; 9] = [
        ("nil family", |_, f, _| f.family_id = Uuid::nil()),
        ("other subject", |_, f, _| f.user_id = Uuid::new_v4()),
        ("other family Company", |_, f, _| f.org_id = OrgId::knl()),
        ("matching nonplatform Company", |i, f, _| {
            i.org_id = OrgId::knl();
            i.platform = false;
            f.org_id = OrgId::knl();
        }),
        ("missing platform marker", |i, _, _| i.platform = false),
        ("view as", |i, _, _| i.view_as = true),
        ("read only", |i, _, _| i.read_only = true),
        ("expiry boundary", |i, f, _| f.expires_at = i.issued_at),
        ("unknown group role", |_, _, g| {
            g.push("UNREGISTERED".into())
        }),
    ];
    for (name, mutate) in faults {
        let f = Fixture::new();
        let mut input = f.input();
        let mut family = f.family.clone();
        let mut groups = vec![];
        mutate(&mut input, &mut family, &mut groups);
        assert!(
            f.issuer
                .issue_platform_access_token_for_family(input, groups, &family)
                .is_err(),
            "issuer accepted {name}"
        );
    }
    let f = Fixture::new();
    let mut expired = f.family.clone();
    expired.expires_at = f.now - Duration::seconds(1);
    assert!(
        f.issuer
            .issue_platform_access_token_for_family(f.input(), vec![], &expired)
            .is_err()
    );
}
