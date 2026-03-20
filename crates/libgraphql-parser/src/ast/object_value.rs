use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::ObjectField;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use inherent::inherent;

/// A GraphQL input object value (e.g., `{x: 1, y: 2}`).
///
/// See the
/// [Input Object Values](https://spec.graphql.org/September2025/#sec-Input-Object-Values)
/// section of the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectValue<'src> {
    pub fields: Vec<ObjectField<'src>>,
    pub span: ByteSpan,
    pub syntax: Option<Box<ObjectValueSyntax<'src>>>,
}

/// Syntax detail for an [`ObjectValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectValueSyntax<'src> {
    pub braces: DelimiterPair<'src>,
}

#[inherent]
impl AstNode for ObjectValue<'_> {
    /// See [`AstNode::append_source()`](crate::ast::AstNode::append_source).
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

    /// Returns this object value's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this object value's position to line/column
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
