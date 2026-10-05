// Child of company_setup. All business population is the existing real UI.
// Seven-relation reads begin only after the authenticated Group entry exists;
// no missing routine/schema/compile failure is asserted to be a product RED.
use super::*;
use std::path::Path;

const TABLES: [&str; 7] = [
    "native_group_identity_policy_heads_v1",
    "native_group_process_versions_v1",
    "native_group_process_heads_v1",
    "native_group_process_head_revisions_v1",
    "native_group_process_inputs_v1",
    "native_group_process_effects_v1",
    "native_group_process_results_v1",
];
const PHASES: [&str; 23] = [
    "GROUP_ENTRY_READY",
    "GROUP_ADOPT_FORM_READY",
    "GROUP_ADOPT_READY",
    "GROUP_ADOPTED",
    "GROUP_ADOPT_REOPENED",
    "GROUP_CURRENT_ACTIVE",
    "GROUP_SUSPEND_FORM_READY",
    "GROUP_SUSPEND_READY",
    "GROUP_SUSPENDED",
    "GROUP_SUSPEND_REOPENED",
    "GROUP_CURRENT_SUSPENDED",
    "GROUP_REPLACE_FORM_READY",
    "GROUP_REPLACE_READY",
    "GROUP_REPLACED",
    "GROUP_REPLACE_REOPENED",
    "GROUP_CURRENT_REPLACED",
    "GROUP_ORIGINAL_RECEIPT_REOPENED",
    "GROUP_DESIGNATION_REVOKE_READY",
    "GROUP_OWN_RECEIPT_AFTER_REVOKE",
    "GROUP_RETRY_FORM_READY",
    "GROUP_TERMINAL_REPLAY_READY",
    "GROUP_TERMINAL_REPLAYED",
    "GROUP_CURRENT_DENIED",
];
const CONTENT: [&str; 10] = [
    "title",
    "method",
    "intended_claimant_matching_procedure",
    "account_possession_procedure",
    "physical_human_evidence_procedure",
    "duplicate_contradictory_claim_procedure",
    "qualification_criteria_instruction",
    "escalation_adjudication_procedure",
    "evidence_minimization_retention_description",
    "recipient_responsibility",
];

