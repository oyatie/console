//! Composition oracle for `console-contracts`.
//!
//! Three properties, each of which has been a real source of silent contract
//! corruption in hand-merged OpenAPI documents:
//!   1. duplicate path / operation / schema keys are ERRORS, never last-writer-wins;
//!   2. output bytes do not depend on fragment order, so two runs agree;
//!   3. output bytes do not depend on how the fragment author indented the body.
//!
//! Tests return `Result` so they can use `?` without tripping the workspace
//! `unwrap_used` / `expect_used` / `panic` lints.

use console_contracts::{DuplicateKind, Fragment, NamedYaml, Operation, PathItem, compose};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

const TODOS: Fragment = Fragment {
    source: "console-demo-todos-rest",
    paths: &[PathItem {
        path: "/api/v1/me/todos",
        operations: &[
            Operation {
                method: "get",
                body: "operationId: listMyTodos\nresponses:\n  '200':\n    description: ok\n",
            },
            Operation {
                method: "post",
                body: "operationId: createMyTodo\nresponses:\n  '201':\n    description: created\n",
            },
        ],
    }],
    schemas: &[NamedYaml {
        name: "TodoSummary",
        body: "type: object\n",
    }],
    external_schemas: &[],
};

const NOTES: Fragment = Fragment {
    source: "console-demo-notes-rest",
    paths: &[PathItem {
        path: "/api/v1/me/notes",
        operations: &[Operation {
            method: "get",
            body: "operationId: listMyNotes\nresponses:\n  '200':\n    description: ok\n",
        }],
    }],
    schemas: &[NamedYaml {
        name: "NoteSummary",
        body: "type: object\n",
    }],
    external_schemas: &[],
};

// ---------------------------------------------------------------------------
// 1. Duplicate keys are errors
// ---------------------------------------------------------------------------

#[test]
fn duplicate_path_key_across_fragments_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    const SHADOW: Fragment = Fragment {
        source: "console-demo-shadow-rest",
        paths: &[PathItem {
            path: "/api/v1/me/todos",
            operations: &[Operation {
                method: "delete",
                body: "operationId: shadow\n",
            }],
        }],
        schemas: &[],
        external_schemas: &[],
    };

    let errors = compose(&[&TODOS, &SHADOW])
        .err()
        .ok_or("composing two fragments that both claim /api/v1/me/todos must fail")?
        .duplicates;

    let dup = errors
        .iter()
        .find(|e| e.kind == DuplicateKind::Path)
        .ok_or_else(|| format!("expected a duplicate Path error, got {errors:#?}"))?;
    assert_eq!(dup.key, "/api/v1/me/todos");
    assert_eq!(dup.first, "console-demo-todos-rest");
    assert_eq!(dup.second, "console-demo-shadow-rest");
    Ok(())
}

#[test]
fn duplicate_operation_key_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    // Same path listed once, but `get` declared twice — and with different
    // casing, so a case-blind merge would emit both instead of colliding.
    const DOUBLE_GET: Fragment = Fragment {
        source: "console-demo-double-rest",
        paths: &[PathItem {
            path: "/api/v1/thing",
            operations: &[
                Operation {
                    method: "get",
                    body: "operationId: first\n",
                },
                Operation {
                    method: "GET",
                    body: "operationId: second\n",
                },
            ],
        }],
        schemas: &[],
        external_schemas: &[],
    };

    let errors = compose(&[&DOUBLE_GET])
        .err()
        .ok_or("declaring GET twice on one path must fail")?
        .duplicates;
    let dup = errors
        .iter()
        .find(|e| e.kind == DuplicateKind::Operation)
        .ok_or_else(|| format!("expected a duplicate Operation error, got {errors:#?}"))?;
    assert_eq!(dup.key, "get /api/v1/thing");
    Ok(())
}

#[test]
fn duplicate_schema_key_across_fragments_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    const CLASH: Fragment = Fragment {
        source: "console-demo-clash-rest",
        paths: &[],
        schemas: &[NamedYaml {
            name: "TodoSummary",
            body: "type: string\n",
        }],
        external_schemas: &[],
    };

    let errors = compose(&[&TODOS, &CLASH])
        .err()
        .ok_or("two fragments defining TodoSummary must fail")?
        .duplicates;
    let dup = errors
        .iter()
        .find(|e| e.kind == DuplicateKind::Schema)
        .ok_or_else(|| format!("expected a duplicate Schema error, got {errors:#?}"))?;
    assert_eq!(dup.key, "TodoSummary");
    assert_eq!(dup.first, "console-demo-todos-rest");
    assert_eq!(dup.second, "console-demo-clash-rest");
    Ok(())
}

