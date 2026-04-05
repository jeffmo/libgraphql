use crate::names::graphql_name::GraphQLName;
use inherent::inherent;
use std::borrow::Borrow;

/// A GraphQL [field or argument name](https://spec.graphql.org/September2025/#sec-Names)
/// (e.g. `firstName`, `id`, `if`).
///
/// Field names identify fields on object types, interface types,
/// and input object types. They also name arguments (parameters)
/// on fields and directives. Using a dedicated newtype prevents
/// accidental mixing with other name domains like
/// [`TypeName`](crate::names::TypeName) or
/// [`DirectiveName`](crate::names::DirectiveName).
///
/// # Construction
///
/// ```ignore
/// use libgraphql_core::names::FieldName;
///
/// let name = FieldName::new("firstName");
/// assert_eq!(name.as_str(), "firstName");
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(serde::Deserialize, serde::Serialize)]
#[repr(transparent)]
pub struct FieldName(String);

#[inherent]
impl GraphQLName for FieldName {
    pub fn new(s: impl Into<String>) -> Self { Self(s.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

impl AsRef<str> for FieldName {
    fn as_ref(&self) -> &str { &self.0 }
}

impl Borrow<str> for FieldName {
    fn borrow(&self) -> &str { &self.0 }
}

impl std::fmt::Display for FieldName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for FieldName {
    fn from(s: &str) -> Self { Self(s.to_owned()) }
}

impl From<String> for FieldName {
    fn from(s: String) -> Self { Self(s) }
}
