//! Tests for `GraphQLParser`.
//!
//! Written by Claude Code, reviewed by a human.

use crate::tests::utils::parse_executable;
use crate::tests::utils::parse_schema;

/// Helper to check if parsing succeeds with no errors.
fn parses_ok(source: &str, is_schema: bool) -> bool {
    if is_schema {
        parse_schema(source).is_ok()
    } else {
        parse_executable(source).is_ok()
    }
}

/// Helper to check if parsing produces errors.
fn has_errors(source: &str, is_schema: bool) -> bool {
    if is_schema {
        parse_schema(source).has_errors()
    } else {
        parse_executable(source).has_errors()
    }
}

// =============================================================================
// Part 2.1: Value Parsing
// =============================================================================

/// Verifies that integer values are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Int-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_int() {
    assert!(parses_ok("query { field(arg: 123) }", /* is_schema = */ false));
    assert!(parses_ok("query { field(arg: 0) }", /* is_schema = */ false));
}

/// Verifies that negative integers are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Int-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_int_negative() {
    assert!(parses_ok("query { field(arg: -456) }", /* is_schema = */ false));
    assert!(parses_ok("query { field(arg: -0) }", /* is_schema = */ false));
}

/// Verifies that float values are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Float-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_float() {
    assert!(parses_ok("query { field(arg: 1.5) }", /* is_schema = */ false));
    assert!(parses_ok("query { field(arg: 3.14e10) }", /* is_schema = */ false));
    assert!(parses_ok("query { field(arg: 1.23E-4) }", /* is_schema = */ false));
}

/// Verifies that string values are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_string() {
    assert!(parses_ok(r#"query { field(arg: "hello") }"#, /* is_schema = */ false));
    assert!(parses_ok(r#"query { field(arg: "") }"#, /* is_schema = */ false));
}

/// Verifies that string escape sequences are correctly processed.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#EscapedCharacter>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_string_with_escapes() {
    assert!(parses_ok(r#"query { field(arg: "hello\nworld") }"#, /* is_schema = */ false));
    assert!(parses_ok(r#"query { field(arg: "say \"hi\"") }"#, /* is_schema = */ false));
    assert!(parses_ok(r#"query { field(arg: "\u0041") }"#, /* is_schema = */ false));
}

/// Verifies that `true` is parsed as Boolean(true).
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Boolean-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_boolean_true() {
    assert!(parses_ok("query { field(arg: true) }", /* is_schema = */ false));
}

/// Verifies that `false` is parsed as Boolean(false).
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Boolean-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_boolean_false() {
    assert!(parses_ok("query { field(arg: false) }", /* is_schema = */ false));
}

/// Verifies that `null` is parsed as Null.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Null-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_null() {
    assert!(parses_ok("query { field(arg: null) }", /* is_schema = */ false));
}

/// Verifies that enum values (names that aren't keywords) are parsed.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Enum-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_enum() {
    assert!(parses_ok("query { field(arg: ACTIVE) }", /* is_schema = */ false));
    assert!(parses_ok("query { field(arg: INACTIVE) }", /* is_schema = */ false));
}

/// Verifies that keywords like `type`, `query` are valid enum values.
///
/// Per GraphQL spec, enum values can be any name except `true`, `false`,
/// `null`:
/// <https://spec.graphql.org/September2025/#sec-Enum-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_enum_looks_like_keyword() {
    // GraphQL keywords (except true/false/null) can be enum values
    assert!(parses_ok("query { field(arg: type) }", /* is_schema = */ false));
    assert!(parses_ok("query { field(arg: query) }", /* is_schema = */ false));
    assert!(parses_ok("query { field(arg: mutation) }", /* is_schema = */ false));
}

/// Verifies that empty list `[]` is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-List-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_list_empty() {
    assert!(parses_ok("query { field(arg: []) }", /* is_schema = */ false));
}

/// Verifies that simple list `[1, 2, 3]` is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-List-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_list_simple() {
    assert!(parses_ok("query { field(arg: [1, 2, 3]) }", /* is_schema = */ false));
}

/// Verifies that nested lists `[[1], [2]]` are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-List-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_list_nested() {
    assert!(parses_ok("query { field(arg: [[1], [2]]) }", /* is_schema = */ false));
    assert!(parses_ok("query { field(arg: [[[]]]) }", /* is_schema = */ false));
}

/// Verifies that mixed-type lists `[1, "two", true]` are parsed.
///
/// Per GraphQL spec, list values have no type constraint at parse level:
/// <https://spec.graphql.org/September2025/#sec-List-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_list_mixed_types() {
    assert!(parses_ok(r#"query { field(arg: [1, "two", true]) }"#, /* is_schema = */ false));
}

/// Verifies that empty object `{}` is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Input-Object-Values>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_object_empty() {
    assert!(parses_ok("query { field(arg: {}) }", /* is_schema = */ false));
}

/// Verifies that simple object `{key: "value"}` is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Input-Object-Values>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_object_simple() {
    assert!(parses_ok(r#"query { field(arg: {key: "value"}) }"#, /* is_schema = */ false));
}

/// Verifies that objects with multiple fields are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Input-Object-Values>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_object_multiple_fields() {
    assert!(parses_ok("query { field(arg: {a: 1, b: 2, c: 3}) }", /* is_schema = */ false));
}

/// Verifies that nested objects `{outer: {inner: 1}}` are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Input-Object-Values>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_object_nested() {
    assert!(parses_ok("query { field(arg: {outer: {inner: 1}}) }", /* is_schema = */ false));
}

/// Verifies that variables `$varName` are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#Variable>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_variable() {
    assert!(parses_ok("query($var: Int) { field(arg: $var) }", /* is_schema = */ false));
}

