use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;

/// A directive location with its own span (unlike
/// `graphql_parser` which uses a plain enum).
///
/// See
/// [Directive Locations](https://spec.graphql.org/September2025/#DirectiveLocations)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveLocation<'src> {
    pub kind: DirectiveLocationKind,
    pub span: ByteSpan,
    pub syntax: Option<Box<DirectiveLocationSyntax<'src>>>,
}

/// The kind of location where a directive may be applied.
///
/// See
/// [Directive Locations](https://spec.graphql.org/September2025/#DirectiveLocations)
/// in the spec.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum DirectiveLocationKind {
    ArgumentDefinition,
    Enum,
    EnumValue,
    Field,
    FieldDefinition,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,
    InputFieldDefinition,
    InputObject,
    Interface,
    Mutation,
    Object,
    Query,
    Scalar,
    Schema,
    Subscription,
    Union,
    VariableDefinition,
}

/// Syntax detail for a [`DirectiveLocation`].
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveLocationSyntax<'src> {
    /// The `|` pipe token before this location (`None` for
    /// the first location).
    pub pipe: Option<GraphQLToken<'src>>,
    /// The location name token (e.g. `FIELD`, `QUERY`).
    pub token: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for DirectiveLocation<'_> {
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

    /// Returns this directive location's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this directive location's position to line/column
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
