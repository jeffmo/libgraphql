//! `libgraphql-core` provides the core schema building,
//! validation, and operation building libraries for the
//! [`libgraphql`](https://docs.rs/libgraphql) crate.
//!
//! This crate consumes AST produced by
//! [`libgraphql-parser`](https://docs.rs/libgraphql-parser) and
//! transforms it into fully validated, owned semantic types. It
//! implements comprehensive
//! [GraphQL specification](https://spec.graphql.org/September2025/)
//! validation for both schema definitions and executable operations.
//!
//! ## Architecture
//!
//! - **Name newtypes** (`TypeName`, `FieldName`, etc.) prevent
//!   cross-domain string confusion
//! - **Builder pattern** for type-safe, incremental construction of
//!   schemas and operations
//! - **Owned types** (no lifetime parameters) enable caching,
//!   serialization, and long-lived storage
//! - **Serde + bincode** support for compile-time schema embedding
//!   via `libgraphql-macros`, caching support, and thread-safety
//!
//! ## Usage
//!
//! ```ignore
//! use libgraphql_core::schema::SchemaBuilder;
//!
//! let schema = SchemaBuilder::build_from_str(
//!     "type Query { hello: String }",
//! ).unwrap();
//! ```
