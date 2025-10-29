use crate::graphql_parse_error::GraphQLParseErrors;
use crate::graphql_schema_parser_v2::GraphQLSchemaParser;
use crate::rust_to_graphql_token_adapter::RustToGraphQLTokenAdapter;
use libgraphql_core::ast;
use proc_macro2::Span;
use std::str::FromStr;

/// Helper to parse schema with error recovery
///
/// Returns both the parse result (Ok or Err) to allow testing whether
/// the parser recovered enough to produce a partial document or just errors.
pub fn parse_with_recovery(
    schema: &str,
) -> Result<ast::schema::Document, GraphQLParseErrors> {
    let token_stream = match proc_macro2::TokenStream::from_str(schema) {
        Ok(ts) => ts,
        Err(_) => {
            // If TokenStream parsing fails, return synthetic error
            let mut errors = GraphQLParseErrors::new();
            errors.add(crate::graphql_parse_error::GraphQLParseError::new(
                "Invalid Rust TokenStream".to_string(),
                Span::call_site(),
                crate::graphql_parse_error::GraphQLParseErrorKind::InvalidSyntax,
            ));
            return Err(errors);
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(token_stream);
    let parser = GraphQLSchemaParser::new(adapter);
    parser.parse_document()
}
