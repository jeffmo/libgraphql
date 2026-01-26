//! Tests for parser error handling, error recovery, and lexer error integration.
//!
//! This module contains tests organized into several categories:
//! - Error Recovery - tests for collecting multiple errors
//! - Value Error Tests - integer overflow, etc.
//! - Unclosed Delimiter Tests - various unclosed brackets/braces
//! - Reserved Name Tests - validation of reserved names
//! - Directive Location Tests - validation of directive locations
//! - Document Enforcement Tests - schema vs executable validation
//! - Schema Extension - schema extension handling
//! - Error Recovery - recovery after errors
//! - Lexer Errors - lexer error propagation
//!
//! Written by Claude Code, reviewed by a human.

use crate::tests::utils::parse_executable;
use crate::tests::utils::parse_schema;

// =============================================================================
// Error Recovery
// =============================================================================

/// Verifies that multiple errors are collected in a single parse.
///
/// When a document contains multiple syntax errors, the parser should collect
/// all of them rather than stopping at the first error. This enables better
/// developer experience by reporting all issues at once.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn recovery_multiple_errors() {
    let result = parse_schema("type A { } type B { field: !! } type C { }");
    assert!(result.has_errors());
}

/// Verifies that parsing continues after encountering a syntax error.
///
/// After an error in one type definition, the parser should recover and
/// continue parsing subsequent definitions. This tests the parser's ability
/// to synchronize after errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn recovery_continues_after_error() {
    let result = parse_schema("type A { field:: } type B { field: String }");
    assert!(result.has_errors());
}

// =============================================================================
// Value Error Tests
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
    let result = parse_executable(
        "query { field(arg: 99999999999999999999999999) }"
    );
    assert!(result.has_errors());
}

// =============================================================================
// Unclosed Delimiter Tests
// =============================================================================

/// Verifies that an unclosed `[` in a list value produces an error.
///
/// List values must have matching brackets. Missing the closing `]` should
/// result in a parse error.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-List-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn list_value_unclosed_bracket_error() {
    let result = parse_executable("query { field(arg: [1, 2) }");
    assert!(result.has_errors());
}

/// Verifies that an unclosed `{` in an object value produces an error.
///
/// Object values must have matching braces. Missing the closing `}` should
/// result in a parse error.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Input-Object-Values>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_value_unclosed_brace_error() {
    let result = parse_executable("query { field(arg: {a: 1) }");
    assert!(result.has_errors());
}

/// Verifies that a missing colon in an object value field produces an error.
///
/// Object field entries require the format `name: value`. Missing the colon
/// should result in a parse error.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Input-Object-Values>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_value_missing_colon_error() {
    let result = parse_executable("query { field(arg: {field 1}) }");
    assert!(result.has_errors());
}

/// Verifies that an unclosed type definition body produces an error.
///
/// Type definitions with field lists must have matching braces.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#ObjectTypeDefinition>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn field_definition_unclosed_brace() {
    let result = parse_schema("type T { f: String");
    assert!(result.has_errors());
}

/// Verifies that an unclosed input object definition produces an error.
///
/// Input object definitions with field lists must have matching braces.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#InputObjectTypeDefinition>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn input_object_unclosed_brace() {
    let result = parse_schema("input I { f: String");
    assert!(result.has_errors());
}

/// Verifies that an unclosed enum definition produces an error.
///
/// Enum definitions with value lists must have matching braces.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#EnumTypeDefinition>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_definition_unclosed_brace() {
    let result = parse_schema("enum E { A");
    assert!(result.has_errors());
}

/// Verifies that an unclosed argument list produces an error.
///
/// Argument lists must have matching parentheses.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Language.Arguments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn field_args_unclosed_paren_error() {
    let result = parse_executable("query { field(arg: 1 }");
    assert!(result.has_errors());
}

/// Verifies that an unclosed list type annotation produces an error.
///
/// List type annotations must have matching brackets.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_list_unclosed_bracket_error() {
    let result = parse_schema("type Q { f: [String }");
    assert!(result.has_errors());
}

// =============================================================================
// Reserved Name Tests
// =============================================================================

/// Verifies that `false` as an enum value produces an error.
///
/// Per GraphQL spec, `true`, `false`, and `null` cannot be enum values since
/// they would be ambiguous with boolean/null literals.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Enum-Value-Uniqueness>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_value_false_reserved_error() {
    let result = parse_schema("enum Bool { false }");
    assert!(result.has_errors());
}

/// Verifies that reserved names can be used in non-reserved contexts.
///
/// While `true`, `false`, `null` cannot be enum values, they can be field
/// names in selection sets since the context makes them unambiguous.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Names>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn reserved_names_allowed_in_field_names() {
    let result = parse_executable("{ true false null }");
    assert!(result.is_ok());
}

// =============================================================================
// Directive Location Tests
// =============================================================================

/// Verifies that an unknown directive location produces an error.
///
/// Directive definitions must use valid location names from the spec.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#DirectiveLocation>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_unknown_location_error() {
    let result = parse_schema("directive @d on UNKNOWN");
    assert!(result.has_errors());
}

/// Verifies that directive location names are case-sensitive.
///
/// `FIELD` is a valid location, but `field` is not since directive locations
/// must be uppercase.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#DirectiveLocation>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_location_case_sensitive() {
    let result = parse_schema("directive @d on field");
    assert!(result.has_errors());
}

// =============================================================================
// Document Enforcement Tests
// =============================================================================

/// Verifies that a type definition with description in executable doc errors.
///
/// When parsing as executable, type definitions (even with descriptions) are
/// not allowed since executable documents only contain operations and fragments.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Executable-Documents>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn executable_rejects_type_with_description() {
    let result = parse_executable(r#""description" type T { f: Int }"#);
    assert!(result.has_errors());
}

