/// Identifies whether a scalar type is one of the built-in GraphQL
/// scalars or a custom (user-defined) scalar.
///
/// See [Scalars](https://spec.graphql.org/September2025/#sec-Scalars).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum ScalarKind {
    Boolean,
    Custom,
    Float,
    ID,
    Int,
    String,
}
