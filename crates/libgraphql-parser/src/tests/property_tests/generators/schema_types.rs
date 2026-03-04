//! Strategies for generating all 6 GraphQL type definitions
//! plus schema definitions.
//!
//! Each type definition generator produces syntactically valid source
//! text for its respective type system construct.
//!
//! See [Type System](https://spec.graphql.org/September2025/#sec-Type-System)
//! in the spec.
//!
//! Written by Claude Code, reviewed by a human.

use proptest::prelude::*;
use proptest::strategy::BoxedStrategy;

use crate::tests::property_tests::generators::directives::arb_const_directives;
use crate::tests::property_tests::generators::directives::arb_directive_definition;
use crate::tests::property_tests::generators::fields::arb_enum_value_definitions;
use crate::tests::property_tests::generators::fields::arb_field_definitions;
use crate::tests::property_tests::generators::fields::arb_input_value_definitions;
use crate::tests::property_tests::generators::names::arb_type_name;

/// Generates a ScalarTypeDefinition:
/// `scalar Name @directives`
///
/// See [Scalar Type](https://spec.graphql.org/September2025/#sec-Scalars).
pub fn arb_scalar_type_definition(depth: usize) -> BoxedStrategy<String> {
    (
        arb_optional_description(),
        arb_type_name(),
        arb_const_directives(depth),
    )
        .prop_map(|(desc, name, dirs)| {
            let desc_str = desc.map_or(String::new(), |d| format!("{d}\n"));
            format!("{desc_str}scalar {name}{dirs}")
        })
        .boxed()
}

/// Generates an ObjectTypeDefinition:
/// `type Name implements I1 & I2 @dirs { field1 field2 }`
///
/// See [Object Type](https://spec.graphql.org/September2025/#sec-Objects).
pub fn arb_object_type_definition(depth: usize) -> BoxedStrategy<String> {
    (
        arb_optional_description(),
        arb_type_name(),
        arb_implements_interfaces(0..4),
        arb_const_directives(depth),
        arb_field_definitions(depth),
    )
        .prop_map(|(desc, name, implements, dirs, fields)| {
            let desc_str = desc.map_or(String::new(), |d| format!("{d}\n"));
            format!("{desc_str}type {name}{implements}{dirs} {{\n  {fields}\n}}")
        })
        .boxed()
}

/// Generates an InterfaceTypeDefinition:
/// `interface Name implements I1 & I2 @dirs { field1 field2 }`
///
/// See [Interface Type](https://spec.graphql.org/September2025/#sec-Interfaces).
pub fn arb_interface_type_definition(depth: usize) -> BoxedStrategy<String> {
    (
        arb_optional_description(),
        arb_type_name(),
        arb_implements_interfaces(0..3),
        arb_const_directives(depth),
        arb_field_definitions(depth),
    )
        .prop_map(|(desc, name, implements, dirs, fields)| {
            let desc_str = desc.map_or(String::new(), |d| format!("{d}\n"));
            format!("{desc_str}interface {name}{implements}{dirs} {{\n  {fields}\n}}")
        })
        .boxed()
}

/// Generates a UnionTypeDefinition:
/// `union Name @dirs = Type1 | Type2 | Type3`
///
/// See [Union Type](https://spec.graphql.org/September2025/#sec-Unions).
pub fn arb_union_type_definition(depth: usize) -> BoxedStrategy<String> {
    (
        arb_optional_description(),
        arb_type_name(),
        arb_const_directives(depth),
        arb_union_member_types(1..6),
    )
        .prop_map(|(desc, name, dirs, members)| {
            let desc_str = desc.map_or(String::new(), |d| format!("{d}\n"));
            format!("{desc_str}union {name}{dirs} = {members}")
        })
        .boxed()
}

/// Generates an EnumTypeDefinition:
/// `enum Name @dirs { VALUE1 VALUE2 }`
///
/// See [Enum Type](https://spec.graphql.org/September2025/#sec-Enums).
pub fn arb_enum_type_definition(depth: usize) -> BoxedStrategy<String> {
    (
        arb_optional_description(),
        arb_type_name(),
        arb_const_directives(depth),
        arb_enum_value_definitions(depth, 1..8),
    )
        .prop_map(|(desc, name, dirs, values)| {
            let desc_str = desc.map_or(String::new(), |d| format!("{d}\n"));
            format!("{desc_str}enum {name}{dirs} {{\n  {values}\n}}")
        })
        .boxed()
}

/// Generates an InputObjectTypeDefinition:
/// `input Name @dirs { field1 field2 }`
///
/// See [Input Object Type](https://spec.graphql.org/September2025/#sec-Input-Objects).
pub fn arb_input_object_type_definition(depth: usize) -> BoxedStrategy<String> {
    (
        arb_optional_description(),
        arb_type_name(),
        arb_const_directives(depth),
        arb_input_value_definitions(depth),
    )
        .prop_map(|(desc, name, dirs, fields)| {
            let desc_str = desc.map_or(String::new(), |d| format!("{d}\n"));
            format!("{desc_str}input {name}{dirs} {{\n  {fields}\n}}")
        })
        .boxed()
}

/// Generates a SchemaDefinition:
/// `schema @dirs { query: Query mutation: Mutation }`
///
/// See [Schema](https://spec.graphql.org/September2025/#sec-Schema).
pub fn arb_schema_definition(depth: usize) -> BoxedStrategy<String> {
    (
        arb_optional_description(),
        arb_const_directives(depth),
        arb_root_operation_type_definitions(),
    )
        .prop_map(|(desc, dirs, ops)| {
            let desc_str = desc.map_or(String::new(), |d| format!("{d}\n"));
            format!("{desc_str}schema{dirs} {{\n  {ops}\n}}")
        })
        .boxed()
}

/// Generates any type definition (one of the 6 kinds).
pub fn arb_type_definition(depth: usize) -> BoxedStrategy<String> {
    prop_oneof![
        2 => arb_object_type_definition(depth),
        1 => arb_interface_type_definition(depth),
        1 => arb_union_type_definition(depth),
        1 => arb_enum_type_definition(depth),
        1 => arb_input_object_type_definition(depth),
        1 => arb_scalar_type_definition(depth),
    ]
    .boxed()
}

/// Generates any type-system definition (type def, schema def,
/// or directive def).
pub fn arb_type_system_definition(depth: usize) -> BoxedStrategy<String> {
    prop_oneof![
        5 => arb_type_definition(depth),
        1 => arb_schema_definition(depth),
        1 => arb_directive_definition(depth),
    ]
    .boxed()
}

// ─── Helpers ────────────────────────────────────────────────

/// Generates an `implements` clause: ` implements I1 & I2`.
fn arb_implements_interfaces(
    count: std::ops::Range<usize>,
) -> BoxedStrategy<String> {
    prop::collection::vec(arb_type_name(), count)
        .prop_map(|ifaces| {
            if ifaces.is_empty() {
                String::new()
            } else {
                format!(" implements {}", ifaces.join(" & "))
            }
        })
        .boxed()
}

/// Generates union member types: `Type1 | Type2 | Type3`.
fn arb_union_member_types(
    count: std::ops::Range<usize>,
) -> BoxedStrategy<String> {
    prop::collection::vec(arb_type_name(), count)
        .prop_map(|members| members.join(" | "))
        .boxed()
}

/// Generates root operation type definitions for schema bodies.
fn arb_root_operation_type_definitions() -> BoxedStrategy<String> {
    (
        prop::option::weighted(0.9, arb_type_name()),
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
