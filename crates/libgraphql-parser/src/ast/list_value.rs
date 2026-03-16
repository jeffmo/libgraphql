use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::Value;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use inherent::inherent;

/// A GraphQL list value (e.g., `[1, 2, 3]`).
///
/// See the
/// [List Value](https://spec.graphql.org/September2025/#sec-List-Value)
/// section of the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct ListValue<'src> {
    pub span: ByteSpan,
    pub syntax: Option<Box<ListValueSyntax<'src>>>,
    pub values: Vec<Value<'src>>,
}

/// Syntax detail for a [`ListValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct ListValueSyntax<'src> {
    pub brackets: DelimiterPair<'src>,
}

#[inherent]
impl AstNode for ListValue<'_> {
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

    /// Returns this list value's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this list value's position to line/column
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
