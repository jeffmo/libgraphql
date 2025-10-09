mod graphql_token_parser;
mod rust_to_graphql_token_adapter;

#[cfg(test)]
mod tests;

use syn::LitStr;
use syn::parse_macro_input;
use quote::quote;

/// Evaluates to a [`Schema`](libgraphql::schema::Schema) object given direct GraphQL
/// schema document syntax.
///
/// This macro is effectively a compile-time version of
/// [`SchemaBuilder::build_from_str()`](libgraphql::schema::SchemaBuilder::build_from_ast()),
/// except you write GraphQL syntax in your Rust file and the macro parses it as
/// GraphQL for you.
///
/// > **⚠️ NOTE:** Due to limitations in Rust macros' ability to tokenize `#` as
/// > a single token, `#` cannot be used to specify a GraphQL comment in a
/// > GraphQL schema document defined with `graphql_schema! { .. }`. Instead,
/// > you can use Rust's `//` inline comment syntax directly in your GraphQL
/// > syntax.
///
/// Example usage:
///
/// ```rust
/// use libgraphql::macros::graphql_schema;
///
/// let schema = graphql_schema! {
///     type Query {
///         // This field always resolves the currently-authenticated `User`.
///         me: User,
///     }
///
///     type User {
///         firstName: String,
///         lastName: String,
///     }
/// };
///
/// let user_type =
///     schema.defined_types()
///         .get("User")
///         .unwrap()
///         .as_object()
///         .unwrap();
///
/// assert_eq!(user_type.name(), "User");
/// assert_eq!(user_type.fields().get("firstName").is_some(), true);
/// assert_eq!(user_type.fields().get("firstName").is_some(), true);
/// assert_eq!(user_type.fields().get("doesntExist").is_some(), false);
/// ```
#[proc_macro]
pub fn graphql_schema(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    // Parse tokens into GraphQL AST at compile time
    let adapter = rust_to_graphql_token_adapter::RustToGraphQLTokenAdapter::new(input);
    let parser = graphql_token_parser::GraphQLTokenParser::new(adapter);
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
/// Evaluates to a [`Schema`](libgraphql::schema::Schema) object given a literal
/// Rust `str` containing GraphQL document text that represents a GraphQL
/// schema.
///
/// This macro is effectively a compile-time version of
/// [`SchemaBuilder::build_from_str()`](libgraphql::schema::SchemaBuilder::build_from_str()).
///
/// Example usage:
///
/// ```rust
/// use libgraphql::macros::graphql_schema_from_str;
///
/// let schema = graphql_schema_from_str!(r#"
///     type Query {
///         me: User,
///
///     }
///
///     type User {
///         firstName: String,
///         lastName: String,
///     }
/// "#r);
///
/// let user_type =
///     schema.defined_types()
///         .get("User")
///         .unwrap()
///         .as_object()
///         .unwrap();
///
/// assert_eq!(user_type.name(), "User");
/// assert_eq!(user_type.fields().get("firstName").is_some(), true);
/// assert_eq!(user_type.fields().get("firstName").is_some(), true);
/// assert_eq!(user_type.fields().get("doesntExist").is_some(), false);
/// ```
#[proc_macro]
pub fn graphql_schema_from_str(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as LitStr);
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
