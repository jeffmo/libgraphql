use crate::ast;
use crate::compat_graphql_parser_v0_4::from_graphql_parser_query_ast;

/// Shorthand for constructing a 1-based
/// `graphql_parser::Pos`.
fn pos(
    line: usize,
    column: usize,
) -> graphql_parser::Pos {
    graphql_parser::Pos { line, column }
}

/// Helper to build a graphql_parser query Document
/// from a single Definition.
fn doc_with(
    def: graphql_parser::query::Definition<
        'static,
        String,
    >,
) -> graphql_parser::query::Document<'static, String> {
    graphql_parser::query::Document {
        definitions: vec![def],
    }
}

/// Helper to build an empty selection set at a given
/// position.
fn empty_sel_set(
    start: graphql_parser::Pos,
) -> graphql_parser::query::SelectionSet<
    'static,
    String,
> {
    graphql_parser::query::SelectionSet {
        span: (start, start),
        items: vec![],
    }
}

/// Helper to build a selection set with a single
/// field.
fn sel_set_with_field(
    field_pos: graphql_parser::Pos,
    field_name: &str,
) -> graphql_parser::query::SelectionSet<
    'static,
    String,
> {
    graphql_parser::query::SelectionSet {
        span: (field_pos, field_pos),
        items: vec![
            graphql_parser::query::Selection::Field(
                graphql_parser::query::Field {
                    position: field_pos,
                    alias: None,
                    name: field_name.to_string(),
                    arguments: vec![],
                    directives: vec![],
                    selection_set:
                        graphql_parser::query::SelectionSet {
                            span: (
                                field_pos,
                                field_pos,
                            ),
                            items: vec![],
                        },
                },
            ),
        ],
    }
}

/// Verifies that a shorthand query (SelectionSet
/// variant) converts to an OperationDefinition with
/// Query kind and no name.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_shorthand_query() {
    let gp_doc = doc_with(
        graphql_parser::query::Definition::Operation(
            graphql_parser::query::OperationDefinition
                ::SelectionSet(
                sel_set_with_field(
                    pos(1, 3),
                    "users",
                ),
            ),
        ),
    );

    let doc = from_graphql_parser_query_ast(&gp_doc);
    assert_eq!(doc.definitions.len(), 1);

    match &doc.definitions[0] {
        ast::Definition::OperationDefinition(op) => {
            assert_eq!(
                op.operation_kind,
                ast::OperationKind::Query,
            );
            assert!(op.name.is_none());
            assert!(op.directives.is_empty());
            assert!(
                op.variable_definitions.is_empty(),
            );
            assert_eq!(
                op.selection_set.selections.len(),
                1,
            );
            match &op.selection_set.selections[0] {
                ast::Selection::Field(f) => {
                    assert_eq!(
                        f.name.value, "users",
                    );
                    assert_eq!(
                        f.span.start_inclusive.line(),
                        0,
                    );
                    assert_eq!(
                        f.span
                            .start_inclusive
                            .col_utf8(),
                        2,
                    );
                },
                other => panic!(
                    "Expected Field, got {:?}",
                    other,
                ),
            }
        },
        other => panic!(
            "Expected OperationDefinition, \
             got {:?}",
            other,
        ),
    }
}

