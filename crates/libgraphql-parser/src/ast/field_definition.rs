use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::InputValueDefinition;
use crate::ast::Name;
use crate::ast::StringValue;
use crate::ast::TypeAnnotation;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;

/// A field definition within an object type or interface
/// type.
///
/// See
/// [Field Definitions](https://spec.graphql.org/September2025/#FieldsDefinition)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldDefinition<'src> {
    pub parameters: Vec<InputValueDefinition<'src>>,
    pub description: Option<StringValue<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub field_type: TypeAnnotation<'src>,
    pub name: Name<'src>,
    pub span: ByteSpan,
    pub syntax: Option<Box<FieldDefinitionSyntax<'src>>>,
}

/// Syntax detail for a [`FieldDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct FieldDefinitionSyntax<'src> {
    pub argument_parens: Option<DelimiterPair<'src>>,
    pub colon: GraphQLToken<'src>,
}

impl<'src> FieldDefinition<'src> {
    /// Returns the name of this field definition as a string
    /// slice.
    ///
    /// Convenience accessor for `self.name.value`.
    #[inline]
    pub fn name_value(&self) -> &str {
        self.name.value.as_ref()
    }
}

#[inherent]
impl AstNode for FieldDefinition<'_> {
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

    /// Returns this field definition's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this field definition's position to line/column
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
