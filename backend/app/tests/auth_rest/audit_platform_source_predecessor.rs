// Included inside platform_source_transition. Only the independently checked
// Platform extension is inverted before unchanged historical boundary oracles.
fn strip_checked_platform_source(before: &Value, after: &Value, addition: &Value) -> Value {
    adversarial::assert_metadata_shape(before);
    adversarial::assert_metadata_shape(after);
    let prior = before["functions"].as_array().unwrap();
    let current = after["functions"].as_array().unwrap();
    let id = addition["oid"].as_str().expect("vetted bridge OID");
    let numeric = id.parse::<u32>().expect("numeric bridge OID");
    assert!(numeric > 0 && numeric.to_string() == id && addition["proname"] == BRIDGE);
    assert_eq!(addition["prosrc"], INSTALL.split("$body$").nth(1).unwrap());
    assert!(
        !prior
            .iter()
            .any(|row| row["oid"] == addition["oid"] || row["proname"] == BRIDGE)
    );
    assert_eq!(
        current
            .iter()
            .filter(|row| row["proname"] == BRIDGE)
            .count(),
        1
    );
    assert_eq!(
        current.iter().filter(|row| *row == addition).count(),
        1,
        "observed bridge must equal the independently checked live pg_proc row"
    );
    let old_context = prior
        .iter()
        .filter(|row| row["proname"] == CONTEXT)
        .collect::<Vec<_>>();
    let new_context = current
        .iter()
        .filter(|row| row["proname"] == CONTEXT)
        .collect::<Vec<_>>();
    assert_eq!((old_context.len(), new_context.len()), (1, 1));
    let (old_context, new_context) = (old_context[0], new_context[0]);
    assert_ne!(old_context["oid"], addition["oid"]);
    assert_eq!(old_context["proargtypes"], json!(["2950", "2950"]));
    let owners = before["roles"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|role| role["oid"] == old_context["proowner"])
        .collect::<Vec<_>>();
    assert_eq!(owners.len(), 1);
    assert_eq!(owners[0]["rolname"], "console_app");
    // Exact frozen ACL text, including original order and grantor, is retained.
    // Never normalize an arbitrary old/new ACL by deleting a grantee name.
    assert_eq!(
        old_context["proacl"],
        json!(["console_app=X/console_app", "console_auth_rt=X/console_app"])
    );
    assert_eq!(
        new_context["proacl"],
        json!([
            "console_app=X/console_app",
            "console_auth_rt=X/console_app",
            "console_credential_owner=X/console_app"
        ])
    );
    let mut preserved_context = new_context.clone();
    preserved_context["proacl"] = old_context["proacl"].clone();
    assert!(
        preserved_context == *old_context,
        "Platform extension changed the old context definition"
    );
    let mut stripped = after.clone();
    stripped["functions"]
        .as_array_mut()
        .unwrap()
        .retain(|row| row != addition);
    let context = stripped["functions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["oid"] == old_context["oid"])
        .unwrap();
    context["proacl"] = old_context["proacl"].clone();
    // Everything else remains in the historical one-/two-helper/full-state
    // oracle. Those checks, not a blanket function/metadata projection, decide.
    stripped
}

pub(super) async fn without_checked_platform_source(
    connection: &mut PgConnection,
    before: &Value,
    after: &Value,
) -> Value {
    checked_bridge(connection).await;
    let bridge_row:Value=sqlx::query_scalar("SELECT to_jsonb(p) FROM pg_proc p WHERE p.oid='public.auth_legacy_platform_source_material_v1(uuid,uuid)'::regprocedure")
        .fetch_one(connection).await.unwrap();
    strip_checked_platform_source(before, after, &bridge_row)
}

pub(super) async fn rollback_checked_platform_source(
    connection: &mut PgConnection,
    original: &Value,
) {
    // Caller already holds compose's maintenance locks and has drained real
    // consumers. This is the precise test inverse, never a serving downgrade.
    let current = metadata(connection).await;
    let expected = without_checked_platform_source(connection, original, &current).await;
    let complete = complete_metadata(connection).await;
    let rows = all_business_state(connection).await;
    sqlx::raw_sql("DROP FUNCTION public.auth_legacy_platform_source_material_v1(uuid,uuid) RESTRICT; REVOKE EXECUTE ON FUNCTION public.auth_legacy_session_context_v1(uuid,uuid) FROM console_credential_owner;")
        .execute(&mut *connection).await.unwrap();
    assert!(
        expected == metadata(connection).await,
        "Platform inverse failed exact intermediate equality"
    );
    let restored = complete_metadata(connection).await;
    exact_two_object_delta(&restored, &complete);
    assert!(
        rows == all_business_state(connection).await,
        "Platform inverse changed business/history rows"
    );
}

fn retained_chain_fixture(kind: usize) -> (Value, Value, Value, Value, Vec<Value>) {
    let mut before = adversarial::metadata_control(false);
    before["functions"][0]["proname"] = json!("unrelated_retained");
    before["roles"][0]["rolname"] = json!("console_app");
    before["functions"].as_array_mut().unwrap().push(json!({
        "oid":"610","proname":CONTEXT,"proowner":"500","proargtypes":["2950","2950"],
        "prosrc":"unchanged historical context","proacl":["console_app=X/console_app","console_auth_rt=X/console_app"]
    }));
    let mut after = if kind == 2 {
        adversarial::metadata_control(true)
    } else {
        before.clone()
    };
    after["functions"] = before["functions"].clone();
    after["roles"] = before["roles"].clone();
    after["functions"][1]["proacl"]
        .as_array_mut()
        .unwrap()
        .push(json!("console_credential_owner=X/console_app"));
    let eligibility = json!({"oid":"900","proname":super::NAME});
    let sessions = if kind == 0 {
        vec![]
    } else {
        vec![
            json!({"oid":"700","proname":super::super::NAMES[0]}),
            json!({"oid":"701","proname":super::super::NAMES[1]}),
        ]
    };
    for row in &sessions {
        after["functions"].as_array_mut().unwrap().push(row.clone());
    }
    after["functions"]
        .as_array_mut()
        .unwrap()
        .push(eligibility.clone());
    let bridge =
        json!({"oid":"910","proname":BRIDGE,"prosrc":INSTALL.split("$body$").nth(1).unwrap()});
    after["functions"]
        .as_array_mut()
        .unwrap()
        .push(bridge.clone());
    (before, after, bridge, eligibility, sessions)
}

fn retained_chain_oracle(
    kind: usize,
    before: &Value,
    after: &Value,
    bridge: &Value,
    eligibility: &Value,
    sessions: &[Value],
) {
    let without_source = strip_checked_platform_source(before, after, bridge);
    let without_eligibility = super::strip_checked_one(before, &without_source, eligibility);
    if kind == 0 {
        assert_eq!(
            without_source["functions"].as_array().unwrap().len(),
            before["functions"].as_array().unwrap().len() + 1
        );
        assert!(
            without_eligibility == *before,
            "original one-helper exact metadata oracle"
        );
    } else {
        let historical =
            super::super::strip_exact_additions(before, &without_eligibility, sessions);
        if kind == 1 {
            assert!(
                historical == *before,
                "original two-helper exact metadata oracle"
            );
        } else {
            adversarial::assert_transition_metadata(before, &historical);
        }
    }
}

#[test]
fn platform_source_composition_retains_one_two_and_full_historical_oracles() {
    for kind in 0..3 {
        let (before, after, bridge, eligibility, sessions) = retained_chain_fixture(kind);
        let saved = (before.clone(), after.clone(), bridge.clone());
        retained_chain_oracle(kind, &before, &after, &bridge, &eligibility, &sessions);
        assert!(
            (before, after, bridge) == saved,
            "composition mutated its source evidence"
        );
    }
}

#[test]
fn platform_source_composition_rejects_stale_bridge_acl_and_unrelated_history() {
    for kind in 0..3 {
        let (before, after, bridge, eligibility, sessions) = retained_chain_fixture(kind);
        for corruption in 0..24 {
            let mut left = before.clone();
            let mut right = after.clone();
            let mut checked = bridge.clone();
            let bridge_index = right["functions"].as_array().unwrap().len() - 1;
            match corruption {
                0 => {
                    right["functions"].as_array_mut().unwrap().pop();
                }
                1 => {
                    right["functions"]
                        .as_array_mut()
                        .unwrap()
                        .push(bridge.clone());
                }
                2 => {
                    right["functions"][bridge_index]["prosrc"] = json!("changed body");
                }
                3 => {
                    checked["oid"] = json!("911");
                }
                4 => {
                    checked["proname"] = json!("wrong helper");
                }
                5 => {
                    right["functions"][bridge_index]["oid"] = json!("600");
                    checked["oid"] = json!("600");
                }
                6 => {
                    left["functions"]
                        .as_array_mut()
                        .unwrap()
                        .push(bridge.clone());
                }
                7 => {
                    right["functions"][1]["proacl"] = left["functions"][1]["proacl"].clone();
                }
                8 => {
                    right["functions"][1]["proacl"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!("PUBLIC=X/console_app"));
                }
                9 => {
                    right["functions"][1]["proacl"][2] =
                        json!("console_credential_owner=X*/console_app");
                }
                10 => {
                    right["functions"][1]["proacl"][2] =
                        json!("console_credential_owner=X/console_credential_owner");
                }
                11 => {
                    right["functions"][1]["proacl"]
                        .as_array_mut()
                        .unwrap()
                        .remove(1);
                }
                12 => {
                    right["functions"][1]["prosrc"] = json!("changed old body");
                }
                13 => {
                    right["functions"][1]["proowner"] = json!("501");
                }
                14 => {
                    right["functions"][1]["proargtypes"] = json!(["25", "2950"]);
                }
                15 => {
                    right["functions"][0]["prosrc"] = json!("unrelated drift");
                }
                16 => {
                    right["functions"].as_array_mut().unwrap().remove(0);
                }
                17 => {
                    right["functions"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!({"oid":"920","proname":"extra"}));
                }
                18 => {
                    right["roles"][0]["rolsuper"] = json!(true);
                }
                19 => {
                    right["memberships"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!({"oid":"921","roleid":"500","member":"501"}));
                }
                20 => {
                    right["ledger"][0]["version"] = json!(229);
                }
                21 => {
                    right["relations"][0]["rls"] = json!(true);
                }
                22 => {
                    left["functions"][1]["proacl"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!("PUBLIC=X/console_app"));
                    right["functions"][1]["proacl"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!("PUBLIC=X/console_app"));
                }
                23 => {
                    right["functions"].as_array_mut().unwrap().remove(1);
                }
                _ => unreachable!(),
            }
            assert!(
                std::panic::catch_unwind(|| retained_chain_oracle(
                    kind,
                    &left,
                    &right,
                    &checked,
                    &eligibility,
                    &sessions
                ))
                .is_err(),
                "historical chain {kind} missed corruption {corruption}"
            );
        }
        retained_chain_oracle(kind, &before, &after, &bridge, &eligibility, &sessions);
    }
}
