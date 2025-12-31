use crate::loc;
use crate::types::ListTypeAnnotation;
use crate::types::NamedGraphQLTypeRef;
use crate::types::NamedTypeAnnotation;
use crate::types::TypeAnnotation;

#[test]
fn named_type_annotation_equivalence_same_type_same_nullability() {
    // Create two NamedTypeAnnotation instances with:
    // - Same type name ("Int")
    // - Same nullability (both nullable)
    // - Different source locations
    // These should be considered equivalent

    let location1 = loc::SourceLocation::SchemaFile(loc::FilePosition {
        col: 10,
        file: Box::new("schema1.graphql".into()),
        line: 5,
    });

    let location2 = loc::SourceLocation::SchemaFile(loc::FilePosition {
        col: 25,
        file: Box::new("schema2.graphql".into()),
        line: 100,
    });

    let annot1 = NamedTypeAnnotation {
        nullable: true,
        type_ref: NamedGraphQLTypeRef::new("Int", location1),
    };

    let annot2 = NamedTypeAnnotation {
        nullable: true,
        type_ref: NamedGraphQLTypeRef::new("Int", location2),
    };

    assert!(annot1.is_equivalent_to(&annot2));
    assert!(annot2.is_equivalent_to(&annot1)); // Should be symmetric
}

#[test]
fn named_type_annotation_equivalence_same_type_different_nullability() {
    // Create two NamedTypeAnnotation instances with:
    // - Same type name ("String")
    // - Different nullability (one nullable, one non-null)
    // These should NOT be considered equivalent

    let location = loc::SourceLocation::Schema;

    let annot_nullable = NamedTypeAnnotation {
        nullable: true,
        type_ref: NamedGraphQLTypeRef::new("String", location.clone()),
    };

    let annot_non_null = NamedTypeAnnotation {
        nullable: false,
        type_ref: NamedGraphQLTypeRef::new("String", location),
    };

    assert!(!annot_nullable.is_equivalent_to(&annot_non_null));
    assert!(!annot_non_null.is_equivalent_to(&annot_nullable)); // Should be symmetric
}

#[test]
fn named_type_annotation_equivalence_different_type_same_nullability() {
    // Create two NamedTypeAnnotation instances with:
    // - Different type names ("Int" vs "String")
    // - Same nullability (both nullable)
    // These should NOT be considered equivalent

    let location = loc::SourceLocation::Schema;

    let annot_int = NamedTypeAnnotation {
        nullable: true,
        type_ref: NamedGraphQLTypeRef::new("Int", location.clone()),
    };

    let annot_string = NamedTypeAnnotation {
        nullable: true,
        type_ref: NamedGraphQLTypeRef::new("String", location),
    };

    assert!(!annot_int.is_equivalent_to(&annot_string));
    assert!(!annot_string.is_equivalent_to(&annot_int)); // Should be symmetric
}

#[test]
fn named_type_annotation_equivalence_non_null_types() {
    // Create two NamedTypeAnnotation instances with:
    // - Same type name ("Boolean")
    // - Same nullability (both non-null)
    // - Different source locations
    // These should be considered equivalent

    let location1 = loc::SourceLocation::SchemaFile(loc::FilePosition {
        col: 1,
        file: Box::new("a.graphql".into()),
        line: 1,
    });

    let location2 = loc::SourceLocation::SchemaFile(loc::FilePosition {
        col: 50,
        file: Box::new("b.graphql".into()),
        line: 200,
    });

    let annot1 = NamedTypeAnnotation {
        nullable: false,
        type_ref: NamedGraphQLTypeRef::new("Boolean", location1),
    };

    let annot2 = NamedTypeAnnotation {
        nullable: false,
        type_ref: NamedGraphQLTypeRef::new("Boolean", location2),
    };

    assert!(annot1.is_equivalent_to(&annot2));
    assert!(annot2.is_equivalent_to(&annot1)); // Should be symmetric
}

#[test]
fn list_type_annotation_equivalence_same_inner_type_same_nullability() {
    // Create two ListTypeAnnotation instances with:
    // - Same inner type ([Int])
    // - Same nullability (both nullable)
    // - Different source locations
    // These should be considered equivalent

    let location1 = loc::SourceLocation::SchemaFile(loc::FilePosition {
        col: 10,
        file: Box::new("schema1.graphql".into()),
        line: 5,
    });

    let location2 = loc::SourceLocation::SchemaFile(loc::FilePosition {
        col: 25,
        file: Box::new("schema2.graphql".into()),
        line: 100,
    });

    let inner1 = TypeAnnotation::Named(NamedTypeAnnotation {
        nullable: true,
        type_ref: NamedGraphQLTypeRef::new("Int", location1.clone()),
    });

    let inner2 = TypeAnnotation::Named(NamedTypeAnnotation {
        nullable: true,
        type_ref: NamedGraphQLTypeRef::new("Int", location2.clone()),
    });

    let list1 = ListTypeAnnotation {
        nullable: true,
        inner_type_ref: Box::new(inner1),
        ref_location: location1,
    };

    let list2 = ListTypeAnnotation {
        nullable: true,
        inner_type_ref: Box::new(inner2),
        ref_location: location2,
    };

    assert!(list1.is_equivalent_to(&list2));
    assert!(list2.is_equivalent_to(&list1)); // Should be symmetric
}

