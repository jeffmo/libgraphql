use std::borrow::Cow;

use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A GraphQL enum value (an unquoted name that is not
/// `true`, `false`, or `null`).
///
/// See the
/// [Enum Value](https://spec.graphql.org/September2025/#sec-Enum-Value)
/// section of the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct EnumValue<'src> {
    pub value: Cow<'src, str>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<EnumValueSyntax<'src>>,
}

/// Syntax detail for an [`EnumValue`] (the enum value
/// literal, not the enum value definition).
#[derive(Clone, Debug, PartialEq)]
pub struct EnumValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for EnumValue<'_> {
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
