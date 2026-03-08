//! Tests for `GraphQLParserConfig` behavior, specifically the `retain_syntax`
//! flag that controls whether `*Syntax` structs are populated on AST nodes.
//!
//! Written by Claude Code, reviewed by a human.

use crate::ast;
use crate::GraphQLParser;
use crate::GraphQLParserConfig;
use crate::token::GraphQLTriviaToken;

/// Verifies that the default parser config populates `.syntax` on
/// representative AST nodes (OperationDefinition, SelectionSet, Field, Name).
///
/// The default config uses `retain_syntax = true`, so all syntax detail should
/// be present.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn default_config_populates_syntax() {
    let source = "query { field }";
    let result = GraphQLParser::new(source).parse_executable_document();
    assert!(!result.has_errors());

    let doc = result.into_valid_ast().unwrap();

    // Document syntax
    assert!(doc.syntax.is_some(), "Document should have syntax with default config");

    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        // OperationDefinition syntax
        assert!(
            op.syntax.is_some(),
            "OperationDefinition should have syntax with default config",
        );
        let op_syntax = op.syntax.as_ref().unwrap();
        assert!(
            op_syntax.operation_keyword.is_some(),
            "Non-shorthand query should have an operation keyword token",
        );

        // SelectionSet syntax
        assert!(
            op.selection_set.syntax.is_some(),
            "SelectionSet should have syntax with default config",
        );

        // Field syntax
        if let ast::Selection::Field(field) = &op.selection_set.selections[0] {
            assert!(
                field.syntax.is_some(),
                "Field should have syntax with default config",
            );

            // Name syntax
            assert!(
                field.name.syntax.is_some(),
                "Name should have syntax with default config",
            );
        } else {
            panic!("Expected a Field selection");
        }
    } else {
        panic!("Expected an OperationDefinition");
    }
}

/// Verifies that `GraphQLParserConfig::lean()` omits `.syntax` on all AST
/// nodes. Every `syntax` field should be `None`.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn lean_config_omits_syntax() {
    let source = "query { field }";
    let result = GraphQLParser::with_config(source, GraphQLParserConfig::lean())
        .parse_executable_document();
    assert!(!result.has_errors());

    let doc = result.into_valid_ast().unwrap();

    // Document syntax
    assert!(doc.syntax.is_none(), "Document should have no syntax with lean config");

    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        // OperationDefinition syntax
        assert!(
            op.syntax.is_none(),
            "OperationDefinition should have no syntax with lean config",
        );

        // SelectionSet syntax
        assert!(
            op.selection_set.syntax.is_none(),
            "SelectionSet should have no syntax with lean config",
        );

        // Field and Name syntax
        if let ast::Selection::Field(field) = &op.selection_set.selections[0] {
            assert!(
                field.syntax.is_none(),
                "Field should have no syntax with lean config",
            );
            assert!(
                field.name.syntax.is_none(),
                "Name should have no syntax with lean config",
            );
        } else {
            panic!("Expected a Field selection");
        }
    } else {
        panic!("Expected an OperationDefinition");
    }
}

/// Verifies that lean config still produces correct span information. Spans
/// are always present regardless of the `retain_syntax` setting.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn lean_config_still_has_spans() {
    //                      11111
    //            012345678901234
    let source = "query { field }";
    let result = GraphQLParser::with_config(source, GraphQLParserConfig::lean())
        .parse_executable_document();
    assert!(!result.has_errors());

    let doc = result.into_valid_ast().unwrap();

    // Document span covers entire source
    assert_eq!(doc.span.start, 0);
    assert_eq!(doc.span.end as usize, 15);

    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        // Operation starts at "query" (byte 0)
        assert_eq!(op.span.start, 0);

        // SelectionSet starts at "{" (byte 6)
        assert_eq!(op.selection_set.span.start, 6);
        assert_eq!(op.selection_set.span.end as usize, 15);

        // Field "field" starts at byte 8
        if let ast::Selection::Field(field) = &op.selection_set.selections[0] {
            assert_eq!(field.span.start, 8);
            assert_eq!(field.name.span.start, 8);
        } else {
            panic!("Expected a Field selection");
        }
    } else {
        panic!("Expected an OperationDefinition");
    }
}

/// Verifies that lean config preserves all semantic data: name values,
/// operation kinds, field counts, type definitions, etc.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn lean_config_still_has_semantic_data() {
    let lean = GraphQLParserConfig::lean();

    // Test executable document
    let exec_source = "query MyQuery { user { name age } }";
    let exec_result = GraphQLParser::with_config(exec_source, lean.clone())
        .parse_executable_document();
    assert!(!exec_result.has_errors());

    let exec_doc = exec_result.into_valid_ast().unwrap();
    if let ast::Definition::OperationDefinition(op) = &exec_doc.definitions[0] {
        assert_eq!(op.operation_kind, ast::OperationKind::Query);
        assert!(!op.shorthand);
        assert_eq!(op.name.as_ref().map(|n| n.value.as_ref()), Some("MyQuery"));
        assert_eq!(op.selection_set.selections.len(), 1);

        if let ast::Selection::Field(user_field) = &op.selection_set.selections[0] {
            assert_eq!(user_field.name.value, "user");
            let nested_ss = user_field.selection_set.as_ref().unwrap();
            assert_eq!(nested_ss.selections.len(), 2);
        } else {
            panic!("Expected a Field selection");
        }
    } else {
        panic!("Expected an OperationDefinition");
    }

    // Test schema document
    let schema_source = "type User { name: String age: Int }";
    let schema_result = GraphQLParser::with_config(schema_source, lean)
        .parse_schema_document();
    assert!(!schema_result.has_errors());

    let schema_doc = schema_result.into_valid_ast().unwrap();
    if let ast::Definition::TypeDefinition(
        ast::TypeDefinition::Object(obj),
    ) = &schema_doc.definitions[0] {
        assert_eq!(obj.name.value, "User");
        assert_eq!(obj.fields.len(), 2);
        assert_eq!(obj.fields[0].name.value, "name");
        assert_eq!(obj.fields[1].name.value, "age");
    } else {
        panic!("Expected an Object type definition");
    }
}

/// Verifies that the default config produces syntax tokens that carry trivia.
/// Parses a query with extra whitespace and checks that the whitespace is
/// captured as `Whitespace` trivia on the appropriate tokens.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Insignificant-Commas>
/// <https://spec.graphql.org/September2025/#sec-White-Space>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn default_config_syntax_has_tokens_with_trivia() {
    let source = "  query { field }";
    let result = GraphQLParser::new(source).parse_executable_document();
    assert!(!result.has_errors());

    let doc = result.into_valid_ast().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        let op_syntax = op.syntax.as_ref().unwrap();
        let kw = op_syntax.operation_keyword.as_ref().unwrap();

        // The "query" keyword should have leading whitespace trivia ("  ")
        assert!(
            !kw.preceding_trivia.is_empty(),
            "query keyword should have preceding trivia for leading whitespace",
        );
        let has_whitespace = kw.preceding_trivia.iter().any(|t| {
            matches!(t, GraphQLTriviaToken::Whitespace { .. })
        });
        assert!(
            has_whitespace,
            "query keyword trivia should contain Whitespace",
        );

        // Verify the whitespace value is the leading "  "
        if let GraphQLTriviaToken::Whitespace { value, .. } = &kw.preceding_trivia[0] {
            assert_eq!(value, "  ");
        } else {
            panic!("Expected Whitespace trivia as first trivia item");
        }
    } else {
        panic!("Expected an OperationDefinition");
    }
}
