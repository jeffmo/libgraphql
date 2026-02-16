//! Tests for Document Types.
//!
//! These tests verify that schema documents only accept type system definitions
//! and that executable documents only accept operations and fragments. They also
//! verify edge cases like empty documents and documents containing only
//! whitespace or comments.
//!
//! Written by Claude Code, reviewed by a human.

use crate::legacy_ast;
use crate::DefinitionKind;
use crate::DocumentKind;
use crate::GraphQLParseErrorKind;
use crate::tests::utils::parse_executable;
use crate::tests::utils::parse_schema;

// =============================================================================
// Schema Document Tests
// =============================================================================

/// Verifies that schema documents accept type definitions.
///
/// A schema document should successfully parse type definitions such as object
/// types, interface types, scalar types, etc.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Type-System>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_schema_document_only_types() {
    let doc = parse_schema("type Query { field: String }")
        .into_valid_ast()
        .unwrap();
    assert_eq!(doc.definitions.len(), 1);
    assert!(matches!(
        &doc.definitions[0],
        legacy_ast::schema::Definition::TypeDefinition(_),
    ));
}

/// Verifies that query operations in schema documents produce errors.
///
/// Schema documents should only contain type system definitions, not executable
/// definitions like queries.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Type-System>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_schema_rejects_operation() {
    let result = parse_schema("query GetUser { name }");
    assert!(result.has_errors());

    // Verify at least one error is a WrongDocumentKind indicating an operation
    // was found in a schema document. Additional errors may occur due to error
    // recovery.
    let has_wrong_doc_error = result.errors.iter().any(|e| {
        matches!(
            e.kind(),
            GraphQLParseErrorKind::WrongDocumentKind {
                found: DefinitionKind::Operation,
                document_kind: DocumentKind::Schema,
            },
        )
    });
    assert!(
        has_wrong_doc_error,
        "Expected at least one WrongDocumentKind(Operation, Schema) error, \
         got: {:?}",
        result.errors.iter().map(|e| e.kind()).collect::<Vec<_>>(),
    );
}

/// Verifies that fragment definitions in schema documents produce errors.
///
/// Fragments are executable definitions and should not appear in schema
/// documents.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Type-System>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_schema_rejects_fragment() {
    let result = parse_schema("fragment UserFields on User { name }");
    assert!(result.has_errors());
}

/// Verifies that mutation operations in schema documents produce errors.
///
/// This test ensures the parser correctly rejects mutations in schema documents
/// and does not hang during error recovery.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Type-System>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_schema_rejects_mutation() {
    let result = parse_schema("mutation CreateUser { createUser { id } }");
    assert!(result.has_errors());

    // Verify at least one error is a WrongDocumentKind indicating a mutation
    // (operation) was found in a schema document
    let has_wrong_doc_error = result.errors.iter().any(|e| {
        matches!(
            e.kind(),
            GraphQLParseErrorKind::WrongDocumentKind {
                found: DefinitionKind::Operation,
                document_kind: DocumentKind::Schema,
            },
        )
    });
    assert!(
        has_wrong_doc_error,
        "Expected at least one WrongDocumentKind(Operation, Schema) error, \
         got: {:?}",
        result.errors.iter().map(|e| e.kind()).collect::<Vec<_>>(),
    );
}

/// Verifies that subscription operations in schema documents produce errors.
///
/// This test ensures the parser correctly rejects subscriptions in schema
/// documents and does not hang during error recovery.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Type-System>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_schema_rejects_subscription() {
    let result = parse_schema("subscription OnMessage { newMessage { text } }");
    assert!(result.has_errors());

    // Verify at least one error is a WrongDocumentKind indicating a
    // subscription (operation) was found in a schema document
    let has_wrong_doc_error = result.errors.iter().any(|e| {
        matches!(
            e.kind(),
            GraphQLParseErrorKind::WrongDocumentKind {
                found: DefinitionKind::Operation,
                document_kind: DocumentKind::Schema,
            },
        )
    });
    assert!(
        has_wrong_doc_error,
        "Expected at least one WrongDocumentKind(Operation, Schema) error, \
         got: {:?}",
        result.errors.iter().map(|e| e.kind()).collect::<Vec<_>>(),
    );
}

/// Verifies that shorthand queries in schema documents produce errors.
///
/// A shorthand query is an anonymous query written as just a selection set
/// (e.g., `{ field }`). These are executable definitions and should not appear
/// in schema documents.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Type-System>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_schema_rejects_shorthand_query() {
    let result = parse_schema("{ field }");
    assert!(result.has_errors());

    // Verify at least one error is a WrongDocumentKind indicating a shorthand
    // query (operation) was found in a schema document
    let has_wrong_doc_error = result.errors.iter().any(|e| {
        matches!(
            e.kind(),
            GraphQLParseErrorKind::WrongDocumentKind {
                found: DefinitionKind::Operation,
                document_kind: DocumentKind::Schema,
            },
        )
    });
    assert!(
        has_wrong_doc_error,
        "Expected at least one WrongDocumentKind(Operation, Schema) error, \
         got: {:?}",
        result.errors.iter().map(|e| e.kind()).collect::<Vec<_>>(),
    );
}

