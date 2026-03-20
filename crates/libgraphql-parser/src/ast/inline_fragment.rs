use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DirectiveAnnotation;
use crate::ast::SelectionSet;
use crate::ast::TypeCondition;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;

/// An inline fragment (`... on Type { ... }` or
/// `... { ... }`).
///
/// See
/// [Inline Fragments](https://spec.graphql.org/September2025/#InlineFragment)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct InlineFragment<'src> {
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub selection_set: SelectionSet<'src>,
    pub span: ByteSpan,
    pub syntax: Option<Box<InlineFragmentSyntax<'src>>>,
    pub type_condition: Option<TypeCondition<'src>>,
}

/// Syntax detail for an [`InlineFragment`].
#[derive(Clone, Debug, PartialEq)]
pub struct InlineFragmentSyntax<'src> {
    pub ellipsis: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for InlineFragment<'_> {
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

    /// Returns this inline fragment's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this inline fragment's position to line/column
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
