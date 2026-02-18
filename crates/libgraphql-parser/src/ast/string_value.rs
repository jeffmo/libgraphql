use std::borrow::Cow;

use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A GraphQL string value.
///
/// Per the
/// [String Value](https://spec.graphql.org/September2025/#sec-String-Value)
/// section of the spec, string values may be quoted strings
/// or block strings. This struct contains the processed
/// string after escape-sequence resolution and block-string
/// indentation stripping. Borrows from source when no
/// transformation was needed; owned when escapes were resolved
/// or block-string stripping produced a non-contiguous result.
#[derive(Clone, Debug, PartialEq)]
pub struct StringValue<'src> {
    /// Whether this string was written as a block string
    /// (`"""..."""`) rather than a quoted string (`"..."`).
    /// Both forms produce the same semantic value after
    /// processing, but tools (formatters, schema differs)
    /// may need to preserve or inspect the original form.
    pub is_block: bool,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<StringValueSyntax<'src>>,
    /// The processed string value after escape-sequence
    /// resolution and block-string indentation stripping.
    pub value: Cow<'src, str>,
}

/// Syntax detail for a [`StringValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct StringValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for StringValue<'_> {
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
