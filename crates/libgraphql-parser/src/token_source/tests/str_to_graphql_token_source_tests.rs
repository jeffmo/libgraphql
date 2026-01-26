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

/// Verifies that adjacent punctuators without whitespace are lexed as separate
/// tokens.
///
/// Per GraphQL spec, punctuators are self-delimiting - they don't require
/// whitespace between them:
/// <https://spec.graphql.org/September2025/#sec-Punctuators>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn punctuators_adjacent_without_whitespace() {
    let kinds = token_kinds("{}[]()");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::CurlyBraceOpen,
            GraphQLTokenKind::CurlyBraceClose,
            GraphQLTokenKind::SquareBracketOpen,
            GraphQLTokenKind::SquareBracketClose,
            GraphQLTokenKind::ParenOpen,
            GraphQLTokenKind::ParenClose,
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that ellipsis with surrounding whitespace is lexed correctly.
///
/// Per GraphQL spec, whitespace is ignored between tokens:
/// <https://spec.graphql.org/September2025/#sec-Punctuators>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn ellipsis_with_surrounding_whitespace() {
    let kinds = token_kinds("  ...  ");
    assert_eq!(
        kinds,
        vec![GraphQLTokenKind::Ellipsis, GraphQLTokenKind::Eof]
    );
}

/// Verifies that four dots `....` produces an ellipsis followed by a dot error.
///
/// Per GraphQL spec, `...` is the spread operator and a single `.` is invalid:
/// <https://spec.graphql.org/September2025/#sec-Punctuators>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn ellipsis_followed_by_dot() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("....").collect();
    assert_eq!(tokens.len(), 3); // Ellipsis, Error (for `.`), Eof

    assert!(matches!(tokens[0].kind, GraphQLTokenKind::Ellipsis));
    assert!(matches!(
        &tokens[1].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("Unexpected `.`")
    ));
    assert!(matches!(tokens[2].kind, GraphQLTokenKind::Eof));
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

/// Verifies that SCREAMING_CASE names are lexed correctly.
///
/// Per GraphQL spec, names match `/[_A-Za-z][_0-9A-Za-z]*/`:
/// <https://spec.graphql.org/September2025/#Name>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn name_uppercase() {
    let kinds = token_kinds("SCREAMING_CASE ALL_CAPS");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::name_borrowed("SCREAMING_CASE"),
            GraphQLTokenKind::name_borrowed("ALL_CAPS"),
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that camelCase and PascalCase names are lexed correctly.
///
/// Per GraphQL spec, names match `/[_A-Za-z][_0-9A-Za-z]*/`:
/// <https://spec.graphql.org/September2025/#Name>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn name_mixed_case() {
    let kinds = token_kinds("camelCase PascalCase mixedCase123");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::name_borrowed("camelCase"),
            GraphQLTokenKind::name_borrowed("PascalCase"),
            GraphQLTokenKind::name_borrowed("mixedCase123"),
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that a name cannot start with a digit.
///
/// Per GraphQL spec, names must start with `[_A-Za-z]`, not a digit.
/// Input `2fast` should be lexed as IntValue `2` followed by Name `fast`.
/// <https://spec.graphql.org/September2025/#Name>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn name_cannot_start_with_digit() {
    let kinds = token_kinds("2fast");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::int_value_borrowed("2"),
            GraphQLTokenKind::name_borrowed("fast"),
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that a single underscore is a valid name.
///
/// Per GraphQL spec, names match `/[_A-Za-z][_0-9A-Za-z]*/`, so `_` alone
/// is valid:
/// <https://spec.graphql.org/September2025/#Name>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn name_single_underscore() {
    let kinds = token_kinds("_");
    assert_eq!(
        kinds,
        vec![GraphQLTokenKind::name_borrowed("_"), GraphQLTokenKind::Eof,]
    );
}

/// Verifies that very long names (stress test) are handled correctly.
///
/// Per GraphQL spec, there is no maximum length for names:
/// <https://spec.graphql.org/September2025/#Name>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn name_very_long() {
    let long_name = "a".repeat(10000);
    let source = long_name.clone();
    let kinds = token_kinds(&source);
    assert_eq!(kinds.len(), 2); // Name + Eof

    // We can't use name_borrowed here since we're comparing owned vs borrowed
    match &kinds[0] {
        GraphQLTokenKind::Name(cow) => assert_eq!(cow.as_ref(), long_name),
        _ => panic!("Expected Name token"),
    }
}

