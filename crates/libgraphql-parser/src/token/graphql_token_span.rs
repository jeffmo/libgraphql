use crate::SourcePosition;

/// Represents the span of a token from start to end position.
///
/// The span is a half-open interval: `[start_inclusive, end_exclusive)`.
/// - `start_inclusive`: Position of the first character of the token
/// - `end_exclusive`: Position immediately after the last character of the token
///
/// Fields are public to allow third-party `GraphQLTokenSource` implementations
/// to easily construct spans directly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphQLTokenSpan {
    pub start_inclusive: SourcePosition,
    pub end_exclusive: SourcePosition,
}
