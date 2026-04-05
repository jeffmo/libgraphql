use crate::names::graphql_name::GraphQLName;
use inherent::inherent;
use std::borrow::Borrow;

/// A GraphQL [type name](https://spec.graphql.org/September2025/#sec-Names)
/// (e.g. `User`, `String`, `Query`).
///
/// Type names identify schema-defined types: object types, interfaces,
/// unions, enums, scalars, and input objects. Using a dedicated newtype
/// prevents accidental mixing with other name domains like
/// [`FieldName`](crate::names::FieldName) or
/// [`VariableName`](crate::names::VariableName).
///
/// # Construction
///
/// ```ignore
/// use libgraphql_core::names::TypeName;
///
/// let name = TypeName::new("User");
/// assert_eq!(name.as_str(), "User");
///
/// let from_str: TypeName = "Query".into();
/// let from_string: TypeName = String::from("Query").into();
/// assert_eq!(from_str, from_string);
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(serde::Deserialize, serde::Serialize)]
#[repr(transparent)]
pub struct TypeName(String);

#[inherent]
impl GraphQLName for TypeName {
    pub fn new(s: impl Into<String>) -> Self { Self(s.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

impl AsRef<str> for TypeName {
    fn as_ref(&self) -> &str { &self.0 }
}

impl Borrow<str> for TypeName {
    fn borrow(&self) -> &str { &self.0 }
}

impl std::fmt::Display for TypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for TypeName {
    fn from(s: &str) -> Self { Self(s.to_owned()) }
}

impl From<String> for TypeName {
    fn from(s: String) -> Self { Self(s) }
}
