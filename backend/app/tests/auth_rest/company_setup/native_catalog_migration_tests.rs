use super::*;

const NATIVE_VERSION: &str = "native-company-identity-2026-09-19.1";
const NATIVE_DIGEST: &str = "\\x0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935";

pub(super) fn catalog_allowlist_preserved(old: &str, new: &str) -> bool {
    use serde_json::value::RawValue;
    let check = || -> Option<()> {
        let before: Vec<&RawValue> = serde_json::from_str(old).ok()?;
        let after: Vec<&RawValue> = serde_json::from_str(new).ok()?;
        if after.len() != before.len() + 1 {
            return None;
        }
        let mut remaining = Vec::new();
        let mut additions = 0;
        for row in &before {
            let fields: BTreeMap<String, &RawValue> = serde_json::from_str(row.get()).ok()?;
            if serde_json::from_str::<String>(fields.get("catalog_version")?.get()).ok()?
                == NATIVE_VERSION
            {
                return None;
            }
        }
        for row in after {
            let fields: BTreeMap<String, &RawValue> = serde_json::from_str(row.get()).ok()?;
            if serde_json::from_str::<String>(fields.get("catalog_version")?.get()).ok()?
                != NATIVE_VERSION
            {
                remaining.push(row.get());
                continue;
            }
            additions += 1;
            if fields.keys().map(String::as_str).collect::<BTreeSet<_>>()
                != BTreeSet::from(["catalog_version", "created_at", "manifest_digest"])
            {
                return None;
            }
            if serde_json::from_str::<String>(fields.get("manifest_digest")?.get()).ok()?
                != NATIVE_DIGEST
            {
                return None;
            }
            let text: String = serde_json::from_str(fields.get("created_at")?.get()).ok()?;
            OffsetDateTime::parse(&text, &time::format_description::well_known::Rfc3339).ok()?;
        }
        let mut before: Vec<_> = before.iter().map(|r| r.get()).collect();
        before.sort_unstable();
        remaining.sort_unstable();
        (additions == 1 && before == remaining).then_some(())
    };
    check().is_some()
}

pub(super) async fn populate_legacy_catalog(pool: &PgPool, org: Uuid, user: Uuid) {
    use console_ontology_adapter_postgres::seed::{
        BUILTIN_CATALOG_VERSION, builtin_catalog_manifest,
    };
    let command = login_test_pool(pool, TestDatabaseLogin::OntologyCommand).await;
    let manifest = builtin_catalog_manifest().unwrap();
    let expected_count = manifest["object_types"].as_array().unwrap().len() as i64;
    let before = all_rows(pool).await;
    let mut tx = command.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org',$1,true)")
        .bind(org.to_string())
        .execute(tx.as_mut())
        .await
        .unwrap();
    let result:(bool,i64)=sqlx::query_as("SELECT installed,object_type_count FROM ontology_api.install_builtin_catalog($1,$2,$3,$4,$5,$6)")
        .bind(org).bind(BUILTIN_CATALOG_VERSION).bind(&manifest).bind(user)
        .bind("00112233445566778899aabbccddeeff").bind("0011223344556677").fetch_one(tx.as_mut()).await.unwrap();
    assert_eq!(result, (true, expected_count));
    let object:Uuid=sqlx::query_scalar("SELECT id FROM public.ont_object_types WHERE org_id=$1 AND stable_key='customer' AND lifecycle_state='published'")
        .bind(org).fetch_one(tx.as_mut()).await.unwrap();
    let row = json!({"effect":"forbid","action":"view","resource_type":"customer","conditions":[]});
    let policy: Uuid =
        sqlx::query_scalar("SELECT ont_policy_api.attach_object_policy($1,$2,$3,$4,$5,$6,$7,$8)")
            .bind(org)
            .bind(user)
            .bind(object)
            .bind("forbid")
            .bind(row)
            .bind("ontology-runtime-filter-v1")
            .bind("00112233445566778899aabbccddeeff")
            .bind("0011223344556677")
            .fetch_one(tx.as_mut())
            .await
            .unwrap();
    assert!(!policy.is_nil());
    tx.commit().await.unwrap();
    let after = all_rows(pool).await;
    let audits = added_rows(&before["audit_events"], &after["audit_events"]).unwrap();
    assert_eq!(audits.len(), expected_count as usize + 2);
    assert_eq!(
        audits
            .iter()
            .filter(|a| a["action"] == "ontology.object_type.builtin_install")
            .count(),
        expected_count as usize
    );
    assert_eq!(
        audits
            .iter()
            .filter(|a| a["action"] == "ontology.builtin_catalog.install")
            .count(),
        1,
        "legacy marker audit must remain"
    );
    assert_eq!(
        audits
            .iter()
            .filter(|a| a["action"] == "ontology.object_policy.attach")
            .count(),
        1
    );
    for table in [
        "ont_object_types",
        "ont_builtin_catalog_installs",
        "cedar_policy_catalog_entries",
        "ont_object_policies",
        "ont_property_defs",
        "ont_action_types",
        "ont_object_type_key_revisions",
    ] {
        assert!(
            !identity_rows(&after, table, org).is_empty(),
            "genuine legacy catalog prerequisite empty: {table}"
        );
    }
    command.close().await;
}

