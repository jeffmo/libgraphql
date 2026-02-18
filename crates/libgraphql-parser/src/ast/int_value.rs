use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A GraphQL integer value.
///
/// Per the
/// [Int Value](https://spec.graphql.org/September2025/#sec-Int-Value)
/// section of the spec, Int is a signed 32-bit integer. On
/// overflow/underflow the parser emits a diagnostic and clamps
/// to `i32::MAX` / `i32::MIN`.
#[derive(Clone, Debug, PartialEq)]
pub struct IntValue<'src> {
    /// The parsed 32-bit integer value. On overflow/underflow
    /// the parser emits a diagnostic and clamps to
    /// `i32::MAX` / `i32::MIN`.
    pub value: i32,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<IntValueSyntax<'src>>,
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
