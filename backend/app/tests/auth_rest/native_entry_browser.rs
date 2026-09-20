// Private test candidate. Included in account_browser only with test-browser.
// The actual app and restricted owners serve an empty real browser context.
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

async fn browser_owner_event(
    reader: &mut tokio::io::BufReader<tokio::process::ChildStdout>,
) -> Value {
    let mut line = Vec::new();
    tokio::time::timeout(
        std::time::Duration::from_secs(35),
        reader.take(8193).read_until(b'\n', &mut line),
    )
    .await
    .expect("browser event deadline")
    .expect("browser event read");
    assert!(
        !line.is_empty() && line.len() <= 8192 && line.last() == Some(&b'\n'),
        "bounded browser event required"
    );
    serde_json::from_slice(&line).expect("browser event must be JSON")
}

async fn browser_owner_continue(input: &mut tokio::process::ChildStdin, phase: &str) {
    let mut bytes = serde_json::to_vec(&json!({"kind":"CONTINUE","phase":phase})).unwrap();
    bytes.push(b'\n');
    input
        .write_all(&bytes)
        .await
        .expect("browser checkpoint acknowledgment");
    input.flush().await.unwrap();
}

fn browser_checkpoint(event: &Value, phase: &str) -> Uuid {
    exact_keys(event, &["kind", "phase", "account_id"]);
    assert!(
        event["kind"] == "CHECKPOINT" && event["phase"] == phase,
        "browser checkpoint missing or out of order; inspect sanitized result"
    );
    let text = event["account_id"]
        .as_str()
        .expect("checkpoint Account UUID");
    let id = Uuid::parse_str(text).expect("checkpoint Account UUID");
    assert!(!id.is_nil() && id.hyphenated().to_string() == text);
    id
}

fn browser_login_history(before: &Value, after: &Value, account: Uuid, consumed_at: &Value) {
    for name in ["security", "terms", "events"] {
        assert!(
            before[name] == after[name],
            "primary login changed retained Account history"
        );
    }
    let old_families = before["families"].as_array().unwrap();
    let families = after["families"].as_array().unwrap();
    assert_eq!(old_families.len(), 1);
    assert_eq!(families.len(), 2);
    assert!(
        families.contains(&old_families[0]),
        "old revoked family lost or changed"
    );
    let fresh = families
        .iter()
        .find(|f| f["id"] != old_families[0]["id"])
        .unwrap();
    assert!(fresh["user_id"] == account.to_string() && fresh["org_id"].is_null());
    assert!(fresh["protocol"] == "ACCOUNT_V1" && fresh["revoked_at"].is_null());
    let tokens = after["tokens"].as_array().unwrap();
    let old_tokens = before["tokens"].as_array().unwrap();
    assert_eq!(old_tokens.len(), 1);
    assert_eq!(tokens.len(), 2);
    assert!(
        tokens.contains(&old_tokens[0]),
        "old revoked token lost or changed"
    );
    let token = tokens
        .iter()
        .find(|t| t["id"] != old_tokens[0]["id"])
        .unwrap();
    assert!(
        token["family_id"] == fresh["id"]
            && token["revoked_at"].is_null()
            && token["used_at"].is_null()
    );
    let keys = after["keys"].as_array().unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!(before["keys"].as_array().unwrap().len(), 1);
    let old_key = &before["keys"][0];
    let counter = keys[0]["passkey_json"]["cred"]["counter"].as_u64().unwrap();
    assert!(
        counter > old_key["passkey_json"]["cred"]["counter"].as_u64().unwrap(),
        "virtual authenticator counter did not advance"
    );
    assert!(consumed_at.is_string() && keys[0]["last_used_at"] == *consumed_at);
    let mut expected_key = old_key.clone();
    expected_key["last_used_at"] = consumed_at.clone();
    expected_key["passkey_json"]["cred"]["counter"] = json!(counter);
    assert!(
        keys[0] == expected_key,
        "login changed key material beyond counter and use time"
    );
    // Discoverable login starts without an Account and retains a null user_id;
    // its new ceremony is checked separately against the complete global roster.
    assert!(
        before["ceremonies"] == after["ceremonies"],
        "login changed registration history"
    );
}

#[derive(Clone, Copy)]
enum BrowserCheckpointPhase {
    Enrolled,
    LoggedOut,
    LoggedIn,
}

// Subtract the complete historical row set using PostgreSQL-rendered bytes.
// Value equality could round numeric fields inside historical JSON. The source
// tables have primary keys, so a duplicate full baseline row is invalid evidence.
fn browser_added_rows(before: &str, after: &str) -> Option<Vec<Value>> {
    use serde_json::value::RawValue;
    let before: Vec<&RawValue> = serde_json::from_str(before).ok()?;
    let after: Vec<&RawValue> = serde_json::from_str(after).ok()?;
    if !before
        .iter()
        .chain(&after)
        .all(|row| row.get().trim_start().starts_with('{'))
    {
        return None;
    }
    let mut retained = BTreeSet::new();
    for row in &before {
        if !retained.insert(row.get()) {
            return None;
        }
    }
    let mut added = Vec::new();
    for row in &after {
        if !retained.remove(row.get()) {
            added.push(serde_json::from_str(row.get()).ok()?);
        }
    }
    retained.is_empty().then_some(added)
}

