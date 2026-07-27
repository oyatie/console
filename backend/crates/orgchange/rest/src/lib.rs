//! Authenticated org-change routes (STORY-ORG-001).
//!
//! Deny-by-default authorization: every route checks an explicit role floor
//! before touching the store, and RLS `org_isolation` conceals out-of-scope
//! rows (absent and forbidden objects are both 404 — deny by omission).
//!
//! Feature-floor note: the `org_change_read/draft/approve/apply` feature keys
//! are registered in `feature_catalog` (migration 0189) but the shared
//! `mnt_platform_authz::Feature` enum is integrator-owned (see the lane's
//! integration-manifest.json). Until the enum gains the variants, custom-role
//! grants for these keys cannot exist (`Feature::from_str` skips unknown keys
//! fail-closed), so the built-in role floors below are the complete authorized
//! surface and byte-for-byte match `permission_for` for the scout floors:
//! read/draft `[D,D,D,A,A,A]`, approve/apply `[D,D,D,D,A,A]`. Swapping to
//! `authorize(...)` once the variants land is mechanical.
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use mnt_kernel_core::{ErrorKind, KernelError};
use mnt_orgchange_adapter_postgres::{
    CreateOrgChange, DraftPatch, ListFilter, PgOrgChangeError, PgOrgChangeStore,
};
use mnt_orgchange_domain::{OrgChangeKind, OrgChangeStatus, OrgChangeTarget, OrgProposalOp};
use mnt_platform_auth::JwtVerifier;
use mnt_platform_authz::{Principal, Role};
use mnt_platform_request_context::RequestContextError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::Date;
use uuid::Uuid;

// The workspace `time` build has no `serde-human-readable`, so a bare `Date`
// would be read off the wire as `[year, ordinal]`; this pins `YYYY-MM-DD`.
time::serde::format_description!(iso_date, Date, "[year]-[month]-[day]");

pub const ORG_CHANGE_ROUTE_PATHS: &[&str] = &[
    "/api/v1/org-changes",
    "/api/v1/org-changes/{id}",
    "/api/v1/org-changes/{id}/preflight",
    "/api/v1/org-changes/{id}/submit",
    "/api/v1/org-changes/{id}/approval-steps/{step_id}/decision",
    "/api/v1/org-changes/{id}/effectuate",
    "/api/v1/org-changes/{id}/settlement-items/{item_id}/complete",
    "/api/v1/org-changes/{id}/archive",
    "/api/v1/org-changes/{id}/cancel",
    "/api/v1/org-entities",
];

#[derive(Clone)]
pub struct OrgChangeRestState {
    store: PgOrgChangeStore,
    jwt: Option<JwtVerifier>,
}

impl OrgChangeRestState {
    #[must_use]
    pub fn new(store: PgOrgChangeStore, jwt: Option<JwtVerifier>) -> Self {
        Self { store, jwt }
    }
}

pub fn router(state: OrgChangeRestState) -> Router {
    let verifier = state.jwt.clone();
    let pool = state.store.pool().clone();
    let r = Router::new()
        .route("/api/v1/org-changes", post(create).get(list))
        .route("/api/v1/org-changes/{id}", get(detail).patch(update_draft))
        .route("/api/v1/org-changes/{id}/preflight", post(preflight))
        .route("/api/v1/org-changes/{id}/submit", post(submit))
        .route(
            "/api/v1/org-changes/{id}/approval-steps/{step_id}/decision",
            post(decide),
        )
        .route("/api/v1/org-changes/{id}/effectuate", post(effectuate))
        .route(
            "/api/v1/org-changes/{id}/settlement-items/{item_id}/complete",
            post(complete_settlement),
        )
        .route("/api/v1/org-changes/{id}/archive", post(archive))
        .route("/api/v1/org-changes/{id}/cancel", post(cancel))
        .route("/api/v1/org-entities", get(org_entities))
        .with_state(state);
    mnt_platform_request_context::with_request_context(r, verifier, pool)
}

