use crate::error_note::ErrorNote;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::schema::TypeValidationError;
use crate::schema::TypeValidationErrorKind;
use crate::types::GraphQLType;
use crate::types::InputField;
use crate::types::InputObjectType;
use crate::types::TypeAnnotation;
use indexmap::IndexMap;
use std::collections::HashSet;

/// Validates an input object type's field type references,
/// input-type legality, and circular non-nullable reference
/// chains.
///
/// Per the GraphQL spec, all input object fields must reference
/// valid input types (scalars, enums, or other input objects) and
/// input object types must not form non-nullable circular
/// references (which would make them impossible to construct).
///
/// See [Input Objects](https://spec.graphql.org/September2025/#sec-Input-Objects).
pub(crate) struct InputObjectTypeValidator<'a> {
    errors: Vec<TypeValidationError>,
    type_: &'a InputObjectType,
    types_map: &'a IndexMap<TypeName, GraphQLType>,
}

impl<'a> InputObjectTypeValidator<'a> {
    pub fn new(
        type_: &'a InputObjectType,
        types_map: &'a IndexMap<TypeName, GraphQLType>,
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
        type_name: &'a TypeName,
        fields: &'a IndexMap<FieldName, InputField>,
        path: &mut Vec<(&'a TypeName, Option<&'a FieldName>)>,
        seen_type_names: HashSet<&'a TypeName>,
    ) {
        for (field_name, field) in fields {
            let type_annot = field.type_annotation();
            let innermost_type_name =
                type_annot.innermost_type_name();
            let innermost_type =
                self.types_map.get(innermost_type_name);

            let innermost_type =
                if let Some(innermost_type) = innermost_type {
                    // Input object fields must not use non-input
                    // types (Object, Interface, Union are output-only).
                    //
                    // https://spec.graphql.org/September2025/#sel-IAHhBXDDBFCAACEB4iG
                    if !innermost_type.is_input_type() {
                        self.errors.push(TypeValidationError::new(
                            TypeValidationErrorKind::InvalidInputFieldWithOutputType {
                                field_name:
                                    field_name.to_string(),
                                invalid_type_name:
                                    innermost_type_name.to_string(),
                                parent_type_name:
                                    type_name.to_string(),
                            },
                            field.type_annotation().span(),
                            vec![ErrorNote::spec(
                                "https://spec.graphql.org/September2025/#sel-IAHhBXDDBFCAACEB4iG",
                            )],
                        ));
                    }

                    innermost_type
                } else {
                    // https://spec.graphql.org/September2025/#sec-Input-Objects
                    self.errors.push(TypeValidationError::new(
                        TypeValidationErrorKind::UndefinedTypeName {
                            undefined_type_name:
                                innermost_type_name.to_string(),
                        },
                        field.type_annotation().span(),
                        vec![],
                    ));
                    continue;
                };

            // Look for input-type cycles that aren't broken by
            // at least one nullable type.
            let is_cycle_breaking =
                annot_breaks_circular_chain(
                    field.type_annotation(),
                );
            if !is_cycle_breaking {
                path.extend_from_slice(&[
                    (type_name, Some(field_name)),
                    (innermost_type_name, None),
                ]);
                if seen_type_names.contains(innermost_type_name) {
                    // https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation
                    self.errors.push(TypeValidationError::new(
                        TypeValidationErrorKind::CircularInputFieldChain {
                            circular_field_path: path
                                .iter()
                                .map(|(tn, fn_opt)| {
                                    if let Some(fn_) = fn_opt {
                                        format!("{tn}.{fn_}")
                                    } else {
                                        format!("{tn}")
                                    }
                                })
                                .collect(),
                        },
                        field.type_annotation().span(),
                        vec![ErrorNote::spec(
                            "https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation",
                        )],
                    ));
                } else if let GraphQLType::InputObject(input_obj_type) =
                    innermost_type
                {
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
                path.pop();
            }
        }
    }
}

fn annot_breaks_circular_chain(
    type_annot: &TypeAnnotation,
) -> bool {
    match type_annot {
        TypeAnnotation::List(_) => true,
        TypeAnnotation::Named(named_annot) => named_annot.nullable(),
    }
}
