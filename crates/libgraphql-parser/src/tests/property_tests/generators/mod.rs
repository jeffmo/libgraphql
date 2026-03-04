//! Hierarchical source-text generators mirroring the GraphQL grammar.
//!
//! Each generator produces a `BoxedStrategy<String>` of valid GraphQL
//! source text. They compose from leaf strategies (names, values) up
//! through type system definitions and executable operations to full
//! documents.
//!
//! Written by Claude Code, reviewed by a human.

pub mod directives;
pub mod documents;
pub mod extensions;
pub mod fields;
pub mod mutations;
pub mod names;
pub mod operations;
pub mod schema_types;
pub mod selections;
pub mod type_annotations;
pub mod values;
pub mod whitespace;
