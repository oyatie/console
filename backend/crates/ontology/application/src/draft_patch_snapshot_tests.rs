//! Private test-only candidate for the existing ontology application owner.
//! Fixtures are plain schema/address facts, never current authority or DB evidence.
use crate::draft::{
    DraftCell as Cell, DraftEditScope, DraftItem as Item, DraftProvenance as Provenance,
    DraftSchema, DraftSnapshot, FieldAddress, FieldBinding, FieldValue, GovernedRevision,
    InputSchemaRef, apply_draft_patches, parse_draft_patches,
};
use serde_json::{Value, json};
use uuid::Uuid;

fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}
fn address(field: u128, parents: &[u128], item: Option<u128>) -> FieldAddress {
    FieldAddress {
        field_id: id(field),
        parent_item_ids: parents.iter().copied().map(id).collect(),
        item_id: item.map(id),
    }
}
fn wire_address(field: u128, parents: &[u128], item: Option<u128>) -> Value {
    json!({"field_id":id(field),"parent_item_ids":parents.iter().copied().map(id).collect::<Vec<_>>(),"item_id":item.map(id)})
}
fn reference(n: u128) -> GovernedRevision {
    serde_json::from_value(json!({"org_id":id(1),"object_type_id":id(2),"instance_id":id(n),"revision_id":id(n+1),"version":"1","row_hash":"a".repeat(64)})).unwrap()
}
fn schema_ref() -> InputSchemaRef {
    serde_json::from_value(json!({"org_id":id(1),"schema_id":id(3),"revision":"1","definition_revision":reference(4),"codec_digest":"b".repeat(64)})).unwrap()
}
fn value(kind: &str, text: &str) -> Cell {
    Cell::Value(serde_json::from_value::<FieldValue>(json!({"kind":kind,"value":text})).unwrap())
}
fn null() -> Cell {
    Cell::Value(serde_json::from_value::<FieldValue>(json!({"kind":"NULL"})).unwrap())
}
fn record(cells: Vec<(u128, Cell)>) -> Cell {
    Cell::Record(
        cells
            .into_iter()
            .map(|(field, cell)| (id(field), cell))
            .collect(),
    )
}
fn list(items: Vec<(u128, Cell)>) -> Cell {
    Cell::List(
        items
            .into_iter()
            .map(|(item_id, cell)| Item {
                item_id: id(item_id),
                cell,
            })
            .collect(),
    )
}
fn binding(field: u128, path: &[&str]) -> FieldBinding {
    FieldBinding {
        field_id: id(field),
        owner_schema_path: path.iter().map(|s| (*s).to_owned()).collect(),
    }
}
fn snapshot(
    cells: Vec<(u128, Cell)>,
    provenance: Vec<(FieldAddress, Provenance)>,
) -> DraftSnapshot {
    DraftSnapshot {
        schema_ref: schema_ref(),
        cells: cells
            .into_iter()
            .map(|(field, cell)| (id(field), cell))
            .collect(),
        provenance: provenance.into_iter().collect(),
        basis_refs: vec![reference(20)],
    }
}
fn scope(editable: Vec<FieldAddress>, anchors: Vec<FieldAddress>) -> DraftEditScope {
    DraftEditScope {
        editable: editable.into_iter().collect(),
        permitted_anchors: anchors.into_iter().collect(),
    }
}
fn patches(operations: Value) -> Vec<crate::draft::Patch> {
    parse_draft_patches(&serde_json::to_vec(&operations).unwrap()).unwrap()
}
fn apply(
    schema: &DraftSchema,
    before: &DraftSnapshot,
    allowed: &DraftEditScope,
    operations: Value,
    expected: &DraftSnapshot,
    changed: bool,
) {
    let retained = before.clone();
    let operations = patches(operations);
    let retained_operations = operations.clone();
    let actual = apply_draft_patches(schema, before, &operations, allowed).unwrap();
    assert_eq!(actual.snapshot, *expected);
    assert_eq!(actual.changed, changed);
    assert_eq!(*before, retained, "application mutated authoritative input");
    assert_eq!(
        operations, retained_operations,
        "application mutated submitted operations"
    );
}
fn refuses(
    schema: &DraftSchema,
    before: &DraftSnapshot,
    allowed: &DraftEditScope,
    operations: Value,
) {
    let retained = before.clone();
    let operations = patches(operations);
    let retained_operations = operations.clone();
    assert!(apply_draft_patches(schema, before, &operations, allowed).is_err());
    assert_eq!(
        *before, retained,
        "refusal must preserve every input field/provenance/basis"
    );
    assert_eq!(operations, retained_operations);
}

