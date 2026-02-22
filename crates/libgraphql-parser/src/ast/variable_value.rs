use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::Name;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A variable reference in a GraphQL value position
/// (e.g., `$id`).
///
/// See the
/// [Variables](https://spec.graphql.org/September2025/#sec-Language.Variables)
/// section of the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct VariableValue<'src> {
    pub name: Name<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<Box<VariableValueSyntax<'src>>>,
}

/// Syntax detail for a [`VariableValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct VariableValueSyntax<'src> {
    pub dollar: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for VariableValue<'_> {
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
