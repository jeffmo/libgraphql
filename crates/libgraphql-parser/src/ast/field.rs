use crate::ast::ast_node::append_span_source_slice;
use crate::ast::Argument;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::Name;
use crate::ast::SelectionSet;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A field selection within a selection set, optionally
/// aliased, with arguments, directives, and a nested
/// selection set.
///
/// See
/// [Fields](https://spec.graphql.org/September2025/#sec-Language.Fields)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct Field<'src> {
    pub alias: Option<Name<'src>>,
    pub arguments: Vec<Argument<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub name: Name<'src>,
    pub selection_set: Option<SelectionSet<'src>>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<FieldSyntax<'src>>,
}

/// Syntax detail for a [`Field`].
#[derive(Clone, Debug, PartialEq)]
pub struct FieldSyntax<'src> {
    /// The colon between alias and field name. `None`
    /// when no alias is present.
    pub alias_colon: Option<GraphQLToken<'src>>,
    pub argument_parens: Option<DelimiterPair<'src>>,
}

#[inherent]
impl AstNode for Field<'_> {
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
