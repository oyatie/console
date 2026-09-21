use super::*;

// Include in company_setup with reviewed native_identity_catalog.json alongside.
// catalog.json from this packet is copied byte-for-byte to that include name.
// The candidate must be fully installed by genuine finalized fixture preparation.
pub(super) fn identity_catalog_contract() -> Value {
    serde_json::from_str(include_str!("native_identity_catalog.json")).unwrap()
}

pub(super) async fn identity_catalog_observation(pool: &PgPool) -> Value {
    let contract = identity_catalog_contract();
    let mut tables = Vec::new();
    let mut triggers = Vec::new();
    let mut routines = Vec::new();
    for table in contract["new_identity_tables"]
        .as_array()
        .unwrap()
        .iter()
        .chain(contract["new_ontology_mapping_tables"].as_array().unwrap())
    {
        let table = table.as_str().unwrap();
        let row:Value=sqlx::query_scalar(r#"
          SELECT jsonb_build_object('name',c.relname,'owner',pg_get_userbyid(c.relowner),
            'rls',c.relrowsecurity,'force_rls',c.relforcerowsecurity,
            'policies',(SELECT coalesce(jsonb_agg(jsonb_build_object('name',p.polname,
                'command',p.polcmd,'permissive',p.polpermissive,'roles',ARRAY(SELECT role_id::bigint FROM unnest(p.polroles) role_id),
                'using',pg_get_expr(p.polqual,p.polrelid),'check',pg_get_expr(p.polwithcheck,p.polrelid)) ORDER BY p.polname),'[]'::jsonb)
              FROM pg_policy p WHERE p.polrelid=c.oid),
            'table_acl',(SELECT coalesce(jsonb_agg(jsonb_build_object('role',CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
              'privilege',a.privilege_type,'grantable',a.is_grantable) ORDER BY a.grantee,a.privilege_type),'[]'::jsonb)
              FROM aclexplode(coalesce(c.relacl,acldefault('r',c.relowner))) a),
            'column_acl',(SELECT coalesce(jsonb_agg(jsonb_build_object('column',p.attname,
              'role',CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
              'privilege',a.privilege_type,'grantable',a.is_grantable) ORDER BY p.attnum,a.grantee,a.privilege_type),'[]'::jsonb)
              FROM pg_attribute p CROSS JOIN LATERAL aclexplode(p.attacl) a
              WHERE p.attrelid=c.oid AND p.attnum>0 AND NOT p.attisdropped),
            'session_family_fks',(SELECT count(*) FROM pg_constraint k
              WHERE k.conrelid=c.oid AND k.contype='f' AND k.confrelid='public.auth_refresh_token_families'::regclass))
          FROM pg_class c WHERE c.oid=to_regclass('public.'||$1)
        "#).bind(table).fetch_optional(pool).await.unwrap()
            .unwrap_or_else(||panic!("identity catalog prerequisite absent: {table}"));
        tables.push(row);
    }
    for t in contract["triggers"].as_array().unwrap() {
        let row:Value=sqlx::query_scalar(r#"
          SELECT jsonb_build_object('name',t.tgname,'table',c.relname,'function',p.proname,
            'type',t.tgtype::integer,'enabled',t.tgenabled,'deferrable',t.tgdeferrable,
            'initially_deferred',t.tginitdeferred,'when',pg_get_expr(t.tgqual,t.tgrelid),
            'argument_count',t.tgnargs,'argument_bytes',octet_length(t.tgargs),'internal',t.tgisinternal,
            'function_schema',(SELECT nspname FROM pg_namespace WHERE oid=p.pronamespace),
            'exact_function',t.tgfoid=to_regprocedure('public.'||$3||'()'),
            'update_columns',ARRAY(SELECT a.attname::text FROM unnest(t.tgattr::smallint[]) x(id) JOIN pg_attribute a ON a.attrelid=t.tgrelid AND a.attnum=x.id ORDER BY a.attnum),
            'old_transition_table',t.tgoldtable,'new_transition_table',t.tgnewtable,'parent_trigger',t.tgparentid<>0)
          FROM pg_trigger t JOIN pg_class c ON c.oid=t.tgrelid JOIN pg_proc p ON p.oid=t.tgfoid
          WHERE c.oid=to_regclass('public.'||$1) AND t.tgname=$2
        "#).bind(t["table"].as_str().unwrap()).bind(t["name"].as_str().unwrap()).bind(t["function"].as_str().unwrap())
            .fetch_optional(pool).await.unwrap().unwrap_or(Value::Null);
        triggers.push(row);
    }
    let mut signatures: Vec<String> = contract["trigger_function_definer"]
        .as_object()
        .unwrap()
        .keys()
        .map(|name| format!("public.{name}()"))
        .collect();
    signatures.extend(
        contract["additional_routines"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["signature"].as_str().unwrap().to_owned()),
    );
    for signature in signatures {
        let row:Value=sqlx::query_scalar(r#"
          SELECT jsonb_build_object('signature',$1::text,'owner',pg_get_userbyid(p.proowner),
            'definer',p.prosecdef,'config',p.proconfig,'kind',p.prokind,
            'acl',(SELECT coalesce(jsonb_agg(jsonb_build_object('role',CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
                'privilege',a.privilege_type,'grantable',a.is_grantable) ORDER BY a.grantee,a.privilege_type),'[]'::jsonb)
              FROM aclexplode(coalesce(p.proacl,acldefault('f',p.proowner))) a))
          FROM pg_proc p WHERE p.oid=to_regprocedure($1)
        "#).bind(&signature).fetch_optional(pool).await.unwrap().unwrap_or(Value::Null);
        routines.push(row);
    }
    // Find extras which invoke any of the sealed functions on protected tables.
    let names: Vec<String> = contract["trigger_function_definer"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid WHERE p.pronamespace='public'::regnamespace AND p.proname=ANY($1)")
        .bind(names).fetch_one(pool).await.unwrap();
    let mut foreign_keys = Vec::new();
    for expected in contract["new_foreign_keys"].as_array().unwrap() {
        let columns: Vec<String> = expected["columns"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect();
        let rows:Vec<Value>=sqlx::query_scalar(r#"
          SELECT jsonb_build_object('table',c.relname,
            'columns',(SELECT jsonb_agg(a.attname ORDER BY q.n) FROM unnest(k.conkey) WITH ORDINALITY q(id,n) JOIN pg_attribute a ON a.attrelid=k.conrelid AND a.attnum=q.id),
            'target',r.relname,'target_schema',(SELECT nspname FROM pg_namespace WHERE oid=r.relnamespace),'match',k.confmatchtype,
            'target_columns',(SELECT jsonb_agg(a.attname ORDER BY q.n) FROM unnest(k.confkey) WITH ORDINALITY q(id,n) JOIN pg_attribute a ON a.attrelid=k.confrelid AND a.attnum=q.id),
            'update',k.confupdtype,'delete',k.confdeltype,'deferrable',k.condeferrable,'initially_deferred',k.condeferred,'validated',k.convalidated)
          FROM pg_constraint k JOIN pg_class c ON c.oid=k.conrelid JOIN pg_class r ON r.oid=k.confrelid
          WHERE k.contype='f' AND c.oid=to_regclass('public.'||$1)
            AND ARRAY(SELECT a.attname::text FROM unnest(k.conkey) WITH ORDINALITY q(id,n) JOIN pg_attribute a ON a.attrelid=k.conrelid AND a.attnum=q.id ORDER BY q.n)=$2::text[]
        "#).bind(expected["table"].as_str().unwrap()).bind(columns).fetch_all(pool).await.unwrap();
        foreign_keys.push(if rows.len() == 1 {
            rows[0].clone()
        } else {
            Value::Null
        });
    }
    json!({"tables":tables,"triggers":triggers,"routines":routines,"trigger_count":count,"foreign_keys":foreign_keys})
}

fn identity_acl_matches(actual: &Value, expected: &Value) -> bool {
    let Some(rows) = actual.as_array() else {
        return false;
    };
    let Some(expected) = expected.as_object() else {
        return false;
    };
    let mut set = BTreeSet::new();
    for row in rows {
        let (Some(role), Some(privilege)) = (row["role"].as_str(), row["privilege"].as_str())
        else {
            return false;
        };
        if row["grantable"] != false || !set.insert((role.to_owned(), privilege.to_owned())) {
            return false;
        }
    }
    let wanted: BTreeSet<_> = expected
        .iter()
        .flat_map(|(role, privileges)| {
            privileges
                .as_array()
                .unwrap()
                .iter()
                .map(move |p| (role.to_owned(), p.as_str().unwrap().to_owned()))
        })
        .collect();
    set == wanted
}

pub(super) fn identity_catalog_matches(observed: &Value) -> bool {
    let contract = identity_catalog_contract();
    let check = || -> Option<()> {
        if observed["foreign_keys"] != contract["new_foreign_keys"] {
            return None;
        }
        let tables = observed["tables"].as_array()?;
        let wanted: BTreeSet<_> = contract["new_identity_tables"]
            .as_array()?
            .iter()
            .chain(contract["new_ontology_mapping_tables"].as_array()?)
            .map(|v| v.as_str())
            .collect();
        if tables.len() != 9
            || tables
                .iter()
                .map(|v| v["name"].as_str())
                .collect::<BTreeSet<_>>()
                != wanted
            || observed["trigger_count"] != 37
        {
            return None;
        }
        for table in tables {
            let name = table["name"].as_str()?;
            if table["owner"] != contract["table_owners"][name]
                || table["rls"] != true
                || table["force_rls"] != true
                || table["session_family_fks"] != 0
                || !identity_acl_matches(&table["table_acl"], &contract["new_table_acl"][name])
            {
                return None;
            }
            let policies = table["policies"].as_array()?;
            if policies.len() != 1
                || policies[0]["name"] != "org_isolation"
                || policies[0]["command"] != "*"
                || policies[0]["permissive"] != true
                || policies[0]["roles"] != json!([0])
            {
                return None;
            }
            // Exact PostgreSQL18 deparse is independently fixed, not compared to
            // the implementation's own SQL string or substituted with substrings.
            let predicate = "(org_id = (NULLIF(current_setting('app.current_org'::text, true), ''::text))::uuid)";
            if policies[0]["using"] != predicate || policies[0]["check"] != predicate {
                return None;
            }
            let columns = table["column_acl"].as_array()?;
            if name == "company_authority_heads" {
                if columns.len() != 1
                    || columns[0]
                        != json!({"column":"epoch","role":"console_account_owner","privilege":"UPDATE","grantable":false})
                {
                    return None;
                }
            } else if !columns.is_empty() {
                return None;
            }
        }
        let ts = observed["triggers"].as_array()?;
        if ts.len() != 37 {
            return None;
        }
        for (actual, expected) in ts.iter().zip(contract["triggers"].as_array()?) {
            let mut kind = if expected["level"] == "ROW" { 1 } else { 0 };
            if expected["timing"] == "BEFORE" {
                kind |= 2;
            }
            for event in expected["events"].as_array()? {
                kind |= match event.as_str()? {
                    "INSERT" => 4,
                    "DELETE" => 8,
                    "UPDATE" => 16,
                    "TRUNCATE" => 32,
                    _ => return None,
                };
            }
            if actual["name"] != expected["name"]
                || actual["table"] != expected["table"]
                || actual["function"] != expected["function"]
                || actual["type"] != kind
                || actual["enabled"] != "A"
                || actual["deferrable"] != expected["deferrable"]
                || actual["initially_deferred"]
                    != expected["initially_deferred"].as_bool().unwrap_or(false)
                || !actual["when"].is_null()
                || actual["argument_count"] != 0
                || actual["argument_bytes"] != 0
                || actual["internal"] != false
                || actual["function_schema"] != "public"
                || actual["exact_function"] != true
                || actual["update_columns"] != json!([])
                || !actual["old_transition_table"].is_null()
                || !actual["new_transition_table"].is_null()
                || actual["parent_trigger"] != false
            {
                return None;
            }
        }
        let routine_rows = observed["routines"].as_array()?;
        let mut wanted: BTreeSet<String> = contract["trigger_function_definer"]
            .as_object()?
            .keys()
            .map(|n| format!("public.{n}()"))
            .collect();
        wanted.extend(
            contract["additional_routines"]
                .as_array()?
                .iter()
                .map(|r| r["signature"].as_str().unwrap().to_owned()),
        );
        if routine_rows.len() != wanted.len()
            || routine_rows
                .iter()
                .filter_map(|r| r["signature"].as_str().map(str::to_owned))
                .collect::<BTreeSet<_>>()
                != wanted
        {
            return None;
        }
        for routine in routine_rows {
            let sig = routine["signature"].as_str()?;
            let name = sig.strip_prefix("public.")?.split('(').next()?;
            let (owner, definer, execute) =
                if let Some(definer) = contract["trigger_function_definer"][name].as_bool() {
                    (
                        &contract["function_owners"][name],
                        definer,
                        &contract["trigger_function_execute_acl"][name],
                    )
                } else {
                    let row = contract["additional_routines"]
                        .as_array()?
                        .iter()
                        .find(|r| r["signature"] == sig)?;
                    (
                        &row["owner"],
                        row["security_definer"].as_bool()?,
                        &row["execute"],
                    )
                };
            let acl: serde_json::Map<String, Value> = execute
                .as_array()?
                .iter()
                .map(|role| (role.as_str().unwrap().to_owned(), json!(["EXECUTE"])))
                .collect();
            if routine["owner"] != *owner
                || routine["definer"] != definer
                || routine["kind"] != "f"
                || routine["config"] != contract["function_config"]
                || !identity_acl_matches(&routine["acl"], &Value::Object(acl))
            {
                return None;
            }
        }
        Some(())
    };
    check().is_some()
}

#[sqlx::test(migrations = false)]
async fn identity_catalog_exact_guards_acl_rls_and_corruption_controls(pool: PgPool) {
    let (_app, _account, _cookies, startup, _) = designated(&pool).await;
    assert_native_topology_metadata(&pool).await;
    let actual = identity_catalog_observation(&pool).await;
    assert!(
        identity_catalog_matches(&actual),
        "exact installed identity catalog differs"
    );
    for (section, index, key, bad) in [
        ("foreign_keys", 0, "target_schema", json!("other")),
        ("foreign_keys", 0, "match", json!("f")),
        ("foreign_keys", 0, "target", json!("users")),
        ("foreign_keys", 0, "validated", json!(false)),
        ("foreign_keys", 0, "columns", json!(["org_id"])),
        ("foreign_keys", 0, "deferrable", json!(false)),
        ("tables", 0, "owner", json!("console_rt")),
        ("tables", 0, "rls", json!(false)),
        ("tables", 0, "force_rls", json!(false)),
        ("tables", 0, "session_family_fks", json!(1)),
        ("triggers", 0, "enabled", json!("D")),
        ("triggers", 0, "when", json!("true")),
        ("triggers", 0, "function_schema", json!("other")),
        ("triggers", 0, "exact_function", json!(false)),
        (
            "triggers",
            0,
            "update_columns",
            json!(["native_current_revision"]),
        ),
        ("triggers", 0, "old_transition_table", json!("old_rows")),
        ("triggers", 0, "parent_trigger", json!(true)),
        ("triggers", 0, "type", json!(4)),
        ("triggers", 0, "function", json!("wrong_guard")),
        ("triggers", 1, "initially_deferred", json!(false)),
        ("triggers", 0, "argument_count", json!(1)),
        ("routines", 0, "owner", json!("console_rt")),
    ] {
        let mut corrupt = actual.clone();
        corrupt[section][index][key] = bad;
        assert!(
            corrupt != actual,
            "corruption control did not change observation"
        );
        assert!(
            !identity_catalog_matches(&corrupt),
            "metadata oracle accepted {section}.{key}"
        );
    }
    let mut corrupt = actual.clone();
    corrupt["tables"][0]["table_acl"]
        .as_array_mut()
        .unwrap()
        .push(json!({"role":"PUBLIC","privilege":"SELECT","grantable":false}));
    assert!(
        !identity_catalog_matches(&corrupt),
        "metadata oracle accepted public data access"
    );
    let mut corrupt = actual.clone();
    corrupt["routines"][0]["definer"] = json!(!actual["routines"][0]["definer"].as_bool().unwrap());
    assert!(
        !identity_catalog_matches(&corrupt),
        "metadata oracle accepted definer toggle"
    );
    let mut corrupt = actual.clone();
    corrupt["triggers"].as_array_mut().unwrap().pop();
    assert!(
        !identity_catalog_matches(&corrupt),
        "metadata oracle accepted omitted trigger"
    );
    let before = all_rows(&pool).await;
    for login in [
        TestDatabaseLogin::Business,
        TestDatabaseLogin::Auth,
        TestDatabaseLogin::LeaveCommand,
        TestDatabaseLogin::OntologyCommand,
        TestDatabaseLogin::PlatformForceCommand,
    ] {
        let serving = login_test_pool(&pool, login).await;
        for table in identity_catalog_contract()["table_owners"]
            .as_object()
            .unwrap()
            .keys()
            .filter(|t| *t != "policy_roles" && *t != "user_role_assignments")
        {
            let escaped = format!("public.{table}");
            let denied:bool=sqlx::query_scalar("SELECT NOT (has_table_privilege(current_user,$1,'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN') OR has_any_column_privilege(current_user,$1,'SELECT,INSERT,UPDATE,REFERENCES'))")
                .bind(escaped).fetch_one(&serving).await.unwrap();
            assert!(
                denied,
                "serving role has direct or inherited native table/column privilege"
            );
        }
        for owner in [
            "console_account_owner",
            "console_ontology_writer",
            "console_app",
        ] {
            let inherited: bool =
                sqlx::query_scalar("SELECT pg_has_role(current_user,$1,'MEMBER')")
                    .bind(owner)
                    .fetch_one(&serving)
                    .await
                    .unwrap();
            assert!(!inherited, "serving role can assume protected owner");
        }
        serving.close().await;
    }
    assert!(before == all_rows(&pool).await);
    startup.close().await;
}
