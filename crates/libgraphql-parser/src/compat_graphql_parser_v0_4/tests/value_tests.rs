use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::ast;
use crate::ast::tests::ast_test_utils::make_name;
use crate::ast::tests::ast_test_utils::zero_span;
use crate::compat_graphql_parser_v0_4::value_to_gp;

use graphql_parser::query::Value as GpValue;

/// Verifies that `Value::Int` converts correctly to
/// `graphql_parser::query::Value::Int`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_value_to_graphql_parser_int() {
    let lg_val = ast::Value::Int(ast::IntValue {
        span: zero_span(),
        syntax: None,
        value: 42,
    });
    let gp_val = value_to_gp(&lg_val);
    assert_eq!(
        gp_val,
        GpValue::Int(42i32.into()),
    );
}

/// Verifies that `Value::Float` converts correctly,
/// preserving the `f64` value.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_value_to_graphql_parser_float() {
    let lg_val = ast::Value::Float(ast::FloatValue {
        span: zero_span(),
        syntax: None,
        value: 3.14,
    });
    let gp_val = value_to_gp(&lg_val);
    assert_eq!(gp_val, GpValue::Float(3.14));
}

/// Verifies that `Value::String` converts correctly,
/// producing an owned `String` from a `Cow<str>`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_value_to_graphql_parser_string() {
    let lg_val = ast::Value::String(ast::StringValue {
        is_block: false,
        span: zero_span(),
        syntax: None,
        value: Cow::Borrowed("hello"),
    });
    let gp_val = value_to_gp(&lg_val);
    assert_eq!(
        gp_val,
        GpValue::String("hello".to_string()),
    );
}

/// Verifies that `Value::Boolean` converts correctly
/// for both `true` and `false`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_value_to_graphql_parser_boolean() {
    let lg_true = ast::Value::Boolean(ast::BooleanValue {
        span: zero_span(),
        syntax: None,
        value: true,
    });
    assert_eq!(
        value_to_gp(&lg_true),
        GpValue::Boolean(true),
    );

    let lg_false =
        ast::Value::Boolean(ast::BooleanValue {
            span: zero_span(),
            syntax: None,
            value: false,
        });
    assert_eq!(
        value_to_gp(&lg_false),
        GpValue::Boolean(false),
    );
}

/// Verifies that `Value::Null` converts correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_value_to_graphql_parser_null() {
    let lg_val = ast::Value::Null(ast::NullValue {
        span: zero_span(),
        syntax: None,
    });
    assert_eq!(value_to_gp(&lg_val), GpValue::Null);
}

/// Verifies that `Value::Enum` converts correctly,
/// producing an owned `String` from a `Cow<str>`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_value_to_graphql_parser_enum() {
    let lg_val = ast::Value::Enum(ast::EnumValue {
        span: zero_span(),
        syntax: None,
        value: Cow::Borrowed("ACTIVE"),
    });
    assert_eq!(
        value_to_gp(&lg_val),
        GpValue::Enum("ACTIVE".to_string()),
    );
}

/// Verifies that `Value::Variable` converts correctly,
/// extracting the name as an owned `String`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_value_to_graphql_parser_variable() {
    let lg_val =
        ast::Value::Variable(ast::VariableValue {
            name: make_name("userId", 0, 6),
            span: zero_span(),
            syntax: None,
        });
    assert_eq!(
        value_to_gp(&lg_val),
        GpValue::Variable("userId".to_string()),
    );
}

/// Verifies that `Value::List` converts correctly,
/// including nested values.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_value_to_graphql_parser_list() {
    let lg_val = ast::Value::List(ast::ListValue {
        span: zero_span(),
        syntax: None,
        values: vec![
            ast::Value::Int(ast::IntValue {
                span: zero_span(),
                syntax: None,
                value: 1,
            }),
            ast::Value::Null(ast::NullValue {
                span: zero_span(),
                syntax: None,
            }),
            ast::Value::String(ast::StringValue {
                is_block: false,
                span: zero_span(),
                syntax: None,
                value: Cow::Borrowed("two"),
            }),
        ],
    });
    assert_eq!(
        value_to_gp(&lg_val),
        GpValue::List(vec![
            GpValue::Int(1i32.into()),
            GpValue::Null,
            GpValue::String("two".to_string()),
        ]),
    );
}

/// Verifies that `Value::Object` converts correctly,
/// and that fields are reordered alphabetically by key
/// (since `graphql_parser` uses `BTreeMap`).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_value_to_graphql_parser_object() {
    // Fields intentionally in non-alphabetical order
    let lg_val = ast::Value::Object(ast::ObjectValue {
        span: zero_span(),
        syntax: None,
        fields: vec![
            ast::ObjectField {
                name: make_name("z_last", 0, 6),
                span: zero_span(),
                syntax: None,
                value: ast::Value::Int(ast::IntValue {
                    span: zero_span(),
                    syntax: None,
                    value: 1,
                }),
            },
            ast::ObjectField {
                name: make_name("a_first", 0, 7),
                span: zero_span(),
                syntax: None,
                value: ast::Value::Int(ast::IntValue {
                    span: zero_span(),
                    syntax: None,
                    value: 2,
                }),
            },
        ],
    });
    let gp_val = value_to_gp(&lg_val);

    // BTreeMap orders keys alphabetically
    let mut expected = BTreeMap::new();
    expected.insert(
        "a_first".to_string(),
        GpValue::Int(2i32.into()),
    );
    expected.insert(
        "z_last".to_string(),
        GpValue::Int(1i32.into()),
    );
    assert_eq!(gp_val, GpValue::Object(expected));
}
