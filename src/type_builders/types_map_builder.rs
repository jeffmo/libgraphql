use crate::loc;
use crate::SchemaBuildError;
use crate::types::GraphQLType;
use std::collections::HashMap;

type Result<T> = std::result::Result<T, SchemaBuildError>;

#[derive(Debug)]
pub struct TypesMapBuilder {
    types: HashMap<String, GraphQLType>,
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
        file_position: loc::FilePosition,
        type_name: &str,
        type_: GraphQLType,
    ) -> Result<()> {
        if let Some(conflicting_type) = self.types.get(type_name) {
            return Err(SchemaBuildError::DuplicateTypeDefinition {
                type_name: type_name.to_string(),
                def1: conflicting_type.get_def_location().clone(),
                def2: loc::SchemaDefLocation::Schema(
                    file_position.clone(),
                ),
            });
        }

        self.types.insert(type_name.to_string(), type_);
        Ok(())
    }

    pub fn into_types_map(self) -> Result<HashMap<String, GraphQLType>> {
        // TODO: Implement type-checking here
        Ok(self.types)
    }

    pub fn get_type_mut(
        &mut self,
        type_name: &str,
    ) -> Option<&mut GraphQLType> {
        self.types.get_mut(type_name)
    }
}
