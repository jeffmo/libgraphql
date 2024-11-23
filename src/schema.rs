use crate::ast;
use crate::schema_builder::SchemaBuilder;
use crate::types::Directive;
use crate::types::SchemaType;
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents a fully built, typechecked, and immutable GraphQL schema.
#[derive(Debug)]
pub struct Schema {
    pub directives: HashMap<String, Directive>,
    pub query_type: String,
    pub mutation_type: String,
    pub subscription_type: String,
    pub types: HashMap<String, SchemaType>,
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
    pub location: ast::FileLocation,
    pub type_name: String,
}
impl TypeDefFileLocation {
    pub(crate) fn from_pos(
        type_name: String,
        file: PathBuf,
        pos: graphql_parser::Pos,
    ) -> Self {
        Self {
            location: ast::FileLocation::from_pos(file, pos),
            type_name,
        }
    }
}
