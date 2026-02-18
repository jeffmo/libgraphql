//! Tests for the [`crate::ast::Value`] enum's
//! `append_source` delegation to inner variants.

use std::borrow::Cow;

use crate::ast::BooleanValue;
use crate::ast::EnumValue;
use crate::ast::FloatValue;
use crate::ast::IntValue;
use crate::ast::ListValue;
use crate::ast::NullValue;
use crate::ast::ObjectField;
use crate::ast::ObjectValue;
use crate::ast::StringValue;
use crate::ast::Value;
use crate::ast::VariableValue;
use crate::ast::tests::ast_test_helpers::make_name;
use crate::ast::tests::ast_test_helpers::make_span;

/// Verify the `Value` enum delegates `append_source`
/// correctly for each variant. We test a representative
/// subset: Int, Boolean, String, and Null.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Input-Values
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_enum_delegates_append_source() {
    let src_int = "42";
    let val_int = Value::Int(IntValue {
        value: 42,
        span: make_span(0, 2),
        syntax: None,
    });
    let mut sink = String::new();
    val_int.append_source(&mut sink, Some(src_int));
    assert_eq!(sink, "42");

    let src_bool = "false";
    let val_bool = Value::Boolean(BooleanValue {
        value: false,
        span: make_span(0, 5),
        syntax: None,
    });
    sink.clear();
    val_bool.append_source(
        &mut sink,
        Some(src_bool),
    );
    assert_eq!(sink, "false");

    let src_str = r#""hi""#;
    let val_str = Value::String(StringValue {
        value: Cow::Borrowed("hi"),
        span: make_span(0, 4),
        syntax: None,
    });
    sink.clear();
    val_str.append_source(
        &mut sink,
        Some(src_str),
    );
    assert_eq!(sink, r#""hi""#);

    let src_null = "null";
    let val_null = Value::Null(NullValue {
        span: make_span(0, 4),
        syntax: None,
    });
    sink.clear();
    val_null.append_source(
        &mut sink,
        Some(src_null),
    );
    assert_eq!(sink, "null");
}

/// Verify the `Value::Variable` variant delegates
/// `append_source` correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_enum_variable_variant_source_slice() {
    let source = "$id";
    let val = Value::Variable(VariableValue {
        name: make_name("id", 1, 3),
        span: make_span(0, 3),
        syntax: None,
    });
    let mut sink = String::new();
    val.append_source(&mut sink, Some(source));
    assert_eq!(sink, "$id");
}

/// Verify `Value::Enum` variant delegates
/// `append_source` correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_enum_enum_variant_source_slice() {
    let source = "ACTIVE";
    let val = Value::Enum(EnumValue {
        value: Cow::Borrowed("ACTIVE"),
        span: make_span(0, 6),
        syntax: None,
    });
    let mut sink = String::new();
    val.append_source(&mut sink, Some(source));
    assert_eq!(sink, "ACTIVE");
}

/// Verify `Value::Float` variant delegates
/// `append_source` correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_enum_float_variant_source_slice() {
    let source = "9.81";
    let val = Value::Float(FloatValue {
        value: 9.81,
        span: make_span(0, 4),
        syntax: None,
    });
    let mut sink = String::new();
    val.append_source(&mut sink, Some(source));
    assert_eq!(sink, "9.81");
}

/// Verify `Value::List` variant delegates
/// `append_source` correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_enum_list_variant_source_slice() {
    let source = "[true]";
    let val = Value::List(ListValue {
        values: vec![Value::Boolean(BooleanValue {
            value: true,
            span: make_span(1, 5),
            syntax: None,
        })],
        span: make_span(0, 6),
        syntax: None,
    });
    let mut sink = String::new();
    val.append_source(&mut sink, Some(source));
    assert_eq!(sink, "[true]");
}

/// Verify `Value::Object` variant delegates
/// `append_source` correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn value_enum_object_variant_source_slice() {
    let source = "{a: 1}";
    let val = Value::Object(ObjectValue {
        fields: vec![ObjectField {
            name: make_name("a", 1, 2),
            value: Value::Int(IntValue {
                value: 1,
                span: make_span(4, 5),
                syntax: None,
            }),
            span: make_span(1, 5),
            syntax: None,
        }],
        span: make_span(0, 6),
        syntax: None,
    });
    let mut sink = String::new();
    val.append_source(&mut sink, Some(source));
    assert_eq!(sink, "{a: 1}");
}