fn rows(snapshot: &BTreeMap<String, String>, table: &str) -> Vec<Value> {
    serde_json::from_str(
        snapshot
            .get(table)
            .expect("actual installed successor table required"),
    )
    .unwrap()
}
fn selected(snapshot: &BTreeMap<String, String>, table: &str, actor: Uuid, command: Uuid) -> Value {
    let found: Vec<_> = rows(snapshot, table)
        .into_iter()
        .filter(|row| {
            row["actor_account_id"] == json!(actor) && row["command_id"] == json!(command)
        })
        .collect();
    assert_eq!(found.len(), 1, "actual actor-bound owner row cardinality");
    found[0].clone()
}
fn scoped(snapshot: &BTreeMap<String, String>, table: &str, group: Uuid) -> Value {
    let found: Vec<_> = rows(snapshot, table)
        .into_iter()
        .filter(|row| row["group_id"] == json!(group))
        .collect();
    assert_eq!(found.len(), 1, "actual Group head cardinality");
    found[0].clone()
}
fn raw(value: &Value) -> Vec<u8> {
    let text = value.as_str().expect("actual bytea field");
    let body = text
        .strip_prefix("\\x")
        .expect("PostgreSQL bytea hex representation");
    assert!(
        body.len().is_multiple_of(2)
            && body
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    );
    hex::decode(body).unwrap()
}
fn positive(value: &Value) -> u64 {
    let number = value.as_u64().expect("positive exact integer field");
    assert!((1..=i64::MAX as u64).contains(&number));
    number
}
fn micros(value: &Value) -> i64 {
    let at = OffsetDateTime::parse(
        value.as_str().unwrap(),
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap();
    i64::try_from(at.unix_timestamp_nanos() / 1000).unwrap()
}
fn text_bytes(bytes: &mut Vec<u8>, value: &str, maximum: usize) {
    assert!(!value.trim().is_empty() && value.len() <= maximum);
    assert!(
        value
            .chars()
            .all(|c| !c.is_control() || c == '\n' || c == '\t')
    );
    bytes.extend_from_slice(&u32::try_from(value.len()).unwrap().to_be_bytes());
    bytes.extend_from_slice(value.as_bytes());
}
fn fields(event: &Value) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    for pair in event["fields"]
        .as_array()
        .expect("actual posted named fields")
    {
        let pair = pair.as_array().unwrap();
        assert_eq!(pair.len(), 2);
        assert!(
            result
                .insert(
                    pair[0].as_str().unwrap().to_owned(),
                    pair[1].as_str().unwrap().to_owned()
                )
                .is_none()
        );
    }
    assert!(
        !result.contains_key("csrf_proof"),
        "never export Auth proof bytes"
    );
    result
}
fn expected_input(event: &Value, actor: Uuid, group: Uuid) -> Vec<u8> {
    let named = fields(event);
    let operation = u16::try_from(event["operation"].as_u64().unwrap()).unwrap();
    let mut bytes = b"CONSOLE.IDENTITY.GROUP\0".to_vec();
    bytes.extend_from_slice(&1u16.to_be_bytes());
    bytes.extend_from_slice(&operation.to_be_bytes());
    for uuid in [
        actor,
        policy_uuid(&event["command_id"]),
        group,
        policy_uuid(&json!(named["expected_group_incarnation"])),
    ] {
        bytes.extend_from_slice(uuid.as_bytes());
    }
    for key in [
        "expected_group_revision",
        "expected_group_identity_policy_revision",
    ] {
        let value: u64 = named[key].parse().unwrap();
        assert!(value <= i64::MAX as u64);
        if key == "expected_group_revision" {
            assert!(value > 0);
        }
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    let common = [
        "command_id",
        "expected_group_revision",
        "expected_group_incarnation",
        "expected_group_identity_policy_revision",
    ];
    let mut keys = BTreeSet::from(common);
    if operation == 1 {
        keys.extend([
            "process_id",
            "expected_prior_process_revision",
            "process_expiry",
            "operator_responsibility",
        ]);
        keys.extend(CONTENT);
        bytes.extend_from_slice(policy_uuid(&json!(named["process_id"])).as_bytes());
        let prior: u64 = named["expected_prior_process_revision"].parse().unwrap();
        assert!(prior <= i64::MAX as u64);
        bytes.extend_from_slice(&prior.to_be_bytes());
        let expiry = time::PrimitiveDateTime::parse(
            &named["process_expiry"],
            time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]"),
        )
        .unwrap()
        .assume_offset(time::UtcOffset::from_hms(9, 0, 0).unwrap());
        bytes.extend_from_slice(
            &i64::try_from(expiry.unix_timestamp_nanos() / 1000)
                .unwrap()
                .to_be_bytes(),
        );
        assert_eq!(named["operator_responsibility"], "1");
        bytes.extend_from_slice(&1u16.to_be_bytes());
        text_bytes(&mut bytes, &named["title"], 120);
        assert_eq!(
            named["method"],
            "ATTENDED_ACCOUNT_AND_DOCUMENTARY_REVIEW_V1"
        );
        bytes.extend_from_slice(&1u16.to_be_bytes());
        for key in CONTENT.iter().skip(2) {
            text_bytes(&mut bytes, &named[*key], 2048);
        }
        assert!(bytes.len() <= 16521);
    } else {
        assert_eq!(operation, 6);
        keys.extend([
            "process_version",
            "process_digest",
            "expected_process_head_revision",
            "expected_process_head_digest",
            "reason",
        ]);
        bytes.extend_from_slice(policy_uuid(&event["process_id"]).as_bytes());
        for (revision, digest) in [
            ("process_version", "process_digest"),
            (
                "expected_process_head_revision",
                "expected_process_head_digest",
            ),
        ] {
            let value: u64 = named[revision].parse().unwrap();
            assert!((1..=i64::MAX as u64).contains(&value));
            bytes.extend_from_slice(&value.to_be_bytes());
            let decoded = hex::decode(&named[digest]).unwrap();
            assert_eq!(decoded.len(), 32);
            bytes.extend_from_slice(&decoded);
        }
        text_bytes(&mut bytes, &named["reason"], 2048);
        assert!(bytes.len() <= 2255);
    }
    assert_eq!(
        named.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        keys
    );
    bytes
}
struct Reader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}
impl Reader<'_> {
    fn take(&mut self, length: usize) -> &[u8] {
        let end = self.cursor.checked_add(length).unwrap();
        assert!(end <= self.bytes.len());
        let result = &self.bytes[self.cursor..end];
        self.cursor = end;
        result
    }
    fn number(&mut self) -> u64 {
        u64::from_be_bytes(self.take(8).try_into().unwrap())
    }
    fn signed(&mut self) -> i64 {
        i64::from_be_bytes(self.take(8).try_into().unwrap())
    }
    fn uuid(&mut self) -> Value {
        let id = Uuid::from_slice(self.take(16)).unwrap();
        assert!(!id.is_nil());
        json!(id)
    }
    fn digest(&mut self) -> Value {
        json!(format!("\\x{}", hex::encode(self.take(32))))
    }
    fn text(&mut self) -> Value {
        let length = u32::from_be_bytes(self.take(4).try_into().unwrap()) as usize;
        assert!((1..=128).contains(&length));
        json!(std::str::from_utf8(self.take(length)).unwrap())
    }
    fn policy(&mut self) -> Value {
        match self.take(1)[0] {
            0 => Value::Null,
            1 => {
                let revision = self.number();
                assert!((1..=i64::MAX as u64).contains(&revision));
                json!({"revision": revision, "head_digest": self.digest()})
            }
            _ => panic!("unknown policy-reference tag"),
        }
    }
    fn head(&mut self) -> Value {
        match self.take(1)[0] {
            0 => Value::Null,
            1 => {
                let process = self.uuid();
                let head = self.number();
                let version = self.number();
                assert!(
                    (1..=i64::MAX as u64).contains(&head)
                        && (1..=i64::MAX as u64).contains(&version)
                );
                let content = self.digest();
                let digest = self.digest();
                let state = u16::from_be_bytes(self.take(2).try_into().unwrap());
                assert!([1, 2].contains(&state));
                json!({"process_id":process,"head_revision":head,"content_version":version,
                    "content_digest":content,"head_digest":digest,"state":if state==1{"ACTIVE"}else{"SUSPENDED"},
                    "expiry_us":self.signed()})
            }
            _ => panic!("unknown process-reference tag"),
        }
    }
}
fn decode_result(bytes: &[u8]) -> Value {
    let mut reader = Reader { bytes, cursor: 0 };
    assert_eq!(
        reader.take(b"CONSOLE.IDENTITY.PROCESS.RESULT\0".len()),
        b"CONSOLE.IDENTITY.PROCESS.RESULT\0"
    );
    assert_eq!(reader.take(2), 2u16.to_be_bytes());
    let mut out = json!({});
    for key in [
        "actor_account_id",
        "command_id",
        "group_id",
        "group_incarnation",
    ] {
        out[key] = reader.uuid();
    }
    out["operation"] = json!(u16::from_be_bytes(reader.take(2).try_into().unwrap()));
    out["input_digest"] = reader.digest();
    out["intake_receipt_id"] = reader.uuid();
    out["result_receipt_id"] = reader.uuid();
    out["terminal_code"] = json!(
        match u16::from_be_bytes(reader.take(2).try_into().unwrap()) {
            1 => "ADOPTED",
            2 => "REPLACED",
            3 => "SUSPENDED",
            4 => "REJECTED_STALE_EXPECTATION",
            5 => "REJECTED_PROCESS_EXPIRED",
            6 => "REJECTED_ALREADY_SUSPENDED",
            _ => panic!("unknown result terminal code"),
        }
    );
    out["accepted_at_us"] = json!(reader.signed());
    out["executed_at_us"] = json!(reader.signed());
    out["execution_session_id"] = reader.uuid();
    out["account_security_generation"] = json!(reader.number());
    out["designation_receipt_id"] = reader.uuid();
    out["designation_revision"] = json!(reader.number());
    out["observed_group_revision"] = json!(reader.number());
    out["policy_before"] = reader.policy();
    out["policy_after"] = reader.policy();
    out["schema_id"] = reader.text();
    for key in ["schema_digest", "policy_digest", "codec_contract_digest"] {
        out[key] = reader.digest();
    }
    out["registration_manifest_version"] = json!(reader.number());
    out["registration_manifest_digest"] = reader.digest();
    out["cedar_sdk_version"] = reader.text();
    out["cedar_language_version"] = reader.text();
    out["requested_process_id"] = reader.uuid();
    out["before_head"] = reader.head();
    out["after_head"] = reader.head();
    out["effect_xid"] = json!(reader.number());
    out["effect_backend_pid"] = json!(u32::from_be_bytes(reader.take(4).try_into().unwrap()));
    assert_eq!(
        reader.cursor,
        bytes.len(),
        "result must consume every original byte"
    );
    out
}
fn head_projection(row: &Value) -> Value {
    json!({"process_id":row["process_id"],"head_revision":row["head_revision"],
        "content_version":row["content_version"],"content_digest":row["content_digest"],
        "head_digest":row["head_digest"],"state":row["state"],"expiry_us":micros(&row["expires_at"])})
}
const EXPECTED_ACTION_FIELDS: &str = r#"[{"key":"identity.verifier.process.adopt/1","fields":["process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","title","method","intended_claimant_matching_procedure","account_possession_procedure","physical_human_evidence_procedure","duplicate_contradictory_claim_procedure","qualification_criteria_instruction","escalation_adjudication_procedure","evidence_minimization_retention_description","recipient_responsibility","receipt_locator"]},{"key":"identity.verifier.process.suspend/1","fields":["process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","reason","receipt_locator"]},{"key":"identity.verifier.process.read/1","fields":["group_context","process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","title","method","intended_claimant_matching_procedure","account_possession_procedure","physical_human_evidence_procedure","duplicate_contradictory_claim_procedure","qualification_criteria_instruction","escalation_adjudication_procedure","evidence_minimization_retention_description","recipient_responsibility","history","allowed_actions"]},{"key":"identity.verifier.process.receipt.read-own/1","fields":["command_id","input_digest","intake_receipt_id","result_receipt_id","terminal_code","accepted_at","executed_at","process_id","before_head","after_head","original_content","original_reason","receipt_locator"]}]"#;
// Independent layout1 encoders: no production encoder or declared digest is an oracle.
fn layout1(prefix: &[u8]) -> Vec<u8> {
    let mut bytes = prefix.to_vec();
    bytes.extend_from_slice(&1u16.to_be_bytes());
    bytes
}
fn append_uuid(bytes: &mut Vec<u8>, value: &Value) {
    bytes.extend_from_slice(policy_uuid(value).as_bytes());
}
fn append_positive(bytes: &mut Vec<u8>, value: &Value) {
    bytes.extend_from_slice(&positive(value).to_be_bytes());
}
fn append_digest(bytes: &mut Vec<u8>, value: &Value) {
    let digest = raw(value);
    assert_eq!(digest.len(), 32);
    bytes.extend_from_slice(&digest);
}
fn append_at(bytes: &mut Vec<u8>, value: &Value) {
    bytes.extend_from_slice(&micros(value).to_be_bytes());
}
fn assert_digest(value: &Value, bytes: &[u8]) {
    assert_eq!(
        raw(value),
        Sha256::digest(bytes).as_slice(),
        "independently rebuilt layout digest"
    );
}
fn checked_regular(path: &Path) -> Vec<u8> {
    assert!(path.is_absolute());
    for ancestor in path.ancestors() {
        assert!(
            !std::fs::symlink_metadata(ancestor)
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }
    let metadata = std::fs::metadata(path).unwrap();
    assert!(metadata.is_file() && metadata.len() <= 1_048_576);
    std::fs::read(path).unwrap()
}
fn source_descriptor() -> Value {
    // This future-GREEN fixture is deliberately unavailable on the initial RED.
    // Root must independently review descriptor SHA, Git candidate/source custody,
    // actual installed source/registration IDs and native installer before use.
    // Environment transports that reviewed fixture; it creates no authority.
    let path = PathBuf::from(
        std::env::var_os("CONSOLE_GROUP_PROCESS_SOURCE_DESCRIPTOR").expect(
            "complete root-reviewed installed source descriptor required for business GREEN",
        ),
    );
    let expected = std::env::var("CONSOLE_GROUP_PROCESS_SOURCE_DESCRIPTOR_SHA256")
        .expect("root-reviewed immutable descriptor SHA required for business GREEN");
    assert!(
        expected.len() == 64
            && expected
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    );
    let bytes = checked_regular(&path);
    assert_eq!(hex::encode(Sha256::digest(&bytes)), expected);
    let descriptor: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        descriptor["schema"],
        "console.native_group_process_installed_test_source.v1"
    );
    assert_eq!(descriptor["schema_id"], "native-group-process-v1");
    let candidate = descriptor["source_candidate"].as_str().unwrap();
    assert!(
        candidate.len() == 40
            && candidate
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    );
    let git_root = PathBuf::from(descriptor["git_root"].as_str().unwrap());
    assert!(git_root.is_absolute());
    for ancestor in git_root.ancestors() {
        assert!(
            !std::fs::symlink_metadata(ancestor)
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }
    let git = |arguments: &[&str]| {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(&git_root)
            .args(arguments)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "read-only exact source candidate lookup"
        );
        output.stdout
    };
    assert_eq!(
        String::from_utf8(git(&["rev-parse", "HEAD"]))
            .unwrap()
            .trim(),
        candidate
    );
    let sources = descriptor["sources"].as_array().unwrap();
    assert_eq!(sources.len(), 3);
    let mut keys = BTreeSet::new();
    for source in sources {
        let key = source["key"].as_str().unwrap();
        assert!(["schema_digest", "policy_digest", "codec_contract_digest"].contains(&key));
        assert!(keys.insert(key.to_owned()));
        let relative = Path::new(source["repository_path"].as_str().unwrap());
        assert!(!relative.is_absolute() && !relative.as_os_str().is_empty());
        assert!(
            relative
                .components()
                .all(|c| matches!(c, std::path::Component::Normal(_)))
        );
        let object = format!("{candidate}:{}", relative.to_str().unwrap());
        let blob = String::from_utf8(git(&["rev-parse", &object])).unwrap();
        assert_eq!(blob.trim(), source["git_blob"].as_str().unwrap());
        let staged = checked_regular(Path::new(source["staged_path"].as_str().unwrap()));
        assert_eq!(
            staged,
            git(&["show", &object]),
            "staged source differs from actual candidate Git blob"
        );
        assert_eq!(
            hex::encode(Sha256::digest(&staged)),
            source["sha256"].as_str().unwrap()
        );
        assert_eq!(
            descriptor[key],
            json!(format!("\\x{}", source["sha256"].as_str().unwrap()))
        );
    }
    for key in ["cedar_sdk_version", "cedar_language_version"] {
        text_bytes(&mut Vec::new(), descriptor[key].as_str().unwrap(), 128);
    }
    assert_eq!(descriptor["registration_manifest_version"], 1);
    descriptor
}
fn required_column<'a>(row: &'a Value, key: &str) -> &'a Value {
    row.as_object()
        .unwrap()
        .get(key)
        .expect("complete structured row requires every declared column")
}
fn policy_columns(row: &Value, prefix: &str) -> Value {
    let tag = required_column(row, &format!("{prefix}_tag"));
    let revision = required_column(row, &format!("{prefix}_revision"));
    let digest = required_column(row, &format!("{prefix}_head_digest"));
    match tag.as_u64().unwrap() {
        0 => {
            assert!(revision.is_null() && digest.is_null());
            Value::Null
        }
        1 => {
            assert_eq!(raw(digest).len(), 32);
            json!({"revision":positive(revision),"head_digest":digest})
        }
        _ => panic!("unknown structured policy-reference tag"),
    }
}
fn head_columns(row: &Value, prefix: &str) -> Value {
    let keys = [
        "process_id",
        "head_revision",
        "content_version",
        "content_digest",
        "head_digest",
        "state",
        "expires_at",
    ];
    let values: Vec<&Value> = keys
        .iter()
        .map(|key| required_column(row, &format!("{prefix}_{key}")))
        .collect();
    if values[0].is_null() {
        assert!(
            values.iter().all(|v| v.is_null()),
            "absent process reference has no hidden payload"
        );
        Value::Null
    } else {
        append_uuid(&mut Vec::new(), values[0]);
        positive(values[1]);
        positive(values[2]);
        assert_eq!(raw(values[3]).len(), 32);
        assert_eq!(raw(values[4]).len(), 32);
        assert!(values[5] == "ACTIVE" || values[5] == "SUSPENDED");
        json!({"process_id":values[0],"head_revision":values[1],"content_version":values[2],
            "content_digest":values[3],"head_digest":values[4],"state":values[5],"expiry_us":micros(values[6])})
    }
}
fn assert_bundle(row: &Value, decoded: &Value, source: &Value) {
    for key in [
        "schema_id",
        "schema_digest",
        "policy_digest",
        "codec_contract_digest",
        "registration_manifest_version",
        "cedar_sdk_version",
        "cedar_language_version",
    ] {
        assert_eq!(
            row[key], decoded[key],
            "structured evaluated bundle differs from result bytes"
        );
        assert_eq!(
            row[key], source[key],
            "evaluated bundle differs from independently reviewed actual source"
        );
    }
    assert_eq!(
        row["registration_manifest_digest"],
        decoded["registration_manifest_digest"]
    );
}
fn registration_bytes(policy: &Value, source: &Value) -> Vec<u8> {
    let mut bytes = layout1(b"CONSOLE.IDENTITY.GROUP.PROCESS.REGISTRATION\0");
    append_uuid(&mut bytes, &policy["group_id"]);
    append_uuid(&mut bytes, &policy["group_incarnation"]);
    bytes.extend_from_slice(&1u64.to_be_bytes());
    text_bytes(&mut bytes, source["schema_id"].as_str().unwrap(), 128);
    for key in ["schema_digest", "policy_digest", "codec_contract_digest"] {
        append_digest(&mut bytes, &source[key]);
    }
    let actions = source["registered_actions"].as_array().unwrap();
    assert_eq!(actions.len(), 4);
    assert_eq!(policy["registered_actions"], source["registered_actions"]);
    let expected: Value = serde_json::from_str(EXPECTED_ACTION_FIELDS).unwrap();
    bytes.extend_from_slice(&4u16.to_be_bytes());
    let mut ids = BTreeSet::new();
    for (action, declared) in actions.iter().zip(expected.as_array().unwrap()) {
        assert_eq!(action["key"], declared["key"]);
        assert_eq!(action["fields"], declared["fields"]);
        assert_eq!(action["revision"], 1);
        let id = policy_uuid(&action["action_id"]);
        assert!(ids.insert(id));
        text_bytes(&mut bytes, action["key"].as_str().unwrap(), 128);
        bytes.extend_from_slice(id.as_bytes());
        bytes.extend_from_slice(&1u64.to_be_bytes());
        let fields = declared["fields"].as_array().unwrap();
        bytes.extend_from_slice(&u16::try_from(fields.len()).unwrap().to_be_bytes());
        for field in fields {
            text_bytes(&mut bytes, field.as_str().unwrap(), 128);
        }
    }
    bytes
}
fn policy_head_bytes(row: &Value) -> Vec<u8> {
    let mut bytes = layout1(b"CONSOLE.IDENTITY.GROUP.POLICY.HEAD\0");
    for key in ["group_id", "group_incarnation"] {
        append_uuid(&mut bytes, &row[key]);
    }
    append_positive(&mut bytes, &row["revision"]);
    text_bytes(&mut bytes, row["schema_id"].as_str().unwrap(), 128);
    for key in [
        "schema_digest",
        "policy_digest",
        "codec_contract_digest",
        "registration_manifest_digest",
    ] {
        append_digest(&mut bytes, &row[key]);
    }
    for key in ["first_actor_account_id", "first_command_id"] {
        append_uuid(&mut bytes, &row[key]);
    }
    append_digest(&mut bytes, &row["first_input_digest"]);
    append_uuid(&mut bytes, &row["activation_receipt_id"]);
    append_at(&mut bytes, &row["activated_at"]);
    bytes
}
fn version_bytes(row: &Value) -> Vec<u8> {
    let mut bytes = layout1(b"CONSOLE.IDENTITY.PROCESS.VERSION\0");
    for key in ["group_id", "group_incarnation", "process_id"] {
        append_uuid(&mut bytes, &row[key]);
    }
    append_positive(&mut bytes, &row["version"]);
    assert_eq!(row["schema_id"], "GROUP_VERIFIER_PROCESS_V1");
    text_bytes(&mut bytes, "GROUP_VERIFIER_PROCESS_V1", 128);
    for key in ["actor_account_id", "designation_receipt_id"] {
        append_uuid(&mut bytes, &row[key]);
    }
    for key in ["designation_revision", "policy_revision"] {
        append_positive(&mut bytes, &row[key]);
    }
    append_digest(&mut bytes, &row["policy_head_digest"]);
    append_uuid(&mut bytes, &row["adopt_command_id"]);
    append_digest(&mut bytes, &row["input_digest"]);
    for key in ["admitted_at", "expires_at"] {
        append_at(&mut bytes, &row[key]);
    }
    assert_eq!(row["operator_responsibility"], 1);
    bytes.extend_from_slice(&1u16.to_be_bytes());
    text_bytes(&mut bytes, row["title"].as_str().unwrap(), 120);
    assert_eq!(row["method"], "ATTENDED_ACCOUNT_AND_DOCUMENTARY_REVIEW_V1");
    bytes.extend_from_slice(&1u16.to_be_bytes());
    for key in CONTENT.iter().skip(2) {
        text_bytes(&mut bytes, row[*key].as_str().unwrap(), 2048);
    }
    bytes
}
fn process_head_bytes(row: &Value) -> Vec<u8> {
    let mut bytes = layout1(b"CONSOLE.IDENTITY.PROCESS.HEAD\0");
    for key in ["group_id", "group_incarnation", "process_id"] {
        append_uuid(&mut bytes, &row[key]);
    }
    for key in ["head_revision", "content_version"] {
        append_positive(&mut bytes, &row[key]);
    }
    append_digest(&mut bytes, &row["content_digest"]);
    let state: u16 = match row["state"].as_str().unwrap() {
        "ACTIVE" => 1,
        "SUSPENDED" => 2,
        _ => panic!("unknown persisted process state"),
    };
    bytes.extend_from_slice(&state.to_be_bytes());
    append_at(&mut bytes, &row["expires_at"]);
    for key in ["last_actor_account_id", "last_command_id"] {
        append_uuid(&mut bytes, &row[key]);
    }
    append_digest(&mut bytes, &row["last_input_digest"]);
    append_uuid(&mut bytes, &row["result_receipt_id"]);
    append_at(&mut bytes, &row["updated_at"]);
    bytes
}
fn exact_version(
    snapshot: &BTreeMap<String, String>,
    group: Uuid,
    process: Uuid,
    version: u64,
) -> Value {
    let found: Vec<_> = rows(snapshot, TABLES[1])
        .into_iter()
        .filter(|row| {
            row["group_id"] == json!(group)
                && row["process_id"] == json!(process)
                && row["version"] == json!(version)
        })
        .collect();
    assert_eq!(found.len(), 1);
    found[0].clone()
}
// PostgreSQL xid8 JSON is canonical unsigned decimal text; the frozen wire
// codec is a positive u64. Both actual owner-closure assertions use this rule.
fn assert_xid8_json_projection(database: &Value, wire: &Value) {
    let xid = wire.as_u64().expect("exact unsigned xid8 field");
    assert!(xid > 0);
    assert_eq!(database, &json!(xid.to_string()));
}