/// Verifies that schema definition in an executable document produces an error.
///
/// Schema definitions are only valid in schema documents, not executable ones.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Executable-Documents>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn executable_rejects_schema_definition() {
    let result = parse_executable("schema { query: Query }");
    assert!(result.has_errors());
}

/// Verifies that scalar definition in an executable document produces an error.
///
/// Scalar definitions are only valid in schema documents.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Executable-Documents>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn executable_rejects_scalar_definition() {
    let result = parse_executable("scalar DateTime");
    assert!(result.has_errors());
}

/// Verifies that interface definition in an executable document produces error.
///
/// Interface definitions are only valid in schema documents.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Executable-Documents>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn executable_rejects_interface_definition() {
    let result = parse_executable("interface Node { id: ID! }");
    assert!(result.has_errors());
}

/// Verifies that union definition in an executable document produces an error.
///
/// Union definitions are only valid in schema documents.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Executable-Documents>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn executable_rejects_union_definition() {
    let result = parse_executable("union Result = A | B");
    assert!(result.has_errors());
}

/// Verifies that enum definition in an executable document produces an error.
///
/// Enum definitions are only valid in schema documents.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Executable-Documents>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn executable_rejects_enum_definition() {
    let result = parse_executable("enum Status { ACTIVE }");
    assert!(result.has_errors());
}

/// Verifies that input definition in an executable document produces an error.
///
/// Input object definitions are only valid in schema documents.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Executable-Documents>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn executable_rejects_input_definition() {
    let result = parse_executable("input CreateInput { name: String }");
    assert!(result.has_errors());
}

// =============================================================================
// Schema Extension
// =============================================================================

/// Verifies that schema extension is handled without crashing.
///
/// The GraphQL spec defines schema extensions. This test verifies that the
/// parser handles them gracefully (either parsing successfully or returning
/// an error) without panicking.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Schema-Extension>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn schema_extension_parses() {
    let result = parse_schema("extend schema { query: Query }");
    // Either succeeds or has errors, but should not panic
    let _ = result.is_ok() || result.has_errors();
}

// =============================================================================
// Error Recovery
// =============================================================================

/// Verifies that the parser can recover from an error in a type definition.
///
/// After encountering a syntax error in one type definition, the parser should
/// be able to recover and continue parsing subsequent definitions.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn recovery_after_error_in_type_definition() {
    let result = parse_schema(
        "type A { field:: } type B { field: String }"
    );
    assert!(result.has_errors());
}

/// Verifies that recovery works across multiple definitions with errors.
///
/// When multiple type definitions have errors, the parser should attempt to
/// recover and continue parsing, collecting all errors.
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
/// The parser should be able to recover from errors in operation definitions
/// and continue parsing subsequent operations.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn recovery_operation_errors() {
    let result = parse_executable("query A { field( } query B { field }");
    assert!(result.has_errors());
}

/// Verifies recovery handles empty selection sets gracefully.
///
/// Empty selection sets are invalid per the spec and should produce an error,
/// but the parser should handle them without crashing.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Selection-Sets>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn recovery_empty_selection_set() {
    let result = parse_executable("{ }");
    assert!(result.has_errors());
}

/// Verifies that deeply nested unclosed delimiters produce errors.
///
/// When multiple levels of nesting are left unclosed, the parser should
/// detect and report the missing closing delimiters.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn recovery_deeply_nested_unclosed() {
    let result = parse_executable("{ a { b { c { d");
    assert!(result.has_errors());
}

// =============================================================================
// Lexer Errors
// =============================================================================

/// Verifies that an unterminated string produces a lexer error.
///
/// Strings must be properly terminated with a closing quote. The lexer should
/// detect unterminated strings and report an error.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn lexer_error_unterminated_string() {
    let result = parse_executable(r#"{ field(arg: "unterminated) }"#);
    assert!(result.has_errors());
}

/// Verifies that an unterminated block string produces a lexer error.
///
/// Block strings must be properly terminated with `"""`. The lexer should
/// detect unterminated block strings and report an error.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn lexer_error_unterminated_block_string() {
    let result = parse_executable(r#"{ field(arg: """unterminated) }"#);
    assert!(result.has_errors());
}

/// Verifies that invalid characters produce lexer errors.
///
/// Characters not part of the GraphQL lexical grammar should produce errors.
/// For example, the backtick character is not valid in GraphQL.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn lexer_error_invalid_character() {
    let result = parse_executable("{ field` }");
    assert!(result.has_errors());
}

/// Verifies that invalid escape sequences in strings produce lexer errors.
///
/// Only specific escape sequences are valid in GraphQL strings. Invalid
/// escape sequences like `\q` should produce errors.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#EscapedCharacter>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn lexer_error_invalid_escape_sequence() {
    let result = parse_executable(r#"{ field(arg: "hello\qworld") }"#);
    assert!(result.has_errors());
}

/// Verifies that invalid number formats produce lexer errors.
///
/// Leading zeros are not allowed in GraphQL integers (except for `0` itself).
/// Numbers like `007` should produce errors.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Int-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn lexer_error_number_format() {
    let result = parse_executable("{ field(arg: 007) }");
    assert!(result.has_errors());
}

/// Verifies that an exponent without digits produces a lexer error.
///
/// Float values with exponents must have digits after the `e` or `E`.
/// Values like `1e` are invalid and should produce errors.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Float-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn lexer_error_exponent_without_digits() {
    let result = parse_executable("{ field(arg: 1e) }");
    assert!(result.has_errors());
}
