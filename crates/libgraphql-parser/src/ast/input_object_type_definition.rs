use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::InputValueDefinition;
use crate::ast::Name;
use crate::ast::StringValue;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// An input object type definition.
///
/// See
/// [Input Objects](https://spec.graphql.org/September2025/#sec-Input-Objects)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct InputObjectTypeDefinition<'src> {
    pub description: Option<StringValue<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<InputValueDefinition<'src>>,
    pub name: Name<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax:
        Option<Box<InputObjectTypeDefinitionSyntax<'src>>>,
}

/// Syntax detail for an [`InputObjectTypeDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct InputObjectTypeDefinitionSyntax<'src> {
    pub braces: Option<DelimiterPair<'src>>,
    pub input_keyword: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for InputObjectTypeDefinition<'_> {
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
