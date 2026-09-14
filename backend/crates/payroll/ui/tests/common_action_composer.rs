//! Presentation acceptance for design30 screen-integration.txt and FieldAddress.
//! Proposed API, not an existing route or authority implementation. Inputs are
//! already policy-projected by the owner; this renderer must never fetch or
//! infer permission. Rust HTML assertions do not certify browser form behavior,
//! focus movement, CSRF verification, persistence, or server authorization.
#![cfg(feature = "ssr")]

use console_payroll_ui::common_action_composer::{
    AuthorizedComposer, AuthorizedField, Choice, ComposerOperation, ConflictRecovery, EditorValue,
    FieldAddress, FieldError, MutationContext, control_id, control_name, render,
};

fn address(parent: &str) -> FieldAddress {
    FieldAddress {
        field_id: "00000000-0000-4000-8000-000000000011".into(),
        parent_item_ids: vec![parent.into()],
        item_id: None,
    }
}
fn field(parent: &str, value: EditorValue) -> AuthorizedField {
    AuthorizedField {
        address: address(parent),
        label: "수량".into(),
        value,
        required: true,
    }
}
fn screen() -> AuthorizedComposer {
    AuthorizedComposer {
        title: "업무 신청".into(),
        fields: vec![field("00000000-0000-4000-8000-000000000021", EditorValue::Decimal("1.25".into()))],
        errors: vec![],
        mutation: Some(MutationContext {
            action_path: "/_ui/companies/00000000-0000-4000-8000-000000000001/actions/00000000-0000-4000-8000-000000000002".into(),
            csrf_token: "csrf-test-opaque".into(),
            command_id: "command-test-opaque".into(),
            expected_token: "expected-test-opaque".into(),
            draft_token: "draft-test-opaque".into(),
            operations: vec![ComposerOperation::Save, ComposerOperation::Submit],
        }),
        conflict: None,
    }
}

// These intentionally bounded oracles inspect generated start tags, not HTML
// parsing generally. Exact double-quoted attribute serialization is the local
// renderer contract. Real browser form-association checks remain required.
fn start_tags<'a>(html: &'a str, tag: &str) -> Vec<&'a str> {
    let prefix = format!("<{tag}");
    html.match_indices(&prefix)
        .filter_map(|(at, _)| {
            let tail = &html[at + prefix.len()..];
            if !tail.starts_with([' ', '>']) {
                return None;
            }
            let end = html[at..].find('>').expect("unterminated generated tag");
            Some(&html[at..=at + end])
        })
        .collect()
}
fn attr(tag: &str, name: &str) -> Option<String> {
    let needle = format!(" {name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')? + start;
    Some(tag[start..end].into())
}
fn one_named<'a>(html: &'a str, tag: &str, name: &str) -> &'a str {
    let found: Vec<_> = start_tags(html, tag)
        .into_iter()
        .filter(|t| attr(t, "name").as_deref() == Some(name))
        .collect();
    assert_eq!(
        found.len(),
        1,
        "expected exactly one {tag} named {name}: {html}"
    );
    found[0]
}
fn one_id<'a>(html: &'a str, tag: &str, id: &str) -> &'a str {
    let found: Vec<_> = start_tags(html, tag)
        .into_iter()
        .filter(|t| attr(t, "id").as_deref() == Some(id))
        .collect();
    assert_eq!(found.len(), 1, "missing or duplicate {id}: {html}");
    found[0]
}

#[test]
fn no_js_composer_uses_one_post_form_and_distinct_explicit_intents() {
    let model = screen();
    let html = render(&model);
    let forms = start_tags(&html, "form");
    assert_eq!(forms.len(), 1, "{html}");
    assert_eq!(attr(forms[0], "method").as_deref(), Some("post"));
    assert_eq!(
        attr(forms[0], "action"),
        Some(model.mutation.as_ref().unwrap().action_path.clone())
    );
    let operations: Vec<_> = start_tags(&html, "button")
        .into_iter()
        .filter(|t| attr(t, "name").as_deref() == Some("operation"))
        .collect();
    assert_eq!(operations.len(), 2, "{html}");
    for intent in ["save", "submit"] {
        let matches: Vec<_> = operations
            .iter()
            .filter(|t| attr(t, "value").as_deref() == Some(intent))
            .collect();
        assert_eq!(matches.len(), 1, "missing explicit {intent}");
        assert_eq!(attr(matches[0], "type").as_deref(), Some("submit"));
    }
    for forbidden in [
        "formaction=",
        "formmethod=",
        "onclick=",
        "onsubmit=",
        "<script",
        "/_ui/pkg/",
    ] {
        assert!(
            !html.contains(forbidden),
            "no-JS composer introduced {forbidden}: {html}"
        );
    }
}

