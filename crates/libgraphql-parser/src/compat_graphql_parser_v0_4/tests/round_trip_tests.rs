use std::borrow::Cow;

use crate::ast;
use crate::ast::tests::ast_test_utils::make_name;
use crate::ast::tests::ast_test_utils::make_span;
use crate::ast::tests::ast_test_utils::zero_span;
use crate::compat_graphql_parser_v0_4::from_graphql_parser_query_ast;
use crate::compat_graphql_parser_v0_4::from_graphql_parser_schema_ast;
use crate::compat_graphql_parser_v0_4::to_graphql_parser_query_ast;
use crate::compat_graphql_parser_v0_4::to_graphql_parser_schema_ast;

// ─────────────────────────────────────────────
// libgraphql → gp → libgraphql (schema)
// ─────────────────────────────────────────────

/// Verifies that libgraphql → gp → libgraphql
/// round-trip preserves all semantic content for a
/// schema document containing an object type with
/// description, fields, implements, and directives.
///
/// Per the Conversion Loss Inventory, we assert
/// on names, types, directive names + arguments,
/// and structural hierarchy. We do NOT assert on
/// syntax fields, span equality, Cow variant, or
/// object-value field ordering.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_round_trip_object_type() {
    let doc = ast::Document {
        definitions: vec![
            ast::Definition::TypeDefinition(
                ast::TypeDefinition::Object(
                    ast::ObjectTypeDefinition {
                        description: Some(
                            ast::StringValue {
                                is_block: false,
                                span: zero_span(),
                                syntax: None,
                                value: Cow::Borrowed(
                                    "A user account",
                                ),
                            },
                        ),
                        directives: vec![
                            ast::DirectiveAnnotation {
                                arguments: vec![],
                                name: make_name(
                                    "entity",
                                    0, 6,
                                ),
                                span: zero_span(),
                                syntax: None,
                            },
                        ],
                        fields: vec![
                            ast::FieldDefinition {
                                arguments: vec![],
                                description: None,
                                directives: vec![],
                                field_type:
                                    ast::TypeAnnotation::Named(
                                        ast::NamedTypeAnnotation {
                                            name: make_name(
                                                "ID",
                                                0, 2,
                                            ),
                                            nullability:
                                                ast::Nullability::NonNull {
                                                    syntax: None,
                                                },
                                            span: zero_span(),
                                        },
                                    ),
                                name: make_name(
                                    "id", 0, 2,
                                ),
                                span: make_span(
                                    2, 2,
                                ),
                                syntax: None,
                            },
                            ast::FieldDefinition {
                                arguments: vec![],
                                description: None,
                                directives: vec![],
                                field_type:
                                    ast::TypeAnnotation::Named(
                                        ast::NamedTypeAnnotation {
                                            name: make_name(
                                                "String",
                                                0, 6,
                                            ),
                                            nullability:
                                                ast::Nullability::Nullable,
                                            span: zero_span(),
                                        },
                                    ),
                                name: make_name(
                                    "email", 0, 5,
                                ),
                                span: make_span(
                                    3, 2,
                                ),
                                syntax: None,
                            },
                        ],
                        implements: vec![
                            make_name(
                                "Node", 0, 4,
                            ),
                        ],
                        name: make_name(
                            "User", 0, 4,
                        ),
                        span: make_span(1, 0),
                        syntax: None,
                    },
                ),
            ),
        ],
        span: zero_span(),
        syntax: None,
    };

    let gp_doc =
        to_graphql_parser_schema_ast(&doc)
            .into_ast()
            .expect("no unsupported features");
    let rt = from_graphql_parser_schema_ast(&gp_doc);

    assert_eq!(rt.definitions.len(), 1);

    match &rt.definitions[0] {
        ast::Definition::TypeDefinition(
            ast::TypeDefinition::Object(obj),
        ) => {
            assert_eq!(obj.name.value, "User");
            assert_eq!(
                obj.description
                    .as_ref()
                    .map(|d| d.value.as_ref()),
                Some("A user account"),
            );
            assert_eq!(obj.implements.len(), 1);
            assert_eq!(
                obj.implements[0].value, "Node",
            );
            assert_eq!(obj.directives.len(), 1);
            assert_eq!(
                obj.directives[0].name.value,
                "entity",
            );
            assert_eq!(obj.fields.len(), 2);
            assert_eq!(
                obj.fields[0].name.value, "id",
            );
            assert_eq!(
                obj.fields[1].name.value, "email",
            );

            // Verify type annotations survived
            match &obj.fields[0].field_type {
                ast::TypeAnnotation::Named(n) => {
                    assert_eq!(
                        n.name.value, "ID",
                    );
                    assert!(matches!(
                        n.nullability,
                        ast::Nullability::NonNull {
                            ..
                        },
                    ));
                },
                _ => panic!(
                    "Expected Named type for id",
                ),
            }
            match &obj.fields[1].field_type {
                ast::TypeAnnotation::Named(n) => {
                    assert_eq!(
                        n.name.value, "String",
                    );
                    assert!(matches!(
                        n.nullability,
                        ast::Nullability::Nullable,
                    ));
                },
                _ => panic!(
                    "Expected Named type for email",
                ),
            }
        },
        _ => panic!("Expected Object type"),
    }
}

