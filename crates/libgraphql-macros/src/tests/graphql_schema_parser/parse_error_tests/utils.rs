use crate::graphql_parse_error::GraphQLParseError;
use crate::graphql_parse_error::GraphQLParseErrorKind;
use crate::graphql_parse_error::GraphQLParseErrors;
use crate::graphql_schema_parser::GraphQLSchemaParser;
use crate::rust_to_graphql_token_adapter::RustToGraphQLTokenAdapter;
use proc_macro2::Span;
use std::str::FromStr;

/// Helper to parse schema and expect an error
///
/// Note: If the schema contains syntax that makes it invalid as a Rust
/// TokenStream (e.g., unclosed delimiters, unterminated strings), this
/// will return a synthetic error since our parser never gets invoked.
pub fn parse_expecting_error(schema: &str) -> GraphQLParseErrors {
    let token_stream = match proc_macro2::TokenStream::from_str(schema) {
        Ok(ts) => ts,
        Err(_) => {
            // If TokenStream parsing fails, return synthetic error
            // This happens for unclosed delimiters, EOF, etc.
            let mut errors = GraphQLParseErrors::new();
            errors.add(GraphQLParseError::new(
                "Invalid Rust TokenStream".to_string(),
                Span::call_site(),
                GraphQLParseErrorKind::InvalidSyntax,
            ));
            return errors;
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(token_stream);
    let parser = GraphQLSchemaParser::new(adapter);
    parser
        .parse_document()
        .expect_err("Expected parse to fail")
}
