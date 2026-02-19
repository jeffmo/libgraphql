//! Tests for [`crate::ast::EnumValueDefinition`].

use crate::ast::EnumValueDefinition;
use crate::ast::tests::ast_test_utils::make_byte_span;
use crate::ast::tests::ast_test_utils::make_name;

/// Verify `EnumValueDefinition` stores name and optional
/// description/directives.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#EnumValuesDefinition
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_value_definition_construct_and_source_slice() {
    let source = "ACTIVE";
    let evd = EnumValueDefinition {
        span: make_byte_span(0, 6),
        description: None,
        name: make_name("ACTIVE", 0, 6),
        directives: vec![],
    };
    assert_eq!(evd.name.value, "ACTIVE");

    let mut sink = String::new();
    evd.append_source(&mut sink, Some(source));
    assert_eq!(sink, "ACTIVE");
}
