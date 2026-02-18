use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DirectiveAnnotation;
use crate::ast::Name;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A scalar type extension.
///
/// See
/// [Scalar Extensions](https://spec.graphql.org/September2025/#sec-Scalar-Extensions)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct ScalarTypeExtension<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax:
        Option<ScalarTypeExtensionSyntax<'src>>,
}

/// Syntax detail for a [`ScalarTypeExtension`].
#[derive(Clone, Debug, PartialEq)]
pub struct ScalarTypeExtensionSyntax<'src> {
    pub extend_keyword: GraphQLToken<'src>,
    pub scalar_keyword: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for ScalarTypeExtension<'_> {
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
