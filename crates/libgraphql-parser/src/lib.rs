//! A GraphQL parsing library to parse schema documents, executable documents,
//! and documents that mix both together.
//!
//! This crate provides a unified token-based parser infrastructure with
//! support for multiple token sources (string input, proc-macro input, etc.).

pub mod ast;
mod definition_kind;
mod document_kind;
mod graphql_error_note;
mod graphql_error_note_kind;
mod graphql_parse_error;
mod graphql_parse_error_kind;
mod graphql_parser;
mod graphql_source_span;
mod graphql_string_parsing_error;
mod graphql_token_stream;
pub mod legacy_ast;
mod parse_result;
mod reserved_name_context;
mod source_position;
pub mod token;
pub mod token_source;
mod value_parsing_error;

pub use definition_kind::DefinitionKind;
pub use document_kind::DocumentKind;
pub use graphql_error_note::GraphQLErrorNote;
pub use graphql_error_note::GraphQLErrorNotes;
pub use graphql_error_note_kind::GraphQLErrorNoteKind;
pub use graphql_parse_error::GraphQLParseError;
pub use graphql_parse_error_kind::GraphQLParseErrorKind;
pub use graphql_parser::GraphQLParser;
pub use graphql_source_span::GraphQLSourceSpan;
pub use graphql_string_parsing_error::GraphQLStringParsingError;
pub use graphql_token_stream::GraphQLTokenStream;
pub use parse_result::ParseResult;
pub use reserved_name_context::ReservedNameContext;
pub use smallvec::smallvec;
pub use smallvec::SmallVec;
pub use source_position::SourcePosition;
pub use value_parsing_error::ValueParsingError;

#[cfg(test)]
mod tests;
