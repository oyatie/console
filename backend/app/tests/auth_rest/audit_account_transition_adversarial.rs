// Included inside account_browser::audit_account_transition. Actual custody SQL
// and historical producers are owned by the parent; no replacement owner here.
mod adversarial {
    use super::{compose, metadata, preservation_holds, prior228, rows, state};
    use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
    use serde_json::{Value, json};
    use sqlx::{Connection, PgConnection, PgPool};
    use std::time::Duration;

    struct Edge {
        constraint: i64,
        triggers: Vec<i64>,
    }

    const METADATA_KEYS: &[&str] = &["relations", "functions", "roles", "memberships", "ledger"];
    const RELATION_KEYS: &[&str] = &[
        "name",
        "oid",
        "owner",
        "acl",
        "rls",
        "force",
        "columns",
        "defaults",
        "constraints",
        "triggers",
        "indexes",
        "policies",
    ];

    fn exact_keys(value: &Value, expected: &[&str]) -> bool {
        value.as_object().is_some_and(|object| {
            object.len() == expected.len() && expected.iter().all(|key| object.contains_key(*key))
        })
    }

    // A before/after equality is meaningful only for complete observations.
    // These keys mirror the constructed JSON SELECT, independently of either
    // observed value. Raw catalog rows remain unprojected in the collector.
    fn catalog_oid(value: &Value) -> Option<i64> {
        // PostgreSQL emits OID values as canonical decimal JSON strings. Keep
        // raw observations intact; parse only to identify the declared edge.
        // Zero is a valid sentinel (e.g. a nonconstraint trigger's tgconstraint).
        let text = value.as_str()?;
        let oid = text.parse::<u32>().ok()?;
        (oid.to_string() == text).then_some(i64::from(oid))
    }

    fn metadata_shape_valid(value: &Value) -> bool {
        if !exact_keys(value, METADATA_KEYS) {
            return false;
        }
        for name in ["functions", "roles", "memberships", "ledger"] {
            let Some(records) = value.get(name).and_then(Value::as_array) else {
                return false;
            };
            if (name != "memberships" && records.is_empty())
                || records.iter().any(|record| !record.is_object())
            {
                return false;
            }
        }
        let Some(relations) = value.get("relations").and_then(Value::as_array) else {
            return false;
        };
        if relations.len() != super::TABLES.len() {
            return false;
        }
        let mut names = std::collections::BTreeSet::new();
        let mut identities = std::collections::BTreeSet::new();
        for relation in relations {
            if !exact_keys(relation, RELATION_KEYS) {
                return false;
            }
            let Some(name) = relation.get("name").and_then(Value::as_str) else {
                return false;
            };
            let Some(oid) = relation.get("oid").and_then(catalog_oid) else {
                return false;
            };
            if !super::TABLES.contains(&name)
                || !names.insert(name)
                || oid == 0
                || !identities.insert(oid)
                || !relation
                    .get("owner")
                    .and_then(catalog_oid)
                    .is_some_and(|owner| owner > 0)
                || !relation.get("rls").is_some_and(Value::is_boolean)
                || !relation.get("force").is_some_and(Value::is_boolean)
                || !relation.get("acl").is_some_and(|acl| {
                    acl.is_null()
                        || acl
                            .as_array()
                            .is_some_and(|acl| acl.iter().all(Value::is_string))
                })
            {
                return false;
            }
            for field in [
                "columns",
                "defaults",
                "constraints",
                "triggers",
                "indexes",
                "policies",
            ] {
                let Some(records) = relation.get(field).and_then(Value::as_array) else {
                    return false;
                };
                if (field == "columns" && records.is_empty())
                    || records.iter().any(|record| !record.is_object())
                {
                    return false;
                }
            }
        }
        true
    }

    pub(super) fn assert_metadata_shape(value: &Value) {
        assert!(
            metadata_shape_valid(value),
            "incomplete metadata observation; payloads omitted"
        );
    }

