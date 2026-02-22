//! Tests for [`crate::ast::NullValue`].

use crate::ast::NullValue;
use crate::ast::tests::ast_test_utils::make_byte_span;

/// Verify `NullValue` has no value field and slices the
/// correct source range.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Null-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn null_value_construct_and_source_slice() {
    let source = "null";
    let nv = NullValue {
        span: make_byte_span(0, 4),
        syntax: None,
    };

    let mut sink = String::new();
    nv.append_source(&mut sink, Some(source));
    assert_eq!(sink, "null");
}
