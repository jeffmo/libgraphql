use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::Name;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;

/// A variable reference in a GraphQL value position
/// (e.g., `$id`).
///
/// See the
/// [Variables](https://spec.graphql.org/September2025/#sec-Language.Variables)
/// section of the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct VariableReference<'src> {
    pub name: Name<'src>,
    pub span: ByteSpan,
    pub syntax: Option<Box<VariableReferenceSyntax<'src>>>,
}

/// Syntax detail for a [`VariableReference`].
#[derive(Clone, Debug, PartialEq)]
pub struct VariableReferenceSyntax<'src> {
    pub dollar: GraphQLToken<'src>,
}

impl<'src> VariableReference<'src> {
    /// Returns the name of this variable reference as a
    /// string slice.
    ///
    /// Convenience accessor for `self.name.value`.
    #[inline]
    pub fn name_value(&self) -> &str {
        self.name.value.as_ref()
    }
}

#[inherent]
impl AstNode for VariableReference<'_> {
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

    /// Returns this variable reference's byte-offset span
    /// within the source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this variable reference's position to
    /// line/column coordinates using the given
    /// [`SourceMap`].
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