fn assert_structured_closure(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
    ready: &Value,
    input: &Value,
    effect: &Value,
    terminal: &Value,
    decoded: &Value,
    policy: &Value,
    head: &Value,
    source: &Value,
    actor: Uuid,
    group: Uuid,
    process: Uuid,
    index: usize,
) {
    for row in [input, effect, terminal] {
        assert_bundle(row, decoded, &source);
    }
    assert_ne!(decoded["group_id"], decoded["group_incarnation"]);
    assert!(decoded["effect_xid"].as_u64().unwrap() > 0);
    assert!((1..=i32::MAX as u64).contains(&decoded["effect_backend_pid"].as_u64().unwrap()));
    assert_eq!(terminal["accepted_at"], input["accepted_at"]);
    assert_eq!(input["codec_version"], 1);
    assert_eq!(terminal["layout_version"], 2);
    assert_eq!(policy["group_incarnation"], decoded["group_incarnation"]);
    assert_eq!(policy["schema_id"], source["schema_id"]);
    for key in ["schema_digest", "policy_digest", "codec_contract_digest"] {
        assert_eq!(policy[key], source[key]);
    }
    assert_digest(
        &policy["registration_manifest_digest"],
        &registration_bytes(policy, &source),
    );
    assert_digest(&policy["head_digest"], &policy_head_bytes(policy));
    assert_eq!(policy["first_actor_account_id"], json!(actor));
    let first_input = selected(
        after,
        TABLES[4],
        actor,
        policy_uuid(&policy["first_command_id"]),
    );
    let first_terminal = selected(
        after,
        TABLES[6],
        actor,
        policy_uuid(&policy["first_command_id"]),
    );
    assert_eq!(policy["first_input_digest"], first_input["input_digest"]);
    assert_eq!(
        policy["activation_receipt_id"],
        first_terminal["result_receipt_id"]
    );
    assert_eq!(policy["activated_at"], first_terminal["executed_at"]);
    if index == 0 {
        assert_eq!(policy["first_command_id"], decoded["command_id"]);
    }
    for row in [effect, terminal] {
        for key in ["policy_before", "policy_after"] {
            assert_eq!(policy_columns(row, key), decoded[key]);
        }
        for key in ["before", "after"] {
            assert_eq!(head_columns(row, key), decoded[format!("{key}_head")]);
        }
        assert_eq!(row["requested_process_id"], json!(process));
        assert_eq!(row["terminal_code"], decoded["terminal_code"]);
        assert_eq!(row["executed_at"], terminal["executed_at"]);
        for key in [
            "execution_session_id",
            "account_security_generation",
            "designation_receipt_id",
            "designation_revision",
            "observed_group_revision",
            "effect_backend_pid",
        ] {
            assert_eq!(row[key], decoded[key]);
        }
        assert_xid8_json_projection(&row["effect_xid"], &decoded["effect_xid"]);
    }
    assert_eq!(
        policy_columns(input, "accepted_policy"),
        decoded["policy_before"]
    );
    assert_eq!(
        input["accepted_session_id"],
        decoded["execution_session_id"]
    );
    for key in [
        "account_security_generation",
        "designation_receipt_id",
        "designation_revision",
        "observed_group_revision",
    ] {
        assert_eq!(input[key], decoded[key]);
    }
    append_uuid(&mut Vec::new(), &effect["effect_id"]);
    assert_eq!(terminal["effect_id"], effect["effect_id"]);
    assert_eq!(
        effect["effect_census"],
        json!({"policy_heads":if index==0{1}else{0},
        "versions":if index==1{0}else{1},"heads":1,"head_revisions":1,"effects":1,"results":1,"audits":1})
    );
    for key in ["group_id", "group_incarnation"] {
        assert_eq!(head[key], decoded[key]);
    }
    for (column, result) in [
        ("last_actor_account_id", "actor_account_id"),
        ("last_command_id", "command_id"),
        ("last_input_digest", "input_digest"),
        ("result_receipt_id", "result_receipt_id"),
    ] {
        assert_eq!(head[column], decoded[result]);
    }
    assert_eq!(head["updated_at"], terminal["executed_at"]);
    assert_digest(&head["head_digest"], &process_head_bytes(head));
    let version = exact_version(after, group, process, positive(&head["content_version"]));
    assert_eq!(head["content_digest"], version["content_digest"]);
    assert_eq!(head["expires_at"], version["expires_at"]);
    assert_eq!(version["group_incarnation"], decoded["group_incarnation"]);
    assert_eq!(version["policy_revision"], policy["revision"]);
    assert_eq!(version["policy_head_digest"], policy["head_digest"]);
    assert_digest(&version["content_digest"], &version_bytes(&version));
    if index != 1 {
        let named = fields(ready);
        for key in CONTENT {
            assert_eq!(
                version[key],
                json!(named[key]),
                "stored structured prose differs from actual submitted fields"
            );
        }
        assert_eq!(version["actor_account_id"], json!(actor));
        assert_eq!(version["adopt_command_id"], decoded["command_id"]);
        assert_eq!(version["input_digest"], input["input_digest"]);
        assert_eq!(version["admitted_at"], input["accepted_at"]);
        assert_eq!(
            version["designation_receipt_id"],
            decoded["designation_receipt_id"]
        );
        assert_eq!(
            version["designation_revision"],
            decoded["designation_revision"]
        );
        let bytes = expected_input(ready, actor, group);
        // Adopt expiry follows fixed command header + process UUID/prior revision.
        let expiry_offset = b"CONSOLE.IDENTITY.GROUP\0".len() + 2 + 2 + 4 * 16 + 2 * 8 + 16 + 8;
        let expiry =
            i64::from_be_bytes(bytes[expiry_offset..expiry_offset + 8].try_into().unwrap());
        assert_eq!(micros(&version["expires_at"]), expiry);
    } else {
        assert_eq!(version, exact_version(before, group, process, 1));
    }
    let history: Vec<_> = rows(after, TABLES[3])
        .into_iter()
        .filter(|row| {
            row["group_id"] == json!(group) && row["head_revision"] == head["head_revision"]
        })
        .collect();
    assert_eq!(history.len(), 1);
    let history = &history[0];
    for key in [
        "group_id",
        "group_incarnation",
        "process_id",
        "head_revision",
        "content_version",
        "content_digest",
        "state",
        "expires_at",
        "last_actor_account_id",
        "last_command_id",
        "last_input_digest",
        "result_receipt_id",
        "updated_at",
        "head_digest",
    ] {
        assert_eq!(
            history[key], head[key],
            "history is exact resulting structured head snapshot"
        );
    }
    assert_digest(&history["head_digest"], &process_head_bytes(history));
    assert_eq!(
        required_column(history, "before_head_digest"),
        &if index == 0 {
            Value::Null
        } else {
            scoped(before, TABLES[2], group)["head_digest"].clone()
        }
    );
}

