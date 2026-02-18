//! Tests for [`crate::ast::VariableValue`].

use crate::ast::VariableValue;
use crate::ast::tests::ast_test_helpers::make_name;
use crate::ast::tests::ast_test_helpers::make_span;

/// Verify `VariableValue` stores a `Name` for the
/// variable and slices the correct source range
/// (including the `$` prefix).
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Language.Variables
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn variable_value_construct_and_source_slice() {
    let source = "$userId";
    // The name portion is "userId" at bytes 1..7
    let vv = VariableValue {
        name: make_name("userId", 1, 7),
        span: make_span(0, 7),
        syntax: None,
    };
    assert_eq!(vv.name.value, "userId");

    let mut sink = String::new();
    vv.append_source(&mut sink, Some(source));
    assert_eq!(sink, "$userId");
}