// Actual registered input: source.propose_qualification / QUALIFICATION / TAX_TABLE.
// These are field-to-owner-path bindings only. Types, nullability and limits are
// resolved from the existing fixed normalized owner28 schemas, never copied here.
const PAYLOAD: u128 = 100;
const SUPPORT: u128 = 101;
const COUNTS: u128 = 102;
const CHILDREN: u128 = 103;
const S1: u128 = 201;
const S2: u128 = 202;
const S3: u128 = 203;
const HIDDEN: u128 = 204;
fn tax_bindings() -> Vec<FieldBinding> {
    vec![
        binding(PAYLOAD, &["payload"]),
        binding(SUPPORT, &["payload", "support"]),
        binding(COUNTS, &["payload", "support", "dependent_counts"]),
        binding(CHILDREN, &["payload", "support", "eligible_child_counts"]),
    ]
}
fn tax_schema() -> DraftSchema {
    DraftSchema::compile_registered("source.propose_qualification", schema_ref(), tax_bindings())
        .unwrap()
}
fn tax(counts: Cell, sources: Vec<(FieldAddress, Provenance)>) -> DraftSnapshot {
    snapshot(
        vec![(
            PAYLOAD,
            record(vec![(
                SUPPORT,
                record(vec![
                    (COUNTS, counts),
                    (CHILDREN, list(vec![(HIDDEN, value("INTEGER", "7"))])),
                ]),
            )]),
        )],
        sources,
    )
}
fn ordinary_tax() -> DraftSnapshot {
    tax(
        list(vec![
            (S1, value("INTEGER", "1")),
            (S2, value("INTEGER", "2")),
        ]),
        vec![
            (address(COUNTS, &[], Some(S1)), Provenance::User),
            (address(COUNTS, &[], Some(S2)), Provenance::User),
            (
                address(CHILDREN, &[], Some(HIDDEN)),
                Provenance::Source {
                    reference: reference(30),
                },
            ),
        ],
    )
}
fn scalar_scope() -> DraftEditScope {
    scope(
        vec![
            address(COUNTS, &[], None),
            address(COUNTS, &[], Some(S1)),
            address(COUNTS, &[], Some(S2)),
            address(COUNTS, &[], Some(S3)),
        ],
        vec![
            address(COUNTS, &[], Some(S1)),
            address(COUNTS, &[], Some(S2)),
            address(COUNTS, &[], Some(S3)),
        ],
    )
}
fn set(field: u128, parents: &[u128], item: Option<u128>, kind: &str, text: &str) -> Value {
    json!({"kind":"SET","address":wire_address(field,parents,item),"value":{"kind":kind,"value":text}})
}

#[test]
fn set_preserves_full_hidden_sibling_and_basis() {
    let before = ordinary_tax();
    let expected = tax(
        list(vec![
            (S1, value("INTEGER", "1")),
            (S2, value("INTEGER", "3")),
        ]),
        before.provenance.clone().into_iter().collect(),
    );
    apply(
        &tax_schema(),
        &before,
        &scalar_scope(),
        json!([set(COUNTS, &[], Some(S2), "INTEGER", "3")]),
        &expected,
        true,
    );
}

#[test]
fn source_touch_records_user_then_equal_user_set_is_noop() {
    let before = tax(
        list(vec![
            (S1, value("INTEGER", "1")),
            (S2, value("INTEGER", "2")),
        ]),
        vec![
            (address(COUNTS, &[], Some(S1)), Provenance::User),
            (
                address(COUNTS, &[], Some(S2)),
                Provenance::Source {
                    reference: reference(40),
                },
            ),
            (
                address(CHILDREN, &[], Some(HIDDEN)),
                Provenance::Source {
                    reference: reference(30),
                },
            ),
        ],
    );
    let expected = ordinary_tax();
    let operation = json!([set(COUNTS, &[], Some(S2), "INTEGER", "2")]);
    apply(
        &tax_schema(),
        &before,
        &scalar_scope(),
        operation.clone(),
        &expected,
        true,
    );
    apply(
        &tax_schema(),
        &expected,
        &scalar_scope(),
        operation,
        &expected,
        false,
    );
}

#[test]
fn incomplete_text_is_exact_and_typed_repair_preserves_item_identity() {
    let before = ordinary_tax();
    let text = "- 한글 e\u{301}";
    let incomplete = tax(
        list(vec![
            (S1, value("INTEGER", "1")),
            (
                S2,
                Cell::Incomplete {
                    text: text.to_owned(),
                },
            ),
        ]),
        before.provenance.clone().into_iter().collect(),
    );
    apply(
        &tax_schema(),
        &before,
        &scalar_scope(),
        json!([{"kind":"SET_INCOMPLETE","address":wire_address(COUNTS,&[],Some(S2)),"editor_text":text}]),
        &incomplete,
        true,
    );
    apply(
        &tax_schema(),
        &incomplete,
        &scalar_scope(),
        json!([set(COUNTS, &[], Some(S2), "INTEGER", "2")]),
        &before,
        true,
    );
}

