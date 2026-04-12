use crate::located::Located;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::schema::TypeValidationErrorKind;
use crate::span::Span;
use crate::types::FieldDefinition;
use crate::types::FieldedTypeData;
use crate::types::GraphQLType;
use crate::types::InterfaceType;
use crate::types::ObjectType;
use crate::types::ScalarKind;
use crate::types::ScalarType;
use crate::types::TypeAnnotation;
use crate::types::UnionType;
use crate::validators::UnionTypeValidator;
use indexmap::IndexMap;

fn string_scalar() -> GraphQLType {
    GraphQLType::Scalar(Box::new(ScalarType {
        description: None,
        directives: vec![],
        kind: ScalarKind::String,
        name: TypeName::new("String"),
        span: Span::builtin(),
    }))
}

fn make_object_type(name: &str) -> GraphQLType {
    let mut fields = IndexMap::new();
    fields.insert(FieldName::new("id"), FieldDefinition {
        description: None,
        directives: vec![],
        name: FieldName::new("id"),
        parameters: IndexMap::new(),
        parent_type_name: TypeName::new(name),
        span: Span::dummy(),
        type_annotation: TypeAnnotation::named(
            "String",
            /* nullable = */ false,
        ),
    });
    GraphQLType::Object(Box::new(ObjectType(FieldedTypeData {
        description: None,
        directives: vec![],
        fields,
        interfaces: vec![],
        name: TypeName::new(name),
        span: Span::dummy(),
    })))
}

fn located_type_name(name: &str) -> Located<TypeName> {
    Located {
        value: TypeName::new(name),
        span: Span::dummy(),
    }
}

// Verifies that a valid union with all object members produces
// no errors.
// https://spec.graphql.org/September2025/#sel-HAHdfFDABABlG3ib
// Written by Claude Code, reviewed by a human.
#[test]
fn valid_union_type() {
    let union_type = UnionType {
        description: None,
        directives: vec![],
        members: vec![
            located_type_name("Dog"),
            located_type_name("Cat"),
        ],
        name: TypeName::new("Pet"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("String"), string_scalar());
    types_map.insert(TypeName::new("Dog"), make_object_type("Dog"));
    types_map.insert(TypeName::new("Cat"), make_object_type("Cat"));

    let validator = UnionTypeValidator::new(&union_type, &types_map);
    let errors = validator.validate();
    assert!(
        errors.is_empty(),
        "expected no errors, got: {errors:?}",
    );
}

// Verifies that a union referencing an undefined member type
// produces an UndefinedTypeName error.
// https://spec.graphql.org/September2025/#sel-HAHdfFDABABlG3ib
// Written by Claude Code, reviewed by a human.
#[test]
fn union_with_undefined_member() {
    let union_type = UnionType {
        description: None,
        directives: vec![],
        members: vec![located_type_name("Ghost")],
        name: TypeName::new("Pet"),
        span: Span::dummy(),
    };

    let types_map = IndexMap::new();
    let validator = UnionTypeValidator::new(&union_type, &types_map);
    let errors = validator.validate();
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::UndefinedTypeName {
            undefined_type_name,
        } if undefined_type_name == "Ghost"
    ));
}

// Verifies that a union member that is not an object type
// (e.g. an interface) produces an InvalidUnionMemberTypeKind
// error.
// https://spec.graphql.org/September2025/#sel-HAHdfFDABABlG3ib
// Written by Claude Code, reviewed by a human.
#[test]
fn union_with_non_object_member() {
    let iface = InterfaceType(FieldedTypeData {
        description: None,
        directives: vec![],
        fields: IndexMap::new(),
        interfaces: vec![],
        name: TypeName::new("Node"),
        span: Span::dummy(),
    });

    let union_type = UnionType {
        description: None,
        directives: vec![],
        members: vec![located_type_name("Node")],
        name: TypeName::new("Result"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(
        TypeName::new("Node"),
        GraphQLType::Interface(Box::new(iface)),
    );

    let validator = UnionTypeValidator::new(&union_type, &types_map);
    let errors = validator.validate();
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::InvalidUnionMemberTypeKind {
            member_name,
            union_type_name,
        } if member_name == "Node"
            && union_type_name == "Result"
    ));
}

// Verifies that a union with a scalar member produces an
// InvalidUnionMemberTypeKind error.
// https://spec.graphql.org/September2025/#sel-HAHdfFDABABlG3ib
// Written by Claude Code, reviewed by a human.
#[test]
fn union_with_scalar_member() {
    let union_type = UnionType {
        description: None,
        directives: vec![],
        members: vec![located_type_name("String")],
        name: TypeName::new("SearchResult"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("String"), string_scalar());

    let validator = UnionTypeValidator::new(&union_type, &types_map);
    let errors = validator.validate();
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::InvalidUnionMemberTypeKind {
            member_name,
            union_type_name,
        } if member_name == "String"
            && union_type_name == "SearchResult"
    ));
}