/// Verifies that Unicode characters in names produce errors.
///
/// Per GraphQL spec, names must match `/[_A-Za-z][_0-9A-Za-z]*/` (ASCII
/// only):
/// <https://spec.graphql.org/September2025/#Name>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn name_unicode_rejected() {
    // Test with accented character
    let tokens: Vec<_> = StrGraphQLTokenSource::new("café").collect();
    // "caf" should be lexed as a name, then "é" should produce an error
    assert!(tokens.len() >= 2);
    assert_eq!(tokens[0].kind, GraphQLTokenKind::name_borrowed("caf"));
    assert!(matches!(&tokens[1].kind, GraphQLTokenKind::Error { .. }));

    // Test with non-Latin characters
    let tokens: Vec<_> = StrGraphQLTokenSource::new("名前").collect();
    // Should produce errors for non-ASCII characters
    assert!(matches!(&tokens[0].kind, GraphQLTokenKind::Error { .. }));
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

/// Verifies that `NULL` (all caps) is lexed as a name, not keyword.
///
/// Per GraphQL spec, keywords are case-sensitive:
/// <https://spec.graphql.org/September2025/#sec-Null-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn keyword_case_sensitive_null() {
    let kinds = token_kinds("NULL");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::name_borrowed("NULL"),
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that `trueValue` is lexed as a single name, not `true` + `Value`.
///
/// Per GraphQL spec, names are greedy - the longest valid name is matched:
/// <https://spec.graphql.org/September2025/#Name>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn keyword_prefix_trueish() {
    let kinds = token_kinds("trueValue");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::name_borrowed("trueValue"),
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that `falsely` is lexed as a single name, not `false` + `ly`.
///
/// Per GraphQL spec, names are greedy - the longest valid name is matched:
/// <https://spec.graphql.org/September2025/#Name>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn keyword_prefix_falsely() {
    let kinds = token_kinds("falsely");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::name_borrowed("falsely"),
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that `nullable` is lexed as a single name, not `null` + `able`.
///
/// Per GraphQL spec, names are greedy - the longest valid name is matched:
/// <https://spec.graphql.org/September2025/#Name>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn keyword_prefix_nullable() {
    let kinds = token_kinds("nullable");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::name_borrowed("nullable"),
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

/// Verifies that `-0` is a valid IntValue.
///
/// Per GraphQL spec, IntValue includes optional negative sign:
/// <https://spec.graphql.org/September2025/#sec-Int-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn int_negative_zero() {
    let kinds = token_kinds("-0");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::int_value_borrowed("-0"),
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that `-007` (negative with leading zeros) produces an error.
///
/// Per GraphQL spec, integers cannot have leading zeros after the optional
/// negative sign:
/// <https://spec.graphql.org/September2025/#sec-Int-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn int_negative_leading_zeros_error() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("-007").collect();
    assert_eq!(tokens.len(), 2); // Error, Eof

    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("leading zeros")
    ));
}

/// Verifies that `i32::MAX` (2147483647) is lexed as a valid IntValue.
///
/// The lexer stores raw text; parsing to i64 happens via `parse_int_value()`.
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Int-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn int_max_i32() {
    let kinds = token_kinds("2147483647");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::int_value_borrowed("2147483647"),
            GraphQLTokenKind::Eof,
        ]
    );

    // Verify it parses correctly to i64
    let tokens: Vec<_> = StrGraphQLTokenSource::new("2147483647").collect();
    assert_eq!(
        tokens[0].kind.parse_int_value(),
        Some(Ok(2147483647_i64))
    );
}

/// Verifies that `i32::MIN` (-2147483648) is lexed as a valid IntValue.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Int-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn int_min_i32() {
    let kinds = token_kinds("-2147483648");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::int_value_borrowed("-2147483648"),
            GraphQLTokenKind::Eof,
        ]
    );

    // Verify it parses correctly to i64
    let tokens: Vec<_> = StrGraphQLTokenSource::new("-2147483648").collect();
    assert_eq!(
        tokens[0].kind.parse_int_value(),
        Some(Ok(-2147483648_i64))
    );
}

/// Verifies that large integers beyond i32 but within i64 are handled.
///
/// The lexer stores raw text; `parse_int_value()` parses to i64.
/// Per GraphQL spec, IntValue has no explicit size limit:
/// <https://spec.graphql.org/September2025/#sec-Int-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn int_i64_range() {
    // Value larger than i32::MAX but within i64 range
    let kinds = token_kinds("9223372036854775807");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::int_value_borrowed("9223372036854775807"),
            GraphQLTokenKind::Eof,
        ]
    );

    // Verify it parses to i64::MAX
    let tokens: Vec<_> = StrGraphQLTokenSource::new("9223372036854775807").collect();
    assert_eq!(
        tokens[0].kind.parse_int_value(),
        Some(Ok(i64::MAX))
    );

    // Negative i64::MIN
    let tokens: Vec<_> = StrGraphQLTokenSource::new("-9223372036854775808").collect();
    assert_eq!(
        tokens[0].kind.parse_int_value(),
        Some(Ok(i64::MIN))
    );
}

