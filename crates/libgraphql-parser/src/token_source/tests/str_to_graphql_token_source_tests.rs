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

// =============================================================================
// Position tracking tests
// =============================================================================

/// Verifies that token spans have correct start positions on a single line.
///
/// Positions should be 0-based for both line and column.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_position_single_line() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("abc def").collect();
    assert_eq!(tokens.len(), 3); // abc, def, Eof

    // First token starts at (0, 0)
    assert_eq!(tokens[0].span.start_inclusive.line(), 0);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
    assert_eq!(tokens[0].span.start_inclusive.col_utf16(), Some(0));
    assert_eq!(tokens[0].span.start_inclusive.byte_offset(), 0);

    // First token ends at (0, 3)
    assert_eq!(tokens[0].span.end_exclusive.line(), 0);
    assert_eq!(tokens[0].span.end_exclusive.col_utf8(), 3);
    assert_eq!(tokens[0].span.end_exclusive.col_utf16(), Some(3));
    assert_eq!(tokens[0].span.end_exclusive.byte_offset(), 3);

    // Second token starts at (0, 4) - after space
    assert_eq!(tokens[1].span.start_inclusive.line(), 0);
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 4);
    assert_eq!(tokens[1].span.start_inclusive.col_utf16(), Some(4));
    assert_eq!(tokens[1].span.start_inclusive.byte_offset(), 4);

    // Second token ends at (0, 7)
    assert_eq!(tokens[1].span.end_exclusive.line(), 0);
    assert_eq!(tokens[1].span.end_exclusive.col_utf8(), 7);
    assert_eq!(tokens[1].span.end_exclusive.col_utf16(), Some(7));
    assert_eq!(tokens[1].span.end_exclusive.byte_offset(), 7);
}

/// Verifies that token positions track correctly across multiple lines.
///
/// Line numbers increment on newlines; column resets to 0.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_position_multiple_lines() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("abc\ndef\nghi").collect();
    assert_eq!(tokens.len(), 4); // abc, def, ghi, Eof

    // First token: line 0
    assert_eq!(tokens[0].span.start_inclusive.line(), 0);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
    assert_eq!(tokens[0].span.start_inclusive.col_utf16(), Some(0));

    // Second token: line 1, column 0
    assert_eq!(tokens[1].span.start_inclusive.line(), 1);
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 0);
    assert_eq!(tokens[1].span.start_inclusive.col_utf16(), Some(0));

    // Third token: line 2, column 0
    assert_eq!(tokens[2].span.start_inclusive.line(), 2);
    assert_eq!(tokens[2].span.start_inclusive.col_utf8(), 0);
    assert_eq!(tokens[2].span.start_inclusive.col_utf16(), Some(0));
}

/// Verifies that CRLF (`\r\n`) is treated as a single newline.
///
/// Per GraphQL spec, line terminators include `\r\n`:
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Line-Terminators>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_position_crlf_newline() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("abc\r\ndef").collect();
    assert_eq!(tokens.len(), 3); // abc, def, Eof

    // First token: line 0
    assert_eq!(tokens[0].span.start_inclusive.line(), 0);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
    assert_eq!(tokens[0].span.start_inclusive.col_utf16(), Some(0));

    // Second token: line 1 (CRLF counts as one newline)
    assert_eq!(tokens[1].span.start_inclusive.line(), 1);
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 0);
    assert_eq!(tokens[1].span.start_inclusive.col_utf16(), Some(0));
}

/// Verifies that CR alone (`\r`) is treated as a newline.
///
/// Per GraphQL spec, `\r` is a valid line terminator:
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Line-Terminators>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_position_cr_newline() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("abc\rdef").collect();
    assert_eq!(tokens.len(), 3); // abc, def, Eof

    // First token: line 0
    assert_eq!(tokens[0].span.start_inclusive.line(), 0);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
    assert_eq!(tokens[0].span.start_inclusive.col_utf16(), Some(0));

    // Second token: line 1
    assert_eq!(tokens[1].span.start_inclusive.line(), 1);
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 0);
    assert_eq!(tokens[1].span.start_inclusive.col_utf16(), Some(0));
}

// =============================================================================
// UTF-16 column tracking tests
// =============================================================================

/// Verifies that UTF-16 column tracking works for ASCII characters.
///
/// For ASCII, UTF-8 and UTF-16 columns should be identical.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_utf16_column_ascii() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("abc def").collect();

    // For ASCII, UTF-8 and UTF-16 columns are the same
    assert_eq!(
        tokens[0].span.start_inclusive.col_utf16(),
        Some(tokens[0].span.start_inclusive.col_utf8())
    );
    assert_eq!(
        tokens[1].span.start_inclusive.col_utf16(),
        Some(tokens[1].span.start_inclusive.col_utf8())
    );
}

