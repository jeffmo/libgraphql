use crate::error_note::ErrorNote;
use crate::names::TypeName;
use crate::schema::TypeValidationError;
use crate::schema::TypeValidationErrorKind;
use crate::types::GraphQLType;
use crate::types::HasFieldsAndInterfaces;
use crate::types::InterfaceType;
use indexmap::IndexMap;
use std::collections::HashSet;

/// Validates an object or interface type's interface
/// implementations, field output-type legality, and parameter
/// input-type legality.
///
/// Implements the
/// [IsValidImplementation](https://spec.graphql.org/September2025/#IsValidImplementation())
/// algorithm from the GraphQL specification.
pub(crate) struct ObjectOrInterfaceTypeValidator<'a, T: HasFieldsAndInterfaces> {
    errors: Vec<TypeValidationError>,
    implemented_iface_names: HashSet<&'a TypeName>,
    type_: &'a T,
    types_map: &'a IndexMap<TypeName, GraphQLType>,
}

impl<'a, T: HasFieldsAndInterfaces> ObjectOrInterfaceTypeValidator<'a, T> {
    pub fn new(
        type_: &'a T,
        types_map: &'a IndexMap<TypeName, GraphQLType>,
    ) -> Self {
        Self {
            errors: vec![],
            implemented_iface_names: type_
                .interfaces()
                .iter()
                .map(|l| &l.value)
                .collect(),
            type_,
            types_map,
        }
    }

