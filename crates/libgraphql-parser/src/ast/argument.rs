use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::Name;
use crate::ast::Value;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;

/// A single argument in a field selection or directive
/// annotation.
///
/// See
/// [Arguments](https://spec.graphql.org/September2025/#sec-Language.Arguments)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct Argument<'src> {
    pub name: Name<'src>,
    pub span: ByteSpan,
    pub syntax: Option<Box<ArgumentSyntax<'src>>>,
    pub value: Value<'src>,
}

/// Syntax detail for an [`Argument`].
#[derive(Clone, Debug, PartialEq)]
pub struct ArgumentSyntax<'src> {
    pub colon: GraphQLToken<'src>,
}

impl<'src> Argument<'src> {
    /// Returns the name of this argument as a string
    /// slice.
    ///
    /// Convenience accessor for `self.name.value`.
    #[inline]
    pub fn name_value(&self) -> &str {
        self.name.value.as_ref()
    }
}

#[inherent]
impl AstNode for Argument<'_> {
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

    /// Returns this argument's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this argument's position to line/column
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
