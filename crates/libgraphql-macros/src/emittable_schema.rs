use libgraphql_core::schema::Schema;
use quote::quote;

pub(crate) struct EmittableSchema(Schema);
impl EmittableSchema {
    pub fn new(schema: Schema) -> Self {
        Self(schema)
    }
}
impl std::convert::From<EmittableSchema> for proc_macro::TokenStream {
    fn from(val: EmittableSchema) -> Self {
        // Serialize the schema using bincode
        let serialized_schema = match bincode::serde::encode_to_vec(
            &val.0,
            bincode::config::standard()
        ) {
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
