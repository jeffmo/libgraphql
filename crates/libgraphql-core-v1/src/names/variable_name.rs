use crate::names::graphql_name::GraphQLName;
use inherent::inherent;
use std::borrow::Borrow;

/// A GraphQL [variable name](https://spec.graphql.org/September2025/#sec-Language.Variables)
/// (e.g. `userId`, `limit`).
///
/// Variable names identify operation variables. The `$` prefix
/// used in GraphQL syntax is **not** stored — `VariableName`
/// holds only the bare identifier (e.g. `"userId"`, not
/// `"$userId"`). Using a dedicated newtype prevents accidental
/// mixing with other name domains like
/// [`FieldName`](crate::names::FieldName).
///
/// # Construction
///
/// ```ignore
/// use libgraphql_core::names::VariableName;
///
/// let name = VariableName::new("userId");
/// assert_eq!(name.as_str(), "userId");
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(serde::Deserialize, serde::Serialize)]
#[repr(transparent)]
pub struct VariableName(String);

#[inherent]
impl GraphQLName for VariableName {
    pub fn new(s: impl Into<String>) -> Self { Self(s.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

impl AsRef<str> for VariableName {
    fn as_ref(&self) -> &str { &self.0 }
}

impl Borrow<str> for VariableName {
    fn borrow(&self) -> &str { &self.0 }
}

impl std::fmt::Display for VariableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for VariableName {
    fn from(s: &str) -> Self { Self(s.to_owned()) }
}

impl From<String> for VariableName {
    fn from(s: String) -> Self { Self(s) }
}
