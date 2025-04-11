use crate::SchemaBuilder;
use crate::types::Directive;
use crate::types::GraphQLType;
use crate::types::NamedGraphQLTypeRef;
use std::collections::HashMap;

/// Represents a fully built, typechecked, and immutable GraphQL schema.
#[derive(Clone, Debug, PartialEq)]
pub struct Schema {
    pub(super) directive_defs: HashMap<String, Directive>,
    pub(super) query_type: NamedGraphQLTypeRef,
    pub(super) mutation_type: Option<NamedGraphQLTypeRef>,
    pub(super) subscription_type: Option<NamedGraphQLTypeRef>,
    pub(super) types: HashMap<String, GraphQLType>,
}
impl Schema {
    pub fn builder() -> SchemaBuilder {
        SchemaBuilder::new()
    }

    pub fn mutation_type(&self) -> Option<&GraphQLType> {
        self.mutation_type.as_ref().map(
            |named_ref| named_ref.deref(self).unwrap()
        )
    }

    pub fn query_type(&self) -> &GraphQLType {
        self.query_type.deref(self).unwrap()
    }

    pub fn subscription_type(&self) -> Option<&GraphQLType> {
        self.subscription_type.as_ref().map(
            |named_ref| named_ref.deref(self).unwrap()
        )
    }
}
