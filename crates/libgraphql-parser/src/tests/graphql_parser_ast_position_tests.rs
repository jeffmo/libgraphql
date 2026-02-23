//! Tests for position tracking in parsed AST nodes.
//!
//! These tests verify that the parser correctly populates line/column position
//! information in all AST nodes, using 1-based line and column numbers.
//!
//! Written by Claude Code, reviewed by a human.

use crate::legacy_ast;
use crate::tests::utils::parse_executable;
use crate::tests::utils::parse_schema;

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

    let doc = result.into_valid_ast().unwrap();
    assert_eq!(doc.definitions.len(), 1);

    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        assert_eq!(query.position.line, 1);
        assert_eq!(query.position.column, 1);
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

    let doc = result.into_valid_ast().unwrap();
    assert_eq!(doc.definitions.len(), 1);

    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Mutation(mutation),
    ) = &doc.definitions[0] {
        assert_eq!(mutation.position.line, 1);
        assert_eq!(mutation.position.column, 1);
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

    let doc = result.into_valid_ast().unwrap();
    assert_eq!(doc.definitions.len(), 1);

    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Subscription(sub),
    ) = &doc.definitions[0] {
        assert_eq!(sub.position.line, 1);
        assert_eq!(sub.position.column, 1);
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        assert_eq!(query.selection_set.items.len(), 1);
        if let legacy_ast::operation::Selection::Field(field) =
            &query.selection_set.items[0] {
            // "myField" starts at column 9 (after "query { ")
            assert_eq!(field.position.line, 1);
            assert_eq!(field.position.column, 9);
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        if let legacy_ast::operation::Selection::Field(field) =
            &query.selection_set.items[0] {
            // "alias" starts at column 9
            assert_eq!(field.position.line, 1);
            assert_eq!(field.position.column, 9);
            assert_eq!(field.alias.as_deref(), Some("alias"));
            assert_eq!(field.name, "realField");
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        assert_eq!(query.directives.len(), 1);
        let directive = &query.directives[0];
        // "@skip" starts at column 7 (after "query ")
        assert_eq!(directive.position.line, 1);
        assert_eq!(directive.position.column, 7);
        assert_eq!(directive.name, "skip");
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        assert_eq!(query.variable_definitions.len(), 1);
        let var_def = &query.variable_definitions[0];
        // "$id" starts at column 8 (after "query (")
        assert_eq!(var_def.position.line, 1);
        assert_eq!(var_def.position.column, 8);
        assert_eq!(var_def.name, "id");
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Fragment(frag) = &doc.definitions[0] {
        assert_eq!(frag.position.line, 1);
        assert_eq!(frag.position.column, 1);
        assert_eq!(frag.name, "MyFragment");
    } else {
        panic!("Expected a Fragment definition");
    }
}

/// Verifies that fragment spread position is correctly captured.
///
/// `graphql_parser` records position after consuming `...`, so the
/// position points at the fragment name, not the ellipsis.
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        if let legacy_ast::operation::Selection::FragmentSpread(spread) =
            &query.selection_set.items[0] {
            // "MyFragment" starts at column 12 (after "...")
            assert_eq!(spread.position.line, 1);
            assert_eq!(spread.position.column, 12);
            assert_eq!(spread.fragment_name, "MyFragment");
        } else {
            panic!("Expected a FragmentSpread selection");
        }
    } else {
        panic!("Expected a Query definition");
    }
}

/// Verifies that inline fragment position is correctly captured.
///
/// `graphql_parser` records position after consuming `...`, so the
/// position points at the `on` keyword (or first token after `...`).
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        if let legacy_ast::operation::Selection::InlineFragment(inline) =
            &query.selection_set.items[0] {
            // "on" starts at column 13 (after "... ")
            assert_eq!(inline.position.line, 1);
            assert_eq!(inline.position.column, 13);
        } else {
            panic!("Expected an InlineFragment selection");
        }
    } else {
        panic!("Expected a Query definition");
    }
}