// Custody preserves migration-owned legacy Accounts. Every baseline row must
// remain byte-identical; cardinality and ownership apply to the exact additions.
fn browser_global_effects(
    baseline: &BTreeMap<String, String>,
    current: &BTreeMap<String, String>,
    account: Uuid,
    phase: BrowserCheckpointPhase,
    origin: &str,
) -> bool {
    if account.is_nil()
        || !native_extension_rows_equal(baseline, baseline)
        || !native_extension_rows_equal(current, current)
    {
        eprintln!("native_browser_global: roster_or_account");
        return false;
    }
    let logged_in = matches!(phase, BrowserCheckpointPhase::LoggedIn);
    let after_logout = !matches!(phase, BrowserCheckpointPhase::Enrolled);
    let owned = [
        ("accounts", "id", 1usize),
        ("account_security", "account_id", 1),
        (
            "account_security_events",
            "account_id",
            if after_logout { 4 } else { 3 },
        ),
        ("account_terms_acceptances", "account_id", 2),
        ("auth_webauthn_credentials", "user_id", 1),
        (
            "auth_refresh_token_families",
            "user_id",
            if logged_in { 2 } else { 1 },
        ),
        (
            "auth_refresh_tokens",
            "user_id",
            if logged_in { 2 } else { 1 },
        ),
    ];
    let account_text = account.to_string();
    let mut changed: BTreeSet<&str> = BTreeSet::new();
    for (table, owner, count) in owned {
        changed.insert(table);
        let before: Value = serde_json::from_str(&baseline[table]).unwrap();
        let Some(rows) = browser_added_rows(&baseline[table], &current[table]) else {
            eprintln!("native_browser_global: baseline_preservation {table}");
            return false;
        };
        if before
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row[owner].as_str() == Some(account_text.as_str()))
            || rows.len() != count
            || !rows
                .iter()
                .all(|row| row[owner].as_str() == Some(account_text.as_str()))
        {
            eprintln!("native_browser_global: owned_table_shape {table}");
            return false;
        }
        if matches!(
            table,
            "auth_webauthn_credentials" | "auth_refresh_token_families"
        ) && !rows
            .iter()
            .all(|row| row.get("org_id").is_some_and(Value::is_null))
        {
            eprintln!("native_browser_global: company_free_scope {table}");
            return false;
        }
        if table == "account_security_events" {
            if !rows
                .iter()
                .all(|row| row["actor_account_id"].as_str() == Some(account_text.as_str()))
            {
                eprintln!("native_browser_global: event_actor");
                return false;
            }
            for (kind, expected) in [
                ("TERMS_ACCEPTED", 2),
                ("ENROLLED", 1),
                ("SESSION_REVOKED", usize::from(after_logout)),
            ] {
                if rows.iter().filter(|row| row["kind"] == kind).count() != expected {
                    eprintln!("native_browser_global: event_kind {kind}");
                    return false;
                }
            }
        }
    }
    changed.insert("auth_webauthn_ceremonies");
    let before: Value = serde_json::from_str(&baseline["auth_webauthn_ceremonies"]).unwrap();
    let Some(rows) = browser_added_rows(
        &baseline["auth_webauthn_ceremonies"],
        &current["auth_webauthn_ceremonies"],
    ) else {
        eprintln!("native_browser_global: baseline_preservation auth_webauthn_ceremonies");
        return false;
    };
    if before
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["user_id"].as_str() == Some(account_text.as_str()))
        || rows.len() != if logged_in { 2 } else { 1 }
    {
        eprintln!("native_browser_global: ceremony_cardinality");
        return false;
    }
    if !rows
        .iter()
        .all(|row| row["browser_origin"] == origin && row["consumed_at"].is_string())
    {
        eprintln!("native_browser_global: ceremony_origin_or_consumption");
        return false;
    }
    if rows
        .iter()
        .filter(|row| {
            row["ceremony_kind"] == "registration"
                && row["account_browser_flow"] == "ACCOUNT_REGISTRATION"
                && row["user_id"].as_str() == Some(account_text.as_str())
        })
        .count()
        != 1
    {
        eprintln!("native_browser_global: registration_binding");
        return false;
    }
    if logged_in
        && rows
            .iter()
            .filter(|row| {
                row["ceremony_kind"] == "authentication"
                    && row["account_browser_flow"] == "ACCOUNT_LOGIN"
                    && row.get("user_id").is_some_and(Value::is_null)
            })
            .count()
            != 1
    {
        eprintln!("native_browser_global: login_binding");
        return false;
    }
    baseline
        .iter()
        .filter(|(table, _)| !changed.contains(table.as_str()))
        .all(|(table, before)| {
            let retained = current.get(table) == Some(before);
            if !retained {
                eprintln!("native_browser_global: retained_table {table}");
            }
            retained
        })
}

const BROWSER_BUSINESS_TABLES: [&str; 3] = ["users", "organizations", "groups"];

