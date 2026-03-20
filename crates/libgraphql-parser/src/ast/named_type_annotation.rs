use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::Name;
use crate::ast::Nullability;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
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
    pub span: ByteSpan,
}

impl<'src> NamedTypeAnnotation<'src> {
    /// Returns `true` if this type annotation is nullable
    /// (i.e. does **not** have a trailing `!`).
    pub fn nullable(&self) -> bool {
        matches!(self.nullability, Nullability::Nullable)
    }

    /// Returns the name of this type annotation as a string
    /// slice.
    ///
    /// Convenience accessor for `self.name.value`.
    #[inline]
    pub fn name_value(&self) -> &str {
        self.name.value.as_ref()
    }
}

#[inherent]
impl AstNode for NamedTypeAnnotation<'_> {
    /// See [`AstNode::append_source()`](crate::ast::AstNode::append_source).
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                self.span, sink, src,
            );
        }
    }

    /// Returns this type annotation's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this type annotation's position to line/column
    /// coordinates using the given [`SourceMap`].
    ///
    /// Returns [`None`] if the byte offsets cannot be resolved
    /// (e.g. the span was synthetically constructed without
    /// valid position data).
    #[inline]
    pub fn source_span(
        &self,
        source_map: &SourceMap,
    ) -> Option<SourceSpan> {
        self.byte_span().resolve(source_map)
    }
}
