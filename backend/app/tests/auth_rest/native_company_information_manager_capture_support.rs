// Helpers are intentionally local to the Manager prerequisite leaf.
fn manager_source_pins() -> Value {
    let mut pins = BTreeMap::new();
    for (name, source, expected) in [
        (
            "owner",
            SOURCE,
            "b6385f8c1ce7859011376dbb0df1b89e02673a14a5598d6261a3bf3e4b903291",
        ),
        (
            "capture",
            QUERY,
            "4913e6fcf501de10353c314c69368335082476ee4d8e6ce89cd0c2325b281a94",
        ),
        (
            "old_capture",
            OLD_QUERY,
            "d5d2c45c691383ddde9ac213d9fe2674db0750f3ceabdc733eaaf2a570d937e3",
        ),
        (
            "policy_v1",
            POLICY_INSTALLER,
            "4e7fc41b1d2ed6c2155d44d43347c18815ed9e70611eb9285f21e1c590bf996a",
        ),
        (
            "policy_classifier",
            POLICY_CLASSIFIER,
            "072794defc065f8730eafeab66a111bdd2439c49b8c1d7c3052ca83a7edf1479",
        ),
        (
            "app_policy_classifier",
            APP_POLICY_CLASSIFIER,
            "072794defc065f8730eafeab66a111bdd2439c49b8c1d7c3052ca83a7edf1479",
        ),
        (
            "account",
            ACCOUNT,
            "fb532ef3b82f6d03b58d6e164a26567683039444098cf1a0a98d33e27e5dd4f8",
        ),
        (
            "credentials",
            CREDENTIALS,
            "2f960163c9bd8832cdd3a058a6ae69d7503cf1009c476ef292af9443036624d9",
        ),
        (
            "company",
            COMPANY,
            "bc35b52d5692e474a3c890dde73112f56e7b85a241ab390075d58b5d5b43a92f",
        ),
        (
            "company_decoder",
            include_str!("../../../../ops/postgres-company-enrollment-input.sql"),
            "cbc685bff861fec809b930e57673cce5426a771a236323471955510de0631290",
        ),
        (
            "observer",
            OBSERVER,
            "ffe7038b43de0207d6ae3e3ce0487498edc4d061c21e4249abacc91abaacdbbc",
        ),
        (
            "ledger231",
            LEDGER,
            "42079d3f1b8077e163960adc65f35f1959c22a67bf42acf43d6b816721ba1357",
        ),
        (
            "classifier_session",
            CLASSIFIER_SESSION,
            "728593aa17d371220a9f62bfe5bf4ecab5096d41eba7c8852907845ae65a87bd",
        ),
    ] {
        assert_eq!(
            digest(source),
            expected,
            "unreviewed declared {name} source"
        );
        assert!(pins.insert(name, expected).is_none());
    }
    json!(pins)
}

async fn policy_capture(connection: &mut PgConnection, source: &'static str) -> Capture {
    sqlx::raw_sql(CLASSIFIER_SESSION)
        .execute(&mut *connection)
        .await
        .unwrap();
    let body = source
        .strip_suffix(";\n")
        .expect("exact declared query terminator");
    let query = format!(
        "SELECT snapshot::text,snapshot_sha256,native_policy_startup_rights_valid FROM ({body}) original_capture"
    );
    let (text, sha256, rights): (String, String, Option<bool>) =
        sqlx::query_as(sqlx::AssertSqlSafe(query))
            .fetch_one(connection)
            .await
            .expect("metadata capture must execute; SQL failure is not measured custody");
    assert_eq!(
        digest(&text),
        sha256,
        "database hash differs from raw UTF-8 text"
    );
    let rights = rights.expect("capture must report the actual startup rights result");
    let snapshot = serde_json::from_str::<Value>(&text).unwrap();
    assert!(snapshot.is_object());
    Capture {
        text,
        sha256,
        rights,
        snapshot,
    }
}

fn absent_extension(old: &Capture, extended: &Capture) {
    let old_fields = old.snapshot.as_object().unwrap();
    let extended_fields = extended.snapshot.as_object().unwrap();
    assert!(!old_fields.contains_key(NAMESPACE_KEY));
    assert_eq!(extended_fields.len(), old_fields.len() + 1);
    assert!(extended_fields.get(NAMESPACE_KEY).unwrap().is_null());
    for (key, value) in old_fields {
        assert_eq!(
            extended_fields.get(key),
            Some(value),
            "changed inherited field: {key}"
        );
    }
    // Compare structure without rewriting either historical raw capture.
    assert_ne!(old.text, extended.text);
    assert_ne!(old.sha256, extended.sha256);
}

async fn manager_observer_absent(connection: &mut PgConnection) {
    let absent: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_durability_observer') AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname='public' AND p.proname='console_durability_observation_v1')")
        .fetch_one(connection).await.unwrap();
    assert!(
        absent,
        "dedicated capture must have actually absent observer"
    );
}

async fn manager_namespace(connection: &mut PgConnection) -> Value {
    sqlx::query_scalar("SELECT jsonb_agg(jsonb_build_array(n.nspname,p.proname,pg_catalog.pg_get_function_identity_arguments(p.oid),p.prokind,pg_catalog.pg_get_userbyid(p.proowner),p.prosecdef) ORDER BY n.nspname,p.proname,pg_catalog.pg_get_function_identity_arguments(p.oid)) FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace WHERE starts_with(p.proname,'identity_company_information_')")
        .fetch_one(connection).await.map(|v: Option<Value>| v.unwrap_or(Value::Null)).unwrap()
}