    pub fn validate(
        mut self,
        verified_interface_impls: &mut HashSet<&'a TypeName>,
    ) -> Vec<TypeValidationError> {
        let type_name = self.type_.name();
        let type_fields = self.type_.fields();
        let type_span = self.type_.span();

        for located_iface in self.type_.interfaces() {
            let iface_name = &located_iface.value;

            // Since interfaces can implement other interfaces,
            // it's possible that we're validating a
            // recursively-implemented interface that we've
            // already validated on this type; so short-circuit
            // if/when we encounter this scenario.
            let iface_name_already_verified =
                !verified_interface_impls.insert(iface_name);
            if iface_name_already_verified {
                continue;
            }

            // Verify that this implemented interface name is
            // actually a defined type.
            //
            // https://spec.graphql.org/September2025/#IsValidImplementation()
            let Some(iface_type) = self.types_map.get(iface_name) else {
                self.errors.push(TypeValidationError::new(
                    TypeValidationErrorKind::ImplementsUndefinedInterface {
                        type_name: type_name.to_string(),
                        undefined_interface_name: iface_name.to_string(),
                    },
                    located_iface.span,
                    vec![ErrorNote::spec(
                        "https://spec.graphql.org/September2025/#IsValidImplementation()",
                    )],
                ));
                continue;
            };

            // Verify that the defined type being implemented is
            // an interface type.
            //
            // https://spec.graphql.org/September2025/#IsValidImplementation()
            let Some(iface) = iface_type.as_interface() else {
                self.errors.push(TypeValidationError::new(
                    TypeValidationErrorKind::ImplementsNonInterfaceType {
                        type_name: type_name.to_string(),
                        non_interface_type_name: iface_name.to_string(),
                    },
                    located_iface.span,
                    vec![
                        ErrorNote::general_with_span(
                            format!("`{iface_name}` is defined here"),
                            iface_type.span(),
                        ),
                        ErrorNote::spec(
                            "https://spec.graphql.org/September2025/#IsValidImplementation()",
                        ),
                    ],
                ));
                continue;
            };

            // Verify that the implementing object/interface type
            // also explicitly implements each of the interfaces
            // *this* interface itself implements -- including
            // interfaces it implements transitively.
            //
            // This must walk `iface`'s own interface chain (not
            // `self.type_`'s interface chain) to detect cases
            // like:
            //
            //   interface Root { ... }
            //   interface Entity implements Root { ... }
            //   interface Node implements Entity & Root { ... }
            //   type User implements Node { ... }
            //
            // Here, `User` must transitively declare `Entity` and
            // `Root` in addition to `Node`. The main loop here
            // iterates `User`'s declared interfaces (just `Node`
            // in this case), so the recursive walk steps into
            // `Node`'s own interface chain to surface BOTH
            // missing transitive requirements.
            //
            // https://spec.graphql.org/September2025/#IsValidImplementation()
            self.check_interface_chain(
                iface,
                &[],
                verified_interface_impls,
            );

            let iface_fields = iface.fields();
            for (field_name, iface_field) in iface_fields {
                let Some(type_field) = type_fields.get(field_name)
                else {
                    // The implementing type must define every
                    // field the interface declares.
                    //
                    // https://spec.graphql.org/September2025/#IsValidImplementation()
                    self.errors.push(TypeValidationError::new(
                        TypeValidationErrorKind::MissingInterfaceSpecifiedField {
                            field_name: field_name.to_string(),
                            interface_name: iface_name.to_string(),
                            type_name: type_name.to_string(),
                        },
                        type_span,
                        vec![ErrorNote::spec(
                            "https://spec.graphql.org/September2025/#IsValidImplementation()",
                        )],
                    ));
                    continue;
                };

                let iface_field_params = iface_field.parameters();
                let type_field_params = type_field.parameters();

                // For each parameter defined on this field in
                // the interface, there must be a corresponding
                // and equivalently-typed parameter on the
                // implementing type.
                //
                // https://spec.graphql.org/September2025/#IsValidImplementation()
                for (param_name, iface_field_param) in iface_field_params {
                    let Some(type_param) =
                        type_field_params.get(param_name)
                    else {
                        // https://spec.graphql.org/September2025/#IsValidImplementation()
                        self.errors.push(TypeValidationError::new(
                            TypeValidationErrorKind::MissingInterfaceSpecifiedFieldParameter {
                                field_name: field_name.to_string(),
                                interface_name: iface_name.to_string(),
                                missing_parameter_name:
                                    param_name.to_string(),
                                type_name: type_name.to_string(),
                            },
                            type_field.span(),
                            vec![ErrorNote::spec(
                                "https://spec.graphql.org/September2025/#IsValidImplementation()",
                            )],
                        ));
                        continue;
                    };

                    let iface_param_type =
                        iface_field_param.type_annotation();
                    let type_param_type =
                        type_param.type_annotation();
                    if !type_param_type
                        .is_equivalent_to(iface_param_type)
                    {
                        // https://spec.graphql.org/September2025/#IsValidImplementation()
                        self.errors.push(TypeValidationError::new(
                            TypeValidationErrorKind::InvalidInterfaceSpecifiedFieldParameterType {
                                actual_type:
                                    type_param_type.to_string(),
                                expected_type:
                                    iface_param_type.to_string(),
                                field_name: field_name.to_string(),
                                interface_name:
                                    iface_name.to_string(),
                                parameter_name:
                                    param_name.to_string(),
                                type_name: type_name.to_string(),
                            },
                            type_param.span(),
                            vec![
                                ErrorNote::general_with_span(
                                    format!(
                                        "interface defines this \
                                        parameter as `{iface_param_type}`",
                                    ),
                                    iface_field_param.span(),
                                ),
                                ErrorNote::spec(
                                    "https://spec.graphql.org/September2025/#IsValidImplementation()",
                                ),
                            ],
                        ));
                    }
                }

                // Any parameters defined on the implementing
                // field which aren't also defined on the
                // interface's corresponding field must be
                // optional (either nullable or defined with a
                // default value).
                //
                // See step 2.d at
                // https://spec.graphql.org/September2025/#IsValidImplementation()
                let iface_field_param_names: HashSet<_> =
                    iface_field_params.keys().collect();
                let type_field_param_names: HashSet<_> =
                    type_field_params.keys().collect();
                let additional_field_param_names =
                    type_field_param_names
                        .difference(&iface_field_param_names);

                for additional_param_name in additional_field_param_names {
                    let additional_param = type_field_params
                        .get(*additional_param_name)
                        .unwrap();
                    let additional_param_annot =
                        additional_param.type_annotation();

                    let is_nullable = additional_param_annot.nullable();
                    let has_default =
                        additional_param.default_value().is_some();
                    if !is_nullable && !has_default {
                        // https://spec.graphql.org/September2025/#IsValidImplementation()
                        self.errors.push(TypeValidationError::new(
                            TypeValidationErrorKind::InvalidRequiredAdditionalParameterOnInterfaceSpecifiedField {
                                field_name: field_name.to_string(),
                                interface_name:
                                    iface_name.to_string(),
                                parameter_name:
                                    additional_param_name.to_string(),
                                type_name: type_name.to_string(),
                            },
                            additional_param.span(),
                            vec![
                                ErrorNote::general_with_span(
                                    "field definition on implemented \
                                    interface",
                                    iface_field.span(),
                                ),
                                ErrorNote::spec(
                                    "https://spec.graphql.org/September2025/#IsValidImplementation()",
                                ),
                            ],
                        ));
                    }
                }

                // Field return types must be covariant subtypes.
                //
                // https://spec.graphql.org/September2025/#IsValidImplementation()
                let type_field_annot = type_field.type_annotation();
                let iface_field_annot =
                    iface_field.type_annotation();
                if !type_field_annot.is_subtype_of(
                    self.types_map,
                    iface_field_annot,
                ) {
                    // https://spec.graphql.org/September2025/#IsValidImplementation()
                    self.errors.push(TypeValidationError::new(
                        TypeValidationErrorKind::InvalidInterfaceSpecifiedFieldType {
                            actual_type:
                                type_field_annot.to_string(),
                            expected_type:
                                iface_field_annot.to_string(),
                            field_name: field_name.to_string(),
                            interface_name:
                                iface_name.to_string(),
                            type_name: type_name.to_string(),
                        },
                        type_field.span(),
                        vec![
                            ErrorNote::general_with_span(
                                format!(
                                    "interface field has return \
                                    type `{iface_field_annot}`",
                                ),
                                iface_field.span(),
                            ),
                            ErrorNote::spec(
                                "https://spec.graphql.org/September2025/#IsValidImplementation()",
                            ),
                        ],
                    ));
                }

                // TODO: IsValidImplementation step 2.f -- if the interface field
                // is NOT deprecated, the implementing field must also NOT be
                // deprecated. This check is deferred until DeprecationState is
                // queryable from FieldDefinition.
                // https://spec.graphql.org/September2025/#IsValidImplementation()
            }
        }

        // Validate that all fields use output types and all
        // parameters use input types.
        for (field_name, field) in type_fields {
            let innermost_type_name =
                field.type_annotation().innermost_type_name();
            let innermost_type =
                self.types_map.get(innermost_type_name);

            if let Some(innermost_type) = innermost_type {
                // All fields on an object/interface type must be
                // declared with an output type.
                //
                // https://spec.graphql.org/September2025/#sel-JAHZhCFDBFABLBgB_pM
                if !innermost_type.is_output_type() {
                    self.errors.push(TypeValidationError::new(
                        TypeValidationErrorKind::InvalidOutputFieldWithInputType {
                            field_name: field_name.to_string(),
                            input_type_name:
                                innermost_type_name.to_string(),
                            parent_type_name:
                                type_name.to_string(),
                        },
                        field.type_annotation().span(),
                        vec![ErrorNote::spec(
                            "https://spec.graphql.org/September2025/#sel-JAHZhCFDBFABLBgB_pM",
                        )],
                    ));
                }
            } else {
                // https://spec.graphql.org/September2025/#sec-Objects
                self.errors.push(TypeValidationError::new(
                    TypeValidationErrorKind::UndefinedTypeName {
                        undefined_type_name:
                            innermost_type_name.to_string(),
                    },
                    field.type_annotation().span(),
                    vec![],
                ));
            }

            for (param_name, param) in field.parameters() {
                let innermost_type_name =
                    param.type_annotation().innermost_type_name();
                let innermost_type =
                    self.types_map.get(innermost_type_name);

                if let Some(innermost_type) = innermost_type {
                    // All parameters must be declared with an
                    // input type.
                    //
                    // https://spec.graphql.org/September2025/#sel-KAHZhCFDBHBDCAACEB6yD
                    if !innermost_type.is_input_type() {
                        self.errors.push(TypeValidationError::new(
                            TypeValidationErrorKind::InvalidParameterWithOutputOnlyType {
                                field_name:
                                    field_name.to_string(),
                                invalid_type_name:
                                    innermost_type_name.to_string(),
                                parameter_name:
                                    param_name.to_string(),
                                type_name:
                                    type_name.to_string(),
                            },
                            param.type_annotation().span(),
                            vec![ErrorNote::spec(
                                "https://spec.graphql.org/September2025/#sel-KAHZhCFDBHBDCAACEB6yD",
                            )],
                        ));
                    }
                } else {
                    // https://spec.graphql.org/September2025/#sec-Objects
                    self.errors.push(TypeValidationError::new(
                        TypeValidationErrorKind::UndefinedTypeName {
                            undefined_type_name:
                                innermost_type_name.to_string(),
                        },
                        param.type_annotation().span(),
                        vec![],
                    ));
                }
            }
        }

        self.errors
    }

