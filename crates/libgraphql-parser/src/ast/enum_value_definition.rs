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
///
/// Unlike most other AST node types, this struct has no
/// `syntax` field. The grammar
/// (`Description? EnumValue Directives[Const]?`) contains no
/// tokens beyond what the child nodes already capture:
/// the name token is in [`Name`]'s syntax, directives in
/// [`DirectiveAnnotation`]'s syntax, and description in
/// [`StringValue`]'s syntax.
#[derive(Clone, Debug, PartialEq)]
pub struct EnumValueDefinition<'src> {
    pub description: Option<StringValue<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub name: Name<'src>,
    pub span: GraphQLSourceSpan,
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
