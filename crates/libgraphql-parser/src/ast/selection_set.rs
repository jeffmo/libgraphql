use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::Selection;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use inherent::inherent;

/// A selection set — the set of fields and fragments
/// selected within braces `{ ... }`.
///
/// See
/// [Selection Sets](https://spec.graphql.org/September2025/#sec-Selection-Sets)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectionSet<'src> {
    pub selections: Vec<Selection<'src>>,
    pub span: ByteSpan,
    pub syntax: Option<Box<SelectionSetSyntax<'src>>>,
}

/// Syntax detail for a [`SelectionSet`].
#[derive(Clone, Debug, PartialEq)]
pub struct SelectionSetSyntax<'src> {
    pub braces: DelimiterPair<'src>,
}

#[inherent]
impl AstNode for SelectionSet<'_> {
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

    /// Returns this selection set's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this selection set's position to line/column
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
