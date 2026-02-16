//! Tests for Type Annotations.
//!
//! These tests verify that the parser correctly handles GraphQL type
//! annotations including named types, non-null types, list types, and
//! various combinations thereof.
//!
//! Written by Claude Code, reviewed by a human.

use crate::legacy_ast;
use crate::tests::ast_utils::extract_first_object_type;
use crate::tests::utils::parse_schema;

// =============================================================================
// Type Annotations
// =============================================================================

/// Verifies that named types like `String`, `User`, `Int` are parsed correctly
/// and produce the expected NamedType AST node.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_named() {
    // Test String type
    let obj = extract_first_object_type("type Query { field: String }");
    let field = &obj.fields[0];
    if let legacy_ast::schema::Type::NamedType(name) = &field.field_type {
        assert_eq!(name, "String");
    } else {
        panic!("Expected NamedType, got: {:?}", field.field_type);
    }

    // Test User type
    let obj = extract_first_object_type("type Query { field: User }");
    let field = &obj.fields[0];
    if let legacy_ast::schema::Type::NamedType(name) = &field.field_type {
        assert_eq!(name, "User");
    } else {
        panic!("Expected NamedType, got: {:?}", field.field_type);
    }

    // Test Int type
    let obj = extract_first_object_type("type Query { field: Int }");
    let field = &obj.fields[0];
    if let legacy_ast::schema::Type::NamedType(name) = &field.field_type {
        assert_eq!(name, "Int");
    } else {
        panic!("Expected NamedType, got: {:?}", field.field_type);
    }
}

/// Verifies that non-null types `String!` are parsed correctly and produce
/// a NonNullType wrapping a NamedType.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_non_null() {
    let obj = extract_first_object_type("type Query { field: String! }");
    let field = &obj.fields[0];

    if let legacy_ast::schema::Type::NonNullType(inner) = &field.field_type {
        if let legacy_ast::schema::Type::NamedType(name) = inner.as_ref() {
            assert_eq!(name, "String");
        } else {
            panic!("Expected NamedType inside NonNull, got: {inner:?}");
        }
    } else {
        panic!("Expected NonNullType, got: {:?}", field.field_type);
    }
}

/// Verifies that list types `[String]` are parsed correctly and produce
/// a ListType wrapping a NamedType.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_list() {
    let obj = extract_first_object_type("type Query { field: [String] }");
    let field = &obj.fields[0];

    if let legacy_ast::schema::Type::ListType(inner) = &field.field_type {
        if let legacy_ast::schema::Type::NamedType(name) = inner.as_ref() {
            assert_eq!(name, "String");
        } else {
            panic!("Expected NamedType inside List, got: {inner:?}");
        }
    } else {
        panic!("Expected ListType, got: {:?}", field.field_type);
    }
}

/// Verifies that `[String]!` (non-null list of nullable elements) is parsed
/// correctly as a NonNullType wrapping a ListType wrapping a NamedType.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_list_non_null() {
    let obj = extract_first_object_type("type Query { field: [String]! }");
    let field = &obj.fields[0];

    if let legacy_ast::schema::Type::NonNullType(non_null_inner) = &field.field_type {
        if let legacy_ast::schema::Type::ListType(list_inner) = non_null_inner.as_ref() {
            if let legacy_ast::schema::Type::NamedType(name) = list_inner.as_ref() {
                assert_eq!(name, "String");
            } else {
                panic!(
                    "Expected NamedType inside List inside NonNull, got: {list_inner:?}"
                );
            }
        } else {
            panic!(
                "Expected ListType inside NonNull, got: {non_null_inner:?}"
            );
        }
    } else {
        panic!("Expected NonNullType, got: {:?}", field.field_type);
    }
}

/// Verifies that `[String!]` (nullable list of non-null elements) is parsed
/// correctly as a ListType wrapping a NonNullType wrapping a NamedType.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_non_null_list() {
    let obj = extract_first_object_type("type Query { field: [String!] }");
    let field = &obj.fields[0];

    if let legacy_ast::schema::Type::ListType(list_inner) = &field.field_type {
        if let legacy_ast::schema::Type::NonNullType(non_null_inner) = list_inner.as_ref() {
            if let legacy_ast::schema::Type::NamedType(name) = non_null_inner.as_ref() {
                assert_eq!(name, "String");
            } else {
                panic!(
                    "Expected NamedType inside NonNull inside List, got: {non_null_inner:?}"
                );
            }
        } else {
            panic!("Expected NonNullType inside List, got: {list_inner:?}");
        }
    } else {
        panic!("Expected ListType, got: {:?}", field.field_type);
    }
}

