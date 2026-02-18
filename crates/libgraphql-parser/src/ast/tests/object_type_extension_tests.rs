//! Tests for [`crate::ast::ObjectTypeExtension`] and
//! [`crate::ast::ObjectTypeExtensionSyntax`].

use crate::ast::FieldDefinition;
use crate::ast::NamedTypeAnnotation;
use crate::ast::Nullability;
use crate::ast::ObjectTypeExtension;
use crate::ast::TypeAnnotation;
use crate::ast::tests::ast_test_helpers::make_name;
use crate::ast::tests::ast_test_helpers::make_span;

/// Verify `ObjectTypeExtension` stores name, optional
/// implements, directives, and fields.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Object-Extensions
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_type_extension_construct_and_source_slice() {
    let source =
        "extend type Query { age: Int }";
    let ote = ObjectTypeExtension {
        span: make_span(0, 30),
        name: make_name("Query", 12, 17),
        implements: vec![],
        directives: vec![],
        fields: vec![FieldDefinition {
            span: make_span(20, 28),
            description: None,
            name: make_name("age", 20, 23),
            arguments: vec![],
            field_type: TypeAnnotation::Named(
                NamedTypeAnnotation {
                    name: make_name(
                        "Int", 25, 28,
                    ),
                    nullability:
                        Nullability::Nullable,
                    span: make_span(25, 28),
                },
            ),
            directives: vec![],
            syntax: None,
        }],
        syntax: None,
    };
    assert_eq!(ote.name.value, "Query");
    assert_eq!(ote.fields.len(), 1);

    let mut sink = String::new();
    ote.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
