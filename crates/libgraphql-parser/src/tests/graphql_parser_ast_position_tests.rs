//! Tests for position tracking in parsed AST nodes.
//!
//! These tests verify that the parser correctly populates byte-offset span
//! information in all AST nodes. Line/column resolution is performed via
//! `SourceMap` when needed.
//!
//! Written by Claude Code, reviewed by a human.

use crate::SourceMap;
use crate::ast;
use crate::tests::utils::parse_executable;
use crate::tests::utils::parse_schema;

/// Helper: resolve a byte offset to (line, col_utf8) using a SourceMap built
/// from the given source text.
fn resolve(source: &str, byte_offset: u32) -> (usize, usize) {
    let sm = SourceMap::new_with_source(source, None);
    let pos = sm.resolve_offset(byte_offset)
        .expect("byte offset should be resolvable");
    (pos.line(), pos.col_utf8())
}

// =============================================================================
// Basic Position Tests - Operations
// =============================================================================

/// Verifies that the `query` keyword position is correctly captured.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_query_keyword() {
    //                      11111
    //            012345678901234
    let source = "query { field }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    assert_eq!(doc.definitions.len(), 1);

    // Document span covers entire source
    assert_eq!(doc.span.start as usize, 0);
    assert_eq!(doc.span.end as usize, source.len());

    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        assert_eq!(op.operation_kind, ast::OperationKind::Query);
        assert!(!op.shorthand);
        assert_eq!(resolve(source, doc.span.start).0, 0);
        assert_eq!(resolve(source, op.span.start).1, 0);
        assert_eq!(op.span.start as usize, 0);
        assert_eq!(resolve(source, op.span.end).0, 0);
        assert_eq!(resolve(source, op.span.end).1, 15);
        assert_eq!(op.span.end as usize, 15);
    } else {
        panic!("Expected a Query definition");
    }
}

/// Verifies that the `mutation` keyword position is correctly captured.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_mutation_keyword() {
    //                      1111111111
    //            01234567890123456789
    let source = "mutation { doThing }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    assert_eq!(doc.definitions.len(), 1);

    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        assert_eq!(op.operation_kind, ast::OperationKind::Mutation);
        assert!(!op.shorthand);
        assert_eq!(resolve(source, op.span.start).0, 0);
        assert_eq!(resolve(source, op.span.start).1, 0);
        assert_eq!(op.span.start as usize, 0);
        assert_eq!(resolve(source, op.span.end).0, 0);
        assert_eq!(resolve(source, op.span.end).1, 20);
        assert_eq!(op.span.end as usize, 20);
    } else {
        panic!("Expected a Mutation definition");
    }
}

/// Verifies that the `subscription` keyword position is correctly captured.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_subscription_keyword() {
    //                      11111111112222
    //            012345678901234567890123
    let source = "subscription { onEvent }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    assert_eq!(doc.definitions.len(), 1);

    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        assert_eq!(op.operation_kind, ast::OperationKind::Subscription);
        assert!(!op.shorthand);
        assert_eq!(resolve(source, op.span.start).0, 0);
        assert_eq!(resolve(source, op.span.start).1, 0);
        assert_eq!(op.span.start as usize, 0);
        assert_eq!(resolve(source, op.span.end).0, 0);
        assert_eq!(resolve(source, op.span.end).1, 24);
        assert_eq!(op.span.end as usize, 24);
    } else {
        panic!("Expected a Subscription definition");
    }
}

// =============================================================================
// Field Position Tests
// =============================================================================

/// Verifies that field name positions are correctly captured.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fields>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_field_simple() {
    //                      1111111
    //            01234567890123456
    let source = "query { myField }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        assert_eq!(op.selection_set.selections.len(), 1);
        if let ast::Selection::Field(field) = &op.selection_set.selections[0] {
            // "myField" starts at column 8 (0-based, after "query { ")
            assert_eq!(resolve(source, field.span.start).0, 0);
            assert_eq!(resolve(source, field.span.start).1, 8);
            assert_eq!(field.span.start as usize, 8);

            // field.name sub-span covers "myField" (cols 8..15)
            assert_eq!(resolve(source, field.name.span.start).0, 0);
            assert_eq!(resolve(source, field.name.span.start).1, 8);
            assert_eq!(field.name.span.start as usize, 8);
            assert_eq!(resolve(source, field.name.span.end).0, 0);
            assert_eq!(resolve(source, field.name.span.end).1, 15);
            assert_eq!(field.name.span.end as usize, 15);
        } else {
            panic!("Expected a Field selection");
        }
    } else {
        panic!("Expected a Query definition");
    }
}

