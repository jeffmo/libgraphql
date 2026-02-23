//! Tests for [`crate::ast::Document`] and
//! [`crate::ast::DocumentSyntax`].

use crate::ast::Definition;
use crate::ast::DirectiveAnnotation;
use crate::ast::DirectiveDefinition;
use crate::ast::DirectiveLocation;
use crate::ast::DirectiveLocationKind;
use crate::ast::Document;
use crate::ast::FragmentDefinition;
use crate::ast::ObjectTypeDefinition;
use crate::ast::ObjectTypeExtension;
use crate::ast::OperationDefinition;
use crate::ast::OperationKind;
use crate::ast::SchemaDefinition;
use crate::ast::SchemaExtension;
use crate::ast::SelectionSet;
use crate::ast::TypeCondition;
use crate::ast::TypeDefinition;
use crate::ast::TypeExtension;
use crate::ast::tests::ast_test_utils::make_byte_span;
use crate::ast::tests::ast_test_utils::make_name;

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
        span: make_byte_span(0, 28),
        definitions: vec![
            Definition::TypeDefinition(
                TypeDefinition::Object(
                    ObjectTypeDefinition {
                        span: make_byte_span(0, 28),
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
        span: make_byte_span(0, 100),
        definitions: vec![
            // Type-system definition
            Definition::TypeDefinition(
                TypeDefinition::Scalar(
                    crate::ast::ScalarTypeDefinition {
                        span: make_byte_span(0, 10),
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
                    span: make_byte_span(11, 30),
                    description: None,
                    operation_kind:
                        OperationKind::Query,
                    name: None,
                    variable_definitions: vec![],
                    directives: vec![],
                    selection_set: SelectionSet {
                        span: make_byte_span(17, 30),
                        selections: vec![],
                        syntax: None,
                    },
                    shorthand: false,
                    syntax: None,
                },
            ),
            // Schema definition
            Definition::SchemaDefinition(
                SchemaDefinition {
                    span: make_byte_span(31, 60),
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
        span: make_byte_span(0, 100),
        definitions: vec![
            // Scalar type (not executable)
            Definition::TypeDefinition(
                TypeDefinition::Scalar(
                    crate::ast::ScalarTypeDefinition {
                        span: make_byte_span(0, 10),
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
                    span: make_byte_span(11, 30),
                    description: None,
                    operation_kind:
                        OperationKind::Query,
                    name: None,
                    variable_definitions: vec![],
                    directives: vec![],
                    selection_set: SelectionSet {
                        span: make_byte_span(17, 30),
                        selections: vec![],
                        syntax: None,
                    },
                    shorthand: false,
                    syntax: None,
                },
            ),
            // Fragment (executable)
            Definition::FragmentDefinition(
                FragmentDefinition {
                    span: make_byte_span(31, 60),
                    description: None,
                    name: make_name(
                        "F", 40, 41,
                    ),
                    type_condition:
                        TypeCondition {
                            span: make_byte_span(
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
                            span: make_byte_span(
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
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Document
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn document_empty_definitions() {
    let doc = Document {
        span: make_byte_span(0, 0),
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

/// Verify that `schema_definitions()` and
/// `executable_definitions()` correctly classify all 7
/// `Definition` variants. Schema-type variants:
/// TypeDefinition, SchemaDefinition,
/// DirectiveDefinition, SchemaExtension,
/// TypeExtension. Executable variants:
/// OperationDefinition, FragmentDefinition.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Document
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn document_filter_all_variants() {
    let doc = Document {
        span: make_byte_span(0, 200),
        definitions: vec![
            // Schema-type: TypeDefinition
            Definition::TypeDefinition(
                TypeDefinition::Scalar(
                    crate::ast::ScalarTypeDefinition {
                        span: make_byte_span(
                            0, 10,
                        ),
                        description: None,
                        name: make_name(
                            "URL", 7, 10,
                        ),
                        directives: vec![],
                        syntax: None,
                    },
                ),
            ),
            // Schema-type: SchemaDefinition
            Definition::SchemaDefinition(
                SchemaDefinition {
                    span: make_byte_span(
                        11, 30,
                    ),
                    description: None,
                    directives: vec![],
                    root_operations: vec![],
                    syntax: None,
                },
            ),
            // Schema-type: DirectiveDefinition
            Definition::DirectiveDefinition(
                DirectiveDefinition {
                    span: make_byte_span(
                        31, 55,
                    ),
                    description: None,
                    name: make_name(
                        "skip", 42, 46,
                    ),
                    arguments: vec![],
                    repeatable: false,
                    locations: vec![
                        DirectiveLocation {
                            kind: DirectiveLocationKind::Field,
                            span: make_byte_span(
                                50, 55,
                            ),
                            syntax: None,
                        },
                    ],
                    syntax: None,
                },
            ),
            // Schema-type: SchemaExtension
            Definition::SchemaExtension(
                SchemaExtension {
                    span: make_byte_span(
                        56, 75,
                    ),
                    directives: vec![
                        DirectiveAnnotation {
                            span: make_byte_span(
                                70, 75,
                            ),
                            name: make_name(
                                "auth",
                                71, 75,
                            ),
                            arguments: vec![],
                            syntax: None,
                        },
                    ],
                    root_operations: vec![],
                    syntax: None,
                },
            ),
            // Schema-type: TypeExtension
            Definition::TypeExtension(
                TypeExtension::Object(
                    ObjectTypeExtension {
                        span: make_byte_span(
                            76, 100,
                        ),
                        name: make_name(
                            "Q", 88, 89,
                        ),
                        implements: vec![],
                        directives: vec![],
                        fields: vec![],
                        syntax: None,
                    },
                ),
            ),
            // Executable: OperationDefinition
            Definition::OperationDefinition(
                OperationDefinition {
                    span: make_byte_span(
                        101, 120,
                    ),
                    description: None,
                    operation_kind:
                        OperationKind::Query,
                    name: None,
                    variable_definitions: vec![],
                    directives: vec![],
                    selection_set: SelectionSet {
                        span: make_byte_span(
                            107, 120,
                        ),
                        selections: vec![],
                        syntax: None,
                    },
                    shorthand: false,
                    syntax: None,
                },
            ),
            // Executable: FragmentDefinition
            Definition::FragmentDefinition(
                FragmentDefinition {
                    span: make_byte_span(
                        121, 150,
                    ),
                    description: None,
                    name: make_name(
                        "F", 130, 131,
                    ),
                    type_condition: TypeCondition
                    {
                        span: make_byte_span(
                            132, 140,
                        ),
                        named_type: make_name(
                            "User", 135, 139,
                        ),
                        syntax: None,
                    },
                    directives: vec![],
                    selection_set: SelectionSet {
                        span: make_byte_span(
                            141, 150,
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

    let schema: Vec<_> =
        doc.schema_definitions().collect();
    assert_eq!(
        schema.len(),
        5,
        "5 schema-type definitions expected",
    );
    assert!(matches!(
        schema[0],
        Definition::TypeDefinition(_),
    ));
    assert!(matches!(
        schema[1],
        Definition::SchemaDefinition(_),
    ));
    assert!(matches!(
        schema[2],
        Definition::DirectiveDefinition(_),
    ));
    assert!(matches!(
        schema[3],
        Definition::SchemaExtension(_),
    ));
    assert!(matches!(
        schema[4],
        Definition::TypeExtension(_),
    ));

    let exec: Vec<_> =
        doc.executable_definitions().collect();
    assert_eq!(
        exec.len(),
        2,
        "2 executable definitions expected",
    );
    assert!(matches!(
        exec[0],
        Definition::OperationDefinition(_),
    ));
    assert!(matches!(
        exec[1],
        Definition::FragmentDefinition(_),
    ));
}
