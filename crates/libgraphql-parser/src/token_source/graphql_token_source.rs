use crate::SourceMap;
use crate::token::GraphQLToken;

/// Trait for [`GraphQLToken`] lexers (iterators that generate
/// [`GraphQLToken`]).
///
/// This trait enables extensibility over different sources of GraphQL text to
/// be parsed. For example:
/// [`StrGraphQLTokenSource`](crate::token_source::StrGraphQLTokenSource) is a
/// lexer over `&str` types,
/// [`libgraphql_macros::RustMacroGraphQLTokenSource`](https://github.com/jeffmo/libgraphql/blob/59aa00fe928249c9d7abcd2576e2e37e45345955/crates/libgraphql-macros/src/rust_macro_graphql_token_source.rs#L101)
/// is a lexer over
/// [`proc_macro2::Span`](https://docs.rs/proc-macro2/latest/proc_macro2/struct.Span.html)
/// (for lexing Rust procedural macro input), etc.
///
/// Implementors define an [`Iterator`] that produces tokens one at a time.
/// All lookahead, buffering, and peeking is handled by `GraphQLTokenStream`.
///
/// Lexers are responsible for:
/// - Skipping whitespace (an "ignored token" per the GraphQL spec)
/// - Accumulating trivia (comments, commas) and attaching to the next token
/// - Emitting [`GraphQLTokenKind::Error`](crate::token::GraphQLTokenKind::Error)
///   for lexer errors (enables error recovery)
/// - Emitting a final token with
///   [`GraphQLTokenKind::Eof`](crate::token::GraphQLTokenKind::Eof) carrying
///   any trailing trivia
///
/// # Lifetime Parameter
///
/// The `'src` lifetime represents the source text that tokens are lexed from.
/// For string-based lexers, this enables zero-copy lexing where token values
/// can borrow directly from the input. For proc-macro lexers that must
/// allocate strings, use `'static` as the lifetime.
///
/// # SourceMap
///
/// Each token source carries a [`SourceMap`] that maps byte offsets (stored
/// compactly in [`ByteSpan`](crate::ByteSpan)) to resolved line/column
/// positions. The `source_map()` method borrows it for mid-stream lookups
/// (e.g. IDE hover), and `into_source_map()` transfers ownership to
/// `ParseResult` after parsing completes.
pub trait GraphQLTokenSource<'src>: Iterator<Item = GraphQLToken<'src>> {
    /// Borrows the [`SourceMap`] for resolving byte offsets to
    /// line/column positions while the token source is still active.
    fn source_map(&self) -> &SourceMap<'src>;

    /// Consumes the token source and returns the owned [`SourceMap`].
    ///
    /// Called by the parser after consuming all tokens (EOF) so the
    /// `SourceMap` can be bundled into `ParseResult`.
    fn into_source_map(self) -> SourceMap<'src>;

    /// Collects all tokens and returns them alongside the [`SourceMap`].
    ///
    /// This is the preferred way to consume a token source when you
    /// need both the tokens and position-resolution capability — e.g.
    /// in tests or IDE tooling that operates on a full token stream.
    ///
    /// Using `Iterator::collect()` alone would consume `self`, making
    /// `into_source_map()` unreachable. This method avoids that by
    /// collecting via `by_ref()` and then extracting the source map.
    fn collect_with_source_map(mut self) -> (Vec<GraphQLToken<'src>>, SourceMap<'src>)
    where Self: Sized {
        let tokens: Vec<_> = self.by_ref().collect();
        let source_map = self.into_source_map();
        (tokens, source_map)
    }
}