/// Verifies that UTF-16 columns count BMP characters correctly.
///
/// Characters in the Basic Multilingual Plane (U+0000 to U+FFFF) take
/// 1 UTF-16 code unit each.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_utf16_column_bmp_characters() {
    // "α" (U+03B1) is 2 UTF-8 bytes but 1 UTF-16 code unit
    let tokens: Vec<_> = StrGraphQLTokenSource::new("α x").collect();

    // After "α" (1 char, 2 bytes, 1 UTF-16 unit) and space
    // Second token starts at UTF-8 col 2, UTF-16 col 2
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 2);
    assert_eq!(tokens[1].span.start_inclusive.col_utf16(), Some(2));
}

/// Verifies that UTF-16 columns count supplementary characters correctly.
///
/// Characters outside the BMP (U+10000 and above) take 2 UTF-16 code units
/// (a surrogate pair), but still count as 1 UTF-8 character.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_utf16_column_supplementary_characters() {
    // "🎉" (U+1F389) is 4 UTF-8 bytes but 2 UTF-16 code units
    let tokens: Vec<_> = StrGraphQLTokenSource::new("\"🎉\" x").collect();

    // String token: "🎉" (3 chars: ", 🎉, ")
    // - UTF-8 bytes: 1 + 4 + 1 = 6
    // - UTF-8 chars: 3
    // - UTF-16 units: 1 + 2 + 1 = 4

    // After string (3 chars, 6 bytes, 4 UTF-16 units) and space
    // Second token starts at UTF-8 col 4, UTF-16 col 5
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 4);
    assert_eq!(tokens[1].span.start_inclusive.col_utf16(), Some(5));
}

// =============================================================================
// Dot pattern edge case tests
// =============================================================================

/// Verifies that adjacent dots `..` produce an error suggesting to add third
/// dot.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_dot_pattern_adjacent_two() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("..").collect();
    assert_eq!(tokens.len(), 2); // Error, Eof

    // Check for the specific error message pattern: "Unexpected `..`" (exactly
    // two dots in backticks)
    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("Unexpected `..`")
    ));
}

/// Verifies that spaced dots `. .` produce an error about spacing.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_dot_pattern_spaced_two() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new(". .").collect();
    assert_eq!(tokens.len(), 2); // Error, Eof

    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("Unexpected `. .`")
    ));
}

/// Verifies that `.. .` (first two adjacent, third spaced) produces an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_dot_pattern_two_adjacent_one_spaced() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new(".. .").collect();
    assert_eq!(tokens.len(), 2); // Error, Eof

    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("Unexpected `.. .`")
    ));
}

/// Verifies that `. ..` (first spaced, last two adjacent) produces an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_dot_pattern_one_spaced_two_adjacent() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new(". ..").collect();
    assert_eq!(tokens.len(), 2); // Error, Eof

    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("Unexpected `. ..`")
    ));
}

/// Verifies that `. . .` (all spaced) produces an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_dot_pattern_all_spaced() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new(". . .").collect();
    assert_eq!(tokens.len(), 2); // Error, Eof

    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("Unexpected `. . .`")
    ));
}

/// Verifies that a single dot produces a generic error.
///
/// Single dot errors don't assume it was meant to be ellipsis - could be
/// `Foo.Bar` style syntax from another language.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_dot_pattern_single() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new(".").collect();
    assert_eq!(tokens.len(), 2); // Error, Eof

    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("Unexpected `.`")
    ));
}

/// Verifies that dots on separate lines are treated as separate errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_dot_pattern_separate_lines() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new(".\n.").collect();
    assert_eq!(tokens.len(), 3); // Error, Error, Eof

    // Both should be single-dot errors
    assert!(matches!(&tokens[0].kind, GraphQLTokenKind::Error { .. }));
    assert!(matches!(&tokens[1].kind, GraphQLTokenKind::Error { .. }));
}

/// Verifies that `..\n.` (two adjacent dots, newline, one dot) produces two
/// separate errors.
///
/// Per GraphQL spec, the `...` spread operator must be a single contiguous
/// token with no whitespace or line breaks within it:
/// <https://spec.graphql.org/September2025/#sec-Punctuators>
///
/// The spec states that "no Ignored may appear within a Token", meaning dots
/// separated by newlines cannot form a valid ellipsis—even if there are
/// exactly three dots total.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_dot_pattern_two_dots_newline_one_dot() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("..\n.").collect();
    assert_eq!(tokens.len(), 3); // Error (for `..`), Error (for `.`), Eof

    // First error: adjacent `..` on line 1
    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("Unexpected `..`")
    ));

    // Second error: single `.` on line 2
    assert!(matches!(
        &tokens[1].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("Unexpected `.`")
    ));
}

