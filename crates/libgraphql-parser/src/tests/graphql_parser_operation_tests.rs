//! Tests for Operations (Part 2.5) and Fragments (Part 2.6) of the GraphQL
//! specification.
//!
//! These tests verify that the parser correctly parses operation definitions
//! (query, mutation, subscription) and fragment definitions, validating the
//! AST structure.
//!
//! Written by Claude Code, reviewed by a human.

use crate::legacy_ast;
use crate::tests::ast_utils::extract_fragment;
use crate::tests::ast_utils::extract_mutation;
use crate::tests::ast_utils::extract_query;
use crate::tests::ast_utils::extract_selection_set;
use crate::tests::ast_utils::extract_subscription;
use crate::tests::ast_utils::first_field;
use crate::tests::ast_utils::inner_type_name;
use crate::tests::utils::parse_executable;

// =============================================================================
// Operations
// =============================================================================

/// Verifies that a named query operation is correctly parsed.
///
/// The parser should produce a Query with name "GetUser" and a selection set
/// containing one field.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_query_named() {
    let query = extract_query("query GetUser { name }");

    assert_eq!(query.name.as_deref(), Some("GetUser"));
    assert_eq!(query.selection_set.items.len(), 1);
}

/// Verifies that an anonymous query operation is correctly parsed.
///
/// The parser should produce a Query with no name (None) when the query
/// keyword is used without a name.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_query_anonymous() {
    let query = extract_query("query { name }");

    assert!(query.name.is_none());
    assert_eq!(query.selection_set.items.len(), 1);
}

/// Verifies that the shorthand query form (just a selection set) is correctly
/// parsed.
///
/// The parser should produce a SelectionSet directly when only `{ ... }` is
/// provided without the `query` keyword.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_query_shorthand() {
    let ss = extract_selection_set("{ name }");

    assert_eq!(ss.items.len(), 1);
    let field = first_field(&ss);
    assert_eq!(field.name, "name");
}

/// Verifies that a mutation operation is correctly parsed.
///
/// The parser should produce a Mutation with the name "CreateUser" and
/// appropriate selection set.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_mutation() {
    let mutation =
        extract_mutation("mutation CreateUser { createUser { id } }");

    assert_eq!(mutation.name.as_deref(), Some("CreateUser"));
    assert_eq!(mutation.selection_set.items.len(), 1);

    let field = first_field(&mutation.selection_set);
    assert_eq!(field.name, "createUser");
}

/// Verifies that a subscription operation is correctly parsed.
///
/// The parser should produce a Subscription with the name "OnNewMessage" and
/// appropriate selection set.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_subscription() {
    let subscription =
        extract_subscription("subscription OnMessage { newMsg { text } }");

    assert_eq!(subscription.name.as_deref(), Some("OnMessage"));
    assert_eq!(subscription.selection_set.items.len(), 1);

    let field = first_field(&subscription.selection_set);
    assert_eq!(field.name, "newMsg");
}

/// Verifies that an operation with variable definitions is correctly parsed.
///
/// The parser should produce a Query with variable definitions containing the
/// variable name and type.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Variables>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_with_variables() {
    let query =
        extract_query("query Q($id: ID!, $name: String) { user(id: $id) }");

    assert_eq!(query.name.as_deref(), Some("Q"));
    assert_eq!(query.variable_definitions.len(), 2);

    // Verify first variable ($id: ID!)
    let var1 = &query.variable_definitions[0];
    assert_eq!(var1.name, "id");
    assert_eq!(inner_type_name(&var1.var_type), "ID");

    // Verify second variable ($name: String)
    let var2 = &query.variable_definitions[1];
    assert_eq!(var2.name, "name");
    assert_eq!(inner_type_name(&var2.var_type), "String");
}

/// Verifies that an operation with directives is correctly parsed.
///
/// The parser should produce a Query with directives attached.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
/// <https://spec.graphql.org/September2025/#sec-Language.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_with_directives() {
    let query = extract_query("query GetUser @deprecated { name }");

    assert_eq!(query.name.as_deref(), Some("GetUser"));
    assert!(!query.directives.is_empty());
    assert_eq!(query.directives.len(), 1);
    assert_eq!(query.directives[0].name, "deprecated");
}