/// Verifies that a named Query with variables converts
/// correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_named_query_with_variables() {
    let gp_doc = doc_with(
        graphql_parser::query::Definition::Operation(
            graphql_parser::query::OperationDefinition
                ::Query(
                graphql_parser::query::Query {
                    position: pos(1, 1),
                    name: Some(
                        "GetUser".to_string(),
                    ),
                    variable_definitions: vec![
                        graphql_parser::query::VariableDefinition {
                            position: pos(1, 15),
                            name: "id".to_string(),
                            var_type:
                                graphql_parser::schema::Type
                                    ::NonNullType(
                                    Box::new(
                                        graphql_parser::schema::Type
                                            ::NamedType(
                                            "ID".to_string(),
                                        ),
                                    ),
                                ),
                            default_value: None,
                        },
                    ],
                    directives: vec![],
                    selection_set:
                        sel_set_with_field(
                            pos(2, 3),
                            "user",
                        ),
                },
            ),
        ),
    );

    let doc = from_graphql_parser_query_ast(&gp_doc);

    match &doc.definitions[0] {
        ast::Definition::OperationDefinition(op) => {
            assert_eq!(
                op.operation_kind,
                ast::OperationKind::Query,
            );
            assert_eq!(
                op.name
                    .as_ref()
                    .map(|n| n.value.as_ref()),
                Some("GetUser"),
            );
            assert_eq!(
                op.variable_definitions.len(),
                1,
            );
            let var_def =
                &op.variable_definitions[0];
            assert_eq!(
                var_def.variable.value, "id",
            );
            // gp has no directives on var defs
            assert!(var_def.directives.is_empty());
            // Check type: ID!
            match &var_def.var_type {
                ast::TypeAnnotation::Named(n) => {
                    assert_eq!(n.name.value, "ID");
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
            "Expected OperationDefinition, \
             got {:?}",
            other,
        ),
    }
}

/// Verifies that a Mutation converts with correct
/// operation kind.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_mutation() {
    let gp_doc = doc_with(
        graphql_parser::query::Definition::Operation(
            graphql_parser::query::OperationDefinition
                ::Mutation(
                graphql_parser::query::Mutation {
                    position: pos(3, 1),
                    name: Some(
                        "CreateUser".to_string(),
                    ),
                    variable_definitions: vec![],
                    directives: vec![],
                    selection_set:
                        sel_set_with_field(
                            pos(4, 3),
                            "createUser",
                        ),
                },
            ),
        ),
    );

    let doc = from_graphql_parser_query_ast(&gp_doc);

    match &doc.definitions[0] {
        ast::Definition::OperationDefinition(op) => {
            assert_eq!(
                op.operation_kind,
                ast::OperationKind::Mutation,
            );
            assert_eq!(
                op.name
                    .as_ref()
                    .map(|n| n.value.as_ref()),
                Some("CreateUser"),
            );
            assert_eq!(
                op.span.start_inclusive.line(), 2,
            );
        },
        other => panic!(
            "Expected OperationDefinition, \
             got {:?}",
            other,
        ),
    }
}

/// Verifies that a Subscription converts with correct
/// operation kind.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_subscription() {
    let gp_doc = doc_with(
        graphql_parser::query::Definition::Operation(
            graphql_parser::query::OperationDefinition
                ::Subscription(
                graphql_parser::query::Subscription {
                    position: pos(5, 1),
                    name: None,
                    variable_definitions: vec![],
                    directives: vec![],
                    selection_set:
                        sel_set_with_field(
                            pos(6, 3),
                            "onMessage",
                        ),
                },
            ),
        ),
    );

    let doc = from_graphql_parser_query_ast(&gp_doc);

    match &doc.definitions[0] {
        ast::Definition::OperationDefinition(op) => {
            assert_eq!(
                op.operation_kind,
                ast::OperationKind::Subscription,
            );
            assert!(op.name.is_none());
        },
        other => panic!(
            "Expected OperationDefinition, \
             got {:?}",
            other,
        ),
    }
}

/// Verifies that a FragmentDefinition converts with
/// type condition, name, and selections.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_fragment_definition() {
    let gp_doc = doc_with(
        graphql_parser::query::Definition::Fragment(
            graphql_parser::query::FragmentDefinition {
                position: pos(7, 1),
                name: "UserFields".to_string(),
                type_condition:
                    graphql_parser::query
                        ::TypeCondition::On(
                        "User".to_string(),
                    ),
                directives: vec![],
                selection_set: sel_set_with_field(
                    pos(8, 3),
                    "name",
                ),
            },
        ),
    );

    let doc = from_graphql_parser_query_ast(&gp_doc);

    match &doc.definitions[0] {
        ast::Definition::FragmentDefinition(frag) => {
            assert_eq!(
                frag.name.value, "UserFields",
            );
            assert_eq!(
                frag.span.start_inclusive.line(), 6,
            );
            assert_eq!(
                frag.type_condition
                    .named_type
                    .value,
                "User",
            );
            assert_eq!(
                frag.selection_set.selections.len(),
                1,
            );
        },
        other => panic!(
            "Expected FragmentDefinition, got {:?}",
            other,
        ),
    }
}

