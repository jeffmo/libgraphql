use crate::span::BUILTIN_SOURCE_MAP_ID;
use crate::span::SourceMapId;
use crate::span::Span;
use libgraphql_parser::ByteSpan;

// Verifies Span construction and field access.
// Written by Claude Code, reviewed by a human.
#[test]
fn span_construction() {
    let span = Span::new(ByteSpan::new(10, 20), SourceMapId(1));
    assert_eq!(span.byte_span.start, 10);
    assert_eq!(span.byte_span.end, 20);
    assert_eq!(span.source_map_id, SourceMapId(1));
}

// Verifies builtin span has source_map_id 0 and empty byte span.
// Written by Claude Code, reviewed by a human.
#[test]
fn builtin_span() {
    let span = Span::builtin();
    assert_eq!(span.source_map_id, BUILTIN_SOURCE_MAP_ID);
    assert!(span.byte_span.is_empty());
}

// Verifies Span is Copy (important for zero-cost storage on every
// semantic node).
// Written by Claude Code, reviewed by a human.
#[test]
fn span_is_copy() {
    let span = Span::builtin();
    let copy = span;
    assert_eq!(span, copy);
}

// Verifies serde round-trip via bincode for Span.
// Written by Claude Code, reviewed by a human.
#[test]
fn span_serde_roundtrip() {
    let span = Span::new(ByteSpan::new(5, 15), SourceMapId(3));
    let bytes = bincode::serde::encode_to_vec(
        span,
        bincode::config::standard(),
    ).unwrap();
    let (deserialized, _): (Span, _) =
        bincode::serde::decode_from_slice(
            &bytes,
            bincode::config::standard(),
        ).unwrap();
    assert_eq!(span, deserialized);
}
