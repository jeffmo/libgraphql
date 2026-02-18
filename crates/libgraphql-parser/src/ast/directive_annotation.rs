use crate::ast::ast_node::append_span_source_slice;
use crate::ast::Argument;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::Name;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A directive annotation applied to a definition or field
/// (e.g. `@deprecated(reason: "Use newField")`).
///
/// See
/// [Directives](https://spec.graphql.org/September2025/#sec-Language.Directives)
/// in the spec. Note: this represents an *applied* directive
/// (an annotation), not a directive *definition*.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveAnnotation<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub arguments: Vec<Argument<'src>>,
    pub syntax: Option<DirectiveAnnotationSyntax<'src>>,
}

/// Syntax detail for a [`DirectiveAnnotation`].
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveAnnotationSyntax<'src> {
    pub at_sign: GraphQLToken<'src>,
    pub argument_parens: Option<DelimiterPair<'src>>,
}

#[inherent]
impl AstNode for DirectiveAnnotation<'_> {
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