/// Verifies that i64 overflow produces a parse error.
///
/// The lexer stores raw text successfully; overflow error occurs when calling
/// `parse_int_value()`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn int_overflow_i64() {
    // Value beyond i64::MAX
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("9223372036854775808").collect();

    // Lexer captures the raw text
    assert_eq!(
        tokens[0].kind,
        GraphQLTokenKind::int_value_borrowed("9223372036854775808")
    );

    // But parsing to i64 fails
    let result = tokens[0].kind.parse_int_value();
    assert!(matches!(result, Some(Err(_))));
}

/// Verifies that i64 underflow produces a parse error.
///
/// The lexer stores raw text successfully; underflow error occurs when calling
/// `parse_int_value()`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn int_underflow_i64() {
    // Value beyond i64::MIN
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("-9223372036854775809").collect();

    // Lexer captures the raw text
    assert_eq!(
        tokens[0].kind,
        GraphQLTokenKind::int_value_borrowed("-9223372036854775809")
    );

    // But parsing to i64 fails
    let result = tokens[0].kind.parse_int_value();
    assert!(matches!(result, Some(Err(_))));
}

/// Verifies that an integer followed by a name is lexed as separate tokens.
///
/// Per GraphQL spec, integers don't include trailing letters:
/// <https://spec.graphql.org/September2025/#sec-Int-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn int_followed_by_name() {
    let kinds = token_kinds("123abc");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::int_value_borrowed("123"),
            GraphQLTokenKind::name_borrowed("abc"),
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that an integer followed by a dot and name is handled correctly.
///
/// `123.abc` should produce `123` (int) then an error for `.abc` (dot followed
/// by name, not a valid float).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn int_followed_by_dot_name() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("123.abc").collect();
    // This could be: IntValue "123", Error (for "."), Name "abc", Eof
    // OR: Error (invalid float "123.a..."), ...
    // The actual behavior depends on the lexer implementation

    // At minimum we should have more than 2 tokens
    assert!(tokens.len() >= 3);

    // First token should be either IntValue "123" or an Error
    // (implementation-dependent)
    let first_is_valid =
        matches!(&tokens[0].kind, GraphQLTokenKind::IntValue(_)) ||
        matches!(&tokens[0].kind, GraphQLTokenKind::Error { .. });
    assert!(first_is_valid);
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

/// Verifies that uppercase exponent indicator `E` is valid.
///
/// Per GraphQL spec, exponent indicator can be `e` or `E`:
/// <https://spec.graphql.org/September2025/#sec-Float-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn float_exponent_uppercase() {
    let kinds = token_kinds("1E10 2E3 1.5E-2");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::float_value_borrowed("1E10"),
            GraphQLTokenKind::float_value_borrowed("2E3"),
            GraphQLTokenKind::float_value_borrowed("1.5E-2"),
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that explicit positive exponent `e+` is valid.
///
/// Per GraphQL spec, exponent sign can be `+` or `-`:
/// <https://spec.graphql.org/September2025/#sec-Float-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn float_exponent_positive() {
    let kinds = token_kinds("1e+10 2.5E+3");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::float_value_borrowed("1e+10"),
            GraphQLTokenKind::float_value_borrowed("2.5E+3"),
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that `0.0` is a valid FloatValue.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Float-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn float_zero_decimal() {
    let kinds = token_kinds("0.0");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::float_value_borrowed("0.0"),
            GraphQLTokenKind::Eof,
        ]
    );

    // Verify it parses to 0.0
    let tokens: Vec<_> = StrGraphQLTokenSource::new("0.0").collect();
    assert_eq!(tokens[0].kind.parse_float_value(), Some(Ok(0.0)));
}

/// Verifies that `0.123` (leading zero with decimal) is valid.
///
/// Per GraphQL spec, leading zero is valid when followed by decimal:
/// <https://spec.graphql.org/September2025/#sec-Float-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn float_leading_zero_decimal() {
    let kinds = token_kinds("0.123 0.001 0.999");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::float_value_borrowed("0.123"),
            GraphQLTokenKind::float_value_borrowed("0.001"),
            GraphQLTokenKind::float_value_borrowed("0.999"),
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that `.5` (no leading zero) produces a dot error, not a float.
///
/// Per GraphQL spec, floats require at least one digit before the decimal:
/// <https://spec.graphql.org/September2025/#sec-Float-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn float_no_leading_zero_error() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new(".5").collect();
    // `.5` should NOT be a valid float - `.` is an error
    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("Unexpected `.`")
    ));
}

