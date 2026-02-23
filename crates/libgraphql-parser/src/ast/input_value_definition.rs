use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DirectiveAnnotation;
use crate::ast::Name;
use crate::ast::StringValue;
use crate::ast::TypeAnnotation;
use crate::ast::Value;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// An input value definition, used for field arguments and
/// input object fields.
///
/// See
/// [Input Values Definitions](https://spec.graphql.org/September2025/#InputValueDefinition)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct InputValueDefinition<'src> {
    pub default_value: Option<Value<'src>>,
    pub description: Option<StringValue<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub name: Name<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<Box<InputValueDefinitionSyntax<'src>>>,
    pub value_type: TypeAnnotation<'src>,
}

/// Syntax detail for an [`InputValueDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct InputValueDefinitionSyntax<'src> {
    pub colon: GraphQLToken<'src>,
    pub equals: Option<GraphQLToken<'src>>,
}

#[inherent]
impl AstNode for InputValueDefinition<'_> {
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
