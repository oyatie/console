// Test-only global footprint, grounded in the retained Company owner and exact birth oracles.
const CONTROLLER_COMPANY_ADDITIONS: &[(&str, usize)] = &[
    ("organizations", 1),
    ("groups", 1),
    ("group_memberships", 1),
    ("group_authority_heads", 1),
    ("group_membership_revisions", 1),
    ("company_enrollment_requests", 1),
    ("company_enrollment_request_events", 2),
    ("company_enrollment_effect_bindings", 1),
    ("company_enrollment_receipts", 1),
    ("company_actors", 1),
    ("account_context_candidates", 1),
    ("policy_roles", 1),
    ("policy_role_revisions", 1),
    ("policy_capability_clauses", 7),
    ("policy_capability_clause_fields", 16),
    ("user_role_assignments", 1),
    ("policy_assignment_revisions", 1),
    ("company_authority_heads", 1),
    ("native_company_catalog_installs", 1),
    ("native_company_object_refs", 2),
    ("native_company_action_refs", 5),
    ("native_company_property_refs", 10),
    ("ont_builtin_catalog_installs", 1),
    ("ont_object_type_key_revisions", 2),
    ("ont_object_types", 2),
    ("ont_property_defs", 10),
    ("ont_action_types", 5),
    ("cedar_policy_catalog_entries", 2),
    ("ont_object_policies", 2),
    ("audit_events", 5),
];

fn controller_security_delta(before: &str, after: &str, administrator: Uuid) -> bool {
    use serde_json::value::RawValue;
    let check = || -> Option<()> {
        let original: Vec<&RawValue> = serde_json::from_str(before).ok()?;
        let observed: Vec<&RawValue> = serde_json::from_str(after).ok()?;
        if original.len() != observed.len() {
            return None;
        }
        let mut retained = BTreeSet::new();
        let mut selected = None;
        for row in &original {
            let fields: BTreeMap<&str, &RawValue> = serde_json::from_str(row.get()).ok()?;
            if serde_json::from_str::<Value>(fields.get("account_id")?.get()).ok()?
                == json!(administrator)
            {
                if selected.replace(fields).is_some() {
                    return None;
                }
            } else if !retained.insert(row.get()) {
                return None;
            }
        }
        let previous = selected?;
        let mut changes = 0;
        for row in &observed {
            let fields: BTreeMap<&str, &RawValue> = serde_json::from_str(row.get()).ok()?;
            if serde_json::from_str::<Value>(fields.get("account_id")?.get()).ok()?
                != json!(administrator)
            {
                if !retained.remove(row.get()) {
                    return None;
                }
                continue;
            }
            changes += 1;
            if fields.keys().collect::<Vec<_>>() != previous.keys().collect::<Vec<_>>() {
                return None;
            }
            let generation: i64 =
                serde_json::from_str(previous.get("context_generation")?.get()).ok()?;
            let next: i64 = serde_json::from_str(fields.get("context_generation")?.get()).ok()?;
            if !(1..=256).contains(&generation) || Some(next) != generation.checked_add(1) {
                return None;
            }
            for (key, value) in &previous {
                if *key != "context_generation" && fields.get(key)?.get() != value.get() {
                    return None;
                }
            }
        }
        (changes == 1 && retained.is_empty()).then_some(())
    };
    check().is_some()
}

fn controller_company_delta(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
    r: &Committed,
    operator: Uuid,
) -> bool {
    if before.keys().collect::<Vec<_>>() != after.keys().collect::<Vec<_>>()
        || CONTROLLER_COMPANY_ADDITIONS
            .iter()
            .any(|(table, _)| !before.contains_key(*table))
        || !before.contains_key("account_security")
    {
        return false;
    }
    for (table, previous) in before {
        if table == "account_security" {
            if !controller_security_delta(previous, &after[table], r.administrator) {
                return false;
            }
        } else if let Some((_, count)) = CONTROLLER_COMPANY_ADDITIONS
            .iter()
            .find(|(name, _)| *name == table)
        {
            let Some(added) = added_rows(previous, &after[table]) else {
                return false;
            };
            if added.len() != *count
                || added.iter().any(|row| match table.as_str() {
                    "groups" => row["id"] != json!(r.group),
                    "group_authority_heads" => row["group_id"] != json!(r.group),
                    "organizations" => row["id"] != json!(r.company),
                    "company_enrollment_requests" | "company_enrollment_request_events" => {
                        row["account_id"] != json!(operator)
                            || row["command_id"] != json!(r.command)
                    }
                    "account_context_candidates" => {
                        row["account_id"] != json!(r.administrator)
                            || row["context_id"] != json!(r.company)
                    }
                    _ => row["org_id"] != json!(r.company),
                })
            {
                return false;
            }
        } else if previous != &after[table] {
            return false;
        }
    }
    true
}

