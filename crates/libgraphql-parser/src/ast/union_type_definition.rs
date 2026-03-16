use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DirectiveAnnotation;
use crate::ast::Name;
use crate::ast::StringValue;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;

/// A union type definition.
///
/// See
/// [Unions](https://spec.graphql.org/September2025/#sec-Unions)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct UnionTypeDefinition<'src> {
    pub description: Option<StringValue<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub members: Vec<Name<'src>>,
    pub name: Name<'src>,
    pub span: ByteSpan,
    pub syntax: Option<Box<UnionTypeDefinitionSyntax<'src>>>,
}

/// Syntax detail for a [`UnionTypeDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct UnionTypeDefinitionSyntax<'src> {
    pub equals: Option<GraphQLToken<'src>>,
    pub leading_pipe: Option<GraphQLToken<'src>>,
    pub pipes: Vec<GraphQLToken<'src>>,
    pub union_keyword: GraphQLToken<'src>,
}

impl<'src> UnionTypeDefinition<'src> {
    /// Returns the name of this union type definition as a string
    /// slice.
    ///
    /// Convenience accessor for `self.name.value`.
    #[inline]
    pub fn name_value(&self) -> &str {
        self.name.value.as_ref()
    }
}

#[inherent]
impl AstNode for UnionTypeDefinition<'_> {
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

    /// Returns this union type definition's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this union type definition's position to line/column
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
