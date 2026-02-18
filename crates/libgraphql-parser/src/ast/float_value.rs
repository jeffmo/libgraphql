use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A GraphQL float value.
///
/// Per the
/// [Float Value](https://spec.graphql.org/September2025/#sec-Float-Value)
/// section of the spec, Float is a double-precision
/// floating-point value (IEEE 754). On overflow the parser
/// emits a diagnostic and stores
/// `f64::INFINITY` / `f64::NEG_INFINITY`.
#[derive(Clone, Debug, PartialEq)]
pub struct FloatValue<'src> {
    /// The parsed `f64` value. On overflow the parser emits a
    /// diagnostic and stores
    /// `f64::INFINITY` / `f64::NEG_INFINITY`.
    pub value: f64,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<FloatValueSyntax<'src>>,
}

/// Syntax detail for a [`FloatValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct FloatValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for FloatValue<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                &self.span, sink, src,
            );
        }
    }
}
