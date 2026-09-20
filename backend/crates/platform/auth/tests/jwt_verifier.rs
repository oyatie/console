#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use console_kernel_core::{BranchId, OrgId, UserId};
use console_platform_auth::{AccessTokenInput, JwtIssuer, JwtSettings, JwtVerifier};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use p256::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use time::{Duration, OffsetDateTime};

#[test]
fn public_key_verifier_accepts_es256_access_token() {
    let (issuer, verifier, _) = es256_fixture();

    let user_id = UserId::new();
    let branch_id = BranchId::new();
    let token = issuer
        .issue_access_token(AccessTokenInput {
            subject: user_id,
            org_id: OrgId::knl(),
            roles: vec!["ADMIN".to_owned()],
            branches: vec![branch_id],
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

    let claims = verifier.verify_access_token(&token).unwrap();

    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.roles, vec!["ADMIN"]);
    assert_eq!(claims.branches, vec![branch_id.to_string()]);
}

// Share the existing test key/configuration setup without changing its assertions.
fn es256_fixture() -> (JwtIssuer, JwtVerifier, EncodingKey) {
    let signing_key = SigningKey::random(&mut OsRng);
    let private_pem = signing_key.to_pkcs8_pem(LineEnding::LF).unwrap();
    let public_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let settings = JwtSettings {
        issuer: "console-platform-auth".to_owned(),
        audience: "console-api".to_owned(),
        access_token_ttl: Duration::minutes(15),
    };
    let issuer = JwtIssuer::from_es256_pem(
        settings.clone(),
        private_pem.as_bytes(),
        public_pem.as_bytes(),
    )
    .unwrap();
    let verifier = JwtVerifier::from_es256_public_pem(settings, public_pem.as_bytes()).unwrap();

    let encoding_key = EncodingKey::from_ec_pem(private_pem.as_bytes()).unwrap();
    (issuer, verifier, encoding_key)
}

// AS1/BW31: these are cryptographic parser tests, not live Account sessions.
mod account_protocol_separation {
    use super::*;
    use serde_json::Value;

    struct Fixture {
        issuer: JwtIssuer,
        verifier: JwtVerifier,
        signing_key: EncodingKey,
    }

    impl Fixture {
        fn new() -> Self {
            let (issuer, verifier, signing_key) = es256_fixture();
            Self {
                issuer,
                verifier,
                signing_key,
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
                    issued_at: OffsetDateTime::now_utc(),
                })
                .unwrap_or_else(|_| panic!("ACCOUNT_PROTOCOL_CONTROL: legacy issuance failed"))
        }

        fn verified_legacy_value(&self, token: &str) -> Value {
            let issuer_claims = self.issuer.verify_access_token(token).unwrap_or_else(|_| {
                panic!("ACCOUNT_PROTOCOL_CONTROL: issuer refused valid legacy")
            });
            let public_claims = self
                .verifier
                .verify_access_token(token)
                .unwrap_or_else(|_| {
                    panic!("ACCOUNT_PROTOCOL_CONTROL: verifier refused valid legacy")
                });
            assert!(
                issuer_claims == public_claims,
                "ACCOUNT_PROTOCOL_CONTROL: existing verifiers disagree"
            );
            let value = serde_json::to_value(issuer_claims).unwrap();
            assert!(value.get("token_kind").is_none());
            assert!(value["aud"] == "console-api");
            assert!(value["org"] == OrgId::knl().to_string());
            assert!(value["roles"] == serde_json::json!(["MEMBER"]));
            value
        }
    }

    #[test]
    fn unmarked_legacy_token_remains_valid_in_both_verifiers() {
        let fixture = Fixture::new();
        let legacy = fixture.legacy_token();
        let _ = fixture.verified_legacy_value(&legacy);
    }

    fn assert_recognized_account_marker_is_refused(token_kind: &'static str) {
        let fixture = Fixture::new();
        // Validate the exact issuer, key, audience, tenant shape and current times
        // before changing only the recognized Account protocol marker.
        let legacy = fixture.legacy_token();
        let mut claims = fixture.verified_legacy_value(&legacy);
        let resigned_control = encode(
            &Header::new(Algorithm::ES256),
            &claims,
            &fixture.signing_key,
        )
        .unwrap_or_else(|_| panic!("ACCOUNT_PROTOCOL_CONTROL: control signing failed"));
        let _ = fixture.verified_legacy_value(&resigned_control);
        claims["token_kind"] = Value::String(token_kind.to_owned());
        let marked = encode(
            &Header::new(Algorithm::ES256),
            &claims,
            &fixture.signing_key,
        )
        .unwrap_or_else(|_| panic!("ACCOUNT_PROTOCOL_CONTROL: test signing failed"));

        // Evaluate both actual existing entry points before asserting, so failure
        // evidence identifies each parser without printing a credential or claims.
        let issuer_accepted = fixture.issuer.verify_access_token(&marked).is_ok();
        let verifier_accepted = fixture.verifier.verify_access_token(&marked).is_ok();
        assert!(
            !issuer_accepted && !verifier_accepted,
            "ACCOUNT_PROTOCOL_SEPARATION: recognized {token_kind} entered legacy parser; issuer_accepted={issuer_accepted}; verifier_accepted={verifier_accepted}"
        );

        // Refusing the marked token must not alter normal legacy verification.
        let _ = fixture.verified_legacy_value(&legacy);
    }

    #[test]
    fn account_access_marker_cannot_enter_legacy_parser_with_legacy_shaped_claims() {
        assert_recognized_account_marker_is_refused("account_v1");
    }

    #[test]
    fn account_csrf_marker_cannot_enter_legacy_parser_with_legacy_shaped_claims() {
        assert_recognized_account_marker_is_refused("account_csrf_v1");
    }
}

