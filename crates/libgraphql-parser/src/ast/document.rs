use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DirectiveDefinition;
use crate::ast::FragmentDefinition;
use crate::ast::OperationDefinition;
use crate::ast::SchemaDefinition;
use crate::ast::SchemaExtension;
use crate::ast::TypeDefinition;
use crate::ast::TypeExtension;
use crate::token::GraphQLTriviaToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

// =========================================================
// Document
// =========================================================

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
    pub span: GraphQLSourceSpan,
    pub syntax: Option<Box<DocumentSyntax<'src>>>,
}

impl<'src> Document<'src> {
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
}

// =========================================================
// Definition
// =========================================================

/// A top-level definition in a GraphQL document.
///
/// Covers both type-system definitions (schema, types,
/// directives, extensions) and executable definitions
/// (operations, fragments).
///
/// See
/// [Document](https://spec.graphql.org/September2025/#sec-Document)
/// in the spec.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum Definition<'src> {
    DirectiveDefinition(DirectiveDefinition<'src>),
    FragmentDefinition(FragmentDefinition<'src>),
    OperationDefinition(OperationDefinition<'src>),
    SchemaDefinition(SchemaDefinition<'src>),
    SchemaExtension(SchemaExtension<'src>),
    TypeDefinition(TypeDefinition<'src>),
    TypeExtension(TypeExtension<'src>),
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
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                &self.span, sink, src,
            );
        }
    }
}

#[inherent]
impl AstNode for Definition<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        match self {
            Definition::DirectiveDefinition(d) => {
                d.append_source(sink, source)
            },
            Definition::FragmentDefinition(d) => {
                d.append_source(sink, source)
            },
            Definition::OperationDefinition(d) => {
                d.append_source(sink, source)
            },
            Definition::SchemaDefinition(d) => {
                d.append_source(sink, source)
            },
            Definition::SchemaExtension(d) => {
                d.append_source(sink, source)
            },
            Definition::TypeDefinition(d) => {
                d.append_source(sink, source)
            },
            Definition::TypeExtension(d) => {
                d.append_source(sink, source)
            },
        }
    }
}
