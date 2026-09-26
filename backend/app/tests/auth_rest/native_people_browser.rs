//! Browser-to-database oracle for the native directory journey. This module
//! provisions no business rows. All mutations come from the observed browser.
use super::*;

pub(super) const DRIVER_SHA256: &str =
    "861343c520b6e556f28f23a2ce9d5632a5800892626199f096204460bed1b4f1";
const NAME: &str = "김하늘 <연구 & 운영>";
const NUMBER: &str = "UI-사람-001";
const PHASES: &[&str] = &[
    "PEOPLE_INSTALLED",
    "PEOPLE_READ_GRANTED",
    "PEOPLE_READ_ONLY",
    "PEOPLE_CREATE_GRANTED",
    "PEOPLE_PREPARED",
    "PEOPLE_PENDING_REOPENED",
    "PEOPLE_COMMITTED",
    "PEOPLE_RECEIPT_REOPENED",
    "PEOPLE_DETAIL",
    "PEOPLE_DETAIL_REOPENED",
    "PEOPLE_LIST",
    "PEOPLE_READ_REVOKED",
    "PEOPLE_READ_DENIED",
    "PEOPLE_OWN_RECEIPT",
    "PEOPLE_CREATE_REVOKED",
    "PEOPLE_RECEIPT_DENIED",
];

// All tables remain in the comparison, including future tables. Every allowed
// addition has exact cardinality and preserves all prior row bytes.
fn additions(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
    counts: &[(&str, usize)],
) -> Option<BTreeMap<String, String>> {
    if before.keys().ne(after.keys()) {
        return None;
    }
    let mut remaining = after.clone();
    for (table, count) in counts {
        if added_rows(before.get(*table)?, after.get(*table)?)?.len() != *count {
            return None;
        }
        remaining.insert((*table).to_owned(), before[*table].clone());
    }
    Some(remaining)
}

fn only_added(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
    table: &str,
) -> Value {
    let rows = added_rows(&before[table], &after[table]).expect("old row bytes retained");
    assert_eq!(rows.len(), 1, "one expected owner effect");
    rows[0].clone()
}

fn actor(row: &Value, account: Uuid, company: Uuid) {
    assert_eq!(row["org_id"], json!(company));
    assert_eq!(row["actor_kind"], "ACCOUNT");
    assert_eq!(row["actor_account_id"], json!(account));
    assert!(row["actor_id"].is_null());
}

fn people_action_matches(action: &Value, company: Uuid, object: &Value, key: &str) -> bool {
    action["org_id"] == json!(company)
        && action["object_type_id"] == *object
        && action.get("dispatch_target").and_then(Value::as_str) == Some(key)
        && ["people.directory.read", "people.directory.create"].contains(&key)
}

fn directory_record_valid(record: &Value, accepted: &Value, company: Uuid) -> bool {
    let Some(command) = accepted["command_id"].as_str() else {
        return false;
    };
    record["id"] == accepted["employee_id"]
        && record["org_id"] == json!(company)
        && record["name"] == NAME
        && record["employee_number"] == NUMBER
        && record["employment_status"] == "UNKNOWN"
        && record["source_kind"] == "NATIVE_DIRECTORY"
        && record["native_command_id"] == accepted["command_id"]
        && record["source_key"] == format!("native-directory:{command}")
        && record["raw_row"] == json!({})
        && record["source_metadata"] == json!({})
        && record["identity_resolution_strategy"] == "employee_number"
        && record["identity_resolution_confidence"] == "low"
        && record["identity_review_required"] == true
        && record["identity_name_only_merge"] == false
        && [
            "source_filename",
            "source_sheet",
            "source_row",
            "home_branch_id",
            "hire_date",
            "exit_date",
            "org_unit",
            "job",
            "position",
            "worksite_name",
            "worksite_address",
            "leave_accrued",
            "leave_used",
            "leave_remaining",
        ]
        .iter()
        .all(|field| record.get(*field) == Some(&Value::Null))
}

