//! Tests for [`crate::ast::ScalarTypeDefinition`] and
//! [`crate::ast::ScalarTypeDefinitionSyntax`].

use crate::ast::ScalarTypeDefinition;
use crate::ast::tests::ast_test_utils::make_byte_span;
use crate::ast::tests::ast_test_utils::make_name;

/// Verify `ScalarTypeDefinition` stores name and
/// slices the correct source range via
/// `append_source`.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Scalars
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn scalar_type_definition_construct_and_source_slice() {
    let source = "scalar DateTime";
    let node = ScalarTypeDefinition {
        span: make_byte_span(0, 15),
        description: None,
        name: make_name("DateTime", 7, 15),
        directives: vec![],
        syntax: None,
    };
    assert_eq!(node.name.value, "DateTime");

    let mut sink = String::new();
    node.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
