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
use crate::types::ParameterDefinition;
use crate::types::ScalarKind;
use crate::types::ScalarType;
use crate::types::TypeAnnotation;
use crate::validators::ObjectOrInterfaceTypeValidator;
use indexmap::IndexMap;
use std::collections::HashSet;

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

fn boolean_scalar() -> GraphQLType {
    GraphQLType::Scalar(Box::new(ScalarType {
        description: None,
        directives: vec![],
        kind: ScalarKind::Boolean,
        name: TypeName::new("Boolean"),
        span: Span::builtin(),
    }))
}

fn make_field(
    name: &str,
    parent: &str,
    type_annot: TypeAnnotation,
) -> FieldDefinition {
    FieldDefinition {
        description: None,
        directives: vec![],
        name: FieldName::new(name),
        parameters: IndexMap::new(),
        parent_type_name: TypeName::new(parent),
        span: Span::dummy(),
        type_annotation: type_annot,
    }
}

fn make_field_with_params(
    name: &str,
    parent: &str,
    type_annot: TypeAnnotation,
    params: IndexMap<FieldName, ParameterDefinition>,
) -> FieldDefinition {
    FieldDefinition {
        description: None,
        directives: vec![],
        name: FieldName::new(name),
        parameters: params,
        parent_type_name: TypeName::new(parent),
        span: Span::dummy(),
        type_annotation: type_annot,
    }
}

fn make_param(
    name: &str,
    type_annot: TypeAnnotation,
) -> ParameterDefinition {
    ParameterDefinition {
        default_value: None,
        description: None,
        directives: vec![],
        name: FieldName::new(name),
        span: Span::dummy(),
        type_annotation: type_annot,
    }
}

fn make_interface(
    name: &str,
    fields: IndexMap<FieldName, FieldDefinition>,
    interfaces: Vec<Located<TypeName>>,
) -> InterfaceType {
    InterfaceType(FieldedTypeData {
        description: None,
        directives: vec![],
        fields,
        interfaces,
        name: TypeName::new(name),
        span: Span::dummy(),
    })
}

fn make_object(
    name: &str,
    fields: IndexMap<FieldName, FieldDefinition>,
    interfaces: Vec<Located<TypeName>>,
) -> ObjectType {
    ObjectType(FieldedTypeData {
        description: None,
        directives: vec![],
        fields,
        interfaces,
        name: TypeName::new(name),
        span: Span::dummy(),
    })
}

fn located_type_name(name: &str) -> Located<TypeName> {
    Located {
        value: TypeName::new(name),
        span: Span::dummy(),
    }
}

// Verifies that an object type correctly implementing an
// interface produces no validation errors.
// https://spec.graphql.org/September2025/#IsValidImplementation()
// Written by Claude Code, reviewed by a human.
#[test]
fn valid_object_implementing_interface() {
    let mut iface_fields = IndexMap::new();
    iface_fields.insert(
        FieldName::new("name"),
        make_field(
            "name",
            "Node",
            TypeAnnotation::named("String", /* nullable = */ true),
        ),
    );
    let iface = make_interface("Node", iface_fields, vec![]);

    let mut obj_fields = IndexMap::new();
    obj_fields.insert(
        FieldName::new("name"),
        make_field(
            "name",
            "User",
            TypeAnnotation::named("String", /* nullable = */ true),
        ),
    );
    let obj = make_object(
        "User",
        obj_fields,
        vec![located_type_name("Node")],
    );

    let mut types_map = IndexMap::new();
    types_map.insert(
        TypeName::new("String"),
        string_scalar(),
    );
    types_map.insert(
        TypeName::new("Node"),
        GraphQLType::Interface(Box::new(iface)),
    );
    types_map.insert(
        TypeName::new("User"),
        GraphQLType::Object(Box::new(obj.clone())),
    );

    let validator = ObjectOrInterfaceTypeValidator::new(
        &obj,
        &types_map,
    );
    let mut verified = HashSet::new();
    let errors = validator.validate(&mut verified);
    assert!(
        errors.is_empty(),
        "expected no errors, got: {errors:?}",
    );
}

// Verifies that implementing an undefined interface produces
// an ImplementsUndefinedInterface error whose span points at
// the interface reference (not the whole type definition).
// https://spec.graphql.org/September2025/#IsValidImplementation()
// Written by Claude Code, reviewed by a human.
#[test]
fn implements_undefined_interface() {
    let mut obj_fields = IndexMap::new();
    obj_fields.insert(
        FieldName::new("id"),
        make_field(
            "id",
            "User",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    let obj = make_object(
        "User",
        obj_fields,
        vec![located_type_name("NonExistent")],
    );

    let mut types_map = IndexMap::new();
    types_map.insert(
        TypeName::new("String"),
        string_scalar(),
    );

    let validator = ObjectOrInterfaceTypeValidator::new(
        &obj,
        &types_map,
    );
    let mut verified = HashSet::new();
    let errors = validator.validate(&mut verified);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::ImplementsUndefinedInterface {
            type_name,
            undefined_interface_name,
        } if type_name == "User"
            && undefined_interface_name == "NonExistent"
    ));
}

