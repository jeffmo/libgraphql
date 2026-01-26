//! Tests for Edge Cases and Additional Edge Cases.
//!
//! These tests verify that the parser correctly handles edge cases in GraphQL
//! documents, including contextual keywords, Unicode, multiple definitions,
//! and complex type structures. Each test verifies the AST structure.
//!
//! Written by Claude Code, reviewed by a human.

use crate::ast;
use crate::tests::ast_utils::extract_first_object_type;
use crate::tests::ast_utils::extract_query;
use crate::tests::ast_utils::field_at;
use crate::tests::ast_utils::first_field;
use crate::tests::ast_utils::first_fragment_spread;
use crate::tests::ast_utils::first_inline_fragment;
use crate::tests::utils::parse_executable;
use crate::tests::utils::parse_schema;

// =============================================================================
// Edge Cases (AST Verification Tests)
// =============================================================================

/// Verifies that GraphQL keywords can be used as field names in selection sets.
///
/// Per GraphQL spec, keywords are contextual and names in selection sets can
/// be any valid name including `type`, `query`, `mutation`, etc.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Names>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn keyword_as_field_name() {
    let query = extract_query("query { type query mutation }");
    assert_eq!(query.selection_set.items.len(), 3);

    let field0 = field_at(&query.selection_set, 0);
    assert_eq!(field0.name, "type");

    let field1 = field_at(&query.selection_set, 1);
    assert_eq!(field1.name, "query");

    let field2 = field_at(&query.selection_set, 2);
    assert_eq!(field2.name, "mutation");
}

/// Verifies that GraphQL keywords can be used as argument names.
///
/// Per GraphQL spec, argument names can be any valid name including keywords.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Names>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn keyword_as_argument_name() {
    let query = extract_query("query { field(type: 1, query: 2) }");
    let field = first_field(&query.selection_set);

    assert_eq!(field.arguments.len(), 2);
    assert_eq!(field.arguments[0].0, "type");
    assert_eq!(field.arguments[1].0, "query");

    // Verify argument values
    if let ast::Value::Int(n) = &field.arguments[0].1 {
        assert_eq!(n.as_i64(), Some(1));
    } else {
        panic!("Expected Int value for 'type' argument");
    }

    if let ast::Value::Int(n) = &field.arguments[1].1 {
        assert_eq!(n.as_i64(), Some(2));
    } else {
        panic!("Expected Int value for 'query' argument");
    }
}

/// Verifies that Unicode characters in string values are correctly parsed.
///
/// Per GraphQL spec, string values can contain Unicode characters including
/// characters outside the ASCII range and emojis.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unicode_in_strings_allowed() {
    let query = extract_query(r#"query { field(arg: "日本語 🎉") }"#);
    let field = first_field(&query.selection_set);

    assert_eq!(field.arguments.len(), 1);
    if let ast::Value::String(s) = &field.arguments[0].1 {
        assert!(s.contains("日本語"));
        assert!(s.contains("🎉"));
    } else {
        panic!("Expected String value, got: {:?}", field.arguments[0].1);
    }
}

/// Verifies that Unicode characters in descriptions are correctly parsed.
///
/// Per GraphQL spec, descriptions can contain Unicode characters.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Descriptions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unicode_in_descriptions() {
    let obj = extract_first_object_type(r#""日本語の説明" type User { name: String }"#);

    assert_eq!(obj.name, "User");
    assert!(obj.description.is_some());
    let desc = obj.description.as_ref().unwrap();
    assert!(desc.contains("日本語"));
}

