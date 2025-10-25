use crate::emittable_schema::EmittableSchema;
use crate::graphql_schema_parser::GraphQLSchemaParser;
use crate::rust_to_graphql_token_adapter::RustToGraphQLTokenAdapter;
use quote::quote;

pub(crate) struct GraphQLSchemaTokenConsumer(proc_macro::TokenStream);
impl GraphQLSchemaTokenConsumer {
    pub fn new(input: proc_macro::TokenStream) -> Self {
        Self(input)
    }
}

impl std::convert::From<GraphQLSchemaTokenConsumer> for proc_macro::TokenStream {
    fn from(val: GraphQLSchemaTokenConsumer) -> Self {
        let input = proc_macro2::TokenStream::from(val.0);

        // Parse tokens into GraphQL AST at compile time
        let adapter = RustToGraphQLTokenAdapter::new(input);
        let parser = GraphQLSchemaParser::new(adapter);
        let ast_doc = match parser.parse_document() {
            Ok(doc) => doc,
            Err(err) => {
                let error_msg = format!("Failed to parse GraphQL schema: {err}");
                return quote! {
                    compile_error!(#error_msg);
                }.into();
            }
        };

        // Build the schema at compile time
        let schema = match libgraphql_core::schema::SchemaBuilder::build_from_ast(None, ast_doc) {
            Ok(schema) => schema,
            Err(err) => {
                let error_msg = format!("Failed to build GraphQL schema: {err}");
                return quote! {
                    compile_error!(#error_msg);
                }.into();
            }
        };

        EmittableSchema::new(schema).into()
    }
}
