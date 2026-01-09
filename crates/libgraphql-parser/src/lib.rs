//! A GraphQL parsing library to parse schema documents, executable documents,
//! and documents that mix both together.
//!
//! This crate provides a unified token-based parser infrastructure with
//! support for multiple token sources (string input, proc-macro input, etc.).

pub mod ast;
mod graphql_error_notes;
mod graphql_source_span;
mod graphql_string_parsing_error;
mod graphql_token_stream;
mod source_position;
pub mod token;
pub mod token_source;

pub use graphql_string_parsing_error::GraphQLStringParsingError;
pub use graphql_error_notes::GraphQLErrorNotes;
pub use graphql_source_span::GraphQLSourceSpan;
pub use graphql_token_stream::GraphQLTokenStream;
pub use smallvec::smallvec;
pub use smallvec::SmallVec;
pub use source_position::SourcePosition;

#[cfg(test)]
mod tests;
