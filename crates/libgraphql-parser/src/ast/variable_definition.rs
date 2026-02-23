use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DirectiveAnnotation;
use crate::ast::Name;
use crate::ast::StringValue;
use crate::ast::TypeAnnotation;
use crate::ast::Value;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A variable definition within an operation's
/// variable list (e.g. `$id: ID! = "default"`).
///
/// See
/// [Variable Definitions](https://spec.graphql.org/September2025/#sec-Language.Variables)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct VariableDefinition<'src> {
    pub default_value: Option<Value<'src>>,
    pub description: Option<StringValue<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub span: GraphQLSourceSpan,
    pub syntax:
        Option<Box<VariableDefinitionSyntax<'src>>>,
    pub var_type: TypeAnnotation<'src>,
    pub variable: Name<'src>,
}

/// Syntax detail for a [`VariableDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct VariableDefinitionSyntax<'src> {
    pub colon: GraphQLToken<'src>,
    pub dollar: GraphQLToken<'src>,
    pub equals: Option<GraphQLToken<'src>>,
}

#[inherent]
impl AstNode for VariableDefinition<'_> {
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