fn committed(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
    event: &Value,
    ready: &Value,
    source: &Value,
    actor: Uuid,
    group: Uuid,
    process: Uuid,
    index: usize,
) -> Value {
    let command = policy_uuid(&event["command_id"]);
    assert_eq!(ready["command_id"], event["command_id"]);
    let input = selected(after, TABLES[4], actor, command);
    let effect = selected(after, TABLES[5], actor, command);
    let terminal = selected(after, TABLES[6], actor, command);
    let bytes = expected_input(ready, actor, group);
    assert_eq!(
        raw(&input["input_bytes"]),
        bytes,
        "actual immutable input differs from UI fields and actor"
    );
    assert_eq!(
        raw(&input["input_digest"]),
        Sha256::digest(&bytes).as_slice()
    );
    let result_bytes = raw(&terminal["result_bytes"]);
    assert_eq!(
        raw(&terminal["result_digest"]),
        Sha256::digest(&result_bytes).as_slice()
    );
    let decoded = decode_result(&result_bytes);
    for key in [
        "actor_account_id",
        "command_id",
        "group_id",
        "group_incarnation",
        "operation",
        "input_digest",
        "intake_receipt_id",
    ] {
        assert_eq!(input[key], decoded[key]);
        assert_eq!(effect[key], decoded[key]);
        assert_eq!(terminal[key], decoded[key]);
    }
    assert_eq!(decoded["actor_account_id"], json!(actor));
    assert_eq!(decoded["group_id"], json!(group));
    assert_eq!(decoded["requested_process_id"], json!(process));
    assert_eq!(decoded["operation"], ready["operation"]);
    assert_eq!(
        decoded["group_incarnation"],
        json!(fields(ready)["expected_group_incarnation"])
    );
    assert_eq!(decoded["schema_id"], "native-group-process-v1");
    assert_eq!(decoded["registration_manifest_version"], 1);
    assert_eq!(decoded["accepted_at_us"], micros(&input["accepted_at"]));
    assert_eq!(decoded["executed_at_us"], micros(&terminal["executed_at"]));
    assert!(
        decoded["accepted_at_us"].as_i64().unwrap() <= decoded["executed_at_us"].as_i64().unwrap()
    );
    for key in [
        "result_receipt_id",
        "effect_backend_pid",
        "execution_session_id",
        "account_security_generation",
        "designation_receipt_id",
        "designation_revision",
        "observed_group_revision",
        "terminal_code",
    ] {
        assert_eq!(terminal[key], decoded[key]);
        if ["result_receipt_id", "effect_backend_pid"].contains(&key) {
            assert_eq!(effect[key], decoded[key]);
        }
    }
    // PostgreSQL renders xid8 as JSON text; the frozen result codec uses u64BE.
    // Compare the full unsigned value exactly, leaving every other field typed.
    for row in [&terminal, &effect] {
        assert_xid8_json_projection(&row["effect_xid"], &decoded["effect_xid"]);
    }
    let code = ["ADOPTED", "SUSPENDED", "REPLACED"][index];
    assert_eq!(decoded["terminal_code"], code);
    let policy = scoped(after, TABLES[0], group);
    let head = scoped(after, TABLES[2], group);
    let policy_ref =
        json!({"revision":positive(&policy["revision"]),"head_digest":policy["head_digest"]});
    assert_eq!(policy_ref["revision"], 1);
    assert_eq!(decoded["policy_after"], policy_ref);
    assert_eq!(
        decoded["policy_before"],
        if index == 0 {
            Value::Null
        } else {
            policy_ref.clone()
        }
    );
    for key in [
        "schema_digest",
        "policy_digest",
        "codec_contract_digest",
        "registration_manifest_digest",
    ] {
        assert_eq!(decoded[key], policy[key]);
        assert_eq!(raw(&decoded[key]).len(), 32);
    }
    assert_eq!(decoded["after_head"], head_projection(&head));
    assert_eq!(head["process_id"], json!(process));
    assert_eq!(head["head_revision"], index + 1);
    assert_eq!(head["content_version"], if index == 2 { 2 } else { 1 });
    assert_eq!(
        head["state"],
        if index == 1 { "SUSPENDED" } else { "ACTIVE" }
    );
    assert_eq!(
        decoded["before_head"],
        if index == 0 {
            Value::Null
        } else {
            head_projection(&scoped(before, TABLES[2], group))
        }
    );
    assert_structured_closure(
        before, after, ready, &input, &effect, &terminal, &decoded, &policy, &head, source, actor,
        group, process, index,
    );
    let mut allowed = BTreeSet::from([
        TABLES[4],
        TABLES[5],
        TABLES[6],
        TABLES[3],
        "audit_events",
        TABLES[2],
    ]);
    if index == 0 {
        allowed.insert(TABLES[0]);
    }
    if index != 1 {
        allowed.insert(TABLES[1]);
    }
    assert_eq!(
        before.keys().collect::<BTreeSet<_>>(),
        after.keys().collect::<BTreeSet<_>>(),
        "no hidden schema changes during workflow"
    );
    for (name, prior) in before {
        if !allowed.contains(name.as_str()) {
            assert!(
                prior == &after[name],
                "Group command changed unrelated table"
            );
            continue;
        }
        if name == TABLES[2] {
            let prior_rows = rows(before, name);
            let current_rows = rows(after, name);
            let unchanged = |all: Vec<Value>| {
                all.into_iter()
                    .filter(|row| row["group_id"] != json!(group))
                    .collect::<Vec<_>>()
            };
            assert_eq!(unchanged(prior_rows), unchanged(current_rows));
            continue;
        }
        let added = added_rows(prior, &after[name])
            .expect("all acknowledged history remains byte-identical");
        assert_eq!(
            added.len(),
            if name == "audit_events" { 2 } else { 1 },
            "complete effect census must match exact closure"
        );
        for row in &added {
            if name != "audit_events" {
                assert_eq!(row["group_id"], json!(group));
            }
        }
    }
    let audits = added_rows(&before["audit_events"], &after["audit_events"]).unwrap();
    for (action, target, receipt, at) in [
        (
            "identity.group_process.accept",
            TABLES[4],
            &input["intake_receipt_id"],
            &input["accepted_at"],
        ),
        (
            "identity.group_process.complete",
            TABLES[6],
            &terminal["result_receipt_id"],
            &terminal["executed_at"],
        ),
    ] {
        let found: Vec<_> = audits
            .iter()
            .filter(|row| row["action"] == action)
            .collect();
        assert_eq!(found.len(), 1);
        let audit = found[0];
        if action == "identity.group_process.complete" {
            assert_eq!(terminal["audit_id"], audit["id"]);
        }
        assert_eq!(audit["actor"], json!(actor));
        assert!(audit["org_id"].is_null() && audit["before_snap"].is_null());
        assert_eq!(audit["target_type"], target);
        assert_eq!(&audit["target_id"], receipt);
        assert_eq!(&audit["occurred_at"], at);
        let minimal = &audit["after_snap"];
        assert_eq!(minimal["protocol"], "GROUP_PROCESS_V1");
        assert_eq!(minimal["command_id"], json!(command));
        assert_eq!(minimal["group_id"], json!(group));
        assert_eq!(
            minimal["input_digest"],
            hex::encode(raw(&input["input_digest"]))
        );
        let keys = minimal
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let allowed = BTreeSet::from([
            "protocol",
            "command_id",
            "group_id",
            "group_incarnation",
            "operation",
            "input_digest",
            "intake_receipt_id",
            "result_receipt_id",
            "session_id",
            "terminal_code",
        ]);
        assert!(
            keys.is_subset(&allowed),
            "general audit must never contain process prose or credentials"
        );
    }
    let projection = &event["projection"];
    assert_eq!(projection["command_id"], decoded["command_id"]);
    assert_eq!(
        projection["result_receipt_id"],
        decoded["result_receipt_id"]
    );
    assert_eq!(projection["terminal_code"], decoded["terminal_code"]);
    assert_eq!(projection["process_id"], json!(process));
    for (view, column) in [
        ("content_version", "content_version"),
        ("head_revision", "head_revision"),
    ] {
        assert_eq!(projection[view], head[column].as_u64().unwrap().to_string());
    }
    for key in ["content_digest", "head_digest"] {
        assert_eq!(projection[key], hex::encode(raw(&head[key])));
    }
    json!({"process":process,"version":head["content_version"],"head_revision":head["head_revision"],
        "terminal_code":code,"result_receipt_id":decoded["result_receipt_id"]})
}

