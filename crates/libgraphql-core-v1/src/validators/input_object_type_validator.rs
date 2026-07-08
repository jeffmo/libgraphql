use crate::error_note::ErrorNote;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::schema::TypeValidationError;
use crate::schema::TypeValidationErrorKind;
use crate::types::find_deprecated_annotation;
use crate::types::GraphQLType;
use crate::types::InputField;
use crate::types::InputObjectType;
use crate::types::TypeAnnotation;
use crate::validators::edit_distance::find_similar_names;
use indexmap::IndexMap;
use std::collections::HashSet;

/// Validates an input object type's field type references,
/// input-type legality, circular non-nullable reference chains,
/// `@oneOf` constraints, and `@deprecated` constraints.
///
/// Per the GraphQL spec, all input object fields must reference
/// valid input types (scalars, enums, or other input objects) and
/// input object types must not form non-nullable circular
/// references (which would make them impossible to construct).
/// Additionally, every field of a `@oneOf` input object must have
/// a nullable type and must not declare a default value, and
/// `@deprecated` must not be applied to required (non-null
/// without a default value) input fields.
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
        self.validate_oneof_constraints();
        self.validate_deprecated_constraints();
        let fields = self.type_.fields();
        self.validate_fields_recursive(
            self.type_.name(),
            fields,
            &mut vec![],
            HashSet::from([self.type_.name()]),
        );
        self.errors
    }

    /// Enforces that `@deprecated` is not applied to any required
    /// (non-null type without a default value) input field. To
    /// deprecate a required input field, it must first be made
    /// optional.
    ///
    /// See [@deprecated](https://spec.graphql.org/September2025/#sec--deprecated).
    fn validate_deprecated_constraints(&mut self) {
        for (field_name, field) in self.type_.fields() {
            let is_required = !field.type_annotation().nullable()
                && field.default_value().is_none();
            // `@deprecated` must not appear on a required input
            // field.
            //
            // https://spec.graphql.org/September2025/#sec--deprecated
            if is_required && field.deprecation_state().is_deprecated() {
                // Point the error at the input field's
                // `@deprecated` annotation when possible;
                // otherwise fall back to the field itself.
                let error_span =
                    find_deprecated_annotation(field.directives())
                        .map(|annot| annot.span())
                        .unwrap_or_else(|| field.span());
                self.errors.push(TypeValidationError::new(
                    TypeValidationErrorKind::DeprecatedRequiredInputField {
                        field_name: field_name.to_string(),
                        parent_type_name: self.type_.name().to_string(),
                    },
                    error_span,
                    vec![
                        ErrorNote::help(
                            "to deprecate a required input field, first \
                            make it optional by changing its type to \
                            nullable or adding a default value",
                        ),
                        ErrorNote::spec(
                            "https://spec.graphql.org/September2025/#sec--deprecated",
                        ),
                    ],
                ));
            }
        }
    }

    /// Enforces the `@oneOf` input object constraints: every field
    /// must have a nullable type and must not declare a default
    /// value.
    ///
    /// See [Input Objects — Type Validation](https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation).
    fn validate_oneof_constraints(&mut self) {
        let is_oneof = self.type_.directives().iter().any(
            |annot| annot.name().as_str() == "oneOf",
        );
        if !is_oneof {
            return;
        }

        for (field_name, field) in self.type_.fields() {
            if !field.type_annotation().nullable() {
                // https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation
                self.errors.push(TypeValidationError::new(
                    TypeValidationErrorKind::InvalidNonNullableOneOfInputField {
                        field_name: field_name.to_string(),
                        parent_type_name:
                            self.type_.name().to_string(),
                    },
                    field.type_annotation().span(),
                    vec![ErrorNote::spec(
                        "https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation",
                    )],
                ));
            }

            if field.default_value().is_some() {
                // https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation
                self.errors.push(TypeValidationError::new(
                    TypeValidationErrorKind::InvalidOneOfInputFieldWithDefaultValue {
                        field_name: field_name.to_string(),
                        parent_type_name:
                            self.type_.name().to_string(),
                    },
                    field.span(),
                    vec![ErrorNote::spec(
                        "https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation",
                    )],
                ));
            }
        }
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
                    let mut notes = Vec::new();
                    let suggestions = find_similar_names(
                        innermost_type_name.as_str(),
                        self.types_map.keys(),
                    );
                    if let Some(best) = suggestions.first() {
                        notes.push(ErrorNote::help(
                            format!("did you mean `{best}`?"),
                        ));
                    }
                    notes.push(ErrorNote::spec(
                        "https://spec.graphql.org/September2025/#sec-Types",
                    ));
                    self.errors.push(TypeValidationError::new(
                        TypeValidationErrorKind::UndefinedTypeName {
                            undefined_type_name:
                                innermost_type_name.to_string(),
                        },
                        field.type_annotation().span(),
                        notes,
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
