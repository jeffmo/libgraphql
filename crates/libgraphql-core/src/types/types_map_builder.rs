use crate::loc;
use crate::schema::SchemaBuildError;
use crate::types::GraphQLType;
use crate::types::InputObjectTypeValidator;
use crate::types::ObjectOrInterfaceTypeValidator;
use crate::types::UnionTypeValidator;
use std::collections::HashMap;
use std::collections::HashSet;

type Result<T> = std::result::Result<T, SchemaBuildError>;

#[derive(Debug)]
pub struct TypesMapBuilder {
    pub(super) types: HashMap<String, GraphQLType>,
}
impl TypesMapBuilder {
    pub fn new() -> Self {
        Self {
            types: HashMap::from([
                ("Boolean".to_string(), GraphQLType::Bool),
                ("Float".to_string(), GraphQLType::Float),
                ("ID".to_string(), GraphQLType::ID),
                ("Int".to_string(), GraphQLType::Int),
                ("String".to_string(), GraphQLType::String),
            ]),
        }
    }

    pub fn add_new_type(
        &mut self,
        type_name: &str,
        type_loc: &loc::SourceLocation,
        type_: GraphQLType,
    ) -> Result<()> {
        if type_name.starts_with("__") {
            return Err(SchemaBuildError::InvalidDunderPrefixedTypeName {
                def_location: type_loc.to_owned(),
                type_name: type_name.to_string(),
            });
        }

        if let Some(conflicting_type) = self.types.get(type_name) {
            return Err(SchemaBuildError::DuplicateTypeDefinition {
                type_name: type_name.to_string(),
                def1: conflicting_type.def_location().to_owned(),
                def2: type_loc.to_owned(),
            });
        }

        self.types.insert(type_name.to_string(), type_);
        Ok(())
    }

    pub fn into_types_map(self) -> Result<HashMap<String, GraphQLType>> {
        // Final validation of all types together.
        let mut errors = vec![];
        for type_ in self.types.values() {
            match type_ {
                GraphQLType::Bool
                | GraphQLType::Enum(_)
                | GraphQLType::Float
                | GraphQLType::ID
                | GraphQLType::Int
                | GraphQLType::Scalar(_)
                | GraphQLType::String
                    => (),

                GraphQLType::InputObject(type_) => errors.append(
                    &mut InputObjectTypeValidator::new(type_, &self.types)
                        .validate()
                ),

                GraphQLType::Interface(type_) => errors.append(
                    &mut ObjectOrInterfaceTypeValidator::new(&type_.0, &self.types)
                        .validate(&mut HashSet::new())
                ),

                GraphQLType::Object(type_) => errors.append(
                    &mut ObjectOrInterfaceTypeValidator::new(&type_.0, &self.types)
                        .validate(&mut HashSet::new())
                ),

                GraphQLType::Union(type_) => errors.append(
                    &mut UnionTypeValidator::new(type_, &self.types)
                        .validate()
                ),
            }
        }

        if !errors.is_empty() {
            return Err(SchemaBuildError::TypeValidationErrors { errors });
        }

        Ok(self.types)
    }

    pub fn get_type_mut(
        &mut self,
        type_name: &str,
    ) -> Option<&mut GraphQLType> {
        self.types.get_mut(type_name)
    }
}