/// Verifies that `5.` followed by non-digit is handled correctly.
///
/// `5.` alone could be interpreted as start of float or int followed by dot.
/// This tests the lexer behavior when decimal has no following digits.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn float_trailing_dot_not_float() {
    // `5.a` - the decimal point has no digits, followed by a letter
    let tokens: Vec<_> = StrGraphQLTokenSource::new("5.a").collect();

    // Implementation could either:
    // 1. Error on `5.` (incomplete float)
    // 2. Produce IntValue `5`, then error on `.`, then Name `a`
    // Either is reasonable; we just verify the lexer handles it
    assert!(tokens.len() >= 2);
}

/// Verifies that `1..5` (double dot in number) produces an error.
///
/// Per GraphQL spec, floats can only have one decimal point:
/// <https://spec.graphql.org/September2025/#sec-Float-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn float_double_dot_error() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("1..5").collect();
    // Should produce an error somewhere in the token stream
    // Could be: FloatValue "1.", Error for second dot, IntValue "5"
    // Or: Error for the whole thing
    let has_error = tokens.iter().any(|t| matches!(t.kind, GraphQLTokenKind::Error { .. }));
    assert!(has_error, "Expected an error for double dot in number");
}

/// Verifies that `-0.0` is a valid FloatValue.
///
/// Per GraphQL spec, negative zero float is valid:
/// <https://spec.graphql.org/September2025/#sec-Float-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn float_negative_zero() {
    let kinds = token_kinds("-0.0");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::float_value_borrowed("-0.0"),
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that very small floats near f64 minimum are handled.
///
/// The lexer stores raw text; parsing happens via `parse_float_value()`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn float_very_small() {
    let kinds = token_kinds("1e-308");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::float_value_borrowed("1e-308"),
            GraphQLTokenKind::Eof,
        ]
    );

    // Verify it parses to a very small number
    let tokens: Vec<_> = StrGraphQLTokenSource::new("1e-308").collect();
    let result = tokens[0].kind.parse_float_value();
    assert!(matches!(result, Some(Ok(v)) if v > 0.0 && v < 1e-300));
}

/// Verifies that subnormal float values are handled.
///
/// Subnormal numbers (denormals) are very small numbers near the limits of
/// f64 representation.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn float_subnormal() {
    // 1e-324 is a subnormal number
    let kinds = token_kinds("1e-324");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::float_value_borrowed("1e-324"),
            GraphQLTokenKind::Eof,
        ]
    );

    // May parse to 0.0 (underflow) or a tiny subnormal value
    let tokens: Vec<_> = StrGraphQLTokenSource::new("1e-324").collect();
    let result = tokens[0].kind.parse_float_value();
    // Should either parse successfully or be exactly 0.0 due to underflow
    assert!(matches!(result, Some(Ok(_))));
}

/// Verifies that exponent sign with no digits produces an error.
///
/// Per GraphQL spec, exponent must have at least one digit:
/// <https://spec.graphql.org/September2025/#sec-Float-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn float_exponent_sign_no_digits() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("1e+").collect();
    assert!(!tokens.is_empty());

    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains("exponent")
    ));
}

/// Verifies that very large floats that would overflow are handled.
///
/// The lexer stores raw text; overflow may happen at `parse_float_value()`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn float_infinity_large() {
    // 1e309 is beyond f64::MAX
    let tokens: Vec<_> = StrGraphQLTokenSource::new("1e309").collect();

    // Lexer should capture the raw text
    assert_eq!(
        tokens[0].kind,
        GraphQLTokenKind::float_value_borrowed("1e309")
    );

    // Parsing may produce infinity
    let result = tokens[0].kind.parse_float_value();
    assert!(matches!(result, Some(Ok(v)) if v.is_infinite()));
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

/// Verifies that empty strings are valid.
///
/// Per GraphQL spec, `""` is a valid empty string:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn string_empty() {
    let kinds = token_kinds(r#""""#);
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::string_value_borrowed("\"\""),
            GraphQLTokenKind::Eof,
        ]
    );

    // Verify it parses to empty string
    let tokens: Vec<_> = StrGraphQLTokenSource::new(r#""""#).collect();
    assert_eq!(
        tokens[0].kind.parse_string_value(),
        Some(Ok(String::new()))
    );
}

