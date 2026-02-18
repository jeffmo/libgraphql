//! Tests for [`crate::ast::Argument`].

use crate::ast::Argument;
use crate::ast::IntValue;
use crate::ast::Value;
use crate::ast::tests::ast_test_helpers::make_byte_span;
use crate::ast::tests::ast_test_helpers::make_name;

/// Verify `Argument` stores a name and value, and slices
/// the correct source range.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Language.Arguments
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn argument_construct_and_source_slice() {
    let source = "limit: 10";
    let arg = Argument {
        span: make_byte_span(0, 9),
        name: make_name("limit", 0, 5),
        value: Value::Int(IntValue {
            value: 10,
            span: make_byte_span(7, 9),
            syntax: None,
        }),
        syntax: None,
    };
    assert_eq!(arg.name.value, "limit");

    let mut sink = String::new();
    arg.append_source(&mut sink, Some(source));
    assert_eq!(sink, "limit: 10");
}
