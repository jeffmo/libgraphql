use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;

/// A GraphQL float value.
///
/// Per the
/// [Float Value](https://spec.graphql.org/September2025/#sec-Float-Value)
/// section of the spec, Float is a double-precision
/// floating-point value (IEEE 754). On overflow the parser
/// emits a diagnostic and stores
/// `f64::INFINITY` / `f64::NEG_INFINITY`.
#[derive(Clone, Debug)]
pub struct FloatValue<'src> {
    pub span: ByteSpan,
    pub syntax: Option<Box<FloatValueSyntax<'src>>>,
    /// The parsed `f64` value. On overflow the parser emits a
    /// diagnostic and stores
    /// `f64::INFINITY` / `f64::NEG_INFINITY`.
    pub value: f64,
}

impl PartialEq for FloatValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.value.to_bits() == other.value.to_bits()
            && self.span == other.span
            && self.syntax == other.syntax
    }
}

/// Syntax detail for a [`FloatValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct FloatValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for FloatValue<'_> {
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

    /// Returns this float value's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this float value's position to line/column
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
