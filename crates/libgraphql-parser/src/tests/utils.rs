//! Various test utils.
//!
//! Written by Claude Code, reviewed by a human.

use crate::ast;
use crate::token::GraphQLToken;
use crate::token::GraphQLTokenKind;
use crate::GraphQLParser;
use crate::GraphQLSourceSpan;
use crate::SourcePosition;
use smallvec::smallvec;

/// Creates a mock token with the given kind and minimal span/trivia.
///
/// Uses `'static` lifetime since test tokens use owned strings.
pub fn mock_token(kind: GraphQLTokenKind<'static>) -> GraphQLToken<'static> {
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
pub fn mock_name_token(name: &str) -> GraphQLToken<'static> {
    mock_token(GraphQLTokenKind::name_owned(name.to_string()))
}

/// Creates a mock Eof token.
pub fn mock_eof_token() -> GraphQLToken<'static> {
    mock_token(GraphQLTokenKind::Eof)
}

/// A mock token source that produces tokens from a Vec.
///
/// Uses `'static` lifetime since mock tokens use owned strings.
pub struct MockTokenSource {
    tokens: std::vec::IntoIter<GraphQLToken<'static>>,
}

impl MockTokenSource {
    pub fn new(tokens: Vec<GraphQLToken<'static>>) -> Self {
        Self {
            tokens: tokens.into_iter(),
        }
    }
}

impl Iterator for MockTokenSource {
    type Item = GraphQLToken<'static>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.next()
    }
}

/// Helper to parse a schema document and return errors if any.
pub(super) fn parse_schema(
    source: &str,
) -> crate::ParseResult<ast::schema::Document> {
    let parser = GraphQLParser::new(source);
    parser.parse_schema_document()
}

/// Helper to parse an executable document and return errors if any.
pub(super) fn parse_executable(
    source: &str,
) -> crate::ParseResult<ast::operation::Document> {
    let parser = GraphQLParser::new(source);
    parser.parse_executable_document()
}

/// Helper to parse a mixed document and return errors if any.
pub(super) fn parse_mixed(
    source: &str,
) -> crate::ParseResult<ast::MixedDocument> {
    let parser = GraphQLParser::new(source);
    parser.parse_mixed_document()
}
