use std::borrow::Cow;

use crate::ast;
use crate::ast::tests::ast_test_utils::make_name;
use crate::ast::tests::ast_test_utils::make_span;
use crate::ast::tests::ast_test_utils::zero_span;
use crate::compat_graphql_parser_v0_4::to_graphql_parser_schema_ast;

use graphql_parser::schema::Definition as GpDef;
use graphql_parser::schema::TypeDefinition as GpTd;
use graphql_parser::schema::TypeExtension as GpTe;

/// Shorthand for constructing a 1-based
/// `graphql_parser::Pos`.
fn pos(
    line: usize,
    column: usize,
) -> graphql_parser::Pos {
    graphql_parser::Pos { line, column }
}

/// Verifies that a simple `ObjectTypeDefinition` converts
/// to a `graphql_parser` `TypeDefinition::Object` with
/// the correct name, fields, implements list, and
/// position information.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_object_type_definition() {
    // Simulates:
    //   """A user"""               (line 0)
    //   type User implements Node { (line 1, col 0)
    //     name: String!             (line 2, col 2)
    //   }
    let doc = ast::Document {
        definitions: vec![ast::Definition::TypeDefinition(
            ast::TypeDefinition::Object(
                ast::ObjectTypeDefinition {
                    description: Some(
                        ast::StringValue {
                            is_block: false,
                            span: zero_span(),
                            syntax: None,
                            value: Cow::Borrowed(
                                "A user",
                            ),
                        },
                    ),
                    directives: vec![],
                    fields: vec![
                        ast::FieldDefinition {
                            name: make_name(
                                "name", 0, 4,
                            ),
                            description: None,
                            arguments: vec![],
                            directives: vec![],
                            field_type:
                                ast::TypeAnnotation::Named(
                                    ast::NamedTypeAnnotation {
                                        name: make_name(
                                            "String",
                                            0, 6,
                                        ),
                                        nullability:
                                            ast::Nullability::NonNull {
                                                syntax: None,
                                            },
                                        span: zero_span(),
                                    },
                                ),
                            span: make_span(2, 2),
                            syntax: None,
                        },
                    ],
                    implements: vec![make_name(
                        "Node", 0, 4,
                    )],
                    name: make_name("User", 0, 4),
                    span: make_span(1, 0),
                    syntax: None,
                },
            ),
        )],
        span: zero_span(),
        syntax: None,
    };

    let result = to_graphql_parser_schema_ast(&doc);
    assert!(result.is_ok());
    let gp_doc = result.into_valid_ast().unwrap();
    assert_eq!(gp_doc.definitions.len(), 1);

    match &gp_doc.definitions[0] {
        GpDef::TypeDefinition(GpTd::Object(obj)) => {
            assert_eq!(obj.position, pos(2, 1));
            assert_eq!(obj.name, "User");
            assert_eq!(
                obj.description,
                Some("A user".to_string()),
            );
            assert_eq!(
                obj.implements_interfaces,
                vec!["Node".to_string()],
            );
            assert_eq!(obj.fields.len(), 1);
            assert_eq!(
                obj.fields[0].position, pos(3, 3),
            );
            assert_eq!(obj.fields[0].name, "name");
        },
        other => {
            panic!(
                "Expected TypeDefinition::Object, \
                 got {:?}",
                other
            )
        },
    }
}

/// Verifies that a `SchemaDefinition` with root
/// operations converts correctly, mapping
/// `RootOperationTypeDefinition` to the `query`,
/// `mutation`, `subscription` Option fields.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_definition() {
    let doc = ast::Document {
        definitions: vec![
            ast::Definition::SchemaDefinition(
                ast::SchemaDefinition {
                    description: None,
                    directives: vec![],
                    root_operations: vec![
                        ast::RootOperationTypeDefinition {
                            named_type: make_name(
                                "Query", 0, 5,
                            ),
                            operation_kind:
                                ast::OperationKind::Query,
                            span: zero_span(),
                            syntax: None,
                        },
                        ast::RootOperationTypeDefinition {
                            named_type: make_name(
                                "Mutation", 0, 8,
                            ),
                            operation_kind:
                                ast::OperationKind::Mutation,
                            span: zero_span(),
                            syntax: None,
                        },
                    ],
                    span: make_span(3, 0),
                    syntax: None,
                },
            ),
        ],
        span: zero_span(),
        syntax: None,
    };

    let result = to_graphql_parser_schema_ast(&doc);
    assert!(result.is_ok());
    let gp_doc = result.into_valid_ast().unwrap();

    match &gp_doc.definitions[0] {
        GpDef::SchemaDefinition(sd) => {
            assert_eq!(sd.position, pos(4, 1));
            assert_eq!(
                sd.query,
                Some("Query".to_string()),
            );
            assert_eq!(
                sd.mutation,
                Some("Mutation".to_string()),
            );
            assert_eq!(sd.subscription, None);
        },
        other => {
            panic!(
                "Expected SchemaDefinition, got {:?}",
                other,
            )
        },
    }
}

