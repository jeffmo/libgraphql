use indexmap::IndexMap;

use crate::schema::TypeValidationError;
use crate::types::GraphQLType;
use crate::types::InputField;
use crate::types::InputObjectType;
use crate::types::TypeAnnotation;
use std::collections::HashMap;
use std::collections::HashSet;

pub(super) struct InputObjectTypeValidator<'a> {
    errors: Vec<TypeValidationError>,
    type_: &'a InputObjectType,
    types_map: &'a HashMap<String, GraphQLType>,
}
impl<'a> InputObjectTypeValidator<'a> {
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
        self.validate_fields_recursive(
            self.type_.name(),
            fields,
            &mut vec![],
            HashSet::from([self.type_.name()]),
        );
        self.errors
    }

    fn validate_fields_recursive(
        &mut self,
        type_name: &'a str,
        fields: &'a IndexMap<String, InputField>,
        path: &mut Vec<(&'a str, Option<&'a str>)>,
        seen_type_names: HashSet<&'a str>,
    ) {
        for (field_name, field) in fields {
            let type_annot = field.type_annotation();
            let innermost_type_name =
               type_annot
                    .innermost_named_type_annotation()
                    .graphql_type_name();
            let innermost_type = self.types_map.get(innermost_type_name);

            let innermost_type = if let Some(innermost_type) = innermost_type {
                // Input object fields can not be declared with a non-input
                // "Object" type.
                //
                // https://spec.graphql.org/October2021/#sel-IAHhBXDDBFCAACEB4iG
                if innermost_type.as_object().is_some() {
                    self.errors.push(
                        TypeValidationError::InvalidInputFieldWithOutputType {
                            def_location:
                                field.type_annotation()
                                    .ref_location()
                                    .to_owned(),
                            field_name: field_name.to_owned(),
                            invalid_type_name: innermost_type_name.to_string(),
                            parent_type_name: type_name.to_owned(),
                        }
                    );
                }

                innermost_type
            } else {
                self.errors.push(TypeValidationError::UndefinedTypeName {
                    ref_location:
                        field.type_annotation()
                            .ref_location()
                            .to_owned(),
                        undefined_type_name:
                            innermost_type_name.to_string(),
                });
                continue
            };

            // Look for input-type cycles that aren't broken by at least one
            // nullable type.
            let is_cycle_breaking =
                annot_contains_cycle_breaking_nullable_type(
                    field.type_annotation(),
                );
            if !is_cycle_breaking {
                path.extend_from_slice(&[
                    (type_name, Some(field_name)),
                    (innermost_type_name, None),
                ]);
                if seen_type_names.contains(innermost_type_name) {
                    // TODO: Kind of annoying that this will result in 1x
                    //       CircularInputFieldChain error for each type in the
                    //       chain. Consider storing the path as some kind of
                    //       `Hash`-compatible ringbuffer that we can keep in a
                    //       HashSet to see if we've already discovered and
                    //       surfaced an error for any given
                    //       CircularInputFieldChain error.
                    self.errors.push(TypeValidationError::CircularInputFieldChain {
                        circular_field_path: path.iter().map(|(type_name, field_name)| {
                            if let Some(field_name) = field_name {
                                format!("{type_name}.{field_name}")
                            } else {
                                type_name.to_string()
                            }
                        }).collect(),
                    });
                } else if let GraphQLType::InputObject(input_obj_type) = innermost_type {
                    let mut seen_type_names = seen_type_names.clone();
                    seen_type_names.insert(innermost_type_name);
                    self.validate_fields_recursive(
                        innermost_type_name,
                        input_obj_type.fields(),
                        path,
                        seen_type_names,
                    );
                }
                path.pop();
            }
        }
    }

}

fn annot_contains_cycle_breaking_nullable_type(
    type_annot: &TypeAnnotation,
) -> bool {
    match type_annot {
        TypeAnnotation::List(list_annot) =>
            list_annot.nullable() || annot_contains_cycle_breaking_nullable_type(
                list_annot.inner_type_annotation()
            ),
        TypeAnnotation::Named(named_annot) =>
            named_annot.nullable(),
    }
}
