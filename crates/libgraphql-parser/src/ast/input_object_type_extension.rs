use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::InputValueDefinition;
use crate::ast::Name;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// An input object type extension.
///
/// See
/// [Input Object Extensions](https://spec.graphql.org/September2025/#sec-Input-Object-Extensions)
/// in the spec.
///
/// # Spec invariant
///
/// The spec's directives-only form
/// (`extend input Name Directives[Const]`) requires at
/// least one directive when no `fields` are present.
/// For a spec-valid node, `directives` and `fields`
/// are never both empty.
#[derive(Clone, Debug, PartialEq)]
pub struct InputObjectTypeExtension<'src> {
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<InputValueDefinition<'src>>,
    pub name: Name<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax:
        Option<InputObjectTypeExtensionSyntax<'src>>,
}

/// Syntax detail for an
/// [`InputObjectTypeExtension`].
#[derive(Clone, Debug, PartialEq)]
pub struct InputObjectTypeExtensionSyntax<'src> {
    pub braces: Option<DelimiterPair<'src>>,
    pub extend_keyword: GraphQLToken<'src>,
    pub input_keyword: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for InputObjectTypeExtension<'_> {
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
