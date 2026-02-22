use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A GraphQL boolean value (`true` or `false`).
///
/// See the
/// [Boolean Value](https://spec.graphql.org/September2025/#sec-Boolean-Value)
/// section of the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct BooleanValue<'src> {
    pub span: GraphQLSourceSpan,
    pub syntax: Option<Box<BooleanValueSyntax<'src>>>,
    pub value: bool,
}

/// Syntax detail for a [`BooleanValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct BooleanValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for BooleanValue<'_> {
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
