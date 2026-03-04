//! Strategies for generating complete GraphQL documents.
//!
//! Documents are composed of definitions: type-system definitions
//! (schema docs), executable definitions (executable docs), or
//! both (mixed docs).
//!
//! See [Document](https://spec.graphql.org/September2025/#sec-Document)
//! in the spec.
//!
//! Written by Claude Code, reviewed by a human.

use proptest::prelude::*;
use proptest::strategy::BoxedStrategy;

use crate::tests::property_tests::generators::extensions::arb_type_system_extension;
use crate::tests::property_tests::generators::operations::arb_executable_definition;
use crate::tests::property_tests::generators::operations::arb_named_executable_definition;
use crate::tests::property_tests::generators::schema_types::arb_type_system_definition;

/// Generates a schema document (type-system definitions only).
///
/// Contains 1-5 type definitions, schema definitions, directive
/// definitions, and/or type extensions.
pub fn arb_schema_document(depth: usize) -> BoxedStrategy<String> {
    prop::collection::vec(arb_schema_doc_definition(depth), 1..5)
        .prop_map(|defs| defs.join("\n\n"))
        .boxed()
}

/// Generates an executable document (operations and fragments only).
///
/// Contains 1-5 operation definitions and/or fragment definitions.
pub fn arb_executable_document(depth: usize) -> BoxedStrategy<String> {
    prop::collection::vec(arb_executable_definition(depth), 1..5)
        .prop_map(|defs| defs.join("\n\n"))
        .boxed()
}

/// Generates a mixed document (both type-system and executable
/// definitions).
pub fn arb_mixed_document(depth: usize) -> BoxedStrategy<String> {
    prop::collection::vec(arb_any_definition(depth), 1..5)
        .prop_map(|defs| defs.join("\n\n"))
        .boxed()
}

/// Generates a single definition for a schema document.
fn arb_schema_doc_definition(depth: usize) -> BoxedStrategy<String> {
    prop_oneof![
        3 => arb_type_system_definition(depth),
        1 => arb_type_system_extension(depth),
    ]
    .boxed()
}

/// Generates any definition (type-system or executable).
///
/// Uses named operations only (no shorthand queries) to avoid
/// ambiguity where `{ ... }` could be parsed as a type extension
/// body rather than a shorthand query.
fn arb_any_definition(depth: usize) -> BoxedStrategy<String> {
    prop_oneof![
        2 => arb_type_system_definition(depth),
        1 => arb_type_system_extension(depth),
        2 => arb_named_executable_definition(depth),
    ]
    .boxed()
}
