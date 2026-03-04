//! Grammar-guided property-based tests for libgraphql-parser.
//!
//! Uses `proptest` to generate structurally valid GraphQL source text
//! and verify parser correctness via spec conformance, AST faithfulness,
//! round-trip stability, and differential comparison against
//! `graphql_parser` v0.4.
//!
//! Written by Claude Code, reviewed by a human.

mod generators;
mod properties;

use proptest::prelude::ProptestConfig;

/// Shared proptest configuration for all property tests.
pub fn proptest_config() -> ProptestConfig {
    ProptestConfig {
        cases: 500,
        max_shrink_iters: 4000,
        timeout: 30_000,
        ..Default::default()
    }
}
