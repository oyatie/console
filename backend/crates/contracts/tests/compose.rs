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

use console_contracts::{
    DuplicateKind, Fragment, NamedYaml, Operation, PathItem, compose, schema_refs,
};

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
    let unresolvable = error
        .unresolvable
        .first()
        .ok_or_else(|| format!("expected an UnresolvableRef, got {error:#?}"))?;
    assert_eq!(
        unresolvable.value,
        "#/components/schemas/Todo/properties/id"
    );
    assert_eq!(unresolvable.source, "console-demo-nested-rest");
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
        error.unresolvable.first().map(|r| r.value.as_str()),
        Some("#/components/schemas/"),
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

// ---------------------------------------------------------------------------
// Refs into component sections other than `schemas` (console-qjb)
// ---------------------------------------------------------------------------

/// A `$ref` into a section OpenAPI does not define resolves in NO document, so
/// it must not compose.
///
/// `#/components/schema/Todo` is a one-character typo of `schemas`. It is not a
/// ref this crate declines to resolve — it is a ref nothing can ever resolve,
/// and before this was checked it composed clean and shipped into the published
/// document.
#[test]
fn a_ref_into_a_component_section_openapi_does_not_define_is_rejected()
-> Result<(), Box<dyn std::error::Error>> {
    const TYPO: Fragment = Fragment {
        source: "console-demo-typo-rest",
        paths: &[PathItem {
            path: "/api/v1/demo",
            operations: &[Operation {
                method: "get",
                body: "responses:\n  '200':\n    content:\n      application/json:\n        schema:\n          $ref: '#/components/schema/Todo'\n",
            }],
        }],
        schemas: &[],
        external_schemas: &[],
    };
    let error = compose(&[&TYPO])
        .err()
        .ok_or("a ref into a section OpenAPI does not define must be rejected")?;
    assert_eq!(
        error.unresolvable.first().map(|r| r.value.as_str()),
        Some("#/components/schema/Todo"),
        "got {error:#?}"
    );
    assert_eq!(
        error.unresolvable.first().map(|r| r.source),
        Some("console-demo-typo-rest"),
        "got {error:#?}"
    );
    assert!(
        error.to_string().contains("#/components/schema/Todo"),
        "an operator must find the bad ref in the message: {error}"
    );
    Ok(())
}

/// The converse, and the one that is load-bearing in production: a ref into a
/// component section OpenAPI DOES define is legitimate and must still compose.
///
/// `backend/crates/todos/rest/src/openapi.rs` ships exactly these refs and
/// `backend/openapi/openapi.yaml` defines them under `components/responses` and
/// `components/parameters`. `Fragment` models only schemas, so the published
/// document — not the fragment set — is what resolves them. Rejecting them here
/// would break the shipping todos face, so this test is the guard against
/// "fixing" the section check by making it total over everything but `schemas`.
#[test]
fn a_ref_into_a_real_non_schema_component_section_composes()
-> Result<(), Box<dyn std::error::Error>> {
    const BORROWED: Fragment = Fragment {
        source: "console-demo-sections-rest",
        paths: &[PathItem {
            path: "/api/v1/demo",
            operations: &[Operation {
                method: "get",
                body: "parameters:\n  - $ref: '#/components/parameters/BranchId'\nresponses:\n  '401':\n    $ref: '#/components/responses/Unauthorized'\n",
            }],
        }],
        schemas: &[],
        external_schemas: &[],
    };
    compose(&[&BORROWED])?;
    Ok(())
}

