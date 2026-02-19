//! Tests for
//! [`crate::ast::RootOperationTypeDefinition`] and
//! [`crate::ast::RootOperationTypeDefinitionSyntax`].

use crate::ast::OperationKind;
use crate::ast::RootOperationTypeDefinition;
use crate::ast::tests::ast_test_utils::make_byte_span;
use crate::ast::tests::ast_test_utils::make_name;

/// Verify `RootOperationTypeDefinition` stores
/// operation kind and named type, and produces
/// the correct source slice.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Root-Operation-Types
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn root_operation_type_definition_query() {
    let source = "query: Query";
    let rotd = RootOperationTypeDefinition {
        named_type: make_name("Query", 7, 12),
        operation_kind: OperationKind::Query,
        span: make_byte_span(0, 12),
        syntax: None,
    };
    assert_eq!(rotd.named_type.value, "Query");
    assert_eq!(
        rotd.operation_kind,
        OperationKind::Query,
    );

    let mut sink = String::new();
    rotd.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `RootOperationTypeDefinition` with mutation.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Root-Operation-Types
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn root_operation_type_definition_mutation() {
    let source = "mutation: Mutation";
    let rotd = RootOperationTypeDefinition {
        named_type: make_name("Mutation", 10, 18),
        operation_kind: OperationKind::Mutation,
        span: make_byte_span(0, 18),
        syntax: None,
    };
    assert_eq!(rotd.named_type.value, "Mutation");
    assert_eq!(
        rotd.operation_kind,
        OperationKind::Mutation,
    );

    let mut sink = String::new();
    rotd.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