fn calibrate_census(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
    event: &Value,
    ready: &Value,
    source: &Value,
    actor: Uuid,
    group: Uuid,
    process: Uuid,
    index: usize,
) {
    // Corrupt only detached observed snapshots, never product/DB history.
    // Every transition must detect omitted request/effect/result/head/history/audit.
    let mut changed = vec![
        TABLES[2],
        TABLES[3],
        TABLES[4],
        TABLES[5],
        TABLES[6],
        "audit_events",
    ];
    if index == 0 {
        changed.push(TABLES[0]);
    }
    if index != 1 {
        changed.push(TABLES[1]);
    }
    for table in changed {
        let mut omitted = after.clone();
        omitted.insert(table.to_owned(), before[table].clone());
        let rejected = std::panic::catch_unwind(|| {
            committed(
                before, &omitted, event, ready, source, actor, group, process, index,
            )
        });
        assert!(
            rejected.is_err(),
            "complete owner census accepted an omitted effect/history"
        );
    }
}

fn changed_field(value: &Value) -> Value {
    match value {
        Value::String(text) if text.starts_with("\\x") => {
            let mut bytes = raw(value);
            assert!(!bytes.is_empty());
            bytes[0] ^= 1;
            json!(format!("\\x{}", hex::encode(bytes)))
        }
        Value::String(text) if Uuid::parse_str(text).is_ok() => {
            let changed = json!(Uuid::new_v4());
            assert_ne!(&changed, value);
            changed
        }
        Value::String(text)
            if OffsetDateTime::parse(text, &time::format_description::well_known::Rfc3339)
                .is_ok() =>
        {
            let at =
                OffsetDateTime::from_unix_timestamp_nanos((i128::from(micros(value)) + 1) * 1000)
                    .unwrap();
            json!(
                at.format(&time::format_description::well_known::Rfc3339)
                    .unwrap()
            )
        }
        Value::String(text) => json!(format!("{text} [변조]")),
        Value::Number(_) => json!(value.as_u64().unwrap().checked_add(1).unwrap()),
        Value::Bool(boolean) => json!(!boolean),
        Value::Null => json!(1),
        Value::Array(array) => {
            let mut changed = array.clone();
            assert!(!changed.is_empty());
            changed[0] = Value::Null;
            json!(changed)
        }
        Value::Object(object) => {
            let mut changed = object.clone();
            let key = changed.keys().next().unwrap().clone();
            changed.insert(key, Value::Null);
            json!(changed)
        }
    }
}
fn replace_digest_value(value: &mut Value, original: &[u8], changed: &[u8]) -> usize {
    match value {
        Value::String(text) if text.starts_with("\\x") => {
            let mut bytes = raw(&json!(text));
            let mut count = 0;
            let mut offset = 0;
            while offset + original.len() <= bytes.len() {
                if &bytes[offset..offset + original.len()] == original {
                    bytes[offset..offset + original.len()].copy_from_slice(changed);
                    count += 1;
                    offset += original.len();
                } else {
                    offset += 1;
                }
            }
            if count > 0 {
                *text = format!("\\x{}", hex::encode(bytes));
            }
            count
        }
        Value::String(text) if *text == hex::encode(original) => {
            *text = hex::encode(changed);
            1
        }
        Value::Array(array) => array
            .iter_mut()
            .map(|item| replace_digest_value(item, original, changed))
            .sum(),
        Value::Object(object) => object
            .values_mut()
            .map(|item| replace_digest_value(item, original, changed))
            .sum(),
        _ => 0,
    }
}
fn calibrate_fields_and_shared_digests(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
    event: &Value,
    ready: &Value,
    source: &Value,
    actor: Uuid,
    group: Uuid,
    process: Uuid,
    index: usize,
) {
    let command = event["command_id"].clone();
    let version_number = if index == 2 { 2 } else { 1 };
    let specifications: Vec<(&str, Vec<&str>)> = vec![
        (
            TABLES[0],
            vec![
                "group_incarnation",
                "revision",
                "schema_id",
                "schema_digest",
                "policy_digest",
                "codec_contract_digest",
                "registration_manifest_digest",
                "registered_actions",
                "first_actor_account_id",
                "first_command_id",
                "first_input_digest",
                "activation_receipt_id",
                "activated_at",
                "head_digest",
            ],
        ),
        (
            TABLES[1],
            vec![
                "group_incarnation",
                "process_id",
                "version",
                "schema_id",
                "actor_account_id",
                "designation_receipt_id",
                "designation_revision",
                "policy_revision",
                "policy_head_digest",
                "adopt_command_id",
                "input_digest",
                "admitted_at",
                "expires_at",
                "operator_responsibility",
                "title",
                "method",
                "intended_claimant_matching_procedure",
                "account_possession_procedure",
                "physical_human_evidence_procedure",
                "duplicate_contradictory_claim_procedure",
                "qualification_criteria_instruction",
                "escalation_adjudication_procedure",
                "evidence_minimization_retention_description",
                "recipient_responsibility",
                "content_digest",
            ],
        ),
        (
            TABLES[2],
            vec![
                "group_incarnation",
                "process_id",
                "head_revision",
                "content_version",
                "content_digest",
                "state",
                "expires_at",
                "last_actor_account_id",
                "last_command_id",
                "last_input_digest",
                "result_receipt_id",
                "updated_at",
                "head_digest",
            ],
        ),
        (
            TABLES[3],
            vec![
                "group_incarnation",
                "process_id",
                "head_revision",
                "content_version",
                "content_digest",
                "state",
                "expires_at",
                "last_actor_account_id",
                "last_command_id",
                "last_input_digest",
                "result_receipt_id",
                "updated_at",
                "head_digest",
                "before_head_digest",
            ],
        ),
        (
            TABLES[4],
            vec![
                "codec_version",
                "group_incarnation",
                "operation",
                "input_bytes",
                "input_digest",
                "intake_receipt_id",
                "accepted_at",
                "accepted_session_id",
                "account_security_generation",
                "designation_receipt_id",
                "designation_revision",
                "observed_group_revision",
                "accepted_policy_tag",
                "accepted_policy_revision",
                "accepted_policy_head_digest",
                "schema_id",
                "schema_digest",
                "policy_digest",
                "codec_contract_digest",
                "registration_manifest_version",
                "registration_manifest_digest",
                "cedar_sdk_version",
                "cedar_language_version",
            ],
        ),
        (
            TABLES[5],
            vec![
                "effect_id",
                "group_incarnation",
                "operation",
                "input_digest",
                "intake_receipt_id",
                "result_receipt_id",
                "terminal_code",
                "requested_process_id",
                "executed_at",
                "execution_session_id",
                "account_security_generation",
                "designation_receipt_id",
                "designation_revision",
                "observed_group_revision",
                "effect_xid",
                "effect_backend_pid",
                "effect_census",
                "policy_before_tag",
                "policy_before_revision",
                "policy_before_head_digest",
                "policy_after_tag",
                "policy_after_revision",
                "policy_after_head_digest",
                "before_process_id",
                "before_head_revision",
                "before_content_version",
                "before_content_digest",
                "before_head_digest",
                "before_state",
                "before_expires_at",
                "after_process_id",
                "after_head_revision",
                "after_content_version",
                "after_content_digest",
                "after_head_digest",
                "after_state",
                "after_expires_at",
                "schema_id",
                "schema_digest",
                "policy_digest",
                "codec_contract_digest",
                "registration_manifest_version",
                "registration_manifest_digest",
                "cedar_sdk_version",
                "cedar_language_version",
            ],
        ),
        (
            TABLES[6],
            vec![
                "layout_version",
                "effect_id",
                "audit_id",
                "group_incarnation",
                "operation",
                "input_digest",
                "intake_receipt_id",
                "result_receipt_id",
                "accepted_at",
                "result_bytes",
                "result_digest",
                "terminal_code",
                "requested_process_id",
                "executed_at",
                "execution_session_id",
                "account_security_generation",
                "designation_receipt_id",
                "designation_revision",
                "observed_group_revision",
                "effect_xid",
                "effect_backend_pid",
                "policy_before_tag",
                "policy_before_revision",
                "policy_before_head_digest",
                "policy_after_tag",
                "policy_after_revision",
                "policy_after_head_digest",
                "before_process_id",
                "before_head_revision",
                "before_content_version",
                "before_content_digest",
                "before_head_digest",
                "before_state",
                "before_expires_at",
                "after_process_id",
                "after_head_revision",
                "after_content_version",
                "after_content_digest",
                "after_head_digest",
                "after_state",
                "after_expires_at",
                "schema_id",
                "schema_digest",
                "policy_digest",
                "codec_contract_digest",
                "registration_manifest_version",
                "registration_manifest_digest",
                "cedar_sdk_version",
                "cedar_language_version",
            ],
        ),
    ];
    for (table, columns) in specifications {
        let observed = rows(after, table);
        let matches: Vec<usize> = observed
            .iter()
            .enumerate()
            .filter_map(|(position, row)| {
                let selected = if [TABLES[4], TABLES[5], TABLES[6]].contains(&table) {
                    row["actor_account_id"] == json!(actor) && row["command_id"] == command
                } else if table == TABLES[1] {
                    row["group_id"] == json!(group)
                        && row["process_id"] == json!(process)
                        && row["version"] == json!(version_number)
                } else if table == TABLES[3] {
                    row["group_id"] == json!(group) && row["head_revision"] == json!(index + 1)
                } else {
                    row["group_id"] == json!(group)
                };
                selected.then_some(position)
            })
            .collect();
        assert_eq!(matches.len(), 1);
        let target = matches[0];
        for column in columns {
            assert!(
                observed[target].as_object().unwrap().contains_key(column),
                "required structured column missing from real source"
            );
            for missing in [false, true] {
                let mut detached = observed.clone();
                if missing {
                    detached[target].as_object_mut().unwrap().remove(column);
                } else {
                    detached[target][column] = changed_field(&observed[target][column]);
                }
                assert_ne!(detached, observed);
                assert_eq!(detached.len(), observed.len());
                let mut corrupt = after.clone();
                corrupt.insert(table.to_owned(), serde_json::to_string(&detached).unwrap());
                assert!(
                    std::panic::catch_unwind(|| committed(
                        before, &corrupt, event, ready, source, actor, group, process, index
                    ))
                    .is_err(),
                    "same-count missing/changed inner field escaped independent owner oracle: {table}.{column}"
                );
            }
        }
    }
    if index == 0 {
        let policy = scoped(after, TABLES[0], group);
        let head = scoped(after, TABLES[2], group);
        for digest in [
            &policy["head_digest"],
            &head["content_digest"],
            &head["head_digest"],
        ] {
            let original = raw(digest);
            assert_eq!(original.len(), 32);
            let mut changed = original.clone();
            changed[0] ^= 1;
            let mut corrupt = after.clone();
            let mut replacements = 0;
            for table in TABLES {
                let mut detached = json!(rows(after, table));
                replacements += replace_digest_value(&mut detached, &original, &changed);
                if table == TABLES[6] {
                    for row in detached.as_array_mut().unwrap() {
                        row["result_digest"] = json!(format!(
                            "\\x{}",
                            hex::encode(Sha256::digest(raw(&row["result_bytes"])))
                        ));
                    }
                }
                corrupt.insert(table.to_owned(), serde_json::to_string(&detached).unwrap());
            }
            let mut matching_event = event.clone();
            replace_digest_value(&mut matching_event, &original, &changed);
            assert!(
                replacements >= 3,
                "shared-digest calibration must change real reciprocal copies"
            );
            let rejected = std::panic::catch_unwind(|| {
                committed(
                    before,
                    &corrupt,
                    &matching_event,
                    ready,
                    source,
                    actor,
                    group,
                    process,
                    index,
                )
            })
            .unwrap_err();
            let message = rejected
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| rejected.downcast_ref::<&str>().copied())
                .unwrap_or("");
            assert!(
                message.contains("independently rebuilt layout digest"),
                "mutually matching rows/bytes/projection must fail at independent layout reconstruction: {message}"
            );
        }
    }
}

