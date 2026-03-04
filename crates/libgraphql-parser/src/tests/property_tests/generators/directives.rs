//! Strategies for generating GraphQL directive annotations and
//! directive definitions.
//!
//! Directive annotations (`@name(args)`) appear on many constructs.
//! Directive definitions declare custom directives with their argument
//! signatures and applicable locations.
//!
//! See [Directives](https://spec.graphql.org/September2025/#sec-Language.Directives)
//! in the spec.
//!
//! Written by Claude Code, reviewed by a human.

use proptest::prelude::*;
use proptest::strategy::BoxedStrategy;

use crate::tests::property_tests::generators::names::arb_directive_name;
use crate::tests::property_tests::generators::names::arb_field_name;
use crate::tests::property_tests::generators::type_annotations::arb_type_annotation;
use crate::tests::property_tests::generators::values::arb_const_value;
use crate::tests::property_tests::generators::values::arb_value;
use crate::tests::property_tests::generators::whitespace::arb_separator;
use crate::tests::property_tests::generators::whitespace::join_items;

/// Generates a directive annotation with runtime values:
/// `@name` or `@name(arg: value, ...)`.
pub fn arb_directive_annotation(depth: usize) -> BoxedStrategy<String> {
    (
        arb_directive_name(),
        prop::option::of(arb_directive_arguments(depth)),
    )
        .prop_map(|(name, args)| match args {
            Some(a) => format!("@{name}({a})"),
            None => format!("@{name}"),
        })
        .boxed()
}

/// Generates a directive annotation with only const values
/// (for type-system contexts).
pub fn arb_const_directive_annotation(depth: usize) -> BoxedStrategy<String> {
    (
        arb_directive_name(),
        prop::option::of(arb_const_directive_arguments(depth)),
    )
        .prop_map(|(name, args)| match args {
            Some(a) => format!("@{name}({a})"),
            None => format!("@{name}"),
        })
        .boxed()
}

/// Generates 0-3 directive annotations (runtime context).
pub fn arb_directives(depth: usize) -> BoxedStrategy<String> {
    prop::collection::vec((arb_directive_annotation(depth), arb_separator()), 0..3)
        .prop_map(|pairs| {
            if pairs.is_empty() {
                String::new()
            } else {
                format!(" {}", join_items(&pairs))
            }
        })
        .boxed()
}

/// Generates 0-3 const directive annotations (type-system context).
pub fn arb_const_directives(depth: usize) -> BoxedStrategy<String> {
    prop::collection::vec(
        (arb_const_directive_annotation(depth), arb_separator()),
        0..3,
    )
        .prop_map(|pairs| {
            if pairs.is_empty() {
                String::new()
            } else {
                format!(" {}", join_items(&pairs))
            }
        })
        .boxed()
}

/// Generates 1-3 directive annotations (at least one required).
pub fn arb_directives_non_empty(depth: usize) -> BoxedStrategy<String> {
    prop::collection::vec(
        (arb_const_directive_annotation(depth), arb_separator()),
        1..4,
    )
        .prop_map(|pairs| format!(" {}", join_items(&pairs)))
        .boxed()
}

/// Generates directive argument list contents: `arg: value, ...`.
fn arb_directive_arguments(depth: usize) -> BoxedStrategy<String> {
    let arb_arg = (arb_field_name(), arb_value(depth))
        .prop_map(|(name, val)| format!("{name}: {val}"));
    prop::collection::vec((arb_arg, arb_separator()), 1..4)
        .prop_map(|pairs| join_items(&pairs))
        .boxed()
}

/// Generates const directive argument list contents.
fn arb_const_directive_arguments(depth: usize) -> BoxedStrategy<String> {
    let arb_arg = (arb_field_name(), arb_const_value(depth))
        .prop_map(|(name, val)| format!("{name}: {val}"));
    prop::collection::vec((arb_arg, arb_separator()), 1..4)
        .prop_map(|pairs| join_items(&pairs))
        .boxed()
}

/// All valid directive location names (executable + type-system).
const EXECUTABLE_DIRECTIVE_LOCATIONS: &[&str] = &[
    "FIELD",
    "FIELD_DEFINITION",
    "FRAGMENT_DEFINITION",
    "FRAGMENT_SPREAD",
    "INLINE_FRAGMENT",
    "MUTATION",
    "QUERY",
    "SUBSCRIPTION",
];

const TYPE_SYSTEM_DIRECTIVE_LOCATIONS: &[&str] = &[
    "ARGUMENT_DEFINITION",
    "ENUM",
    "ENUM_VALUE",
    "INPUT_FIELD_DEFINITION",
    "INPUT_OBJECT",
    "INTERFACE",
    "OBJECT",
    "SCALAR",
    "SCHEMA",
    "UNION",
    "VARIABLE_DEFINITION",
];

/// Generates a directive definition with argument signatures
/// and location list.
///
/// Example: `directive @skip(if: Boolean!) on FIELD | FRAGMENT_SPREAD`
pub fn arb_directive_definition(depth: usize) -> BoxedStrategy<String> {
    (
        arb_optional_description(),
        arb_directive_name(),
        prop::option::of(arb_input_value_definitions(depth)),
        prop::bool::ANY,
        arb_directive_locations(),
    )
        .prop_map(|(desc, name, args, repeatable, locations)| {
            let desc_str = desc.map_or(String::new(), |d| format!("{d}\n"));
            let args_str = args.map_or(String::new(), |a| format!("({a})"));
            let rep_str = if repeatable { " repeatable" } else { "" };
            format!("{desc_str}directive @{name}{args_str}{rep_str} on {locations}")
        })
        .boxed()
}

/// Generates 1-4 directive locations joined by ` | `.
fn arb_directive_locations() -> BoxedStrategy<String> {
    let all_locations: Vec<&str> = EXECUTABLE_DIRECTIVE_LOCATIONS
        .iter()
        .chain(TYPE_SYSTEM_DIRECTIVE_LOCATIONS.iter())
        .copied()
        .collect();

    prop::sample::subsequence(all_locations.clone(), 1..=4)
        .prop_map(|locs| {
            locs.into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(" | ")
        })
        .boxed()
}

/// Generates an optional description (block string or single-line
/// string).
fn arb_optional_description() -> BoxedStrategy<Option<String>> {
    prop::option::weighted(
        0.3,
        prop_oneof![
            "[a-zA-Z0-9 .,!?-]{1,40}".prop_map(|s| format!("\"{s}\"")),
            "[a-zA-Z0-9 .,!?-]{1,60}".prop_map(|s| format!("\"\"\"{s}\"\"\"")),
        ],
    )
    .boxed()
}

/// Generates input value definitions for directive/field arguments:
/// `name: Type` with optional default and directives.
fn arb_input_value_definitions(depth: usize) -> BoxedStrategy<String> {
    prop::collection::vec((arb_input_value_definition(depth), arb_separator()), 1..4)
        .prop_map(|pairs| join_items(&pairs))
        .boxed()
}

/// Generates a single input value definition: `name: Type` with
/// optional `= defaultValue` and directives.
fn arb_input_value_definition(depth: usize) -> BoxedStrategy<String> {
    (
        arb_field_name(),
        arb_type_annotation(2),
        prop::option::weighted(0.3, arb_const_value(depth)),
        arb_const_directives(depth),
    )
        .prop_map(|(name, ty, default, dirs)| {
            let default_str = default.map_or(String::new(), |d| format!(" = {d}"));
            format!("{name}: {ty}{default_str}{dirs}")
        })
        .boxed()
}
