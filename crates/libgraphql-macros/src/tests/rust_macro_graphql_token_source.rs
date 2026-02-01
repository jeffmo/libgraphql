//! Tests for `RustMacroGraphQLTokenSource`.
//!
//! These tests verify that the token source correctly translates Rust proc-macro
//! tokens into GraphQL tokens per the GraphQL specification.
//!
//! See: https://spec.graphql.org/September2025/#sec-Lexical-Tokens
//!
//! ## Note on `quote!` vs string-based tests
//!
//! Most tests use `quote! { ... }` which provides compile-time Rust syntax
//! checking. However, `quote!` generates synthetic spans that don't preserve
//! accurate position information (all tokens report column 0). For tests that
//! require accurate position tracking (block strings, spaced dots, etc.),
//! we use `TokenStream::from_str()` which preserves real positions.

use crate::rust_macro_graphql_token_source::RustMacroGraphQLTokenSource;
use libgraphql_parser::token::GraphQLToken;
use libgraphql_parser::token::GraphQLTokenKind;
use libgraphql_parser::token::GraphQLTriviaToken;
use proc_macro2::TokenStream;
use quote::quote;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

/// Helper function to tokenize a GraphQL-like token stream and return the token
/// kinds.
///
/// Uses `'static` lifetime since `RustMacroGraphQLTokenSource` produces owned
/// strings (not borrowed from source).
fn tokenize(input: TokenStream) -> Vec<GraphQLTokenKind<'static>> {
    let span_map = Rc::new(RefCell::new(HashMap::new()));
    let source =
        RustMacroGraphQLTokenSource::new(input, span_map);
    source.map(|t| t.kind).collect()
}

/// Helper function to tokenize and return the full tokens (for span/trivia
/// testing).
///
/// Uses `'static` lifetime since `RustMacroGraphQLTokenSource` produces owned
/// strings.
fn tokenize_full(input: TokenStream) -> Vec<GraphQLToken<'static>> {
    let span_map = Rc::new(RefCell::new(HashMap::new()));
    let source =
        RustMacroGraphQLTokenSource::new(input, span_map);
    source.collect()
}

/// Helper to tokenize from a string (preserves accurate positions).
///
/// Uses `'static` lifetime since `RustMacroGraphQLTokenSource` produces owned
/// strings.
fn tokenize_str(input: &str) -> Vec<GraphQLTokenKind<'static>> {
    let stream = TokenStream::from_str(input)
        .expect("Failed to parse as Rust tokens");
    let span_map = Rc::new(RefCell::new(HashMap::new()));
    let source =
        RustMacroGraphQLTokenSource::new(stream, span_map);
    source.map(|t| t.kind).collect()
}

/// Helper to tokenize from a string and return full tokens.
///
/// Uses `'static` lifetime since `RustMacroGraphQLTokenSource` produces owned
/// strings.
fn tokenize_str_full(input: &str) -> Vec<GraphQLToken<'static>> {
    let stream = TokenStream::from_str(input)
        .expect("Failed to parse as Rust tokens");
    let span_map = Rc::new(RefCell::new(HashMap::new()));
    let source =
        RustMacroGraphQLTokenSource::new(stream, span_map);
    source.collect()
}

/// Tests that a simple GraphQL type definition produces the expected token
/// sequence.
///
/// This verifies basic tokenization of Names, punctuators, and Eof.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_simple_type_definition() {
    let kinds = tokenize(quote! { type Query { name: String } });

    // Expected: type, Query, {, name, :, String, }, Eof
    assert_eq!(kinds.len(), 8, "Expected 8 tokens including Eof");

    assert!(matches!(&kinds[0], GraphQLTokenKind::Name(n) if n == "type"));
    assert!(matches!(&kinds[1], GraphQLTokenKind::Name(n) if n == "Query"));
    assert!(matches!(&kinds[2], GraphQLTokenKind::CurlyBraceOpen));
    assert!(matches!(&kinds[3], GraphQLTokenKind::Name(n) if n == "name"));
    assert!(matches!(&kinds[4], GraphQLTokenKind::Colon));
    assert!(matches!(&kinds[5], GraphQLTokenKind::Name(n) if n == "String"));
    assert!(matches!(&kinds[6], GraphQLTokenKind::CurlyBraceClose));
    assert!(matches!(&kinds[7], GraphQLTokenKind::Eof));
}

