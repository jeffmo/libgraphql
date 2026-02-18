use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::Name;
use crate::ast::Value;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A single field within a GraphQL
/// [input object value](https://spec.graphql.org/September2025/#sec-Input-Object-Values).
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectField<'src> {
    pub name: Name<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<ObjectFieldSyntax<'src>>,
    pub value: Value<'src>,
}

/// Syntax detail for an [`ObjectField`].
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectFieldSyntax<'src> {
    pub colon: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for ObjectField<'_> {
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
