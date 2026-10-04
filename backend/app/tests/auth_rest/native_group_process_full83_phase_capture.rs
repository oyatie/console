// Additive metadata diagnostic only. Both plain/observer variants roll back.
// Historical migration rows remain. No serving profile, workflow acceptance,
// business-empty/UI evidence, Group activation or production qualification.
mod native_group_process_full83_phase_capture {
    use super::*;

    const GROUP_OWNER: &str =
        include_str!("../../../../ops/postgres-native-group-process-v1-owner.sql");
    const GROUP_CAPTURE: &str =
        include_str!("../../../../ops/postgres-capture-native-group-process-v1-custody.sql");
    const ORACLE: &str = include_str!("fixtures/native-group-full83-diagnostic-oracle-v1.json");
    const GROUP_SCHEMA: &str = include_str!(
        "../../../../backend/crates/platform/authz/src/group_process/process-v1.cedarschema"
    );
    const GROUP_POLICY: &str = include_str!(
        "../../../../backend/crates/platform/authz/src/group_process/process-v1.cedar"
    );
    const GROUP_CODEC: &str =
        include_str!("../../../../ops/native-group-process/codec-contract-v1.json");
    const DECLARATIONS: [(&str, &str, &str); 14] = [
        (
            "ops/native-group-process/schema-v1.sql",
            include_str!("../../../../ops/native-group-process/schema-v1.sql"),
            "7b0278ac5993e7ef972740f86c0d969b3f0a0b724abd5a873f3241cd637926c4",
        ),
        (
            "ops/native-group-process/codec-v1.sql",
            include_str!("../../../../ops/native-group-process/codec-v1.sql"),
            "feb28474e77d423da87a0eb2e8cd1588791a2c9ecf6e6112e00afc6e12153219",
        ),
        (
            "ops/native-group-process/result-codec-v2.sql",
            include_str!("../../../../ops/native-group-process/result-codec-v2.sql"),
            "032c8222d7a123ac5614ee29053662c0e543949d557c96b27b865106a6ed9e47",
        ),
        (
            "ops/native-group-process/source-v1.sql",
            include_str!("../../../../ops/native-group-process/source-v1.sql"),
            "6cda12aa3051768db0418b0b0aecb82a590ddfac0c753ef01d07e027720beea7",
        ),
        (
            "ops/native-group-process/locks-v1.sql",
            include_str!("../../../../ops/native-group-process/locks-v1.sql"),
            "a74237a27cc270a00bf8fd82595e8f3cfeb79f3f2bb2c981719cef4d0d8abe01",
        ),
        (
            "ops/native-group-process/closure-v1.sql",
            include_str!("../../../../ops/native-group-process/closure-v1.sql"),
            "4ca58c102d9ed803e0decff8b5e4eaa130f3c594a55b251facc2e98d6042ca2f",
        ),
        (
            "ops/native-group-process/context-v1.sql",
            include_str!("../../../../ops/native-group-process/context-v1.sql"),
            "0b51c44a6401f5eaf5452b25458cd9cbddb1c321d10e3199133a5441e6c80a2a",
        ),
        (
            "ops/native-group-process/material-v1.sql",
            include_str!("../../../../ops/native-group-process/material-v1.sql"),
            "1d78d9336ca88f482bd157da2b18324e6ed3bc1374add4f898867ee753d53ff3",
        ),
        (
            "ops/native-group-process/transition-v1.sql",
            include_str!("../../../../ops/native-group-process/transition-v1.sql"),
            "55b3fe3456e0e1d92565621856ccf9fbffda64f3b619881ef2e224f17eb7effb",
        ),
        (
            "ops/native-group-process/guards-v1.sql",
            include_str!("../../../../ops/native-group-process/guards-v1.sql"),
            "011700957a179bb1b4d862512eb53e30c37a5fb15262e3d831eb32c8b4f17949",
        ),
        (
            "ops/native-group-process/commands-v1.sql",
            include_str!("../../../../ops/native-group-process/commands-v1.sql"),
            "9b86d73c28b5f5f98da684b7b8da6ba8225c115228d3d7bb437f64bb133f28c7",
        ),
        (
            "ops/native-group-process/audit-v1.sql",
            include_str!("../../../../ops/native-group-process/audit-v1.sql"),
            "5ed8aa96a668de24cd0b0daef13bc5084bf3951fac581597d3bf3c746ac4607f",
        ),
        (
            "ops/native-group-process/discovery-v1.sql",
            include_str!("../../../../ops/native-group-process/discovery-v1.sql"),
            "58908576d4790f9e7ced9b1040484de9c9594dcf4560f8439c6378cee2e61779",
        ),
        (
            "ops/native-group-process/acl-v1.sql",
            include_str!("../../../../ops/native-group-process/acl-v1.sql"),
            "5661c038e1d07076d0ced512cdc43ce9c04e5bb918f8bf3a33c3bc0821bf30e0",
        ),
    ];

