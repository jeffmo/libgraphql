use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DirectiveAnnotation;
use crate::ast::Name;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A union type extension.
///
/// See
/// [Union Extensions](https://spec.graphql.org/September2025/#sec-Union-Extensions)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct UnionTypeExtension<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub members: Vec<Name<'src>>,
    pub syntax:
        Option<UnionTypeExtensionSyntax<'src>>,
}

/// Syntax detail for a [`UnionTypeExtension`].
#[derive(Clone, Debug, PartialEq)]
pub struct UnionTypeExtensionSyntax<'src> {
    pub extend_keyword: GraphQLToken<'src>,
    pub union_keyword: GraphQLToken<'src>,
    pub equals: Option<GraphQLToken<'src>>,
    pub leading_pipe: Option<GraphQLToken<'src>>,
    pub pipes: Vec<GraphQLToken<'src>>,
}

#[inherent]
impl AstNode for UnionTypeExtension<'_> {
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
