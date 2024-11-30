use crate::loc;
use crate::schema_builder::SchemaBuilder;
use crate::types::Directive;
use crate::types::GraphQLType;
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents a fully built, typechecked, and immutable GraphQL schema.
#[derive(Debug)]
pub struct Schema {
    pub(crate) directives: HashMap<String, Directive>,
    pub(crate) query_type: String,
    pub(crate) mutation_type: String,
    pub(crate) subscription_type: String,
    pub(crate) types: HashMap<String, GraphQLType>,
}
impl Schema {
    pub fn builder() -> SchemaBuilder {
        SchemaBuilder::new()
    }
}

/// Represents the file location of a given type's definition in the schema.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct TypeDefFileLocation {
    pub location: loc::FilePosition,
    pub type_name: String,
}
impl TypeDefFileLocation {
    pub(crate) fn from_pos(
        type_name: String,
        file: PathBuf,
        pos: graphql_parser::Pos,
    ) -> Self {
        Self {
            location: loc::FilePosition::from_pos(file, pos),
            type_name,
        }
    }
}