/// Verifies that `.\n..` (one dot, newline, two adjacent dots) produces two
/// separate errors.
///
/// Per GraphQL spec, the `...` spread operator must be a single contiguous
/// token with no whitespace or line breaks within it:
/// <https://spec.graphql.org/September2025/#sec-Punctuators>
///
/// The spec states that "no Ignored may appear within a Token", meaning dots
/// separated by newlines cannot form a valid ellipsis—even if there are
/// exactly three dots total.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_dot_pattern_one_dot_newline_two_dots() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new(".\n..").collect();
    assert_eq!(tokens.len(), 3); // Error (for `.`), Error (for `..`), Eof

    // First error: single `.` on line 1
    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("Unexpected `.`")
    ));

    // Second error: adjacent `..` on line 2
    assert!(matches!(
        &tokens[1].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("Unexpected `..`")
    ));
}

// =============================================================================
// Number error edge cases
// =============================================================================

/// Verifies that leading zeros produce an error.
///
/// Per GraphQL spec, integers cannot have leading zeros:
/// <https://spec.graphql.org/September2025/#sec-Int-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_number_leading_zeros() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("007").collect();
    assert_eq!(tokens.len(), 2); // Error, Eof

    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("leading zeros")
    ));
}

/// Verifies that exponent without digits produces an error.
///
/// Per GraphQL spec, exponent must have at least one digit:
/// <https://spec.graphql.org/September2025/#sec-Float-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_number_exponent_no_digits() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("1e").collect();
    assert_eq!(tokens.len(), 2); // Error, Eof

    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("exponent")
    ));
}

/// Verifies that a lone minus sign produces an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_number_lone_minus() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("- ").collect();
    assert_eq!(tokens.len(), 2); // Error, Eof

    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("`-`")
    ));
}

// =============================================================================
// String error edge cases
// =============================================================================

/// Verifies that unterminated single-line strings produce an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_string_unterminated() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("\"hello").collect();
    assert_eq!(tokens.len(), 2); // Error, Eof

    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("Unterminated")
    ));
}

/// Verifies that unescaped newlines in single-line strings produce an error.
///
/// Per GraphQL spec, single-line strings cannot contain unescaped newlines:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_string_unescaped_newline() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("\"hello\nworld\"").collect();
    // First token is error (string with newline), then we continue with valid
    // tokens
    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("Unterminated")
    ));
}

/// Verifies that unterminated block strings produce an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_block_string_unterminated() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("\"\"\"hello").collect();
    assert_eq!(tokens.len(), 2); // Error, Eof

    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("Unterminated block string")
    ));
}

/// Verifies that escaped triple quotes in block strings don't end the string.
///
/// Per GraphQL spec, `\"""` inside a block string does not close it:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_block_string_escaped_triple_quote() {
    let kinds = token_kinds(r#""""hello \""" world""""#);
    assert_eq!(kinds.len(), 2); // StringValue, Eof

    // Verify the entire content is captured as a single string token
    assert_eq!(
        kinds[0],
        GraphQLTokenKind::string_value_borrowed(r#""""hello \""" world""""#)
    );
}

// =============================================================================
// Trivia tests
// =============================================================================

/// Verifies that commas are captured as trivia.
///
/// Per GraphQL spec, commas are ignored tokens (like whitespace):
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Insignificant-Commas>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_comma_as_trivia() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("a, b").collect();
    assert_eq!(tokens.len(), 3); // a, b, Eof

    // Comma should be attached as trivia to "b" token
    assert_eq!(tokens[1].preceding_trivia.len(), 1);
    assert!(matches!(
        &tokens[1].preceding_trivia[0],
        crate::token::GraphQLTriviaToken::Comma { .. }
    ));
}

/// Verifies that multiple comments are accumulated as trivia.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_multiple_comments_as_trivia() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("# first\n# second\nfield").collect();
    assert_eq!(tokens.len(), 2); // field, Eof

    // Both comments should be attached as trivia to "field" token
    assert_eq!(tokens[0].preceding_trivia.len(), 2);
    assert!(matches!(
        &tokens[0].preceding_trivia[0],
        crate::token::GraphQLTriviaToken::Comment { value, .. } if value == " first"
    ));
    assert!(matches!(
        &tokens[0].preceding_trivia[1],
        crate::token::GraphQLTriviaToken::Comment { value, .. } if value == " second"
    ));
}

/// Verifies that trailing trivia is attached to EOF.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_trailing_comment_on_eof() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("field # trailing").collect();
    assert_eq!(tokens.len(), 2); // field, Eof

    // Trailing comment should be attached to EOF token
    assert_eq!(tokens[1].preceding_trivia.len(), 1);
    assert!(matches!(
        &tokens[1].preceding_trivia[0],
        crate::token::GraphQLTriviaToken::Comment { value, .. } if value == " trailing"
    ));
}
