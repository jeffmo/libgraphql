//! Tests for `GraphQLTokenKind` public API.
//!
//! These tests verify the token classification methods and value parsing
//! functions work correctly per the GraphQL specification.
//!
//! Written by Claude Code, reviewed by a human.

use crate::token::GraphQLTokenKind;
use crate::GraphQLStringParsingError;

// =============================================================================
// Part 1.1: Punctuator Classification Tests
// =============================================================================

/// Verifies that `is_punctuator()` returns true for all 14 GraphQL punctuators.
///
/// Per GraphQL spec, the punctuators are: `!`, `$`, `&`, `(`, `)`, `...`, `:`,
/// `=`, `@`, `[`, `]`, `{`, `|`, `}`.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#Punctuator>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn is_punctuator_returns_true_for_punctuators() {
    let punctuators: Vec<GraphQLTokenKind<'static>> = vec![
        GraphQLTokenKind::Ampersand,
        GraphQLTokenKind::At,
        GraphQLTokenKind::Bang,
        GraphQLTokenKind::Colon,
        GraphQLTokenKind::CurlyBraceClose,
        GraphQLTokenKind::CurlyBraceOpen,
        GraphQLTokenKind::Dollar,
        GraphQLTokenKind::Ellipsis,
        GraphQLTokenKind::Equals,
        GraphQLTokenKind::ParenClose,
        GraphQLTokenKind::ParenOpen,
        GraphQLTokenKind::Pipe,
        GraphQLTokenKind::SquareBracketClose,
        GraphQLTokenKind::SquareBracketOpen,
    ];

    for punctuator in punctuators {
        assert!(
            punctuator.is_punctuator(),
            "{punctuator:?} should be identified as a punctuator"
        );
    }
}

/// Verifies that `is_punctuator()` returns false for non-punctuator tokens.
///
/// Names, values, keywords (true/false/null), EOF, and Error tokens should
/// not be classified as punctuators.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn is_punctuator_returns_false_for_non_punctuators() {
    let non_punctuators: Vec<GraphQLTokenKind<'static>> = vec![
        GraphQLTokenKind::name_owned("foo".to_string()),
        GraphQLTokenKind::int_value_owned("123".to_string()),
        GraphQLTokenKind::float_value_owned("1.5".to_string()),
        GraphQLTokenKind::string_value_owned("\"hello\"".to_string()),
        GraphQLTokenKind::True,
        GraphQLTokenKind::False,
        GraphQLTokenKind::Null,
        GraphQLTokenKind::Eof,
        GraphQLTokenKind::error("test error", Default::default()),
    ];

    for token in non_punctuators {
        assert!(
            !token.is_punctuator(),
            "{token:?} should NOT be identified as a punctuator"
        );
    }
}

/// Verifies that `as_punctuator_str()` returns the correct string for each
/// punctuator.
///
/// This is essential for error messages and code generation that need to
/// display the actual punctuation character.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#Punctuator>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn as_punctuator_str_returns_correct_strings() {
    let expected: Vec<(GraphQLTokenKind<'static>, &'static str)> = vec![
        (GraphQLTokenKind::Ampersand, "&"),
        (GraphQLTokenKind::At, "@"),
        (GraphQLTokenKind::Bang, "!"),
        (GraphQLTokenKind::Colon, ":"),
        (GraphQLTokenKind::CurlyBraceClose, "}"),
        (GraphQLTokenKind::CurlyBraceOpen, "{"),
        (GraphQLTokenKind::Dollar, "$"),
        (GraphQLTokenKind::Ellipsis, "..."),
        (GraphQLTokenKind::Equals, "="),
        (GraphQLTokenKind::ParenClose, ")"),
        (GraphQLTokenKind::ParenOpen, "("),
        (GraphQLTokenKind::Pipe, "|"),
        (GraphQLTokenKind::SquareBracketClose, "]"),
        (GraphQLTokenKind::SquareBracketOpen, "["),
    ];

    for (token, expected_str) in expected {
        assert_eq!(
            token.as_punctuator_str(),
            Some(expected_str),
            "{token:?} should return {expected_str:?}"
        );
    }
}