async fn assert_controller_company_effects(
    pool: &PgPool,
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
    r: &Committed,
    submitted: &Value,
    operator: Uuid,
) {
    assert!(
        controller_company_delta(before, after, r, operator),
        "Company effect footprint differs"
    );
    assert!(
        identity_graph_matches(after, r, operator),
        "exact identity graph differs"
    );
    assert!(
        native_catalog_attribution_with_schema(after, r, operator, "2026-07-ontology-authoring-v1"),
        "exact catalog attribution differs"
    );
    assert_native_catalog_content(after, r.company);
    identity_digests_match(pool, after, r).await;
    let rows = |table: &str| added_rows(&before[table], &after[table]).unwrap();
    let binding = rows("company_enrollment_effect_bindings").remove(0);
    let request = rows("company_enrollment_requests").remove(0);
    let receipt = rows("company_enrollment_receipts").remove(0);
    let membership = rows("group_memberships").remove(0);
    let history = rows("group_membership_revisions").remove(0);
    let encoded = console_identity_application::CompanyEnrollmentV1::from_json_slice(
        &serde_json::to_vec(submitted).unwrap(),
    )
    .unwrap()
    .encode(operator)
    .unwrap();
    let digest = json!(format!("\\x{}", hex::encode(Sha256::digest(&encoded))));
    assert!(policy_fields_match(
        &binding,
        &json!({"account_id":operator,"command_id":r.command,
        "org_id":r.company,"group_id":r.group,"receipt_id":r.receipt,
        "administrative_account_id":r.administrator,"input_digest":digest,"codec_version":1})
    ));
    assert!(policy_fields_match(
        &request,
        &json!({"account_id":operator,"command_id":r.command,
        "codec_version":1,"input_bytes":format!("\\x{}",hex::encode(encoded)),"input_digest":digest,
        "state":"COMMITTED","committed_receipt_id":r.receipt,"terminal_at":binding["started_at"],
        "designation_receipt_id":binding["designation_receipt_id"]})
    ));
    assert!(policy_fields_match(
        &receipt,
        &json!({"account_id":operator,"command_id":r.command,
        "receipt_id":r.receipt,"org_id":r.company,"group_id":r.group,"codec_version":1,
        "administrative_account_id":r.administrator,"input_digest":digest,
        "designation_receipt_id":binding["designation_receipt_id"],"session_id":binding["session_id"],
        "committed_at":binding["started_at"]})
    ));
    for (revision, from, to, reason, at) in [
        (
            1,
            Value::Null,
            "PENDING",
            "PREPARED",
            &request["created_at"],
        ),
        (
            2,
            json!("PENDING"),
            "COMMITTED",
            "COMMITTED",
            &binding["started_at"],
        ),
    ] {
        let events = rows("company_enrollment_request_events");
        let matching: Vec<_> = events
            .iter()
            .filter(|e| e["event_revision"] == revision)
            .collect();
        assert_eq!(matching.len(), 1);
        assert!(policy_fields_match(
            matching[0],
            &json!({"account_id":operator,"command_id":r.command,
            "event_revision":revision,"from_state":from,"to_state":to,"reason_code":reason,
            "actor_account_id":operator,"session_id":binding["session_id"],"occurred_at":at})
        ));
    }
    assert!(policy_fields_match(
        &membership,
        &json!({"group_id":r.group,"org_id":r.company,
        "current_revision":1,"created_at":binding["started_at"]})
    ));
    assert!(policy_fields_match(
        &history,
        &json!({"group_id":r.group,"org_id":r.company,
        "membership_id":membership["membership_id"],"incarnation":membership["incarnation"],
        "revision":1,"from_time":binding["started_at"],"to_time":null,"state":"ACTIVE",
        "provenance_kind":"COMPANY_ENROLLMENT_V1","native_account_id":operator,
        "legacy_actor_user_id":null,"command_id":r.command,"command_receipt":r.receipt})
    ));
    for (table, id) in [("organizations", r.company), ("groups", r.group)] {
        let actual = rows(table).remove(0);
        assert!(policy_fields_match(
            &actual,
            &json!({"id":id,"name":submitted["name"],"status":"ACTIVE",
            "created_at":binding["started_at"],"updated_at":binding["started_at"],
            "origin_account_id":operator,"origin_command_id":r.command,"origin_receipt_id":r.receipt})
        ));
        if table == "organizations" {
            assert_eq!(actual["slug"], submitted["slug"]);
        }
    }
    let head = rows("group_authority_heads").remove(0);
    assert!(policy_fields_match(
        &head,
        &json!({"group_id":r.group,"revision":1,"state":"ACTIVE"})
    ));
    for id in [
        &head["incarnation"],
        &membership["membership_id"],
        &membership["incarnation"],
    ] {
        let text = id.as_str().unwrap();
        let parsed: Uuid = text.parse().unwrap();
        assert!(!parsed.is_nil() && parsed.to_string() == text);
    }
    // Corrupt observations only; retain every original raw byte so failure
    // proves rejection of the injected effect rather than serializer whitespace.
    for (table, raw) in after {
        if table == "account_security" {
            continue;
        }
        let mut corrupt = after.clone();
        let extra = json!({"id":Uuid::new_v4(),"org_id":r.company,"unexpected":true}).to_string();
        let prefix = raw.strip_suffix(']').unwrap();
        let comma = if raw.trim() == "[]" { "" } else { ", " };
        corrupt.insert(table.clone(), format!("{prefix}{comma}{extra}]"));
        assert!(
            !controller_company_delta(before, &corrupt, r, operator),
            "effect oracle accepted additional rows in {table}"
        );
    }
    use serde_json::value::RawValue;
    for (table, _) in CONTROLLER_COMPANY_ADDITIONS {
        let previous: Vec<&RawValue> = serde_json::from_str(&before[*table]).unwrap();
        let actual: Vec<&RawValue> = serde_json::from_str(&after[*table]).unwrap();
        let selected = actual
            .iter()
            .find(|row| !previous.iter().any(|old| old.get() == row.get()))
            .unwrap()
            .get();
        let mut value: Value = serde_json::from_str(selected).unwrap();
        let key = match *table {
            "organizations" | "groups" => "id",
            "group_authority_heads" => "group_id",
            "company_enrollment_requests" | "company_enrollment_request_events" => "command_id",
            "account_context_candidates" => "context_id",
            _ => "org_id",
        };
        let mut represented = after.clone();
        represented.insert(
            (*table).into(),
            after[*table].replacen(selected, &value.to_string(), 1),
        );
        assert!(
            controller_company_delta(before, &represented, r, operator),
            "positive control rejected unchanged added-row meaning"
        );
        value[key] = json!(Uuid::new_v4());
        represented.insert(
            (*table).into(),
            after[*table].replacen(selected, &value.to_string(), 1),
        );
        assert!(
            !controller_company_delta(before, &represented, r, operator),
            "effect oracle accepted foreign same-cardinality effect in {table}"
        );
    }
    let security: Vec<&RawValue> = serde_json::from_str(&after["account_security"]).unwrap();
    let selected = security
        .iter()
        .find(|row| {
            serde_json::from_str::<Value>(row.get()).unwrap()["account_id"]
                == json!(r.administrator)
        })
        .unwrap()
        .get();
    let generation = serde_json::from_str::<Value>(selected).unwrap()["context_generation"]
        .as_i64()
        .unwrap();
    for (from, to) in [
        (
            format!("\"context_generation\": {generation}"),
            format!("\"context_generation\": {}", generation + 1),
        ),
        (
            "\"security_state\": \"ACTIVE\"".into(),
            "\"security_state\": \"REVOKED\"".into(),
        ),
    ] {
        assert_eq!(
            selected.matches(&from).count(),
            1,
            "ineffective security corruption"
        );
        let changed = selected.replacen(&from, &to, 1);
        let mut corrupt = after.clone();
        assert_eq!(after["account_security"].matches(selected).count(), 1);
        corrupt.insert(
            "account_security".into(),
            after["account_security"].replacen(selected, &changed, 1),
        );
        assert!(
            !controller_company_delta(before, &corrupt, r, operator),
            "effect oracle accepted another security field or extra context generation"
        );
    }
}
