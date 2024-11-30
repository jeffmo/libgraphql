use crate::schema_builder::SchemaBuilder;
use crate::types::Directive;
use crate::types::GraphQLType;
use crate::types::NamedGraphQLTypeRef;
use std::collections::HashMap;

/// Represents a fully built, typechecked, and immutable GraphQL schema.
#[derive(Clone, Debug)]
pub struct Schema {
    pub(crate) directives: HashMap<String, Directive>,
    pub(crate) query_type: NamedGraphQLTypeRef,
    pub(crate) mutation_type: NamedGraphQLTypeRef,
    pub(crate) subscription_type: NamedGraphQLTypeRef,
    pub(crate) types: HashMap<String, GraphQLType>,
}
impl Schema {
    pub fn builder() -> SchemaBuilder {
        SchemaBuilder::new()
    }
}