/// Verifies libgraphql → gp → libgraphql
/// round-trip for an enum type and a union type
/// in the same document.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_round_trip_enum_and_union() {
    let doc = ast::Document {
        definitions: vec![
            ast::Definition::TypeDefinition(
                ast::TypeDefinition::Enum(
                    ast::EnumTypeDefinition {
                        description: None,
                        directives: vec![],
                        name: make_name(
                            "Role", 0, 4,
                        ),
                        span: make_span(0, 0),
                        syntax: None,
                        values: vec![
                            ast::EnumValueDefinition {
                                description: None,
                                directives: vec![],
                                name: make_name(
                                    "ADMIN", 0, 5,
                                ),
                                span: zero_span(),
                            },
                            ast::EnumValueDefinition {
                                description: None,
                                directives: vec![],
                                name: make_name(
                                    "USER", 0, 4,
                                ),
                                span: zero_span(),
                            },
                        ],
                    },
                ),
            ),
            ast::Definition::TypeDefinition(
                ast::TypeDefinition::Union(
                    ast::UnionTypeDefinition {
                        description: Some(
                            ast::StringValue {
                                is_block: false,
                                span: zero_span(),
                                syntax: None,
                                value: Cow::Borrowed(
                                    "A search result",
                                ),
                            },
                        ),
                        directives: vec![],
                        members: vec![
                            make_name(
                                "User", 0, 4,
                            ),
                            make_name(
                                "Post", 0, 4,
                            ),
                        ],
                        name: make_name(
                            "SearchResult",
                            0,
                            12,
                        ),
                        span: make_span(5, 0),
                        syntax: None,
                    },
                ),
            ),
        ],
        span: zero_span(),
        syntax: None,
    };

    let gp_doc =
        to_graphql_parser_schema_ast(&doc)
            .into_ast()
            .expect("no unsupported features");
    let rt = from_graphql_parser_schema_ast(&gp_doc);

    assert_eq!(rt.definitions.len(), 2);

    match &rt.definitions[0] {
        ast::Definition::TypeDefinition(
            ast::TypeDefinition::Enum(e),
        ) => {
            assert_eq!(e.name.value, "Role");
            assert_eq!(e.values.len(), 2);
            assert_eq!(
                e.values[0].name.value, "ADMIN",
            );
            assert_eq!(
                e.values[1].name.value, "USER",
            );
        },
        _ => panic!("Expected Enum"),
    }

    match &rt.definitions[1] {
        ast::Definition::TypeDefinition(
            ast::TypeDefinition::Union(u),
        ) => {
            assert_eq!(
                u.name.value, "SearchResult",
            );
            assert_eq!(
                u.description
                    .as_ref()
                    .map(|d| d.value.as_ref()),
                Some("A search result"),
            );
            assert_eq!(u.members.len(), 2);
            assert_eq!(
                u.members[0].value, "User",
            );
            assert_eq!(
                u.members[1].value, "Post",
            );
        },
        _ => panic!("Expected Union"),
    }
}

