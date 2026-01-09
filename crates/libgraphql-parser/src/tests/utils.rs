//! Various test utils.
//!
//! Written by Claude Code, reviewed by a human.

use crate::token::GraphQLToken;
use crate::token::GraphQLTokenKind;
use crate::GraphQLSourceSpan;
use crate::SourcePosition;
use smallvec::smallvec;

/// Creates a mock token with the given kind and minimal span/trivia.
pub fn mock_token(kind: GraphQLTokenKind) -> GraphQLToken {
    let pos = SourcePosition::new(0, 0, Some(0), 0);
    GraphQLToken {
        kind,
        preceding_trivia: smallvec![],
        span: GraphQLSourceSpan {
            start_inclusive: pos.clone(),
            end_exclusive: pos,
            file_path: None,
        },
    }
}

/// Creates a mock Name token with the given name.
pub fn mock_name_token(name: &str) -> GraphQLToken {
    mock_token(GraphQLTokenKind::Name(name.to_string()))
}

/// Creates a mock Eof token.
pub fn mock_eof_token() -> GraphQLToken {
    mock_token(GraphQLTokenKind::Eof)
}

/// A mock token source that produces tokens from a Vec.
pub struct MockTokenSource {
    tokens: std::vec::IntoIter<GraphQLToken>,
}

impl MockTokenSource {
    pub fn new(tokens: Vec<GraphQLToken>) -> Self {
        Self {
            tokens: tokens.into_iter(),
        }
    }
}

impl Iterator for MockTokenSource {
    type Item = GraphQLToken;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.next()
    }
}
