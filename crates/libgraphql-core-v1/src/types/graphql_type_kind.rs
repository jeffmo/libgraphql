use crate::types::scalar_kind::ScalarKind;

/// Discriminates all GraphQL type categories, including
/// individual built-in scalar identities.
///
/// This enum has 11 variants — the 6 data-carrying categories
/// plus the 5 built-in scalars broken out from `Scalar`. Use
/// [`GraphQLType::type_kind()`](crate::types::GraphQLType::type_kind)
/// when you need exhaustive matching over all type identities.
///
/// See [Types](https://spec.graphql.org/September2025/#sec-Types).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum GraphQLTypeKind {
    Boolean,
    Enum,
    Float,
    ID,
    InputObject,
    Int,
    Interface,
    Object,
    Scalar,
    String,
    Union,
}

impl From<ScalarKind> for GraphQLTypeKind {
    fn from(kind: ScalarKind) -> Self {
        match kind {
            ScalarKind::Boolean => Self::Boolean,
            ScalarKind::Custom => Self::Scalar,
            ScalarKind::Float => Self::Float,
            ScalarKind::ID => Self::ID,
            ScalarKind::Int => Self::Int,
            ScalarKind::String => Self::String,
        }
    }
}
