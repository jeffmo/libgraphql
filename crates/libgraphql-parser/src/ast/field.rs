use crate::ast::Argument;
use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::Name;
use crate::ast::SelectionSet;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;

/// A field selection within a selection set, optionally
/// aliased, with arguments, directives, and a nested
/// selection set.
///
/// See
/// [Fields](https://spec.graphql.org/September2025/#sec-Language.Fields)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct Field<'src> {
    pub alias: Option<Name<'src>>,
    pub arguments: Vec<Argument<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub name: Name<'src>,
    pub selection_set: Option<SelectionSet<'src>>,
    pub span: ByteSpan,
    pub syntax: Option<Box<FieldSyntax<'src>>>,
}

/// Syntax detail for a [`Field`].
#[derive(Clone, Debug, PartialEq)]
pub struct FieldSyntax<'src> {
    /// The colon between alias and field name. `None`
    /// when no alias is present.
    pub alias_colon: Option<GraphQLToken<'src>>,
    pub argument_parens: Option<DelimiterPair<'src>>,
}

impl<'src> Field<'src> {
    /// Returns the name of this field as a string
    /// slice.
    ///
    /// Convenience accessor for `self.name.value`.
    #[inline]
    pub fn name_value(&self) -> &str {
        self.name.value.as_ref()
    }
}

#[inherent]
impl AstNode for Field<'_> {
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

    /// Returns this field's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this field's position to line/column
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