// Native Account entry may not invent or alter unrelated Company identities.
// Independent scope supplements the existing15Account-table roster.
async fn browser_business_rows(pool: &PgPool) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    for table in BROWSER_BUSINESS_TABLES {
        let sql = format!(
            "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text COLLATE \"C\"),'[]'::jsonb)::text FROM public.{table} t"
        );
        let rows: String = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .fetch_one(pool)
            .await
            .unwrap();
        assert!(result.insert(table.to_owned(), rows).is_none());
    }
    assert!(browser_business_rows_equal(&result, &result));
    result
}

fn browser_business_rows_equal(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
) -> bool {
    let expected: BTreeSet<_> = BROWSER_BUSINESS_TABLES.into_iter().collect();
    let complete = |rows: &BTreeMap<String, String>| {
        rows.keys().map(String::as_str).collect::<BTreeSet<_>>() == expected
            && rows
                .values()
                .all(|raw| serde_json::from_str::<Value>(raw).is_ok_and(|value| value.is_array()))
    };
    complete(before) && complete(after) && before == after
}

#[test]
fn browser_business_oracle_requires_complete_lossless_unchanged_rows() {
    let baseline: BTreeMap<String, String> = BROWSER_BUSINESS_TABLES
        .into_iter()
        .map(|table| {
            (
                table.to_owned(),
                r#"[{"id":"retained","precise":9007199254740993}]"#.to_owned(),
            )
        })
        .collect();
    assert!(browser_business_rows_equal(&baseline, &baseline));
    assert!(!browser_business_rows_equal(
        &BTreeMap::new(),
        &BTreeMap::new()
    ));
    for table in BROWSER_BUSINESS_TABLES {
        let mut missing = baseline.clone();
        missing.remove(table);
        assert!(!browser_business_rows_equal(&baseline, &missing));
        assert!(!browser_business_rows_equal(&missing, &missing));
        for value in [
            "[]",
            "null",
            "{}",
            "invalid",
            r#"[{"id":"retained","precise":9007199254740992}]"#,
            r#"[{"id":"unrelated"},{"id":"retained","precise":9007199254740993}]"#,
        ] {
            let mut changed = baseline.clone();
            changed.insert(table.to_owned(), value.to_owned());
            assert!(!browser_business_rows_equal(&baseline, &changed));
        }
    }
    let mut extra = baseline.clone();
    extra.insert("unexpected".to_owned(), "[]".to_owned());
    assert!(!browser_business_rows_equal(&extra, &extra));
}

fn browser_pid_alive(pid: u32) -> Option<bool> {
    // Node exposes ESRCH distinctly from EPERM/observer failure without adding
    // an unsafe syscall or platform library. Signal zero sends no signal.
    let output = std::process::Command::new("node")
        .arg("-e")
        .arg("try{process.kill(Number(process.argv[1]),0);process.stdout.write('ALIVE')}catch(e){process.stdout.write(e.code==='ESRCH'?'ABSENT':'UNKNOWN')}")
        .arg(pid.to_string())
        .env_remove("DEBUG")
        .env_remove("PWDEBUG")
        .env_remove("NODE_DEBUG")
        .stdin(std::process::Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() || !output.stderr.is_empty() {
        return None;
    }
    match output.stdout.as_slice() {
        b"ALIVE" => Some(true),
        b"ABSENT" => Some(false),
        _ => None,
    }
}

#[test]
fn browser_login_history_oracle_requires_exact_key_and_retained_effects() {
    let account = Uuid::from_u128(1);
    let consumed_at = json!("2026-09-16T12:00:01Z");
    // Machinery-only values. Real acceptance uses DB snapshots, never these rows.
    let before = json!({
        "security":{"security_state":"ACTIVE","security_generation":1},
        "terms":[{"id":"retained-consent"}],
        "events":[{"id":"retained-event"}],
        "ceremonies":[{"id":"registration","consumed_at":"earlier"}],
        "keys":[{"id":"key","user_id":account,"org_id":null,
            "credential_id":"retained-credential","last_used_at":null,"created_at":"earlier",
            "passkey_json":{"cred":{"counter":1,"cred":"retained-public-key","user_verified":true,
                "backup_eligible":false,"backup_state":false},"policy":"required"}}],
        "families":[{"id":"old-family","user_id":account,"org_id":null,
            "protocol":"ACCOUNT_V1","revoked_at":"earlier","retained":"family-history"}],
        "tokens":[{"id":"old-token","family_id":"old-family","user_id":account,
            "revoked_at":"earlier","used_at":null,"retained":"token-history"}]
    });
    let mut after = before.clone();
    after["keys"][0]["last_used_at"] = consumed_at.clone();
    after["keys"][0]["passkey_json"]["cred"]["counter"] = json!(2);
    after["families"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"new-family","user_id":account,
        "org_id":null,"protocol":"ACCOUNT_V1","revoked_at":null}));
    after["tokens"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"new-token","user_id":account,
        "family_id":"new-family","revoked_at":null,"used_at":null}));
    browser_login_history(&before, &after, account, &consumed_at);
    let rejects = |candidate: &Value| {
        std::panic::catch_unwind(|| {
            browser_login_history(&before, candidate, account, &consumed_at)
        })
        .is_err()
    };
    for path in [
        "/keys/0/id",
        "/keys/0/user_id",
        "/keys/0/org_id",
        "/keys/0/credential_id",
        "/keys/0/created_at",
        "/keys/0/passkey_json/cred/cred",
        "/keys/0/passkey_json/cred/user_verified",
        "/keys/0/passkey_json/cred/backup_eligible",
        "/keys/0/passkey_json/cred/backup_state",
        "/keys/0/passkey_json/policy",
        "/keys/0/last_used_at",
        "/families/0/retained",
        "/tokens/0/retained",
        "/families/1/user_id",
        "/families/1/org_id",
        "/families/1/protocol",
        "/families/1/revoked_at",
        "/tokens/1/family_id",
        "/tokens/1/revoked_at",
        "/tokens/1/used_at",
    ] {
        let mut bad = after.clone();
        *bad.pointer_mut(path).unwrap() = json!("tampered");
        assert!(rejects(&bad), "tampered field escaped oracle: {path}");
    }
    for name in [
        "security",
        "terms",
        "events",
        "ceremonies",
        "keys",
        "families",
        "tokens",
    ] {
        let mut bad = after.clone();
        bad.as_object_mut().unwrap().remove(name);
        assert!(rejects(&bad), "omitted effect escaped oracle: {name}");
    }
    for name in ["families", "tokens"] {
        let mut bad = after.clone();
        bad[name].as_array_mut().unwrap().pop();
        assert!(rejects(&bad), "omitted new effect escaped oracle: {name}");
    }
    let mut bad = after.clone();
    bad["keys"][0]["passkey_json"]["cred"]["counter"] = json!(1);
    assert!(rejects(&bad), "counter update omitted");
    let mut bad = after.clone();
    bad["keys"][0]["unexpected"] = json!(true);
    assert!(
        rejects(&bad),
        "unexpected key field escaped exact comparison"
    );
}