    /// Walks `iface`'s own transitive interface chain and
    /// verifies that `self.type_` declares every interface in
    /// that chain.
    ///
    /// `chain_from_implementing_type` is the path from
    /// `self.type_`'s directly-declared interface down to
    /// `iface`'s parent (exclusive of `iface` itself). For a
    /// top-level call -- i.e. when `iface` is an interface that
    /// `self.type_` directly implements -- this is empty.
    ///
    /// All errors emitted here are scoped to `self.type_.name()`
    /// as the implementing type; `iface` and its transitive
    /// ancestors are NOT validated here -- they get validated
    /// independently when `SchemaBuilder::build()` calls the
    /// validator for each type in the schema.
    fn check_interface_chain(
        &mut self,
        iface: &'a InterfaceType,
        chain_from_implementing_type: &[&'a TypeName],
        verified_interface_impls: &mut HashSet<&'a TypeName>,
    ) {
        let iface_name = iface.name();

        // For each interface that `iface` itself implements,
        // check whether `self.type_` also declares it. If not,
        // emit a MissingRecursiveInterfaceImplementation error
        // scoped to `self.type_`.
        for located_sub_iface in iface.interfaces() {
            let sub_iface_name = &located_sub_iface.value;
            if !self.implemented_iface_names.contains(sub_iface_name) {
                // Build an inheritance path that leads from
                // `self.type_`'s directly-declared interface all
                // the way down to `iface` (which is the interface
                // that transitively requires `sub_iface_name`).
                let mut inheritance_path: Vec<String> =
                    chain_from_implementing_type
                        .iter()
                        .map(|n| n.to_string())
                        .collect();
                inheritance_path.push(iface_name.to_string());

                // https://spec.graphql.org/September2025/#IsValidImplementation()
                self.errors.push(TypeValidationError::new(
                    TypeValidationErrorKind::MissingRecursiveInterfaceImplementation {
                        inheritance_path,
                        missing_recursive_interface_name:
                            sub_iface_name.to_string(),
                        type_name: self.type_.name().to_string(),
                    },
                    self.type_.span(),
                    vec![ErrorNote::spec(
                        "https://spec.graphql.org/September2025/#IsValidImplementation()",
                    )],
                ));
            }
        }

        // Recursively walk each of `iface`'s own interfaces.
        // `verified_interface_impls` prevents infinite recursion
        // if the schema contains a (malformed) cycle and also
        // avoids duplicating errors when multiple paths lead to
        // the same interface.
        for located_sub_iface in iface.interfaces() {
            let sub_iface_name = &located_sub_iface.value;
            if !verified_interface_impls.insert(sub_iface_name) {
                continue;
            }
            let Some(sub_iface_type) =
                self.types_map.get(sub_iface_name) else {
                continue;
            };
            let Some(sub_iface) = sub_iface_type.as_interface() else {
                continue;
            };
            let mut new_chain: Vec<&'a TypeName> =
                chain_from_implementing_type.to_vec();
            new_chain.push(iface_name);
            self.check_interface_chain(
                sub_iface,
                &new_chain,
                verified_interface_impls,
            );
        }
    }
}