#[test]
fn list_type_annotation_equivalence_different_inner_nullability() {
    // Create two ListTypeAnnotation instances with:
    // - Same outer nullability
    // - Different inner nullability ([Int] vs [Int!])
    // These should NOT be considered equivalent

    let location = loc::SourceLocation::Schema;

    let inner_nullable = TypeAnnotation::Named(NamedTypeAnnotation {
        nullable: true,
        type_ref: NamedGraphQLTypeRef::new("Int", location.clone()),
    });

    let inner_non_null = TypeAnnotation::Named(NamedTypeAnnotation {
        nullable: false,
        type_ref: NamedGraphQLTypeRef::new("Int", location.clone()),
    });

    let list1 = ListTypeAnnotation {
        nullable: true,
        inner_type_ref: Box::new(inner_nullable),
        ref_location: location.clone(),
    };

    let list2 = ListTypeAnnotation {
        nullable: true,
        inner_type_ref: Box::new(inner_non_null),
        ref_location: location,
    };

    assert!(!list1.is_equivalent_to(&list2));
    assert!(!list2.is_equivalent_to(&list1)); // Should be symmetric
}

#[test]
fn list_type_annotation_equivalence_different_outer_nullability() {
    // Create two ListTypeAnnotation instances with:
    // - Same inner type
    // - Different outer nullability ([Int] vs [Int]!)
    // These should NOT be considered equivalent

    let location = loc::SourceLocation::Schema;

    let inner = TypeAnnotation::Named(NamedTypeAnnotation {
        nullable: true,
        type_ref: NamedGraphQLTypeRef::new("Int", location.clone()),
    });

    let list_nullable = ListTypeAnnotation {
        nullable: true,
        inner_type_ref: Box::new(inner.clone()),
        ref_location: location.clone(),
    };

    let list_non_null = ListTypeAnnotation {
        nullable: false,
        inner_type_ref: Box::new(inner),
        ref_location: location,
    };

    assert!(!list_nullable.is_equivalent_to(&list_non_null));
    assert!(!list_non_null.is_equivalent_to(&list_nullable)); // Should be symmetric
}

#[test]
fn list_type_annotation_equivalence_nested_lists() {
    // Create two ListTypeAnnotation instances with:
    // - Nested list structure ([[Int]])
    // - Same nullability at all levels
    // - Different source locations
    // These should be considered equivalent

    let location1 = loc::SourceLocation::SchemaFile(loc::FilePosition {
        col: 1,
        file: Box::new("a.graphql".into()),
        line: 1,
    });

    let location2 = loc::SourceLocation::SchemaFile(loc::FilePosition {
        col: 50,
        file: Box::new("b.graphql".into()),
        line: 200,
    });

    // Build [[Int]] for first annotation
    let inner_named1 = TypeAnnotation::Named(NamedTypeAnnotation {
        nullable: true,
        type_ref: NamedGraphQLTypeRef::new("Int", location1.clone()),
    });
    let inner_list1 = TypeAnnotation::List(ListTypeAnnotation {
        nullable: true,
        inner_type_ref: Box::new(inner_named1),
        ref_location: location1.clone(),
    });
    let outer_list1 = ListTypeAnnotation {
        nullable: true,
        inner_type_ref: Box::new(inner_list1),
        ref_location: location1,
    };

    // Build [[Int]] for second annotation
    let inner_named2 = TypeAnnotation::Named(NamedTypeAnnotation {
        nullable: true,
        type_ref: NamedGraphQLTypeRef::new("Int", location2.clone()),
    });
    let inner_list2 = TypeAnnotation::List(ListTypeAnnotation {
        nullable: true,
        inner_type_ref: Box::new(inner_named2),
        ref_location: location2.clone(),
    });
    let outer_list2 = ListTypeAnnotation {
        nullable: true,
        inner_type_ref: Box::new(inner_list2),
        ref_location: location2,
    };

    assert!(outer_list1.is_equivalent_to(&outer_list2));
    assert!(outer_list2.is_equivalent_to(&outer_list1)); // Should be symmetric
}

