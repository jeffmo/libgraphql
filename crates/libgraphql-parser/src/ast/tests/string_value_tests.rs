//! Tests for [`crate::ast::StringValue`].

use std::borrow::Cow;

use crate::ast::StringValue;
use crate::ast::tests::ast_test_helpers::make_span;

/// Verify `StringValue` stores the processed string and
/// slices the correct source range (including quotes).
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-String-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn string_value_construct_and_source_slice() {
    let source = r#""hello""#;
    let sv = StringValue {
        value: Cow::Borrowed("hello"),
        span: make_span(0, 7),
        syntax: None,
    };
    assert_eq!(sv.value, "hello");

    let mut sink = String::new();
    sv.append_source(&mut sink, Some(source));
    assert_eq!(sink, r#""hello""#);
}