/// Verifies libgraphql → gp → libgraphql
/// round-trip for a directive definition with
/// arguments, locations, and repeatable flag.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_round_trip_directive_definition() {
    let doc = ast::Document {
        definitions: vec![
            ast::Definition::DirectiveDefinition(
                ast::DirectiveDefinition {
                    arguments: vec![
                        ast::InputValueDefinition {
                            default_value: Some(
                                ast::Value::String(
                                    ast::StringValue {
                                        is_block: false,
                                        span: zero_span(),
                                        syntax: None,
                                        value:
                                            Cow::Borrowed(
                                                "n/a",
                                            ),
                                    },
                                ),
                            ),
                            description: None,
                            directives: vec![],
                            name: make_name(
                                "reason", 0, 6,
                            ),
                            span: zero_span(),
                            syntax: None,
                            value_type:
                                ast::TypeAnnotation::Named(
                                    ast::NamedTypeAnnotation {
                                        name: make_name(
                                            "String",
                                            0, 6,
                                        ),
                                        nullability:
                                            ast::Nullability::Nullable,
                                        span: zero_span(),
                                    },
                                ),
                        },
                    ],
                    description: Some(
                        ast::StringValue {
                            is_block: false,
                            span: zero_span(),
                            syntax: None,
                            value: Cow::Borrowed(
                                "Marks deprecated",
                            ),
                        },
                    ),
                    locations: vec![
                        ast::DirectiveLocation {
                            kind:
                                ast::DirectiveLocationKind::FieldDefinition,
                            span: zero_span(),
                            syntax: None,
                        },
                        ast::DirectiveLocation {
                            kind:
                                ast::DirectiveLocationKind::EnumValue,
                            span: zero_span(),
                            syntax: None,
                        },
                    ],
                    name: make_name(
                        "deprecated", 0, 10,
                    ),
                    repeatable: true,
                    span: make_span(0, 0),
                    syntax: None,
                },
            ),
        ],
        span: zero_span(),
        syntax: None,
    };

    let gp_doc =
        to_graphql_parser_schema_ast(&doc)
            .into_ast()
            .expect("no unsupported features");
    let rt = from_graphql_parser_schema_ast(&gp_doc);

    assert_eq!(rt.definitions.len(), 1);

    match &rt.definitions[0] {
        ast::Definition::DirectiveDefinition(dd) => {
            assert_eq!(
                dd.name.value, "deprecated",
            );
            assert!(dd.repeatable);
            assert_eq!(
                dd.description
                    .as_ref()
                    .map(|d| d.value.as_ref()),
                Some("Marks deprecated"),
            );
            assert_eq!(dd.arguments.len(), 1);
            assert_eq!(
                dd.arguments[0].name.value,
                "reason",
            );
            match &dd.arguments[0].default_value {
                Some(ast::Value::String(s)) => {
                    assert_eq!(
                        s.value.as_ref(),
                        "n/a",
                    );
                },
                other => panic!(
                    "Expected String default, \
                     got {other:?}",
                ),
            }
            assert_eq!(dd.locations.len(), 2);
            assert_eq!(
                dd.locations[0].kind,
                ast::DirectiveLocationKind::FieldDefinition,
            );
            assert_eq!(
                dd.locations[1].kind,
                ast::DirectiveLocationKind::EnumValue,
            );
        },
        _ => panic!(
            "Expected DirectiveDefinition",
        ),
    }
}

// ─────────────────────────────────────────────
// libgraphql → gp → libgraphql (query)
// ─────────────────────────────────────────────