/// Verifies that aliased field positions point to the alias, not the name.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fields>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_field_with_alias() {
    //                      1111111111222222
    //            01234567890123456789012345
    let source = "query { alias: realField }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        if let ast::Selection::Field(field) = &op.selection_set.selections[0] {
            // "alias" starts at column 8 (0-based)
            assert_eq!(resolve(source, field.span.start).0, 0);
            assert_eq!(resolve(source, field.span.start).1, 8);
            assert_eq!(field.span.start as usize, 8);
            assert_eq!(
                field.alias.as_ref().map(|a| &*a.value),
                Some("alias"),
            );
            assert_eq!(field.name.value, "realField");

            // alias sub-span covers "alias" (cols 8..13)
            let alias = field.alias.as_ref().unwrap();
            assert_eq!(resolve(source, alias.span.start).1, 8);
            assert_eq!(alias.span.start as usize, 8);
            assert_eq!(resolve(source, alias.span.end).1, 13);
            assert_eq!(alias.span.end as usize, 13);

            // name sub-span covers "realField" (cols 15..24)
            assert_eq!(resolve(source, field.name.span.start).1, 15);
            assert_eq!(field.name.span.start as usize, 15);
            assert_eq!(resolve(source, field.name.span.end).1, 24);
            assert_eq!(field.name.span.end as usize, 24);
        } else {
            panic!("Expected a Field selection");
        }
    } else {
        panic!("Expected a Query definition");
    }
}

// =============================================================================
// Directive Position Tests
// =============================================================================

/// Verifies that directive `@` symbol position is correctly captured.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_directive_at_symbol() {
    //                      111111111122222222223
    //            0123456789012345678901234567890
    let source = "query @skip(if: true) { field }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        assert_eq!(op.directives.len(), 1);
        let directive = &op.directives[0];
        // "@skip" starts at column 6 (0-based, after "query ")
        assert_eq!(resolve(source, directive.span.start).0, 0);
        assert_eq!(resolve(source, directive.span.start).1, 6);
        assert_eq!(directive.span.start as usize, 6);
        assert_eq!(directive.name.value, "skip");

        // directive.name sub-span covers "skip" (cols 7..11, after the @)
        assert_eq!(resolve(source, directive.name.span.start).1, 7);
        assert_eq!(directive.name.span.start as usize, 7);
        assert_eq!(resolve(source, directive.name.span.end).1, 11);
        assert_eq!(directive.name.span.end as usize, 11);
    } else {
        panic!("Expected a Query definition");
    }
}

// =============================================================================
// Variable Position Tests
// =============================================================================

/// Verifies that variable `$` symbol position is correctly captured.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Variables>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_variable_dollar() {
    //                      1111111111222222
    //            01234567890123456789012345
    let source = "query ($id: ID!) { field }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        assert_eq!(op.variable_definitions.len(), 1);
        let var_def = &op.variable_definitions[0];
        // "$id" starts at column 7 (0-based, after "query (")
        assert_eq!(resolve(source, var_def.span.start).0, 0);
        assert_eq!(resolve(source, var_def.span.start).1, 7);
        assert_eq!(var_def.span.start as usize, 7);
        assert_eq!(var_def.variable.value, "id");

        // variable name sub-span covers "id" (cols 8..10, after the $)
        assert_eq!(resolve(source, var_def.variable.span.start).1, 8);
        assert_eq!(var_def.variable.span.start as usize, 8);
        assert_eq!(resolve(source, var_def.variable.span.end).1, 10);
        assert_eq!(var_def.variable.span.end as usize, 10);
    } else {
        panic!("Expected a Query definition");
    }
}

// =============================================================================
// Fragment Position Tests
// =============================================================================

/// Verifies that `fragment` keyword position is correctly captured.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_fragment_definition() {
    //                      11111111112222222222333333
    //            012345678901234567890123456789012345
    let source = "fragment MyFragment on User { name }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::FragmentDefinition(frag) = &doc.definitions[0] {
        assert_eq!(resolve(source, frag.span.start).0, 0);
        assert_eq!(resolve(source, frag.span.start).1, 0);
        assert_eq!(frag.span.start as usize, 0);
        assert_eq!(frag.name.value, "MyFragment");

        // frag.name sub-span covers "MyFragment" (cols 9..19)
        assert_eq!(resolve(source, frag.name.span.start).1, 9);
        assert_eq!(frag.name.span.start as usize, 9);
        assert_eq!(resolve(source, frag.name.span.end).1, 19);
        assert_eq!(frag.name.span.end as usize, 19);

        // type_condition sub-span covers "on User" (cols 20..27)
        assert_eq!(resolve(source, frag.type_condition.span.start).1, 20);
        assert_eq!(frag.type_condition.span.start as usize, 20);
        assert_eq!(resolve(source, frag.type_condition.span.end).1, 27);
        assert_eq!(frag.type_condition.span.end as usize, 27);

        // type_condition.named_type sub-span covers "User" (cols 23..27)
        assert_eq!(
            resolve(source, frag.type_condition.named_type.span.start).1, 23,
        );
        assert_eq!(
            resolve(source, frag.type_condition.named_type.span.end).1, 27,
        );
    } else {
        panic!("Expected a Fragment definition");
    }
}