/// Verifies that `as_punctuator_str()` returns None for non-punctuators.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn as_punctuator_str_returns_none_for_non_punctuators() {
    let non_punctuators: Vec<GraphQLTokenKind<'static>> = vec![
        GraphQLTokenKind::name_owned("foo".to_string()),
        GraphQLTokenKind::int_value_owned("123".to_string()),
        GraphQLTokenKind::True,
        GraphQLTokenKind::Eof,
    ];

    for token in non_punctuators {
        assert_eq!(
            token.as_punctuator_str(),
            None,
            "{token:?} should return None for as_punctuator_str()"
        );
    }
}

// =============================================================================
// Part 1.1: Value Classification Tests
// =============================================================================

/// Verifies that `is_value()` returns true for value literal tokens.
///
/// Value literals are: IntValue, FloatValue, StringValue, true, false, null.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn is_value_returns_true_for_value_tokens() {
    let value_tokens: Vec<GraphQLTokenKind<'static>> = vec![
        GraphQLTokenKind::int_value_owned("42".to_string()),
        GraphQLTokenKind::float_value_owned("3.14".to_string()),
        GraphQLTokenKind::string_value_owned("\"hello\"".to_string()),
        GraphQLTokenKind::True,
        GraphQLTokenKind::False,
        GraphQLTokenKind::Null,
    ];

    for token in value_tokens {
        assert!(
            token.is_value(),
            "{token:?} should be identified as a value"
        );
    }
}

/// Verifies that `is_value()` returns false for non-value tokens.
///
/// Names, punctuators, EOF, and Error are not value literals.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn is_value_returns_false_for_non_value_tokens() {
    let non_value_tokens: Vec<GraphQLTokenKind<'static>> = vec![
        GraphQLTokenKind::name_owned("foo".to_string()),
        GraphQLTokenKind::Ampersand,
        GraphQLTokenKind::CurlyBraceOpen,
        GraphQLTokenKind::Eof,
        GraphQLTokenKind::error("test", Default::default()),
    ];

    for token in non_value_tokens {
        assert!(
            !token.is_value(),
            "{token:?} should NOT be identified as a value"
        );
    }
}

// =============================================================================
// Part 1.1: Error Detection Tests
// =============================================================================

/// Verifies that `is_error()` returns true only for Error token kind.
///
/// This is used during parsing to detect lexer errors that were collected
/// for error recovery.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn is_error_returns_true_only_for_error_kind() {
    let error_token = GraphQLTokenKind::error("test error", Default::default());
    assert!(error_token.is_error());

    // Non-error tokens
    let non_error_tokens: Vec<GraphQLTokenKind<'static>> = vec![
        GraphQLTokenKind::name_owned("foo".to_string()),
        GraphQLTokenKind::int_value_owned("123".to_string()),
        GraphQLTokenKind::True,
        GraphQLTokenKind::Eof,
        GraphQLTokenKind::Bang,
    ];

    for token in non_error_tokens {
        assert!(
            !token.is_error(),
            "{token:?} should NOT be identified as an error"
        );
    }
}

// =============================================================================
// Part 1.1: Integer Parsing Tests
// =============================================================================

/// Verifies that `parse_int_value()` correctly parses valid integers.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Int-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_int_value_valid_integers() {
    let test_cases = [
        ("0", 0i64),
        ("1", 1),
        ("42", 42),
        ("-1", -1),
        ("-42", -42),
        ("123456789", 123456789),
        ("-123456789", -123456789),
        ("9223372036854775807", i64::MAX),   // Max i64
        ("-9223372036854775808", i64::MIN),  // Min i64
    ];

    for (raw, expected) in test_cases {
        let token = GraphQLTokenKind::int_value_owned(raw.to_string());
        let result = token.parse_int_value();
        assert!(
            result.is_some(),
            "parse_int_value() should return Some for IntValue token"
        );
        assert_eq!(
            result.unwrap().unwrap(),
            expected,
            "Parsing {raw:?} should yield {expected}"
        );
    }
}

