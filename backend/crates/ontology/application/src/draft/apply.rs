use super::schema::{Kind, Node};
use super::*;

// Conservative work/allocation accounting only. This does not certify any
// serialized persistence format or canonical draft-revision byte size.
struct Budget(usize);
impl Budget {
    fn spend(&mut self, amount: usize) -> Result<(), DraftError> {
        self.0 = self
            .0
            .checked_sub(amount)
            .ok_or(DraftError("draft work bound"))?;
        Ok(())
    }
    fn value(&mut self, value: &Value) -> Result<(), DraftError> {
        self.spend(owner28::canonical(value)?.len())
    }
}

pub(crate) fn apply_draft_patches(
    schema: &DraftSchema,
    current: &DraftSnapshot,
    patches: &[Patch],
    scope: &DraftEditScope,
) -> Result<DraftSnapshotChange, DraftError> {
    check(
        !patches.is_empty() && patches.len() <= PATCH_COUNT,
        "draft patch count",
    )?;
    let mut patch_budget = Budget(PATCH_BYTES);
    // Include the enclosing array and separators for directly constructed patches.
    patch_budget.spend(patches.len() + 1)?;
    for patch in patches {
        patch.address().validate()?;
        if let Patch::Incomplete { text, .. } = patch {
            check(text.len() <= 16_384, "draft editor text bound")?;
        }
        let wire = patch.wire();
        patch_budget.value(&wire)?;
        check(
            owner28::schema::draft_definition("Patch", &wire)?,
            "invalid draft patch",
        )?;
        check(
            scope.editable.contains(patch.address()),
            "draft edit scope refused",
        )?;
    }
    check(
        current.schema_ref == schema.schema_ref,
        "draft schema mismatch",
    )?;
    let mut budget = Budget(schema.work_budget);
    let mut live_items = validate_snapshot(schema, current, &mut budget)?;
    // Only validated bounded input is cloned; all edits occur on this private
    // copy. Failed operations never write back a partially edited snapshot.
    let mut next = current.clone();
    for patch in patches {
        apply_one(
            schema,
            &mut next,
            patch,
            scope,
            &mut budget,
            &mut live_items,
        )?;
    }
    validate_snapshot(schema, &next, &mut Budget(schema.work_budget))?;
    let changed = next != *current;
    Ok(DraftSnapshotChange {
        snapshot: next,
        changed,
    })
}

