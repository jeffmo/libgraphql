//! Strategies for generating all 6 GraphQL type extensions
//! plus schema extensions.
//!
//! Each extension must have at least one non-empty clause (directives,
//! body, or implements).
//!
//! See [Type Extensions](https://spec.graphql.org/September2025/#sec-Type-Extensions)
//! in the spec.
//!
//! Written by Claude Code, reviewed by a human.

use proptest::prelude::*;
use proptest::strategy::BoxedStrategy;

use crate::tests::property_tests::generators::directives::arb_const_directives;
use crate::tests::property_tests::generators::directives::arb_directives_non_empty;
use crate::tests::property_tests::generators::fields::arb_enum_value_definitions;
use crate::tests::property_tests::generators::fields::arb_field_definitions;
use crate::tests::property_tests::generators::fields::arb_input_value_definitions;
use crate::tests::property_tests::generators::names::arb_type_name;

/// Generates a scalar type extension: `extend scalar Name @dirs`.
///
/// Scalars can only be extended with directives.
pub fn arb_scalar_type_extension(depth: usize) -> BoxedStrategy<String> {
    (arb_type_name(), arb_directives_non_empty(depth))
        .prop_map(|(name, dirs)| format!("extend scalar {name}{dirs}"))
        .boxed()
}

/// Generates an object type extension with at least one clause.
pub fn arb_object_type_extension(depth: usize) -> BoxedStrategy<String> {
    prop_oneof![
        // With fields (may also have implements + directives)
        (
            arb_type_name(),
            arb_implements_opt(),
            arb_const_directives(depth),
            arb_field_definitions(depth),
        )
            .prop_map(|(name, implements, dirs, fields)| {
                format!("extend type {name}{implements}{dirs} {{\n  {fields}\n}}")
            }),
        // Directives only (no fields)
        (arb_type_name(), arb_implements_opt(), arb_directives_non_empty(depth))
            .prop_map(|(name, implements, dirs)| {
                format!("extend type {name}{implements}{dirs}")
            }),
        // Implements only (no directives, no fields)
        (arb_type_name(), arb_implements_required())
            .prop_map(|(name, implements)| {
                format!("extend type {name}{implements}")
            }),
    ]
    .boxed()
}

/// Generates an interface type extension with at least one clause.
pub fn arb_interface_type_extension(depth: usize) -> BoxedStrategy<String> {
    prop_oneof![
        // With fields
        (
            arb_type_name(),
            arb_implements_opt(),
            arb_const_directives(depth),
            arb_field_definitions(depth),
        )
            .prop_map(|(name, implements, dirs, fields)| {
                format!("extend interface {name}{implements}{dirs} {{\n  {fields}\n}}")
            }),
        // Directives only
        (arb_type_name(), arb_directives_non_empty(depth))
            .prop_map(|(name, dirs)| {
                format!("extend interface {name}{dirs}")
            }),
        // Implements only
        (arb_type_name(), arb_implements_required())
            .prop_map(|(name, implements)| {
                format!("extend interface {name}{implements}")
            }),
    ]
    .boxed()
}

/// Generates a union type extension with at least one clause.
pub fn arb_union_type_extension(depth: usize) -> BoxedStrategy<String> {
    prop_oneof![
        // With member types
        (
            arb_type_name(),
            arb_const_directives(depth),
            arb_union_members_required(),
        )
            .prop_map(|(name, dirs, members)| {
                format!("extend union {name}{dirs} = {members}")
            }),
        // Directives only
        (arb_type_name(), arb_directives_non_empty(depth))
            .prop_map(|(name, dirs)| format!("extend union {name}{dirs}")),
    ]
    .boxed()
}

