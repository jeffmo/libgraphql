use crate::types::TypeAnnotation;

// Verifies identical type annotations are equivalent.
// Per GraphQL spec Section 3.6.1, parameter types must be
// structurally identical.
// https://spec.graphql.org/September2025/#IsValidImplementation()
// Written by Claude Code, reviewed by a human.
#[test]
fn equivalent_named_types() {
    let a = TypeAnnotation::named("String", false);
    let b = TypeAnnotation::named("String", false);
    assert!(a.is_equivalent_to(&b));
}

// Verifies nullability difference breaks equivalence.
// Written by Claude Code, reviewed by a human.
#[test]
fn non_equivalent_nullability() {
    let nullable = TypeAnnotation::named("String", true);
    let non_null = TypeAnnotation::named("String", false);
    assert!(!nullable.is_equivalent_to(&non_null));
}

// Verifies different type names break equivalence.
// Written by Claude Code, reviewed by a human.
#[test]
fn non_equivalent_different_names() {
    let a = TypeAnnotation::named("String", false);
    let b = TypeAnnotation::named("Int", false);
    assert!(!a.is_equivalent_to(&b));
}

// Verifies list vs named breaks equivalence.
// Written by Claude Code, reviewed by a human.
#[test]
fn non_equivalent_list_vs_named() {
    let named = TypeAnnotation::named("String", false);
    let list = TypeAnnotation::list(
        TypeAnnotation::named("String", false),
        false,
    );
    assert!(!named.is_equivalent_to(&list));
}

// Verifies nested list equivalence.
// Written by Claude Code, reviewed by a human.
#[test]
fn equivalent_nested_lists() {
    let a = TypeAnnotation::list(
        TypeAnnotation::named("Int", false),
        true,
    );
    let b = TypeAnnotation::list(
        TypeAnnotation::named("Int", false),
        true,
    );
    assert!(a.is_equivalent_to(&b));
}

// Verifies nested list non-equivalence when inner nullability differs.
// Written by Claude Code, reviewed by a human.
#[test]
fn non_equivalent_nested_list_inner_nullability() {
    let a = TypeAnnotation::list(
        TypeAnnotation::named("Int", false),
        true,
    );
    let b = TypeAnnotation::list(
        TypeAnnotation::named("Int", true),
        true,
    );
    assert!(!a.is_equivalent_to(&b));
}

// Verifies Display formatting matches GraphQL syntax.
// Written by Claude Code, reviewed by a human.
#[test]
fn display_formatting() {
    assert_eq!(
        TypeAnnotation::named("String", false).to_string(),
        "String!",
    );
    assert_eq!(
        TypeAnnotation::named("String", true).to_string(),
        "String",
    );
    assert_eq!(
        TypeAnnotation::list(
            TypeAnnotation::named("Int", false),
            true,
        ).to_string(),
        "[Int!]",
    );
    assert_eq!(
        TypeAnnotation::list(
            TypeAnnotation::named("Int", false),
            false,
        ).to_string(),
        "[Int!]!",
    );
    assert_eq!(
        TypeAnnotation::list(
            TypeAnnotation::list(
                TypeAnnotation::named("String", true),
                false,
            ),
            true,
        ).to_string(),
        "[[String]!]",
    );
}

// Verifies innermost_type_name unwraps nested lists.
// Written by Claude Code, reviewed by a human.
#[test]
fn innermost_type_name_nested() {
    let annot = TypeAnnotation::list(
        TypeAnnotation::list(
            TypeAnnotation::named("User", false),
            true,
        ),
        false,
    );
    assert_eq!(annot.innermost_type_name().as_str(), "User");
}

// Verifies innermost_type_name on a plain named type.
// Written by Claude Code, reviewed by a human.
#[test]
fn innermost_type_name_named() {
    let annot = TypeAnnotation::named("Boolean", true);
    assert_eq!(annot.innermost_type_name().as_str(), "Boolean");
}

// Verifies nullable() accessor for both named and list.
// Written by Claude Code, reviewed by a human.
#[test]
fn nullable_accessor() {
    assert!(!TypeAnnotation::named("String", false).nullable());
    assert!(TypeAnnotation::named("String", true).nullable());
    assert!(!TypeAnnotation::list(
        TypeAnnotation::named("Int", true),
        false,
    ).nullable());
    assert!(TypeAnnotation::list(
        TypeAnnotation::named("Int", false),
        true,
    ).nullable());
}

// Verifies serde round-trip via bincode for TypeAnnotation.
// Written by Claude Code, reviewed by a human.
#[test]
fn type_annotation_serde_roundtrip() {
    let annot = TypeAnnotation::list(
        TypeAnnotation::named("User", false),
        true,
    );
    let bytes = bincode::serde::encode_to_vec(
        &annot,
        bincode::config::standard(),
    ).unwrap();
    let (deserialized, _): (TypeAnnotation, _) =
        bincode::serde::decode_from_slice(
            &bytes,
            bincode::config::standard(),
        ).unwrap();
    assert_eq!(annot, deserialized);
}