#[test]
fn browser_global_oracle_requires_complete_owned_effects_and_immutable_tables() {
    let account = Uuid::from_u128(1);
    let other = Uuid::from_u128(2);
    let origin = "https://localhost:12345";
    let mut baseline: BTreeMap<String, String> = NATIVE_EXTENSION_TABLES
        .into_iter()
        .map(|table| (table.to_owned(), "[]".to_owned()))
        .collect();
    baseline.insert(
        "_sqlx_migrations".to_owned(),
        "[{\"version\":227}]".to_owned(),
    );
    baseline.insert("account_terms_head".to_owned(), "[{\"id\":1}]".to_owned());
    baseline.insert(
        "account_terms_release_receipts".to_owned(),
        "[{\"id\":\"retained\"}]".to_owned(),
    );
    for phase in [
        BrowserCheckpointPhase::Enrolled,
        BrowserCheckpointPhase::LoggedOut,
        BrowserCheckpointPhase::LoggedIn,
    ] {
        let logged_in = matches!(phase, BrowserCheckpointPhase::LoggedIn);
        let after_logout = !matches!(phase, BrowserCheckpointPhase::Enrolled);
        let mut current = baseline.clone();
        for (table, owner, count) in [
            ("accounts", "id", 1),
            ("account_security", "account_id", 1),
            ("account_terms_acceptances", "account_id", 2),
            ("auth_webauthn_credentials", "user_id", 1),
            (
                "auth_refresh_token_families",
                "user_id",
                if logged_in { 2 } else { 1 },
            ),
            (
                "auth_refresh_tokens",
                "user_id",
                if logged_in { 2 } else { 1 },
            ),
        ] {
            let row = json!({owner:account.to_string(),"org_id":null});
            current.insert(
                table.to_owned(),
                serde_json::to_string(&vec![row; count]).unwrap(),
            );
        }
        let mut events = vec![
            json!({"account_id":account.to_string(),"actor_account_id":account.to_string(),"kind":"ENROLLED"}),
        ];
        for _ in 0..2 {
            events.push(json!({"account_id":account.to_string(),"actor_account_id":account.to_string(),"kind":"TERMS_ACCEPTED"}));
        }
        if after_logout {
            events.push(json!({"account_id":account.to_string(),"actor_account_id":account.to_string(),"kind":"SESSION_REVOKED"}));
        }
        current.insert(
            "account_security_events".to_owned(),
            serde_json::to_string(&events).unwrap(),
        );
        let mut ceremonies = vec![
            json!({"user_id":account.to_string(),"browser_origin":origin,"ceremony_kind":"registration","account_browser_flow":"ACCOUNT_REGISTRATION","consumed_at":"observed"}),
        ];
        if logged_in {
            ceremonies.push(json!({"user_id":null,"browser_origin":origin,"ceremony_kind":"authentication","account_browser_flow":"ACCOUNT_LOGIN","consumed_at":"observed"}));
        }
        current.insert(
            "auth_webauthn_ceremonies".to_owned(),
            serde_json::to_string(&ceremonies).unwrap(),
        );
        assert!(browser_global_effects(
            &baseline, &current, account, phase, origin
        ));
        for table in NATIVE_EXTENSION_TABLES {
            let mut missing = current.clone();
            missing.remove(table);
            assert!(!browser_global_effects(
                &baseline, &missing, account, phase, origin
            ));
        }
        for table in [
            "accounts",
            "auth_webauthn_credentials",
            "auth_refresh_tokens",
            "account_terms_acceptances",
            "account_security_events",
            "auth_webauthn_ceremonies",
        ] {
            let mut omitted = current.clone();
            omitted.insert(table.to_owned(), "[]".to_owned());
            assert!(!browser_global_effects(
                &baseline, &omitted, account, phase, origin
            ));
        }
        for table in [
            "accounts",
            "account_context_candidates",
            "company_actors",
            "auth_bootstrap_credentials",
            "auth_webauthn_ceremony_bindings",
            "account_terms_head",
        ] {
            let mut extra = current.clone();
            let mut rows: Value = serde_json::from_str(&extra[table]).unwrap();
            rows.as_array_mut()
                .unwrap()
                .push(json!({"id":other.to_string()}));
            extra.insert(table.to_owned(), serde_json::to_string(&rows).unwrap());
            assert!(!browser_global_effects(
                &baseline, &extra, account, phase, origin
            ));
        }
        let mut wrong = current.clone();
        wrong.insert(
            "accounts".to_owned(),
            json!([{"id":other.to_string()}]).to_string(),
        );
        assert!(!browser_global_effects(
            &baseline, &wrong, account, phase, origin
        ));
        let mut wrong = current.clone();
        let mut rows: Value = serde_json::from_str(&wrong["auth_webauthn_credentials"]).unwrap();
        rows[0]["org_id"] = json!(other.to_string());
        wrong.insert("auth_webauthn_credentials".to_owned(), rows.to_string());
        assert!(!browser_global_effects(
            &baseline, &wrong, account, phase, origin
        ));
        assert!(!browser_global_effects(
            &baseline,
            &current,
            account,
            phase,
            "https://foreign.invalid"
        ));

        // Synthetic oracle controls, not DB fixtures: each changed table has
        // unrelated prior rows, including opaque numeric data that Value would
        // round. Historical event kinds/origins/org scope need not match the
        // newly enrolled Account; every historical byte must instead survive.
        let mut populated_baseline = baseline.clone();
        let mut populated_current = current.clone();
        let old_tables = [
            ("accounts", "id"),
            ("account_security", "account_id"),
            ("account_security_events", "account_id"),
            ("account_terms_acceptances", "account_id"),
            ("auth_webauthn_credentials", "user_id"),
            ("auth_refresh_token_families", "user_id"),
            ("auth_refresh_tokens", "user_id"),
            ("auth_webauthn_ceremonies", "user_id"),
        ];
        let mut retained_rows = BTreeMap::new();
        for (table, owner) in old_tables {
            let old = json!({owner:other.to_string(), "org_id":other.to_string(),
                "created_at":"1970-01-01T00:00:00Z", "kind":"OLD_EVENT",
                "browser_origin":"https://historical.invalid", "consumed_at":null,
                "exact_decimal":"9007199254740993.1"})
            .to_string()
            .replace("\"9007199254740993.1\"", "9007199254740993.1");
            populated_baseline.insert(table.to_owned(), format!("[{old}]"));
            let additions = current[table].strip_prefix('[').unwrap();
            populated_current.insert(table.to_owned(), format!("[{old},{additions}"));
            retained_rows.insert(table, old);
        }
        assert!(
            browser_global_effects(
                &populated_baseline,
                &populated_current,
                account,
                phase,
                origin
            ),
            "populated legacy baseline must survive each native phase"
        );
        for (table, owner) in old_tables {
            // Removing prior rows while preserving all required new effects is
            // still data loss. Omitting new effects while retaining old rows is
            // still an incomplete command. Neither may pass by total count.
            let mut deleted = populated_current.clone();
            deleted.insert(table.to_owned(), current[table].clone());
            assert!(
                !browser_global_effects(&populated_baseline, &deleted, account, phase, origin),
                "deleted historical row {table}"
            );
            let mut omitted = populated_current.clone();
            omitted.insert(table.to_owned(), populated_baseline[table].clone());
            assert!(
                !browser_global_effects(&populated_baseline, &omitted, account, phase, origin),
                "omitted new effect {table}"
            );
            for (before, after) in [
                ("9007199254740993.1", "9007199254740993.2"),
                ("1970-01-01T00:00:00Z", "2000-01-01T00:00:00Z"),
            ] {
                let mut mutated = populated_current.clone();
                mutated.insert(
                    table.to_owned(),
                    populated_current[table].replace(before, after),
                );
                assert!(
                    !browser_global_effects(&populated_baseline, &mutated, account, phase, origin),
                    "historical numeric or creation-time mutation {table}"
                );
            }
            let old = &retained_rows[table];
            for extra_row in [
                old.clone(),
                old.replace(&other.to_string(), &Uuid::from_u128(3).to_string()),
            ] {
                let mut extra = populated_current.clone();
                extra.insert(
                    table.to_owned(),
                    format!("[{extra_row},{}", &populated_current[table][1..]),
                );
                assert!(
                    !browser_global_effects(&populated_baseline, &extra, account, phase, origin),
                    "extra unrelated row or historical duplicate {table}"
                );
            }
            let mut misattributed = populated_current.clone();
            let mut additions: Value = serde_json::from_str(&current[table]).unwrap();
            additions[0][owner] = json!(other.to_string());
            misattributed.insert(
                table.to_owned(),
                format!("[{old},{}", &additions.to_string()[1..]),
            );
            assert!(
                !browser_global_effects(
                    &populated_baseline,
                    &misattributed,
                    account,
                    phase,
                    origin
                ),
                "misattributed new effect {table}"
            );
            let mut duplicate_baseline = populated_baseline.clone();
            let mut duplicate_current = populated_current.clone();
            duplicate_baseline.insert(table.to_owned(), format!("[{old},{old}]"));
            duplicate_current.insert(
                table.to_owned(),
                format!("[{old},{old},{}", &current[table][1..]),
            );
            assert!(
                !browser_global_effects(
                    &duplicate_baseline,
                    &duplicate_current,
                    account,
                    phase,
                    origin
                ),
                "duplicate historical baseline is invalid evidence {table}"
            );
            // A command cannot claim an already-existing target as enrollment,
            // even if it appends the otherwise expected number of new rows.
            let mut preexisting_baseline = populated_baseline.clone();
            let mut preexisting_current = populated_current.clone();
            let target_old = old.replace(&other.to_string(), &account.to_string());
            preexisting_baseline.insert(table.to_owned(), format!("[{target_old}]"));
            preexisting_current.insert(
                table.to_owned(),
                format!("[{target_old},{}", &current[table][1..]),
            );
            assert!(
                !browser_global_effects(
                    &preexisting_baseline,
                    &preexisting_current,
                    account,
                    phase,
                    origin
                ),
                "target already present in baseline {table}"
            );
        }
        // Cardinality alone cannot prove preservation: replacing a distinct
        // historical row with a duplicate retained row must fail. A duplicated
        // baseline is itself invalid; distinct historical rows may reorder.
        let old = &retained_rows["accounts"];
        let other_old = old.replace(&other.to_string(), &Uuid::from_u128(3).to_string());
        let added = current["accounts"]
            .trim_start_matches('[')
            .trim_end_matches(']');
        assert!(
            browser_added_rows(
                &format!("[{old},{other_old}]"),
                &format!("[{old},{old},{added}]")
            )
            .is_none()
        );
        assert!(
            browser_added_rows(&format!("[{old},{old}]"), &format!("[{old},{old},{added}]"))
                .is_none()
        );
        let reordered = browser_added_rows(
            &format!("[{old},{other_old}]"),
            &format!("[{other_old},{added},{old}]"),
        )
        .unwrap();
        assert!(reordered == serde_json::from_str::<Vec<Value>>(&current["accounts"]).unwrap());
        assert!(browser_added_rows("[null]", "[null]").is_none());
        assert!(browser_added_rows("[]", "[null]").is_none());
    }
}

