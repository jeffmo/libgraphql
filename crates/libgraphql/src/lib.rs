pub use libgraphql_core::*;

/// Helpful macros for doing things with GraphQL at Rust compile-time
/// (e.g. Define a GraphQL [`Schema`](crate::schema::Schema) with
/// compile-time GraphQL validation, etc)
pub mod macros {
    pub use libgraphql_macros::*;
}
