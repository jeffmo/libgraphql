use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::EnumValueDefinition;
use crate::ast::Name;
use crate::ast::StringValue;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// An enum type definition.
///
/// See
/// [Enums](https://spec.graphql.org/September2025/#sec-Enums)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct EnumTypeDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub values: Vec<EnumValueDefinition<'src>>,
    pub syntax: Option<EnumTypeDefinitionSyntax<'src>>,
}

/// Syntax detail for an [`EnumTypeDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct EnumTypeDefinitionSyntax<'src> {
    pub enum_keyword: GraphQLToken<'src>,
    pub braces: Option<DelimiterPair<'src>>,
}

#[inherent]
impl AstNode for EnumTypeDefinition<'_> {
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