fn validate_snapshot(
    schema: &DraftSchema,
    snapshot: &DraftSnapshot,
    budget: &mut Budget,
) -> Result<BTreeSet<Uuid>, DraftError> {
    check(
        snapshot.cells.len() == schema.root.len(),
        "draft field coverage mismatch",
    )?;
    check(
        snapshot.provenance.len() <= schema.work_budget / 32
            && snapshot.basis_refs.len() <= PATCH_COUNT,
        "draft metadata count",
    )?;
    budget.value(&snapshot.schema_ref.0)?;
    for reference in &snapshot.basis_refs {
        budget.value(&reference.0)?;
    }
    let mut live = BTreeSet::new();
    let mut live_items = BTreeSet::new();
    for (field, cell) in &snapshot.cells {
        let node = schema
            .root
            .get(field)
            .ok_or(DraftError("unknown draft field"))?;
        let address = FieldAddress {
            field_id: *field,
            parent_item_ids: Vec::new(),
            item_id: None,
        };
        validate_cell(node, cell, &address, 0, budget, &mut live, &mut live_items)?;
    }
    for (address, provenance) in &snapshot.provenance {
        check(live.contains(address), "orphan draft provenance")?;
        budget.spend(64 + address.parent_item_ids.len() * 16)?;
        if let DraftProvenance::Source { reference } = provenance {
            budget.value(&reference.0)?;
        }
    }
    Ok(live_items)
}
fn validate_cell(
    node: &Node,
    cell: &DraftCell,
    address: &FieldAddress,
    depth: usize,
    budget: &mut Budget,
    live: &mut BTreeSet<FieldAddress>,
    live_items: &mut BTreeSet<Uuid>,
) -> Result<(), DraftError> {
    check(depth <= CELL_DEPTH, "draft cell depth")?;
    address.validate()?;
    budget.spend(64 + address.parent_item_ids.len() * 16)?;
    check(live.insert(address.clone()), "duplicate draft address")?;
    match cell {
        DraftCell::Unset => Ok(()),
        DraftCell::Incomplete { text } => {
            check(
                node.scalar() && text.len() <= 16_384 && !text.contains('\0'),
                "invalid incomplete draft cell",
            )?;
            budget.spend(text.len())?;
            check(text.chars().count() <= 4096, "draft editor text bound")
        }
        DraftCell::Value(value) => {
            budget.value(&value.0)?;
            check(
                node.validate_value(value),
                "draft value does not match registered field",
            )
        }
        DraftCell::Record(cells) => {
            let Kind::Record(children) = &node.kind else {
                return Err(DraftError("draft record shape mismatch"));
            };
            check(
                cells.len() == children.len(),
                "draft record field coverage mismatch",
            )?;
            for (field, child) in cells {
                let node = children
                    .get(field)
                    .ok_or(DraftError("unknown draft child"))?;
                validate_cell(
                    node,
                    child,
                    &FieldAddress {
                        field_id: *field,
                        parent_item_ids: address.ancestors(),
                        item_id: None,
                    },
                    depth + 1,
                    budget,
                    live,
                    live_items,
                )?;
            }
            Ok(())
        }
        DraftCell::List(items) => {
            let Kind::List { item, maximum } = &node.kind else {
                return Err(DraftError("draft list shape mismatch"));
            };
            check(items.len() <= *maximum, "draft list bound")?;
            for entry in items {
                check(
                    !entry.item_id.is_nil() && live_items.insert(entry.item_id),
                    "duplicate draft item",
                )?;
                validate_cell(
                    item,
                    &entry.cell,
                    &FieldAddress {
                        field_id: address.field_id,
                        parent_item_ids: address.ancestors(),
                        item_id: Some(entry.item_id),
                    },
                    depth + 1,
                    budget,
                    live,
                    live_items,
                )?;
            }
            Ok(())
        }
    }
}