/// Verifies libgraphql → gp → libgraphql
/// round-trip for a query operation with named
/// fields, aliases, arguments, and inline
/// fragments.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_query_round_trip_operation() {
    let doc = ast::Document {
        definitions: vec![
            ast::Definition::OperationDefinition(
                ast::OperationDefinition {
                    description: None,
                    directives: vec![],
                    name: Some(make_name(
                        "GetUser", 0, 7,
                    )),
                    operation_kind:
                        ast::OperationKind::Query,
                    selection_set: ast::SelectionSet {
                        selections: vec![
                            ast::Selection::Field(
                                ast::Field {
                                    alias: Some(
                                        make_name(
                                            "u",
                                            0,
                                            1,
                                        ),
                                    ),
                                    arguments: vec![
                                        ast::Argument {
                                            name: make_name(
                                                "id",
                                                0,
                                                2,
                                            ),
                                            span: zero_span(),
                                            syntax: None,
                                            value: ast::Value::Int(
                                                ast::IntValue {
                                                    span: zero_span(),
                                                    syntax: None,
                                                    value: 42,
                                                },
                                            ),
                                        },
                                    ],
                                    directives: vec![],
                                    name: make_name(
                                        "user",
                                        0, 4,
                                    ),
                                    selection_set:
                                        Some(
                                            ast::SelectionSet {
                                                selections: vec![
                                                    ast::Selection::Field(
                                                        ast::Field {
                                                            alias: None,
                                                            arguments: vec![],
                                                            directives: vec![],
                                                            name: make_name(
                                                                "name",
                                                                0, 4,
                                                            ),
                                                            selection_set: None,
                                                            span: zero_span(),
                                                            syntax: None,
                                                        },
                                                    ),
                                                ],
                                                span: zero_span(),
                                                syntax: None,
                                            },
                                        ),
                                    span: make_span(
                                        1, 2,
                                    ),
                                    syntax: None,
                                },
                            ),
                        ],
                        span: zero_span(),
                        syntax: None,
                    },
                    shorthand: false,
                    span: make_span(0, 0),
                    syntax: None,
                    variable_definitions: vec![],
                },
            ),
        ],
        span: zero_span(),
        syntax: None,
    };

    let gp_doc =
        to_graphql_parser_query_ast(&doc)
            .into_ast()
            .expect("no unsupported features");
    let rt = from_graphql_parser_query_ast(&gp_doc);

    assert_eq!(rt.definitions.len(), 1);

    match &rt.definitions[0] {
        ast::Definition::OperationDefinition(op) => {
            assert_eq!(
                op.name.as_ref().map(
                    |n| n.value.as_ref()
                ),
                Some("GetUser"),
            );
            assert_eq!(
                op.operation_kind,
                ast::OperationKind::Query,
            );
            assert_eq!(
                op.selection_set.selections.len(),
                1,
            );

            // Check alias and arguments
            match &op.selection_set.selections[0] {
                ast::Selection::Field(f) => {
                    assert_eq!(
                        f.name.value, "user",
                    );
                    assert_eq!(
                        f.alias
                            .as_ref()
                            .map(
                                |a| a.value.as_ref()
                            ),
                        Some("u"),
                    );
                    assert_eq!(
                        f.arguments.len(), 1,
                    );
                    assert_eq!(
                        f.arguments[0].name.value,
                        "id",
                    );
                    match &f.arguments[0].value {
                        ast::Value::Int(i) => {
                            assert_eq!(
                                i.value, 42,
                            );
                        },
                        _ => panic!(
                            "Expected Int arg",
                        ),
                    }

                    // Nested field
                    let inner =
                        f.selection_set.as_ref()
                            .expect(
                                "should have \
                                 selection_set",
                            );
                    assert_eq!(
                        inner.selections.len(), 1,
                    );
                    match &inner.selections[0] {
                        ast::Selection::Field(
                            inner_f,
                        ) => {
                            assert_eq!(
                                inner_f.name.value,
                                "name",
                            );
                        },
                        _ => panic!(
                            "Expected inner Field",
                        ),
                    }
                },
                _ => panic!("Expected Field"),
            }
        },
        _ => panic!("Expected OperationDefinition"),
    }
}

