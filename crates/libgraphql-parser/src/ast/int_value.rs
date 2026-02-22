use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A GraphQL integer value.
///
/// Per the [Int](https://spec.graphql.org/September2025/#sec-Int)
/// section of the spec, implementations should support
/// "at minimum, the range of a signed 32-bit integer."
/// This parser represents Int values as `i32`. On
/// overflow/underflow the parser emits a diagnostic and
/// clamps to `i32::MAX` / `i32::MIN`.
#[derive(Clone, Debug, PartialEq)]
pub struct IntValue<'src> {
    pub span: GraphQLSourceSpan,
    pub syntax: Option<Box<IntValueSyntax<'src>>>,
    /// The parsed 32-bit integer value. On overflow/underflow
    /// the parser emits a diagnostic and clamps to
    /// `i32::MAX` / `i32::MIN`.
    pub value: i32,
}

impl IntValue<'_> {
    /// Widen to `i64` (infallible).
    pub fn as_i64(&self) -> i64 {
        self.value as i64
    }
}

/// Syntax detail for an [`IntValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct IntValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for IntValue<'_> {
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