fn descend_lists<'a, 'n>(
    mut cell: &'a mut DraftCell,
    mut node: &'n Node,
    parents: &[Uuid],
    cursor: &mut usize,
    until_record: bool,
) -> Result<(&'a mut DraftCell, &'n Node), DraftError> {
    while matches!(node.kind, Kind::List { .. }) && (until_record || *cursor < parents.len()) {
        let id = parents
            .get(*cursor)
            .ok_or(DraftError("missing draft parent item"))?;
        let (DraftCell::List(items), Kind::List { item, .. }) = (cell, &node.kind) else {
            return Err(DraftError("draft parent container unavailable"));
        };
        cell = &mut items
            .iter_mut()
            .find(|entry| entry.item_id == *id)
            .ok_or(DraftError("draft parent item unavailable"))?
            .cell;
        node = item;
        *cursor += 1;
    }
    Ok((cell, node))
}
fn named<'a, 'n>(
    cells: &'a mut BTreeMap<Uuid, DraftCell>,
    nodes: &'n BTreeMap<Uuid, Node>,
    route: &[Uuid],
    parents: &[Uuid],
    cursor: &mut usize,
) -> Result<(&'a mut DraftCell, &'n Node), DraftError> {
    let (first, rest) = route
        .split_first()
        .ok_or(DraftError("missing draft field route"))?;
    let cell = cells
        .get_mut(first)
        .ok_or(DraftError("draft field unavailable"))?;
    let node = nodes.get(first).ok_or(DraftError("unknown draft field"))?;
    let (cell, node) = descend_lists(cell, node, parents, cursor, !rest.is_empty())?;
    if rest.is_empty() {
        return Ok((cell, node));
    }
    let (DraftCell::Record(cells), Kind::Record(nodes)) = (cell, &node.kind) else {
        return Err(DraftError("draft parent record unavailable"));
    };
    named(cells, nodes, rest, parents, cursor)
}
fn locate<'a, 'n>(
    schema: &'n DraftSchema,
    cells: &'a mut BTreeMap<Uuid, DraftCell>,
    address: &FieldAddress,
) -> Result<(&'a mut DraftCell, &'n Node), DraftError> {
    let route = schema
        .routes
        .get(&address.field_id)
        .ok_or(DraftError("unknown draft field"))?;
    let mut cursor = 0;
    let (cell, node) = named(
        cells,
        &schema.root,
        route,
        &address.parent_item_ids,
        &mut cursor,
    )?;
    check(
        cursor == address.parent_item_ids.len(),
        "wrong draft parent path",
    )?;
    if let Some(id) = address.item_id {
        let (DraftCell::List(items), Kind::List { item, .. }) = (cell, &node.kind) else {
            return Err(DraftError("draft item is not in a collection"));
        };
        Ok((
            &mut items
                .iter_mut()
                .find(|entry| entry.item_id == id)
                .ok_or(DraftError("draft item unavailable"))?
                .cell,
            item,
        ))
    } else {
        Ok((cell, node))
    }
}
fn collection<'a, 'n>(
    schema: &'n DraftSchema,
    cells: &'a mut BTreeMap<Uuid, DraftCell>,
    address: &FieldAddress,
) -> Result<(&'a mut Vec<DraftItem>, &'n Node, usize), DraftError> {
    check(address.item_id.is_some(), "draft collection item required")?;
    let container = FieldAddress {
        field_id: address.field_id,
        parent_item_ids: address.parent_item_ids.clone(),
        item_id: None,
    };
    let (cell, node) = locate(schema, cells, &container)?;
    match (cell, &node.kind) {
        (DraftCell::List(items), Kind::List { item, maximum }) => Ok((items, item, *maximum)),
        _ => Err(DraftError("draft collection unavailable")),
    }
}
fn position(
    items: &[DraftItem],
    address: &FieldAddress,
    after: Option<Uuid>,
    scope: &DraftEditScope,
) -> Result<usize, DraftError> {
    match after {
        None => Ok(0),
        Some(id) => {
            check(
                !id.is_nil() && Some(id) != address.item_id,
                "invalid draft anchor",
            )?;
            let anchor = FieldAddress {
                field_id: address.field_id,
                parent_item_ids: address.parent_item_ids.clone(),
                item_id: Some(id),
            };
            check(
                scope.permitted_anchors.contains(&anchor),
                "draft anchor scope refused",
            )?;
            items
                .iter()
                .position(|entry| entry.item_id == id)
                .map(|i| i + 1)
                .ok_or(DraftError("draft sibling anchor unavailable"))
        }
    }
}
fn construct(
    node: &Node,
    kind: ContainerKind,
    budget: &mut Budget,
) -> Result<DraftCell, DraftError> {
    let cells = match &node.kind {
        Kind::Record(children) => children.len() + 1,
        _ => 1,
    };
    budget.spend(cells * 64)?;
    node.empty(kind)
}
fn insertion(
    items: &[DraftItem],
    address: &FieldAddress,
    after: Option<Uuid>,
    scope: &DraftEditScope,
    maximum: usize,
    live_items: &mut BTreeSet<Uuid>,
) -> Result<(Uuid, usize), DraftError> {
    let id = address
        .item_id
        .ok_or(DraftError("draft item identity required"))?;
    check(
        !id.is_nil() && live_items.insert(id),
        "draft item already exists",
    )?;
    check(items.len() < maximum, "draft list bound")?;
    Ok((id, position(items, address, after, scope)?))
}
fn subtree(
    cell: &DraftCell,
    address: &FieldAddress,
    depth: usize,
    addresses: &mut BTreeSet<FieldAddress>,
) -> Result<(), DraftError> {
    check(depth <= CELL_DEPTH, "draft subtree depth")?;
    addresses.insert(address.clone());
    match cell {
        DraftCell::Record(children) => {
            for (field, child) in children {
                subtree(
                    child,
                    &FieldAddress {
                        field_id: *field,
                        parent_item_ids: address.ancestors(),
                        item_id: None,
                    },
                    depth + 1,
                    addresses,
                )?;
            }
        }
        DraftCell::List(items) => {
            for item in items {
                subtree(
                    &item.cell,
                    &FieldAddress {
                        field_id: address.field_id,
                        parent_item_ids: address.ancestors(),
                        item_id: Some(item.item_id),
                    },
                    depth + 1,
                    addresses,
                )?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn apply_one(
    schema: &DraftSchema,
    next: &mut DraftSnapshot,
    patch: &Patch,
    scope: &DraftEditScope,
    budget: &mut Budget,
    live_items: &mut BTreeSet<Uuid>,
) -> Result<(), DraftError> {
    let address = patch.address();
    match patch {
        Patch::Set { value, .. } => {
            budget.value(&value.0)?;
            let (cell, node) = locate(schema, &mut next.cells, address)?;
            check(node.validate_value(value), "draft scalar value mismatch")?;
            *cell = DraftCell::Value(value.clone());
            next.provenance
                .insert(address.clone(), DraftProvenance::User);
        }
        Patch::Clear { .. } | Patch::Incomplete { .. } => {
            let (cell, node) = locate(schema, &mut next.cells, address)?;
            check(node.scalar(), "draft operation requires scalar")?;
            *cell = match patch {
                Patch::Incomplete { text, .. } => {
                    budget.spend(text.len())?;
                    DraftCell::Incomplete { text: text.clone() }
                }
                _ => DraftCell::Unset,
            };
            next.provenance
                .insert(address.clone(), DraftProvenance::User);
        }
        Patch::Insert { value, after, .. } => {
            budget.value(&value.0)?;
            budget.spend(64)?;
            let (items, node, maximum) = collection(schema, &mut next.cells, address)?;
            check(
                node.validate_value(value),
                "draft insertion requires valid scalar element",
            )?;
            let (item_id, at) = insertion(items, address, *after, scope, maximum, live_items)?;
            items.insert(
                at,
                DraftItem {
                    item_id,
                    cell: DraftCell::Value(value.clone()),
                },
            );
            next.provenance
                .insert(address.clone(), DraftProvenance::User);
        }
        Patch::Create {
            after, container, ..
        } => {
            if address.item_id.is_some() {
                let (items, node, maximum) = collection(schema, &mut next.cells, address)?;
                let (item_id, at) = insertion(items, address, *after, scope, maximum, live_items)?;
                let cell = construct(node, *container, budget)?;
                items.insert(at, DraftItem { item_id, cell });
            } else {
                check(
                    after.is_none(),
                    "singular draft container cannot have anchor",
                )?;
                let (cell, node) = locate(schema, &mut next.cells, address)?;
                check(
                    matches!(cell, DraftCell::Unset),
                    "draft container already exists",
                )?;
                *cell = construct(node, *container, budget)?;
            }
            next.provenance
                .insert(address.clone(), DraftProvenance::User);
        }
        Patch::Move { after, .. } => {
            let (items, _, _) = collection(schema, &mut next.cells, address)?;
            let at = position(items, address, *after, scope)?;
            let from = items
                .iter()
                .position(|entry| Some(entry.item_id) == address.item_id)
                .ok_or(DraftError("draft item unavailable"))?;
            let item = items.remove(from);
            items.insert(if from < at { at - 1 } else { at }, item);
        }
        Patch::Remove { .. } => {
            let removed = {
                let (items, _, _) = collection(schema, &mut next.cells, address)?;
                let index = items
                    .iter()
                    .position(|entry| Some(entry.item_id) == address.item_id)
                    .ok_or(DraftError("draft item unavailable"))?;
                let mut affected = BTreeSet::new();
                subtree(&items[index].cell, address, 0, &mut affected)?;
                check(
                    affected.iter().all(|entry| scope.editable.contains(entry)),
                    "draft descendant scope refused",
                )?;
                items.remove(index);
                affected
            };
            for removed_address in &removed {
                if let Some(item_id) = removed_address.item_id {
                    live_items.remove(&item_id);
                }
            }
            next.provenance.retain(|entry, _| !removed.contains(entry));
        }
    }
    Ok(())
}