// Verifies that implementing a non-interface type (e.g. a
// scalar) produces an ImplementsNonInterfaceType error.
// https://spec.graphql.org/September2025/#IsValidImplementation()
// Written by Claude Code, reviewed by a human.
#[test]
fn implements_non_interface_type() {
    let mut obj_fields = IndexMap::new();
    obj_fields.insert(
        FieldName::new("id"),
        make_field(
            "id",
            "User",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    let obj = make_object(
        "User",
        obj_fields,
        vec![located_type_name("String")],
    );

    let mut types_map = IndexMap::new();
    types_map.insert(
        TypeName::new("String"),
        string_scalar(),
    );

    let validator = ObjectOrInterfaceTypeValidator::new(
        &obj,
        &types_map,
    );
    let mut verified = HashSet::new();
    let errors = validator.validate(&mut verified);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::ImplementsNonInterfaceType {
            type_name,
            non_interface_type_name,
        } if type_name == "User"
            && non_interface_type_name == "String"
    ));
}

// Verifies that a missing interface field produces a
// MissingInterfaceSpecifiedField error.
// https://spec.graphql.org/September2025/#IsValidImplementation()
// Written by Claude Code, reviewed by a human.
#[test]
fn missing_interface_specified_field() {
    let mut iface_fields = IndexMap::new();
    iface_fields.insert(
        FieldName::new("name"),
        make_field(
            "name",
            "Node",
            TypeAnnotation::named("String", /* nullable = */ true),
        ),
    );
    iface_fields.insert(
        FieldName::new("id"),
        make_field(
            "id",
            "Node",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    let iface = make_interface("Node", iface_fields, vec![]);

    // Object only defines "name", missing "id"
    let mut obj_fields = IndexMap::new();
    obj_fields.insert(
        FieldName::new("name"),
        make_field(
            "name",
            "User",
            TypeAnnotation::named("String", /* nullable = */ true),
        ),
    );
    let obj = make_object(
        "User",
        obj_fields,
        vec![located_type_name("Node")],
    );

    let mut types_map = IndexMap::new();
    types_map.insert(
        TypeName::new("String"),
        string_scalar(),
    );
    types_map.insert(
        TypeName::new("Node"),
        GraphQLType::Interface(Box::new(iface)),
    );

    let validator = ObjectOrInterfaceTypeValidator::new(
        &obj,
        &types_map,
    );
    let mut verified = HashSet::new();
    let errors = validator.validate(&mut verified);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::MissingInterfaceSpecifiedField {
            field_name,
            interface_name,
            type_name,
        } if field_name == "id"
            && interface_name == "Node"
            && type_name == "User"
    ));
}

// Verifies that a wrong parameter type on an implementing
// field produces an InvalidInterfaceSpecifiedFieldParameterType
// error.
// https://spec.graphql.org/September2025/#IsValidImplementation()
// Written by Claude Code, reviewed by a human.
#[test]
fn invalid_interface_field_parameter_type() {
    let mut iface_params = IndexMap::new();
    iface_params.insert(
        FieldName::new("first"),
        make_param(
            "first",
            TypeAnnotation::named("Int", /* nullable = */ true),
        ),
    );
    let mut iface_fields = IndexMap::new();
    iface_fields.insert(
        FieldName::new("items"),
        make_field_with_params(
            "items",
            "Node",
            TypeAnnotation::named("String", /* nullable = */ true),
            iface_params,
        ),
    );
    let iface = make_interface("Node", iface_fields, vec![]);

    // Object defines "first" param as String instead of Int
    let mut obj_params = IndexMap::new();
    obj_params.insert(
        FieldName::new("first"),
        make_param(
            "first",
            TypeAnnotation::named("String", /* nullable = */ true),
        ),
    );
    let mut obj_fields = IndexMap::new();
    obj_fields.insert(
        FieldName::new("items"),
        make_field_with_params(
            "items",
            "User",
            TypeAnnotation::named("String", /* nullable = */ true),
            obj_params,
        ),
    );
    let obj = make_object(
        "User",
        obj_fields,
        vec![located_type_name("Node")],
    );

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("String"), string_scalar());
    types_map.insert(TypeName::new("Int"), int_scalar());
    types_map.insert(
        TypeName::new("Node"),
        GraphQLType::Interface(Box::new(iface)),
    );

    let validator = ObjectOrInterfaceTypeValidator::new(
        &obj,
        &types_map,
    );
    let mut verified = HashSet::new();
    let errors = validator.validate(&mut verified);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::InvalidInterfaceSpecifiedFieldParameterType {
            actual_type,
            expected_type,
            field_name,
            interface_name,
            parameter_name,
            type_name,
        } if actual_type == "String"
            && expected_type == "Int"
            && field_name == "items"
            && interface_name == "Node"
            && parameter_name == "first"
            && type_name == "User"
    ));
    // The error should include a note pointing at the
    // interface parameter definition.
    assert!(
        !errors[0].notes().is_empty(),
        "expected notes on InvalidInterfaceSpecifiedFieldParameterType",
    );
}

