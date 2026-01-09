/// Contexts where certain names are reserved in GraphQL.
///
/// Some names have special meaning in specific contexts and cannot be used
/// as identifiers there. This enum is used by `GraphQLParseErrorKind::ReservedName`
/// to indicate which context rejected the name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReservedNameContext {
    /// Fragment names cannot be `on` (it introduces the type condition).
    ///
    /// Invalid: `fragment on on User { ... }`
    /// The first `on` would be parsed as the fragment name, but `on` is
    /// reserved in this context.
    FragmentName,

    /// Enum values cannot be `true`, `false`, or `null`.
    ///
    /// Invalid: `enum Bool { true false }` or `enum Maybe { null some }`
    /// These would be ambiguous with boolean/null literals in value contexts.
    EnumValue,
}
