use crate::loc;
use crate::types::GraphQLTypeRef;

/// Represents a defined field on a [GraphQLObjectType] or
/// [GraphQLInterfaceType].
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub def_location: loc::SchemaDefLocation,
    pub type_ref: GraphQLTypeRef,
}

