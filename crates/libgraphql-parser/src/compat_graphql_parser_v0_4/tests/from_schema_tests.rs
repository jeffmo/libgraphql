use std::borrow::Cow;

use crate::ast;
use crate::compat_graphql_parser_v0_4::from_graphql_parser_schema_ast;

use graphql_parser::schema::Definition as GpDef;
use graphql_parser::schema::DirectiveDefinition
    as GpDirectiveDef;
use graphql_parser::schema::DirectiveLocation
    as GpDirLoc;
use graphql_parser::schema::EnumType as GpEnum;
use graphql_parser::schema::EnumValue as GpEnumValue;
use graphql_parser::schema::InputObjectType
    as GpInputObject;
use graphql_parser::schema::InterfaceType
    as GpInterface;
use graphql_parser::schema::ObjectType as GpObject;
use graphql_parser::schema::ScalarType as GpScalar;
use graphql_parser::schema::SchemaDefinition
    as GpSchemaDef;
use graphql_parser::schema::TypeDefinition as GpTd;
use graphql_parser::schema::UnionType as GpUnion;

/// Shorthand for constructing a 1-based
/// `graphql_parser::Pos`.
fn pos(
    line: usize,
    column: usize,
) -> graphql_parser::Pos {
    graphql_parser::Pos { line, column }
}

/// Verifies that a graphql_parser `ObjectType` converts
/// to our `ObjectTypeDefinition`, preserving name,
/// description, implements, fields, and position.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_object_type() {
    let gp_doc = graphql_parser::schema::Document {
        definitions: vec![GpDef::TypeDefinition(
            GpTd::Object(GpObject {
                position: pos(2, 1),
                description: Some(
                    "A user".to_string(),
                ),
                name: "User".to_string(),
                implements_interfaces: vec![
                    "Node".to_string(),
                ],
                directives: vec![],
                fields: vec![
                    graphql_parser::schema::Field {
                        position: pos(3, 3),
                        description: None,
                        name: "name".to_string(),
                        arguments: vec![],
                        field_type:
                            graphql_parser::schema::Type
                                ::NonNullType(
                                Box::new(
                                    graphql_parser::schema::Type
                                        ::NamedType(
                                        "String"
                                            .to_string(),
                                    ),
                                ),
                            ),
                        directives: vec![],
                    },
                ],
            }),
        )],
    };

    let doc = from_graphql_parser_schema_ast(&gp_doc);
    assert_eq!(doc.definitions.len(), 1);

    match &doc.definitions[0] {
        ast::Definition::TypeDefinition(
            ast::TypeDefinition::Object(obj),
        ) => {
            assert_eq!(obj.name.value, "User");
            assert_eq!(
                obj.span.start_inclusive.line(), 1,
            );
            assert_eq!(
                obj.span.start_inclusive.col_utf8(), 0,
            );
            assert_eq!(
                obj.description.as_ref().map(
                    |d| d.value.as_ref()
                ),
                Some("A user"),
            );
            assert_eq!(obj.implements.len(), 1);
            assert_eq!(
                obj.implements[0].value,
                "Node",
            );
            assert_eq!(obj.fields.len(), 1);
            assert_eq!(
                obj.fields[0].name.value, "name",
            );
            assert_eq!(
                obj.fields[0]
                    .span
                    .start_inclusive
                    .line(),
                2,
            );
            assert_eq!(
                obj.fields[0]
                    .span
                    .start_inclusive
                    .col_utf8(),
                2,
            );
            // Verify field type is NonNull String
            match &obj.fields[0].field_type {
                ast::TypeAnnotation::Named(n) => {
                    assert_eq!(
                        n.name.value, "String",
                    );
                    assert!(matches!(
                        n.nullability,
                        ast::Nullability::NonNull {
                            ..
                        },
                    ));
                },
                other => panic!(
                    "Expected Named, got {:?}",
                    other,
                ),
            }
        },
        other => panic!(
            "Expected Object, got {:?}",
            other,
        ),
    }
}

