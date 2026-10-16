// Exact finite Manager namespace and full routine ABI oracle.
fn manager_specs() -> Vec<(
    &'static str,
    &'static str,
    Vec<&'static str>,
    Vec<&'static str>,
    Vec<&'static str>,
    &'static str,
)> {
    vec![
        (
            "group_lock",
            "console_app",
            vec!["p_company", "p_group"],
            vec!["group_row", "group_head_row"],
            vec!["jsonb", "jsonb"],
            "f4fd844cbe546e36f288c6ae62d4535e123cec59f20b612a05d2fa3ae145b086",
        ),
        (
            "manager_current",
            "console_account_owner",
            vec![
                "p_account",
                "p_family",
                "p_company",
                "p_command",
                "p_expected_group",
            ],
            vec![
                "actor_account_id",
                "session_id",
                "org_id",
                "command_id",
                "current_group_id",
                "company_epoch",
                "current_policy_receipt_id",
                "context_generation",
                "assignment_id",
                "assignment_revision",
                "role_id",
                "role_revision",
                "registered_clauses",
                "company_name",
                "company_slug",
                "installed_object_type_id",
                "observed_at",
                "source_xid",
                "source_backend_pid",
                "source_material",
            ],
            vec![
                "uuid",
                "uuid",
                "uuid",
                "uuid",
                "uuid",
                "bigint",
                "uuid",
                "bigint",
                "uuid",
                "bigint",
                "uuid",
                "bigint",
                "jsonb",
                "text",
                "text",
                "uuid",
                "timestamp with time zone",
                "xid8",
                "integer",
                "jsonb",
            ],
            "4f45d405fcc2543d52f95124b953f2eb88526ce8e5ebe51442d49297ba775415",
        ),
        (
            "root_material",
            "console_account_owner",
            vec!["p_company", "p_group"],
            vec![
                "root_account_id",
                "company_epoch",
                "current_policy_receipt_id",
                "assignment_id",
                "role_id",
                "installed_object_type_id",
                "registered_clauses",
                "root_material",
            ],
            vec![
                "uuid", "bigint", "uuid", "uuid", "uuid", "uuid", "jsonb", "jsonb",
            ],
            "c7d147815f5ae05afba0bc257076e321789baaf1189f61d586676acb67c72fe2",
        ),
        (
            "selected_lock",
            "console_app",
            vec!["p_company", "p_group"],
            vec!["company_row", "membership_row", "membership_revision_row"],
            vec!["jsonb", "jsonb", "jsonb"],
            "f5e9abaf6553bbd6599b7d27c3f4f8f95ff594eba3cb2f33439d8f43b8565c8e",
        ),
    ]
}

fn expected_namespace() -> Value {
    json!(
        manager_specs()
            .into_iter()
            .map(|(name, owner, inputs, _, _, _)| {
                // Identity arguments exclude TABLE output arguments. The output ABI
                // is independently checked through result/argnames/modes/allargtypes.
                let arguments = inputs
                    .iter()
                    .map(|n| format!("{n} uuid"))
                    .collect::<Vec<_>>();
                json!([
                    "public",
                    format!("identity_company_information_{name}_v1"),
                    arguments.join(", "),
                    "f",
                    owner,
                    true
                ])
            })
            .collect::<Vec<_>>()
    )
}

