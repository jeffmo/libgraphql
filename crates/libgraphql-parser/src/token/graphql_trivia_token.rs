use crate::GraphQLSourceSpan;
use std::borrow::Cow;

/// A "trivia token" is a token that doesn't affect parsing but is still
/// preserved (e.g. for tooling use).
///
/// Trivia includes comments and commas, which are attached to the following
/// token as "preceding trivia". This allows formatters and linters to preserve
/// these elements without the parser needing to handle them explicitly.
///
/// # Lifetime Parameter
///
/// The `'src` lifetime enables zero-copy lexing for comment values:
/// `StrGraphQLTokenSource` can borrow comment text directly from the source.
#[derive(Clone, Debug, PartialEq)]
pub enum GraphQLTriviaToken<'src> {
    /// A GraphQL comment, which starts with `#` and extends to the end of the
    /// line.
    Comment {
        /// The comment text (excluding the leading `#`).
        /// Uses `Cow<'src, str>` to enable zero-copy lexing from string sources.
        value: Cow<'src, str>,
        /// The source location of the comment.
        span: GraphQLSourceSpan,
    },

    /// A comma separator. In GraphQL, commas are optional and treated as
    /// whitespace, but we preserve them as trivia.
    Comma {
        /// The source location of the comma.
        span: GraphQLSourceSpan,
    },
}
