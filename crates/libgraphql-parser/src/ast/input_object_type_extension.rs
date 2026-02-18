use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::InputValueDefinition;
use crate::ast::Name;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// An input object type extension.
///
/// See
/// [Input Object Extensions](https://spec.graphql.org/September2025/#sec-Input-Object-Extensions)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct InputObjectTypeExtension<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<InputValueDefinition<'src>>,
    pub syntax:
        Option<InputObjectTypeExtensionSyntax<'src>>,
}

/// Syntax detail for an
/// [`InputObjectTypeExtension`].
#[derive(Clone, Debug, PartialEq)]
pub struct InputObjectTypeExtensionSyntax<'src> {
    pub extend_keyword: GraphQLToken<'src>,
    pub input_keyword: GraphQLToken<'src>,
    pub braces: Option<DelimiterPair<'src>>,
}

#[inherent]
impl AstNode for InputObjectTypeExtension<'_> {
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
