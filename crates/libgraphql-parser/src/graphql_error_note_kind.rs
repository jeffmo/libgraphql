/// The kind of an error note (determinies how the note is rendered).
///
/// Notes provide additional context beyond the primary error message.
/// Different kinds are rendered with different prefixes in CLI output
/// and may be handled differently by IDEs or other tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphQLErrorNoteKind {
    /// General context or explanation about the error.
    ///
    /// Rendered as `= note: ...` in CLI output.
    /// Example: "Opening `{` here" (with span pointing to the opener)
    General,

    /// Actionable suggestion for fixing the error.
    ///
    /// Rendered as `= help: ...` in CLI output.
    /// Example: "Did you mean: `userName: String`?"
    Help,

    /// Reference to the GraphQL specification.
    ///
    /// Rendered as `= spec: ...` in CLI output.
    /// Example: "https://spec.graphql.org/September2025/#FieldDefinition"
    Spec,
}