    // Independently check the actual column, all actor-bearing constraints and
    // exact RI endpoints. The metadata comparator excludes only these IDs.
    async fn edge(connection: &mut PgConnection, target: &str) -> Edge {
        assert!(matches!(target, "users" | "accounts"));
        let column: (i16, bool) = sqlx::query_as(
            "SELECT attnum,atttypid='uuid'::regtype AND NOT attnotnull AND NOT attisdropped AND attidentity='' AND attgenerated='' AND NOT atthasdef FROM pg_attribute WHERE attrelid='public.audit_events'::regclass AND attname='actor' AND attnum>0"
        ).fetch_one(&mut *connection).await.unwrap();
        assert!(column.1, "nullable unchanged UUID actor required");
        let candidates: Vec<(i64, bool)> = sqlx::query_as(
            "SELECT k.oid::bigint,k.conname='audit_events_actor_fkey' AND k.confrelid=to_regclass('public.'||$2) AND k.conkey=ARRAY[$1]::smallint[] AND k.confkey=ARRAY[a.attnum]::smallint[] AND k.convalidated AND k.conenforced AND NOT k.condeferrable AND NOT k.condeferred AND k.confupdtype='a' AND k.confdeltype='r' AND k.confmatchtype='s' AND k.conparentid=0 AND k.conislocal AND k.coninhcount=0 AND k.confdelsetcols IS NULL AND k.conindid=p.conindid AND i.indisunique AND i.indisvalid AND i.indisready AND i.indislive AND i.indimmediate AND NOT i.indisexclusion FROM pg_constraint k JOIN pg_attribute a ON a.attrelid=to_regclass('public.'||$2) AND a.attname='id' AND NOT a.attisdropped JOIN pg_constraint p ON p.conrelid=a.attrelid AND p.contype='p' AND p.conkey=ARRAY[a.attnum]::smallint[] JOIN pg_index i ON i.indexrelid=p.conindid WHERE k.conrelid='public.audit_events'::regclass AND k.contype='f' AND $1=ANY(k.conkey)"
        ).bind(column.0).bind(target).fetch_all(&mut *connection).await.unwrap();
        assert!(
            candidates.len() == 1 && candidates[0].1,
            "one exact actor FK required"
        );
        let constraint = candidates[0].0;
        let actual: Vec<(i64, Value, bool)> = sqlx::query_as(
            "SELECT t.oid::bigint,jsonb_build_array(c.relname,o.relname,p.proname,t.tgtype::integer),t.tgisinternal AND t.tgenabled='O' AND NOT t.tgdeferrable AND NOT t.tginitdeferred AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr::text='' AND t.tgqual IS NULL AND t.tgparentid=0 AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL AND t.tgconstrindid=k.conindid AND t.tgname ~ '^RI_ConstraintTrigger_[ac]_[0-9]+$' AND n.nspname='pg_catalog' FROM pg_trigger t JOIN pg_constraint k ON k.oid=t.tgconstraint JOIN pg_class c ON c.oid=t.tgrelid JOIN pg_class o ON o.oid=t.tgconstrrelid JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace n ON n.oid=p.pronamespace WHERE t.tgconstraint=$1::bigint::oid ORDER BY p.proname"
        ).bind(constraint).fetch_all(&mut *connection).await.unwrap();
        let expected = [
            json!(["audit_events", target, "RI_FKey_check_ins", 5]),
            json!(["audit_events", target, "RI_FKey_check_upd", 17]),
            json!([target, "audit_events", "RI_FKey_noaction_upd", 17]),
            json!([target, "audit_events", "RI_FKey_restrict_del", 9]),
        ];
        assert!(
            actual.len() == 4 && actual.iter().zip(expected).all(|(a, e)| a.2 && a.1 == e),
            "four exact enabled actor RI triggers required"
        );
        Edge {
            constraint,
            triggers: actual.into_iter().map(|item| item.0).collect(),
        }
    }

    fn without_edge(mut observed: Value, edge: &Edge) -> Value {
        assert_metadata_shape(&observed);
        let mut constraints = 0;
        let mut triggers = 0;
        for relation in observed
            .get_mut("relations")
            .and_then(Value::as_array_mut)
            .expect("complete relation census")
        {
            relation
                .get_mut("constraints")
                .and_then(Value::as_array_mut)
                .expect("constraint census")
                .retain(|row| {
                    let keep = row
                        .get("oid")
                        .and_then(catalog_oid)
                        .filter(|oid| *oid > 0)
                        .expect("constraint identity")
                        != edge.constraint;
                    if !keep {
                        constraints += 1;
                    }
                    keep
                });
            relation
                .get_mut("triggers")
                .and_then(Value::as_array_mut)
                .expect("trigger census")
                .retain(|row| {
                    let keep = !edge.triggers.contains(
                        &row.get("oid")
                            .and_then(catalog_oid)
                            .filter(|oid| *oid > 0)
                            .expect("trigger identity"),
                    );
                    if !keep {
                        triggers += 1;
                    }
                    keep
                });
        }
        assert!(
            constraints == 1 && triggers == 4,
            "only the exact actor edge may be projected out"
        );
        observed
    }

    pub(super) fn assert_transition_metadata(before: &Value, after: &Value) {
        assert_metadata_shape(before);
        assert_metadata_shape(after);
        let identify = |metadata: &Value, target: &str| {
            let relations = metadata
                .get("relations")
                .and_then(Value::as_array)
                .expect("complete relation census");
            let relation = |name: &str| {
                let found: Vec<_> = relations
                    .iter()
                    .filter(|r| r.get("name").and_then(Value::as_str) == Some(name))
                    .collect();
                assert!(found.len() == 1, "exact relation identity required");
                found[0]
            };
            let audit = relation("audit_events");
            let actor = audit
                .get("columns")
                .and_then(Value::as_array)
                .unwrap()
                .iter()
                .find(|c| c.get("attname").and_then(Value::as_str) == Some("actor"))
                .expect("actor column identity")
                .get("attnum")
                .and_then(Value::as_i64)
                .unwrap();
            let candidates: Vec<_> = audit
                .get("constraints")
                .and_then(Value::as_array)
                .unwrap()
                .iter()
                .filter(|c| {
                    c.get("contype").and_then(Value::as_str) == Some("f")
                        && c.get("conkey")
                            .and_then(Value::as_array)
                            .is_some_and(|keys| keys.iter().any(|key| key.as_i64() == Some(actor)))
                })
                .collect();
            assert!(candidates.len() == 1, "no additional actor-bearing FK");
            let constraint = candidates[0];
            assert!(
                constraint.get("conname").and_then(Value::as_str)
                    == Some("audit_events_actor_fkey")
                    && constraint.get("confrelid") == relation(target).get("oid"),
                "exact old/new actor parent"
            );
            let oid = constraint
                .get("oid")
                .and_then(catalog_oid)
                .filter(|oid| *oid > 0)
                .expect("positive actor constraint identity");
            let mut triggers = Vec::new();
            for r in relations {
                for trigger in r.get("triggers").and_then(Value::as_array).unwrap() {
                    if trigger
                        .get("tgconstraint")
                        .and_then(catalog_oid)
                        .expect("canonical trigger constraint OID")
                        == oid
                    {
                        triggers.push(
                            trigger
                                .get("oid")
                                .and_then(catalog_oid)
                                .filter(|oid| *oid > 0)
                                .expect("positive actor trigger identity"),
                        );
                    }
                }
            }
            assert!(triggers.len() == 4, "exact actor RI census");
            Edge {
                constraint: oid,
                triggers,
            }
        };
        let old = identify(before, "users");
        let new = identify(after, "accounts");
        assert!(old.constraint != new.constraint);
        assert!(
            without_edge(before.clone(), &old) == without_edge(after.clone(), &new),
            "metadata changed outside exact actor edge; payloads omitted"
        );
    }

