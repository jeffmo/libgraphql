//! Tests for Selection Sets.
//!
//! These tests verify that the parser correctly parses selection sets, fields,
//! fragment spreads, and inline fragments, validating the AST structure.
//!
//! Written by Claude Code, reviewed by a human.

use crate::legacy_ast;
use crate::tests::ast_utils::extract_query;
use crate::tests::ast_utils::extract_selection_set;
use crate::tests::ast_utils::field_at;
use crate::tests::ast_utils::first_field;
use crate::tests::ast_utils::first_fragment_spread;
use crate::tests::ast_utils::first_inline_fragment;
use crate::tests::utils::parse_executable;

// =============================================================================
// Selection Set Tests
// =============================================================================

/// Verifies that a simple selection set with a single field is correctly
/// parsed.
///
/// The parser should produce a SelectionSet with exactly one Field item
/// named "name".
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Selection-Sets>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn selection_set_simple() {
    let ss = extract_selection_set("{ name }");

    assert_eq!(ss.items.len(), 1);
    let field = first_field(&ss);
    assert_eq!(field.name, "name");
}

/// Verifies that a selection set with multiple fields is correctly parsed.
///
/// The parser should produce a SelectionSet with three Field items in order:
/// "name", "age", "email".
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Selection-Sets>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn selection_set_multiple_fields() {
    let ss = extract_selection_set("{ name age email }");

    assert_eq!(ss.items.len(), 3);
    assert_eq!(field_at(&ss, 0).name, "name");
    assert_eq!(field_at(&ss, 1).name, "age");
    assert_eq!(field_at(&ss, 2).name, "email");
}

/// Verifies that nested selection sets are correctly parsed.
///
/// The outer selection set should contain a "user" field with its own nested
/// selection set containing a "name" field.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Selection-Sets>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn selection_set_nested() {
    let ss = extract_selection_set("{ user { name } }");

    assert_eq!(ss.items.len(), 1);
    let user_field = first_field(&ss);
    assert_eq!(user_field.name, "user");

    // Verify nested selection set
    assert_eq!(user_field.selection_set.items.len(), 1);
    let nested_field = first_field(&user_field.selection_set);
    assert_eq!(nested_field.name, "name");
}

/// Verifies that an empty selection set `{ }` produces a parse error.
///
/// Per GraphQL spec, a selection set must contain at least one selection:
/// <https://spec.graphql.org/September2025/#sec-Selection-Sets>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn selection_set_empty_error() {
    let result = parse_executable("{ }");
    assert!(result.has_errors());
}

/// Verifies that an unclosed selection set produces a parse error.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Selection-Sets>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn selection_set_unclosed_error() {
    let result = parse_executable("{ name");
    assert!(result.has_errors());
}

// =============================================================================
// Field Tests
// =============================================================================

/// Verifies that a simple field is correctly parsed.
///
/// The field should have the name "name" with no alias, no arguments, and no
/// nested selection set.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fields>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn field_simple() {
    let query = extract_query("query { name }");
    let field = first_field(&query.selection_set);

    assert_eq!(field.name, "name");
    assert!(field.alias.is_none());
    assert!(field.arguments.is_empty());
}

/// Verifies that a field with an alias is correctly parsed.
///
/// The field should have alias "userName" and name "name" as separate values.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fields>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn field_with_alias() {
    let query = extract_query("query { userName: name }");
    let field = first_field(&query.selection_set);

    assert_eq!(field.alias.as_deref(), Some("userName"));
    assert_eq!(field.name, "name");
}

/// Verifies that a field with arguments is correctly parsed.
///
/// The field should have arguments populated (non-empty arguments list).
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fields>
/// <https://spec.graphql.org/September2025/#sec-Language.Arguments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn field_with_args() {
    let query = extract_query("query { user(id: 1) }");
    let field = first_field(&query.selection_set);

    assert_eq!(field.name, "user");
    assert!(!field.arguments.is_empty());
    assert_eq!(field.arguments.len(), 1);

    // Verify argument name
    let (arg_name, _) = &field.arguments[0];
    assert_eq!(arg_name, "id");
}

