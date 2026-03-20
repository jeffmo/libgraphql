use crate::ast::AstNode;
use crate::ast::DirectiveAnnotation;
use crate::ast::FieldSelection;
use crate::ast::FragmentSpread;
use crate::ast::InlineFragment;
use crate::ast::Name;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use inherent::inherent;

/// A single selection within a selection set.
///
/// See
/// [Selection Sets](https://spec.graphql.org/September2025/#sec-Selection-Sets)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub enum Selection<'src> {
    Field(FieldSelection<'src>),
    FragmentSpread(FragmentSpread<'src>),
    InlineFragment(InlineFragment<'src>),
}

impl<'src> Selection<'src> {
    /// Returns the directive annotations applied to this selection.
    pub fn directive_annotations(&self) -> &[DirectiveAnnotation<'src>] {
        match self {
            Self::Field(s) => &s.directives,
            Self::FragmentSpread(s) => &s.directives,
            Self::InlineFragment(s) => &s.directives,
        }
    }

    /// Returns the name of this selection, or [`None`] for
    /// inline fragments (which have no name).
    pub fn name(&self) -> Option<&Name<'src>> {
        match self {
            Self::Field(s) => Some(&s.name),
            Self::FragmentSpread(s) => Some(&s.name),
            Self::InlineFragment(_) => None,
        }
    }

    /// Returns the name of this selection as a string slice,
    /// or [`None`] for inline fragments.
    ///
    /// Convenience accessor for `self.name().value`.
    pub fn name_value(&self) -> Option<&str> {
        self.name().map(|n| n.value.as_ref())
    }
}

#[inherent]
impl AstNode for Selection<'_> {
    /// See [`AstNode::append_source()`](crate::ast::AstNode::append_source).
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        match self {
            Selection::Field(s) => {
                s.append_source(sink, source)
            },
            Selection::FragmentSpread(s) => {
                s.append_source(sink, source)
            },
            Selection::InlineFragment(s) => {
                s.append_source(sink, source)
            },
        }
    }

    /// Returns this selection's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    pub fn byte_span(&self) -> ByteSpan {
        match self {
            Self::Field(s) => s.span,
            Self::FragmentSpread(s) => s.span,
            Self::InlineFragment(s) => s.span,
        }
    }

    /// Resolves this selection's position to line/column
    /// coordinates using the given [`SourceMap`].
    ///
    /// Returns [`None`] if the byte offsets cannot be resolved
    /// (e.g. the span was synthetically constructed without
    /// valid position data).
    pub fn source_span(
        &self,
        source_map: &SourceMap,
    ) -> Option<SourceSpan> {
        self.byte_span().resolve(source_map)
    }
}
