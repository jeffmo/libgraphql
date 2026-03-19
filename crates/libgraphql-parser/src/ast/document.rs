use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::Definition;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use crate::token::GraphQLTriviaToken;
use inherent::inherent;

/// Root AST node for any GraphQL document.
///
/// A document contains a list of [`Definition`]s which
/// may be type-system definitions, type-system extensions,
/// or executable definitions (operations and fragments).
///
/// The spec's
/// [`Document`](https://spec.graphql.org/September2025/#sec-Document)
/// grammar production allows both executable and
/// type-system definitions to coexist:
///
/// ```text
/// Document : Definition+
/// Definition :
///     ExecutableDefinition |
///     TypeSystemDefinitionOrExtension
/// ```
///
/// However, more constrained document types exist for
/// specific use cases:
/// [`ExecutableDocument`](https://spec.graphql.org/September2025/#ExecutableDocument)
/// (operations and fragments only) and
/// [`TypeSystemDocument`](https://spec.graphql.org/September2025/#TypeSystemDocument)
/// (type-system definitions only). The spec
/// [mandates](https://spec.graphql.org/September2025/#sec-Executable-Definitions)
/// that a document submitted for execution must contain
/// only executable definitions — but parsing is a separate
/// concern from execution.
///
/// This AST uses a single unified `Document` type that
/// can represent any of these document forms. This is
/// useful because a parser library serves many tools
/// beyond just execution services: schema linters,
/// formatters, code generators, IDE language servers, and
/// document merge/federation tools all benefit from being
/// able to parse any valid GraphQL syntax without
/// rejecting it at the parse level. Validation of which
/// definition kinds are permitted is left to downstream
/// consumers (e.g. an execution engine rejecting
/// type-system definitions). The convenience methods
/// [`schema_definitions()`](Document::schema_definitions)
/// and
/// [`executable_definitions()`](Document::executable_definitions)
/// provide easy filtering when needed.
///
/// See
/// [Document](https://spec.graphql.org/September2025/#sec-Document)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct Document<'src> {
    pub definitions: Vec<Definition<'src>>,
    pub span: ByteSpan,
    pub syntax: Option<Box<DocumentSyntax<'src>>>,
}

impl<'src> Document<'src> {
    /// Iterate over only the executable definitions
    /// (operations and fragments) in this document.
    pub fn executable_definitions(
        &self,
    ) -> impl Iterator<Item = &Definition<'src>> {
        self.definitions.iter().filter(|d| {
            matches!(
                d,
                Definition::FragmentDefinition(_)
                    | Definition::OperationDefinition(_)
            )
        })
    }

    /// Iterate over only the type-system definitions
    /// and extensions in this document.
    pub fn schema_definitions(
        &self,
    ) -> impl Iterator<Item = &Definition<'src>> {
        self.definitions.iter().filter(|d| {
            matches!(
                d,
                Definition::DirectiveDefinition(_)
                    | Definition::SchemaDefinition(_)
                    | Definition::SchemaExtension(_)
                    | Definition::TypeDefinition(_)
                    | Definition::TypeExtension(_)
            )
        })
    }

    /// Returns the trailing trivia tokens (whitespace,
    /// comments) that appear after the last definition in
    /// the document, if syntax detail was captured.
    pub fn trailing_trivia(&self) -> Option<&Vec<GraphQLTriviaToken<'src>>> {
        self.syntax.as_ref().map(|s| &s.trailing_trivia)
    }
}

// =========================================================
// Document syntax
// =========================================================

/// Syntax detail for a [`Document`].
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentSyntax<'src> {
    /// Trailing trivia at end-of-file (after the last
    /// definition). Captures whitespace, comments, etc.
    /// that would otherwise be lost.
    pub trailing_trivia:
        Vec<GraphQLTriviaToken<'src>>,
}

#[inherent]
impl AstNode for Document<'_> {
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
            // Append any trailing trivia (whitespace, comments)
            // that follows the last definition. These fall
            // outside the document span but are captured in the
            // syntax node for lossless reconstruction.
            if let Some(syntax) = &self.syntax {
                for trivia in &syntax.trailing_trivia {
                    let trivia_span = match trivia {
                        GraphQLTriviaToken::Comment { span, .. }
                        | GraphQLTriviaToken::Comma { span, .. }
                        | GraphQLTriviaToken::Whitespace {
                            span, ..
                        } => span,
                    };
                    append_span_source_slice(
                        *trivia_span, sink, src,
                    );
                }
            }
        }
    }

    /// Returns this document's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        self.span
    }

    /// Resolves this document's position to line/column
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

