use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DirectiveAnnotation;
use crate::ast::Name;
use crate::ast::StringValue;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
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
    pub span: ByteSpan,
}

impl<'src> EnumValueDefinition<'src> {
    /// Returns the name of this enum value definition as a string
    /// slice.
    ///
    /// Convenience accessor for `self.name.value`.
    #[inline]
    pub fn name_value(&self) -> &str {
        self.name.value.as_ref()
    }
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
                self.span, sink, src,
            );
        }
    }

    /// Returns this enum value definition's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this enum value definition's position to line/column
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
