use crate::names::TypeName;
use crate::span::Span;

/// A named type reference with nullability
/// (e.g. `String`, `String!`).
///
/// See [Named Types](https://spec.graphql.org/September2025/#NamedType).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct NamedTypeAnnotation {
    pub(crate) nullable: bool,
    pub(crate) span: Span,
    pub(crate) type_name: TypeName,
}

impl NamedTypeAnnotation {
    #[inline]
    pub fn nullable(&self) -> bool { self.nullable }
    #[inline]
    pub fn span(&self) -> Span { self.span }
    #[inline]
    pub fn type_name(&self) -> &TypeName { &self.type_name }
}