#[test]
fn clear_unsets_without_removing_and_null_is_distinct() {
    let before = ordinary_tax();
    let cleared = tax(
        list(vec![(S1, value("INTEGER", "1")), (S2, Cell::Unset)]),
        before.provenance.clone().into_iter().collect(),
    );
    apply(
        &tax_schema(),
        &before,
        &scalar_scope(),
        json!([{"kind":"CLEAR","address":wire_address(COUNTS,&[],Some(S2))}]),
        &cleared,
        true,
    );
    refuses(
        &tax_schema(),
        &before,
        &scalar_scope(),
        json!([{"kind":"SET","address":wire_address(COUNTS,&[],Some(S1)),"value":{"kind":"NULL"}}]),
    );

    // Actual nullable scalar: calendar.assign.to_date_exclusive is Date|null.
    // NormalizerRef is a RECORD and is deliberately not used as a scalar here.
    const FROM: u128 = 105;
    const TO: u128 = 106;
    let schema = DraftSchema::compile_registered(
        "calendar.assign",
        schema_ref(),
        vec![
            binding(FROM, &["from_date"]),
            binding(TO, &["to_date_exclusive"]),
        ],
    )
    .unwrap();
    let allowed = scope(vec![address(TO, &[], None)], vec![]);
    let origins = vec![
        (
            address(FROM, &[], None),
            Provenance::Source {
                reference: reference(60),
            },
        ),
        (address(TO, &[], None), Provenance::User),
    ];
    let date = snapshot(
        vec![
            (FROM, value("DATE", "2026-09-01")),
            (TO, value("DATE", "2026-10-01")),
        ],
        origins.clone(),
    );
    let with_null = snapshot(
        vec![(FROM, value("DATE", "2026-09-01")), (TO, null())],
        origins.clone(),
    );
    let unset = snapshot(
        vec![(FROM, value("DATE", "2026-09-01")), (TO, Cell::Unset)],
        origins,
    );
    apply(
        &schema,
        &date,
        &allowed,
        json!([{"kind":"SET","address":wire_address(TO,&[],None),"value":{"kind":"NULL"}}]),
        &with_null,
        true,
    );
    apply(
        &schema,
        &with_null,
        &allowed,
        json!([{"kind":"CLEAR","address":wire_address(TO,&[],None)}]),
        &unset,
        true,
    );
    apply(
        &schema,
        &unset,
        &allowed,
        json!([set(TO, &[], None, "DATE", "2026-10-01")]),
        &date,
        true,
    );
}

#[test]
fn ordered_insert_then_same_address_incomplete_move_and_remove() {
    let before = ordinary_tax();
    let mut p = before.provenance.clone();
    p.insert(address(COUNTS, &[], Some(S3)), Provenance::User);
    let inserted = tax(
        list(vec![
            (S1, value("INTEGER", "1")),
            (
                S3,
                Cell::Incomplete {
                    text: "-".to_owned(),
                },
            ),
            (S2, value("INTEGER", "2")),
        ]),
        p.clone().into_iter().collect(),
    );
    apply(
        &tax_schema(),
        &before,
        &scalar_scope(),
        json!([
            {"kind":"INSERT_ITEM","address":wire_address(COUNTS,&[],Some(S3)),"after_item_id":id(S1),"value":{"kind":"INTEGER","value":"3"}},
            {"kind":"SET_INCOMPLETE","address":wire_address(COUNTS,&[],Some(S3)),"editor_text":"-"}
        ]),
        &inserted,
        true,
    );
    let moved = tax(
        list(vec![
            (
                S3,
                Cell::Incomplete {
                    text: "-".to_owned(),
                },
            ),
            (S1, value("INTEGER", "1")),
            (S2, value("INTEGER", "2")),
        ]),
        p.into_iter().collect(),
    );
    apply(
        &tax_schema(),
        &inserted,
        &scalar_scope(),
        json!([{"kind":"MOVE_ITEM","address":wire_address(COUNTS,&[],Some(S3)),"after_item_id":null}]),
        &moved,
        true,
    );
    apply(
        &tax_schema(),
        &moved,
        &scalar_scope(),
        json!([{"kind":"REMOVE_ITEM","address":wire_address(COUNTS,&[],Some(S3))}]),
        &before,
        true,
    );
}

