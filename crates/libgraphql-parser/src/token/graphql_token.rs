use crate::token::GraphQLTokenKind;
use crate::token::GraphQLTriviaToken;
use crate::GraphQLSourceSpan;
use smallvec::SmallVec;

/// Type alias for trivia storage. Uses SmallVec to avoid heap allocation
/// for the common case of 0-2 trivia items per token.
///
/// The `'src` lifetime matches the source text lifetime from the token source.
pub type GraphQLTriviaTokenVec<'src> = SmallVec<[GraphQLTriviaToken<'src>; 2]>;

/// A GraphQL token with location (span) information and an ordered list of any
/// preceding trivia (comments, commas).
///
/// Trivia is attached to the *following* token, so parsers can simply
/// call `peek()` and `consume()` without worrying about skipping trivia.
///
/// # Lifetime Parameter
///
/// The `'src` lifetime represents the source text that this token was lexed
/// from. For `StrGraphQLTokenSource`, this enables zero-copy lexing where
/// token values borrow directly from the input string. For
/// `RustMacroGraphQLTokenSource`, tokens use owned strings and the lifetime
/// can be `'static`.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphQLToken<'src> {
    /// The kind of token (including Error for lexer errors).
    pub kind: GraphQLTokenKind<'src>,

    /// Trivia (comments, commas) that precede this token.
    pub preceding_trivia: GraphQLTriviaTokenVec<'src>,

    /// The source location span of this token.
    pub span: GraphQLSourceSpan,
}

impl<'src> GraphQLToken<'src> {
    /// Convenience constructor for a token with no preceding trivia.
    pub fn new(kind: GraphQLTokenKind<'src>, span: GraphQLSourceSpan) -> Self {
        Self {
            kind,
            preceding_trivia: SmallVec::new(),
            span,
        }
    }
}
