//! Strategies for generating all 9 GraphQL Value variants.
//!
//! See [Input Values](https://spec.graphql.org/September2025/#sec-Input-Values)
//! in the spec.
//!
//! Written by Claude Code, reviewed by a human.

use proptest::prelude::*;
use proptest::strategy::BoxedStrategy;

use crate::tests::property_tests::generators::names::arb_enum_value_name;
use crate::tests::property_tests::generators::names::arb_field_name;
use crate::tests::property_tests::generators::names::arb_name;

/// Generates a valid GraphQL IntValue.
///
/// Per the spec, `IntValue` is `IntegerPart` which is an optional
/// negative sign followed by `0` or a non-zero digit followed by
/// more digits.
pub fn arb_int_value() -> BoxedStrategy<String> {
    prop_oneof![
        Just("0".to_string()),
        (-999_999i32..999_999i32).prop_map(|n| n.to_string()),
    ]
    .boxed()
}

/// Generates a valid GraphQL FloatValue.
///
/// A FloatValue must have either a fractional part or an exponent
/// part (or both) to distinguish it from an IntValue.
pub fn arb_float_value() -> BoxedStrategy<String> {
    prop_oneof![
        // IntegerPart . Digits
        (-999i32..999, 0u32..999_999).prop_map(|(int, frac)| {
            format!("{int}.{frac}")
        }),
        // IntegerPart ExponentPart
        (-999i32..999, -10i32..10).prop_map(|(int, exp)| {
            format!("{int}e{exp}")
        }),
        // IntegerPart . Digits ExponentPart
        (-99i32..99, 0u32..9999, -5i32..5).prop_map(|(int, frac, exp)| {
            format!("{int}.{frac}e{exp}")
        }),
    ]
    .boxed()
}

/// Generates a valid GraphQL single-line StringValue.
///
/// Restricted to safe ASCII with proper escape sequences.
pub fn arb_string_value() -> BoxedStrategy<String> {
    prop_oneof![
        // Simple strings with safe ASCII
        "[a-zA-Z0-9 _.,!?:;@#$%^&*()+=/<>-]{0,30}"
            .prop_map(|s| format!("\"{}\"", escape_string_chars(&s))),
        // Empty string
        Just("\"\"".to_string()),
        // String with escape sequences
        prop::collection::vec(arb_string_char(), 1..10)
            .prop_map(|chars| format!("\"{}\"", chars.join(""))),
    ]
    .boxed()
}

/// Generates a single character or escape sequence for a string value.
fn arb_string_char() -> BoxedStrategy<String> {
    prop_oneof![
        4 => "[a-zA-Z0-9 ]".prop_map(|s| s.to_string()),
        1 => Just("\\n".to_string()),
        1 => Just("\\t".to_string()),
        1 => Just("\\\\".to_string()),
        1 => Just("\\\"".to_string()),
        1 => Just("\\/".to_string()),
    ]
    .boxed()
}

/// Escapes characters that need escaping in a GraphQL string.
fn escape_string_chars(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            _ => result.push(ch),
        }
    }
    result
}

/// Generates a valid GraphQL block string (`"""..."""`).
///
/// Block strings use triple-quote delimiters and have special
/// escaping rules: only `\"""` is treated as an escape.
pub fn arb_block_string_value() -> BoxedStrategy<String> {
    "[a-zA-Z0-9 _.,!?:;@#$%^&*()+=/<>\\n-]{0,50}"
        .prop_filter("no triple quotes", |s| !s.contains("\"\"\""))
        .prop_map(|s| format!("\"\"\"{s}\"\"\""))
        .boxed()
}

/// Generates a BooleanValue (`true` or `false`).
pub fn arb_boolean_value() -> BoxedStrategy<String> {
    prop_oneof![Just("true".to_string()), Just("false".to_string()),].boxed()
}

/// Generates a NullValue.
pub fn arb_null_value() -> BoxedStrategy<String> {
    Just("null".to_string()).boxed()
}