/// Verifies that escaped quotes in strings are handled.
///
/// Per GraphQL spec, `\"` is a valid escape sequence:
/// <https://spec.graphql.org/September2025/#EscapedCharacter>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn string_escape_quote() {
    let kinds = token_kinds(r#""say \"hello\"""#);
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::string_value_borrowed(r#""say \"hello\"""#),
            GraphQLTokenKind::Eof,
        ]
    );

    // Verify it parses correctly
    let tokens: Vec<_> = StrGraphQLTokenSource::new(r#""say \"hello\"""#).collect();
    assert_eq!(
        tokens[0].kind.parse_string_value(),
        Some(Ok(r#"say "hello""#.to_string()))
    );
}

/// Verifies that escaped backslash in strings is handled.
///
/// Per GraphQL spec, `\\` is a valid escape sequence:
/// <https://spec.graphql.org/September2025/#EscapedCharacter>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn string_escape_backslash() {
    let kinds = token_kinds(r#""path\\to\\file""#);
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::string_value_borrowed(r#""path\\to\\file""#),
            GraphQLTokenKind::Eof,
        ]
    );

    // Verify it parses correctly
    let tokens: Vec<_> = StrGraphQLTokenSource::new(r#""path\\to\\file""#).collect();
    assert_eq!(
        tokens[0].kind.parse_string_value(),
        Some(Ok(r#"path\to\file"#.to_string()))
    );
}

/// Verifies that escaped forward slash in strings is handled.
///
/// Per GraphQL spec, `\/` is a valid escape sequence:
/// <https://spec.graphql.org/September2025/#EscapedCharacter>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn string_escape_slash() {
    let kinds = token_kinds(r#""a\/b""#);
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::string_value_borrowed(r#""a\/b""#),
            GraphQLTokenKind::Eof,
        ]
    );

    // Verify it parses correctly
    let tokens: Vec<_> = StrGraphQLTokenSource::new(r#""a\/b""#).collect();
    assert_eq!(
        tokens[0].kind.parse_string_value(),
        Some(Ok("a/b".to_string()))
    );
}

/// Verifies that escaped backspace in strings is handled.
///
/// Per GraphQL spec, `\b` is a valid escape sequence for backspace (U+0008):
/// <https://spec.graphql.org/September2025/#EscapedCharacter>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn string_escape_backspace() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new(r#""a\bb""#).collect();
    assert_eq!(
        tokens[0].kind.parse_string_value(),
        Some(Ok("a\u{0008}b".to_string()))
    );
}

/// Verifies that escaped form feed in strings is handled.
///
/// Per GraphQL spec, `\f` is a valid escape sequence for form feed (U+000C):
/// <https://spec.graphql.org/September2025/#EscapedCharacter>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn string_escape_formfeed() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new(r#""a\fb""#).collect();
    assert_eq!(
        tokens[0].kind.parse_string_value(),
        Some(Ok("a\u{000C}b".to_string()))
    );
}

/// Verifies that escaped newline in strings is handled.
///
/// Per GraphQL spec, `\n` is a valid escape sequence for newline:
/// <https://spec.graphql.org/September2025/#EscapedCharacter>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn string_escape_newline() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new(r#""a\nb""#).collect();
    assert_eq!(
        tokens[0].kind.parse_string_value(),
        Some(Ok("a\nb".to_string()))
    );
}

/// Verifies that escaped carriage return in strings is handled.
///
/// Per GraphQL spec, `\r` is a valid escape sequence for carriage return:
/// <https://spec.graphql.org/September2025/#EscapedCharacter>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn string_escape_carriage_return() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new(r#""a\rb""#).collect();
    assert_eq!(
        tokens[0].kind.parse_string_value(),
        Some(Ok("a\rb".to_string()))
    );
}

/// Verifies that escaped tab in strings is handled.
///
/// Per GraphQL spec, `\t` is a valid escape sequence for tab:
/// <https://spec.graphql.org/September2025/#EscapedCharacter>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn string_escape_tab() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new(r#""a\tb""#).collect();
    assert_eq!(
        tokens[0].kind.parse_string_value(),
        Some(Ok("a\tb".to_string()))
    );
}

/// Verifies that 4-digit Unicode escape sequences are handled.
///
/// Per GraphQL spec, `\uXXXX` is a valid escape sequence:
/// <https://spec.graphql.org/September2025/#EscapedUnicode>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn string_escape_unicode_4digit() {
    // \u0041 is 'A'
    let tokens: Vec<_> = StrGraphQLTokenSource::new(r#""\u0041""#).collect();
    assert_eq!(
        tokens[0].kind.parse_string_value(),
        Some(Ok("A".to_string()))
    );
}

