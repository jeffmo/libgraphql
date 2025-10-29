use crate::schema::TypeValidationError;
use crate::types::GraphQLType;
use crate::types::UnionType;
use std::collections::HashMap;

pub(super) struct UnionTypeValidator<'a> {
    errors: Vec<TypeValidationError>,
    type_: &'a UnionType,
    types_map: &'a HashMap<String, GraphQLType>,
}
impl<'a> UnionTypeValidator<'a> {
    pub fn new(
        type_: &'a UnionType,
        types_map: &'a HashMap<String, GraphQLType>,
    ) -> Self {
        Self {
            errors: vec![],
            type_,
            types_map,
        }
    }

    pub fn validate(mut self) -> Vec<TypeValidationError> {
        for (member_type_name, member_type_ref) in &self.type_.members {
            // Member types of a union type can only be object types.
            // https://spec.graphql.org/October2021/#sel-HAHdfFDABABlG3ib
            let member_type = self.types_map.get(member_type_name);
            let member_type =
                if let Some(member_type) = member_type {
                    member_type
                } else {
                    self.errors.push(
                        TypeValidationError::UndefinedTypeName {
                            ref_location: self.type_.def_location().to_owned(),
                            undefined_type_name: member_type_name.to_string(),
                        }
                    );
                    continue;
                };
            if !matches!(member_type, GraphQLType::Object(_)) {
                self.errors.push(
                    TypeValidationError::InvalidUnionMemberTypeKind {
                        location: member_type_ref.ref_location().to_owned().into(),
                        union_type_name: self.type_.name().to_string(),
                        invalid_member_type: member_type.to_owned(),
                    }
                );
            }
        }

        self.errors
    }
}
