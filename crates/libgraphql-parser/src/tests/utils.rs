//! Various test utils.
//!
//! Written by Claude Code, reviewed by a human.

use crate::ByteSpan;
use crate::GraphQLParser;
use crate::ParseResult;
use crate::SourceMap;
use crate::ast;
use crate::token::GraphQLToken;
use crate::token::GraphQLTokenKind;
use crate::token::GraphQLTokenSource;
use smallvec::smallvec;

/// Creates a mock token with the given kind and minimal span/trivia.
///
/// Uses `'static` lifetime since test tokens use owned strings.
pub fn mock_token(kind: GraphQLTokenKind<'static>) -> GraphQLToken<'static> {
    GraphQLToken {
        kind,
        preceding_trivia: smallvec![],
        span: ByteSpan::new(0, 0),
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
    source_map: SourceMap<'static>,
}

impl MockTokenSource {
    pub fn new(tokens: Vec<GraphQLToken<'static>>) -> Self {
        Self {
            tokens: tokens.into_iter(),
            source_map: SourceMap::empty(),
        }
    }
}

impl Iterator for MockTokenSource {
    type Item = GraphQLToken<'static>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.next()
    }
}

impl GraphQLTokenSource<'static> for MockTokenSource {
    fn source_map(&self) -> &SourceMap<'static> {
        &self.source_map
    }

    fn into_source_map(self) -> SourceMap<'static> {
        self.source_map
    }
}

/// Helper to parse a schema document and return errors if any.
pub(super) fn parse_schema(source: &str) -> ParseResult<'_, ast::Document<'_>> {
    GraphQLParser::new(source).parse_schema_document()
}

/// Helper to parse an executable document and return errors if any.
pub(super) fn parse_executable(
    source: &str,
) -> ParseResult<'_, ast::Document<'_>> {
    GraphQLParser::new(source).parse_executable_document()
}

/// Helper to parse a mixed document and return errors if any.
pub(super) fn parse_mixed(source: &str) -> ParseResult<'_, ast::Document<'_>> {
    GraphQLParser::new(source).parse_mixed_document()
}
