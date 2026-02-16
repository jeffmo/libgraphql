use crate::ByteSpan;
use crate::SourcePosition;
use std::path::PathBuf;

/// Represents a span of source text from start to end position.
///
/// The span is a half-open interval: `[start_inclusive, end_exclusive)`.
/// - `start_inclusive`: Position of the first character of the source text
/// - `end_exclusive`: Position immediately after the last character
///
/// Optionally includes a file path for the referenced source text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphQLSourceSpan {
    pub start_inclusive: SourcePosition,
    pub end_exclusive: SourcePosition,
    /// The file path to the source text this span refers to, if available.
    pub file_path: Option<PathBuf>,
}

impl GraphQLSourceSpan {
    /// Creates a span without file path information.
    pub fn new(start: SourcePosition, end: SourcePosition) -> Self {
        Self {
            start_inclusive: start,
            end_exclusive: end,
            file_path: None,
        }
    }

    /// Creates a span with file path information.
    pub fn with_file(
        start: SourcePosition,
        end: SourcePosition,
        file_path: PathBuf,
    ) -> Self {
        Self {
            start_inclusive: start,
            end_exclusive: end,
            file_path: Some(file_path),
        }
    }

    /// Extracts a compact `ByteSpan` from this span's byte
    /// offsets, discarding line/column and file path information.
    pub fn byte_span(&self) -> ByteSpan {
        ByteSpan {
            start: self.start_inclusive.byte_offset() as u32,
            end: self.end_exclusive.byte_offset() as u32,
        }
    }
}
