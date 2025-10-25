use crate::rust_to_graphql_token_adapter::{GraphQLToken, RustToGraphQLTokenAdapter};
use proc_macro2::Span;
use std::collections::VecDeque;

/// Streaming token parser with bounded lookahead buffer.
///
/// This structure wraps a `RustToGraphQLTokenAdapter` and provides lookahead
/// capabilities while maintaining efficient streaming behavior. It only buffers
/// tokens as needed for lookahead operations, not the entire token stream.
///
/// Memory usage is O(lookahead_distance) instead of O(total_tokens), making it
/// suitable for parsing large GraphQL schemas without excessive memory consumption.
pub struct GraphQLTokenStream {
    adapter: RustToGraphQLTokenAdapter,
    buffer: VecDeque<(GraphQLToken, Span)>,
    current_span: Option<Span>,
}

impl GraphQLTokenStream {
    /// Creates a new token stream from an adapter
    pub fn new(adapter: RustToGraphQLTokenAdapter) -> Self {
        Self {
            adapter,
            buffer: VecDeque::new(),
            current_span: None,
        }
    }

    /// Peek at the next token without consuming it
    ///
    /// This method fills the buffer if needed. Returns `None` if at end of stream.
    pub fn peek(&mut self) -> Option<&(GraphQLToken, Span)> {
        if self.buffer.is_empty()
            && let Some(token) = self.adapter.next()
        {
            self.buffer.push_back(token);
        }
        self.buffer.front()
    }

    /// Peek at the nth token ahead (0-indexed, where 0 = peek())
    ///
    /// This method fills the buffer up to n+1 elements if needed.
    /// Returns `None` if the stream ends before reaching position n.
    pub fn peek_nth(&mut self, n: usize) -> Option<&(GraphQLToken, Span)> {
        // Fill buffer up to n+1 elements if needed
        while self.buffer.len() <= n {
            if let Some(token) = self.adapter.next() {
                self.buffer.push_back(token);
            } else {
                break;
            }
        }
        self.buffer.get(n)
    }

    /// Consume and return the next token
    ///
    /// Returns `None` if at end of stream.
    pub fn next(&mut self) -> Option<(GraphQLToken, Span)> {
        let token = if let Some(token) = self.buffer.pop_front() {
            token
        } else {
            self.adapter.next()?
        };
        self.current_span = Some(token.1);
        Some(token)
    }

    /// Get the span of the most recently consumed token
    ///
    /// If no token has been consumed yet, returns `Span::call_site()`.
    pub fn current_span(&self) -> Span {
        self.current_span.unwrap_or_else(Span::call_site)
    }

    /// Check if next token matches the expected token without consuming
    ///
    /// Uses `PartialEq` to compare tokens. Returns `false` if at end of stream.
    pub fn check(&mut self, expected: &GraphQLToken) -> bool {
        self.peek()
            .map(|(tok, _)| tok == expected)
            .unwrap_or(false)
    }

    /// Check if next token is a Name with specific value
    ///
    /// Returns `false` if the next token is not a Name or if at end of stream.
    pub fn check_name(&mut self, name: &str) -> bool {
        matches!(
            self.peek(),
            Some((GraphQLToken::Name(n), _)) if n == name
        )
    }

    /// Check if next token is a Punctuator with specific value
    ///
    /// Returns `false` if the next token is not a Punctuator or if at end of stream.
    pub fn check_punctuator(&mut self, punct: &str) -> bool {
        matches!(
            self.peek(),
            Some((GraphQLToken::Punctuator(p), _)) if p == punct
        )
    }

    /// Check if we've reached the end of the stream
    ///
    /// Returns `true` if there are no more tokens to consume.
    pub fn is_at_end(&mut self) -> bool {
        self.peek().is_none()
    }
}