/// Verifies that fragment spread position is correctly captured.
///
/// The span covers the entire fragment spread including the `...` ellipsis.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#FragmentSpread>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_fragment_spread() {
    //                      1111111111222
    //            01234567890123456789012
    let source = "query { ...MyFragment }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        if let ast::Selection::FragmentSpread(spread) =
            &op.selection_set.selections[0] {
            // "..." starts at column 8 (0-based, after "query { ")
            assert_eq!(resolve(source, spread.span.start).0, 0);
            assert_eq!(resolve(source, spread.span.start).1, 8);
            assert_eq!(spread.span.start as usize, 8);
            assert_eq!(spread.name.value, "MyFragment");

            // spread.name sub-span covers "MyFragment" (cols 11..21)
            assert_eq!(resolve(source, spread.name.span.start).1, 11);
            assert_eq!(spread.name.span.start as usize, 11);
            assert_eq!(resolve(source, spread.name.span.end).1, 21);
            assert_eq!(spread.name.span.end as usize, 21);
        } else {
            panic!("Expected a FragmentSpread selection");
        }
    } else {
        panic!("Expected a Query definition");
    }
}

/// Verifies that inline fragment position is correctly captured.
///
/// The span covers the entire inline fragment including the `...` ellipsis.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#InlineFragment>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_inline_fragment() {
    //                      11111111112222222222
    //            012345678901234567890123456789
    let source = "query { ... on User { name } }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        if let ast::Selection::InlineFragment(inline) =
            &op.selection_set.selections[0] {
            // "..." starts at column 8 (0-based, after "query { ")
            assert_eq!(resolve(source, inline.span.start).0, 0);
            assert_eq!(resolve(source, inline.span.start).1, 8);
            assert_eq!(inline.span.start as usize, 8);

            // type_condition sub-span covers "on User" (cols 12..19)
            let tc = inline.type_condition.as_ref().unwrap();
            assert_eq!(resolve(source, tc.span.start).1, 12);
            assert_eq!(tc.span.start as usize, 12);
            assert_eq!(resolve(source, tc.span.end).1, 19);
            assert_eq!(tc.span.end as usize, 19);
        } else {
            panic!("Expected an InlineFragment selection");
        }
    } else {
        panic!("Expected a Query definition");
    }
}

/// Verifies that inline fragment without type condition has correct position.
///
/// The span covers the entire inline fragment including the `...` ellipsis.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#InlineFragment>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_inline_fragment_no_type() {
    //                      111111111122
    //            0123456789012345678901
    let source = "query { ... { name } }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        if let ast::Selection::InlineFragment(inline) =
            &op.selection_set.selections[0] {
            // "..." starts at column 8 (0-based, after "query { ")
            assert_eq!(resolve(source, inline.span.start).0, 0);
            assert_eq!(resolve(source, inline.span.start).1, 8);
            assert!(inline.type_condition.is_none());
        } else {
            panic!("Expected an InlineFragment selection");
        }
    } else {
        panic!("Expected a Query definition");
    }
}

// =============================================================================
// SelectionSet Span Tests
// =============================================================================

/// Verifies that selection set braces positions are correctly captured.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#SelectionSet>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_selection_set_span() {
    //                      11111
    //            012345678901234
    let source = "query { field }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        // Open brace at column 6 (0-based), close brace at column 14
        assert_eq!(resolve(source, op.selection_set.span.start).0, 0);
        assert_eq!(resolve(source, op.selection_set.span.start).1, 6);
        assert_eq!(op.selection_set.span.start as usize, 6);
        assert_eq!(resolve(source, op.selection_set.span.end).0, 0);
        assert_eq!(resolve(source, op.selection_set.span.end).1, 15);
        assert_eq!(op.selection_set.span.end as usize, 15);
    } else {
        panic!("Expected a Query definition");
    }
}

/// Verifies that multiline selection set span is correct.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#SelectionSet>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_selection_set_multiline() {
    let source = "query {\n  field\n}";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        // Open brace at (0, 6), close brace at (2, 0)
        assert_eq!(resolve(source, op.selection_set.span.start).0, 0);
        assert_eq!(resolve(source, op.selection_set.span.start).1, 6);
        assert_eq!(resolve(source, op.selection_set.span.end).0, 2);
        assert_eq!(resolve(source, op.selection_set.span.end).1, 1);
    } else {
        panic!("Expected a Query definition");
    }
}

