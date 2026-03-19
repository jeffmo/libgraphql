use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;
use std::borrow::Cow;

/// A GraphQL string value.
///
/// Per the
/// [String Value](https://spec.graphql.org/September2025/#sec-String-Value)
/// section of the spec, string values may be quoted strings
/// or block strings. This struct contains the processed
/// string after escape-sequence resolution and block-string
/// indentation stripping. Borrows from source when no
/// transformation was needed; owned when escapes were resolved
/// or block-string stripping produced a non-contiguous result.
#[derive(Clone, Debug, PartialEq)]
pub struct StringValue<'src> {
    /// Whether this string was written as a block string
    /// (`"""..."""`) rather than a quoted string (`"..."`).
    /// Both forms produce the same semantic value after
    /// processing, but tools (formatters, schema differs)
    /// may need to preserve or inspect the original form.
    pub is_block: bool,
    pub span: ByteSpan,
    pub syntax: Option<Box<StringValueSyntax<'src>>>,
    /// The processed string value after escape-sequence
    /// resolution and block-string indentation stripping.
    pub value: Cow<'src, str>,
}

/// Syntax detail for a [`StringValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct StringValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for StringValue<'_> {
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

    /// Returns this string value's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this string value's position to line/column
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
