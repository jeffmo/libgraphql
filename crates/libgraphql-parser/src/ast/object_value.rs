use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::ObjectField;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A GraphQL input object value (e.g., `{x: 1, y: 2}`).
///
/// See the
/// [Input Object Values](https://spec.graphql.org/September2025/#sec-Input-Object-Values)
/// section of the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectValue<'src> {
    pub fields: Vec<ObjectField<'src>>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<Box<ObjectValueSyntax<'src>>>,
}

/// Syntax detail for an [`ObjectValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectValueSyntax<'src> {
    pub braces: DelimiterPair<'src>,
}

#[inherent]
impl AstNode for ObjectValue<'_> {
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
