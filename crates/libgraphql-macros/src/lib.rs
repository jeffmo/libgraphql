mod emittable_schema;
mod graphql_schema_from_str_token_consumer;
mod graphql_schema_token_consumer;
mod parse_error_converter;
mod rust_macro_graphql_token_source;
mod span_map;

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
///   2) Block-quoted strings (`"""`) *are* supported, but if you need to
///      nest a quoted string /within/ a block-quoted string, you must
///      escape the inner quotes with `\"`. This is because Rust's
///      tokenizer treats unescaped `"` as string delimiters, which
///      breaks the block string recombination.
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
///      But the workaround is to escape the inner quotes:
///      ```rust
///      # use libgraphql::macros::graphql_schema;
///      let schema = graphql_schema! {
///        type Query {
///          me: User,
///        }
///
///        type User {
///          """
///          The user's \"primary\" address.
///          """
///          address: String,
///        }
///      };
///      ```
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
