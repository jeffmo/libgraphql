use crate::SourcePosition;

/// Represents the span of some source text from start to end position.
///
/// The span is a half-open interval: `[start_inclusive, end_exclusive)`.
/// - `start_inclusive`: Position of the first character of the source text
/// - `end_exclusive`: Position immediately after the last character of the
///   source text
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphQLSourceSpan {
    pub start_inclusive: SourcePosition,
    pub end_exclusive: SourcePosition,
}
