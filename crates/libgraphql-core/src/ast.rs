/// Re-exports from [`libgraphql_parser`] for AST types and parsing utilities.
///
/// This module provides a stable import surface for AST types from the
/// [`libgraphql_parser`] crate. All builder APIs in `libgraphql-core` accept
/// these types.
// Re-export all AST node types directly so that downstream code can write
// `crate::ast::Document`, `crate::ast::Definition`, etc.
pub use libgraphql_parser::ast::*;

// Span / position / source-map types used throughout builders and loc.rs
pub use libgraphql_parser::ByteSpan;
pub use libgraphql_parser::SourceMap;
pub use libgraphql_parser::SourcePosition;
pub use libgraphql_parser::SourceSpan;

// Parsing entry points
pub use libgraphql_parser::parse_executable;
pub use libgraphql_parser::parse_schema;

// Parse error and result types
pub use libgraphql_parser::GraphQLParseError;
pub use libgraphql_parser::ParseResult;
