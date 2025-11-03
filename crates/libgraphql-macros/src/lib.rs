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

/// Evaluates to a [`Schema`](libgraphql_core::schema::Schema) object given
/// direct GraphQL schema document syntax.
///
/// This macro is effectively a compile-time version of
/// [`SchemaBuilder::build_from_str()`](libgraphql_core::schema::SchemaBuilder::build_from_ast()),
/// except you write GraphQL syntax in your Rust file and the macro parses it as
/// GraphQL for you.
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
///
/// ## **⚠️ NOTE:**
///
/// Due to limitations downstream of how Rust macros tokenize syntax, there
/// are a few inline GraphQL syntax edge-cases that are not supported by this
/// macro:
///
///   1) `#` cannot be used to specify GraphQL comments. Instead, you can use
///      Rust's `//` or `/*` comment syntax.
///
///      So for example, this won't compile:
///
///      ```rust,compile_fail
///      let schema = graphql_schema! {
///        type Query {
///          me: User,
///        }
///
///        ## Represents a user in the system.
///        type User {
///          firstName: String,
///          lastName: String,
///        }
///      };
///      ```
///
///      But you can use rust's `//` and `/*` comment syntax instead:
///      ```rust
///      # use libgraphql::macros::graphql_schema;
///      let schema = graphql_schema! {
///        type Query {
///          me: User,
///        }
///
///        // Represents a user in the system.
///        type User {
///          /* The user's first name. */
///          firstName: String,
///
///          /* The user's last name. */
///          lastName: String,
///        }
///      };
///      ```
///
///   2) Block-quoted strings (`"""`) *are* supported, but if you ever need to
///      nest a quoted string of any kind /within/ a block-quoted string,
///      you'll need to use Rust's `r#""#` "raw string" syntax instead.
///
///      So for example, this won't compile:
///      ```rust,compile_fail
///      let schema = graphql_schema! {
///        type Query {
///          me: User,
///        }
///
///         type User {
///           """
///           The user's "primary" address.
///           """
///           address: String,
///         }
///      };
///      ```
///
///      But the workaround is to use raw-string syntax instead:
///      ```rust
///      # use libgraphql::macros::graphql_schema;
///      let schema = graphql_schema! {
///        type Query {
///          me: User,
///        }
///
///        type User {
///          r#"
///          The user's "primary" address.
///          "#
///          address: String,
///        }
///      };
///        ```
#[proc_macro]
pub fn graphql_schema(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    GraphQLSchemaTokenConsumer::new(input).into()
}

/// Evaluates to a [`Schema`](libgraphql_core::schema::Schema) object given a literal
/// Rust `str` containing GraphQL document text that represents a GraphQL
/// schema.
///
/// This macro is effectively a compile-time version of
/// [`SchemaBuilder::build_from_str()`](libgraphql_core::schema::SchemaBuilder::build_from_str()).
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
