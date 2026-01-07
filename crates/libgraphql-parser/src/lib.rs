//! A GraphQL parsing library to parse schema documents, executable documents,
//! and documents that mix both together.
//!
//! This crate provides a unified token-based parser infrastructure with
//! support for multiple token sources (string input, proc-macro input, etc.).

pub mod ast;
mod source_position;
pub mod token;
pub mod token_source;

pub use smallvec::SmallVec;
pub use source_position::SourcePosition;

#[cfg(test)]
mod tests;
