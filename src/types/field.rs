use crate::loc;
use crate::Schema;
use crate::types::FieldType;
use crate::types::GraphQLType;
use crate::types::GraphQLTypeRef;

/// Represents a defined field on a [GraphQLObjectType] or
/// [GraphQLInterfaceType].
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub(super) def_location: loc::SchemaDefLocation,
    pub(super) type_ref: GraphQLTypeRef,
}

impl Field {
    // TODO: Encode this into a commonly-used trait (to ensure it's consistent
    //       across all types)
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        &self.def_location
    }

    pub fn type_ref(&self) -> &GraphQLTypeRef {
        &self.type_ref
    }
}