/// Verifies that a `SchemaDefinition` converts
/// correctly, preserving root operation types.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_definition() {
    let gp_doc = graphql_parser::schema::Document {
        definitions: vec![
            GpDef::SchemaDefinition(GpSchemaDef {
                position: pos(1, 1),
                directives: vec![],
                query: Some("Query".to_string()),
                mutation: Some(
                    "Mutation".to_string(),
                ),
                subscription: None,
            }),
        ],
    };

    let doc = from_graphql_parser_schema_ast(&gp_doc);

    match &doc.definitions[0] {
        ast::Definition::SchemaDefinition(sd) => {
            assert_eq!(sd.root_operations.len(), 2);
            assert_eq!(
                sd.root_operations[0]
                    .named_type
                    .value,
                "Query",
            );
            assert_eq!(
                sd.root_operations[0].operation_kind,
                ast::OperationKind::Query,
            );
            assert_eq!(
                sd.root_operations[1]
                    .named_type
                    .value,
                "Mutation",
            );
            assert_eq!(
                sd.root_operations[1].operation_kind,
                ast::OperationKind::Mutation,
            );
        },
        other => panic!(
            "Expected SchemaDefinition, got {:?}",
            other,
        ),
    }
}

/// Verifies that a `ScalarType` converts correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_scalar_type() {
    let gp_doc = graphql_parser::schema::Document {
        definitions: vec![GpDef::TypeDefinition(
            GpTd::Scalar(GpScalar {
                position: pos(1, 1),
                description: None,
                name: "DateTime".to_string(),
                directives: vec![],
            }),
        )],
    };

    let doc = from_graphql_parser_schema_ast(&gp_doc);

    match &doc.definitions[0] {
        ast::Definition::TypeDefinition(
            ast::TypeDefinition::Scalar(s),
        ) => {
            assert_eq!(s.name.value, "DateTime");
            assert!(s.description.is_none());
        },
        other => panic!(
            "Expected Scalar, got {:?}",
            other,
        ),
    }
}

/// Verifies that an `EnumType` with values converts
/// correctly, preserving value names and positions.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_enum_type() {
    let gp_doc = graphql_parser::schema::Document {
        definitions: vec![GpDef::TypeDefinition(
            GpTd::Enum(GpEnum {
                position: pos(1, 1),
                description: None,
                name: "Status".to_string(),
                directives: vec![],
                values: vec![
                    GpEnumValue {
                        position: pos(2, 3),
                        description: None,
                        name: "ACTIVE".to_string(),
                        directives: vec![],
                    },
                    GpEnumValue {
                        position: pos(3, 3),
                        description: Some(
                            "Deactivated"
                                .to_string(),
                        ),
                        name: "INACTIVE"
                            .to_string(),
                        directives: vec![],
                    },
                ],
            }),
        )],
    };

    let doc = from_graphql_parser_schema_ast(&gp_doc);

    match &doc.definitions[0] {
        ast::Definition::TypeDefinition(
            ast::TypeDefinition::Enum(e),
        ) => {
            assert_eq!(e.name.value, "Status");
            assert_eq!(e.values.len(), 2);
            assert_eq!(
                e.values[0].name.value, "ACTIVE",
            );
            assert_eq!(
                e.values[0]
                    .span
                    .start_inclusive
                    .line(),
                1,
            );
            assert_eq!(
                e.values[1].name.value, "INACTIVE",
            );
            assert_eq!(
                e.values[1]
                    .description
                    .as_ref()
                    .map(|d| d.value.as_ref()),
                Some("Deactivated"),
            );
        },
        other => panic!(
            "Expected Enum, got {:?}",
            other,
        ),
    }
}