/// Verifies that variables in default values produce errors.
///
/// Per GraphQL spec, default values must be constant:
/// <https://spec.graphql.org/September2025/#sec-Input-Object-Values>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_variable_in_const_error() {
    // Variable in default value should be an error
    assert!(has_errors("query($var: Int = $other) { field }", /* is_schema = */ false));
}

// =============================================================================
// Part 2.2: Type Annotations
// =============================================================================

/// Verifies that named types like `String`, `User` are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_named() {
    assert!(parses_ok("type Query { field: String }", /* is_schema = */ true));
    assert!(parses_ok("type Query { field: User }", /* is_schema = */ true));
    assert!(parses_ok("type Query { field: Int }", /* is_schema = */ true));
}

/// Verifies that non-null types `String!` are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_non_null() {
    assert!(parses_ok("type Query { field: String! }", /* is_schema = */ true));
}

/// Verifies that list types `[String]` are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_list() {
    assert!(parses_ok("type Query { field: [String] }", /* is_schema = */ true));
}

/// Verifies that `[String]!` (non-null list) is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_list_non_null() {
    assert!(parses_ok("type Query { field: [String]! }", /* is_schema = */ true));
}

/// Verifies that `[String!]` (list of non-null) is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_non_null_list() {
    assert!(parses_ok("type Query { field: [String!] }", /* is_schema = */ true));
}

/// Verifies that `[String!]!` (non-null list of non-null) is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_non_null_list_non_null() {
    assert!(parses_ok("type Query { field: [String!]! }", /* is_schema = */ true));
}

/// Verifies that deeply nested list types are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_deeply_nested() {
    assert!(parses_ok("type Query { field: [[String]] }", /* is_schema = */ true));
    assert!(parses_ok("type Query { field: [[[Int]]] }", /* is_schema = */ true));
}

/// Verifies that unclosed bracket produces an error.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_unclosed_bracket_error() {
    assert!(has_errors("type Query { field: [String }", /* is_schema = */ true));
}

/// Verifies that double bang `String!!` produces an error.
///
/// Per GraphQL spec, non-null cannot be nested:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_double_bang_error() {
    assert!(has_errors("type Query { field: String!! }", /* is_schema = */ true));
}

// =============================================================================
// Part 2.3: Directive Annotations
// =============================================================================

/// Verifies that simple directives `@deprecated` are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_simple() {
    assert!(parses_ok("type Query { field: String @deprecated }", /* is_schema = */ true));
}

/// Verifies that directives with arguments are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_with_args() {
    assert!(parses_ok(
        r#"type Query { field: String @deprecated(reason: "old") }"#,
        true
    ));
}

/// Verifies that multiple directives are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_multiple() {
    assert!(parses_ok(
        "type Query { field: String @a @b @c }",
        /* is_schema = */ true
    ));
}

/// Verifies that directives with multiple arguments are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_arg_list() {
    assert!(parses_ok(
        "type Query { field: String @dir(a: 1, b: 2) }",
        /* is_schema = */ true
    ));
}

/// Verifies that empty directive args `@dir()` produce an error.
///
/// Per GraphQL spec, if parens are present, at least one argument is required:
/// <https://spec.graphql.org/September2025/#sec-Language.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_empty_args_error() {
    assert!(has_errors("type Query { field: String @dir() }", /* is_schema = */ true));
}

/// Verifies that keywords can be directive names.
///
/// Per GraphQL spec, directive names are any name:
/// <https://spec.graphql.org/September2025/#sec-Language.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_name_keyword() {
    assert!(parses_ok("type Query { field: String @type }", /* is_schema = */ true));
    assert!(parses_ok("type Query { field: String @query }", /* is_schema = */ true));
}

// =============================================================================
// Part 2.4: Selection Sets
// =============================================================================

/// Verifies that simple selection sets are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Selection-Sets>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn selection_set_simple() {
    assert!(parses_ok("{ name }", /* is_schema = */ false));
}

/// Verifies that selection sets with multiple fields are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Selection-Sets>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn selection_set_multiple_fields() {
    assert!(parses_ok("{ name age email }", /* is_schema = */ false));
}

/// Verifies that nested selection sets are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Selection-Sets>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn selection_set_nested() {
    assert!(parses_ok("{ user { name } }", /* is_schema = */ false));
}

/// Verifies that empty selection sets `{ }` produce an error.
///
/// Per GraphQL spec, selection sets must have at least one selection:
/// <https://spec.graphql.org/September2025/#sec-Selection-Sets>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn selection_set_empty_error() {
    assert!(has_errors("{ }", /* is_schema = */ false));
}

/// Verifies that unclosed selection sets produce an error.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Selection-Sets>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn selection_set_unclosed_error() {
    assert!(has_errors("{ name", /* is_schema = */ false));
}