    fn group_pins(oracle: &Value) -> Value {
        let mut pins = source_pins().as_object().unwrap().clone();
        for (name, source, expected) in [
            (
                "closed_owner",
                CLOSED_OWNER,
                "aba221ad2cfa03390eca638ccd290901877fef615450b17cf14a96304e9f2609",
            ),
            (
                "historical_wider76",
                WIDER_CAPTURE,
                "6be2e3d095d59bbdb9e1b932dac8da48bde261601455cdcd166c6f2a649e6010",
            ),
            (
                "group_owner",
                GROUP_OWNER,
                "cbf641175a7bf589bd46fc21dc775fe2fab8b1a8ab7e46b04dee3c32422038f9",
            ),
            (
                "group_capture",
                GROUP_CAPTURE,
                "3406bac381896fab4e9a1c079110d3dc770a9d89b0b564e7744034379f60cd3b",
            ),
            (
                "group_schema",
                GROUP_SCHEMA,
                "c711017368596094ad0ff9b1123eb17573722df47f1b4abb15a31ce1ce8b1e44",
            ),
            (
                "group_policy",
                GROUP_POLICY,
                "2453684b70134124a8cf77f2d882fb7497d8c1498325fac1321ca7e616791ed8",
            ),
            (
                "group_codec",
                GROUP_CODEC,
                "595376f9edea8ebd5a698bd27d6f7310a470f5e5d95522d7eed7c01db8169067",
            ),
            (
                "oracle",
                ORACLE,
                "16f95a8bdc1f38bae675a8fce908ce54a959fdcd50f7a524362ed1cdc0c7a909",
            ),
        ] {
            assert_eq!(digest(source), expected, "unreviewed diagnostic {name}");
            assert!(pins.insert(name.into(), json!(expected)).is_none());
        }
        assert!(
            oracle["phase_pairs"].is_null(),
            "installed phase hashes must be observed"
        );
        assert_eq!(oracle["source_owner_sha256"], digest(GROUP_OWNER));
        let declared: Vec<_> = DECLARATIONS.iter().map(|(path, _, _)| *path).collect();
        assert_eq!(oracle["source_order"], json!(declared));
        let mut exact_owner = String::from(
            "-- Generated UNINSTALLED native Group source; not a custody finalizer.\n\
             -- No installed profile or serving readiness is asserted by this artifact.\n",
        );
        for (index, (path, source, expected)) in DECLARATIONS.into_iter().enumerate() {
            assert_eq!(digest(source), expected, "unreviewed declaration: {path}");
            assert!(pins.insert(path.into(), json!(expected)).is_none());
            exact_owner.push_str(&format!("-- source: {path}\n"));
            exact_owner.push_str(source);
            if index + 1 != DECLARATIONS.len() {
                exact_owner.push('\n');
            }
        }
        assert_eq!(
            GROUP_OWNER, exact_owner,
            "complete exact14 owner order/bytes differ"
        );
        json!(pins)
    }

    async fn full83_capture(connection: &mut PgConnection, expected_rights: bool) -> Capture {
        sqlx::raw_sql(CLASSIFIER_SESSION)
            .execute(&mut *connection)
            .await
            .unwrap();
        let body = GROUP_CAPTURE
            .strip_suffix(";\n")
            .expect("reviewed capture terminator");
        let query = format!(
            "SELECT snapshot::text,snapshot_sha256,native_group_process_startup_rights_valid FROM ({body}) original_capture"
        );
        let (text, sha256, rights): (String, String, Option<bool>) =
            sqlx::query_as(sqlx::AssertSqlSafe(query))
                .fetch_one(connection)
                .await
                .unwrap();
        assert_eq!(
            digest(&text),
            sha256,
            "exact PostgreSQL UTF-8 capture digest differs"
        );
        assert_eq!(
            rights,
            Some(expected_rights),
            "complete83 effective-rights verdict differs"
        );
        let snapshot: Value = serde_json::from_str(&text).unwrap();
        assert!(snapshot.is_object());
        assert_eq!(
            snapshot["deployment_operator_boundary"]["startup_final_rights_valid"], false,
            "serialized historical18 predicate must stay raw FALSE for both83 phases"
        );
        Capture {
            text,
            sha256,
            rights: expected_rights,
            snapshot,
        }
    }

