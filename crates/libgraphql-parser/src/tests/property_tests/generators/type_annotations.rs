//! Strategies for generating GraphQL type annotations.
//!
//! Type annotations describe the shape of fields, arguments, and
//! variables: `NamedType`, `ListType`, and `NonNullType`.
//!
//! See [Types](https://spec.graphql.org/September2025/#sec-Types) in the spec.
//!
//! Written by Claude Code, reviewed by a human.

use proptest::prelude::*;
use proptest::strategy::BoxedStrategy;

use crate::tests::property_tests::generators::names::arb_type_name;

/// Generates a valid GraphQL type annotation at the given nesting depth.
///
/// Examples: `String`, `String!`, `[Int]`, `[Int!]!`, `[[Float!]]`
pub fn arb_type_annotation(depth: usize) -> BoxedStrategy<String> {
    if depth == 0 {
        arb_named_type()
    } else {
        prop_oneof![
            3 => arb_named_type(),
            1 => arb_list_type(depth - 1),
        ]
        .boxed()
    }
}

/// Generates a named type, optionally non-null: `TypeName` or `TypeName!`.
fn arb_named_type() -> BoxedStrategy<String> {
    (arb_type_name(), prop::bool::ANY)
        .prop_map(|(name, non_null)| {
            if non_null { format!("{name}!") } else { name }
        })
        .boxed()
}

/// Generates a list type: `[InnerType]` or `[InnerType]!`.
fn arb_list_type(depth: usize) -> BoxedStrategy<String> {
    (arb_type_annotation(depth), prop::bool::ANY)
        .prop_map(|(inner, non_null)| {
            if non_null {
                format!("[{inner}]!")
            } else {
                format!("[{inner}]")
            }
        })
        .boxed()
}
