//! Tests for [`crate::ast::BooleanValue`].

use crate::ast::BooleanValue;
use crate::ast::tests::ast_test_helpers::make_byte_span;

/// Verify `BooleanValue` stores the boolean and slices
/// the correct source range.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Boolean-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn boolean_value_construct_and_source_slice() {
    let source = "true";
    let bv = BooleanValue {
        value: true,
        span: make_byte_span(0, 4),
        syntax: None,
    };
    assert!(bv.value);

    let mut sink = String::new();
    bv.append_source(&mut sink, Some(source));
    assert_eq!(sink, "true");
}

/// Verify `BooleanValue` stores `false` and slices
/// the correct source range.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Boolean-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn boolean_value_false_source_slice() {
    let source = "false";
    let bv = BooleanValue {
        value: false,
        span: make_byte_span(0, 5),
        syntax: None,
    };
    assert!(!bv.value);

    let mut sink = String::new();
    bv.append_source(&mut sink, Some(source));
    assert_eq!(sink, "false");
}