/// Verifies that a field without a selection set has `selection_set: None`.
///
/// In the new AST, fields without braces have no SelectionSet at all
/// (unlike the legacy AST which always had an empty one).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_empty_selection_set_simple_field() {
    //                      1111111
    //            01234567890123456
    let source = "query { myField }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        if let ast::Selection::Field(field) = &op.selection_set.selections[0] {
            // Field has no nested selection set
            assert!(field.selection_set.is_none());
        } else {
            panic!("Expected a Field selection");
        }
    } else {
        panic!("Expected a Query definition");
    }
}

/// Verifies that a field with arguments but no selection set has
/// `selection_set: None`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_empty_selection_set_field_with_args() {
    //                      1111111111222222
    //            01234567890123456789012345
    let source = "query { myField(id: 123) }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        if let ast::Selection::Field(field) = &op.selection_set.selections[0] {
            // Field has arguments but no nested selection set
            assert!(!field.arguments.is_empty());
            assert!(field.selection_set.is_none());
        } else {
            panic!("Expected a Field selection");
        }
    } else {
        panic!("Expected a Query definition");
    }
}

/// Verifies that a field with directives but no selection set has
/// `selection_set: None`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_empty_selection_set_field_with_directive() {
    //                      1111111111222222222233
    //            012345678901234567890123456789012
    let source = "query { myField @skip(if: true) }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        if let ast::Selection::Field(field) = &op.selection_set.selections[0] {
            // Field has a directive but no nested selection set
            assert!(!field.directives.is_empty());
            assert!(field.selection_set.is_none());
        } else {
            panic!("Expected a Field selection");
        }
    } else {
        panic!("Expected a Query definition");
    }
}

/// Verifies that an aliased field without a selection set has
/// `selection_set: None`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_empty_selection_set_aliased_field() {
    //                      1111111111222222
    //            01234567890123456789012345
    let source = "query { alias: realField }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        if let ast::Selection::Field(field) = &op.selection_set.selections[0] {
            // Field has an alias but no nested selection set
            assert_eq!(
                field.alias.as_ref().map(|a| &*a.value),
                Some("alias"),
            );
            assert!(field.selection_set.is_none());
        } else {
            panic!("Expected a Field selection");
        }
    } else {
        panic!("Expected a Query definition");
    }
}

// =============================================================================
// Schema Definition Position Tests
// =============================================================================

/// Verifies that `schema` keyword position is correctly captured.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Schema>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_schema_definition() {
    //                      1111111111222
    //            01234567890123456789012
    let source = "schema { query: Query }";
    let result = parse_schema(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::SchemaDefinition(schema_def) = &doc.definitions[0] {
        assert_eq!(resolve(source, schema_def.span.start).0, 0);
        assert_eq!(resolve(source, schema_def.span.start).1, 0);
        assert_eq!(schema_def.span.start as usize, 0);
        assert_eq!(resolve(source, schema_def.span.end).0, 0);
        assert_eq!(resolve(source, schema_def.span.end).1, 23);
        assert_eq!(schema_def.span.end as usize, 23);
    } else {
        panic!("Expected a SchemaDefinition");
    }
}

// =============================================================================
// Type Definition Position Tests
// =============================================================================

/// Verifies that `scalar` keyword position is correctly captured.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Scalars>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_scalar_type_definition() {
    //                      11111
    //            012345678901234
    let source = "scalar DateTime";
    let result = parse_schema(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::TypeDefinition(
        ast::TypeDefinition::Scalar(scalar),
    ) = &doc.definitions[0] {
        assert_eq!(resolve(source, scalar.span.start).0, 0);
        assert_eq!(resolve(source, scalar.span.start).1, 0);
        assert_eq!(scalar.name.value, "DateTime");
    } else {
        panic!("Expected a Scalar type definition");
    }
}

/// Verifies that `type` keyword position is correctly captured for object types.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_object_type_definition() {
    //                      11111111112
    //            012345678901234567890
    let source = "type User { id: ID! }";
    let result = parse_schema(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::TypeDefinition(
        ast::TypeDefinition::Object(obj),
    ) = &doc.definitions[0] {
        assert_eq!(resolve(source, obj.span.start).0, 0);
        assert_eq!(resolve(source, obj.span.start).1, 0);
        assert_eq!(obj.span.start as usize, 0);
        assert_eq!(obj.name.value, "User");

        // obj.name sub-span covers "User" (cols 5..9)
        assert_eq!(resolve(source, obj.name.span.start).1, 5);
        assert_eq!(obj.name.span.start as usize, 5);
        assert_eq!(resolve(source, obj.name.span.end).1, 9);
        assert_eq!(obj.name.span.end as usize, 9);
    } else {
        panic!("Expected an Object type definition");
    }
}

