use crate::rust_macro_graphql_token_source::RustMacroGraphQLTokenSource;
use libgraphql_core::ast;
use libgraphql_parser::GraphQLParser;
use libgraphql_parser::ParseResult;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

/// Helper to parse schema with error recovery.
///
/// Returns the full `ParseResult` so callers can inspect both
/// the (potentially partial) AST and the errors list. The
/// parser always attempts recovery, so the AST may be present
/// even when errors were encountered.
pub fn parse_with_recovery(
    schema: &str,
) -> ParseResult<ast::schema::Document> {
    let token_stream =
        proc_macro2::TokenStream::from_str(schema)
            .expect("Invalid Rust TokenStream in test input");

    let span_map = Rc::new(RefCell::new(HashMap::new()));
    let token_source =
        RustMacroGraphQLTokenSource::new(token_stream, span_map);
    let parser = GraphQLParser::new(token_source);
    parser.parse_schema_document()
}