// Supplemental wire-compatibility controls; keep the separately admitted
// account_protocol_separation:: three-test probe unchanged.
mod legacy_protocol_compatibility {
    use super::*;
    use serde::ser::SerializeMap;
    use serde::{Serialize, Serializer};
    use serde_json::{Value, json};

    // Serialize duplicate keys directly into the signed JSON payload. A Value
    // roundtrip here would erase the condition these tests must exercise.
    struct DuplicateClaim<'a> {
        claims: &'a Value,
        name: &'a str,
        value: &'a Value,
        extra_first: bool,
    }

    impl Serialize for DuplicateClaim<'_> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let claims = self
                .claims
                .as_object()
                .expect("legacy fixture is an object");
            let mut map = serializer.serialize_map(Some(claims.len() + 1))?;
            if self.extra_first {
                map.serialize_entry(self.name, self.value)?;
            }
            for (name, value) in claims {
                map.serialize_entry(name, value)?;
            }
            if !self.extra_first {
                map.serialize_entry(self.name, self.value)?;
            }
            map.end()
        }
    }

    fn acceptance(issuer: &JwtIssuer, verifier: &JwtVerifier, token: &str) -> (bool, bool) {
        let issuer_accepted = issuer.verify_access_token(token).is_ok();
        let verifier_accepted = verifier.verify_access_token(token).is_ok();
        (issuer_accepted, verifier_accepted)
    }

    fn control() -> (JwtIssuer, JwtVerifier, EncodingKey, Value) {
        let (issuer, verifier, key) = es256_fixture();
        let now = OffsetDateTime::now_utc();
        let claims = json!({
            "iss": "console-platform-auth", "aud": "console-api",
            "sub": UserId::new().to_string(), "org": OrgId::knl().to_string(),
            "iat": now.unix_timestamp(), "nbf": now.unix_timestamp(),
            "exp": (now + Duration::minutes(15)).unix_timestamp(),
            "jti": uuid::Uuid::new_v4().to_string(), "alg": "ES256",
            "roles": ["MEMBER"], "branches": [],
            "scope_level": "org", "scope_node": OrgId::knl().to_string()
        });
        let token = encode(&Header::new(Algorithm::ES256), &claims, &key).unwrap();
        assert!(
            acceptance(&issuer, &verifier, &token) == (true, true),
            "LEGACY_COMPATIBILITY_CONTROL: same-signer scoped legacy token refused"
        );
        (issuer, verifier, key, claims)
    }

    #[test]
    fn duplicate_known_claims_remain_refused_in_both_verifiers() {
        let (issuer, verifier, key, claims) = control();
        let mut admitted = Vec::new();
        for name in ["org", "roles", "scope_level", "scope_node"] {
            let duplicate = DuplicateClaim {
                claims: &claims,
                name,
                value: &claims[name],
                extra_first: false,
            };
            let token = encode(&Header::new(Algorithm::ES256), &duplicate, &key).unwrap();
            let result = acceptance(&issuer, &verifier, &token);
            if result != (false, false) {
                admitted.push((name, result));
            }
        }
        assert!(
            admitted.is_empty(),
            "LEGACY_DUPLICATE_CLAIM: typed duplicate accepted: {admitted:?}"
        );
    }

    #[test]
    fn duplicate_recognized_protocol_markers_are_refused_in_both_orders() {
        let (issuer, verifier, key, mut claims) = control();
        let mut admitted = Vec::new();
        for marker in ["account_v1", "account_csrf_v1"] {
            claims["token_kind"] = json!(marker);
            for (other_name, other, extra_first) in [
                ("null", Value::Null, false),
                ("null", Value::Null, true),
                ("unknown", json!("unrelated_v9"), false),
                ("unknown", json!("unrelated_v9"), true),
                ("same", json!(marker), false),
            ] {
                let duplicate = DuplicateClaim {
                    claims: &claims,
                    name: "token_kind",
                    value: &other,
                    extra_first,
                };
                let token = encode(&Header::new(Algorithm::ES256), &duplicate, &key).unwrap();
                let result = acceptance(&issuer, &verifier, &token);
                if result != (false, false) {
                    admitted.push((marker, other_name, extra_first, result));
                }
            }
        }
        assert!(
            admitted.is_empty(),
            "LEGACY_DUPLICATE_PROTOCOL: recognized marker overwritten or ignored: {admitted:?}"
        );
    }

    #[test]
    fn unrelated_unknown_fields_and_null_protocol_marker_preserve_legacy_compatibility() {
        let (issuer, verifier, key, mut claims) = control();
        claims["unrelated_extension"] = json!({"display_only": true});
        let mut refused = Vec::new();
        for (case, marker) in [("null", Value::Null), ("unknown", json!("unrelated_v9"))] {
            claims["token_kind"] = marker;
            let token = encode(&Header::new(Algorithm::ES256), &claims, &key).unwrap();
            let result = acceptance(&issuer, &verifier, &token);
            if result != (true, true) {
                refused.push((case, result));
            }
        }
        assert!(
            refused.is_empty(),
            "LEGACY_EXTENSION_COMPATIBILITY: unrelated extension refused: {refused:?}"
        );
    }
}

