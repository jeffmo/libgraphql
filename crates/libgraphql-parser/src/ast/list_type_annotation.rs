use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::Nullability;
use crate::ast::TypeAnnotation;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A list type reference (e.g. `[String]`, `[String!]!`).
///
/// See
/// [Type References](https://spec.graphql.org/September2025/#sec-Type-References)
/// in the spec. The `span` covers brackets and trailing `!`
/// when present.
#[derive(Clone, Debug, PartialEq)]
pub struct ListTypeAnnotation<'src> {
    pub element_type: Box<TypeAnnotation<'src>>,
    pub nullability: Nullability<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<Box<ListTypeAnnotationSyntax<'src>>>,
}

/// Syntax detail for a [`ListTypeAnnotation`].
#[derive(Clone, Debug, PartialEq)]
pub struct ListTypeAnnotationSyntax<'src> {
    pub brackets: DelimiterPair<'src>,
}

#[inherent]
impl AstNode for ListTypeAnnotation<'_> {
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
