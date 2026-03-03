use crate::GraphQLSourceSpan;
use std::borrow::Cow;

/// A "trivia token" is a token that doesn't affect parsing but is still
/// preserved (e.g. for tooling use).
///
/// Trivia includes comments, commas, and whitespace, which are attached to
/// the following token as "preceding trivia". This allows formatters and
/// linters to preserve these elements without the parser needing to handle
/// them explicitly.
///
/// # Lifetime Parameter
///
/// The `'src` lifetime enables zero-copy lexing for comment and whitespace
/// values: `StrGraphQLTokenSource` can borrow text directly from the source.
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

    /// A run of whitespace characters (spaces, tabs, newlines, BOM).
    ///
    /// In GraphQL, whitespace is insignificant (it doesn't affect parsing),
    /// but preserving it as trivia enables lossless source reconstruction.
    /// Each `Whitespace` trivia token captures a contiguous run of whitespace
    /// exactly as it appears in the source.
    Whitespace {
        /// The whitespace text (spaces, tabs, newlines, BOM).
        /// Uses `Cow<'src, str>` to enable zero-copy lexing from string sources.
        value: Cow<'src, str>,
        /// The source location of the whitespace run.
        span: GraphQLSourceSpan,
    },
}
