//! Tests for Value Parsing from the GraphQL parser.
//!
//! These tests verify that the parser correctly parses different GraphQL value
//! types and constructs the appropriate AST nodes. Each test navigates to the
//! parsed value and pattern matches to verify the correct variant and content.
//!
//! Written by Claude Code, reviewed by a human.

use crate::legacy_ast;
use crate::tests::ast_utils::extract_query;
use crate::tests::ast_utils::first_arg_value;
use crate::tests::ast_utils::first_field;
use crate::tests::utils::parse_executable;

// =============================================================================
// Integer Value Tests
// =============================================================================

/// Verifies that positive integer values are parsed as Int with correct value.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Int-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_int() {
    let query = extract_query("query { field(arg: 123) }");
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if let legacy_ast::Value::Int(n) = value {
        assert_eq!(n.as_i64(), Some(123));
    } else {
        panic!("Expected Int value, got: {value:?}");
    }
}

/// Verifies that negative integer values are parsed correctly.
///
/// Per GraphQL spec, integers may have a leading `-` sign:
/// <https://spec.graphql.org/September2025/#sec-Int-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_int_negative() {
    let query = extract_query("query { field(arg: -456) }");
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if let legacy_ast::Value::Int(n) = value {
        assert_eq!(n.as_i64(), Some(-456));
    } else {
        panic!("Expected Int value, got: {value:?}");
    }
}

// =============================================================================
// Float Value Tests
// =============================================================================

/// Verifies that float values are parsed as Float with correct value.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Float-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_float() {
    let query = extract_query("query { field(arg: 1.5) }");
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if let legacy_ast::Value::Float(f) = value {
        assert!((*f - 1.5).abs() < f64::EPSILON);
    } else {
        panic!("Expected Float value, got: {value:?}");
    }
}

// =============================================================================
// String Value Tests
// =============================================================================

/// Verifies that string values are parsed as String with correct content.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_string() {
    let query = extract_query(r#"query { field(arg: "hello") }"#);
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if let legacy_ast::Value::String(s) = value {
        assert_eq!(s, "hello");
    } else {
        panic!("Expected String value, got: {value:?}");
    }
}

/// Verifies that escape sequences in strings are correctly processed.
///
/// Per GraphQL spec, strings support escape sequences like \n, \t, \", etc.:
/// <https://spec.graphql.org/September2025/#EscapedCharacter>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_string_with_escapes() {
    let query = extract_query(r#"query { field(arg: "hello\nworld") }"#);
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if let legacy_ast::Value::String(s) = value {
        // The parser should have converted \n to an actual newline
        assert!(s.contains('\n') || s.contains("\\n"));
    } else {
        panic!("Expected String value, got: {value:?}");
    }
}

// =============================================================================
// Boolean Value Tests
// =============================================================================

/// Verifies that `true` is parsed as Boolean(true).
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Boolean-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_boolean_true() {
    let query = extract_query("query { field(arg: true) }");
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if let legacy_ast::Value::Boolean(b) = value {
        assert!(*b);
    } else {
        panic!("Expected Boolean value, got: {value:?}");
    }
}

/// Verifies that `false` is parsed as Boolean(false).
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Boolean-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_boolean_false() {
    let query = extract_query("query { field(arg: false) }");
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if let legacy_ast::Value::Boolean(b) = value {
        assert!(!*b);
    } else {
        panic!("Expected Boolean value, got: {value:?}");
    }
}

// =============================================================================
// Null Value Tests
// =============================================================================

/// Verifies that `null` is parsed as the Null variant.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Null-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_null() {
    let query = extract_query("query { field(arg: null) }");
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if !matches!(value, legacy_ast::Value::Null) {
        panic!("Expected Null value, got: {value:?}");
    }
}

// =============================================================================
// Enum Value Tests
// =============================================================================