/// Verifies that `interface` keyword position is correctly captured.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Interfaces>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_interface_type_definition() {
    //                      1111111111222222
    //            01234567890123456789012345
    let source = "interface Node { id: ID! }";
    let result = parse_schema(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::TypeDefinition(
        ast::TypeDefinition::Interface(iface),
    ) = &doc.definitions[0] {
        assert_eq!(resolve(source, iface.span.start).0, 0);
        assert_eq!(resolve(source, iface.span.start).1, 0);
        assert_eq!(iface.name.value, "Node");
    } else {
        panic!("Expected an Interface type definition");
    }
}

/// Verifies that `union` keyword position is correctly captured.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Unions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_union_type_definition() {
    //                      1111111111222222222233
    //            01234567890123456789012345678901
    let source = "union SearchResult = User | Post";
    let result = parse_schema(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::TypeDefinition(
        ast::TypeDefinition::Union(union_type),
    ) = &doc.definitions[0] {
        assert_eq!(resolve(source, union_type.span.start).0, 0);
        assert_eq!(resolve(source, union_type.span.start).1, 0);
        assert_eq!(union_type.name.value, "SearchResult");
    } else {
        panic!("Expected a Union type definition");
    }
}

/// Verifies that `enum` keyword position is correctly captured.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Enums>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_enum_type_definition() {
    //                      111111111122222222223
    //            0123456789012345678901234567890
    let source = "enum Status { ACTIVE INACTIVE }";
    let result = parse_schema(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::TypeDefinition(
        ast::TypeDefinition::Enum(enum_type),
    ) = &doc.definitions[0] {
        assert_eq!(resolve(source, enum_type.span.start).0, 0);
        assert_eq!(resolve(source, enum_type.span.start).1, 0);
        assert_eq!(enum_type.name.value, "Status");
    } else {
        panic!("Expected an Enum type definition");
    }
}

/// Verifies that `input` keyword position is correctly captured.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Input-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_input_object_type_definition() {
    //                      111111111122222222223333333333
    //            0123456789012345678901234567890123456789
    let source = "input CreateUserInput { name: String! }";
    let result = parse_schema(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::TypeDefinition(
        ast::TypeDefinition::InputObject(input_obj),
    ) = &doc.definitions[0] {
        assert_eq!(resolve(source, input_obj.span.start).0, 0);
        assert_eq!(resolve(source, input_obj.span.start).1, 0);
        assert_eq!(input_obj.name.value, "CreateUserInput");
    } else {
        panic!("Expected an InputObject type definition");
    }
}

/// Verifies that `directive` keyword position is correctly captured.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_directive_definition() {
    //                      111111111122222222223
    //            0123456789012345678901234567890
    let source = "directive @myDirective on FIELD";
    let result = parse_schema(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::DirectiveDefinition(dir_def) = &doc.definitions[0] {
        assert_eq!(resolve(source, dir_def.span.start).0, 0);
        assert_eq!(resolve(source, dir_def.span.start).1, 0);
        assert_eq!(dir_def.name.value, "myDirective");
    } else {
        panic!("Expected a DirectiveDefinition");
    }
}

// =============================================================================
// Schema Field and Input Value Position Tests
// =============================================================================

/// Verifies that field definition positions are correctly captured.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#FieldDefinition>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_field_definition() {
    //                      1111111111222222
    //            01234567890123456789012345
    let source = "type User { name: String }";
    let result = parse_schema(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::TypeDefinition(
        ast::TypeDefinition::Object(obj),
    ) = &doc.definitions[0] {
        assert_eq!(obj.fields.len(), 1);
        let field = &obj.fields[0];
        // "name" starts at column 12 (0-based, after "type User { ")
        assert_eq!(resolve(source, field.span.start).0, 0);
        assert_eq!(resolve(source, field.span.start).1, 12);
        assert_eq!(field.span.start as usize, 12);
        assert_eq!(field.name.value, "name");

        // field.name sub-span covers "name" (cols 12..16)
        assert_eq!(resolve(source, field.name.span.start).1, 12);
        assert_eq!(field.name.span.start as usize, 12);
        assert_eq!(resolve(source, field.name.span.end).1, 16);
        assert_eq!(field.name.span.end as usize, 16);

        // field.field_type sub-span covers "String" (cols 18..24)
        if let ast::TypeAnnotation::Named(named) = &field.field_type {
            assert_eq!(resolve(source, named.span.start).1, 18);
            assert_eq!(named.span.start as usize, 18);
            assert_eq!(resolve(source, named.span.end).1, 24);
            assert_eq!(named.span.end as usize, 24);
        } else {
            panic!("Expected a Named type annotation");
        }
    } else {
        panic!("Expected an Object type definition");
    }
}