// Verifies that a missing parameter on an implementing field
// produces a MissingInterfaceSpecifiedFieldParameter error.
// https://spec.graphql.org/September2025/#IsValidImplementation()
// Written by Claude Code, reviewed by a human.
#[test]
fn missing_interface_specified_field_parameter() {
    let mut iface_params = IndexMap::new();
    iface_params.insert(
        FieldName::new("first"),
        make_param(
            "first",
            TypeAnnotation::named("Int", /* nullable = */ true),
        ),
    );
    let mut iface_fields = IndexMap::new();
    iface_fields.insert(
        FieldName::new("items"),
        make_field_with_params(
            "items",
            "Node",
            TypeAnnotation::named("String", /* nullable = */ true),
            iface_params,
        ),
    );
    let iface = make_interface("Node", iface_fields, vec![]);

    // Object defines "items" field but without the "first" param
    let mut obj_fields = IndexMap::new();
    obj_fields.insert(
        FieldName::new("items"),
        make_field(
            "items",
            "User",
            TypeAnnotation::named("String", /* nullable = */ true),
        ),
    );
    let obj = make_object(
        "User",
        obj_fields,
        vec![located_type_name("Node")],
    );

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("String"), string_scalar());
    types_map.insert(TypeName::new("Int"), int_scalar());
    types_map.insert(
        TypeName::new("Node"),
        GraphQLType::Interface(Box::new(iface)),
    );

    let validator = ObjectOrInterfaceTypeValidator::new(
        &obj,
        &types_map,
    );
    let mut verified = HashSet::new();
    let errors = validator.validate(&mut verified);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::MissingInterfaceSpecifiedFieldParameter {
            field_name,
            interface_name,
            missing_parameter_name,
            type_name,
        } if field_name == "items"
            && interface_name == "Node"
            && missing_parameter_name == "first"
            && type_name == "User"
    ));
}

// Verifies that a required additional parameter (not in the
// interface) on the implementing field produces an
// InvalidRequiredAdditionalParameterOnInterfaceSpecifiedField
// error.
// https://spec.graphql.org/September2025/#IsValidImplementation()
// Written by Claude Code, reviewed by a human.
#[test]
fn required_additional_parameter_on_interface_field() {
    let mut iface_fields = IndexMap::new();
    iface_fields.insert(
        FieldName::new("name"),
        make_field(
            "name",
            "Node",
            TypeAnnotation::named("String", /* nullable = */ true),
        ),
    );
    let iface = make_interface("Node", iface_fields, vec![]);

    // Object adds a required (non-null, no default) extra param
    let mut obj_params = IndexMap::new();
    obj_params.insert(
        FieldName::new("extra"),
        make_param(
            "extra",
            TypeAnnotation::named("Boolean", /* nullable = */ false),
        ),
    );
    let mut obj_fields = IndexMap::new();
    obj_fields.insert(
        FieldName::new("name"),
        make_field_with_params(
            "name",
            "User",
            TypeAnnotation::named("String", /* nullable = */ true),
            obj_params,
        ),
    );
    let obj = make_object(
        "User",
        obj_fields,
        vec![located_type_name("Node")],
    );

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("String"), string_scalar());
    types_map.insert(TypeName::new("Boolean"), boolean_scalar());
    types_map.insert(
        TypeName::new("Node"),
        GraphQLType::Interface(Box::new(iface)),
    );

    let validator = ObjectOrInterfaceTypeValidator::new(
        &obj,
        &types_map,
    );
    let mut verified = HashSet::new();
    let errors = validator.validate(&mut verified);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::InvalidRequiredAdditionalParameterOnInterfaceSpecifiedField {
            field_name,
            interface_name,
            parameter_name,
            type_name,
        } if field_name == "name"
            && interface_name == "Node"
            && parameter_name == "extra"
            && type_name == "User"
    ));
    // The error should include a note pointing at the
    // interface field definition.
    assert!(
        !errors[0].notes().is_empty(),
        "expected notes on \
        InvalidRequiredAdditionalParameterOnInterfaceSpecifiedField",
    );
}

// Verifies that a non-covariant return type on an implementing
// field produces an InvalidInterfaceSpecifiedFieldType error.
// https://spec.graphql.org/September2025/#IsValidImplementation()
// Written by Claude Code, reviewed by a human.
#[test]
fn invalid_interface_specified_field_type() {
    let mut iface_fields = IndexMap::new();
    iface_fields.insert(
        FieldName::new("name"),
        make_field(
            "name",
            "Node",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    let iface = make_interface("Node", iface_fields, vec![]);

    // Object returns nullable String where interface requires
    // non-null String (widening nullability is not covariant)
    let mut obj_fields = IndexMap::new();
    obj_fields.insert(
        FieldName::new("name"),
        make_field(
            "name",
            "User",
            TypeAnnotation::named("String", /* nullable = */ true),
        ),
    );
    let obj = make_object(
        "User",
        obj_fields,
        vec![located_type_name("Node")],
    );

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("String"), string_scalar());
    types_map.insert(
        TypeName::new("Node"),
        GraphQLType::Interface(Box::new(iface)),
    );

    let validator = ObjectOrInterfaceTypeValidator::new(
        &obj,
        &types_map,
    );
    let mut verified = HashSet::new();
    let errors = validator.validate(&mut verified);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::InvalidInterfaceSpecifiedFieldType {
            actual_type,
            expected_type,
            field_name,
            interface_name,
            type_name,
        } if actual_type == "String"
            && expected_type == "String!"
            && field_name == "name"
            && interface_name == "Node"
            && type_name == "User"
    ));
    // The error should include a note pointing at the
    // interface field's return type declaration.
    assert!(
        !errors[0].notes().is_empty(),
        "expected notes on InvalidInterfaceSpecifiedFieldType",
    );
}