/// Verifies that `[String!]!` (non-null list of non-null elements) is parsed
/// correctly as NonNullType(ListType(NonNullType(NamedType))).
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_non_null_list_non_null() {
    let obj = extract_first_object_type("type Query { field: [String!]! }");
    let field = &obj.fields[0];

    // Outer: NonNullType
    if let legacy_ast::schema::Type::NonNullType(outer_non_null) = &field.field_type {
        // Next: ListType
        if let legacy_ast::schema::Type::ListType(list_inner) = outer_non_null.as_ref() {
            // Next: NonNullType
            if let legacy_ast::schema::Type::NonNullType(inner_non_null) = list_inner.as_ref() {
                // Innermost: NamedType
                if let legacy_ast::schema::Type::NamedType(name) = inner_non_null.as_ref() {
                    assert_eq!(name, "String");
                } else {
                    panic!(
                        "Expected NamedType at innermost level, got: {inner_non_null:?}"
                    );
                }
            } else {
                panic!("Expected NonNullType inside List, got: {list_inner:?}");
            }
        } else {
            panic!(
                "Expected ListType inside outer NonNull, got: {outer_non_null:?}"
            );
        }
    } else {
        panic!(
            "Expected NonNullType at outer level, got: {:?}",
            field.field_type
        );
    }
}

/// Verifies that deeply nested list types like `[[String]]` and `[[[Int]]]`
/// are parsed correctly with the appropriate nesting of ListType nodes.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_deeply_nested() {
    // Test [[String]] - two levels of list nesting
    let obj = extract_first_object_type("type Query { field: [[String]] }");
    let field = &obj.fields[0];

    if let legacy_ast::schema::Type::ListType(outer_list) = &field.field_type {
        if let legacy_ast::schema::Type::ListType(inner_list) = outer_list.as_ref() {
            if let legacy_ast::schema::Type::NamedType(name) = inner_list.as_ref() {
                assert_eq!(name, "String");
            } else {
                panic!(
                    "Expected NamedType at innermost level, got: {inner_list:?}"
                );
            }
        } else {
            panic!("Expected ListType at second level, got: {outer_list:?}");
        }
    } else {
        panic!(
            "Expected ListType at outer level, got: {:?}",
            field.field_type
        );
    }

    // Test [[[Int]]] - three levels of list nesting
    let obj = extract_first_object_type("type Query { field: [[[Int]]] }");
    let field = &obj.fields[0];

    if let legacy_ast::schema::Type::ListType(level1) = &field.field_type {
        if let legacy_ast::schema::Type::ListType(level2) = level1.as_ref() {
            if let legacy_ast::schema::Type::ListType(level3) = level2.as_ref() {
                if let legacy_ast::schema::Type::NamedType(name) = level3.as_ref() {
                    assert_eq!(name, "Int");
                } else {
                    panic!("Expected NamedType at innermost level, got: {level3:?}");
                }
            } else {
                panic!("Expected ListType at third level, got: {level2:?}");
            }
        } else {
            panic!("Expected ListType at second level, got: {level1:?}");
        }
    } else {
        panic!(
            "Expected ListType at outer level, got: {:?}",
            field.field_type
        );
    }
}

/// Verifies that an unclosed bracket in a list type produces a parse error.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_unclosed_bracket_error() {
    let result = parse_schema("type Query { field: [String }");
    assert!(
        result.has_errors(),
        "Expected parse error for unclosed bracket in list type"
    );
}

/// Verifies that double bang `String!!` produces a parse error since non-null
/// types cannot be nested directly.
///
/// Per GraphQL spec, the `!` suffix applies to the preceding type, and
/// NonNullType cannot wrap another NonNullType:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_double_bang_error() {
    let result = parse_schema("type Query { field: String!! }");
    assert!(
        result.has_errors(),
        "Expected parse error for double bang (String!!)"
    );
}