#[test]
fn invalid_identity_parent_anchor_and_scope_preserve_entire_input() {
    let before = ordinary_tax();
    let schema = tax_schema();
    let mut allowed = scalar_scope();
    for a in [
        address(999, &[], Some(S2)),
        address(COUNTS, &[S1], Some(S2)),
        address(COUNTS, &[], Some(999)),
    ] {
        allowed.editable.insert(a);
    }
    allowed
        .permitted_anchors
        .insert(address(CHILDREN, &[], Some(HIDDEN)));
    for bad in [
        set(999, &[], Some(S2), "INTEGER", "4"),
        set(COUNTS, &[S1], Some(S2), "INTEGER", "4"),
        set(COUNTS, &[], Some(999), "INTEGER", "4"),
        json!({"kind":"INSERT_ITEM","address":wire_address(COUNTS,&[],Some(S1)),"after_item_id":null,"value":{"kind":"INTEGER","value":"4"}}),
        json!({"kind":"MOVE_ITEM","address":wire_address(COUNTS,&[],Some(S2)),"after_item_id":id(HIDDEN)}),
        json!({"kind":"MOVE_ITEM","address":wire_address(COUNTS,&[],Some(S2)),"after_item_id":id(S2)}),
        json!({"kind":"REMOVE_ITEM","address":wire_address(COUNTS,&[],None)}),
    ] {
        refuses(
            &schema,
            &before,
            &allowed,
            json!([set(COUNTS, &[], Some(S1), "INTEGER", "5"), bad]),
        );
    }
    let mut no_anchor = allowed.clone();
    no_anchor
        .permitted_anchors
        .remove(&address(COUNTS, &[], Some(S1)));
    refuses(
        &schema,
        &before,
        &no_anchor,
        json!([{"kind":"MOVE_ITEM","address":wire_address(COUNTS,&[],Some(S2)),"after_item_id":id(S1)}]),
    );
    let mut denied = allowed;
    denied.editable.remove(&address(COUNTS, &[], Some(S2)));
    refuses(
        &schema,
        &before,
        &denied,
        json!([set(COUNTS, &[], Some(S2), "INTEGER", "5")]),
    );
}

#[test]
fn registered_element_kind_bound_and_partial_minimum_are_distinct() {
    let schema = tax_schema();
    let before = ordinary_tax();
    let allowed = scalar_scope();
    for bad in [
        set(COUNTS, &[], Some(S2), "TEXT", "2"),
        set(COUNTS, &[], Some(S2), "INTEGER", "0"),
        set(COUNTS, &[], Some(S2), "DECIMAL", "2.5"),
    ] {
        refuses(&schema, &before, &allowed, json!([bad]));
    }
    // Required-at-submit minimum is allowed to remain incomplete during editing.
    let empty = tax(
        list(vec![]),
        vec![(
            address(CHILDREN, &[], Some(HIDDEN)),
            Provenance::Source {
                reference: reference(30),
            },
        )],
    );
    apply(
        &schema,
        &before,
        &allowed,
        json!([{"kind":"REMOVE_ITEM","address":wire_address(COUNTS,&[],Some(S1))},{"kind":"REMOVE_ITEM","address":wire_address(COUNTS,&[],Some(S2))}]),
        &empty,
        true,
    );
    let items = (0..99).map(|n| (500 + n, value("INTEGER", "1"))).collect();
    let mut origins = vec![(
        address(CHILDREN, &[], Some(HIDDEN)),
        Provenance::Source {
            reference: reference(30),
        },
    )];
    origins.extend((0..99).map(|n| (address(COUNTS, &[], Some(500 + n)), Provenance::User)));
    let full = tax(list(items), origins);
    let allowed = scope(
        vec![
            address(COUNTS, &[], Some(500)),
            address(COUNTS, &[], Some(S3)),
        ],
        vec![],
    );
    apply(
        &schema,
        &full,
        &allowed,
        json!([set(COUNTS, &[], Some(500), "INTEGER", "1")]),
        &full,
        false,
    );
    refuses(
        &schema,
        &full,
        &allowed,
        json!([{"kind":"INSERT_ITEM","address":wire_address(COUNTS,&[],Some(S3)),"after_item_id":null,"value":{"kind":"INTEGER","value":"1"}}]),
    );
}

