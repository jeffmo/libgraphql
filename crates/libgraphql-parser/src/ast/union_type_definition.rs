use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DirectiveAnnotation;
use crate::ast::Name;
use crate::ast::StringValue;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A union type definition.
///
/// See
/// [Unions](https://spec.graphql.org/September2025/#sec-Unions)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct UnionTypeDefinition<'src> {
    pub description: Option<StringValue<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub members: Vec<Name<'src>>,
    pub name: Name<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<UnionTypeDefinitionSyntax<'src>>,
}

/// Syntax detail for a [`UnionTypeDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct UnionTypeDefinitionSyntax<'src> {
    pub equals: Option<GraphQLToken<'src>>,
    pub leading_pipe: Option<GraphQLToken<'src>>,
    pub pipes: Vec<GraphQLToken<'src>>,
    pub union_keyword: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for UnionTypeDefinition<'_> {
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
