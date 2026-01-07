use crate::token::GraphQLTokenKind;
use crate::token::GraphQLTokenSpan;
use crate::token::GraphQLTriviaToken;
use smallvec::SmallVec;

/// Type alias for trivia storage. Uses SmallVec to avoid heap allocation
/// for the common case of 0-2 trivia items per token.
pub type GraphQLTriviaTokenVec = SmallVec<[GraphQLTriviaToken; 2]>;

/// A GraphQL token with location (span) information and an ordered list of any
/// preceding trivia (comments, commas).
///
/// Trivia is attached to the *following* token, so parsers can simply
/// call `peek()` and `consume()` without worrying about skipping trivia.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphQLToken {
    /// The kind of token (including Error for lexer errors).
    pub kind: GraphQLTokenKind,

    /// Trivia (comments, commas) that precede this token.
    pub preceding_trivia: GraphQLTriviaTokenVec,

    /// The source location span of this token.
    pub span: GraphQLTokenSpan,
}

impl GraphQLToken {
    /// Convenience constructor for a token with no preceding trivia.
    pub fn new(kind: GraphQLTokenKind, span: GraphQLTokenSpan) -> Self {
        Self {
            kind,
            preceding_trivia: SmallVec::new(),
            span,
        }
    }
}