// Real calendar.publish paths: overrides[].duties[].breaks[]. Each repeated
// ancestor contributes its identity exactly once; singular containers add none.
const OVERRIDES: u128 = 300;
const DATE: u128 = 301;
const KIND: u128 = 302;
const DUTIES: u128 = 303;
const WORK: u128 = 304;
const PAID: u128 = 305;
const RULE: u128 = 306;
const START: u128 = 310;
const END: u128 = 311;
const BREAKS: u128 = 312;
const BREAK_START: u128 = 320;
const BREAK_END: u128 = 321;
const TREATMENT: u128 = 322;
const DAY: u128 = 400;
const DUTY: u128 = 401;
const BREAK: u128 = 402;
const OTHER_DAY: u128 = 403;
const OTHER_DUTY: u128 = 404;
fn calendar_bindings() -> Vec<FieldBinding> {
    vec![
        binding(OVERRIDES, &["overrides"]),
        binding(DATE, &["overrides", "date"]),
        binding(KIND, &["overrides", "day_kind"]),
        binding(DUTIES, &["overrides", "duties"]),
        binding(WORK, &["overrides", "expected_work_minutes"]),
        binding(PAID, &["overrides", "expected_paid_minutes"]),
        binding(RULE, &["overrides", "rule_ref"]),
        binding(START, &["overrides", "duties", "start_minute"]),
        binding(END, &["overrides", "duties", "end_minute"]),
        binding(BREAKS, &["overrides", "duties", "breaks"]),
        binding(
            BREAK_START,
            &["overrides", "duties", "breaks", "start_minute"],
        ),
        binding(BREAK_END, &["overrides", "duties", "breaks", "end_minute"]),
        binding(TREATMENT, &["overrides", "duties", "breaks", "treatment"]),
    ]
}
fn calendar_schema() -> DraftSchema {
    DraftSchema::compile_registered("calendar.publish", schema_ref(), calendar_bindings()).unwrap()
}
fn day(duties: Cell) -> Cell {
    record(vec![
        (DATE, Cell::Unset),
        (KIND, Cell::Unset),
        (DUTIES, duties),
        (WORK, Cell::Unset),
        (PAID, Cell::Unset),
        (RULE, Cell::Unset),
    ])
}
fn duty(start: Cell, end: Cell, breaks: Cell) -> Cell {
    record(vec![(START, start), (END, end), (BREAKS, breaks)])
}
fn calendar_scope() -> DraftEditScope {
    let mut editable = vec![
        address(OVERRIDES, &[], None),
        address(OVERRIDES, &[], Some(DAY)),
        address(DUTIES, &[DAY], None),
        address(DUTIES, &[DAY], Some(DUTY)),
        address(BREAKS, &[DAY, DUTY], None),
        address(BREAKS, &[DAY, DUTY], Some(BREAK)),
    ];
    for field in [DATE, KIND, WORK, PAID, RULE] {
        editable.push(address(field, &[DAY], None));
    }
    for field in [START, END] {
        editable.push(address(field, &[DAY, DUTY], None));
    }
    for field in [BREAK_START, BREAK_END, TREATMENT] {
        editable.push(address(field, &[DAY, DUTY, BREAK], None));
    }
    scope(editable, vec![])
}
fn create(field: u128, parents: &[u128], item: Option<u128>, kind: &str) -> Value {
    json!({"kind":"CREATE_CONTAINER","address":wire_address(field,parents,item),"after_item_id":null,"container_kind":kind})
}

#[test]
fn create_nested_containers_then_edit_children_uses_fixed_paths() {
    let before = snapshot(vec![(OVERRIDES, Cell::Unset)], vec![]);
    let cells = list(vec![(
        DAY,
        day(list(vec![(
            DUTY,
            duty(
                value("INTEGER", "480"),
                value("INTEGER", "1020"),
                list(vec![(
                    BREAK,
                    record(vec![
                        (BREAK_START, value("INTEGER", "720")),
                        (BREAK_END, Cell::Unset),
                        (TREATMENT, Cell::Unset),
                    ]),
                )]),
            ),
        )])),
    )]);
    let edited = vec![
        address(OVERRIDES, &[], None),
        address(OVERRIDES, &[], Some(DAY)),
        address(DUTIES, &[DAY], None),
        address(DUTIES, &[DAY], Some(DUTY)),
        address(START, &[DAY, DUTY], None),
        address(END, &[DAY, DUTY], None),
        address(BREAKS, &[DAY, DUTY], None),
        address(BREAKS, &[DAY, DUTY], Some(BREAK)),
        address(BREAK_START, &[DAY, DUTY, BREAK], None),
    ];
    let expected = snapshot(
        vec![(OVERRIDES, cells)],
        edited.into_iter().map(|a| (a, Provenance::User)).collect(),
    );
    apply(
        &calendar_schema(),
        &before,
        &calendar_scope(),
        json!([
            create(OVERRIDES, &[], None, "LIST"),
            create(OVERRIDES, &[], Some(DAY), "RECORD"),
            create(DUTIES, &[DAY], None, "LIST"),
            create(DUTIES, &[DAY], Some(DUTY), "RECORD"),
            set(START, &[DAY, DUTY], None, "INTEGER", "480"),
            set(END, &[DAY, DUTY], None, "INTEGER", "1020"),
            create(BREAKS, &[DAY, DUTY], None, "LIST"),
            create(BREAKS, &[DAY, DUTY], Some(BREAK), "RECORD"),
            set(BREAK_START, &[DAY, DUTY, BREAK], None, "INTEGER", "720")
        ]),
        &expected,
        true,
    );
}

