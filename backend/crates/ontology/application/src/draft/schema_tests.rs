//! Structural compiler boundary controls over one actual registered definition.
//! Added assertions are hostile inputs to the private compiler, not registration
//! proposals, alternate product schemas, or current-authority evidence.
use super::*;
use serde_json::json;

const TAX_SUPPORT: &str = "https://console.invalid/owner28/2026-09-12-source-payload-contract-19/source-types.schema.json/definitions/TaxSupport";

#[test]
fn conflicting_registered_reference_sibling_refuses() {
    let actual = fixed::draft_resource(TAX_SUPPORT).unwrap();
    assert_eq!(actual["type"], "object");
    let supported = json!({"$ref": TAX_SUPPORT});
    let (shape, _) = head(&supported).unwrap();
    assert_eq!(shape, *actual);
    assert!(matches!(
        compile(&supported, &Trie::default(), 0).unwrap().kind,
        Kind::Record(_)
    ));
    assert!(head(&json!({"$ref": TAX_SUPPORT, "type": "array"})).is_err());
}

#[test]
fn unsupported_registered_container_assertion_refuses() {
    let supported = json!({"$ref": TAX_SUPPORT});
    assert!(matches!(
        compile(&supported, &Trie::default(), 0).unwrap().kind,
        Kind::Record(_)
    ));
    // The editor compiler does not implement maxProperties. Silently dropping
    // this assertion would treat a different contract as the supported one.
    assert!(
        compile(
            &json!({"$ref": TAX_SUPPORT, "maxProperties": 1}),
            &Trie::default(),
            0
        )
        .is_err()
    );
}