// ---------------------------------------------------------------------------
// Request DTOs
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateOrgChangeRequest {
    kind: OrgChangeKind,
    target: OrgChangeTarget,
    #[serde(with = "iso_date")]
    effective_date: Date,
    reason: String,
    proposal: Vec<OrgProposalOp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    supersedes_id: Option<Uuid>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UpdateOrgChangeDraftRequest {
    #[serde(default)]
    kind: Option<OrgChangeKind>,
    #[serde(default, with = "iso_date::option")]
    effective_date: Option<Date>,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    proposal: Option<Vec<OrgProposalOp>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum WireDecision {
    Approved,
    Rejected,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OrgChangeDecisionRequest {
    decision: WireDecision,
    #[serde(default)]
    memo: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CompleteSettlementItemRequest {
    #[serde(default)]
    memo: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CancelOrgChangeRequest {
    reason: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ListQuery {
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    offset: Option<i64>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn create(
    State(s): State<OrgChangeRestState>,
    h: HeaderMap,
    Json(b): Json<CreateOrgChangeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), RestError> {
    let p = principal(&s, &h).await?;
    allow_draft(&p)?;
    let key = idem_header(&h)?;
    let fingerprint_input = serde_json::to_value(&b)
        .map_err(|_| RestError::kernel(KernelError::validation("request body is not canonical")))?;
    let (detail, replayed) = s
        .store
        .create(
            p.user_id,
            CreateOrgChange {
                kind: b.kind,
                target: b.target,
                effective_date: b.effective_date,
                reason: b.reason,
                proposal: b.proposal,
                supersedes_id: b.supersedes_id,
                idempotency_key: key,
                fingerprint_input,
            },
        )
        .await
        .map_err(RestError::store)?;
    let status = if replayed {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };
    let body = serde_json::to_value(&detail)
        .map_err(|_| RestError::internal("response serialization failed"))?;
    Ok((status, Json(body)))
}

async fn list(
    State(s): State<OrgChangeRestState>,
    h: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, RestError> {
    let p = principal(&s, &h).await?;
    allow_read(&p)?;
    let status = q
        .status
        .as_deref()
        .map(OrgChangeStatus::from_db)
        .transpose()
        .map_err(|_| RestError::kernel(KernelError::validation("unknown status filter")))?;
    let kind = q
        .kind
        .as_deref()
        .map(OrgChangeKind::from_db)
        .transpose()
        .map_err(|_| RestError::kernel(KernelError::validation("unknown kind filter")))?;
    let page = s
        .store
        .list(ListFilter {
            status,
            kind,
            limit: q.limit,
            offset: q.offset,
        })
        .await
        .map_err(RestError::store)?;
    respond(&page)
}

async fn detail(
    State(s): State<OrgChangeRestState>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, RestError> {
    let p = principal(&s, &h).await?;
    allow_read(&p)?;
    respond(&s.store.get(id).await.map_err(RestError::store)?)
}

async fn update_draft(
    State(s): State<OrgChangeRestState>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
    Json(b): Json<UpdateOrgChangeDraftRequest>,
) -> Result<Json<serde_json::Value>, RestError> {
    let p = principal(&s, &h).await?;
    allow_draft(&p)?;
    respond(
        &s.store
            .update_draft(
                p.user_id,
                id,
                DraftPatch {
                    kind: b.kind,
                    effective_date: b.effective_date,
                    reason: b.reason,
                    proposal: b.proposal,
                },
            )
            .await
            .map_err(RestError::store)?,
    )
}

async fn preflight(
    State(s): State<OrgChangeRestState>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, RestError> {
    let p = principal(&s, &h).await?;
    allow_draft(&p)?;
    respond(
        &s.store
            .preflight(p.user_id, id)
            .await
            .map_err(RestError::store)?,
    )
}

async fn submit(
    State(s): State<OrgChangeRestState>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, RestError> {
    let p = principal(&s, &h).await?;
    allow_draft(&p)?;
    respond(
        &s.store
            .submit(p.user_id, id)
            .await
            .map_err(RestError::store)?,
    )
}

async fn decide(
    State(s): State<OrgChangeRestState>,
    h: HeaderMap,
    Path((id, step_id)): Path<(Uuid, Uuid)>,
    Json(b): Json<OrgChangeDecisionRequest>,
) -> Result<Json<serde_json::Value>, RestError> {
    let p = principal(&s, &h).await?;
    allow_approve(&p)?;
    let approved = matches!(b.decision, WireDecision::Approved);
    respond(
        &s.store
            .decide_step(p.user_id, id, step_id, approved, b.memo)
            .await
            .map_err(RestError::store)?,
    )
}

async fn effectuate(
    State(s): State<OrgChangeRestState>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, RestError> {
    let p = principal(&s, &h).await?;
    allow_apply(&p)?;
    respond(
        &s.store
            .effectuate(p.user_id, id)
            .await
            .map_err(RestError::store)?,
    )
}

async fn complete_settlement(
    State(s): State<OrgChangeRestState>,
    h: HeaderMap,
    Path((id, item_id)): Path<(Uuid, Uuid)>,
    Json(b): Json<CompleteSettlementItemRequest>,
) -> Result<Json<serde_json::Value>, RestError> {
    let p = principal(&s, &h).await?;
    allow_apply(&p)?;
    respond(
        &s.store
            .complete_settlement_item(p.user_id, id, item_id, b.memo)
            .await
            .map_err(RestError::store)?,
    )
}

async fn archive(
    State(s): State<OrgChangeRestState>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, RestError> {
    let p = principal(&s, &h).await?;
    allow_apply(&p)?;
    respond(
        &s.store
            .archive(p.user_id, id)
            .await
            .map_err(RestError::store)?,
    )
}

async fn cancel(
    State(s): State<OrgChangeRestState>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
    Json(b): Json<CancelOrgChangeRequest>,
) -> Result<Json<serde_json::Value>, RestError> {
    let p = principal(&s, &h).await?;
    allow_draft(&p)?;
    respond(
        &s.store
            .cancel(p.user_id, id, b.reason)
            .await
            .map_err(RestError::store)?,
    )
}

async fn org_entities(
    State(s): State<OrgChangeRestState>,
    h: HeaderMap,
) -> Result<Json<serde_json::Value>, RestError> {
    let p = principal(&s, &h).await?;
    allow_read(&p)?;
    respond(
        &s.store
            .org_entities(p.user_id)
            .await
            .map_err(RestError::store)?,
    )
}

// ---------------------------------------------------------------------------
// Authorization floors (see the module doc for the Feature-enum handoff)
// ---------------------------------------------------------------------------

fn role_floor(p: &Principal, floor: &[Role], denied: &'static str) -> Result<(), RestError> {
    if p.roles.iter().any(|role| floor.contains(role)) {
        Ok(())
    } else {
        Err(RestError::kernel(KernelError::forbidden(denied)))
    }
}

/// `org_change_read` / `org_change_draft` floor `[D,D,D,A,A,A]`.
fn allow_read(p: &Principal) -> Result<(), RestError> {
    role_floor(
        p,
        &[Role::Admin, Role::Executive, Role::SuperAdmin],
        "role is not allowed to read org changes",
    )
}

fn allow_draft(p: &Principal) -> Result<(), RestError> {
    role_floor(
        p,
        &[Role::Admin, Role::Executive, Role::SuperAdmin],
        "role is not allowed to draft org changes",
    )
}

/// `org_change_approve` / `org_change_apply` floor `[D,D,D,D,A,A]`.
fn allow_approve(p: &Principal) -> Result<(), RestError> {
    role_floor(
        p,
        &[Role::Executive, Role::SuperAdmin],
        "role is not allowed to approve org changes",
    )
}

fn allow_apply(p: &Principal) -> Result<(), RestError> {
    role_floor(
        p,
        &[Role::Executive, Role::SuperAdmin],
        "role is not allowed to apply org changes",
    )
}

// ---------------------------------------------------------------------------
// Plumbing
// ---------------------------------------------------------------------------

fn respond<T: Serialize>(value: &T) -> Result<Json<serde_json::Value>, RestError> {
    serde_json::to_value(value)
        .map(Json)
        .map_err(|_| RestError::internal("response serialization failed"))
}

fn idem_header(h: &HeaderMap) -> Result<String, RestError> {
    h.get("Idempotency-Key")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
        .ok_or_else(|| {
            RestError::kernel(KernelError::validation(
                "Idempotency-Key header is required",
            ))
        })
}

async fn principal(s: &OrgChangeRestState, h: &HeaderMap) -> Result<Principal, RestError> {
    let verifier = s.jwt.as_ref().ok_or_else(|| {
        RestError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "unavailable",
            "JWT verification is not configured",
        )
    })?;
    mnt_platform_request_context::resolve_principal(verifier, s.store.pool(), h)
        .await
        .map_err(|e| match e {
            RequestContextError::MissingBearer
            | RequestContextError::InvalidToken
            | RequestContextError::InvalidClaim(_) => RestError::new(
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "missing, malformed, or invalid bearer token",
            ),
            RequestContextError::WrongTokenTier | RequestContextError::AccessScope(_) => {
                RestError::kernel(KernelError::forbidden(
                    "token is not authorized for org changes",
                ))
            }
            RequestContextError::VerifierUnavailable => RestError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "unavailable",
                "JWT verification is not configured",
            ),
            RequestContextError::BranchScope(m) | RequestContextError::EffectivePolicy(m) => {
                RestError::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", m)
            }
            RequestContextError::MissingOrg => RestError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                "no tenant context is bound",
            ),
        })
}

struct RestError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl RestError {
    fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", message)
    }

    fn kernel(e: KernelError) -> Self {
        match e.kind {
            ErrorKind::Validation => {
                Self::new(StatusCode::UNPROCESSABLE_ENTITY, "validation", e.message)
            }
            ErrorKind::NotFound => Self::new(StatusCode::NOT_FOUND, "not_found", e.message),
            ErrorKind::Forbidden => Self::new(StatusCode::FORBIDDEN, "forbidden", e.message),
            ErrorKind::Conflict | ErrorKind::InvalidTransition => {
                Self::new(StatusCode::CONFLICT, "conflict", e.message)
            }
            ErrorKind::Internal => {
                Self::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", e.message)
            }
        }
    }

    fn store(e: PgOrgChangeError) -> Self {
        let message = e.message();
        match e.kind() {
            ErrorKind::Validation => {
                Self::new(StatusCode::UNPROCESSABLE_ENTITY, "validation", message)
            }
            ErrorKind::NotFound => Self::new(StatusCode::NOT_FOUND, "not_found", message),
            ErrorKind::Forbidden => Self::new(StatusCode::FORBIDDEN, "forbidden", message),
            ErrorKind::Conflict | ErrorKind::InvalidTransition => {
                Self::new(StatusCode::CONFLICT, "conflict", message)
            }
            ErrorKind::Internal => {
                Self::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", message)
            }
        }
    }
}

impl IntoResponse for RestError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(json!({"error":{"code":self.code,"message":self.message}})),
        )
            .into_response()
    }
}
