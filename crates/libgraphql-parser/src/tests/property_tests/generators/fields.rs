//! Strategies for generating field definitions, input value
//! definitions, and enum value definitions.
//!
//! These appear within type definitions (object types, interfaces,
//! input objects, enums).
//!
//! Written by Claude Code, reviewed by a human.

use proptest::prelude::*;
use proptest::strategy::BoxedStrategy;

use crate::tests::property_tests::generators::directives::arb_const_directives;
use crate::tests::property_tests::generators::names::arb_enum_value_name;
use crate::tests::property_tests::generators::names::arb_field_name;
use crate::tests::property_tests::generators::type_annotations::arb_type_annotation;
use crate::tests::property_tests::generators::values::arb_const_value;
use crate::tests::property_tests::generators::whitespace::arb_separator;
use crate::tests::property_tests::generators::whitespace::join_items;

/// Generates a field definition for object types and interfaces:
/// `name(args): Type @directives`.
///
/// See [FieldDefinition](https://spec.graphql.org/September2025/#FieldDefinition).
pub fn arb_field_definition(depth: usize) -> BoxedStrategy<String> {
    (
        arb_optional_description(),
        arb_field_name(),
        prop::option::weighted(0.3, arb_arguments_definition(depth)),
        arb_type_annotation(2),
        arb_const_directives(depth),
    )
        .prop_map(|(desc, name, args, ty, dirs)| {
            let desc_str = desc.map_or(String::new(), |d| format!("{d}\n  "));
            let args_str = args.map_or(String::new(), |a| format!("({a})"));
            format!("{desc_str}{name}{args_str}: {ty}{dirs}")
        })
        .boxed()
}

/// Generates 1-5 field definitions for object type / interface bodies.
pub fn arb_field_definitions(depth: usize) -> BoxedStrategy<String> {
    prop::collection::vec((arb_field_definition(depth), arb_separator()), 1..5)
        .prop_map(|pairs| join_items(&pairs))
        .boxed()
}

/// Generates an input value definition: `name: Type = default @dirs`.
///
/// Used in input objects, field arguments, and directive arguments.
/// See [InputValueDefinition](https://spec.graphql.org/September2025/#InputValueDefinition).
pub fn arb_input_value_definition(depth: usize) -> BoxedStrategy<String> {
    (
        arb_optional_description(),
        arb_field_name(),
        arb_type_annotation(2),
        prop::option::weighted(0.3, arb_const_value(depth)),
        arb_const_directives(depth),
    )
        .prop_map(|(desc, name, ty, default, dirs)| {
            let desc_str = desc.map_or(String::new(), |d| format!("{d}\n  "));
            let default_str = default.map_or(String::new(), |d| format!(" = {d}"));
            format!("{desc_str}{name}: {ty}{default_str}{dirs}")
        })
        .boxed()
}

/// Generates 1-5 input value definitions for input object bodies.
pub fn arb_input_value_definitions(depth: usize) -> BoxedStrategy<String> {
    prop::collection::vec((arb_input_value_definition(depth), arb_separator()), 1..5)
        .prop_map(|pairs| join_items(&pairs))
        .boxed()
}

/// Generates argument definitions for field/directive args:
/// `(name: Type, ...)`.
pub fn arb_arguments_definition(depth: usize) -> BoxedStrategy<String> {
    prop::collection::vec((arb_input_value_definition(depth), arb_separator()), 1..4)
        .prop_map(|pairs| join_items(&pairs))
        .boxed()
}

/// Generates an enum value definition: `NAME @directives`.
///
/// See [EnumValueDefinition](https://spec.graphql.org/September2025/#EnumValueDefinition).
pub fn arb_enum_value_definition(depth: usize) -> BoxedStrategy<String> {
    (
        arb_optional_description(),
        arb_enum_value_name(),
        arb_const_directives(depth),
    )
        .prop_map(|(desc, name, dirs)| {
            let desc_str = desc.map_or(String::new(), |d| format!("{d}\n  "));
            format!("{desc_str}{name}{dirs}")
        })
        .boxed()
}

/// Generates 1-8 enum value definitions for enum type bodies.
pub fn arb_enum_value_definitions(
    depth: usize,
    num_values: std::ops::Range<usize>,
) -> BoxedStrategy<String> {
    prop::collection::vec((arb_enum_value_definition(depth), arb_separator()), num_values)
        .prop_map(|pairs| join_items(&pairs))
        .boxed()
}

/// Generates an optional description string.
fn arb_optional_description() -> BoxedStrategy<Option<String>> {
    prop::option::weighted(
        0.2,
        prop_oneof![
            "[a-zA-Z0-9 .,!?-]{1,40}".prop_map(|s| format!("\"{s}\"")),
            "[a-zA-Z0-9 .,!?-]{1,60}".prop_map(|s| format!("\"\"\"{s}\"\"\"")),
        ],
    )
    .boxed()
}
