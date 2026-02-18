use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DirectiveAnnotation;
use crate::ast::Name;
use crate::ast::StringValue;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A scalar type definition (e.g. `scalar DateTime`).
///
/// See
/// [Scalars](https://spec.graphql.org/September2025/#sec-Scalars)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct ScalarTypeDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax: Option<ScalarTypeDefinitionSyntax<'src>>,
}

/// Syntax detail for a [`ScalarTypeDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct ScalarTypeDefinitionSyntax<'src> {
    pub scalar_keyword: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for ScalarTypeDefinition<'_> {
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
