//! Strategies for generating valid GraphQL names and identifiers.
//!
//! GraphQL names follow the pattern `[_A-Za-z][_0-9A-Za-z]*`.
//! See [Names](https://spec.graphql.org/September2025/#Name) in the spec.
//!
//! Written by Claude Code, reviewed by a human.

use proptest::prelude::*;
use proptest::strategy::BoxedStrategy;

/// Generates a valid GraphQL name: `[_A-Za-z][_0-9A-Za-z]{0,15}`.
///
/// No reserved-word filtering is applied here. In GraphQL, `true`,
/// `false`, and `null` are only reserved in specific contexts (enum
/// values), not as general names — they are valid as type names,
/// field names, directive names, etc.
pub fn arb_name() -> BoxedStrategy<String> {
    "[_A-Za-z][_0-9A-Za-z]{0,15}".boxed()
}

/// Generates a valid GraphQL name suitable for use as a fragment name.
///
/// Fragment names must not be `on` (which is reserved for type
/// conditions), per
/// [FragmentName](https://spec.graphql.org/September2025/#FragmentName).
pub fn arb_fragment_name() -> BoxedStrategy<String> {
    "[_A-Za-z][_0-9A-Za-z]{0,15}"
        .prop_filter("'on' is reserved for type conditions", |s| s != "on")
        .boxed()
}

/// Generates a valid GraphQL name suitable for use as an enum value.
///
/// Enum value names must not be `true`, `false`, or `null`, per
/// [EnumValue](https://spec.graphql.org/September2025/#EnumValue).
pub fn arb_enum_value_name() -> BoxedStrategy<String> {
    "[_A-Za-z][_0-9A-Za-z]{0,15}"
        .prop_filter("true/false/null are reserved enum value names", |s| {
            s != "true" && s != "false" && s != "null"
        })
        .boxed()
}

/// Generates a simple type name (capitalised by convention).
///
/// While GraphQL does not enforce capitalisation, type names are
/// conventionally PascalCase. We generate names starting with an
/// uppercase letter to produce more realistic documents.
pub fn arb_type_name() -> BoxedStrategy<String> {
    "[A-Z][_0-9A-Za-z]{0,15}".boxed()
}

/// Generates a simple field/argument name (lowercase by convention).
pub fn arb_field_name() -> BoxedStrategy<String> {
    "[_a-z][_0-9A-Za-z]{0,15}".boxed()
}

/// Generates a directive name (lowercase by convention).
pub fn arb_directive_name() -> BoxedStrategy<String> {
    arb_field_name()
}
