//! Bind the frozen V4 source to its outer columns and original Auth namespace.
//! SQL owns source custody/graph validation; this layer refuses substitution.
use super::*;
use console_platform_auth::AccountAssurance;
use time::format_description::well_known::Rfc3339;

mod intervals;
pub(super) fn valid_at(m: &Value, now: OffsetDateTime) -> Result<(), Error> {
    intervals::valid_at(m, now)
}

const KEYS: &[&str] = &[
    "account",
    "assignment",
    "assignment_revision",
    "birth_effect",
    "birth_events",
    "birth_receipt",
    "birth_request",
    "catalog",
    "company",
    "company_actor",
    "company_head",
    "consent",
    "context_presence",
    "family",
    "group",
    "group_head",
    "kind",
    "membership",
    "membership_revision",
    "registered_clauses",
    "registration",
    "request",
    "role",
    "role_revision",
    "workspace",
];

pub(super) fn equal(row: &Value, key: &str, expected: Value) -> Result<(), Error> {
    if row.get(key) != Some(&expected) {
        return Err(Error::Unavailable);
    }
    Ok(())
}
fn shape(row: &Value, keys: &[&str]) -> Result<(), Error> {
    let map = row.as_object().ok_or(Error::Unavailable)?;
    if map.len() != keys.len() || keys.iter().any(|key| !map.contains_key(*key)) {
        return Err(Error::Unavailable);
    }
    Ok(())
}
pub(super) fn time(row: &Value, key: &str) -> Result<OffsetDateTime, Error> {
    let text = row
        .get(key)
        .and_then(Value::as_str)
        .ok_or(Error::Unavailable)?;
    let value = OffsetDateTime::parse(text, &Rfc3339).map_err(|_| Error::Unavailable)?;
    if value.offset() != time::UtcOffset::UTC {
        return Err(Error::Unavailable);
    }
    exact_time(value)?;
    Ok(value)
}
fn array(value: &Value, min: usize, max: usize) -> Result<&[Value], Error> {
    let rows = value.as_array().ok_or(Error::Unavailable)?;
    if !(min..=max).contains(&rows.len()) || rows.iter().any(|row| !row.is_object()) {
        return Err(Error::Unavailable);
    }
    Ok(rows)
}

