use crate::names::graphql_name::GraphQLName;
use inherent::inherent;
use std::borrow::Borrow;

/// A GraphQL [directive name](https://spec.graphql.org/September2025/#sec-Type-System.Directives)
/// (e.g. `deprecated`, `skip`, `include`).
///
/// Directive names identify directives in the schema and in
/// operations. The `@` prefix used in GraphQL syntax is **not**
/// stored — `DirectiveName` holds only the bare identifier
/// (e.g. `"deprecated"`, not `"@deprecated"`). Using a dedicated
/// newtype prevents accidental mixing with other name domains
/// like [`TypeName`](crate::names::TypeName).
///
/// # Construction
///
/// ```ignore
/// use libgraphql_core::names::DirectiveName;
///
/// let name = DirectiveName::new("deprecated");
/// assert_eq!(name.as_str(), "deprecated");
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(serde::Deserialize, serde::Serialize)]
#[repr(transparent)]
pub struct DirectiveName(String);

#[inherent]
impl GraphQLName for DirectiveName {
    pub fn new(s: impl Into<String>) -> Self { Self(s.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

impl AsRef<str> for DirectiveName {
    fn as_ref(&self) -> &str { &self.0 }
}

impl Borrow<str> for DirectiveName {
    fn borrow(&self) -> &str { &self.0 }
}

impl std::fmt::Display for DirectiveName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for DirectiveName {
    fn from(s: &str) -> Self { Self(s.to_owned()) }
}

impl From<String> for DirectiveName {
    fn from(s: String) -> Self { Self(s) }
}
