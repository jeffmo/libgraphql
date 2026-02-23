use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::RootOperationTypeDefinition;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A schema extension.
///
/// See
/// [Schema Extension](https://spec.graphql.org/September2025/#sec-Schema-Extension)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct SchemaExtension<'src> {
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub root_operations:
        Vec<RootOperationTypeDefinition<'src>>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<Box<SchemaExtensionSyntax<'src>>>,
}

/// Syntax detail for a [`SchemaExtension`].
#[derive(Clone, Debug, PartialEq)]
pub struct SchemaExtensionSyntax<'src> {
    pub braces: Option<DelimiterPair<'src>>,
    pub extend_keyword: GraphQLToken<'src>,
    pub schema_keyword: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for SchemaExtension<'_> {
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
