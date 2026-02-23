use std::borrow::Cow;

use crate::ast;
use crate::ast::tests::ast_test_utils::make_name;
use crate::ast::tests::ast_test_utils::make_span;
use crate::ast::tests::ast_test_utils::zero_span;
use crate::compat_graphql_parser_v0_4::to_graphql_parser_query_ast;

use graphql_parser::query::Definition as GpDef;
use graphql_parser::query::OperationDefinition as GpOp;
use graphql_parser::query::Selection as GpSel;

/// Shorthand for constructing a 1-based
/// `graphql_parser::Pos`.
fn pos(
    line: usize,
    column: usize,
) -> graphql_parser::Pos {
    graphql_parser::Pos { line, column }
}

/// Verifies that a shorthand query (no keyword, no name,
/// no variables, no directives) maps to the
/// `OperationDefinition::SelectionSet` variant, and that
/// position information is preserved.
///
/// For a shorthand query, graphql_parser uses
/// `SelectionSet.span` (not a `position` field), which
/// comes from the `SelectionSet` node's span.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_shorthand_query() {
    // Simulates:
    //   {           (line 0, col 0 - selection_set)
    //     viewer    (line 1, col 2 - field)
    //   }
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
                        selections: vec![
                            ast::Selection::Field(
                                ast::Field {
                                    alias: None,
                                    arguments: vec![],
                                    directives: vec![],
                                    name: make_name(
                                        "viewer",
                                        0, 6,
                                    ),
                                    selection_set: None,
                                    span: make_span(
                                        1, 2,
                                    ),
                                    syntax: None,
                                },
                            ),
                        ],
                        span: make_span(0, 0),
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

    let result = to_graphql_parser_query_ast(&doc);
    assert!(result.is_ok());
    let gp_doc = result.into_valid_ast().unwrap();
    assert_eq!(gp_doc.definitions.len(), 1);

    match &gp_doc.definitions[0] {
        GpDef::Operation(GpOp::SelectionSet(ss)) => {
            assert_eq!(
                ss.span,
                (pos(1, 1), pos(1, 1)),
            );
            assert_eq!(ss.items.len(), 1);
            match &ss.items[0] {
                GpSel::Field(field) => {
                    assert_eq!(
                        field.position, pos(2, 3),
                    );
                    assert_eq!(field.name, "viewer");
                    assert!(field.alias.is_none());
                },
                other => panic!(
                    "Expected Field, got {other:?}",
                ),
            }
        },
        other => panic!(
            "Expected SelectionSet, got {other:?}",
        ),
    }
}

/// Verifies that a named query with variables maps to
/// the `OperationDefinition::Query` variant with correct
/// variable definitions and position information.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_named_query_with_variables() {
    // Simulates:
    //   query GetUser($id: ID!) { }  (line 0, col 0)
    //   variable $id starts at          (line 0, col 14)
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
                        selections: vec![],
                        span: zero_span(),
                        syntax: None,
                    },
                    span: make_span(0, 0),
                    syntax: None,
                    variable_definitions: vec![
                        ast::VariableDefinition {
                            default_value: None,
                            description: None,
                            directives: vec![],
                            span: make_span(0, 14),
                            syntax: None,
                            var_type:
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
                                        span:
                                            zero_span(),
                                    },
                                ),
                            variable: make_name(
                                "id", 0, 2,
                            ),
                        },
                    ],
                },
            ),
        ],
        span: zero_span(),
        syntax: None,
    };

    let result = to_graphql_parser_query_ast(&doc);
    assert!(result.is_ok());
    let gp_doc = result.into_valid_ast().unwrap();

    match &gp_doc.definitions[0] {
        GpDef::Operation(GpOp::Query(query)) => {
            assert_eq!(query.position, pos(1, 1));
            assert_eq!(
                query.name,
                Some("GetUser".to_string()),
            );
            assert_eq!(
                query.variable_definitions.len(),
                1,
            );
            let var_def =
                &query.variable_definitions[0];
            assert_eq!(
                var_def.position, pos(1, 15),
            );
            assert_eq!(var_def.name, "id");
            assert_eq!(
                var_def.var_type,
                graphql_parser::schema::Type::NonNullType(
                    Box::new(
                        graphql_parser::schema::Type::NamedType(
                            "ID".to_string(),
                        ),
                    ),
                ),
            );
        },
        other => panic!(
            "Expected Query, got {other:?}",
        ),
    }
}

