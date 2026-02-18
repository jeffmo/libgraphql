//! Tests for the [`crate::ast::TypeDefinition`] enum's
//! `append_source` delegation to inner variants.

use crate::ast::FieldDefinition;
use crate::ast::NamedTypeAnnotation;
use crate::ast::Nullability;
use crate::ast::ObjectTypeDefinition;
use crate::ast::TypeAnnotation;
use crate::ast::TypeDefinition;
use crate::ast::tests::ast_test_helpers::make_name;
use crate::ast::tests::ast_test_helpers::make_span;

/// Verify `TypeDefinition::Object` variant delegates
/// `append_source` correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_definition_object_variant_source_slice() {
    let source = "type Foo { x: Int }";
    let td = TypeDefinition::Object(
        ObjectTypeDefinition {
            span: make_span(0, 19),
            description: None,
            name: make_name("Foo", 5, 8),
            implements: vec![],
            directives: vec![],
            fields: vec![FieldDefinition {
                span: make_span(11, 17),
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
                        span: make_span(14, 17),
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
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_definition_scalar_variant_source_slice() {
    let source = "scalar DateTime";
    let td = TypeDefinition::Scalar(
        crate::ast::ScalarTypeDefinition {
            span: make_span(0, 15),
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
