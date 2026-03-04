//! Strategies for generating GraphQL selection sets, fields,
//! fragment spreads, and inline fragments.
//!
//! Selection sets are the recursive core of executable GraphQL:
//! they appear in operations, fields, and inline fragments. The
//! `depth` parameter controls recursion to keep generation bounded.
//!
//! See [Selection Sets](https://spec.graphql.org/September2025/#sec-Selection-Sets)
//! in the spec.
//!
//! Written by Claude Code, reviewed by a human.

use proptest::prelude::*;
use proptest::strategy::BoxedStrategy;

use crate::tests::property_tests::generators::directives::arb_directives;
use crate::tests::property_tests::generators::names::arb_field_name;
use crate::tests::property_tests::generators::names::arb_fragment_name;
use crate::tests::property_tests::generators::names::arb_type_name;
use crate::tests::property_tests::generators::values::arb_value;

/// Generates a selection set: `{ selection+ }`.
///
/// At depth 0, only simple fields (no sub-selections) are generated.
pub fn arb_selection_set(depth: usize) -> BoxedStrategy<String> {
    prop::collection::vec(arb_selection(depth), 1..4)
        .prop_map(|sels| format!("{{ {} }}", sels.join(" ")))
        .boxed()
}

/// Generates a single selection: field, fragment spread, or
/// inline fragment.
fn arb_selection(depth: usize) -> BoxedStrategy<String> {
    if depth == 0 {
        arb_simple_field()
    } else {
        prop_oneof![
            4 => arb_field(depth),
            1 => arb_fragment_spread(depth),
            1 => arb_inline_fragment(depth),
        ]
        .boxed()
    }
}

/// Generates a simple field with no sub-selection set:
/// `name` or `alias: name`.
fn arb_simple_field() -> BoxedStrategy<String> {
    (
        prop::option::weighted(0.2, arb_field_name()),
        arb_field_name(),
        arb_directives(0),
    )
        .prop_map(|(alias, name, dirs)| match alias {
            Some(a) => format!("{a}: {name}{dirs}"),
            None => format!("{name}{dirs}"),
        })
        .boxed()
}

/// Generates a field with optional alias, arguments, directives,
/// and sub-selection set.
///
/// See [Fields](https://spec.graphql.org/September2025/#sec-Language.Fields).
fn arb_field(depth: usize) -> BoxedStrategy<String> {
    (
        prop::option::weighted(0.2, arb_field_name()),
        arb_field_name(),
        prop::option::weighted(0.3, arb_field_arguments(depth)),
        arb_directives(depth.min(1)),
        prop::option::weighted(0.4, arb_selection_set(depth - 1)),
    )
        .prop_map(|(alias, name, args, dirs, sub_sel)| {
            let alias_str = alias.map_or(String::new(), |a| format!("{a}: "));
            let args_str = args.map_or(String::new(), |a| format!("({a})"));
            let sub_str = sub_sel.map_or(String::new(), |s| format!(" {s}"));
            format!("{alias_str}{name}{args_str}{dirs}{sub_str}")
        })
        .boxed()
}

/// Generates field arguments: `name: value, ...`.
fn arb_field_arguments(depth: usize) -> BoxedStrategy<String> {
    prop::collection::vec(
        (arb_field_name(), arb_value(depth.min(2))),
        1..4,
    )
    .prop_map(|args| {
        args.into_iter()
            .map(|(name, val)| format!("{name}: {val}"))
            .collect::<Vec<_>>()
            .join(", ")
    })
    .boxed()
}

/// Generates a fragment spread: `...FragmentName @directives`.
///
/// See [Fragment Spreads](https://spec.graphql.org/September2025/#FragmentSpread).
fn arb_fragment_spread(depth: usize) -> BoxedStrategy<String> {
    (arb_fragment_name(), arb_directives(depth.min(1)))
        .prop_map(|(name, dirs)| format!("...{name}{dirs}"))
        .boxed()
}

/// Generates an inline fragment:
/// `... on TypeName @dirs { selections }`.
///
/// The type condition is optional per the spec.
/// See [Inline Fragments](https://spec.graphql.org/September2025/#InlineFragment).
fn arb_inline_fragment(depth: usize) -> BoxedStrategy<String> {
    (
        prop::option::weighted(0.8, arb_type_name()),
        arb_directives(depth.min(1)),
        arb_selection_set(depth - 1),
    )
        .prop_map(|(type_cond, dirs, sel_set)| {
            let tc_str = type_cond.map_or(String::new(), |t| format!(" on {t}"));
            format!("...{tc_str}{dirs} {sel_set}")
        })
        .boxed()
}
