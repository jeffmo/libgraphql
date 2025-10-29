#[doc = include_str!("../../../README.md")]

pub use libgraphql_core::*;

/// Helpful macros for doing things with GraphQL at Rust compile-time
/// (e.g. Define, validate, and typecheck a GraphQL [`Schema`](crate::schema::Schema)
/// at compile-time, etc)
#[doc(inline)]
pub use libgraphql_macros as macros;
