//! Tests for `GraphQLToken` construction and basic operations.
//!
//! Written by Claude Code, reviewed by a human.

use crate::token::GraphQLToken;
use crate::token::GraphQLTokenKind;
use crate::GraphQLSourceSpan;
use crate::SourcePosition;

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
    let span = GraphQLSourceSpan::new(
        SourcePosition::new(0, 0, Some(0), 0),
        SourcePosition::new(0, 3, Some(3), 3),
    );
    let token = GraphQLToken::new(
        GraphQLTokenKind::name_owned("foo".to_string()),
        span.clone(),
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
    let span = GraphQLSourceSpan::new(
        SourcePosition::new(0, 0, Some(0), 0),
        SourcePosition::new(0, 1, Some(1), 1),
    );

    // Test punctuator
    let token = GraphQLToken::new(GraphQLTokenKind::Bang, span.clone());
    assert!(matches!(token.kind, GraphQLTokenKind::Bang));
    assert!(token.preceding_trivia.is_empty());

    // Test keyword
    let token = GraphQLToken::new(GraphQLTokenKind::True, span.clone());
    assert!(matches!(token.kind, GraphQLTokenKind::True));
    assert!(token.preceding_trivia.is_empty());

    // Test EOF
    let token = GraphQLToken::new(GraphQLTokenKind::Eof, span.clone());
    assert!(matches!(token.kind, GraphQLTokenKind::Eof));
    assert!(token.preceding_trivia.is_empty());
}

/// Verifies that token span is preserved correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn graphql_token_preserves_span() {
    let start = SourcePosition::new(5, 10, Some(10), 100);
    let end = SourcePosition::new(5, 15, Some(15), 105);
    let span = GraphQLSourceSpan::new(start.clone(), end.clone());

    let token = GraphQLToken::new(
        GraphQLTokenKind::name_owned("hello".to_string()),
        span,
    );

    assert_eq!(token.span.start_inclusive.line(), 5);
    assert_eq!(token.span.start_inclusive.col_utf8(), 10);
    assert_eq!(token.span.start_inclusive.byte_offset(), 100);
    assert_eq!(token.span.end_exclusive.line(), 5);
    assert_eq!(token.span.end_exclusive.col_utf8(), 15);
    assert_eq!(token.span.end_exclusive.byte_offset(), 105);
}