/// Verifies that inline fragment without type condition has correct position.
///
/// `graphql_parser` records position after consuming `...`, so the
/// position points at the `{` when no type condition is present.
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        if let legacy_ast::operation::Selection::InlineFragment(inline) =
            &query.selection_set.items[0] {
            // "{" starts at column 13 (after "... ")
            assert_eq!(inline.position.line, 1);
            assert_eq!(inline.position.column, 13);
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        // Open brace at column 7, close brace at column 15
        assert_eq!(query.selection_set.span.0.line, 1);
        assert_eq!(query.selection_set.span.0.column, 7);
        assert_eq!(query.selection_set.span.1.line, 1);
        assert_eq!(query.selection_set.span.1.column, 15);
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        // Open brace at (1, 7), close brace at (3, 1)
        assert_eq!(query.selection_set.span.0.line, 1);
        assert_eq!(query.selection_set.span.0.column, 7);
        assert_eq!(query.selection_set.span.1.line, 3);
        assert_eq!(query.selection_set.span.1.column, 1);
    } else {
        panic!("Expected a Query definition");
    }
}

/// Verifies that a field without a selection set has its empty selection_set
/// span anchored to the field's position.
///
/// When a field has no nested selection set (no `{}`), the AST still contains
/// a SelectionSet struct. Rather than using (0,0) which loses location context,
/// the span should use the field's position as an anchor point.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_empty_selection_set_simple_field() {
    //                      1111111
    //            01234567890123456
    let source = "query { myField }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        if let legacy_ast::operation::Selection::Field(field) =
            &query.selection_set.items[0] {
            // Field has no nested selection set
            assert!(field.selection_set.items.is_empty());

            // The empty selection set span should use the field's position
            // "myField" starts at column 9
            assert_eq!(field.selection_set.span.0.line, 1);
            assert_eq!(field.selection_set.span.0.column, 9);
            assert_eq!(field.selection_set.span.1.line, 1);
            assert_eq!(field.selection_set.span.1.column, 9);
        } else {
            panic!("Expected a Field selection");
        }
    } else {
        panic!("Expected a Query definition");
    }
}

/// Verifies that a field with arguments but no selection set has its empty
/// selection_set span anchored to the field's position.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_empty_selection_set_field_with_args() {
    //                      1111111111222222
    //            01234567890123456789012345
    let source = "query { myField(id: 123) }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        if let legacy_ast::operation::Selection::Field(field) =
            &query.selection_set.items[0] {
            // Field has arguments but no nested selection set
            assert!(!field.arguments.is_empty());
            assert!(field.selection_set.items.is_empty());

            // The empty selection set span should use the field's position
            // "myField" starts at column 9
            assert_eq!(field.selection_set.span.0.line, 1);
            assert_eq!(field.selection_set.span.0.column, 9);
            assert_eq!(field.selection_set.span.1.line, 1);
            assert_eq!(field.selection_set.span.1.column, 9);
        } else {
            panic!("Expected a Field selection");
        }
    } else {
        panic!("Expected a Query definition");
    }
}

/// Verifies that a field with directives but no selection set has its empty
/// selection_set span anchored to the field's position.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_empty_selection_set_field_with_directive() {
    //                      1111111111222222222233
    //            012345678901234567890123456789012
    let source = "query { myField @skip(if: true) }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        if let legacy_ast::operation::Selection::Field(field) =
            &query.selection_set.items[0] {
            // Field has a directive but no nested selection set
            assert!(!field.directives.is_empty());
            assert!(field.selection_set.items.is_empty());

            // The empty selection set span should use the field's position
            // "myField" starts at column 9
            assert_eq!(field.selection_set.span.0.line, 1);
            assert_eq!(field.selection_set.span.0.column, 9);
            assert_eq!(field.selection_set.span.1.line, 1);
            assert_eq!(field.selection_set.span.1.column, 9);
        } else {
            panic!("Expected a Field selection");
        }
    } else {
        panic!("Expected a Query definition");
    }
}

