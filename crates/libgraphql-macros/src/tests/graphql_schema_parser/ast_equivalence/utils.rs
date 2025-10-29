use crate::graphql_schema_parser::GraphQLSchemaParser;
use crate::rust_to_graphql_token_adapter::RustToGraphQLTokenAdapter;
use libgraphql_core::ast;
use std::str::FromStr;

/// Helper to parse schema using GraphQLSchemaParser
pub fn parse_with_graphqlschemaparser(
    schema: &str,
) -> ast::schema::Document {
    let token_stream = proc_macro2::TokenStream::from_str(schema)
        .expect("Should parse as valid TokenStream");
    let adapter = RustToGraphQLTokenAdapter::new(token_stream);
    let parser = GraphQLSchemaParser::new(adapter);
    parser.parse_document().expect("Parse should succeed")
}

/// Helper to parse schema using graphql_parser
pub fn parse_with_graphql_parser(
    schema: &str,
) -> ast::schema::Document {
    ast::schema::parse(schema).expect("Parse should succeed")
}
