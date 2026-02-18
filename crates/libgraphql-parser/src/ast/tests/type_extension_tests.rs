//! Tests for the [`crate::ast::TypeExtension`] enum's
//! `append_source` delegation to inner variants.

use crate::ast::FieldDefinition;
use crate::ast::NamedTypeAnnotation;
use crate::ast::Nullability;
use crate::ast::ObjectTypeExtension;
use crate::ast::TypeAnnotation;
use crate::ast::TypeExtension;
use crate::ast::tests::ast_test_helpers::make_name;
use crate::ast::tests::ast_test_helpers::make_span;

/// Verify `TypeExtension::Object` variant delegates
/// `append_source` correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_extension_object_variant_source_slice() {
    let source = "extend type Foo { y: Int }";
    let te = TypeExtension::Object(
        ObjectTypeExtension {
            span: make_span(0, 26),
            name: make_name("Foo", 12, 15),
            implements: vec![],
            directives: vec![],
            fields: vec![FieldDefinition {
                span: make_span(18, 24),
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
                        span: make_span(21, 24),
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
