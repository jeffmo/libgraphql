use crate::token::GraphQLToken;

/// Marker trait for [`GraphQLToken`] lexers (iterators that generate
/// [`GraphQLToken`]).
///
/// This trait enables extensibility over different sources of GraphQL text to
/// be parsed. For example: [`StrToGraphQLTokenSource`] is a lexer over `&str`
/// types, [`libgraphql_macros::RustMacroGraphQLTokenSource`] is a lexer over
/// [`proc_macro2::Span`] (for lexing Rust procedural macro input), etc.
///
/// Implementors define an [`Iterator`] that produces tokens one at a time.
/// All lookahead, buffering, and peeking is handled by `GraphQLTokenStream`.
///
/// Lexers are responsible for:
/// - Skipping whitespace (an "ignored token" per the GraphQL spec)
/// - Accumulating trivia (comments, commas) and attaching to the next token
/// - Emitting [`GraphQLTokenKind::Error`] for lexer errors (enables error
///   recovery)
/// - Emitting a final token with [`GraphQLTokenKind::Eof`] carrying any trailing
///   trivia
pub trait GraphQLTokenSource: Iterator<Item = GraphQLToken> {}

impl<T> GraphQLTokenSource for T where T: Iterator<Item = GraphQLToken> {}
