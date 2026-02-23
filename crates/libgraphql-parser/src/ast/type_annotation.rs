use crate::ast::AstNode;
use crate::ast::ListTypeAnnotation;
use crate::ast::NamedTypeAnnotation;
use inherent::inherent;

/// A GraphQL
/// [type reference](https://spec.graphql.org/September2025/#sec-Type-References)
/// (type annotation).
///
/// Represents [`NamedType`](https://spec.graphql.org/September2025/#NamedType) and
/// [`ListType`](https://spec.graphql.org/September2025/#ListType) from the spec grammar. The spec's
/// [`NonNullType`](https://spec.graphql.org/September2025/#NonNullType) is not a separate variant
/// here — instead, nullability is expressed via the [`Nullability`](crate::ast::Nullability) field
/// on each variant's inner struct.
// TODO: Revisit whether this allow is still needed after
// the ByteSpan/SourceMap work — the `GraphQLToken` size
// may change.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum TypeAnnotation<'src> {
    List(ListTypeAnnotation<'src>),
    Named(NamedTypeAnnotation<'src>),
}

#[inherent]
impl AstNode for TypeAnnotation<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        match self {
            TypeAnnotation::List(v) => {
                v.append_source(sink, source)
            },
            TypeAnnotation::Named(v) => {
                v.append_source(sink, source)
            },
        }
    }
}
