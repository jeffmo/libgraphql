use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DirectiveAnnotation;
use crate::ast::SelectionSet;
use crate::ast::TypeCondition;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// An inline fragment (`... on Type { ... }` or
/// `... { ... }`).
///
/// See
/// [Inline Fragments](https://spec.graphql.org/September2025/#InlineFragment)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct InlineFragment<'src> {
    pub span: GraphQLSourceSpan,
    pub type_condition: Option<TypeCondition<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub selection_set: SelectionSet<'src>,
    pub syntax: Option<InlineFragmentSyntax<'src>>,
}

/// Syntax detail for an [`InlineFragment`].
#[derive(Clone, Debug, PartialEq)]
pub struct InlineFragmentSyntax<'src> {
    pub ellipsis: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for InlineFragment<'_> {
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