// Verifies that a missing transitive interface implementation
// produces a MissingRecursiveInterfaceImplementation error.
// https://spec.graphql.org/September2025/#IsValidImplementation()
// Written by Claude Code, reviewed by a human.
#[test]
fn missing_recursive_interface_implementation() {
    // Base interface
    let mut base_fields = IndexMap::new();
    base_fields.insert(
        FieldName::new("id"),
        make_field(
            "id",
            "Base",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    let base_iface = make_interface("Base", base_fields, vec![]);

    // Middle interface implements Base
    let mut mid_fields = IndexMap::new();
    mid_fields.insert(
        FieldName::new("id"),
        make_field(
            "id",
            "Middle",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    mid_fields.insert(
        FieldName::new("name"),
        make_field(
            "name",
            "Middle",
            TypeAnnotation::named("String", /* nullable = */ true),
        ),
    );
    let mid_iface = make_interface(
        "Middle",
        mid_fields,
        vec![located_type_name("Base")],
    );

    // Object implements Middle but NOT Base
    let mut obj_fields = IndexMap::new();
    obj_fields.insert(
        FieldName::new("id"),
        make_field(
            "id",
            "User",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    obj_fields.insert(
        FieldName::new("name"),
        make_field(
            "name",
            "User",
            TypeAnnotation::named("String", /* nullable = */ true),
        ),
    );
    let obj = make_object(
        "User",
        obj_fields,
        vec![located_type_name("Middle")],
    );

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("String"), string_scalar());
    types_map.insert(
        TypeName::new("Base"),
        GraphQLType::Interface(Box::new(base_iface)),
    );
    types_map.insert(
        TypeName::new("Middle"),
        GraphQLType::Interface(Box::new(mid_iface)),
    );

    let validator = ObjectOrInterfaceTypeValidator::new(
        &obj,
        &types_map,
    );
    let mut verified = HashSet::new();
    let errors = validator.validate(&mut verified);

    let missing_recursive_errors: Vec<_> = errors
        .iter()
        .filter(|e| matches!(
            e.kind(),
            TypeValidationErrorKind::MissingRecursiveInterfaceImplementation { .. }
        ))
        .collect();
    assert_eq!(missing_recursive_errors.len(), 1);
    assert!(matches!(
        missing_recursive_errors[0].kind(),
        TypeValidationErrorKind::MissingRecursiveInterfaceImplementation {
            inheritance_path,
            missing_recursive_interface_name,
            type_name,
        } if missing_recursive_interface_name == "Base"
            && type_name == "User"
            && !inheritance_path.is_empty()
            && inheritance_path.contains(&"Middle".to_string())
    ));

    // The Display output must not contain a dangling
    // "implements ," (nothing between "implements" and the
    // comma), which would indicate the inheritance_path
    // vector had been left empty at the point of error.
    let msg = missing_recursive_errors[0].to_string();
    assert!(
        !msg.contains("implements ,"),
        "error message should not contain dangling 'implements ,' \
        (indicates empty inheritance_path): {msg}",
    );
    assert!(
        msg.contains("`Middle`"),
        "error message should mention the transitive interface \
        `Middle`: {msg}",
    );
}

// Verifies that a field referencing an undefined return type
// produces an UndefinedTypeName error.
// https://spec.graphql.org/September2025/#sec-Objects
// Written by Claude Code, reviewed by a human.
#[test]
fn object_field_with_undefined_type() {
    let mut obj_fields = IndexMap::new();
    obj_fields.insert(
        FieldName::new("data"),
        make_field(
            "data",
            "Query",
            TypeAnnotation::named(
                "NonExistent",
                /* nullable = */ true,
            ),
        ),
    );
    let obj = make_object("Query", obj_fields, vec![]);

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("String"), string_scalar());

    let validator = ObjectOrInterfaceTypeValidator::new(
        &obj,
        &types_map,
    );
    let mut verified = HashSet::new();
    let errors = validator.validate(&mut verified);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::UndefinedTypeName {
            undefined_type_name,
        } if undefined_type_name == "NonExistent"
    ));
}

// Verifies that a field using an input-only type (InputObject)
// as a return type produces an InvalidOutputFieldWithInputType
// error.
// https://spec.graphql.org/September2025/#sel-JAHZhCFDBFABLBgB_pM
// Written by Claude Code, reviewed by a human.
#[test]
fn object_field_with_input_only_type() {
    use crate::types::InputObjectType;

    let input_obj = InputObjectType {
        description: None,
        directives: vec![],
        fields: IndexMap::new(),
        name: TypeName::new("CreateUserInput"),
        span: Span::dummy(),
    };

    let mut obj_fields = IndexMap::new();
    obj_fields.insert(
        FieldName::new("data"),
        make_field(
            "data",
            "Query",
            TypeAnnotation::named(
                "CreateUserInput",
                /* nullable = */ true,
            ),
        ),
    );
    let obj = make_object("Query", obj_fields, vec![]);

    let mut types_map = IndexMap::new();
    types_map.insert(
        TypeName::new("CreateUserInput"),
        GraphQLType::InputObject(Box::new(input_obj)),
    );

    let validator = ObjectOrInterfaceTypeValidator::new(
        &obj,
        &types_map,
    );
    let mut verified = HashSet::new();
    let errors = validator.validate(&mut verified);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::InvalidOutputFieldWithInputType {
            field_name,
            input_type_name,
            parent_type_name,
        } if field_name == "data"
            && input_type_name == "CreateUserInput"
            && parent_type_name == "Query"
    ));
}