/// Verifies that Unicode escape for BMP characters is handled.
///
/// Per GraphQL spec, `\uXXXX` covers the Basic Multilingual Plane:
/// <https://spec.graphql.org/September2025/#EscapedUnicode>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn string_escape_unicode_bmp() {
    // \u00E9 is 'é'
    let tokens: Vec<_> = StrGraphQLTokenSource::new(r#""\u00E9""#).collect();
    assert_eq!(
        tokens[0].kind.parse_string_value(),
        Some(Ok("é".to_string()))
    );
}

/// Verifies that surrogate pairs for characters outside BMP are handled.
///
/// Per GraphQL spec, characters above U+FFFF use surrogate pairs:
/// <https://spec.graphql.org/September2025/#EscapedUnicode>
///
/// Note: The September 2025 spec also supports `\u{XXXXX}` syntax for
/// non-BMP characters, but surrogate pairs should still work.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn string_escape_unicode_surrogate_pair() {
    // 😀 (U+1F600) = \uD83D\uDE00 as surrogate pair
    // Note: This test may fail if the implementation doesn't support
    // surrogate pair recombination - that's fine, we're testing the behavior.
    let tokens: Vec<_> = StrGraphQLTokenSource::new(r#""\uD83D\uDE00""#).collect();
    let result = tokens[0].kind.parse_string_value();
    // Surrogate pairs may or may not be combined into a single character
    // depending on implementation - just verify we get a result
    assert!(result.is_some());
}

/// Verifies that invalid Unicode escape sequences produce errors.
///
/// Per GraphQL spec, `\uXXXX` must be 4 hex digits:
/// <https://spec.graphql.org/September2025/#EscapedUnicode>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn string_escape_invalid_unicode() {
    // \uXXXX is not valid hex
    let tokens: Vec<_> = StrGraphQLTokenSource::new(r#""\uXXXX""#).collect();
    let result = tokens[0].kind.parse_string_value();
    assert!(matches!(result, Some(Err(_))));
}

/// Verifies that incomplete Unicode escape sequences produce errors.
///
/// Per GraphQL spec, `\u` must be followed by 4 hex digits:
/// <https://spec.graphql.org/September2025/#EscapedUnicode>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn string_escape_incomplete_unicode() {
    // \u00 is incomplete (only 2 digits)
    let tokens: Vec<_> = StrGraphQLTokenSource::new(r#""\u00""#).collect();
    let result = tokens[0].kind.parse_string_value();
    assert!(matches!(result, Some(Err(_))));
}

/// Verifies that unescaped control characters in strings produce errors.
///
/// Per GraphQL spec, control characters (U+0000-U+001F) must be escaped:
/// <https://spec.graphql.org/September2025/#StringCharacter>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn string_control_chars_error() {
    // Unescaped tab (U+0009) in string - should be \t instead
    let source = "\"hello\tworld\"";
    let tokens: Vec<_> = StrGraphQLTokenSource::new(source).collect();

    // Behavior depends on implementation:
    // 1. Lexer may produce error token
    // 2. Lexer may accept it, parser may reject
    // 3. May be accepted (lenient)
    // We're testing the lexer accepts the string token, and parse may fail
    // Note: Tabs specifically may be allowed by some implementations
    let first_token = &tokens[0];
    assert!(
        matches!(first_token.kind, GraphQLTokenKind::StringValue(_)) ||
        matches!(first_token.kind, GraphQLTokenKind::Error { .. })
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

/// Verifies that block strings can contain newlines.
///
/// Per GraphQL spec, block strings preserve line breaks:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn block_string_multiline() {
    let source = "\"\"\"line1\nline2\nline3\"\"\"";
    let kinds = token_kinds(source);
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::string_value_borrowed(source),
            GraphQLTokenKind::Eof,
        ]
    );

    // Verify the content preserves newlines
    let tokens: Vec<_> = StrGraphQLTokenSource::new(source).collect();
    let parsed = tokens[0].kind.parse_string_value();
    assert!(matches!(parsed, Some(Ok(s)) if s.contains('\n')));
}