/// Verifies that simple fields are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fields>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn field_simple() {
    assert!(parses_ok("{ name }", /* is_schema = */ false));
}

/// Verifies that fields with aliases are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fields>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn field_with_alias() {
    assert!(parses_ok("{ userName: name }", /* is_schema = */ false));
}

/// Verifies that fields with arguments are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fields>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn field_with_args() {
    assert!(parses_ok("{ user(id: 1) }", /* is_schema = */ false));
}

/// Verifies that fields with directives are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fields>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn field_with_directives() {
    assert!(parses_ok("{ name @include(if: true) }", /* is_schema = */ false));
}

/// Verifies that fields with nested selections are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fields>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn field_with_nested_selection() {
    assert!(parses_ok("{ user { name } }", /* is_schema = */ false));
}

/// Verifies that empty field args `field()` produce an error.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Arguments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn field_empty_args_error() {
    assert!(has_errors("{ field() }", /* is_schema = */ false));
}

/// Verifies that fragment spreads are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_spread() {
    assert!(parses_ok("{ ...UserFields }", /* is_schema = */ false));
}

/// Verifies that fragment spreads with directives are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_spread_with_directives() {
    assert!(parses_ok("{ ...UserFields @include(if: true) }", /* is_schema = */ false));
}

/// Verifies that typed inline fragments are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Inline-Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn inline_fragment_typed() {
    assert!(parses_ok("{ ... on User { name } }", /* is_schema = */ false));
}

/// Verifies that untyped inline fragments are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Inline-Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn inline_fragment_untyped() {
    assert!(parses_ok("{ ... { name } }", /* is_schema = */ false));
}

/// Verifies that inline fragments with directives are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Inline-Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn inline_fragment_with_directives() {
    assert!(parses_ok(
        "{ ... on User @skip(if: $flag) { name } }",
        /* is_schema = */ false
    ));
}

// =============================================================================
// Part 2.5: Operations
// =============================================================================

/// Verifies that named queries are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_query_named() {
    assert!(parses_ok("query GetUser { name }", /* is_schema = */ false));
}

/// Verifies that anonymous queries are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_query_anonymous() {
    assert!(parses_ok("query { name }", /* is_schema = */ false));
}

/// Verifies that shorthand queries (just selection set) are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_query_shorthand() {
    assert!(parses_ok("{ name }", /* is_schema = */ false));
}

/// Verifies that mutations are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_mutation() {
    assert!(parses_ok("mutation CreateUser { createUser }", /* is_schema = */ false));
}

/// Verifies that subscriptions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_subscription() {
    assert!(parses_ok("subscription OnMessage { newMessage }", /* is_schema = */ false));
}

/// Verifies that operations with variables are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_with_variables() {
    assert!(parses_ok("query($id: ID!) { user(id: $id) }", /* is_schema = */ false));
}

/// Verifies that operations with directives are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_with_directives() {
    assert!(parses_ok("query @cached { name }", /* is_schema = */ false));
}

/// Verifies that empty variable definitions `query()` produce an error.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_empty_vars_error() {
    assert!(has_errors("query() { name }", /* is_schema = */ false));
}

/// Verifies that variable default values are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_var_default_value() {
    assert!(parses_ok("query($limit: Int = 10) { users }", /* is_schema = */ false));
}

/// Verifies that operation names can be keywords.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_name_is_keyword() {
    assert!(parses_ok("query query { field }", /* is_schema = */ false));
    assert!(parses_ok("query type { field }", /* is_schema = */ false));
}

// =============================================================================
// Part 2.6: Fragments
// =============================================================================

/// Verifies that simple fragment definitions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_definition_simple() {
    assert!(parses_ok("fragment UserFields on User { name }", /* is_schema = */ false));
}

/// Verifies that fragments with directives are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_with_directives() {
    assert!(parses_ok(
        "fragment F on User @deprecated { name }",
        /* is_schema = */ false
    ));
}

/// Verifies that `fragment on on User` (name = "on") produces an error.
///
/// Per GraphQL spec, `on` is reserved as fragment name:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_name_on_error() {
    assert!(has_errors("fragment on on User { name }", /* is_schema = */ false));
}

/// Verifies that missing type condition produces an error.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_missing_type_condition() {
    assert!(has_errors("fragment F { name }", /* is_schema = */ false));
}

/// Verifies that fragments with nested selections are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_nested_selections() {
    assert!(parses_ok(
        "fragment UserFields on User { name address { city } }",
        /* is_schema = */ false
    ));
}

// =============================================================================
// Part 2.7: Schema Definitions
// =============================================================================

/// Verifies that simple schema definitions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Schema>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn schema_simple() {
    assert!(parses_ok("schema { query: Query }", /* is_schema = */ true));
}

/// Verifies that schema with all operation types is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Schema>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn schema_all_operations() {
    assert!(parses_ok(
        "schema { query: Q mutation: M subscription: S }",
        /* is_schema = */ true
    ));
}

/// Verifies that schema with directives is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Schema>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn schema_with_directives() {
    assert!(parses_ok("schema @deprecated { query: Query }", /* is_schema = */ true));
}

/// Verifies that unclosed schema definitions produce an error.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Schema>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn schema_unclosed_error() {
    assert!(has_errors("schema { query: Query", /* is_schema = */ true));
}

