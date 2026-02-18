use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::Name;
use crate::ast::OperationKind;
use crate::ast::SelectionSet;
use crate::ast::StringValue;
use crate::ast::VariableDefinition;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// An operation definition (query, mutation, or
/// subscription).
///
/// See
/// [Operations](https://spec.graphql.org/September2025/#sec-Language.Operations)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct OperationDefinition<'src> {
    pub description: Option<StringValue<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub name: Option<Name<'src>>,
    pub operation_kind: OperationKind,
    pub selection_set: SelectionSet<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax:
        Option<OperationDefinitionSyntax<'src>>,
    pub variable_definitions:
        Vec<VariableDefinition<'src>>,
}

/// Syntax detail for an [`OperationDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct OperationDefinitionSyntax<'src> {
    /// The operation keyword (`query`, `mutation`,
    /// `subscription`). `None` for shorthand queries.
    pub operation_keyword: Option<GraphQLToken<'src>>,
    pub variable_definition_parens:
        Option<DelimiterPair<'src>>,
}

#[inherent]
impl AstNode for OperationDefinition<'_> {
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
