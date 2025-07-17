use crate::loc;
use crate::types::GraphQLType;
use crate::types::TypeAnnotation;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum TypeValidationError {
    #[error("Attempted to implement a type that is not defined as an interface")]
    ImplementsNonInterfaceType {
        type_name: String,
        non_interface_type_name: String,
        loc: loc::SchemaDefLocation,
    },

    #[error("Attempted to implement an interface that is not defined in the schema")]
    ImplementsUndefinedInterface {
        type_name: String,
        undefined_interface_name: String,
        loc: loc::SchemaDefLocation,
    },

    #[error(
        "Output fields can not be declared with an input type: The \
        `{parent_type_name}.{field_name}` field is an output field, but the \
        `{input_type_name}` type is an input-type"
    )]
    InvalidOutputFieldWithInputType {
        def_location: loc::SchemaDefLocation,
        field_name: String,
        input_type_name: String,
        parent_type_name: String,
    },

    #[error(
        "Parameters can only be declared with input-compatible types: The \
        `{parameter_name}` parameter was declared with the \
        `{outputonly_type_name}` type, which is not an input-compatible type."
    )]
    InvalidParameterWithOutputOnlyType {
        def_location: loc::SchemaDefLocation,
        parameter_name: String,
        outputonly_type_name: String,
    },

    #[error(
        "Invalid parameter type: The `{type_name}.{field_name}` field \
        defines the `{parameter_name}` parameter with a type of \
        `{invalid_parameter_type:?}`, but `{interface_name}.{field_name}` \
        defines this parameter with type `{expected_parameter_type:?}`"
    )]
    InvalidInterfaceSpecifiedFieldParameterType {
        def_location: loc::SchemaDefLocation,
        expected_parameter_type: TypeAnnotation,
        field_name: String,
        interface_name: String,
        invalid_parameter_type: TypeAnnotation,
        parameter_name: String,
        type_name: String,
    },

    #[error(
        "Invalid interface-specified field type: The \
        `{type_name}.{field_name}` field's type is defined as \
        `{invalid_field_type:?}` which is incompatible with \
        `{interface_name}.{field_name}` whose type is defined as `{expected_field_type:?}`."
    )]
    InvalidInterfaceSpecifiedFieldType {
        def_location: loc::SchemaDefLocation,
        expected_field_type: TypeAnnotation,
        field_name: String,
        interface_name: String,
        invalid_field_type: TypeAnnotation,
        type_name: String,
    },

    #[error(
        "Additional parameters defined on interface-specified fields must not \
        be required"
    )]
    InvalidRequiredAdditionalParameterOnInterfaceSpecifiedField {
        def_location: loc::SchemaDefLocation,
        field_name: String,
        interface_name: String,
        parameter_name: String,
        type_name: String,
    },

    #[error(
        "Invalid union member type: The `{union_type_name}` type defines one \
        of its members as `{}`, but this type is a {} type and union members \
        can only be object types.",
        invalid_member_type.name(),
        invalid_member_type.type_kind_name(),
    )]
    InvalidUnionMemberTypeKind {
        def_location: loc::SchemaDefLocation,
        union_type_name: String,
        invalid_member_type: GraphQLType,
    },

    #[error(
        "The `{type_name}` type implements the `{interface_name}` interface, \
        but does not define a field named `{field_name}`"
    )]
    MissingInterfaceSpecifiedField {
        def_location: loc::SchemaDefLocation,
        field_name: String,
        interface_name: String,
        type_name: String,
    },

    #[error(
        "The `{type_name}` type implements the `{interface_name}` interface \
        which defines a `{missing_parameter_name}` parameter on the \
        `{field_name}` field, but `{type_name}` has no \
        `{missing_parameter_name}` parameter defined on \
        `{type_name}.{field_name}`"
    )]
    MissingInterfaceSpecifiedFieldParameter {
        def_location: loc::SchemaDefLocation,
        field_name: String,
        interface_name: String,
        missing_parameter_name: String,
        type_name: String,
    },

    #[error(
        "The `{type_name}` type implements {}, therefore \
        `{type_name}` must also implement \
        `{missing_recursive_interface_name}`",
        inheritance_path.iter()
            .map(|iface_name| format!("the `{iface_name}` interface"))
            .collect::<Vec<_>>()
            .join(" which implements "),
    )]
    MissingRecursiveInterfaceImplementation {
        def_location: loc::SchemaDefLocation,
        inheritance_path: Vec<String>,
        missing_recursive_interface_name: String,
        type_name: String,
    },

    #[error("There is no type defined with the name `{undefined_type_name}`")]
    UndefinedTypeName {
        def_location: loc::SchemaDefLocation,
        undefined_type_name: String,
    }
}
