use crate::error_note::ErrorNote;
use crate::names::TypeName;
use crate::schema::TypeValidationError;
use crate::schema::TypeValidationErrorKind;
use crate::types::GraphQLType;
use crate::types::UnionType;
use crate::validators::edit_distance::find_similar_names;
use indexmap::IndexMap;

/// Validates a union type's member references.
///
/// Checks that every member of a union type exists in the schema
/// and is an object type, per
/// [Union Members](https://spec.graphql.org/September2025/#sel-HAHdfFDABABlG3ib).
///
/// Note: the empty-union check (`EmptyUnionType`) is a build-level
/// error handled by `SchemaBuildErrorKind`; this validator only
/// covers member-exists and member-is-object checks.
pub(crate) struct UnionTypeValidator<'a> {
    errors: Vec<TypeValidationError>,
    type_: &'a UnionType,
    types_map: &'a IndexMap<TypeName, GraphQLType>,
}

impl<'a> UnionTypeValidator<'a> {
    pub fn new(
        type_: &'a UnionType,
        types_map: &'a IndexMap<TypeName, GraphQLType>,
    ) -> Self {
        Self {
            errors: vec![],
            type_,
            types_map,
        }
    }

    pub fn validate(mut self) -> Vec<TypeValidationError> {
        for member in self.type_.members() {
            let member_name = &member.value;

            // Member types of a union type must be defined.
            //
            // https://spec.graphql.org/September2025/#sel-HAHdfFDABABlG3ib
            let Some(member_type) = self.types_map.get(member_name)
            else {
                let mut notes = Vec::new();
                let max_dist =
                    member_name.as_str().len() / 3 + 1;
                let suggestions = find_similar_names(
                    member_name.as_str(),
                    self.types_map.keys(),
                    max_dist,
                );
                if let Some(best) = suggestions.first() {
                    notes.push(ErrorNote::help(
                        format!("did you mean `{best}`?"),
                    ));
                }
                notes.push(ErrorNote::spec(
                    "https://spec.graphql.org/September2025/#sel-HAHdfFDABABlG3ib",
                ));
                self.errors.push(TypeValidationError::new(
                    TypeValidationErrorKind::UndefinedTypeName {
                        undefined_type_name:
                            member_name.to_string(),
                    },
                    member.span,
                    notes,
                ));
                continue;
            };

            // Member types of a union type can only be object
            // types.
            //
            // https://spec.graphql.org/September2025/#sel-HAHdfFDABABlG3ib
            if !matches!(member_type, GraphQLType::Object(_)) {
                self.errors.push(TypeValidationError::new(
                    TypeValidationErrorKind::InvalidUnionMemberTypeKind {
                        member_name: member_name.to_string(),
                        union_type_name:
                            self.type_.name().to_string(),
                    },
                    member.span,
                    vec![
                        ErrorNote::general_with_span(
                            format!(
                                "`{member_name}` is defined here",
                            ),
                            member_type.span(),
                        ),
                        ErrorNote::spec(
                            "https://spec.graphql.org/September2025/#sel-HAHdfFDABABlG3ib",
                        ),
                    ],
                ));
            }
        }

        self.errors
    }
}
