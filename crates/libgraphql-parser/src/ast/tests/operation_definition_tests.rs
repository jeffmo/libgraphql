//! Tests for [`crate::ast::OperationDefinition`] and
//! [`crate::ast::OperationDefinitionSyntax`].

use crate::ast::Field;
use crate::ast::OperationDefinition;
use crate::ast::OperationKind;
use crate::ast::Selection;
use crate::ast::SelectionSet;
use crate::ast::tests::ast_test_helpers::make_byte_span;
use crate::ast::tests::ast_test_helpers::make_name;

/// Verify `OperationDefinition` stores operation kind,
/// name, variable definitions, directives, and selection
/// set.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Language.Operations
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_definition_query_source_slice() {
    let source = "query GetUser { name }";
    let od = OperationDefinition {
        span: make_byte_span(0, 22),
        description: None,
        operation_kind: OperationKind::Query,
        name: Some(
            make_name("GetUser", 6, 13),
        ),
        variable_definitions: vec![],
        directives: vec![],
        selection_set: SelectionSet {
            span: make_byte_span(14, 22),
            selections: vec![
                Selection::Field(Field {
                    span: make_byte_span(16, 20),
                    alias: None,
                    name: make_name(
                        "name", 16, 20,
                    ),
                    arguments: vec![],
                    directives: vec![],
                    selection_set: None,
                    syntax: None,
                }),
            ],
            syntax: None,
        },
        syntax: None,
    };
    assert_eq!(
        od.operation_kind,
        OperationKind::Query,
    );
    assert_eq!(
        od.name.as_ref().unwrap().value,
        "GetUser",
    );

    let mut sink = String::new();
    od.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `OperationDefinition` with
/// `OperationKind::Mutation`.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Language.Operations
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_definition_mutation() {
    let source =
        "mutation CreateUser { createUser }";
    let od = OperationDefinition {
        span: make_byte_span(0, 34),
        description: None,
        operation_kind: OperationKind::Mutation,
        name: Some(
            make_name("CreateUser", 9, 19),
        ),
        variable_definitions: vec![],
        directives: vec![],
        selection_set: SelectionSet {
            span: make_byte_span(20, 34),
            selections: vec![
                Selection::Field(Field {
                    span: make_byte_span(22, 32),
                    alias: None,
                    name: make_name(
                        "createUser", 22, 32,
                    ),
                    arguments: vec![],
                    directives: vec![],
                    selection_set: None,
                    syntax: None,
                }),
            ],
            syntax: None,
        },
        syntax: None,
    };
    assert_eq!(
        od.operation_kind,
        OperationKind::Mutation,
    );

    let mut sink = String::new();
    od.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `OperationDefinition` with
/// `OperationKind::Subscription`.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Language.Operations
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_definition_subscription() {
    let source =
        "subscription OnMsg { msg }";
    let od = OperationDefinition {
        span: make_byte_span(0, 26),
        description: None,
        operation_kind:
            OperationKind::Subscription,
        name: Some(
            make_name("OnMsg", 13, 18),
        ),
        variable_definitions: vec![],
        directives: vec![],
        selection_set: SelectionSet {
            span: make_byte_span(19, 26),
            selections: vec![
                Selection::Field(Field {
                    span: make_byte_span(21, 24),
                    alias: None,
                    name: make_name(
                        "msg", 21, 24,
                    ),
                    arguments: vec![],
                    directives: vec![],
                    selection_set: None,
                    syntax: None,
                }),
            ],
            syntax: None,
        },
        syntax: None,
    };
    assert_eq!(
        od.operation_kind,
        OperationKind::Subscription,
    );

    let mut sink = String::new();
    od.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
