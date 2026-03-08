// ByteSpan unit tests.
// Written by Claude Code, reviewed by a human.

use crate::ByteSpan;

/// Verifies that ByteSpan is exactly 8 bytes (two u32s with #[repr(C)]).
/// This is architecture-independent: u32 is always 4 bytes, and #[repr(C)]
/// guarantees no padding between two identically-aligned fields.
#[test]
fn byte_span_size_is_8_bytes() {
    assert_eq!(std::mem::size_of::<ByteSpan>(), 8);
}

/// Verifies basic construction and field access.
#[test]
fn byte_span_new_and_fields() {
    let span = ByteSpan::new(10, 25);
    assert_eq!(span.start, 10);
    assert_eq!(span.end, 25);
    assert_eq!(span.len(), 15);
    assert!(!span.is_empty());
}

/// Verifies empty_at creates a zero-width span at the given offset.
#[test]
fn byte_span_empty_at() {
    let span = ByteSpan::empty_at(42);
    assert_eq!(span.start, 42);
    assert_eq!(span.end, 42);
    assert_eq!(span.len(), 0);
    assert!(span.is_empty());
}

/// Verifies default creates a zero-width span at offset 0.
#[test]
fn byte_span_default() {
    let span = ByteSpan::default();
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 0);
    assert!(span.is_empty());
}

/// Verifies merge produces a span covering both inputs.
#[test]
fn byte_span_merge() {
    let a = ByteSpan::new(5, 10);
    let b = ByteSpan::new(8, 20);
    let merged = a.merge(b);
    assert_eq!(merged.start, 5);
    assert_eq!(merged.end, 20);
}

/// Verifies merge with non-overlapping spans.
#[test]
fn byte_span_merge_non_overlapping() {
    let a = ByteSpan::new(0, 5);
    let b = ByteSpan::new(10, 15);
    let merged = a.merge(b);
    assert_eq!(merged.start, 0);
    assert_eq!(merged.end, 15);
}

/// Verifies merge is commutative.
#[test]
fn byte_span_merge_commutative() {
    let a = ByteSpan::new(3, 7);
    let b = ByteSpan::new(1, 12);
    assert_eq!(a.merge(b), b.merge(a));
}

/// Verifies that ByteSpan implements Copy (no clone needed).
#[test]
fn byte_span_is_copy() {
    let span = ByteSpan::new(1, 2);
    let span2 = span; // Copy, not move
    assert_eq!(span, span2); // original still usable
}

/// Verifies equality and hashing.
#[test]
fn byte_span_eq_and_hash() {
    use std::collections::HashSet;
    let a = ByteSpan::new(0, 10);
    let b = ByteSpan::new(0, 10);
    let c = ByteSpan::new(0, 11);
    assert_eq!(a, b);
    assert_ne!(a, c);

    let mut set = HashSet::new();
    set.insert(a);
    assert!(set.contains(&b));
    assert!(!set.contains(&c));
}