#[test]
fn structural_refusals_cannot_reset_or_clear_containers_or_guess_parent() {
    let before = snapshot(
        vec![(
            OVERRIDES,
            list(vec![(
                DAY,
                day(list(vec![(
                    DUTY,
                    duty(
                        value("INTEGER", "480"),
                        value("INTEGER", "1020"),
                        Cell::Unset,
                    ),
                )])),
            )]),
        )],
        vec![
            (address(START, &[DAY, DUTY], None), Provenance::User),
            (address(END, &[DAY, DUTY], None), Provenance::User),
        ],
    );
    let mut allowed = calendar_scope();
    allowed.editable.extend([
        address(START, &[DAY], None),
        address(START, &[DAY, OTHER_DUTY], None),
        address(DUTIES, &[DAY], Some(BREAK)),
    ]);
    apply(
        &calendar_schema(),
        &before,
        &allowed,
        json!([set(END, &[DAY, DUTY], None, "INTEGER", "1020")]),
        &before,
        false,
    );
    for bad in [
        create(DUTIES, &[DAY], None, "LIST"),
        create(DUTIES, &[DAY], Some(DUTY), "RECORD"),
        create(BREAKS, &[DAY, DUTY], None, "RECORD"),
        json!({"kind":"CLEAR","address":wire_address(DUTIES,&[DAY],None)}),
        set(START, &[DAY], None, "INTEGER", "600"),
        set(START, &[DAY, OTHER_DUTY], None, "INTEGER", "600"),
        json!({"kind":"INSERT_ITEM","address":wire_address(DUTIES,&[DAY],Some(BREAK)),"after_item_id":null,"value":{"kind":"INTEGER","value":"1"}}),
    ] {
        refuses(&calendar_schema(), &before, &allowed, json!([bad]));
    }
}

#[test]
fn removing_parent_requires_each_descendant_and_move_preserves_subtree() {
    let subtree = day(list(vec![(
        DUTY,
        duty(
            value("INTEGER", "480"),
            value("INTEGER", "1020"),
            Cell::Unset,
        ),
    )]));
    let other = day(Cell::Unset);
    let origin = address(START, &[DAY, DUTY], None);
    let before = snapshot(
        vec![(
            OVERRIDES,
            list(vec![(DAY, subtree.clone()), (OTHER_DAY, other.clone())]),
        )],
        vec![
            (
                origin.clone(),
                Provenance::Source {
                    reference: reference(50),
                },
            ),
            (address(END, &[DAY, DUTY], None), Provenance::User),
        ],
    );
    let mut allowed = calendar_scope();
    allowed.editable.remove(&origin);
    refuses(
        &calendar_schema(),
        &before,
        &allowed,
        json!([{"kind":"REMOVE_ITEM","address":wire_address(OVERRIDES,&[],Some(DAY))}]),
    );
    allowed
        .permitted_anchors
        .insert(address(OVERRIDES, &[], Some(OTHER_DAY)));
    let moved = snapshot(
        vec![(
            OVERRIDES,
            list(vec![(OTHER_DAY, other.clone()), (DAY, subtree)]),
        )],
        before.provenance.clone().into_iter().collect(),
    );
    apply(
        &calendar_schema(),
        &before,
        &allowed,
        json!([{"kind":"MOVE_ITEM","address":wire_address(OVERRIDES,&[],Some(DAY)),"after_item_id":id(OTHER_DAY)}]),
        &moved,
        true,
    );
    allowed.editable.insert(origin);
    let removed = snapshot(vec![(OVERRIDES, list(vec![(OTHER_DAY, other)]))], vec![]);
    apply(
        &calendar_schema(),
        &before,
        &allowed,
        json!([{"kind":"REMOVE_ITEM","address":wire_address(OVERRIDES,&[],Some(DAY))}]),
        &removed,
        true,
    );
}

#[test]
fn schema_binding_and_existing_snapshot_identity_corruption_refuse() {
    let mut duplicate = tax_bindings();
    duplicate.push(binding(
        COUNTS,
        &["payload", "support", "eligible_child_counts"],
    ));
    assert!(
        DraftSchema::compile_registered("source.propose_qualification", schema_ref(), duplicate)
            .is_err()
    );
    assert!(
        DraftSchema::compile_registered("unregistered.action", schema_ref(), tax_bindings())
            .is_err()
    );
    assert!(
        DraftSchema::compile_registered(
            "source.propose_qualification",
            schema_ref(),
            vec![binding(COUNTS, &["payload", "not_a_registered_field"])]
        )
        .is_err()
    );
    let bad = tax(
        list(vec![
            (S1, value("INTEGER", "1")),
            (S1, value("INTEGER", "2")),
        ]),
        vec![],
    );
    refuses(
        &tax_schema(),
        &bad,
        &scalar_scope(),
        json!([set(COUNTS, &[], Some(S1), "INTEGER", "3")]),
    );
    let mut mismatch = ordinary_tax();
    mismatch.schema_ref=serde_json::from_value(json!({"org_id":id(1),"schema_id":id(999),"revision":"1","definition_revision":reference(4),"codec_digest":"b".repeat(64)})).unwrap();
    refuses(
        &tax_schema(),
        &mismatch,
        &scalar_scope(),
        json!([set(COUNTS, &[], Some(S1), "INTEGER", "3")]),
    );
}

