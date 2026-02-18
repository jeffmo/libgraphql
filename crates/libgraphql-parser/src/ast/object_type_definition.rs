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

/// An object type definition.
///
/// See
/// [Objects](https://spec.graphql.org/September2025/#sec-Objects)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectTypeDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub implements: Vec<Name<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<FieldDefinition<'src>>,
    pub syntax:
        Option<ObjectTypeDefinitionSyntax<'src>>,
}

/// Syntax detail for an [`ObjectTypeDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectTypeDefinitionSyntax<'src> {
    pub type_keyword: GraphQLToken<'src>,
    pub implements_keyword: Option<GraphQLToken<'src>>,
    pub leading_ampersand: Option<GraphQLToken<'src>>,
    pub ampersands: Vec<GraphQLToken<'src>>,
    pub braces: Option<DelimiterPair<'src>>,
}

#[inherent]
impl AstNode for ObjectTypeDefinition<'_> {
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