/// Verifies that a mutation maps to the
/// `OperationDefinition::Mutation` variant with correct
/// position.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_mutation() {
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
                        selections: vec![],
                        span: zero_span(),
                        syntax: None,
                    },
                    span: make_span(2, 0),
                    syntax: None,
                    variable_definitions: vec![],
                },
            ),
        ],
        span: zero_span(),
        syntax: None,
    };

    let result = to_graphql_parser_query_ast(&doc);
    assert!(result.is_ok());
    let gp_doc = result.into_valid_ast().unwrap();

    match &gp_doc.definitions[0] {
        GpDef::Operation(GpOp::Mutation(mutation)) => {
            assert_eq!(mutation.position, pos(3, 1));
            assert_eq!(
                mutation.name,
                Some("CreateUser".to_string()),
            );
        },
        other => panic!(
            "Expected Mutation, got {other:?}",
        ),
    }
}

/// Verifies that a subscription maps to the
/// `OperationDefinition::Subscription` variant with
/// correct position.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_subscription() {
    let doc = ast::Document {
        definitions: vec![
            ast::Definition::OperationDefinition(
                ast::OperationDefinition {
                    description: None,
                    directives: vec![],
                    name: Some(make_name(
                        "OnMessage", 0, 9,
                    )),
                    operation_kind:
                        ast::OperationKind::Subscription,
                    selection_set: ast::SelectionSet {
                        selections: vec![],
                        span: zero_span(),
                        syntax: None,
                    },
                    span: make_span(4, 0),
                    syntax: None,
                    variable_definitions: vec![],
                },
            ),
        ],
        span: zero_span(),
        syntax: None,
    };

    let result = to_graphql_parser_query_ast(&doc);
    assert!(result.is_ok());
    let gp_doc = result.into_valid_ast().unwrap();

    match &gp_doc.definitions[0] {
        GpDef::Operation(GpOp::Subscription(sub)) => {
            assert_eq!(sub.position, pos(5, 1));
            assert_eq!(
                sub.name,
                Some("OnMessage".to_string()),
            );
        },
        other => panic!(
            "Expected Subscription, got {other:?}",
        ),
    }
}

/// Verifies that a fragment definition converts
/// correctly, including the type condition, selection set,
/// and position information at each level.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_fragment_definition() {
    // Simulates:
    //   fragment UserFields on User { (line 0, col 0)
    //     name                        (line 1, col 2)
    //   }
    let doc = ast::Document {
        definitions: vec![
            ast::Definition::FragmentDefinition(
                ast::FragmentDefinition {
                    description: None,
                    directives: vec![],
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
                                        "name",
                                        0, 4,
                                    ),
                                    selection_set: None,
                                    span: make_span(
                                        1, 2,
                                    ),
                                    syntax: None,
                                },
                            ),
                        ],
                        span: make_span(0, 28),
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

    let result = to_graphql_parser_query_ast(&doc);
    assert!(result.is_ok());
    let gp_doc = result.into_valid_ast().unwrap();

    match &gp_doc.definitions[0] {
        GpDef::Fragment(frag) => {
            assert_eq!(frag.position, pos(1, 1));
            assert_eq!(frag.name, "UserFields");
            assert_eq!(
                frag.type_condition,
                graphql_parser::query::TypeCondition
                    ::On("User".to_string()),
            );
            assert_eq!(
                frag.selection_set.items.len(),
                1,
            );
            match &frag.selection_set.items[0] {
                GpSel::Field(field) => {
                    assert_eq!(
                        field.position, pos(2, 3),
                    );
                    assert_eq!(field.name, "name");
                },
                other => panic!(
                    "Expected Field, got {other:?}",
                ),
            }
        },
        other => panic!(
            "Expected Fragment, got {other:?}",
        ),
    }
}

