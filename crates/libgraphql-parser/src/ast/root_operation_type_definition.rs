use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::Name;
use crate::ast::OperationKind;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A root operation type definition within a schema
/// definition (e.g. `query: Query`).
///
/// See
/// [Root Operation Types](https://spec.graphql.org/September2025/#sec-Root-Operation-Types)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct RootOperationTypeDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub operation_kind: OperationKind,
    pub named_type: Name<'src>,
    pub syntax:
        Option<RootOperationTypeDefinitionSyntax<'src>>,
}

/// Syntax detail for a
/// [`RootOperationTypeDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct RootOperationTypeDefinitionSyntax<'src> {
    pub colon: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for RootOperationTypeDefinition<'_> {
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
