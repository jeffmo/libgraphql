//! Tests for [`crate::ast::NamedTypeAnnotation`].

use crate::ast::NamedTypeAnnotation;
use crate::ast::Nullability;
use crate::ast::tests::ast_test_helpers::make_name;
use crate::ast::tests::ast_test_helpers::make_span;

/// Verify `NamedTypeAnnotation` stores name and
/// nullability, and slices the correct source range.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Type-References
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn named_type_annotation_nullable() {
    let source = "String";
    let nta = NamedTypeAnnotation {
        name: make_name("String", 0, 6),
        nullability: Nullability::Nullable,
        span: make_span(0, 6),
    };
    assert_eq!(nta.name.value, "String");
    assert_eq!(
        nta.nullability,
        Nullability::Nullable,
    );

    let mut sink = String::new();
    nta.append_source(&mut sink, Some(source));
    assert_eq!(sink, "String");
}

/// Verify `NamedTypeAnnotation` with `NonNull`
/// nullability slices the `!` as part of the span.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Type-References
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn named_type_annotation_non_null() {
    let source = "String!";
    let nta = NamedTypeAnnotation {
        name: make_name("String", 0, 6),
        nullability: Nullability::NonNull {
            syntax: None,
        },
        span: make_span(0, 7),
    };
    assert!(matches!(
        nta.nullability,
        Nullability::NonNull { .. },
    ));

    let mut sink = String::new();
    nta.append_source(&mut sink, Some(source));
    assert_eq!(sink, "String!");
}