// Contract-only parser prerequisites. These exercise the existing cryptographic
// verifiers; family liveness and current policy belong to separate owner tests.
mod legacy_family_binding {
    use super::*;
    use serde::ser::SerializeMap;
    use serde::{Serialize, Serializer};
    use serde_json::{Value, json};

    struct Fixture {
        issuer: JwtIssuer,
        verifier: JwtVerifier,
        key: EncodingKey,
        claims: Value,
    }

    impl Fixture {
        fn new() -> Self {
            let (issuer, verifier, key) = es256_fixture();
            let now = OffsetDateTime::now_utc();
            let claims = json!({
                "iss": "console-platform-auth", "aud": "console-api",
                "sub": UserId::new().to_string(), "org": OrgId::knl().to_string(),
                "iat": now.unix_timestamp(), "nbf": now.unix_timestamp(),
                "exp": (now + Duration::minutes(15)).unix_timestamp(),
                "jti": uuid::Uuid::new_v4().to_string(), "alg": "ES256",
                "roles": ["MEMBER"], "branches": [],
                "platform": false, "view_as": false, "read_only": false
            });
            let fixture = Self {
                issuer,
                verifier,
                key,
                claims,
            };
            fixture.accepted(&fixture.claims, None);
            fixture
        }

        fn sign(&self, claims: &impl Serialize) -> String {
            encode(&Header::new(Algorithm::ES256), claims, &self.key).unwrap()
        }

        fn accepted(&self, claims: &Value, binding: Option<&Value>) {
            let token = self.sign(claims);
            for verified in [
                self.issuer.verify_access_token(&token),
                self.verifier.verify_access_token(&token),
            ] {
                let value =
                    serde_json::to_value(verified.expect("valid signed control rejected")).unwrap();
                assert!(
                    value.get("legacy_session") == binding,
                    "LEGACY_FAMILY_PARSER: binding was lost or absence changed"
                );
            }
        }

