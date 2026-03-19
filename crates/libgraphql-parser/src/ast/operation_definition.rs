use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::Name;
use crate::ast::OperationKind;
use crate::ast::SelectionSet;
use crate::ast::StringValue;
use crate::ast::VariableDefinition;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLToken;
use inherent::inherent;

/// An operation definition (query, mutation, or
/// subscription).
///
/// See
/// [Operations](https://spec.graphql.org/September2025/#sec-Language.Operations)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct OperationDefinition<'src> {
    pub description: Option<StringValue<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub name: Option<Name<'src>>,
    pub operation_kind: OperationKind,
    pub selection_set: SelectionSet<'src>,
    /// `true` for shorthand queries (`{ field }`)
    /// that omit the `query` keyword.
    pub shorthand: bool,
    pub span: ByteSpan,
    pub syntax:
        Option<Box<OperationDefinitionSyntax<'src>>>,
    pub variable_definitions:
        Vec<VariableDefinition<'src>>,
}

/// Syntax detail for an [`OperationDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct OperationDefinitionSyntax<'src> {
    /// The operation keyword (`query`, `mutation`,
    /// `subscription`). `None` for shorthand queries.
    pub operation_keyword: Option<GraphQLToken<'src>>,
    pub variable_definition_parens:
        Option<DelimiterPair<'src>>,
}

impl<'src> OperationDefinition<'src> {
    /// Returns the operation name as a string slice, or
    /// [`None`] for anonymous (shorthand) operations.
    ///
    /// Convenience accessor for `self.name`.
    #[inline]
    pub fn name_value(&self) -> Option<&str> {
        self.name.as_ref().map(|n| n.value.as_ref())
    }
}

#[inherent]
impl AstNode for OperationDefinition<'_> {
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

    /// Returns this operation's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this operation's position to line/column
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