/// Tests that commas are accumulated as trivia (attached to subsequent tokens)
/// and not emitted as separate tokens.
///
/// Per GraphQL spec, commas are optional and act as trivia.
/// See: https://spec.graphql.org/September2025/#sec-Punctuators
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_commas_as_trivia() {
    let tokens = tokenize_full(quote! { a, b, c });

    // Expected: a, b, c, Eof (4 tokens - commas are trivia)
    assert_eq!(tokens.len(), 4, "Expected 4 tokens (commas are trivia)");

    // First token has no preceding trivia
    assert!(tokens[0].preceding_trivia.is_empty());

    // Second token should have comma trivia
    assert_eq!(
        tokens[1].preceding_trivia.len(),
        1,
        "Second token should have comma trivia"
    );
    assert!(matches!(
        &tokens[1].preceding_trivia[0],
        GraphQLTriviaToken::Comma { .. }
    ));

    // Third token should have comma trivia
    assert_eq!(
        tokens[2].preceding_trivia.len(),
        1,
        "Third token should have comma trivia"
    );
    assert!(matches!(
        &tokens[2].preceding_trivia[0],
        GraphQLTriviaToken::Comma { .. }
    ));
}

/// Tests tokenization of integer literals.
///
/// GraphQL integers are signed 32-bit values.
/// See: https://spec.graphql.org/September2025/#sec-Int-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_integer_literals() {
    let kinds = tokenize(quote! { 0 42 1000000 });

    assert_eq!(kinds.len(), 4, "Expected 4 tokens including Eof");

    assert!(matches!(&kinds[0], GraphQLTokenKind::IntValue(v) if v == "0"));
    assert!(matches!(&kinds[1], GraphQLTokenKind::IntValue(v) if v == "42"));
    assert!(matches!(&kinds[2], GraphQLTokenKind::IntValue(v) if v == "1000000"));
    assert!(matches!(&kinds[3], GraphQLTokenKind::Eof));
}

/// Tests tokenization of negative integer literals.
///
/// In GraphQL, `-17` is a valid IntValue. Rust tokenizes this as two tokens
/// (`-` and `17`), so we combine them into a single negative IntValue.
///
/// See: https://spec.graphql.org/September2025/#sec-Int-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_negative_integer_literals() {
    let kinds = tokenize(quote! { 42 -17 -1 });

    assert_eq!(kinds.len(), 4, "Expected 4 tokens including Eof");

    assert!(matches!(&kinds[0], GraphQLTokenKind::IntValue(v) if v == "42"));
    assert!(matches!(&kinds[1], GraphQLTokenKind::IntValue(v) if v == "-17"));
    assert!(matches!(&kinds[2], GraphQLTokenKind::IntValue(v) if v == "-1"));
    assert!(matches!(&kinds[3], GraphQLTokenKind::Eof));
}

/// Tests tokenization of float literals.
///
/// GraphQL floats follow the JSON number format with optional exponents.
/// See: https://spec.graphql.org/September2025/#sec-Float-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_float_literals() {
    let kinds = tokenize(quote! { 3.14 0.5 1e10 2.5e-3 });

    assert_eq!(kinds.len(), 5, "Expected 5 tokens including Eof");

    assert!(matches!(&kinds[0], GraphQLTokenKind::FloatValue(v) if v == "3.14"));
    assert!(matches!(&kinds[1], GraphQLTokenKind::FloatValue(v) if v == "0.5"));
    assert!(matches!(&kinds[2], GraphQLTokenKind::FloatValue(v) if v == "1e10"));
    assert!(matches!(&kinds[3], GraphQLTokenKind::FloatValue(v) if v == "2.5e-3"));
    assert!(matches!(&kinds[4], GraphQLTokenKind::Eof));
}

/// Tests tokenization of negative float literals.
///
/// Similar to integers, negative floats like `-3.14` are valid FloatValues.
///
/// See: https://spec.graphql.org/September2025/#sec-Float-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_negative_float_literals() {
    let kinds = tokenize(quote! { 3.14 -2.718 -1e5 });

    assert_eq!(kinds.len(), 4, "Expected 4 tokens including Eof");

    assert!(matches!(&kinds[0], GraphQLTokenKind::FloatValue(v) if v == "3.14"));
    assert!(matches!(&kinds[1], GraphQLTokenKind::FloatValue(v) if v == "-2.718"));
    assert!(matches!(&kinds[2], GraphQLTokenKind::FloatValue(v) if v == "-1e5"));
    assert!(matches!(&kinds[3], GraphQLTokenKind::Eof));
}

