use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::Name;
use crate::ast::Value;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A single argument in a field, directive, or field
/// definition.
///
/// See
/// [Arguments](https://spec.graphql.org/September2025/#sec-Language.Arguments)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct Argument<'src> {
    pub name: Name<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<Box<ArgumentSyntax<'src>>>,
    pub value: Value<'src>,
}

/// Syntax detail for an [`Argument`].
#[derive(Clone, Debug, PartialEq)]
pub struct ArgumentSyntax<'src> {
    pub colon: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for Argument<'_> {
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
