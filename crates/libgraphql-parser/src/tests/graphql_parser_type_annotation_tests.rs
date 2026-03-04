//! Tests for Type Annotations.
//!
//! These tests verify that the parser correctly handles GraphQL type
//! annotations including named types, non-null types, list types, and
//! various combinations thereof.
//!
//! Written by Claude Code, reviewed by a human.

use crate::ast;
use crate::tests::ast_utils::extract_first_object_type;
use crate::tests::utils::parse_schema;

// =============================================================================
// Type Annotations
// =============================================================================

/// Verifies that named types like `String`, `User`, `Int` are parsed correctly
/// and produce a nullable named TypeAnnotation.
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
    if let ast::TypeAnnotation::Named(named) = &field.field_type {
        assert_eq!(named.name.value, "String");
        assert!(named.nullable());
    } else {
        panic!("Expected Named TypeAnnotation, got: {:?}", field.field_type);
    }

    // Test User type
    let obj = extract_first_object_type("type Query { field: User }");
    let field = &obj.fields[0];
    if let ast::TypeAnnotation::Named(named) = &field.field_type {
        assert_eq!(named.name.value, "User");
        assert!(named.nullable());
    } else {
        panic!("Expected Named TypeAnnotation, got: {:?}", field.field_type);
    }

    // Test Int type
    let obj = extract_first_object_type("type Query { field: Int }");
    let field = &obj.fields[0];
    if let ast::TypeAnnotation::Named(named) = &field.field_type {
        assert_eq!(named.name.value, "Int");
        assert!(named.nullable());
    } else {
        panic!("Expected Named TypeAnnotation, got: {:?}", field.field_type);
    }
}

/// Verifies that `String!` is parsed as a non-nullable named TypeAnnotation.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_non_null() {
    let obj = extract_first_object_type("type Query { field: String! }");
    let field = &obj.fields[0];

    if let ast::TypeAnnotation::Named(named) = &field.field_type {
        assert_eq!(named.name.value, "String");
        assert!(!named.nullable());
    } else {
        panic!("Expected Named TypeAnnotation, got: {:?}", field.field_type);
    }
}

/// Verifies that `[String]` is parsed as a nullable List TypeAnnotation
/// wrapping a nullable named TypeAnnotation.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_list() {
    let obj = extract_first_object_type("type Query { field: [String] }");
    let field = &obj.fields[0];

    if let ast::TypeAnnotation::List(list) = &field.field_type {
        assert!(list.nullable());
        if let ast::TypeAnnotation::Named(named) = list.element_type.as_ref() {
            assert_eq!(named.name.value, "String");
            assert!(named.nullable());
        } else {
            panic!("Expected Named element type, got: {:?}", list.element_type);
        }
    } else {
        panic!("Expected List TypeAnnotation, got: {:?}", field.field_type);
    }
}

/// Verifies that `[String]!` is parsed as a non-nullable List TypeAnnotation
/// wrapping a nullable named TypeAnnotation.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_list_non_null() {
    let obj = extract_first_object_type("type Query { field: [String]! }");
    let field = &obj.fields[0];

    if let ast::TypeAnnotation::List(list) = &field.field_type {
        assert!(!list.nullable());
        if let ast::TypeAnnotation::Named(named) = list.element_type.as_ref() {
            assert_eq!(named.name.value, "String");
            assert!(named.nullable());
        } else {
            panic!("Expected Named element type, got: {:?}", list.element_type);
        }
    } else {
        panic!("Expected List TypeAnnotation, got: {:?}", field.field_type);
    }
}

/// Verifies that `[String!]` is parsed as a nullable List TypeAnnotation
/// wrapping a non-nullable named TypeAnnotation.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_non_null_list() {
    let obj = extract_first_object_type("type Query { field: [String!] }");
    let field = &obj.fields[0];

    if let ast::TypeAnnotation::List(list) = &field.field_type {
        assert!(list.nullable());
        if let ast::TypeAnnotation::Named(named) = list.element_type.as_ref() {
            assert_eq!(named.name.value, "String");
            assert!(!named.nullable());
        } else {
            panic!("Expected Named element type, got: {:?}", list.element_type);
        }
    } else {
        panic!("Expected List TypeAnnotation, got: {:?}", field.field_type);
    }
}

/// Verifies that `[String!]!` is parsed as a non-nullable List TypeAnnotation
/// wrapping a non-nullable named TypeAnnotation.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_non_null_list_non_null() {
    let obj = extract_first_object_type("type Query { field: [String!]! }");
    let field = &obj.fields[0];

    if let ast::TypeAnnotation::List(list) = &field.field_type {
        assert!(!list.nullable());
        if let ast::TypeAnnotation::Named(named) = list.element_type.as_ref() {
            assert_eq!(named.name.value, "String");
            assert!(!named.nullable());
        } else {
            panic!("Expected Named element type, got: {:?}", list.element_type);
        }
    } else {
        panic!("Expected List TypeAnnotation, got: {:?}", field.field_type);
    }
}

/// Verifies that deeply nested list types like `[[String]]` and `[[[Int]]]`
/// are parsed correctly with the appropriate nesting of List TypeAnnotation
/// nodes.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_deeply_nested() {
    // Test [[String]] - two levels of list nesting, all nullable
    let obj = extract_first_object_type("type Query { field: [[String]] }");
    let field = &obj.fields[0];

    if let ast::TypeAnnotation::List(outer_list) = &field.field_type {
        assert!(outer_list.nullable());
        if let ast::TypeAnnotation::List(inner_list) = outer_list.element_type.as_ref() {
            assert!(inner_list.nullable());
            if let ast::TypeAnnotation::Named(named) = inner_list.element_type.as_ref() {
                assert_eq!(named.name.value, "String");
                assert!(named.nullable());
            } else {
                panic!(
                    "Expected Named at innermost level, got: {:?}",
                    inner_list.element_type,
                );
            }
        } else {
            panic!(
                "Expected List at second level, got: {:?}",
                outer_list.element_type,
            );
        }
    } else {
        panic!(
            "Expected List at outer level, got: {:?}",
            field.field_type,
        );
    }

    // Test [[[Int]]] - three levels of list nesting, all nullable
    let obj = extract_first_object_type("type Query { field: [[[Int]]] }");
    let field = &obj.fields[0];

    if let ast::TypeAnnotation::List(level1) = &field.field_type {
        assert!(level1.nullable());
        if let ast::TypeAnnotation::List(level2) = level1.element_type.as_ref() {
            assert!(level2.nullable());
            if let ast::TypeAnnotation::List(level3) = level2.element_type.as_ref() {
                assert!(level3.nullable());
                if let ast::TypeAnnotation::Named(named) = level3.element_type.as_ref() {
                    assert_eq!(named.name.value, "Int");
                    assert!(named.nullable());
                } else {
                    panic!(
                        "Expected Named at innermost level, got: {:?}",
                        level3.element_type,
                    );
                }
            } else {
                panic!(
                    "Expected List at third level, got: {:?}",
                    level2.element_type,
                );
            }
        } else {
            panic!(
                "Expected List at second level, got: {:?}",
                level1.element_type,
            );
        }
    } else {
        panic!(
            "Expected List at outer level, got: {:?}",
            field.field_type,
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