/// Tests tokenization of string literals.
///
/// String values are stored as raw source text including quotes and escape
/// sequences. Processing (unescaping) happens via `cook_string_value()`.
///
/// See: https://spec.graphql.org/September2025/#sec-String-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_string_literals() {
    let kinds = tokenize(quote! { "hello" "with\nescape" });

    assert_eq!(kinds.len(), 3, "Expected 3 tokens including Eof");

    // String literals stored as raw source text
    assert!(matches!(&kinds[0], GraphQLTokenKind::StringValue(v) if v == "\"hello\""));
    // Note: The backslash-n is stored literally as two characters (\ and n),
    // not as a newline character. The cook_string_value() method processes it.
    assert!(matches!(&kinds[1], GraphQLTokenKind::StringValue(v) if v == "\"with\\nescape\""));
    assert!(matches!(&kinds[2], GraphQLTokenKind::Eof));
}

/// Tests tokenization of boolean literals.
///
/// `true` and `false` are special keywords in GraphQL.
/// See: https://spec.graphql.org/September2025/#sec-Boolean-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_boolean_literals() {
    let kinds = tokenize(quote! { true false });

    assert_eq!(kinds.len(), 3, "Expected 3 tokens including Eof");

    assert!(matches!(&kinds[0], GraphQLTokenKind::True));
    assert!(matches!(&kinds[1], GraphQLTokenKind::False));
    assert!(matches!(&kinds[2], GraphQLTokenKind::Eof));
}

/// Tests tokenization of the null literal.
///
/// See: https://spec.graphql.org/September2025/#sec-Null-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_null_literal() {
    let kinds = tokenize(quote! { null });

    assert_eq!(kinds.len(), 2, "Expected 2 tokens including Eof");

    assert!(matches!(&kinds[0], GraphQLTokenKind::Null));
    assert!(matches!(&kinds[1], GraphQLTokenKind::Eof));
}

/// Tests tokenization of all GraphQL punctuators.
///
/// See: https://spec.graphql.org/September2025/#sec-Punctuators
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_punctuators() {
    let kinds = tokenize(quote! { ! $ & ( ) : = @ [ ] { | } });

    assert_eq!(kinds.len(), 14, "Expected 14 tokens including Eof");

    assert!(matches!(&kinds[0], GraphQLTokenKind::Bang));
    assert!(matches!(&kinds[1], GraphQLTokenKind::Dollar));
    assert!(matches!(&kinds[2], GraphQLTokenKind::Ampersand));
    assert!(matches!(&kinds[3], GraphQLTokenKind::ParenOpen));
    assert!(matches!(&kinds[4], GraphQLTokenKind::ParenClose));
    assert!(matches!(&kinds[5], GraphQLTokenKind::Colon));
    assert!(matches!(&kinds[6], GraphQLTokenKind::Equals));
    assert!(matches!(&kinds[7], GraphQLTokenKind::At));
    assert!(matches!(&kinds[8], GraphQLTokenKind::SquareBracketOpen));
    assert!(matches!(&kinds[9], GraphQLTokenKind::SquareBracketClose));
    assert!(matches!(&kinds[10], GraphQLTokenKind::CurlyBraceOpen));
    assert!(matches!(&kinds[11], GraphQLTokenKind::Pipe));
    assert!(matches!(&kinds[12], GraphQLTokenKind::CurlyBraceClose));
    assert!(matches!(&kinds[13], GraphQLTokenKind::Eof));
}

/// Tests tokenization of the ellipsis (`...`) spread operator.
///
/// The spread operator is a single punctuator, but Rust tokenizes it as three
/// separate `.` tokens. We combine adjacent `.` tokens into `...`.
///
/// See: https://spec.graphql.org/September2025/#sec-Punctuators
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_ellipsis() {
    let kinds = tokenize(quote! { ...Fragment });

    assert_eq!(kinds.len(), 3, "Expected 3 tokens including Eof");

    assert!(matches!(&kinds[0], GraphQLTokenKind::Ellipsis));
    assert!(matches!(&kinds[1], GraphQLTokenKind::Name(n) if n == "Fragment"));
    assert!(matches!(&kinds[2], GraphQLTokenKind::Eof));
}