        fn refused(&self, claims: &impl Serialize, label: &str) {
            let token = self.sign(claims);
            let issuer = self.issuer.verify_access_token(&token).is_ok();
            let public = self.verifier.verify_access_token(&token).is_ok();
            assert!(
                !issuer && !public,
                "LEGACY_FAMILY_PARSER: {label}; issuer_accepted={issuer}; public_accepted={public}"
            );
        }

        fn bound(&self) -> Value {
            let mut claims = self.claims.clone();
            claims["legacy_session"] = json!({
                "version": 1, "family_id": uuid::Uuid::new_v4().to_string(),
                "home_org": OrgId::knl().to_string(), "kind": "direct"
            });
            claims
        }
    }

    #[test]
    fn absent_binding_keeps_historical_defaults_and_unknown_outer_claims() {
        let fixture = Fixture::new();
        let mut old = fixture.claims.clone();
        for key in ["platform", "view_as", "read_only"] {
            old.as_object_mut().unwrap().remove(key);
        }
        old["future_display_hint"] = json!({"text": "compatibility"});
        fixture.accepted(&old, None);
        let token = fixture.sign(&old);
        let claims = fixture.verifier.verify_access_token(&token).unwrap();
        assert!(!claims.platform && !claims.view_as && !claims.read_only);
        assert_eq!(claims.authz_subject_version, 0);
        assert_eq!(claims.authz_policy_version, 0);
        assert_eq!(claims.session_generation, 0);
    }

    #[test]
    fn valid_binding_is_retained_for_all_four_closed_kinds() {
        let fixture = Fixture::new();
        for kind in [
            "direct",
            "group_admin",
            "platform_view_as",
            "platform_tenant_context",
        ] {
            let mut claims = fixture.bound();
            claims["legacy_session"]["kind"] = json!(kind);
            match kind {
                "group_admin" => {
                    let home = uuid::Uuid::new_v4().to_string();
                    claims["legacy_session"]["home_org"] = json!(home);
                    claims["actor_home_org"] = json!(home);
                    claims["tenant_context"] = json!("group_admin");
                    claims["group_context_id"] = json!(uuid::Uuid::new_v4().to_string());
                    claims["group_roles"] = json!(["GROUP_ADMIN"]);
                    claims["roles"] = json!(["ADMIN"]);
                }
                "platform_view_as" | "platform_tenant_context" => {
                    claims["legacy_session"]["home_org"] = json!(OrgId::platform().to_string());
                    let read_only = kind == "platform_view_as";
                    claims["view_as"] = json!(read_only);
                    claims["read_only"] = json!(read_only);
                    claims["roles"] = if read_only {
                        json!(["ADMIN"])
                    } else {
                        json!(["SUPER_ADMIN"])
                    };
                }
                _ => {}
            }
            fixture.accepted(&claims, claims.get("legacy_session"));
        }
        let mut platform = fixture.bound();
        platform["org"] = json!(OrgId::platform().to_string());
        platform["platform"] = json!(true);
        platform["roles"] = json!(["SUPER_ADMIN"]);
        platform["legacy_session"]["home_org"] = platform["org"].clone();
        fixture.accepted(&platform, platform.get("legacy_session"));
    }

    #[test]
    fn present_null_partial_extra_and_wrong_typed_binding_are_rejected() {
        let fixture = Fixture::new();
        for invalid in [
            Value::Null,
            json!(false),
            json!(1),
            json!("direct"),
            json!([]),
        ] {
            let mut claims = fixture.bound();
            claims["legacy_session"] = invalid;
            fixture.refused(&claims, "binding must be a complete object");
        }
        for field in ["version", "family_id", "home_org", "kind"] {
            let mut claims = fixture.bound();
            claims["legacy_session"]
                .as_object_mut()
                .unwrap()
                .remove(field);
            fixture.refused(&claims, field);
            claims = fixture.bound();
            claims["legacy_session"][field] = Value::Null;
            fixture.refused(&claims, field);
        }
        let mut extra = fixture.bound();
        extra["legacy_session"]["authority"] = json!("SUPER_ADMIN");
        fixture.refused(&extra, "unknown nested member");
    }