fn canonical_links_valid(
    receipt: &Value,
    revision: &Value,
    binding: &Value,
    terminal: &Value,
    accepted: &Value,
) -> bool {
    let result =
        json!({"person_id":accepted["employee_id"],"version":1,"target":"people.create_person"});
    let valid_digest = receipt["payload_digest"].as_str().is_some_and(|value| {
        value
            .strip_prefix("\\x")
            .and_then(|raw| hex::decode(raw).ok())
            .is_some_and(|raw| raw.len() == 32)
    });
    valid_digest
        && ["command_id", "employee_id", "intake_receipt_id"]
            .iter()
            .all(|key| {
                accepted[*key]
                    .as_str()
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .is_some_and(|id| !id.is_nil())
            })
        && accepted["input_digest"].as_str().is_some_and(|s| {
            s.strip_prefix("\\x")
                .and_then(|raw| hex::decode(raw).ok())
                .is_some_and(|bytes| bytes.len() == 32)
        })
        && receipt["owner"] == "person"
        && receipt["target"] == "people.create_person"
        && receipt["receipt"] == result
        && revision["receipt"] == result
        && receipt["payload_digest"] == revision["payload_digest"]
        && receipt["payload_digest"] == binding["payload_digest"]
        && [receipt, revision, terminal]
            .iter()
            .all(|row| row["command_id"] == accepted["command_id"])
        && terminal["input_digest"] == accepted["input_digest"]
        && terminal["intake_receipt_id"] == accepted["intake_receipt_id"]
        && terminal["employee_id"] == accepted["employee_id"]
        && terminal["person_id"] == accepted["employee_id"]
}

fn audited(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
    account: Uuid,
    company: Uuid,
    actions: &[&str],
) {
    let rows = added_rows(&before["audit_events"], &after["audit_events"]).unwrap();
    assert_eq!(rows.len(), actions.len());
    let mut found = Vec::new();
    for row in &rows {
        assert_eq!(row["actor"], json!(account));
        assert_eq!(row["org_id"], json!(company));
        found.push(row["action"].as_str().unwrap());
    }
    found.sort();
    let mut expected = actions.to_vec();
    expected.sort();
    assert_eq!(found, expected);
}

