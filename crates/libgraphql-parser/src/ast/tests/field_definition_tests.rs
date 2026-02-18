//! Tests for [`crate::ast::FieldDefinition`].

use crate::ast::FieldDefinition;
use crate::ast::NamedTypeAnnotation;
use crate::ast::Nullability;
use crate::ast::TypeAnnotation;
use crate::ast::tests::ast_test_helpers::make_name;
use crate::ast::tests::ast_test_helpers::make_span;

/// Verify `FieldDefinition` stores name, type, and
/// optional arguments/directives.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#FieldsDefinition
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn field_definition_construct_and_source_slice() {
    let source = "name: String";
    let fd = FieldDefinition {
        span: make_span(0, 12),
        description: None,
        name: make_name("name", 0, 4),
        arguments: vec![],
        field_type: TypeAnnotation::Named(
            NamedTypeAnnotation {
                name: make_name("String", 6, 12),
                nullability: Nullability::Nullable,
                span: make_span(6, 12),
            },
        ),
        directives: vec![],
        syntax: None,
    };
    assert_eq!(fd.name.value, "name");

    let mut sink = String::new();
    fd.append_source(&mut sink, Some(source));
    assert_eq!(sink, "name: String");
}
