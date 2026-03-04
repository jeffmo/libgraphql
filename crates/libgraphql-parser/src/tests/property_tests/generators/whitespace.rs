//! Strategies for generating insignificant whitespace, commas,
//! and comments.
//!
//! In GraphQL, commas are treated as insignificant whitespace, and
//! comments (`# ... \n`) are ignored by the parser. These strategies
//! inject valid trivia between tokens to exercise the parser's
//! whitespace handling.
//!
//! See [Insignificant Commas](https://spec.graphql.org/September2025/#sec-Insignificant-Commas)
//! and [Comments](https://spec.graphql.org/September2025/#sec-Comments)
//! in the spec.
//!
//! Written by Claude Code, reviewed by a human.

#![allow(dead_code)]

use proptest::prelude::*;
use proptest::strategy::BoxedStrategy;

/// Generates a single whitespace token (space, tab, or newline).
pub fn arb_whitespace_char() -> BoxedStrategy<String> {
    prop_oneof![
        Just(" ".to_string()),
        Just("\t".to_string()),
        Just("\n".to_string()),
    ]
    .boxed()
}

/// Generates a stretch of insignificant whitespace (1-3 chars).
pub fn arb_whitespace() -> BoxedStrategy<String> {
    prop_oneof![
        Just(" ".to_string()),
        Just("  ".to_string()),
        Just("\n".to_string()),
        Just(" \n ".to_string()),
        Just("\t".to_string()),
    ]
    .boxed()
}

/// Generates optional insignificant separator: whitespace, comma,
/// or comment. In GraphQL, commas are insignificant separators.
pub fn arb_separator() -> BoxedStrategy<String> {
    prop_oneof![
        4 => Just(" ".to_string()),
        2 => Just(", ".to_string()),
        2 => Just(",".to_string()),
        1 => Just(" , ".to_string()),
        1 => arb_comment(),
    ]
    .boxed()
}

/// Generates a comment: `# text \n`.
///
/// Comment text is restricted to safe ASCII to avoid encoding issues.
pub fn arb_comment() -> BoxedStrategy<String> {
    "[a-zA-Z0-9 _-]{0,40}"
        .prop_map(|text| format!("# {text}\n"))
        .boxed()
}

/// Generates optional whitespace (empty or some whitespace).
pub fn arb_optional_whitespace() -> BoxedStrategy<String> {
    prop_oneof![
        3 => Just(String::new()),
        1 => arb_whitespace(),
    ]
    .boxed()
}

/// Generates required whitespace between tokens that need it.
pub fn arb_required_whitespace() -> BoxedStrategy<String> {
    prop_oneof![
        4 => Just(" ".to_string()),
        1 => Just("  ".to_string()),
        1 => Just("\n".to_string()),
    ]
    .boxed()
}
