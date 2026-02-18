//! Tests for [`crate::ast::ObjectValue`] and
//! [`crate::ast::ObjectField`].

use crate::ast::IntValue;
use crate::ast::ObjectField;
use crate::ast::ObjectValue;
use crate::ast::Value;
use crate::ast::tests::ast_test_helpers::make_name;
use crate::ast::tests::ast_test_helpers::make_span;

/// Verify `ObjectValue` and `ObjectField` store fields
/// correctly and slice the right source range.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Input-Object-Values
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_value_construct_and_source_slice() {
    let source = "{x: 1}";
    let ov = ObjectValue {
        fields: vec![ObjectField {
            name: make_name("x", 1, 2),
            value: Value::Int(IntValue {
                value: 1,
                span: make_span(4, 5),
                syntax: None,
            }),
            span: make_span(1, 5),
            syntax: None,
        }],
        span: make_span(0, 6),
        syntax: None,
    };
    assert_eq!(ov.fields.len(), 1);
    assert_eq!(ov.fields[0].name.value, "x");

    let mut sink = String::new();
    ov.append_source(&mut sink, Some(source));
    assert_eq!(sink, "{x: 1}");
}
