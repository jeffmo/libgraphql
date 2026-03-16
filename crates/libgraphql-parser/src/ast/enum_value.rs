use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;
use std::borrow::Cow;

/// A GraphQL enum value (an unquoted name that is not
/// `true`, `false`, or `null`).
///
/// See the
/// [Enum Value](https://spec.graphql.org/September2025/#sec-Enum-Value)
/// section of the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct EnumValue<'src> {
    pub span: ByteSpan,
    pub syntax: Option<Box<EnumValueSyntax<'src>>>,
    pub value: Cow<'src, str>,
}

/// Syntax detail for an [`EnumValue`] (the enum value
/// literal, not the enum value definition).
#[derive(Clone, Debug, PartialEq)]
pub struct EnumValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for EnumValue<'_> {
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

    /// Returns this enum value's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this enum value's position to line/column
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
