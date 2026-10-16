//! Pure compatibility oracles only; not actual SQL/reader integration evidence.
use super::*;
use serde_json::json;
fn actor() -> Uuid {
    Uuid::from_u128(1)
}
fn object() -> Uuid {
    Uuid::from_u128(2)
}
fn expectation(target: Option<DispatchTarget>) -> LegacyReceiptExpectation<'static> {
    LegacyReceiptExpectation {
        actor: actor(),
        target,
        action_key: "execute",
        object_type_id: object(),
    }
}
fn binding(target: DispatchTarget) -> LegacyReceiptBinding<'static> {
    LegacyReceiptBinding {
        actor: Some(actor()),
        owner: ReceiptOwner::Canonical(target.object()).as_str(),
        target: Some(target.as_str()),
        action_key: Some("execute"),
        object_type_id: Some(object()),
    }
}
#[test]
fn absent_both_and_exact_user_json_null_are_the_only_legacy_actor_shapes() {
    assert!(legacy_user_receipt_shape(None, None));
    assert!(legacy_user_receipt_shape(
        Some(&json!("USER")),
        Some(&Value::Null)
    ));
    let atoms = [
        json!("USER"),
        json!("ACCOUNT"),
        json!("user"),
        json!("UNKNOWN"),
        Value::Null,
        json!(actor()),
        json!(1),
        json!({}),
        json!([]),
    ];
    let options: Vec<_> = std::iter::once(None)
        .chain(atoms.iter().map(Some))
        .collect();
    let mut accepted = 0;
    for kind in &options {
        for account in &options {
            let want = kind.is_none() && account.is_none()
                || kind.is_some_and(|v| v == &json!("USER")) && account.is_some_and(Value::is_null);
            let got = legacy_user_receipt_shape(*kind, *account);
            assert_eq!(got, want, "kind={kind:?}, account={account:?}");
            accepted += usize::from(got);
        }
    }
    assert_eq!(accepted, 2);
}
#[test]
fn every_dispatch_target_accepts_its_exact_attribution_and_historical_default() {
    assert_eq!(DispatchTarget::ALL.len(), 13);
    for &target in DispatchTarget::ALL {
        let expected = expectation(Some(target));
        let mut stored = binding(target);
        assert!(legacy_receipt_binding(&stored, &expected));
        assert_eq!(
            legacy_receipt_result_target(Some(target), &json!({"target":target.as_str()})),
            Ok(())
        );
        stored.owner = "ontology.action";
        stored.target = None;
        assert!(legacy_receipt_binding(&stored, &expected));
        stored.action_key = None;
        stored.object_type_id = None;
        assert!(legacy_receipt_binding(&stored, &expected));
        assert_eq!(
            legacy_receipt_result_target(Some(target), &json!({"target":target.as_str()})),
            Ok(())
        );
    }
}
#[test]
fn explicit_wrong_owner_target_and_partial_default_never_use_legacy_fallback() {
    for &target in DispatchTarget::ALL {
        let expected = expectation(Some(target));
        for owner in ReceiptOwner::ALL {
            let mut stored = binding(target);
            stored.owner = owner.as_str();
            assert_eq!(
                legacy_receipt_binding(&stored, &expected),
                *owner == ReceiptOwner::Canonical(target.object())
            );
        }
        for &other in DispatchTarget::ALL {
            let mut stored = binding(target);
            stored.target = Some(other.as_str());
            assert_eq!(legacy_receipt_binding(&stored, &expected), other == target);
        }
        let mut stored = binding(target);
        stored.target = None;
        assert!(!legacy_receipt_binding(&stored, &expected));
        stored.owner = "unknown";
        assert!(!legacy_receipt_binding(&stored, &expected));
        stored.owner = "ontology.action";
        stored.target = Some(target.as_str());
        assert!(!legacy_receipt_binding(&stored, &expected));
    }
}
#[test]
fn actor_and_present_wrapper_bindings_are_exact_but_legacy_nulls_remain_compatible() {
    let target = DispatchTarget::PeopleCreatePerson;
    let expected = expectation(Some(target));
    for actor in [None, Some(Uuid::nil()), Some(Uuid::from_u128(99))] {
        let mut stored = binding(target);
        stored.actor = actor;
        assert!(!legacy_receipt_binding(&stored, &expected));
    }
    for key in ["", "Execute", "other"] {
        let mut stored = binding(target);
        stored.action_key = Some(key);
        assert!(!legacy_receipt_binding(&stored, &expected));
    }
    for id in [Uuid::nil(), Uuid::from_u128(99)] {
        let mut stored = binding(target);
        stored.object_type_id = Some(id);
        assert!(!legacy_receipt_binding(&stored, &expected));
    }
    for (key, id) in [
        (None, None),
        (Some("execute"), None),
        (None, Some(object())),
    ] {
        let mut stored = binding(target);
        stored.action_key = key;
        stored.object_type_id = id;
        assert!(legacy_receipt_binding(&stored, &expected));
    }
}
#[test]
fn historical_default_does_not_excuse_wrong_or_unreadable_result_target() {
    for &target in DispatchTarget::ALL {
        for &other in DispatchTarget::ALL {
            assert_eq!(
                legacy_receipt_result_target(Some(target), &json!({"target":other.as_str()})),
                if other == target {
                    Ok(())
                } else {
                    Err(LegacyReceiptTargetError::Conflict)
                }
            );
        }
        for result in [
            json!({}),
            json!({"target":null}),
            json!({"target":1}),
            json!({"target":"unknown"}),
            json!([]),
            Value::Null,
        ] {
            assert_eq!(
                legacy_receipt_result_target(Some(target), &result),
                Err(LegacyReceiptTargetError::Unreadable)
            );
        }
    }
}
#[test]
fn instance_replay_rejects_every_canonical_owner_or_result_target() {
    let expected = expectation(None);
    let stored = LegacyReceiptBinding {
        actor: Some(actor()),
        owner: "ontology.action",
        target: None,
        action_key: None,
        object_type_id: None,
    };
    assert!(legacy_receipt_binding(&stored, &expected));
    assert_eq!(
        legacy_receipt_result_target(None, &json!({"instance":{},"command_id":actor()})),
        Ok(())
    );
    for &target in DispatchTarget::ALL {
        assert!(!legacy_receipt_binding(&binding(target), &expected));
        assert_eq!(
            legacy_receipt_result_target(None, &json!({"target":target.as_str()})),
            Err(LegacyReceiptTargetError::Conflict)
        );
    }
    assert_eq!(
        legacy_receipt_result_target(None, &json!({"target":null})),
        Err(LegacyReceiptTargetError::Conflict)
    );
}