/// Verifies that `SchemaExtension` nodes produce an
/// `UnsupportedFeature` error and are omitted from the
/// output document.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_extension_produces_error() {
    let doc = ast::Document {
        definitions: vec![
            ast::Definition::SchemaExtension(
                ast::SchemaExtension {
                    directives: vec![],
                    root_operations: vec![],
                    span: make_span(5, 0),
                    syntax: None,
                },
            ),
        ],
        span: zero_span(),
        syntax: None,
    };

    let result = to_graphql_parser_schema_ast(&doc);
    assert!(result.has_errors());
    assert_eq!(result.errors.len(), 1);

    let err = &result.errors[0];
    match err.kind() {
        crate::GraphQLParseErrorKind::UnsupportedFeature {
            feature,
        } => {
            assert_eq!(feature, "schema extension");
        },
        other => {
            panic!(
                "Expected UnsupportedFeature, got {:?}",
                other,
            )
        },
    }

    // Schema extension is dropped from output
    let gp_doc = result.into_ast().unwrap();
    assert!(gp_doc.definitions.is_empty());
}

/// Verifies that a `ScalarTypeDefinition` and an
/// `EnumTypeDefinition` convert correctly, including
/// position information for each node.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_scalar_and_enum() {
    // Simulates:
    //   scalar DateTime     (line 0, col 0)
    //                       (line 1 blank)
    //   enum Status {       (line 2, col 0)
    //     ACTIVE            (line 3, col 2)
    //     INACTIVE          (line 4, col 2)
    //   }
    let doc = ast::Document {
        definitions: vec![
            ast::Definition::TypeDefinition(
                ast::TypeDefinition::Scalar(
                    ast::ScalarTypeDefinition {
                        description: None,
                        directives: vec![],
                        name: make_name(
                            "DateTime", 0, 8,
                        ),
                        span: make_span(0, 0),
                        syntax: None,
                    },
                ),
            ),
            ast::Definition::TypeDefinition(
                ast::TypeDefinition::Enum(
                    ast::EnumTypeDefinition {
                        description: None,
                        directives: vec![],
                        name: make_name(
                            "Status", 0, 6,
                        ),
                        span: make_span(2, 0),
                        syntax: None,
                        values: vec![
                            ast::EnumValueDefinition {
                                description: None,
                                directives: vec![],
                                name: make_name(
                                    "ACTIVE", 0, 6,
                                ),
                                span: make_span(3, 2),
                            },
                            ast::EnumValueDefinition {
                                description: None,
                                directives: vec![],
                                name: make_name(
                                    "INACTIVE", 0, 8,
                                ),
                                span: make_span(4, 2),
                            },
                        ],
                    },
                ),
            ),
        ],
        span: zero_span(),
        syntax: None,
    };

    let result = to_graphql_parser_schema_ast(&doc);
    assert!(result.is_ok());
    let gp_doc = result.into_valid_ast().unwrap();
    assert_eq!(gp_doc.definitions.len(), 2);

    match &gp_doc.definitions[0] {
        GpDef::TypeDefinition(GpTd::Scalar(s)) => {
            assert_eq!(s.position, pos(1, 1));
            assert_eq!(s.name, "DateTime");
        },
        other => {
            panic!("Expected Scalar, got {:?}", other)
        },
    }

    match &gp_doc.definitions[1] {
        GpDef::TypeDefinition(GpTd::Enum(e)) => {
            assert_eq!(e.position, pos(3, 1));
            assert_eq!(e.name, "Status");
            assert_eq!(e.values.len(), 2);
            assert_eq!(
                e.values[0].position, pos(4, 3),
            );
            assert_eq!(e.values[0].name, "ACTIVE");
            assert_eq!(
                e.values[1].position, pos(5, 3),
            );
            assert_eq!(e.values[1].name, "INACTIVE");
        },
        other => {
            panic!("Expected Enum, got {:?}", other)
        },
    }
}

