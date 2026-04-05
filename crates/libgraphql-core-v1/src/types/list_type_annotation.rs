use crate::span::Span;
use crate::types::type_annotation::TypeAnnotation;

/// A list type wrapper with nullability
/// (e.g. `[String]`, `[String!]!`).
///
/// See [List Types](https://spec.graphql.org/September2025/#ListType).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ListTypeAnnotation {
    pub(crate) inner: Box<TypeAnnotation>,
    pub(crate) nullable: bool,
    pub(crate) span: Span,
}

impl ListTypeAnnotation {
    #[inline]
    pub fn inner(&self) -> &TypeAnnotation { &self.inner }
    #[inline]
    pub fn nullable(&self) -> bool { self.nullable }
    #[inline]
    pub fn span(&self) -> Span { self.span }
}
