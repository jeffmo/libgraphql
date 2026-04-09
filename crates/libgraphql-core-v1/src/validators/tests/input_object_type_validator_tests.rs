use crate::names::FieldName;
use crate::names::TypeName;
use crate::schema::TypeValidationErrorKind;
use crate::span::Span;
use crate::types::FieldedTypeData;
use crate::types::GraphQLType;
use crate::types::InputField;
use crate::types::InputObjectType;
use crate::types::InterfaceType;
use crate::types::ObjectType;
use crate::types::ScalarKind;
use crate::types::ScalarType;
use crate::types::TypeAnnotation;
use crate::types::UnionType;
use crate::validators::InputObjectTypeValidator;
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

fn int_scalar() -> GraphQLType {
    GraphQLType::Scalar(Box::new(ScalarType {
        description: None,
        directives: vec![],
        kind: ScalarKind::Int,
        name: TypeName::new("Int"),
        span: Span::builtin(),
    }))
}

fn make_input_field(
    name: &str,
    parent: &str,
    type_annot: TypeAnnotation,
) -> InputField {
    InputField {
        default_value: None,
        description: None,
        directives: vec![],
        name: FieldName::new(name),
        parent_type_name: TypeName::new(parent),
        span: Span::dummy(),
        type_annotation: type_annot,
    }
}

// Verifies that a valid input object with only input-type fields
// produces no errors.
// https://spec.graphql.org/September2025/#sec-Input-Objects
// Written by Claude Code, reviewed by a human.
#[test]
fn valid_input_object_type() {
    let mut fields = IndexMap::new();
    fields.insert(
        FieldName::new("name"),
        make_input_field(
            "name",
            "CreateUserInput",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    fields.insert(
        FieldName::new("age"),
        make_input_field(
            "age",
            "CreateUserInput",
            TypeAnnotation::named("Int", /* nullable = */ true),
        ),
    );
    let input_obj = InputObjectType {
        description: None,
        directives: vec![],
        fields,
        name: TypeName::new("CreateUserInput"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("String"), string_scalar());
    types_map.insert(TypeName::new("Int"), int_scalar());

    let validator = InputObjectTypeValidator::new(
        &input_obj,
        &types_map,
    );
    let errors = validator.validate();
    assert!(
        errors.is_empty(),
        "expected no errors, got: {errors:?}",
    );
}

// Verifies that an input field referencing an Object type
// (output-only) produces an InvalidInputFieldWithOutputType
// error. This is the critical fix over v0 which only checked
// as_object().is_some() -- v1 uses !is_input_type() to also
// reject Interface and Union types.
// https://spec.graphql.org/September2025/#sel-IAHhBXDDBFCAACEB4iG
// Written by Claude Code, reviewed by a human.
#[test]
fn input_field_with_object_type() {
    let result_obj = GraphQLType::Object(Box::new(
        ObjectType(FieldedTypeData {
            description: None,
            directives: vec![],
            fields: IndexMap::new(),
            interfaces: vec![],
            name: TypeName::new("User"),
            span: Span::dummy(),
        }),
    ));

    let mut fields = IndexMap::new();
    fields.insert(
        FieldName::new("user"),
        make_input_field(
            "user",
            "CreateInput",
            TypeAnnotation::named("User", /* nullable = */ true),
        ),
    );
    let input_obj = InputObjectType {
        description: None,
        directives: vec![],
        fields,
        name: TypeName::new("CreateInput"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("User"), result_obj);

    let validator = InputObjectTypeValidator::new(
        &input_obj,
        &types_map,
    );
    let errors = validator.validate();
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::InvalidInputFieldWithOutputType {
            field_name,
            invalid_type_name,
            parent_type_name,
        } if field_name == "user"
            && invalid_type_name == "User"
            && parent_type_name == "CreateInput"
    ));
}

// Verifies that an input field referencing an Interface type
// (output-only) produces an InvalidInputFieldWithOutputType
// error. This covers the bug in v0 where only Object types
// were rejected.
// https://spec.graphql.org/September2025/#sel-IAHhBXDDBFCAACEB4iG
// Written by Claude Code, reviewed by a human.
#[test]
fn input_field_with_interface_type() {
    let iface = GraphQLType::Interface(Box::new(
        InterfaceType(FieldedTypeData {
            description: None,
            directives: vec![],
            fields: IndexMap::new(),
            interfaces: vec![],
            name: TypeName::new("Node"),
            span: Span::dummy(),
        }),
    ));

    let mut fields = IndexMap::new();
    fields.insert(
        FieldName::new("node"),
        make_input_field(
            "node",
            "SearchInput",
            TypeAnnotation::named("Node", /* nullable = */ true),
        ),
    );
    let input_obj = InputObjectType {
        description: None,
        directives: vec![],
        fields,
        name: TypeName::new("SearchInput"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("Node"), iface);

    let validator = InputObjectTypeValidator::new(
        &input_obj,
        &types_map,
    );
    let errors = validator.validate();
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::InvalidInputFieldWithOutputType {
            field_name,
            invalid_type_name,
            parent_type_name,
        } if field_name == "node"
            && invalid_type_name == "Node"
            && parent_type_name == "SearchInput"
    ));
}

// Verifies that an input field referencing a Union type
// (output-only) produces an InvalidInputFieldWithOutputType
// error. This covers the bug in v0 where only Object types
// were rejected.
// https://spec.graphql.org/September2025/#sel-IAHhBXDDBFCAACEB4iG
// Written by Claude Code, reviewed by a human.
#[test]
fn input_field_with_union_type() {
    let union_type = GraphQLType::Union(Box::new(UnionType {
        description: None,
        directives: vec![],
        members: vec![],
        name: TypeName::new("SearchResult"),
        span: Span::dummy(),
    }));

    let mut fields = IndexMap::new();
    fields.insert(
        FieldName::new("result"),
        make_input_field(
            "result",
            "FilterInput",
            TypeAnnotation::named(
                "SearchResult",
                /* nullable = */ true,
            ),
        ),
    );
    let input_obj = InputObjectType {
        description: None,
        directives: vec![],
        fields,
        name: TypeName::new("FilterInput"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("SearchResult"), union_type);

    let validator = InputObjectTypeValidator::new(
        &input_obj,
        &types_map,
    );
    let errors = validator.validate();
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::InvalidInputFieldWithOutputType {
            field_name,
            invalid_type_name,
            parent_type_name,
        } if field_name == "result"
            && invalid_type_name == "SearchResult"
            && parent_type_name == "FilterInput"
    ));
}