/// Verifies that `parse_int_value()` returns None for non-IntValue tokens.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_int_value_returns_none_for_non_int() {
    let non_int_tokens: Vec<GraphQLTokenKind<'static>> = vec![
        GraphQLTokenKind::float_value_owned("1.5".to_string()),
        GraphQLTokenKind::string_value_owned("\"123\"".to_string()),
        GraphQLTokenKind::name_owned("foo".to_string()),
        GraphQLTokenKind::True,
        GraphQLTokenKind::Eof,
    ];

    for token in non_int_tokens {
        assert!(
            token.parse_int_value().is_none(),
            "parse_int_value() should return None for {token:?}"
        );
    }
}

/// Verifies that integer overflow produces a ParseIntError.
///
/// GraphQL integers are stored in the raw token text and parsed to i64.
/// Values exceeding i64 range should produce parse errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_int_value_overflow_returns_error() {
    // Value larger than i64::MAX
    let token = GraphQLTokenKind::int_value_owned(
        "99999999999999999999999999".to_string()
    );
    let result = token.parse_int_value();
    assert!(result.is_some());
    assert!(
        result.unwrap().is_err(),
        "Overflow value should produce ParseIntError"
    );
}

// =============================================================================
// Part 1.1: Float Parsing Tests
// =============================================================================

/// Verifies that `parse_float_value()` correctly parses valid floats.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Float-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_float_value_valid_floats() {
    let test_cases = [
        ("0.0", 0.0f64),
        ("1.0", 1.0),
        ("3.25", 3.25),
        ("-3.25", -3.25),
        ("1e10", 1e10),
        ("1E10", 1e10),
        ("1.5e10", 1.5e10),
        ("1.5E-10", 1.5e-10),
        ("-1.5e+10", -1.5e10),
    ];

    for (raw, expected) in test_cases {
        let token = GraphQLTokenKind::float_value_owned(raw.to_string());
        let result = token.parse_float_value();
        assert!(
            result.is_some(),
            "parse_float_value() should return Some for FloatValue token"
        );
        let parsed = result.unwrap().unwrap();
        assert!(
            (parsed - expected).abs() < 1e-10,
            "Parsing {raw:?} should yield {expected}, got {parsed}"
        );
    }
}

/// Verifies that `parse_float_value()` returns None for non-FloatValue tokens.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_float_value_returns_none_for_non_float() {
    let non_float_tokens: Vec<GraphQLTokenKind<'static>> = vec![
        GraphQLTokenKind::int_value_owned("123".to_string()),
        GraphQLTokenKind::string_value_owned("\"1.5\"".to_string()),
        GraphQLTokenKind::name_owned("foo".to_string()),
        GraphQLTokenKind::True,
    ];

    for token in non_float_tokens {
        assert!(
            token.parse_float_value().is_none(),
            "parse_float_value() should return None for {token:?}"
        );
    }
}

// =============================================================================
// Part 1.1: Basic String Parsing Tests
// =============================================================================