/// Verifies that `TypeExtension` nodes (Object extension)
/// convert correctly, including position.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_type_extension() {
    // Simulates:
    //   extend type User {  (line 0, col 0)
    //     age: Int          (line 1, col 2)
    //   }
    let doc = ast::Document {
        definitions: vec![ast::Definition::TypeExtension(
            ast::TypeExtension::Object(
                ast::ObjectTypeExtension {
                    directives: vec![],
                    fields: vec![ast::FieldDefinition {
                        name: make_name("age", 0, 3),
                        description: None,
                        arguments: vec![],
                        directives: vec![],
                        field_type:
                            ast::TypeAnnotation::Named(
                                ast::NamedTypeAnnotation {
                                    name: make_name(
                                        "Int", 0, 3,
                                    ),
                                    nullability:
                                        ast::Nullability::Nullable,
                                    span: zero_span(),
                                },
                            ),
                        span: make_span(1, 2),
                        syntax: None,
                    }],
                    implements: vec![],
                    name: make_name("User", 0, 4),
                    span: make_span(0, 0),
                    syntax: None,
                },
            ),
        )],
        span: zero_span(),
        syntax: None,
    };

    let result = to_graphql_parser_schema_ast(&doc);
    assert!(result.is_ok());
    let gp_doc = result.into_valid_ast().unwrap();

    match &gp_doc.definitions[0] {
        GpDef::TypeExtension(GpTe::Object(ext)) => {
            assert_eq!(ext.position, pos(1, 1));
            assert_eq!(ext.name, "User");
            assert_eq!(ext.fields.len(), 1);
            assert_eq!(
                ext.fields[0].position, pos(2, 3),
            );
            assert_eq!(ext.fields[0].name, "age");
        },
        other => {
            panic!(
                "Expected TypeExtension::Object, \
                 got {:?}",
                other,
            )
        },
    }
}

/// Verifies that executable definitions (operations,
/// fragments) are silently skipped during schema
/// conversion.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_executable_defs_skipped() {
    let doc = ast::Document {
        definitions: vec![
            ast::Definition::OperationDefinition(
                ast::OperationDefinition {
                    description: None,
                    directives: vec![],
                    name: None,
                    operation_kind:
                        ast::OperationKind::Query,
                    selection_set: ast::SelectionSet {
                        selections: vec![],
                        span: zero_span(),
                        syntax: None,
                    },
                    span: zero_span(),
                    syntax: None,
                    variable_definitions: vec![],
                },
            ),
        ],
        span: zero_span(),
        syntax: None,
    };

    let result = to_graphql_parser_schema_ast(&doc);
    assert!(result.is_ok());
    let gp_doc = result.into_valid_ast().unwrap();
    assert!(gp_doc.definitions.is_empty());
}

/// Verifies that a `DirectiveDefinition` converts
/// correctly, including locations, repeatable flag, and
/// position.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_directive_definition() {
    let doc = ast::Document {
        definitions: vec![
            ast::Definition::DirectiveDefinition(
                ast::DirectiveDefinition {
                    arguments: vec![],
                    description: Some(
                        ast::StringValue {
                            is_block: false,
                            span: zero_span(),
                            syntax: None,
                            value: Cow::Borrowed(
                                "Mark as cached",
                            ),
                        },
                    ),
                    locations: vec![
                        ast::DirectiveLocation {
                            kind: ast::DirectiveLocationKind::FieldDefinition,
                            span: zero_span(),
                            syntax: None,
                        },
                        ast::DirectiveLocation {
                            kind: ast::DirectiveLocationKind::Object,
                            span: zero_span(),
                            syntax: None,
                        },
                    ],
                    name: make_name("cached", 0, 6),
                    repeatable: true,
                    span: make_span(6, 0),
                    syntax: None,
                },
            ),
        ],
        span: zero_span(),
        syntax: None,
    };

    let result = to_graphql_parser_schema_ast(&doc);
    assert!(result.is_ok());
    let gp_doc = result.into_valid_ast().unwrap();

    match &gp_doc.definitions[0] {
        GpDef::DirectiveDefinition(dd) => {
            assert_eq!(dd.position, pos(7, 1));
            assert_eq!(dd.name, "cached");
            assert_eq!(
                dd.description,
                Some("Mark as cached".to_string()),
            );
            assert!(dd.repeatable);
            assert_eq!(dd.locations.len(), 2);
            assert_eq!(
                dd.locations[0],
                graphql_parser::schema::DirectiveLocation::FieldDefinition,
            );
            assert_eq!(
                dd.locations[1],
                graphql_parser::schema::DirectiveLocation::Object,
            );
        },
        other => {
            panic!(
                "Expected DirectiveDefinition, \
                 got {:?}",
                other,
            )
        },
    }
}
