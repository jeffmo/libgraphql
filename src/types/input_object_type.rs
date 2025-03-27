use crate::loc;
use crate::types::DirectiveAnnotation;
use std::collections::BTreeMap;

/// Information associated with [GraphQLType::InputObject]
#[derive(Clone, Debug, PartialEq)]
pub struct InputObjectType {
    pub def_location: loc::FilePosition,
    pub directives: Vec<DirectiveAnnotation>,
    pub fields: BTreeMap<String, InputField>,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InputField {
    pub def_location: loc::SchemaDefLocation,
    // TODO: There's more to input fields...
}

