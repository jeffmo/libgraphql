use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DirectiveAnnotation;
use crate::ast::Name;
use crate::ast::StringValue;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// An enum value definition within an enum type.
///
/// See
/// [Enum Value Definitions](https://spec.graphql.org/September2025/#EnumValuesDefinition)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct EnumValueDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
}

#[inherent]
impl AstNode for EnumValueDefinition<'_> {
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
