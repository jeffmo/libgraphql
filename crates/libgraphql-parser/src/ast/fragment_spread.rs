use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DirectiveAnnotation;
use crate::ast::Name;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A named fragment spread (`...FragmentName`).
///
/// See
/// [Fragment Spreads](https://spec.graphql.org/September2025/#FragmentSpread)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct FragmentSpread<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax: Option<FragmentSpreadSyntax<'src>>,
}

/// Syntax detail for a [`FragmentSpread`].
#[derive(Clone, Debug, PartialEq)]
pub struct FragmentSpreadSyntax<'src> {
    pub ellipsis: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for FragmentSpread<'_> {
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
