use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DirectiveAnnotation;
use crate::ast::Name;
use crate::ast::SelectionSet;
use crate::ast::StringValue;
use crate::ast::TypeCondition;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A named fragment definition.
///
/// See
/// [Fragment Definitions](https://spec.graphql.org/September2025/#sec-Language.Fragments)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct FragmentDefinition<'src> {
    pub description: Option<StringValue<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub name: Name<'src>,
    pub selection_set: SelectionSet<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax:
        Option<Box<FragmentDefinitionSyntax<'src>>>,
    pub type_condition: TypeCondition<'src>,
}

/// Syntax detail for a [`FragmentDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct FragmentDefinitionSyntax<'src> {
    pub fragment_keyword: GraphQLToken<'src>,
    pub on_keyword: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for FragmentDefinition<'_> {
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
