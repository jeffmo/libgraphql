/// Compact byte-offset span. 8 bytes per node.
///
/// Represents a half-open interval `[start, end)` of byte offsets
/// into a byte-array of source text. Both offsets are 0-based.
///
/// `u32` offsets support documents up to 4 GiB, which is sufficient
/// for any GraphQL document (the largest public schema we could
/// find — Shopify's Admin API — is ~3.1 MB).
///
/// `#[repr(C)]` ensures a predictable memory layout for FFI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct ByteSpan {
    /// Byte offset of the first byte of this node in a byte-array
    /// of source text (0-based, inclusive).
    pub start: u32,
    /// Byte offset one past the last byte of this node in a
    /// byte-array of source text (0-based, exclusive).
    pub end: u32,
}

impl ByteSpan {
    /// Creates a new `ByteSpan` from start (inclusive) and end
    /// (exclusive) byte offsets.
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Returns the length of this span in bytes.
    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    /// Returns `true` if this span has zero length.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}
