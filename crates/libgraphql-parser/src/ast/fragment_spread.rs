use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DirectiveAnnotation;
use crate::ast::Name;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;

/// A named fragment spread (`...FragmentName`).
///
/// See
/// [Fragment Spreads](https://spec.graphql.org/September2025/#FragmentSpread)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct FragmentSpread<'src> {
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub name: Name<'src>,
    pub span: ByteSpan,
    pub syntax: Option<Box<FragmentSpreadSyntax<'src>>>,
}

/// Syntax detail for a [`FragmentSpread`].
#[derive(Clone, Debug, PartialEq)]
pub struct FragmentSpreadSyntax<'src> {
    pub ellipsis: GraphQLToken<'src>,
}

impl<'src> FragmentSpread<'src> {
    /// Returns the name of this fragment spread as a string
    /// slice.
    ///
    /// Convenience accessor for `self.name.value`.
    #[inline]
    pub fn name_value(&self) -> &str {
        self.name.value.as_ref()
    }
}

#[inherent]
impl AstNode for FragmentSpread<'_> {
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

    /// Returns this fragment spread's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this fragment spread's position to line/column
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
