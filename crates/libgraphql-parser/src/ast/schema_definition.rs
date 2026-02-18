use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::RootOperationTypeDefinition;
use crate::ast::StringValue;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A GraphQL schema definition.
///
/// See
/// [Schema](https://spec.graphql.org/September2025/#sec-Schema)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct SchemaDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub root_operations:
        Vec<RootOperationTypeDefinition<'src>>,
    pub syntax: Option<SchemaDefinitionSyntax<'src>>,
}

/// Syntax detail for a [`SchemaDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct SchemaDefinitionSyntax<'src> {
    pub schema_keyword: GraphQLToken<'src>,
    pub braces: DelimiterPair<'src>,
}

#[inherent]
impl AstNode for SchemaDefinition<'_> {
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