/// Verifies that an `InterfaceType` with implements
/// converts correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_interface_type() {
    let gp_doc = graphql_parser::schema::Document {
        definitions: vec![GpDef::TypeDefinition(
            GpTd::Interface(GpInterface {
                position: pos(4, 1),
                description: Some(
                    "A node".to_string(),
                ),
                name: "Node".to_string(),
                implements_interfaces: vec![],
                directives: vec![],
                fields: vec![
                    graphql_parser::schema::Field {
                        position: pos(5, 3),
                        description: None,
                        name: "id".to_string(),
                        arguments: vec![],
                        field_type:
                            graphql_parser::schema::Type
                                ::NamedType(
                                "ID".to_string(),
                            ),
                        directives: vec![],
                    },
                ],
            }),
        )],
    };

    let doc = from_graphql_parser_schema_ast(&gp_doc);

    match &doc.definitions[0] {
        ast::Definition::TypeDefinition(
            ast::TypeDefinition::Interface(iface),
        ) => {
            assert_eq!(iface.name.value, "Node");
            assert_eq!(
                iface.span.start_inclusive.line(), 3,
            );
            assert_eq!(iface.fields.len(), 1);
            assert_eq!(
                iface.fields[0].name.value, "id",
            );
        },
        other => panic!(
            "Expected Interface, got {:?}",
            other,
        ),
    }
}

/// Verifies that a `UnionType` converts correctly,
/// preserving member types.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_union_type() {
    let gp_doc = graphql_parser::schema::Document {
        definitions: vec![GpDef::TypeDefinition(
            GpTd::Union(GpUnion {
                position: pos(6, 1),
                description: None,
                name: "SearchResult".to_string(),
                directives: vec![],
                types: vec![
                    "User".to_string(),
                    "Post".to_string(),
                ],
            }),
        )],
    };

    let doc = from_graphql_parser_schema_ast(&gp_doc);

    match &doc.definitions[0] {
        ast::Definition::TypeDefinition(
            ast::TypeDefinition::Union(u),
        ) => {
            assert_eq!(
                u.name.value, "SearchResult",
            );
            assert_eq!(u.members.len(), 2);
            assert_eq!(
                u.members[0].value, "User",
            );
            assert_eq!(
                u.members[1].value, "Post",
            );
        },
        other => panic!(
            "Expected Union, got {:?}",
            other,
        ),
    }
}

/// Verifies that an `InputObjectType` converts
/// correctly with fields.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_input_object_type() {
    let gp_doc = graphql_parser::schema::Document {
        definitions: vec![GpDef::TypeDefinition(
            GpTd::InputObject(GpInputObject {
                position: pos(7, 1),
                description: None,
                name: "CreateUserInput"
                    .to_string(),
                directives: vec![],
                fields: vec![
                    graphql_parser::schema::InputValue {
                        position: pos(8, 3),
                        description: None,
                        name: "name".to_string(),
                        value_type:
                            graphql_parser::schema::Type
                                ::NamedType(
                                "String".to_string(),
                            ),
                        default_value: None,
                        directives: vec![],
                    },
                ],
            }),
        )],
    };

    let doc = from_graphql_parser_schema_ast(&gp_doc);

    match &doc.definitions[0] {
        ast::Definition::TypeDefinition(
            ast::TypeDefinition::InputObject(io),
        ) => {
            assert_eq!(
                io.name.value, "CreateUserInput",
            );
            assert_eq!(io.fields.len(), 1);
            assert_eq!(
                io.fields[0].name.value, "name",
            );
        },
        other => panic!(
            "Expected InputObject, got {:?}",
            other,
        ),
    }
}

