use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::EnumValueDefinition;
use crate::ast::Name;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// An enum type extension.
///
/// See
/// [Enum Extensions](https://spec.graphql.org/September2025/#sec-Enum-Extensions)
/// in the spec.
///
/// # Spec invariant
///
/// The spec's directives-only form
/// (`extend enum Name Directives[Const]`) requires at
/// least one directive when no `values` are present.
/// For a spec-valid node, `directives` and `values`
/// are never both empty.
#[derive(Clone, Debug, PartialEq)]
pub struct EnumTypeExtension<'src> {
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub name: Name<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax:
        Option<EnumTypeExtensionSyntax<'src>>,
    pub values: Vec<EnumValueDefinition<'src>>,
}

/// Syntax detail for an [`EnumTypeExtension`].
#[derive(Clone, Debug, PartialEq)]
pub struct EnumTypeExtensionSyntax<'src> {
    pub braces: Option<DelimiterPair<'src>>,
    pub enum_keyword: GraphQLToken<'src>,
    pub extend_keyword: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for EnumTypeExtension<'_> {
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
