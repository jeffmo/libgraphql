use crate::loc;
use crate::types::GraphQLTypeRef;
use crate::types::Parameter;
use std::collections::BTreeMap;

/// Represents a defined field on a [GraphQLObjectType] or
/// [GraphQLInterfaceType].
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub(super) def_location: loc::SchemaDefLocation,
    pub(super) params: BTreeMap<String, Parameter>,
    pub(super) type_ref: GraphQLTypeRef,
}

impl Field {
    // TODO: Encode this into a commonly-used trait (to ensure it's consistent
    //       across all types)
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        &self.def_location
    }

    pub fn parameters(&self) -> &BTreeMap<String, Parameter> {
        &self.params
    }

    pub fn type_ref(&self) -> &GraphQLTypeRef {
        &self.type_ref
    }
}
