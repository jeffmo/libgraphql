//! Tests for `GraphQLTokenStream` after the B5 optimization that
//! changed `consume()` from returning a borrowed reference
//! (`Option<&GraphQLToken>`) to an owned value
//! (`Option<GraphQLToken>`) via `VecDeque::pop_front()`.
//!
//! ## Optimization summary
//!
//! Prior to B5, the internal buffer was a `Vec<GraphQLToken>` with
//! a `current_index` tracker. `consume()` returned a reference into
//! the buffer, which meant the parser had to `.clone()` every token
//! it consumed. B5 replaced the buffer with a `VecDeque` and changed
//! `consume()` to return the token via `pop_front()`, transferring
//! ownership directly to the caller. This also eliminated
//! `compact_buffer()` and `current_token()` since the `VecDeque`
//! naturally discards consumed tokens.
//!
//! ## What these tests verify
//!
//! - `consume()` returns a fully owned token (kind, span, trivia)
//! - Consumed tokens are independent of the stream (mutation doesn't
//!   affect subsequent tokens)
//! - The peek-then-drop-then-consume pattern (used by `expect()`)
//!   works without lifetime issues
//! - All tokens can be collected into a `Vec` with full ownership
//! - The `VecDeque` ring buffer stays bounded without explicit
//!   compaction
//!
//! Written by Claude Code, reviewed by a human.

use crate::tests::utils;
use crate::token::GraphQLTokenKind;
use crate::GraphQLTokenStream;

// =============================================================================
// Owned consume semantics
// =============================================================================

/// Verifies that `consume()` returns a fully owned token with the
/// correct kind, span, and trivia.
///
/// The owned return value is the key B5 change: `pop_front()`
/// transfers ownership from the `VecDeque` buffer to the caller
/// without cloning.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn consume_returns_owned_token_with_correct_fields() {
    let tokens = vec![
        utils::mock_name_token("type"),
        utils::mock_eof_token(),
    ];
    let mut stream = GraphQLTokenStream::new(
        utils::MockTokenSource::new(tokens),
    );

    let token = stream.consume().expect("should return a token");

    // Verify the token has the correct kind
    assert!(
        matches!(token.kind, GraphQLTokenKind::Name(ref n) if n == "type"),
    );

    // Verify span is present (mock tokens have zeroed positions)
    assert_eq!(token.span.start_inclusive.byte_offset(), 0);
    assert_eq!(token.span.end_exclusive.byte_offset(), 0);

    // Verify trivia is empty (mock tokens have no trivia)
    assert!(token.preceding_trivia.is_empty());
}

/// Verifies that a consumed token is fully independent of the
/// stream. Mutating the consumed token does not affect subsequent
/// tokens from the stream.
///
/// This confirms true ownership transfer: after `pop_front()`,
/// the token is no longer part of the buffer and modifications
/// are purely local.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn consumed_token_is_independent_of_stream() {
    let tokens = vec![
        utils::mock_name_token("first"),
        utils::mock_name_token("second"),
        utils::mock_eof_token(),
    ];
    let mut stream = GraphQLTokenStream::new(
        utils::MockTokenSource::new(tokens),
    );

    // Consume first token and mutate it
    let mut consumed = stream.consume().unwrap();
    consumed.kind = GraphQLTokenKind::Eof;

    // The mutation should not affect the next token in the stream
    let next = stream.peek().unwrap();
    assert!(
        matches!(next.kind, GraphQLTokenKind::Name(ref n) if n == "second"),
        "mutating consumed token should not affect stream",
    );
}

// =============================================================================
// Peek-then-consume pattern (used by parser's expect())
// =============================================================================

