use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::Name;
use crate::ast::OperationKind;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;

/// A root operation type definition within a schema
/// definition (e.g. `query: Query`).
///
/// See
/// [Root Operation Types](https://spec.graphql.org/September2025/#sec-Root-Operation-Types)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct RootOperationTypeDefinition<'src> {
    pub named_type: Name<'src>,
    pub operation_kind: OperationKind,
    pub span: ByteSpan,
    pub syntax:
        Option<Box<RootOperationTypeDefinitionSyntax<'src>>>,
}

/// Syntax detail for a
/// [`RootOperationTypeDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct RootOperationTypeDefinitionSyntax<'src> {
    pub colon: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for RootOperationTypeDefinition<'_> {
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

    /// Returns this root operation type definition's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this root operation type definition's position to line/column
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