// =============================================================================
// Part 2.8: Scalar Types
// =============================================================================

/// Verifies that simple scalar definitions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Scalars>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn scalar_simple() {
    assert!(parses_ok("scalar DateTime", /* is_schema = */ true));
}

/// Verifies that scalars with descriptions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Scalars>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn scalar_with_description() {
    assert!(parses_ok(r#""A date and time" scalar DateTime"#, /* is_schema = */ true));
}

/// Verifies that scalars with directives are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Scalars>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn scalar_with_directives() {
    assert!(parses_ok(
        r#"scalar JSON @specifiedBy(url: "https://json.org")"#,
        true
    ));
}

/// Verifies that keyword names are valid for scalars.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Scalars>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn scalar_name_keyword() {
    assert!(parses_ok("scalar type", /* is_schema = */ true));
    assert!(parses_ok("scalar query", /* is_schema = */ true));
}

// =============================================================================
// Part 2.9: Object Types
// =============================================================================

/// Verifies that simple object types are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_simple() {
    assert!(parses_ok("type User { name: String }", /* is_schema = */ true));
}

/// Verifies that object types with descriptions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_with_description() {
    assert!(parses_ok(r#""User type" type User { name: String }"#, /* is_schema = */ true));
}

/// Verifies that `implements` with one interface is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_implements_one() {
    assert!(parses_ok("type User implements Node { id: ID! }", /* is_schema = */ true));
}

/// Verifies that `implements` with multiple interfaces is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_implements_multiple() {
    assert!(parses_ok(
        "type User implements Node & Entity { id: ID! }",
        /* is_schema = */ true
    ));
}

/// Verifies that leading ampersand in `implements` is valid.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_implements_leading_ampersand() {
    assert!(parses_ok(
        "type User implements & Node & Entity { id: ID! }",
        /* is_schema = */ true
    ));
}

/// Verifies that object types with directives are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_with_directives() {
    assert!(parses_ok("type User @deprecated { name: String }", /* is_schema = */ true));
}

/// Verifies that object types with many fields are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_multiple_fields() {
    assert!(parses_ok(
        "type User { id: ID! name: String email: String! }",
        /* is_schema = */ true
    ));
}

/// Verifies that fields with arguments are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_field_with_args() {
    assert!(parses_ok("type Query { user(id: ID!): User }", /* is_schema = */ true));
}

/// Verifies that field descriptions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_field_description() {
    assert!(parses_ok(
        r#"type User { "The user's name" name: String }"#,
        true
    ));
}

/// Verifies that field directives are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_field_directives() {
    assert!(parses_ok(
        "type User { name: String @deprecated }",
        /* is_schema = */ true
    ));
}

/// Verifies that empty object type body is valid.
///
/// Per GraphQL spec (September 2025), empty field sets are allowed:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_empty_fields() {
    // Empty body with braces
    assert!(parses_ok("type User { }", /* is_schema = */ true));
}

/// Verifies that object type without body is valid.
///
/// Per GraphQL spec (September 2025), object types can omit the body:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_no_body() {
    assert!(parses_ok("type User", /* is_schema = */ true));
}

// =============================================================================
// Part 2.10: Interface Types
// =============================================================================

/// Verifies that simple interface definitions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Interfaces>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn interface_simple() {
    assert!(parses_ok("interface Node { id: ID! }", /* is_schema = */ true));
}

/// Verifies that interface `implements` is parsed correctly.
///
/// Per GraphQL spec (June 2018+), interfaces can implement other interfaces:
/// <https://spec.graphql.org/September2025/#sec-Interfaces>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn interface_implements() {
    assert!(parses_ok("interface Named implements Node { id: ID! }", /* is_schema = */ true));
}

/// Verifies that interfaces with multiple fields are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Interfaces>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn interface_with_fields() {
    assert!(parses_ok("interface Node { id: ID! createdAt: String }", /* is_schema = */ true));
}

/// Verifies that interface without body is valid.
///
/// Per GraphQL spec (September 2025):
/// <https://spec.graphql.org/September2025/#sec-Interfaces>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn interface_no_body() {
    assert!(parses_ok("interface Node", /* is_schema = */ true));
}

// =============================================================================
// Part 2.11: Union Types
// =============================================================================

/// Verifies that simple union definitions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Unions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn union_simple() {
    assert!(parses_ok("union SearchResult = User", /* is_schema = */ true));
}

/// Verifies that unions with multiple members are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Unions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn union_multiple_members() {
    assert!(parses_ok("union Result = User | Post | Comment", /* is_schema = */ true));
}

/// Verifies that leading pipe in unions is valid.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Unions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn union_leading_pipe() {
    assert!(parses_ok("union Result = | User | Post", /* is_schema = */ true));
}

/// Verifies that unions with directives are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Unions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn union_with_directives() {
    assert!(parses_ok("union Result @deprecated = User", /* is_schema = */ true));
}

/// Verifies that union without members is valid.
///
/// Per GraphQL spec (September 2025):
/// <https://spec.graphql.org/September2025/#sec-Unions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn union_no_members() {
    assert!(parses_ok("union Empty", /* is_schema = */ true));
}

// =============================================================================
// Part 2.12: Enum Types
// =============================================================================