// =============================================================================
// Executable Document Tests
// =============================================================================

/// Verifies that executable documents accept operations and fragments.
///
/// Executable documents should successfully parse queries, mutations,
/// subscriptions, and fragment definitions.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Executable-Definitions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_executable_document_only_ops() {
    // Test query operation
    let query_doc = parse_executable("query GetUser { name }")
        .into_valid_ast()
        .unwrap();
    assert_eq!(query_doc.definitions.len(), 1);
    assert!(matches!(
        &query_doc.definitions[0],
        legacy_ast::operation::Definition::Operation(_),
    ));

    // Test fragment definition
    let frag_doc = parse_executable("fragment UserFields on User { name }")
        .into_valid_ast()
        .unwrap();
    assert_eq!(frag_doc.definitions.len(), 1);
    assert!(matches!(
        &frag_doc.definitions[0],
        legacy_ast::operation::Definition::Fragment(_),
    ));
}

/// Verifies that type definitions in executable documents produce errors.
///
/// Type definitions are schema definitions and should not appear in executable
/// documents.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Executable-Definitions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_executable_rejects_type() {
    let result = parse_executable("type Query { field: String }");
    assert!(result.has_errors());

    // Verify at least one error is a WrongDocumentKind indicating a type
    // definition was found in an executable document
    let has_wrong_doc_error = result.errors.iter().any(|e| {
        matches!(
            e.kind(),
            GraphQLParseErrorKind::WrongDocumentKind {
                found: DefinitionKind::TypeDefinition,
                document_kind: DocumentKind::Executable,
            },
        )
    });
    assert!(
        has_wrong_doc_error,
        "Expected at least one WrongDocumentKind(TypeDefinition, Executable) \
         error, got: {:?}",
        result.errors.iter().map(|e| e.kind()).collect::<Vec<_>>(),
    );
}

/// Verifies that directive definitions in executable documents produce errors.
///
/// Directive definitions are schema definitions and should not appear in
/// executable documents.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Executable-Definitions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_executable_rejects_directive_def() {
    let result = parse_executable("directive @deprecated on FIELD_DEFINITION");
    assert!(result.has_errors());

    // Verify at least one error is a WrongDocumentKind indicating a directive
    // definition was found in an executable document
    let has_wrong_doc_error = result.errors.iter().any(|e| {
        matches!(
            e.kind(),
            GraphQLParseErrorKind::WrongDocumentKind {
                found: DefinitionKind::DirectiveDefinition,
                document_kind: DocumentKind::Executable,
            },
        )
    });
    assert!(
        has_wrong_doc_error,
        "Expected at least one WrongDocumentKind(DirectiveDefinition, \
         Executable) error, got: {:?}",
        result.errors.iter().map(|e| e.kind()).collect::<Vec<_>>(),
    );
}

// =============================================================================
// Empty and Trivial Document Tests
// =============================================================================

/// Verifies that empty documents parse successfully.
///
/// An empty document (containing no definitions) is valid for both schema and
/// executable document types.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_empty_document() {
    // Empty schema document
    let schema_doc = parse_schema("").into_valid_ast().unwrap();
    assert_eq!(schema_doc.definitions.len(), 0);

    // Empty executable document
    let exec_doc = parse_executable("").into_valid_ast().unwrap();
    assert_eq!(exec_doc.definitions.len(), 0);
}

/// Verifies that whitespace-only documents parse successfully.
///
/// Documents containing only whitespace (spaces, tabs, newlines) should parse
/// as empty documents with no definitions.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_whitespace_only() {
    // Schema document with only whitespace
    let schema_doc = parse_schema("   \n\t   ").into_valid_ast().unwrap();
    assert_eq!(schema_doc.definitions.len(), 0);

    // Executable document with only whitespace
    let exec_doc = parse_executable("   \n\t   ").into_valid_ast().unwrap();
    assert_eq!(exec_doc.definitions.len(), 0);
}

/// Verifies that comments-only documents parse successfully.
///
/// Documents containing only comments should parse as empty documents with no
/// definitions. Comments are ignored by the parser.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_comments_only() {
    // Schema document with only comments
    let schema_doc = parse_schema("# This is a comment\n# Another comment")
        .into_valid_ast()
        .unwrap();
    assert_eq!(schema_doc.definitions.len(), 0);

    // Executable document with only comments
    let exec_doc = parse_executable("# Just a comment\n# And another one")
        .into_valid_ast()
        .unwrap();
    assert_eq!(exec_doc.definitions.len(), 0);
}
