//! Strategies for generating GraphQL operation definitions,
//! variable definitions, and fragment definitions.
//!
//! Operations are the entry points for executable documents:
//! queries, mutations, and subscriptions.
//!
//! See [Executable Definitions](https://spec.graphql.org/September2025/#ExecutableDefinition)
//! in the spec.
//!
//! Written by Claude Code, reviewed by a human.

use proptest::prelude::*;
use proptest::strategy::BoxedStrategy;

use crate::tests::property_tests::generators::directives::arb_const_directives;
use crate::tests::property_tests::generators::directives::arb_directives;
use crate::tests::property_tests::generators::names::arb_field_name;
use crate::tests::property_tests::generators::names::arb_fragment_name;
use crate::tests::property_tests::generators::names::arb_name;
use crate::tests::property_tests::generators::names::arb_type_name;
use crate::tests::property_tests::generators::selections::arb_selection_set;
use crate::tests::property_tests::generators::type_annotations::arb_type_annotation;
use crate::tests::property_tests::generators::values::arb_const_value;
use crate::tests::property_tests::generators::whitespace::arb_separator;
use crate::tests::property_tests::generators::whitespace::join_items;

/// Generates a named operation definition:
/// `query|mutation|subscription Name($var: Type) @dirs { ... }`.
///
/// See [OperationDefinition](https://spec.graphql.org/September2025/#OperationDefinition).
pub fn arb_operation_definition(depth: usize) -> BoxedStrategy<String> {
    prop_oneof![
        3 => arb_named_operation(depth),
        1 => arb_shorthand_query(depth),
    ]
    .boxed()
}

/// Generates a named (non-shorthand) operation.
fn arb_named_operation(depth: usize) -> BoxedStrategy<String> {
    (
        arb_operation_type(),
        prop::option::weighted(0.8, arb_name()),
        prop::option::weighted(0.4, arb_variable_definitions()),
        arb_directives(depth.min(1)),
        arb_selection_set(depth),
    )
        .prop_map(|(op_type, name, vars, dirs, sel_set)| {
            let name_str = name.map_or(String::new(), |n| format!(" {n}"));
            let vars_str = vars.map_or(String::new(), |v| format!("({v})"));
            format!("{op_type}{name_str}{vars_str}{dirs} {sel_set}")
        })
        .boxed()
}

/// Generates a shorthand query (anonymous selection set):
/// `{ field1 field2 }`.
fn arb_shorthand_query(depth: usize) -> BoxedStrategy<String> {
    arb_selection_set(depth)
}

/// Generates an operation type keyword.
fn arb_operation_type() -> BoxedStrategy<String> {
    prop_oneof![
        3 => Just("query".to_string()),
        1 => Just("mutation".to_string()),
        1 => Just("subscription".to_string()),
    ]
    .boxed()
}

/// Generates variable definitions: `$var: Type = default, ...`.
///
/// See [VariableDefinitions](https://spec.graphql.org/September2025/#VariableDefinitions).
fn arb_variable_definitions() -> BoxedStrategy<String> {
    prop::collection::vec((arb_variable_definition(), arb_separator()), 1..4)
        .prop_map(|pairs| join_items(&pairs))
        .boxed()
}

/// Generates a single variable definition:
/// `$name: Type` with optional default value and directives.
///
/// Per the spec, directives on variable definitions must use
/// const values (no variable references).
fn arb_variable_definition() -> BoxedStrategy<String> {
    (
        arb_field_name(),
        arb_type_annotation(2),
        prop::option::weighted(0.3, arb_const_value(1)),
        arb_const_directives(0),
    )
        .prop_map(|(name, ty, default, dirs)| {
            let default_str = default.map_or(String::new(), |d| format!(" = {d}"));
            format!("${name}: {ty}{default_str}{dirs}")
        })
        .boxed()
}

/// Generates a fragment definition:
/// `fragment Name on TypeName @dirs { ... }`.
///
/// See [FragmentDefinition](https://spec.graphql.org/September2025/#FragmentDefinition).
pub fn arb_fragment_definition(depth: usize) -> BoxedStrategy<String> {
    (
        arb_fragment_name(),
        arb_type_name(),
        arb_directives(depth.min(1)),
        arb_selection_set(depth),
    )
        .prop_map(|(name, type_cond, dirs, sel_set)| {
            format!("fragment {name} on {type_cond}{dirs} {sel_set}")
        })
        .boxed()
}

/// Generates any executable definition (operation or fragment).
pub fn arb_executable_definition(depth: usize) -> BoxedStrategy<String> {
    prop_oneof![
        3 => arb_operation_definition(depth),
        1 => arb_fragment_definition(depth),
    ]
    .boxed()
}

/// Generates an executable definition that always uses named
/// operations (no shorthand queries).
///
/// This avoids ambiguity in mixed documents where a shorthand query
/// `{ ... }` could be confused with a type extension body.
pub fn arb_named_executable_definition(depth: usize) -> BoxedStrategy<String> {
    prop_oneof![
        3 => arb_named_operation(depth),
        1 => arb_fragment_definition(depth),
    ]
    .boxed()
}