#[test]
fn strict_patch_decoder_preserves_decimal_strings_and_rejects_raw_ambiguity() {
    let good = serde_json::to_vec(&json!([
        set(COUNTS, &[], Some(S1), "INTEGER", "2"),
        set(COUNTS, &[], Some(S2), "DECIMAL", "1.250000")
    ]))
    .unwrap();
    let parsed = parse_draft_patches(&good).unwrap();
    assert_eq!(parsed.len(), 2);
    let changed_spelling = serde_json::to_vec(&json!([
        set(COUNTS, &[], Some(S1), "INTEGER", "2"),
        set(COUNTS, &[], Some(S2), "DECIMAL", "1.25")
    ]))
    .unwrap();
    assert_ne!(
        parsed,
        parse_draft_patches(&changed_spelling).unwrap(),
        "decoder lost exact decimal spelling"
    );
    let reordered = serde_json::to_vec(&json!([
        set(COUNTS, &[], Some(S2), "DECIMAL", "1.250000"),
        set(COUNTS, &[], Some(S1), "INTEGER", "2")
    ]))
    .unwrap();
    assert_ne!(
        parsed,
        parse_draft_patches(&reordered).unwrap(),
        "decoder lost operation order"
    );
    for bad_value in [
        json!({"kind":"INTEGER","value":2}),
        json!({"kind":"INTEGER","value":"02"}),
        json!({"kind":"INTEGER","value":"9007199254740992"}),
        json!({"kind":"DECIMAL","value":1.25}),
        json!({"kind":"DECIMAL","value":"1e2"}),
    ] {
        assert!(parse_draft_patches(&serde_json::to_vec(&json!([{"kind":"SET","address":wire_address(COUNTS,&[],Some(S1)),"value":bad_value}])).unwrap()).is_err());
    }
    let valid = String::from_utf8(
        serde_json::to_vec(&json!([set(COUNTS, &[], Some(S1), "INTEGER", "2")])).unwrap(),
    )
    .unwrap();
    let duplicate = valid.replacen("\"kind\":\"SET\"", "\"kind\":\"SET\",\"kind\":\"CLEAR\"", 1);
    assert_ne!(duplicate, valid);
    assert!(parse_draft_patches(duplicate.as_bytes()).is_err());
    assert!(parse_draft_patches(&serde_json::to_vec(&json!([{"kind":"CLEAR","address":wire_address(COUNTS,&[],Some(S1)),"unexpected":true}])).unwrap()).is_err());
    assert!(parse_draft_patches(&serde_json::to_vec(&json!([{"kind":"SET_INCOMPLETE","address":wire_address(COUNTS,&[],Some(S1)),"editor_text":"x".repeat(4097)}])).unwrap()).is_err());
    assert!(parse_draft_patches(&serde_json::to_vec(&json!([{"kind":"SET_INCOMPLETE","address":wire_address(COUNTS,&[],Some(S1)),"editor_text":"\u{0000}"}])).unwrap()).is_err());
    assert!(parse_draft_patches(b"[]").is_err());
    // Debug bounds used by assertions never authorize logging draft contents.
    // These parser-only operands are data, not values admitted to the tax field.
    const CANARY: &str = "TEST_ONLY_DRAFT_CONTENT_MUST_NOT_APPEAR_IN_DIAGNOSTICS";
    let secret_value: FieldValue =
        serde_json::from_value(json!({"kind":"TEXT","value":CANARY})).unwrap();
    let secret_cell = Cell::Incomplete {
        text: CANARY.to_owned(),
    };
    let secret_patches = patches(json!([set(COUNTS, &[], Some(S1), "TEXT", CANARY)]));
    const SOURCE_REFERENCE_CANARY: &str =
        "e4f2d8b17c9a03f6e4f2d8b17c9a03f6e4f2d8b17c9a03f6e4f2d8b17c9a03f6";
    let mut source_json = serde_json::to_value(reference(70)).unwrap();
    source_json["row_hash"] = json!(SOURCE_REFERENCE_CANARY);
    let source_reference: GovernedRevision = serde_json::from_value(source_json).unwrap();
    let secret_snapshot = snapshot(
        vec![(COUNTS, secret_cell.clone())],
        vec![(
            address(COUNTS, &[], None),
            Provenance::Source {
                reference: source_reference,
            },
        )],
    );
    assert!(
        !format!("{secret_snapshot:?}").contains(SOURCE_REFERENCE_CANARY),
        "snapshot Debug exposed source-reference bytes"
    );
    for diagnostic in [
        format!("{secret_value:?}"),
        format!("{secret_cell:?}"),
        format!("{secret_patches:?}"),
        format!("{secret_snapshot:?}"),
    ] {
        assert!(
            !diagnostic.contains(CANARY),
            "draft Debug must redact input contents"
        );
    }
    let malformed = serde_json::to_vec(
        &json!([{"kind":CANARY,"address":wire_address(COUNTS,&[],Some(S1)),"editor_text":CANARY}]),
    )
    .unwrap();
    let error = parse_draft_patches(&malformed).unwrap_err();
    assert!(
        !format!("{error:?}").contains(CANARY),
        "error Debug echoed submitted input"
    );
    assert!(
        !format!("{error}").contains(CANARY),
        "error Display echoed submitted input"
    );
}