#[test]
fn list_type_annotation_equivalence_complex_nested_non_null() {
    // Create two ListTypeAnnotation instances with:
    // - Complex nested structure ([[Int!]]!)
    // - Same nullability at all levels
    // - Different source locations
    // These should be considered equivalent

    let location1 = loc::SourceLocation::Schema;
    let location2 = loc::SourceLocation::GraphQLBuiltIn;

    // Build [[Int!]]! for first annotation
    let inner_named1 = TypeAnnotation::Named(NamedTypeAnnotation {
        nullable: false, // Int!
        type_ref: NamedGraphQLTypeRef::new("Int", location1.clone()),
    });
    let inner_list1 = TypeAnnotation::List(ListTypeAnnotation {
        nullable: true, // [Int!]
        inner_type_ref: Box::new(inner_named1),
        ref_location: location1.clone(),
    });
    let outer_list1 = ListTypeAnnotation {
        nullable: false, // [[Int!]]!
        inner_type_ref: Box::new(inner_list1),
        ref_location: location1,
    };

    // Build [[Int!]]! for second annotation
    let inner_named2 = TypeAnnotation::Named(NamedTypeAnnotation {
        nullable: false, // Int!
        type_ref: NamedGraphQLTypeRef::new("Int", location2.clone()),
    });
    let inner_list2 = TypeAnnotation::List(ListTypeAnnotation {
        nullable: true, // [Int!]
        inner_type_ref: Box::new(inner_named2),
        ref_location: location2.clone(),
    });
    let outer_list2 = ListTypeAnnotation {
        nullable: false, // [[Int!]]!
        inner_type_ref: Box::new(inner_list2),
        ref_location: location2,
    };

    assert!(outer_list1.is_equivalent_to(&outer_list2));
    assert!(outer_list2.is_equivalent_to(&outer_list1)); // Should be symmetric
}

#[test]
fn list_type_annotation_equivalence_different_nesting_depth() {
    // Create two ListTypeAnnotation instances with:
    // - Different nesting depths ([Int] vs [[Int]])
    // These should NOT be considered equivalent

    let location = loc::SourceLocation::Schema;

    // Build [Int]
    let inner_named = TypeAnnotation::Named(NamedTypeAnnotation {
        nullable: true,
        type_ref: NamedGraphQLTypeRef::new("Int", location.clone()),
    });
    let list1 = ListTypeAnnotation {
        nullable: true,
        inner_type_ref: Box::new(inner_named.clone()),
        ref_location: location.clone(),
    };

    // Build [[Int]]
    let inner_list = TypeAnnotation::List(ListTypeAnnotation {
        nullable: true,
        inner_type_ref: Box::new(inner_named),
        ref_location: location.clone(),
    });
    let list2 = ListTypeAnnotation {
        nullable: true,
        inner_type_ref: Box::new(inner_list),
        ref_location: location,
    };

    assert!(!list1.is_equivalent_to(&list2));
    assert!(!list2.is_equivalent_to(&list1)); // Should be symmetric
}

#[test]
fn type_annotation_equivalence_named_vs_list_mismatch() {
    // Test that Named and List variants are not equivalent
    let location = loc::SourceLocation::Schema;

    let named = TypeAnnotation::Named(NamedTypeAnnotation {
        nullable: true,
        type_ref: NamedGraphQLTypeRef::new("Int", location.clone()),
    });

    let inner = TypeAnnotation::Named(NamedTypeAnnotation {
        nullable: true,
        type_ref: NamedGraphQLTypeRef::new("Int", location.clone()),
    });

    let list = TypeAnnotation::List(ListTypeAnnotation {
        nullable: true,
        inner_type_ref: Box::new(inner),
        ref_location: location,
    });

    assert!(!named.is_equivalent_to(&list));
    assert!(!list.is_equivalent_to(&named)); // Should be symmetric
}

#[test]
fn type_annotation_equivalence_list_vs_named_mismatch() {
    // Test the opposite direction: [String] vs String
    let location = loc::SourceLocation::Schema;

    let inner = TypeAnnotation::Named(NamedTypeAnnotation {
        nullable: true,
        type_ref: NamedGraphQLTypeRef::new("String", location.clone()),
    });

    let list = TypeAnnotation::List(ListTypeAnnotation {
        nullable: true,
        inner_type_ref: Box::new(inner),
        ref_location: location.clone(),
    });

    let named = TypeAnnotation::Named(NamedTypeAnnotation {
        nullable: true,
        type_ref: NamedGraphQLTypeRef::new("String", location),
    });

    assert!(!list.is_equivalent_to(&named));
    assert!(!named.is_equivalent_to(&list)); // Should be symmetric
}
