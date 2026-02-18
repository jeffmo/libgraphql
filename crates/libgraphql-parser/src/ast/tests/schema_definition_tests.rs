//! Tests for [`crate::ast::SchemaDefinition`] and
//! [`crate::ast::SchemaDefinitionSyntax`].

use crate::ast::OperationKind;
use crate::ast::SchemaDefinition;
use crate::ast::tests::ast_test_helpers::make_byte_span;
use crate::ast::tests::ast_test_helpers::make_name;

/// Verify `SchemaDefinition` stores root operation type
/// definitions.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Schema
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn schema_definition_construct_and_source_slice() {
    let source = "schema { query: Query }";
    let sd = SchemaDefinition {
        span: make_byte_span(0, 23),
        description: None,
        directives: vec![],
        root_operations: vec![
            crate::ast::RootOperationTypeDefinition {
                span: make_byte_span(9, 21),
                operation_kind: OperationKind::Query,
                named_type: make_name(
                    "Query", 16, 21,
                ),
                syntax: None,
            },
        ],
        syntax: None,
    };
    assert_eq!(sd.root_operations.len(), 1);

    let mut sink = String::new();
    sd.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
