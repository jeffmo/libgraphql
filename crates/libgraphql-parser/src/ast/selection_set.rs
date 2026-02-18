use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::Selection;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A selection set — the set of fields and fragments
/// selected within braces `{ ... }`.
///
/// See
/// [Selection Sets](https://spec.graphql.org/September2025/#sec-Selection-Sets)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectionSet<'src> {
    pub span: GraphQLSourceSpan,
    pub selections: Vec<Selection<'src>>,
    pub syntax: Option<SelectionSetSyntax<'src>>,
}

/// Syntax detail for a [`SelectionSet`].
#[derive(Clone, Debug, PartialEq)]
pub struct SelectionSetSyntax<'src> {
    pub braces: DelimiterPair<'src>,
}

#[inherent]
impl AstNode for SelectionSet<'_> {
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
