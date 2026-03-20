use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::FieldDefinition;
use crate::ast::Name;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;

/// An interface type extension.
///
/// See
/// [Interface Extensions](https://spec.graphql.org/September2025/#sec-Interface-Extensions)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceTypeExtension<'src> {
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<FieldDefinition<'src>>,
    pub implements: Vec<Name<'src>>,
    pub name: Name<'src>,
    pub span: ByteSpan,
    pub syntax:
        Option<Box<InterfaceTypeExtensionSyntax<'src>>>,
}

/// Syntax detail for an [`InterfaceTypeExtension`].
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceTypeExtensionSyntax<'src> {
    pub ampersands: Vec<GraphQLToken<'src>>,
    pub braces: Option<DelimiterPair<'src>>,
    pub extend_keyword: GraphQLToken<'src>,
    pub implements_keyword:
        Option<GraphQLToken<'src>>,
    pub interface_keyword: GraphQLToken<'src>,
    pub leading_ampersand:
        Option<GraphQLToken<'src>>,
}

impl<'src> InterfaceTypeExtension<'src> {
    /// Returns the name of this interface type extension as a string
    /// slice.
    ///
    /// Convenience accessor for `self.name.value`.
    #[inline]
    pub fn name_value(&self) -> &str {
        self.name.value.as_ref()
    }
}

#[inherent]
impl AstNode for InterfaceTypeExtension<'_> {
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

    /// Returns this interface type extension's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this interface type extension's position to line/column
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
