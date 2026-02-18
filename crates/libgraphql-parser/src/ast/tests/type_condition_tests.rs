//! Tests for [`crate::ast::TypeCondition`].

use crate::ast::TypeCondition;
use crate::ast::tests::ast_test_helpers::make_name;
use crate::ast::tests::ast_test_helpers::make_span;

/// Verify `TypeCondition` stores the named type and
/// slices correctly.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Type-Conditions
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_condition_construct_and_source_slice() {
    let source = "on User";
    let tc = TypeCondition {
        span: make_span(0, 7),
        named_type: make_name("User", 3, 7),
        syntax: None,
    };
    assert_eq!(tc.named_type.value, "User");

    let mut sink = String::new();
    tc.append_source(&mut sink, Some(source));
    assert_eq!(sink, "on User");
}
