//! Shared test helpers for constructing AST nodes in
//! unit tests.

use std::borrow::Cow;

use crate::ast::Name;
use crate::ByteSpan;

/// Helper: build a `ByteSpan` covering
/// `[start_byte, end_byte)`.
pub fn make_byte_span(
    start_byte: usize,
    end_byte: usize,
) -> ByteSpan {
    ByteSpan::new(start_byte as u32, end_byte as u32)
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
