use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DirectiveAnnotation;
use crate::ast::Name;
use crate::ast::SelectionSet;
use crate::ast::StringValue;
use crate::ast::TypeCondition;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
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
    pub span: ByteSpan,
    pub syntax:
        Option<Box<FragmentDefinitionSyntax<'src>>>,
    pub type_condition: TypeCondition<'src>,
}

/// Syntax detail for a [`FragmentDefinition`].
///
/// The `on` keyword token is stored in the
/// [`TypeConditionSyntax`](crate::ast::TypeConditionSyntax)
/// of the fragment's [`type_condition`](FragmentDefinition::type_condition).
#[derive(Clone, Debug, PartialEq)]
pub struct FragmentDefinitionSyntax<'src> {
    pub fragment_keyword: GraphQLToken<'src>,
}

impl<'src> FragmentDefinition<'src> {
    /// Returns the name of this fragment definition as a string
    /// slice.
    ///
    /// Convenience accessor for `self.name.value`.
    #[inline]
    pub fn name_value(&self) -> &str {
        self.name.value.as_ref()
    }
}

#[inherent]
impl AstNode for FragmentDefinition<'_> {
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

    /// Returns this fragment definition's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this fragment definition's position to line/column
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