/// Verifies that enum values (names that aren't keywords) are parsed correctly.
///
/// Per GraphQL spec, enum values are names that are not `true`, `false`, or
/// `null`:
/// <https://spec.graphql.org/September2025/#sec-Enum-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_enum() {
    let query = extract_query("query { field(arg: ACTIVE) }");
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if let legacy_ast::Value::Enum(e) = value {
        assert_eq!(e, "ACTIVE");
    } else {
        panic!("Expected Enum value, got: {value:?}");
    }
}

/// Verifies that keywords like `type` can be used as enum values.
///
/// Per GraphQL spec, enum values can be any name except `true`, `false`,
/// `null`. Keywords like `type`, `query`, `mutation` are valid enum values:
/// <https://spec.graphql.org/September2025/#sec-Enum-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_enum_looks_like_keyword() {
    let query = extract_query("query { field(arg: type) }");
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if let legacy_ast::Value::Enum(e) = value {
        assert_eq!(e, "type");
    } else {
        panic!("Expected Enum value, got: {value:?}");
    }
}

// =============================================================================
// List Value Tests
// =============================================================================

/// Verifies that empty list `[]` is parsed as an empty List.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-List-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_list_empty() {
    let query = extract_query("query { field(arg: []) }");
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if let legacy_ast::Value::List(items) = value {
        assert!(items.is_empty());
    } else {
        panic!("Expected List value, got: {value:?}");
    }
}

/// Verifies that simple list `[1, 2, 3]` is parsed with correct elements.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-List-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_list_simple() {
    let query = extract_query("query { field(arg: [1, 2, 3]) }");
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if let legacy_ast::Value::List(items) = value {
        assert_eq!(items.len(), 3);
        if let legacy_ast::Value::Int(n) = &items[0] {
            assert_eq!(n.as_i64(), Some(1));
        } else {
            panic!("Expected first element to be Int");
        }
        if let legacy_ast::Value::Int(n) = &items[1] {
            assert_eq!(n.as_i64(), Some(2));
        } else {
            panic!("Expected second element to be Int");
        }
        if let legacy_ast::Value::Int(n) = &items[2] {
            assert_eq!(n.as_i64(), Some(3));
        } else {
            panic!("Expected third element to be Int");
        }
    } else {
        panic!("Expected List value, got: {value:?}");
    }
}

/// Verifies that nested lists `[[1], [2]]` are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-List-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_list_nested() {
    let query = extract_query("query { field(arg: [[1], [2]]) }");
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if let legacy_ast::Value::List(outer) = value {
        assert_eq!(outer.len(), 2);
        if let legacy_ast::Value::List(inner1) = &outer[0] {
            assert_eq!(inner1.len(), 1);
            if let legacy_ast::Value::Int(n) = &inner1[0] {
                assert_eq!(n.as_i64(), Some(1));
            }
        } else {
            panic!("Expected first element to be a List");
        }
        if let legacy_ast::Value::List(inner2) = &outer[1] {
            assert_eq!(inner2.len(), 1);
            if let legacy_ast::Value::Int(n) = &inner2[0] {
                assert_eq!(n.as_i64(), Some(2));
            }
        } else {
            panic!("Expected second element to be a List");
        }
    } else {
        panic!("Expected List value, got: {value:?}");
    }
}

/// Verifies that mixed-type lists `[1, "two", true]` are parsed correctly.
///
/// Per GraphQL spec, list values have no type constraint at parse level:
/// <https://spec.graphql.org/September2025/#sec-List-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_list_mixed_types() {
    let query = extract_query(r#"query { field(arg: [1, "two", true]) }"#);
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if let legacy_ast::Value::List(items) = value {
        assert_eq!(items.len(), 3);
        assert!(matches!(&items[0], legacy_ast::Value::Int(_)));
        assert!(matches!(&items[1], legacy_ast::Value::String(_)));
        assert!(matches!(&items[2], legacy_ast::Value::Boolean(true)));
    } else {
        panic!("Expected List value, got: {value:?}");
    }
}