/// Verifies that a field with an alias, arguments, and
/// nested selections converts correctly with position
/// information at each nesting level.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_field_with_alias_and_args() {
    // Simulates:
    //   query Q {                              (line 0)
    //     hero: character(episode: JEDI) {     (line 1)
    //       name                               (line 2)
    //     }
    //   }
    let doc = ast::Document {
        definitions: vec![
            ast::Definition::OperationDefinition(
                ast::OperationDefinition {
                    description: None,
                    directives: vec![],
                    name: Some(make_name(
                        "Q", 0, 1,
                    )),
                    operation_kind:
                        ast::OperationKind::Query,
                    selection_set: ast::SelectionSet {
                        selections: vec![
                            ast::Selection::Field(
                                ast::Field {
                                    alias: Some(
                                        make_name(
                                            "hero",
                                            0, 4,
                                        ),
                                    ),
                                    arguments: vec![
                                        ast::Argument {
                                            name:
                                                make_name(
                                                    "episode",
                                                    0, 7,
                                                ),
                                            span:
                                                zero_span(),
                                            syntax: None,
                                            value:
                                                ast::Value::Enum(
                                                    ast::EnumValue {
                                                        span: zero_span(),
                                                        syntax: None,
                                                        value: Cow::Borrowed("JEDI"),
                                                    },
                                                ),
                                        },
                                    ],
                                    directives: vec![],
                                    name: make_name(
                                        "character",
                                        0, 9,
                                    ),
                                    selection_set: Some(
                                        ast::SelectionSet {
                                            selections: vec![
                                                ast::Selection::Field(
                                                    ast::Field {
                                                        alias: None,
                                                        arguments: vec![],
                                                        directives: vec![],
                                                        name: make_name("name", 0, 4),
                                                        selection_set: None,
                                                        span: make_span(2, 4),
                                                        syntax: None,
                                                    },
                                                ),
                                            ],
                                            span: make_span(1, 31),
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
                        span: make_span(0, 8),
                        syntax: None,
                    },
                    span: make_span(0, 0),
                    syntax: None,
                    variable_definitions: vec![],
                },
            ),
        ],
        span: zero_span(),
        syntax: None,
    };

    let result = to_graphql_parser_query_ast(&doc);
    assert!(result.is_ok());
    let gp_doc = result.into_valid_ast().unwrap();

    match &gp_doc.definitions[0] {
        GpDef::Operation(GpOp::Query(query)) => {
            assert_eq!(query.position, pos(1, 1));
            assert_eq!(
                query.selection_set.items.len(),
                1,
            );
            match &query.selection_set.items[0] {
                GpSel::Field(field) => {
                    assert_eq!(
                        field.position, pos(2, 3),
                    );
                    assert_eq!(
                        field.alias,
                        Some("hero".to_string()),
                    );
                    assert_eq!(
                        field.name, "character",
                    );
                    assert_eq!(
                        field.arguments.len(),
                        1,
                    );
                    assert_eq!(
                        field.arguments[0].0,
                        "episode",
                    );
                    assert_eq!(
                        field.arguments[0].1,
                        graphql_parser::query::Value
                            ::Enum(
                            "JEDI".to_string(),
                        ),
                    );
                    // Nested selection set
                    assert_eq!(
                        field
                            .selection_set
                            .items
                            .len(),
                        1,
                    );
                    match &field
                        .selection_set
                        .items[0]
                    {
                        GpSel::Field(nested) => {
                            assert_eq!(
                                nested.position,
                                pos(3, 5),
                            );
                            assert_eq!(
                                nested.name, "name",
                            );
                        },
                        other => panic!(
                            "Expected nested Field, \
                             got {other:?}",
                        ),
                    }
                },
                other => panic!(
                    "Expected Field, got {other:?}",
                ),
            }
        },
        other => panic!(
            "Expected Query, got {other:?}",
        ),
    }
}

/// Verifies that fragment spreads and inline fragments
/// convert correctly within a selection set, with
/// position information preserved at each level.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_fragment_spread_and_inline_fragment() {
    // Simulates:
    //   query Q {                   (line 0)
    //     ...UserFields             (line 1, col 2)
    //     ... on Admin {            (line 2, col 2)
    //       age                     (line 3, col 4)
    //     }
    //   }
    let doc = ast::Document {
        definitions: vec![
            ast::Definition::OperationDefinition(
                ast::OperationDefinition {
                    description: None,
                    directives: vec![],
                    name: Some(make_name(
                        "Q", 0, 1,
                    )),
                    operation_kind:
                        ast::OperationKind::Query,
                    selection_set: ast::SelectionSet {
                        selections: vec![
                            ast::Selection::FragmentSpread(
                                ast::FragmentSpread {
                                    directives: vec![],
                                    name: make_name(
                                        "UserFields",
                                        0, 10,
                                    ),
                                    span: make_span(
                                        1, 2,
                                    ),
                                    syntax: None,
                                },
                            ),
                            ast::Selection::InlineFragment(
                                ast::InlineFragment {
                                    directives: vec![],
                                    selection_set:
                                        ast::SelectionSet {
                                            selections: vec![
                                                ast::Selection::Field(
                                                    ast::Field {
                                                        alias: None,
                                                        arguments: vec![],
                                                        directives: vec![],
                                                        name: make_name("age", 0, 3),
                                                        selection_set: None,
                                                        span: make_span(3, 4),
                                                        syntax: None,
                                                    },
                                                ),
                                            ],
                                            span: make_span(2, 20),
                                            syntax: None,
                                        },
                                    span: make_span(
                                        2, 2,
                                    ),
                                    syntax: None,
                                    type_condition: Some(
                                        ast::TypeCondition {
                                            named_type: make_name("Admin", 0, 5),
                                            span: zero_span(),
                                            syntax: None,
                                        },
                                    ),
                                },
                            ),
                        ],
                        span: make_span(0, 8),
                        syntax: None,
                    },
                    span: make_span(0, 0),
                    syntax: None,
                    variable_definitions: vec![],
                },
            ),
        ],
        span: zero_span(),
        syntax: None,
    };

    let result = to_graphql_parser_query_ast(&doc);
    assert!(result.is_ok());
    let gp_doc = result.into_valid_ast().unwrap();

    match &gp_doc.definitions[0] {
        GpDef::Operation(GpOp::Query(query)) => {
            assert_eq!(
                query.selection_set.items.len(),
                2,
            );

            match &query.selection_set.items[0] {
                GpSel::FragmentSpread(
                    frag_spread,
                ) => {
                    assert_eq!(
                        frag_spread.position,
                        pos(2, 3),
                    );
                    assert_eq!(
                        frag_spread.fragment_name,
                        "UserFields",
                    );
                },
                other => panic!(
                    "Expected FragmentSpread, \
                     got {other:?}",
                ),
            }

            match &query.selection_set.items[1] {
                GpSel::InlineFragment(
                    inline_frag,
                ) => {
                    assert_eq!(
                        inline_frag.position,
                        pos(3, 3),
                    );
                    assert_eq!(
                        inline_frag.type_condition,
                        Some(
                            graphql_parser::query
                                ::TypeCondition::On(
                                "Admin".to_string(),
                            ),
                        ),
                    );
                    assert_eq!(
                        inline_frag
                            .selection_set
                            .items
                            .len(),
                        1,
                    );
                    match &inline_frag
                        .selection_set
                        .items[0]
                    {
                        GpSel::Field(field) => {
                            assert_eq!(
                                field.position,
                                pos(4, 5),
                            );
                            assert_eq!(
                                field.name, "age",
                            );
                        },
                        other => panic!(
                            "Expected Field, \
                             got {other:?}",
                        ),
                    }
                },
                other => panic!(
                    "Expected InlineFragment, \
                     got {other:?}",
                ),
            }
        },
        other => panic!(
            "Expected Query, got {other:?}",
        ),
    }
}

/// Verifies that variable definitions with directives
/// produce an `UnsupportedFeature` error, since
/// `graphql_parser` v0.4 has no directives field on
/// `VariableDefinition`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_variable_directives_produce_error() {
    let doc = ast::Document {
        definitions: vec![
            ast::Definition::OperationDefinition(
                ast::OperationDefinition {
                    description: None,
                    directives: vec![],
                    name: Some(make_name(
                        "Q", 0, 1,
                    )),
                    operation_kind:
                        ast::OperationKind::Query,
                    selection_set: ast::SelectionSet {
                        selections: vec![],
                        span: zero_span(),
                        syntax: None,
                    },
                    span: zero_span(),
                    syntax: None,
                    variable_definitions: vec![
                        ast::VariableDefinition {
                            default_value: None,
                            description: None,
                            directives: vec![
                                ast::DirectiveAnnotation {
                                    name: make_name(
                                        "deprecated",
                                        0, 10,
                                    ),
                                    span: zero_span(),
                                    syntax: None,
                                    arguments: vec![],
                                },
                            ],
                            span: zero_span(),
                            syntax: None,
                            var_type:
                                ast::TypeAnnotation::Named(
                                    ast::NamedTypeAnnotation {
                                        name: make_name(
                                            "Int",
                                            0, 3,
                                        ),
                                        nullability:
                                            ast::Nullability::Nullable,
                                        span:
                                            zero_span(),
                                    },
                                ),
                            variable: make_name(
                                "x", 0, 1,
                            ),
                        },
                    ],
                },
            ),
        ],
        span: zero_span(),
        syntax: None,
    };

    let result = to_graphql_parser_query_ast(&doc);
    assert!(result.has_errors());
    assert_eq!(result.errors.len(), 1);

    match result.errors[0].kind() {
        crate::GraphQLParseErrorKind::UnsupportedFeature {
            feature,
        } => {
            assert_eq!(
                feature, "variable directives",
            );
        },
        other => panic!(
            "Expected UnsupportedFeature, got {other:?}",
        ),
    }
}

/// Verifies that type-system definitions are silently
/// skipped during query conversion.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_type_system_defs_skipped() {
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
                        span: zero_span(),
                        syntax: None,
                    },
                ),
            ),
        ],
        span: zero_span(),
        syntax: None,
    };

    let result = to_graphql_parser_query_ast(&doc);
    assert!(result.is_ok());
    let gp_doc = result.into_valid_ast().unwrap();
    assert!(gp_doc.definitions.is_empty());
}
