use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DirectiveAnnotation;
use crate::ast::Name;
use crate::ast::StringValue;
use crate::ast::TypeAnnotation;
use crate::ast::Value;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;

/// An input value definition, used for field arguments and
/// input object fields.
///
/// See
/// [Input Values Definitions](https://spec.graphql.org/September2025/#InputValueDefinition)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct InputValueDefinition<'src> {
    pub default_value: Option<Value<'src>>,
    pub description: Option<StringValue<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub name: Name<'src>,
    pub span: ByteSpan,
    pub syntax: Option<Box<InputValueDefinitionSyntax<'src>>>,
    pub value_type: TypeAnnotation<'src>,
}

/// Syntax detail for an [`InputValueDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct InputValueDefinitionSyntax<'src> {
    pub colon: GraphQLToken<'src>,
    pub equals: Option<GraphQLToken<'src>>,
}

impl<'src> InputValueDefinition<'src> {
    /// Returns the name of this input value definition as a string
    /// slice.
    ///
    /// Convenience accessor for `self.name.value`.
    #[inline]
    pub fn name_value(&self) -> &str {
        self.name.value.as_ref()
    }
}

#[inherent]
impl AstNode for InputValueDefinition<'_> {
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

    /// Returns this input value definition's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this input value definition's position to line/column
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