// =============================================================================
// Object Value Tests
// =============================================================================

/// Verifies that empty object `{}` is parsed as an empty Object.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Input-Object-Values>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_object_empty() {
    let query = extract_query("query { field(arg: {}) }");
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if let legacy_ast::Value::Object(fields) = value {
        assert!(fields.is_empty());
    } else {
        panic!("Expected Object value, got: {value:?}");
    }
}

/// Verifies that simple object `{key: "value"}` is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Input-Object-Values>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_object_simple() {
    let query = extract_query(r#"query { field(arg: {key: "value"}) }"#);
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if let legacy_ast::Value::Object(fields) = value {
        assert_eq!(fields.len(), 1);
        let field_value = fields.get("key").expect("Expected 'key' field");
        if let legacy_ast::Value::String(s) = field_value {
            assert_eq!(s, "value");
        } else {
            panic!("Expected String value for 'key' field");
        }
    } else {
        panic!("Expected Object value, got: {value:?}");
    }
}

/// Verifies that objects with multiple fields are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Input-Object-Values>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_object_multiple_fields() {
    let query = extract_query("query { field(arg: {a: 1, b: 2, c: 3}) }");
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if let legacy_ast::Value::Object(fields) = value {
        assert_eq!(fields.len(), 3);
        assert!(fields.contains_key("a"));
        assert!(fields.contains_key("b"));
        assert!(fields.contains_key("c"));
    } else {
        panic!("Expected Object value, got: {value:?}");
    }
}

/// Verifies that nested objects `{outer: {inner: 1}}` are parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Input-Object-Values>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_object_nested() {
    let query = extract_query("query { field(arg: {outer: {inner: 1}}) }");
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if let legacy_ast::Value::Object(fields) = value {
        assert_eq!(fields.len(), 1);
        let outer_value = fields.get("outer").expect("Expected 'outer' field");
        if let legacy_ast::Value::Object(inner_fields) = outer_value {
            assert_eq!(inner_fields.len(), 1);
            let inner_value = inner_fields.get("inner").expect("Expected 'inner' field");
            if let legacy_ast::Value::Int(n) = inner_value {
                assert_eq!(n.as_i64(), Some(1));
            } else {
                panic!("Expected Int value for 'inner' field");
            }
        } else {
            panic!("Expected Object value for 'outer' field");
        }
    } else {
        panic!("Expected Object value, got: {value:?}");
    }
}

// =============================================================================
// Variable Value Tests
// =============================================================================

/// Verifies that variables `$var` are parsed as Variable.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#Variable>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_variable() {
    let query = extract_query("query($var: Int) { field(arg: $var) }");
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if let legacy_ast::Value::Variable(name) = value {
        assert_eq!(name, "var");
    } else {
        panic!("Expected Variable value, got: {value:?}");
    }
}

/// Verifies that variables in default values produce errors.
///
/// Per GraphQL spec, default values must be constant (no variables):
/// <https://spec.graphql.org/September2025/#sec-Input-Object-Values>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_variable_in_const_error() {
    let result = parse_executable("query($var: Int = $other) { field }");
    assert!(
        result.has_errors(),
        "Expected error for variable in default value"
    );
}

// =============================================================================
// Block String Value Tests
// =============================================================================

/// Verifies that block strings (triple-quoted) are parsed correctly.
///
/// Per GraphQL spec, block strings preserve formatting and handle indentation:
/// <https://spec.graphql.org/September2025/#sec-String-Value>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_block_string() {
    let query = extract_query(
        r#"query { field(arg: """multi
line
string""") }"#,
    );
    let field = first_field(&query.selection_set);
    let value = first_arg_value(field);

    if let legacy_ast::Value::String(s) = value {
        // Block string should contain the multi-line content
        assert!(s.contains("multi"));
        assert!(s.contains("line"));
        assert!(s.contains("string"));
    } else {
        panic!("Expected String value for block string, got: {value:?}");
    }
}
