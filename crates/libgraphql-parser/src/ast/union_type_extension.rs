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
///
/// # Spec invariant
///
/// The spec's directives-only form
/// (`extend union Name Directives[Const]`) requires at
/// least one directive when no `members` are present.
/// For a spec-valid node, `directives` and `members`
/// are never both empty.
#[derive(Clone, Debug, PartialEq)]
pub struct UnionTypeExtension<'src> {
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub members: Vec<Name<'src>>,
    pub name: Name<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax:
        Option<UnionTypeExtensionSyntax<'src>>,
}

/// Syntax detail for a [`UnionTypeExtension`].
#[derive(Clone, Debug, PartialEq)]
pub struct UnionTypeExtensionSyntax<'src> {
    pub equals: Option<GraphQLToken<'src>>,
    pub extend_keyword: GraphQLToken<'src>,
    pub leading_pipe: Option<GraphQLToken<'src>>,
    pub pipes: Vec<GraphQLToken<'src>>,
    pub union_keyword: GraphQLToken<'src>,
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