/// Tests that two non-adjacent dots on the same line produce a terminal error
/// with a helpful message about the spread operator.
///
/// When the user writes `. .` with spaces on the same line, we emit a single
/// error covering both dots (since they can't become `...`), with a note
/// suggesting to remove the spacing.
///
/// Note: Uses `tokenize_str` because `quote!` doesn't preserve position info
/// needed to detect "same line but not adjacent" dots.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_spaced_dots_produce_error() {
    // Using string-based tokenization to preserve position information
    let kinds = tokenize_str(". .");

    // Should be: error (for `. .`), Eof
    assert_eq!(kinds.len(), 2, "Expected 2 tokens including Eof");

    assert!(matches!(
        &kinds[0],
        GraphQLTokenKind::Error { message, .. }
            if message.contains("`. .`")
    ));
    assert!(matches!(&kinds[1], GraphQLTokenKind::Eof));
}

/// Tests that a single `.` produces an error.
///
/// GraphQL does not have a single dot punctuator - only `...`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_single_dot_produces_error() {
    let kinds = tokenize(quote! { name . field });

    // Should be: name, error (for `.`), field, Eof
    assert_eq!(kinds.len(), 4, "Expected 4 tokens including Eof");

    assert!(matches!(&kinds[0], GraphQLTokenKind::Name(n) if n == "name"));
    assert!(matches!(
        &kinds[1],
        GraphQLTokenKind::Error { message, .. }
            if message.contains("`.`")
    ));
    assert!(matches!(&kinds[2], GraphQLTokenKind::Name(n) if n == "field"));
    assert!(matches!(&kinds[3], GraphQLTokenKind::Eof));
}

/// Tests that two adjacent dots (`..`) produce an error.
///
/// Two dots are not valid GraphQL syntax - only `...` is valid.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_double_dot_produces_error() {
    let kinds = tokenize(quote! { name..field });

    // Should be: name, error (for `..`), field, Eof
    assert_eq!(kinds.len(), 4, "Expected 4 tokens including Eof");

    assert!(matches!(&kinds[0], GraphQLTokenKind::Name(n) if n == "name"));
    assert!(matches!(
        &kinds[1],
        GraphQLTokenKind::Error { message, .. }
            if message.contains("`..`")
    ));
    assert!(matches!(&kinds[2], GraphQLTokenKind::Name(n) if n == "field"));
    assert!(matches!(&kinds[3], GraphQLTokenKind::Eof));
}

/// Tests that Rust raw strings produce an error with a helpful suggestion.
///
/// Raw strings (`r"..."`, `r#"..."#`) are Rust-specific syntax. The error
/// includes a suggestion for equivalent GraphQL string syntax.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_raw_string_produces_error() {
    let kinds = tokenize(quote! { r"raw content" });

    assert_eq!(kinds.len(), 2, "Expected 2 tokens including Eof");

    assert!(matches!(
        &kinds[0],
        GraphQLTokenKind::Error { message, error_notes }
            if message.contains("raw string")
            && error_notes.len() == 1
            && error_notes[0].message.contains("Consider using:")
    ));
    assert!(matches!(&kinds[1], GraphQLTokenKind::Eof));
}

/// Tests that an unexpected minus sign (not followed by a number) produces an
/// error.
///
/// The `-` character is only valid when immediately followed by a number.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_standalone_minus_produces_error() {
    let kinds = tokenize(quote! { a - b });

    // Should be: a, error (for `-`), b, Eof
    assert_eq!(kinds.len(), 4, "Expected 4 tokens including Eof");

    assert!(matches!(&kinds[0], GraphQLTokenKind::Name(n) if n == "a"));
    assert!(matches!(
        &kinds[1],
        GraphQLTokenKind::Error { message, .. }
            if message.contains("`-`")
    ));
    assert!(matches!(&kinds[2], GraphQLTokenKind::Name(n) if n == "b"));
    assert!(matches!(&kinds[3], GraphQLTokenKind::Eof));
}