// Verifies that a field parameter using an output-only type
// (Object) produces an InvalidParameterWithOutputOnlyType
// error.
// https://spec.graphql.org/September2025/#sel-KAHZhCFDBHBDCAACEB6yD
// Written by Claude Code, reviewed by a human.
#[test]
fn object_field_param_with_output_only_type() {
    let result_obj = ObjectType(FieldedTypeData {
        description: None,
        directives: vec![],
        fields: IndexMap::new(),
        interfaces: vec![],
        name: TypeName::new("Result"),
        span: Span::dummy(),
    });

    let mut obj_params = IndexMap::new();
    obj_params.insert(
        FieldName::new("input"),
        make_param(
            "input",
            TypeAnnotation::named("Result", /* nullable = */ true),
        ),
    );
    let mut obj_fields = IndexMap::new();
    obj_fields.insert(
        FieldName::new("doSomething"),
        make_field_with_params(
            "doSomething",
            "Query",
            TypeAnnotation::named("String", /* nullable = */ true),
            obj_params,
        ),
    );
    let obj = make_object("Query", obj_fields, vec![]);

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("String"), string_scalar());
    types_map.insert(
        TypeName::new("Result"),
        GraphQLType::Object(Box::new(result_obj)),
    );

    let validator = ObjectOrInterfaceTypeValidator::new(
        &obj,
        &types_map,
    );
    let mut verified = HashSet::new();
    let errors = validator.validate(&mut verified);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::InvalidParameterWithOutputOnlyType {
            field_name,
            invalid_type_name,
            parameter_name,
            type_name,
        } if field_name == "doSomething"
            && invalid_type_name == "Result"
            && parameter_name == "input"
            && type_name == "Query"
    ));
}