/// A `$ref` carrying a file-or-URL prefix is rejected EVEN WHEN the local set
/// knows the trailing name.
///
/// `compose` emits one document, so `common.yaml#/components/schemas/Uuid`
/// cannot be satisfied by it however the local schemas are arranged. Both
/// fixtures below make the trailing name resolvable — one via
/// `external_schemas`, which is the shipping todos face's exact shape, one by
/// defining `Uuid` locally — because a check that only catches the prefix when
/// the SUFFIX also happens to be unknown is not checking the prefix at all. The
/// URL case is the sharp end: a validated ref that instructs every generated
/// client to fetch a foreign host.
#[test]
fn a_cross_file_ref_is_rejected_even_when_the_local_set_knows_the_name()
-> Result<(), Box<dyn std::error::Error>> {
    const BORROWED_NAME: Fragment = Fragment {
        source: "console-demo-foreign-rest",
        paths: &[],
        schemas: &[NamedYaml {
            name: "Page",
            body: "type: object\nproperties:\n  id:\n    $ref: 'common.yaml#/components/schemas/Uuid'\n",
        }],
        external_schemas: &["Uuid"],
    };
    const FOREIGN_HOST: Fragment = Fragment {
        source: "console-demo-foreign-host-rest",
        paths: &[],
        schemas: &[
            NamedYaml {
                name: "Uuid",
                body: "type: string\n",
            },
            NamedYaml {
                name: "Page",
                body: "type: object\nproperties:\n  id:\n    $ref: 'https://evil.example/x.yaml#/components/schemas/Uuid'\n",
            },
        ],
        external_schemas: &[],
    };
    for (fragment, value) in [
        (&BORROWED_NAME, "common.yaml#/components/schemas/Uuid"),
        (
            &FOREIGN_HOST,
            "https://evil.example/x.yaml#/components/schemas/Uuid",
        ),
    ] {
        let error = compose(&[fragment])
            .err()
            .ok_or("a ref into another file must not compose against the local set")?;
        assert_eq!(
            error.unresolvable.first().map(|r| r.value.as_str()),
            Some(value),
            "got {error:#?}"
        );
    }
    Ok(())
}

/// A `$ref` that stops before naming a target resolves in no document, in EVERY
/// section — not just in `schemas`.
///
/// `#/components/responses` is an author deleting `/Unauthorized` mid-edit. It
/// points at the responses MAP, not at a Response Object, so nothing can
/// resolve it; the `schemas` twin of this input is already rejected by
/// `an_empty_ref_target_is_rejected`.
#[test]
fn a_ref_truncated_before_its_target_is_rejected_in_every_section()
-> Result<(), Box<dyn std::error::Error>> {
    const TRUNCATED: Fragment = Fragment {
        source: "console-demo-truncated-rest",
        paths: &[PathItem {
            path: "/api/v1/demo",
            operations: &[Operation {
                method: "get",
                body: "responses:\n  '401':\n    $ref: '#/components/responses'\n",
            }],
        }],
        schemas: &[],
        external_schemas: &[],
    };
    let error = compose(&[&TRUNCATED])
        .err()
        .ok_or("a ref naming a component SECTION but no component must be rejected")?;
    assert_eq!(
        error.unresolvable.first().map(|r| r.value.as_str()),
        Some("#/components/responses"),
        "got {error:#?}"
    );
    assert_eq!(
        error.unresolvable.first().map(|r| r.source),
        Some("console-demo-truncated-rest"),
        "got {error:#?}"
    );
    Ok(())
}

/// A `$ref` value that is not a `#/components/…` pointer at all is rejected.
///
/// Each of these composed clean while the check split on the literal
/// `#/components/`: the typo is one segment to the LEFT of the section name, or
/// the pointer is a different dialect entirely, so the substring the check
/// looked for simply is not there and the ref was invisible rather than
/// checked. `#/definitions/…` is the Swagger 2.0 / JSON Schema spelling and the
/// likeliest paste-in error.
#[test]
fn a_ref_that_is_not_a_components_pointer_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    // `Todo` IS defined below, so nothing here is rejected for being unknown —
    // only for not being a pointer the composed document can follow.
    const DIALECTS: [(&str, &[PathItem]); 4] = [
        (
            "#/componentss/schemas/Todo",
            &[PathItem {
                path: "/api/v1/demo",
                operations: &[Operation {
                    method: "get",
                    body: "responses:\n  '200':\n    $ref: '#/componentss/schemas/Todo'\n",
                }],
            }],
        ),
        (
            "#/component/schemas/Todo",
            &[PathItem {
                path: "/api/v1/demo",
                operations: &[Operation {
                    method: "get",
                    body: "responses:\n  '200':\n    $ref: '#/component/schemas/Todo'\n",
                }],
            }],
        ),
        (
            "#/definitions/Todo",
            &[PathItem {
                path: "/api/v1/demo",
                operations: &[Operation {
                    method: "get",
                    body: "responses:\n  '200':\n    $ref: '#/definitions/Todo'\n",
                }],
            }],
        ),
        (
            "Todo.yaml",
            &[PathItem {
                path: "/api/v1/demo",
                operations: &[Operation {
                    method: "get",
                    body: "responses:\n  '200':\n    $ref: 'Todo.yaml'\n",
                }],
            }],
        ),
    ];
    for (value, paths) in DIALECTS {
        let fragment = Fragment {
            source: "console-demo-dialect-rest",
            paths,
            schemas: &[NamedYaml {
                name: "Todo",
                body: "type: object\n",
            }],
            external_schemas: &[],
        };
        let error = compose(&[&fragment])
            .err()
            .ok_or_else(|| format!("`{value}` is not a component pointer and must be rejected"))?;
        assert_eq!(
            error.unresolvable.first().map(|r| r.value.as_str()),
            Some(value),
            "got {error:#?}"
        );
    }
    Ok(())
}

