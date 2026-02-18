//! Tests for [`crate::ast::ListTypeAnnotation`].

use crate::ast::ListTypeAnnotation;
use crate::ast::NamedTypeAnnotation;
use crate::ast::Nullability;
use crate::ast::TypeAnnotation;
use crate::ast::tests::ast_test_helpers::make_name;
use crate::ast::tests::ast_test_helpers::make_span;

/// Verify `ListTypeAnnotation` with a named element type.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Type-References
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn list_type_annotation_construct_and_source_slice() {
    let source = "[Int]";
    let lta = ListTypeAnnotation {
        element_type: Box::new(
            TypeAnnotation::Named(
                NamedTypeAnnotation {
                    name: make_name("Int", 1, 4),
                    nullability: Nullability::Nullable,
                    span: make_span(1, 4),
                },
            ),
        ),
        nullability: Nullability::Nullable,
        span: make_span(0, 5),
        syntax: None,
    };
    assert!(matches!(
        *lta.element_type,
        TypeAnnotation::Named(_),
    ));

    let mut sink = String::new();
    lta.append_source(&mut sink, Some(source));
    assert_eq!(sink, "[Int]");
}