// Verifies that an interface type implementing another interface
// is validated correctly. Per the September 2025 spec, interfaces
// can implement other interfaces, and the same IsValidImplementation
// rules apply. This test validates that the validator works with
// InterfaceType as the type under validation (not just ObjectType).
// https://spec.graphql.org/September2025/#IsValidImplementation()
// Written by Claude Code, reviewed by a human.
#[test]
fn interface_implementing_interface_validates() {
    // interface Node { id: ID! }
    let mut node_fields = IndexMap::new();
    node_fields.insert(
        FieldName::new("id"),
        make_field(
            "id",
            "Node",
            TypeAnnotation::named("ID", /* nullable = */ false),
        ),
    );
    let node_iface = make_interface("Node", node_fields, vec![]);

    // interface Resource implements Node { id: ID!, url: String! }
    let mut resource_fields = IndexMap::new();
    resource_fields.insert(
        FieldName::new("id"),
        make_field(
            "id",
            "Resource",
            TypeAnnotation::named("ID", /* nullable = */ false),
        ),
    );
    resource_fields.insert(
        FieldName::new("url"),
        make_field(
            "url",
            "Resource",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    let resource_iface = make_interface(
        "Resource",
        resource_fields,
        vec![located_type_name("Node")],
    );

    let id_scalar = GraphQLType::Scalar(Box::new(ScalarType {
        description: None,
        directives: vec![],
        kind: ScalarKind::ID,
        name: TypeName::new("ID"),
        span: Span::builtin(),
    }));

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("ID"), id_scalar);
    types_map.insert(TypeName::new("String"), string_scalar());
    types_map.insert(
        TypeName::new("Node"),
        GraphQLType::Interface(Box::new(node_iface)),
    );
    types_map.insert(
        TypeName::new("Resource"),
        GraphQLType::Interface(Box::new(resource_iface.clone())),
    );

    let validator = ObjectOrInterfaceTypeValidator::new(
        &resource_iface,
        &types_map,
    );
    let mut verified = HashSet::new();
    let errors = validator.validate(&mut verified);
    assert!(
        errors.is_empty(),
        "expected no errors, got: {errors:?}",
    );
}

// Regression test for a bug where the recursive child
// validator used the current interface's implemented interfaces
// instead of the implementing type's interfaces.
//
// Setup:
//   interface B { id: ID! }
//   interface A implements B { id: ID! }
//   type C implements A & B { id: ID! }
//
// When validating C's implementation of A, the validator
// recursively checks that C also implements everything A
// implements (i.e. B). The recursive check must look at C's
// declared interfaces (which includes B), NOT A's interfaces.
// Before the fix, the child validator was initialized with A's
// interface set, so it would produce a false
// MissingRecursiveInterfaceImplementation error for C even
// though C explicitly declares `implements A & B`.
//
// https://spec.graphql.org/September2025/#IsValidImplementation()
// Written by Claude Code, reviewed by a human.
#[test]
fn recursive_validation_uses_implementing_types_interfaces() {
    let id_scalar = GraphQLType::Scalar(Box::new(ScalarType {
        description: None,
        directives: vec![],
        kind: ScalarKind::ID,
        name: TypeName::new("ID"),
        span: Span::builtin(),
    }));

    // interface B { id: ID! }
    let mut b_fields = IndexMap::new();
    b_fields.insert(
        FieldName::new("id"),
        make_field(
            "id",
            "B",
            TypeAnnotation::named("ID", /* nullable = */ false),
        ),
    );
    let b_iface = make_interface("B", b_fields, vec![]);

    // interface A implements B { id: ID! }
    let mut a_fields = IndexMap::new();
    a_fields.insert(
        FieldName::new("id"),
        make_field(
            "id",
            "A",
            TypeAnnotation::named("ID", /* nullable = */ false),
        ),
    );
    let a_iface = make_interface(
        "A",
        a_fields,
        vec![located_type_name("B")],
    );

    // type C implements A & B { id: ID! }
    let mut c_fields = IndexMap::new();
    c_fields.insert(
        FieldName::new("id"),
        make_field(
            "id",
            "C",
            TypeAnnotation::named("ID", /* nullable = */ false),
        ),
    );
    let c_obj = make_object(
        "C",
        c_fields,
        vec![
            located_type_name("A"),
            located_type_name("B"),
        ],
    );

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("ID"), id_scalar);
    types_map.insert(
        TypeName::new("A"),
        GraphQLType::Interface(Box::new(a_iface)),
    );
    types_map.insert(
        TypeName::new("B"),
        GraphQLType::Interface(Box::new(b_iface)),
    );
    types_map.insert(
        TypeName::new("C"),
        GraphQLType::Object(Box::new(c_obj.clone())),
    );

    let validator = ObjectOrInterfaceTypeValidator::new(
        &c_obj,
        &types_map,
    );
    let mut verified = HashSet::new();
    let errors = validator.validate(&mut verified);

    // C correctly declares both A and B, so there should be
    // no errors. Before the fix, the child validator would
    // have used A's interface set (just {B}) when checking C's
    // recursive implementations of A's parents, which would
    // produce a spurious MissingRecursiveInterfaceImplementation
    // error because A's interface set was being compared against
    // itself rather than C's.
    assert!(
        errors.is_empty(),
        "expected no errors (C correctly implements A & B), \
        got: {errors:?}",
    );
}

