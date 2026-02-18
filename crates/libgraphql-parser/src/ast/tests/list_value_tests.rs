//! Tests for [`crate::ast::ListValue`].

use crate::ast::IntValue;
use crate::ast::ListValue;
use crate::ast::Value;
use crate::ast::tests::ast_test_helpers::make_byte_span;

/// Verify `ListValue` stores a vector of `Value` items
/// and slices the correct source range.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-List-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn list_value_construct_and_source_slice() {
    let source = "[1, 2]";
    let lv = ListValue {
        values: vec![
            Value::Int(IntValue {
                value: 1,
                span: make_byte_span(1, 2),
                syntax: None,
            }),
            Value::Int(IntValue {
                value: 2,
                span: make_byte_span(4, 5),
                syntax: None,
            }),
        ],
        span: make_byte_span(0, 6),
        syntax: None,
    };
    assert_eq!(lv.values.len(), 2);

    let mut sink = String::new();
    lv.append_source(&mut sink, Some(source));
    assert_eq!(sink, "[1, 2]");
}
