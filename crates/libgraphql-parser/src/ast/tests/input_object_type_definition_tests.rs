//! Tests for
//! [`crate::ast::InputObjectTypeDefinition`] and
//! [`crate::ast::InputObjectTypeDefinitionSyntax`].

use crate::ast::InputObjectTypeDefinition;
use crate::ast::InputValueDefinition;
use crate::ast::NamedTypeAnnotation;
use crate::ast::Nullability;
use crate::ast::TypeAnnotation;
use crate::ast::tests::ast_test_helpers::make_byte_span;
use crate::ast::tests::ast_test_helpers::make_name;

/// Verify `InputObjectTypeDefinition` stores name and
/// fields, and `append_source` slices the correct
/// source range.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Input-Objects
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn input_object_type_definition_source_slice() {
    let source =
        "input CreateUser { name: String }";
    let iotd = InputObjectTypeDefinition {
        span: make_byte_span(0, 33),
        description: None,
        name: make_name("CreateUser", 6, 16),
        directives: vec![],
        fields: vec![InputValueDefinition {
            span: make_byte_span(19, 31),
            description: None,
            name: make_name("name", 19, 23),
            value_type: TypeAnnotation::Named(
                NamedTypeAnnotation {
                    name: make_name(
                        "String", 25, 31,
                    ),
                    nullability:
                        Nullability::Nullable,
                    span: make_byte_span(25, 31),
                },
            ),
            default_value: None,
            directives: vec![],
            syntax: None,
        }],
        syntax: None,
    };
    assert_eq!(iotd.name.value, "CreateUser");
    assert_eq!(iotd.fields.len(), 1);

    let mut sink = String::new();
    iotd.append_source(
        &mut sink,
        Some(source),
    );
    assert_eq!(sink, source);
}
