//! A lossless [GraphQL parser](GraphQLParser) and parsing library for schema documents, executable
//! documents, and mixed schema + executable documents. Targets the
//! [September 2025 GraphQL Spec](https://spec.graphql.org/September2025/).
//!
//! # [`GraphQLParser`]
//!
//! ```rust
//! # use libgraphql_parser;
//! # use libgraphql_parser::parse_query;
//! # use libgraphql_parser::GraphQLParser;
//!
//! /****************************/
//! /** Convenience functions: **/
//! /****************************/
//!
//! // Parse any GraphQL document
//! let parse_result = libgraphql_parser::parse(r#"
//!   type User {
//!     firstName: String,
//!     lastName: String,
//!   }
//!
//!   type Query {
//!     me: User,
//!   }
//!
//!   query GetUserFullName {
//!     me {
//!       firstName,
//!       lastName,
//!     }
//!   }
//! "#);
//!
//! # // This example demonstrates parsing of a valid GraphQL document.
//! # parse_result?;
//!
//! // If any errors were encountered while parsing, print rust-style error message output.
//! if parse_result.has_errors() {
//!     eprintln!("Some parsing errors were encountered:\n{}", parse_result.format_errors());
//! }
//!
//! // Extract the AST and count the number of definitions that were parsed.
//! //
//! // Note that, if parsing errors occurred, a partial/error-recovered AST is generally still
//! // provided.
//! if let Some(ast) = parse_result.ast() {
//!   println!(
//!     "No parsing errors, found {} GraphQL definitions.",
//!     valid_ast.definitions.len(),
//!   );
//! }
//!
//! ```
//!
//! A GraphQL parser that accepts  that accepts  type-parame
//!
//! This crate provides a unified token-based parser infrastructure with support for multiple token
//! sources (string input, proc-macro input, etc.).

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
pub fn parse_executable(
    source: &str,
) -> ParseResult<ast::Document<'_>> {
    let parser = GraphQLParser::new(source);
    parser.parse_executable_document()
}

/// Parses a mixed document (both schema and executable
/// definitions) from a string.
pub fn parse(
    source: &str,
) -> ParseResult<ast::Document<'_>> {
    let parser = GraphQLParser::new(source);
    parser.parse_mixed_document()
}

#[cfg(test)]
mod tests;
