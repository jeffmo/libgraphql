//! Tests for GraphQLTokenStream.
//!
//! Written by Claude Code, reviewed by a human.

use crate::tests::utils;
use crate::token::GraphQLToken;
use crate::token::GraphQLTokenKind;
use crate::GraphQLTokenStream;

// =============================================================================
// Basic functionality tests
// =============================================================================

/// Verifies that peek() returns the next token without consuming it.
/// Multiple peeks should return the same token.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_peek_without_consuming() {
    let tokens = vec![
        utils::mock_name_token("type"),
        utils::mock_name_token("Query"),
        utils::mock_eof_token(),
    ];
    let mut stream = GraphQLTokenStream::new(utils::MockTokenSource::new(tokens));

    // Peek multiple times should return same token
    let first_peek = stream.peek().map(|t| t.kind.clone());
    let second_peek = stream.peek().map(|t| t.kind.clone());

    assert_eq!(first_peek, second_peek);
    assert!(
        matches!(first_peek, Some(GraphQLTokenKind::Name(ref name)) if name == "type")
    );

    // Now consume it
    let consumed = stream.consume().map(|t| t.kind.clone());
    assert_eq!(first_peek, consumed);
}

/// Verifies that consume() advances to the next token.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_consume_advances_token() {
    let tokens = vec![
        utils::mock_name_token("type"),
        utils::mock_name_token("Query"),
        utils::mock_eof_token(),
    ];
    let mut stream = GraphQLTokenStream::new(utils::MockTokenSource::new(tokens));

    // Consume first token
    let first = stream.consume().map(|t| t.kind.clone());
    assert!(matches!(first, Some(GraphQLTokenKind::Name(name)) if name == "type"));

    // Next peek should be different token
    let second = stream.peek().map(|t| t.kind.clone());
    assert!(matches!(second, Some(GraphQLTokenKind::Name(name)) if name == "Query"));
}

/// Verifies that current_token() returns the most recently consumed token.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_current_token() {
    let tokens = vec![
        utils::mock_name_token("type"),
        utils::mock_name_token("Query"),
        utils::mock_eof_token(),
    ];
    let mut stream = GraphQLTokenStream::new(utils::MockTokenSource::new(tokens));

    // Before consuming, current_token should be None
    assert!(stream.current_token().is_none());

    // After consuming, current_token should return that token
    stream.consume();
    let current = stream.current_token().map(|t| t.kind.clone());
    assert!(matches!(current, Some(GraphQLTokenKind::Name(name)) if name == "type"));

    // Consume another and verify current_token updates
    stream.consume();
    let current = stream.current_token().map(|t| t.kind.clone());
    assert!(matches!(current, Some(GraphQLTokenKind::Name(name)) if name == "Query"));
}

// =============================================================================
// Lookahead tests
// =============================================================================

/// Verifies that peek_nth() provides correct lookahead.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_peek_nth_lookahead() {
    let tokens = vec![
        utils::mock_name_token("type"),
        utils::mock_name_token("Query"),
        utils::mock_token(GraphQLTokenKind::CurlyBraceOpen),
        utils::mock_name_token("field"),
        utils::mock_eof_token(),
    ];
    let mut stream = GraphQLTokenStream::new(utils::MockTokenSource::new(tokens));

    // Peek at different positions
    let token_0 = stream.peek_nth(0).map(|t| t.kind.clone());
    let token_1 = stream.peek_nth(1).map(|t| t.kind.clone());
    let token_2 = stream.peek_nth(2).map(|t| t.kind.clone());

    assert!(
        matches!(token_0, Some(GraphQLTokenKind::Name(ref name)) if name == "type")
    );
    assert!(
        matches!(token_1, Some(GraphQLTokenKind::Name(ref name)) if name == "Query")
    );
    assert!(matches!(token_2, Some(GraphQLTokenKind::CurlyBraceOpen)));

    // Consuming shouldn't affect what peek_nth saw for remaining tokens
    stream.consume();
    let new_token_0 = stream.peek_nth(0).map(|t| t.kind.clone());
    assert_eq!(token_1, new_token_0);
}

/// Verifies that peek_nth() returns None when looking beyond stream end.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_peek_nth_beyond_end() {
    let tokens = vec![utils::mock_name_token("type"), utils::mock_eof_token()];
    let mut stream = GraphQLTokenStream::new(utils::MockTokenSource::new(tokens));

    // Peek way beyond the stream
    let result = stream.peek_nth(100);
    assert!(result.is_none());
}

/// Verifies that peek_nth(0) is equivalent to peek().
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_peek_nth_zero_equals_peek() {
    let tokens = vec![
        utils::mock_name_token("type"),
        utils::mock_name_token("Query"),
        utils::mock_eof_token(),
    ];
    let mut stream = GraphQLTokenStream::new(utils::MockTokenSource::new(tokens));

    let peek_result = stream.peek().map(|t| t.kind.clone());
    let peek_nth_result = stream.peek_nth(0).map(|t| t.kind.clone());

    assert_eq!(peek_result, peek_nth_result);
}

// =============================================================================
// End-of-stream tests
// =============================================================================

/// Verifies that is_at_end() returns true when stream is exhausted.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_is_at_end() {
    let tokens = vec![utils::mock_name_token("type"), utils::mock_eof_token()];
    let mut stream = GraphQLTokenStream::new(utils::MockTokenSource::new(tokens));

    assert!(!stream.is_at_end());

    stream.consume(); // consume "type"
    assert!(stream.is_at_end()); // next is Eof
}

