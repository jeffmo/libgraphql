use crate::GraphQLErrorNoteKind;
use crate::SourceSpan;

/// An error note providing additional context about an error.
///
/// Notes augment the primary error message with:
/// - Explanatory context (why the error occurred)
/// - Actionable suggestions (how to fix it)
/// - Specification references (where to learn more)
/// - Related source locations (e.g., where a delimiter was opened)
#[derive(Debug, Clone, PartialEq)]
pub struct GraphQLErrorNote {
    /// The kind of note (determines rendering prefix).
    pub kind: GraphQLErrorNoteKind,

    /// The note message.
    pub message: String,

    /// Optional pre-resolved span pointing to a related location.
    ///
    /// When present, the note is rendered with a source snippet
    /// pointing to this location. Like the primary error's
    /// `source_span`, this is eagerly resolved at error
    /// construction time so it carries line/column/byte-offset
    /// information without requiring a `SourceMap` at display time.
    pub span: Option<SourceSpan>,
}

impl GraphQLErrorNote {
    /// Creates a general note without a span.
    pub fn general(message: impl Into<String>) -> Self {
        Self {
            kind: GraphQLErrorNoteKind::General,
            message: message.into(),
            span: None,
        }
    }

    /// Creates a general note with a pre-resolved span.
    pub fn general_with_span(
        message: impl Into<String>,
        span: SourceSpan,
    ) -> Self {
        Self {
            kind: GraphQLErrorNoteKind::General,
            message: message.into(),
            span: Some(span),
        }
    }

    /// Creates a help note without a span.
    pub fn help(message: impl Into<String>) -> Self {
        Self {
            kind: GraphQLErrorNoteKind::Help,
            message: message.into(),
            span: None,
        }
    }

    /// Creates a help note with a pre-resolved span.
    pub fn help_with_span(
        message: impl Into<String>,
        span: SourceSpan,
    ) -> Self {
        Self {
            kind: GraphQLErrorNoteKind::Help,
            message: message.into(),
            span: Some(span),
        }
    }

    /// Creates a spec reference note.
    pub fn spec(url: impl Into<String>) -> Self {
        Self {
            kind: GraphQLErrorNoteKind::Spec,
            message: url.into(),
            span: None,
        }
    }
}
