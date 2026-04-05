use crate::located::Located;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::EnumType;
use crate::types::FieldDefinition;
use crate::types::FieldedTypeData;
use crate::types::GraphQLType;
use crate::types::GraphQLTypeKind;
use crate::types::InputObjectType;
use crate::types::InterfaceType;
use crate::types::ObjectType;
use crate::types::ScalarKind;
use crate::types::ScalarType;
use crate::types::TypeAnnotation;
use crate::types::UnionType;
use indexmap::IndexMap;

fn builtin_scalar(kind: ScalarKind, name: &str) -> GraphQLType {
    GraphQLType::Scalar(Box::new(ScalarType {
        description: None,
        directives: vec![],
        kind,
        name: TypeName::new(name),
        span: Span::builtin(),
    }))
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

// Verifies is_input_type/is_output_type per the spec's
// input/output type classification.
// https://spec.graphql.org/September2025/#sec-Input-and-Output-Types
// Written by Claude Code, reviewed by a human.
#[test]
fn input_output_classification() {
    // Scalar: both input and output
    let scalar = builtin_scalar(ScalarKind::String, "String");
    assert!(scalar.is_input_type());
    assert!(scalar.is_output_type());

    // Enum: both input and output
    let enum_t = GraphQLType::Enum(Box::new(EnumType {
        description: None,
        directives: vec![],
        name: TypeName::new("Status"),
        span: Span::builtin(),
        values: IndexMap::new(),
    }));
    assert!(enum_t.is_input_type());
    assert!(enum_t.is_output_type());

    // InputObject: input only
    let input_obj = GraphQLType::InputObject(Box::new(InputObjectType {
        description: None,
        directives: vec![],
        fields: IndexMap::new(),
        name: TypeName::new("CreateUserInput"),
        span: Span::builtin(),
    }));
    assert!(input_obj.is_input_type());
    assert!(!input_obj.is_output_type());

    // Object: output only
    let obj = sample_object("User");
    assert!(!obj.is_input_type());
    assert!(obj.is_output_type());

    // Interface: output only
    let iface = GraphQLType::Interface(Box::new(InterfaceType(FieldedTypeData {
        description: None,
        directives: vec![],
        fields: IndexMap::new(),
        interfaces: vec![],
        name: TypeName::new("Node"),
        span: Span::builtin(),
    })));
    assert!(!iface.is_input_type());
    assert!(iface.is_output_type());

    // Union: output only
    let union_t = GraphQLType::Union(Box::new(UnionType {
        description: None,
        directives: vec![],
        members: vec![],
        name: TypeName::new("SearchResult"),
        span: Span::builtin(),
    }));
    assert!(!union_t.is_input_type());
    assert!(union_t.is_output_type());
}

// Verifies type_kind() maps ScalarKind to correct
// GraphQLTypeKind (11-variant exhaustive matching).
// Written by Claude Code, reviewed by a human.
#[test]
fn type_kind_mapping() {
    assert_eq!(
        builtin_scalar(ScalarKind::Boolean, "Boolean").type_kind(),
        GraphQLTypeKind::Boolean,
    );
    assert_eq!(
        builtin_scalar(ScalarKind::Float, "Float").type_kind(),
        GraphQLTypeKind::Float,
    );
    assert_eq!(
        builtin_scalar(ScalarKind::ID, "ID").type_kind(),
        GraphQLTypeKind::ID,
    );
    assert_eq!(
        builtin_scalar(ScalarKind::Int, "Int").type_kind(),
        GraphQLTypeKind::Int,
    );
    assert_eq!(
        builtin_scalar(ScalarKind::String, "String").type_kind(),
        GraphQLTypeKind::String,
    );
    assert_eq!(
        builtin_scalar(ScalarKind::Custom, "DateTime").type_kind(),
        GraphQLTypeKind::Scalar,
    );
    assert_eq!(
        sample_object("User").type_kind(),
        GraphQLTypeKind::Object,
    );
}

// Verifies is_composite_type and is_leaf_type classification.
// Written by Claude Code, reviewed by a human.
#[test]
fn composite_and_leaf_classification() {
    let scalar = builtin_scalar(ScalarKind::Int, "Int");
    assert!(scalar.is_leaf_type());
    assert!(!scalar.is_composite_type());

    let obj = sample_object("User");
    assert!(obj.is_composite_type());
    assert!(!obj.is_leaf_type());
}

// Verifies is_builtin() only returns true for built-in scalars.
// Written by Claude Code, reviewed by a human.
#[test]
fn is_builtin() {
    assert!(builtin_scalar(ScalarKind::Int, "Int").is_builtin());
    assert!(!builtin_scalar(ScalarKind::Custom, "BigInt").is_builtin());
    assert!(!sample_object("User").is_builtin());
}

// Verifies typed downcasts return correct variants.
// Written by Claude Code, reviewed by a human.
#[test]
fn typed_downcasts() {
    let scalar = builtin_scalar(ScalarKind::String, "String");
    assert!(scalar.as_scalar().is_some());
    assert!(scalar.as_object().is_none());
    assert!(scalar.as_enum().is_none());

    let obj = sample_object("User");
    assert!(obj.as_object().is_some());
    assert!(obj.as_scalar().is_none());
}

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
    // Same nullability is subtype
    assert!(non_null.is_subtype_of(&types_map, &non_null));
    // Nullable is NOT subtype of non-null
    assert!(!nullable.is_subtype_of(&types_map, &non_null));
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

    // Define interface Node
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

    // Define object User implements Node
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

    // User is subtype of Node (implements it)
    assert!(user_annot.is_subtype_of(&types_map, &node_annot));
    // Node is NOT subtype of User
    assert!(!node_annot.is_subtype_of(&types_map, &user_annot));
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
                Located { value: TypeName::new("User"), span: Span::builtin() },
            ],
            name: TypeName::new("SearchResult"),
            span: Span::builtin(),
        })),
    );

    let user_annot = TypeAnnotation::named("User", false);
    let search_annot = TypeAnnotation::named("SearchResult", false);

    // User is subtype of SearchResult (member of union)
    assert!(user_annot.is_subtype_of(&types_map, &search_annot));
    // SearchResult is NOT subtype of User
    assert!(!search_annot.is_subtype_of(&types_map, &user_annot));
}
