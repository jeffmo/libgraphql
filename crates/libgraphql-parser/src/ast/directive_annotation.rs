use crate::ast::Argument;
use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::Name;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;

/// A directive annotation applied to a definition or field
/// (e.g. `@deprecated(reason: "Use newField")`).
///
/// See
/// [Directives](https://spec.graphql.org/September2025/#sec-Language.Directives)
/// in the spec. Note: this represents an *applied* directive
/// (an annotation), not a directive *definition*.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveAnnotation<'src> {
    pub arguments: Vec<Argument<'src>>,
    pub name: Name<'src>,
    pub span: ByteSpan,
    pub syntax: Option<Box<DirectiveAnnotationSyntax<'src>>>,
}

/// Syntax detail for a [`DirectiveAnnotation`].
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveAnnotationSyntax<'src> {
    pub argument_parens: Option<DelimiterPair<'src>>,
    pub at_sign: GraphQLToken<'src>,
}

impl<'src> DirectiveAnnotation<'src> {
    /// Returns the name of this directive annotation as a string
    /// slice.
    ///
    /// Convenience accessor for `self.name.value`.
    #[inline]
    pub fn name_value(&self) -> &str {
        self.name.value.as_ref()
    }
}

#[inherent]
impl AstNode for DirectiveAnnotation<'_> {
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

    /// Returns this directive annotation's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this directive annotation's position to line/column
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
