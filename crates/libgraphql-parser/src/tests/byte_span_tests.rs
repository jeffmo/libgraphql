use crate::ByteSpan;
use crate::GraphQLSourceSpan;
use crate::SourcePosition;

/// Verifies that ByteSpan::new() constructs a span with the
/// correct start and end byte offsets.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn byte_span_new_stores_offsets() {
    let span = ByteSpan::new(10, 25);
    assert_eq!(span.start, 10);
    assert_eq!(span.end, 25);
}

/// Verifies that ByteSpan::len() correctly computes the byte
/// length as end - start.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn byte_span_len() {
    let span = ByteSpan::new(5, 15);
    assert_eq!(span.len(), 10);
}

/// Verifies that a zero-width span (start == end) reports
/// len() == 0 and is_empty() == true.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn byte_span_zero_width() {
    let span = ByteSpan::new(42, 42);
    assert_eq!(span.len(), 0);
    assert!(span.is_empty());
}

/// Verifies that a non-empty span reports is_empty() == false.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn byte_span_non_empty() {
    let span = ByteSpan::new(0, 1);
    assert!(!span.is_empty());
}

/// Verifies that ByteSpan implements Copy (can be duplicated
/// without move semantics), which is important for an 8-byte
/// value type stored on every AST node.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn byte_span_is_copy() {
    let span = ByteSpan::new(0, 10);
    let copy = span;
    // Both `span` and `copy` are usable — proves Copy.
    assert_eq!(span, copy);
}

/// Verifies that ByteSpan equality works correctly for both
/// equal and non-equal spans.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn byte_span_equality() {
    let a = ByteSpan::new(5, 10);
    let b = ByteSpan::new(5, 10);
    let c = ByteSpan::new(5, 11);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

/// Verifies that GraphQLSourceSpan::byte_span() correctly
/// extracts byte offsets from start/end SourcePositions into
/// a compact ByteSpan, discarding line/column and file path
/// information.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn graphql_source_span_to_byte_span() {
    let start = SourcePosition::new(
        /* line = */ 2,
        /* col_utf8 = */ 5,
        /* col_utf16 = */ Some(5),
        /* byte_offset = */ 30,
    );
    let end = SourcePosition::new(
        /* line = */ 2,
        /* col_utf8 = */ 12,
        /* col_utf16 = */ Some(12),
        /* byte_offset = */ 37,
    );
    let source_span = GraphQLSourceSpan::new(start, end);
    let byte_span = source_span.byte_span();

    assert_eq!(byte_span.start, 30);
    assert_eq!(byte_span.end, 37);
    assert_eq!(byte_span.len(), 7);
}

/// Verifies that GraphQLSourceSpan::byte_span() works correctly
/// when the source span includes a file path — the file path
/// is discarded in the ByteSpan since it will live on the
/// SourceMap instead.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn graphql_source_span_to_byte_span_with_file_path() {
    let start = SourcePosition::new(0, 0, Some(0), 0);
    let end = SourcePosition::new(0, 5, Some(5), 5);
    let source_span = GraphQLSourceSpan::with_file(
        start,
        end,
        std::path::PathBuf::from("schema.graphql"),
    );
    let byte_span = source_span.byte_span();

    // File path is discarded — ByteSpan only has offsets.
    assert_eq!(byte_span.start, 0);
    assert_eq!(byte_span.end, 5);
}