    // Deliberately synthetic structural evidence, not a PostgreSQL profile or
    // authority fixture. The actual schema/FK/RI oracle remains the DB test.
    pub(super) fn metadata_control(successor: bool) -> Value {
        let mut relations: Vec<Value> = super::TABLES.iter().enumerate().map(|(index, name)| {
            json!({"name":name,"oid":(index as u64 + 1).to_string(),"owner":"500","acl":null,"rls":false,"force":false,
                "columns":[{"attname":"id","attnum":1}],"defaults":[],"constraints":[],"triggers":[],"indexes":[],"policies":[]})
        }).collect();
        let table_oid = |name| {
            super::TABLES
                .iter()
                .position(|actual| *actual == name)
                .unwrap() as u64
                + 1
        };
        let audit = &mut relations[super::TABLES
            .iter()
            .position(|name| *name == "audit_events")
            .unwrap()];
        let constraint: u32 = if successor { 2000 } else { 1000 };
        audit["columns"]
            .as_array_mut()
            .unwrap()
            .push(json!({"attname":"actor","attnum":2}));
        audit["constraints"] = json!([{"oid":constraint.to_string(),"conname":"audit_events_actor_fkey","contype":"f","conkey":[2],"confrelid":table_oid(if successor {"accounts"} else {"users"}).to_string()}]);
        audit["triggers"] = json!(
            (0..4)
                .map(|offset| json!({"oid":(constraint*10+offset).to_string(),"tgconstraint":constraint.to_string()}))
                .collect::<Vec<_>>()
        );
        // Real non-FK triggers carry the canonical zero sentinel. They remain
        // in the complete equality comparison and must not match the actor FK.
        audit["triggers"]
            .as_array_mut()
            .unwrap()
            .push(json!({"oid":"40000","tgconstraint":"0"}));
        json!({"relations":relations,"functions":[{"oid":"600"}],"roles":[{"oid":"500"}],"memberships":[],"ledger":[{"version":228}]})
    }

    #[test]
    fn metadata_oracle_requires_complete_select_roster_even_when_both_sides_omit_it() {
        let before = metadata_control(false);
        let after = metadata_control(true);
        assert_metadata_shape(&before);
        assert_metadata_shape(&after);
        assert_transition_metadata(&before, &after);
        let mut pairs = Vec::new();
        for key in METADATA_KEYS {
            let mut left = before.clone();
            let mut right = after.clone();
            left.as_object_mut().unwrap().remove(*key);
            right.as_object_mut().unwrap().remove(*key);
            pairs.push((left, right));
        }
        for key in RELATION_KEYS {
            let mut left = before.clone();
            let mut right = after.clone();
            for value in [&mut left, &mut right] {
                for relation in value["relations"].as_array_mut().unwrap() {
                    relation.as_object_mut().unwrap().remove(*key);
                }
            }
            pairs.push((left, right));
        }
        for variant in 0..7 {
            let mut left = before.clone();
            let mut right = after.clone();
            for value in [&mut left, &mut right] {
                let relations = value["relations"].as_array_mut().unwrap();
                match variant {
                    0 => {
                        relations.remove(0);
                    }
                    1 => {
                        relations[0] = relations[1].clone();
                    }
                    2 => {
                        relations[0]["owner"] = Value::Null;
                    }
                    3 => {
                        relations[0]["acl"] = json!("not an ACL array");
                    }
                    4 => {
                        relations[0]["columns"] = json!([]);
                    }
                    5 => {
                        relations[0]["policies"] = Value::Null;
                    }
                    6 => {
                        relations[0]["unselected_field"] = Value::Null;
                    }
                    _ => unreachable!(),
                }
            }
            pairs.push((left, right));
        }
        assert!(pairs.len() == 24);
        for (left, right) in pairs {
            assert!(
                !metadata_shape_valid(&left) && !metadata_shape_valid(&right),
                "paired missing or malformed evidence accepted"
            );
            // Proves the actual comparison invokes the structural guard, not
            // merely that a detached checker recognizes the negative fixture.
            assert!(
                std::panic::catch_unwind(|| assert_transition_metadata(&left, &right)).is_err(),
                "comparison bypassed completeness validation"
            );
        }
        assert_eq!(catalog_oid(&json!("0")), Some(0));
        assert_eq!(catalog_oid(&json!("1")), Some(1));
        assert_eq!(catalog_oid(&json!("4294967295")), Some(i64::from(u32::MAX)));
        let malformed = [
            Value::Null,
            json!(1),
            json!(-1),
            json!(1.0),
            json!(true),
            json!([]),
            json!({}),
            json!(""),
            json!("00"),
            json!("01"),
            json!("-1"),
            json!("+1"),
            json!(" 1"),
            json!("1 "),
            json!("1.0"),
            json!("1e0"),
            json!("4294967296"),
            json!("١"),
        ];
        let audit_index = super::TABLES
            .iter()
            .position(|name| *name == "audit_events")
            .unwrap();
        for invalid in malformed {
            assert!(catalog_oid(&invalid).is_none(), "noncanonical OID accepted");
            for field in 0..6 {
                let mut left = before.clone();
                let mut right = after.clone();
                for value in [&mut left, &mut right] {
                    match field {
                        0 => value["relations"][0]["oid"] = invalid.clone(),
                        1 => value["relations"][0]["owner"] = invalid.clone(),
                        2 => {
                            value["relations"][audit_index]["constraints"][0]["oid"] =
                                invalid.clone()
                        }
                        3 => {
                            value["relations"][audit_index]["triggers"][0]["oid"] = invalid.clone()
                        }
                        4 => {
                            value["relations"][audit_index]["triggers"][0]["tgconstraint"] =
                                invalid.clone()
                        }
                        5 => {
                            value["relations"][audit_index]["constraints"][0]["confrelid"] =
                                invalid.clone()
                        }
                        _ => unreachable!(),
                    }
                }
                assert!(
                    std::panic::catch_unwind(|| assert_transition_metadata(&left, &right)).is_err(),
                    "paired malformed OID passed actual comparator"
                );
            }
        }
        // Zero parses for sentinel fields, but never identifies a real catalog row.
        for field in 0..6 {
            let mut left = before.clone();
            let mut right = after.clone();
            for value in [&mut left, &mut right] {
                match field {
                    0 => value["relations"][0]["oid"] = json!("0"),
                    1 => value["relations"][0]["owner"] = json!("0"),
                    2 => value["relations"][audit_index]["constraints"][0]["oid"] = json!("0"),
                    3 => value["relations"][audit_index]["triggers"][0]["oid"] = json!("0"),
                    4 => {
                        value["relations"][audit_index]["triggers"][0]["tgconstraint"] = json!("0")
                    }
                    5 => {
                        value["relations"][audit_index]["constraints"][0]["confrelid"] = json!("0")
                    }
                    _ => unreachable!(),
                }
            }
            assert!(
                std::panic::catch_unwind(|| assert_transition_metadata(&left, &right)).is_err(),
                "zero cannot identify actor edge or relation authority"
            );
        }
        let mut changed = after.clone();
        changed["relations"][0]["owner"] = json!("999");
        assert!(metadata_shape_valid(&changed));
        assert!(
            std::panic::catch_unwind(|| assert_transition_metadata(&before, &changed)).is_err(),
            "valid shape must still compare every retained value"
        );
    }