    #[test]
    fn unknown_version_kind_and_invalid_or_nil_identifiers_are_rejected() {
        let fixture = Fixture::new();
        for (field, invalid) in [
            ("version", json!(0)),
            ("version", json!(2)),
            ("version", json!("1")),
            ("version", json!(1.5)),
            ("version", json!(true)),
            ("kind", json!("DIRECT")),
            ("kind", json!("account_v1")),
            ("kind", json!(9)),
            ("family_id", json!("")),
            ("family_id", json!("not-a-uuid")),
            ("family_id", json!(uuid::Uuid::nil().to_string())),
            ("family_id", json!(1)),
            ("home_org", json!("")),
            ("home_org", json!("not-a-uuid")),
            ("home_org", json!(uuid::Uuid::nil().to_string())),
            ("home_org", json!(1)),
        ] {
            let mut claims = fixture.bound();
            claims["legacy_session"][field] = invalid;
            fixture.refused(&claims, field);
        }
    }

    // Explicit map serialization preserves duplicate signed keys. No Value
    // parse/roundtrip is allowed between this serializer and JWT signing.
    struct Duplicate<'a> {
        object: &'a Value,
        key: &'a str,
        extra: &'a Value,
        first: bool,
    }
    impl Serialize for Duplicate<'_> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let object = self.object.as_object().unwrap();
            let mut map = serializer.serialize_map(Some(object.len() + 1))?;
            if self.first {
                map.serialize_entry(self.key, self.extra)?;
            }
            for (key, value) in object {
                map.serialize_entry(key, value)?;
            }
            if !self.first {
                map.serialize_entry(self.key, self.extra)?;
            }
            map.end()
        }
    }
    struct NestedDuplicate<'a> {
        claims: &'a Value,
        duplicate: Duplicate<'a>,
    }
    impl Serialize for NestedDuplicate<'_> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let object = self.claims.as_object().unwrap();
            let mut map = serializer.serialize_map(Some(object.len()))?;
            for (key, value) in object {
                if key == "legacy_session" {
                    map.serialize_entry(key, &self.duplicate)?;
                } else {
                    map.serialize_entry(key, value)?;
                }
            }
            map.end()
        }
    }

    #[test]
    fn repeated_outer_binding_is_rejected_in_both_orders_including_identical_values() {
        let fixture = Fixture::new();
        let claims = fixture.bound();
        for extra in [
            claims["legacy_session"].clone(),
            Value::Null,
            fixture.bound()["legacy_session"].clone(),
        ] {
            for first in [false, true] {
                let duplicate = Duplicate {
                    object: &claims,
                    key: "legacy_session",
                    extra: &extra,
                    first,
                };
                let bytes = serde_json::to_string(&duplicate).unwrap();
                assert_eq!(bytes.matches("\"legacy_session\":").count(), 2);
                fixture.refused(&duplicate, "duplicate outer legacy_session");
            }
        }
    }

    #[test]
    fn repeated_nested_members_are_rejected_in_both_orders_including_identical_values() {
        let fixture = Fixture::new();
        let claims = fixture.bound();
        for key in ["version", "family_id", "home_org", "kind"] {
            for extra in [claims["legacy_session"][key].clone(), Value::Null] {
                for first in [false, true] {
                    let duplicate = NestedDuplicate {
                        claims: &claims,
                        duplicate: Duplicate {
                            object: &claims["legacy_session"],
                            key,
                            extra: &extra,
                            first,
                        },
                    };
                    let bytes = serde_json::to_string(&duplicate).unwrap();
                    assert_eq!(bytes.matches(&format!("\"{key}\":")).count(), 2);
                    fixture.refused(&duplicate, "duplicate nested member");
                }
            }
        }
    }

    #[test]
    fn binding_rejects_contradictory_home_tier_and_derived_flags() {
        let fixture = Fixture::new();
        for (path, value) in [
            (
                "/legacy_session/home_org",
                json!(uuid::Uuid::new_v4().to_string()),
            ),
            ("/platform", json!(true)),
            ("/view_as", json!(true)),
            ("/read_only", json!(true)),
            ("/legacy_session/kind", json!("group_admin")),
            ("/legacy_session/kind", json!("platform_view_as")),
            ("/legacy_session/kind", json!("platform_tenant_context")),
        ] {
            let mut claims = fixture.bound();
            *claims.pointer_mut(path).unwrap() = value;
            fixture.refused(&claims, path);
        }
        let mut platform = fixture.bound();
        platform["org"] = json!(OrgId::platform().to_string());
        platform["legacy_session"]["home_org"] = platform["org"].clone();
        fixture.refused(&platform, "platform sentinel requires platform flag");
    }
}