fn manager_exact_routines(catalog: &Value) {
    let routines = routine_map(catalog);
    for (name, owner, inputs, outputs, types, body) in manager_specs() {
        let arguments = vec!["uuid"; inputs.len()].join(", ");
        let identity = format!("public.identity_company_information_{name}_v1({arguments})");
        let routine = &routines[&identity];
        assert_eq!(routine["owner"], owner);
        assert_eq!(routine["language"], "plpgsql");
        assert_eq!(routine["source_sha256"], body);
        assert_eq!(digest(routine["raw"]["prosrc"].as_str().unwrap()), body);
        let fields = outputs
            .iter()
            .zip(&types)
            .map(|(n, t)| format!("{n} {t}"))
            .collect::<Vec<_>>();
        assert_eq!(routine["result"], format!("TABLE({})", fields.join(", ")));
        let acl = if name == "manager_current" {
            json!([
                [owner, "console_account_owner", "EXECUTE", false],
                [owner, "console_rt", "EXECUTE", false]
            ])
        } else {
            json!([[owner, "console_account_owner", "EXECUTE", false]])
        };
        assert_eq!(routine["acl"], acl);
        let raw = &routine["raw"];
        for (field, value) in [
            ("prokind", json!("f")),
            ("provolatile", json!("v")),
            ("proparallel", json!("u")),
            ("prosecdef", json!(true)),
            ("proretset", json!(true)),
            ("proisstrict", json!(false)),
            ("proleakproof", json!(false)),
            ("procost", json!(100)),
            ("prorows", json!(1000)),
            ("pronargdefaults", json!(0)),
            ("pronargs", json!(inputs.len())),
        ] {
            assert_eq!(raw[field], value, "{identity}: {field}");
        }
        assert_eq!(
            raw["proargnames"],
            json!(inputs.iter().chain(&outputs).collect::<Vec<_>>())
        );
        assert_eq!(
            raw["proargmodes"],
            json!(
                vec!["i"; inputs.len()]
                    .into_iter()
                    .chain(vec!["t"; outputs.len()])
                    .collect::<Vec<_>>()
            )
        );
        assert_eq!(
            raw["proconfig"],
            json!([
                "search_path=pg_catalog, pg_temp",
                "row_security=on",
                "TimeZone=UTC",
                "bytea_output=hex",
                "DateStyle=ISO, YMD",
                "IntervalStyle=postgres"
            ])
        );
        for field in ["proargdefaults", "probin", "prosqlbody", "protrftypes"] {
            assert!(raw[field].is_null(), "{identity}: unexpected {field}");
        }
        assert_eq!(raw["provariadic"], "0");
        assert_eq!(raw["prorettype"], "2249"); // Built-in record, not an invented named type.
        assert_eq!(routine["support_oid"], "0");
        assert_eq!(
            raw["proacl"].as_array().unwrap().len(),
            acl.as_array().unwrap().len()
        );
        let type_oid = |t: &str| match t {
            "uuid" => "2950",
            "jsonb" => "3802",
            "bigint" => "20",
            "text" => "25",
            "timestamp with time zone" => "1184",
            "xid8" => "5069",
            "integer" => "23",
            _ => panic!("unreviewed type"),
        };
        assert_eq!(raw["proargtypes"], json!(vec!["2950"; inputs.len()]));
        assert_eq!(
            raw["proallargtypes"],
            json!(
                vec!["2950"; inputs.len()]
                    .into_iter()
                    .chain(types.iter().map(|t| type_oid(t)))
                    .collect::<Vec<_>>()
            )
        );
        let rights: BTreeMap<_, _> = routine["effective_execute"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                (
                    r[0].as_str().unwrap(),
                    (r[1].as_bool().unwrap(), r[2].as_bool().unwrap()),
                )
            })
            .collect();
        for role in [
            "console_app",
            "console_account_owner",
            "console_rt",
            "console_auth_rt",
            "console_auth_startup",
            "console_terms_owner",
            "console_credential_owner",
            "console_ontology_writer",
        ] {
            // Ownership retains grant options; the exact helper ACL revokes
            // console_app ordinary EXECUTE and grants only account owner.
            let execute = role == "console_account_owner"
                || (role == "console_rt" && name == "manager_current");
            assert_eq!(
                rights[role],
                (execute, role == owner),
                "{identity}: effective {role} rights"
            );
        }
    }
}

fn manager_catalog_delta(before: &Value, after: &Value) {
    let old = routine_map(before);
    let new = routine_map(after);
    assert_eq!(new.len(), old.len() + 4);
    assert!(
        old.iter()
            .all(|(identity, record)| new.get(identity) == Some(record)),
        "existing routine/dependencies changed"
    );
    let added: Vec<_> = new
        .keys()
        .filter(|identity| !old.contains_key(*identity))
        .cloned()
        .collect();
    let expected: Vec<_> = manager_specs()
        .into_iter()
        .map(|(n, _, inputs, _, _, _)| {
            format!(
                "public.identity_company_information_{n}_v1({})",
                vec!["uuid"; inputs.len()].join(", ")
            )
        })
        .collect();
    assert_eq!(added, expected);
    // Keep original raw catalog immutable; compare all nonroutine projections.
    let old_fields = before.as_object().unwrap();
    let new_fields = after.as_object().unwrap();
    assert_eq!(old_fields.len(), new_fields.len());
    for (key, value) in old_fields {
        if key != "routines" {
            assert_eq!(
                new_fields.get(key),
                Some(value),
                "changed nonroutine catalog: {key}"
            );
        }
    }
    manager_exact_routines(after);
}
