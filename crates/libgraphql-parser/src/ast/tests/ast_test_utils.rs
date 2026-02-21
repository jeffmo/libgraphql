//! Shared test helpers for constructing AST nodes in
//! unit tests.

use std::borrow::Cow;

use crate::ast::Name;
use crate::source_position::SourcePosition;
use crate::GraphQLSourceSpan;

/// Helper: build a `GraphQLSourceSpan` covering
/// `[start_byte, end_byte)` on line 0, col 0.
pub fn make_byte_span(
    start_byte: usize,
    end_byte: usize,
) -> GraphQLSourceSpan {
    GraphQLSourceSpan::new(
        SourcePosition::new(0, 0, None, start_byte),
        SourcePosition::new(0, 0, None, end_byte),
    )
}

/// Helper: build a `GraphQLSourceSpan` at the given
/// 0-based `line` and `col`, with zero byte offsets.
pub fn make_span(
    line: usize,
    col: usize,
) -> GraphQLSourceSpan {
    GraphQLSourceSpan::new(
        SourcePosition::new(line, col, None, 0),
        SourcePosition::new(line, col, None, 0),
    )
}

/// Helper: build a zero-width `GraphQLSourceSpan` at the
/// origin (line 0, col 0, byte 0).
pub fn zero_span() -> GraphQLSourceSpan {
    make_byte_span(0, 0)
}

/// Helper: build a `Name` borrowing from `value` with a
/// span of `[start, end)`.
///
/// Panics if `end - start` does not equal `value.len()`,
/// catching accidental span/value mismatches early.
pub fn make_name<'a>(
    value: &'a str,
    start: usize,
    end: usize,
) -> Name<'a> {
    assert_eq!(
        end - start,
        value.len(),
        "make_name: span length ({}) does not match \
         value length ({}) for {:?}",
        end - start,
        value.len(),
        value,
    );
    Name {
        value: Cow::Borrowed(value),
        span: make_byte_span(start, end),
        syntax: None,
    }
}