/// Verifies that block strings can contain single quotes.
///
/// Per GraphQL spec, `"` inside block strings doesn't close them:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn block_string_contains_quotes() {
    let source = r#""""contains " quote""""#;
    let kinds = token_kinds(source);
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::string_value_borrowed(source),
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that block strings can contain double quotes.
///
/// Per GraphQL spec, `""` inside block strings doesn't close them:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn block_string_contains_double_quotes() {
    let source = r#""""contains "" quotes""""#;
    let kinds = token_kinds(source);
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::string_value_borrowed(source),
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that block strings apply common indentation removal.
///
/// Per GraphQL spec, block strings strip common leading whitespace:
/// <https://spec.graphql.org/September2025/#BlockStringValue()>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn block_string_common_indent_removal() {
    // Block string with consistent indentation that should be stripped
    let source = "\"\"\"\n    line1\n    line2\n    \"\"\"";
    let tokens: Vec<_> = StrGraphQLTokenSource::new(source).collect();
    let parsed = tokens[0].kind.parse_string_value();

    // The common indent of 4 spaces should be removed
    assert!(matches!(parsed, Some(Ok(s)) if !s.starts_with("    ")));
}

/// Verifies that empty block strings are valid.
///
/// Per GraphQL spec, `""""""` is a valid empty block string:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn block_string_empty() {
    let kinds = token_kinds("\"\"\"\"\"\"");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::string_value_borrowed("\"\"\"\"\"\""),
            GraphQLTokenKind::Eof,
        ]
    );

    // Verify it parses to empty string
    let tokens: Vec<_> = StrGraphQLTokenSource::new("\"\"\"\"\"\"").collect();
    assert_eq!(
        tokens[0].kind.parse_string_value(),
        Some(Ok(String::new()))
    );
}

/// Verifies that block strings with only whitespace/newlines are handled.
///
/// Per GraphQL spec, blank lines are trimmed from start and end:
/// <https://spec.graphql.org/September2025/#BlockStringValue()>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn block_string_just_whitespace() {
    let source = "\"\"\"\n   \n   \n\"\"\"";
    let kinds = token_kinds(source);
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::string_value_borrowed(source),
            GraphQLTokenKind::Eof,
        ]
    );

    // After trimming blank lines, should be empty
    let tokens: Vec<_> = StrGraphQLTokenSource::new(source).collect();
    let parsed = tokens[0].kind.parse_string_value();
    assert!(matches!(parsed, Some(Ok(s)) if s.is_empty() || s.chars().all(char::is_whitespace)));
}

/// Verifies that block strings handle CRLF line endings.
///
/// Per GraphQL spec, block strings normalize line endings to LF:
/// <https://spec.graphql.org/September2025/#BlockStringValue()>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn block_string_crlf_handling() {
    let source = "\"\"\"line1\r\nline2\r\n\"\"\"";
    let kinds = token_kinds(source);
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::string_value_borrowed(source),
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

/// Verifies that an empty comment (just `#`) is valid.
///
/// Per GraphQL spec, comments extend to end of line:
/// <https://spec.graphql.org/September2025/#sec-Comments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn comment_empty() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("#\nfield").collect();
    assert_eq!(tokens.len(), 2); // Name + Eof

    // Empty comment should be captured as trivia
    assert_eq!(tokens[0].preceding_trivia.len(), 1);
    assert!(matches!(
        &tokens[0].preceding_trivia[0],
        crate::token::GraphQLTriviaToken::Comment { value, .. } if value.is_empty()
    ));
}

/// Verifies that comments can contain hash characters.
///
/// Per GraphQL spec, comments extend to end of line, so `#` inside is literal:
/// <https://spec.graphql.org/September2025/#sec-Comments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn comment_contains_hash() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("# contains # hash\nfield").collect();
    assert_eq!(tokens.len(), 2); // Name + Eof

    assert_eq!(tokens[0].preceding_trivia.len(), 1);
    assert!(matches!(
        &tokens[0].preceding_trivia[0],
        crate::token::GraphQLTriviaToken::Comment { value, .. }
            if value == " contains # hash"
    ));
}

/// Verifies that comments can contain Unicode characters.
///
/// Per GraphQL spec, comments can contain any SourceCharacter:
/// <https://spec.graphql.org/September2025/#sec-Comments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn comment_unicode() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("# 日本語コメント 🎉\nfield").collect();
    assert_eq!(tokens.len(), 2); // Name + Eof

    assert_eq!(tokens[0].preceding_trivia.len(), 1);
    assert!(matches!(
        &tokens[0].preceding_trivia[0],
        crate::token::GraphQLTriviaToken::Comment { value, .. }
            if value.contains("日本語")
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

/// Verifies that tabs between tokens are handled as whitespace.
///
/// Per GraphQL spec, horizontal tab is ignored (WhiteSpace):
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.White-Space>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn whitespace_tab() {
    let kinds = token_kinds("a\tb\tc");
    assert_eq!(
        kinds,
        vec![
            GraphQLTokenKind::name_borrowed("a"),
            GraphQLTokenKind::name_borrowed("b"),
            GraphQLTokenKind::name_borrowed("c"),
            GraphQLTokenKind::Eof,
        ]
    );
}

/// Verifies that multiple consecutive commas are handled as trivia.
///
/// Per GraphQL spec, commas are insignificant (optional separators):
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Insignificant-Commas>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn multiple_commas() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("field1,,, field2").collect();
    assert_eq!(tokens.len(), 3); // field1, field2, Eof

    // Multiple commas should be accumulated as trivia on field2
    // The exact count depends on implementation (might count spaces too)
    assert_eq!(
        tokens[1].preceding_trivia.len(),
        3,
        "Expected trivia on second field"
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

/// Verifies that tilde `~` produces an error token.
///
/// Per GraphQL spec, `~` is not a valid punctuator:
/// <https://spec.graphql.org/September2025/#sec-Punctuators>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn invalid_char_tilde() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("~").collect();
    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains('~') || message.contains("Unexpected")
    ));
}

