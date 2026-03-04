//! Negative-case mutation strategies that break valid GraphQL
//! documents in structured ways.
//!
//! These strategies take valid source text and introduce specific
//! kinds of errors to test the parser's error detection and
//! recovery.
//!
//! Written by Claude Code, reviewed by a human.

#![allow(dead_code)]

use proptest::prelude::*;
use proptest::strategy::BoxedStrategy;

/// Truncates a valid document at a random byte position.
///
/// This tests that the parser correctly reports errors when
/// encountering an unexpected end of input.
pub fn arb_truncated(source: String) -> BoxedStrategy<String> {
    let len = source.len();
    if len <= 1 {
        return Just(String::new()).boxed();
    }
    // Truncate at a random position from 1..len-1 to ensure we
    // actually remove something but keep at least one character
    (1..len)
        .prop_map(move |pos| {
            // Find a valid char boundary at or before `pos`
            let mut boundary = pos;
            while boundary > 0 && !source.is_char_boundary(boundary) {
                boundary -= 1;
            }
            source[..boundary].to_string()
        })
        .boxed()
}

/// Swaps matching delimiters in the source to create mismatched
/// delimiter errors.
///
/// For example, `{` becomes `[` and `}` becomes `]`, or
/// `(` becomes `{` and `)` becomes `}`.
pub fn arb_delimiter_swap(source: String) -> BoxedStrategy<String> {
    prop_oneof![
        Just(source.replace('{', "[")),
        Just(source.replace('}', "]")),
        Just(source.replace('(', "{")),
        Just(source.replace(')', "}")),
        Just(source.replace('{', "(")),
    ]
    .prop_filter("mutation must change the source", {
        let original = source.clone();
        move |mutated| mutated != &original
    })
    .boxed()
}

/// Inserts garbage text at a random position in the source.
pub fn arb_garbage_insertion(source: String) -> BoxedStrategy<String> {
    let len = source.len();
    if len == 0 {
        return Just("@@@GARBAGE@@@".to_string()).boxed();
    }
    let garbage_options = vec![
        "???".to_string(),
        "@@@".to_string(),
        "~~~".to_string(),
        "!!!".to_string(),
        "<<<>>>".to_string(),
    ];
    (0..len, prop::sample::select(garbage_options))
        .prop_map(move |(pos, garbage)| {
            let mut boundary = pos;
            while boundary > 0 && !source.is_char_boundary(boundary) {
                boundary -= 1;
            }
            let mut result = source[..boundary].to_string();
            result.push_str(&garbage);
            result.push_str(&source[boundary..]);
            result
        })
        .boxed()
}

/// Creates a fragment definition with the reserved name `on`.
///
/// Per the spec, `on` is not a valid fragment name. This tests
/// that the parser correctly rejects it.
pub fn arb_reserved_fragment_name() -> BoxedStrategy<String> {
    Just("fragment on on SomeType { field }".to_string()).boxed()
}
