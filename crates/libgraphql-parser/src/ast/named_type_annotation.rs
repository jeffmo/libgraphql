use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::Name;
use crate::ast::Nullability;
use crate::GraphQLSourceSpan;
use inherent::inherent;

/// A named type reference (e.g. `String`, `String!`).
///
/// See
/// [Type References](https://spec.graphql.org/September2025/#sec-Type-References)
/// in the spec. The `span` covers the full annotation
/// including `!` when present. The underlying name span is
/// available via `name.span`.
///
/// Unlike most other AST node types, this struct has no
/// `syntax` field. The grammar contains no tokens beyond
/// what the child nodes already capture: the name token
/// is in [`Name`]'s syntax and the `!` token (if present)
/// is in [`Nullability::NonNull`]'s syntax.
#[derive(Clone, Debug, PartialEq)]
pub struct NamedTypeAnnotation<'src> {
    pub name: Name<'src>,
    pub nullability: Nullability<'src>,
    pub span: GraphQLSourceSpan,
}

#[inherent]
impl AstNode for NamedTypeAnnotation<'_> {
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