/// Verifies that a `DirectiveDefinition` converts
/// correctly with locations and repeatable flag.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_directive_definition() {
    let gp_doc = graphql_parser::schema::Document {
        definitions: vec![
            GpDef::DirectiveDefinition(
                GpDirectiveDef {
                    position: pos(9, 1),
                    description: Some(
                        "Cache hint".to_string(),
                    ),
                    name: "cached".to_string(),
                    arguments: vec![],
                    repeatable: true,
                    locations: vec![
                        GpDirLoc::FieldDefinition,
                        GpDirLoc::Object,
                    ],
                },
            ),
        ],
    };

    let doc = from_graphql_parser_schema_ast(&gp_doc);

    match &doc.definitions[0] {
        ast::Definition::DirectiveDefinition(dd) => {
            assert_eq!(dd.name.value, "cached");
            assert!(dd.repeatable);
            assert_eq!(
                dd.description
                    .as_ref()
                    .map(|d| d.value.as_ref()),
                Some("Cache hint"),
            );
            assert_eq!(dd.locations.len(), 2);
            assert_eq!(
                dd.locations[0].kind,
                ast::DirectiveLocationKind
                    ::FieldDefinition,
            );
            assert_eq!(
                dd.locations[1].kind,
                ast::DirectiveLocationKind::Object,
            );
        },
        other => panic!(
            "Expected DirectiveDefinition, got {:?}",
            other,
        ),
    }
}

/// Verifies that type extensions convert correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_type_extension() {
    use graphql_parser::schema::ObjectTypeExtension
        as GpObjExt;
    use graphql_parser::schema::TypeExtension
        as GpTe;

    let gp_doc = graphql_parser::schema::Document {
        definitions: vec![GpDef::TypeExtension(
            GpTe::Object(GpObjExt {
                position: pos(10, 1),
                name: "User".to_string(),
                implements_interfaces: vec![],
                directives: vec![],
                fields: vec![
                    graphql_parser::schema::Field {
                        position: pos(11, 3),
                        description: None,
                        name: "age".to_string(),
                        arguments: vec![],
                        field_type:
                            graphql_parser::schema::Type
                                ::NamedType(
                                "Int".to_string(),
                            ),
                        directives: vec![],
                    },
                ],
            }),
        )],
    };

    let doc = from_graphql_parser_schema_ast(&gp_doc);

    match &doc.definitions[0] {
        ast::Definition::TypeExtension(
            ast::TypeExtension::Object(ext),
        ) => {
            assert_eq!(ext.name.value, "User");
            assert_eq!(
                ext.span.start_inclusive.line(), 9,
            );
            assert_eq!(ext.fields.len(), 1);
            assert_eq!(
                ext.fields[0].name.value, "age",
            );
        },
        other => panic!(
            "Expected Object extension, got {:?}",
            other,
        ),
    }
}

/// Verifies that all syntax fields are None in the
/// converted AST.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_syntax_fields_are_none() {
    let gp_doc = graphql_parser::schema::Document {
        definitions: vec![GpDef::TypeDefinition(
            GpTd::Scalar(GpScalar {
                position: pos(1, 1),
                description: None,
                name: "JSON".to_string(),
                directives: vec![],
            }),
        )],
    };

    let doc = from_graphql_parser_schema_ast(&gp_doc);

    match &doc.definitions[0] {
        ast::Definition::TypeDefinition(
            ast::TypeDefinition::Scalar(s),
        ) => {
            assert!(s.syntax.is_none());
        },
        _ => panic!("Expected Scalar"),
    }
    assert!(doc.syntax.is_none());
}

/// Verifies that strings in the converted AST are
/// `Cow::Owned`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_strings_are_cow_owned() {
    let gp_doc = graphql_parser::schema::Document {
        definitions: vec![GpDef::TypeDefinition(
            GpTd::Scalar(GpScalar {
                position: pos(1, 1),
                description: Some(
                    "JSON data".to_string(),
                ),
                name: "JSON".to_string(),
                directives: vec![],
            }),
        )],
    };

    let doc = from_graphql_parser_schema_ast(&gp_doc);

    match &doc.definitions[0] {
        ast::Definition::TypeDefinition(
            ast::TypeDefinition::Scalar(s),
        ) => {
            assert!(matches!(
                &s.name.value,
                Cow::Owned(_),
            ));
            assert!(matches!(
                &s.description.as_ref().unwrap().value,
                Cow::Owned(_),
            ));
        },
        _ => panic!("Expected Scalar"),
    }
}