/// Verifies that an operation with empty variable list `query()` produces a
/// parse error.
///
/// Per GraphQL spec, if parentheses are present, at least one variable must
/// be defined:
/// <https://spec.graphql.org/September2025/#sec-Language.Variables>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_empty_vars_error() {
    let result = parse_executable("query() { name }");
    assert!(result.has_errors());
}

/// Verifies that a variable with a default value is correctly parsed.
///
/// The parser should produce a Query with a variable definition that has a
/// default value.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Variables>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_var_default_value() {
    let query =
        extract_query("query GetUser($id: ID = \"123\") { user(id: $id) }");

    assert_eq!(query.variable_definitions.len(), 1);

    let var = &query.variable_definitions[0];
    assert_eq!(var.name, "id");
    assert!(var.default_value.is_some());
}

/// Verifies that GraphQL keywords can be used as operation names.
///
/// The parser should allow keywords like "query" and "type" as operation
/// names since they are contextually valid.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Names>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_name_is_keyword() {
    // "query" as operation name
    let query1 = extract_query("query query { name }");
    assert_eq!(query1.name.as_deref(), Some("query"));

    // "type" as operation name
    let query2 = extract_query("query type { name }");
    assert_eq!(query2.name.as_deref(), Some("type"));
}

// =============================================================================
// Fragments
// =============================================================================

/// Verifies that a simple fragment definition is correctly parsed.
///
/// The parser should produce a FragmentDefinition with the name "UserFields"
/// and type condition "User".
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_definition_simple() {
    let fragment =
        extract_fragment("fragment UserFields on User { name email }");

    assert_eq!(fragment.name, "UserFields");
    match &fragment.type_condition {
        legacy_ast::operation::TypeCondition::On(name) => assert_eq!(name, "User"),
    }
    assert_eq!(fragment.selection_set.items.len(), 2);
}

/// Verifies that a fragment definition with directives is correctly parsed.
///
/// The parser should produce a FragmentDefinition with directives attached.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
/// <https://spec.graphql.org/September2025/#sec-Language.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_with_directives() {
    let fragment =
        extract_fragment("fragment UserFields on User @deprecated { name }");

    assert_eq!(fragment.name, "UserFields");
    match &fragment.type_condition {
        legacy_ast::operation::TypeCondition::On(name) => assert_eq!(name, "User"),
    }
    assert!(!fragment.directives.is_empty());
    assert_eq!(fragment.directives.len(), 1);
    assert_eq!(fragment.directives[0].name, "deprecated");
}

/// Verifies that `fragment on on User` produces a parse error.
///
/// The fragment name "on" is reserved and cannot be used as a fragment name.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_name_on_error() {
    let result = parse_executable("fragment on on User { name }");
    assert!(result.has_errors());
}

/// Verifies that a fragment without a type condition produces a parse error.
///
/// Per GraphQL spec, fragments must have a type condition:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_missing_type_condition() {
    let result = parse_executable("fragment UserFields { name }");
    assert!(result.has_errors());
}

/// Verifies that a fragment with nested selections is correctly parsed.
///
/// The parser should produce a FragmentDefinition with nested fields in the
/// selection set.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_nested_selections() {
    let fragment = extract_fragment(
        "fragment UserFields on User { name address { city country } }",
    );

    assert_eq!(fragment.name, "UserFields");
    match &fragment.type_condition {
        legacy_ast::operation::TypeCondition::On(name) => assert_eq!(name, "User"),
    }
    assert_eq!(fragment.selection_set.items.len(), 2);

    // Verify nested selection
    let address_field = &fragment.selection_set.items[1];
    match address_field {
        legacy_ast::operation::Selection::Field(f) => {
            assert_eq!(f.name, "address");
            assert_eq!(f.selection_set.items.len(), 2);

            let city_field = first_field(&f.selection_set);
            assert_eq!(city_field.name, "city");
        },
        _ => panic!("Expected Field selection"),
    }
}