#[sqlx::test(migrations = false)]
async fn native_entry_real_browser_enroll_logout_login(pool: PgPool) {
    use futures::FutureExt;
    use std::process::Stdio;
    // Explicit local evidence prerequisites, never a silently skipped browser test.
    let driver = PathBuf::from(
        std::env::var_os("CONSOLE_BROWSER_JOURNEY_DRIVER")
            .expect("reviewed browser driver required"),
    );
    let expected =
        std::env::var("CONSOLE_BROWSER_JOURNEY_SHA256").expect("reviewed driver SHA256 required");
    assert!(
        driver.is_absolute()
            && !std::fs::symlink_metadata(&driver)
                .unwrap()
                .file_type()
                .is_symlink()
    );
    let driver_bytes = std::fs::read(&driver).unwrap();
    assert_eq!(
        hex::encode(Sha256::digest(&driver_bytes)),
        expected,
        "reviewed browser source differs"
    );
    let output = PathBuf::from(
        std::env::var_os("CONSOLE_BROWSER_JOURNEY_OUTPUT").expect("fresh browser output required"),
    );
    assert!(
        output.is_absolute() && !output.exists(),
        "browser output must be a fresh owned directory"
    );
    prepare_http_database(&pool).await;
    seed_terms(&pool).await;
    let artifacts = Artifacts::new();
    let key = SigningKey::random(&mut OsRng);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let baseline = native_extension_rows(&pool).await;
    let business_baseline = browser_business_rows(&pool).await;
    let mut child = tokio::process::Command::new("node")
        .arg(&driver)
        .arg(address.port().to_string())
        .arg(&output)
        .env_remove("DEBUG")
        .env_remove("PWDEBUG")
        .env_remove("NODE_DEBUG")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .expect("browser driver launch prerequisite");
    let mut input = child.stdin.take().unwrap();
    let mut events = tokio::io::BufReader::new(child.stdout.take().unwrap());
    let mut server = None;
    let mut shutdown = None;
    let mut state_to_close = None;
    let mut checkpoint_receipts = Vec::new();
    let mut owned_browser_pid = None;
    let mut browser_seen_alive = false;
    let outcome = std::panic::AssertUnwindSafe(async {
        let ready = browser_owner_event(&mut events).await;
        exact_keys(
            &ready,
            &[
                "kind",
                "origin",
                "rp_id",
                "tls_spki_sha256",
                "upstream_port",
            ],
        );
        assert!(ready["kind"] == "READY" && ready["rp_id"] == "localhost");
        assert_eq!(
            ready["upstream_port"].as_u64(),
            Some(u64::from(address.port()))
        );
        let origin = ready["origin"].as_str().unwrap();
        let parsed = url::Url::parse(origin).unwrap();
        assert!(
            parsed.scheme() == "https"
                && parsed.host_str() == Some("localhost")
                && parsed.port().is_some()
                && parsed.username().is_empty()
                && parsed.password().is_none()
                && parsed.path() == "/"
                && parsed.query().is_none()
                && parsed.fragment().is_none()
        );
        assert_eq!(
            base64::engine::general_purpose::STANDARD
                .decode(ready["tls_spki_sha256"].as_str().unwrap())
                .unwrap()
                .len(),
            32
        );
        let mut config = account_browser_config(&pool, artifacts.root.clone(), &key);
        let auth = config.auth_rest.as_mut().unwrap();
        auth.rp_id = "localhost".to_owned();
        auth.rp_origin = origin.to_owned();
        auth.cookie_secure = true;
        let state = AppState::from_config(config)
            .await
            .expect("real native browser app prerequisite");
        state_to_close = Some(state.clone());
        let router = build_router(state);
        let (stop, stopped) = tokio::sync::oneshot::channel();
        shutdown = Some(stop);
        server = Some(tokio::spawn(async move {
            axum::serve(
                listener,
                router.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(async {
                let _ = stopped.await;
            })
            .await
        }));
        assert!(
            native_extension_rows_equal(&baseline, &native_extension_rows(&pool).await),
            "serving startup changed Account data"
        );
        assert!(
            browser_business_rows_equal(&business_baseline, &browser_business_rows(&pool).await),
            "startup changed Company identity state"
        );
        input.write_all(b"{\"kind\":\"START\"}\n").await.unwrap();
        input.flush().await.unwrap();
        let owned = browser_owner_event(&mut events).await;
        exact_keys(&owned, &["kind", "pid", "executable_sha256"]);
        assert!(
            owned["kind"] == "BROWSER_OWNED" && owned["pid"].as_u64().is_some_and(|pid| pid > 1)
        );
        assert_eq!(
            owned["executable_sha256"],
            match (std::env::consts::OS, std::env::consts::ARCH) {
                ("macos", "aarch64") =>
                    "a0bfe7b4da4787b66058477d696cd1d09065d25f06a548947722b9af77ee8282",
                ("linux", "x86_64") =>
                    "ded93a9c9a53a1ae040f08124badcca95c938e9d5015ff340c3b5538c41bf39e",
                _ => panic!("unsupported reviewed browser platform"),
            }
        );

        let pid = u32::try_from(owned["pid"].as_u64().unwrap()).unwrap();
        owned_browser_pid = Some(pid);
        browser_seen_alive = browser_pid_alive(pid) == Some(true);
        assert!(browser_seen_alive, "owned browser PID not observed alive");

        let enrolled_event = browser_owner_event(&mut events).await;
        let account = browser_checkpoint(&enrolled_event, "ENROLLED");
        let ceremonies: Vec<Uuid> =
            sqlx::query_scalar("SELECT id FROM auth_webauthn_ceremonies WHERE user_id=$1")
                .bind(account)
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(ceremonies.len(), 1);
        assert_account_committed_ids(&pool, account, ceremonies[0]).await;
        let enrolled_global = native_extension_rows(&pool).await;
        assert!(
            browser_global_effects(
                &baseline,
                &enrolled_global,
                account,
                BrowserCheckpointPhase::Enrolled,
                origin
            ),
            "enrollment changed unowned global state or omitted an effect"
        );
        let enrolled = snapshot(&pool, account).await;
        assert_eq!(enrolled["tokens"].as_array().unwrap().len(), 1);
        assert!(
            enrolled["families"][0]["revoked_at"].is_null()
                && enrolled["tokens"][0]["revoked_at"].is_null()
        );
        assert!(
            browser_business_rows_equal(&business_baseline, &browser_business_rows(&pool).await),
            "ENROLLED changed unrelated legacy users, Companies or Groups"
        );
        checkpoint_receipts.push("ENROLLED");
        browser_owner_continue(&mut input, "ENROLLED").await;

        let logout_event = browser_owner_event(&mut events).await;
        assert_eq!(browser_checkpoint(&logout_event, "LOGGED_OUT"), account);
        let logged_out_global = native_extension_rows(&pool).await;
        assert!(
            browser_global_effects(
                &baseline,
                &logged_out_global,
                account,
                BrowserCheckpointPhase::LoggedOut,
                origin
            ),
            "logout changed unowned global state or omitted an effect"
        );
        assert!(
            enrolled_global["accounts"] == logged_out_global["accounts"],
            "logout changed persistent Account identity"
        );
        let logged_out = snapshot(&pool, account).await;
        assert_logout_transition(&enrolled, &logged_out, account);
        let old_ceremonies = native_login_ceremony_rows(&pool).await;
        assert!(
            browser_business_rows_equal(&business_baseline, &browser_business_rows(&pool).await),
            "LOGGED_OUT changed unrelated legacy users, Companies or Groups"
        );
        checkpoint_receipts.push("LOGGED_OUT");
        browser_owner_continue(&mut input, "LOGGED_OUT").await;

        let login_event = browser_owner_event(&mut events).await;
        assert_eq!(browser_checkpoint(&login_event, "LOGGED_IN"), account);
        let logged_in = snapshot(&pool, account).await;
        let logged_in_global = native_extension_rows(&pool).await;
        assert!(
            browser_global_effects(
                &baseline,
                &logged_in_global,
                account,
                BrowserCheckpointPhase::LoggedIn,
                origin
            ),
            "login changed unowned global state or omitted an effect"
        );
        assert!(
            enrolled_global["accounts"] == logged_in_global["accounts"],
            "login changed persistent Account identity"
        );
        let all_ceremonies = native_login_ceremony_rows(&pool).await;
        let previous = old_ceremonies.as_array().unwrap();
        let current = all_ceremonies.as_array().unwrap();
        assert_eq!(current.len(), previous.len() + 1);
        assert!(
            previous.iter().all(|row| current.contains(row)),
            "login lost old global ceremony history"
        );
        let added = current.iter().find(|row| !previous.contains(row)).unwrap();
        assert!(
            added["ceremony_kind"] == "authentication"
                && added["consumed_at"].is_string()
                && added["user_id"].is_null(),
            "real discoverable ceremony must be consumed"
        );
        browser_login_history(&logged_out, &logged_in, account, &added["consumed_at"]);
        assert_no_company_identity(&pool, account).await;
        assert!(
            browser_business_rows_equal(&business_baseline, &browser_business_rows(&pool).await),
            "LOGGED_IN changed unrelated legacy users, Companies or Groups"
        );
        checkpoint_receipts.push("LOGGED_IN");
        browser_owner_continue(&mut input, "LOGGED_IN").await;
        let final_event = browser_owner_event(&mut events).await;
        exact_keys(&final_event, &["kind", "status", "result_path"]);
        assert!(
            final_event["kind"] == "RESULT" && final_event["status"] == "BROWSER_LEAF_PASSED",
            "browser leaf failed; inspect sanitized result"
        );
        assert_eq!(
            final_event["result_path"].as_str(),
            output.join("result.json").to_str()
        );
    })
    .catch_unwind()
    .await;

    // Even failed assertions close stdin so the owner-controlled browser driver
    // cleans up its exact browser/TLS relay before this test propagates failure.
    drop(input);
    let child_status = tokio::time::timeout(std::time::Duration::from_secs(40), child.wait()).await;
    if child_status.is_err() {
        let _ = child.kill().await;
    }
    if let Some(stop) = shutdown {
        let _ = stop.send(());
    }
    let mut server_clean = true;
    if let Some(mut task) = server {
        match tokio::time::timeout(std::time::Duration::from_secs(5), &mut task).await {
            Ok(Ok(Ok(()))) => {}
            _ => {
                server_clean = false;
                task.abort();
                let _ = task.await;
            }
        }
    }
    if let Some(state) = state_to_close {
        state.shutdown_realtime().await;
    }
    let browser_exit_observation = owned_browser_pid.and_then(browser_pid_alive);
    let browser_exit_confirmed = browser_seen_alive && browser_exit_observation == Some(false);
    let source_unchanged = std::fs::read(&driver).is_ok_and(|bytes| bytes == driver_bytes);
    let exit_ok = matches!(child_status, Ok(Ok(status)) if status.success());
    let receipt = json!({"kind":"INDEPENDENT_NATIVE_UI_DATABASE_CHECKPOINTS","checkpoints":checkpoint_receipts,"source_unchanged":source_unchanged,"driver_exit_success":exit_ok,"server_shutdown":server_clean,"browser_pid":owned_browser_pid,"browser_seen_alive":browser_seen_alive,"browser_pid_exit_confirmed":browser_exit_confirmed,"browser_final_alive_observation":browser_exit_observation,"limits":"TEST_ONLY publication seed; synthetic browser authenticator; no production exposure or human usability certification"});
    if output.is_dir() {
        std::fs::write(
            output.join("owner-receipt.json"),
            serde_json::to_vec_pretty(&receipt).unwrap(),
        )
        .unwrap();
    }
    assert!(
        source_unchanged && server_clean && (owned_browser_pid.is_none() || browser_exit_confirmed),
        "browser source or owned app/browser PID cleanup failed"
    );
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
    assert!(
        exit_ok && browser_exit_confirmed,
        "browser did not exit successfully with its owned PID absent after all checkpoints"
    );
}
