//! Shared helpers for token source parity tests.
//!
//! Provides cross-lifetime comparison functions, tokenization
//! helpers, and assertion functions used by the parity test
//! modules.

use crate::rust_macro_graphql_token_source::RustMacroGraphQLTokenSource;
use libgraphql_parser::token::GraphQLToken;
use libgraphql_parser::token::GraphQLTokenKind;
use libgraphql_parser::token::GraphQLTriviaToken;
use proc_macro2::TokenStream;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

// =========================================================================
// Cross-lifetime comparison helpers
// =========================================================================

/// Compares two `GraphQLTokenKind` values that may have different
/// lifetimes (`'static` vs `'src`).
///
/// The derived `PartialEq` on `GraphQLTokenKind<'src>` constrains
/// both sides to the same `'src`, but `RustMacroGraphQLTokenSource`
/// yields `'static` while `StrGraphQLTokenSource` yields a borrowed
/// `'src`. This function works across lifetimes by matching on
/// variant pairs and comparing inner `Cow` values (which delegate
/// to `str == str`).
///
/// For `Error` variants, this compares both the message string and
/// the error notes (kind + message for each note). Error note spans
/// are intentionally skipped because the two token sources use
/// fundamentally different position tracking.
pub(crate) fn token_kinds_match(
    a: &GraphQLTokenKind<'static>,
    b: &GraphQLTokenKind<'_>,
) -> bool {
    use GraphQLTokenKind::*;
    match (a, b) {
        // Parameterless punctuators
        (Ampersand, Ampersand)
        | (At, At)
        | (Bang, Bang)
        | (Colon, Colon)
        | (CurlyBraceClose, CurlyBraceClose)
        | (CurlyBraceOpen, CurlyBraceOpen)
        | (Dollar, Dollar)
        | (Ellipsis, Ellipsis)
        | (Equals, Equals)
        | (ParenClose, ParenClose)
        | (ParenOpen, ParenOpen)
        | (Pipe, Pipe)
        | (SquareBracketClose, SquareBracketClose)
        | (SquareBracketOpen, SquareBracketOpen) => true,

        // Keywords
        (True, True) | (False, False) | (Null, Null) => true,

        // End of input
        (Eof, Eof) => true,

        // Cow-carrying variants — cross-lifetime Cow comparison
        (Name(a), Name(b)) => a == b,
        (IntValue(a), IntValue(b)) => a == b,
        (FloatValue(a), FloatValue(b)) => a == b,
        (StringValue(a), StringValue(b)) => a == b,

        // Errors — compare messages and error notes
        (
            Error {
                message: m1,
                error_notes: n1,
            },
            Error {
                message: m2,
                error_notes: n2,
            },
        ) => {
            m1 == m2
                && n1.len() == n2.len()
                && n1.iter().zip(n2.iter()).all(|(a, b)| {
                    a.kind == b.kind && a.message == b.message
                    // Intentionally skip `span` — the two
                    // sources use different position tracking
                })
        },

        _ => false,
    }
}

/// Compares two `GraphQLTriviaToken` values across lifetimes.
///
/// Checks variant (Comma vs Comment) and, for Comments, compares
/// the `value` field. Spans are intentionally ignored because
/// the two token sources use fundamentally different position
/// tracking.
pub(crate) fn trivia_kinds_match(
    a: &GraphQLTriviaToken<'static>,
    b: &GraphQLTriviaToken<'_>,
) -> bool {
    match (a, b) {
        (
            GraphQLTriviaToken::Comma { .. },
            GraphQLTriviaToken::Comma { .. },
        ) => true,
        (
            GraphQLTriviaToken::Comment { value: va, .. },
            GraphQLTriviaToken::Comment { value: vb, .. },
        ) => va == vb,
        _ => false,
    }
}

// =========================================================================
// Tokenization helpers
// =========================================================================

/// Tokenize a string via `RustMacroGraphQLTokenSource`.
///
/// Parses the input into a `TokenStream` first, then feeds it to
/// the Rust-macro token source.
pub(crate) fn tokenize_via_rust(
    input: &str,
) -> Vec<GraphQLToken<'static>> {
    let stream = TokenStream::from_str(input)
        .expect("Failed to parse as Rust tokens");
    let span_map = Rc::new(RefCell::new(HashMap::new()));
    let source =
        RustMacroGraphQLTokenSource::new(stream, span_map);
    source.collect()
}

/// Tokenize a string via `StrGraphQLTokenSource`.
pub(crate) fn tokenize_via_str(
    input: &str,
) -> Vec<GraphQLToken<'_>> {
    let source =
        libgraphql_parser::token_source::StrGraphQLTokenSource::new(
            input,
        );
    source.collect()
}

// =========================================================================
// Assertion helpers
// =========================================================================

/// Asserts that both token sources produce identical output for the
/// given input.
///
/// Checks:
/// 1. Same number of tokens
/// 2. Token kinds match at each position (including error notes)
/// 3. Same number of trivia items at each position
/// 4. Trivia kinds match at each position
pub(crate) fn assert_parity(input: &str) {
    let rust_tokens = tokenize_via_rust(input);
    let str_tokens = tokenize_via_str(input);

    assert_eq!(
        rust_tokens.len(),
        str_tokens.len(),
        "Token count mismatch for input: {input:?}\n\
         Rust tokens: {rust_kinds:?}\n\
         Str tokens:  {str_kinds:?}",
        rust_kinds = rust_tokens
            .iter()
            .map(|t| format!("{:?}", t.kind))
            .collect::<Vec<_>>(),
        str_kinds = str_tokens
            .iter()
            .map(|t| format!("{:?}", t.kind))
            .collect::<Vec<_>>(),
    );

    for (i, (rt, st)) in rust_tokens.iter().zip(str_tokens.iter()).enumerate() {
        assert!(
            token_kinds_match(&rt.kind, &st.kind),
            "Token kind mismatch at position {i} for input: \
             {input:?}\n  Rust: {:?}\n  Str:  {:?}",
            rt.kind,
            st.kind,
        );

        assert_eq!(
            rt.preceding_trivia.len(),
            st.preceding_trivia.len(),
            "Trivia count mismatch at position {i} for input: \
             {input:?}\n  Rust trivia: {:?}\n  Str trivia:  {:?}",
            rt.preceding_trivia,
            st.preceding_trivia,
        );

        for (j, (rtv, stv)) in rt
            .preceding_trivia
            .iter()
            .zip(st.preceding_trivia.iter())
            .enumerate()
        {
            assert!(
                trivia_kinds_match(rtv, stv),
                "Trivia mismatch at position {i}, trivia {j} \
                 for input: {input:?}\n  Rust: {rtv:?}\n  \
                 Str:  {stv:?}",
            );
        }
    }
}

