//! Tests for the [`crate::ast::TypeDefinition`] enum's
//! `append_source` delegation to inner variants.

use crate::ast::EnumTypeDefinition;
use crate::ast::EnumValueDefinition;
use crate::ast::FieldDefinition;
use crate::ast::InputObjectTypeDefinition;
use crate::ast::InputValueDefinition;
use crate::ast::InterfaceTypeDefinition;
use crate::ast::NamedTypeAnnotation;
use crate::ast::Nullability;
use crate::ast::ObjectTypeDefinition;
use crate::ast::TypeAnnotation;
use crate::ast::TypeDefinition;
use crate::ast::UnionTypeDefinition;
use crate::ast::tests::ast_test_utils::make_byte_span;
use crate::ast::tests::ast_test_utils::make_name;

/// Verify `TypeDefinition::Object` variant delegates
/// `append_source` correctly.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Types
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_definition_object_variant_source_slice() {
    let source = "type Foo { x: Int }";
    let td = TypeDefinition::Object(
        ObjectTypeDefinition {
            span: make_byte_span(0, 19),
            description: None,
            name: make_name("Foo", 5, 8),
            implements: vec![],
            directives: vec![],
            fields: vec![FieldDefinition {
                span: make_byte_span(11, 17),
                description: None,
                name: make_name("x", 11, 12),
                arguments: vec![],
                field_type: TypeAnnotation::Named(
                    NamedTypeAnnotation {
                        name: make_name(
                            "Int", 14, 17,
                        ),
                        nullability:
                            Nullability::Nullable,
                        span: make_byte_span(14, 17),
                    },
                ),
                directives: vec![],
                syntax: None,
            }],
            syntax: None,
        },
    );
    let mut sink = String::new();
    td.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `TypeDefinition::Scalar` variant delegates
/// `append_source` correctly.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Types
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_definition_scalar_variant_source_slice() {
    let source = "scalar DateTime";
    let td = TypeDefinition::Scalar(
        crate::ast::ScalarTypeDefinition {
            span: make_byte_span(0, 15),
            description: None,
            name: make_name("DateTime", 7, 15),
            directives: vec![],
            syntax: None,
        },
    );
    let mut sink = String::new();
    td.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `TypeDefinition::Enum` variant delegates
/// `append_source` correctly.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Types
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_definition_enum_variant_source_slice() {
    let source = "enum Status { ACTIVE }";
    let td = TypeDefinition::Enum(
        EnumTypeDefinition {
            span: make_byte_span(0, 22),
            description: None,
            name: make_name("Status", 5, 11),
            directives: vec![],
            values: vec![EnumValueDefinition {
                span: make_byte_span(14, 20),
                description: None,
                name: make_name(
                    "ACTIVE", 14, 20,
                ),
                directives: vec![],
            }],
            syntax: None,
        },
    );
    let mut sink = String::new();
    td.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `TypeDefinition::InputObject` variant
/// delegates `append_source` correctly.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Types
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_definition_input_object_variant_source_slice()
{
    let source =
        "input Filters { limit: Int }";
    let td = TypeDefinition::InputObject(
        InputObjectTypeDefinition {
            span: make_byte_span(0, 28),
            description: None,
            name: make_name("Filters", 6, 13),
            directives: vec![],
            fields: vec![InputValueDefinition {
                span: make_byte_span(16, 26),
                description: None,
                name: make_name("limit", 16, 21),
                value_type: TypeAnnotation::Named(
                    NamedTypeAnnotation {
                        name: make_name(
                            "Int", 23, 26,
                        ),
                        nullability:
                            Nullability::Nullable,
                        span: make_byte_span(
                            23, 26,
                        ),
                    },
                ),
                default_value: None,
                directives: vec![],
                syntax: None,
            }],
            syntax: None,
        },
    );
    let mut sink = String::new();
    td.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `TypeDefinition::Interface` variant delegates
/// `append_source` correctly.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Types
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_definition_interface_variant_source_slice() {
    let source = "interface Node { id: ID }";
    let td = TypeDefinition::Interface(
        InterfaceTypeDefinition {
            span: make_byte_span(0, 25),
            description: None,
            name: make_name("Node", 10, 14),
            implements: vec![],
            directives: vec![],
            fields: vec![FieldDefinition {
                span: make_byte_span(17, 23),
                description: None,
                name: make_name("id", 17, 19),
                arguments: vec![],
                field_type: TypeAnnotation::Named(
                    NamedTypeAnnotation {
                        name: make_name(
                            "ID", 21, 23,
                        ),
                        nullability:
                            Nullability::Nullable,
                        span: make_byte_span(
                            21, 23,
                        ),
                    },
                ),
                directives: vec![],
                syntax: None,
            }],
            syntax: None,
        },
    );
    let mut sink = String::new();
    td.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `TypeDefinition::Union` variant delegates
/// `append_source` correctly.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Types
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_definition_union_variant_source_slice() {
    let source = "union Result = A | B";
    let td = TypeDefinition::Union(
        UnionTypeDefinition {
            span: make_byte_span(0, 20),
            description: None,
            name: make_name("Result", 6, 12),
            directives: vec![],
            members: vec![
                make_name("A", 15, 16),
                make_name("B", 19, 20),
            ],
            syntax: None,
        },
    );
    let mut sink = String::new();
    td.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
