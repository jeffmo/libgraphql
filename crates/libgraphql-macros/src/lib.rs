mod graphql_schema_token_consumer;
mod graphql_schema_from_str_token_consumer;
mod emittable_schema;
mod graphql_schema_parser;
mod graphql_parse_error;
mod graphql_token_stream;
mod rust_to_graphql_token_adapter;

#[cfg(test)]
mod tests;

use crate::graphql_schema_token_consumer::GraphQLSchemaTokenConsumer;
use crate::graphql_schema_from_str_token_consumer::GraphQLSchemaFromStrTokenConsumer;

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
    GraphQLSchemaTokenConsumer::new(input).into()
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
    GraphQLSchemaFromStrTokenConsumer::new(input).into()
}
