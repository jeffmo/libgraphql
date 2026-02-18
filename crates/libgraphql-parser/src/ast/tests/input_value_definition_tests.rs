//! Tests for [`crate::ast::InputValueDefinition`].

use crate::ast::InputValueDefinition;
use crate::ast::IntValue;
use crate::ast::NamedTypeAnnotation;
use crate::ast::Nullability;
use crate::ast::TypeAnnotation;
use crate::ast::Value;
use crate::ast::tests::ast_test_helpers::make_byte_span;
use crate::ast::tests::ast_test_helpers::make_name;

/// Verify `InputValueDefinition` stores name, type,
/// optional default value, and directives.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#InputValueDefinition
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn input_value_definition_construct_and_source_slice() {
    let source = "limit: Int = 10";
    let ivd = InputValueDefinition {
        span: make_byte_span(0, 15),
        description: None,
        name: make_name("limit", 0, 5),
        value_type: TypeAnnotation::Named(
            NamedTypeAnnotation {
                name: make_name("Int", 7, 10),
                nullability: Nullability::Nullable,
                span: make_byte_span(7, 10),
            },
        ),
        default_value: Some(Value::Int(IntValue {
            value: 10,
            span: make_byte_span(13, 15),
            syntax: None,
        })),
        directives: vec![],
        syntax: None,
    };
    assert_eq!(ivd.name.value, "limit");
    assert!(ivd.default_value.is_some());

    let mut sink = String::new();
    ivd.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