/// `schema_refs` yields schema refs and nothing else — including for a ref into
/// a real non-schema section, which is a component reference but not a schema
/// one.
///
/// This is a MECHANISM test over the parser, and it pins the boundary that
/// `console_todos_rest`'s drift test inherits: that test roots a transitive
/// `$ref` closure at the published path blocks and walks it with `schema_refs`,
/// so the closure STOPS at a non-schema component and never enters its body. No
/// schema is missed today only because every published response reaches
/// `ErrorBody`, which the todos paths also ref directly.
#[test]
fn schema_refs_yields_schema_refs_only() {
    let body = "parameters:\n  - $ref: '#/components/parameters/BranchId'\nresponses:\n  '401':\n    $ref: '#/components/responses/Unauthorized'\n  '200':\n    schema:\n      $ref: '#/components/schemas/TodoPage'\n";
    assert_eq!(
        schema_refs(body).collect::<Vec<_>>(),
        vec!["TodoPage"],
        "schema_refs must see the schema ref and neither of the two \
         non-schema component refs"
    );
}

/// `schema_refs` follows a `discriminator.mapping`, because a mapping entry is
/// a schema edge like any other.
///
/// Asserted separately from `compose` because `schema_refs` is PUBLIC and is
/// what `console_todos_rest`'s drift test walks the published document with:
/// while the parser keyed on `$ref:`, a subtype reachable only through a
/// mapping dropped out of that closure and the drift oracle shrank silently.
#[test]
fn schema_refs_follows_a_discriminator_mapping() {
    let body = "oneOf:\n- $ref: '#/components/schemas/Ok'\ndiscriminator:\n  propertyName: kind\n  mapping:\n    ok: '#/components/schemas/Ok'\n    gone: '#/components/schemas/Subtype'\n";
    assert_eq!(
        schema_refs(body).collect::<Vec<_>>(),
        vec!["Ok", "Ok", "Subtype"],
        "a subtype named only by a mapping is still a schema this document needs"
    );
}

// ---------------------------------------------------------------------------
// 5. A pointer is checked wherever it is written, not only after `$ref:`
//
// `$ref` is a KEY, and a key has more than one legal spelling; a pointer is
// also written where there is no `$ref` key at all. Keying the check on the
// literal text `$ref:` made every other position invisible rather than
// checked, which is the same fail-open as keying it on `#/components/`.
// ---------------------------------------------------------------------------

/// Bodies below all point at `Ghost`, which no fragment defines and none
/// declares external. Every one must be a [`DanglingRef`].
///
/// `mapping` is the load-bearing entry: an OpenAPI `discriminator.mapping`
/// value is a pointer with NO `$ref` key anywhere near it, and
/// `backend/openapi/openapi.yaml` publishes 27 of them today. The rest are
/// spellings of the `$ref` KEY that YAML and JSON-in-YAML both accept.
const GHOST_POINTERS: [(&str, &str); 6] = [
    (
        "discriminator mapping",
        "oneOf:\n- $ref: '#/components/schemas/Ok'\ndiscriminator:\n  propertyName: kind\n  mapping:\n    gone: '#/components/schemas/Ghost'\n",
    ),
    (
        "double-quoted key",
        "type: object\nproperties:\n  id:\n    \"$ref\": '#/components/schemas/Ghost'\n",
    ),
    (
        "single-quoted key",
        "type: object\nproperties:\n  id:\n    '$ref': '#/components/schemas/Ghost'\n",
    ),
    (
        "space before the colon",
        "type: object\nproperties:\n  id:\n    $ref : '#/components/schemas/Ghost'\n",
    ),
    (
        "JSON-in-YAML flow mapping",
        "type: object\nproperties:\n  id: {\"$ref\": \"#/components/schemas/Ghost\"}\n",
    ),
    (
        "value on the next line",
        "type: object\nproperties:\n  id:\n    $ref:\n      '#/components/schemas/Ghost'\n",
    ),
];