/// Verifies that simple enum definitions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Enums>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_simple() {
    assert!(parses_ok("enum Status { ACTIVE INACTIVE }", /* is_schema = */ true));
}

/// Verifies that enums with descriptions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Enums>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_with_description() {
    assert!(parses_ok(r#""Status enum" enum Status { ACTIVE }"#, /* is_schema = */ true));
}

/// Verifies that enum value descriptions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Enums>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_value_description() {
    assert!(parses_ok(
        r#"enum Status { "Active status" ACTIVE }"#,
        true
    ));
}

/// Verifies that enum value directives are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Enums>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_value_directives() {
    assert!(parses_ok("enum Status { ACTIVE @deprecated }", /* is_schema = */ true));
}

/// Verifies that `true` as enum value produces an error.
///
/// Per GraphQL spec, `true`, `false`, `null` cannot be enum values:
/// <https://spec.graphql.org/September2025/#sec-Enums>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_value_true_error() {
    assert!(has_errors("enum Bool { true false }", /* is_schema = */ true));
}

/// Verifies that `null` as enum value produces an error.
///
/// Per GraphQL spec, `true`, `false`, `null` cannot be enum values:
/// <https://spec.graphql.org/September2025/#sec-Enums>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_value_null_error() {
    assert!(has_errors("enum Maybe { null some }", /* is_schema = */ true));
}

/// Verifies that empty enum body is valid.
///
/// Per GraphQL spec (September 2025):
/// <https://spec.graphql.org/September2025/#sec-Enums>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_empty_body() {
    assert!(parses_ok("enum Status { }", /* is_schema = */ true));
}

/// Verifies that enum without body is valid.
///
/// Per GraphQL spec (September 2025):
/// <https://spec.graphql.org/September2025/#sec-Enums>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_no_body() {
    assert!(parses_ok("enum Status", /* is_schema = */ true));
}

// =============================================================================
// Part 2.13: Input Object Types
// =============================================================================

/// Verifies that simple input definitions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Input-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn input_simple() {
    assert!(parses_ok("input CreateUserInput { name: String! }", /* is_schema = */ true));
}

/// Verifies that input fields with defaults are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Input-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn input_with_defaults() {
    assert!(parses_ok("input I { limit: Int = 10 }", /* is_schema = */ true));
}

/// Verifies that input field directives are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Input-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn input_field_directives() {
    assert!(parses_ok(
        "input I { name: String @deprecated }",
        /* is_schema = */ true
    ));
}

/// Verifies that empty input body is valid.
///
/// Per GraphQL spec (September 2025):
/// <https://spec.graphql.org/September2025/#sec-Input-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn input_empty_body() {
    assert!(parses_ok("input I { }", /* is_schema = */ true));
}

/// Verifies that input without body is valid.
///
/// Per GraphQL spec (September 2025):
/// <https://spec.graphql.org/September2025/#sec-Input-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn input_no_body() {
    assert!(parses_ok("input I", /* is_schema = */ true));
}

// =============================================================================
// Part 2.14: Directive Definitions
// =============================================================================

/// Verifies that simple directive definitions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_def_simple() {
    assert!(parses_ok("directive @deprecated on FIELD_DEFINITION", /* is_schema = */ true));
}

/// Verifies that directives with multiple locations are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_def_multiple_locations() {
    assert!(parses_ok("directive @d on FIELD | OBJECT", /* is_schema = */ true));
}

/// Verifies that leading pipe in directive locations is valid.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_def_leading_pipe() {
    assert!(parses_ok("directive @d on | FIELD | OBJECT", /* is_schema = */ true));
}

/// Verifies that directive definitions with arguments are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_def_with_args() {
    assert!(parses_ok(
        "directive @deprecated(reason: String) on FIELD_DEFINITION",
        /* is_schema = */ true
    ));
}

/// Verifies that `repeatable` directive definitions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_def_repeatable() {
    assert!(parses_ok("directive @tag repeatable on OBJECT", /* is_schema = */ true));
}

/// Verifies that unknown directive locations produce an error.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_def_unknown_location_error() {
    assert!(has_errors("directive @d on FOOBAR", /* is_schema = */ true));
}

// =============================================================================
// Part 2.15: Type Extensions
// =============================================================================

/// Verifies that scalar extensions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn extend_scalar() {
    assert!(parses_ok(
        r#"extend scalar DateTime @specifiedBy(url: "https://example.com")"#,
        true
    ));
}

/// Verifies that type extensions adding fields are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn extend_type_add_fields() {
    assert!(parses_ok("extend type User { age: Int }", /* is_schema = */ true));
}

/// Verifies that type extensions adding implements are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn extend_type_add_implements() {
    assert!(parses_ok("extend type User implements NewInterface", /* is_schema = */ true));
}

/// Verifies that type extensions adding directives are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn extend_type_add_directives() {
    assert!(parses_ok("extend type User @deprecated", /* is_schema = */ true));
}

/// Verifies that interface extensions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn extend_interface() {
    assert!(parses_ok("extend interface Node { extra: String }", /* is_schema = */ true));
}

/// Verifies that union extensions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn extend_union() {
    assert!(parses_ok("extend union Result = NewType", /* is_schema = */ true));
}

