//! Section A — Valid Input Parity tests.
//!
//! These tests feed identical valid input strings to both
//! `RustMacroGraphQLTokenSource` and `StrGraphQLTokenSource`,
//! asserting their outputs match exactly (token kinds, trivia
//! structure, and error notes).
//!
//! See: https://spec.graphql.org/September2025/#sec-Lexical-Tokens

use crate::tests::token_source_parity_utils::assert_parity;

/// Tests that an empty input produces identical output from both
/// token sources: a single Eof token.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_empty() {
    assert_parity("");
}

/// Tests that simple name tokens produce identical output.
///
/// See: https://spec.graphql.org/September2025/#Name
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_names() {
    assert_parity("type Query");
}

/// Tests that all GraphQL punctuators produce identical tokens.
///
/// See: https://spec.graphql.org/September2025/#sec-Punctuators
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_all_punctuators() {
    assert_parity("! $ & ( ) : = @ [ ] { | }");
}

/// Tests that the ellipsis (`...`) spread operator followed by a
/// name produces identical tokens.
///
/// See: https://spec.graphql.org/September2025/#sec-Punctuators
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_ellipsis() {
    assert_parity("...Fragment");
}

/// Tests that integer literals produce identical tokens.
///
/// See: https://spec.graphql.org/September2025/#sec-Int-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_integers() {
    assert_parity("0 42 1000000");
}

/// Tests that a negative integer literal produces identical tokens.
///
/// Both sources should combine `-` and `17` into a single
/// `IntValue("-17")`.
///
/// See: https://spec.graphql.org/September2025/#sec-Int-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_negative_integer() {
    assert_parity("-17");
}

/// Tests that float literals produce identical tokens.
///
/// See: https://spec.graphql.org/September2025/#sec-Float-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_floats() {
    assert_parity("3.14 0.5");
}

/// Tests that a negative float literal produces identical tokens.
///
/// See: https://spec.graphql.org/September2025/#sec-Float-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_negative_float() {
    assert_parity("-2.718");
}

/// Tests that a simple string literal produces identical tokens.
///
/// See: https://spec.graphql.org/September2025/#sec-String-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_string() {
    assert_parity(r#""hello""#);
}

/// Tests that a string with an escape sequence produces identical
/// tokens.
///
/// Note: The raw source text `"with\nescape"` is stored as-is by
/// both token sources (the `\n` is two characters, not a newline).
///
/// See: https://spec.graphql.org/September2025/#sec-String-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_string_with_escape() {
    assert_parity(r#""with\nescape""#);
}

/// Tests that boolean literals `true` and `false` produce
/// identical tokens.
///
/// See: https://spec.graphql.org/September2025/#sec-Boolean-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_booleans() {
    assert_parity("true false");
}

/// Tests that the `null` literal produces identical tokens.
///
/// See: https://spec.graphql.org/September2025/#sec-Null-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_null() {
    assert_parity("null");
}

/// Tests that identifiers starting with keyword prefixes (like
/// `trueValue`, `nullable`) are tokenized as `Name` by both
/// sources, not as keywords.
///
/// See: https://spec.graphql.org/September2025/#Name
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_keyword_prefixes() {
    assert_parity("trueValue nullable");
}

/// Tests that commas are captured as trivia (not tokens) by both
/// sources.
///
/// See: https://spec.graphql.org/September2025/#sec-Insignificant-Commas
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_commas_trivia() {
    assert_parity("a, b, c");
}

/// Tests that multiple consecutive commas are all captured as
/// trivia on the following token.
///
/// See: https://spec.graphql.org/September2025/#sec-Insignificant-Commas
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_multiple_commas() {
    assert_parity("a,,, b");
}

/// Tests that a trailing comma is captured as trivia on the Eof
/// token.
///
/// See: https://spec.graphql.org/September2025/#sec-Insignificant-Commas
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_trailing_comma() {
    assert_parity("a,");
}

/// Tests that a realistic schema snippet produces identical tokens.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_schema_snippet() {
    assert_parity("type Query { name: String }");
}

/// Tests that directive-with-arguments syntax produces identical
/// tokens.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_directive_args() {
    assert_parity(r#"@deprecated(reason: "old")"#);
}

/// Tests that union type syntax produces identical tokens.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_union() {
    assert_parity("union R = A | B | C");
}

/// Tests that variable syntax produces identical tokens.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_variable_syntax() {
    assert_parity("($id: ID!)");
}

/// Tests that float literals with exponent notation (`1e10`)
/// produce identical tokens.
///
/// Empirically verified: Rust's `TokenStream::from_str("1e10")`
/// produces a float literal, matching what `StrGraphQLTokenSource`
/// produces as a `FloatValue`.
///
/// See: https://spec.graphql.org/September2025/#sec-Float-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_float_exponent() {
    assert_parity("1e10");
}

/// Tests that block strings produce identical tokens when both
/// token sources can handle them.
///
/// Empirically verified: `TokenStream::from_str(r#""""block
/// content""""#)` is successfully combined by
/// `RustMacroGraphQLTokenSource` into a single block string token.
///
/// See: https://spec.graphql.org/September2025/#sec-String-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parity_block_string() {
    assert_parity(r#""""block content""""#);
}