/// Verifies that input value definition positions are correctly captured.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#InputValueDefinition>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_input_value_definition() {
    //                      111111111122222222223333333333
    //            0123456789012345678901234567890123456789
    let source = "input CreateUserInput { name: String! }";
    let result = parse_schema(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::TypeDefinition(
        ast::TypeDefinition::InputObject(input_obj),
    ) = &doc.definitions[0] {
        assert_eq!(input_obj.fields.len(), 1);
        let field = &input_obj.fields[0];
        // "name" starts at column 24 (0-based, after "input CreateUserInput { ")
        assert_eq!(resolve(source, field.span.start).0, 0);
        assert_eq!(resolve(source, field.span.start).1, 24);
        assert_eq!(field.span.start as usize, 24);
        assert_eq!(field.name.value, "name");

        // field.name sub-span covers "name" (cols 24..28)
        assert_eq!(resolve(source, field.name.span.start).1, 24);
        assert_eq!(field.name.span.start as usize, 24);
        assert_eq!(resolve(source, field.name.span.end).1, 28);
        assert_eq!(field.name.span.end as usize, 28);
    } else {
        panic!("Expected an InputObject type definition");
    }
}

/// Verifies that enum value definition positions are correctly captured.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#EnumValueDefinition>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_enum_value_definition() {
    //                      111111111122
    //            0123456789012345678901
    let source = "enum Status { ACTIVE }";
    let result = parse_schema(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::TypeDefinition(
        ast::TypeDefinition::Enum(enum_type),
    ) = &doc.definitions[0] {
        assert_eq!(enum_type.values.len(), 1);
        let value = &enum_type.values[0];
        // "ACTIVE" starts at column 14 (0-based, after "enum Status { ")
        assert_eq!(resolve(source, value.span.start).0, 0);
        assert_eq!(resolve(source, value.span.start).1, 14);
        assert_eq!(value.span.start as usize, 14);
        assert_eq!(value.name.value, "ACTIVE");

        // value.name sub-span covers "ACTIVE" (cols 14..20)
        assert_eq!(resolve(source, value.name.span.start).1, 14);
        assert_eq!(value.name.span.start as usize, 14);
        assert_eq!(resolve(source, value.name.span.end).1, 20);
        assert_eq!(value.name.span.end as usize, 20);
    } else {
        panic!("Expected an Enum type definition");
    }
}

// =============================================================================
// Type Extension Position Tests
// =============================================================================

/// Verifies that type extension span covers the full extension including
/// the `extend` keyword for scalar extension.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_scalar_type_extension() {
    //                      111111111122222222223333
    //            0123456789012345678901234567890123
    let source = "extend scalar DateTime @deprecated";
    let result = parse_schema(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::TypeExtension(
        ast::TypeExtension::Scalar(ext),
    ) = &doc.definitions[0] {
        // Span starts at "extend" at column 0 (0-based)
        assert_eq!(resolve(source, ext.span.start).0, 0);
        assert_eq!(resolve(source, ext.span.start).1, 0);
        assert_eq!(ext.name.value, "DateTime");
    } else {
        panic!("Expected a Scalar type extension");
    }
}

/// Verifies that type extension span covers the full extension including
/// the `extend` keyword for object extension.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_object_type_extension() {
    //                      111111111122222222223333
    //            0123456789012345678901234567890123
    let source = "extend type User { email: String }";
    let result = parse_schema(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::TypeExtension(
        ast::TypeExtension::Object(ext),
    ) = &doc.definitions[0] {
        // Span starts at "extend" at column 0 (0-based)
        assert_eq!(resolve(source, ext.span.start).0, 0);
        assert_eq!(resolve(source, ext.span.start).1, 0);
        assert_eq!(ext.name.value, "User");
    } else {
        panic!("Expected an Object type extension");
    }
}

/// Verifies that type extension span covers the full extension including
/// the `extend` keyword for interface extension.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_interface_type_extension() {
    //                      11111111112222222222333333333344444
    //            012345678901234567890123456789012345678901234
    let source = "extend interface Node { createdAt: DateTime }";
    let result = parse_schema(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::TypeExtension(
        ast::TypeExtension::Interface(ext),
    ) = &doc.definitions[0] {
        // Span starts at "extend" at column 0 (0-based)
        assert_eq!(resolve(source, ext.span.start).0, 0);
        assert_eq!(resolve(source, ext.span.start).1, 0);
        assert_eq!(ext.name.value, "Node");
    } else {
        panic!("Expected an Interface type extension");
    }
}

/// Verifies that type extension span covers the full extension including
/// the `extend` keyword for union extension.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_union_type_extension() {
    //                      1111111111222222222233333
    //            01234567890123456789012345678901234
    let source = "extend union SearchResult = Comment";
    let result = parse_schema(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::TypeExtension(
        ast::TypeExtension::Union(ext),
    ) = &doc.definitions[0] {
        // Span starts at "extend" at column 0 (0-based)
        assert_eq!(resolve(source, ext.span.start).0, 0);
        assert_eq!(resolve(source, ext.span.start).1, 0);
        assert_eq!(ext.name.value, "SearchResult");
    } else {
        panic!("Expected a Union type extension");
    }
}

