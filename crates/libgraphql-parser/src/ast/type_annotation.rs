use crate::ast::AstNode;
use crate::ast::ListTypeAnnotation;
use crate::ast::NamedTypeAnnotation;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
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
#[derive(Clone, Debug, PartialEq)]
pub enum TypeAnnotation<'src> {
    List(ListTypeAnnotation<'src>),
    Named(NamedTypeAnnotation<'src>),
}

impl<'src> TypeAnnotation<'src> {
    /// Returns `true` if this type annotation is nullable
    /// (i.e. does **not** have a trailing `!`).
    #[inline]
    pub fn nullable(&self) -> bool {
        match self {
            Self::List(annot) => annot.nullable(),
            Self::Named(annot) => annot.nullable(),
        }
    }
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

    /// Returns this type annotation's byte-offset span within
    /// the source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        match self {
            Self::List(annot) => annot.span,
            Self::Named(annot) => annot.span,
        }
    }

    /// Resolves this type annotation's position to line/column
    /// coordinates using the given [`SourceMap`].
    ///
    /// Returns [`None`] if the byte offsets cannot be resolved
    /// (e.g. the span was synthetically constructed without
    /// valid position data).
    #[inline]
    pub fn source_span(
        &self,
        source_map: &SourceMap,
    ) -> Option<SourceSpan> {
        self.byte_span().resolve(source_map)
    }
}
