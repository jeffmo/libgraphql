use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveLocation;
use crate::ast::InputValueDefinition;
use crate::ast::Name;
use crate::ast::StringValue;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A directive definition.
///
/// See
/// [Directive Definitions](https://spec.graphql.org/September2025/#sec-Type-System.Directives)
/// in the spec.
///
/// # Spec invariant
///
/// The spec grammar requires at least one directive
/// location. For a spec-valid node, `locations` is
/// always non-empty.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveDefinition<'src> {
    pub arguments: Vec<InputValueDefinition<'src>>,
    pub description: Option<StringValue<'src>>,
    pub locations: Vec<DirectiveLocation<'src>>,
    pub name: Name<'src>,
    pub repeatable: bool,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<Box<DirectiveDefinitionSyntax<'src>>>,
}

/// Syntax detail for a [`DirectiveDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveDefinitionSyntax<'src> {
    pub argument_parens: Option<DelimiterPair<'src>>,
    pub at_sign: GraphQLToken<'src>,
    pub directive_keyword: GraphQLToken<'src>,
    pub on_keyword: GraphQLToken<'src>,
    pub repeatable_keyword: Option<GraphQLToken<'src>>,
}

#[inherent]
impl AstNode for DirectiveDefinition<'_> {
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
