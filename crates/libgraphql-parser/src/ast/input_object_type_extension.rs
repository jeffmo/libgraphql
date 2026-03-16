use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::InputValueDefinition;
use crate::ast::Name;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;

/// An input object type extension.
///
/// See
/// [Input Object Extensions](https://spec.graphql.org/September2025/#sec-Input-Object-Extensions)
/// in the spec.
///
/// # Spec invariant
///
/// The spec's directives-only form
/// (`extend input Name Directives[Const]`) requires at
/// least one directive when no `fields` are present.
/// For a spec-valid node, `directives` and `fields`
/// are never both empty.
#[derive(Clone, Debug, PartialEq)]
pub struct InputObjectTypeExtension<'src> {
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<InputValueDefinition<'src>>,
    pub name: Name<'src>,
    pub span: ByteSpan,
    pub syntax:
        Option<Box<InputObjectTypeExtensionSyntax<'src>>>,
}

/// Syntax detail for an
/// [`InputObjectTypeExtension`].
#[derive(Clone, Debug, PartialEq)]
pub struct InputObjectTypeExtensionSyntax<'src> {
    pub braces: Option<DelimiterPair<'src>>,
    pub extend_keyword: GraphQLToken<'src>,
    pub input_keyword: GraphQLToken<'src>,
}

impl<'src> InputObjectTypeExtension<'src> {
    /// Returns the name of this input object type extension as a string
    /// slice.
    ///
    /// Convenience accessor for `self.name.value`.
    #[inline]
    pub fn name_value(&self) -> &str {
        self.name.value.as_ref()
    }
}

#[inherent]
impl AstNode for InputObjectTypeExtension<'_> {
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

    /// Returns this input object type extension's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this input object type extension's position to line/column
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