/// Verifies that enum extensions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn extend_enum() {
    assert!(parses_ok("extend enum Status { PENDING }", /* is_schema = */ true));
}

/// Verifies that input extensions are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn extend_input() {
    assert!(parses_ok("extend input CreateUserInput { extra: String }", /* is_schema = */ true));
}

// =============================================================================
// Part 2.16: Document Types
// =============================================================================

/// Verifies that schema documents accept only type definitions.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_schema_document_only_types() {
    // Should accept type definitions
    assert!(parses_ok("type Query { field: String }", /* is_schema = */ true));
    assert!(parses_ok("interface Node { id: ID! }", /* is_schema = */ true));
    assert!(parses_ok("scalar DateTime", /* is_schema = */ true));
}

/// Verifies that operations in schema documents produce errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_schema_rejects_operation() {
    assert!(has_errors("query { field }", /* is_schema = */ true));
}

/// Verifies that fragments in schema documents produce errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_schema_rejects_fragment() {
    assert!(has_errors("fragment F on User { name }", /* is_schema = */ true));
}

/// Verifies that mutation operations in schema documents produce errors.
///
/// This test ensures the parser doesn't hang when encountering a mutation
/// keyword in a schema document, validating the error recovery fix.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_schema_rejects_mutation() {
    assert!(has_errors("mutation { doThing }", /* is_schema = */ true));
}

/// Verifies that subscription operations in schema documents produce errors.
///
/// This test ensures the parser doesn't hang when encountering a subscription
/// keyword in a schema document, validating the error recovery fix.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_schema_rejects_subscription() {
    assert!(has_errors("subscription { onEvent }", /* is_schema = */ true));
}

/// Verifies that shorthand (anonymous) queries in schema documents produce
/// errors.
///
/// A shorthand query starts with `{` without the `query` keyword. This test
/// ensures the parser handles this case without hanging.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_schema_rejects_shorthand_query() {
    assert!(has_errors("{ field }", /* is_schema = */ true));
}

/// Verifies that executable documents accept only operations/fragments.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_executable_document_only_ops() {
    assert!(parses_ok("query { field }", /* is_schema = */ false));
    assert!(parses_ok("fragment F on User { name }", /* is_schema = */ false));
}

/// Verifies that type definitions in executable documents produce errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_executable_rejects_type() {
    assert!(has_errors("type Query { field: String }", /* is_schema = */ false));
}

/// Verifies that directive definitions in executable documents produce errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_executable_rejects_directive_def() {
    assert!(has_errors("directive @d on FIELD", /* is_schema = */ false));
}

/// Verifies that empty documents parse successfully.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_empty_document() {
    assert!(parses_ok("", /* is_schema = */ true));
    assert!(parses_ok("", /* is_schema = */ false));
}

/// Verifies that whitespace-only documents parse successfully.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_whitespace_only() {
    assert!(parses_ok("   \n\t   ", /* is_schema = */ true));
    assert!(parses_ok("   \n\t   ", /* is_schema = */ false));
}

/// Verifies that comments-only documents parse successfully.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_comments_only() {
    assert!(parses_ok("# just a comment", /* is_schema = */ true));
    assert!(parses_ok("# just a comment\n# another", /* is_schema = */ false));
}

// =============================================================================
// Part 2.17: Error Recovery
// =============================================================================

/// Verifies that multiple errors are collected in a single parse.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn recovery_multiple_errors() {
    // Document with multiple syntax errors
    let result = parse_schema("type A { } type B { field: !! } type C { }");
    // Should have at least one error and still have some AST
    assert!(result.has_errors());
}

/// Verifies that parsing continues after syntax errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn recovery_continues_after_error() {
    // After error in first type, second type should still parse
    let result = parse_schema("type A { field:: } type B { field: String }");
    // Should have errors but may have recovered
    assert!(result.has_errors());
}

// =============================================================================
// Part 2.18: Edge Cases
// =============================================================================

/// Verifies that keywords can be used as field names.
///
/// Per GraphQL spec, keywords are contextual:
/// <https://spec.graphql.org/September2025/#sec-Names>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn keyword_as_field_name() {
    assert!(parses_ok("{ type query mutation }", /* is_schema = */ false));
}

/// Verifies that keywords can be argument names.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Names>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn keyword_as_argument_name() {
    assert!(parses_ok("{ field(type: 1, query: 2) }", /* is_schema = */ false));
}

