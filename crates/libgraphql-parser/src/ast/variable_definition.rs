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

/// A variable definition within an operation's
/// variable list (e.g. `$id: ID! = "default"`).
///
/// See
/// [Variable Definitions](https://spec.graphql.org/September2025/#sec-Language.Variables)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct VariableDefinition<'src> {
    pub default_value: Option<Value<'src>>,
    pub description: Option<StringValue<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub span: ByteSpan,
    pub syntax:
        Option<Box<VariableDefinitionSyntax<'src>>>,
    pub var_type: TypeAnnotation<'src>,
    pub variable: Name<'src>,
}

/// Syntax detail for a [`VariableDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct VariableDefinitionSyntax<'src> {
    pub colon: GraphQLToken<'src>,
    pub dollar: GraphQLToken<'src>,
    pub equals: Option<GraphQLToken<'src>>,
}

#[inherent]
impl AstNode for VariableDefinition<'_> {
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

    /// Returns this variable definition's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this variable definition's position to line/column
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