/// Verifies that a field with alias and arguments
/// converts correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_field_with_alias_and_args() {
    let gp_doc = doc_with(
        graphql_parser::query::Definition::Operation(
            graphql_parser::query::OperationDefinition
                ::Query(
                graphql_parser::query::Query {
                    position: pos(1, 1),
                    name: None,
                    variable_definitions: vec![],
                    directives: vec![],
                    selection_set:
                        graphql_parser::query::SelectionSet {
                            span: (
                                pos(1, 1),
                                pos(5, 1),
                            ),
                            items: vec![
                                graphql_parser::query::Selection::Field(
                                    graphql_parser::query::Field {
                                        position: pos(2, 3),
                                        alias: Some(
                                            "myUser"
                                                .to_string(),
                                        ),
                                        name: "user"
                                            .to_string(),
                                        arguments: vec![
                                            (
                                                "id"
                                                    .to_string(),
                                                graphql_parser::query::Value::String(
                                                    "123"
                                                        .to_string(),
                                                ),
                                            ),
                                        ],
                                        directives: vec![],
                                        selection_set:
                                            sel_set_with_field(
                                                pos(3, 5),
                                                "name",
                                            ),
                                    },
                                ),
                            ],
                        },
                },
            ),
        ),
    );

    let doc = from_graphql_parser_query_ast(&gp_doc);

    match &doc.definitions[0] {
        ast::Definition::OperationDefinition(op) => {
            match &op.selection_set.selections[0] {
                ast::Selection::Field(field) => {
                    assert_eq!(
                        field.alias
                            .as_ref()
                            .map(|a| a.value.as_ref()),
                        Some("myUser"),
                    );
                    assert_eq!(
                        field.name.value, "user",
                    );
                    assert_eq!(
                        field.arguments.len(),
                        1,
                    );
                    assert_eq!(
                        field.arguments[0]
                            .name
                            .value,
                        "id",
                    );
                    // Nested selection set
                    assert!(
                        field
                            .selection_set
                            .is_some(),
                    );
                    let nested = field
                        .selection_set
                        .as_ref()
                        .unwrap();
                    assert_eq!(
                        nested.selections.len(),
                        1,
                    );
                },
                other => panic!(
                    "Expected Field, got {:?}",
                    other,
                ),
            }
        },
        other => panic!(
            "Expected OperationDefinition, \
             got {:?}",
            other,
        ),
    }
}

/// Verifies that fragment spread and inline fragment
/// selections convert correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_fragment_spread_and_inline_fragment() {
    let gp_doc = doc_with(
        graphql_parser::query::Definition::Operation(
            graphql_parser::query::OperationDefinition
                ::Query(
                graphql_parser::query::Query {
                    position: pos(1, 1),
                    name: None,
                    variable_definitions: vec![],
                    directives: vec![],
                    selection_set:
                        graphql_parser::query::SelectionSet {
                            span: (
                                pos(1, 1),
                                pos(10, 1),
                            ),
                            items: vec![
                                graphql_parser::query::Selection::FragmentSpread(
                                    graphql_parser::query::FragmentSpread {
                                        position: pos(2, 3),
                                        fragment_name:
                                            "UserFields"
                                                .to_string(),
                                        directives:
                                            vec![],
                                    },
                                ),
                                graphql_parser::query::Selection::InlineFragment(
                                    graphql_parser::query::InlineFragment {
                                        position: pos(3, 3),
                                        type_condition:
                                            Some(
                                                graphql_parser::query::TypeCondition::On(
                                                    "Admin"
                                                        .to_string(),
                                                ),
                                            ),
                                        directives:
                                            vec![],
                                        selection_set:
                                            sel_set_with_field(
                                                pos(4, 5),
                                                "role",
                                            ),
                                    },
                                ),
                            ],
                        },
                },
            ),
        ),
    );

    let doc = from_graphql_parser_query_ast(&gp_doc);

    match &doc.definitions[0] {
        ast::Definition::OperationDefinition(op) => {
            assert_eq!(
                op.selection_set.selections.len(),
                2,
            );

            // Fragment spread
            match &op.selection_set.selections[0] {
                ast::Selection::FragmentSpread(
                    spread,
                ) => {
                    assert_eq!(
                        spread.name.value,
                        "UserFields",
                    );
                    assert_eq!(
                        spread
                            .span
                            .start_inclusive
                            .line(),
                        1,
                    );
                },
                other => panic!(
                    "Expected FragmentSpread, \
                     got {:?}",
                    other,
                ),
            }

            // Inline fragment
            match &op.selection_set.selections[1] {
                ast::Selection::InlineFragment(
                    inline,
                ) => {
                    assert_eq!(
                        inline
                            .type_condition
                            .as_ref()
                            .map(|tc| {
                                tc.named_type
                                    .value
                                    .as_ref()
                            }),
                        Some("Admin"),
                    );
                    assert_eq!(
                        inline
                            .selection_set
                            .selections
                            .len(),
                        1,
                    );
                },
                other => panic!(
                    "Expected InlineFragment, \
                     got {:?}",
                    other,
                ),
            }
        },
        other => panic!(
            "Expected OperationDefinition, \
             got {:?}",
            other,
        ),
    }
}

