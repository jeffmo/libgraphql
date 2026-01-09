/// The kind of definition found in a GraphQL document.
///
/// Used for error reporting and programmatic categorization of definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionKind {
    /// `schema { ... }` or `extend schema { ... }`
    Schema,

    /// Type definitions: `type`, `interface`, `union`, `enum`, `scalar`,
    /// `input`, or their `extend` variants.
    TypeDefinition,

    /// `directive @name on ...`
    DirectiveDefinition,

    /// Operations: `query`, `mutation`, `subscription`, or anonymous `{ ... }`
    Operation,

    /// `fragment Name on Type { ... }`
    Fragment,
}
