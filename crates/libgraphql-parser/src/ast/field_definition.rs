use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::InputValueDefinition;
use crate::ast::Name;
use crate::ast::StringValue;
use crate::ast::TypeAnnotation;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A field definition within an object type or interface
/// type.
///
/// See
/// [Field Definitions](https://spec.graphql.org/September2025/#FieldsDefinition)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldDefinition<'src> {
    pub arguments: Vec<InputValueDefinition<'src>>,
    pub description: Option<StringValue<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub field_type: TypeAnnotation<'src>,
    pub name: Name<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<FieldDefinitionSyntax<'src>>,
}

/// Syntax detail for a [`FieldDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct FieldDefinitionSyntax<'src> {
    pub argument_parens: Option<DelimiterPair<'src>>,
    pub colon: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for FieldDefinition<'_> {
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
