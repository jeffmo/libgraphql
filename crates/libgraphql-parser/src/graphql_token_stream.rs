//! Streaming lexer that produces [`GraphQLToken`]s given some
//! [`GraphQLTokenSource`] with a bounded lookahead buffer.

use crate::token::GraphQLToken;
use crate::token::GraphQLTokenKind;
use crate::token_source::GraphQLTokenSource;

/// Streaming lexer that produces [`GraphQLToken`]s given some
/// [`GraphQLTokenSource`] with a bounded lookahead buffer.
///
/// This structure accepts any [`GraphQLTokenSource`] and provides lookahead
/// capabilities while maintaining efficient streaming behavior. It centralizes
/// buffering, peeking, and lookahead logic.
///
/// Since trivia is already attached to tokens by the lexer, the parser can
/// simply call `peek()` and `consume()` without worrying about trivia.
///
/// # Internal Buffer Management
///
/// Tokens are stored in an internal buffer. The `current_index` points to the
/// most recently consumed token. Periodically, consumed tokens are compacted
/// (removed from the front of the internal buffer) to prevent unbounded growth.
///
/// `compact_buffer()` should be called whenever there may be unreferenceable
/// tokens in the internal buffer (i.e., tokens before `current_index` that will
/// never be accessed again). Typically this is after successfully parsing a
/// complete top-level definition.
///
/// # Type Parameters
///
/// * `'src` - The lifetime of the source text that tokens are lexed from.
/// * `TTokenSource` - The underlying token source, which must implement
///   [`GraphQLTokenSource`] (i.e., `Iterator<Item = GraphQLToken>`).
///
/// # Future TODOs
///
/// - Consider adding a `GraphQLTokenStreamOptions` struct to configure
///   behavior:
///   - `include_trivia: bool` - Whether to include preceding_trivia in tokens
///     (can be disabled for performance when trivia is not needed)
///   - `max_tokens: Option<usize>` - Limit total tokens returned (DoS
///     protection)
/// - Investigate whether auto-compaction (calling `compact_buffer()`
///   automatically after each `consume()`) hurts performance in any meaningful
///   way. If not, consider making `compact_buffer()` private and compacting
///   automatically.
pub struct GraphQLTokenStream<'src, TTokenSource: GraphQLTokenSource<'src>> {
    token_source: TTokenSource,
    /// Internal buffer of tokens. Grows as needed for lookahead.
    buffer: Vec<GraphQLToken<'src>>,
    /// Index of the current (most recently consumed) token in the internal
    /// buffer. `None` if no token has been consumed yet.
    current_index: Option<usize>,
}

impl<'src, TTokenSource: GraphQLTokenSource<'src>> GraphQLTokenStream<'src, TTokenSource> {
    /// Compact the internal buffer by removing tokens before `current_index`.
    ///
    /// Call this after parsing each top-level definition to prevent unbounded
    /// internal buffer growth. Should be called whenever there may be
    /// unreferenceable tokens in the internal buffer.
    pub fn compact_buffer(&mut self) {
        if let Some(idx) = self.current_index && idx > 0 {
            self.buffer.drain(0..idx);
            self.current_index = Some(0);
        } else if self.current_index.is_none() {
            // When the entire `GraphQLTokenSource` iterator is consumed,
            // `consume()` sets `current_index` back to `None` and there are
            // typically still a few [now-unaccessible] tokens in the buffer.
            self.buffer.clear();
        }
        // Note: we intentionally do NOT call
        // `self.buffer.shrink_to_fit()` here.
        //
        // Performance (B7 in benchmark-optimizations.md):
        // compact_buffer() is called after each top-level
        // definition, so for a 1000-type schema that's 1000
        // calls. shrink_to_fit() may trigger a realloc to
        // shrink the Vec, only for the next definition to
        // grow it again — creating a "sawtooth" alloc pattern.
        // Retaining capacity avoids this realloc churn at the
        // cost of a few KB of retained heap memory.
    }

    /// Advance to the next token and return a reference to it.
    ///
    /// The token is retained in the internal buffer for access via
    /// `current_token()`. Returns `None` if the stream is exhausted.
    pub fn consume(&mut self) -> Option<&GraphQLToken<'src>> {
        let next_index = match self.current_index {
            Some(idx) => idx + 1,
            None => 0,
        };
        self.ensure_buffer_has(next_index + 1);
        if next_index < self.buffer.len() {
            self.current_index = Some(next_index);
            Some(&self.buffer[next_index])
        } else {
            self.current_index = None;
            None
        }
    }

    /// Returns the number of [`GraphQLToken`]s currently stored in the buffer.
    pub fn current_buffer_len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns the most recently consumed token.
    ///
    /// Returns `None` if no token has been consumed yet.
    pub fn current_token(&self) -> Option<&GraphQLToken<'src>> {
        self.current_index.map(|i| &self.buffer[i])
    }

    /// Fill the internal buffer to ensure it has at least `count` elements.
    fn ensure_buffer_has(&mut self, count: usize) {
        while self.buffer.len() < count {
            if let Some(token) = self.token_source.next() {
                self.buffer.push(token);
            } else {
                break;
            }
        }
    }

    /// Check if we've reached the end of the stream.
    ///
    /// Returns `true` if there are no more tokens to consume, or if the next
    /// token is `Eof`.
    pub fn is_at_end(&mut self) -> bool {
        match self.peek() {
            None => true,
            Some(token) => matches!(token.kind, GraphQLTokenKind::Eof),
        }
    }

    /// Creates a new token stream from a token source.
    pub fn new(token_source: TTokenSource) -> Self {
        Self {
            token_source,
            buffer: Vec::new(),
            current_index: None,
        }
    }

    /// Peek at the next token without consuming it.
    ///
    /// Returns the token at `current_index + 1` (or index 0 if nothing has been
    /// consumed yet). Returns `None` if the stream is exhausted.
    #[inline]
    pub fn peek(&mut self) -> Option<&GraphQLToken<'src>> {
        self.peek_nth(0)
    }

    /// Peek at the nth token ahead (0-indexed from next unconsumed token).
    ///
    /// `peek_nth(0)` is equivalent to `peek()`.
    ///
    /// This method fills the internal buffer up to `n+1` elements beyond the
    /// current position if needed. Returns `None` if the stream ends before
    /// reaching position n.
    pub fn peek_nth(&mut self, n: usize) -> Option<&GraphQLToken<'src>> {
        let target_index = match self.current_index {
            Some(idx) => idx + 1 + n,
            None => n,
        };
        self.ensure_buffer_has(target_index + 1);
        self.buffer.get(target_index)
    }
}