/// Verifies that type extension span covers the full extension including
/// the `extend` keyword for enum extension.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_enum_type_extension() {
    //                      11111111112222222222
    //            012345678901234567890123456789
    let source = "extend enum Status { PENDING }";
    let result = parse_schema(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::TypeExtension(
        ast::TypeExtension::Enum(ext),
    ) = &doc.definitions[0] {
        // Span starts at "extend" at column 0 (0-based)
        assert_eq!(resolve(source, ext.span.start).0, 0);
        assert_eq!(resolve(source, ext.span.start).1, 0);
        assert_eq!(ext.name.value, "Status");
    } else {
        panic!("Expected an Enum type extension");
    }
}

/// Verifies that type extension span covers the full extension including
/// the `extend` keyword for input object extension.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_input_object_type_extension() {
    //                      111111111122222222223333333333444444
    //            0123456789012345678901234567890123456789012345
    let source = "extend input CreateUserInput { email: String }";
    let result = parse_schema(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::TypeExtension(
        ast::TypeExtension::InputObject(ext),
    ) = &doc.definitions[0] {
        // Span starts at "extend" at column 0 (0-based)
        assert_eq!(resolve(source, ext.span.start).0, 0);
        assert_eq!(resolve(source, ext.span.start).1, 0);
        assert_eq!(ext.name.value, "CreateUserInput");
    } else {
        panic!("Expected an InputObject type extension");
    }
}

// =============================================================================
// Edge Cases
// =============================================================================

/// Verifies that shorthand query (just selection set) is represented as an
/// OperationDefinition with `shorthand: true`.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_shorthand_query() {
    //            012345678
    let source = "{ field }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        assert!(op.shorthand);

        // Shorthand query op.span matches the selection set extent
        assert_eq!(resolve(source, op.span.start).0, 0);
        assert_eq!(resolve(source, op.span.start).1, 0);
        assert_eq!(op.span.start as usize, 0);
        assert_eq!(resolve(source, op.span.end).0, 0);
        assert_eq!(resolve(source, op.span.end).1, 9);
        assert_eq!(op.span.end as usize, 9);

        // The selection set span should have the braces positions (0-based)
        assert_eq!(resolve(source, op.span.start).0, 0);
        assert_eq!(resolve(source, op.selection_set.span.start).1, 0);
        assert_eq!(op.selection_set.span.start as usize, 0);
        assert_eq!(resolve(source, op.selection_set.span.end).0, 0);
        assert_eq!(resolve(source, op.selection_set.span.end).1, 9);
        assert_eq!(op.selection_set.span.end as usize, 9);
    } else {
        panic!("Expected an OperationDefinition");
    }
}

/// Verifies that positions work with leading whitespace.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_with_leading_whitespace() {
    let source = "\n\nquery { field }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        // "query" starts on line 2 (0-based), column 0
        assert_eq!(resolve(source, op.span.start).0, 2);
        assert_eq!(resolve(source, op.span.start).1, 0);
    } else {
        panic!("Expected a Query definition");
    }
}

/// Verifies that positions work with leading comments.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_with_leading_comments() {
    let source = "# This is a comment\nquery { field }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        // "query" starts on line 1 (0-based), column 0
        assert_eq!(resolve(source, op.span.start).0, 1);
        assert_eq!(resolve(source, op.span.start).1, 0);
    } else {
        panic!("Expected a Query definition");
    }
}

/// Verifies that multiline field positions are correct.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_multiline_selections() {
    let source = "query {\n  field1\n  field2\n}";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        assert_eq!(op.selection_set.selections.len(), 2);

        if let ast::Selection::Field(field1) =
            &op.selection_set.selections[0] {
            assert_eq!(resolve(source, field1.span.start).0, 1);
            assert_eq!(resolve(source, field1.span.start).1, 2);
        }

        if let ast::Selection::Field(field2) =
            &op.selection_set.selections[1] {
            assert_eq!(resolve(source, field2.span.start).0, 2);
            assert_eq!(resolve(source, field2.span.start).1, 2);
        }
    } else {
        panic!("Expected a Query definition");
    }
}

