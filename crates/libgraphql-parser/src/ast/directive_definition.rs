use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveLocation;
use crate::ast::InputValueDefinition;
use crate::ast::Name;
use crate::ast::StringValue;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;

/// A directive definition.
///
/// See
/// [Directive Definitions](https://spec.graphql.org/September2025/#sec-Type-System.Directives)
/// in the spec.
///
/// # Spec invariant
///
/// The spec grammar requires at least one directive
/// location. For a spec-valid node, `locations` is
/// always non-empty.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveDefinition<'src> {
    pub arguments: Vec<InputValueDefinition<'src>>,
    pub description: Option<StringValue<'src>>,
    pub locations: Vec<DirectiveLocation<'src>>,
    pub name: Name<'src>,
    pub repeatable: bool,
    pub span: ByteSpan,
    pub syntax: Option<Box<DirectiveDefinitionSyntax<'src>>>,
}

/// Syntax detail for a [`DirectiveDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveDefinitionSyntax<'src> {
    pub argument_parens: Option<DelimiterPair<'src>>,
    pub at_sign: GraphQLToken<'src>,
    pub directive_keyword: GraphQLToken<'src>,
    pub on_keyword: GraphQLToken<'src>,
    pub repeatable_keyword: Option<GraphQLToken<'src>>,
}

impl<'src> DirectiveDefinition<'src> {
    /// Returns the name of this directive definition as a string
    /// slice.
    ///
    /// Convenience accessor for `self.name.value`.
    #[inline]
    pub fn name_value(&self) -> &str {
        self.name.value.as_ref()
    }
}

#[inherent]
impl AstNode for DirectiveDefinition<'_> {
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

    /// Returns this directive definition's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this directive definition's position to line/column
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
