use crate::GraphQLSourceSpan;

/// A "trivia token" is a token that doesn't affect parsing but is still
/// preserved (e.g. for tooling use).
///
/// Trivia includes comments and commas, which are attached to the following
/// token as "preceding trivia". This allows formatters and linters to preserve
/// these elements without the parser needing to handle them explicitly.
#[derive(Clone, Debug, PartialEq)]
pub enum GraphQLTriviaToken {
    /// A GraphQL comment, which starts with `#` and extends to the end of the
    /// line.
    Comment {
        /// The comment text (excluding the leading `#`).
        value: String,
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
