use crate::schema::TypeValidationError;
use crate::types::GraphQLType;
use crate::types::ObjectOrInterfaceTypeData;
use std::collections::HashMap;
use std::collections::HashSet;

pub(super) struct ObjectOrInterfaceTypeValidator<'a> {
    errors: Vec<TypeValidationError>,
    implemented_iface_names: HashSet<&'a str>,
    inheritance_path: Vec<&'a str>,
    type_: &'a ObjectOrInterfaceTypeData,
    types_map: &'a HashMap<String, GraphQLType>,
}
impl<'a> ObjectOrInterfaceTypeValidator<'a> {
    pub fn new(
        type_: &'a ObjectOrInterfaceTypeData,
        types_map: &'a HashMap<String, GraphQLType>,
    ) -> Self {
        Self {
            errors: vec![],
            implemented_iface_names:
                type_.interface_names()
                    .iter()
                    .copied()
                    .collect(),
            inheritance_path: vec![],
            type_,
            types_map,
        }
    }

    pub fn validate(
        mut self,
        verified_interface_impls: &mut HashSet<&'a str>,
    ) -> Vec<TypeValidationError> {
        let type_name = self.type_.name();
        let type_fields = self.type_.fields();
        for iface_name in &self.implemented_iface_names {
            // Since interfaces can implement other interfaces, it's possible
            // that we're validating a recursively-implemented interface that
            // we've already validated on this type; So short-circuit if/when
            // we encounter this scenario.
            let iface_name_already_verified_present =
                !verified_interface_impls.insert(iface_name);
            if iface_name_already_verified_present {
                continue;
            }

            // Verify that this implemented interface name is actually a defined
            // type.
            let iface_type = self.types_map.get(*iface_name);
            let iface_type =
                if let Some(iface_type) = iface_type { iface_type } else {
                    self.errors.push(
                        TypeValidationError::ImplementsUndefinedInterface {
                            type_name: type_name.to_string(),
                            undefined_interface_name: iface_name.to_string(),
                            loc: self.type_.def_location().to_owned(),
                        }
                    );
                    continue;
                };

            // Verify that the defined type being implemented is an interface
            // type.
            let iface = iface_type.as_interface();
            let iface =
                if let Some(iface) = iface { iface } else {
                    self.errors.push(
                        TypeValidationError::ImplementsNonInterfaceType {
                            type_name: type_name.to_string(),
                            non_interface_type_name: iface_type.name().to_string(),
                            loc: self.type_.def_location().to_owned(),
                        }
                    );
                    continue;
                };

            // Verify that the implementing object/interface type also
            // explicitly implements each of the interfaces *this* interface
            // itself implements.
            //
            // https://spec.graphql.org/October2021/#IsValidImplementation()
            let iface_implemented_iface_names =
                iface.interface_names()
                    .into_iter()
                    .collect::<HashSet<_>>();
            let missing_recursive_interface_names =
                iface_implemented_iface_names.difference(&self.implemented_iface_names)
                    .collect::<Vec<_>>();
            for missing_rec_iface_name in missing_recursive_interface_names {
                self.errors.push(
                    TypeValidationError::MissingRecursiveInterfaceImplementation {
                        def_location: self.type_.def_location().to_owned(),
                        inheritance_path:
                            self.inheritance_path.iter()
                                .map(|s| s.to_string())
                                .collect(),
                        missing_recursive_interface_name:
                            missing_rec_iface_name.to_string(),
                        type_name: type_name.to_string(),
                    }
                );
            }

            // Verify that all of this interface's fields are implemented on
            // the implementing type.
            let mut child_inheritance_path = self.inheritance_path.to_owned();
            child_inheritance_path.push(iface_name);
            let child_validator = ObjectOrInterfaceTypeValidator {
                errors: vec![],
                implemented_iface_names:
                    iface_implemented_iface_names.iter()
                        .copied()
                        .collect(),
                inheritance_path: child_inheritance_path,
                type_: self.type_,
                types_map: self.types_map,
            };
            self.errors.append(&mut child_validator.validate(
                verified_interface_impls,
            ));

            let iface_fields = iface.fields();
            for (field_name, iface_field) in iface_fields {
                let type_field = type_fields.get(field_name);
                let type_field =
                    if let Some(type_field) = type_field {
                        type_field
                    } else {
                        self.errors.push(
                            TypeValidationError::MissingInterfaceSpecifiedField {
                                def_location: self.type_.def_location().to_owned(),
                                field_name: field_name.to_string(),
                                interface_name: iface_name.to_string(),
                                type_name: type_name.to_string(),
                            }
                        );
                        continue
                    };

                let iface_field_params = iface_field.parameters();
                let type_field_params = type_field.parameters();

                // For each parameter defined on this field in the interface,
                // there must be a corresponding and equivalently-typed
                // parameter defined on the implementing type.
                //
                // https://spec.graphql.org/October2021/#IsValidImplementation()
                for (param_name, iface_field_param) in iface_field_params {
                    let type_param = type_field_params.get(param_name);
                    let type_param =
                        if let Some(type_param) = type_param {
                            type_param
                        } else {
                            self.errors.push(
                                TypeValidationError::MissingInterfaceSpecifiedFieldParameter {
                                    def_location: type_field.def_location().to_owned(),
                                    field_name: field_name.to_string(),
                                    interface_name: iface_name.to_string(),
                                    missing_parameter_name: param_name.to_string(),
                                    type_name: type_name.to_string(),
                                }
                            );
                            continue;
                        };

                    let iface_param_type = iface_field_param.type_annotation();
                    let type_param_type = type_param.type_annotation();
                    if !type_param_type.is_equivalent_to(iface_param_type) {
                        self.errors.push(
                            TypeValidationError::InvalidInterfaceSpecifiedFieldParameterType {
                                def_location: type_param.def_location().to_owned(),
                                expected_parameter_type: iface_param_type.to_owned(),
                                field_name: field_name.to_string(),
                                interface_name: iface_name.to_string(),
                                invalid_parameter_type: type_param_type.to_owned(),
                                parameter_name: param_name.to_string(),
                                type_name: type_name.to_string(),
                            }
                        );
                    }
                }

                // Any parameters defined on the implementing field which aren't
                // also defined on the interface's corresponding field must be
                // optional (either nullable or defined with a default value).
                //
                // See 2.d at https://spec.graphql.org/October2021/#IsValidImplementation()
                let iface_field_param_names =
                    iface_field_params.keys().collect::<HashSet<_>>();
                let type_field_param_names =
                    type_field_params.keys().collect::<HashSet<_>>();
                let additional_field_param_names =
                    type_field_param_names.difference(&iface_field_param_names);
                for additional_param_name in additional_field_param_names {
                    let additional_param =
                        type_field_params.get(*additional_param_name).unwrap();
                    let additional_param_type_annot = additional_param.type_annotation();

                    let is_nullable =
                        additional_param_type_annot.nullable();
                    let has_default =
                        additional_param.default_value().is_some();
                    if !is_nullable && !has_default  {
                        self.errors.push(
                            TypeValidationError::InvalidRequiredAdditionalParameterOnInterfaceSpecifiedField {
                                location:
                                    additional_param_type_annot.ref_location()
                                        .to_owned(),
                                field_name: field_name.to_string(),
                                interface_name: iface_name.to_string(),
                                parameter_name: additional_param_name.to_string(),
                                type_name: type_name.to_string(),
                            }
                        );
                    }
                }

                let type_field_annot = type_field.type_annotation();
                let iface_field_annot = iface_field.type_annotation();
                if !type_field_annot.is_subtype_of_impl(
                    self.types_map,
                    iface_field_annot,
                ) {
                    self.errors.push(
                        TypeValidationError::InvalidInterfaceSpecifiedFieldType {
                            location:
                                type_field_annot.ref_location().to_owned(),
                            expected_field_type: iface_field_annot.to_owned(),
                            field_name: field_name.to_string(),
                            interface_name: iface_name.to_string(),
                            invalid_field_type: type_field_annot.to_owned(),
                            type_name: type_name.to_string(),
                        }
                    );
                }
            }
        }

        for (field_name, field) in type_fields {
            // All fields on an object type must be declared with an output
            // type.
            //
            // https://spec.graphql.org/October2021/#sel-JAHZhCFDBFABLBgB_pM
            let innermost_type_name =
                field.type_annotation()
                    .innermost_named_type_annotation()
                    .graphql_type_name();
            let innermost_type = self.types_map.get(innermost_type_name);
            if let Some(innermost_type) = innermost_type {
                if !innermost_type.is_output_type() {
                    self.errors.push(
                        TypeValidationError::InvalidOutputFieldWithInputType {
                            def_location:
                                field.type_annotation()
                                    .ref_location()
                                    .to_owned(),
                            field_name: field_name.to_string(),
                            input_type_name: innermost_type_name.to_string(),
                            parent_type_name: type_name.to_string(),
                        }
                    );
                }
            } else {
                self.errors.push(TypeValidationError::UndefinedTypeName {
                    ref_location:
                        field.type_annotation()
                            .ref_location()
                            .to_owned(),
                    undefined_type_name:
                        innermost_type_name.to_string(),
                });
            }

            for (param_name, param) in field.parameters() {
                // All parameters must be declared with an output type.
                //
                // https://spec.graphql.org/October2021/#sel-KAHZhCFDBHBDCAACEB6yD
                let innermost_type_name =
                    param.type_annotation()
                        .innermost_named_type_annotation()
                        .graphql_type_name();
                let innermost_type = self.types_map.get(innermost_type_name);
                if let Some(innermost_type) = innermost_type {
                    if !innermost_type.is_input_type() {
                        self.errors.push(
                            TypeValidationError::InvalidParameterWithOutputOnlyType {
                                def_location:
                                    param.type_annotation()
                                        .ref_location()
                                        .to_owned(),
                                outputonly_type_name:
                                    innermost_type_name.to_string(),
                                parameter_name: param_name.to_string(),
                            }
                        );
                    }
                } else {
                    self.errors.push(TypeValidationError::UndefinedTypeName {
                        ref_location:
                            param.type_annotation()
                                .ref_location()
                                .to_owned(),
                        undefined_type_name:
                            innermost_type_name.to_string(),
                    })
                }
            }
        }

        self.errors
    }
}
