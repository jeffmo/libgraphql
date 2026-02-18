//! Tests for the [`crate::ast::TypeAnnotation`] enum's
//! `append_source` delegation to inner variants.

use crate::ast::ListTypeAnnotation;
use crate::ast::NamedTypeAnnotation;
use crate::ast::Nullability;
use crate::ast::TypeAnnotation;
use crate::ast::tests::ast_test_helpers::make_byte_span;
use crate::ast::tests::ast_test_helpers::make_name;

/// Verify `TypeAnnotation::Named` variant delegates
/// `append_source` correctly.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Type-References
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_annotation_named_variant_source_slice() {
    let source = "Boolean";
    let ta = TypeAnnotation::Named(
        NamedTypeAnnotation {
            name: make_name("Boolean", 0, 7),
            nullability: Nullability::Nullable,
            span: make_byte_span(0, 7),
        },
    );
    let mut sink = String::new();
    ta.append_source(&mut sink, Some(source));
    assert_eq!(sink, "Boolean");
}

/// Verify `TypeAnnotation::List` variant delegates
/// `append_source` correctly.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Type-References
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn type_annotation_list_variant_source_slice() {
    let source = "[String!]!";
    let inner = NamedTypeAnnotation {
        name: make_name("String", 1, 7),
        nullability: Nullability::NonNull {
            syntax: None,
        },
        span: make_byte_span(1, 8),
    };
    let ta = TypeAnnotation::List(
        ListTypeAnnotation {
            element_type: Box::new(
                TypeAnnotation::Named(inner),
            ),
            nullability: Nullability::NonNull {
                syntax: None,
            },
            span: make_byte_span(0, 10),
            syntax: None,
        },
    );
    let mut sink = String::new();
    ta.append_source(&mut sink, Some(source));
    assert_eq!(sink, "[String!]!");
}
