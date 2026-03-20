use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::Name;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;

/// A type condition (e.g., `on User`) used in fragment
/// definitions and inline fragments.
///
/// See
/// [Type Conditions](https://spec.graphql.org/September2025/#sec-Type-Conditions)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct TypeCondition<'src> {
    pub named_type: Name<'src>,
    pub span: ByteSpan,
    pub syntax: Option<Box<TypeConditionSyntax<'src>>>,
}

/// Syntax detail for a [`TypeCondition`].
#[derive(Clone, Debug, PartialEq)]
pub struct TypeConditionSyntax<'src> {
    pub on_keyword: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for TypeCondition<'_> {
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

    /// Returns this type condition's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this type condition's position to line/column
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