/// Verifies that an aliased field without a selection set has its empty
/// selection_set span anchored to the alias position (the field's position).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_empty_selection_set_aliased_field() {
    //                      1111111111222222
    //            01234567890123456789012345
    let source = "query { alias: realField }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        if let legacy_ast::operation::Selection::Field(field) =
            &query.selection_set.items[0] {
            // Field has an alias but no nested selection set
            assert_eq!(field.alias.as_deref(), Some("alias"));
            assert!(field.selection_set.items.is_empty());

            // The empty selection set span should use the field's position
            // (which is the alias position) "alias" starts at column 9
            assert_eq!(field.selection_set.span.0.line, 1);
            assert_eq!(field.selection_set.span.0.column, 9);
            assert_eq!(field.selection_set.span.1.line, 1);
            assert_eq!(field.selection_set.span.1.column, 9);
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::schema::Definition::SchemaDefinition(schema_def) =
        &doc.definitions[0] {
        assert_eq!(schema_def.position.line, 1);
        assert_eq!(schema_def.position.column, 1);
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::schema::Definition::TypeDefinition(
        legacy_ast::schema::TypeDefinition::Scalar(scalar),
    ) = &doc.definitions[0] {
        assert_eq!(scalar.position.line, 1);
        assert_eq!(scalar.position.column, 1);
        assert_eq!(scalar.name, "DateTime");
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::schema::Definition::TypeDefinition(
        legacy_ast::schema::TypeDefinition::Object(obj),
    ) = &doc.definitions[0] {
        assert_eq!(obj.position.line, 1);
        assert_eq!(obj.position.column, 1);
        assert_eq!(obj.name, "User");
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::schema::Definition::TypeDefinition(
        legacy_ast::schema::TypeDefinition::Interface(iface),
    ) = &doc.definitions[0] {
        assert_eq!(iface.position.line, 1);
        assert_eq!(iface.position.column, 1);
        assert_eq!(iface.name, "Node");
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::schema::Definition::TypeDefinition(
        legacy_ast::schema::TypeDefinition::Union(union_type),
    ) = &doc.definitions[0] {
        assert_eq!(union_type.position.line, 1);
        assert_eq!(union_type.position.column, 1);
        assert_eq!(union_type.name, "SearchResult");
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::schema::Definition::TypeDefinition(
        legacy_ast::schema::TypeDefinition::Enum(enum_type),
    ) = &doc.definitions[0] {
        assert_eq!(enum_type.position.line, 1);
        assert_eq!(enum_type.position.column, 1);
        assert_eq!(enum_type.name, "Status");
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::schema::Definition::TypeDefinition(
        legacy_ast::schema::TypeDefinition::InputObject(input_obj),
    ) = &doc.definitions[0] {
        assert_eq!(input_obj.position.line, 1);
        assert_eq!(input_obj.position.column, 1);
        assert_eq!(input_obj.name, "CreateUserInput");
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::schema::Definition::DirectiveDefinition(dir_def) =
        &doc.definitions[0] {
        assert_eq!(dir_def.position.line, 1);
        assert_eq!(dir_def.position.column, 1);
        assert_eq!(dir_def.name, "myDirective");
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::schema::Definition::TypeDefinition(
        legacy_ast::schema::TypeDefinition::Object(obj),
    ) = &doc.definitions[0] {
        assert_eq!(obj.fields.len(), 1);
        let field = &obj.fields[0];
        // "name" starts at column 13 (after "type User { ")
        assert_eq!(field.position.line, 1);
        assert_eq!(field.position.column, 13);
        assert_eq!(field.name, "name");
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::schema::Definition::TypeDefinition(
        legacy_ast::schema::TypeDefinition::InputObject(input_obj),
    ) = &doc.definitions[0] {
        assert_eq!(input_obj.fields.len(), 1);
        let field = &input_obj.fields[0];
        // "name" starts at column 25 (after "input CreateUserInput { ")
        assert_eq!(field.position.line, 1);
        assert_eq!(field.position.column, 25);
        assert_eq!(field.name, "name");
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::schema::Definition::TypeDefinition(
        legacy_ast::schema::TypeDefinition::Enum(enum_type),
    ) = &doc.definitions[0] {
        assert_eq!(enum_type.values.len(), 1);
        let value = &enum_type.values[0];
        // "ACTIVE" starts at column 15 (after "enum Status { ")
        assert_eq!(value.position.line, 1);
        assert_eq!(value.position.column, 15);
        assert_eq!(value.name, "ACTIVE");
    } else {
        panic!("Expected an Enum type definition");
    }
}

// =============================================================================
// Type Extension Position Tests
// =============================================================================

/// Verifies that type extension position is correctly captured for scalar
/// extension.
///
/// `graphql_parser` records position after consuming `extend`, so the
/// position points at the type keyword (`scalar` at column 8).
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::schema::Definition::TypeExtension(
        legacy_ast::schema::TypeExtension::Scalar(ext),
    ) = &doc.definitions[0] {
        assert_eq!(ext.position.line, 1);
        assert_eq!(ext.position.column, 8);
        assert_eq!(ext.name, "DateTime");
    } else {
        panic!("Expected a Scalar type extension");
    }
}

/// Verifies that type extension position is correctly captured for object
/// extension.
///
/// `graphql_parser` records position after consuming `extend`, so the
/// position points at the type keyword (`type` at column 8).
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::schema::Definition::TypeExtension(
        legacy_ast::schema::TypeExtension::Object(ext),
    ) = &doc.definitions[0] {
        assert_eq!(ext.position.line, 1);
        assert_eq!(ext.position.column, 8);
        assert_eq!(ext.name, "User");
    } else {
        panic!("Expected an Object type extension");
    }
}

/// Verifies that type extension position is correctly captured for interface
/// extension.
///
/// `graphql_parser` records position after consuming `extend`, so the
/// position points at the type keyword (`interface` at column 8).
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::schema::Definition::TypeExtension(
        legacy_ast::schema::TypeExtension::Interface(ext),
    ) = &doc.definitions[0] {
        assert_eq!(ext.position.line, 1);
        assert_eq!(ext.position.column, 8);
        assert_eq!(ext.name, "Node");
    } else {
        panic!("Expected an Interface type extension");
    }
}

/// Verifies that type extension position is correctly captured for union
/// extension.
///
/// `graphql_parser` records position after consuming `extend`, so the
/// position points at the type keyword (`union` at column 8).
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::schema::Definition::TypeExtension(
        legacy_ast::schema::TypeExtension::Union(ext),
    ) = &doc.definitions[0] {
        assert_eq!(ext.position.line, 1);
        assert_eq!(ext.position.column, 8);
        assert_eq!(ext.name, "SearchResult");
    } else {
        panic!("Expected a Union type extension");
    }
}

/// Verifies that type extension position is correctly captured for enum
/// extension.
///
/// `graphql_parser` records position after consuming `extend`, so the
/// position points at the type keyword (`enum` at column 8).
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::schema::Definition::TypeExtension(
        legacy_ast::schema::TypeExtension::Enum(ext),
    ) = &doc.definitions[0] {
        assert_eq!(ext.position.line, 1);
        assert_eq!(ext.position.column, 8);
        assert_eq!(ext.name, "Status");
    } else {
        panic!("Expected an Enum type extension");
    }
}

/// Verifies that type extension position is correctly captured for input
/// object extension.
///
/// `graphql_parser` records position after consuming `extend`, so the
/// position points at the type keyword (`input` at column 8).
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::schema::Definition::TypeExtension(
        legacy_ast::schema::TypeExtension::InputObject(ext),
    ) = &doc.definitions[0] {
        assert_eq!(ext.position.line, 1);
        assert_eq!(ext.position.column, 8);
        assert_eq!(ext.name, "CreateUserInput");
    } else {
        panic!("Expected an InputObject type extension");
    }
}

// =============================================================================
// Edge Cases
// =============================================================================

/// Verifies that shorthand query (just selection set) gets no Query position.
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

    let doc = result.into_valid_ast().unwrap();
    // Shorthand query is represented as SelectionSet directly
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::SelectionSet(ss),
    ) = &doc.definitions[0] {
        // The selection set span should have the braces positions
        assert_eq!(ss.span.0.line, 1);
        assert_eq!(ss.span.0.column, 1);
        assert_eq!(ss.span.1.line, 1);
        assert_eq!(ss.span.1.column, 9);
    } else {
        panic!("Expected a SelectionSet operation definition");
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        // "query" starts on line 3, column 1
        assert_eq!(query.position.line, 3);
        assert_eq!(query.position.column, 1);
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        // "query" starts on line 2, column 1
        assert_eq!(query.position.line, 2);
        assert_eq!(query.position.column, 1);
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        assert_eq!(query.selection_set.items.len(), 2);

        if let legacy_ast::operation::Selection::Field(field1) =
            &query.selection_set.items[0] {
            assert_eq!(field1.position.line, 2);
            assert_eq!(field1.position.column, 3);
        }

        if let legacy_ast::operation::Selection::Field(field2) =
            &query.selection_set.items[1] {
            assert_eq!(field2.position.line, 3);
            assert_eq!(field2.position.column, 3);
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        // First level: "a" at column 9
        if let legacy_ast::operation::Selection::Field(field_a) =
            &query.selection_set.items[0] {
            assert_eq!(field_a.position.line, 1);
            assert_eq!(field_a.position.column, 9);
            assert_eq!(field_a.name, "a");

            // Second level: "b" at column 13
            if let legacy_ast::operation::Selection::Field(field_b) =
                &field_a.selection_set.items[0] {
                assert_eq!(field_b.position.line, 1);
                assert_eq!(field_b.position.column, 13);
                assert_eq!(field_b.name, "b");

                // Third level: "c" at column 17
                if let legacy_ast::operation::Selection::Field(field_c) =
                    &field_b.selection_set.items[0] {
                    assert_eq!(field_c.position.line, 1);
                    assert_eq!(field_c.position.column, 17);
                    assert_eq!(field_c.name, "c");

                    // Fourth level: "d" at column 21
                    if let legacy_ast::operation::Selection::Field(field_d) =
                        &field_c.selection_set.items[0] {
                        assert_eq!(field_d.position.line, 1);
                        assert_eq!(field_d.position.column, 21);
                        assert_eq!(field_d.name, "d");
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0]
        && let legacy_ast::operation::Selection::Field(field) =
            &query.selection_set.items[0] {
        // "field" starts at column 103 (7 for "query {" + 95 spaces + 1)
        assert_eq!(field.position.line, 1);
        assert_eq!(field.position.column, 103);
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

    let doc = result.into_valid_ast().unwrap();
    assert_eq!(doc.definitions.len(), 3);

    // Query A at (1, 1)
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        assert_eq!(query.position.line, 1);
        assert_eq!(query.position.column, 1);
    }

    // Mutation B at (2, 1)
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Mutation(mutation),
    ) = &doc.definitions[1] {
        assert_eq!(mutation.position.line, 2);
        assert_eq!(mutation.position.column, 1);
    }

    // Subscription C at (3, 1)
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Subscription(sub),
    ) = &doc.definitions[2] {
        assert_eq!(sub.position.line, 3);
        assert_eq!(sub.position.column, 1);
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        // "query" starts on line 2, column 1 (unicode doesn't affect line count)
        assert_eq!(query.position.line, 2);
        assert_eq!(query.position.column, 1);
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

    let doc = result.into_valid_ast().unwrap();
    if let legacy_ast::operation::Definition::Operation(
        legacy_ast::operation::OperationDefinition::Query(query),
    ) = &doc.definitions[0] {
        // Check that "other" field position is captured correctly
        // Note: Column position depends on byte vs character counting
        assert_eq!(query.selection_set.items.len(), 2);
        if let legacy_ast::operation::Selection::Field(other_field) =
            &query.selection_set.items[1] {
            assert_eq!(other_field.name, "other");
            // The position should be after the closing ) of the argument
            assert_eq!(other_field.position.line, 1);
            // Column counts characters (not bytes), so the emoji is 1 character
            // "query { field(arg: \"" = 20 chars, then 1 for emoji,
            // then "\") " = 3 chars, so "other" starts at 20 + 1 + 3 + 1 = 25
            assert_eq!(other_field.position.column, 25);
        }
    }
}
