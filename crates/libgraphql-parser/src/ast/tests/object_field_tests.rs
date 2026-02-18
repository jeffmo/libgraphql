//! Tests for [`crate::ast::ObjectField`] and
//! [`crate::ast::ObjectFieldSyntax`].

use crate::ast::IntValue;
use crate::ast::ObjectField;
use crate::ast::Value;
use crate::ast::tests::ast_test_helpers::make_byte_span;
use crate::ast::tests::ast_test_helpers::make_name;

/// Verify `ObjectField` stores name and value and
/// produces the correct source slice.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Input-Object-Values
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_field_source_slice() {
    let source = "x: 42";
    let of = ObjectField {
        name: make_name("x", 0, 1),
        span: make_byte_span(0, 5),
        syntax: None,
        value: Value::Int(IntValue {
            span: make_byte_span(3, 5),
            syntax: None,
            value: 42,
        }),
    };
    assert_eq!(of.name.value, "x");

    let mut sink = String::new();
    of.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
