//! Tests for compat-layer behavior that **cannot** be
//! validated via ground-truth comparison.
//!
//! Ground-truth tests parse the same source string with
//! both our parser and `graphql_parser`, then compare
//! the resulting ASTs. That approach cannot cover these
//! scenarios because:
//!
//! 1. **Unsupported features** — Our AST supports
//!    constructs (schema extensions, variable directives)
//!    that `graphql_parser` v0.4 has no AST
//!    representation for, so no parseable source can
//!    produce a reference AST on the `graphql_parser`
//!    side.
//!
//! 2. **Mixed-document filtering** — A single GraphQL
//!    document may contain both type-system and
//!    executable definitions, but `graphql_parser`
//!    exposes separate `parse_schema` / `parse_query`
//!    entry points that each reject the other kind.
//!
//! These tests parse source strings with our parser,
//! then assert on the compat layer's error or filtering
//! behavior.

use crate::compat_graphql_parser_v0_4::to_graphql_parser_query_ast;
use crate::compat_graphql_parser_v0_4::to_graphql_parser_schema_ast;
use crate::GraphQLParser;

// ─────────────────────────────────────────────
// Unsupported-feature error reporting
// ─────────────────────────────────────────────

/// Verifies that `SchemaExtension` nodes produce an
/// `UnsupportedFeature` error and are omitted from the
/// output document.
///
/// `graphql_parser` v0.4 has no `SchemaExtension`
/// variant in its `schema::Definition` enum, so our
/// compat layer must report this as unsupported rather
/// than silently dropping data.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_extension_produces_error() {
    let our_ast = GraphQLParser::new(
        "extend schema @auth { query: Query }",
    )
    .parse_schema_document();
    assert!(
        !our_ast.has_errors(),
        "Our parser should accept schema extensions",
    );
    let doc = our_ast.into_valid_ast().unwrap();

    let result = to_graphql_parser_schema_ast(&doc);
    assert!(result.has_errors());
    assert_eq!(result.errors.len(), 1);

    match result.errors[0].kind() {
        crate::GraphQLParseErrorKind::UnsupportedFeature {
            feature,
        } => {
            assert_eq!(feature, "schema extension");
        },
        other => panic!(
            "Expected UnsupportedFeature, got {other:?}",
        ),
    }

    // Schema extension is dropped from output
    let gp_doc = result.into_ast().unwrap();
    assert!(gp_doc.definitions.is_empty());
}

/// Verifies that variable definitions with directives
/// produce an `UnsupportedFeature` error, since
/// `graphql_parser` v0.4 has no directives field on
/// `VariableDefinition`.
///
/// Variable directives are valid per the GraphQL
/// September 2025 spec, but `graphql_parser` v0.4
/// predates that addition.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_variable_directives_produce_error() {
    let our_ast = GraphQLParser::new(
        "query Q($x: Int @deprecated) { field }",
    )
    .parse_executable_document();
    assert!(
        !our_ast.has_errors(),
        "Our parser should accept variable directives",
    );
    let doc = our_ast.into_valid_ast().unwrap();

    let result = to_graphql_parser_query_ast(&doc);
    assert!(result.has_errors());
    assert_eq!(result.errors.len(), 1);

    match result.errors[0].kind() {
        crate::GraphQLParseErrorKind::UnsupportedFeature {
            feature,
        } => {
            assert_eq!(feature, "variable directives");
        },
        other => panic!(
            "Expected UnsupportedFeature, got {other:?}",
        ),
    }
}

// ─────────────────────────────────────────────
// Mixed-document definition filtering
// ─────────────────────────────────────────────

/// Verifies that executable definitions (operations,
/// fragments) are silently skipped during schema
/// conversion.
///
/// A valid GraphQL document may contain both type-system
/// and executable definitions, but `graphql_parser`
/// uses separate `parse_schema` / `parse_query` entry
/// points. Our compat layer's
/// `to_graphql_parser_schema_ast` must silently ignore
/// executable definitions.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_executable_defs_skipped_in_schema_conversion() {
    let our_ast = GraphQLParser::new(
        "\
query Q { field }

type User { id: ID! }
",
    )
    .parse_mixed_document();
    assert!(!our_ast.has_errors());
    let doc = our_ast.into_valid_ast().unwrap();

    let result = to_graphql_parser_schema_ast(&doc);
    assert!(result.is_ok());
    let gp_doc = result.into_valid_ast().unwrap();

    // Only the type definition should be present;
    // the query operation should be silently skipped.
    assert_eq!(gp_doc.definitions.len(), 1);
    match &gp_doc.definitions[0] {
        graphql_parser::schema::Definition::TypeDefinition(
            graphql_parser::schema::TypeDefinition::Object(obj),
        ) => {
            assert_eq!(obj.name, "User");
        },
        other => panic!(
            "Expected TypeDefinition::Object(User), \
             got {other:?}",
        ),
    }
}

/// Verifies that type-system definitions are silently
/// skipped during query conversion.
///
/// Mirrors `test_executable_defs_skipped_in_schema_conversion`
/// for the query direction: our compat layer's
/// `to_graphql_parser_query_ast` must silently ignore
/// type definitions.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_type_system_defs_skipped_in_query_conversion() {
    let our_ast = GraphQLParser::new(
        "\
scalar DateTime

query Q { field }
",
    )
    .parse_mixed_document();
    assert!(!our_ast.has_errors());
    let doc = our_ast.into_valid_ast().unwrap();

    let result = to_graphql_parser_query_ast(&doc);
    assert!(result.is_ok());
    let gp_doc = result.into_valid_ast().unwrap();

    // Only the query operation should be present;
    // the scalar type definition should be silently
    // skipped.
    assert_eq!(gp_doc.definitions.len(), 1);
    match &gp_doc.definitions[0] {
        graphql_parser::query::Definition::Operation(
            graphql_parser::query::OperationDefinition::Query(q),
        ) => {
            assert_eq!(q.name, Some("Q".to_string()));
        },
        other => panic!(
            "Expected Operation::Query(Q), got {other:?}",
        ),
    }
}