/// Verifies libgraphql → gp → libgraphql
/// round-trip for a mutation with variable
/// definitions (without directives, since those
/// are dropped per the Loss Inventory).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_query_round_trip_mutation_with_variables() {
    let doc = ast::Document {
        definitions: vec![
            ast::Definition::OperationDefinition(
                ast::OperationDefinition {
                    description: None,
                    directives: vec![],
                    name: Some(make_name(
                        "CreateUser", 0, 10,
                    )),
                    operation_kind:
                        ast::OperationKind::Mutation,
                    selection_set: ast::SelectionSet {
                        selections: vec![
                            ast::Selection::Field(
                                ast::Field {
                                    alias: None,
                                    arguments: vec![],
                                    directives: vec![],
                                    name: make_name(
                                        "createUser",
                                        0, 10,
                                    ),
                                    selection_set:
                                        None,
                                    span: zero_span(),
                                    syntax: None,
                                },
                            ),
                        ],
                        span: zero_span(),
                        syntax: None,
                    },
                    shorthand: false,
                    span: make_span(0, 0),
                    syntax: None,
                    variable_definitions: vec![
                        ast::VariableDefinition {
                            default_value: Some(
                                ast::Value::String(
                                    ast::StringValue {
                                        is_block: false,
                                        span: zero_span(),
                                        syntax: None,
                                        value:
                                            Cow::Borrowed(
                                                "Anonymous",
                                            ),
                                    },
                                ),
                            ),
                            description: None,
                            directives: vec![],
                            span: zero_span(),
                            syntax: None,
                            var_type:
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
                            variable: make_name(
                                "name", 0, 4,
                            ),
                        },
                    ],
                },
            ),
        ],
        span: zero_span(),
        syntax: None,
    };

    let gp_doc =
        to_graphql_parser_query_ast(&doc)
            .into_ast()
            .expect("no unsupported features");
    let rt = from_graphql_parser_query_ast(&gp_doc);

    match &rt.definitions[0] {
        ast::Definition::OperationDefinition(op) => {
            assert_eq!(
                op.operation_kind,
                ast::OperationKind::Mutation,
            );
            assert_eq!(
                op.name.as_ref().map(
                    |n| n.value.as_ref()
                ),
                Some("CreateUser"),
            );

            // Variable definition preserved
            assert_eq!(
                op.variable_definitions.len(), 1,
            );
            let var = &op.variable_definitions[0];
            assert_eq!(
                var.variable.value, "name",
            );
            // Directives always empty after
            // round-trip (gp doesn't support them)
            assert!(var.directives.is_empty());

            match &var.var_type {
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
                _ => panic!(
                    "Expected Named type",
                ),
            }

            // Default value preserved
            match &var.default_value {
                Some(ast::Value::String(s)) => {
                    assert_eq!(
                        s.value.as_ref(),
                        "Anonymous",
                    );
                },
                other => panic!(
                    "Expected String default, \
                     got {other:?}",
                ),
            }
        },
        _ => panic!("Expected OperationDefinition"),
    }
}

/// Verifies libgraphql → gp → libgraphql
/// round-trip for a fragment definition with
/// a type condition and directives.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_query_round_trip_fragment() {
    let doc = ast::Document {
        definitions: vec![
            ast::Definition::FragmentDefinition(
                ast::FragmentDefinition {
                    description: None,
                    directives: vec![
                        ast::DirectiveAnnotation {
                            arguments: vec![],
                            name: make_name(
                                "include", 0, 7,
                            ),
                            span: zero_span(),
                            syntax: None,
                        },
                    ],
                    name: make_name(
                        "UserFields", 0, 10,
                    ),
                    selection_set: ast::SelectionSet {
                        selections: vec![
                            ast::Selection::Field(
                                ast::Field {
                                    alias: None,
                                    arguments: vec![],
                                    directives: vec![],
                                    name: make_name(
                                        "email",
                                        0, 5,
                                    ),
                                    selection_set:
                                        None,
                                    span: zero_span(),
                                    syntax: None,
                                },
                            ),
                        ],
                        span: zero_span(),
                        syntax: None,
                    },
                    span: make_span(0, 0),
                    syntax: None,
                    type_condition:
                        ast::TypeCondition {
                            named_type: make_name(
                                "User", 0, 4,
                            ),
                            span: zero_span(),
                            syntax: None,
                        },
                },
            ),
        ],
        span: zero_span(),
        syntax: None,
    };

    let gp_doc =
        to_graphql_parser_query_ast(&doc)
            .into_ast()
            .expect("no unsupported features");
    let rt = from_graphql_parser_query_ast(&gp_doc);

    match &rt.definitions[0] {
        ast::Definition::FragmentDefinition(frag) => {
            assert_eq!(
                frag.name.value, "UserFields",
            );
            assert_eq!(
                frag.type_condition
                    .named_type
                    .value,
                "User",
            );
            assert_eq!(frag.directives.len(), 1);
            assert_eq!(
                frag.directives[0].name.value,
                "include",
            );
            assert_eq!(
                frag.selection_set.selections.len(),
                1,
            );
            match &frag.selection_set.selections[0] {
                ast::Selection::Field(f) => {
                    assert_eq!(
                        f.name.value, "email",
                    );
                },
                _ => panic!("Expected Field"),
            }
        },
        _ => panic!("Expected FragmentDefinition"),
    }
}

