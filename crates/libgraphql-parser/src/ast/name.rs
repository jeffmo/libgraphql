use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;
use std::borrow::Cow;

/// A GraphQL [name](https://spec.graphql.org/September2025/#sec-Names)
/// (identifier).
///
/// Names are used for type names, field names, argument names,
/// directive names, enum values, and more. The `value` field
/// borrows from the source text when possible
/// (`Cow::Borrowed`) or owns the string when the source is not
/// available (`Cow::Owned`).
///
/// # Syntax Layer
///
/// When the parser retains syntax detail, `syntax` contains the
/// underlying [`GraphQLToken`] with any leading trivia
/// (whitespace, comments, commas).
#[derive(Clone, Debug, PartialEq)]
pub struct Name<'src> {
    pub span: ByteSpan,
    pub syntax: Option<Box<NameSyntax<'src>>>,
    pub value: Cow<'src, str>,
}

/// Syntax detail for a [`Name`] node.
#[derive(Clone, Debug, PartialEq)]
pub struct NameSyntax<'src> {
    pub token: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for Name<'_> {
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

    /// Returns this name's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this name's position to line/column
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