/// Verifies that the peek → drop reference → consume pattern works
/// correctly.
///
/// The parser's `expect()` method peeks to check the token kind,
/// drops the reference, then consumes to take ownership. This test
/// ensures no lifetime or aliasing issues arise from this pattern.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn peek_then_drop_then_consume_pattern() {
    let tokens = vec![
        utils::mock_name_token("type"),
        utils::mock_name_token("Query"),
        utils::mock_eof_token(),
    ];
    let mut stream = GraphQLTokenStream::new(
        utils::MockTokenSource::new(tokens),
    );

    // Peek to check the kind (simulating expect())
    let is_name = matches!(
        stream.peek().map(|t| &t.kind),
        Some(GraphQLTokenKind::Name(_)),
    );
    assert!(is_name);

    // Now consume (the reference from peek is already dropped)
    let owned = stream.consume().unwrap();
    assert!(
        matches!(owned.kind, GraphQLTokenKind::Name(ref n) if n == "type"),
    );

    // Stream should have advanced
    let next = stream.peek().unwrap();
    assert!(
        matches!(next.kind, GraphQLTokenKind::Name(ref n) if n == "Query"),
    );
}

// =============================================================================
// Full ownership collection
// =============================================================================

/// Verifies that all tokens can be consumed from a stream into a
/// `Vec`, giving the caller full ownership of every token.
///
/// After collection, the stream is empty and all tokens are
/// independently accessible in the Vec.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn consume_all_tokens_into_vec() {
    let expected_names = [
        "type", "Query", "field", "String",
    ];
    let tokens: Vec<_> = expected_names
        .iter()
        .map(|n| utils::mock_name_token(n))
        .chain(std::iter::once(utils::mock_eof_token()))
        .collect();

    let mut stream = GraphQLTokenStream::new(
        utils::MockTokenSource::new(tokens),
    );

    // Collect all tokens (including Eof)
    let mut collected = Vec::new();
    while let Some(token) = stream.consume() {
        collected.push(token);
    }

    // Should have 4 Name tokens + 1 Eof
    assert_eq!(collected.len(), 5);

    // Verify each Name token
    for (i, name) in expected_names.iter().enumerate() {
        assert!(
            matches!(
                collected[i].kind,
                GraphQLTokenKind::Name(ref n) if n == name
            ),
            "token {i} should be Name({name})",
        );
    }

    // Last should be Eof
    assert!(matches!(collected[4].kind, GraphQLTokenKind::Eof));

    // Stream should be empty
    assert!(stream.consume().is_none());
    assert_eq!(stream.current_buffer_len(), 0);
}

// =============================================================================
// VecDeque ring buffer behavior
// =============================================================================

/// Verifies that the `VecDeque` buffer stays naturally bounded
/// during alternating lookahead and consume cycles.
///
/// Unlike the old `Vec` + `current_index` approach that required
/// explicit `compact_buffer()` calls, the `VecDeque`'s
/// `pop_front()` naturally discards consumed tokens. The buffer
/// length should only reflect tokens buffered by lookahead, never
/// growing unboundedly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn vecdeque_buffer_bounded_during_lookahead_cycles() {
    let total_tokens = 600;
    let lookahead = 5;
    let tokens: Vec<_> = (0..total_tokens)
        .map(|i| utils::mock_name_token(&format!("t{i}")))
        .chain(std::iter::once(utils::mock_eof_token()))
        .collect();

    let mut stream = GraphQLTokenStream::new(
        utils::MockTokenSource::new(tokens),
    );

    let mut consumed_count = 0usize;
    loop {
        // Lookahead: peek at next `lookahead` tokens
        let peeked_count = (0..lookahead)
            .filter(|&n| stream.peek_nth(n).is_some())
            .count();

        if peeked_count == 0 {
            break;
        }

        // Buffer should have at most `lookahead` elements
        assert!(
            stream.current_buffer_len() <= lookahead,
            "buffer len {} exceeds lookahead {} at token {}",
            stream.current_buffer_len(),
            lookahead,
            consumed_count,
        );

        // Consume one token
        let token = stream.consume().unwrap();
        let expected = format!("t{consumed_count}");
        if consumed_count < total_tokens {
            assert!(
                matches!(
                    token.kind,
                    GraphQLTokenKind::Name(ref n) if n == &expected
                ),
                "expected {expected} at position {consumed_count}",
            );
        }
        consumed_count += 1;
    }

    assert_eq!(consumed_count, total_tokens + 1); // +1 for Eof
}
