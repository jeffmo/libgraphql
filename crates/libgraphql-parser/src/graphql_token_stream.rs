//! Streaming lexer that produces [`GraphQLToken`]s given some
//! [`GraphQLTokenSource`] with a bounded lookahead buffer.

use std::collections::VecDeque;

use crate::token::GraphQLToken;
use crate::token::GraphQLTokenKind;
use crate::token_source::GraphQLTokenSource;

/// Streaming lexer that produces [`GraphQLToken`]s given some
/// [`GraphQLTokenSource`] with a bounded lookahead buffer.
///
/// This structure accepts any [`GraphQLTokenSource`] and provides
/// lookahead capabilities while maintaining efficient streaming
/// behavior. It centralizes buffering, peeking, and lookahead logic.
///
/// Since trivia is already attached to tokens by the lexer, the
/// parser can simply call `peek()` and `consume()` without worrying
/// about trivia.
///
/// # Internal Buffer Management
///
/// Tokens are stored in a [`VecDeque`] ring buffer. Unconsumed
/// tokens are buffered at the back; `consume()` pops from the front
/// and returns the owned token via O(1) `pop_front()`.
///
/// # Type Parameters
///
/// * `'src` - The lifetime of the source text that tokens are lexed
///   from.
/// * `TTokenSource` - The underlying token source, which must
///   implement [`GraphQLTokenSource`] (i.e.,
///   `Iterator<Item = GraphQLToken>`).
///
/// # Future TODOs
///
/// - Consider adding a `GraphQLTokenStreamOptions` struct to
///   configure behavior:
///   - `include_trivia: bool` - Whether to include
///     preceding_trivia in tokens (can be disabled for performance
///     when trivia is not needed)
///   - `max_tokens: Option<usize>` - Limit total tokens returned
///     (DoS protection)
pub struct GraphQLTokenStream<
    'src,
    TTokenSource: GraphQLTokenSource<'src>,
> {
    token_source: TTokenSource,
    /// Ring buffer of unconsumed tokens. Grows at the back via
    /// `ensure_buffer_has()`; consumed from the front via
    /// `pop_front()`.
    buffer: VecDeque<GraphQLToken<'src>>,
}

impl<'src, TTokenSource: GraphQLTokenSource<'src>>
    GraphQLTokenStream<'src, TTokenSource>
{
    /// Advance to the next token and return it as an owned value.
    ///
    /// Returns `None` if the stream is exhausted.
    pub fn consume(&mut self) -> Option<GraphQLToken<'src>> {
        self.ensure_buffer_has(1);
        self.buffer.pop_front()
    }

    /// Returns the number of [`GraphQLToken`]s currently buffered
    /// (unconsumed).
    pub fn current_buffer_len(&self) -> usize {
        self.buffer.len()
    }

    /// Fill the buffer to ensure it has at least `count`
    /// unconsumed elements.
    fn ensure_buffer_has(&mut self, count: usize) {
        while self.buffer.len() < count {
            if let Some(token) = self.token_source.next() {
                self.buffer.push_back(token);
            } else {
                break;
            }
        }
    }

    /// Check if we've reached the end of the stream.
    ///
    /// Returns `true` if there are no more tokens to consume, or
    /// if the next token is `Eof`.
    pub fn is_at_end(&mut self) -> bool {
        match self.peek() {
            None => true,
            Some(token) => {
                matches!(token.kind, GraphQLTokenKind::Eof)
            },
        }
    }

    /// Creates a new token stream from a token source.
    pub fn new(token_source: TTokenSource) -> Self {
        Self {
            token_source,
            buffer: VecDeque::new(),
        }
    }

    /// Peek at the next token without consuming it.
    ///
    /// Returns the front of the buffer (filling it first if
    /// empty). Returns `None` if the stream is exhausted.
    #[inline]
    pub fn peek(&mut self) -> Option<&GraphQLToken<'src>> {
        self.peek_nth(0)
    }

    /// Peek at the nth token ahead (0-indexed from next unconsumed
    /// token).
    ///
    /// `peek_nth(0)` is equivalent to `peek()`.
    ///
    /// Fills the buffer up to `n+1` elements if needed. Returns
    /// `None` if the stream ends before reaching position n.
    pub fn peek_nth(
        &mut self,
        n: usize,
    ) -> Option<&GraphQLToken<'src>> {
        self.ensure_buffer_has(n + 1);
        self.buffer.get(n)
    }
}