    fn expected_group_names(oracle: &Value) -> BTreeSet<String> {
        oracle["relation_roster"]
            .as_array()
            .unwrap()
            .iter()
            .map(|name| name.as_str().unwrap().to_owned())
            .collect()
    }

    fn assert_full83_tables(
        snapshot: &Value,
        oracle: &Value,
        installed: bool,
        direct_catalog: Option<&Value>,
    ) {
        let group_names = expected_group_names(oracle);
        assert_eq!(group_names.len(), 7);
        let tables = snapshot["tables"].as_array().unwrap();
        assert_eq!(
            tables.len(),
            83,
            "complete historical76 plus Group7 roster required"
        );
        let mut names = BTreeSet::new();
        for table in tables {
            let name = table["name"].as_str().unwrap();
            assert!(
                names.insert(name.to_owned()),
                "duplicate table in complete capture"
            );
            if group_names.contains(name) {
                if !installed {
                    assert!(
                        table["owner"].is_null(),
                        "predecessor must have missing Group7"
                    );
                    assert!(
                        table["shape"]["relation"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .all(Value::is_null)
                    );
                    continue;
                }
                assert_eq!(table["owner"], "console_account_owner");
                assert_eq!(table["shape"]["relation"][0], "r");
                assert_eq!(table["shape"]["relation"][1], "p");
                assert_eq!(table["shape"]["relation"][2], true);
                assert_eq!(table["shape"]["relation"][3], true);
                assert_eq!(table["shape"]["relation"][4], false);
                assert_eq!(
                    table["policies"],
                    json!([{
                        "name":"native_group_owner_only","permissive":true,"command":"*",
                        "roles":["console_account_owner"],"using":"true","check":"true"
                    }])
                );
                let acl = table["acl"].as_array().unwrap();
                assert_eq!(
                    acl.len(),
                    8,
                    "all eight owner-only table privileges required"
                );
                let privileges: BTreeSet<_> = acl
                    .iter()
                    .map(|entry| {
                        assert_eq!(entry[0], "console_account_owner");
                        assert_eq!(entry[1], "console_account_owner");
                        assert_eq!(entry[3], false);
                        entry[2].as_str().unwrap().to_owned()
                    })
                    .collect();
                assert_eq!(
                    privileges,
                    BTreeSet::from_iter(
                        [
                            "SELECT",
                            "INSERT",
                            "UPDATE",
                            "DELETE",
                            "TRUNCATE",
                            "REFERENCES",
                            "TRIGGER",
                            "MAINTAIN"
                        ]
                        .map(str::to_owned)
                    )
                );
                for column in table["column_security"].as_array().unwrap() {
                    assert!(
                        column["acl"].is_null()
                            || column["acl"].as_array().is_some_and(Vec::is_empty),
                        "Group column ACL must have no added grantee"
                    );
                }
            } else {
                assert!(
                    table["owner"].as_str().is_some(),
                    "missing historical predecessor relation"
                );
                assert_eq!(table["shape"]["relation"][0], "r");
            }
        }
        assert!(group_names.is_subset(&names));
        let expected_names: BTreeSet<_> = oracle["historical76_relations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r.as_str().unwrap().to_owned())
            .chain(group_names.iter().cloned())
            .collect();
        assert_eq!(expected_names.len(), 83);
        assert_eq!(
            names, expected_names,
            "exact historical76 plus Group7 names differ"
        );
        if installed {
            let boundary = &snapshot["deployment_operator_boundary"];
            let repeated: BTreeSet<_> = oracle["startup_repeated_nine_relations"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| r.as_str().unwrap())
                .collect();
            assert_eq!(repeated.len(), 9);
            assert!(repeated.iter().all(|name| names.contains(*name)));
            let mut expected_table_checks = BTreeMap::new();
            for name in &names {
                for privilege in [
                    "SELECT",
                    "INSERT",
                    "UPDATE",
                    "DELETE",
                    "TRUNCATE",
                    "REFERENCES",
                    "TRIGGER",
                    "MAINTAIN",
                ] {
                    assert!(
                        expected_table_checks
                            .insert(
                                (name.clone(), privilege.to_owned()),
                                if repeated.contains(name.as_str()) {
                                    2_usize
                                } else {
                                    1
                                }
                            )
                            .is_none()
                    );
                }
            }
            let table_rights = boundary["startup_table_rights"].as_array().unwrap();
            assert_eq!(
                table_rights.len(),
                736,
                "92 relation entries times eight privileges, retaining historical UNION ALL"
            );
            let mut actual_table_checks = BTreeMap::new();
            for check in table_rights {
                assert_eq!(
                    check[2], false,
                    "NULL/allowed startup table privilege fails"
                );
                *actual_table_checks
                    .entry((
                        check[0].as_str().unwrap().to_owned(),
                        check[1].as_str().unwrap().to_owned(),
                    ))
                    .or_insert(0_usize) += 1;
            }
            assert_eq!(
                actual_table_checks, expected_table_checks,
                "exact per-name/per-privilege multiplicities required"
            );
            assert_eq!(
                actual_table_checks
                    .keys()
                    .map(|key| key.0.as_str())
                    .collect::<BTreeSet<_>>()
                    .len(),
                83
            );
            // Enumerate the columns from independent direct pg_attribute/catalog
            // readback, never from the startup matrix being tested. Retain the
            // nine duplicated relation entries in the frozen historical query.
            let direct_catalog =
                direct_catalog.expect("installed capture needs direct catalog readback");
            let public: Vec<_> = direct_catalog["schemas"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|n| n["nspname"] == "public")
                .collect();
            assert_eq!(public.len(), 1);
            let public_oid = &public[0]["oid"];
            let mut expected_column_checks = BTreeMap::new();
            for name in &names {
                let relations: Vec<_> = direct_catalog["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|r| r["relnamespace"] == *public_oid && r["relname"] == name.as_str())
                    .collect();
                assert_eq!(
                    relations.len(),
                    1,
                    "one direct public relation required: {name}"
                );
                let columns: Vec<_> = direct_catalog["attributes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|a| {
                        a["attrelid"] == relations[0]["oid"]
                            && a["attnum"].as_i64().unwrap() > 0
                            && a["attisdropped"] == false
                    })
                    .collect();
                assert!(
                    !columns.is_empty(),
                    "direct relation has no live user columns: {name}"
                );
                for column in columns {
                    for privilege in ["SELECT", "INSERT", "UPDATE", "REFERENCES"] {
                        assert!(
                            expected_column_checks
                                .insert(
                                    (
                                        name.clone(),
                                        column["attname"].as_str().unwrap().to_owned(),
                                        privilege.to_owned()
                                    ),
                                    if repeated.contains(name.as_str()) {
                                        2_usize
                                    } else {
                                        1
                                    }
                                )
                                .is_none()
                        );
                    }
                }
            }
            let column_rights = boundary["startup_column_rights"].as_array().unwrap();
            assert!(!column_rights.is_empty());
            let mut actual_column_checks = BTreeMap::new();
            for check in column_rights {
                assert_eq!(
                    check[3], false,
                    "NULL/allowed startup column privilege fails"
                );
                *actual_column_checks
                    .entry((
                        check[0].as_str().unwrap().to_owned(),
                        check[1].as_str().unwrap().to_owned(),
                        check[2].as_str().unwrap().to_owned(),
                    ))
                    .or_insert(0_usize) += 1;
            }
            assert_eq!(
                actual_column_checks, expected_column_checks,
                "every direct column/privilege and historical multiplicity required"
            );
        }
    }

    fn assert_group_routines(catalog: &Value, snapshot: &Value, oracle: &Value) {
        let actual = routine_map(catalog);
        let expected = oracle["routines"].as_array().unwrap();
        assert_eq!(expected.len(), 41);
        assert_eq!(
            expected
                .iter()
                .filter(|r| r["runtime_execute"] == true)
                .count(),
            5,
            "exactly five reviewed Group runtime entrypoints required"
        );
        let names: BTreeSet<_> = expected
            .iter()
            .map(|r| r["name"].as_str().unwrap())
            .collect();
        assert_eq!(names.len(), 41);
        let group: Vec<_> = actual
            .values()
            .filter(|r| {
                let name = r["name"].as_str().unwrap();
                name.starts_with("native_group_process")
                    || name.starts_with("native_group_identity_policy")
                    || name.starts_with("identity_native_group_process")
            })
            .collect();
        assert_eq!(
            group.len(),
            41,
            "full reserved routine namespace must match exact41 ABIs"
        );
        let captured = snapshot["routines"].as_array().unwrap();
        for expected in expected {
            let identity = format!(
                "public.{}({})",
                expected["name"].as_str().unwrap(),
                expected["type_arguments"].as_str().unwrap()
            );
            let routine = &actual[&identity];
            let raw = &routine["raw"];
            assert_eq!(routine["schema"], "public");
            assert_eq!(routine["owner"], "console_account_owner");
            for field in [
                "name",
                "language",
                "identity_arguments",
                "result",
                "source_sha256",
            ] {
                assert_eq!(
                    routine[field], expected[field],
                    "complete declared routine {field}: {identity}"
                );
            }
            assert_eq!(
                raw["prosrc"], expected["body"],
                "every routine body byte must read back exactly: {identity}"
            );
            assert_eq!(
                digest(raw["prosrc"].as_str().unwrap()),
                expected["source_sha256"].as_str().unwrap()
            );
            for (field, expected_field) in [
                ("provolatile", "volatility"),
                ("proparallel", "parallel"),
                ("proisstrict", "strict"),
                ("prosecdef", "security_definer"),
                ("proretset", "returns_set"),
                ("proargnames", "argnames"),
                ("proargmodes", "argmodes"),
                ("proconfig", "config"),
            ] {
                assert_eq!(
                    raw[field], expected[expected_field],
                    "complete routine ABI: {identity}/{field}"
                );
            }
            assert_eq!(raw["prokind"], "f");
            assert_eq!(raw["proleakproof"], false);
            assert_eq!(raw["procost"], 100);
            assert_eq!(
                raw["prorows"],
                if expected["returns_set"] == true {
                    1000
                } else {
                    0
                }
            );
            assert_eq!(raw["provariadic"], "0");
            assert_eq!(raw["pronargdefaults"], 0);
            assert_eq!(routine["support_oid"], "0");
            for field in ["proargdefaults", "probin", "prosqlbody", "protrftypes"] {
                assert!(
                    raw[field].is_null(),
                    "unexpected routine ABI: {identity}/{field}"
                );
            }
            if expected["returns_set"] != true {
                assert!(raw["proallargtypes"].is_null());
            }
            let runtime = expected["runtime_execute"].as_bool().unwrap();
            let mut acl = vec![json!([
                "console_account_owner",
                "console_account_owner",
                "EXECUTE",
                false
            ])];
            if runtime {
                acl.push(json!([
                    "console_account_owner",
                    "console_rt",
                    "EXECUTE",
                    false
                ]));
            }
            assert_eq!(
                routine["acl"],
                json!(acl),
                "exact owner/runtime-only routine ACL: {identity}"
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
            assert_eq!(rights["console_rt"], (runtime, false));
            assert_eq!(rights["console_auth_rt"], (false, false));
            assert_eq!(rights["console_auth_startup"], (false, false));
            assert!(rights["console_account_owner"].0);
            let exact: Vec<_> = captured
                .iter()
                .filter(|r| {
                    r["metadata"]["schema"] == "public"
                        && r["metadata"]["name"] == expected["name"]
                        && r["metadata"]["identity_arguments"] == expected["identity_arguments"]
                })
                .collect();
            assert_eq!(
                exact.len(),
                1,
                "complete capture omitted/duplicated ABI: {identity}"
            );
            assert_eq!(exact[0]["extra_valid"], true);
            let metadata = &exact[0]["metadata"];
            for field in [
                "schema",
                "name",
                "owner",
                "language",
                "identity_arguments",
                "result",
                "source_sha256",
                "acl",
            ] {
                assert_eq!(
                    metadata[field], routine[field],
                    "direct readback/capture mismatch: {identity}/{field}"
                );
            }
        }
        let namespace = snapshot["native_group_process_routine_namespace"]
            .as_array()
            .unwrap();
        assert_eq!(namespace.len(), 41);
        for entry in namespace {
            assert_eq!(entry[0], "public");
            assert_eq!(entry[3], "f");
            assert_eq!(entry[4], "console_account_owner");
            let expected: Vec<_> = oracle["routines"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|r| r["name"] == entry[1] && r["identity_arguments"] == entry[2])
                .collect();
            assert_eq!(expected.len(), 1, "unclaimed reserved routine");
        }
    }

    async fn group_effective_denials(connection: &mut PgConnection, oracle: &Value) -> Value {
        let names: Vec<String> = expected_group_names(oracle).into_iter().collect();
        let evidence: Value = sqlx::query_scalar(
            "WITH relations AS (SELECT c.oid,c.relname FROM pg_catalog.pg_class c \
             JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relname=ANY($1)), \
             roles AS (SELECT oid,rolname FROM pg_catalog.pg_roles WHERE rolname IN('console_rt','console_auth_rt','console_auth_startup')), \
             table_checks AS (SELECT r.relname,role.rolname,p.name AS privilege,pg_catalog.has_table_privilege(role.oid,r.oid,p.name) AS allowed \
             FROM relations r CROSS JOIN roles role CROSS JOIN (VALUES('SELECT'),('INSERT'),('UPDATE'),('DELETE'),('TRUNCATE'),('REFERENCES'),('TRIGGER'),('MAINTAIN')) p(name)), \
             column_checks AS (SELECT r.relname,role.rolname,a.attnum,a.attname,p.name AS privilege,pg_catalog.has_column_privilege(role.oid,r.oid,a.attnum,p.name) AS allowed \
             FROM relations r JOIN pg_catalog.pg_attribute a ON a.attrelid=r.oid CROSS JOIN roles role CROSS JOIN (VALUES('SELECT'),('INSERT'),('UPDATE'),('REFERENCES')) p(name) \
             WHERE a.attnum>0 AND NOT a.attisdropped) \
             SELECT jsonb_build_object('roles',(SELECT jsonb_agg(rolname ORDER BY rolname COLLATE \"C\") FROM roles), \
             'relations',(SELECT jsonb_agg(relname ORDER BY relname COLLATE \"C\") FROM relations), \
             'tables',(SELECT jsonb_agg(jsonb_build_array(relname,rolname,privilege,allowed) ORDER BY relname COLLATE \"C\",rolname COLLATE \"C\",privilege COLLATE \"C\") FROM table_checks), \
             'columns',(SELECT jsonb_agg(jsonb_build_array(relname,rolname,attnum,attname,privilege,allowed) ORDER BY relname COLLATE \"C\",rolname COLLATE \"C\",attnum,privilege COLLATE \"C\") FROM column_checks))"
        ).bind(&names).fetch_one(connection).await.unwrap();
        assert_eq!(
            evidence["roles"],
            json!(["console_auth_rt", "console_auth_startup", "console_rt"])
        );
        assert_eq!(evidence["relations"], json!(names));
        let tables = evidence["tables"].as_array().unwrap();
        assert_eq!(tables.len(), 7 * 3 * 8);
        assert!(tables.iter().all(|r| r[3] == false));
        assert_eq!(
            tables
                .iter()
                .map(|r| serde_json::to_string(&r.as_array().unwrap()[..3]).unwrap())
                .collect::<BTreeSet<_>>()
                .len(),
            tables.len()
        );
        let columns = evidence["columns"].as_array().unwrap();
        assert!(!columns.is_empty());
        assert!(columns.iter().all(|r| r[5] == false));
        let column_identities: BTreeSet<_> = columns
            .iter()
            .map(|r| (r[0].as_str().unwrap(), r[2].as_i64().unwrap()))
            .collect();
        assert_eq!(columns.len(), column_identities.len() * 3 * 4);
        assert_eq!(
            column_identities
                .iter()
                .map(|r| r.0)
                .collect::<BTreeSet<_>>(),
            names.iter().map(String::as_str).collect()
        );
        assert_eq!(
            columns
                .iter()
                .map(|r| serde_json::to_string(&r.as_array().unwrap()[..5]).unwrap())
                .collect::<BTreeSet<_>>()
                .len(),
            columns.len()
        );
        evidence
    }

    fn unchanged_existing_rows(
        before: &BTreeMap<String, String>,
        after: &BTreeMap<String, String>,
        oracle: &Value,
    ) {
        let mut existing = after.clone();
        for name in expected_group_names(oracle) {
            let identity = serde_json::to_string(&("public", name)).unwrap();
            assert!(!before.contains_key(&identity));
            assert_eq!(
                existing.remove(&identity).as_deref(),
                Some("[]"),
                "new Group tables must have no business rows"
            );
        }
        assert_eq!(
            existing, *before,
            "Group DDL changed any durable prerequisite/business/history row"
        );
        assert_eq!(after.len(), before.len() + 7);
    }

    #[sqlx::test(migrations = false)]
    async fn collects_full83_plain_observer_declared_metadata_and_rolls_back(pool: PgPool) {
        let oracle: Value = serde_json::from_str(ORACLE).unwrap();
        let pins = group_pins(&oracle);
        let mut initial = pool.begin().await.unwrap();
        marked(initial.as_mut(), true).await;
        initial.rollback().await.unwrap();
        prepare_http_database_staging(&pool).await;
        let mut initial = pool.begin().await.unwrap();
        marked(initial.as_mut(), false).await;
        let identity: Value = sqlx::query_scalar("SELECT jsonb_build_object('database',current_database(),'database_oid',(SELECT oid FROM pg_catalog.pg_database WHERE datname=current_database()),'session_user',session_user,'current_user',current_user,'fixture_marker',current_setting('console.sqlx_test_bootstrap',true),'system_identifier',(pg_catalog.pg_control_system()).system_identifier::text,'server_version_num',current_setting('server_version_num'),'server_encoding',current_setting('server_encoding'))").fetch_one(initial.as_mut()).await.unwrap();
        let ledger = applied_ledger(initial.as_mut()).await;
        let baseline_rows = rows(initial.as_mut()).await;
        let baseline_catalog = catalog(initial.as_mut()).await;
        let absent: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_durability_observer') AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_proc WHERE proname='console_durability_observation_v1')").fetch_one(initial.as_mut()).await.unwrap();
        assert!(
            absent,
            "dedicated disposable cluster must start without observer"
        );
        initial.rollback().await.unwrap();
        let mut packets = Vec::new();
        for variant in 0..2 {
            let mut tx = pool.begin().await.unwrap();
            let outcome = AssertUnwindSafe(async {
                sqlx::raw_sql("SET TRANSACTION ISOLATION LEVEL READ COMMITTED; SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='120s'; SET LOCAL search_path=pg_catalog,pg_temp").execute(tx.as_mut()).await.unwrap();
                marked(tx.as_mut(), false).await;
                let _: String = sqlx::query_scalar("SELECT oid::text FROM pg_catalog.pg_authid WHERE rolname='console_account_owner' FOR UPDATE").fetch_one(tx.as_mut()).await.unwrap();
                if variant==1 { execute(&mut tx, OBSERVER).await; }
                for source in [ACCOUNT,CREDENTIALS,COMPANY,POLICY_INSTALLER,POLICY_V2,ROW_LOCK] { execute(&mut tx, source).await; }
                assert_eq!(capture(tx.as_mut(), PREDECESSOR_CAPTURE).await.sha256, CORRECTED[variant]);
                assert_eq!(row_lock_state(tx.as_mut()).await, "native_people_directory.finalized");
                execute(&mut tx, OWNER).await;
                assert_eq!(capture(tx.as_mut(), CAPTURE).await.sha256, CLASSIFIER_INSTALLED[variant]);
                let prerequisite_rows = rows(tx.as_mut()).await;
                execute(&mut tx, CLOSED_OWNER).await;
                let historical73_before = capture(tx.as_mut(), CAPTURE).await;
                let historical76_before = raw_wider_capture(tx.as_mut()).await;
                assert_eq!(historical73_before.sha256,oracle["accepted_closed_predecessor_pairs"]["closed73"][variant].as_str().unwrap(),
                    "exact accepted historical73 CLOSED predecessor required; never normalize/reseal");
                assert_eq!(historical76_before["snapshot_sha256"],oracle["accepted_closed_predecessor_pairs"]["closed76"][variant],
                    "exact accepted historical76 CLOSED predecessor required; never normalize/reseal");
                let denied_org_before = added_three_denied_rights(tx.as_mut()).await;
                assert_eq!(rows(tx.as_mut()).await, prerequisite_rows);
                assert_eq!(applied_ledger(tx.as_mut()).await, ledger);
                let before83 = full83_capture(tx.as_mut(), false).await;
                assert_full83_tables(&before83.snapshot, &oracle, false, None);
                for key in ["native_group_process_relation_namespace","native_group_process_schema_namespace","native_group_process_type_namespace","native_group_process_routine_namespace"] {
                    assert!(before83.snapshot[key].is_null(), "Group reserved namespace must be absent: {key}");
                }
                let before_catalog = catalog(tx.as_mut()).await;
                let before_rows = rows(tx.as_mut()).await;
                marked(tx.as_mut(), false).await;
                execute(&mut tx, GROUP_OWNER).await;
                let installed83 = full83_capture(tx.as_mut(), true).await;
                let after_catalog = catalog(tx.as_mut()).await;
                assert_full83_tables(&installed83.snapshot, &oracle, true, Some(&after_catalog.1));
                assert_ne!(before83.text, installed83.text, "complete capture omitted source installation");
                assert_eq!(full83_capture(tx.as_mut(), true).await, installed83, "exact installed raw83 must be deterministic");
                assert_group_routines(&after_catalog.1, &installed83.snapshot, &oracle);
                let old_routines = routine_map(&before_catalog.1);
                let new_routines = routine_map(&after_catalog.1);
                assert_eq!(new_routines.len(), old_routines.len()+41);
                assert!(old_routines.iter().all(|(identity,r)| new_routines.get(identity)==Some(r)), "Group source changed an existing routine");
                for key in ["roles","memberships","database","database_role_settings","schemas","default_acls","event_triggers"] {
                    assert_eq!(before_catalog.1[key],after_catalog.1[key], "Group source changed unrelated catalog: {key}");
                }
                assert!(installed83.snapshot["native_group_process_schema_namespace"].is_null());
                let types = installed83.snapshot["native_group_process_type_namespace"].as_array().unwrap();
                assert_eq!(types.len(),7);
                assert_eq!(types.iter().map(|r| { assert_eq!(r[0],"public"); assert_eq!(r[2],"c"); assert_eq!(r[3],"console_account_owner"); r[1].as_str().unwrap().to_owned() }).collect::<BTreeSet<_>>(),expected_group_names(&oracle));
                let relations = installed83.snapshot["native_group_process_relation_namespace"].as_array().unwrap();
                assert_eq!(relations.len(),30, "reviewed source has seven tables and23 indexes");
                assert!(relations.iter().all(|r| r[0]=="public" && r[3]=="console_account_owner" && (r[2]=="r" || r[2]=="i")));
                assert_eq!(relations.iter().filter(|r| r[2]=="r").map(|r| r[1].as_str().unwrap().to_owned()).collect::<BTreeSet<_>>(),expected_group_names(&oracle));
                let effective_denials = group_effective_denials(tx.as_mut(),&oracle).await;
                let historical73_after = capture(tx.as_mut(), CAPTURE).await;
                let historical76_after = raw_wider_capture(tx.as_mut()).await;
                assert_ne!(historical73_before.text,historical73_after.text,"untouched historical73 serializer must notice new owner/routine metadata");
                assert_ne!(historical76_before["snapshot_text"],historical76_after["snapshot_text"],"untouched historical76 serializer must notice new owner/routine metadata");
                assert_eq!(added_three_denied_rights(tx.as_mut()).await,denied_org_before);
                let after_rows = rows(tx.as_mut()).await;
                unchanged_existing_rows(&before_rows,&after_rows,&oracle);
                assert_eq!(applied_ledger(tx.as_mut()).await,ledger);
                assert_eq!(catalog(tx.as_mut()).await,after_catalog,"complete catalog readback must be deterministic");
                json!({"schema":"console.native_group_full83_phase_diagnostic.v1","variant":if variant==0 {"plain"} else {"observer"},
                    "sources":pins,"database_identity":identity,"applied_ledger":ledger,
                    "historical73_before":historical73_before.record(),"historical73_after":historical73_after.record(),
                    "historical76_before":historical76_before,"historical76_after":historical76_after,
                    "full83_before":before83.record(),"full83_installed":installed83.record(),"group_effective_denials":effective_denials,
                    "raw_catalog_before_text":before_catalog.0,"raw_catalog_after_text":after_catalog.0,
                    "raw_catalog_before_sha256":digest(&before_catalog.0),"raw_catalog_after_sha256":digest(&after_catalog.0),
                    "business_rows_before":row_summary(&before_rows),"business_rows_after":row_summary(&after_rows),
                    "existing_business_history_unchanged":true,"new_group_tables_empty":true,"source_readback41_verified":true,
                    "observed_phase_hashes_not_accepted":true,"serving_profile_accepted":false,"group_browser_accepted":false,"business_empty":false,"production_qualified":false})
            }).catch_unwind().await;
            tx.rollback()
                .await
                .expect("all prerequisite/Group source/observer DDL must roll back");
            let mut restored = pool.begin().await.unwrap();
            assert_eq!(
                rows(restored.as_mut()).await,
                baseline_rows,
                "variant rollback changed any durable row"
            );
            assert_eq!(
                catalog(restored.as_mut()).await,
                baseline_catalog,
                "variant rollback changed full raw security/catalog metadata"
            );
            assert_eq!(applied_ledger(restored.as_mut()).await, ledger);
            restored.rollback().await.unwrap();
            match outcome {
                Ok(mut packet) => {
                    packet["complete_catalog_and_rows_rollback_verified"] = json!(true);
                    packets.push(packet);
                }
                Err(panic) => std::panic::resume_unwind(panic),
            }
        }
        assert_eq!(packets.len(), 2);
        assert_ne!(
            packets[0]["full83_installed"]["snapshot_sha256"],
            packets[1]["full83_installed"]["snapshot_sha256"],
            "actual observer variant must change complete capture"
        );
        let mut output = std::io::stderr().lock();
        for packet in packets {
            writeln!(&mut output, "GROUP_FULL83_PHASE_DIAGNOSTIC {packet}")
                .expect("diagnostic output write failed");
        }
        writeln!(&mut output,"GROUP_FULL83_PHASE_DIAGNOSTIC_COMPLETE variants=2 source_order=14 routines=41 runtime_executors=5 full83_predecessor_rights=false full83_installed_rights=true ledger231=unchanged rows=unchanged new_group_tables=7-empty rollback=verified acceptance=not_claimed").expect("diagnostic completion write failed");
        output.flush().expect("diagnostic evidence flush failed");
    }
}