/// Verifies that block string descriptions are correctly parsed.
///
/// Per GraphQL spec, descriptions can use block strings (triple-quoted).
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Descriptions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn block_string_description() {
    let obj = extract_first_object_type(
        r#""""
        Block string description
        with multiple lines
        """
        type User { name: String }"#,
    );

    assert_eq!(obj.name, "User");
    assert!(obj.description.is_some());
    let desc = obj.description.as_ref().unwrap();
    assert!(desc.contains("Block string description"));
    assert!(desc.contains("multiple lines"));
}

/// Verifies that multiple operations in one document are correctly parsed.
///
/// Per GraphQL spec, an executable document can contain multiple operations.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn consecutive_operations() {
    let doc = parse_executable("query A { field } query B { field } mutation C { field }")
        .into_valid_ast()
        .unwrap();

    assert_eq!(doc.definitions.len(), 3);

    // Verify first operation is query A
    match &doc.definitions[0] {
        ast::operation::Definition::Operation(
            ast::operation::OperationDefinition::Query(q),
        ) => {
            assert_eq!(q.name.as_deref(), Some("A"));
        },
        other => panic!("Expected Query A, got: {other:?}"),
    }

    // Verify second operation is query B
    match &doc.definitions[1] {
        ast::operation::Definition::Operation(
            ast::operation::OperationDefinition::Query(q),
        ) => {
            assert_eq!(q.name.as_deref(), Some("B"));
        },
        other => panic!("Expected Query B, got: {other:?}"),
    }

    // Verify third operation is mutation C
    match &doc.definitions[2] {
        ast::operation::Definition::Operation(
            ast::operation::OperationDefinition::Mutation(m),
        ) => {
            assert_eq!(m.name.as_deref(), Some("C"));
        },
        other => panic!("Expected Mutation C, got: {other:?}"),
    }
}

/// Verifies that multiple fragments in one document are correctly parsed.
///
/// Per GraphQL spec, an executable document can contain multiple fragments.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn consecutive_fragments() {
    let doc = parse_executable("fragment A on User { name } fragment B on User { email }")
        .into_valid_ast()
        .unwrap();

    assert_eq!(doc.definitions.len(), 2);

    // Verify first fragment
    match &doc.definitions[0] {
        ast::operation::Definition::Fragment(f) => {
            assert_eq!(f.name, "A");
            match &f.type_condition {
                ast::operation::TypeCondition::On(name) => {
                    assert_eq!(name, "User");
                },
            }
        },
        other => panic!("Expected Fragment A, got: {other:?}"),
    }

    // Verify second fragment
    match &doc.definitions[1] {
        ast::operation::Definition::Fragment(f) => {
            assert_eq!(f.name, "B");
            match &f.type_condition {
                ast::operation::TypeCondition::On(name) => {
                    assert_eq!(name, "User");
                },
            }
        },
        other => panic!("Expected Fragment B, got: {other:?}"),
    }
}

/// Verifies that fragments can appear before operations in a document.
///
/// Per GraphQL spec, definitions in an executable document can appear in any
/// order.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_before_operation() {
    let doc = parse_executable("fragment F on User { name } query { ...F }")
        .into_valid_ast()
        .unwrap();

    assert_eq!(doc.definitions.len(), 2);

    // Verify first definition is fragment
    match &doc.definitions[0] {
        ast::operation::Definition::Fragment(f) => {
            assert_eq!(f.name, "F");
        },
        other => panic!("Expected Fragment, got: {other:?}"),
    }

    // Verify second definition is query with fragment spread
    match &doc.definitions[1] {
        ast::operation::Definition::Operation(
            ast::operation::OperationDefinition::Query(q),
        ) => {
            let spread = first_fragment_spread(&q.selection_set);
            assert_eq!(spread.fragment_name, "F");
        },
        other => panic!("Expected Query, got: {other:?}"),
    }
}

/// Verifies that duplicate field names are allowed at the parse level.
///
/// Duplicate field selection validation happens at validation phase, not
/// parsing. The parser should accept duplicate field names.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn duplicate_field_names() {
    let query = extract_query("query { name name }");

    assert_eq!(query.selection_set.items.len(), 2);

    let field0 = field_at(&query.selection_set, 0);
    assert_eq!(field0.name, "name");

    let field1 = field_at(&query.selection_set, 1);
    assert_eq!(field1.name, "name");
}

// =============================================================================
// Additional Edge Cases (AST Verification Tests)
// =============================================================================

/// Verifies that very deeply nested list types are correctly parsed.
///
/// This tests the parser's ability to handle multiple levels of list type
/// nesting with a non-null wrapper.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Type-References>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn deeply_nested_list_types() {
    let obj = extract_first_object_type("type Q { f: [[[[[String]]]]]! }");
    let field = &obj.fields[0];

    assert_eq!(field.name, "f");

    // Structure should be: NonNull(List(List(List(List(List(Named("String")))))))
    // Outer: NonNullType
    if let ast::schema::Type::NonNullType(level0) = &field.field_type {
        // Level 1: ListType
        if let ast::schema::Type::ListType(level1) = level0.as_ref() {
            // Level 2: ListType
            if let ast::schema::Type::ListType(level2) = level1.as_ref() {
                // Level 3: ListType
                if let ast::schema::Type::ListType(level3) = level2.as_ref() {
                    // Level 4: ListType
                    if let ast::schema::Type::ListType(level4) = level3.as_ref() {
                        // Level 5: ListType
                        if let ast::schema::Type::ListType(level5) = level4.as_ref() {
                            // Innermost: NamedType
                            if let ast::schema::Type::NamedType(name) = level5.as_ref() {
                                assert_eq!(name, "String");
                            } else {
                                panic!(
                                    "Expected NamedType at innermost level, got: {level5:?}"
                                );
                            }
                        } else {
                            panic!("Expected ListType at level 5, got: {level4:?}");
                        }
                    } else {
                        panic!("Expected ListType at level 4, got: {level3:?}");
                    }
                } else {
                    panic!("Expected ListType at level 3, got: {level2:?}");
                }
            } else {
                panic!("Expected ListType at level 2, got: {level1:?}");
            }
        } else {
            panic!("Expected ListType at level 1, got: {level0:?}");
        }
    } else {
        panic!("Expected NonNullType at outer level, got: {:?}", field.field_type);
    }
}

/// Verifies that complex argument lists with different value types are parsed.
///
/// This tests the parser's ability to handle multiple arguments with various
/// value types: Int, Float, String, Boolean, Null, and Enum.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Language.Arguments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn complex_argument_list() {
    let query = extract_query(
        r#"query { field(a: 1, b: 2.5, c: "str", d: true, e: null, f: ENUM) }"#,
    );
    let field = first_field(&query.selection_set);

    assert_eq!(field.arguments.len(), 6);

    // Verify argument names
    assert_eq!(field.arguments[0].0, "a");
    assert_eq!(field.arguments[1].0, "b");
    assert_eq!(field.arguments[2].0, "c");
    assert_eq!(field.arguments[3].0, "d");
    assert_eq!(field.arguments[4].0, "e");
    assert_eq!(field.arguments[5].0, "f");

    // Verify argument a: Int
    if let ast::Value::Int(n) = &field.arguments[0].1 {
        assert_eq!(n.as_i64(), Some(1));
    } else {
        panic!("Expected Int for arg a, got: {:?}", field.arguments[0].1);
    }

    // Verify argument b: Float
    if let ast::Value::Float(f) = &field.arguments[1].1 {
        assert!((*f - 2.5).abs() < f64::EPSILON);
    } else {
        panic!("Expected Float for arg b, got: {:?}", field.arguments[1].1);
    }

    // Verify argument c: String
    if let ast::Value::String(s) = &field.arguments[2].1 {
        assert_eq!(s, "str");
    } else {
        panic!("Expected String for arg c, got: {:?}", field.arguments[2].1);
    }

    // Verify argument d: Boolean(true)
    if let ast::Value::Boolean(b) = &field.arguments[3].1 {
        assert!(*b);
    } else {
        panic!("Expected Boolean for arg d, got: {:?}", field.arguments[3].1);
    }

    // Verify argument e: Null
    assert!(
        matches!(&field.arguments[4].1, ast::Value::Null),
        "Expected Null for arg e, got: {:?}",
        field.arguments[4].1
    );

    // Verify argument f: Enum
    if let ast::Value::Enum(e) = &field.arguments[5].1 {
        assert_eq!(e, "ENUM");
    } else {
        panic!("Expected Enum for arg f, got: {:?}", field.arguments[5].1);
    }
}

/// Verifies that complex variable definitions with defaults are parsed.
///
/// This tests variable definitions with non-null types, default values, and
/// list types.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Language.Variables>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn complex_variable_definitions() {
    let query = extract_query(
        r#"query($a: Int!, $b: String = "default", $c: [Int!]! = [1, 2]) { f }"#,
    );

    assert_eq!(query.variable_definitions.len(), 3);

    // Verify variable $a: Int!
    let var_a = &query.variable_definitions[0];
    assert_eq!(var_a.name, "a");
    if let ast::operation::Type::NonNullType(inner) = &var_a.var_type {
        if let ast::operation::Type::NamedType(name) = inner.as_ref() {
            assert_eq!(name, "Int");
        } else {
            panic!("Expected NamedType inside NonNull for $a");
        }
    } else {
        panic!("Expected NonNullType for $a, got: {:?}", var_a.var_type);
    }
    assert!(var_a.default_value.is_none());

    // Verify variable $b: String = "default"
    let var_b = &query.variable_definitions[1];
    assert_eq!(var_b.name, "b");
    if let ast::operation::Type::NamedType(name) = &var_b.var_type {
        assert_eq!(name, "String");
    } else {
        panic!("Expected NamedType for $b, got: {:?}", var_b.var_type);
    }
    assert!(var_b.default_value.is_some());
    if let Some(ast::Value::String(s)) = &var_b.default_value {
        assert_eq!(s, "default");
    } else {
        panic!("Expected String default for $b, got: {:?}", var_b.default_value);
    }

    // Verify variable $c: [Int!]! = [1, 2]
    let var_c = &query.variable_definitions[2];
    assert_eq!(var_c.name, "c");
    // Type should be NonNull(List(NonNull(Named("Int"))))
    if let ast::operation::Type::NonNullType(outer) = &var_c.var_type {
        if let ast::operation::Type::ListType(list) = outer.as_ref() {
            if let ast::operation::Type::NonNullType(inner) = list.as_ref() {
                if let ast::operation::Type::NamedType(name) = inner.as_ref() {
                    assert_eq!(name, "Int");
                } else {
                    panic!("Expected NamedType inside inner NonNull for $c");
                }
            } else {
                panic!("Expected NonNullType inside List for $c");
            }
        } else {
            panic!("Expected ListType inside outer NonNull for $c");
        }
    } else {
        panic!("Expected NonNullType for $c, got: {:?}", var_c.var_type);
    }

    // Verify default value [1, 2]
    assert!(var_c.default_value.is_some());
    if let Some(ast::Value::List(items)) = &var_c.default_value {
        assert_eq!(items.len(), 2);
        if let ast::Value::Int(n1) = &items[0] {
            assert_eq!(n1.as_i64(), Some(1));
        }
        if let ast::Value::Int(n2) = &items[1] {
            assert_eq!(n2.as_i64(), Some(2));
        }
    } else {
        panic!("Expected List default for $c, got: {:?}", var_c.default_value);
    }
}

/// Verifies that directives on all schema definition locations parse correctly.
///
/// This tests directives on: schema, scalar, type, interface, union, enum,
/// enum value, input, and input field.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Type-System.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_on_schema_locations() {
    let doc = parse_schema(
        r#"
        schema @a { query: Q }
        scalar S @b
        type T @c { f: Int @d }
        interface I @e { f: Int }
        union U @f = A | B
        enum E @g { V @h }
        input In @i { f: Int @j }
        "#,
    )
    .into_valid_ast()
    .unwrap();

    // We should have 7 definitions
    assert_eq!(doc.definitions.len(), 7);

    // Verify schema @a
    match &doc.definitions[0] {
        ast::schema::Definition::SchemaDefinition(sd) => {
            assert_eq!(sd.directives.len(), 1);
            assert_eq!(sd.directives[0].name, "a");
        },
        other => panic!("Expected SchemaDefinition, got: {other:?}"),
    }

    // Verify scalar S @b
    match &doc.definitions[1] {
        ast::schema::Definition::TypeDefinition(
            ast::schema::TypeDefinition::Scalar(s),
        ) => {
            assert_eq!(s.name, "S");
            assert_eq!(s.directives.len(), 1);
            assert_eq!(s.directives[0].name, "b");
        },
        other => panic!("Expected Scalar, got: {other:?}"),
    }

    // Verify type T @c { f: Int @d }
    match &doc.definitions[2] {
        ast::schema::Definition::TypeDefinition(
            ast::schema::TypeDefinition::Object(obj),
        ) => {
            assert_eq!(obj.name, "T");
            assert_eq!(obj.directives.len(), 1);
            assert_eq!(obj.directives[0].name, "c");
            assert_eq!(obj.fields.len(), 1);
            assert_eq!(obj.fields[0].directives.len(), 1);
            assert_eq!(obj.fields[0].directives[0].name, "d");
        },
        other => panic!("Expected ObjectType, got: {other:?}"),
    }

    // Verify interface I @e
    match &doc.definitions[3] {
        ast::schema::Definition::TypeDefinition(
            ast::schema::TypeDefinition::Interface(iface),
        ) => {
            assert_eq!(iface.name, "I");
            assert_eq!(iface.directives.len(), 1);
            assert_eq!(iface.directives[0].name, "e");
        },
        other => panic!("Expected Interface, got: {other:?}"),
    }

    // Verify union U @f
    match &doc.definitions[4] {
        ast::schema::Definition::TypeDefinition(
            ast::schema::TypeDefinition::Union(u),
        ) => {
            assert_eq!(u.name, "U");
            assert_eq!(u.directives.len(), 1);
            assert_eq!(u.directives[0].name, "f");
        },
        other => panic!("Expected Union, got: {other:?}"),
    }

    // Verify enum E @g { V @h }
    match &doc.definitions[5] {
        ast::schema::Definition::TypeDefinition(
            ast::schema::TypeDefinition::Enum(e),
        ) => {
            assert_eq!(e.name, "E");
            assert_eq!(e.directives.len(), 1);
            assert_eq!(e.directives[0].name, "g");
            assert_eq!(e.values.len(), 1);
            assert_eq!(e.values[0].directives.len(), 1);
            assert_eq!(e.values[0].directives[0].name, "h");
        },
        other => panic!("Expected Enum, got: {other:?}"),
    }

    // Verify input In @i { f: Int @j }
    match &doc.definitions[6] {
        ast::schema::Definition::TypeDefinition(
            ast::schema::TypeDefinition::InputObject(io),
        ) => {
            assert_eq!(io.name, "In");
            assert_eq!(io.directives.len(), 1);
            assert_eq!(io.directives[0].name, "i");
            assert_eq!(io.fields.len(), 1);
            assert_eq!(io.fields[0].directives.len(), 1);
            assert_eq!(io.fields[0].directives[0].name, "j");
        },
        other => panic!("Expected InputObject, got: {other:?}"),
    }
}

/// Verifies that directives on all executable locations parse correctly.
///
/// This tests directives on: query operation, field, inline fragment (untyped),
/// fragment spread, and fragment definition.
///
/// Spec reference:
/// <https://spec.graphql.org/September2025/#sec-Language.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_on_executable_locations() {
    let doc = parse_executable(
        r#"
        query Q @a {
            field @b
            ... @c { nested }
            ...Frag @d
        }
        fragment Frag on T @e { f }
        "#,
    )
    .into_valid_ast()
    .unwrap();

    assert_eq!(doc.definitions.len(), 2);

    // Verify query Q @a
    match &doc.definitions[0] {
        ast::operation::Definition::Operation(
            ast::operation::OperationDefinition::Query(q),
        ) => {
            assert_eq!(q.name.as_deref(), Some("Q"));
            assert_eq!(q.directives.len(), 1);
            assert_eq!(q.directives[0].name, "a");

            // 3 selections: field @b, ... @c { nested }, ...Frag @d
            assert_eq!(q.selection_set.items.len(), 3);

            // Verify field @b
            let field = field_at(&q.selection_set, 0);
            assert_eq!(field.name, "field");
            assert_eq!(field.directives.len(), 1);
            assert_eq!(field.directives[0].name, "b");

            // Verify ... @c { nested } (untyped inline fragment)
            let inline = first_inline_fragment(&q.selection_set);
            assert!(inline.type_condition.is_none());
            assert_eq!(inline.directives.len(), 1);
            assert_eq!(inline.directives[0].name, "c");

            // Verify ...Frag @d
            let spread = first_fragment_spread(&q.selection_set);
            assert_eq!(spread.fragment_name, "Frag");
            assert_eq!(spread.directives.len(), 1);
            assert_eq!(spread.directives[0].name, "d");
        },
        other => panic!("Expected Query, got: {other:?}"),
    }

    // Verify fragment Frag on T @e
    match &doc.definitions[1] {
        ast::operation::Definition::Fragment(f) => {
            assert_eq!(f.name, "Frag");
            assert_eq!(f.directives.len(), 1);
            assert_eq!(f.directives[0].name, "e");
        },
        other => panic!("Expected Fragment, got: {other:?}"),
    }
}
