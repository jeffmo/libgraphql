//! `libgraphql-parser` provides a lossless, error-tolerant, and
//! highly-optimized
//! [GraphQL tokenizer](crate::token::StrGraphQLTokenSource) and
//! [GraphQL parser](GraphQLParser). Capable of parsing schema documents,
//! executable documents, and mixed schema + executable documents.
//!
//! By default, `libgraphql-parser` targets the
//! [September 2025 GraphQL Spec](https://spec.graphql.org/September2025/).
//!
//! ## Usage
//!
//! ##### Parse valid documents
//!
//! ```rust
//! # use libgraphql_parser;
//! # use libgraphql_parser::GraphQLParser;
//! # use libgraphql_parser::ParseResult;
//! use libgraphql_parser::ast;
//!
//! // Parse any GraphQL document
//! let parse_result = libgraphql_parser::parse(r#"
//!   type User { firstName: String, lastName: String }
//!   type Query { me: User }
//!
//!   query GetUserFullName {
//!     me {
//!       firstName,
//!       lastName,
//!     }
//!   }
//! "#);
//! # assert!(parse_result.valid().is_some());
//!
//! // Count and print the number of top-level definitions parsed out of the
//! // GraphQL document.
//! let ast: &ast::Document<'_> = parse_result.ast();
//! println!("Parsed {} GraphQL definitions.", ast.definitions.len());
//! ```
//!
//! ##### Parse documents with errors
//!
//! ```rust
//! # use libgraphql_parser;
//! // Parse GraphQL documents with errors
//! let parse_result = libgraphql_parser::parse(r#"
//!   type User { firstName String }
//!   type Query { me: User }
//! "#);
//! # assert!(!parse_result.errors().is_empty(), "Expected a 'missing : token' error");
//!
//! // Access an "error recovered" version of the AST -- best-effort parsing.
//! let (recovered_ast, parse_errors, _) = parse_result.recovered().unwrap();
//! # assert_eq!(recovered_ast.definitions.len(), 1, "Expected 1 recovered definition");
//! # assert_eq!(parse_errors.len(), 1, "Expected 1 parse error");
//! // Print nicely-formatted output for all parse errors
//! eprintln!(
//!   "Found {} errors while parsing:\n{}",
//!   parse_errors.len(),
//!   parse_result.formatted_errors(),
//! );
//!
//! println!(
//!   "Found {} definitions after best-effort parse error recovery.",
//!   recovered_ast.definitions.len(),
//! );
//! ```
//!
//! This crate provides a unified token-based parser infrastructure with support for multiple token
//! sources (string input, proc-macro input, etc.).

pub mod ast;
mod byte_span;
pub mod compat;
mod graphql_error_note;
mod graphql_error_note_kind;
mod graphql_parse_error;
mod graphql_parse_error_kind;
mod graphql_parser;
mod graphql_parser_config;
mod source_span;
mod graphql_string_parsing_error;
mod graphql_token_stream;
mod parse_result;
mod reserved_name_context;
pub mod smallvec;
mod source_map;
mod source_position;
pub mod token;
mod value_parsing_error;

pub use byte_span::ByteSpan;
pub use graphql_error_note::GraphQLErrorNote;
pub use graphql_error_note_kind::GraphQLErrorNoteKind;
pub use graphql_parse_error::GraphQLParseError;
pub use graphql_parse_error_kind::GraphQLParseErrorKind;
pub use graphql_parser::GraphQLParser;
pub use graphql_parser_config::GraphQLParserConfig;
pub use source_span::SourceSpan;
pub use graphql_string_parsing_error::GraphQLStringParsingError;
pub use graphql_token_stream::GraphQLTokenStream;
pub use parse_result::ParseResult;
pub use reserved_name_context::ReservedNameContext;
pub use source_map::SourceMap;
pub use source_position::SourcePosition;
pub use value_parsing_error::ValueParsingError;

/// Parses a schema document from a string.
pub fn parse_schema(
    source: &str,
) -> ParseResult<'_, ast::Document<'_>> {
    let parser = GraphQLParser::new(source);
    parser.parse_schema_document()
}

/// Parses an executable document (operations and
/// fragments) from a string.
pub fn parse_executable(
    source: &str,
) -> ParseResult<'_, ast::Document<'_>> {
    let parser = GraphQLParser::new(source);
    parser.parse_executable_document()
}

/// Parses a mixed document (both schema and executable
/// definitions) from a string.
pub fn parse(
    source: &str,
) -> ParseResult<'_, ast::Document<'_>> {
    let parser = GraphQLParser::new(source);
    parser.parse_mixed_document()
}

#[cfg(test)]
mod tests;

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct _ReadmeDocTests;
