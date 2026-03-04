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

use proptest::prelude::*;
use proptest::strategy::BoxedStrategy;

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
fn arb_comment() -> BoxedStrategy<String> {
    "[a-zA-Z0-9 _-]{0,40}"
        .prop_map(|text| format!("# {text}\n"))
        .boxed()
}

/// Joins `(item, separator)` pairs into a single string.
///
/// The separator from each pair is placed before its corresponding
/// item, except for the first item (whose separator is unused).
/// Designed to work with `prop::collection::vec` of
/// `(item_strategy, arb_separator())` pairs.
pub fn join_items(pairs: &[(String, String)]) -> String {
    let mut result = String::new();
    for (i, (item, sep)) in pairs.iter().enumerate() {
        if i > 0 {
            result.push_str(sep);
        }
        result.push_str(item);
    }
    result
}