/// Verifies that a field with empty selection_set
/// items results in selection_set being None in our
/// AST.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_empty_selection_set_becomes_none() {
    let gp_doc = doc_with(
        graphql_parser::query::Definition::Operation(
            graphql_parser::query::OperationDefinition
                ::Query(
                graphql_parser::query::Query {
                    position: pos(1, 1),
                    name: None,
                    variable_definitions: vec![],
                    directives: vec![],
                    selection_set:
                        graphql_parser::query::SelectionSet {
                            span: (
                                pos(1, 1),
                                pos(3, 1),
                            ),
                            items: vec![
                                graphql_parser::query::Selection::Field(
                                    graphql_parser::query::Field {
                                        position:
                                            pos(2, 3),
                                        alias: None,
                                        name: "name"
                                            .to_string(),
                                        arguments:
                                            vec![],
                                        directives:
                                            vec![],
                                        selection_set:
                                            empty_sel_set(
                                                pos(2, 3),
                                            ),
                                    },
                                ),
                            ],
                        },
                },
            ),
        ),
    );

    let doc = from_graphql_parser_query_ast(&gp_doc);

    match &doc.definitions[0] {
        ast::Definition::OperationDefinition(op) => {
            match &op.selection_set.selections[0] {
                ast::Selection::Field(field) => {
                    assert!(
                        field
                            .selection_set
                            .is_none(),
                    );
                },
                other => panic!(
                    "Expected Field, got {:?}",
                    other,
                ),
            }
        },
        _ => panic!("Expected OperationDefinition"),
    }
}

/// Verifies that variable definitions have empty
/// directives since graphql_parser doesn't support
/// directives on variables.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_variable_def_has_empty_directives() {
    let gp_doc = doc_with(
        graphql_parser::query::Definition::Operation(
            graphql_parser::query::OperationDefinition
                ::Query(
                graphql_parser::query::Query {
                    position: pos(1, 1),
                    name: None,
                    variable_definitions: vec![
                        graphql_parser::query::VariableDefinition {
                            position: pos(1, 10),
                            name: "x".to_string(),
                            var_type:
                                graphql_parser::schema::Type
                                    ::NamedType(
                                    "Int".to_string(),
                                ),
                            default_value: Some(
                                graphql_parser::query::Value
                                    ::Int(
                                    42i32.into(),
                                ),
                            ),
                        },
                    ],
                    directives: vec![],
                    selection_set:
                        sel_set_with_field(
                            pos(2, 3),
                            "value",
                        ),
                },
            ),
        ),
    );

    let doc = from_graphql_parser_query_ast(&gp_doc);

    match &doc.definitions[0] {
        ast::Definition::OperationDefinition(op) => {
            let var_def =
                &op.variable_definitions[0];
            assert!(var_def.directives.is_empty());
            assert!(var_def.description.is_none());
            // Check default value
            match &var_def.default_value {
                Some(ast::Value::Int(i)) => {
                    assert_eq!(i.value, 42);
                },
                other => panic!(
                    "Expected Int default, \
                     got {:?}",
                    other,
                ),
            }
        },
        _ => panic!("Expected OperationDefinition"),
    }
}
