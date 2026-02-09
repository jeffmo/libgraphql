use crate::rust_macro_graphql_token_source::RustMacroGraphQLTokenSource;
use libgraphql_core::ast;
use libgraphql_parser::GraphQLParser;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

/// Helper to parse schema using the new libgraphql_parser pipeline
/// (RustMacroGraphQLTokenSource → GraphQLParser)
pub fn parse_with_graphqlschemaparser(
    schema: &str,
) -> ast::schema::Document {
    let token_stream =
        proc_macro2::TokenStream::from_str(schema)
            .expect("Should parse as valid TokenStream");
    let span_map = Rc::new(RefCell::new(HashMap::new()));
    let token_source =
        RustMacroGraphQLTokenSource::new(token_stream, span_map);
    let parser =
        GraphQLParser::from_token_source(token_source);
    let result = parser.parse_schema_document();
    result
        .into_valid_ast()
        .expect("Parse should succeed with no errors")
}

/// Helper to parse schema using graphql_parser
pub fn parse_with_graphql_parser(
    schema: &str,
) -> ast::schema::Document {
    ast::schema::parse(schema).expect("Parse should succeed")
}