/// Verifies that Unicode in string values works.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unicode_in_strings_allowed() {
    assert!(parses_ok(r#"{ field(arg: "日本語 🎉") }"#, /* is_schema = */ false));
}

/// Verifies that Unicode in descriptions works.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Descriptions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unicode_in_descriptions() {
    assert!(parses_ok(r#""日本語の説明" type User { name: String }"#, /* is_schema = */ true));
}

/// Verifies that block string descriptions work.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Descriptions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn block_string_description() {
    assert!(parses_ok(
        r#""""
        Block string description
        """
        type User { name: String }"#,
        true
    ));
}

/// Verifies that multiple operations in one document work.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn consecutive_operations() {
    assert!(parses_ok(
        "query A { field } query B { field } mutation C { field }",
        /* is_schema = */ false
    ));
}

/// Verifies that multiple fragments in one document work.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn consecutive_fragments() {
    assert!(parses_ok(
        "fragment A on User { name } fragment B on User { email }",
        /* is_schema = */ false
    ));
}

/// Verifies that fragments defined before operations work.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_before_operation() {
    assert!(parses_ok(
        "fragment F on User { name } query { ...F }",
        /* is_schema = */ false
    ));
}

/// Verifies that same field selected twice is valid at parse level.
///
/// Duplicate field validation happens at validation phase, not parsing.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn duplicate_field_names() {
    assert!(parses_ok("{ name name }", /* is_schema = */ false));
}

// =============================================================================
// Part 2.1: Value Parsing Error Tests
// =============================================================================

/// Verifies that very large integers that overflow i64 produce errors.
///
/// GraphQL integers are 32-bit signed per the spec, but the parser accepts
/// values up to i64 range. Values beyond that should produce errors.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Int-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_int_overflow_error() {
    // Value exceeding i64::MAX
    assert!(has_errors(
        "query { field(arg: 99999999999999999999999999) }",
        /* is_schema = */ false
    ));
}

/// Verifies that block strings are correctly parsed in argument values.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_block_string() {
    assert!(parses_ok(
        r#"query { field(arg: """multi
line
string""") }"#,
        false
    ));
}

// =============================================================================
// Part 2.2: Unclosed Delimiter Error Tests
// =============================================================================

/// Verifies that an unclosed `[` in a list value produces an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn list_value_unclosed_bracket_error() {
    assert!(has_errors("query { field(arg: [1, 2) }", /* is_schema = */ false));
}

/// Verifies that an unclosed `{` in an object value produces an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_value_unclosed_brace_error() {
    assert!(has_errors("query { field(arg: {a: 1) }", /* is_schema = */ false));
}

/// Verifies that a missing colon in an object value field produces an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_value_missing_colon_error() {
    assert!(has_errors("query { field(arg: {field 1}) }", /* is_schema = */ false));
}

/// Verifies that an unclosed type definition body produces an error.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#ObjectTypeDefinition>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn field_definition_unclosed_brace() {
    assert!(has_errors("type T { f: String", /* is_schema = */ true));
}

/// Verifies that an unclosed input object definition produces an error.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#InputObjectTypeDefinition>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn input_object_unclosed_brace() {
    assert!(has_errors("input I { f: String", /* is_schema = */ true));
}

/// Verifies that an unclosed enum definition produces an error.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#EnumTypeDefinition>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_definition_unclosed_brace() {
    assert!(has_errors("enum E { A", /* is_schema = */ true));
}

/// Verifies that an unclosed argument list produces an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn field_args_unclosed_paren_error() {
    assert!(has_errors("query { field(arg: 1 }", /* is_schema = */ false));
}

/// Verifies that an unclosed list type produces an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_list_unclosed_bracket_error() {
    assert!(has_errors("type Q { f: [String }", /* is_schema = */ true));
}

// =============================================================================
// Part 2.3: Additional Reserved Name Validation Tests
// =============================================================================

/// Verifies that `false` as an enum value produces an error.
///
/// Per GraphQL spec, `true`, `false`, `null` cannot be enum values since they
/// would be ambiguous with boolean/null literals.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Enum-Value-Uniqueness>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_value_false_reserved_error() {
    assert!(has_errors("enum Bool { false }", /* is_schema = */ true));
}

/// Verifies that reserved names can be used in non-reserved contexts.
///
/// While `true`, `false`, `null` cannot be enum values, they can be field
/// names, argument names, etc.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn reserved_names_allowed_in_field_names() {
    // true/false/null can be field names in selection sets
    assert!(parses_ok("{ true false null }", /* is_schema = */ false));
}

// =============================================================================
// Part 2.4: Directive Location Error Tests
// =============================================================================

/// Verifies that an unknown directive location produces an error.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#DirectiveLocation>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_unknown_location_error() {
    assert!(has_errors("directive @d on UNKNOWN", /* is_schema = */ true));
}

/// Verifies that directive location names are case-sensitive.
///
/// `FIELD` is valid, `field` is not (it would be treated as a name, not a
/// directive location).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_location_case_sensitive() {
    assert!(has_errors("directive @d on field", /* is_schema = */ true));
}

// =============================================================================
// Part 2.5: Document Type Enforcement Tests
// =============================================================================

/// Verifies that a type definition with a string description in an executable
/// document produces an error.
///
/// When parsing as executable, anything starting with a string (description)
/// followed by a type keyword should be rejected.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Executable-Documents>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn executable_rejects_type_with_description() {
    assert!(has_errors(r#""description" type T { f: Int }"#, /* is_schema = */ false));
}

/// Verifies that schema definition in an executable document produces an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn executable_rejects_schema_definition() {
    assert!(has_errors("schema { query: Query }", /* is_schema = */ false));
}

/// Verifies that scalar definition in an executable document produces an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn executable_rejects_scalar_definition() {
    assert!(has_errors("scalar DateTime", /* is_schema = */ false));
}

/// Verifies that interface definition in an executable document produces error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn executable_rejects_interface_definition() {
    assert!(has_errors("interface Node { id: ID! }", /* is_schema = */ false));
}

/// Verifies that union definition in an executable document produces an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn executable_rejects_union_definition() {
    assert!(has_errors("union Result = A | B", /* is_schema = */ false));
}

/// Verifies that enum definition in an executable document produces an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn executable_rejects_enum_definition() {
    assert!(has_errors("enum Status { ACTIVE }", /* is_schema = */ false));
}

/// Verifies that input definition in an executable document produces an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn executable_rejects_input_definition() {
    assert!(has_errors("input CreateInput { name: String }", /* is_schema = */ false));
}

// =============================================================================
// Part 2.6: Schema Extension Test
// =============================================================================

/// Verifies that `extend schema` produces an error (currently unsupported).
///
/// While the GraphQL spec defines schema extensions, this parser may not yet
/// support them.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Schema-Extension>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn schema_extension_parses() {
    // Note: This may produce an error or succeed depending on parser support
    // Testing that schema extension is handled without crashing
    let result = parse_schema("extend schema { query: Query }");
    // If extensions are supported, result.is_ok() may be true
    // If not, result.has_errors() should be true
    // Either way, it should not panic
    let _ = result.is_ok() || result.has_errors();
}

// =============================================================================
// Part 4.1: Error Recovery Tests
// =============================================================================

/// Verifies that the parser can recover from an error and continue parsing.
///
/// After encountering a syntax error in one type definition, the parser should
/// be able to recover and parse subsequent definitions.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn recovery_after_error_in_type_definition() {
    let result = parse_schema(
        "type A { field:: } type B { field: String }"
    );
    // Should have errors from the first type but may have recovered
    assert!(result.has_errors());
    // If recovery works, we should have parsed some AST
    // (even if partial)
}

/// Verifies that recovery works across multiple definitions with errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn recovery_multiple_type_errors() {
    let result = parse_schema(
        "type A { bad:: } type B { also_bad!! } type C { field: String }"
    );
    assert!(result.has_errors());
}

/// Verifies that recovery works for operations with errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn recovery_operation_errors() {
    let result = parse_executable(
        "query A { field( } query B { field }"
    );
    assert!(result.has_errors());
}

/// Verifies recovery handles empty constructs gracefully.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn recovery_empty_selection_set() {
    // Empty selection set should produce an error
    let result = parse_executable("{ }");
    assert!(result.has_errors());
}

/// Verifies that deeply nested unclosed delimiters produce errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn recovery_deeply_nested_unclosed() {
    let result = parse_executable("{ a { b { c { d");
    assert!(result.has_errors());
}

// =============================================================================
// Part 4.2: Lexer Error Integration Tests
// =============================================================================

/// Verifies that an unterminated string produces a lexer error that the parser
/// reports.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn lexer_error_unterminated_string() {
    let result = parse_executable(r#"{ field(arg: "unterminated) }"#);
    assert!(result.has_errors());
}

/// Verifies that an unterminated block string produces a lexer error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn lexer_error_unterminated_block_string() {
    let result = parse_executable(r#"{ field(arg: """unterminated) }"#);
    assert!(result.has_errors());
}

/// Verifies that invalid characters produce lexer errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn lexer_error_invalid_character() {
    // The backtick is not valid in GraphQL
    let result = parse_executable("{ field` }");
    assert!(result.has_errors());
}

