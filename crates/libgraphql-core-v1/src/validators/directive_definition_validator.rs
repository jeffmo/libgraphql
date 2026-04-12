use crate::error_note::ErrorNote;
use crate::names::DirectiveName;
use crate::names::TypeName;
use crate::schema::TypeValidationError;
use crate::schema::TypeValidationErrorKind;
use crate::types::DirectiveDefinition;
use crate::types::GraphQLType;
use crate::validators::edit_distance::find_similar_names;
use indexmap::IndexMap;

/// Validates custom directive definitions.
///
/// Checks that every parameter on a custom (non-builtin) directive
/// definition references a valid input type. Built-in directives
/// are skipped since they are validated by the spec itself.
///
/// See [Type System Directives](https://spec.graphql.org/September2025/#sec-Type-System.Directives).
pub(crate) fn validate_directive_definitions(
    directive_defs: &IndexMap<DirectiveName, DirectiveDefinition>,
    types_map: &IndexMap<TypeName, GraphQLType>,
) -> Vec<TypeValidationError> {
    let mut errors = Vec::new();

    for directive_def in directive_defs.values() {
        // Only validate custom directives; built-in directives
        // are spec-defined and assumed correct.
        if directive_def.is_builtin() {
            continue;
        }

        for (param_name, param) in directive_def.parameters() {
            let innermost_type_name =
                param.type_annotation().innermost_type_name();
            let innermost_type =
                types_map.get(innermost_type_name);

            if let Some(innermost_type) = innermost_type {
                // All directive parameters must be declared with
                // an input type.
                //
                // https://spec.graphql.org/September2025/#sec-Type-System.Directives
                if !innermost_type.is_input_type() {
                    errors.push(TypeValidationError::new(
                        TypeValidationErrorKind::InvalidDirectiveParameterType {
                            directive_name:
                                directive_def.name().to_string(),
                            invalid_type_name:
                                innermost_type_name.to_string(),
                            parameter_name:
                                param_name.to_string(),
                        },
                        param.type_annotation().span(),
                        vec![ErrorNote::spec(
                            "https://spec.graphql.org/September2025/#sec-Type-System.Directives",
                        )],
                    ));
                }
            } else {
                // https://spec.graphql.org/September2025/#sec-Type-System.Directives
                let mut notes = Vec::new();
                let suggestions = find_similar_names(
                    innermost_type_name.as_str(),
                    types_map.keys(),
                );
                if let Some(best) = suggestions.first() {
                    notes.push(ErrorNote::help(
                        format!("did you mean `{best}`?"),
                    ));
                }
                notes.push(ErrorNote::spec(
                    "https://spec.graphql.org/September2025/#sec-Types",
                ));
                errors.push(TypeValidationError::new(
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

    errors
}
