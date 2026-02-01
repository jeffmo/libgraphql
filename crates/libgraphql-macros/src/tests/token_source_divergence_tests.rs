//! Section C — Documented Divergence tests.
//!
//! These tests document cases where the two token sources
//! intentionally produce different output due to fundamental
//! architectural differences (Rust's tokenizer vs. a custom
//! GraphQL lexer).
//!
//! Each test explains **why** the divergence exists and asserts
//! the actual divergent behavior from both sides.

use crate::rust_macro_graphql_token_source::RustMacroGraphQLTokenSource;
use crate::tests::token_source_parity_utils::tokenize_via_str;
use crate::tests::token_source_parity_utils::tokenize_via_rust;
use libgraphql_parser::token::GraphQLTokenKind;
use libgraphql_parser::token::GraphQLTriviaToken;
use proc_macro2::TokenStream;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

/// Documents that `#` is treated as a comment by
/// `StrGraphQLTokenSource` but as an error by
/// `RustMacroGraphQLTokenSource`.
///
/// **Why:** Rust's tokenizer converts `#` into a `Punct('#')`
/// token, which the Rust-macro source doesn't recognize as a
/// comment prefix. By contrast, `StrGraphQLTokenSource` lexes `#`
/// as comment trivia per the GraphQL spec.
///
/// See: https://spec.graphql.org/September2025/#sec-Comments
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn divergence_hash_is_comment_vs_error() {
    // Str source: "# comment" is trivia, "field" is a Name, then
    // Eof
    let str_tokens = tokenize_via_str("# comment\nfield");
    assert_eq!(str_tokens.len(), 2); // field + Eof
    assert!(matches!(
        &str_tokens[0].kind,
        GraphQLTokenKind::Name(n) if n == "field",
    ));
    // The comment should be trivia on the "field" token
    assert_eq!(str_tokens[0].preceding_trivia.len(), 1);
    assert!(matches!(
        &str_tokens[0].preceding_trivia[0],
        GraphQLTriviaToken::Comment { value, .. }
            if value == " comment",
    ));

    // Rust source: `# comment\nfield` — Rust's tokenizer sees `#`
    // as Punct, then `comment` as Ident, then `field` as Ident.
    // The `#` is not on the "allowed punct" list so it becomes an
    // error token.
    let rust_tokens = tokenize_via_rust("# comment\nfield");
    // We expect: Error(#), Name("comment"), Name("field"), Eof
    assert!(
        rust_tokens.len() >= 3,
        "Expected at least 3 tokens, got: {:?}",
        rust_tokens
            .iter()
            .map(|t| format!("{:?}", t.kind))
            .collect::<Vec<_>>(),
    );
    assert!(
        rust_tokens[0].kind.is_error(),
        "Expected error for `#`, got: {:?}",
        rust_tokens[0].kind,
    );
}

/// Documents that Rust raw strings are handled differently by
/// each source.
///
/// - `RustMacroGraphQLTokenSource`: Sees `r"raw"` as a single
///   raw-string literal and emits a specific "raw strings not valid
///   GraphQL" error.
/// - `StrGraphQLTokenSource`: Has no concept of Rust raw strings.
///   It sees `r` as a Name token and `"raw"` as a StringValue
///   token.
///
/// This test uses `quote!` to generate the raw string token since
/// `TokenStream::from_str` can produce raw strings, but we verify
/// the Str source sees different tokens for the equivalent text.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn divergence_raw_string_rust_only() {
    use quote::quote;

    // Rust source: raw string literal → error token
    let stream = quote! { r"raw content" };
    let span_map = Rc::new(RefCell::new(HashMap::new()));
    let source =
        RustMacroGraphQLTokenSource::new(stream, span_map);
    let rust_tokens: Vec<_> = source.collect();

    assert!(
        rust_tokens[0].kind.is_error(),
        "Rust source should produce an error for raw string, \
         got: {:?}",
        rust_tokens[0].kind,
    );
    let rust_msg = match &rust_tokens[0].kind {
        GraphQLTokenKind::Error { message, .. } => message,
        _ => panic!("Expected error"),
    };
    assert!(
        rust_msg.contains("raw string"),
        "Error should mention 'raw string', got: {rust_msg}",
    );

    // Str source: `r"raw content"` → Name("r") + StringValue
    let str_tokens = tokenize_via_str(r#"r"raw content""#);
    assert!(matches!(
        &str_tokens[0].kind,
        GraphQLTokenKind::Name(n) if n == "r",
    ));
    assert!(matches!(
        &str_tokens[1].kind,
        GraphQLTokenKind::StringValue(_),
    ));
}

/// Documents that unterminated strings are caught at different
/// stages by each source.
///
/// - `StrGraphQLTokenSource`: Sees `"unterminated` as an error
///   token (unterminated string).
/// - `RustMacroGraphQLTokenSource`: `TokenStream::from_str`
///   rejects the input before it even reaches the token source, so
///   we can't produce tokens at all.
///
/// This is an inherent limitation of the Rust-macro approach:
/// string validation happens in Rust's tokenizer, not ours.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn divergence_unterminated_string() {
    // Str source can tokenize and reports an error
    let str_tokens = tokenize_via_str("\"unterminated");
    assert!(
        str_tokens.iter().any(|t| t.kind.is_error()),
        "Str source should report an error for unterminated \
         string",
    );

    // Rust source: from_str fails before we can tokenize
    let result = TokenStream::from_str("\"unterminated");
    assert!(
        result.is_err(),
        "TokenStream::from_str should reject unterminated string",
    );
}
