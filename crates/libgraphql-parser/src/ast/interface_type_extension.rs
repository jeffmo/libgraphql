use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::FieldDefinition;
use crate::ast::Name;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// An interface type extension.
///
/// See
/// [Interface Extensions](https://spec.graphql.org/September2025/#sec-Interface-Extensions)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceTypeExtension<'src> {
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<FieldDefinition<'src>>,
    pub implements: Vec<Name<'src>>,
    pub name: Name<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax:
        Option<InterfaceTypeExtensionSyntax<'src>>,
}

/// Syntax detail for an [`InterfaceTypeExtension`].
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceTypeExtensionSyntax<'src> {
    pub ampersands: Vec<GraphQLToken<'src>>,
    pub braces: Option<DelimiterPair<'src>>,
    pub extend_keyword: GraphQLToken<'src>,
    pub implements_keyword:
        Option<GraphQLToken<'src>>,
    pub interface_keyword: GraphQLToken<'src>,
    pub leading_ampersand:
        Option<GraphQLToken<'src>>,
}

#[inherent]
impl AstNode for InterfaceTypeExtension<'_> {
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