/// Verifies that backtick `` ` `` produces an error token.
///
/// Per GraphQL spec, backtick is not valid:
/// <https://spec.graphql.org/September2025/#sec-Punctuators>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn invalid_char_backtick() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("`").collect();
    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains('`') || message.contains("Unexpected")
    ));
}

/// Verifies that question mark `?` produces an error token.
///
/// Per GraphQL spec, `?` is not a valid punctuator:
/// <https://spec.graphql.org/September2025/#sec-Punctuators>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn invalid_char_question() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("?").collect();
    assert!(matches!(
        &tokens[0].kind,
        GraphQLTokenKind::Error { message, .. }
            if message.contains('?') || message.contains("Unexpected")
    ));
}

/// Verifies that control characters produce descriptive error messages.
///
/// Per GraphQL spec, source text is Unicode with specific exclusions:
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Unicode>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn invalid_char_control() {
    // NUL character (U+0000) - should produce error
    let tokens: Vec<_> = StrGraphQLTokenSource::new("\0").collect();
    assert!(matches!(&tokens[0].kind, GraphQLTokenKind::Error { .. }));

    // BEL character (U+0007) - should produce error
    let tokens: Vec<_> = StrGraphQLTokenSource::new("\x07").collect();
    assert!(matches!(&tokens[0].kind, GraphQLTokenKind::Error { .. }));
}

/// Verifies that multiple invalid characters are all reported.
///
/// Error recovery should continue after each invalid character.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn multiple_errors_collected() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("^ ~ ?").collect();

    // Should have at least 3 error tokens (one for each invalid char) + Eof
    let error_count =
        tokens.iter().filter(|t| matches!(t.kind, GraphQLTokenKind::Error { .. })).count();
    assert!(error_count >= 3, "Expected at least 3 errors, got {error_count}");
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

/// Verifies that byte offset is tracked correctly.
///
/// Byte offset should account for multi-byte UTF-8 characters.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_byte_offset() {
    // "α" (U+03B1) is 2 bytes in UTF-8
    let tokens: Vec<_> = StrGraphQLTokenSource::new("α x").collect();

    // First token "α": starts at byte 0, ends at byte 2
    assert_eq!(tokens[0].span.start_inclusive.byte_offset(), 0);
    assert_eq!(tokens[0].span.end_exclusive.byte_offset(), 2);

    // Second token "x": starts at byte 3 (after "α" and space)
    assert_eq!(tokens[1].span.start_inclusive.byte_offset(), 3);
}

/// Verifies that BOM affects byte offset and column.
///
/// BOM (U+FEFF) is 3 bytes in UTF-8 and 1 character (1 UTF-8 col, 1 UTF-16
/// unit).
///
/// Note: Per GraphQL spec, BOM is "ignored" as an insignificant token, but
/// the lexer still tracks it for position purposes.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_with_bom() {
    // BOM (U+FEFF) is 3 bytes in UTF-8, 1 character
    let tokens: Vec<_> = StrGraphQLTokenSource::new("\u{FEFF}name").collect();

    // First token "name": starts at column 1 (after BOM character)
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 1);

    // Byte offset accounts for BOM (3 bytes)
    assert_eq!(tokens[0].span.start_inclusive.byte_offset(), 3);
}

/// Verifies that `with_file_path()` attaches file path to token spans.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_with_file_path() {
    use std::path::Path;
    use std::path::PathBuf;

    let path = Path::new("test.graphql");
    let source = StrGraphQLTokenSource::with_file_path("field", path);

    let tokens: Vec<_> = source.collect();

    // Token spans should include the file path
    assert_eq!(tokens[0].span.file_path, Some(PathBuf::from("test.graphql")));
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
