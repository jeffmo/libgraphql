//! A GraphQL parsing library to parse schema documents, executable documents,
//! and documents that mix both together.
//!
//! This crate provides a unified token-based parser infrastructure with
//! support for multiple token sources (string input, proc-macro input, etc.).

pub mod ast;
pub mod compat_graphql_parser_v0_4;
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

/// Parses a schema document from a string.
pub fn parse_schema(
    source: &str,
) -> ParseResult<ast::Document<'_>> {
    let parser = GraphQLParser::new(source);
    parser.parse_schema_document()
}

/// Parses an executable document (operations and
/// fragments) from a string.
pub fn parse_query(
    source: &str,
) -> ParseResult<ast::Document<'_>> {
    let parser = GraphQLParser::new(source);
    parser.parse_executable_document()
}

/// Parses a mixed document (both schema and executable
/// definitions) from a string.
pub fn parse_mixed(
    source: &str,
) -> ParseResult<ast::Document<'_>> {
    let parser = GraphQLParser::new(source);
    parser.parse_mixed_document()
}

#[cfg(test)]
mod tests;
