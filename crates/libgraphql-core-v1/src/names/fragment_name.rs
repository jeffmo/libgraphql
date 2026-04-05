use crate::names::graphql_name::GraphQLName;
use inherent::inherent;
use std::borrow::Borrow;

/// A GraphQL [fragment name](https://spec.graphql.org/September2025/#sec-Language.Fragments)
/// (e.g. `UserFields`, `PostSummary`).
///
/// Fragment names identify named fragment definitions and their
/// corresponding spreads in operations. Using a dedicated newtype
/// prevents accidental mixing with other name domains like
/// [`TypeName`](crate::names::TypeName) or
/// [`FieldName`](crate::names::FieldName).
///
/// # Construction
///
/// ```ignore
/// use libgraphql_core::names::FragmentName;
///
/// let name = FragmentName::new("UserFields");
/// assert_eq!(name.as_str(), "UserFields");
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(serde::Deserialize, serde::Serialize)]
#[repr(transparent)]
pub struct FragmentName(String);

#[inherent]
impl GraphQLName for FragmentName {
    pub fn new(s: impl Into<String>) -> Self { Self(s.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

impl AsRef<str> for FragmentName {
    fn as_ref(&self) -> &str { &self.0 }
}

impl Borrow<str> for FragmentName {
    fn borrow(&self) -> &str { &self.0 }
}

impl std::fmt::Display for FragmentName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for FragmentName {
    fn from(s: &str) -> Self { Self(s.to_owned()) }
}

impl From<String> for FragmentName {
    fn from(s: String) -> Self { Self(s) }
}
