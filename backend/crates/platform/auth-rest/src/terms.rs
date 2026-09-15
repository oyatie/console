//! Public immutable terms reads. The trusted release index identifies available
//! artifacts; only the restricted Auth projection selects the current release.
//! File serviceability is not publication or qualified content authority.

use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path as FilePath, PathBuf};

use axum::Json;
use axum::extract::{Path, State, rejection::PathRejection};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{AuthRestState, RestError};

pub(super) const CURRENT_PATH: &str = "/api/v2/auth/terms";
pub(super) const MANIFEST_PATH: &str = "/api/v2/auth/terms/manifests/{sha256}";
pub(super) const CONTENT_PATH: &str = "/api/v2/auth/terms/content/{sha256}";
const MANIFEST_LIMIT: u64 = 16 * 1024;
const CONTENT_LIMIT: u64 = 64 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactEntry {
    path: PathBuf,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactIndex {
    manifest: ArtifactEntry,
    content: Vec<ArtifactEntry>,
    // Descriptive release metadata, never permission to publish or activate.
    #[serde(rename = "publication")]
    _publication: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TermsManifest {
    format_version: u8,
    #[serde(rename = "fixture_only")]
    _fixture_only: bool,
    items: Vec<TermsItem>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TermsItem {
    terms_kind: String,
    title: String,
    locale: String,
    content_sha256: String,
    required: bool,
}

#[derive(Serialize)]
struct TermsCurrent {
    terms_version: String,
    terms_revision: String,
    manifest_url: String,
}

pub(super) struct TermsArtifacts {
    root: PathBuf,
    index: ArtifactIndex,
}

impl TermsArtifacts {
    /// Registry metadata is fixed at composition. Missing/malformed metadata
    /// closes this authority without preventing unrelated App construction.
    pub(super) fn load(root: PathBuf) -> Result<Self, RestError> {
        let root = root.canonicalize().map_err(|_| unavailable())?;
        let bytes = read_bounded(
            &root,
            FilePath::new("fixtures/artifact-index.json"),
            MANIFEST_LIMIT,
        )?;
        let index: ArtifactIndex = serde_json::from_slice(&bytes).map_err(|_| unavailable())?;
        if !(1..=8).contains(&index.content.len()) {
            return Err(unavailable());
        }
        let mut paths = BTreeSet::new();
        let mut content_digests = BTreeSet::new();
        for entry in std::iter::once(&index.manifest).chain(&index.content) {
            if !valid_digest(&entry.sha256)
                || !relative_artifact_path(&entry.path)
                || !paths.insert(entry.path.clone())
            {
                return Err(unavailable());
            }
        }
        for entry in &index.content {
            if !content_digests.insert(entry.sha256.clone()) {
                return Err(unavailable());
            }
        }
        Ok(Self { root, index })
    }

    fn read(&self, entry: &ArtifactEntry, limit: u64) -> Result<Vec<u8>, RestError> {
        let bytes = read_bounded(&self.root, &entry.path, limit)?;
        if format!("{:x}", Sha256::digest(&bytes)) != entry.sha256
            || std::str::from_utf8(&bytes).is_err()
        {
            return Err(unavailable());
        }
        Ok(bytes)
    }

    fn manifest(&self) -> Result<(Vec<u8>, TermsManifest), RestError> {
        let bytes = self.read(&self.index.manifest, MANIFEST_LIMIT)?;
        let manifest: TermsManifest = serde_json::from_slice(&bytes).map_err(|_| unavailable())?;
        if manifest.format_version != 1 || !(1..=8).contains(&manifest.items.len()) {
            return Err(unavailable());
        }
        let mut kinds = BTreeSet::new();
        for item in &manifest.items {
            if !valid_kind(&item.terms_kind)
                || !kinds.insert(item.terms_kind.clone())
                || !(1..=128).contains(&item.title.chars().count())
                || !(2..=32).contains(&item.locale.chars().count())
                || !item.required
                || !valid_digest(&item.content_sha256)
                || self.content_entry(&item.content_sha256).is_none()
            {
                return Err(unavailable());
            }
        }
        Ok((bytes, manifest))
    }

    fn content_entry(&self, digest: &str) -> Option<&ArtifactEntry> {
        self.index
            .content
            .iter()
            .find(|entry| entry.sha256 == digest)
    }
}

pub(super) async fn current(State(state): State<AuthRestState>) -> Response {
    private_response(current_metadata(&state).await.map(Json).into_response())
}

async fn current_metadata(state: &AuthRestState) -> Result<TermsCurrent, RestError> {
    let artifacts = state.terms_artifacts.clone().ok_or_else(unavailable)?;
    let auth = state.auth_database.as_ref().ok_or_else(unavailable)?;
    // This exact projection is the only database read. Never substitute the
    // Company pool, cached revision or local index when Auth is unavailable.
    let (digest, revision): (Vec<u8>, i64) =
        sqlx::query_as("SELECT manifest_sha256, revision FROM public.account_terms_current_v1()")
            .fetch_optional(auth)
            .await
            .map_err(|_| unavailable())?
            .ok_or_else(unavailable)?;
    if digest.len() != 32 || revision <= 0 {
        return Err(unavailable());
    }
    let digest: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    if digest != artifacts.index.manifest.sha256 {
        return Err(unavailable());
    }
    tokio::task::spawn_blocking(move || -> Result<(), RestError> {
        let (_, manifest) = artifacts.manifest()?;
        for item in &manifest.items {
            let entry = artifacts
                .content_entry(&item.content_sha256)
                .ok_or_else(unavailable)?;
            artifacts.read(entry, CONTENT_LIMIT)?;
        }
        Ok(())
    })
    .await
    .map_err(|_| unavailable())??;
    Ok(TermsCurrent {
        manifest_url: format!("/api/v2/auth/terms/manifests/{digest}"),
        terms_version: digest,
        terms_revision: revision.to_string(),
    })
}

pub(super) async fn manifest(
    State(state): State<AuthRestState>,
    path: Result<Path<String>, PathRejection>,
) -> Response {
    let artifacts = state.terms_artifacts;
    let result = tokio::task::spawn_blocking(move || {
        let digest = requested_digest(path)?;
        let artifacts = artifacts.ok_or_else(unavailable)?;
        if digest != artifacts.index.manifest.sha256 {
            return Err(not_found());
        }
        artifacts.manifest().map(|(bytes, _)| bytes)
    })
    .await
    .map_err(|_| unavailable())
    .and_then(|result| result);
    artifact_response(result, "application/json; charset=utf-8")
}

pub(super) async fn content(
    State(state): State<AuthRestState>,
    path: Result<Path<String>, PathRejection>,
) -> Response {
    let artifacts = state.terms_artifacts;
    let result = tokio::task::spawn_blocking(move || {
        let digest = requested_digest(path)?;
        let artifacts = artifacts.ok_or_else(unavailable)?;
        let entry = artifacts.content_entry(&digest).ok_or_else(not_found)?;
        artifacts.read(entry, CONTENT_LIMIT)
    })
    .await
    .map_err(|_| unavailable())
    .and_then(|result| result);
    artifact_response(result, "text/plain; charset=utf-8")
}

fn artifact_response(result: Result<Vec<u8>, RestError>, content_type: &'static str) -> Response {
    match result {
        Ok(bytes) => (
            [
                (header::CONTENT_TYPE, content_type),
                (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            ],
            bytes,
        )
            .into_response(),
        Err(error) => private_response(error.into_response()),
    }
}

fn private_response(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
        .headers_mut()
        .insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
}

fn requested_digest(path: Result<Path<String>, PathRejection>) -> Result<String, RestError> {
    let Path(digest) = path.map_err(|_| not_found())?;
    if !valid_digest(&digest) {
        return Err(not_found());
    }
    Ok(digest)
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_kind(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.as_bytes()[0].is_ascii_lowercase()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'-')
        })
}

fn relative_artifact_path(path: &FilePath) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn read_bounded(root: &FilePath, relative: &FilePath, limit: u64) -> Result<Vec<u8>, RestError> {
    if !relative_artifact_path(relative) {
        return Err(unavailable());
    }
    let path = root
        .join(relative)
        .canonicalize()
        .map_err(|_| unavailable())?;
    if !path.starts_with(root) || !path.is_file() {
        return Err(unavailable());
    }
    let file = File::open(path).map_err(|_| unavailable())?;
    let metadata = file.metadata().map_err(|_| unavailable())?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(unavailable());
    }
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| unavailable())?;
    if bytes.len() as u64 > limit {
        return Err(unavailable());
    }
    Ok(bytes)
}

fn unavailable() -> RestError {
    RestError {
        status: StatusCode::SERVICE_UNAVAILABLE,
        code: "authority_unavailable",
        message: "Terms authority unavailable".to_owned(),
    }
}

fn not_found() -> RestError {
    RestError::not_found("Terms artifact not found")
}