/// A fragment defining `Ok` plus an `Envelope` carrying `body`.
///
/// Leaked because `Fragment` is `&'static` end to end — the composer's real
/// inputs are compile-time tables — while a corpus test derives its bodies at
/// runtime so both directions are driven from ONE list of spellings.
fn envelope_fragment(source: &'static str, body: &'static str) -> Fragment {
    Fragment {
        source,
        paths: &[],
        schemas: Box::leak(Box::new([
            NamedYaml {
                name: "Ok",
                body: "type: object\n",
            },
            NamedYaml {
                name: "Envelope",
                body,
            },
        ])),
        external_schemas: &[],
    }
}

#[test]
fn a_pointer_to_an_undefined_schema_is_rejected_in_every_position()
-> Result<(), Box<dyn std::error::Error>> {
    for (spelling, body) in GHOST_POINTERS {
        let fragment = envelope_fragment("console-demo-position-rest", body);
        let error = compose(&[&fragment]).err().ok_or_else(|| {
            format!("a pointer to Ghost written as `{spelling}` must not compose")
        })?;
        assert_eq!(
            error.dangling.first().map(|d| d.schema.as_str()),
            Some("Ghost"),
            "`{spelling}`: got {error:#?}"
        );
    }
    Ok(())
}

/// The same six positions, resolving this time: a pointer the fragment set CAN
/// satisfy must compose from every one of them.
///
/// Without this the check above is satisfied by rejecting everything, and a
/// face writing a discriminated union — or a `$ref` value on its own line,
/// which is legal YAML — gets a composition failure for a valid document.
#[test]
fn a_resolving_pointer_composes_from_every_position() -> Result<(), Box<dyn std::error::Error>> {
    for (spelling, ghost_body) in GHOST_POINTERS {
        let body: &'static str = Box::leak(ghost_body.replace("Ghost", "Ok").into_boxed_str());
        let fragment = envelope_fragment("console-demo-position-ok-rest", body);
        compose(&[&fragment])
            .map_err(|error| format!("`{spelling}` points at the defined `Ok`: {error}"))?;
    }
    Ok(())
}

/// A file-or-URL prefix is rejected wherever the pointer sits, not only after a
/// `$ref` key.
///
/// `a_cross_file_ref_is_rejected_even_when_the_local_set_knows_the_name` pins
/// this for the `$ref:` position and calls the URL case "the sharp end". The
/// identical value one key over, in a `discriminator.mapping`, instructs every
/// generated client to resolve that subtype against a foreign host — so it is
/// the same defect and must be the same error.
#[test]
fn a_foreign_host_pointer_is_rejected_outside_a_ref_key() -> Result<(), Box<dyn std::error::Error>>
{
    const MAPPED_FOREIGN: Fragment = Fragment {
        source: "console-demo-foreign-mapping-rest",
        paths: &[],
        schemas: &[
            NamedYaml {
                name: "Ok",
                body: "type: object\n",
            },
            NamedYaml {
                name: "Envelope",
                body: "oneOf:\n- $ref: '#/components/schemas/Ok'\ndiscriminator:\n  propertyName: kind\n  mapping:\n    evil: 'https://evil.example/x.yaml#/components/schemas/Ok'\n",
            },
        ],
        external_schemas: &[],
    };
    let error = compose(&[&MAPPED_FOREIGN])
        .err()
        .ok_or("a foreign-host pointer in a discriminator mapping must not compose")?;
    assert_eq!(
        error.unresolvable.first().map(|r| r.value.as_str()),
        Some("https://evil.example/x.yaml#/components/schemas/Ok"),
        "got {error:#?}"
    );
    Ok(())
}

/// The characters `$ref:` inside a description are prose, not a reference.
///
/// A check keyed on the literal text `$ref:` reports this as an unfollowable
/// ref and a face author gets a composition failure for writing documentation.
#[test]
fn prose_that_mentions_a_ref_key_is_not_a_ref() -> Result<(), Box<dyn std::error::Error>> {
    const PROSE: Fragment = Fragment {
        source: "console-demo-prose-rest",
        paths: &[],
        schemas: &[NamedYaml {
            name: "Page",
            body: "type: object\ndescription: 'authors write $ref: pointers here'\n",
        }],
        external_schemas: &[],
    };
    compose(&[&PROSE])?;
    Ok(())
}