// Root review clarification: editor item UUIDs are unique across the complete
// live snapshot. Field/ancestor addresses do not create separate UUID namespaces.
#[test]
fn duplicate_item_ids_across_registered_collections_refuse() {
    let schema = tax_schema();
    let before = ordinary_tax();
    apply(
        &schema,
        &before,
        &scalar_scope(),
        json!([set(COUNTS, &[], Some(S1), "INTEGER", "1")]),
        &before,
        false,
    );
    let corrupt = tax(
        list(vec![
            (S1, value("INTEGER", "1")),
            (HIDDEN, value("INTEGER", "2")),
        ]),
        vec![
            (address(COUNTS, &[], Some(S1)), Provenance::User),
            (address(COUNTS, &[], Some(HIDDEN)), Provenance::User),
            (
                address(CHILDREN, &[], Some(HIDDEN)),
                Provenance::Source {
                    reference: reference(30),
                },
            ),
        ],
    );
    refuses(
        &schema,
        &corrupt,
        &scalar_scope(),
        json!([set(COUNTS, &[], Some(S1), "INTEGER", "1")]),
    );
}

#[test]
fn insert_requires_fresh_live_id_before_later_removal() {
    let schema = tax_schema();
    let before = ordinary_tax();
    let mut allowed = scalar_scope();
    allowed.editable.extend([
        address(COUNTS, &[], Some(HIDDEN)),
        address(CHILDREN, &[], Some(HIDDEN)),
    ]);
    let expected = snapshot(
        vec![(
            PAYLOAD,
            record(vec![(
                SUPPORT,
                record(vec![
                    (
                        COUNTS,
                        list(vec![
                            (S3, value("INTEGER", "3")),
                            (S1, value("INTEGER", "1")),
                            (S2, value("INTEGER", "2")),
                        ]),
                    ),
                    (CHILDREN, list(vec![])),
                ]),
            )]),
        )],
        vec![
            (address(COUNTS, &[], Some(S1)), Provenance::User),
            (address(COUNTS, &[], Some(S2)), Provenance::User),
            (address(COUNTS, &[], Some(S3)), Provenance::User),
        ],
    );
    // The same structural save with a fresh ID succeeds, proving the removal
    // scope and partial empty required list are not the refusal's cause.
    apply(
        &schema,
        &before,
        &allowed,
        json!([
            {"kind":"INSERT_ITEM","address":wire_address(COUNTS,&[],Some(S3)),"after_item_id":null,"value":{"kind":"INTEGER","value":"3"}},
            {"kind":"REMOVE_ITEM","address":wire_address(CHILDREN,&[],Some(HIDDEN))}
        ]),
        &expected,
        true,
    );
    // A final-only uniqueness check would miss the collision after removal.
    refuses(
        &schema,
        &before,
        &allowed,
        json!([
            {"kind":"INSERT_ITEM","address":wire_address(COUNTS,&[],Some(HIDDEN)),"after_item_id":null,"value":{"kind":"INTEGER","value":"3"}},
            {"kind":"REMOVE_ITEM","address":wire_address(CHILDREN,&[],Some(HIDDEN))}
        ]),
    );
}

#[test]
fn create_container_requires_fresh_live_id_before_subtree_removal() {
    let schema = calendar_schema();
    let before = snapshot(
        vec![(
            OVERRIDES,
            list(vec![(
                DAY,
                day(list(vec![(
                    DUTY,
                    duty(Cell::Unset, Cell::Unset, Cell::Unset),
                )])),
            )]),
        )],
        vec![],
    );
    let mut allowed = calendar_scope();
    allowed.editable.extend([
        address(OVERRIDES, &[], Some(OTHER_DAY)),
        address(OVERRIDES, &[], Some(DUTY)),
    ]);
    let expected = snapshot(
        vec![(OVERRIDES, list(vec![(OTHER_DAY, day(Cell::Unset))]))],
        vec![(address(OVERRIDES, &[], Some(OTHER_DAY)), Provenance::User)],
    );
    apply(
        &schema,
        &before,
        &allowed,
        json!([
            create(OVERRIDES, &[], Some(OTHER_DAY), "RECORD"),
            {"kind":"REMOVE_ITEM","address":wire_address(OVERRIDES,&[],Some(DAY))}
        ]),
        &expected,
        true,
    );
    // DUTY is already a live nested item. Removing its old subtree afterward
    // cannot make that UUID fresh at the earlier CREATE_CONTAINER operation.
    refuses(
        &schema,
        &before,
        &allowed,
        json!([
            create(OVERRIDES, &[], Some(DUTY), "RECORD"),
            {"kind":"REMOVE_ITEM","address":wire_address(OVERRIDES,&[],Some(DAY))}
        ]),
    );
}
