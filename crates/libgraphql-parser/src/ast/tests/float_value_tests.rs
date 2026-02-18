//! Tests for [`crate::ast::FloatValue`].

use crate::ast::FloatValue;
use crate::ast::tests::ast_test_helpers::make_span;

/// Verify `FloatValue` stores the parsed f64 and slices
/// the correct source range.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Float-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn float_value_construct_and_source_slice() {
    let source = "1.25";
    let fv = FloatValue {
        value: 1.25,
        span: make_span(0, 4),
        syntax: None,
    };
    assert!((fv.value - 1.25).abs() < f64::EPSILON);

    let mut sink = String::new();
    fv.append_source(&mut sink, Some(source));
    assert_eq!(sink, "1.25");
}
