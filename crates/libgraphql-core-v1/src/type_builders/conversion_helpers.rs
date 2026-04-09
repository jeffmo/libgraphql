use crate::names::TypeName;
use crate::type_builders::enum_value_def_builder::EnumValueDefBuilder;
use crate::type_builders::field_def_builder::FieldDefBuilder;
use crate::type_builders::input_field_def_builder::InputFieldDefBuilder;
use crate::type_builders::parameter_def_builder::ParameterDefBuilder;
use crate::types::EnumValue;
use crate::types::FieldDefinition;
use crate::types::InputField;
use crate::types::ParameterDefinition;

pub(crate) fn field_def_from_builder(
    b: FieldDefBuilder,
    parent_type_name: &TypeName,
) -> FieldDefinition {
    FieldDefinition {
        description: b.description,
        directives: b.directives,
        name: b.name,
        parameters: b.parameters.into_iter().map(|p| {
            let param = param_def_from_builder(p);
            (param.name.clone(), param)
        }).collect(),
        parent_type_name: parent_type_name.clone(),
        span: b.span,
        type_annotation: b.type_annotation,
    }
}

pub(crate) fn param_def_from_builder(
    b: ParameterDefBuilder,
) -> ParameterDefinition {
    ParameterDefinition {
        default_value: b.default_value,
        description: b.description,
        directives: b.directives,
        name: b.name,
        span: b.span,
        type_annotation: b.type_annotation,
    }
}

pub(crate) fn input_field_from_builder(
    b: InputFieldDefBuilder,
    parent_type_name: &TypeName,
) -> InputField {
    InputField {
        default_value: b.default_value,
        description: b.description,
        directives: b.directives,
        name: b.name,
        parent_type_name: parent_type_name.clone(),
        span: b.span,
        type_annotation: b.type_annotation,
    }
}

pub(crate) fn enum_value_from_builder(
    b: EnumValueDefBuilder,
    parent_type_name: &TypeName,
) -> EnumValue {
    EnumValue {
        description: b.description,
        directives: b.directives,
        name: b.name,
        parent_type_name: parent_type_name.clone(),
        span: b.span,
    }
}