    async fn finalized(pool: &PgPool) {
        let mut tx = pool.begin().await.unwrap();
        compose(&mut tx).await.unwrap();
        edge(&mut tx, "accounts").await;
        assert!(state(&mut tx).await == "account_custody.native_finalized");
        tx.commit().await.unwrap();
    }

    #[sqlx::test(migrations = false)]
    async fn sole_actor_edge_changes_and_adjacent_metadata_is_identical(pool: PgPool) {
        prior228(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        let old_edge = edge(&mut tx, "users").await;
        assert!(state(&mut tx).await == "account_custody.native_upgrade_required");
        let before_rows = rows(&mut tx).await;
        let before_metadata = metadata(&mut tx).await;
        let before = without_edge(before_metadata.clone(), &old_edge);
        compose(&mut tx).await.unwrap();
        let new_edge = edge(&mut tx, "accounts").await;
        assert!(new_edge.constraint != old_edge.constraint);
        assert!(
            preservation_holds(&before_rows, &rows(&mut tx).await),
            "transition changed complete rows"
        );
        let after_metadata = metadata(&mut tx).await;
        let predecessor_equivalent = super::business_session_transition::predecessor_equivalent(
            &mut tx,
            &before_metadata,
            &after_metadata,
        )
        .await;
        assert!(
            before == without_edge(predecessor_equivalent, &new_edge),
            "metadata changed beyond exact actor constraint, four RI triggers and two vetted session helpers"
        );
        assert!(state(&mut tx).await == "account_custody.native_finalized");
        tx.commit().await.unwrap();
    }

    fn replace_fk(target: &str, options: &str) -> String {
        assert!(matches!(target, "users" | "accounts"));
        format!(
            "ALTER TABLE public.audit_events DROP CONSTRAINT audit_events_actor_fkey, ADD CONSTRAINT audit_events_actor_fkey FOREIGN KEY(actor) REFERENCES public.{target}(id) {options}"
        )
    }

    async fn drift(pool: &PgPool, mutation: fn(&str) -> String) {
        prior228(pool).await;
        for target in ["users", "accounts"] {
            if target == "accounts" {
                finalized(pool).await;
            }
            let mut tx = pool.begin().await.unwrap();
            sqlx::raw_sql("SET LOCAL search_path=pg_catalog,pg_temp; SET LOCAL jit=off")
                .execute(&mut *tx)
                .await
                .unwrap();
            edge(&mut tx, target).await;
            let original_state = state(&mut tx).await;
            let original_rows = rows(&mut tx).await;
            let original_meta = metadata(&mut tx).await;
            sqlx::raw_sql("SAVEPOINT audit_fault")
                .execute(&mut *tx)
                .await
                .unwrap();
            sqlx::raw_sql(sqlx::AssertSqlSafe(mutation(target)))
                .execute(&mut *tx)
                .await
                .unwrap();
            let fault_rows = rows(&mut tx).await;
            let fault_meta = metadata(&mut tx).await;
            assert!(fault_meta != original_meta, "fault was not installed");
            assert!(
                state(&mut tx).await == "account_native.profile_mismatch",
                "serving classifier accepted audit drift"
            );
            let credentials: String = sqlx::query_scalar(include_str!(
                "../../src/account_credential_custody_state.sql"
            ))
            .fetch_one(&mut *tx)
            .await
            .unwrap();
            assert!(
                credentials == "account_native.profile_mismatch",
                "credential classifier accepted audit drift"
            );
            sqlx::raw_sql("SAVEPOINT audit_attempt")
                .execute(&mut *tx)
                .await
                .unwrap();
            let result = compose(&mut tx).await;
            sqlx::raw_sql("ROLLBACK TO SAVEPOINT audit_attempt")
                .execute(&mut *tx)
                .await
                .unwrap();
            let error = result.expect_err("actual operator accepted audit drift");
            let database = error
                .as_database_error()
                .expect("SQL refusal, not transport error");
            assert!(
                database.code().as_deref() == Some("P0001")
                    && database.message() == "account_native.profile_mismatch",
                "wrong operator refusal boundary"
            );
            assert!(
                preservation_holds(&fault_rows, &rows(&mut tx).await),
                "refusal changed rows"
            );
            assert!(
                fault_meta == metadata(&mut tx).await,
                "refusal repaired or changed fault metadata"
            );
            sqlx::raw_sql("ROLLBACK TO SAVEPOINT audit_fault")
                .execute(&mut *tx)
                .await
                .unwrap();
            assert!(preservation_holds(&original_rows, &rows(&mut tx).await));
            assert!(original_meta == metadata(&mut tx).await);
            assert!(
                state(&mut tx).await == original_state,
                "positive control did not recover"
            );
            edge(&mut tx, target).await;
            tx.rollback().await.unwrap();
        }
    }

    macro_rules! refusal {
        ($name:ident, $mutation:expr) => {
            #[sqlx::test(migrations = false)]
            async fn $name(pool: PgPool) {
                drift(&pool, $mutation).await;
            }
        };
    }

    refusal!(missing_actor_fk_refuses_without_repair, |_| {
        "ALTER TABLE public.audit_events DROP CONSTRAINT audit_events_actor_fkey".into()
    });
    refusal!(wrong_parent_refuses_without_repair, |_| {
        "CREATE TABLE public.audit_test_wrong_parent(id uuid PRIMARY KEY); INSERT INTO public.audit_test_wrong_parent SELECT id FROM public.accounts; ALTER TABLE public.audit_events DROP CONSTRAINT audit_events_actor_fkey, ADD CONSTRAINT audit_events_actor_fkey FOREIGN KEY(actor) REFERENCES public.audit_test_wrong_parent(id) ON UPDATE NO ACTION ON DELETE RESTRICT NOT DEFERRABLE".into()
    });
    refusal!(unvalidated_actor_fk_refuses_without_repair, |target| {
        replace_fk(
            target,
            "ON UPDATE NO ACTION ON DELETE RESTRICT NOT DEFERRABLE NOT VALID",
        )
    });
    refusal!(
        cascade_actor_fk_refuses_without_repair,
        |target| replace_fk(
            target,
            "ON UPDATE NO ACTION ON DELETE CASCADE NOT DEFERRABLE"
        )
    );
    refusal!(set_null_actor_fk_refuses_without_repair, |target| {
        replace_fk(
            target,
            "ON UPDATE NO ACTION ON DELETE SET NULL NOT DEFERRABLE",
        )
    });
    refusal!(deferred_actor_fk_refuses_without_repair, |target| {
        replace_fk(
            target,
            "ON UPDATE NO ACTION ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED",
        )
    });
    refusal!(composite_legacy_actor_fk_refuses_without_repair, |_| {
        "ALTER TABLE public.audit_events ADD CONSTRAINT audit_test_legacy_composite FOREIGN KEY(actor,org_id) REFERENCES public.users(id,org_id) ON DELETE RESTRICT".into()
    });
    refusal!(disabled_internal_ri_refuses_without_repair, |_| {
        r#"DO $fault$ DECLARE trigger_name name; BEGIN SELECT t.tgname INTO STRICT trigger_name FROM pg_trigger t JOIN pg_constraint k ON k.oid=t.tgconstraint JOIN pg_proc p ON p.oid=t.tgfoid WHERE k.conrelid='public.audit_events'::regclass AND k.conname='audit_events_actor_fkey' AND t.tgrelid=k.conrelid AND p.proname='RI_FKey_check_ins'; EXECUTE format('ALTER TABLE public.audit_events DISABLE TRIGGER %I',trigger_name); END $fault$"#.into()
    });
    refusal!(immutable_update_trigger_removed_refuses, |_| {
        "DROP TRIGGER trg_audit_events_no_update ON public.audit_events".into()
    });
    refusal!(immutable_delete_trigger_removed_refuses, |_| {
        "DROP TRIGGER trg_audit_events_no_delete ON public.audit_events".into()
    });
    refusal!(immutable_body_replacement_refuses, |_| {
        "CREATE OR REPLACE FUNCTION public.audit_events_immutable() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NEW; END $$".into()
    });
    refusal!(widened_audit_table_acl_refuses, |_| {
        "GRANT UPDATE ON public.audit_events TO console_auth_startup".into()
    });
    refusal!(widened_audit_column_acl_refuses, |_| {
        "GRANT SELECT(actor) ON public.audit_events TO PUBLIC".into()
    });
    refusal!(disabled_audit_rls_refuses, |_| {
        "ALTER TABLE public.audit_events DISABLE ROW LEVEL SECURITY".into()
    });

    #[sqlx::test(migrations = false)]
    async fn failure_after_actual_actor_alter_restores_exact_predecessor(pool: PgPool) {
        prior228(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        let original_rows = rows(&mut tx).await;
        let original_meta = metadata(&mut tx).await;
        edge(&mut tx, "users").await;
        sqlx::raw_sql("SAVEPOINT test_fault_fixture")
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::raw_sql(r#"
            CREATE FUNCTION public.audit_test_after_conversion() RETURNS event_trigger LANGUAGE plpgsql AS $$
            BEGIN
              IF EXISTS(SELECT 1 FROM pg_constraint WHERE conrelid='public.audit_events'::regclass AND conname='audit_events_actor_fkey' AND confrelid='public.accounts'::regclass AND convalidated AND conenforced) THEN
                RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='AUDIT_TEST_AFTER_ACTUAL_CONVERSION';
              END IF;
            END $$;
            CREATE EVENT TRIGGER audit_test_after_conversion ON ddl_command_end WHEN TAG IN ('ALTER TABLE') EXECUTE FUNCTION public.audit_test_after_conversion()
        "#).execute(&mut *tx).await.unwrap();
        let fault_meta = metadata(&mut tx).await;
        sqlx::raw_sql("SAVEPOINT actual_transition")
            .execute(&mut *tx)
            .await
            .unwrap();
        let result = compose(&mut tx).await;
        sqlx::raw_sql("ROLLBACK TO SAVEPOINT actual_transition")
            .execute(&mut *tx)
            .await
            .unwrap();
        let error = result.expect_err("post-ALTER fault did not execute");
        let database = error
            .as_database_error()
            .expect("real PostgreSQL fault required");
        assert!(
            database.code().as_deref() == Some("P0001")
                && database.message() == "AUDIT_TEST_AFTER_ACTUAL_CONVERSION",
            "failure must prove execution reached actual actor conversion"
        );
        edge(&mut tx, "users").await;
        assert!(preservation_holds(&original_rows, &rows(&mut tx).await));
        assert!(
            fault_meta == metadata(&mut tx).await,
            "failed transition left metadata changes"
        );
        sqlx::raw_sql("ROLLBACK TO SAVEPOINT test_fault_fixture")
            .execute(&mut *tx)
            .await
            .unwrap();
        assert!(original_meta == metadata(&mut tx).await);
        tx.commit().await.unwrap();
        finalized(&pool).await;
        let mut check = pool.acquire().await.unwrap();
        assert!(preservation_holds(&original_rows, &rows(&mut check).await));
    }

    async fn insert_writer(connection: &mut PgConnection, actor: Option<uuid::Uuid>) {
        sqlx::query("INSERT INTO public.audit_events(id,actor,action,target_type,target_id,trace_id,span_id,occurred_at) VALUES($1,$2,'audit.transition_contention','test_only','held_uncommitted',repeat('a',32),repeat('b',16),'2026-09-17T01:02:03Z')")
            .bind(uuid::Uuid::new_v4()).bind(actor).execute(connection).await.unwrap();
    }

    #[sqlx::test(migrations = false)]
    async fn deleting_legacy_user_preserves_account_root_and_exact_audit_attribution(pool: PgPool) {
        prior228(&pool).await;
        finalized(&pool).await;
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let account = uuid::Uuid::new_v4();
        let company = *console_kernel_core::OrgId::knl().as_uuid();
        let mut tx = runtime.begin().await.unwrap();
        sqlx::query("SELECT set_config('app.current_org',$1,true)")
            .bind(company.to_string())
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::query("INSERT INTO public.users(id,display_name,roles,org_id,created_at) VALUES($1,'AUDIT RETENTION TEST ONLY',ARRAY['MECHANIC']::text[],$2,'2026-09-17T01:02:03Z')")
            .bind(account).bind(company).execute(&mut *tx).await.unwrap();
        insert_writer(&mut tx, Some(account)).await;
        tx.commit().await.unwrap();
        let before: Value =
            sqlx::query_scalar("SELECT to_jsonb(a) FROM public.audit_events a WHERE actor=$1")
                .bind(account)
                .fetch_one(&pool)
                .await
                .unwrap();
        let mut tx = runtime.begin().await.unwrap();
        sqlx::query("SELECT set_config('app.current_org',$1,true)")
            .bind(company.to_string())
            .execute(&mut *tx)
            .await
            .unwrap();
        let deleted = sqlx::query("DELETE FROM public.users WHERE id=$1")
            .bind(account)
            .execute(&mut *tx)
            .await
            .unwrap()
            .rows_affected();
        assert!(
            deleted == 1,
            "only the disposable legacy profile was deleted"
        );
        tx.commit().await.unwrap();
        let retained:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.accounts WHERE id=$1) AND NOT EXISTS(SELECT 1 FROM public.users WHERE id=$1) AND NOT EXISTS(SELECT 1 FROM public.account_security WHERE account_id=$1)").bind(account).fetch_one(&pool).await.unwrap();
        assert!(
            retained,
            "persistent root survives without fabricating authentication authority"
        );
        let after: Value =
            sqlx::query_scalar("SELECT to_jsonb(a) FROM public.audit_events a WHERE actor=$1")
                .bind(account)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(
            before == after,
            "legacy profile deletion rewrote immutable audit attribution"
        );
        let mut owner = pool.begin().await.unwrap();
        let result = sqlx::query("DELETE FROM public.accounts WHERE id=$1")
            .bind(account)
            .execute(&mut *owner)
            .await;
        owner.rollback().await.unwrap();
        let error = result.expect_err("persistent Account root was deleted");
        let database = error
            .as_database_error()
            .expect("actual Account immutability refusal");
        assert!(
            database.code().as_deref() == Some("P0001")
                && database.message() == "account_roots.immutable"
        );
        let after: Value =
            sqlx::query_scalar("SELECT to_jsonb(a) FROM public.audit_events a WHERE actor=$1")
                .bind(account)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(before == after);
        runtime.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn null_actor_writer_has_observed_users_blocker_and_bounded_lock_timeout(pool: PgPool) {
        prior228(&pool).await;
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let mut check = pool.acquire().await.unwrap();
        let before_rows = rows(&mut check).await;
        let before_meta = metadata(&mut check).await;
        let mut writer = runtime.begin().await.unwrap();
        insert_writer(&mut writer, None).await;
        let holding: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *writer)
            .await
            .unwrap();
        // The genuine NULL-actor insert was observed holding users AccessShare.
        // Witness that root lock; do not claim this isolates audit NOWAIT.
        let held: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_locks WHERE pid=$1 AND relation='public.audit_events'::regclass AND mode='RowExclusiveLock' AND granted)")
            .bind(holding).fetch_one(&mut *check).await.unwrap();
        assert!(held, "actual audit writer lock witness required");
        let mut operator = PgConnection::connect_with(&pool.connect_options())
            .await
            .unwrap();
        sqlx::raw_sql("SET application_name='audit-account-transition-lock-test'")
            .execute(&mut operator)
            .await
            .unwrap();
        let waiting: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut operator)
            .await
            .unwrap();
        let mut task = tokio::spawn(async move {
            let mut tx = operator.begin().await.unwrap();
            let result = compose(&mut tx).await;
            tx.rollback().await.unwrap();
            result
        });
        let witnessed=tokio::time::timeout(Duration::from_secs(4),async {
            loop {
                let blocked:bool=sqlx::query_scalar("SELECT $2=ANY(pg_blocking_pids($1)) AND EXISTS(SELECT 1 FROM pg_locks WHERE pid=$1 AND relation='public.users'::regclass AND mode='AccessExclusiveLock' AND NOT granted) AND EXISTS(SELECT 1 FROM pg_locks WHERE pid=$2 AND relation='public.users'::regclass AND mode='AccessShareLock' AND granted)")
                    .bind(waiting).bind(holding).fetch_one(&mut *check).await.unwrap();
                if blocked { break; }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }).await.is_ok();
        let outcome = tokio::time::timeout(Duration::from_secs(12), &mut task).await;
        if outcome.is_err() {
            let _: bool = sqlx::query_scalar("SELECT pg_cancel_backend($1)")
                .bind(waiting)
                .fetch_one(&mut *check)
                .await
                .unwrap();
            let _ = tokio::time::timeout(Duration::from_secs(5), &mut task).await;
            if !task.is_finished() {
                task.abort();
            }
        }
        writer.rollback().await.unwrap();
        runtime.close().await;
        assert!(witnessed, "exact users-lock blocker not observed");
        let result = outcome
            .expect("actual finalizer exceeded its finite lock budget")
            .expect("operator task failed");
        let error = result.expect_err("NULL writer root lock was not respected");
        let database = error
            .as_database_error()
            .expect("PostgreSQL lock timeout required");
        assert!(
            database.code().as_deref() == Some("55P03")
                && database.message().contains("lock timeout"),
            "expected actual finalizer lock_timeout"
        );
        assert!(preservation_holds(&before_rows, &rows(&mut check).await));
        assert!(before_meta == metadata(&mut check).await);
        edge(&mut check, "users").await;
        tokio::time::timeout(Duration::from_secs(20), finalized(&pool))
            .await
            .expect("bounded clean retry after NULL writer rollback");
        assert!(preservation_holds(&before_rows, &rows(&mut check).await));
    }

    #[sqlx::test(migrations = false)]
    async fn audit_only_relation_lock_causes_nowait_refusal_and_clean_retry(pool: PgPool) {
        prior228(&pool).await;
        let mut check = pool.acquire().await.unwrap();
        let before_rows = rows(&mut check).await;
        let before_meta = metadata(&mut check).await;
        // Deliberate owner-held lock fixture, not a business-writer surrogate.
        // Leave every real trigger and production grant unchanged.
        let mut holder = pool.begin().await.unwrap();
        sqlx::raw_sql("LOCK TABLE ONLY public.audit_events IN ROW EXCLUSIVE MODE")
            .execute(&mut *holder)
            .await
            .unwrap();
        let holding: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *holder)
            .await
            .unwrap();
        let isolated: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM pg_locks WHERE pid=$1 AND relation='public.audit_events'::regclass AND mode='RowExclusiveLock' AND granted) AND NOT EXISTS(SELECT 1 FROM pg_locks WHERE pid=$1 AND granted AND relation IN (SELECT to_regclass('public.'||name) FROM unnest($2::text[]) AS names(name) WHERE name<>'audit_events'))"
        ).bind(holding).bind(super::TABLES).fetch_one(&mut *check).await.unwrap();
        assert!(
            isolated,
            "audit-only holder must not lock earlier finalizer relations"
        );
        let mut attempt = pool.begin().await.unwrap();
        let outcome = tokio::time::timeout(Duration::from_secs(20), compose(&mut attempt)).await;
        // Roll back before interpreting outcomes, including timeout, so no
        // attempt or fixture lock survives into preservation checks or retry.
        attempt.rollback().await.unwrap();
        holder.rollback().await.unwrap();
        let result = outcome.expect("bounded audit-only NOWAIT admission");
        let error = result.expect_err("isolated audit relation lock was accepted");
        let database = error
            .as_database_error()
            .expect("PostgreSQL NOWAIT refusal required");
        assert!(
            database.code().as_deref() == Some("55P03")
                && database.message()
                    == "could not obtain lock on relation \"public.audit_events\"",
            "must refuse the isolated audit relation through NOWAIT, not lock_timeout"
        );
        assert!(preservation_holds(&before_rows, &rows(&mut check).await));
        assert!(before_meta == metadata(&mut check).await);
        edge(&mut check, "users").await;
        tokio::time::timeout(Duration::from_secs(20), finalized(&pool))
            .await
            .expect("bounded clean retry after audit-only lock release");
        assert!(preservation_holds(&before_rows, &rows(&mut check).await));
    }

    #[sqlx::test(migrations = false)]
    async fn legacy_actor_writer_has_observed_blocker_and_bounded_lock_timeout(pool: PgPool) {
        prior228(&pool).await;
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let mut check = pool.acquire().await.unwrap();
        let before_rows = rows(&mut check).await;
        let before_meta = metadata(&mut check).await;
        let actor: uuid::Uuid = sqlx::query_scalar(
            "SELECT actor FROM public.audit_events WHERE actor IS NOT NULL ORDER BY id LIMIT 1",
        )
        .fetch_one(&mut *check)
        .await
        .unwrap();
        let mut writer = runtime.begin().await.unwrap();
        insert_writer(&mut writer, Some(actor)).await;
        let holding: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *writer)
            .await
            .unwrap();
        let mut operator = PgConnection::connect_with(&pool.connect_options())
            .await
            .unwrap();
        sqlx::raw_sql("SET application_name='audit-account-transition-lock-test'")
            .execute(&mut operator)
            .await
            .unwrap();
        let waiting: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut operator)
            .await
            .unwrap();
        let mut task = tokio::spawn(async move {
            let mut tx = operator.begin().await.unwrap();
            let result = compose(&mut tx).await;
            tx.rollback().await.unwrap();
            result
        });
        let witnessed=tokio::time::timeout(Duration::from_secs(4),async {
            loop {
                let blocked:bool=sqlx::query_scalar("SELECT $2=ANY(pg_blocking_pids($1)) AND EXISTS(SELECT 1 FROM pg_locks WHERE pid=$1 AND relation='public.users'::regclass AND mode='AccessExclusiveLock' AND NOT granted) AND EXISTS(SELECT 1 FROM pg_locks WHERE pid=$2 AND relation='public.users'::regclass AND granted)")
                    .bind(waiting).bind(holding).fetch_one(&mut *check).await.unwrap();
                if blocked { break; }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }).await.is_ok();
        let outcome = tokio::time::timeout(Duration::from_secs(12), &mut task).await;
        if outcome.is_err() {
            let _: bool = sqlx::query_scalar("SELECT pg_cancel_backend($1)")
                .bind(waiting)
                .fetch_one(&mut *check)
                .await
                .unwrap();
            let _ = tokio::time::timeout(Duration::from_secs(5), &mut task).await;
            if !task.is_finished() {
                task.abort();
            }
        }
        writer.rollback().await.unwrap();
        runtime.close().await;
        assert!(witnessed, "exact users-lock blocker not observed");
        let result = outcome
            .expect("actual finalizer exceeded its finite lock budget")
            .expect("operator task failed");
        let error = result.expect_err("legacy writer lock was not respected");
        let database = error
            .as_database_error()
            .expect("PostgreSQL lock timeout required");
        assert!(
            database.code().as_deref() == Some("55P03")
                && database.message().contains("lock timeout"),
            "expected actual finalizer lock_timeout"
        );
        assert!(preservation_holds(&before_rows, &rows(&mut check).await));
        assert!(before_meta == metadata(&mut check).await);
        edge(&mut check, "users").await;
        finalized(&pool).await;
        assert!(preservation_holds(&before_rows, &rows(&mut check).await));
    }

    #[sqlx::test(migrations = false)]
    async fn preservation_oracle_rejects_missing_duplicated_and_changed_evidence(pool: PgPool) {
        prior228(&pool).await;
        let mut connection = pool.acquire().await.unwrap();
        let baseline = rows(&mut connection).await;
        assert!(
            preservation_holds(&baseline, &baseline),
            "identical complete positive control"
        );
        let audit = baseline
            .get("audit_events")
            .and_then(|v| v.get("rows"))
            .and_then(Value::as_array)
            .expect("audit row census");
        let null_index = audit
            .iter()
            .position(|r| r.get("actor") == Some(&Value::Null))
            .expect("actual NULL fixture");
        let actor_index = audit
            .iter()
            .position(|r| r.get("actor").is_some_and(|a| a.is_string()))
            .expect("actual nonnull fixture");
        let mut faults = Vec::new();
        let mut missing_table = baseline.clone();
        missing_table
            .as_object_mut()
            .unwrap()
            .remove("audit_events");
        faults.push(missing_table);
        let mut missing_columns = baseline.clone();
        missing_columns
            .get_mut("audit_events")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove("columns");
        faults.push(missing_columns);
        let mut missing_column = baseline.clone();
        missing_column["audit_events"]["columns"]
            .as_array_mut()
            .unwrap()
            .retain(|v| v.as_str() != Some("actor"));
        faults.push(missing_column);
        let mut missing_row = baseline.clone();
        missing_row["audit_events"]["rows"]
            .as_array_mut()
            .unwrap()
            .remove(actor_index);
        faults.push(missing_row);
        let mut duplicate = baseline.clone();
        duplicate["audit_events"]["rows"]
            .as_array_mut()
            .unwrap()
            .push(audit[actor_index].clone());
        faults.push(duplicate);
        let mut same_count = baseline.clone();
        same_count["audit_events"]["rows"][null_index] = audit[actor_index].clone();
        faults.push(same_count);
        let mut changed_actor = baseline.clone();
        changed_actor["audit_events"]["rows"][actor_index]["actor"] = json!(uuid::Uuid::nil());
        faults.push(changed_actor);
        let mut absent_null = baseline.clone();
        absent_null["audit_events"]["rows"][null_index]
            .as_object_mut()
            .unwrap()
            .remove("actor");
        faults.push(absent_null);
        let mut changed_snapshot = baseline.clone();
        changed_snapshot["audit_events"]["rows"][actor_index]["after_snap"] =
            json!({"test":"changed"});
        faults.push(changed_snapshot);
        let mut changed_id = baseline.clone();
        changed_id["audit_events"]["rows"][actor_index]["id"] = json!(uuid::Uuid::nil());
        faults.push(changed_id);
        let mut extra_column = baseline.clone();
        extra_column["audit_events"]["rows"][actor_index]
            .as_object_mut()
            .unwrap()
            .insert("omitted_column_control".into(), Value::Null);
        faults.push(extra_column);
        assert!(faults.len() == 11);
        for fault in faults {
            assert!(
                !preservation_holds(&baseline, &fault),
                "evidence oracle accepted corruption"
            );
        }
        let mut consistently_missing = baseline.clone();
        consistently_missing["audit_events"]["columns"]
            .as_array_mut()
            .unwrap()
            .retain(|v| v.as_str() != Some("actor"));
        for row in consistently_missing["audit_events"]["rows"]
            .as_array_mut()
            .unwrap()
        {
            row.as_object_mut().unwrap().remove("actor");
        }
        assert!(
            !preservation_holds(&consistently_missing, &consistently_missing),
            "paired missing audit column cannot certify its own omission"
        );
        assert!(
            !preservation_holds(&json!({}), &json!({})),
            "two incomplete observations cannot be a positive"
        );
    }
}
