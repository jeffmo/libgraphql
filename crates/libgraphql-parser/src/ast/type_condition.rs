use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::Name;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A type condition (e.g., `on User`) used in fragment
/// definitions and inline fragments.
///
/// See
/// [Type Conditions](https://spec.graphql.org/September2025/#sec-Type-Conditions)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct TypeCondition<'src> {
    pub named_type: Name<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<TypeConditionSyntax<'src>>,
}

/// Syntax detail for a [`TypeCondition`].
#[derive(Clone, Debug, PartialEq)]
pub struct TypeConditionSyntax<'src> {
    pub on_keyword: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for TypeCondition<'_> {
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