pub(super) async fn observe(
    pool: &PgPool,
    input: &mut tokio::process::ChildStdin,
    events: &mut tokio::io::BufReader<tokio::process::ChildStdout>,
    actor: Uuid,
    company: Uuid,
    group: Uuid,
) {
    let mut prior = all_rows(pool).await;
    let mut pending: Option<Value> = None;
    let mut process = None;
    let mut committed_count = 0;
    let mut original_result: Option<Value> = None;
    let mut original_projection: Option<Value> = None;
    let mut started: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(pool)
        .await
        .unwrap();
    for phase in PHASES {
        let event = browser_owner_event(events).await;
        assert_eq!(event["kind"], "CHECKPOINT");
        assert_eq!(event["phase"], phase);
        assert_eq!(event["account_id"], json!(actor));
        assert_eq!(event["org_id"], json!(company));
        assert_eq!(event["group_id"], json!(group));
        let now: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(pool)
            .await
            .unwrap();
        let mut current = all_rows(pool).await;
        let authority =
            policy_only_row(&current, "group_authority_heads", "group_id", &json!(group));
        let mut answer = json!({"phase":phase,"group":group,"company":company,"account":actor,
            "owner_effects_verified":true,"observed_at_us":i64::try_from(now.unix_timestamp_nanos()/1000).unwrap()});
        match phase {
            "GROUP_ENTRY_READY" => {
                assert!(prior == current, "Group entry read changed durable state");
                // Group successor must be installed and reviewed before this can be GREEN.
                for table in TABLES {
                    assert!(
                        rows(&current, table).is_empty(),
                        "fresh UI Group has no prepopulated process business rows"
                    );
                }
                answer["policy_revision"] = json!(0);
                answer["version"] = json!(0);
                answer["head_revision"] = json!(0);
            }
            "GROUP_ADOPT_FORM_READY"
            | "GROUP_SUSPEND_FORM_READY"
            | "GROUP_REPLACE_FORM_READY"
            | "GROUP_RETRY_FORM_READY" => {
                assert!(
                    policy_preflight_effects(&prior, &current, started, now),
                    "one actual Auth proof admission only"
                );
                if phase != "GROUP_RETRY_FORM_READY" {
                    let named = fields(&event);
                    assert_eq!(
                        named["expected_group_revision"],
                        authority["revision"].as_u64().unwrap().to_string()
                    );
                    assert_eq!(
                        named["expected_group_incarnation"],
                        authority["incarnation"].as_str().unwrap()
                    );
                    let current_process = policy_uuid(&event["process_id"]);
                    if let Some(original) = process {
                        assert_eq!(original, current_process);
                    } else {
                        process = Some(current_process);
                    }
                    let expected_policy = if committed_count == 0 { "0" } else { "1" };
                    assert_eq!(
                        named["expected_group_identity_policy_revision"],
                        expected_policy
                    );
                }
            }
            "GROUP_ADOPT_READY" | "GROUP_SUSPEND_READY" | "GROUP_REPLACE_READY" => {
                assert!(
                    prior == current,
                    "client input created durable effects before submission"
                );
                assert!(pending.is_none());
                let mut ready = event.clone();
                ready["process_id"] = json!(process.unwrap());
                let _ = expected_input(&ready, actor, group);
                pending = Some(ready);
            }
            "GROUP_ADOPTED" | "GROUP_SUSPENDED" | "GROUP_REPLACED" => {
                let ready = pending.take().unwrap();
                let source = source_descriptor();
                let details = committed(
                    &prior,
                    &current,
                    &event,
                    &ready,
                    &source,
                    actor,
                    group,
                    process.unwrap(),
                    committed_count,
                );
                calibrate_census(
                    &prior,
                    &current,
                    &event,
                    &ready,
                    &source,
                    actor,
                    group,
                    process.unwrap(),
                    committed_count,
                );
                calibrate_fields_and_shared_digests(
                    &prior,
                    &current,
                    &event,
                    &ready,
                    &source,
                    actor,
                    group,
                    process.unwrap(),
                    committed_count,
                );
                if committed_count == 0 {
                    let command = policy_uuid(&event["command_id"]);
                    original_result = Some(selected(&current, TABLES[6], actor, command));
                    original_projection = Some(event["projection"].clone());
                }
                for (key, value) in details.as_object().unwrap() {
                    answer[key] = value.clone();
                }
                committed_count += 1;
            }
            "GROUP_CURRENT_ACTIVE" | "GROUP_CURRENT_SUSPENDED" | "GROUP_CURRENT_REPLACED" => {
                assert!(
                    prior == current,
                    "current process reopening wrote business state"
                );
                let head = scoped(&current, TABLES[2], group);
                answer["process"] = head["process_id"].clone();
                answer["version"] = head["content_version"].clone();
                answer["head_revision"] = head["head_revision"].clone();
            }
            "GROUP_DESIGNATION_REVOKE_READY" => {
                assert!(
                    prior == current,
                    "revocation preparation must not write process state"
                );
                let existing = designation(pool, actor).await;
                let operational = startup(pool).await;
                let command = Uuid::new_v4();
                let receipt = revoke(
                    &operational,
                    &existing,
                    command,
                    1,
                    "actual aggregate receipt-retention control",
                )
                .await
                .unwrap();
                operational.close().await;
                assert_eq!(receipt.1, 2);
                assert!(!receipt.2);
                current = all_rows(pool).await;
                let allowed =
                    BTreeSet::from(["deployment_operator_head", "deployment_operator_receipts"]);
                for (table, rows) in &prior {
                    if !allowed.contains(table.as_str()) {
                        assert!(
                            rows == &current[table],
                            "designation owner changed unrelated state"
                        );
                    }
                }
                let terminal = original_result.as_ref().unwrap();
                assert_eq!(
                    selected(
                        &current,
                        TABLES[6],
                        actor,
                        policy_uuid(&terminal["command_id"])
                    ),
                    *terminal
                );
            }
            "GROUP_ADOPT_REOPENED"
            | "GROUP_SUSPEND_REOPENED"
            | "GROUP_REPLACE_REOPENED"
            | "GROUP_ORIGINAL_RECEIPT_REOPENED"
            | "GROUP_OWN_RECEIPT_AFTER_REVOKE"
            | "GROUP_TERMINAL_REPLAY_READY"
            | "GROUP_TERMINAL_REPLAYED"
            | "GROUP_CURRENT_DENIED" => {
                assert!(
                    prior == current,
                    "receipt/status/terminal replay/denied read wrote durable state"
                );
                if [
                    "GROUP_ORIGINAL_RECEIPT_REOPENED",
                    "GROUP_OWN_RECEIPT_AFTER_REVOKE",
                ]
                .contains(&phase)
                {
                    assert_eq!(event["projection"], *original_projection.as_ref().unwrap());
                }
                if let Some(terminal) = &original_result {
                    assert_eq!(
                        selected(
                            &current,
                            TABLES[6],
                            actor,
                            policy_uuid(&terminal["command_id"])
                        ),
                        *terminal
                    );
                }
            }
            _ => panic!("unrecognized Group process checkpoint"),
        }
        if let Some(process) = process {
            answer["process"] = json!(process);
        }
        prior = current;
        started = now;
        policy_browser_ack(input, phase, answer).await;
    }
    assert_eq!(committed_count, 3);
    assert!(pending.is_none());
}

