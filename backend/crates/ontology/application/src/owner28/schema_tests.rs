//! Fixed-registry adapter tests; plain schemas do not carry runtime authority.
use super::*;
#[test]
fn all_fifty_five_registered_input_validators_compile() {
    let schemas = registered().unwrap();
    assert_eq!(schemas.bindings.len(), 55);
    let manifest: Value =
        serde_json::from_str(include_str!("../owner28_schemas/owner-bindings.json")).unwrap();
    for binding in manifest["bindings"].as_array().unwrap() {
        assert!(
            schemas
                .bindings
                .contains_key(binding["key"].as_str().unwrap())
        );
    }
}
#[test]
fn definition_resource_rewrite_preserves_ref_sibling_constraints() {
    let original = serde_json::json!({
        "$defs":{"Scalar":{"type":"string","minLength":2},"Bounded":{"$ref":"#/$defs/Scalar","maxLength":3}},
        "$ref":"#/$defs/Bounded"
    });
    let reference = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(&original)
        .unwrap();
    let document = "contract/example.schema.json";
    let mut resources = BTreeMap::new();
    for (name, node) in original["$defs"].as_object().unwrap() {
        let mut node = node.clone();
        rewrite_references(&mut node, document).unwrap();
        resources.insert(
            definition_uri(document, &format!("#/$defs/{name}")).unwrap(),
            node,
        );
    }
    let indexed = compile(
        &definition_uri(document, "#/$defs/Bounded").unwrap(),
        &resources,
    )
    .unwrap();
    for value in [
        Value::Null,
        serde_json::json!(0),
        serde_json::json!(""),
        serde_json::json!("a"),
        serde_json::json!("ab"),
        serde_json::json!("abc"),
        serde_json::json!("abcd"),
    ] {
        assert_eq!(indexed.is_valid(&value), reference.is_valid(&value));
    }
    assert!(indexed.is_valid(&serde_json::json!("ab")));
    assert!(!indexed.is_valid(&serde_json::json!("abcd")));
}
#[test]
fn missing_reachable_schema_fails_and_decimal_bounds_are_exact() {
    assert!(
        compile(
            &format!("{BASE}missing.schema.json/definitions/X"),
            &BTreeMap::new()
        )
        .is_err()
    );
    let root_uri = format!("{BASE}test.schema.json/definitions/Root");
    let absent_child = format!("{BASE}test.schema.json/definitions/Missing");
    let missing_dependency =
        BTreeMap::from([(root_uri.clone(), serde_json::json!({"$ref":absent_child}))]);
    assert!(compile(&root_uri, &missing_dependency).is_err());
    let uri = format!("{BASE}test.schema.json/definitions/Counter");
    let resources = BTreeMap::from([(
        uri.clone(),
        serde_json::json!({"type":"string","pattern":"^[1-9][0-9]*$","x-decimal-min":2,"x-decimal-max":9223372036854775807i64}),
    )]);
    let validator = compile(&uri, &resources).unwrap();
    for valid in ["2", "9223372036854775807"] {
        assert!(validator.is_valid(&serde_json::json!(valid)));
    }
    for invalid in ["1", "0", "-1", "02", "9223372036854775808"] {
        assert!(!validator.is_valid(&serde_json::json!(invalid)));
    }
    assert!(!validator.is_valid(&serde_json::json!(2)));
}
