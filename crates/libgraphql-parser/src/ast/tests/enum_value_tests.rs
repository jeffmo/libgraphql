//! Tests for [`crate::ast::EnumValue`].

use std::borrow::Cow;

use crate::ast::EnumValue;
use crate::ast::tests::ast_test_helpers::make_byte_span;

/// Verify `EnumValue` stores its value and slices the
/// correct source range.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Enum-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_value_construct_and_source_slice() {
    let source = "ACTIVE";
    let ev = EnumValue {
        value: Cow::Borrowed("ACTIVE"),
        span: make_byte_span(0, 6),
        syntax: None,
    };
    assert_eq!(ev.value, "ACTIVE");

    let mut sink = String::new();
    ev.append_source(&mut sink, Some(source));
    assert_eq!(sink, "ACTIVE");
}
