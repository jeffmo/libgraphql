#[doc(inline)]
pub use libgraphql_core::*;

/// Helpful macros for doing things with GraphQL at Rust compile-time
/// (e.g. Define a compile-time validated and typechecked GraphQL
/// [`Schema`](crate::schema::Schema), etc)
#[doc(inline)]
pub use libgraphql_macros as macros;

#[cfg(doctest)]
#[doc = include_str!("../../../README.md")]
struct README;
