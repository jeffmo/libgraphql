use crate::{SourceMap, SourceSpan};

/// A compact source span representing a half-open byte range `[start, end)`.
///
/// This is the primary span type stored on all tokens, AST nodes, and parse
/// errors. It is 8 bytes, `Copy`, and `#[repr(C)]` for predictable layout.
///
/// `ByteSpan` does **not** carry line/column/file information — those are
/// resolved on demand via [`SourceMap::resolve_span()`](crate::SourceMap::resolve_span).
///
/// # Indexing Convention
///
/// Both `start` and `end` are 0-based byte offsets from the beginning of the
/// source text. The range is half-open: `start` is the byte offset of the
/// first character, and `end` is the byte offset immediately after the last
/// character.
#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq)]
#[repr(C)]
pub struct ByteSpan {
    /// Byte offset of the first character (inclusive).
    pub start: u32,

    /// Byte offset immediately after the last character (exclusive).
    pub end: u32,
}

impl ByteSpan {
    /// Creates a new `ByteSpan` from start (inclusive) and end (exclusive)
    /// byte offsets.
    #[inline]
    pub fn new(start: u32, end: u32) -> Self {
        debug_assert!(
            start <= end,
            "ByteSpan::new called with start ({start}) > end ({end})",
        );
        Self { start, end }
    }

    /// Returns the length of this span in bytes.
    #[inline]
    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    /// Returns `true` if the span is empty (zero length).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Creates an empty span at the given byte offset.
    ///
    /// Useful for representing zero-width positions (e.g., EOF).
    #[inline]
    pub fn empty_at(offset: u32) -> Self {
        Self {
            start: offset,
            end: offset,
        }
    }

    /// Creates a span that covers both `self` and `other`.
    ///
    /// The resulting span starts at the minimum start and ends at the maximum
    /// end of the two spans.
    #[inline]
    pub fn merge(self, other: ByteSpan) -> ByteSpan {
        ByteSpan {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Resolves this byte span to a [`SourceSpan`] with
    /// line/column positions using the given [`SourceMap`].
    ///
    /// Returns [`None`] if the byte offsets cannot be resolved.
    /// Convenience wrapper for
    /// [`SourceMap::resolve_span()`].
    #[inline]
    pub fn resolve(&self, source_map: &SourceMap) -> Option<SourceSpan> {
        source_map.resolve_span(*self)
    }
}
