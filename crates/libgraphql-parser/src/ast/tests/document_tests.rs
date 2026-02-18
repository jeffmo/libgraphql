//! Tests for [`crate::ast::Document`] and
//! [`crate::ast::DocumentSyntax`].

use crate::ast::Definition;
use crate::ast::Document;
use crate::ast::FragmentDefinition;
use crate::ast::ObjectTypeDefinition;
use crate::ast::OperationDefinition;
use crate::ast::OperationKind;
use crate::ast::SchemaDefinition;
use crate::ast::SelectionSet;
use crate::ast::TypeCondition;
use crate::ast::TypeDefinition;
use crate::ast::tests::ast_test_helpers::make_name;
use crate::ast::tests::ast_test_helpers::make_span;

/// Verify `Document` stores definitions and
/// `append_source` slices the entire document.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Document
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn document_construct_and_source_slice() {
    let source =
        "type Query { hello: String }";
    let doc = Document {
        span: make_span(0, 28),
        definitions: vec![
            Definition::TypeDefinition(
                TypeDefinition::Object(
                    ObjectTypeDefinition {
                        span: make_span(0, 28),
                        description: None,
                        name: make_name(
                            "Query", 5, 10,
                        ),
                        implements: vec![],
                        directives: vec![],
                        fields: vec![],
                        syntax: None,
                    },
                ),
            ),
        ],
        syntax: None,
    };
    assert_eq!(doc.definitions.len(), 1);

    let mut sink = String::new();
    doc.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `Document::append_source` with `source = None`
/// is a no-op.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn document_source_none_is_noop() {
    let doc = Document {
        span: make_span(0, 10),
        definitions: vec![],
        syntax: None,
    };
    let mut sink = String::new();
    doc.append_source(&mut sink, None);
    assert_eq!(sink, "");
}

/// Verify `Document::schema_definitions()` returns only
/// type-system definitions and extensions, filtering
/// out executable definitions.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Document
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn document_schema_definitions_filter() {
    let doc = Document {
        span: make_span(0, 100),
        definitions: vec![
            // Type-system definition
            Definition::TypeDefinition(
                TypeDefinition::Scalar(
                    crate::ast::ScalarTypeDefinition {
                        span: make_span(0, 10),
                        description: None,
                        name: make_name(
                            "URL", 7, 10,
                        ),
                        directives: vec![],
                        syntax: None,
                    },
                ),
            ),
            // Executable definition (operation)
            Definition::OperationDefinition(
                OperationDefinition {
                    span: make_span(11, 30),
                    description: None,
                    operation_kind:
                        OperationKind::Query,
                    name: None,
                    variable_definitions: vec![],
                    directives: vec![],
                    selection_set: SelectionSet {
                        span: make_span(17, 30),
                        selections: vec![],
                        syntax: None,
                    },
                    syntax: None,
                },
            ),
            // Schema definition
            Definition::SchemaDefinition(
                SchemaDefinition {
                    span: make_span(31, 60),
                    description: None,
                    directives: vec![],
                    root_operations: vec![],
                    syntax: None,
                },
            ),
        ],
        syntax: None,
    };

    let schema_defs: Vec<_> =
        doc.schema_definitions().collect();
    assert_eq!(
        schema_defs.len(),
        2,
        "Should have 2 schema definitions \
         (scalar + schema), not the operation",
    );

    // Verify the filter kept the right ones
    assert!(matches!(
        schema_defs[0],
        Definition::TypeDefinition(_),
    ));
    assert!(matches!(
        schema_defs[1],
        Definition::SchemaDefinition(_),
    ));
}

/// Verify `Document::executable_definitions()` returns
/// only operations and fragments, filtering out
/// type-system definitions.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#ExecutableDocument
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn document_executable_definitions_filter() {
    let doc = Document {
        span: make_span(0, 100),
        definitions: vec![
            // Scalar type (not executable)
            Definition::TypeDefinition(
                TypeDefinition::Scalar(
                    crate::ast::ScalarTypeDefinition {
                        span: make_span(0, 10),
                        description: None,
                        name: make_name(
                            "URL", 7, 10,
                        ),
                        directives: vec![],
                        syntax: None,
                    },
                ),
            ),
            // Operation (executable)
            Definition::OperationDefinition(
                OperationDefinition {
                    span: make_span(11, 30),
                    description: None,
                    operation_kind:
                        OperationKind::Query,
                    name: None,
                    variable_definitions: vec![],
                    directives: vec![],
                    selection_set: SelectionSet {
                        span: make_span(17, 30),
                        selections: vec![],
                        syntax: None,
                    },
                    syntax: None,
                },
            ),
            // Fragment (executable)
            Definition::FragmentDefinition(
                FragmentDefinition {
                    span: make_span(31, 60),
                    description: None,
                    name: make_name(
                        "F", 40, 41,
                    ),
                    type_condition:
                        TypeCondition {
                            span: make_span(
                                42, 49,
                            ),
                            named_type:
                                make_name(
                                    "User",
                                    45, 49,
                                ),
                            syntax: None,
                        },
                    directives: vec![],
                    selection_set:
                        SelectionSet {
                            span: make_span(
                                50, 60,
                            ),
                            selections: vec![],
                            syntax: None,
                        },
                    syntax: None,
                },
            ),
        ],
        syntax: None,
    };

    let exec_defs: Vec<_> =
        doc.executable_definitions().collect();
    assert_eq!(
        exec_defs.len(),
        2,
        "Should have 2 executable definitions \
         (operation + fragment), not the scalar",
    );

    assert!(matches!(
        exec_defs[0],
        Definition::OperationDefinition(_),
    ));
    assert!(matches!(
        exec_defs[1],
        Definition::FragmentDefinition(_),
    ));
}

/// Verify that `Document` with no definitions returns
/// empty iterators for both `schema_definitions()` and
/// `executable_definitions()`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn document_empty_definitions() {
    let doc = Document {
        span: make_span(0, 0),
        definitions: vec![],
        syntax: None,
    };
    assert_eq!(
        doc.schema_definitions().count(),
        0,
    );
    assert_eq!(
        doc.executable_definitions().count(),
        0,
    );
}
