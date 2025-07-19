use crate::schema::TypeValidationError;
use crate::types::GraphQLType;
use crate::types::InputObjectType;
use std::collections::HashMap;

pub(super) struct InputObjectOrInterfaceTypeValidator<'a> {
    errors: Vec<TypeValidationError>,
    type_: &'a InputObjectType,
    types_map: &'a HashMap<String, GraphQLType>,
}
impl<'a> InputObjectOrInterfaceTypeValidator<'a> {
    pub fn new(
        type_: &'a InputObjectType,
        types_map: &'a HashMap<String, GraphQLType>,
    ) -> Self {
        Self {
            errors: vec![],
            type_,
            types_map,
        }
    }

    pub fn validate(mut self) -> Vec<TypeValidationError> {
        let fields = self.type_.fields();

        for (field_name, field) in fields {
            let innermost_type_name =
                field.type_annotation()
                    .innermost_named_type_annotation()
                    .graphql_type_name();
            let innermost_type = self.types_map.get(innermost_type_name);
            if let Some(innermost_type) = innermost_type {
                // All fields on an input object type must be declared with an
                // input type.
                //
                // https://spec.graphql.org/October2021/#sel-IAHhBXDDBFCAACEB4iG
                if innermost_type.is_output_type() {
                    self.errors.push(
                        TypeValidationError::InvalidInputFieldWithOutputType {
                            def_location:
                                field.type_annotation()
                                    .def_location()
                                    .to_owned(),
                            field_name: field_name.to_owned(),
                            invalid_type_name: innermost_type_name.to_string(),
                            parent_type_name: self.type_.name().to_owned(),
                        }
                    );
                }
            } else {
                self.errors.push(TypeValidationError::UndefinedTypeName {
                    def_location:
                        field.type_annotation()
                            .def_location()
                            .to_owned(),
                        undefined_type_name:
                            innermost_type_name.to_string(),
                });
            }
        }

        self.errors
    }
}