/// Verifies that `parse_string_value()` correctly parses basic strings.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#StringValue>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_string_value_basic() {
    let test_cases = [
        (r#""""#, ""),                    // Empty string
        (r#""hello""#, "hello"),          // Simple string
        (r#""hello world""#, "hello world"), // With space
        (r#""123""#, "123"),              // Digits
    ];

    for (raw, expected) in test_cases {
        let token = GraphQLTokenKind::string_value_owned(raw.to_string());
        let result = token.parse_string_value();
        assert!(
            result.is_some(),
            "parse_string_value() should return Some for StringValue token"
        );
        assert_eq!(
            result.unwrap().unwrap(),
            expected,
            "Parsing {raw:?} should yield {expected:?}"
        );
    }
}

/// Verifies that `parse_string_value()` returns None for non-StringValue tokens.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_string_value_returns_none_for_non_string() {
    let non_string_tokens: Vec<GraphQLTokenKind<'static>> = vec![
        GraphQLTokenKind::int_value_owned("123".to_string()),
        GraphQLTokenKind::float_value_owned("1.5".to_string()),
        GraphQLTokenKind::name_owned("foo".to_string()),
        GraphQLTokenKind::True,
    ];

    for token in non_string_tokens {
        assert!(
            token.parse_string_value().is_none(),
            "parse_string_value() should return None for {token:?}"
        );
    }
}

// =============================================================================
// Part 1.2: String Escape Sequence Tests
// =============================================================================

/// Verifies that standard escape sequences are correctly processed.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#EscapedCharacter>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_string_standard_escapes() {
    let test_cases = [
        (r#""hello\nworld""#, "hello\nworld"),   // Newline
        (r#""hello\rworld""#, "hello\rworld"),   // Carriage return
        (r#""hello\tworld""#, "hello\tworld"),   // Tab
        (r#""hello\\world""#, "hello\\world"),   // Backslash
        (r#""hello\"world""#, "hello\"world"),   // Quote
        (r#""hello\/world""#, "hello/world"),    // Forward slash
        (r#""hello\bworld""#, "hello\u{0008}world"), // Backspace
        (r#""hello\fworld""#, "hello\u{000C}world"), // Form feed
    ];

    for (raw, expected) in test_cases {
        let token = GraphQLTokenKind::string_value_owned(raw.to_string());
        let result = token.parse_string_value().unwrap();
        assert_eq!(
            result.unwrap(),
            expected,
            "Escape sequence in {raw:?} should produce {expected:?}"
        );
    }
}

/// Verifies that invalid escape sequences produce an error.
///
/// Only specific escape characters are valid per the spec: n, r, t, \, ", /,
/// b, f, u.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#EscapedCharacter>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_string_invalid_escape_sequence() {
    let invalid_escapes = [
        r#""hello\xworld""#,  // \x is not valid
        r#""hello\qworld""#,  // \q is not valid
        r#""hello\!world""#,  // \! is not valid
        r#""hello\aworld""#,  // \a is not valid (not in GraphQL spec)
    ];

    for raw in invalid_escapes {
        let token = GraphQLTokenKind::string_value_owned(raw.to_string());
        let result = token.parse_string_value().unwrap();
        assert!(
            matches!(result, Err(GraphQLStringParsingError::InvalidEscapeSequence(_))),
            "Invalid escape in {raw:?} should produce InvalidEscapeSequence error"
        );
    }
}

/// Verifies that a trailing backslash (unterminated escape) produces an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_string_trailing_backslash() {
    // String ending with backslash: the backslash expects another character
    let token = GraphQLTokenKind::string_value_owned(r#""hello\""#.to_string());
    let result = token.parse_string_value().unwrap();
    assert!(
        matches!(result, Err(GraphQLStringParsingError::InvalidEscapeSequence(_))),
        "Trailing backslash should produce InvalidEscapeSequence error"
    );
}

// =============================================================================
// Part 1.2: Unicode Escape Tests
// =============================================================================

/// Verifies that fixed 4-digit Unicode escapes (\uXXXX) are processed correctly.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#EscapedUnicode>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_unicode_escape_fixed_4_digit() {
    let test_cases = [
        (r#""\u0041""#, "A"),           // U+0041 = 'A'
        (r#""\u0048\u0069""#, "Hi"),    // U+0048 = 'H', U+0069 = 'i'
        (r#""\u00E9""#, "\u{00E9}"),    // U+00E9 = 'é'
        (r#""\u4E2D""#, "\u{4E2D}"),    // U+4E2D = '中'
    ];

    for (raw, expected) in test_cases {
        let token = GraphQLTokenKind::string_value_owned(raw.to_string());
        let result = token.parse_string_value().unwrap();
        assert_eq!(
            result.unwrap(),
            expected,
            "Unicode escape in {raw:?} should produce {expected:?}"
        );
    }
}

/// Verifies that variable-length Unicode escapes (\u{X...}) are processed.
///
/// The variable-length syntax allows 1-6 hex digits for any valid Unicode code
/// point.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#EscapedUnicode>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_unicode_escape_variable_length() {
    let test_cases = [
        (r#""\u{41}""#, "A"),              // Single digit
        (r#""\u{0041}""#, "A"),            // Same as fixed 4-digit
        (r#""\u{1F600}""#, "\u{1F600}"),   // Emoji (grinning face)
        (r#""\u{10FFFF}""#, "\u{10FFFF}"), // Max valid code point
    ];

    for (raw, expected) in test_cases {
        let token = GraphQLTokenKind::string_value_owned(raw.to_string());
        let result = token.parse_string_value().unwrap();
        assert_eq!(
            result.unwrap(),
            expected,
            "Unicode escape in {raw:?} should produce {expected:?}"
        );
    }
}

/// Verifies that empty braces in Unicode escape (\u{}) produce an error.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#EscapedUnicode>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_unicode_escape_empty_braces() {
    let token = GraphQLTokenKind::string_value_owned(r#""\u{}""#.to_string());
    let result = token.parse_string_value().unwrap();
    assert!(
        matches!(result, Err(GraphQLStringParsingError::InvalidUnicodeEscape(_))),
        "Empty unicode braces should produce InvalidUnicodeEscape error"
    );
}

/// Verifies that invalid hex digits in Unicode escape produce an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_unicode_escape_invalid_hex() {
    let invalid_cases = [
        r#""\u{XYZ}""#,    // Non-hex characters
        r#""\u{GHIJ}""#,   // Invalid hex
        r#""\uGHIJ""#,     // Invalid hex in fixed format
    ];

    for raw in invalid_cases {
        let token = GraphQLTokenKind::string_value_owned(raw.to_string());
        let result = token.parse_string_value().unwrap();
        assert!(
            matches!(result, Err(GraphQLStringParsingError::InvalidUnicodeEscape(_))),
            "Invalid hex in {raw:?} should produce InvalidUnicodeEscape error"
        );
    }
}

/// Verifies that Unicode code points above U+10FFFF produce an error.
///
/// U+10FFFF is the maximum valid Unicode code point. Values above this are
/// invalid.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_unicode_escape_out_of_range() {
    let token = GraphQLTokenKind::string_value_owned(
        r#""\u{110000}""#.to_string()
    );
    let result = token.parse_string_value().unwrap();
    assert!(
        matches!(result, Err(GraphQLStringParsingError::InvalidUnicodeEscape(_))),
        "Code point above U+10FFFF should produce InvalidUnicodeEscape error"
    );
}

/// Verifies that incomplete fixed Unicode escapes produce an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_unicode_escape_incomplete_fixed() {
    let incomplete_cases = [
        r#""\u""#,      // No digits
        r#""\u0""#,     // 1 digit
        r#""\u00""#,    // 2 digits
        r#""\u004""#,   // 3 digits
    ];

    for raw in incomplete_cases {
        let token = GraphQLTokenKind::string_value_owned(raw.to_string());
        let result = token.parse_string_value().unwrap();
        assert!(
            matches!(result, Err(GraphQLStringParsingError::InvalidUnicodeEscape(_))),
            "Incomplete Unicode escape in {raw:?} should produce error"
        );
    }
}

/// Verifies that unclosed variable-length Unicode escapes produce an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_unicode_escape_unclosed_braces() {
    let token = GraphQLTokenKind::string_value_owned(
        r#""\u{1234""#.to_string()
    );
    let result = token.parse_string_value().unwrap();
    assert!(
        matches!(result, Err(GraphQLStringParsingError::InvalidUnicodeEscape(_))),
        "Unclosed unicode braces should produce InvalidUnicodeEscape error"
    );
}

// =============================================================================
// Part 1.2: Block String Tests
// =============================================================================

/// Verifies that basic block strings are parsed correctly.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_block_string_basic() {
    let test_cases = [
        (r#""""hello""""#, "hello"),
        (r#""""hello world""""#, "hello world"),
        (r#""""line1
line2""""#, "line1\nline2"),
    ];

    for (raw, expected) in test_cases {
        let token = GraphQLTokenKind::string_value_owned(raw.to_string());
        let result = token.parse_string_value().unwrap();
        assert_eq!(
            result.unwrap(),
            expected,
            "Block string {raw:?} should produce {expected:?}"
        );
    }
}

/// Verifies that block strings handle indentation stripping correctly.
///
/// Per the GraphQL spec, common indentation is stripped from all lines except
/// the first.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_block_string_indentation_stripping() {
    // Common indentation of 4 spaces should be stripped
    let raw = r#""""
    line1
    line2
""""#;
    let token = GraphQLTokenKind::string_value_owned(raw.to_string());
    let result = token.parse_string_value().unwrap().unwrap();
    assert_eq!(result, "line1\nline2");
}

/// Verifies that block strings handle lines shorter than common indent.
///
/// When a line is shorter than the common indentation, it should be preserved
/// as-is (not produce negative substring).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_block_string_indentation_edge_case() {
    // Second line has 4 spaces indent, third line has only 2 spaces
    // Common indent should be 2, so line2 loses 2 spaces, line3 loses 2 spaces
    let raw = r#""""
  short
    longer
""""#;
    let token = GraphQLTokenKind::string_value_owned(raw.to_string());
    let result = token.parse_string_value().unwrap().unwrap();
    assert_eq!(result, "short\n  longer");
}

/// Verifies that escaped triple quotes in block strings are handled.
///
/// The only escape sequence in block strings is \""" which produces """.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_block_string_escaped_triple_quotes() {
    let raw = r#""""contains \""" quotes""""#;
    let token = GraphQLTokenKind::string_value_owned(raw.to_string());
    let result = token.parse_string_value().unwrap().unwrap();
    assert_eq!(result, r#"contains """ quotes"#);
}

/// Verifies that leading and trailing blank lines are trimmed.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_block_string_trims_blank_lines() {
    let raw = r#""""

    content

""""#;
    let token = GraphQLTokenKind::string_value_owned(raw.to_string());
    let result = token.parse_string_value().unwrap().unwrap();
    assert_eq!(result, "content");
}

// =============================================================================
// Part 1.1: Constructor Tests
// =============================================================================

/// Verifies that borrowed constructors create the expected token kinds.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn constructor_borrowed_variants() {
    let name = GraphQLTokenKind::name_borrowed("foo");
    assert!(matches!(name, GraphQLTokenKind::Name(_)));

    let int_val = GraphQLTokenKind::int_value_borrowed("123");
    assert!(matches!(int_val, GraphQLTokenKind::IntValue(_)));

    let float_val = GraphQLTokenKind::float_value_borrowed("1.5");
    assert!(matches!(float_val, GraphQLTokenKind::FloatValue(_)));

    let string_val = GraphQLTokenKind::string_value_borrowed("\"hello\"");
    assert!(matches!(string_val, GraphQLTokenKind::StringValue(_)));
}

/// Verifies that owned constructors create the expected token kinds.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn constructor_owned_variants() {
    let name = GraphQLTokenKind::name_owned("foo".to_string());
    assert!(matches!(name, GraphQLTokenKind::Name(_)));

    let int_val = GraphQLTokenKind::int_value_owned("123".to_string());
    assert!(matches!(int_val, GraphQLTokenKind::IntValue(_)));

    let float_val = GraphQLTokenKind::float_value_owned("1.5".to_string());
    assert!(matches!(float_val, GraphQLTokenKind::FloatValue(_)));

    let string_val = GraphQLTokenKind::string_value_owned("\"hello\"".to_string());
    assert!(matches!(string_val, GraphQLTokenKind::StringValue(_)));
}

/// Verifies that error constructor creates Error token with correct fields.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn constructor_error() {
    let notes = Default::default();
    let error = GraphQLTokenKind::error("test message", notes);

    assert!(error.is_error());
    if let GraphQLTokenKind::Error { message, .. } = error {
        assert_eq!(message, "test message");
    } else {
        panic!("Expected Error variant");
    }
}
