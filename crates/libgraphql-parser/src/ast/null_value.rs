use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A GraphQL null literal.
///
/// See the
/// [Null Value](https://spec.graphql.org/September2025/#sec-Null-Value)
/// section of the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct NullValue<'src> {
    pub span: GraphQLSourceSpan,
    pub syntax: Option<NullValueSyntax<'src>>,
}

/// Syntax detail for a [`NullValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct NullValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for NullValue<'_> {
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