// Verifies that an input field referencing an undefined type
// produces an UndefinedTypeName error.
// https://spec.graphql.org/September2025/#sec-Input-Objects
// Written by Claude Code, reviewed by a human.
#[test]
fn input_field_with_undefined_type() {
    let mut fields = IndexMap::new();
    fields.insert(
        FieldName::new("data"),
        make_input_field(
            "data",
            "MyInput",
            TypeAnnotation::named(
                "NonExistent",
                /* nullable = */ true,
            ),
        ),
    );
    let input_obj = InputObjectType {
        description: None,
        directives: vec![],
        fields,
        name: TypeName::new("MyInput"),
        span: Span::dummy(),
    };

    let types_map = IndexMap::new();
    let validator = InputObjectTypeValidator::new(
        &input_obj,
        &types_map,
    );
    let errors = validator.validate();
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::UndefinedTypeName {
            undefined_type_name,
        } if undefined_type_name == "NonExistent"
    ));
}

// Verifies that a direct non-nullable circular reference between
// two input objects produces a CircularInputFieldChain error.
// https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation
// Written by Claude Code, reviewed by a human.
#[test]
fn circular_non_nullable_input_field_chain() {
    // A! -> B! -> A! (circular)
    let mut a_fields = IndexMap::new();
    a_fields.insert(
        FieldName::new("b"),
        make_input_field(
            "b",
            "A",
            TypeAnnotation::named("B", /* nullable = */ false),
        ),
    );
    let a_type = InputObjectType {
        description: None,
        directives: vec![],
        fields: a_fields,
        name: TypeName::new("A"),
        span: Span::dummy(),
    };

    let mut b_fields = IndexMap::new();
    b_fields.insert(
        FieldName::new("a"),
        make_input_field(
            "a",
            "B",
            TypeAnnotation::named("A", /* nullable = */ false),
        ),
    );
    let b_type = InputObjectType {
        description: None,
        directives: vec![],
        fields: b_fields,
        name: TypeName::new("B"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(
        TypeName::new("A"),
        GraphQLType::InputObject(Box::new(a_type.clone())),
    );
    types_map.insert(
        TypeName::new("B"),
        GraphQLType::InputObject(Box::new(b_type)),
    );

    let validator = InputObjectTypeValidator::new(
        &a_type,
        &types_map,
    );
    let errors = validator.validate();

    let circular_errors: Vec<_> = errors
        .iter()
        .filter(|e| matches!(
            e.kind(),
            TypeValidationErrorKind::CircularInputFieldChain { .. }
        ))
        .collect();
    assert_eq!(circular_errors.len(), 1);
    assert!(matches!(
        circular_errors[0].kind(),
        TypeValidationErrorKind::CircularInputFieldChain {
            circular_field_path,
        } if !circular_field_path.is_empty()
    ));
}

// Verifies that a nullable field breaks a circular reference
// chain and produces no CircularInputFieldChain error.
// https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation
// Written by Claude Code, reviewed by a human.
#[test]
fn nullable_field_breaks_circular_chain() {
    // A -> B (nullable) -> A (non-null)
    // The nullable B field breaks the cycle.
    let mut a_fields = IndexMap::new();
    a_fields.insert(
        FieldName::new("b"),
        make_input_field(
            "b",
            "A",
            // nullable breaks cycle
            TypeAnnotation::named("B", /* nullable = */ true),
        ),
    );
    let a_type = InputObjectType {
        description: None,
        directives: vec![],
        fields: a_fields,
        name: TypeName::new("A"),
        span: Span::dummy(),
    };

    let mut b_fields = IndexMap::new();
    b_fields.insert(
        FieldName::new("a"),
        make_input_field(
            "a",
            "B",
            TypeAnnotation::named("A", /* nullable = */ false),
        ),
    );
    let b_type = InputObjectType {
        description: None,
        directives: vec![],
        fields: b_fields,
        name: TypeName::new("B"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(
        TypeName::new("A"),
        GraphQLType::InputObject(Box::new(a_type.clone())),
    );
    types_map.insert(
        TypeName::new("B"),
        GraphQLType::InputObject(Box::new(b_type)),
    );

    let validator = InputObjectTypeValidator::new(
        &a_type,
        &types_map,
    );
    let errors = validator.validate();
    assert!(
        errors.is_empty(),
        "expected no errors, got: {errors:?}",
    );
}
