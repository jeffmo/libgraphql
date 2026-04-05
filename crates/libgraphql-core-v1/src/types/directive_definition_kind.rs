/// Identifies whether a directive definition is one of the
/// built-in GraphQL directives or a custom (user-defined)
/// directive.
///
/// See [Built-in Directives](https://spec.graphql.org/September2025/#sec-Type-System.Directives.Built-in-Directives).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum DirectiveDefinitionKind {
    Custom,
    Deprecated,
    Include,
    OneOf,
    Skip,
    SpecifiedBy,
}

impl DirectiveDefinitionKind {
    pub fn is_builtin(&self) -> bool {
        !matches!(self, Self::Custom)
    }
}
