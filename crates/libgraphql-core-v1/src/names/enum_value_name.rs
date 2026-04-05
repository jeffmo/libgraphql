use crate::names::graphql_name::GraphQLName;
use inherent::inherent;
use std::borrow::Borrow;

/// A GraphQL [enum value name](https://spec.graphql.org/September2025/#EnumValuesDefinition)
/// (e.g. `ACTIVE`, `ADMIN`, `PUBLISHED`).
///
/// Enum value names identify individual values within an enum
/// type definition. Using a dedicated newtype prevents accidental
/// mixing with other name domains like
/// [`FieldName`](crate::names::FieldName) or
/// [`TypeName`](crate::names::TypeName).
///
/// # Construction
///
/// ```ignore
/// use libgraphql_core::names::EnumValueName;
///
/// let name = EnumValueName::new("ACTIVE");
/// assert_eq!(name.as_str(), "ACTIVE");
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(serde::Deserialize, serde::Serialize)]
#[repr(transparent)]
pub struct EnumValueName(String);

#[inherent]
impl GraphQLName for EnumValueName {
    pub fn new(s: impl Into<String>) -> Self { Self(s.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

impl AsRef<str> for EnumValueName {
    fn as_ref(&self) -> &str { &self.0 }
}

impl Borrow<str> for EnumValueName {
    fn borrow(&self) -> &str { &self.0 }
}

impl std::fmt::Display for EnumValueName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for EnumValueName {
    fn from(s: &str) -> Self { Self(s.to_owned()) }
}

impl From<String> for EnumValueName {
    fn from(s: String) -> Self { Self(s) }
}
