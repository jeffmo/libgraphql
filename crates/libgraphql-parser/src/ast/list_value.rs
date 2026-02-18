use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::Value;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A GraphQL list value (e.g., `[1, 2, 3]`).
///
/// See the
/// [List Value](https://spec.graphql.org/September2025/#sec-List-Value)
/// section of the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct ListValue<'src> {
    pub span: GraphQLSourceSpan,
    pub syntax: Option<ListValueSyntax<'src>>,
    pub values: Vec<Value<'src>>,
}

/// Syntax detail for a [`ListValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct ListValueSyntax<'src> {
    pub brackets: DelimiterPair<'src>,
}

#[inherent]
impl AstNode for ListValue<'_> {
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
