//! Fail-closed typed decode of canonical DispatchTarget execute/preflight params.
//!
//! Chesterton: HTTP `ActionRequest.params` was `serde_json::Value`, and
//! `decode_canonical_query` injects `target` then deserializes the port Query.
//! `CompanyQuery` is an untagged struct, so extra fields (including a
//! caller-supplied `target`) were ignored. Enum queries ignored extra fields
//! without `deny_unknown_fields`. This module is the HTTP trust boundary: the
//! path `action_key` selects the Input schema; unknown fields and a mismatched
//! `target` / `action_key` property fail closed before `prepare`.
//!
//! The original `Value` is returned on success so command-id payload digests
//! do not change shape.
//!
//! Codec structs and `decode_dispatch_target` are generated from
//! `semantic_manifest.json` by `console-openapi-gen` into
//! `typed_action_generated.rs`. This file is the runtime binder, not a second
//! contract.

use console_ontology_canonical_domain::DispatchTarget;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::str::FromStr;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

use super::ActionError;

time::serde::format_description!(iso_date, Date, "[year]-[month]-[day]");

/// Validate `params` against the typed input for a DispatchTarget `action_key`.
///
/// Non-roster keys (generic instance-revision actions) keep an object-or-null
/// params bag, validated later against the action's `params_schema`.
pub(crate) fn bind_canonical_action_params(
    action_key: &str,
    params: &Value,
) -> Result<Value, ActionError> {
    match DispatchTarget::from_str(action_key) {
        Ok(target) => {
            reject_caller_action_key(action_key, params)?;
            decode_dispatch_target(target, params)?;
            Ok(params.clone())
        }
        Err(_) => match params {
            Value::Null | Value::Object(_) => Ok(params.clone()),
            other => Err(ActionError::Validation(format!(
                "params must be a JSON object, got {other}"
            ))),
        },
    }
}

fn reject_caller_action_key(action_key: &str, params: &Value) -> Result<(), ActionError> {
    let obj = params.as_object().ok_or_else(|| {
        ActionError::Validation(format!("params must be a JSON object for {action_key}"))
    })?;
    for field in ["target", "action_key"] {
        let Some(value) = obj.get(field) else {
            continue;
        };
        let sent = match value {
            Value::String(text) => text.as_str(),
            other => {
                return Err(ActionError::Validation(format!(
                    "params.{field} must be a string matching {action_key}, got {other}"
                )));
            }
        };
        if sent != action_key {
            return Err(ActionError::Validation(format!(
                "params.{field} {sent:?} does not match action_key {action_key}"
            )));
        }
        return Err(ActionError::Validation(format!(
            "params.{field} is not a field of the typed {action_key} input"
        )));
    }
    Ok(())
}

fn decode<T: DeserializeOwned>(target: DispatchTarget, params: &Value) -> Result<(), ActionError> {
    serde_json::from_value::<T>(params.clone())
        .map(|_| ())
        .map_err(|error| {
            ActionError::Validation(format!(
                "params do not decode as {}: {error}",
                target.as_str()
            ))
        })
}

include!("typed_action_generated.rs");

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const ID: &str = "00000000-0000-0000-0000-000000000001";
    const WHEN: &str = "2026-03-01T00:00:00Z";

    fn err(action_key: &str, params: Value) -> String {
        match bind_canonical_action_params(action_key, &params) {
            Err(ActionError::Validation(message)) => message,
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    fn employment_attributes() -> Value {
        json!({
            "company": "한빛",
            "employment_status": "ACTIVE",
        })
    }

    fn minimal_params(target: DispatchTarget) -> Value {
        match target {
            DispatchTarget::CompanyRevise => json!({"attributes": {"legal_name": "한빛"}}),
            DispatchTarget::OrganizationCreateOrgUnit => json!({
                "attributes": {"name": "본사", "kind": "site"}
            }),
            DispatchTarget::OrganizationReviseOrgUnit => json!({
                "org_unit_id": ID,
                "attributes": {"name": "본사", "kind": "site"}
            }),
            DispatchTarget::OrganizationCreateJobPosition => json!({
                "org_unit_id": ID,
                "attributes": {"title": "백엔드"}
            }),
            DispatchTarget::OrganizationReviseJobPosition => json!({
                "job_position_id": ID,
                "attributes": {"title": "백엔드"}
            }),
            DispatchTarget::PeopleCreatePerson => json!({
                "attributes": {"legal_name": "홍길동"}
            }),
            DispatchTarget::PeopleRevisePerson => json!({
                "person_id": ID,
                "attributes": {"legal_name": "홍길동"}
            }),
            DispatchTarget::HrAppoint => json!({
                "employee_id": ID,
                "valid_from": WHEN,
                "attributes": employment_attributes(),
            }),
            DispatchTarget::HrPromote | DispatchTarget::HrTransfer => json!({
                "employment_id": ID,
                "valid_from": WHEN,
                "attributes": employment_attributes(),
            }),
            DispatchTarget::PayrollCreateRun => json!({
                "run_id": ID,
                "period_start": "2026-03-01",
                "period_end": "2026-03-31",
            }),
            DispatchTarget::PayrollSubmitRun => json!({"run_id": ID}),
            DispatchTarget::PayrollDecideRun => json!({
                "run_id": ID,
                "decision": "APPROVE",
            }),
        }
    }

    #[test]
    fn unknown_field_on_company_revise_is_rejected() {
        let message = err(
            "company.revise",
            json!({"attributes": {"legal_name": "한빛"}, "bogus": 1}),
        );
        assert!(
            message.contains("bogus") || message.contains("unknown"),
            "{message}"
        );
    }

    #[test]
    fn wrong_action_key_in_params_is_rejected() {
        let message = err(
            "company.revise",
            json!({"attributes": {"legal_name": "한빛"}, "target": "hr.appoint"}),
        );
        assert!(
            message.contains("hr.appoint") || message.contains("target"),
            "{message}"
        );
    }

    #[test]
    fn payroll_submit_run_rejects_decide_fields() {
        let message = err(
            "payroll.submit_run",
            json!({"run_id": ID, "decision": "APPROVE"}),
        );
        assert!(
            message.contains("decision") || message.contains("unknown"),
            "{message}"
        );
    }

    #[test]
    fn generic_instance_revision_params_remain_an_object() {
        let params = json!({"code": "CODE-SWEPT", "extra": true});
        let bound = bind_canonical_action_params("set_priority", &params).unwrap();
        assert_eq!(bound, params);
    }

    #[test]
    fn generic_non_object_params_are_rejected() {
        let message = err("set_priority", json!(["not", "an", "object"]));
        assert!(message.contains("JSON object"), "{message}");
    }

    #[test]
    fn company_revise_valid_params_round_trip() {
        let params = json!({"attributes": {"legal_name": "한빛", "reg_no": null}});
        let bound = bind_canonical_action_params("company.revise", &params).unwrap();
        assert_eq!(bound, params);
    }

    #[test]
    fn hr_appoint_missing_required_is_rejected() {
        let message = err(
            "hr.appoint",
            json!({"employee_id": ID, "attributes": employment_attributes()}),
        );
        assert!(
            message.contains("valid_from") || message.contains("missing"),
            "{message}"
        );
    }

    #[test]
    fn every_dispatch_target_has_a_typed_codec() {
        for target in DispatchTarget::ALL {
            let params = minimal_params(*target);
            bind_canonical_action_params(target.as_str(), &params).unwrap_or_else(|error| {
                panic!("{} rejected its typed input: {error:?}", target.as_str())
            });
        }
        assert_eq!(DispatchTarget::ALL.len(), 13);
    }
}
