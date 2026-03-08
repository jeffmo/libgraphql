//! Tests for `GraphQLToken` construction and basic operations.
//!
//! Written by Claude Code, reviewed by a human.

use crate::ByteSpan;
use crate::token::GraphQLToken;
use crate::token::GraphQLTokenKind;

// =============================================================================
// GraphQLToken Constructor Tests
// =============================================================================

/// Verifies that `GraphQLToken::new()` creates a token with empty preceding
/// trivia.
///
/// The `new()` constructor is a convenience method for the common case where
/// a token has no preceding comments or commas.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn graphql_token_new_creates_empty_trivia() {
    let span = ByteSpan::new(0, 3);
    let token = GraphQLToken::new(
        GraphQLTokenKind::name_owned("foo".to_string()),
        span,
    );

    // Verify the token was created with correct kind and span
    assert!(matches!(token.kind, GraphQLTokenKind::Name(_)));
    assert_eq!(token.span, span);

    // Verify preceding_trivia is empty
    assert!(
        token.preceding_trivia.is_empty(),
        "new() should create token with empty preceding_trivia"
    );
}

/// Verifies that tokens with different kinds are created correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn graphql_token_new_various_kinds() {
    let span = ByteSpan::new(0, 1);

    // Test punctuator
    let token = GraphQLToken::new(GraphQLTokenKind::Bang, span);
    assert!(matches!(token.kind, GraphQLTokenKind::Bang));
    assert!(token.preceding_trivia.is_empty());

    // Test keyword
    let token = GraphQLToken::new(GraphQLTokenKind::True, span);
    assert!(matches!(token.kind, GraphQLTokenKind::True));
    assert!(token.preceding_trivia.is_empty());

    // Test EOF
    let token = GraphQLToken::new(GraphQLTokenKind::Eof, span);
    assert!(matches!(token.kind, GraphQLTokenKind::Eof));
    assert!(token.preceding_trivia.is_empty());
}

/// Verifies that token span is preserved correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn graphql_token_preserves_span() {
    let span = ByteSpan::new(100, 105);

    let token = GraphQLToken::new(
        GraphQLTokenKind::name_owned("hello".to_string()),
        span,
    );

    assert_eq!(token.span.start, 100);
    assert_eq!(token.span.end, 105);
}
