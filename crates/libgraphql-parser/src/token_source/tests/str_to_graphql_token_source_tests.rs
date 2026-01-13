//! Tests for `StrGraphQLTokenSource`.
//!
//! Written by Claude Code, reviewed by a human.

use crate::token::GraphQLTokenKind;
use crate::token_source::StrGraphQLTokenSource;

/// Helper to collect all token kinds from a source string.
fn token_kinds(source: &str) -> Vec<GraphQLTokenKind<'_>> {
    StrGraphQLTokenSource::new(source)
        .map(|t| t.kind)
        .collect()
}

// =============================================================================
// Basic punctuator tests
// =============================================================================

/// Verifies that basic punctuators are lexed correctly.
///
/// Per GraphQL spec, punctuators are single characters with specific meanings:
/// <https://spec.graphql.org/September2025/#sec-Punctuators>
#[test]
fn test_punctuators() {
    let kinds = token_kinds("{ } ( ) [ ] : = @ ! $ & |");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::CurlyBraceOpen,
            GraphQLTokenKind::CurlyBraceClose,
            GraphQLTokenKind::ParenOpen,
            GraphQLTokenKind::ParenClose,
            GraphQLTokenKind::SquareBracketOpen,
            GraphQLTokenKind::SquareBracketClose,
            GraphQLTokenKind::Colon,
            GraphQLTokenKind::Equals,
            GraphQLTokenKind::At,
            GraphQLTokenKind::Bang,
            GraphQLTokenKind::Dollar,
            GraphQLTokenKind::Ampersand,
            GraphQLTokenKind::Pipe,
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that the ellipsis (`...`) is lexed as a single token.
///
/// Per GraphQL spec, `...` is the spread operator:
/// <https://spec.graphql.org/September2025/#sec-Punctuators>
#[test]
fn test_ellipsis() {
    let kinds = token_kinds("...");
    assert_eq!(
        kinds,
        vec![GraphQLTokenKind::Ellipsis, GraphQLTokenKind::Eof]
    );
}

// =============================================================================
// Name and keyword tests
// =============================================================================

/// Verifies that names are lexed correctly.
///
/// Per GraphQL spec, names match `/[_A-Za-z][_0-9A-Za-z]*/`:
/// <https://spec.graphql.org/September2025/#Name>
#[test]
fn test_names() {
    let kinds = token_kinds("hello _private type2 __typename");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::name_borrowed("hello"),
            GraphQLTokenKind::name_borrowed("_private"),
            GraphQLTokenKind::name_borrowed("type2"),
            GraphQLTokenKind::name_borrowed("__typename"),
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that `true`, `false`, and `null` are lexed as distinct tokens.
///
/// Per GraphQL spec, these are reserved words with special meaning:
/// <https://spec.graphql.org/September2025/#sec-Boolean-Value>
/// <https://spec.graphql.org/September2025/#sec-Null-Value>
#[test]
fn test_keywords() {
    let kinds = token_kinds("true false null");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::True,
            GraphQLTokenKind::False,
            GraphQLTokenKind::Null,
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that keywords are case-sensitive.
///
/// `True`, `FALSE`, `NULL` should be lexed as names, not keywords.
#[test]
fn test_keywords_case_sensitive() {
    let kinds = token_kinds("True FALSE Null");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::name_borrowed("True"),
            GraphQLTokenKind::name_borrowed("FALSE"),
            GraphQLTokenKind::name_borrowed("Null"),
            GraphQLTokenKind::Eof,
        ]
    );
}

// =============================================================================
// Number tests
// =============================================================================

/// Verifies that integer literals are lexed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Int-Value>
#[test]
fn test_int_values() {
    let kinds = token_kinds("0 123 -456");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::int_value_borrowed("0"),
            GraphQLTokenKind::int_value_borrowed("123"),
            GraphQLTokenKind::int_value_borrowed("-456"),
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that float literals are lexed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Float-Value>
#[test]
fn test_float_values() {
    let kinds = token_kinds("1.5 -3.14 1e10 1.23e-4");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::float_value_borrowed("1.5"),
            GraphQLTokenKind::float_value_borrowed("-3.14"),
            GraphQLTokenKind::float_value_borrowed("1e10"),
            GraphQLTokenKind::float_value_borrowed("1.23e-4"),
            GraphQLTokenKind::Eof,
        ]
    );
}

// =============================================================================
// String tests
// =============================================================================

/// Verifies that single-line strings are lexed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
#[test]
fn test_single_line_strings() {
    let kinds = token_kinds(r#""hello" "world""#);
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::string_value_borrowed("\"hello\""),
            GraphQLTokenKind::string_value_borrowed("\"world\""),
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that block strings are lexed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
#[test]
fn test_block_strings() {
    let kinds = token_kinds(r#""""block string""""#);
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::string_value_borrowed("\"\"\"block string\"\"\""),
            GraphQLTokenKind::Eof,
        ]
    );
}

// =============================================================================
// Comment tests
// =============================================================================

/// Verifies that comments are captured as trivia.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Comments>
#[test]
fn test_comments_as_trivia() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("# comment\nfield").collect();
    assert_eq!(tokens.len(), 2); // Name + Eof

    // The comment should be attached as trivia to the Name token
    assert_eq!(tokens[0].preceding_trivia.len(), 1);
    assert!(matches!(
        &tokens[0].preceding_trivia[0],
        crate::token::GraphQLTriviaToken::Comment { value, .. } if value == " comment"
    ));
}

// =============================================================================
// Whitespace tests
// =============================================================================

/// Verifies that BOM is ignored anywhere in the document.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Unicode>
#[test]
fn test_bom_ignored() {
    // BOM at start
    let kinds = token_kinds("\u{FEFF}name");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::name_borrowed("name"),
            GraphQLTokenKind::Eof,
        ]
    );

    // BOM in middle (between tokens)
    let kinds = token_kinds("a\u{FEFF}b");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::name_borrowed("a"),
            GraphQLTokenKind::name_borrowed("b"),
            GraphQLTokenKind::Eof,
        ]
    );
}

// =============================================================================
// Error recovery tests
// =============================================================================

/// Verifies that invalid characters produce error tokens but lexing continues.
#[test]
fn test_invalid_character_recovery() {
    let kinds = token_kinds("{ ^ }");
    assert_eq!(kinds.len(), 4); // CurlyBraceOpen, Error, CurlyBraceClose, Eof
    assert!(matches!(kinds[0], GraphQLTokenKind::CurlyBraceOpen));
    assert!(matches!(kinds[1], GraphQLTokenKind::Error { .. }));
    assert!(matches!(kinds[2], GraphQLTokenKind::CurlyBraceClose));
    assert!(matches!(kinds[3], GraphQLTokenKind::Eof));
}
