/// The kind of GraphQL document being parsed.
///
/// Different document kinds allow different definition types:
/// - Schema documents: only type system definitions
/// - Executable documents: only operations and fragments
/// - Mixed documents: both type system and executable definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
    /// Schema document: only type system definitions allowed
    /// (`schema`, `type`, `interface`, `directive`, etc.).
    Schema,

    /// Executable document: only operations and fragments allowed
    /// (`query`, `mutation`, `subscription`, `fragment`).
    Executable,

    /// Mixed document: both type system and executable definitions allowed.
    /// This is useful for tools that process complete GraphQL codebases.
    Mixed,
}