#[test]
fn exact_command_expected_draft_and_csrf_tokens_are_successful_hidden_controls() {
    let html = render(&screen());
    for (name, value) in [
        ("csrf_token", "csrf-test-opaque"),
        ("command_id", "command-test-opaque"),
        ("expected_token", "expected-test-opaque"),
        ("draft_token", "draft-test-opaque"),
    ] {
        let input = one_named(&html, "input", name);
        assert_eq!(attr(input, "type").as_deref(), Some("hidden"));
        assert_eq!(attr(input, "value").as_deref(), Some(value));
        assert!(
            !input.contains(" disabled"),
            "token cannot be omitted by form submission"
        );
    }
}

#[test]
fn incomplete_decimal_is_not_coerced_and_error_links_to_exact_field() {
    let mut model = screen();
    model.fields[0].value = EditorValue::Incomplete("1..2".into());
    model.errors.push(FieldError {
        address: model.fields[0].address.clone(),
        message: "숫자를 확인해 주세요".into(),
    });
    let html = render(&model);
    let id = control_id(&model.fields[0].address);
    let input = one_id(&html, "input", &id);
    assert_eq!(attr(input, "value").as_deref(), Some("1..2"));
    assert_ne!(
        attr(input, "type").as_deref(),
        Some("number"),
        "number controls can discard acknowledged invalid text"
    );
    assert_eq!(attr(input, "aria-invalid").as_deref(), Some("true"));
    let error_id = attr(input, "aria-describedby").expect("field error linkage");
    assert!(
        html.contains(&format!("id=\"{error_id}\"")),
        "dangling error ID"
    );
    assert!(html.contains("숫자를 확인해 주세요"));
    assert!(
        html.contains(&format!("href=\"#{id}\"")),
        "summary must link exact field"
    );
    assert!(
        html.contains("tabindex=\"-1\""),
        "error summary must accept focus"
    );
}

#[test]
fn unresolved_editor_can_save_but_cannot_submit_as_business_value() {
    let mut model = screen();
    model.fields[0].value = EditorValue::Incomplete("1..2".into());
    let html = render(&model);
    let buttons = start_tags(&html, "button");
    let save = buttons
        .iter()
        .find(|b| attr(b, "value").as_deref() == Some("save"))
        .expect("incomplete editor still needs save");
    assert!(!save.contains(" disabled"));
    assert!(
        save.contains(" formnovalidate"),
        "draft save must survive required/browser validation"
    );
    for submit in buttons
        .iter()
        .filter(|b| attr(b, "value").as_deref() == Some("submit"))
    {
        assert!(
            submit.contains(" disabled"),
            "incomplete editor must not offer business submission"
        );
    }
}

#[test]
fn repeated_item_addresses_survive_reorder_without_aliasing() {
    let mut model = screen();
    model.fields.push(field(
        "00000000-0000-4000-8000-000000000022",
        EditorValue::Decimal("2.50".into()),
    ));
    let before = render(&model);
    let addresses: Vec<(String, String)> = model
        .fields
        .iter()
        .map(|f| (control_id(&f.address), control_name(&f.address)))
        .collect();
    assert_ne!(
        addresses[0], addresses[1],
        "same field in different rows cannot alias"
    );
    model.fields.reverse();
    let after = render(&model);
    for entry in &addresses {
        let id: &str = entry.0.as_str();
        let name: &String = &entry.1;
        assert_eq!(
            attr(one_id(&before, "input", &id), "name"),
            Some(name.clone())
        );
        assert_eq!(attr(one_id(&after, "input", &id), "name"), Some(name.clone()));
    }
    let mut nested = model.fields[0].address.clone();
    nested
        .parent_item_ids
        .push("00000000-0000-4000-8000-000000000023".into());
    assert_ne!(
        control_name(&nested),
        control_name(&model.fields[0].address)
    );
    nested.item_id = Some("00000000-0000-4000-8000-000000000024".into());
    assert_ne!(control_id(&nested), control_id(&model.fields[0].address));
}

