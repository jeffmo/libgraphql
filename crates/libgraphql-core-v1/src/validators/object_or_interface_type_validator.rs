use crate::error_note::ErrorNote;
use crate::names::TypeName;
use crate::schema::TypeValidationError;
use crate::schema::TypeValidationErrorKind;
use crate::types::GraphQLType;
use crate::types::HasFieldsAndInterfaces;
use crate::types::InterfaceType;
use crate::validators::edit_distance::find_similar_names;
use indexmap::IndexMap;
use std::collections::HashSet;

/// Validates an object or interface type's interface
/// implementations, field output-type legality, and parameter
/// input-type legality.
///
/// Implements the
/// [IsValidImplementation](https://spec.graphql.org/September2025/#IsValidImplementation())
/// algorithm from the GraphQL specification.
///
/// # Validation phases
///
/// The validator runs three distinct phases in order:
///
/// 1. **Transitive interface completeness** — For each
///    directly-declared interface `I`, verifies that `I` exists
///    and is an interface type, then walks `I`'s own transitive
///    interface chain to ensure the implementing type also
///    declares every transitively-required interface.
///
/// 2. **Field contract validation** — For each directly-declared
///    interface `I`, validates the field contract: every
///    interface field must exist on the implementing type with
///    matching parameters (equivalence) and a covariant return
///    type. Additional parameters must be optional. Uses a
///    separate dedup set so that an interface's field contract
///    is checked exactly once even if multiple declared
///    interfaces share a transitive ancestor.
///
/// 3. **Field type/param checks** — For ALL fields on the
///    implementing type (including non-interface fields),
///    validates that return types are output types and parameter
///    types are input types.
///
/// Each phase uses its own local state, avoiding the
/// shared-state bug where phase 1's transitive walk could
/// prevent phase 2 from validating directly-declared
/// interfaces.
pub(crate) struct ObjectOrInterfaceTypeValidator<'a, T: HasFieldsAndInterfaces> {
    errors: Vec<TypeValidationError>,
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
            type_,
            types_map,
        }
    }

    pub fn validate(mut self) -> Vec<TypeValidationError> {
        self.check_interface_completeness();
        self.check_field_contracts();
        self.check_field_types();
        self.errors
    }

    /// Phase 1: Transitive interface completeness.
    ///
    /// For each interface the type directly declares in
    /// `implements`, verifies:
    ///   - The interface name resolves to a defined type
    ///   - That type is actually an interface
    ///   - Every transitively-required interface (from the
    ///     interface's own chain) is also directly declared by
    ///     the implementing type
    ///
    /// Uses a LOCAL visited set for the transitive walk, fully
    /// independent from the field-contract phase.
    ///
    /// https://spec.graphql.org/September2025/#IsValidImplementation()
    fn check_interface_completeness(&mut self) {
        let type_name = self.type_.name();
        let implemented_iface_names: HashSet<&TypeName> = self
            .type_
            .interfaces()
            .iter()
            .map(|l| &l.value)
            .collect();

        for located_iface in self.type_.interfaces() {
            let iface_name = &located_iface.value;

            // Verify that this implemented interface name is
            // actually a defined type.
            let Some(iface_type) = self.types_map.get(iface_name) else {
                let mut notes = Vec::new();
                let max_dist =
                    iface_name.as_str().len() / 3 + 1;
                let suggestions = find_similar_names(
                    iface_name.as_str(),
                    self.types_map.keys(),
                    max_dist,
                );
                if let Some(best) = suggestions.first() {
                    notes.push(ErrorNote::help(
                        format!("did you mean `{best}`?"),
                    ));
                }
                notes.push(ErrorNote::spec(
                    "https://spec.graphql.org/September2025/#IsValidImplementation()",
                ));
                self.errors.push(TypeValidationError::new(
                    TypeValidationErrorKind::ImplementsUndefinedInterface {
                        type_name: type_name.to_string(),
                        undefined_interface_name: iface_name.to_string(),
                    },
                    located_iface.span,
                    notes,
                ));
                continue;
            };

            // Verify that the defined type being implemented is
            // an interface type.
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

            // Walk the interface's own transitive chain and
            // collect all transitively-required interfaces.
            let mut transitive = HashSet::new();
            self.collect_transitive_interfaces(iface, &mut transitive);

            // For each transitively-required interface, check
            // that the implementing type also directly declares
            // it.
            for required_iface_name in &transitive {
                if !implemented_iface_names.contains(*required_iface_name) {
                    // Build an inheritance path from the
                    // directly-declared interface down to the
                    // interface that transitively requires
                    // `required_iface_name`.
                    let inheritance_path = self.build_inheritance_path(
                        iface,
                        required_iface_name,
                    );

                    self.errors.push(TypeValidationError::new(
                        TypeValidationErrorKind::MissingRecursiveInterfaceImplementation {
                            inheritance_path,
                            missing_recursive_interface_name:
                                required_iface_name.to_string(),
                            type_name: type_name.to_string(),
                        },
                        located_iface.span,
                        vec![ErrorNote::spec(
                            "https://spec.graphql.org/September2025/#IsValidImplementation()",
                        )],
                    ));
                }
            }
        }
    }

    /// Phase 2: Field contract validation.
    ///
    /// For each directly-declared interface, validates that the
    /// implementing type satisfies the interface's field contract
    /// per IsValidImplementation():
    ///   - Every interface field must exist on the type
    ///   - Parameter equivalence (same params, same types)
    ///   - Additional params must be optional
    ///   - Return type must be a covariant subtype
    ///
    /// Uses a separate `field_validated_interfaces` set to avoid
    /// checking the same interface's fields twice (e.g. when
    /// multiple declared interfaces share a common ancestor).
    ///
    /// https://spec.graphql.org/September2025/#IsValidImplementation()
    fn check_field_contracts(&mut self) {
        let type_name = self.type_.name();
        let type_fields = self.type_.fields();
        let type_span = self.type_.span();
        let mut field_validated_interfaces: HashSet<&TypeName> = HashSet::new();

        for located_iface in self.type_.interfaces() {
            let iface_name = &located_iface.value;

            // Skip if we can't resolve to a valid interface (phase 1
            // already reported these errors).
            let Some(iface_type) = self.types_map.get(iface_name) else {
                continue;
            };
            let Some(iface) = iface_type.as_interface() else {
                continue;
            };

            // Dedup: skip if we already validated this
            // interface's field contract.
            if !field_validated_interfaces.insert(iface_name) {
                continue;
            }

            let iface_fields = iface.fields();
            for (field_name, iface_field) in iface_fields {
                let Some(type_field) = type_fields.get(field_name) else {
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
                    let Some(type_param) = type_field_params.get(param_name) else {
                        self.errors.push(TypeValidationError::new(
                            TypeValidationErrorKind::MissingInterfaceSpecifiedFieldParameter {
                                field_name: field_name.to_string(),
                                interface_name: iface_name.to_string(),
                                missing_parameter_name: param_name.to_string(),
                                type_name: type_name.to_string(),
                            },
                            type_field.span(),
                            vec![ErrorNote::spec(
                                "https://spec.graphql.org/September2025/#IsValidImplementation()",
                            )],
                        ));
                        continue;
                    };

                    let iface_param_type = iface_field_param.type_annotation();
                    let type_param_type = type_param.type_annotation();
                    if !type_param_type.is_equivalent_to(iface_param_type) {
                        self.errors.push(TypeValidationError::new(
                            TypeValidationErrorKind::InvalidInterfaceSpecifiedFieldParameterType {
                                actual_type: type_param_type.to_string(),
                                expected_type: iface_param_type.to_string(),
                                field_name: field_name.to_string(),
                                interface_name: iface_name.to_string(),
                                parameter_name: param_name.to_string(),
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
                    type_field_param_names.difference(&iface_field_param_names);

                for additional_param_name in additional_field_param_names {
                    let additional_param = type_field_params
                        .get(*additional_param_name)
                        .unwrap();
                    let additional_param_annot = additional_param.type_annotation();

                    let is_nullable = additional_param_annot.nullable();
                    let has_default = additional_param.default_value().is_some();
                    if !is_nullable && !has_default {
                        self.errors.push(TypeValidationError::new(
                            TypeValidationErrorKind::InvalidRequiredAdditionalParameterOnInterfaceSpecifiedField {
                                field_name: field_name.to_string(),
                                interface_name: iface_name.to_string(),
                                parameter_name: additional_param_name.to_string(),
                                type_name: type_name.to_string(),
                            },
                            additional_param.span(),
                            vec![
                                ErrorNote::general_with_span(
                                    "field definition on implemented interface",
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
                let iface_field_annot = iface_field.type_annotation();
                if !type_field_annot.is_subtype_of(self.types_map, iface_field_annot) {
                    self.errors.push(TypeValidationError::new(
                        TypeValidationErrorKind::InvalidInterfaceSpecifiedFieldType {
                            actual_type: type_field_annot.to_string(),
                            expected_type: iface_field_annot.to_string(),
                            field_name: field_name.to_string(),
                            interface_name: iface_name.to_string(),
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
    }

    /// Phase 3: Field type/param checks for ALL fields.
    ///
    /// Independent of interface validation — validates that every
    /// field on the implementing type (including non-interface
    /// fields) uses valid output types for return values and
    /// valid input types for parameters.
    ///
    /// https://spec.graphql.org/September2025/#sel-JAHZhCFDBFABLBgB_pM
    /// https://spec.graphql.org/September2025/#sel-KAHZhCFDBHBDCAACEB6yD
    fn check_field_types(&mut self) {
        let type_name = self.type_.name();
        let type_fields = self.type_.fields();

        for (field_name, field) in type_fields {
            let innermost_type_name = field.type_annotation().innermost_type_name();
            let innermost_type = self.types_map.get(innermost_type_name);

            if let Some(innermost_type) = innermost_type {
                // All fields on an object/interface type must be
                // declared with an output type.
                //
                // https://spec.graphql.org/September2025/#sel-JAHZhCFDBFABLBgB_pM
                if !innermost_type.is_output_type() {
                    self.errors.push(TypeValidationError::new(
                        TypeValidationErrorKind::InvalidOutputFieldWithInputType {
                            field_name: field_name.to_string(),
                            input_type_name: innermost_type_name.to_string(),
                            parent_type_name: type_name.to_string(),
                        },
                        field.type_annotation().span(),
                        vec![ErrorNote::spec(
                            "https://spec.graphql.org/September2025/#sel-JAHZhCFDBFABLBgB_pM",
                        )],
                    ));
                }
            } else {
                // https://spec.graphql.org/September2025/#sec-Objects
                let mut notes = Vec::new();
                let max_dist =
                    innermost_type_name.as_str().len() / 3 + 1;
                let suggestions = find_similar_names(
                    innermost_type_name.as_str(),
                    self.types_map.keys(),
                    max_dist,
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
            }

            for (param_name, param) in field.parameters() {
                let innermost_type_name =
                    param.type_annotation().innermost_type_name();
                let innermost_type = self.types_map.get(innermost_type_name);

                if let Some(innermost_type) = innermost_type {
                    // All parameters must be declared with an
                    // input type.
                    //
                    // https://spec.graphql.org/September2025/#sel-KAHZhCFDBHBDCAACEB6yD
                    if !innermost_type.is_input_type() {
                        self.errors.push(TypeValidationError::new(
                            TypeValidationErrorKind::InvalidParameterWithOutputOnlyType {
                                field_name: field_name.to_string(),
                                invalid_type_name:
                                    innermost_type_name.to_string(),
                                parameter_name: param_name.to_string(),
                                type_name: type_name.to_string(),
                            },
                            param.type_annotation().span(),
                            vec![ErrorNote::spec(
                                "https://spec.graphql.org/September2025/#sel-KAHZhCFDBHBDCAACEB6yD",
                            )],
                        ));
                    }
                } else {
                    // https://spec.graphql.org/September2025/#sec-Objects
                    let mut notes = Vec::new();
                    let max_dist =
                        innermost_type_name.as_str().len() / 3 + 1;
                    let suggestions = find_similar_names(
                        innermost_type_name.as_str(),
                        self.types_map.keys(),
                        max_dist,
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
                        param.type_annotation().span(),
                        notes,
                    ));
                }
            }
        }
    }

    /// Collects all interfaces transitively required by `iface`
    /// into `result`.
    ///
    /// Walks `iface`'s own `interfaces()` list recursively,
    /// accumulating every interface name encountered. Uses
    /// `result` itself as a visited set to avoid infinite loops
    /// in the presence of malformed cyclic schemas.
    fn collect_transitive_interfaces(
        &self,
        iface: &'a InterfaceType,
        result: &mut HashSet<&'a TypeName>,
    ) {
        for located_sub_iface in iface.interfaces() {
            let sub_iface_name = &located_sub_iface.value;

            // If we've already seen this interface, skip to
            // prevent infinite recursion on cyclic schemas.
            if !result.insert(sub_iface_name) {
                continue;
            }

            // Recurse into the sub-interface's own chain.
            let Some(sub_iface_type) = self.types_map.get(sub_iface_name) else {
                continue;
            };
            let Some(sub_iface) = sub_iface_type.as_interface() else {
                continue;
            };
            self.collect_transitive_interfaces(sub_iface, result);
        }
    }

    /// Builds the inheritance path from a directly-declared
    /// interface down to the interface that transitively requires
    /// `target_name`.
    ///
    /// Returns a vec of interface names representing the path,
    /// e.g. for `Node -> Entity -> Root` where `Root` is the
    /// target, returns `["Node", "Entity"]`.
    fn build_inheritance_path(
        &self,
        start_iface: &'a InterfaceType,
        target_name: &TypeName,
    ) -> Vec<String> {
        let mut path = vec![start_iface.name().to_string()];

        // If the start interface directly declares the target,
        // the path is just [start_name].
        let directly_declares = start_iface
            .interfaces()
            .iter()
            .any(|l| &l.value == target_name);
        if directly_declares {
            return path;
        }

        // Otherwise, do a DFS to find the path to the
        // interface that directly declares `target_name`.
        let mut visited = HashSet::new();
        visited.insert(start_iface.name());
        if self.find_path_to_target(start_iface, target_name, &mut path, &mut visited) {
            return path;
        }

        // Fallback: shouldn't happen if collect_transitive_interfaces
        // found target_name, but return what we have.
        path
    }

    /// DFS helper for `build_inheritance_path`. Returns true if
    /// a path to an interface that directly declares
    /// `target_name` was found.
    fn find_path_to_target(
        &self,
        iface: &'a InterfaceType,
        target_name: &TypeName,
        path: &mut Vec<String>,
        visited: &mut HashSet<&'a TypeName>,
    ) -> bool {
        for located_sub_iface in iface.interfaces() {
            let sub_iface_name = &located_sub_iface.value;
            if !visited.insert(sub_iface_name) {
                continue;
            }

            let Some(sub_iface_type) = self.types_map.get(sub_iface_name) else {
                continue;
            };
            let Some(sub_iface) = sub_iface_type.as_interface() else {
                continue;
            };

            // Check if this sub-interface directly declares
            // target_name.
            let sub_declares_target = sub_iface
                .interfaces()
                .iter()
                .any(|l| &l.value == target_name);
            if sub_declares_target {
                path.push(sub_iface_name.to_string());
                return true;
            }

            // Recurse deeper.
            path.push(sub_iface_name.to_string());
            if self.find_path_to_target(sub_iface, target_name, path, visited) {
                return true;
            }
            path.pop();
        }
        false
    }
}