/// Verifies that a field with directives is correctly parsed.
///
/// The field should have directives populated (non-empty directives list).
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fields>
/// <https://spec.graphql.org/September2025/#sec-Language.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn field_with_directives() {
    let query = extract_query("query { name @include(if: true) }");
    let field = first_field(&query.selection_set);

    assert_eq!(field.name, "name");
    assert!(!field.directives.is_empty());
    assert_eq!(field.directives.len(), 1);
    assert_eq!(field.directives[0].name, "include");
}

/// Verifies that a field with a nested selection set is correctly parsed.
///
/// The "user" field should have a non-empty selection set containing "name".
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fields>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn field_with_nested_selection() {
    let query = extract_query("query { user { name } }");
    let field = first_field(&query.selection_set);

    assert_eq!(field.name, "user");
    assert!(!field.selection_set.items.is_empty());

    let nested_field = first_field(&field.selection_set);
    assert_eq!(nested_field.name, "name");
}

/// Verifies that empty field arguments `field()` produce a parse error.
///
/// Per GraphQL spec, if parentheses are present, at least one argument is
/// required:
/// <https://spec.graphql.org/September2025/#sec-Language.Arguments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn field_empty_args_error() {
    let result = parse_executable("{ field() }");
    assert!(result.has_errors());
}

// =============================================================================
// Fragment Spread Tests
// =============================================================================

/// Verifies that a fragment spread is correctly parsed.
///
/// The fragment spread should reference the fragment named "UserFields".
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
/// <https://spec.graphql.org/September2025/#FragmentSpread>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_spread() {
    let query = extract_query("query { ...UserFields }");
    let spread = first_fragment_spread(&query.selection_set);

    assert_eq!(spread.fragment_name, "UserFields");
}

/// Verifies that a fragment spread with directives is correctly parsed.
///
/// The fragment spread should have the directive attached.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
/// <https://spec.graphql.org/September2025/#FragmentSpread>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_spread_with_directives() {
    let query = extract_query("query { ...UserFields @include(if: true) }");
    let spread = first_fragment_spread(&query.selection_set);

    assert_eq!(spread.fragment_name, "UserFields");
    assert!(!spread.directives.is_empty());
    assert_eq!(spread.directives.len(), 1);
    assert_eq!(spread.directives[0].name, "include");
}

// =============================================================================
// Inline Fragment Tests
// =============================================================================

/// Verifies that a typed inline fragment is correctly parsed.
///
/// The inline fragment should have a type condition of "User" and contain
/// a "name" field in its selection set.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Inline-Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn inline_fragment_typed() {
    let query = extract_query("query { ... on User { name } }");
    let inline = first_inline_fragment(&query.selection_set);

    // Verify type condition
    assert!(inline.type_condition.is_some());
    let type_cond = inline.type_condition.as_ref().unwrap();
    match type_cond {
        legacy_ast::operation::TypeCondition::On(name) => assert_eq!(name, "User"),
    }

    // Verify selection set
    assert!(!inline.selection_set.items.is_empty());
    let field = first_field(&inline.selection_set);
    assert_eq!(field.name, "name");
}

/// Verifies that an untyped inline fragment is correctly parsed.
///
/// The inline fragment should have no type condition but still contain
/// a selection set with "name".
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Inline-Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn inline_fragment_untyped() {
    let query = extract_query("query { ... { name } }");
    let inline = first_inline_fragment(&query.selection_set);

    // Verify no type condition
    assert!(inline.type_condition.is_none());

    // Verify selection set
    assert!(!inline.selection_set.items.is_empty());
    let field = first_field(&inline.selection_set);
    assert_eq!(field.name, "name");
}

/// Verifies that an inline fragment with directives is correctly parsed.
///
/// The inline fragment should have both a type condition and directives.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Inline-Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn inline_fragment_with_directives() {
    let query = extract_query("query { ... on User @skip(if: $flag) { name } }");
    let inline = first_inline_fragment(&query.selection_set);

    // Verify type condition
    assert!(inline.type_condition.is_some());
    let type_cond = inline.type_condition.as_ref().unwrap();
    match type_cond {
        legacy_ast::operation::TypeCondition::On(name) => assert_eq!(name, "User"),
    }

    // Verify directive
    assert!(!inline.directives.is_empty());
    assert_eq!(inline.directives.len(), 1);
    assert_eq!(inline.directives[0].name, "skip");

    // Verify selection set
    assert!(!inline.selection_set.items.is_empty());
    let field = first_field(&inline.selection_set);
    assert_eq!(field.name, "name");
}