#[sqlx::test(migrations = false)]
async fn group_result_xid8_wire_matches_exact_postgresql_json_text(pool: PgPool) {
    for xid in [
        1_u64,
        9_007_199_254_740_993,
        i64::MAX as u64,
        1_u64 << 63,
        u64::MAX,
    ] {
        let (database_json, wire): (Value, Vec<u8>) =
            sqlx::query_as("SELECT to_jsonb($1::text::xid8), xid8send($1::text::xid8)")
                .bind(xid.to_string())
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(database_json, json!(xid.to_string()));
        assert_eq!(wire, xid.to_be_bytes());
        assert_ne!(database_json, json!(xid));
    }
}

#[test]
fn group_result_xid8_projection_rejects_noncanonical_or_mismatched_values() {
    let rejects = |database: &Value, wire: &Value| {
        std::panic::catch_unwind(|| assert_xid8_json_projection(database, wire)).is_err()
    };
    for xid in [
        1_u64,
        9_007_199_254_740_993,
        i64::MAX as u64,
        1_u64 << 63,
        u64::MAX,
    ] {
        let text = xid.to_string();
        let wire = json!(xid);
        let database = json!(text);
        assert_xid8_json_projection(&database, &wire);
        let other = if xid == u64::MAX { xid - 1 } else { xid + 1 };
        for corrupt in [
            json!(xid),
            json!(other.to_string()),
            json!(format!("0{text}")),
            json!(format!("+{text}")),
            json!(format!("-{text}")),
            json!(format!(" {text}")),
            json!(format!("{text} ")),
            json!(format!("{text}\n")),
            json!(format!("{text}\r\n")),
            json!(format!("{text}.0")),
            json!(format!("{text}x")),
            json!([text]),
            json!({"value":text}),
            json!(true),
            Value::Null,
        ] {
            assert!(rejects(&corrupt, &wire));
        }
        for corrupt in [
            json!(text),
            json!(other),
            json!(0),
            json!(-1),
            json!(1.0),
            json!([xid]),
            json!({"value":xid}),
            json!(true),
            Value::Null,
        ] {
            assert!(rejects(&database, &corrupt));
        }
    }
    assert!(rejects(&json!("0"), &json!(0)));
}
