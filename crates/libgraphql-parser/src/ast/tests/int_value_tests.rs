//! Tests for [`crate::ast::IntValue`].

use crate::ast::IntValue;
use crate::ast::tests::ast_test_utils::make_byte_span;

/// Verify `IntValue` stores the parsed i32 and slices the
/// correct source range.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Int-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn int_value_construct_and_source_slice() {
    let source = "42";
    let iv = IntValue {
        value: 42,
        span: make_byte_span(0, 2),
        syntax: None,
    };
    assert_eq!(iv.value, 42);
    assert_eq!(iv.as_i64(), 42i64);

    let mut sink = String::new();
    iv.append_source(&mut sink, Some(source));
    assert_eq!(sink, "42");
}