/// Verifies that deeply nested positions are correct.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_deeply_nested() {
    //                      1111111111222222222
    //            01234567890123456789012345678
    let source = "query { a { b { c { d } } } }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        // First level: "a" at column 8 (0-based)
        if let ast::Selection::Field(field_a) =
            &op.selection_set.selections[0] {
            assert_eq!(resolve(source, field_a.span.start).0, 0);
            assert_eq!(resolve(source, field_a.span.start).1, 8);
            assert_eq!(field_a.span.start as usize, 8);
            assert_eq!(field_a.name.value, "a");

            // field_a.selection_set span covers "{ b { c { d } } }"
            let ss_a = field_a.selection_set.as_ref().unwrap();
            assert_eq!(resolve(source, ss_a.span.start).1, 10);
            assert_eq!(ss_a.span.start as usize, 10);
            assert_eq!(resolve(source, ss_a.span.end).1, 27);
            assert_eq!(ss_a.span.end as usize, 27);

            // Second level: "b" at column 12 (0-based)
            let ss_a = field_a.selection_set.as_ref().unwrap();
            if let ast::Selection::Field(field_b) = &ss_a.selections[0] {
                assert_eq!(resolve(source, ss_a.span.start).0, 0);
                assert_eq!(resolve(source, field_b.span.start).1, 12);
                assert_eq!(field_b.name.value, "b");

                // Third level: "c" at column 16 (0-based)
                let ss_b = field_b.selection_set.as_ref().unwrap();
                if let ast::Selection::Field(field_c) = &ss_b.selections[0] {
                    assert_eq!(resolve(source, field_c.span.start).0, 0);
                    assert_eq!(resolve(source, field_c.span.start).1, 16);
                    assert_eq!(field_c.name.value, "c");

                    // Fourth level: "d" at column 20 (0-based)
                    let ss_c = field_c.selection_set.as_ref().unwrap();
                    if let ast::Selection::Field(field_d) =
                        &ss_c.selections[0] {
                        assert_eq!(resolve(source, field_d.span.start).0, 0);
                        assert_eq!(
                            resolve(source, field_d.span.start).1, 20,
                        );
                        assert_eq!(field_d.name.value, "d");
                    }
                }
            }
        }
    }
}

/// Verifies that positions are correct for long lines.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_long_lines() {
    // Create a query with a field at a high column position
    let padding = " ".repeat(95);
    let source = format!("query {{{padding}field }}");
    let result = parse_executable(&source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0]
        && let ast::Selection::Field(field) =
            &op.selection_set.selections[0] {
        // "field" starts at column 102 (0-based: 7 for "query {" + 95 spaces)
        assert_eq!(resolve(&source, field.span.start).0, 0);
        assert_eq!(resolve(&source, field.span.start).1, 102);
    }
}

/// Verifies that multiple operations have correct positions.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_multiple_operations() {
    let source = "query A { a }\nmutation B { b }\nsubscription C { c }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    assert_eq!(doc.definitions.len(), 3);

    // Query A at (0, 0) (0-based)
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        assert_eq!(op.operation_kind, ast::OperationKind::Query);
        assert_eq!(resolve(source, op.span.start).0, 0);
        assert_eq!(resolve(source, op.span.start).1, 0);
    }

    // Mutation B at (1, 0) (0-based)
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[1] {
        assert_eq!(op.operation_kind, ast::OperationKind::Mutation);
        assert_eq!(resolve(source, op.span.start).0, 1);
        assert_eq!(resolve(source, op.span.start).1, 0);
    }

    // Subscription C at (2, 0) (0-based)
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[2] {
        assert_eq!(op.operation_kind, ast::OperationKind::Subscription);
        assert_eq!(resolve(source, op.span.start).0, 2);
        assert_eq!(resolve(source, op.span.start).1, 0);
    }
}

// =============================================================================
// Unicode Tests
// =============================================================================

/// Verifies that position is correct after unicode comment.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_after_unicode_comment() {
    // Comment with emoji followed by query
    let source = "# Hello world! \u{1F389}\nquery { field }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        // "query" starts on line 1 (0-based), column 0
        assert_eq!(resolve(source, op.span.start).0, 1);
        assert_eq!(resolve(source, op.span.start).1, 0);
    } else {
        panic!("Expected a Query definition");
    }
}

/// Verifies that column position is correct after unicode in string.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_unicode_in_string() {
    // String with emoji, then a field
    let source = "query { field(arg: \"\u{1F389}\") other }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let (doc, _) = result.into_valid().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        // Check that "other" field position is captured correctly
        assert_eq!(op.selection_set.selections.len(), 2);
        if let ast::Selection::Field(other_field) =
            &op.selection_set.selections[1] {
            assert_eq!(other_field.name.value, "other");
            // The position should be after the closing ) of the argument
            assert_eq!(resolve(source, other_field.span.start).0, 0);
            // col_utf8 counts characters (not bytes), so the emoji is 1 char
            // "query { field(arg: \"" = 20 chars, then 🎉 = 1 char,
            // then "\") " = 3 chars, so "other" starts at char 24 (0-based)
            assert_eq!(resolve(source, other_field.span.start).1, 24);
        }
    }
}
