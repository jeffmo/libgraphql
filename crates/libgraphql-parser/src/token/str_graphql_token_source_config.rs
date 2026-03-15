/// Configuration for [`StrGraphQLTokenSource`](crate::token::StrGraphQLTokenSource)
/// controlling which trivia types are captured during lexing.
///
/// All flags default to `true`, meaning the lexer operates in full-fidelity
/// mode by default — every comment, comma, and whitespace run is preserved
/// as trivia on the emitted tokens. Set individual flags to `false` to
/// discard specific trivia types for leaner output.
///
/// # Example
///
/// ```rust
/// use libgraphql_parser::token::StrGraphQLTokenSourceConfig;
///
/// // Full-fidelity (default): all trivia preserved
/// let full = StrGraphQLTokenSourceConfig::default();
/// assert!(full.retain_comments);
/// assert!(full.retain_commas);
/// assert!(full.retain_whitespace);
///
/// // Lean mode: discard all trivia
/// let lean = StrGraphQLTokenSourceConfig::no_trivia();
/// assert!(!lean.retain_comments);
/// assert!(!lean.retain_commas);
/// assert!(!lean.retain_whitespace);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct StrGraphQLTokenSourceConfig {
    /// Whether to preserve `#` comments as
    /// [`GraphQLTriviaToken::Comment`](crate::token::GraphQLTriviaToken::Comment)
    /// on emitted tokens.
    ///
    /// When `false`, comments are still consumed (skipped) by the lexer but
    /// not recorded as trivia.
    pub retain_comments: bool,

    /// Whether to preserve commas as
    /// [`GraphQLTriviaToken::Comma`](crate::token::GraphQLTriviaToken::Comma)
    /// on emitted tokens.
    ///
    /// When `false`, commas are still consumed (skipped) by the lexer but
    /// not recorded as trivia.
    pub retain_commas: bool,

    /// Whether to preserve whitespace runs as
    /// [`GraphQLTriviaToken::Whitespace`](crate::token::GraphQLTriviaToken::Whitespace)
    /// on emitted tokens.
    ///
    /// When `false`, whitespace is still consumed (skipped) by the lexer but
    /// not recorded as trivia. This is the most impactful flag for
    /// performance since whitespace runs are frequent.
    pub retain_whitespace: bool,
}

impl StrGraphQLTokenSourceConfig {
    /// Returns a config that discards all trivia (comments, commas,
    /// whitespace). Useful when only semantic tokens are needed and
    /// source reconstruction is not required.
    pub fn no_trivia() -> Self {
        Self {
            retain_comments: false,
            retain_commas: false,
            retain_whitespace: false,
        }
    }
}

impl Default for StrGraphQLTokenSourceConfig {
    fn default() -> Self {
        Self {
            retain_comments: true,
            retain_commas: true,
            retain_whitespace: true,
        }
    }
}
