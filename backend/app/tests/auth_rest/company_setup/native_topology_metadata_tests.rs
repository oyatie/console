use super::*;

fn contract() -> Value {
    serde_json::from_str(include_str!("native_topology_catalog.json")).unwrap()
}

async fn observation(pool: &PgPool) -> Value {
    let expected = contract();
    let mut tables = Vec::new();
    for name in expected["new_topology_tables"].as_object().unwrap().keys() {
        let row: Value = sqlx::query_scalar(r#"
          SELECT jsonb_build_object('name',c.relname,'owner',pg_get_userbyid(c.relowner),
            'rls',c.relrowsecurity,'force_rls',c.relforcerowsecurity,
            'policies',(SELECT count(*) FROM pg_policy WHERE polrelid=c.oid),
            'table_acl',(SELECT coalesce(jsonb_agg(jsonb_build_object('role',CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
              'privilege',a.privilege_type,'grantable',a.is_grantable)),'[]'::jsonb)
              FROM aclexplode(coalesce(c.relacl,acldefault('r',c.relowner))) a),
            'column_acl',(SELECT coalesce(jsonb_agg(jsonb_build_object('column',p.attname,
              'role',CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
              'privilege',a.privilege_type,'grantable',a.is_grantable)),'[]'::jsonb)
              FROM pg_attribute p CROSS JOIN LATERAL aclexplode(p.attacl) a
              WHERE p.attrelid=c.oid AND p.attnum>0 AND NOT p.attisdropped),
            'foreign_targets',(SELECT coalesce(jsonb_agg(n.nspname||'.'||r.relname ORDER BY n.nspname,r.relname),'[]'::jsonb)
              FROM pg_constraint k JOIN pg_class r ON r.oid=k.confrelid JOIN pg_namespace n ON n.oid=r.relnamespace
              WHERE k.conrelid=c.oid AND k.contype='f'))
          FROM pg_class c WHERE c.oid=to_regclass('public.'||$1)
        "#).bind(name).fetch_optional(pool).await.unwrap().unwrap_or(Value::Null);
        tables.push(row);
    }
    let mut routines = Vec::new();
    for signature in expected["new_guard_functions"].as_object().unwrap().keys() {
        let row: Value = sqlx::query_scalar(r#"
          SELECT jsonb_build_object('signature',$1::text,'owner',pg_get_userbyid(p.proowner),
            'returns',pg_get_function_result(p.oid),'security_definer',p.prosecdef,
            'config',p.proconfig,'kind',p.prokind,'volatility',p.provolatile,'parallel',p.proparallel,
            'acl',(SELECT coalesce(jsonb_agg(jsonb_build_object('role',CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
              'privilege',a.privilege_type,'grantable',a.is_grantable)),'[]'::jsonb)
              FROM aclexplode(coalesce(p.proacl,acldefault('f',p.proowner))) a))
          FROM pg_proc p WHERE p.oid=to_regprocedure($1)
        "#).bind(signature).fetch_optional(pool).await.unwrap().unwrap_or(Value::Null);
        routines.push(row);
    }
    let mut triggers = Vec::new();
    for t in expected["triggers"].as_array().unwrap() {
        let row: Value = sqlx::query_scalar(r#"
          SELECT jsonb_build_object('name',t.tgname,'table',$1::text,'function',$3::text,
            'exact_function',t.tgfoid=to_regprocedure($3),'type',t.tgtype::integer,
            'enabled',t.tgenabled,'deferrable',t.tgdeferrable,'initially_deferred',t.tginitdeferred,
            'when',pg_get_expr(t.tgqual,t.tgrelid),'internal',t.tgisinternal,
            'argument_count',t.tgnargs,'argument_bytes',octet_length(t.tgargs),
            'update_columns',ARRAY(SELECT a.attname::text FROM unnest(t.tgattr::smallint[]) x(id)
              JOIN pg_attribute a ON a.attrelid=t.tgrelid AND a.attnum=x.id ORDER BY a.attnum),
            'old_transition_table',t.tgoldtable,'new_transition_table',t.tgnewtable,'parent_trigger',t.tgparentid<>0)
          FROM pg_trigger t WHERE t.tgrelid=to_regclass($1) AND t.tgname=$2
        "#).bind(t["table"].as_str().unwrap()).bind(t["name"].as_str().unwrap())
          .bind(t["function"].as_str().unwrap()).fetch_optional(pool).await.unwrap().unwrap_or(Value::Null);
        triggers.push(row);
    }
    let signatures: Vec<String> = expected["new_guard_functions"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    // Count every use of these actual functions, including extra triggers on unrelated tables.
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_trigger t WHERE t.tgfoid IN (SELECT to_regprocedure(s) FROM unnest($1::text[]) s)")
        .bind(signatures).fetch_one(pool).await.unwrap();
    json!({"tables":tables,"routines":routines,"triggers":triggers,"trigger_count":count})
}

fn acl(actual: &Value, wanted: &Value, columns: bool) -> bool {
    let check = || -> Option<()> {
        let mut got = BTreeSet::new();
        for row in actual.as_array()? {
            if row["grantable"] != false {
                return None;
            }
            let column = if columns { row["column"].as_str()? } else { "" };
            if !got.insert((
                row["role"].as_str()?.to_owned(),
                row["privilege"].as_str()?.to_owned(),
                column.to_owned(),
            )) {
                return None;
            }
        }
        let mut expected = BTreeSet::new();
        for (role, permissions) in wanted.as_object()? {
            if columns {
                for (permission, names) in permissions.as_object()? {
                    for name in names.as_array()? {
                        expected.insert((
                            role.clone(),
                            permission.clone(),
                            name.as_str()?.to_owned(),
                        ));
                    }
                }
            } else {
                for permission in permissions.as_array()? {
                    expected.insert((role.clone(), permission.as_str()?.to_owned(), String::new()));
                }
            }
        }
        (got == expected).then_some(())
    };
    check().is_some()
}

fn matches(actual: &Value) -> bool {
    let expected = contract();
    let check = || -> Option<()> {
        let tables = actual["tables"].as_array()?;
        let table_contract = expected["new_topology_tables"].as_object()?;
        if tables.len() != 3
            || tables
                .iter()
                .filter_map(|t| t["name"].as_str())
                .collect::<BTreeSet<_>>()
                != table_contract.keys().map(String::as_str).collect()
        {
            return None;
        }
        for table in tables {
            let name = table["name"].as_str()?;
            let want = &table_contract[name];
            for key in ["owner", "rls", "force_rls"] {
                if table[key] != want[key] {
                    return None;
                }
            }
            if table["policies"] != 0
                || !acl(&table["table_acl"], &want["table_acl"], false)
                || !acl(&table["column_acl"], &want["column_acl"], true)
            {
                return None;
            }
            // Historical topology IDs must never acquire live-root/family FKs.
            let targets = match name {
                "company_enrollment_effect_bindings" => json!([
                    "public.accounts",
                    "public.company_enrollment_receipts",
                    "public.company_enrollment_requests",
                    "public.groups",
                    "public.organizations"
                ]),
                "group_authority_heads" => json!(["public.groups"]),
                "group_membership_revisions" => json!([
                    "public.company_enrollment_receipts",
                    "public.platform_legacy_topology_receipts"
                ]),
                _ => return None,
            };
            if table["foreign_targets"] != targets {
                return None;
            }
        }
        let routines = actual["routines"].as_array()?;
        let wanted = expected["new_guard_functions"].as_object()?;
        if routines.len() != 5
            || routines
                .iter()
                .filter_map(|r| r["signature"].as_str())
                .collect::<BTreeSet<_>>()
                != wanted.keys().map(String::as_str).collect()
        {
            return None;
        }
        for routine in routines {
            let want = &wanted[routine["signature"].as_str()?];
            for key in [
                "owner",
                "returns",
                "security_definer",
                "volatility",
                "parallel",
            ] {
                if routine[key] != want[key] {
                    return None;
                }
            }
            let grants: serde_json::Map<String, Value> = want["execute"]
                .as_array()?
                .iter()
                .map(|r| (r.as_str().unwrap().to_owned(), json!(["EXECUTE"])))
                .collect();
            if routine["kind"] != "f"
                || routine["config"]
                    != json!(["search_path=pg_catalog, pg_temp", "row_security=on"])
                || !acl(&routine["acl"], &Value::Object(grants), false)
            {
                return None;
            }
        }
        let triggers = actual["triggers"].as_array()?;
        if triggers.len() != 18 || actual["trigger_count"] != 18 {
            return None;
        }
        for (got, want) in triggers.iter().zip(expected["triggers"].as_array()?) {
            let mut kind = if want["level"] == "ROW" { 1 } else { 0 };
            if want["timing"] == "BEFORE" {
                kind |= 2;
            }
            for event in want["events"].as_array()? {
                kind |= match event.as_str()? {
                    "INSERT" => 4,
                    "DELETE" => 8,
                    "UPDATE" => 16,
                    "TRUNCATE" => 32,
                    _ => return None,
                };
            }
            for key in [
                "name",
                "table",
                "function",
                "deferrable",
                "initially_deferred",
            ] {
                if got[key] != want[key] {
                    return None;
                }
            }
            if got["exact_function"] != true
                || got["type"] != kind
                || got["enabled"] != "A"
                || got["internal"] != false
                || !got["when"].is_null()
                || got["argument_count"] != 0
                || got["argument_bytes"] != 0
                || got["update_columns"] != json!([])
                || !got["old_transition_table"].is_null()
                || !got["new_transition_table"].is_null()
                || got["parent_trigger"] != false
            {
                return None;
            }
        }
        Some(())
    };
    check().is_some()
}

pub(in super::super) async fn assert_native_topology_metadata(pool: &PgPool) {
    let actual = observation(pool).await;
    assert!(
        matches(&actual),
        "actual topology metadata differs from approved physical choices"
    );
    for (section, index, key, bad) in [
        ("tables", 0, "owner", json!("console_rt")),
        ("tables", 0, "rls", json!(true)),
        ("tables", 0, "force_rls", json!(true)),
        ("tables", 0, "policies", json!(1)),
        (
            "tables",
            2,
            "foreign_targets",
            json!(["public.organizations"]),
        ),
        ("routines", 0, "security_definer", json!(true)),
        ("routines", 0, "returns", json!("void")),
        (
            "routines",
            0,
            "config",
            json!(["search_path=public", "row_security=off"]),
        ),
        ("routines", 0, "volatility", json!("s")),
        ("routines", 0, "parallel", json!("s")),
        ("triggers", 0, "exact_function", json!(false)),
        ("triggers", 0, "enabled", json!("D")),
        ("triggers", 0, "type", json!(4)),
        ("triggers", 0, "when", json!("true")),
        ("triggers", 0, "argument_count", json!(1)),
        ("triggers", 0, "argument_bytes", json!(1)),
        ("triggers", 0, "internal", json!(true)),
        ("triggers", 0, "parent_trigger", json!(true)),
        ("triggers", 0, "update_columns", json!(["org_id"])),
        ("triggers", 0, "old_transition_table", json!("old_rows")),
        ("triggers", 12, "initially_deferred", json!(false)),
    ] {
        let mut corrupt = actual.clone();
        corrupt[section][index][key] = bad;
        assert_ne!(corrupt, actual, "ineffective corruption control");
        assert!(
            !matches(&corrupt),
            "topology oracle accepted {section}.{key}"
        );
    }
    for section in ["tables", "routines", "triggers"] {
        let mut corrupt = actual.clone();
        corrupt[section].as_array_mut().unwrap().pop();
        assert!(
            !matches(&corrupt),
            "topology oracle accepted omitted {section}"
        );
    }
    let mut corrupt = actual.clone();
    corrupt["trigger_count"] = json!(19);
    assert!(
        !matches(&corrupt),
        "topology oracle accepted extra guard use"
    );
    for (section, key) in [
        ("tables", "table_acl"),
        ("tables", "column_acl"),
        ("routines", "acl"),
    ] {
        let mut corrupt = actual.clone();
        corrupt[section][0][key].as_array_mut().unwrap().push(
            json!({"role":"PUBLIC","privilege":"SELECT","column":"org_id","grantable":false}),
        );
        assert!(
            !matches(&corrupt),
            "topology oracle accepted additional ACL"
        );
    }
    let before = all_rows(pool).await;
    for login in [
        TestDatabaseLogin::Business,
        TestDatabaseLogin::Auth,
        TestDatabaseLogin::LeaveCommand,
        TestDatabaseLogin::OntologyCommand,
        TestDatabaseLogin::PlatformForceCommand,
    ] {
        let serving = login_test_pool(pool, login).await;
        for table in contract()["new_topology_tables"]
            .as_object()
            .unwrap()
            .keys()
        {
            let denied:bool=sqlx::query_scalar("SELECT NOT (has_table_privilege(current_user,$1,'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN') OR has_any_column_privilege(current_user,$1,'SELECT,INSERT,UPDATE,REFERENCES'))")
                .bind(format!("public.{table}")).fetch_one(&serving).await.unwrap();
            assert!(
                denied,
                "serving login has direct or inherited topology table/column access"
            );
        }
        for signature in contract()["new_guard_functions"]
            .as_object()
            .unwrap()
            .keys()
        {
            let denied: bool =
                sqlx::query_scalar("SELECT NOT has_function_privilege(current_user,$1,'EXECUTE')")
                    .bind(signature)
                    .fetch_one(&serving)
                    .await
                    .unwrap();
            assert!(denied, "serving login can execute private topology guard");
        }
        serving.close().await;
    }
    assert_eq!(
        before,
        all_rows(pool).await,
        "metadata inspection mutated data"
    );
}