#[test]
fn every_duplicate_is_reported_not_just_the_first() -> Result<(), Box<dyn std::error::Error>> {
    const CLASH: Fragment = Fragment {
        source: "console-demo-clash-rest",
        paths: &[PathItem {
            path: "/api/v1/me/todos",
            operations: &[Operation {
                method: "get",
                body: "operationId: clash\n",
            }],
        }],
        schemas: &[NamedYaml {
            name: "TodoSummary",
            body: "type: string\n",
        }],
        external_schemas: &[],
    };

    let errors = compose(&[&TODOS, &CLASH])
        .err()
        .ok_or("must fail")?
        .duplicates;
    assert!(
        errors.iter().any(|e| e.kind == DuplicateKind::Path)
            && errors.iter().any(|e| e.kind == DuplicateKind::Schema),
        "CI needs the whole duplicate list in one run, got {errors:#?}"
    );
    Ok(())
}

#[test]
fn duplicate_error_message_names_both_contributors() -> Result<(), Box<dyn std::error::Error>> {
    const CLASH: Fragment = Fragment {
        source: "console-demo-clash-rest",
        paths: &[],
        schemas: &[NamedYaml {
            name: "TodoSummary",
            body: "type: string\n",
        }],
        external_schemas: &[],
    };
    let errors = compose(&[&TODOS, &CLASH])
        .err()
        .ok_or("must fail")?
        .duplicates;
    let rendered = errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        rendered.contains("TodoSummary")
            && rendered.contains("console-demo-todos-rest")
            && rendered.contains("console-demo-clash-rest"),
        "an operator must be able to find both owners from the message alone: {rendered}"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 2. Byte stability
// ---------------------------------------------------------------------------

#[test]
fn composition_is_byte_identical_across_two_runs() -> Result<(), Box<dyn std::error::Error>> {
    let first = compose(&[&TODOS, &NOTES])?;
    let second = compose(&[&TODOS, &NOTES])?;
    assert_eq!(
        first.as_bytes(),
        second.as_bytes(),
        "two runs over the same fragments must be byte-identical"
    );
    Ok(())
}

#[test]
fn composition_is_byte_identical_regardless_of_fragment_order()
-> Result<(), Box<dyn std::error::Error>> {
    let forward = compose(&[&TODOS, &NOTES])?;
    let reversed = compose(&[&NOTES, &TODOS])?;
    assert_eq!(
        forward, reversed,
        "output must be a function of the fragment SET, not the registration order"
    );
    // and the emitted keys are actually sorted, not merely stable
    let notes_at = forward
        .find("/api/v1/me/notes")
        .ok_or("notes path missing")?;
    let todos_at = forward
        .find("/api/v1/me/todos")
        .ok_or("todos path missing")?;
    assert!(notes_at < todos_at, "path keys must be sorted:\n{forward}");
    let note_schema = forward.find("NoteSummary").ok_or("NoteSummary missing")?;
    let todo_schema = forward.find("TodoSummary").ok_or("TodoSummary missing")?;
    assert!(
        note_schema < todo_schema,
        "schema keys must be sorted:\n{forward}"
    );
    Ok(())
}

#[test]
fn composition_is_byte_identical_regardless_of_author_indentation()
-> Result<(), Box<dyn std::error::Error>> {
    const FLUSH: Fragment = Fragment {
        source: "console-demo-indent-rest",
        paths: &[PathItem {
            path: "/api/v1/thing",
            operations: &[Operation {
                method: "get",
                body: "operationId: getThing\nresponses:\n  '200':\n    description: ok\n",
            }],
        }],
        schemas: &[],
        external_schemas: &[],
    };
    // Same YAML, authored inside an indented raw string with blank padding lines.
    const INDENTED: Fragment = Fragment {
        source: "console-demo-indent-rest",
        paths: &[PathItem {
            path: "/api/v1/thing",
            operations: &[Operation {
                method: "get",
                body: "\n            operationId: getThing   \n            responses:\n              '200':\n                description: ok\n\n",
            }],
        }],
        schemas: &[],
        external_schemas: &[],
    };

    assert_eq!(
        compose(&[&FLUSH])?,
        compose(&[&INDENTED])?,
        "leading indentation and trailing padding must not change the output bytes"
    );
    Ok(())
}

#[test]
fn composed_output_nests_operations_under_their_path() -> Result<(), Box<dyn std::error::Error>> {
    let out = compose(&[&TODOS])?;
    assert_eq!(
        out,
        concat!(
            "paths:\n",
            "  /api/v1/me/todos:\n",
            "    get:\n",
            "      operationId: listMyTodos\n",
            "      responses:\n",
            "        '200':\n",
            "          description: ok\n",
            "    post:\n",
            "      operationId: createMyTodo\n",
            "      responses:\n",
            "        '201':\n",
            "          description: created\n",
            "components:\n",
            "  schemas:\n",
            "    TodoSummary:\n",
            "      type: object\n",
        ),
        "composed document shape changed"
    );
    Ok(())
}

#[test]
fn components_block_is_omitted_when_no_fragment_contributes_a_schema()
-> Result<(), Box<dyn std::error::Error>> {
    const NO_SCHEMAS: Fragment = Fragment {
        source: "console-demo-bare-rest",
        paths: &[PathItem {
            path: "/api/v1/thing",
            operations: &[Operation {
                method: "get",
                body: "operationId: getThing\n",
            }],
        }],
        schemas: &[],
        external_schemas: &[],
    };
    let out = compose(&[&NO_SCHEMAS])?;
    assert!(
        !out.contains("components:"),
        "an empty components block is not valid OpenAPI: {out}"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 3. Referential integrity
// ---------------------------------------------------------------------------

/// Refs `Missing`, defines nothing, declares nothing external.
const DANGLING: Fragment = Fragment {
    source: "console-demo-dangling-rest",
    paths: &[PathItem {
        path: "/api/v1/thing",
        operations: &[Operation {
            method: "get",
            body: "responses:\n  '200':\n    content:\n      application/json:\n        schema:\n          $ref: '#/components/schemas/Missing'\n",
        }],
    }],
    schemas: &[],
    external_schemas: &[],
};

#[test]
fn a_ref_to_a_schema_nobody_defines_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let error = compose(&[&DANGLING])
        .err()
        .ok_or("a $ref to a schema no fragment defines and none declares external must fail")?;
    let dangling = error
        .dangling
        .first()
        .ok_or_else(|| format!("expected a DanglingRef, got {error:#?}"))?;
    assert_eq!(dangling.schema, "Missing");
    assert_eq!(dangling.source, "console-demo-dangling-rest");
    assert!(
        error.to_string().contains("Missing"),
        "an operator must find the unresolved name in the message: {error}"
    );
    Ok(())
}

#[test]
fn a_ref_a_fragment_declares_external_is_not_dangling() -> Result<(), Box<dyn std::error::Error>> {
    const BORROWS: Fragment = Fragment {
        external_schemas: &["Missing"],
        ..DANGLING
    };
    compose(&[&BORROWS])?;
    Ok(())
}

#[test]
fn a_ref_another_fragment_defines_is_not_dangling() -> Result<(), Box<dyn std::error::Error>> {
    const DEFINES_MISSING: Fragment = Fragment {
        source: "console-demo-defines-rest",
        paths: &[],
        schemas: &[NamedYaml {
            name: "Missing",
            body: "type: object\n",
        }],
        external_schemas: &[],
    };
    compose(&[&DANGLING, &DEFINES_MISSING])?;
    Ok(())
}

#[test]
fn a_ref_from_a_schema_body_is_checked_too() -> Result<(), Box<dyn std::error::Error>> {
    // Schema-to-schema refs are how a dropped leaf type hides: no path mentions
    // it, only the parent schema does.
    const SCHEMA_REF: Fragment = Fragment {
        source: "console-demo-schema-ref-rest",
        paths: &[],
        schemas: &[NamedYaml {
            name: "Page",
            body: "type: object\nproperties:\n  items:\n    $ref: '#/components/schemas/Item'\n",
        }],
        external_schemas: &[],
    };
    let error = compose(&[&SCHEMA_REF])
        .err()
        .ok_or("a schema body refing an undefined schema must fail")?;
    assert_eq!(
        error.dangling.first().map(|d| d.schema.as_str()),
        Some("Item"),
        "got {error:#?}"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 4. `$ref` targets are parsed whole
//
// OpenAPI component keys match `^[a-zA-Z0-9._-]+$`, so a ref target truncated
// at the first character outside `[A-Za-z0-9_]` is a different name than the
// one written. Every test below is a fail-open: the truncated prefix resolves,
// so the guard passes on a ref it should reject.
// ---------------------------------------------------------------------------

#[test]
fn a_dangling_ref_is_rejected_even_when_a_prefix_of_its_name_resolves()
-> Result<(), Box<dyn std::error::Error>> {
    // `Todo` exists; `Todo-Summary` does not. Truncating at `-` resolves the
    // wrong name and the deleted schema is silent again.
    const PREFIX_TRAP: Fragment = Fragment {
        source: "console-demo-prefix-rest",
        paths: &[],
        schemas: &[
            NamedYaml {
                name: "Todo",
                body: "type: object\n",
            },
            NamedYaml {
                name: "Page",
                body: "type: object\nproperties:\n  item:\n    $ref: '#/components/schemas/Todo-Summary'\n",
            },
        ],
        external_schemas: &[],
    };

    let error = compose(&[&PREFIX_TRAP])
        .err()
        .ok_or("a ref to Todo-Summary must not resolve just because Todo exists")?;
    assert_eq!(
        error.dangling.first().map(|d| d.schema.as_str()),
        Some("Todo-Summary"),
        "got {error:#?}"
    );
    Ok(())
}

#[test]
fn a_ref_whose_name_contains_legal_punctuation_resolves() -> Result<(), Box<dyn std::error::Error>>
{
    // `.`, `-` and `_` are all legal in a component key. Truncation turns this
    // resolvable ref into a false DanglingRef, which is the same bug pointing
    // the other way.
    const PUNCTUATED: Fragment = Fragment {
        source: "console-demo-punct-rest",
        paths: &[],
        schemas: &[
            NamedYaml {
                name: "Todo.v2-summary_1",
                body: "type: object\n",
            },
            NamedYaml {
                name: "Page",
                body: "type: object\nproperties:\n  item:\n    $ref: '#/components/schemas/Todo.v2-summary_1'\n",
            },
        ],
        external_schemas: &[],
    };
    compose(&[&PUNCTUATED])?;
    Ok(())
}

#[test]
fn a_ref_target_that_is_not_a_component_key_is_rejected() -> Result<(), Box<dyn std::error::Error>>
{
    // A ref into a nested pointer, truncated at `/`, resolves as `Todo`.
    const NESTED: Fragment = Fragment {
        source: "console-demo-nested-rest",
        paths: &[],
        schemas: &[
            NamedYaml {
                name: "Todo",
                body: "type: object\n",
            },
            NamedYaml {
                name: "Page",
                body: "type: object\nproperties:\n  item:\n    $ref: '#/components/schemas/Todo/properties/id'\n",
            },
        ],
        external_schemas: &[],
    };
    let error = compose(&[&NESTED])
        .err()
        .ok_or("a ref target that is not a component key must be rejected")?;
    let malformed = error
        .malformed
        .first()
        .ok_or_else(|| format!("expected a MalformedRef, got {error:#?}"))?;
    assert_eq!(malformed.target, "Todo/properties/id");
    assert_eq!(malformed.source, "console-demo-nested-rest");
    assert!(
        error.to_string().contains("Todo/properties/id"),
        "an operator must find the bad target in the message: {error}"
    );
    Ok(())
}

#[test]
fn an_empty_ref_target_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    const EMPTY: Fragment = Fragment {
        source: "console-demo-empty-rest",
        paths: &[],
        schemas: &[NamedYaml {
            name: "Page",
            body: "type: object\nproperties:\n  item:\n    $ref: '#/components/schemas/'\n",
        }],
        external_schemas: &[],
    };
    let error = compose(&[&EMPTY])
        .err()
        .ok_or("a ref with no target must be rejected")?;
    assert_eq!(
        error.malformed.first().map(|m| m.target.as_str()),
        Some(""),
        "got {error:#?}"
    );
    Ok(())
}

#[test]
fn a_ref_is_terminated_by_the_yaml_delimiter_that_follows_it()
-> Result<(), Box<dyn std::error::Error>> {
    // Double-quoted, and inside a flow sequence: both must yield exactly `Item`,
    // not `Item"` / `Item}`.
    const DELIMITED: Fragment = Fragment {
        source: "console-demo-delim-rest",
        paths: &[],
        schemas: &[
            NamedYaml {
                name: "Item",
                body: "type: object\n",
            },
            NamedYaml {
                name: "Page",
                body: "type: object\nproperties:\n  a:\n    $ref: \"#/components/schemas/Item\"\n  b:\n    allOf: [{$ref: '#/components/schemas/Item'}]\n",
            },
        ],
        external_schemas: &[],
    };
    compose(&[&DELIMITED])?;
    Ok(())
}
