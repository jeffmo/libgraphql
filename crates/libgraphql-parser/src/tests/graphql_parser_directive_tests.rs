//! Tests for Directive Annotations.
//!
//! These tests verify that directive annotations are correctly parsed and
//! their AST structure is accurate.
//!
//! Written by Claude Code, reviewed by a human.

use crate::tests::ast_utils::extract_first_object_type;
use crate::tests::utils::parse_schema;

/// Verifies that a simple directive `@deprecated` is parsed correctly with
/// the correct directive name and no arguments.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_simple() {
    let obj = extract_first_object_type("type Query { field: String @deprecated }");
    let field = &obj.fields[0];

    assert_eq!(field.directives.len(), 1);
    assert_eq!(field.directives[0].name, "deprecated");
    assert!(field.directives[0].arguments.is_empty());
}

/// Verifies that a directive with arguments `@deprecated(reason: "old")` is
/// parsed correctly, including argument name and count.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_with_args() {
    let obj = extract_first_object_type(
        r#"type Query { field: String @deprecated(reason: "old") }"#,
    );
    let field = &obj.fields[0];

    assert_eq!(field.directives.len(), 1);
    assert_eq!(field.directives[0].name, "deprecated");
    assert_eq!(field.directives[0].arguments.len(), 1);
    assert_eq!(field.directives[0].arguments[0].0, "reason");
}

/// Verifies that multiple directives `@a @b @c` are parsed as three separate
/// directive entries in the correct order.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_multiple() {
    let obj = extract_first_object_type("type Query { field: String @a @b @c }");
    let field = &obj.fields[0];

    assert_eq!(field.directives.len(), 3);
    assert_eq!(field.directives[0].name, "a");
    assert_eq!(field.directives[1].name, "b");
    assert_eq!(field.directives[2].name, "c");
}

/// Verifies that a directive with multiple arguments `@dir(a: 1, b: 2)` is
/// parsed correctly with both arguments present.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_arg_list() {
    let obj = extract_first_object_type("type Query { field: String @dir(a: 1, b: 2) }");
    let field = &obj.fields[0];

    assert_eq!(field.directives.len(), 1);
    assert_eq!(field.directives[0].name, "dir");
    assert_eq!(field.directives[0].arguments.len(), 2);
    assert_eq!(field.directives[0].arguments[0].0, "a");
    assert_eq!(field.directives[0].arguments[1].0, "b");
}

/// Verifies that empty directive arguments `@dir()` produce a parse error.
///
/// Per GraphQL spec, if parentheses are present, at least one argument is
/// required:
/// <https://spec.graphql.org/September2025/#sec-Language.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_empty_args_error() {
    let result = parse_schema("type Query { field: String @dir() }");
    assert!(result.has_errors());
}

/// Verifies that GraphQL keywords like `type` and `query` can be used as
/// directive names (`@type`, `@query`).
///
/// Per GraphQL spec, directive names can be any name:
/// <https://spec.graphql.org/September2025/#sec-Language.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_name_keyword() {
    // Test @type
    let obj_type = extract_first_object_type("type Query { field: String @type }");
    let field_type = &obj_type.fields[0];
    assert_eq!(field_type.directives.len(), 1);
    assert_eq!(field_type.directives[0].name, "type");

    // Test @query
    let obj_query = extract_first_object_type("type Query { field: String @query }");
    let field_query = &obj_query.fields[0];
    assert_eq!(field_query.directives.len(), 1);
    assert_eq!(field_query.directives[0].name, "query");
}