/// Generates an EnumValue (a name that is not `true`, `false`,
/// or `null`).
pub fn arb_enum_value() -> BoxedStrategy<String> {
    arb_enum_value_name()
}

/// Generates a Variable reference (`$name`).
pub fn arb_variable_value() -> BoxedStrategy<String> {
    arb_name().prop_map(|n| format!("${n}")).boxed()
}

/// Generates any GraphQL Value at the given recursion depth.
///
/// At depth 0, only scalar (non-recursive) values are generated.
/// At depth > 0, List and Object values may recurse.
pub fn arb_value(depth: usize) -> BoxedStrategy<String> {
    if depth == 0 {
        arb_scalar_value_or_variable()
    } else {
        prop_oneof![
            4 => arb_scalar_value_or_variable(),
            1 => arb_list_value(depth - 1),
            1 => arb_object_value(depth - 1),
        ]
        .boxed()
    }
}

/// Generates a const Value (no variables allowed) at the given depth.
///
/// Const values appear in default values, directive arguments on
/// type definitions, and other non-executable contexts.
/// See [ConstValue](https://spec.graphql.org/September2025/#ConstValue).
pub fn arb_const_value(depth: usize) -> BoxedStrategy<String> {
    if depth == 0 {
        arb_scalar_value()
    } else {
        prop_oneof![
            4 => arb_scalar_value(),
            1 => arb_const_list_value(depth - 1),
            1 => arb_const_object_value(depth - 1),
        ]
        .boxed()
    }
}

/// Generates a scalar value (no List/Object) including variables.
fn arb_scalar_value_or_variable() -> BoxedStrategy<String> {
    prop_oneof![
        2 => arb_int_value(),
        2 => arb_float_value(),
        2 => arb_string_value(),
        1 => arb_block_string_value(),
        1 => arb_boolean_value(),
        1 => arb_null_value(),
        2 => arb_enum_value(),
        1 => arb_variable_value(),
    ]
    .boxed()
}

/// Generates a scalar value (no List/Object, no Variable).
fn arb_scalar_value() -> BoxedStrategy<String> {
    prop_oneof![
        2 => arb_int_value(),
        2 => arb_float_value(),
        2 => arb_string_value(),
        1 => arb_block_string_value(),
        1 => arb_boolean_value(),
        1 => arb_null_value(),
        2 => arb_enum_value(),
    ]
    .boxed()
}

/// Generates a ListValue: `[value, ...]`.
fn arb_list_value(depth: usize) -> BoxedStrategy<String> {
    prop::collection::vec(arb_value(depth), 0..4)
        .prop_map(|vals| format!("[{}]", vals.join(", ")))
        .boxed()
}

/// Generates an ObjectValue: `{name: value, ...}`.
fn arb_object_value(depth: usize) -> BoxedStrategy<String> {
    prop::collection::vec(
        (arb_field_name(), arb_value(depth)),
        0..4,
    )
    .prop_map(|fields| {
        let entries: Vec<String> = fields
            .into_iter()
            .map(|(name, val)| format!("{name}: {val}"))
            .collect();
        format!("{{{}}}", entries.join(", "))
    })
    .boxed()
}

/// Generates a const ListValue (no variables).
fn arb_const_list_value(depth: usize) -> BoxedStrategy<String> {
    prop::collection::vec(arb_const_value(depth), 0..4)
        .prop_map(|vals| format!("[{}]", vals.join(", ")))
        .boxed()
}

/// Generates a const ObjectValue (no variables).
fn arb_const_object_value(depth: usize) -> BoxedStrategy<String> {
    prop::collection::vec(
        (arb_field_name(), arb_const_value(depth)),
        0..4,
    )
    .prop_map(|fields| {
        let entries: Vec<String> = fields
            .into_iter()
            .map(|(name, val)| format!("{name}: {val}"))
            .collect();
        format!("{{{}}}", entries.join(", "))
    })
    .boxed()
}
