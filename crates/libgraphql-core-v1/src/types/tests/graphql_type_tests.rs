use crate::names::TypeName;
use crate::span::Span;
use crate::types::EnumType;
use crate::types::FieldedTypeData;
use crate::types::GraphQLType;
use crate::types::GraphQLTypeKind;
use crate::types::InputObjectType;
use crate::types::InterfaceType;
use crate::types::ObjectType;
use crate::types::ScalarKind;
use crate::types::ScalarType;
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

    let enum_t = GraphQLType::Enum(Box::new(EnumType {
        description: None,
        directives: vec![],
        name: TypeName::new("Status"),
        span: Span::builtin(),
        values: IndexMap::new(),
    }));
    assert_eq!(enum_t.type_kind(), GraphQLTypeKind::Enum);

    let input_obj = GraphQLType::InputObject(Box::new(InputObjectType {
        description: None,
        directives: vec![],
        fields: IndexMap::new(),
        name: TypeName::new("Input"),
        span: Span::builtin(),
    }));
    assert_eq!(input_obj.type_kind(), GraphQLTypeKind::InputObject);

    let iface = GraphQLType::Interface(Box::new(InterfaceType(FieldedTypeData {
        description: None,
        directives: vec![],
        fields: IndexMap::new(),
        interfaces: vec![],
        name: TypeName::new("Node"),
        span: Span::builtin(),
    })));
    assert_eq!(iface.type_kind(), GraphQLTypeKind::Interface);

    let union_t = GraphQLType::Union(Box::new(UnionType {
        description: None,
        directives: vec![],
        members: vec![],
        name: TypeName::new("SearchResult"),
        span: Span::builtin(),
    }));
    assert_eq!(union_t.type_kind(), GraphQLTypeKind::Union);
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

    let enum_t = GraphQLType::Enum(Box::new(EnumType {
        description: None,
        directives: vec![],
        name: TypeName::new("Status"),
        span: Span::builtin(),
        values: IndexMap::new(),
    }));
    assert!(enum_t.as_enum().is_some());
    assert!(enum_t.as_object().is_none());

    let input_obj = GraphQLType::InputObject(Box::new(InputObjectType {
        description: None,
        directives: vec![],
        fields: IndexMap::new(),
        name: TypeName::new("Input"),
        span: Span::builtin(),
    }));
    assert!(input_obj.as_input_object().is_some());
    assert!(input_obj.as_scalar().is_none());

    let iface = GraphQLType::Interface(Box::new(InterfaceType(FieldedTypeData {
        description: None,
        directives: vec![],
        fields: IndexMap::new(),
        interfaces: vec![],
        name: TypeName::new("Node"),
        span: Span::builtin(),
    })));
    assert!(iface.as_interface().is_some());
    assert!(iface.as_object().is_none());

    let union_t = GraphQLType::Union(Box::new(UnionType {
        description: None,
        directives: vec![],
        members: vec![],
        name: TypeName::new("SearchResult"),
        span: Span::builtin(),
    }));
    assert!(union_t.as_union().is_some());
    assert!(union_t.as_scalar().is_none());
}

