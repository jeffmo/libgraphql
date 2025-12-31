use crate::loc;
use crate::types::NamedGraphQLTypeRef;
use crate::types::NamedTypeAnnotation;

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