/// Tests that an unexpected punctuation character produces an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_unexpected_punct_produces_error() {
    let kinds = tokenize(quote! { a % b });

    // Should be: a, error (for `%`), b, Eof
    assert_eq!(kinds.len(), 4, "Expected 4 tokens including Eof");

    assert!(matches!(&kinds[0], GraphQLTokenKind::Name(n) if n == "a"));
    assert!(matches!(
        &kinds[1],
        GraphQLTokenKind::Error { message, .. }
            if message.contains("`%`")
    ));
    assert!(matches!(&kinds[2], GraphQLTokenKind::Name(n) if n == "b"));
    assert!(matches!(&kinds[3], GraphQLTokenKind::Eof));
}

/// Tests tokenization of a complete GraphQL query with variables.
///
/// This is an integration test to verify multiple token types work together.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_complete_query() {
    let kinds = tokenize(quote! { query GetUser($id: ID!) { user(id: $id) { name } } });

    // Count expected tokens:
    // query, GetUser, (, $, id, :, ID, !, ), {, user, (, id, :, $, id, ), {,
    // name, }, }, Eof
    assert_eq!(kinds.len(), 22, "Expected 22 tokens including Eof");

    // Verify key tokens
    assert!(matches!(&kinds[0], GraphQLTokenKind::Name(n) if n == "query"));
    assert!(matches!(&kinds[1], GraphQLTokenKind::Name(n) if n == "GetUser"));
    assert!(matches!(&kinds[2], GraphQLTokenKind::ParenOpen));
    assert!(matches!(&kinds[3], GraphQLTokenKind::Dollar));
    assert!(matches!(&kinds[4], GraphQLTokenKind::Name(n) if n == "id"));
    assert!(matches!(&kinds[6], GraphQLTokenKind::Name(n) if n == "ID"));
    assert!(matches!(&kinds[7], GraphQLTokenKind::Bang));
    assert!(matches!(&kinds[21], GraphQLTokenKind::Eof));
}

/// Tests that block strings are correctly detected and combined.
///
/// Rust tokenizes `"""content"""` as three separate string literals. We detect
/// when they are adjacent and combine them.
///
/// Note: Uses `tokenize_str` because `quote!` produces non-adjacent spans for
/// the three string literals, preventing block string detection.
///
/// See: https://spec.graphql.org/September2025/#sec-String-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_block_string() {
    // Using string-based tokenization to preserve adjacent spans
    let kinds = tokenize_str(r#""""block content""""#);

    // Should be: StringValue (block), Eof
    assert_eq!(kinds.len(), 2, "Expected 2 tokens including Eof");

    assert!(matches!(
        &kinds[0],
        GraphQLTokenKind::StringValue(v) if v == "\"\"\"block content\"\"\""
    ));
    assert!(matches!(&kinds[1], GraphQLTokenKind::Eof));
}

/// Tests that non-adjacent empty strings are NOT combined into a block string.
///
/// `"" "content" ""` with spaces should remain as three separate strings.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_spaced_quotes_not_block_string() {
    let kinds = tokenize(quote! { "" "content" "" });

    // Should be: "", "content", "", Eof (4 tokens)
    assert_eq!(kinds.len(), 4, "Expected 4 tokens including Eof");

    assert!(matches!(&kinds[0], GraphQLTokenKind::StringValue(v) if v == "\"\""));
    assert!(matches!(&kinds[1], GraphQLTokenKind::StringValue(v) if v == "\"content\""));
    assert!(matches!(&kinds[2], GraphQLTokenKind::StringValue(v) if v == "\"\""));
    assert!(matches!(&kinds[3], GraphQLTokenKind::Eof));
}

/// Tests that the Eof token receives any trailing trivia (commas).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_trailing_comma_attached_to_eof() {
    let tokens = tokenize_full(quote! { a, });

    // Should be: a, Eof (2 tokens)
    assert_eq!(tokens.len(), 2, "Expected 2 tokens including Eof");

    // The Eof token should have the trailing comma as trivia
    assert!(matches!(&tokens[1].kind, GraphQLTokenKind::Eof));
    assert_eq!(
        tokens[1].preceding_trivia.len(),
        1,
        "Eof should have trailing comma as trivia"
    );
    assert!(matches!(
        &tokens[1].preceding_trivia[0],
        GraphQLTriviaToken::Comma { .. }
    ));
}