// ─────────────────────────────────────────────
// gp → libgraphql → gp (schema)
// ─────────────────────────────────────────────

/// Shorthand for constructing a 1-based
/// `graphql_parser::Pos`.
fn pos(
    line: usize,
    column: usize,
) -> graphql_parser::Pos {
    graphql_parser::Pos { line, column }
}

/// Verifies that gp → libgraphql → gp round-trip
/// is lossless: the `graphql_parser` schema
/// document should be structurally identical after
/// the round-trip. This works because
/// Pos → zero-width span → Pos round-trips cleanly
/// and BTreeMap ordering is preserved.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_gp_schema_round_trip_lossless() {
    use graphql_parser::schema::Definition as GpDef;
    use graphql_parser::schema::EnumType
        as GpEnum;
    use graphql_parser::schema::EnumValue
        as GpEnumValue;
    use graphql_parser::schema::Field as GpField;
    use graphql_parser::schema::ObjectType
        as GpObject;
    use graphql_parser::schema::Type as GpType;
    use graphql_parser::schema::TypeDefinition
        as GpTd;
    use graphql_parser::schema::UnionType
        as GpUnion;

    let gp_doc = graphql_parser::schema::Document {
        definitions: vec![
            GpDef::TypeDefinition(
                GpTd::Object(GpObject {
                    position: pos(1, 1),
                    description: Some(
                        "A node".to_string(),
                    ),
                    name: "User".to_string(),
                    implements_interfaces: vec![
                        "Node".to_string(),
                    ],
                    directives: vec![],
                    fields: vec![
                        GpField {
                            position: pos(2, 3),
                            description: None,
                            name: "id"
                                .to_string(),
                            arguments: vec![],
                            field_type:
                                GpType::NonNullType(
                                    Box::new(
                                        GpType::NamedType(
                                            "ID".to_string(),
                                        ),
                                    ),
                                ),
                            directives: vec![],
                        },
                        GpField {
                            position: pos(3, 3),
                            description: None,
                            name: "name"
                                .to_string(),
                            arguments: vec![],
                            field_type:
                                GpType::NamedType(
                                    "String"
                                        .to_string(),
                                ),
                            directives: vec![],
                        },
                    ],
                }),
            ),
            GpDef::TypeDefinition(GpTd::Enum(
                GpEnum {
                    position: pos(6, 1),
                    description: None,
                    name: "Role".to_string(),
                    directives: vec![],
                    values: vec![
                        GpEnumValue {
                            position: pos(7, 3),
                            description: None,
                            name: "ADMIN"
                                .to_string(),
                            directives: vec![],
                        },
                        GpEnumValue {
                            position: pos(8, 3),
                            description: None,
                            name: "USER"
                                .to_string(),
                            directives: vec![],
                        },
                    ],
                },
            )),
            GpDef::TypeDefinition(GpTd::Union(
                GpUnion {
                    position: pos(11, 1),
                    description: Some(
                        "Result".to_string(),
                    ),
                    name: "SearchResult"
                        .to_string(),
                    directives: vec![],
                    types: vec![
                        "User".to_string(),
                        "Post".to_string(),
                    ],
                },
            )),
        ],
    };

    let ast_doc =
        from_graphql_parser_schema_ast(&gp_doc);
    let rt =
        to_graphql_parser_schema_ast(&ast_doc)
            .into_ast()
            .expect("round-trip should not fail");

    assert_eq!(rt, gp_doc);
}