/// Generates an enum type extension with at least one clause.
pub fn arb_enum_type_extension(depth: usize) -> BoxedStrategy<String> {
    prop_oneof![
        // With values
        (
            arb_type_name(),
            arb_const_directives(depth),
            arb_enum_value_definitions(depth, 1..5),
        )
            .prop_map(|(name, dirs, values)| {
                format!("extend enum {name}{dirs} {{\n  {values}\n}}")
            }),
        // Directives only
        (arb_type_name(), arb_directives_non_empty(depth))
            .prop_map(|(name, dirs)| format!("extend enum {name}{dirs}")),
    ]
    .boxed()
}

/// Generates an input object type extension with at least one clause.
pub fn arb_input_object_type_extension(depth: usize) -> BoxedStrategy<String> {
    prop_oneof![
        // With fields
        (
            arb_type_name(),
            arb_const_directives(depth),
            arb_input_value_definitions(depth),
        )
            .prop_map(|(name, dirs, fields)| {
                format!("extend input {name}{dirs} {{\n  {fields}\n}}")
            }),
        // Directives only
        (arb_type_name(), arb_directives_non_empty(depth))
            .prop_map(|(name, dirs)| format!("extend input {name}{dirs}")),
    ]
    .boxed()
}

/// Generates a schema extension with at least one clause.
pub fn arb_schema_extension(depth: usize) -> BoxedStrategy<String> {
    prop_oneof![
        // With root operations
        (arb_const_directives(depth), arb_root_ops_required())
            .prop_map(|(dirs, ops)| {
                format!("extend schema{dirs} {{\n  {ops}\n}}")
            }),
        // Directives only
        arb_directives_non_empty(depth)
            .prop_map(|dirs| format!("extend schema{dirs}")),
    ]
    .boxed()
}

/// Generates any type extension (one of the 6 kinds) or a
/// schema extension.
pub fn arb_type_system_extension(depth: usize) -> BoxedStrategy<String> {
    prop_oneof![
        2 => arb_object_type_extension(depth),
        1 => arb_interface_type_extension(depth),
        1 => arb_union_type_extension(depth),
        1 => arb_enum_type_extension(depth),
        1 => arb_input_object_type_extension(depth),
        1 => arb_scalar_type_extension(depth),
        1 => arb_schema_extension(depth),
    ]
    .boxed()
}

// ─── Helpers ────────────────────────────────────────────────

/// Generates an optional `implements` clause (may be empty).
fn arb_implements_opt() -> BoxedStrategy<String> {
    prop::collection::vec(arb_type_name(), 0..3)
        .prop_map(|ifaces| {
            if ifaces.is_empty() {
                String::new()
            } else {
                format!(" implements {}", ifaces.join(" & "))
            }
        })
        .boxed()
}

/// Generates a required `implements` clause (at least 1 interface).
fn arb_implements_required() -> BoxedStrategy<String> {
    prop::collection::vec(arb_type_name(), 1..4)
        .prop_map(|ifaces| format!(" implements {}", ifaces.join(" & ")))
        .boxed()
}

/// Generates union member types (at least 1).
fn arb_union_members_required() -> BoxedStrategy<String> {
    prop::collection::vec(arb_type_name(), 1..5)
        .prop_map(|members| members.join(" | "))
        .boxed()
}

/// Generates root operation definitions (at least 1).
fn arb_root_ops_required() -> BoxedStrategy<String> {
    (
        prop::option::weighted(0.8, arb_type_name()),
        prop::option::weighted(0.4, arb_type_name()),
        prop::option::weighted(0.2, arb_type_name()),
    )
        .prop_filter(
            "at least one root operation required",
            |(q, m, s)| q.is_some() || m.is_some() || s.is_some(),
        )
        .prop_map(|(query, mutation, subscription)| {
            let mut ops = Vec::new();
            if let Some(q) = query {
                ops.push(format!("query: {q}"));
            }
            if let Some(m) = mutation {
                ops.push(format!("mutation: {m}"));
            }
            if let Some(s) = subscription {
                ops.push(format!("subscription: {s}"));
            }
            ops.join("\n  ")
        })
        .boxed()
}
