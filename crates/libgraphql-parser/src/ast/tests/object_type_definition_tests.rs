//! Tests for [`crate::ast::ObjectTypeDefinition`] and
//! [`crate::ast::ObjectTypeDefinitionSyntax`].

use crate::ast::FieldDefinition;
use crate::ast::NamedTypeAnnotation;
use crate::ast::Nullability;
use crate::ast::ObjectTypeDefinition;
use crate::ast::TypeAnnotation;
use crate::ast::tests::ast_test_utils::make_byte_span;
use crate::ast::tests::ast_test_utils::make_name;

/// Verify `ObjectTypeDefinition` stores name, optional
/// implements, fields, and directives.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Objects
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_type_definition_construct_and_source_slice() {
    let source =
        "type Query { hello: String }";
    let otd = ObjectTypeDefinition {
        span: make_byte_span(0, 28),
        description: None,
        name: make_name("Query", 5, 10),
        implements: vec![],
        directives: vec![],
        fields: vec![FieldDefinition {
            span: make_byte_span(13, 26),
            description: None,
            name: make_name("hello", 13, 18),
            arguments: vec![],
            field_type: TypeAnnotation::Named(
                NamedTypeAnnotation {
                    name: make_name(
                        "String", 20, 26,
                    ),
                    nullability:
                        Nullability::Nullable,
                    span: make_byte_span(20, 26),
                },
            ),
            directives: vec![],
            syntax: None,
        }],
        syntax: None,
    };
    assert_eq!(otd.name.value, "Query");
    assert_eq!(otd.fields.len(), 1);
    assert_eq!(
        otd.fields[0].name.value, "hello",
    );

    let mut sink = String::new();
    otd.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `ObjectTypeDefinition` with `implements`
/// interface list.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Objects
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_type_definition_with_implements() {
    let source =
        "type Dog implements Animal { name: String }";
    let otd = ObjectTypeDefinition {
        span: make_byte_span(0, 43),
        description: None,
        name: make_name("Dog", 5, 8),
        implements: vec![
            make_name("Animal", 20, 26),
        ],
        directives: vec![],
        fields: vec![FieldDefinition {
            span: make_byte_span(29, 41),
            description: None,
            name: make_name("name", 29, 33),
            arguments: vec![],
            field_type: TypeAnnotation::Named(
                NamedTypeAnnotation {
                    name: make_name(
                        "String", 35, 41,
                    ),
                    nullability:
                        Nullability::Nullable,
                    span: make_byte_span(35, 41),
                },
            ),
            directives: vec![],
            syntax: None,
        }],
        syntax: None,
    };
    assert_eq!(otd.implements.len(), 1);
    assert_eq!(
        otd.implements[0].value, "Animal",
    );

    let mut sink = String::new();
    otd.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