// ─────────────────────────────────────────────
// gp → libgraphql → gp (query)
// ─────────────────────────────────────────────

/// Verifies that gp → libgraphql → gp round-trip
/// is lossless for a query document with a named
/// query, fields, arguments, and a fragment.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_gp_query_round_trip_lossless() {
    use graphql_parser::query::Definition as GpDef;
    use graphql_parser::query::Field as GpField;
    use graphql_parser::query::FragmentDefinition
        as GpFrag;
    use graphql_parser::query::Number as GpNumber;
    use graphql_parser::query::OperationDefinition
        as GpOp;
    use graphql_parser::query::Query as GpQuery;
    use graphql_parser::query::Selection as GpSel;
    use graphql_parser::query::SelectionSet
        as GpSelSet;
    use graphql_parser::query::TypeCondition
        as GpTypeCond;
    use graphql_parser::query::Value as GpValue;

    let gp_doc = graphql_parser::query::Document {
        definitions: vec![
            GpDef::Operation(GpOp::Query(
                GpQuery {
                    position: pos(1, 1),
                    name: Some(
                        "GetUser".to_string(),
                    ),
                    variable_definitions: vec![],
                    directives: vec![],
                    selection_set: GpSelSet {
                        span: (
                            pos(1, 16),
                            pos(4, 2),
                        ),
                        items: vec![
                            GpSel::Field(GpField {
                                position: pos(
                                    2, 3,
                                ),
                                alias: Some(
                                    "u".to_string(),
                                ),
                                name: "user"
                                    .to_string(),
                                arguments: vec![(
                                    "id".to_string(),
                                    GpValue::Int(
                                        GpNumber::from(
                                            42_i32,
                                        ),
                                    ),
                                )],
                                directives: vec![],
                                selection_set:
                                    GpSelSet {
                                        span: (
                                            pos(
                                                2,
                                                20,
                                            ),
                                            pos(
                                                4,
                                                4,
                                            ),
                                        ),
                                        items: vec![
                                    GpSel::Field(
                                        GpField {
                                            position:
                                                pos(3, 5),
                                            alias: None,
                                            name:
                                                "name"
                                                .to_string(),
                                            arguments:
                                                vec![],
                                            directives:
                                                vec![],
                                            selection_set:
                                                GpSelSet {
                                                    span: (
                                                        pos(3, 5),
                                                        pos(3, 5),
                                                    ),
                                                    items: vec![],
                                                },
                                        },
                                    ),
                                ],
                                    },
                            }),
                        ],
                    },
                },
            )),
            GpDef::Fragment(GpFrag {
                position: pos(6, 1),
                name: "UserFields".to_string(),
                type_condition:
                    GpTypeCond::On(
                        "User".to_string(),
                    ),
                directives: vec![],
                selection_set: GpSelSet {
                    span: (
                        pos(6, 30),
                        pos(8, 2),
                    ),
                    items: vec![GpSel::Field(
                        GpField {
                            position: pos(7, 3),
                            alias: None,
                            name: "email"
                                .to_string(),
                            arguments: vec![],
                            directives: vec![],
                            selection_set: GpSelSet {
                                span: (
                                    pos(7, 3),
                                    pos(7, 3),
                                ),
                                items: vec![],
                            },
                        },
                    )],
                },
            }),
        ],
    };

    let ast_doc =
        from_graphql_parser_query_ast(&gp_doc);
    let rt = to_graphql_parser_query_ast(&ast_doc)
        .into_ast()
        .expect("round-trip should not fail");

    assert_eq!(rt, gp_doc);
}