pub(super) fn bind(source: &Projection, session: &AccountLiveSession) -> Result<(), Error> {
    let m = &source.source_material;
    shape(m, KEYS)?;
    // Frozen material is bounded to the finite declared graph/catalog roster.
    if m.to_string().len() > 1024 * 1024 || source.registered_clauses.to_string().len() > 32768 {
        return Err(Error::Unavailable);
    }
    equal(
        m,
        "kind",
        json!("COMPANY_INFORMATION_MANAGER_CURRENT_SOURCE_V1"),
    )?;
    equal(
        m,
        "request",
        json!({"codec_version":4,"operation":"Grant",
        "account_id":source.actor_account_id,"session_id":source.session_id,
        "org_id":source.org_id,"command_id":source.command_id}),
    )?;
    equal(m, "registered_clauses", source.registered_clauses.clone())?;
    shape(&m["account"], &["id", "created_at", "security"])?;
    shape(&m["family"], &["id", "material"])?;
    shape(
        &m["group_head"],
        &["group_id", "revision", "incarnation", "state"],
    )?;
    array(&source.registered_clauses, 7, 7)?;
    for key in [
        "account",
        "assignment",
        "assignment_revision",
        "birth_effect",
        "birth_receipt",
        "birth_request",
        "catalog",
        "company",
        "company_actor",
        "company_head",
        "context_presence",
        "family",
        "group",
        "group_head",
        "membership",
        "membership_revision",
        "role",
        "role_revision",
        "workspace",
    ] {
        if !m[key].is_object() {
            return Err(Error::Unavailable);
        }
    }
    for (key, id_field, id) in [
        ("account", "id", source.actor_account_id),
        ("family", "id", source.session_id),
        ("company", "id", source.org_id),
        ("group", "id", source.current_group_id),
        ("group_head", "group_id", source.current_group_id),
        ("assignment", "id", source.assignment_id),
        ("assignment_revision", "assignment_id", source.assignment_id),
        ("role", "id", source.role_id),
        ("role_revision", "role_id", source.role_id),
        ("company_head", "org_id", source.org_id),
        (
            "workspace",
            "object_type_id",
            source.installed_object_type_id,
        ),
    ] {
        equal(&m[key], id_field, json!(id))?;
    }
    for key in [
        "assignment",
        "assignment_revision",
        "birth_effect",
        "birth_receipt",
        "company_actor",
        "company_head",
        "membership",
        "membership_revision",
        "role",
        "role_revision",
        "workspace",
    ] {
        equal(&m[key], "org_id", json!(source.org_id))?;
    }
    for key in [
        "company",
        "membership",
        "membership_revision",
        "birth_receipt",
        "birth_effect",
    ] {
        equal(&m[key], "group_id", json!(source.current_group_id))?;
    }
    equal(&m["company"], "name", json!(source.company_name))?;
    equal(&m["company"], "slug", json!(source.company_slug))?;
    equal(&m["company_head"], "epoch", json!(source.company_epoch))?;
    equal(
        &m["company_head"],
        "current_policy_receipt_id",
        json!(source.current_policy_receipt_id),
    )?;
    equal(
        &m["assignment"],
        "account_id",
        json!(source.actor_account_id),
    )?;
    equal(
        &m["assignment_revision"],
        "account_id",
        json!(source.actor_account_id),
    )?;
    equal(
        &m["company_actor"],
        "account_id",
        json!(source.actor_account_id),
    )?;
    equal(
        &m["birth_receipt"],
        "administrative_account_id",
        json!(source.actor_account_id),
    )?;
    equal(
        &m["birth_receipt"],
        "root_assignment_id",
        json!(source.assignment_id),
    )?;
    equal(
        &m["birth_receipt"],
        "root_revision",
        json!(source.assignment_revision),
    )?;
    for key in ["assignment", "assignment_revision"] {
        equal(&m[key], "role_id", json!(source.role_id))?;
    }
    equal(
        &m["assignment_revision"],
        "revision",
        json!(source.assignment_revision),
    )?;
    equal(
        &m["assignment_revision"],
        "role_revision",
        json!(source.role_revision),
    )?;
    equal(&m["role_revision"], "revision", json!(source.role_revision))?;
    equal(
        &m["assignment"],
        "native_current_revision",
        json!(source.assignment_revision),
    )?;
    equal(
        &m["role"],
        "native_current_revision",
        json!(source.role_revision),
    )?;
    for key in ["assignment", "assignment_revision", "role", "role_revision"] {
        equal(&m[key], "subject_protocol", json!("NATIVE_ACCOUNT"))?;
    }
    equal(&m["company"], "status", json!("ACTIVE"))?;
    equal(&m["group"], "status", json!("ACTIVE"))?;
    for key in [
        "group_head",
        "membership_revision",
        "assignment_revision",
        "role_revision",
    ] {
        equal(&m[key], "state", json!("ACTIVE"))?;
    }
    equal(&m["role"], "status", json!("ACTIVE"))?;
    equal(&m["membership_revision"], "revision", json!(1))?;
    equal(
        &m["membership_revision"],
        "provenance_kind",
        json!("COMPANY_ENROLLMENT_V1"),
    )?;
    equal(&m["membership_revision"], "to_time", Value::Null)?;
    for key in ["assignment_revision", "role_revision"] {
        equal(&m[key], "valid_until", Value::Null)?;
    }
    let security = &m["account"]["security"];
    if !security.is_object() || session.assurance != AccountAssurance::PasskeyPrimary {
        return Err(Error::Unavailable);
    }
    equal(security, "account_id", json!(session.account_id))?;
    equal(security, "security_state", json!("ACTIVE"))?;
    equal(
        security,
        "security_generation",
        json!(session.security_generation),
    )?;
    equal(
        security,
        "context_generation",
        json!(source.context_generation),
    )?;
    positive(security["revision"].as_i64().ok_or(Error::Unavailable)?)?;
    shape(
        &m["context_presence"],
        &["context_generation", "has_candidates"],
    )?;
    equal(
        &m["context_presence"],
        "context_generation",
        json!(source.context_generation),
    )?;
    equal(&m["context_presence"], "has_candidates", json!(true))?;
    let family = &m["family"]["material"];
    if !family.is_object() {
        return Err(Error::Unavailable);
    }
    equal(family, "user_id", json!(session.account_id))?;
    equal(
        family,
        "account_security_generation",
        json!(session.security_generation),
    )?;
    equal(family, "protocol", json!("ACCOUNT_V1"))?;
    equal(family, "assurance", json!("PASSKEY_PRIMARY"))?;
    equal(family, "revoked_at", Value::Null)?;
    equal(family, "org_id", Value::Null)?;
    if time(family, "auth_time")? != session.auth_time
        || time(family, "created_at")? < session.auth_time
    {
        return Err(Error::Unavailable);
    }
    array(&m["registration"], 1, 8)?;
    array(&m["consent"], 1, 8)?;
    equal(m, "consent", m["registration"].clone())?;
    array(&m["birth_events"], 2, 2)?;
    let catalog = &m["catalog"];
    shape(
        catalog,
        &[
            "initial_source_attribution",
            "installs",
            "objects",
            "actions",
            "properties",
        ],
    )?;
    for (key, min, max) in [
        ("initial_source_attribution", 3, 3),
        ("installs", 1, 3),
        ("objects", 2, 4),
        ("actions", 5, 8),
        ("properties", 10, 34),
    ] {
        let rows = array(&catalog[key], min, max)?;
        if key != "initial_source_attribution" {
            for row in rows {
                equal(row, "org_id", json!(source.org_id))?;
            }
        }
    }
    Ok(())
}