pub(super) async fn assert_catalog_migration(
    pool: &PgPool,
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
    lower: OffsetDateTime,
    upper: OffsetDateTime,
) {
    assert!(catalog_allowlist_preserved(
        &before["ont_builtin_catalog_allowlist"],
        &after["ont_builtin_catalog_allowlist"]
    ));
    let created: OffsetDateTime = sqlx::query_scalar(
        "SELECT created_at FROM public.ont_builtin_catalog_allowlist WHERE catalog_version=$1",
    )
    .bind(NATIVE_VERSION)
    .fetch_one(pool)
    .await
    .unwrap();
    assert!(
        lower <= created && created <= upper,
        "native allowlist timestamp must belong to actual migration interval"
    );
    for table in [
        "ont_object_types",
        "ont_builtin_catalog_installs",
        "cedar_policy_catalog_entries",
        "ont_object_policies",
    ] {
        let mut omitted = after.clone();
        omitted.insert(table.into(), "[]".into());
        assert!(
            !migration::legacy_values_preserved(before, &omitted),
            "preservation accepted omitted populated {table}"
        );
        for (key, bad) in [
            ("attribution_protocol", json!("NATIVE_ACCOUNT")),
            ("origin_account_id", json!(Uuid::new_v4())),
        ] {
            let mut corrupt = after.clone();
            let mut values: Vec<Value> = serde_json::from_str(&corrupt[table]).unwrap();
            assert!(!values.is_empty());
            values[0][key] = bad;
            corrupt.insert(table.into(), serde_json::to_string(&values).unwrap());
            assert!(
                !migration::legacy_values_preserved(before, &corrupt),
                "preservation accepted {table}.{key}"
            );
        }
    }
}

pub(super) async fn legacy_catalog_replay_after_extension(pool: &PgPool, org: Uuid, user: Uuid) {
    use console_ontology_adapter_postgres::seed::{
        BUILTIN_CATALOG_VERSION, builtin_catalog_manifest,
    };
    let before = all_rows(pool).await;
    let command = login_test_pool(pool, TestDatabaseLogin::OntologyCommand).await;
    let mut tx = command.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org',$1,true)")
        .bind(org.to_string())
        .execute(tx.as_mut())
        .await
        .unwrap();
    let manifest = builtin_catalog_manifest().unwrap();
    let result:(bool,i64)=sqlx::query_as("SELECT installed,object_type_count FROM ontology_api.install_builtin_catalog($1,$2,$3,$4,$5,$6)")
        .bind(org).bind(BUILTIN_CATALOG_VERSION).bind(&manifest).bind(user)
        .bind("00112233445566778899aabbccddeeff").bind("0011223344556677").fetch_one(tx.as_mut()).await.unwrap();
    assert_eq!(
        result,
        (
            false,
            manifest["object_types"].as_array().unwrap().len() as i64
        )
    );
    tx.commit().await.unwrap();
    assert_eq!(
        before,
        all_rows(pool).await,
        "legacy catalog replay after attribution extension changed history"
    );
    command.close().await;
    // The historical NOT NULL becomes this exact tagged CHECK, not nullable
    // legacy attribution. Operator corruption probes the actual constraint.
    let mut tx = pool.begin().await.unwrap();
    let error = sqlx::query(
        "INSERT INTO public.ont_builtin_catalog_installs(org_id,catalog_version,manifest_digest,installed_by,installed_at) SELECT org_id,catalog_version,manifest_digest,NULL,installed_at FROM public.ont_builtin_catalog_installs WHERE org_id=$1",
    )
    .bind(org)
    .execute(tx.as_mut())
    .await
    .expect_err("legacy installer attribution remains required");
    let database = error.as_database_error().unwrap();
    assert_eq!(database.code().as_deref(), Some("23514"));
    assert_eq!(
        database.constraint(),
        Some("ont_builtin_catalog_installs_attribution_shape")
    );
    tx.rollback().await.unwrap();
    assert_eq!(before, all_rows(pool).await);
}