// Regression test: verifies the Display output of
// MissingRecursiveInterfaceImplementation contains the current
// interface's name and does NOT contain a dangling
// "implements ," (with nothing between "implements" and the
// comma). Prior to the fix, the validator passed
// `self.inheritance_path` directly to the error without
// including the current `iface_name`, which meant a top-level
// call (with an empty path) produced an error message like
// "`User` implements , therefore ...".
// https://spec.graphql.org/September2025/#IsValidImplementation()
// Written by Claude Code, reviewed by a human.
#[test]
fn missing_recursive_interface_display_includes_path() {
    // Node interface (the transitively-required ancestor).
    let mut node_fields = IndexMap::new();
    node_fields.insert(
        FieldName::new("id"),
        make_field(
            "id",
            "Node",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    let node_iface = make_interface("Node", node_fields, vec![]);

    // Resource interface implements Node.
    let mut resource_fields = IndexMap::new();
    resource_fields.insert(
        FieldName::new("id"),
        make_field(
            "id",
            "Resource",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    let resource_iface = make_interface(
        "Resource",
        resource_fields,
        vec![located_type_name("Node")],
    );

    // User object implements Resource but NOT Node.
    let mut user_fields = IndexMap::new();
    user_fields.insert(
        FieldName::new("id"),
        make_field(
            "id",
            "User",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    let user_obj = make_object(
        "User",
        user_fields,
        vec![located_type_name("Resource")],
    );

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("String"), string_scalar());
    types_map.insert(
        TypeName::new("Node"),
        GraphQLType::Interface(Box::new(node_iface)),
    );
    types_map.insert(
        TypeName::new("Resource"),
        GraphQLType::Interface(Box::new(resource_iface)),
    );

    let validator = ObjectOrInterfaceTypeValidator::new(
        &user_obj,
        &types_map,
    );
    let mut verified = HashSet::new();
    let errors = validator.validate(&mut verified);

    let missing_recursive_errors: Vec<_> = errors
        .iter()
        .filter(|e| matches!(
            e.kind(),
            TypeValidationErrorKind::MissingRecursiveInterfaceImplementation { .. }
        ))
        .collect();
    assert_eq!(
        missing_recursive_errors.len(), 1,
        "expected exactly one MissingRecursiveInterfaceImplementation \
        error, got: {errors:?}",
    );

    let msg = missing_recursive_errors[0].to_string();

    // The key regression check: no dangling "implements ,"
    // with nothing between "implements" and the comma.
    assert!(
        !msg.contains("implements ,"),
        "error message should not contain dangling 'implements ,' \
        (indicates empty inheritance_path): {msg}",
    );

    // The error message should clearly reference the interface
    // `User` directly implements (Resource), because that is
    // the immediate cause of the transitive requirement.
    assert!(
        msg.contains("`Resource`"),
        "error message should contain the directly-implemented \
        interface `Resource`: {msg}",
    );
    assert!(
        msg.contains("`Node`"),
        "error message should contain the missing transitive \
        interface `Node`: {msg}",
    );
    assert!(
        msg.contains("`User`"),
        "error message should reference the implementing type \
        `User`: {msg}",
    );
}

// Regression test for a bug where the recursive interface
// walker only descended one level deep because the child
// validator re-iterated the implementing type's own
// `interfaces()` list (which ran out of new names after the
// first level) rather than walking each interface's own
// `interfaces()` chain. The visible symptom of the bug was
// that transitive interface requirements more than one level
// deep were silently ignored.
//
// Setup:
//   interface Root { root: String! }
//   interface Entity implements Root {
//     root: String!
//     entity: String!
//   }
//   interface Node implements Entity & Root {
//     root: String!
//     entity: String!
//     node: String!
//   }
//   type User implements Node { ... }
//
// `User` only directly declares `Node`, but per IsValidImplementation
// it must transitively declare every interface `Node` implements
// (including interfaces that `Node`'s own parents implement). So
// validating `User` must produce
// MissingRecursiveInterfaceImplementation errors for BOTH `Entity`
// (one level up from `Node`) AND `Root` (two levels up from `Node`,
// via `Entity`).
//
// https://spec.graphql.org/September2025/#IsValidImplementation()
// Written by Claude Code, reviewed by a human.
#[test]
fn missing_two_level_deep_transitive_interface() {
    // interface Root { root: String! }
    let mut root_fields = IndexMap::new();
    root_fields.insert(
        FieldName::new("root"),
        make_field(
            "root",
            "Root",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    let root_iface = make_interface("Root", root_fields, vec![]);

    // interface Entity implements Root {
    //   root: String!
    //   entity: String!
    // }
    let mut entity_fields = IndexMap::new();
    entity_fields.insert(
        FieldName::new("root"),
        make_field(
            "root",
            "Entity",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    entity_fields.insert(
        FieldName::new("entity"),
        make_field(
            "entity",
            "Entity",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    let entity_iface = make_interface(
        "Entity",
        entity_fields,
        vec![located_type_name("Root")],
    );

    // interface Node implements Entity & Root {
    //   root: String!
    //   entity: String!
    //   node: String!
    // }
    let mut node_fields = IndexMap::new();
    node_fields.insert(
        FieldName::new("root"),
        make_field(
            "root",
            "Node",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    node_fields.insert(
        FieldName::new("entity"),
        make_field(
            "entity",
            "Node",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    node_fields.insert(
        FieldName::new("node"),
        make_field(
            "node",
            "Node",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    let node_iface = make_interface(
        "Node",
        node_fields,
        vec![
            located_type_name("Entity"),
            located_type_name("Root"),
        ],
    );

    // type User implements Node { ... } -- intentionally does
    // NOT declare Entity or Root, which is the spec violation
    // under test.
    let mut user_fields = IndexMap::new();
    user_fields.insert(
        FieldName::new("root"),
        make_field(
            "root",
            "User",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    user_fields.insert(
        FieldName::new("entity"),
        make_field(
            "entity",
            "User",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    user_fields.insert(
        FieldName::new("node"),
        make_field(
            "node",
            "User",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    let user_obj = make_object(
        "User",
        user_fields,
        vec![located_type_name("Node")],
    );

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("String"), string_scalar());
    types_map.insert(
        TypeName::new("Root"),
        GraphQLType::Interface(Box::new(root_iface)),
    );
    types_map.insert(
        TypeName::new("Entity"),
        GraphQLType::Interface(Box::new(entity_iface)),
    );
    types_map.insert(
        TypeName::new("Node"),
        GraphQLType::Interface(Box::new(node_iface)),
    );

    let validator = ObjectOrInterfaceTypeValidator::new(
        &user_obj,
        &types_map,
    );
    let mut verified = HashSet::new();
    let errors = validator.validate(&mut verified);

    let missing_recursive_errors: Vec<_> = errors
        .iter()
        .filter(|e| matches!(
            e.kind(),
            TypeValidationErrorKind::MissingRecursiveInterfaceImplementation { .. }
        ))
        .collect();

    // Collect the set of missing interface names for assertions.
    let missing_names: HashSet<String> = missing_recursive_errors
        .iter()
        .filter_map(|e| match e.kind() {
            TypeValidationErrorKind::MissingRecursiveInterfaceImplementation {
                missing_recursive_interface_name,
                ..
            } => Some(missing_recursive_interface_name.clone()),
            _ => None,
        })
        .collect();

    assert!(
        missing_names.contains("Entity"),
        "expected a MissingRecursiveInterfaceImplementation \
        error for `Entity`, got: {errors:?}",
    );
    assert!(
        missing_names.contains("Root"),
        "expected a MissingRecursiveInterfaceImplementation \
        error for `Root` (two levels deep, transitively required \
        via Node -> Entity), got: {errors:?}",
    );

    // The Entity error should cite a path that starts at `Node`
    // (the interface `User` directly declares). The Root error
    // should cite a path that walks `Node -> Entity` (since
    // `Root` is required via Entity).
    for e in &missing_recursive_errors {
        let TypeValidationErrorKind::MissingRecursiveInterfaceImplementation {
            inheritance_path,
            missing_recursive_interface_name,
            type_name,
        } = e.kind() else {
            continue;
        };
        assert_eq!(type_name, "User");
        assert!(
            !inheritance_path.is_empty(),
            "inheritance_path must not be empty for {missing_recursive_interface_name}: {e:?}",
        );
        // Every error's inheritance path must start with `Node`
        // (the directly-declared interface on `User`).
        assert_eq!(
            inheritance_path[0], "Node",
            "inheritance_path should start at the directly-declared \
            interface `Node`, got: {inheritance_path:?}",
        );
        if missing_recursive_interface_name == "Root" {
            // The Root error may be reported via either the
            // Node -> Entity chain or directly via Node (since
            // Node itself also declares `implements Root`).
            // Either way, the first entry must be Node.
            assert!(
                inheritance_path.last()
                    .map(|s| s == "Node" || s == "Entity")
                    .unwrap_or(false),
                "last entry in inheritance_path for `Root` should \
                be either `Node` or `Entity`: {inheritance_path:?}",
            );
        }
    }
}

// Regression companion to
// `missing_two_level_deep_transitive_interface`: same 3-level
// interface hierarchy, but the implementing type DOES declare
// every transitively-required interface. Validates the
// positive case so that the recursive walker cannot regress to
// a mode where it spuriously emits
// MissingRecursiveInterfaceImplementation errors on correctly
// declared types.
//
// Setup:
//   interface Root { root: String! }
//   interface Entity implements Root { ... }
//   interface Node implements Entity & Root { ... }
//   type User implements Node & Entity & Root { ... }
//
// https://spec.graphql.org/September2025/#IsValidImplementation()
// Written by Claude Code, reviewed by a human.
#[test]
fn valid_three_level_deep_transitive_interface_declaration() {
    // interface Root { root: String! }
    let mut root_fields = IndexMap::new();
    root_fields.insert(
        FieldName::new("root"),
        make_field(
            "root",
            "Root",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    let root_iface = make_interface("Root", root_fields, vec![]);

    // interface Entity implements Root { ... }
    let mut entity_fields = IndexMap::new();
    entity_fields.insert(
        FieldName::new("root"),
        make_field(
            "root",
            "Entity",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    entity_fields.insert(
        FieldName::new("entity"),
        make_field(
            "entity",
            "Entity",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    let entity_iface = make_interface(
        "Entity",
        entity_fields,
        vec![located_type_name("Root")],
    );

    // interface Node implements Entity & Root { ... }
    let mut node_fields = IndexMap::new();
    node_fields.insert(
        FieldName::new("root"),
        make_field(
            "root",
            "Node",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    node_fields.insert(
        FieldName::new("entity"),
        make_field(
            "entity",
            "Node",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    node_fields.insert(
        FieldName::new("node"),
        make_field(
            "node",
            "Node",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    let node_iface = make_interface(
        "Node",
        node_fields,
        vec![
            located_type_name("Entity"),
            located_type_name("Root"),
        ],
    );

    // type User implements Node & Entity & Root { ... }
    let mut user_fields = IndexMap::new();
    user_fields.insert(
        FieldName::new("root"),
        make_field(
            "root",
            "User",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    user_fields.insert(
        FieldName::new("entity"),
        make_field(
            "entity",
            "User",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    user_fields.insert(
        FieldName::new("node"),
        make_field(
            "node",
            "User",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    let user_obj = make_object(
        "User",
        user_fields,
        vec![
            located_type_name("Node"),
            located_type_name("Entity"),
            located_type_name("Root"),
        ],
    );

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("String"), string_scalar());
    types_map.insert(
        TypeName::new("Root"),
        GraphQLType::Interface(Box::new(root_iface)),
    );
    types_map.insert(
        TypeName::new("Entity"),
        GraphQLType::Interface(Box::new(entity_iface)),
    );
    types_map.insert(
        TypeName::new("Node"),
        GraphQLType::Interface(Box::new(node_iface)),
    );

    let validator = ObjectOrInterfaceTypeValidator::new(
        &user_obj,
        &types_map,
    );
    let mut verified = HashSet::new();
    let errors = validator.validate(&mut verified);
    assert!(
        errors.is_empty(),
        "expected no errors (User correctly declares \
        Node & Entity & Root), got: {errors:?}",
    );
}
