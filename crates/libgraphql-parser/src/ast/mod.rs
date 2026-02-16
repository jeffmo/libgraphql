//! Custom AST types for representing parsed GraphQL documents.
//!
//! This module provides a comprehensive, zero-copy AST for GraphQL
//! documents. All node types are parameterized over a `'src` lifetime
//! that borrows strings from the source text via [`Cow<'src, str>`].
//!
//! The AST has two conceptual layers:
//!
//! - **Semantic layer** (always present): Typed structs with names,
//!   values, directives, and all GraphQL semantics. Every node carries
//!   a [`GraphQLSourceSpan`] for source location tracking.
//!
//! - **Syntax layer** (optional): Each node has an
//!   `Option<XyzSyntax<'src>>` field that, when populated, contains
//!   keyword/punctuation tokens with their trivia (whitespace,
//!   comments, commas). This enables lossless source reconstruction
//!   for formatter and IDE tooling.
//!
//! # Example
//!
//! ```rust,ignore
//! use libgraphql_parser::GraphQLParser;
//!
//! let source = "type Query { hello: String }";
//! let parser = GraphQLParser::new(source);
//! let result = parser.parse_schema_document();
//! let doc = result.output;
//! ```
//!
//! [`Cow<'src, str>`]: std::borrow::Cow
//! [`GraphQLSourceSpan`]: crate::GraphQLSourceSpan

mod ast_node;

pub use ast_node::AstNode;
