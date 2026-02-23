use crate::token::GraphQLToken;

/// Marker trait for [`GraphQLToken`] lexers (iterators that generate
/// [`GraphQLToken`]).
///
/// This trait enables extensibility over different sources of GraphQL text to
/// be parsed. For example:
/// [`StrGraphQLTokenSource`](crate::token_source::StrGraphQLTokenSource) is a lexer over `&str`
/// types,
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
/// - Emitting [`GraphQLTokenKind::Error`](crate::token::GraphQLTokenKind::Error) for lexer errors
///   (enables error recovery)
/// - Emitting a final token with [`GraphQLTokenKind::Eof`](crate::token::GraphQLTokenKind::Eof)
///   carrying any trailing trivia
///
/// # Lifetime Parameter
///
/// The `'src` lifetime represents the source text that tokens are lexed from.
/// For string-based lexers, this enables zero-copy lexing where token values
/// can borrow directly from the input. For proc-macro lexers that must allocate
/// strings, use `'static` as the lifetime.
pub trait GraphQLTokenSource<'src>: Iterator<Item = GraphQLToken<'src>> {}

impl<'src, T> GraphQLTokenSource<'src> for T where T: Iterator<Item = GraphQLToken<'src>> {}
