//! Security-only expanded TEST_ONLY registered-kind producer; original B5 hash preserved.
//! TEST_ONLY cryptographic custody producer. It creates input bytes, never verified
//! authority values or terms SQL rows. The production release owner must verify
//! every signature, reference and local file before returning its opaque session.
use console_platform_auth::terms_publication as publisher;
use openssl::{
    pkey::{PKey, Private},
    sign::Signer,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn signature(key: &PKey<Private>, bytes: &[u8]) -> Result<Vec<u8>> {
    Ok(Signer::new_without_digest(key)?.sign_oneshot_to_vec(bytes)?)
}
fn immutable(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    use std::os::unix::fs::PermissionsExt;
    let permissions = fs::Permissions::from_mode(0o400);
    file.set_permissions(permissions)?;
    Ok(())
}
pub struct ExpandedCustodyFixture {
    pub directory: tempfile::TempDir,
    pub config_path: PathBuf,
    operator: PKey<Private>,
    content_authority: PKey<Private>,
    pub manifest: Vec<u8>,
    pub content: Vec<u8>,
    pub content_digest: String,
}
pub struct ApprovedPublication {
    pub approval_bytes: Vec<u8>,
    pub approval_digest: String,
    pub manifest_digest: String,
    pub custody_path: PathBuf,
}
impl ExpandedCustodyFixture {
    pub fn new(label: &str, database: &str, kinds: &[String]) -> Result<Self> {
        assert!(database.starts_with("_sqlx_test_"));
        let mut publisher_url =
            url::Url::parse(&std::env::var("CONSOLE_TERMS_PUBLISHER_DATABASE_URL")?)?;
        assert_eq!(publisher_url.username(), "console_terms_publisher");
        assert!(publisher_url.password().is_some());
        assert!(publisher_url.query().is_none() && publisher_url.fragment().is_none());
        publisher_url.set_path(database);
        let directory = tempfile::Builder::new()
            .prefix("console-TEST_ONLY-terms-")
            .tempdir()?;
        let root = directory.path();
        fs::create_dir(root.join("objects"))?;
        fs::create_dir(root.join("approvals"))?;
        fs::create_dir(root.join("authority"))?;
        let operator = PKey::generate_ed25519()?;
        let content_authority = PKey::generate_ed25519()?;
        // Independent trust principals. This is an explicit ephemeral deployment
        // root in test mode, never a content-authority claim for production.
        assert_ne!(
            operator.public_key_to_der()?,
            content_authority.public_key_to_der()?
        );
        immutable(&root.join("operator.pem"), &operator.public_key_to_pem()?)?;
        immutable(
            &root.join("content.pem"),
            &content_authority.public_key_to_pem()?,
        )?;
        let content =
            format!("TEST_ONLY {label}. No production or legal authority.\n").into_bytes();
        let content_digest = digest(&content);
        immutable(&root.join("objects").join(&content_digest), &content)?;
        assert!(!kinds.is_empty());
        let items:Vec<Value>=kinds.iter().map(|kind|json!({"terms_kind":kind,"title":"TEST_ONLY service","locale":"en-US","content_sha256":content_digest,"required":true})).collect();
        let manifest =
            serde_json::to_vec(&json!({"format_version":1,"fixture_only":true,"items":items}))?;
        immutable(&root.join("objects").join(digest(&manifest)), &manifest)?;
        let config = json!({"format_version":1,"environment":"TEST_ONLY",
            "custody_root":root,"publisher_database_url":publisher_url.as_str(),"operator_keys":{"test.operator":"operator.pem"},
            "content_authorities":{"test.content":{"key_path":"content.pem",
                "qualified_terms_kinds":kinds}},
            "registered_terms_kinds":kinds,
            "object_path_template":"objects/{sha256}","approval_path_template":"approvals/{sha256}"});
        let config_path = root.join("deployment.json");
        immutable(&config_path, &serde_json::to_vec(&config)?)?;
        Ok(Self {
            directory,
            config_path,
            operator,
            content_authority,
            manifest,
            content,
            content_digest,
        })
    }
    pub fn approve(&self, expected: i64) -> Result<ApprovedPublication> {
        self.approve_manifest(expected, &self.manifest)
    }
    pub fn approve_manifest(&self, expected: i64, manifest: &[u8]) -> Result<ApprovedPublication> {
        assert!(expected >= 0);
        let manifest_digest = digest(manifest);
        let root = self.directory.path();
        let object = root.join("objects").join(&manifest_digest);
        if !object.exists() {
            immutable(&object, manifest)?;
        }
        let parsed: Value = serde_json::from_slice(manifest)?;
        let kinds = parsed["items"]
            .as_array()
            .ok_or("manifest items")?
            .iter()
            .map(|i| i["terms_kind"].clone())
            .collect::<Vec<_>>();
        let qualification = serde_json::to_vec(&json!({"kind":"TEST_ONLY_CONTENT_QUALIFICATION",
            "manifest_sha256":manifest_digest,"terms_kinds":kinds,
            "authority":"test.content","fixture_only":true}))?;
        let authority_digest = digest(&qualification);
        let authority_path = root.join("authority").join(&authority_digest);
        if !authority_path.exists() {
            immutable(&authority_path, &qualification)?;
            immutable(
                &authority_path.with_extension("sig"),
                &signature(&self.content_authority, &qualification)?,
            )?;
        }
        // Explicit overflow input keeps the closed wire's decimal grammar. The
        // publisher, not this producer, must reject values beyond signed bigint.
        let next = (i128::from(expected) + 1).to_string();
        let approval_bytes =
            serde_json::to_vec(&json!({"kind":"ACCOUNT_TERMS_PUBLICATION_APPROVAL",
            "approval_id":Uuid::new_v4(),"manifest_sha256":manifest_digest,
            "expected_revision":expected.to_string(),"next_revision":next,"fixture_only":true,
            "approved_by":"test.operator","approved_at":"2026-09-13T00:00:00Z",
            "content_authority_refs":[authority_digest]}))?;
        let approval_digest = digest(&approval_bytes);
        let custody_path = root.join("approvals").join(&approval_digest);
        immutable(&custody_path, &approval_bytes)?;
        immutable(
            &custody_path.with_extension("sig"),
            &signature(&self.operator, &approval_bytes)?,
        )?;
        Ok(ApprovedPublication {
            approval_bytes,
            approval_digest,
            manifest_digest,
            custody_path,
        })
    }
    pub async fn authenticate(
        &self,
        environment: publisher::Environment,
    ) -> Result<publisher::AuthenticatedOperator> {
        // Config comes from the privileged deployment command; tenant requests
        // have no load-config route. Real crypto verification is in this owner.
        let deployment = publisher::load_deployment_custody(&self.config_path, environment).await?;
        let challenge =
            publisher::begin_operator_authentication(&deployment, "test.operator").await?;
        let proof = signature(&self.operator, challenge.signing_bytes())?;
        Ok(publisher::finish_operator_authentication(deployment, challenge, &proof).await?)
    }
    pub fn sign_operator_challenge(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        signature(&self.operator, bytes)
    }
    pub fn corrupt_object(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        // Hostile setup is outside serving custody; do not expose this operation
        // as a product writer. The test owns this entire disposable directory.
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        fs::write(path, bytes)?;
        Ok(())
    }
    pub fn manifest_value(&self) -> Result<Value> {
        Ok(serde_json::from_slice(&self.manifest)?)
    }
}
