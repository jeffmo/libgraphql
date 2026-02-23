use std::borrow::Cow;

use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A GraphQL [name](https://spec.graphql.org/September2025/#sec-Names)
/// (identifier).
///
/// Names are used for type names, field names, argument names,
/// directive names, enum values, and more. The `value` field
/// borrows from the source text when possible
/// (`Cow::Borrowed`) or owns the string when the source is not
/// available (`Cow::Owned`).
///
/// # Syntax Layer
///
/// When the parser retains syntax detail, `syntax` contains the
/// underlying [`GraphQLToken`] with any leading trivia
/// (whitespace, comments, commas).
#[derive(Clone, Debug, PartialEq)]
pub struct Name<'src> {
    pub span: GraphQLSourceSpan,
    pub syntax: Option<Box<NameSyntax<'src>>>,
    pub value: Cow<'src, str>,
}

/// Syntax detail for a [`Name`] node.
#[derive(Clone, Debug, PartialEq)]
pub struct NameSyntax<'src> {
    pub token: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for Name<'_> {
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
