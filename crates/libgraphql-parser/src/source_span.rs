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
pub struct SourceSpan {
    pub start_inclusive: SourcePosition,
    pub end_exclusive: SourcePosition,
    /// The file path to the source text this span refers to, if available.
    pub file_path: Option<PathBuf>,
}

impl SourceSpan {
    /// Creates a zero-position span with no file path.
    ///
    /// Used as a fallback when source position resolution is unavailable
    /// (e.g. errors constructed without a `SourceMap`).
    pub fn zero() -> Self {
        let zero_pos = SourcePosition::new(0, 0, None, 0);
        Self {
            start_inclusive: zero_pos,
            end_exclusive: zero_pos,
            file_path: None,
        }
    }

    /// Creates a span without file path information.
    pub fn new(start: SourcePosition, end: SourcePosition) -> Self {
        Self {
            start_inclusive: start,
            end_exclusive: end,
            file_path: None,
        }
    }

    /// Creates a span with file path information.
    pub fn with_file(start: SourcePosition, end: SourcePosition, file_path: PathBuf) -> Self {
        Self {
            start_inclusive: start,
            end_exclusive: end,
            file_path: Some(file_path),
        }
    }
}