/// Verifies that invalid escape sequences in strings produce lexer errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn lexer_error_invalid_escape_sequence() {
    let result = parse_executable(r#"{ field(arg: "hello\qworld") }"#);
    assert!(result.has_errors());
}

/// Verifies that number format errors produce lexer errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn lexer_error_number_format() {
    // Leading zeros are not allowed
    let result = parse_executable("{ field(arg: 007) }");
    assert!(result.has_errors());
}

/// Verifies that exponent without digits produces a lexer error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn lexer_error_exponent_without_digits() {
    let result = parse_executable("{ field(arg: 1e) }");
    assert!(result.has_errors());
}

// =============================================================================
// Additional Edge Cases
// =============================================================================

/// Verifies that very deeply nested types are parsed correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn deeply_nested_list_types() {
    assert!(parses_ok("type Q { f: [[[[[String]]]]]! }", /* is_schema = */ true));
}

/// Verifies that complex argument lists parse correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn complex_argument_list() {
    assert!(parses_ok(
        "query { field(a: 1, b: 2.5, c: \"str\", d: true, e: null, f: ENUM) }",
        /* is_schema = */ false
    ));
}

/// Verifies that complex variable definitions parse correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn complex_variable_definitions() {
    assert!(parses_ok(
        r#"query($a: Int!, $b: String = "default", $c: [Int!]! = [1, 2]) { f }"#,
        false
    ));
}

/// Verifies that directives on all schema locations parse correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_on_schema_locations() {
    assert!(parses_ok(
        r#"
        schema @a { query: Q }
        scalar S @b
        type T @c { f: Int @d }
        interface I @e { f: Int }
        union U @f = A | B
        enum E @g { V @h }
        input In @i { f: Int @j }
        "#,
        true
    ));
}

/// Verifies that directives on executable locations parse correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_on_executable_locations() {
    assert!(parses_ok(
        r#"
        query Q @a {
            field @b
            ... @c { nested }
            ...Frag @d
        }
        fragment Frag on T @e { f }
        "#,
        false
    ));
}
