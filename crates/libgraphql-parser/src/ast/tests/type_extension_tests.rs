//! Tests for the [`crate::ast::TypeExtension`] enum's
//! `append_source` delegation to inner variants.

use crate::ast::DirectiveAnnotation;
use crate::ast::EnumTypeExtension;
use crate::ast::EnumValueDefinition;
use crate::ast::FieldDefinition;
use crate::ast::InputObjectTypeExtension;
use crate::ast::InterfaceTypeExtension;
use crate::ast::NamedTypeAnnotation;
use crate::ast::Nullability;
use crate::ast::ObjectTypeExtension;
use crate::ast::ScalarTypeExtension;
use crate::ast::TypeAnnotation;
use crate::ast::TypeExtension;
use crate::ast::UnionTypeExtension;
use crate::ast::tests::ast_test_utils::make_byte_span;
use crate::ast::tests::ast_test_utils::make_name;

/// Verify `TypeExtension::Object` variant delegates
/// `append_source` correctly.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Type-Extensions
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_extension_object_variant_source_slice() {
    let source = "extend type Foo { y: Int }";
    let te = TypeExtension::Object(
        ObjectTypeExtension {
            span: make_byte_span(0, 26),
            name: make_name("Foo", 12, 15),
            implements: vec![],
            directives: vec![],
            fields: vec![FieldDefinition {
                span: make_byte_span(18, 24),
                description: None,
                name: make_name("y", 18, 19),
                arguments: vec![],
                field_type: TypeAnnotation::Named(
                    NamedTypeAnnotation {
                        name: make_name(
                            "Int", 21, 24,
                        ),
                        nullability:
                            Nullability::Nullable,
                        span: make_byte_span(21, 24),
                    },
                ),
                directives: vec![],
                syntax: None,
            }],
            syntax: None,
        },
    );
    let mut sink = String::new();
    te.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `TypeExtension::Enum` variant delegates
/// `append_source` correctly.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Type-Extensions
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_extension_enum_variant_source_slice() {
    let source = "extend enum Role { ADMIN }";
    let te = TypeExtension::Enum(
        EnumTypeExtension {
            span: make_byte_span(0, 26),
            name: make_name("Role", 12, 16),
            directives: vec![],
            values: vec![EnumValueDefinition {
                span: make_byte_span(19, 24),
                description: None,
                name: make_name(
                    "ADMIN", 19, 24,
                ),
                directives: vec![],
            }],
            syntax: None,
        },
    );
    let mut sink = String::new();
    te.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `TypeExtension::InputObject` variant
/// delegates `append_source` correctly.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Type-Extensions
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_extension_input_object_variant_source_slice() {
    let source = "extend input Opts @foo";
    let te = TypeExtension::InputObject(
        InputObjectTypeExtension {
            span: make_byte_span(0, 22),
            name: make_name("Opts", 13, 17),
            directives: vec![
                DirectiveAnnotation {
                    span: make_byte_span(18, 22),
                    name: make_name(
                        "foo", 19, 22,
                    ),
                    arguments: vec![],
                    syntax: None,
                },
            ],
            fields: vec![],
            syntax: None,
        },
    );
    let mut sink = String::new();
    te.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `TypeExtension::Interface` variant delegates
/// `append_source` correctly.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Type-Extensions
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_extension_interface_variant_source_slice() {
    let source =
        "extend interface Node @bar";
    let te = TypeExtension::Interface(
        InterfaceTypeExtension {
            span: make_byte_span(0, 26),
            name: make_name("Node", 17, 21),
            implements: vec![],
            directives: vec![
                DirectiveAnnotation {
                    span: make_byte_span(22, 26),
                    name: make_name(
                        "bar", 23, 26,
                    ),
                    arguments: vec![],
                    syntax: None,
                },
            ],
            fields: vec![],
            syntax: None,
        },
    );
    let mut sink = String::new();
    te.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `TypeExtension::Scalar` variant delegates
/// `append_source` correctly.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Type-Extensions
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_extension_scalar_variant_source_slice() {
    let source = "extend scalar Date @tag";
    let te = TypeExtension::Scalar(
        ScalarTypeExtension {
            span: make_byte_span(0, 23),
            name: make_name("Date", 14, 18),
            directives: vec![
                DirectiveAnnotation {
                    span: make_byte_span(19, 23),
                    name: make_name(
                        "tag", 20, 23,
                    ),
                    arguments: vec![],
                    syntax: None,
                },
            ],
            syntax: None,
        },
    );
    let mut sink = String::new();
    te.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `TypeExtension::Union` variant delegates
/// `append_source` correctly.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Type-Extensions
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_extension_union_variant_source_slice() {
    let source = "extend union Result = C";
    let te = TypeExtension::Union(
        UnionTypeExtension {
            span: make_byte_span(0, 23),
            name: make_name("Result", 13, 19),
            directives: vec![],
            members: vec![
                make_name("C", 22, 23),
            ],
            syntax: None,
        },
    );
    let mut sink = String::new();
    te.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