#[test]
fn only_projected_fields_and_choices_exist_in_document() {
    let mut model = screen();
    model.fields[0].value = EditorValue::Enum {
        selected: Some("temporary".into()),
        choices: vec![
            Choice {
                value: "temporary".into(),
                label: "임시 대행".into(),
            },
            Choice {
                value: "permanent".into(),
                label: "영구 이전".into(),
            },
        ],
    };
    let html = render(&model);
    let options = start_tags(&html, "option");
    assert_eq!(
        options.len(),
        2,
        "renderer must not invent or fetch choices"
    );
    assert!(
        options
            .iter()
            .any(|o| attr(o, "value").as_deref() == Some("temporary") && o.contains(" selected"))
    );
    assert!(html.contains("임시 대행") && html.contains("영구 이전"));
    assert!(!html.contains("PARTIAL_SHARE"));
    assert_eq!(start_tags(&html, "select").len(), 1);
    assert!(!html.contains("type=\"password\""));
    // Compare exact allowed control-name set: hidden/projection payloads cannot
    // quietly append business fields to this specific authorized projection.
    let names: Vec<_> = start_tags(&html, "input")
        .iter()
        .filter_map(|t| attr(t, "name"))
        .collect();
    assert_eq!(
        names.len(),
        4,
        "only the four mutation tokens are inputs: {html}"
    );
}

#[test]
fn escaped_labels_values_errors_and_tokens_never_create_markup() {
    let mut model = screen();
    let hostile = "\"><script>globalThis.leak=1</script><img src=x onerror=alert(1)>";
    model.title = hostile.into();
    model.fields[0].label = hostile.into();
    model.fields[0].value = EditorValue::Text(hostile.into());
    model.errors.push(FieldError {
        address: model.fields[0].address.clone(),
        message: hostile.into(),
    });
    model.mutation.as_mut().unwrap().csrf_token = hostile.into();
    let html = render(&model);
    assert!(
        html.contains("&lt;script&gt;"),
        "hostile literal text must remain visible as text"
    );
    assert!(start_tags(&html, "script").is_empty());
    assert!(start_tags(&html, "img").is_empty());
    assert_eq!(start_tags(&html, "form").len(), 1);
    assert!(
        attr(one_named(&html, "input", "csrf_token"), "value")
            .unwrap()
            .contains("&lt;script&gt;")
    );
    assert!(
        attr(
            one_id(&html, "input", &control_id(&model.fields[0].address)),
            "value"
        )
        .unwrap()
        .contains("&lt;script&gt;")
    );
}

#[test]
fn read_only_projection_has_no_mutation_controls_or_tokens() {
    let mut model = screen();
    model.mutation = None;
    model.fields[0].value = EditorValue::Text("현재 허용된 값".into());
    let html = render(&model);
    assert!(html.contains("현재 허용된 값"));
    for tag in ["form", "input", "select", "textarea", "button"] {
        assert!(
            start_tags(&html, tag).is_empty(),
            "read-only emitted {tag}: {html}"
        );
    }
    for token in [
        "csrf-test-opaque",
        "command-test-opaque",
        "expected-test-opaque",
        "draft-test-opaque",
    ] {
        assert!(!html.contains(token));
    }
}

#[test]
fn save_only_projection_never_offers_submit() {
    let mut model = screen();
    model.mutation.as_mut().unwrap().operations = vec![ComposerOperation::Save];
    let html = render(&model);
    let operations: Vec<_> = start_tags(&html, "button")
        .into_iter()
        .filter(|t| attr(t, "name").as_deref() == Some("operation"))
        .collect();
    assert_eq!(operations.len(), 1);
    assert_eq!(attr(operations[0], "value").as_deref(), Some("save"));
}

#[test]
fn conflict_preserves_editor_and_original_tokens_with_authorized_recovery() {
    let mut model = screen();
    model.fields[0].value = EditorValue::Incomplete("1..2".into());
    model.conflict = Some(ConflictRecovery {
        message: "다른 변경이 있어 확인이 필요합니다".into(),
        authorized_reload_path: "/_ui/companies/known/drafts/original".into(),
    });
    let html = render(&model);
    assert!(html.contains("다른 변경이 있어 확인이 필요합니다"));
    assert!(
        start_tags(&html, "a")
            .iter()
            .any(|a| attr(a, "href").as_deref() == Some("/_ui/companies/known/drafts/original"))
    );
    assert_eq!(
        attr(
            one_id(&html, "input", &control_id(&model.fields[0].address)),
            "value"
        )
        .as_deref(),
        Some("1..2")
    );
    assert_eq!(
        attr(one_named(&html, "input", "draft_token"), "value").as_deref(),
        Some("draft-test-opaque")
    );
    assert_eq!(
        attr(one_named(&html, "input", "expected_token"), "value").as_deref(),
        Some("expected-test-opaque")
    );
    assert!(!html.contains("http-equiv=\"refresh\""));
    assert!(
        !html.contains("overwrite"),
        "renderer must not invent unconditional overwrite"
    );
}