fn policy_effects(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
    request: &Value,
    event: &Value,
    account: Uuid,
    company: Uuid,
) -> Value {
    let command = policy_uuid(&event["command_id"]);
    assert_eq!(request["command_id"], json!(command));
    let input = only_added(before, after, "native_company_policy_inputs_v1");
    let receipt = only_added(before, after, "native_company_policy_receipts_v1");
    for row in [&input, &receipt] {
        assert_eq!(row["command_id"], json!(command));
        assert_eq!(row["actor_account_id"], json!(account));
        assert_eq!(row["org_id"], json!(company));
        assert_eq!(row["codec_version"], 2);
    }
    let phase = event["phase"].as_str().unwrap();
    let fields: BTreeMap<&str, &str> = request["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|pair| (pair[0].as_str().unwrap(), pair[1].as_str().unwrap()))
        .collect();
    assert_eq!(fields.len(), request["fields"].as_array().unwrap().len());
    assert_eq!(fields["command_id"], command.to_string());
    let install = phase == "PEOPLE_INSTALLED";
    let revoke = phase.ends_with("REVOKED");
    let op = if install {
        1
    } else if revoke {
        3
    } else {
        2
    };
    assert_eq!(receipt["operation"], op);
    assert_eq!(input["operation"], op);
    assert_eq!(receipt["outcome"], "COMMITTED");
    assert_eq!(receipt["catalog_version"], "native-people-directory-v1");
    assert_eq!(receipt["input_digest"], input["input_digest"]);
    let bytes = hex::decode(
        input["input_bytes"]
            .as_str()
            .unwrap()
            .strip_prefix("\\x")
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        input["input_digest"],
        format!("\\x{}", hex::encode(Sha256::digest(&bytes)))
    );
    let old_head = policy_only_row(before, "company_authority_heads", "org_id", &json!(company));
    let mut head = old_head.clone();
    assert_eq!(receipt["epoch_before"], old_head["epoch"]);
    assert_eq!(
        fields["expected_company_epoch"],
        old_head["epoch"].as_i64().unwrap().to_string()
    );
    assert_eq!(
        receipt["predecessor_receipt_id"],
        old_head["current_policy_receipt_id"]
    );
    head["epoch"] = json!(old_head["epoch"].as_i64().unwrap() + 1);
    head["current_policy_receipt_id"] = receipt["receipt_id"].clone();
    assert_eq!(receipt["epoch_after"], head["epoch"]);
    assert_eq!(
        policy_only_row(after, "company_authority_heads", "org_id", &json!(company)),
        head
    );
    let mut expected_head_rows = policy_rows(before, "company_authority_heads");
    for row in &mut expected_head_rows {
        if row["org_id"] == json!(company) {
            *row = head.clone();
        }
    }
    let ordered = |mut rows: Vec<Value>| {
        rows.sort_by_key(Value::to_string);
        rows
    };
    assert_eq!(
        ordered(expected_head_rows),
        ordered(policy_rows(after, "company_authority_heads"))
    );
    let mut counts = vec![
        ("native_company_policy_inputs_v1", 1),
        ("native_company_policy_receipts_v1", 1),
        ("audit_events", if install { 3 } else { 2 }),
    ];
    if install {
        counts.extend([
            ("ont_object_type_key_revisions", 1),
            ("ont_object_types", 1),
            ("ont_property_defs", 6),
            ("ont_action_types", 2),
            ("ont_builtin_catalog_installs", 1),
            ("native_company_catalog_installs", 1),
            ("native_company_object_refs", 1),
            ("native_company_action_refs", 2),
            ("native_company_property_refs", 6),
        ]);
        let props = added_rows(&before["ont_property_defs"], &after["ont_property_defs"]).unwrap();
        let names: BTreeSet<_> = props.iter().map(|r| r["key"].as_str().unwrap()).collect();
        assert_eq!(
            names,
            BTreeSet::from([
                "employee_id",
                "person_id",
                "legal_name",
                "employee_number",
                "person_version",
                "directory_registered_at"
            ])
        );
        let actions = added_rows(&before["ont_action_types"], &after["ont_action_types"]).unwrap();
        assert_eq!(
            actions
                .iter()
                .map(|r| r["dispatch_target"].as_str().unwrap())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["people.directory.read", "people.directory.create"])
        );
        for row in props.iter().chain(actions.iter()) {
            assert_eq!(row["org_id"], json!(company));
            assert_eq!(row["object_type_id"], receipt["installed_object_type_id"]);
        }
        for action in &actions {
            assert!(people_action_matches(
                action,
                company,
                &receipt["installed_object_type_id"],
                action["dispatch_target"].as_str().unwrap()
            ));
            let references = added_rows(
                &before["native_company_action_refs"],
                &after["native_company_action_refs"],
            )
            .unwrap();
            let matched: Vec<_> = references
                .iter()
                .filter(|r| r["action_type_id"] == action["id"])
                .collect();
            assert_eq!(matched.len(), 1);
            assert!(policy_fields_match(
                matched[0],
                &json!({"org_id":company,"catalog_version":"native-people-directory-v1",
                "action_key":action["dispatch_target"],"object_type_id":action["object_type_id"],"registration_revision":1,"manifest_digest":receipt["manifest_digest"]})
            ));
        }
        for property in &props {
            let references = added_rows(
                &before["native_company_property_refs"],
                &after["native_company_property_refs"],
            )
            .unwrap();
            let matched: Vec<_> = references
                .iter()
                .filter(|r| r["property_id"] == property["id"])
                .collect();
            assert_eq!(matched.len(), 1);
            assert!(policy_fields_match(
                matched[0],
                &json!({"org_id":company,"catalog_version":"native-people-directory-v1",
                "property_key":format!("person.{}",property["key"].as_str().unwrap()),"object_type_id":property["object_type_id"],
                "schema_revision":1,"manifest_digest":receipt["manifest_digest"]})
            ));
        }
    } else {
        let read = event["action"] == "read";
        assert!(read || event["action"] == "create");
        assert_eq!(receipt["recipient_account_id"], json!(account));
        let assignment = policy_only_row(
            after,
            "user_role_assignments",
            "id",
            &receipt["assignment_id"],
        );
        assert_eq!(assignment["account_id"], json!(account));
        assert_eq!(assignment["org_id"], json!(company));
        assert_eq!(assignment["role_id"], receipt["role_id"]);
        assert_eq!(
            assignment["native_current_revision"],
            receipt["assignment_revision_after"]
        );
        let revision = only_added(before, after, "policy_assignment_revisions");
        assert_eq!(revision["assignment_id"], receipt["assignment_id"]);
        assert_eq!(revision["state"], if revoke { "REVOKED" } else { "ACTIVE" });
        assert_eq!(revision["valid_from"], receipt["assignment_valid_from"]);
        assert_eq!(revision["valid_until"], receipt["assignment_valid_until"]);
        if revoke {
            counts.push(("policy_assignment_revisions", 1));
        } else {
            assert_eq!(fields["recipient_account_id"], account.to_string());
            let submitted_expiry = OffsetDateTime::parse(
                &format!("{}:00+09:00", fields["expires_at_local"]),
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap();
            let recorded_expiry = OffsetDateTime::parse(
                receipt["assignment_valid_until"].as_str().unwrap(),
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap();
            assert_eq!(recorded_expiry, submitted_expiry);
            let clause = only_added(before, after, "policy_capability_clauses");
            assert_eq!(clause["effect"], "ALLOW");
            assert_eq!(clause["delegable"], false);
            assert_eq!(clause["resource_org_id"], json!(company));
            assert_eq!(clause["role_id"], receipt["role_id"]);
            assert_eq!(clause["role_revision"], 1);
            let action =
                policy_only_row(after, "ont_action_types", "id", &clause["action_type_id"]);
            assert!(people_action_matches(
                &action,
                company,
                &clause["action_object_type_id"],
                if read {
                    "people.directory.read"
                } else {
                    "people.directory.create"
                }
            ));
            let fields = added_rows(
                &before["policy_capability_clause_fields"],
                &after["policy_capability_clause_fields"],
            )
            .unwrap();
            let mut field_names = BTreeSet::new();
            for field in fields {
                assert_eq!(field["org_id"], json!(company));
                assert_eq!(field["role_id"], receipt["role_id"]);
                assert_eq!(field["role_revision"], 1);
                assert_eq!(field["schema_revision"], 1);
                assert_eq!(field["clause_index"], clause["clause_index"]);
                let property =
                    policy_only_row(after, "ont_property_defs", "id", &field["property_id"]);
                assert_eq!(property["org_id"], json!(company));
                assert_eq!(property["object_type_id"], action["object_type_id"]);
                assert_eq!(field["object_type_id"], action["object_type_id"]);
                assert!(field_names.insert(property["key"].as_str().unwrap().to_owned()));
            }
            let expected_fields: &[&str] = if read {
                &[
                    "employee_id",
                    "person_id",
                    "legal_name",
                    "employee_number",
                    "person_version",
                    "directory_registered_at",
                ]
            } else {
                &["legal_name", "employee_number"]
            };
            assert_eq!(
                field_names,
                expected_fields.iter().map(|s| (*s).to_owned()).collect()
            );
            counts.extend([
                ("policy_roles", 1),
                ("policy_role_revisions", 1),
                ("policy_capability_clauses", 1),
                ("policy_capability_clause_fields", if read { 6 } else { 2 }),
                ("user_role_assignments", 1),
                ("policy_assignment_revisions", 1),
            ]);
        }
    }
    let mut remaining = additions(before, after, &counts)
        .expect("exact People policy cardinalities and unchanged history");
    remaining.insert(
        "company_authority_heads".into(),
        before["company_authority_heads"].clone(),
    );
    if revoke {
        let mut expected = policy_rows(before, "user_role_assignments");
        let mut changed = 0;
        for row in &mut expected {
            if row["id"] == receipt["assignment_id"] {
                assert_eq!(
                    row["native_current_revision"],
                    receipt["assignment_revision_before"]
                );
                row["native_current_revision"] = receipt["assignment_revision_after"].clone();
                changed += 1;
            }
        }
        assert_eq!(changed, 1);
        assert_eq!(
            ordered(expected),
            ordered(policy_rows(after, "user_role_assignments"))
        );
        remaining.insert(
            "user_role_assignments".into(),
            before["user_role_assignments"].clone(),
        );
    }
    assert!(remaining == *before, "unexpected People policy effect");
    audited(
        before,
        after,
        account,
        company,
        if install {
            &[
                "policy.company_command.accept",
                "policy.company_command.complete",
                "ontology.object_type.builtin_install",
            ]
        } else {
            &[
                "policy.company_command.accept",
                "policy.company_command.complete",
            ]
        },
    );
    receipt
}

pub(super) async fn observe(
    pool: &PgPool,
    input: &mut tokio::process::ChildStdin,
    events: &mut tokio::io::BufReader<tokio::process::ChildStdout>,
    account: Uuid,
    company: Uuid,
) {
    let mut prior = all_rows(pool).await;
    let mut at: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(pool)
        .await
        .unwrap();
    let mut next = 0;
    let mut ready: Option<Value> = None;
    let mut accepted: Option<Value> = None;
    let mut employee = None;
    while next < PHASES.len() {
        let event = browser_owner_event(events).await;
        assert_eq!(
            event["kind"], "CHECKPOINT",
            "expected People owner checkpoint; inspect browser result on failure"
        );
        assert_eq!(event["account_id"], json!(account));
        assert_eq!(event["org_id"], json!(company));
        let phase = event["phase"].as_str().unwrap();
        let current = all_rows(pool).await;
        let now: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(pool)
            .await
            .unwrap();
        let mut witness = json!({"phase": phase, "company": company, "account": account});
        if phase == "PEOPLE_READY" {
            assert_eq!(next, 0);
            assert!(ready.is_none() && current == prior);
        } else if phase == "PEOPLE_ACTION_READY" {
            assert!(ready.is_none());
            assert_eq!(event["action_phase"], PHASES[next]);
            let remaining = if PHASES[next] == "PEOPLE_PREPARED" {
                audited(
                    &prior,
                    &current,
                    account,
                    company,
                    &["people.directory.read"],
                );
                additions(&prior, &current, &[("audit_events", 1)]).unwrap()
            } else {
                current.clone()
            };
            assert!(
                remaining == prior || policy_preflight_effects(&prior, &remaining, at, now),
                "unexpected preflight effects"
            );
            ready = Some(event.clone());
        } else {
            assert_eq!(
                phase, PHASES[next],
                "missing or reordered People checkpoint"
            );
            next += 1;
            match phase {
                "PEOPLE_INSTALLED"
                | "PEOPLE_READ_GRANTED"
                | "PEOPLE_CREATE_GRANTED"
                | "PEOPLE_READ_REVOKED"
                | "PEOPLE_CREATE_REVOKED" => {
                    let receipt = policy_effects(
                        &prior,
                        &current,
                        &ready.take().unwrap(),
                        &event,
                        account,
                        company,
                    );
                    witness["assignment_id"] = receipt["assignment_id"].clone();
                }
                "PEOPLE_PREPARED" => {
                    let request = ready.take().unwrap();
                    assert_eq!(request["command_id"], event["command_id"]);
                    let row = only_added(&prior, &current, "native_people_inputs_v1");
                    assert_eq!(row["org_id"], json!(company));
                    assert_eq!(row["actor_account_id"], json!(account));
                    assert_eq!(row["command_id"], request["command_id"]);
                    assert_eq!(row["legal_name"], NAME);
                    assert_eq!(row["employee_number"], NUMBER);
                    policy_uuid(&row["employee_id"]);
                    let remaining = additions(
                        &prior,
                        &current,
                        &[("native_people_inputs_v1", 1), ("audit_events", 1)],
                    )
                    .unwrap();
                    assert!(
                        policy_preflight_effects(&prior, &remaining, at, now),
                        "prepare plus pending form only"
                    );
                    audited(
                        &prior,
                        &current,
                        account,
                        company,
                        &["people.directory.prepare"],
                    );
                    accepted = Some(row);
                }
                "PEOPLE_PENDING_REOPENED" => {
                    assert_eq!(
                        event["command_id"],
                        accepted.as_ref().unwrap()["command_id"]
                    );
                    assert!(
                        policy_preflight_effects(&prior, &current, at, now),
                        "pending reopen only issues form proof"
                    );
                }
                "PEOPLE_COMMITTED" => {
                    let request = ready.take().unwrap();
                    let accepted = accepted.as_ref().unwrap();
                    assert_eq!(request["command_id"], accepted["command_id"]);
                    assert_eq!(event["command_id"], accepted["command_id"]);
                    let record = only_added(&prior, &current, "employees");
                    let person = only_added(&prior, &current, "persons");
                    let revision = only_added(&prior, &current, "person_revisions");
                    let binding = only_added(&prior, &current, "employee_person_bindings");
                    let receipt = only_added(&prior, &current, "ont_action_command_receipts");
                    let terminal = only_added(&prior, &current, "native_people_terminals_v1");
                    assert!(
                        directory_record_valid(&record, accepted, company),
                        "native directory identity/provenance and absent employment fields"
                    );
                    assert!(
                        canonical_links_valid(&receipt, &revision, &binding, &terminal, accepted),
                        "canonical result and command/provenance linkage"
                    );
                    assert_eq!(record["id"], accepted["employee_id"]);
                    assert_eq!(record["org_id"], json!(company));
                    assert_eq!(record["name"], NAME);
                    assert_eq!(record["employee_number"], NUMBER);
                    assert_eq!(record["employment_status"], "UNKNOWN");
                    assert_eq!(record["source_kind"], "NATIVE_DIRECTORY");
                    assert_eq!(record["native_command_id"], accepted["command_id"]);
                    for field in [
                        "source_filename",
                        "source_sheet",
                        "source_row",
                        "home_branch_id",
                        "hire_date",
                    ] {
                        assert!(record[field].is_null());
                    }
                    assert_eq!(person["id"], record["id"]);
                    assert_eq!(person["org_id"], json!(company));
                    assert_eq!(binding["employee_id"], record["id"]);
                    assert_eq!(binding["person_id"], person["id"]);
                    assert_eq!(revision["person_id"], person["id"]);
                    assert_eq!(revision["version"], 1);
                    assert_eq!(revision["attributes"], json!({"legal_name":NAME}));
                    for row in [&revision, &receipt] {
                        assert_eq!(row["command_id"], accepted["command_id"]);
                    }
                    for row in [&revision, &binding, &receipt] {
                        actor(row, account, company);
                    }
                    assert_eq!(receipt["receipt"]["target"], "people.create_person");
                    assert_eq!(terminal["command_id"], accepted["command_id"]);
                    assert_eq!(terminal["org_id"], json!(company));
                    assert_eq!(terminal["actor_account_id"], json!(account));
                    assert_eq!(terminal["outcome"], "COMMITTED");
                    assert_eq!(terminal["employee_id"], record["id"]);
                    assert_eq!(terminal["person_id"], person["id"]);
                    let remaining = additions(
                        &prior,
                        &current,
                        &[
                            ("employees", 1),
                            ("persons", 1),
                            ("person_revisions", 1),
                            ("employee_person_bindings", 1),
                            ("ont_action_command_receipts", 1),
                            ("native_people_terminals_v1", 1),
                            ("audit_events", 1),
                        ],
                    )
                    .unwrap();
                    assert!(
                        remaining == prior,
                        "registration created prohibited or extra effects"
                    );
                    audited(
                        &prior,
                        &current,
                        account,
                        company,
                        &["people.directory.register"],
                    );
                    witness["employee_id"] = record["id"].clone();
                    witness["person_id"] = person["id"].clone();
                    employee = Some(policy_uuid(&record["id"]));
                }
                "PEOPLE_READ_ONLY" | "PEOPLE_DETAIL" | "PEOPLE_DETAIL_REOPENED" | "PEOPLE_LIST" => {
                    audited(
                        &prior,
                        &current,
                        account,
                        company,
                        &["people.directory.read"],
                    );
                    assert!(
                        additions(&prior, &current, &[("audit_events", 1)]).unwrap() == prior,
                        "directory read changed other state"
                    );
                }
                _ => assert!(
                    prior == current,
                    "reopen or denied read changed durable state"
                ),
            }
        }
        witness["owner_effects_verified"] = json!(true);
        prior = current;
        at = now;
        policy_browser_ack(input, phase, witness).await;
    }
    assert!(ready.is_none() && accepted.is_some() && employee.is_some());
}

#[test]
fn people_effect_census_requires_every_effect_and_preserves_unrelated_history() {
    let before = BTreeMap::from([
        ("employees".into(), "[{\"id\":1}]".into()),
        ("other".into(), "[]".into()),
    ]);
    let after = BTreeMap::from([
        ("employees".into(), "[{\"id\":1},{\"id\":2}]".into()),
        ("other".into(), "[]".into()),
    ]);
    assert_eq!(
        additions(&before, &after, &[("employees", 1)]),
        Some(before.clone())
    );
    assert!(additions(&before, &before, &[("employees", 1)]).is_none());
    let mut corrupt = after.clone();
    corrupt.insert("employees".into(), "[{\"id\":3},{\"id\":2}]".into());
    assert!(additions(&before, &corrupt, &[("employees", 1)]).is_none());
    corrupt = after.clone();
    corrupt.insert("other".into(), "[{\"unexpected\":true}]".into());
    assert_ne!(
        additions(&before, &corrupt, &[("employees", 1)]),
        Some(before.clone())
    );
    corrupt = after.clone();
    corrupt.remove("other");
    assert!(additions(&before, &corrupt, &[("employees", 1)]).is_none());
}

#[test]
fn native_directory_shape_oracle_rejects_employment_facts_and_false_provenance() {
    let company = Uuid::new_v4();
    let command = Uuid::new_v4();
    let employee = Uuid::new_v4();
    let accepted = json!({"command_id":command,"employee_id":employee});
    let mut row = json!({"id":employee,"org_id":company,"name":NAME,"employee_number":NUMBER,
        "employment_status":"UNKNOWN","source_kind":"NATIVE_DIRECTORY","native_command_id":command,
        "source_key":format!("native-directory:{command}"),"raw_row":{},"source_metadata":{},
        "identity_resolution_strategy":"employee_number","identity_resolution_confidence":"low",
        "identity_review_required":true,"identity_name_only_merge":false});
    let absent = [
        "source_filename",
        "source_sheet",
        "source_row",
        "home_branch_id",
        "hire_date",
        "exit_date",
        "org_unit",
        "job",
        "position",
        "worksite_name",
        "worksite_address",
        "leave_accrued",
        "leave_used",
        "leave_remaining",
    ];
    for key in absent {
        row[key] = Value::Null;
    }
    assert!(directory_record_valid(&row, &accepted, company));
    for key in absent {
        let mut wrong = row.clone();
        wrong[key] = json!("invented");
        assert!(!directory_record_valid(&wrong, &accepted, company), "{key}");
        wrong.as_object_mut().unwrap().remove(key);
        assert!(
            !directory_record_valid(&wrong, &accepted, company),
            "missing {key}"
        );
    }
    for (key, value) in [
        ("employment_status", json!("ACTIVE")),
        ("source_key", json!("console:001")),
        ("native_command_id", json!(Uuid::new_v4())),
        ("raw_row", json!({"salary":1})),
        ("source_metadata", json!({"file":"fake"})),
        ("identity_name_only_merge", json!(true)),
        (
            "identity_resolution_strategy",
            json!("source_row_fingerprint"),
        ),
        ("identity_resolution_confidence", json!("high")),
        ("identity_review_required", json!(false)),
    ] {
        let mut wrong = row.clone();
        wrong[key] = value;
        assert!(!directory_record_valid(&wrong, &accepted, company), "{key}");
    }
}

#[test]
fn native_receipt_linkage_oracle_rejects_same_count_wrong_identity_or_digest() {
    let command = Uuid::new_v4();
    let employee = Uuid::new_v4();
    let intake = Uuid::new_v4();
    let digest = format!("\\x{}", "ab".repeat(32));
    let accepted = json!({"command_id":command,"employee_id":employee,"intake_receipt_id":intake,"input_digest":digest});
    let result = json!({"person_id":employee,"version":1,"target":"people.create_person"});
    let receipt = json!({"owner":"person","target":"people.create_person","receipt":result,"command_id":command,"payload_digest":digest});
    let revision = json!({"receipt":result,"command_id":command,"payload_digest":digest});
    let binding = json!({"payload_digest":digest});
    let terminal = json!({"command_id":command,"employee_id":employee,"person_id":employee,"intake_receipt_id":intake,"input_digest":digest});
    assert!(canonical_links_valid(
        &receipt, &revision, &binding, &terminal, &accepted
    ));
    for (key, value) in [
        ("owner", json!("ontology.action")),
        ("target", json!("people.revise_person")),
        ("command_id", json!(Uuid::new_v4())),
        (
            "receipt",
            json!({"person_id":Uuid::new_v4(),"version":1,"target":"people.create_person"}),
        ),
        ("payload_digest", json!(format!("\\x{}", "cd".repeat(32)))),
    ] {
        let mut wrong = receipt.clone();
        wrong[key] = value;
        assert!(
            !canonical_links_valid(&wrong, &revision, &binding, &terminal, &accepted),
            "{key}"
        );
    }
    for key in [
        "command_id",
        "employee_id",
        "person_id",
        "intake_receipt_id",
        "input_digest",
    ] {
        let mut wrong = terminal.clone();
        wrong[key] = Value::Null;
        assert!(
            !canonical_links_valid(&receipt, &revision, &binding, &wrong, &accepted),
            "{key}"
        );
    }
}

#[test]
fn people_action_oracle_uses_actual_dispatch_target_column() {
    let company = Uuid::new_v4();
    let object = json!(Uuid::new_v4());
    let row = json!({"org_id":company,"object_type_id":object,"stable_key":"directory_read","dispatch_target":"people.directory.read"});
    assert!(people_action_matches(
        &row,
        company,
        &object,
        "people.directory.read"
    ));
    let mut wrong = row.clone();
    wrong["dispatch_target"] = json!("people.directory.create");
    assert!(!people_action_matches(
        &wrong,
        company,
        &object,
        "people.directory.read"
    ));
    wrong.as_object_mut().unwrap().remove("dispatch_target");
    wrong["key"] = json!("people.directory.read");
    assert!(!people_action_matches(
        &wrong,
        company,
        &object,
        "people.directory.read"
    ));
    assert!(!people_action_matches(
        &row,
        Uuid::new_v4(),
        &object,
        "people.directory.read"
    ));
    assert!(!people_action_matches(
        &row,
        company,
        &json!(Uuid::new_v4()),
        "people.directory.read"
    ));
}