#[test]
fn catalog_allowlist_oracle_detects_missing_extra_altered_history_and_invalid_native_row() {
    let old = r#"[{"catalog_version":"legacy","manifest_digest":"old","created_at":"2020-01-01T00:00:00Z","historical_number":18446744073709551616}]"#;
    let native = format!(
        r#"{{"catalog_version":"{NATIVE_VERSION}","manifest_digest":"\\x0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935","created_at":"2026-09-20T00:00:00Z"}}"#
    );
    let after = format!("{},{}]", old.trim_end_matches(']'), native);
    assert!(catalog_allowlist_preserved(old, &after));
    for bad in [
        old.to_owned(),
        after.replace("18446744073709551616", "18446744073709551617"),
        after.replace("2020-01-01", "2020-01-02"),
        after.replace("0d3d0c3b", "1d3d0c3b"),
        after.replace("2026-09-20T00:00:00Z", "infinity"),
        after.replace("2026-09-20T00:00:00Z", "bad-time"),
        format!("{},{}]", after.trim_end_matches(']'), native),
        format!(
            "{},{{\"catalog_version\":\"extra\"}}]",
            after.trim_end_matches(']')
        ),
    ] {
        assert_ne!(bad, after);
        assert!(
            !catalog_allowlist_preserved(old, &bad),
            "allowlist oracle accepted corrupted rows"
        );
    }
    let extra = after.replace(
        "2026-09-20T00:00:00Z\"}",
        "2026-09-20T00:00:00Z\",\"extra\":null}",
    );
    assert_ne!(extra, after);
    assert!(!catalog_allowlist_preserved(old, &extra));
}