/// Tests an empty input produces only an Eof token.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_empty_input() {
    let kinds = tokenize(quote! {});

    assert_eq!(kinds.len(), 1, "Expected 1 token (Eof)");
    assert!(matches!(&kinds[0], GraphQLTokenKind::Eof));
}

/// Tests position tracking across tokens.
///
/// Verifies that span information is preserved from proc_macro2::Span.
/// Note: proc_macro2 uses 1-based lines, but we convert to 0-based.
///
/// Note: Uses `tokenize_str_full` because `quote!` reports all tokens at
/// column 0, preventing accurate position verification.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_position_tracking() {
    let tokens = tokenize_str_full("type Query");

    assert_eq!(tokens.len(), 3, "Expected 3 tokens including Eof");

    let type_token = &tokens[0];
    let query_token = &tokens[1];

    // proc_macro2 reports all tokens on line 1 (we convert to 0-based)
    assert_eq!(type_token.span.start_inclusive.line(), 0);
    assert_eq!(query_token.span.start_inclusive.line(), 0);

    // "type" starts at column 0, "Query" starts at column 5
    assert_eq!(type_token.span.start_inclusive.col_utf8(), 0);
    assert_eq!(query_token.span.start_inclusive.col_utf8(), 5);
}

/// Tests that directive usage with arguments tokenizes correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_directive_with_arguments() {
    let kinds = tokenize(quote! { @deprecated(reason: "old") });

    // @, deprecated, (, reason, :, "old", ), Eof
    assert_eq!(kinds.len(), 8, "Expected 8 tokens including Eof");

    assert!(matches!(&kinds[0], GraphQLTokenKind::At));
    assert!(matches!(&kinds[1], GraphQLTokenKind::Name(n) if n == "deprecated"));
    assert!(matches!(&kinds[2], GraphQLTokenKind::ParenOpen));
    assert!(matches!(&kinds[3], GraphQLTokenKind::Name(n) if n == "reason"));
    assert!(matches!(&kinds[4], GraphQLTokenKind::Colon));
    assert!(matches!(&kinds[5], GraphQLTokenKind::StringValue(v) if v == "\"old\""));
    assert!(matches!(&kinds[6], GraphQLTokenKind::ParenClose));
    assert!(matches!(&kinds[7], GraphQLTokenKind::Eof));
}

/// Tests array syntax with various value types.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_array_values() {
    let kinds = tokenize(quote! { [1, 2, 3] });

    // [, 1, 2, 3, ], Eof (commas are trivia)
    assert_eq!(kinds.len(), 6, "Expected 6 tokens including Eof");

    assert!(matches!(&kinds[0], GraphQLTokenKind::SquareBracketOpen));
    assert!(matches!(&kinds[1], GraphQLTokenKind::IntValue(v) if v == "1"));
    assert!(matches!(&kinds[2], GraphQLTokenKind::IntValue(v) if v == "2"));
    assert!(matches!(&kinds[3], GraphQLTokenKind::IntValue(v) if v == "3"));
    assert!(matches!(&kinds[4], GraphQLTokenKind::SquareBracketClose));
    assert!(matches!(&kinds[5], GraphQLTokenKind::Eof));
}

/// Tests union type syntax with pipes.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_union_type() {
    let kinds = tokenize(quote! { union SearchResult = User | Post | Comment });

    // union, SearchResult, =, User, |, Post, |, Comment, Eof
    assert_eq!(kinds.len(), 9, "Expected 9 tokens including Eof");

    assert!(matches!(&kinds[0], GraphQLTokenKind::Name(n) if n == "union"));
    assert!(matches!(&kinds[1], GraphQLTokenKind::Name(n) if n == "SearchResult"));
    assert!(matches!(&kinds[2], GraphQLTokenKind::Equals));
    assert!(matches!(&kinds[3], GraphQLTokenKind::Name(n) if n == "User"));
    assert!(matches!(&kinds[4], GraphQLTokenKind::Pipe));
    assert!(matches!(&kinds[5], GraphQLTokenKind::Name(n) if n == "Post"));
    assert!(matches!(&kinds[6], GraphQLTokenKind::Pipe));
    assert!(matches!(&kinds[7], GraphQLTokenKind::Name(n) if n == "Comment"));
    assert!(matches!(&kinds[8], GraphQLTokenKind::Eof));
}