/// Verifies that is_at_end() returns true for an empty stream.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_is_at_end_empty_stream() {
    let tokens: Vec<GraphQLToken> = vec![];
    let mut stream = GraphQLTokenStream::new(utils::MockTokenSource::new(tokens));

    assert!(stream.is_at_end());
}

/// Verifies that consume() returns None when stream is exhausted.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_consume_at_end_returns_none() {
    let tokens = vec![utils::mock_name_token("type")];
    let mut stream = GraphQLTokenStream::new(utils::MockTokenSource::new(tokens));

    assert!(stream.consume().is_some()); // consume "type"
    assert!(stream.consume().is_none()); // no more tokens
}

// =============================================================================
// Internal buffer management tests
// =============================================================================

/// Verifies that peek followed by consume works correctly (internal buffer
/// management).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_mixed_peek_and_consume() {
    let tokens = vec![
        utils::mock_name_token("type"),
        utils::mock_name_token("Query"),
        utils::mock_eof_token(),
    ];
    let mut stream = GraphQLTokenStream::new(utils::MockTokenSource::new(tokens));

    // Peek ahead
    let peeked = stream.peek_nth(1).map(|t| t.kind.clone());

    // Consume first
    stream.consume();

    // What we peeked should now be at position 0
    let now_first = stream.peek().map(|t| t.kind.clone());
    assert_eq!(peeked, now_first);
}

/// Verifies that tokens come in correct order after lookahead.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_buffer_order_after_lookahead() {
    let tokens = vec![
        utils::mock_name_token("type"),
        utils::mock_name_token("Query"),
        utils::mock_token(GraphQLTokenKind::CurlyBraceOpen),
        utils::mock_name_token("field"),
        utils::mock_eof_token(),
    ];
    let mut stream = GraphQLTokenStream::new(utils::MockTokenSource::new(tokens));

    // Force internal buffer to fill by peeking ahead
    stream.peek_nth(3);

    // Now consume tokens and verify they come in correct order
    let mut consumed: Vec<GraphQLTokenKind> = Vec::new();
    for _ in 0..4 {
        if let Some(token) = stream.consume() {
            consumed.push(token.kind.clone());
        }
    }

    assert!(matches!(consumed[0], GraphQLTokenKind::Name(ref n) if n == "type"));
    assert!(matches!(consumed[1], GraphQLTokenKind::Name(ref n) if n == "Query"));
    assert!(matches!(consumed[2], GraphQLTokenKind::CurlyBraceOpen));
    assert!(matches!(consumed[3], GraphQLTokenKind::Name(ref n) if n == "field"));
}

/// Verifies that compact_buffer() keeps internal buffer bounded.
///
/// This is a regression test to ensure that consuming many tokens with
/// periodic compaction doesn't cause unbounded memory growth.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_buffer_compaction_bounds_memory() {
    // Create a source with 10,000+ tokens
    let token_count = 10_000;
    let tokens: Vec<GraphQLToken> = (0..token_count)
        .map(|i| utils::mock_name_token(&format!("token{i}")))
        .chain(std::iter::once(utils::mock_eof_token()))
        .collect();

    let mut stream = GraphQLTokenStream::new(utils::MockTokenSource::new(tokens));

    // Consume all tokens, compacting periodically (every 100 tokens)
    let mut consumed_count: u16 = 0;
    while stream.consume().is_some() {
        consumed_count += 1;
        let modulo = consumed_count % 100;
        eprintln!("\nconsumed_count=`{consumed_count}`, consumed_count%100=`{modulo}`, buff_len=`{}`", stream.current_buffer_len());
        if consumed_count.is_multiple_of(100) {
            eprintln!("  Compacting...");
            let mut expected_buf_len = 101;
            if consumed_count == 100 {
                expected_buf_len -= 1;
            }
            assert_eq!(stream.current_buffer_len(), expected_buf_len);
            stream.compact_buffer();
            assert_eq!(stream.current_buffer_len(), 1);
        }
    }

    assert_eq!(consumed_count, token_count + 1); // +1 for Eof token

    // After final compaction, internal buffer should be minimal
    stream.compact_buffer();
    assert_eq!(stream.current_buffer_len(), 0);
}

/// Verifies that compact_buffer() preserves current token access.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_compact_buffer_preserves_current_token() {
    let tokens = vec![
        utils::mock_name_token("first"),
        utils::mock_name_token("second"),
        utils::mock_name_token("third"),
        utils::mock_eof_token(),
    ];
    let mut stream = GraphQLTokenStream::new(utils::MockTokenSource::new(tokens));

    // Consume two tokens
    stream.consume();
    stream.consume();

    // Current token should be "second"
    let before_compact = stream.current_token().map(|t| t.kind.clone());
    assert!(
        matches!(before_compact, Some(GraphQLTokenKind::Name(ref n)) if n == "second")
    );

    // Compact internal buffer
    stream.compact_buffer();

    // Current token should still be accessible
    let after_compact = stream.current_token().map(|t| t.kind.clone());
    assert_eq!(before_compact, after_compact);

    // Should still be able to consume next token
    let next = stream.consume().map(|t| t.kind.clone());
    assert!(matches!(next, Some(GraphQLTokenKind::Name(ref n)) if n == "third"));
}