#[test]
fn local_tag_oracle_rejects_missing_duplicate_and_prefix_controls() {
    assert!(
        std::panic::catch_unwind(|| one_named("<input name=\"other\">", "input", "csrf_token"))
            .is_err()
    );
    assert!(
        std::panic::catch_unwind(|| one_named(
            "<input name=\"csrf_token\"><input name=\"csrf_token\">",
            "input",
            "csrf_token"
        ))
        .is_err()
    );
    assert!(start_tags("<inputter name=\"csrf_token\">", "input").is_empty());
    assert_eq!(
        attr("<input data-name=\"wrong\" name=\"right\">", "name").as_deref(),
        Some("right")
    );
}

#[test]
fn unsafe_action_paths_never_create_mutation_or_serialize_tokens() {
    for path in [
        "https://outside.example/collect",
        "//outside.example/collect",
        "javascript:alert(1)",
        "/\\outside.example/collect",
        "\n//outside.example/collect",
        "relative/collect",
    ] {
        let mut model = screen();
        model.mutation.as_mut().unwrap().action_path = path.into();
        let html = render(&model);
        assert!(
            start_tags(&html, "form").is_empty(),
            "unsafe action became form: {path:?}"
        );
        for token in [
            "csrf-test-opaque",
            "command-test-opaque",
            "expected-test-opaque",
            "draft-test-opaque",
        ] {
            assert!(
                !html.contains(token),
                "invalid action leaked token: {path:?}"
            );
        }
    }
}

#[test]
fn unsafe_recovery_paths_never_become_active_links() {
    for path in [
        "https://outside.example/collect",
        "//outside.example/collect",
        "javascript:alert(1)",
        "/\\outside.example/collect",
        "\n//outside.example/collect",
        "relative/collect",
    ] {
        let mut model = screen();
        model.conflict = Some(ConflictRecovery {
            message: "확인이 필요합니다".into(),
            authorized_reload_path: path.into(),
        });
        let html = render(&model);
        assert!(
            start_tags(&html, "a").is_empty(),
            "unsafe recovery became link: {path:?}"
        );
        assert!(
            html.contains("확인이 필요합니다"),
            "preserve conflict explanation"
        );
        assert_eq!(
            attr(one_named(&html, "input", "draft_token"), "value").as_deref(),
            Some("draft-test-opaque")
        );
    }
}

#[test]
fn address_encoding_is_injective_for_sequence_and_optional_item_boundaries() {
    let addresses = [
        FieldAddress {
            field_id: "field".into(),
            parent_item_ids: vec!["a/b".into(), "c".into()],
            item_id: None,
        },
        FieldAddress {
            field_id: "field".into(),
            parent_item_ids: vec!["a".into(), "b/c".into()],
            item_id: None,
        },
        FieldAddress {
            field_id: "field".into(),
            parent_item_ids: vec!["a".into(), "b".into()],
            item_id: Some("c".into()),
        },
        FieldAddress {
            field_id: "field".into(),
            parent_item_ids: vec!["a".into(), "b".into(), "c".into()],
            item_id: None,
        },
        FieldAddress {
            field_id: "field".into(),
            parent_item_ids: vec![],
            item_id: None,
        },
        FieldAddress {
            field_id: "field".into(),
            parent_item_ids: vec![],
            item_id: Some(String::new()),
        },
        FieldAddress {
            field_id: "field/a".into(),
            parent_item_ids: vec!["b".into()],
            item_id: None,
        },
        FieldAddress {
            field_id: "field".into(),
            parent_item_ids: vec!["a/b".into()],
            item_id: None,
        },
    ];
    let ids: std::collections::BTreeSet<_> = addresses.iter().map(control_id).collect();
    let names: std::collections::BTreeSet<_> = addresses.iter().map(control_name).collect();
    assert_eq!(ids.len(), addresses.len(), "DOM addresses collide");
    assert_eq!(names.len(), addresses.len(), "POST field addresses collide");
    for id in ids {
        assert!(
            id.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'),
            "DOM fragment identifier needs safe serialization: {id}"
        );
    }
}
