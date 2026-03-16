use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::Nullability;
use crate::ast::TypeAnnotation;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use inherent::inherent;

/// A list type reference (e.g. `[String]`, `[String!]!`).
///
/// See
/// [Type References](https://spec.graphql.org/September2025/#sec-Type-References)
/// in the spec. The `span` covers brackets and trailing `!`
/// when present.
#[derive(Clone, Debug, PartialEq)]
pub struct ListTypeAnnotation<'src> {
    pub element_type: Box<TypeAnnotation<'src>>,
    pub nullability: Nullability<'src>,
    pub span: ByteSpan,
    pub syntax: Option<Box<ListTypeAnnotationSyntax<'src>>>,
}

impl<'src> ListTypeAnnotation<'src> {
    pub fn nullable(&self) -> bool {
        matches!(self.nullability, Nullability::Nullable)
    }
}

/// Syntax detail for a [`ListTypeAnnotation`].
#[derive(Clone, Debug, PartialEq)]
pub struct ListTypeAnnotationSyntax<'src> {
    pub brackets: DelimiterPair<'src>,
}

#[inherent]
impl AstNode for ListTypeAnnotation<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                self.span, sink, src,
            );
        }
    }

    /// Returns this list type annotation's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this list type annotation's position to line/column
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
