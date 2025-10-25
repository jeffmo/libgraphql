use syn::LitStr;
use syn::parse_macro_input;
use quote::quote;

pub(crate) struct GraphQLSchemaFromStrTokenConsumer(proc_macro::TokenStream);
impl GraphQLSchemaFromStrTokenConsumer {
    pub fn new(input: proc_macro::TokenStream) -> Self {
        Self(input)
    }
}
impl std::convert::From<GraphQLSchemaFromStrTokenConsumer> for proc_macro::TokenStream {
    fn from(val: GraphQLSchemaFromStrTokenConsumer) -> Self {
        let token_stream = val.0;
        let input = parse_macro_input!(token_stream as LitStr);
        let schema_str = input.value();

        // Parse and build the schema at compile time
        let schema = match libgraphql_core::schema::SchemaBuilder::build_from_str(None, &schema_str) {
            Ok(schema) => schema,
            Err(err) => {
                let error_msg = format!("Failed to build GraphQL schema: {err}");
                return quote! {
                    compile_error!(#error_msg);
                }.into();
            }
        };

        // Serialize the schema using bincode
        let serialized_schema = match bincode::serde::encode_to_vec(&schema, bincode::config::standard()) {
            Ok(bytes) => bytes,
            Err(err) => {
                let error_msg = format!("Failed to serialize schema: {err}");
                return quote! {
                    compile_error!(#error_msg);
                }.into();
            }
        };

        // Generate code that deserializes the schema at runtime
        let schema_bytes = serialized_schema;
        let output = quote! {
            {
                static SERIALIZED_SCHEMA: &[u8] = &[#(#schema_bytes),*];
                libgraphql::schema::_macro_runtime::build_from_macro_serialized(SERIALIZED_SCHEMA)
            }
        };

        output.into()
    }
}
