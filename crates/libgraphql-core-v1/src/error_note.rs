use crate::span::Span;

/// An error note providing additional context about an error.
///
/// Notes augment the primary error message with:
/// - Explanatory context (why the error occurred)
/// - Actionable suggestions (how to fix it)
/// - Specification references (where to learn more)
/// - Related source locations (e.g., where a type was first
///   defined, or where a conflicting field exists)
///
/// This is the schema-layer analogue of
/// [`libgraphql_parser::GraphQLErrorNote`](libgraphql_parser::GraphQLErrorNote)
/// in the parser crate. The key difference is that spans here
/// are [`libgraphql_core::Span`]s (deferred byte-offset + source map ID) rather
/// than pre-resolved `SourceSpan`s, since schema errors may
/// reference locations across multiple source files loaded at
/// different times.
#[derive(Clone, Debug)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ErrorNote {
    /// The kind of note (determines rendering prefix).
    pub kind: ErrorNoteKind,

    /// The note message.
    pub message: String,

    /// Optional span pointing to a related source location.
    ///
    /// When present, the note is rendered with a source snippet
    /// pointing to this location. Resolution to line/column is
    /// deferred until display time via the schema's
    /// [`SchemaSourceMap`](crate::SchemaSourceMap) collection.
    pub span: Option<Span>,
}

impl ErrorNote {
    /// Creates a general note without a span.
    pub fn general(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorNoteKind::General,
            message: message.into(),
            span: None,
        }
    }

    /// Creates a general note with a span pointing to a related
    /// source location.
    pub fn general_with_span(
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self {
            kind: ErrorNoteKind::General,
            message: message.into(),
            span: Some(span),
        }
    }

    /// Creates a help note without a span.
    pub fn help(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorNoteKind::Help,
            message: message.into(),
            span: None,
        }
    }

    /// Creates a help note with a span.
    pub fn help_with_span(
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self {
            kind: ErrorNoteKind::Help,
            message: message.into(),
            span: Some(span),
        }
    }

    /// Creates a spec-reference note containing a URL to the
    /// relevant section of the GraphQL specification.
    pub fn spec(url: impl Into<String>) -> Self {
        Self {
            kind: ErrorNoteKind::Spec,
            message: url.into(),
            span: None,
        }
    }
}

/// The kind of an error note (determines how the note is
/// rendered).
///
/// Notes provide additional context beyond the primary error
/// message. Different kinds are rendered with different prefixes
/// in CLI output and may be handled differently by IDEs or other
/// tools.
#[derive(Clone, Copy, Debug)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum ErrorNoteKind {
    /// General context or explanation about the error.
    ///
    /// Rendered as `= note: ...` in CLI output.
    /// Example: "first defined here" (with span pointing to the
    /// original definition)
    General,

    /// Actionable suggestion for fixing the error.
    ///
    /// Rendered as `= help: ...` in CLI output.
    /// Example: "rename one of the duplicate fields"
    Help,

    #[allow(rustdoc::bare_urls)]
    /// Reference to the GraphQL specification.
    ///
    /// Rendered as `= spec: ...` in CLI output.
    /// Example: "https://spec.graphql.org/September2025/#sec-Objects"
    Spec,
}
