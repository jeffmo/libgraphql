use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::FieldDefinition;
use crate::ast::Name;
use crate::ast::StringValue;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// An interface type definition.
///
/// See
/// [Interfaces](https://spec.graphql.org/September2025/#sec-Interfaces)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceTypeDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub implements: Vec<Name<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<FieldDefinition<'src>>,
    pub syntax:
        Option<InterfaceTypeDefinitionSyntax<'src>>,
}

/// Syntax detail for an [`InterfaceTypeDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceTypeDefinitionSyntax<'src> {
    pub interface_keyword: GraphQLToken<'src>,
    pub implements_keyword: Option<GraphQLToken<'src>>,
    pub leading_ampersand: Option<GraphQLToken<'src>>,
    pub ampersands: Vec<GraphQLToken<'src>>,
    pub braces: Option<DelimiterPair<'src>>,
}

#[inherent]
impl AstNode for InterfaceTypeDefinition<'_> {
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
