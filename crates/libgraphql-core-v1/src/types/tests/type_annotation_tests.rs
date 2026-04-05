use crate::located::Located;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::FieldDefinition;
use crate::types::FieldedTypeData;
use crate::types::GraphQLType;
use crate::types::InterfaceType;
use crate::types::ObjectType;
use crate::types::TypeAnnotation;
use crate::types::UnionType;
use indexmap::IndexMap;

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

fn sample_object(name: &str) -> GraphQLType {
    GraphQLType::Object(Box::new(ObjectType(FieldedTypeData {
        description: None,
        directives: vec![],
        fields: IndexMap::new(),
        interfaces: vec![],
        name: TypeName::new(name),
        span: Span::builtin(),
    })))
}

// ── is_subtype_of tests ───────────────────────

// Verifies is_subtype_of for same-name types with different
// nullability (non-null is subtype of nullable).
// https://spec.graphql.org/September2025/#IsSubType()
// Written by Claude Code, reviewed by a human.
#[test]
fn subtype_same_name_nullability() {
    let types_map = IndexMap::new();

    let non_null = TypeAnnotation::named("String", false);
    let nullable = TypeAnnotation::named("String", true);

    // Non-null is subtype of nullable
    assert!(non_null.is_subtype_of(&types_map, &nullable));
    // Same nullability (non-null) is subtype of itself
    assert!(non_null.is_subtype_of(&types_map, &non_null));
    // Nullable is subtype of nullable (same name)
    assert!(nullable.is_subtype_of(&types_map, &nullable));
    // Nullable is NOT subtype of non-null
    assert!(!nullable.is_subtype_of(&types_map, &non_null));
}

// Verifies is_subtype_of returns false for cross-kind
// (Named vs List) comparisons.
// https://spec.graphql.org/September2025/#IsSubType()
// Written by Claude Code, reviewed by a human.
#[test]
fn subtype_cross_kind_returns_false() {
    let types_map = IndexMap::new();

    let named = TypeAnnotation::named("String", false);
    let list = TypeAnnotation::list(
        TypeAnnotation::named("String", false),
        false,
    );

    assert!(!named.is_subtype_of(&types_map, &list));
    assert!(!list.is_subtype_of(&types_map, &named));
}

// Verifies is_subtype_of for list types with covariant inner types.
// https://spec.graphql.org/September2025/#IsSubType()
// Written by Claude Code, reviewed by a human.
#[test]
fn subtype_list_covariance() {
    let types_map = IndexMap::new();

    let non_null_list = TypeAnnotation::list(
        TypeAnnotation::named("String", false),
        false,
    );
    let nullable_list = TypeAnnotation::list(
        TypeAnnotation::named("String", false),
        true,
    );

    assert!(non_null_list.is_subtype_of(&types_map, &nullable_list));
    assert!(!nullable_list.is_subtype_of(&types_map, &non_null_list));
}

// Verifies is_subtype_of for abstract type subtyping (object
// implements interface).
// https://spec.graphql.org/September2025/#IsSubType()
// Written by Claude Code, reviewed by a human.
#[test]
fn subtype_interface_implementation() {
    let mut types_map = IndexMap::new();

    let mut node_fields = IndexMap::new();
    node_fields.insert(FieldName::new("id"), FieldDefinition {
        description: None,
        directives: vec![],
        name: FieldName::new("id"),
        parameters: IndexMap::new(),
        parent_type_name: TypeName::new("Node"),
        span: Span::builtin(),
        type_annotation: TypeAnnotation::named("ID", false),
    });
    types_map.insert(
        TypeName::new("Node"),
        GraphQLType::Interface(Box::new(InterfaceType(FieldedTypeData {
            description: None,
            directives: vec![],
            fields: node_fields,
            interfaces: vec![],
            name: TypeName::new("Node"),
            span: Span::builtin(),
        }))),
    );

    types_map.insert(
        TypeName::new("User"),
        GraphQLType::Object(Box::new(ObjectType(FieldedTypeData {
            description: None,
            directives: vec![],
            fields: IndexMap::new(),
            interfaces: vec![Located {
                value: TypeName::new("Node"),
                span: Span::builtin(),
            }],
            name: TypeName::new("User"),
            span: Span::builtin(),
        }))),
    );

    let user_annot = TypeAnnotation::named("User", false);
    let node_annot = TypeAnnotation::named("Node", false);

    assert!(user_annot.is_subtype_of(&types_map, &node_annot));
    assert!(!node_annot.is_subtype_of(&types_map, &user_annot));
}

// Verifies is_subtype_of returns false when the sub-type name
// is absent from the types_map (interface case).
// https://spec.graphql.org/September2025/#IsSubType()
// Written by Claude Code, reviewed by a human.
#[test]
fn subtype_unknown_sub_type_returns_false() {
    let mut types_map = IndexMap::new();

    types_map.insert(
        TypeName::new("Node"),
        GraphQLType::Interface(Box::new(InterfaceType(FieldedTypeData {
            description: None,
            directives: vec![],
            fields: IndexMap::new(),
            interfaces: vec![],
            name: TypeName::new("Node"),
            span: Span::builtin(),
        }))),
    );

    // "Unknown" is not in types_map
    let unknown = TypeAnnotation::named("Unknown", false);
    let node = TypeAnnotation::named("Node", false);

    assert!(!unknown.is_subtype_of(&types_map, &node));
}

// Verifies is_subtype_of for union member subtyping.
// https://spec.graphql.org/September2025/#IsSubType()
// Written by Claude Code, reviewed by a human.
#[test]
fn subtype_union_member() {
    let mut types_map = IndexMap::new();

    types_map.insert(
        TypeName::new("User"),
        sample_object("User"),
    );
    types_map.insert(
        TypeName::new("SearchResult"),
        GraphQLType::Union(Box::new(UnionType {
            description: None,
            directives: vec![],
            members: vec![
                Located {
                    value: TypeName::new("User"),
                    span: Span::builtin(),
                },
            ],
            name: TypeName::new("SearchResult"),
            span: Span::builtin(),
        })),
    );

    let user_annot = TypeAnnotation::named("User", false);
    let search_annot = TypeAnnotation::named("SearchResult", false);

    assert!(user_annot.is_subtype_of(&types_map, &search_annot));
    assert!(!search_annot.is_subtype_of(&types_map, &user_annot));
}