pub(super) async fn legacy_catalog_shape(pool: &PgPool) -> Value {
    sqlx::query_scalar(r#"
      SELECT jsonb_agg(jsonb_build_object('table',c.relname,'owner',pg_get_userbyid(c.relowner),'rls',c.relrowsecurity,'force_rls',c.relforcerowsecurity,
        'columns',(SELECT jsonb_agg(jsonb_build_object('name',a.attname,'type',format_type(a.atttypid,a.atttypmod),'not_null',a.attnotnull,'default',pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
          FROM pg_attribute a LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum WHERE a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped),
        'constraints',(SELECT jsonb_agg(jsonb_build_object('name',k.conname,'kind',k.contype,'definition',pg_get_constraintdef(k.oid,true),'validated',k.convalidated,'deferrable',k.condeferrable,'deferred',k.condeferred) ORDER BY k.conname)
          FROM pg_constraint k WHERE k.conrelid=c.oid)) ORDER BY c.relname)
      FROM pg_class c WHERE c.relnamespace='public'::regnamespace AND c.relname=ANY(ARRAY['ont_object_types','ont_builtin_catalog_installs','cedar_policy_catalog_entries','ont_object_policies'])
    "#).fetch_one(pool).await.unwrap()
}

pub(super) fn legacy_catalog_shape_preserved(before: &Value, after: &Value) -> bool {
    let check = || -> Option<()> {
        let before = before.as_array()?;
        let after = after.as_array()?;
        if before.len() != 4 || after.len() != 4 {
            return None;
        }
        for old in before {
            let table = old["table"].as_str()?;
            let new = after.iter().find(|r| r["table"] == table)?;
            for key in ["owner", "rls", "force_rls"] {
                if old[key] != new[key] {
                    return None;
                }
            }
            for column in old["columns"].as_array()? {
                let name = column["name"].as_str()?;
                let actual = new["columns"]
                    .as_array()?
                    .iter()
                    .find(|c| c["name"] == name)?;
                let mut expected = column.clone();
                if table == "ont_builtin_catalog_installs" && name == "installed_by" {
                    expected["not_null"] = json!(false);
                }
                if actual != &expected {
                    return None;
                }
            }
            for constraint in old["constraints"].as_array()? {
                if table == "ont_builtin_catalog_installs"
                    && constraint["kind"] == "n"
                    && constraint["definition"] == "NOT NULL installed_by"
                {
                    continue;
                }
                if !new["constraints"].as_array()?.contains(constraint) {
                    return None;
                }
            }
        }
        Some(())
    };
    check().is_some()
}

#[test]
fn catalog_legacy_projection_is_finite_and_lossless_for_each_extended_table() {
    for table in [
        "ont_object_types",
        "ont_builtin_catalog_installs",
        "cedar_policy_catalog_entries",
        "ont_object_policies",
    ] {
        for number in ["18446744073709551616", "0.123456789012345678901"] {
            let old = format!("[{{\"old_exact_number\":{number}}}]");
            let attribution = if table == "ont_builtin_catalog_installs" {
                "installed_by_account_id"
            } else {
                "created_by_account_id"
            };
            let mut extras = format!(
                r#", "attribution_protocol":"LEGACY_USER","{attribution}":null,"origin_account_id":null,"origin_command_id":null,"origin_receipt_id":null"#
            );
            if table == "cedar_policy_catalog_entries" {
                extras.push_str(",\"updated_by_account_id\":null");
            }
            let added = old.replace("}]", &format!("{extras}}}]"));
            let before = BTreeMap::from([(table.to_owned(), old.clone())]);
            let after = BTreeMap::from([(table.to_owned(), added.clone())]);
            assert!(migration::legacy_values_preserved(&before, &after));
            for changed in [
                added.replace(number, &format!("{number}1")),
                added.replace("LEGACY_USER", "NATIVE_ACCOUNT"),
                added.replace(
                    &format!("\"{attribution}\":null"),
                    &format!("\"{attribution}\":\"{}\"", Uuid::new_v4()),
                ),
                added.replace("}]", ",\"unreviewed_column\":null}]"),
                old.clone(),
            ] {
                assert_ne!(changed, added);
                let corrupt = BTreeMap::from([(table.to_owned(), changed)]);
                assert!(
                    !migration::legacy_values_preserved(&before, &corrupt),
                    "finite projection accepted changed bytes/columns for {table}"
                );
            }
        }
    }
}

pub(super) async fn legacy_catalog_attachment_after_extension(
    pool: &PgPool,
    org: Uuid,
    user: Uuid,
) {
    let command = login_test_pool(pool, TestDatabaseLogin::OntologyCommand).await;
    let before = all_rows(pool).await;
    let mut tx = command.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org',$1,true)")
        .bind(org.to_string())
        .execute(tx.as_mut())
        .await
        .unwrap();
    let object:Uuid=sqlx::query_scalar("SELECT id FROM public.ont_object_types WHERE org_id=$1 AND stable_key='customer' AND lifecycle_state='published'")
        .bind(org).fetch_one(tx.as_mut()).await.unwrap();
    let row = json!({"effect":"forbid","action":"view","resource_type":"customer","conditions":[]});
    let policy: Uuid =
        sqlx::query_scalar("SELECT ont_policy_api.attach_object_policy($1,$2,$3,$4,$5,$6,$7,$8)")
            .bind(org)
            .bind(user)
            .bind(object)
            .bind("forbid")
            .bind(&row)
            .bind("ontology-runtime-filter-v1")
            .bind("00112233445566778899aabbccddeeff")
            .bind("0011223344556677")
            .fetch_one(tx.as_mut())
            .await
            .unwrap();
    assert!(!policy.is_nil());
    tx.commit().await.unwrap();
    let after = all_rows(pool).await;
    for table in ["cedar_policy_catalog_entries", "ont_object_policies"] {
        let added = added_rows(&before[table], &after[table]).unwrap();
        assert_eq!(added.len(), 1);
        assert_eq!(added[0]["attribution_protocol"], "LEGACY_USER");
        assert_eq!(added[0]["created_by"], json!(user));
        for field in [
            "created_by_account_id",
            "origin_account_id",
            "origin_command_id",
            "origin_receipt_id",
        ] {
            assert_eq!(added[0].get(field), Some(&Value::Null));
        }
    }
    let audits = added_rows(&before["audit_events"], &after["audit_events"]).unwrap();
    assert_eq!(audits.len(), 1);
    assert_eq!(audits[0]["action"], "ontology.object_policy.attach");
    let mut tx = command.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org',$1,true)")
        .bind(org.to_string())
        .execute(tx.as_mut())
        .await
        .unwrap();
    let failed = sqlx::query_scalar::<_, Uuid>(
        "SELECT ont_policy_api.attach_object_policy($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(Uuid::new_v4())
    .bind(user)
    .bind(object)
    .bind("forbid")
    .bind(&row)
    .bind("ontology-runtime-filter-v1")
    .bind("00112233445566778899aabbccddeeff")
    .bind("0011223344556677")
    .fetch_one(tx.as_mut())
    .await
    .expect_err("retained legacy cross-Company floor must refuse actual catalog INSERT");
    let error = failed.as_database_error().unwrap();
    assert_eq!(error.code().as_deref(), Some("42501"));
    assert!(
        error.message().contains("row-level security policy")
            && error.message().contains("cedar_policy_catalog_entries")
    );
    tx.rollback().await.unwrap();
    assert_eq!(after, all_rows(pool).await);
    command.close().await;
}
